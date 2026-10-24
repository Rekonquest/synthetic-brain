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
const NS: [usize; 2] = [512, 768];
const ROUNDS: usize = 3;
const RHO_K1: f64 = 0.75; // FOLD-8's dev choice
const T0_K6: f64 = 0.005; // FOLD-8's dev choice
const KAPPAS: [f64; 3] = [0.1, 0.2, 0.5];
const CS: [f64; 3] = [0.02, 0.05, 0.1];
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Arm {
    Plain,
    /// Hard cancellation of the top rho by margin (FOLD-8 K1; rho 1 = no gate).
    Hard(f64),
    /// Soft, fixed schedule T0 / 2^(r-1) (FOLD-8 K6).
    Schedule(f64),
    /// Soft, each item's own T = kappa sigma^2 / m.
    Virial(f64),
    /// Soft, every item at the round's median virial T (kappa).
    OneTemp(f64),
    /// Soft, T = c m (bound -> hot).
    Literal(f64),
}

/// ASTRO-R: chaos on the per-item temperatures (selected by the round's total margin) and the
/// number of rounds (amplification: more of the same cooling).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ext {
    chaos: bool,
    rounds: usize,
}

const BASE: Ext = Ext { chaos: false, rounds: ROUNDS };

struct Store<'a> {
    keys: &'a [Vec<C>],
    cre: &'a [Vec<f64>],
    cim: &'a [Vec<f64>],
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
                sc[j][v] = if *na > 0.0 { acc / (D as f64 * na).sqrt() } else { 0.0 };
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

/// Accuracy, plus the median per-item temperature of each round (soft arms).
fn recall(arm: Arm, ext: Ext, n: usize, seed: u64) -> (f64, Vec<f64>) {
    let content = random_content(seed, n);
    let codes = codebook(seed);
    let cre: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.re).collect()).collect();
    let cim: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.im).collect()).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| key(seed, i)).collect();
    let st = Store { keys: &keys, cre: &cre, cim: &cim };
    let mut s = vec![C::new(0.0, 0.0); D];
    for (i, &v) in content.iter().enumerate() {
        for (a, b) in s.iter_mut().zip(qb_bind::bind(&keys[i], &codes[v as usize])) {
            *a = a.add(b);
        }
    }
    qb_quantize::quantize(&mut s, BITS);
    let mut reads = read_pass(&st, &s, None);
    let mut temps = Vec::new();
    let rounds = if arm == Arm::Plain { 0 } else { ext.rounds };
    for r in 1..=rounds {
        let acting: Vec<usize> = match arm {
            Arm::Hard(rho) => {
                let mut order: Vec<usize> = (0..n).collect();
                order.sort_by(|&a, &b| reads[b].1.partial_cmp(&reads[a].1).unwrap());
                order[..((rho * n as f64).round() as usize).max(1)].to_vec()
            }
            _ => (0..n).collect(),
        };
        // Each item's temperature this round.
        let t_item: Vec<f64> = match arm {
            Arm::Schedule(t0) => vec![t0 / (1u64 << (r - 1)) as f64; n],
            Arm::Virial(k) => reads.iter().map(|x| astro_virial::temperature(&x.2, k)).collect(),
            Arm::OneTemp(k) => vec![median(reads.iter().map(|x| astro_virial::temperature(&x.2, k)).collect()); n],
            Arm::Literal(c) => reads.iter().map(|x| c * x.1).collect(),
            _ => vec![0.0; n],
        };
        if !matches!(arm, Arm::Hard(_)) {
            temps.push(median(t_item.clone()));
        }
        // PREDICT → CONSERVE → REVISE at the given per-item temperatures.
        let pass = |t_item: &[f64]| -> Vec<(usize, f64, Vec<f64>)> {
            let mut own: Vec<Option<Vec<C>>> = vec![None; n];
            let mut e = vec![C::new(0.0, 0.0); D];
            for &j in &acting {
                let cv: Vec<C> = if let Arm::Hard(_) = arm {
                    codes[reads[j].0].clone()
                } else {
                    let p = qb_classical::temper(&reads[j].2.iter().map(|x| (x - reads[j].2[reads[j].0]).exp()).collect::<Vec<_>>(), t_item[j]);
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
                let b = qb_bind::bind(&keys[j], &cv);
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
            read_pass(&st, &s, Some((&e, alpha, &own)))
        };
        let next = pass(&t_item);
        reads = if ext.chaos && !matches!(arm, Arm::Hard(_)) {
            let total = |rd: &Vec<(usize, f64, Vec<f64>)>| rd.iter().map(|x| x.1).sum::<f64>();
            let mut best = (total(&next), next);
            for v in 1..=3u64 {
                let tv: Vec<f64> = t_item.iter().enumerate().map(|(i, t)| {
                    let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((r as u64) * 8 + v).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (i as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                    t * (1.0 + 0.5 * (qb_chaos::keyed_unit(k) - 0.5))
                }).collect();
                let rd = pass(&tv);
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
    let ok = reads.iter().zip(&content).filter(|(r, &v)| r.0 == v as usize).count();
    (ok as f64 / n as f64, temps)
}

fn run(jobs: &[(Arm, Ext, usize, u64)]) -> Vec<(f64, Vec<f64>)> {
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<(f64, Vec<f64>)>> = jobs.iter().map(|_| std::sync::Mutex::new((0.0, Vec::new()))).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (a, x, n, sd) = jobs[j];
                *out[j].lock().unwrap() = recall(a, x, n, sd);
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap()).collect()
}

/// Per arm: mean accuracy per N, and mean per-round median temperature per N.
fn grid(arms: &[(Arm, Ext)], seeds: &[u64]) -> (Vec<Vec<f64>>, Vec<Vec<Vec<f64>>>) {
    let jobs: Vec<(Arm, Ext, usize, u64)> = arms.iter().flat_map(|&(a, x)| NS.iter().flat_map(move |&n| seeds.iter().map(move |&s| (a, x, n, s)))).collect();
    let res = run(&jobs);
    let k = seeds.len();
    let acc = (0..arms.len()).map(|ai| (0..NS.len()).map(|ni| res[(ai * NS.len() + ni) * k..(ai * NS.len() + ni + 1) * k].iter().map(|r| r.0).sum::<f64>() / k as f64).collect()).collect();
    let tem = (0..arms.len())
        .map(|ai| {
            (0..NS.len())
                .map(|ni| {
                    let sl = &res[(ai * NS.len() + ni) * k..(ai * NS.len() + ni + 1) * k];
                    let rr = sl[0].1.len();
                    (0..rr).map(|r| sl.iter().map(|x| x.1[r]).sum::<f64>() / k as f64).collect()
                })
                .collect()
        })
        .collect();
    (acc, tem)
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
    say(format!("ASTRO-2R: the virial law as each item's own temperature, two candidate laws; N {NS:?}; {ROUNDS} rounds"));
    let mut dev_arms: Vec<(Arm, Ext)> = CS.iter().map(|&c| (Arm::Literal(c), BASE)).collect();
    dev_arms.extend(KAPPAS.iter().map(|&k| (Arm::Virial(k), BASE)));
    let (dv, _) = grid(&dev_arms, &dev);
    for (a, v) in dev_arms.iter().zip(&dv) {
        say(row(&format!("dev {:?}", a.0), v));
    }
    let bi = (0..dev_arms.len()).max_by(|&a, &b| mean(&dv[a]).partial_cmp(&mean(&dv[b])).unwrap()).unwrap();
    let law = dev_arms[bi].0;
    say(format!("  law chosen on dev: {law:?} (L1 = Literal, bound -> hot; L2 = Virial at the new scale)"));
    let arms = [(Arm::Plain, BASE), (Arm::Hard(RHO_K1), BASE), (Arm::Schedule(T0_K6), BASE), (law, BASE)];
    let names = ["plain", "hard K1 (rho 0.75)", "K6 schedule (T0 0.005)", "the chosen virial law"];
    let (tr, tm) = grid(&arms, &test);
    for (nm, v) in names.iter().zip(&tr) {
        say(row(&format!("test {nm}"), v));
    }
    say(format!("  median item temperature per round (law, N 512): {:.5?}", tm[3][0]));
    // Chaos and amplification on the law.
    let ch = (law, Ext { chaos: true, rounds: ROUNDS });
    let am = (law, Ext { chaos: false, rounds: 2 * ROUNDS });
    let (dc, da) = (grid(&[ch], &dev).0, grid(&[am], &dev).0);
    let (tc, ta) = (grid(&[ch], &test).0, grid(&[am], &test).0);
    let chaos_kept = mean(&dc[0]) > mean(&dv[bi]) && mean(&tc[0]) > mean(&tr[3]);
    let amp_gain = mean(&ta[0]) - mean(&tr[3]);
    say(row("dev chaos on the temperatures", &dc[0]));
    say(row("test chaos on the temperatures", &tc[0]));
    say(row("dev rounds x 2", &da[0]));
    say(row("test rounds x 2", &ta[0]));
    let v = [
        (format!("R2a law within +-0.02 of K6 at N 512 ({:.3} vs {:.3})", tr[3][0], tr[2][0]), (tr[3][0] - tr[2][0]).abs() <= 0.02),
        (format!("R2b no arm beats plain at N 768 (plain {:.3}; K1 {:.3}, K6 {:.3}, law {:.3})", tr[0][1], tr[1][1], tr[2][1], tr[3][1]), tr[1..].iter().all(|v| v[1] <= tr[0][1])),
        ("R2c chaos not kept".to_string(), !chaos_kept),
        (format!("R2d rounds x 2 adds <= 0.01 (test mean {amp_gain:+.3})"), amp_gain <= 0.01),
    ];
    say(String::new());
    for (nm, ok) in &v {
        say(format!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
