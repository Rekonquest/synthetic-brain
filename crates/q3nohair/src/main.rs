//! Q3 (docs/PHYS2-POC.md): no-hair forgetting as consolidation. 256 old memories (key -> byte); each is
//! then replaced by key -> its class (top 4 bits) at amplitude a; 256 new memories follow. Plain keeps
//! the old memories whole. Chaos jitters a per old item (selected by the new memories readback);
//! amplification doubles a. Every setting is here.

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

fn add(s: &mut [C], v: &[C], k: f64) {
    for (a, b) in s.iter_mut().zip(v) {
        *a = a.add(b.scale(k));
    }
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

fn keyed(seed: u64, a: u64, b: u64) -> f64 {
    qb_chaos::keyed_unit(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ a.wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ b.wrapping_mul(0x1656_67B1_9E37_79F9))
}

const M: usize = 256;
const OLD: usize = 256;
const NEW: usize = 256;
const A: f64 = 0.25;

/// (new recall, old class recall) for consolidation amplitude `a` (None = plain), chaos variant v.
fn run(a: Option<f64>, variant: u64, seed: u64) -> (f64, f64) {
    let codes: Vec<Vec<C>> = (0..M).map(|v| recipe(seed, 3, v)).collect();
    let classes: Vec<Vec<C>> = (0..16).map(|c| recipe(seed, 5, c)).collect();
    let mut r = Rng(seed ^ 0x01D);
    let vals: Vec<usize> = (0..OLD + NEW).map(|_| (r.next() % 256) as usize).collect();
    let keys: Vec<Vec<C>> = (0..OLD + NEW).map(|i| recipe(seed, 1, i)).collect();
    let mut s = vec![C::new(0.0, 0.0); D];
    for i in 0..OLD {
        add(&mut s, &qb_bind::bind(&keys[i], &codes[vals[i]]), 1.0);
    }
    if let Some(a) = a {
        for i in 0..OLD {
            add(&mut s, &qb_bind::bind(&keys[i], &codes[vals[i]]), -1.0);
            let ai = if variant == 0 { a } else { a * (1.0 + 0.5 * (keyed(seed, variant, i as u64) - 0.5)) };
            add(&mut s, &qb_bind::bind(&keys[i], &classes[vals[i] >> 4]), ai);
        }
    }
    for i in OLD..OLD + NEW {
        add(&mut s, &qb_bind::bind(&keys[i], &codes[vals[i]]), 1.0);
    }
    let q = quantized(&s);
    let new_ok = (OLD..OLD + NEW).filter(|&i| read(&qb_bind::bind(&q, &conj(&keys[i])), &codes).0 == vals[i]).count();
    let old_ok = (0..OLD)
        .filter(|&i| {
            let u = qb_bind::bind(&q, &conj(&keys[i]));
            if a.is_some() { read(&u, &classes).0 == vals[i] >> 4 } else { read(&u, &codes).0 >> 4 == vals[i] >> 4 }
        })
        .count();
    (new_ok as f64 / NEW as f64, old_ok as f64 / OLD as f64)
}

fn mean2(v: &[(f64, f64)]) -> (f64, f64) {
    let k = v.len() as f64;
    (v.iter().map(|x| x.0).sum::<f64>() / k, v.iter().map(|x| x.1).sum::<f64>() / k)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("Q3 no-hair consolidation: {OLD} old then {NEW} new memories in a 2-bit store (D {D}); old reduced to their class at amplitude {A}");
    let p = mean2(&par(10..18, |s| run(None, 0, s)));
    let c = mean2(&par(10..18, |s| run(Some(A), 0, s)));
    println!("  plain: new recall {:.3}, old class recall {:.3}", p.0, p.1);
    println!("  no-hair: new recall {:.3}, old class recall {:.3}", c.0, c.1);
    // Chaos: 3 variants of per-item a, the new memories' readback (dev) selects; then test.
    let dbase = mean2(&par(0..4, |s| run(Some(A), 0, s)));
    let mut best = (dbase.0, 0u64);
    for v in 1..=3 {
        let d = mean2(&par(0..4, |s| run(Some(A), v, s)));
        if d.0 > best.0 {
            best = (d.0, v);
        }
    }
    let tch = mean2(&par(10..18, |s| run(Some(A), best.1, s)));
    let chaos_kept = best.1 > 0 && tch.0 > c.0;
    println!("  chaos on a: dev best variant {} ({:.3} vs {:.3}); test {:.3}/{:.3}; kept {chaos_kept}", best.1, best.0, dbase.0, tch.0, tch.1);
    let am = mean2(&par(10..18, |s| run(Some(2.0 * A), 0, s)));
    println!("  amplification a x 2: new {:.3}, old class {:.3}", am.0, am.1);
    let v = [
        (format!("N1 new recall rises >= 0.1 ({:.3} vs {:.3})", c.0, p.0), c.0 >= p.0 + 0.1),
        (format!("N2 old class recall >= 0.3 ({:.3})", c.1), c.1 >= 0.3),
        ("N3 chaos not kept".to_string(), !chaos_kept),
        (format!("N4 a x 2 trades: new down >= 0.03 ({:+.3}), old class up >= 0.03 ({:+.3})", am.0 - c.0, am.1 - c.1), am.0 <= c.0 - 0.03 && am.1 >= c.1 + 0.03),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
