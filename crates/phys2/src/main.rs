//! PHYS-2 orchestrator (docs/PHYS-POC.md): gauge invariance. FOLD-6's content store (full precision,
//! D 4096) drifts by a smooth local phase φ_i (a random walk along i, step σ). The plain reading
//! fails when φ spans many turns; the Wilson-line reading sees only φ_i − φ_{i+1}. Then chaos on the
//! cue (selected by the reading's margin), amplification (lags 1 + 2), and D2: invariant perception
//! of 64 phasor shapes through an unknown smooth channel. Every setting is here.

use qb_amplitude::C;
use std::f64::consts::PI;

const D: usize = 4096;
const M: usize = 256;
const SIGMAS: [f64; 4] = [0.0, 0.02, 0.1, 0.3];
const NS: [usize; 2] = [8, 16];

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
    fn normal(&mut self) -> f64 {
        (-2.0 * self.unit().max(1e-300).ln()).sqrt() * (2.0 * PI * self.unit()).cos()
    }
}

fn phasors(seed: u64, d: usize) -> Vec<C> {
    let mut r = Rng(seed);
    (0..d).map(|_| C::polar(1.0, 2.0 * PI * r.unit())).collect()
}

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

fn drift(d: usize, sigma: f64, seed: u64) -> Vec<C> {
    let mut r = Rng(seed ^ 0x6A06E);
    let start = 2.0 * PI * r.unit();
    let mut phi = if std::env::var_os("PHI0_ZERO").is_some() { 0.0 } else { start };
    (0..d).map(|_| {
        phi += sigma * r.normal();
        C::polar(1.0, phi)
    }).collect()
}

/// Wilson score, with amplification adding the lag-2 lines (even and odd sub-sequences).
fn wilson_score(w: &[C], amp: bool) -> f64 {
    let base = qb_wilson::wilson(w);
    if !amp {
        return base;
    }
    let even: Vec<C> = w.iter().step_by(2).cloned().collect();
    let odd: Vec<C> = w.iter().skip(1).step_by(2).cloned().collect();
    base + 0.5 * (qb_wilson::wilson(&even) + qb_wilson::wilson(&odd))
}

fn best(sc: &[f64]) -> (usize, f64) {
    let mut idx: Vec<usize> = (0..sc.len()).collect();
    idx.sort_by(|&a, &b| sc[b].partial_cmp(&sc[a]).unwrap());
    (idx[0], sc[idx[0]] - sc[idx[1]])
}

/// (plain accuracy, Wilson accuracy) for one store; `chaos` jitters the cue, selected by margin.
fn store_run(n: usize, sigma: f64, seed: u64, amp: bool, chaos: bool) -> (f64, f64) {
    let codes: Vec<Vec<C>> = (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ 0xC0D, D)).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ 0x6E7, D)).collect();
    let mut r = Rng(seed ^ 0xB17E5);
    let vals: Vec<usize> = (0..n).map(|_| (r.next() % 256) as usize).collect();
    let mut s = vec![C::new(0.0, 0.0); D];
    for i in 0..n {
        for (a, b) in s.iter_mut().zip(qb_bind::bind(&keys[i], &codes[vals[i]])) {
            *a = a.add(b);
        }
    }
    let g = drift(D, sigma, seed);
    let s: Vec<C> = s.iter().zip(&g).map(|(a, b)| a.mul(*b)).collect();
    let (mut okp, mut okw) = (0, 0);
    for i in 0..n {
        let read = |key: &[C]| -> ((usize, f64), (usize, f64)) {
            let u = qb_bind::bind(&s, &conj(key));
            let p: Vec<f64> = codes.iter().map(|c| qb_overlap_phase::overlap_phase(c, &u)).collect();
            let w: Vec<f64> = codes.iter().map(|c| wilson_score(&qb_bind::bind(&u, &conj(c)), amp)).collect();
            (best(&p), best(&w))
        };
        let (p, mut w) = read(&keys[i]);
        if chaos {
            for v in 1..=3u64 {
                let cue: Vec<C> = keys[i].iter().enumerate().map(|(k, c)| {
                    let key = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((i as u64) * 8 + v).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (k as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                    c.mul(C::polar(1.0, 0.2 * (qb_chaos::keyed_unit(key) - 0.5)))
                }).collect();
                let (_, wv) = read(&cue);
                if wv.1 > w.1 {
                    w = wv;
                }
            }
        }
        okp += (p.0 == vals[i]) as usize;
        okw += (w.0 == vals[i]) as usize;
    }
    (okp as f64 / n as f64, okw as f64 / n as f64)
}

fn mean_over(seeds: std::ops::Range<u64>, f: impl Fn(u64) -> (f64, f64) + Sync) -> (f64, f64) {
    let v: Vec<(f64, f64)> = std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let n = v.len() as f64;
    (v.iter().map(|x| x.0).sum::<f64>() / n, v.iter().map(|x| x.1).sum::<f64>() / n)
}

/// D2: 64 phasor shapes of length 512 through a smooth channel; (plain, Wilson) accuracy.
fn perception(sigma: f64, seed: u64) -> (f64, f64) {
    let lib: Vec<Vec<C>> = (0..64).map(|k| phasors(seed.wrapping_mul(0x5CA4_F01D) ^ k as u64, 512)).collect();
    let (mut okp, mut okw) = (0, 0);
    for j in 0..64 {
        let g = drift(512, sigma, seed ^ (j as u64 + 1) << 16);
        let rx: Vec<C> = lib[j].iter().zip(&g).map(|(a, b)| a.mul(*b)).collect();
        let p: Vec<f64> = lib.iter().map(|l| qb_overlap_phase::overlap_phase(l, &rx)).collect();
        let w: Vec<f64> = lib.iter().map(|l| qb_wilson::wilson(&qb_bind::bind(&rx, &conj(l)))).collect();
        okp += (best(&p).0 == j) as usize;
        okw += (best(&w).0 == j) as usize;
    }
    (okp as f64 / 64.0, okw as f64 / 64.0)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS-2: gauge invariance, FOLD-6's store (D {D}, f64) under a smooth local phase drift; test seeds 10-17");
    let mut g = std::collections::HashMap::new();
    for &n in &NS {
        for &s in &SIGMAS {
            let (p, w) = mean_over(10..18, |sd| store_run(n, s, sd, false, false));
            println!("  N {n:2} sigma {s:.2}: plain {p:.3}, Wilson {w:.3}");
            g.insert((n, (s * 100.0) as i64), (p, w));
        }
    }
    let at = |n: usize, s: f64| g[&(n, (s * 100.0) as i64)];
    let (dp, dc) = (mean_over(0..4, |sd| store_run(8, 0.1, sd, false, false)).1, mean_over(0..4, |sd| store_run(8, 0.1, sd, false, true)).1);
    let (tp, tc) = (at(8, 0.1).1, mean_over(10..18, |sd| store_run(8, 0.1, sd, false, true)).1);
    let chaos_kept = dc > dp && tc > tp;
    println!("  chaos on the cue (N 8, sigma 0.1): dev {dp:.3} -> {dc:.3}; test {tp:.3} -> {tc:.3}; kept {chaos_kept}");
    let (da, ta) = (mean_over(0..4, |sd| store_run(16, 0.1, sd, true, false)).1, mean_over(10..18, |sd| store_run(16, 0.1, sd, true, false)).1);
    let dpl = mean_over(0..4, |sd| store_run(16, 0.1, sd, false, false)).1;
    let amp_kept = da > dpl && ta > at(16, 0.1).1;
    println!("  amplification, lags 1 + 2 (N 16, sigma 0.1): dev {dpl:.3} -> {da:.3}; test {:.3} -> {ta:.3}; kept {amp_kept}", at(16, 0.1).1);
    println!("  D2 invariant perception (64 shapes, length 512):");
    let mut d2 = (0.0, 0.0);
    for &s in &SIGMAS {
        let (p, w) = mean_over(10..18, |sd| perception(s, sd));
        println!("    sigma {s:.2}: plain {p:.3}, Wilson {w:.3}");
        if s == 0.1 {
            d2 = (p, w);
        }
    }
    let v = [
        (format!("G1 no drift: plain 1.000 ({:.3}), Wilson >= 0.95 at N 8 ({:.3}), >= 0.7 at N 16 ({:.3})", at(8, 0.0).0, at(8, 0.0).1, at(16, 0.0).1), at(8, 0.0).0 >= 0.999 && at(8, 0.0).1 >= 0.95 && at(16, 0.0).1 >= 0.7),
        (format!("G2 sigma >= 0.1: plain <= 0.05 ({:.3}, {:.3})", at(8, 0.1).0, at(8, 0.3).0), at(8, 0.1).0 <= 0.05 && at(8, 0.3).0 <= 0.05 && at(16, 0.1).0 <= 0.05 && at(16, 0.3).0 <= 0.05),
        (format!("G3 Wilson N 8: >= 0.95 at 0.1 ({:.3}), >= 0.9 at 0.3 ({:.3})", at(8, 0.1).1, at(8, 0.3).1), at(8, 0.1).1 >= 0.95 && at(8, 0.3).1 >= 0.9),
        ("C1 chaos not kept".into(), !chaos_kept),
        ("A1 amplification (lags 1 + 2) not kept".into(), !amp_kept),
        (format!("D2 invariant perception at sigma 0.1: Wilson >= 0.95 ({:.3}), plain <= 0.1 ({:.3})", d2.1, d2.0), d2.1 >= 0.95 && d2.0 <= 0.1),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
