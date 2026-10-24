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
const K_B: usize = 4;
const ETA_B: f64 = 0.5;
const GAMMAS: [f64; 3] = [0.5, 1.0, 2.0];
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
    /// ASTRO-R: pressure with each push divided by the item's collision count; eta multiplier
    /// (amplification); chaos (3 variants per iteration, fewest collisions selects).
    Pressure { gamma: f64, eta_mult: f64, chaos: bool },
}

/// One iteration's pressure delta with per-item push scales; returns (delta, collisions).
#[allow(clippy::too_many_arguments)]
fn pressure_delta(z: &[Vec<f64>], content: &[u8], keys: &[Vec<C>], codes: &[Vec<C>], bound: &[Vec<C>], gamma: f64, sigma: f64, alpha_q: f64, scale: &[f64]) -> (Vec<C>, usize) {
    let mut delta = vec![C::new(0.0, 0.0); D];
    let mut count = 0;
    for (i, zi) in z.iter().enumerate() {
        let own = content[i] as usize;
        let c = zi[own];
        let rivals: Vec<usize> = (0..M).filter(|&v| v != own && zi[v] > c - gamma * sigma).collect();
        count += rivals.len();
        let k = rivals.len().max(1) as f64;
        for &v in &rivals {
            let push = scale[i] * ETA_B * (zi[v] - c + gamma * sigma) / (2.0 * alpha_q) / k;
            let rb = qb_bind::bind(&keys[i], &codes[v]);
            astro_pressure::push(&mut delta, &bound[i], &rb, push);
        }
    }
    (delta, count)
}

fn collisions(z: &[Vec<f64>], content: &[u8], gamma: f64, sigma: f64) -> usize {
    z.iter().enumerate().map(|(i, zi)| {
        let own = content[i] as usize;
        (0..M).filter(|&v| v != own && zi[v] > zi[own] - gamma * sigma).count()
    }).sum()
}

fn noise_and_gain(q: &[C], s: &[C], z: &[Vec<f64>], content: &[u8]) -> (f64, f64) {
    let (mut w2, mut wn) = (0.0, 0usize);
    for (i, zi) in z.iter().enumerate() {
        for (v, x) in zi.iter().enumerate() {
            if v != content[i] as usize {
                w2 += x * x;
                wn += 1;
            }
        }
    }
    let (mut dot, mut nn) = (0.0, 0.0);
    for (a, b) in q.iter().zip(s) {
        dot += a.re * b.re + a.im * b.im;
        nn += b.norm2();
    }
    ((w2 / wn as f64).sqrt(), dot / nn)
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
    if let Arm::Pressure { gamma, eta_mult, chaos } = arm {
        for it in 0..K_B {
            let mut q = s.clone();
            qb_quantize::quantize(&mut q, BITS);
            let z = readings(&q, &keys, &cre, &cim);
            let (sigma, alpha_q) = noise_and_gain(&q, &s, &z, &content);
            let plain_scale = vec![eta_mult; n];
            let (d0, c0) = pressure_delta(&z, &content, &keys, &codes, &bound, gamma, sigma, alpha_q, &plain_scale);
            let mut chosen = d0;
            if chaos {
                // The writer's own check: collisions left after applying the delta.
                let after = |d: &[C]| -> usize {
                    let mut t: Vec<C> = s.iter().zip(d).map(|(a, x)| a.add(*x)).collect();
                    qb_quantize::quantize(&mut t, BITS);
                    collisions(&readings(&t, &keys, &cre, &cim), &content, gamma, sigma)
                };
                let mut best = (after(&chosen), None::<Vec<C>>);
                for v in 1..=3u64 {
                    let sc: Vec<f64> = (0..n).map(|i| {
                        let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((it as u64) * 8 + v).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (i as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                        eta_mult * (1.0 + 0.5 * (qb_chaos::keyed_unit(k) - 0.5))
                    }).collect();
                    let (dv, _) = pressure_delta(&z, &content, &keys, &codes, &bound, gamma, sigma, alpha_q, &sc);
                    let cv = after(&dv);
                    if cv < best.0 {
                        best = (cv, Some(dv));
                    }
                }
                if let Some(dv) = best.1 {
                    chosen = dv;
                }
            }
            for (a, d) in s.iter_mut().zip(&chosen) {
                *a = a.add(*d);
            }
            log.push(c0);
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
    say(format!("ASTRO-1R: degeneracy pressure, each push divided by the item's collisions; 2-bit store D {D}; N {NS:?}"));
    let p = |g: f64| Arm::Pressure { gamma: g, eta_mult: 1.0, chaos: false };
    let mut dev_arms = vec![Arm::Plain];
    dev_arms.extend(GAMMAS.iter().map(|&g| p(g)));
    let dv = grid(&dev_arms, &dev);
    for (a, v) in dev_arms.iter().zip(&dv) {
        say(row(&format!("dev {a:?}"), v));
    }
    let gi = (0..GAMMAS.len()).max_by(|&a, &b| mean(&dv[1 + a]).partial_cmp(&mean(&dv[1 + b])).unwrap()).unwrap();
    let gamma = GAMMAS[gi];
    say(format!("  gamma chosen on dev: {gamma}"));
    let law = p(gamma);
    let tr = grid(&[Arm::Plain, law], &test);
    say(row("test plain", &tr[0]));
    say(row("test pressure (law)", &tr[1]));
    let logs: Vec<Vec<usize>> = run(&NS.iter().map(|&n| (law, n, 10u64)).collect::<Vec<_>>()).into_iter().map(|r| r.1).collect();
    say(format!("  collisions per iteration, seed 10 (N {NS:?}): {logs:?}"));
    let falls = logs.iter().all(|l| l.windows(2).all(|w| w[1] <= w[0]));
    // Chaos at N 512 and amplification (eta x 2) at every N.
    let ch = Arm::Pressure { gamma, eta_mult: 1.0, chaos: true };
    let am = Arm::Pressure { gamma, eta_mult: 2.0, chaos: false };
    let i512 = NS.iter().position(|&x| x == 512).unwrap();
    let cd = |a: Arm, seeds: &[u64]| run(&seeds.iter().map(|&s| (a, 512usize, s)).collect::<Vec<_>>()).iter().map(|r| r.0).sum::<f64>() / seeds.len() as f64;
    let (dl, dc) = (dv[1 + gi][i512], cd(ch, &dev));
    let tc = cd(ch, &test);
    let chaos_kept = dc > dl && tc > tr[1][i512];
    say(format!("  chaos (N 512): dev {dl:.3} -> {dc:.3}; test {:.3} -> {tc:.3}; kept {chaos_kept}", tr[1][i512]));
    let da = grid(&[am], &dev);
    let ta = grid(&[am], &test);
    let amp_kept = mean(&da[0]) > mean(&dv[1 + gi]) && mean(&ta[0]) > mean(&tr[1]);
    say(row("dev amplification (eta x 2)", &da[0]));
    say(row("test amplification (eta x 2)", &ta[0]));
    let i768 = NS.iter().position(|&x| x == 768).unwrap();
    let v = [
        (format!("R1a collisions fall each iteration: {logs:?}"), falls),
        (format!("R1b pressure >= plain + 0.02 at N 512 ({:.3} vs {:.3}) and 768 ({:.3} vs {:.3})", tr[1][i512], tr[0][i512], tr[1][i768], tr[0][i768]), tr[1][i512] >= tr[0][i512] + 0.02 && tr[1][i768] >= tr[0][i768] + 0.02),
        ("R1c chaos not kept".to_string(), !chaos_kept),
        ("R1d amplification not kept".to_string(), !amp_kept),
    ];
    say(String::new());
    for (nm, ok) in &v {
        say(format!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
