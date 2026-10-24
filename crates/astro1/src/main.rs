//! ASTRO-1 orchestrator (docs/ASTRO-POC.md): Pauli exclusion at write time against crosstalk in
//! FOLD-6's 2-bit content store. Every setting is here; the functions are astro-exclude (arm A),
//! astro-pressure (arm B), qb-bind and qb-quantize. FOLD-8's recipes are copied, so plain
//! reproduces 0.800 / 0.636 / 0.425 at N 384 / 512 / 768. Dev seeds choose eta (A) and gamma (B).

use qb_amplitude::C;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

const D: usize = 4096;
const M: usize = 256;
const BITS: u32 = 2;
const NS: [usize; 3] = [384, 512, 768];
const K_A: usize = 8;
const ETAS: [f64; 4] = [0.05, 0.1, 0.25, 0.5];
const ETA_FULL: f64 = 0.5;
const K_B: usize = 4;
const ETA_B: f64 = 0.5;
const GAMMAS: [f64; 3] = [0.5, 1.0, 2.0];
const THREADS: usize = 4;

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

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

/// Every reading z[i][v] = Re<key_i (.) code_v, q> / D (the phase-locked reading, up to its
/// per-item constant), blocked over items so each code is streamed once per block.
fn readings(q: &[C], keys: &[Vec<C>], cre: &[Vec<f64>], cim: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = keys.len();
    let mut z = vec![vec![0.0; M]; n];
    const BLK: usize = 8;
    for b0 in (0..n).step_by(BLK) {
        let b1 = (b0 + BLK).min(n);
        let us: Vec<(Vec<f64>, Vec<f64>)> = (b0..b1)
            .map(|i| {
                let u = qb_bind::bind(q, &conj(&keys[i]));
                (u.iter().map(|c| c.re).collect(), u.iter().map(|c| c.im).collect())
            })
            .collect();
        for v in 0..M {
            let (cr, ci) = (&cre[v], &cim[v]);
            for (j, (ur, ui)) in us.iter().enumerate() {
                let mut acc = 0.0;
                for k in 0..D {
                    acc += cr[k] * ur[k] + ci[k] * ui[k];
                }
                z[b0 + j][v] = acc / D as f64;
            }
        }
    }
    z
}

fn argmax(x: &[f64]) -> usize {
    let mut b = 0;
    for v in 1..x.len() {
        if x[v] > x[b] {
            b = v;
        }
    }
    b
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Arm {
    Plain,
    /// Exclusion of the shared component, eta, K_A iterations, pre-quantization.
    Exclude(f64),
    /// Degeneracy pressure on the colliding states through the quantizer, gamma; `random` = ablation.
    Pressure { gamma: f64, random: bool },
}

/// Fraction recalled, plus (for B) the collisions per iteration.
fn recall(arm: Arm, n: usize, seed: u64) -> (f64, Vec<usize>) {
    let content = random_content(seed, n);
    let codes = codebook(seed);
    let cre: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.re).collect()).collect();
    let cim: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.im).collect()).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| key(seed, i)).collect();
    let bound: Vec<Vec<C>> = (0..n).map(|i| qb_bind::bind(&keys[i], &codes[content[i] as usize])).collect();
    let mut s = vec![C::new(0.0, 0.0); D];
    for b in &bound {
        for (a, x) in s.iter_mut().zip(b) {
            *a = a.add(*x);
        }
    }
    let mut log = Vec::new();
    match arm {
        Arm::Plain => {}
        Arm::Exclude(eta) => {
            let target = vec![1.0; n];
            for _ in 0..K_A {
                astro_exclude::exclude(&mut s, &bound, &target, eta);
            }
        }
        Arm::Pressure { gamma, random } => {
            for it in 0..K_B {
                let mut q = s.clone();
                qb_quantize::quantize(&mut q, BITS);
                let z = readings(&q, &keys, &cre, &cim);
                // Measured noise (rms of wrong readings) and Bussgang gain (least-squares q onto s).
                let (mut w2, mut wn) = (0.0, 0usize);
                for (i, zi) in z.iter().enumerate() {
                    for (v, x) in zi.iter().enumerate() {
                        if v != content[i] as usize {
                            w2 += x * x;
                            wn += 1;
                        }
                    }
                }
                let sigma = (w2 / wn as f64).sqrt();
                let (mut dot, mut nn) = (0.0, 0.0);
                for (a, b) in q.iter().zip(&s) {
                    dot += a.re * b.re + a.im * b.im;
                    nn += b.norm2();
                }
                let alpha_q = dot / nn;
                let mut delta = vec![C::new(0.0, 0.0); D];
                let mut count = 0;
                for (i, zi) in z.iter().enumerate() {
                    let own = content[i] as usize;
                    let c = zi[own];
                    let mut r = Rng(seed.wrapping_mul(0x5851_F42D_4C95_7F2D) ^ ((it * 100_003 + i) as u64).wrapping_mul(0x2545_F491_4F6C_DD1D));
                    for (v, &x) in zi.iter().enumerate() {
                        if v == own || x <= c - gamma * sigma {
                            continue;
                        }
                        let push = ETA_B * (x - c + gamma * sigma) / (2.0 * alpha_q);
                        let rival = if random {
                            let mut w = (r.next() % (M as u64 - 1)) as usize;
                            if w >= own {
                                w += 1;
                            }
                            w
                        } else {
                            v
                        };
                        let rb = qb_bind::bind(&keys[i], &codes[rival]);
                        astro_pressure::push(&mut delta, &bound[i], &rb, push);
                        count += 1;
                    }
                }
                for (a, d) in s.iter_mut().zip(&delta) {
                    *a = a.add(*d);
                }
                log.push(count);
            }
        }
    }
    qb_quantize::quantize(&mut s, BITS);
    let z = readings(&s, &keys, &cre, &cim);
    let ok = z.iter().zip(&content).filter(|(zi, &v)| argmax(zi) == v as usize).count();
    (ok as f64 / n as f64, log)
}

fn run(jobs: &[(Arm, usize, u64)]) -> Vec<(f64, Vec<usize>)> {
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<(f64, Vec<usize>)>> = jobs.iter().map(|_| std::sync::Mutex::new((0.0, Vec::new()))).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (a, n, sd) = jobs[j];
                *out[j].lock().unwrap() = recall(a, n, sd);
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap()).collect()
}

/// Mean accuracy per N for each arm.
fn grid(arms: &[Arm], seeds: &[u64]) -> Vec<Vec<f64>> {
    let jobs: Vec<(Arm, usize, u64)> = arms.iter().flat_map(|&a| NS.iter().flat_map(move |&n| seeds.iter().map(move |&s| (a, n, s)))).collect();
    let res = run(&jobs);
    let k = seeds.len();
    (0..arms.len()).map(|ai| (0..NS.len()).map(|ni| res[(ai * NS.len() + ni) * k..(ai * NS.len() + ni + 1) * k].iter().map(|r| r.0).sum::<f64>() / k as f64).collect()).collect()
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

fn row(name: &str, v: &[f64]) -> String {
    format!("  {name:<44}{}", v.iter().map(|x| format!(" {x:.3}")).collect::<String>())
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    let dev: Vec<u64> = (0..4).collect();
    let test: Vec<u64> = (10..18).collect();
    say(format!("ASTRO-1: Pauli exclusion at write time; 2-bit store D {D}; N {NS:?}; {THREADS} threads"));

    // Dev: eta per N for A (plain wins ties), one gamma for B.
    let mut dev_arms = vec![Arm::Plain];
    dev_arms.extend(ETAS.iter().map(|&e| Arm::Exclude(e)));
    dev_arms.extend(GAMMAS.iter().map(|&g| Arm::Pressure { gamma: g, random: false }));
    let dv = grid(&dev_arms, &dev);
    say("\nDev (seeds 0-3):".into());
    for (a, v) in dev_arms.iter().zip(&dv) {
        say(row(&format!("{a:?}"), v));
    }
    let eta_dev: Vec<f64> = (0..NS.len())
        .map(|ni| {
            let mut best = (dv[0][ni], 0.0);
            for (k, &e) in ETAS.iter().enumerate() {
                if dv[1 + k][ni] > best.0 {
                    best = (dv[1 + k][ni], e);
                }
            }
            best.1
        })
        .collect();
    let gi = (0..GAMMAS.len()).max_by(|&a, &b| mean(&dv[5 + a]).partial_cmp(&mean(&dv[5 + b])).unwrap()).unwrap();
    let gamma = GAMMAS[gi];
    say(format!("  chosen: eta per N {eta_dev:?} (0 = plain); gamma {gamma}  ({:.0} s)", t0.elapsed().as_secs_f64()));

    // Test.
    let arms = [Arm::Plain, Arm::Exclude(ETA_FULL), Arm::Pressure { gamma, random: false }, Arm::Pressure { gamma, random: true }];
    let names = ["plain (eta 0, the ablation of A)", "A: full exclusion (eta 0.5, K 8)", "B: pressure on colliding states", "B ablation: same pushes, random rivals"];
    let tr = grid(&arms, &test);
    // A at the dev-chosen eta per N.
    let ed = &eta_dev;
    let jobs: Vec<(Arm, usize, u64)> = NS.iter().enumerate().flat_map(|(ni, &n)| test.iter().map(move |&s| (if ed[ni] > 0.0 { Arm::Exclude(ed[ni]) } else { Arm::Plain }, n, s))).collect();
    let rd = run(&jobs);
    let a_dev: Vec<f64> = (0..NS.len()).map(|ni| rd[ni * 8..(ni + 1) * 8].iter().map(|r| r.0).sum::<f64>() / 8.0).collect();
    // B's collision counts per iteration (test, mean over seeds) for the record.
    let logs = run(&NS.iter().flat_map(|&n| test.iter().map(move |&s| (Arm::Pressure { gamma, random: false }, n, s))).collect::<Vec<_>>());
    say(format!("\nTest (seeds 10-17), accuracy at N {NS:?}:"));
    for (nm, v) in names.iter().zip(&tr) {
        say(row(nm, v));
    }
    say(row("A at dev-chosen eta", &a_dev));
    for (ni, n) in NS.iter().enumerate() {
        let c: Vec<f64> = (0..K_B).map(|it| logs[ni * 8..(ni + 1) * 8].iter().map(|r| r.1[it] as f64).sum::<f64>() / 8.0).collect();
        say(format!("  B collisions per iteration at N {n}: {c:.0?}"));
    }
    let (pl, a, b, bab) = (&tr[0], &tr[1], &tr[2], &tr[3]);
    let v = [
        (format!("X1 full exclusion at N 384: +0.02..+0.07 ({:+.3})", a[0] - pl[0]), (0.02..=0.07).contains(&(a[0] - pl[0]))),
        (format!("X2 at N 512 no gain: -0.07..+0.02 ({:+.3})", a[1] - pl[1]), (-0.07..=0.02).contains(&(a[1] - pl[1]))),
        (format!("X3 at N 768 loses >= 0.08 ({:+.3})", a[2] - pl[2]), a[2] - pl[2] <= -0.08),
        (format!("X4 dev-chosen eta, test gain at N 768 <= +0.01 ({:+.3})", a_dev[2] - pl[2]), a_dev[2] - pl[2] <= 0.01),
        (format!("X5 B >= plain + 0.10 at N 512 ({:+.3})", b[1] - pl[1]), b[1] - pl[1] >= 0.10),
        (format!("X6 B >= plain + 0.05 at N 768 ({:+.3})", b[2] - pl[2]), b[2] - pl[2] >= 0.05),
        (format!("X7 B >= ablation + 0.08 at N 512 ({:+.3}) and ablation within 0.05 of plain ({:+.3})", b[1] - bab[1], bab[1] - pl[1]), b[1] - bab[1] >= 0.08 && (bab[1] - pl[1]).abs() <= 0.05),
    ];
    say(String::new());
    for (nm, ok) in &v {
        say(format!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
