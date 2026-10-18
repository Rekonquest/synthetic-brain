//! T6 runner, THERMO-1 (docs/T6-THERMO-1.md): T5's hosted quantum and classical brains in a base
//! with a real temperature field θ(site). Every part takes its temperature from the base at its own
//! site (identity couplings): the medium's bath noise on every species, M's heat (quantum occupation
//! heat / classical register floor) = θ at B, and T's Boltzmann decision at θ at A. Swept cold → hot,
//! uniform against local (two port regions), send gain 1 against 1000, and a frozen-chaos round on
//! the temperature field. Copied from T5's runner; every setting lives in `Cfg` and `Medium6`.

use qb_amplitude::{State, C};
use qb_chaos::keyed_unit;
use qb_classical::{argmax, conserve, decay, draw, gate, temper};
use qb_task::{qb7_spec, score, split_acc, world, Spec, World};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use thermo_bath::{damp, exchange};
use thermo_host::Medium;
use thermo_lattice::Lattice;

/// The planned bath (`thermo-bath-fast`, bit-identical to `thermo-bath`), built once for the medium.
fn plan() -> &'static thermo_bath_fast::Plan {
    static PLAN: OnceLock<thermo_bath_fast::Plan> = OnceLock::new();
    PLAN.get_or_init(|| thermo_bath_fast::Plan::new(&Lattice::new(&[MED.side; 3])))
}

const ETA: f64 = 0.15;
const DETECTOR_K: usize = 3;
const DETECTOR_M: usize = 4;
const GRID: [f64; 6] = [0.0, 0.01, 0.02, 0.05, 0.1, 0.2];
/// Thin policy (labelled in the doc): below this the decision is the greedy limit (argmax).
const GREEDY_BELOW: f64 = 0.01;

/// T5's remote medium at r = 2.
#[derive(Debug, Clone, Copy)]
struct Medium6 {
    side: usize,
    r: usize,
    kappa: f64,
    substeps: usize,
    leak: f64,
}

const MED: Medium6 = Medium6 { side: 16, r: 2, kappa: 0.5, substeps: 4, leak: 0.1 };

#[derive(Debug, Clone, Copy, PartialEq)]
enum Brain {
    Quantum,
    Classical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cfg {
    brain: Brain,
    theta_a: f64,
    theta_b: f64,
    gain: f64,
    chaos: f64,
}

/// QB-9's centred chaos for (seed, step, element, tag).
fn chaos(seed: u64, t: usize, element: usize, tag: u64) -> f64 {
    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (element as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ tag.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    keyed_unit(key) - 0.5
}

/// A unit-variance kick, uniform on [−√3, √3], from a SplitMix64 hash (labelled in the doc).
fn kick(key: u64) -> f64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0) * 3f64.sqrt()
}

/// The temperature field: the nearer port along x (minimum image, ties to A) sets the region; frozen
/// chaos multiplies each site by 1 + α(x − ½).
fn theta_field(c: &Cfg) -> Vec<f64> {
    let l = MED.side;
    let (xa, xb) = (l / 2, l / 2 + MED.r);
    let dist = |x: usize, p: usize| {
        let d = (x as i64 - p as i64).rem_euclid(l as i64) as usize;
        d.min(l - d)
    };
    (0..l * l * l)
        .map(|s| {
            let x = s % l;
            let base = if dist(x, xa) <= dist(x, xb) { c.theta_a } else { c.theta_b };
            if c.chaos > 0.0 { base * (1.0 + c.chaos * (keyed_unit(0x7E57_0006 ^ (s as u64).wrapping_mul(0x9E37_79B9)) - 0.5)) } else { base }
        })
        .collect()
}

struct Hosted {
    medium: Medium,
    a: usize,
    b: usize,
    n: usize,
    gain: f64,
    g_e: f64,
    hot: bool,
    site_sd: Vec<f64>,
    edge_sd: Vec<f64>,
}

impl Hosted {
    fn new(theta: &[f64], gain: f64, n: usize) -> Hosted {
        let l = MED.side;
        let c = l / 2;
        let site = |x: usize, y: usize, z: usize| x + l * y + l * l * z;
        let probe = Lattice::new(&[l; 3]);
        let edge_sd: Vec<f64> = (0..l * l * l * 3).map(|k| ((theta[k / 3] + theta[probe.forward(k / 3, k % 3)]) / 2.0).sqrt()).collect();
        let mut me = Hosted {
            medium: Medium::new(&[l; 3], 2 * n),
            a: site(c, c, c),
            b: site(c + MED.r, c, c),
            n,
            gain,
            g_e: 0.0,
            hot: theta.iter().any(|&t| t > 0.0),
            site_sd: theta.iter().map(|t| t.sqrt()).collect(),
            edge_sd,
        };
        me.g_e = me.calibrate();
        me
    }

    /// The peak over 40 steps at B of a pulse of size `gain` injected at A, at T = 0.
    fn calibrate(&self) -> f64 {
        let mut probe = Hosted { medium: Medium::new(&[MED.side; 3], 1), a: self.a, b: self.b, n: 1, gain: self.gain, g_e: 0.0, hot: false, site_sd: vec![], edge_sd: vec![] };
        probe.medium.inject(self.a, 0, &[1.0], self.gain);
        let mut peak: f64 = 0.0;
        for t in 0..40 {
            peak = peak.max(probe.medium.sense(self.b, 0, 1)[0]);
            probe.transport(0, t);
        }
        peak
    }

    fn send_event(&mut self, event: Option<usize>) -> Option<usize> {
        if let Some(e) = event {
            let mut v = vec![0.0; self.n];
            v[e] = 1.0;
            self.medium.inject(self.a, 0, &v, self.gain);
        }
        let got = self.medium.sense(self.b, 0, self.n);
        let top = argmax(&got);
        (got[top] >= 0.5 * self.g_e && self.g_e > 0.0).then_some(top)
    }

    fn send_reading(&mut self, pm: &[f64]) -> Vec<f64> {
        self.medium.inject(self.b, self.n, pm, self.gain);
        let got: Vec<f64> = self.medium.sense(self.a, self.n, self.n).iter().map(|x| x.max(0.0)).collect();
        let s: f64 = got.iter().sum();
        if s > 0.0 { got.iter().map(|x| x / s).collect() } else { vec![1.0 / self.n as f64; self.n] }
    }

    /// The bath at the local temperature: called at temperature 1 with the noise scaled by √θ per
    /// site (damping) and by √(mean θ of the two ends) per edge (exchange) — T2's law with θ(site).
    /// The same step on the planned bath (bit-identical; checked by `T6_PARITY`).
    fn transport(&mut self, seed: u64, t: usize) {
        if std::env::var_os("T6_REFERENCE").is_some() {
            return self.transport_reference(seed, t);
        }
        let (site_sd, edge_sd, hot) = (&self.site_sd, &self.edge_sd, self.hot);
        let p = plan();
        for (sp, f) in self.medium.fields.iter_mut().enumerate() {
            for sub in 0..MED.substeps {
                let key = seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ ((sp * 64 + sub) as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                if hot {
                    thermo_bath_fast::exchange(&mut f.e, p, 1.0, MED.kappa, |s, ax| edge_sd[s * 3 + ax] * kick(key ^ ((s * 4 + ax) as u64).wrapping_mul(0x1656_67B1_9E37_79F9)));
                    thermo_bath_fast::damp(&mut f.e, 1.0, MED.leak, |s| site_sd[s] * kick(key ^ ((s * 4 + 3) as u64).wrapping_mul(0x1656_67B1_9E37_79F9)));
                } else {
                    thermo_bath_fast::exchange(&mut f.e, p, 0.0, MED.kappa, |_, _| 0.0);
                    thermo_bath_fast::damp(&mut f.e, 0.0, MED.leak, |_| 0.0);
                }
            }
        }
    }

    /// The bath through `thermo-bath` itself (T6 run 1's transport).
    fn transport_reference(&mut self, seed: u64, t: usize) {
        let (site_sd, edge_sd, hot) = (&self.site_sd, &self.edge_sd, self.hot);
        for (sp, f) in self.medium.fields.iter_mut().enumerate() {
            for sub in 0..MED.substeps {
                let key = seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ ((sp * 64 + sub) as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                let zero = |_: usize| 0.0;
                let gamma = |_: usize| MED.leak;
                if hot {
                    exchange(f, 1.0, MED.kappa, &zero, &|s, ax| edge_sd[s * 3 + ax] * kick(key ^ ((s * 4 + ax) as u64).wrapping_mul(0x1656_67B1_9E37_79F9)));
                    damp(f, 1.0, &gamma, &zero, &|s| site_sd[s] * kick(key ^ ((s * 4 + 3) as u64).wrapping_mul(0x1656_67B1_9E37_79F9)));
                } else {
                    exchange(f, 0.0, MED.kappa, &zero, &|_, _| 0.0);
                    damp(f, 0.0, &gamma, &zero, &zero);
                }
            }
        }
    }
}

fn decide(w: &[f64], theta: f64, u: f64) -> usize {
    if theta < GREEDY_BELOW { argmax(w) } else { draw(&temper(w, theta), u) }
}

/// One world, one brain, hosted in the base.
fn run(c: &Cfg, spec: &Spec, w: &World, seed: u64) -> Vec<usize> {
    let n = spec.n;
    let theta = theta_field(c);
    let mut host = Hosted::new(&theta, c.gain, n);
    let (theta_a, theta_b) = (theta[host.a], theta[host.b]);
    let mut memory = State::uniform(n);
    let mut tracker = State::uniform(n);
    let (mut register, mut weights) = (vec![1.0 / n as f64; n], vec![1.0 / n as f64; n]);
    let mut choices = Vec::with_capacity(w.obs.len());
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        let fired = gate(&w.obs[t.saturating_sub(DETECTOR_M - 1)..=t], o, DETECTOR_K).then_some(o);
        let event = host.send_event(fired);
        let pm = match c.brain {
            Brain::Quantum => {
                if let Some(e) = event {
                    memory.project(e);
                }
                if theta_b > 0.0 {
                    for j in 0..n {
                        memory.excite(j, C::polar(theta_b, 2.0 * std::f64::consts::PI * (chaos(seed, t, j, 300) + 0.5)));
                    }
                    memory.normalize();
                }
                memory.born_weights()
            }
            Brain::Classical => {
                if let Some(e) = event {
                    register = vec![0.0; n];
                    register[e] = 1.0;
                }
                for r in register.iter_mut() {
                    *r = (1.0 - theta_b) * *r + theta_b / n as f64;
                }
                register.clone()
            }
        };
        let p_hat = host.send_reading(&pm);
        match c.brain {
            Brain::Quantum => {
                let p = tracker.born_weights()[o];
                tracker.scale(o, 1.0 + ETA * 3.0 * (1.0 + p / (p + 0.1)) * (1.0 - p));
                for (j, p) in p_hat.iter().enumerate() {
                    tracker.scale(j, 1.0 + 0.05 * p);
                }
                tracker.power_magnitudes(1.0 - 0.05);
                tracker.normalize();
                choices.push(decide(&tracker.born_weights(), theta_a, w.uniforms[t]));
            }
            Brain::Classical => {
                let p = weights[o];
                weights[o] *= 1.0 + ETA * 4.0 * (1.0 + p / (p + 0.1)) * (1.0 - p);
                for (j, r) in p_hat.iter().enumerate() {
                    weights[j] *= 1.0 + 0.1 * r;
                }
                decay(&mut weights, 0.05);
                conserve(&mut weights);
                choices.push(decide(&weights, theta_a, w.uniforms[t]));
            }
        }
        host.transport(seed, t);
    }
    choices
}

#[derive(Debug, Clone, Copy, Default)]
struct Res {
    acc: f64,
    cued: f64,
    missed: f64,
}

/// The job log: one line per finished (configuration, seed), appended as it finishes, so a stopped
/// run keeps what it did. A restart skips every job already in the log.
fn key(c: &Cfg) -> String {
    format!("{:?}|{}|{}|{}|{}", c.brain, c.theta_a, c.theta_b, c.gain, c.chaos)
}

fn log_path() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("T6_LOG").unwrap_or_else(|_| "evidence/thermo-6/jobs.csv".into()))
}

fn load_log() -> HashMap<(String, u64), Res> {
    let mut m = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(log_path()) {
        for line in text.lines().skip(1) {
            let f: Vec<&str> = line.split(',').collect();
            if f.len() == 5 {
                m.insert((f[0].to_owned(), f[1].parse().unwrap()), Res { acc: f[2].parse().unwrap(), cued: f[3].parse().unwrap(), missed: f[4].parse().unwrap() });
            }
        }
    }
    m
}

/// Every (configuration, seed) job not yet in the log, on a pool of `T6_THREADS` threads (default
/// 8, for the room); then the mean per configuration over its seeds, in seed order.
fn evaluate_all(jobs: &[(Cfg, std::ops::Range<u64>)]) -> Vec<Res> {
    let done = load_log();
    let flat: Vec<(usize, u64)> = jobs.iter().enumerate().flat_map(|(i, (_, s))| s.clone().map(move |seed| (i, seed))).filter(|&(i, seed)| !done.contains_key(&(key(&jobs[i].0), seed))).collect();
    let threads: usize = std::env::var("T6_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    eprintln!("  {} jobs to run ({} already in the log), {threads} threads", flat.len(), jobs.iter().map(|(_, s)| s.end - s.start).sum::<u64>() as usize - flat.len());
    let path = log_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap();
    }
    let fresh = !path.exists();
    let file = Mutex::new(std::fs::OpenOptions::new().create(true).append(true).open(&path).unwrap());
    if fresh {
        writeln!(file.lock().unwrap(), "config,seed,acc,cued,missed").unwrap();
    }
    let next = AtomicUsize::new(0);
    let spec = qb7_spec();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::SeqCst);
                if k >= flat.len() {
                    break;
                }
                let (i, seed) = flat[k];
                let w = world(&spec, seed);
                let ch = run(&jobs[i].0, &spec, &w, seed);
                let (cued, missed) = split_acc(&spec, &w, &ch);
                let acc = score(&spec, &w, &ch).acc_resp;
                let mut f = file.lock().unwrap();
                writeln!(f, "{},{seed},{acc:?},{cued:?},{missed:?}", key(&jobs[i].0)).unwrap();
                f.flush().unwrap();
                drop(f);
                if (k + 1) % 50 == 0 {
                    eprintln!("  {} of {} jobs", k + 1, flat.len());
                }
            });
        }
    });
    let done = load_log();
    jobs.iter()
        .map(|(c, seeds)| {
            let m = (seeds.end - seeds.start) as f64;
            let mut r = Res::default();
            for seed in seeds.clone() {
                let x = done[&(key(c), seed)];
                r.acc += x.acc / m;
                r.cued += x.cued / m;
                r.missed += x.missed / m;
            }
            r
        })
        .collect()
}

/// E0: the stationary per-site variance of one species field at uniform θ.
fn equipartition(theta: f64) -> f64 {
    let l = MED.side;
    let mut h = Hosted::new(&vec![theta; l * l * l], 1.0, 1);
    h.medium = Medium::new(&[l; 3], 1);
    let (mut s2, mut s1, mut k) = (0.0, 0.0, 0.0);
    for t in 0..200 {
        h.transport(77, t);
        if t >= 100 {
            for x in &h.medium.fields[0].e {
                s1 += x;
                s2 += x * x;
                k += 1.0;
            }
        }
    }
    s2 / k - (s1 / k).powi(2)
}

fn main() {
    let (dev, test) = (0..16u64, 100..132u64);
    if std::env::var_os("T6_SMOKE").is_some() {
        // The job log's own check: a tiny batch, then the same batch again (nothing reruns).
        let jobs = vec![(Cfg { brain: Brain::Quantum, theta_a: 0.05, theta_b: 0.02, gain: 1000.0, chaos: 0.0 }, 100..102u64), (Cfg { brain: Brain::Classical, theta_a: 0.0, theta_b: 0.05, gain: 1000.0, chaos: 0.0 }, 100..102u64)];
        let spec = qb7_spec();
        for seed in 100..102u64 {
            let w = world(&spec, seed);
            let (q, c) = (run(&jobs[0].0, &spec, &w, seed), run(&jobs[1].0, &spec, &w, seed));
            let len = spec.trial_len();
            let scored = |i: &usize| i % len >= spec.cue + spec.delay;
            let diff: Vec<usize> = (0..q.len()).filter(|&i| q[i] != c[i]).collect();
            println!("seed {seed}: quantum and classical choices differ at {} of {} steps, {} of them in scored response steps", diff.len(), q.len(), diff.iter().filter(|i| scored(i)).count());
        }
        let a = evaluate_all(&jobs);
        let b = evaluate_all(&jobs);
        println!("smoke: first {:?}; second (from the log) {:?}; identical {}", a.iter().map(|r| r.acc).collect::<Vec<_>>(), b.iter().map(|r| r.acc).collect::<Vec<_>>(), a.iter().zip(&b).all(|(x, y)| x.acc == y.acc && x.cued == y.cued && x.missed == y.missed));
        return;
    }
    if std::env::var_os("T6_PARITY").is_some() {
        // The planned bath against the bath itself, on hot and cold jobs: every choice identical.
        let spec = qb7_spec();
        for c in [Cfg { brain: Brain::Quantum, theta_a: 0.05, theta_b: 0.02, gain: 1000.0, chaos: 0.3 }, Cfg { brain: Brain::Classical, theta_a: 0.0, theta_b: 0.0, gain: 1.0, chaos: 0.0 }] {
            let w = world(&spec, 100);
            let t = std::time::Instant::now();
            let fast = run(&c, &spec, &w, 100);
            let tf = t.elapsed();
            std::env::set_var("T6_REFERENCE", "1");
            let t = std::time::Instant::now();
            let slow = run(&c, &spec, &w, 100);
            let ts = t.elapsed();
            std::env::remove_var("T6_REFERENCE");
            println!("parity {c:?}: {} of {} choices differ; planned {tf:?}, reference {ts:?}", fast.iter().zip(&slow).filter(|(a, b)| a != b).count(), fast.len());
        }
        return;
    }
    println!("T6 THERMO-1: the hosted brains (M at r {}) in a base at temperature; grid {GRID:?}", MED.r);
    let var = equipartition(0.05);
    println!("E0: stationary variance at theta 0.05 = {var:.5} ({:+.2}%)", (var / 0.05 - 1.0) * 100.0);
    let cfg = |brain, a, b, gain| Cfg { brain, theta_a: a, theta_b: b, gain, chaos: 0.0 };
    let brains = [Brain::Quantum, Brain::Classical];
    // Batch 1: the uniform sweep on test (both gains), the local grid on dev (gain 1000).
    let mut jobs = Vec::new();
    for &b in &brains {
        for &g in &[1.0, 1000.0] {
            for &t in &GRID {
                jobs.push((cfg(b, t, t, g), test.clone()));
            }
        }
        for &ta in &GRID {
            for &tb in &GRID {
                jobs.push((cfg(b, ta, tb, 1000.0), dev.clone()));
            }
        }
    }
    let res = evaluate_all(&jobs);
    let find = |c: &Cfg, seeds: &std::ops::Range<u64>| jobs.iter().position(|(x, s)| x == c && s == seeds).map(|i| res[i]);
    let mut summary = Vec::new();
    for &b in &brains {
        println!("\n== {b:?} brain, uniform base temperature (test)");
        for &g in &[1.0, 1000.0] {
            let row: Vec<String> = GRID.iter().map(|&t| { let r = find(&cfg(b, t, t, g), &test).unwrap(); format!("{t}: {:.3} ({:.3}/{:.3})", r.acc, r.cued, r.missed) }).collect();
            println!("  gain {g:>6}: {}", row.join("  "));
        }
        println!("  local grid on dev, gain 1000 (rows theta_A, columns theta_B):");
        for &ta in &GRID {
            let row: Vec<String> = GRID.iter().map(|&tb| format!("{:.4}", find(&cfg(b, ta, tb, 1000.0), &dev).unwrap().acc)).collect();
            println!("    theta_A {ta:<5} {}", row.join(" "));
        }
        let uni_best = GRID.iter().copied().fold((f64::MIN, 0.0), |acc, t| { let a = find(&cfg(b, t, t, 1000.0), &dev).unwrap().acc; if a > acc.0 { (a, t) } else { acc } }).1;
        let mut loc_best = (f64::MIN, (0.0, 0.0));
        for &ta in &GRID {
            for &tb in &GRID {
                let a = find(&cfg(b, ta, tb, 1000.0), &dev).unwrap().acc;
                if a > loc_best.0 {
                    loc_best = (a, (ta, tb));
                }
            }
        }
        summary.push((b, uni_best, loc_best.1));
    }
    // Batch 2: test the best local pairs; the chaos round's dev arms at those pairs.
    let mut jobs2 = Vec::new();
    for &(b, _, (ta, tb)) in &summary {
        jobs2.push((cfg(b, ta, tb, 1000.0), test.clone()));
        jobs2.push((Cfg { chaos: 0.3, ..cfg(b, ta, tb, 1000.0) }, dev.clone()));
    }
    let res2 = evaluate_all(&jobs2);
    let mut jobs3 = Vec::new();
    let mut chaos_dev_win = Vec::new();
    for (k, &(b, _, (ta, tb))) in summary.iter().enumerate() {
        let plain_dev = find(&cfg(b, ta, tb, 1000.0), &dev).unwrap().acc;
        let win = res2[2 * k + 1].acc > plain_dev;
        chaos_dev_win.push(win);
        if win {
            jobs3.push((Cfg { chaos: 0.3, ..cfg(b, ta, tb, 1000.0) }, test.clone()));
        }
    }
    let res3 = evaluate_all(&jobs3);
    let (mut p1, mut p3, mut p4, mut p5) = (true, true, true, true);
    let mut peaks = Vec::new();
    let mut k3 = 0;
    for (k, &(b, ub, (ta, tb))) in summary.iter().enumerate() {
        let u = |t: f64, g: f64| find(&cfg(b, t, t, g), &test).unwrap().acc;
        let interior = [0.01, 0.02, 0.05, 0.1].iter().map(|&t| u(t, 1000.0)).fold(f64::MIN, f64::max);
        p1 &= interior >= u(0.0, 1000.0) + 0.005 && interior >= u(0.2, 1000.0) + 0.005;
        p3 &= GRID.iter().filter(|&&t| t >= 0.01).all(|&t| u(t, 1.0) <= u(t, 1000.0) - 0.05) && GRID.iter().all(|&t| u(t, 1.0) <= u(0.0, 1.0));
        let (uni_test, loc_test) = (u(ub, 1000.0), res2[2 * k].acc);
        p4 &= loc_test >= uni_test + 0.003;
        let plain_dev = find(&cfg(b, ta, tb, 1000.0), &dev).unwrap().acc;
        let chaos_kept = if chaos_dev_win[k] { let t = res3[k3].acc; k3 += 1; t > loc_test } else { false };
        p5 &= !chaos_kept;
        println!(
            "\n{b:?}: best uniform (dev) theta {ub} -> test {uni_test:.3}; best local (dev) (theta_A {ta}, theta_B {tb}) -> test {loc_test:.3}; drop from the uniform peak to 0.2: {:.3}; chaos 0.3 at the local pair: dev {:.4} vs plain {plain_dev:.4} -> {}",
            uni_test - u(0.2, 1000.0),
            res2[2 * k + 1].acc,
            if chaos_kept { "KEPT" } else { "not kept" }
        );
        peaks.push((ub, uni_test - u(0.2, 1000.0)));
    }
    let p2 = peaks[0].0 <= peaks[1].0 && peaks[0].1 > peaks[1].1;
    let v = [
        ("E0 the base has its set temperature (within 2%)", (var / 0.05 - 1.0).abs() <= 0.02),
        ("P1 inverted U at gain 1000, both brains", p1),
        ("P2 quantum peaks colder and degrades faster", p2),
        ("P3 heat silences unamplified signalling", p3),
        ("P4 local beats uniform, both brains (>= 0.003)", p4),
        ("P5 frozen chaos on the temperature field not kept", p5),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
