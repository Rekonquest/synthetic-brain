//! Q4 (docs/PHYS2-POC.md): holographic recovery. The 2-bit association store with its D amplitudes cut
//! into 4 regions. Holographic: every memory over all D. Local: memory i only in region i mod 4. Then
//! one region is erased (random phases). Amplification: N 64. Every setting is here.

use qb_amplitude::C;
use std::f64::consts::PI;

const D: usize = 4096;
const BITS: u32 = 2;

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
    (0..D).map(|_| C::polar(1.0, 2.0 * PI * r.unit())).collect()
}

fn recipe(seed: u64, kind: u64, i: usize) -> Vec<C> {
    phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ kind.wrapping_mul(0x6E7))
}

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}


fn quantized(s: &[C]) -> Vec<C> {
    let mut q = s.to_vec();
    qb_quantize::quantize(&mut q, BITS);
    q
}

/// (best index, margin) of u read against a codebook, phase-locked.
fn read(u: &[C], codes: &[Vec<C>]) -> (usize, f64) {
    let mut sc: Vec<(f64, usize)> = codes.iter().enumerate().map(|(v, c)| (qb_overlap_phase::overlap_phase(c, u), v)).collect();
    sc.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    (sc[0].1, sc[0].0 - sc[1].0)
}

fn par<T: Send>(seeds: std::ops::Range<u64>, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}


const M: usize = 256;
const REG: usize = 4;

/// Recall accuracy for (holographic?, n, damaged?).
fn run(holo: bool, n: usize, damage: bool, seed: u64) -> f64 {
    let codes: Vec<Vec<C>> = (0..M).map(|v| recipe(seed, 3, v)).collect();
    let mut r = Rng(seed ^ 0x401);
    let vals: Vec<usize> = (0..n).map(|_| (r.next() % 256) as usize).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| recipe(seed, 1, i)).collect();
    let w = D / REG;
    let mask = |i: usize, k: usize| holo || k / w == i % REG;
    let mut s = vec![C::new(0.0, 0.0); D];
    for i in 0..n {
        let b = qb_bind::bind(&keys[i], &codes[vals[i]]);
        for k in 0..D {
            if mask(i, k) {
                s[k] = s[k].add(b[k]);
            }
        }
    }
    let mut q = quantized(&s);
    if damage {
        for c in q.iter_mut().take(w) {
            *c = C::polar(1.0, 2.0 * PI * r.unit());
        }
    }
    (0..n)
        .filter(|&i| {
            let cue: Vec<C> = keys[i].iter().enumerate().map(|(k, c)| if mask(i, k) { *c } else { C::new(0.0, 0.0) }).collect();
            read(&qb_bind::bind(&q, &conj(&cue)), &codes).0 == vals[i]
        })
        .count() as f64
        / n as f64
}

fn m(f: impl Fn(u64) -> f64 + Sync) -> f64 {
    let v = par(10..18, f);
    v.iter().sum::<f64>() / v.len() as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("Q4 holographic recovery: 2-bit store (D {D}) in {REG} regions; one region erased");
    let mut res = std::collections::HashMap::new();
    for &n in &[128usize, 64] {
        for holo in [true, false] {
            for damage in [false, true] {
                let a = m(|s| run(holo, n, damage, s));
                println!("  N {n:3} {:<12} {:<9}: {a:.3}", if holo { "holographic" } else { "local" }, if damage { "damaged" } else { "intact" });
                res.insert((n, holo, damage), a);
            }
        }
    }
    let g = |n: usize, h: bool, d: bool| res[&(n, h, d)];
    let v = [
        (format!("H1 N 128 damaged: holographic >= 0.95 ({:.3}), local <= 0.78 ({:.3})", g(128, true, true), g(128, false, true)), g(128, true, true) >= 0.95 && g(128, false, true) <= 0.78),
        (format!("H2 N 128 intact: the layouts tie +-0.02 ({:.3} vs {:.3})", g(128, true, false), g(128, false, false)), (g(128, true, false) - g(128, false, false)).abs() <= 0.02),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
