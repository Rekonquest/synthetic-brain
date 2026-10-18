//! T5 runner (docs/T5-HOST.md): QB-9's final quantum brain and its tuned classical comparator, with
//! D, M and T hosted at sites of a 3D medium. D's events and M's readings travel through the medium
//! as diffusing species (`thermo-host` ports, T2's bath as the transport). Own checks H1–H5, then the
//! amplification (H6), heat (H7) and chaos (H8) rounds. Every setting lives in `Host` and the brains'
//! constants (QB-9's kept values, copied).

use qb_amplitude::{State, C};
use qb_chaos::keyed_unit;
use qb_classical::{argmax, conserve, decay, draw, gate, temper};
use qb_task::{qb7_spec, score, split_acc, world, Spec, World};
use thermo_bath::{damp, exchange, normal};
use thermo_host::Medium;

const ETA: f64 = 0.15;
const DETECTOR_K: usize = 3;
const DETECTOR_M: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Brain {
    Quantum,
    Classical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Host {
    wired: bool,
    side: usize,
    r: usize,
    kappa: f64,
    substeps: usize,
    leak: f64,
    gain: f64,
    temperature: f64,
    chaos: f64,
}

fn ideal() -> Host {
    Host { wired: false, side: 16, r: 0, kappa: 0.0, substeps: 1, leak: f64::INFINITY, gain: 1.0, temperature: 0.0, chaos: 0.0 }
}

fn remote(r: usize) -> Host {
    Host { wired: false, side: 16, r, kappa: 0.5, substeps: 4, leak: 0.1, gain: 1.0, temperature: 0.0, chaos: 0.0 }
}

/// QB-9's centred chaos for (seed, step, element, tag).
fn chaos(seed: u64, t: usize, element: usize, tag: u64) -> f64 {
    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (element as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ tag.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    keyed_unit(key) - 0.5
}

/// A uniform in (0, 1) from a SplitMix64 hash: the bath's noise source (any standard normal satisfies
/// T2's law; the hash is used for speed, the chaos source stays the logistic).
fn hash_unit(key: u64) -> f64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 11) as f64 + 0.5) / (1u64 << 53) as f64
}

/// The medium with its ports: A hosts the sensor, D and T; B = A + (r, 0, 0) hosts M.
struct Hosted {
    medium: Medium,
    a: usize,
    b: usize,
    n: usize,
    h: Host,
    /// The peak of a unit event pulse's response at B (the calibration), times the gain.
    g_e: f64,
    gamma: Vec<f64>,
}

impl Hosted {
    fn new(h: &Host, n: usize) -> Hosted {
        let dims = [h.side; 3];
        let c = h.side / 2;
        let site = |x: usize, y: usize, z: usize| x + h.side * y + h.side * h.side * z;
        let (a, b) = (site(c, c, c), site(c + h.r, c, c));
        let gamma: Vec<f64> = (0..h.side.pow(3)).map(|s| if h.chaos > 0.0 { h.leak * (1.0 + h.chaos * (keyed_unit(0x7E57_0005 ^ (s as u64).wrapping_mul(0x9E37_79B9)) - 0.5)) } else { h.leak }).collect();
        let mut me = Hosted { medium: Medium::new(&dims, 2 * n), a, b, n, h: *h, g_e: 0.0, gamma };
        me.g_e = me.calibrate();
        me
    }

    /// The peak over 40 steps, at B, of a unit pulse injected at A (sensed before each transport),
    /// at T = 0, times the gain.
    fn calibrate(&self) -> f64 {
        let mut probe = Hosted { medium: Medium::new(&[self.h.side; 3], 1), a: self.a, b: self.b, n: 1, h: Host { temperature: 0.0, ..self.h }, g_e: 0.0, gamma: self.gamma.clone() };
        probe.medium.inject(self.a, 0, &[1.0], self.h.gain);
        let mut peak: f64 = 0.0;
        for t in 0..40 {
            peak = peak.max(probe.medium.sense(self.b, 0, 1)[0]);
            probe.transport(0, t);
        }
        peak
    }

    /// D → M: inject the event at A, sense at B; the event is the argmax species if it reaches
    /// ½·G_E (thin policy, labelled in the doc).
    fn send_event(&mut self, event: Option<usize>) -> Option<usize> {
        if let Some(e) = event {
            let mut v = vec![0.0; self.n];
            v[e] = 1.0;
            self.medium.inject(self.a, 0, &v, self.h.gain);
        }
        let got = self.medium.sense(self.b, 0, self.n);
        let top = argmax(&got);
        (got[top] >= 0.5 * self.g_e && self.g_e > 0.0).then_some(top)
    }

    /// M → T: inject the reading at B, sense at A, normalized to a distribution.
    fn send_reading(&mut self, pm: &[f64]) -> Vec<f64> {
        self.medium.inject(self.b, self.n, pm, self.h.gain);
        let got: Vec<f64> = self.medium.sense(self.a, self.n, self.n).iter().map(|x| x.max(0.0)).collect();
        let s: f64 = got.iter().sum();
        if s > 0.0 { got.iter().map(|x| x / s).collect() } else { vec![1.0 / self.n as f64; self.n] }
    }

    /// The medium's step: `substeps` × (exchange at κ, leak towards 0), at the medium's temperature.
    fn transport(&mut self, seed: u64, t: usize) {
        let (temp, kappa) = (self.h.temperature, self.h.kappa);
        let gamma = &self.gamma;
        for (sp, f) in self.medium.fields.iter_mut().enumerate() {
            for sub in 0..self.h.substeps {
                let key = seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ ((sp * 64 + sub) as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                let zero1 = |_: usize| 0.0;
                if kappa > 0.0 {
                    if temp > 0.0 {
                        let nz = |s: usize, ax: usize| {
                            let k = key ^ ((s * 4 + ax) as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                            normal(hash_unit(k), hash_unit(k ^ 0x5555_5555_5555_5555))
                        };
                        exchange(f, temp, kappa, &zero1, &nz);
                    } else {
                        exchange(f, 0.0, kappa, &zero1, &|_, _| 0.0);
                    }
                }
                let g = |s: usize| gamma[s];
                if temp > 0.0 {
                    let nz = |s: usize| {
                        let k = key ^ ((s * 4 + 3) as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                        normal(hash_unit(k), hash_unit(k ^ 0x5555_5555_5555_5555))
                    };
                    damp(f, temp, &g, &zero1, &nz);
                } else {
                    damp(f, 0.0, &g, &zero1, &zero1);
                }
            }
        }
    }
}

struct Trace {
    choices: Vec<usize>,
    /// What T consumed as M's reading, every step.
    p_hat: Vec<Vec<f64>>,
}

/// One world, one brain, wired or hosted.
fn run(brain: Brain, h: &Host, spec: &Spec, w: &World, seed: u64) -> Trace {
    let n = spec.n;
    let mut host = (!h.wired).then(|| Hosted::new(h, n));
    let mut tr = Trace { choices: Vec::new(), p_hat: Vec::new() };
    // The quantum brain (QB-9 kept): collapse memory with occupation heat 0.02; Born tracker, gain ×3,
    // growth κ 1 with product inhibition ι 1, λ 0.05, η_m 0.05, decision temperature 0.05.
    let mut memory = State::uniform(n);
    let mut tracker = State::uniform(n);
    // The classical brain (QB-9 tuned): register with floor 0.05; weights, gain ×4, κ 1, ι 1, λ 0.05,
    // η_m 0.1, argmax decision.
    let (mut register, mut weights) = (vec![1.0 / n as f64; n], vec![1.0 / n as f64; n]);
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        let fired = gate(&w.obs[t.saturating_sub(DETECTOR_M - 1)..=t], o, DETECTOR_K).then_some(o);
        let event = match host.as_mut() {
            Some(hs) => hs.send_event(fired),
            None => fired,
        };
        let pm = match brain {
            Brain::Quantum => {
                if let Some(e) = event {
                    memory.project(e);
                }
                for j in 0..n {
                    memory.excite(j, C::polar(0.02, 2.0 * std::f64::consts::PI * (chaos(seed, t, j, 300) + 0.5)));
                }
                memory.normalize();
                memory.born_weights()
            }
            Brain::Classical => {
                if let Some(e) = event {
                    register = vec![0.0; n];
                    register[e] = 1.0;
                }
                for r in register.iter_mut() {
                    *r = (1.0 - 0.05) * *r + 0.05 / n as f64;
                }
                register.clone()
            }
        };
        let p_hat = match host.as_mut() {
            Some(hs) => hs.send_reading(&pm),
            None => pm,
        };
        match brain {
            Brain::Quantum => {
                let p = tracker.born_weights()[o];
                let growth = 1.0 + 1.0 * p / (p + 0.1);
                let brake = 1.0 - p;
                tracker.scale(o, 1.0 + ETA * 3.0 * growth * brake);
                for (j, p) in p_hat.iter().enumerate() {
                    tracker.scale(j, 1.0 + 0.05 * p);
                }
                tracker.power_magnitudes(1.0 - 0.05);
                tracker.normalize();
                tr.choices.push(draw(&temper(&tracker.born_weights(), 0.05), w.uniforms[t]));
            }
            Brain::Classical => {
                let p = weights[o];
                let growth = 1.0 + 1.0 * p / (p + 0.1);
                let brake = 1.0 - p;
                weights[o] *= 1.0 + ETA * 4.0 * growth * brake;
                for (j, r) in p_hat.iter().enumerate() {
                    weights[j] *= 1.0 + 0.1 * r;
                }
                decay(&mut weights, 0.05);
                conserve(&mut weights);
                tr.choices.push(argmax(&weights));
            }
        }
        tr.p_hat.push(p_hat);
        if let Some(hs) = host.as_mut() {
            hs.transport(seed, t);
        }
    }
    tr
}

#[derive(Debug, Clone, Copy, Default)]
struct Res {
    acc: f64,
    cued: f64,
    missed: f64,
    log: f64,
}

/// The channel's consumer-side check: the mean over steps ≥ cue of ln(max(p̂_target, 1e-9)).
fn log_score(spec: &Spec, w: &World, tr: &Trace) -> f64 {
    let len = spec.trial_len();
    let (mut s, mut k) = (0.0, 0);
    for (i, p) in tr.p_hat.iter().enumerate() {
        if i % len >= spec.cue {
            s += p[w.targets[i / len]].max(1e-9).ln();
            k += 1;
        }
    }
    s / k as f64
}

/// Mean over seeds, one thread per seed; also returns every seed's choices.
fn evaluate(brain: Brain, h: &Host, seeds: std::ops::Range<u64>) -> (Res, Vec<Vec<usize>>) {
    let spec = qb7_spec();
    let seeds: Vec<u64> = seeds.collect();
    let per: Vec<(Res, Vec<usize>)> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let spec = &spec;
                sc.spawn(move || {
                    let w = world(spec, seed);
                    let tr = run(brain, h, spec, &w, seed);
                    let (cued, missed) = split_acc(spec, &w, &tr.choices);
                    (Res { acc: score(spec, &w, &tr.choices).acc_resp, cued, missed, log: log_score(spec, &w, &tr) }, tr.choices)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let k = per.len() as f64;
    let mut m = Res::default();
    for (r, _) in &per {
        m.acc += r.acc / k;
        m.cued += r.cued / k;
        m.missed += r.missed / k;
        m.log += r.log / k;
    }
    (m, per.into_iter().map(|x| x.1).collect())
}

/// H2: the medium's linearity and reciprocity at T = 0, by separate medium runs.
fn linearity(h: &Host) -> (f64, f64) {
    let pulse = |at: &[(usize, f64)], read: usize| -> Vec<f64> {
        let mut m = Hosted::new(h, 1);
        m.medium = Medium::new(&[h.side; 3], 1);
        for &(s, v) in at {
            m.medium.inject(s, 0, &[v], 1.0);
        }
        let mut out = Vec::new();
        for t in 0..30 {
            out.push(m.medium.sense(read, 0, 1)[0]);
            m.transport(0, t);
        }
        out
    };
    let probe = Hosted::new(h, 1);
    let (a, b) = (probe.a, probe.b);
    let c = b + h.side * 3; // a third site, off the A–B axis
    let both = pulse(&[(a, 1.0), (c, 0.7)], b);
    let (one, two) = (pulse(&[(a, 1.0)], b), pulse(&[(c, 0.7)], b));
    let peak = both.iter().cloned().fold(0.0, f64::max);
    let sup = both.iter().zip(one.iter().zip(&two)).map(|(x, (y, z))| (x - y - z).abs()).fold(0.0, f64::max) / peak;
    let (ab, ba) = (pulse(&[(a, 1.0)], b), pulse(&[(b, 1.0)], a));
    let rec = ab.iter().zip(&ba).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max) / ab.iter().cloned().fold(0.0, f64::max);
    (sup, rec)
}

fn main() {
    let (dev, test) = (0..16u64, 100..132u64);
    if std::env::var("T5_DIAG").is_ok() {
        // Diagnostic, added after the run (not a prediction): the A->B and B->A pulse responses at
        // r = 2, step by step, and the same with the lattice mirrored (B->A measured as A->B on x -> L-1-x).
        let h = remote(2);
        let resp = |from: usize, to: usize| -> Vec<f64> {
            let mut m = Hosted::new(&h, 1);
            m.medium = Medium::new(&[h.side; 3], 1);
            m.medium.inject(from, 0, &[1.0], 1.0);
            (0..8).map(|t| { let v = m.medium.sense(to, 0, 1)[0]; m.transport(0, t); v }).collect()
        };
        let p = Hosted::new(&h, 1);
        let fmt = |v: &[f64]| v.iter().map(|x| format!("{x:.5}")).collect::<Vec<_>>().join(" ");
        println!("A(x=8)->B(x=10): {}", fmt(&resp(p.a, p.b)));
        println!("B(x=10)->A(x=8): {}", fmt(&resp(p.b, p.a)));
        println!("x=7 -> x=9 (odd start): {}", fmt(&resp(p.a - 1, p.a + 1)));
        println!("x=9 -> x=7: {}", fmt(&resp(p.a + 1, p.a - 1)));
        return;
    }
    if std::env::var("T5_TIME").is_ok() {
        for h in [ideal(), remote(2), Host { temperature: 1e-6, ..remote(2) }] {
            let t0 = std::time::Instant::now();
            let (r, _) = evaluate(Brain::Quantum, &h, 100..101);
            println!("one seed: {:?} -> acc {:.3}, log {:.4}", t0.elapsed(), r.acc, r.log);
        }
        return;
    }
    println!("T5: QB-9's brains hosted in the medium; QB-7 world (p_miss 0.4); dev {dev:?}, test {test:?}");
    let wired = Host { wired: true, ..ideal() };
    let show = |label: &str, r: &Res| println!("  {label:<34} acc_resp {:.3} (cued {:.3}, missed {:.3}); channel log-score {:.4}", r.acc, r.cued, r.missed, r.log);
    // H1.
    let mut h1 = true;
    for brain in [Brain::Quantum, Brain::Classical] {
        let (rw, cw) = evaluate(brain, &wired, test.clone());
        let (ri, ci) = evaluate(brain, &ideal(), test.clone());
        let diff: usize = cw.iter().zip(&ci).map(|(a, b)| a.iter().zip(b).filter(|(x, y)| x != y).count()).sum();
        show(&format!("{brain:?} wired"), &rw);
        show(&format!("{brain:?} ideal medium"), &ri);
        println!("    choices differing from the wiring: {diff}");
        let want = if brain == Brain::Quantum { 0.776 } else { 0.775 };
        h1 &= diff == 0 && (rw.acc - want).abs() < 0.0005;
    }
    // H2.
    let (sup, rec) = linearity(&remote(2));
    println!("  H2 linearity: superposition error {sup:.2e} of the peak; reciprocity error {rec:.2e}");
    // H3, H4.
    let mut h3 = true;
    let mut h4 = true;
    for r in 1..=3 {
        for brain in [Brain::Quantum, Brain::Classical] {
            let hs = remote(r);
            let g = Hosted::new(&hs, 1).g_e;
            let (res, _) = evaluate(brain, &hs, test.clone());
            show(&format!("{brain:?} remote r {r} (G_E {g:.4})"), &res);
            let (want, tol) = (if brain == Brain::Quantum { 0.776 } else { 0.775 }, if r == 3 { 0.02 } else { 0.01 });
            let ok = (res.acc - want).abs() <= tol;
            if brain == Brain::Quantum { h3 &= ok } else { h4 &= ok }
        }
    }
    // H5.
    let (s16, _) = evaluate(Brain::Quantum, &remote(2), test.clone());
    let (s24, _) = evaluate(Brain::Quantum, &Host { side: 24, ..remote(2) }, test.clone());
    show("Quantum remote r 2, side 24", &s24);
    let h5 = (s16.acc - s24.acc).abs() <= 0.003 && (s16.log - s24.log).abs() <= 0.01;
    // Rounds on the remote quantum brain at r = 2, judged by the channel log-score.
    let mut kept = remote(2);
    let mut verdicts = Vec::new();
    for name in ["amplification", "heat", "chaos"] {
        let variants: Vec<Host> = match name {
            "amplification" => [2.0, 4.0].iter().map(|&g| Host { gain: g, ..kept }).collect(),
            "heat" => [1e-6, 1e-5].iter().map(|&t| Host { temperature: t, ..kept }).collect(),
            _ => [0.1, 0.3].iter().map(|&a| Host { chaos: a, ..kept }).collect(),
        };
        let off = evaluate(Brain::Quantum, &kept, dev.clone()).0;
        println!("   {name}: plain dev log-score {:.6} (acc {:.3})", off.log, off.acc);
        let mut best = (off.log, kept);
        for v in &variants {
            let r = evaluate(Brain::Quantum, v, dev.clone()).0;
            println!("   {name} variant (gain {}, T {:e}, chaos {}): dev log-score {:.6} (acc {:.3})", v.gain, v.temperature, v.chaos, r.log, r.acc);
            if r.log > best.0 + 1e-12 {
                best = (r.log, *v);
            }
        }
        let keep = if best.1 == kept {
            println!("   {name}: plain wins on dev -> not kept");
            false
        } else {
            let (toff, ton) = (evaluate(Brain::Quantum, &kept, test.clone()).0, evaluate(Brain::Quantum, &best.1, test.clone()).0);
            let k = ton.log > toff.log + 1e-12;
            println!("   {name}: test off {:.6} (acc {:.3}) -> on {:.6} (acc {:.3}) -> {}", toff.log, toff.acc, ton.log, ton.acc, if k { "KEPT" } else { "not kept" });
            k
        };
        verdicts.push(keep);
        if keep {
            kept = best.1;
        }
    }
    let v = [
        ("H1 the ideal medium is the wiring (every choice identical)", h1),
        ("H2 linear (<= 1e-12) and reciprocal (<= 1e-12)", sup <= 1e-12 && rec <= 1e-12),
        ("H3 a remote memory works (quantum)", h3),
        ("H4 a remote memory works (classical)", h4),
        ("H5 the mesh is large enough (side 16 -> 24)", h5),
        ("H6 amplification not kept (exact tie)", !verdicts[0]),
        ("H7 heat not kept", !verdicts[1]),
        ("H8 chaos not kept", !verdicts[2]),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
