//! FOLD-8 orchestrator (docs/FOLD-8-PRECOMMIT.md): recall with Jesse's cognitive roots against the
//! plain single read, in FOLD-6's 2-bit content store. The functions (qb-bind, qb-quantize,
//! qb-overlap-phase, qb-classical::temper, qb-chaos) are unchanged; every root lives here:
//! - PERCEPTION: per item, the named hypothesis "item i holds v", citing its margin and round;
//! - GATE (thin policy, labelled): only the top ρ of a round's margins may act;
//! - PREDICT: the expected interference E, the sum of the acting hypotheses' bound vectors;
//! - CONSERVE: E scaled onto the store by least squares;
//! - REVISE: each item re-read from the store with the others' predicted interference dissolved;
//! - ATTEND + REINFORCE: a prior over the 256 values from the acting hypotheses' counts;
//! - hot and cold: soft hypotheses (Boltzmann weights at T) cooling each round (O8);
//! - amplification: write gain, or over-cancellation G; chaos on the cancellation scale.
//! Run from the repository root (it reads docs/GLOSSARY.md). Dev seeds choose every threshold.

use qb_amplitude::C;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

const D: usize = 4096;
const M: usize = 256;
const BITS: u32 = 2;
const NS: [usize; 5] = [256, 384, 512, 768, 1024];
const ROUNDS: usize = 3;
const RHOS: [f64; 3] = [0.25, 0.5, 0.75];
const BETAS: [f64; 4] = [0.002, 0.005, 0.01, 0.02];
const T0S: [f64; 3] = [0.005, 0.01, 0.02];
const GS: [f64; 2] = [1.25, 1.5];
const ATTEND_RHO: f64 = 0.5;
const CHAOS_ALPHA: f64 = 0.3;
const CHAOS_VARIANTS: usize = 3;
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

/// FOLD-6's recipes, so plain reproduces FOLD-6's 2-bit numbers.
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

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cancel {
    None,
    Hard,
    Soft,
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    rounds: usize,
    cancel: Cancel,
    rho: f64,
    conserve: bool,
    beta: f64,
    t0: f64,
    cool: bool,
    write_gain: f64,
    g_amp: f64,
    chaos: bool,
}

const PLAIN: Cfg = Cfg { rounds: 0, cancel: Cancel::None, rho: 1.0, conserve: true, beta: 0.0, t0: 0.0, cool: true, write_gain: 1.0, g_amp: 1.0, chaos: false };

/// Per item: (best value, margin, the 256 readings).
type Reads = Vec<(usize, f64, Vec<f64>)>;

fn margin_of(sc: &[f64]) -> (usize, f64) {
    let (mut b, mut s) = (0usize, usize::MAX);
    for v in 1..sc.len() {
        if sc[v] > sc[b] {
            s = b;
            b = v;
        } else if s == usize::MAX || sc[v] > sc[s] {
            s = v;
        }
    }
    (b, sc[b] - sc[s])
}

/// One read pass: item i read from s − α_i·(E − own_i), with ATTEND's prior (β·ln w).
fn read_pass(s: &[C], e: Option<(&[C], &[f64], &[Option<Vec<C>>])>, prior: &[f64], beta: f64, seed: u64, codes: &[Vec<C>], n: usize) -> Reads {
    (0..n)
        .map(|i| {
            let w: Vec<C> = match e {
                None => s.to_vec(),
                Some((e, alpha, own)) => {
                    let a = alpha[i];
                    s.iter()
                        .enumerate()
                        .map(|(k, x)| {
                            let mut p = e[k];
                            if let Some(o) = &own[i] {
                                p = C::new(p.re - o[k].re, p.im - o[k].im);
                            }
                            C::new(x.re - a * p.re, x.im - a * p.im)
                        })
                        .collect()
                }
            };
            let u = qb_bind::bind(&w, &conj(&key(seed, i)));
            let sc: Vec<f64> = codes.iter().enumerate().map(|(v, c)| qb_overlap_phase::overlap_phase(c, &u) + if beta > 0.0 { beta * prior[v].ln() } else { 0.0 }).collect();
            let (b, m) = margin_of(&sc);
            (b, m, sc)
        })
        .collect()
}

/// Recall every item; per item (correct, final margin).
fn recall(cfg: &Cfg, content: &[u8], seed: u64) -> Vec<(bool, f64)> {
    let n = content.len();
    let codes = codebook(seed);
    let mut s = vec![C::new(0.0, 0.0); D];
    for (i, &v) in content.iter().enumerate() {
        for (a, b) in s.iter_mut().zip(qb_bind::bind(&key(seed, i), &codes[v as usize])) {
            *a = a.add(b.scale(cfg.write_gain));
        }
    }
    qb_quantize::quantize(&mut s, BITS);
    let uniform = vec![1.0 / M as f64; M];
    let mut reads = read_pass(&s, None, &uniform, 0.0, seed, &codes, n);
    for r in 1..=cfg.rounds {
        // PERCEPTION's hypotheses are `reads`; GATE: the top ρ by margin may act.
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| reads[b].1.partial_cmp(&reads[a].1).unwrap());
        let rho = if cfg.cancel == Cancel::Hard { cfg.rho } else { ATTEND_RHO };
        let passed: Vec<usize> = order[..((rho * n as f64).round() as usize).max(1)].to_vec();
        // REINFORCE → ATTEND: a prior over the values from the passed hypotheses.
        let mut prior = vec![1.0; M];
        for &i in &passed {
            prior[reads[i].0] += 1.0;
        }
        let tot: f64 = prior.iter().sum();
        prior.iter_mut().for_each(|x| *x /= tot);
        if cfg.cancel == Cancel::None {
            reads = read_pass(&s, None, &prior, cfg.beta, seed, &codes, n);
            continue;
        }
        // PREDICT: each acting hypothesis's bound vector (hard: its value; soft: the Boltzmann mixture).
        let acting: Vec<usize> = if cfg.cancel == Cancel::Hard { passed } else { (0..n).collect() };
        let t = if cfg.cool { cfg.t0 / (1u64 << (r - 1)) as f64 } else { cfg.t0 };
        let mut own: Vec<Option<Vec<C>>> = vec![None; n];
        let mut e = vec![C::new(0.0, 0.0); D];
        for &j in &acting {
            let content_vec: Vec<C> = if cfg.cancel == Cancel::Hard {
                codes[reads[j].0].clone()
            } else {
                let p = qb_classical::temper(&reads[j].2.iter().map(|x| (x - reads[j].2[reads[j].0]).exp()).collect::<Vec<_>>(), t);
                let mut mix = vec![C::new(0.0, 0.0); D];
                for (v, pv) in p.iter().enumerate() {
                    if *pv > 1e-6 {
                        for (m, c) in mix.iter_mut().zip(&codes[v]) {
                            *m = m.add(c.scale(*pv));
                        }
                    }
                }
                mix
            };
            let b = qb_bind::bind(&key(seed, j), &content_vec);
            for (x, y) in e.iter_mut().zip(&b) {
                *x = x.add(*y);
            }
            own[j] = Some(b);
        }
        // CONSERVE: scale E onto the store by least squares (or 1 when ablated); amplification G.
        let (mut dot, mut nn) = (0.0, 0.0);
        for (x, y) in e.iter().zip(&s) {
            dot += x.re * y.re + x.im * y.im;
            nn += x.norm2();
        }
        let alpha = if cfg.conserve { if nn > 0.0 { dot / nn } else { 0.0 } } else { 1.0 } * cfg.g_amp;
        let plain_alpha = vec![alpha; n];
        // REVISE: dissolve the others' predicted interference, then re-read.
        let next = read_pass(&s, Some((&e, &plain_alpha, &own)), &prior, cfg.beta, seed, &codes, n);
        reads = if cfg.chaos {
            let total = |rd: &Reads| rd.iter().map(|x| x.1).sum::<f64>();
            let mut best = (total(&next), next);
            for v in 1..=CHAOS_VARIANTS {
                let a: Vec<f64> = (0..n)
                    .map(|i| {
                        let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((r * 8 + v) as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (i as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                        alpha * (1.0 + CHAOS_ALPHA * (qb_chaos::keyed_unit(k) - 0.5))
                    })
                    .collect();
                let rd = read_pass(&s, Some((&e, &a, &own)), &prior, cfg.beta, seed, &codes, n);
                let tt = total(&rd);
                if tt > best.0 {
                    best = (tt, rd);
                }
            }
            best.1
        } else {
            next
        };
    }
    reads.iter().zip(content).map(|(r, &v)| (r.0 == v as usize, r.1)).collect()
}

/// Jobs (glossary?, n, seed, cfg) on THREADS workers.
fn run(jobs: &[(bool, usize, u64, Cfg)], glossary: &[u8]) -> Vec<Vec<(bool, f64)>> {
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<Vec<(bool, f64)>>> = jobs.iter().map(|_| std::sync::Mutex::new(Vec::new())).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (text, n, sd, cfg) = jobs[j];
                let content = if text { glossary[..n].to_vec() } else { random_content(sd, n) };
                *out[j].lock().unwrap() = recall(&cfg, &content, sd);
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap()).collect()
}

/// Mean accuracy per N for each cfg (content `text`), over `seeds`.
fn grid(cfgs: &[Cfg], text: bool, seeds: &[u64], glossary: &[u8]) -> Vec<Vec<f64>> {
    let jobs: Vec<(bool, usize, u64, Cfg)> = cfgs.iter().flat_map(|&c| NS.iter().flat_map(move |&n| seeds.iter().map(move |&s| (text, n, s, c)))).collect();
    let res = run(&jobs, glossary);
    let k = seeds.len();
    cfgs.iter()
        .enumerate()
        .map(|(ci, _)| {
            (0..NS.len())
                .map(|ni| {
                    let base = (ci * NS.len() + ni) * k;
                    res[base..base + k].iter().map(|r| r.iter().filter(|x| x.0).count() as f64 / r.len() as f64).sum::<f64>() / k as f64
                })
                .collect()
        })
        .collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

fn row(name: &str, v: &[f64]) -> String {
    format!("  {name:<34}{}", v.iter().map(|x| format!(" {x:.3}")).collect::<String>())
}

fn main() {
    let t0 = std::time::Instant::now();
    let glossary = std::fs::read("docs/GLOSSARY.md").expect("run from the repository root");
    let test: Vec<u64> = (10..18).collect();
    let dev: Vec<u64> = (0..4).collect();
    say(format!("FOLD-8: the 2-bit content store (D {D}, {} bytes); N {NS:?}; dev 0-3 chooses, test 10-17 reports", D * BITS as usize / 8));

    // Dev: choose ρ (K1), β (K2), T0 (K6).
    let k1 = |rho: f64| Cfg { rounds: ROUNDS, cancel: Cancel::Hard, rho, ..PLAIN };
    let k2 = |beta: f64| Cfg { rounds: 1, beta, ..PLAIN };
    let k6 = |t0: f64, cool: bool| Cfg { rounds: ROUNDS, cancel: Cancel::Soft, t0, cool, ..PLAIN };
    let dv: Vec<f64> = grid(&RHOS.map(k1), false, &dev, &glossary).iter().map(|v| mean(v)).collect();
    let rho = RHOS[(0..3).max_by(|&a, &b| dv[a].partial_cmp(&dv[b]).unwrap()).unwrap()];
    let db: Vec<f64> = grid(&BETAS.map(k2), true, &dev, &glossary).iter().map(|v| mean(v)).collect();
    let beta = BETAS[(0..4).max_by(|&a, &b| db[a].partial_cmp(&db[b]).unwrap()).unwrap()];
    let dt: Vec<f64> = grid(&T0S.map(|t| k6(t, true)), false, &dev, &glossary).iter().map(|v| mean(v)).collect();
    let t0s = T0S[(0..3).max_by(|&a, &b| dt[a].partial_cmp(&dt[b]).unwrap()).unwrap()];
    say(format!("\nDev choices: rho {rho} (dev means {dv:.3?}); beta {beta} ({db:.3?}); T0 {t0s} ({dt:.3?})"));

    // The best cognitive stack (K1 or K6) on dev random, then G and chaos on it.
    let (k1c, k6c) = (k1(rho), k6(t0s, true));
    let dbest = grid(&[k1c, k6c], false, &dev, &glossary);
    let cog = if mean(&dbest[1]) > mean(&dbest[0]) { k6c } else { k1c };
    let dg: Vec<f64> = grid(&GS.map(|g| Cfg { g_amp: g, ..cog }), false, &dev, &glossary).iter().map(|v| mean(v)).collect();
    let g = GS[(0..2).max_by(|&a, &b| dg[a].partial_cmp(&dg[b]).unwrap()).unwrap()];
    say(format!("  best cognitive stack on dev: {:?} (K1 {:.3}, K6 {:.3}); G {g} ({dg:.3?})", cog.cancel, mean(&dbest[0]), mean(&dbest[1])));

    // Test: random content.
    let arms: Vec<(&str, Cfg)> = vec![
        ("plain (cognitive off)", PLAIN),
        ("amplification alone (write x 2)", Cfg { write_gain: 2.0, ..PLAIN }),
        ("K1 explaining away", k1c),
        ("K1 ablation: no GATE (rho 1)", k1(1.0)),
        ("K1 ablation: no CONSERVE (alpha 1)", Cfg { conserve: false, ..k1c }),
        ("K2 ATTEND + REINFORCE", k2(beta)),
        ("K1 + K2", Cfg { beta, ..k1c }),
        ("K6 annealed (hot -> cold)", k6c),
        ("K6 hot only (no cooling)", k6(t0s, false)),
        ("together: best cognitive + G", Cfg { g_amp: g, ..cog }),
        ("chaos on the best cognitive", Cfg { chaos: true, ..cog }),
    ];
    let cfgs: Vec<Cfg> = arms.iter().map(|a| a.1).collect();
    say(format!("\nTest, random content; accuracy at N {NS:?}:"));
    let tr = grid(&cfgs, false, &test, &glossary);
    for (a, v) in arms.iter().zip(&tr) {
        say(row(a.0, v));
    }
    say(format!("\nTest, glossary content; accuracy at N {NS:?}:"));
    let gl_arms = [0usize, 2, 5, 6, 7];
    let tg = grid(&gl_arms.map(|i| cfgs[i]), true, &test, &glossary);
    for (i, v) in gl_arms.iter().zip(&tg) {
        say(row(arms[*i].0, v));
    }
    // Chaos dev check for the keep rule.
    let dch = grid(&[cog, Cfg { chaos: true, ..cog }], false, &dev, &glossary);
    let chaos_kept = mean(&dch[1]) > mean(&dch[0]) && mean(&tr[10]) > mean(&tr[if cog.cancel == Cancel::Soft { 7 } else { 2 }]);

    // K4: assert-or-abstain on plain reads, random, N 384: g chosen on dev for precision >= 0.99.
    let collect = |seeds: &[u64]| -> Vec<(bool, f64)> { run(&seeds.iter().map(|&s| (false, 384usize, s, PLAIN)).collect::<Vec<_>>(), &glossary).concat() };
    let mut dvm = collect(&dev);
    dvm.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut gate = f64::INFINITY;
    let mut ok = 0usize;
    for (k, x) in dvm.iter().enumerate() {
        ok += x.0 as usize;
        if ok as f64 / (k + 1) as f64 >= 0.99 {
            gate = x.1;
        }
    }
    let tm = collect(&test);
    let asserted: Vec<&(bool, f64)> = tm.iter().filter(|x| x.1 >= gate).collect();
    let prec = asserted.iter().filter(|x| x.0).count() as f64 / asserted.len().max(1) as f64;
    let cov = asserted.len() as f64 / tm.len() as f64;
    say(format!("\nK4 assert-or-abstain (plain reads, random, N 384): margin gate {gate:.4} (dev); test precision {prec:.3}, coverage {cov:.3}"));

    let at = |v: &Vec<f64>, n: usize| v[NS.iter().position(|&x| x == n).unwrap()];
    let (pl, k1t, nog, nocon, k2t, k12, k6a, k6h, tog) = (&tr[0], &tr[2], &tr[3], &tr[4], &tr[5], &tr[6], &tr[7], &tr[8], &tr[9]);
    let amp_gap = tr[1].iter().zip(pl).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
    let k2rand = k2t.iter().zip(pl).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
    let best_cog = if cog.cancel == Cancel::Soft { k6a } else { k1t };
    let v = [
        (format!("K1a explaining away: N 512 >= 0.95 ({:.3}), N 768 >= 0.80 ({:.3})", at(k1t, 512), at(k1t, 768)), at(k1t, 512) >= 0.95 && at(k1t, 768) >= 0.80),
        (format!("K1b GATE load-bearing: no-gate >= 0.05 worse at N 768 ({:.3} vs {:.3})", at(nog, 768), at(k1t, 768)), at(nog, 768) <= at(k1t, 768) - 0.05),
        (format!("K1c CONSERVE load-bearing: no-conserve worse than plain at N 768 ({:.3} vs {:.3})", at(nocon, 768), at(pl, 768)), at(nocon, 768) < at(pl, 768)),
        (format!("K2a attention on the glossary, N 512 >= plain + 0.05 ({:.3} vs {:.3})", at(&tg[2], 512), at(&tg[0], 512)), at(&tg[2], 512) >= at(&tg[0], 512) + 0.05),
        (format!("K2b attention a tie on random (largest gap {k2rand:.3})"), k2rand <= 0.01),
        (format!("K3 together >= either alone on the glossary at N 768 ({:.3}; K1 {:.3}, K2 {:.3})", at(&tg[3], 768), at(&tg[1], 768), at(&tg[2], 768)), at(&tg[3], 768) >= at(&tg[1], 768).max(at(&tg[2], 768))),
        (format!("K4 assert-or-abstain: precision >= 0.99 ({prec:.3}) with coverage >= 0.6 ({cov:.3})"), prec >= 0.99 && cov >= 0.6),
        (format!("K5 chaos not kept (test {:+.3} mean)", mean(&tr[10]) - mean(best_cog)), !chaos_kept),
        (format!("K6a annealed >= hard K1 + 0.03 at N 768 ({:.3} vs {:.3})", at(k6a, 768), at(k1t, 768)), at(k6a, 768) >= at(k1t, 768) + 0.03),
        (format!("K6b cooling load-bearing: annealed >= hot-only + 0.03 at N 768 ({:.3} vs {:.3})", at(k6a, 768), at(k6h, 768)), at(k6a, 768) >= at(k6h, 768) + 0.03),
        (format!("K7a amplification alone an exact tie (largest gap {amp_gap:.3})"), amp_gap == 0.0),
        (format!("K7b together not kept, <= +0.01 (mean {:+.3})", mean(tog) - mean(best_cog)), mean(tog) - mean(best_cog) <= 0.01),
    ];
    let _ = k12;
    say(String::new());
    for (name, ok) in &v {
        say(format!("{} {name}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
