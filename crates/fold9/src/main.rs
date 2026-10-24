//! ASTRO-2 orchestrator (docs/ASTRO-POC.md): the virial theorem as a self-regulating hot/cold
//! governor in FOLD-8's recall loop (PREDICT -> CONSERVE -> REVISE, soft interference at
//! temperature T). FOLD-8's loop is copied; only who sets T changes:
//! - fixed schedule (FOLD-8 K6): T0, T0/2, T0/4 for every item;
//! - virial: each item's own T_i = kappa sigma_i^2 / m_i (astro-virial), no schedule;
//! - ablation: every item at the round's median virial T;
//! - literal virial (bound -> hot): T_i = c m_i.
//! Hard cancellation (K1, gate rho) and plain are the bars. Every setting is here.

use qb_amplitude::C;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

const D: usize = 4096;
const M: usize = 256;
const BITS: u32 = 2;
const NS: [usize; 4] = [384, 512, 768, 1024];
const THREADS: usize = 8;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn phasors(seed: u64) -> Vec<C> {
    let mut r = Rng(seed);
    (0..D).map(|_| C::polar(1.0, 2.0 * std::f64::consts::PI * r.unit())).collect()
}

fn key(seed: u64, i: usize) -> Vec<C> {
    phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ 0x6E7)
}

fn codebook(seed: u64) -> Vec<Vec<C>> {
    (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ 0xC0D)).collect()
}

fn random_content(seed: u64, n: usize) -> Vec<u8> {
    let mut r = Rng(seed ^ 0xB17E5);
    (0..n).map(|_| (r.next() % 256) as u8).collect()
}

struct Store<'a> {
    keys: &'a [Vec<C>],
    cre: &'a [Vec<f64>],
    cim: &'a [Vec<f64>],
    /// ATTEND: β·ln(prior_v) added to each code's reading (β 0 = off).
    prior: Vec<f64>,
    beta: f64,
}

/// Per item: (best, margin, the 256 phase-locked readings) of the store w_i = s - alpha (E - own_i).
fn read_pass(st: &Store, s: &[C], e: Option<(&[C], f64, &[Option<Vec<C>>])>) -> Vec<(usize, f64, Vec<f64>)> {
    let n = st.keys.len();
    let mut out = Vec::with_capacity(n);
    const BLK: usize = 8;
    for b0 in (0..n).step_by(BLK) {
        let b1 = (b0 + BLK).min(n);
        let us: Vec<(Vec<f64>, Vec<f64>, f64)> = (b0..b1)
            .map(|i| {
                let (mut ur, mut ui, mut na) = (vec![0.0; D], vec![0.0; D], 0.0);
                for k in 0..D {
                    let mut w = s[k];
                    if let Some((e, a, own)) = e {
                        let mut p = e[k];
                        if let Some(o) = &own[i] {
                            p = C::new(p.re - o[k].re, p.im - o[k].im);
                        }
                        w = C::new(w.re - a * p.re, w.im - a * p.im);
                    }
                    let kc = st.keys[i][k];
                    let u = w.mul(C::new(kc.re, -kc.im));
                    ur[k] = u.re;
                    ui[k] = u.im;
                    na += u.norm2();
                }
                (ur, ui, na)
            })
            .collect();
        let mut sc = vec![vec![0.0; M]; b1 - b0];
        for v in 0..M {
            let (cr, ci) = (&st.cre[v], &st.cim[v]);
            for (j, (ur, ui, na)) in us.iter().enumerate() {
                let mut acc = 0.0;
                for k in 0..D {
                    acc += cr[k] * ur[k] + ci[k] * ui[k];
                }
                sc[j][v] = if *na > 0.0 { acc / (D as f64 * na).sqrt() } else { 0.0 } + if st.beta > 0.0 { st.beta * st.prior[v].ln() } else { 0.0 };
            }
        }
        for x in sc {
            let (mut b, mut s2) = (0usize, usize::MAX);
            for v in 1..M {
                if x[v] > x[b] {
                    s2 = b;
                    b = v;
                } else if s2 == usize::MAX || x[v] > x[s2] {
                    s2 = v;
                }
            }
            let m = x[b] - x[s2];
            out.push((b, m, x));
        }
    }
    out
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Gate {
    /// The loop always runs (the ungated stack).
    Always,
    /// The loop never runs (plain, or ATTEND alone).
    Never,
    /// G-P: run only if the "recoverable" reading f >= theta.
    Perceive(f64),
    /// G-V: run, keep only if the total margin beats the first pass's.
    Verify,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cfg {
    gate: Gate,
    attend: bool,
    text: bool,
}

const KAPPA: f64 = 0.1;
const LOOP_ROUNDS: usize = 6;
const BETA: f64 = 0.01;
const PROBES: usize = 64;

fn gloss() -> Vec<u8> {
    std::fs::read("docs/GLOSSARY.md").expect("run from the repository root")
}

/// (accuracy, the "recoverable" reading f).
fn recall(cfg: Cfg, n: usize, seed: u64) -> (f64, f64) {
    let content: Vec<u8> = if cfg.text { gloss()[..n].to_vec() } else { random_content(seed, n) };
    let codes = codebook(seed);
    let cre: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.re).collect()).collect();
    let cim: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.im).collect()).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| key(seed, i)).collect();
    let mut s = vec![C::new(0.0, 0.0); D];
    for (i, &v) in content.iter().enumerate() {
        for (a, b) in s.iter_mut().zip(qb_bind::bind(&keys[i], &codes[v as usize])) {
            *a = a.add(b);
        }
    }
    qb_quantize::quantize(&mut s, BITS);
    let uniform = vec![1.0 / M as f64; M];
    let plain_st = Store { keys: &keys, cre: &cre, cim: &cim, prior: uniform.clone(), beta: 0.0 };
    let first = read_pass(&plain_st, &s, None);
    // SENSING: the noise floor g, the median first margin of keys never written.
    let probe_keys: Vec<Vec<C>> = (0..PROBES).map(|i| key(seed ^ 0x9E0B_E5, i)).collect();
    let probe = read_pass(&Store { keys: &probe_keys, cre: &cre, cim: &cim, prior: uniform.clone(), beta: 0.0 }, &s, None);
    let g = median(probe.iter().map(|x| x.1).collect());
    // PERCEPTION: the reading f behind the hypothesis "recoverable".
    let f = first.iter().filter(|x| x.1 > g).count() as f64 / n as f64;
    // ATTEND + REINFORCE: a prior from the top half of first margins.
    let mut prior = vec![1.0; M];
    if cfg.attend {
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| first[b].1.partial_cmp(&first[a].1).unwrap());
        for &i in &order[..n / 2] {
            prior[first[i].0] += 1.0;
        }
    }
    let tot: f64 = prior.iter().sum();
    prior.iter_mut().for_each(|x| *x /= tot);
    let st = Store { keys: &keys, cre: &cre, cim: &cim, prior, beta: if cfg.attend { BETA } else { 0.0 } };
    let base = if cfg.attend { read_pass(&st, &s, None) } else { first.clone() };
    // JUDGMENT: run the loop or not.
    let run_loop = match cfg.gate {
        Gate::Always | Gate::Verify => true,
        Gate::Never => false,
        Gate::Perceive(theta) => f >= theta,
    };
    let mut reads = base.clone();
    if run_loop {
        for _ in 0..LOOP_ROUNDS {
            let t_item: Vec<f64> = reads.iter().map(|x| astro_virial::temperature(&x.2, KAPPA)).collect();
            let mut own: Vec<Option<Vec<C>>> = vec![None; n];
            let mut e = vec![C::new(0.0, 0.0); D];
            for j in 0..n {
                let p = qb_classical::temper(&reads[j].2.iter().map(|x| (x - reads[j].2[reads[j].0]).exp()).collect::<Vec<_>>(), t_item[j]);
                let mut mix = vec![C::new(0.0, 0.0); D];
                for (v, pv) in p.iter().enumerate() {
                    if *pv > 1e-6 {
                        for (m, c) in mix.iter_mut().zip(&codes[v]) {
                            *m = m.add(c.scale(*pv));
                        }
                    }
                }
                let b = qb_bind::bind(&keys[j], &mix);
                for (x, y) in e.iter_mut().zip(&b) {
                    *x = x.add(*y);
                }
                own[j] = Some(b);
            }
            let (mut dot, mut nn) = (0.0, 0.0);
            for (x, y) in e.iter().zip(&s) {
                dot += x.re * y.re + x.im * y.im;
                nn += x.norm2();
            }
            let alpha = if nn > 0.0 { dot / nn } else { 0.0 };
            reads = read_pass(&st, &s, Some((&e, alpha, &own)));
        }
        if cfg.gate == Gate::Verify {
            let total = |r: &Vec<(usize, f64, Vec<f64>)>| r.iter().map(|x| x.1).sum::<f64>();
            if total(&reads) <= total(&base) {
                reads = base;
            }
        }
    }
    let ok = reads.iter().zip(&content).filter(|(r, &v)| r.0 == v as usize).count();
    (ok as f64 / n as f64, f)
}

fn run(jobs: &[(Cfg, usize, u64)]) -> Vec<(f64, f64)> {
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<(f64, f64)>> = jobs.iter().map(|_| std::sync::Mutex::new((0.0, 0.0))).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (c, n, sd) = jobs[j];
                *out[j].lock().unwrap() = recall(c, n, sd);
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap()).collect()
}

/// Per cfg: (mean accuracy per N, mean f per N).
fn grid(cfgs: &[Cfg], seeds: &[u64]) -> Vec<(Vec<f64>, Vec<f64>)> {
    let jobs: Vec<(Cfg, usize, u64)> = cfgs.iter().flat_map(|&c| NS.iter().flat_map(move |&n| seeds.iter().map(move |&s| (c, n, s)))).collect();
    let res = run(&jobs);
    let k = seeds.len();
    (0..cfgs.len())
        .map(|ci| {
            let sl = |ni: usize| &res[(ci * NS.len() + ni) * k..(ci * NS.len() + ni + 1) * k];
            ((0..NS.len()).map(|ni| sl(ni).iter().map(|r| r.0).sum::<f64>() / k as f64).collect(), (0..NS.len()).map(|ni| sl(ni).iter().map(|r| r.1).sum::<f64>() / k as f64).collect())
        })
        .collect()
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

fn row(name: &str, v: &[f64]) -> String {
    format!("  {name:<46}{}", v.iter().map(|x| format!(" {x:.3}")).collect::<String>())
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    let dev: Vec<u64> = (0..4).collect();
    let test: Vec<u64> = (10..18).collect();
    say(format!("FOLD-9: the PERCEPTION gate on the best recall stack (virial kappa {KAPPA}, {LOOP_ROUNDS} rounds); 2-bit store D {D}; N {NS:?}"));
    let c = |gate: Gate, attend: bool, text: bool| Cfg { gate, attend, text };
    let thetas = [0.6, 0.7, 0.8, 0.9];
    let dg = grid(&thetas.map(|t| c(Gate::Perceive(t), false, false)), &dev);
    let ti = (0..4).max_by(|&a, &b| mean(&dg[a].0).partial_cmp(&mean(&dg[b].0)).unwrap().then(b.cmp(&a))).unwrap();
    let theta = thetas[ti];
    say(format!("  dev theta: {:?} -> means {:.3?}; chosen {theta}", thetas, dg.iter().map(|x| mean(&x.0)).collect::<Vec<_>>()));
    let arms = [("plain", c(Gate::Never, false, false)), ("stack, ungated", c(Gate::Always, false, false)), ("G-P perception gate", c(Gate::Perceive(theta), false, false)), ("G-V verify gate", c(Gate::Verify, false, false))];
    let tr = grid(&arms.map(|a| a.1), &test);
    say(format!("\nRandom content, test, accuracy at N {NS:?}:"));
    for (a, r) in arms.iter().zip(&tr) {
        say(row(a.0, &r.0));
    }
    say(format!("  the reading f per N (test mean): {:.3?}", tr[0].1));
    let garms = [("plain", c(Gate::Never, false, true)), ("ATTEND alone", c(Gate::Never, true, true)), ("stack, ungated", c(Gate::Always, false, true)), ("G-P + ATTEND", c(Gate::Perceive(theta), true, true))];
    let tg = grid(&garms.map(|a| a.1), &test);
    say(format!("\nGlossary, test, accuracy at N {NS:?}:"));
    for (a, r) in garms.iter().zip(&tg) {
        say(row(a.0, &r.0));
    }
    let i = |n: usize| NS.iter().position(|&x| x == n).unwrap();
    let p1 = (0..NS.len()).all(|k| tr[2].0[k] >= tr[0].0[k] - 0.01) && (0..NS.len()).all(|k| tg[3].0[k] >= tg[0].0[k] - 0.01);
    let p2 = [384, 512].iter().all(|&n| tr[2].0[i(n)] >= tr[1].0[i(n)] - 0.01);
    let p3 = tr[3].0[i(768)] <= tr[2].0[i(768)] - 0.02;
    let p4 = (0..NS.len()).all(|k| tg[3].0[k] >= tg[0].0[k].max(tg[1].0[k]) - 0.01) && tg[3].0[i(512)] >= 0.95;
    let v = [
        ("P1 G-P never worse than plain - 0.01 (both contents)".to_string(), p1),
        (format!("P2 G-P keeps the gains at N 384, 512 ({:.3}, {:.3} vs ungated {:.3}, {:.3})", tr[2].0[i(384)], tr[2].0[i(512)], tr[1].0[i(384)], tr[1].0[i(512)]), p2),
        (format!("P3 G-V weaker at N 768 ({:.3} vs G-P {:.3})", tr[3].0[i(768)], tr[2].0[i(768)]), p3),
        (format!("P4 glossary: G-P + ATTEND >= max(plain, ATTEND) - 0.01 everywhere, >= 0.95 at N 512 ({:.3})", tg[3].0[i(512)]), p4),
    ];
    say(String::new());
    for (nm, ok) in &v {
        say(format!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
