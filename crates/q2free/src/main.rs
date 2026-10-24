//! Q2 (docs/PHYS2-POC.md): asymptotic freedom as concept clusters. 16 concepts x 8 members in 2D
//! under noise; each member pulled toward its concept's centre by a law: free inside r0 and a
//! constant pull sigma beyond (the linear potential), a spring of equal force at r0, or a
//! Coulomb-like 1/r^2 of equal force at r0. Chaos jitters sigma per step; amplification doubles it.

use std::f64::consts::PI;

const CONCEPTS: usize = 16;
const MEMBERS: usize = 8;
const SPACING: f64 = 10.0;
const R0: f64 = 1.0;
const SIGMA: f64 = 0.05;
const NOISE: f64 = 0.1;
const T: usize = 20_000;

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

#[derive(Clone, Copy, PartialEq)]
enum Law {
    Free,
    Spring,
    Coulomb,
}

/// (binding: fraction nearest their own centre; RMS distance from own centre of those within 2 r0).
fn run(law: Law, sigma_mult: f64, chaos: bool, seed: u64) -> (f64, f64) {
    let mut r = Rng(seed);
    let centres: Vec<(f64, f64)> = (0..CONCEPTS).map(|c| ((c % 4) as f64 * SPACING, (c / 4) as f64 * SPACING)).collect();
    let mut pts: Vec<(f64, f64, usize)> = (0..CONCEPTS * MEMBERS).map(|i| (centres[i / MEMBERS].0, centres[i / MEMBERS].1, i / MEMBERS)).collect();
    for t in 0..T {
        for (i, p) in pts.iter_mut().enumerate() {
            let (cx, cy) = centres[p.2];
            let (dx, dy) = (cx - p.0, cy - p.1);
            let d = (dx * dx + dy * dy).sqrt().max(1e-9);
            let mut sigma = SIGMA * sigma_mult;
            if chaos {
                let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (i as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                sigma *= 1.0 + 0.5 * (qb_chaos::keyed_unit(k) - 0.5);
            }
            let f = match law {
                Law::Free => if d > R0 { sigma } else { 0.0 },
                Law::Spring => sigma / R0 * d,
                Law::Coulomb => sigma * R0 * R0 / (d * d).max(0.01),
            }
            .min(d);
            p.0 += f * dx / d + NOISE * r.normal();
            p.1 += f * dy / d + NOISE * r.normal();
        }
    }
    let mut bound = 0;
    let (mut s2, mut ns) = (0.0, 0);
    for p in &pts {
        let near = (0..CONCEPTS).min_by(|&a, &b| {
            let da = (centres[a].0 - p.0).powi(2) + (centres[a].1 - p.1).powi(2);
            let db = (centres[b].0 - p.0).powi(2) + (centres[b].1 - p.1).powi(2);
            da.partial_cmp(&db).unwrap()
        }).unwrap();
        bound += (near == p.2) as usize;
        let d2 = (centres[p.2].0 - p.0).powi(2) + (centres[p.2].1 - p.1).powi(2);
        if d2 <= (2.0 * R0).powi(2) {
            s2 += d2;
            ns += 1;
        }
    }
    (bound as f64 / pts.len() as f64, (s2 / ns.max(1) as f64).sqrt())
}

fn m(f: impl Fn(u64) -> (f64, f64) + Sync, seeds: std::ops::Range<u64>) -> (f64, f64) {
    let v: Vec<(f64, f64)> = std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let k = v.len() as f64;
    (v.iter().map(|x| x.0).sum::<f64>() / k, v.iter().map(|x| x.1).sum::<f64>() / k)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("Q2 asymptotic freedom: {CONCEPTS} concepts x {MEMBERS} members, noise {NOISE}, T {T}; binding and inside spread (RMS within 2 r0)");
    let fr = m(|s| run(Law::Free, 1.0, false, s), 10..18);
    let sp = m(|s| run(Law::Spring, 1.0, false, s), 10..18);
    let co = m(|s| run(Law::Coulomb, 1.0, false, s), 10..18);
    println!("  free inside, constant pull outside: binding {:.3}, spread {:.3}", fr.0, fr.1);
    println!("  spring (equal force at r0):         binding {:.3}, spread {:.3}", sp.0, sp.1);
    println!("  Coulomb 1/r^2 (equal force at r0):  binding {:.3}, spread {:.3}", co.0, co.1);
    let (dp, dc) = (m(|s| run(Law::Free, 1.0, false, s), 0..4), m(|s| run(Law::Free, 1.0, true, s), 0..4));
    let tc = m(|s| run(Law::Free, 1.0, true, s), 10..18);
    let better = |a: (f64, f64), b: (f64, f64)| a.0 >= b.0 && a.1 > b.1;
    let chaos_kept = better(dc, dp) && better(tc, fr);
    println!("  chaos on sigma: dev {:.3}/{:.3} -> {:.3}/{:.3}; test -> {:.3}/{:.3}; kept {chaos_kept}", dp.0, dp.1, dc.0, dc.1, tc.0, tc.1);
    let am = m(|s| run(Law::Free, 2.0, false, s), 10..18);
    println!("  amplification sigma x 2: binding {:.3}, spread {:.3}", am.0, am.1);
    let v = [
        (format!("F1 free law binding >= 0.99 ({:.3}), spread >= 1.5x spring ({:.2}x)", fr.0, fr.1 / sp.1), fr.0 >= 0.99 && fr.1 >= 1.5 * sp.1),
        (format!("F2 Coulomb binding <= 0.8 ({:.3})", co.0), co.0 <= 0.8),
        ("F3 chaos not kept".to_string(), !chaos_kept),
        (format!("F4 sigma x 2 tightens without gain (binding {:.3} vs {:.3})", am.0, fr.0), am.0 <= fr.0 + 0.005),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
