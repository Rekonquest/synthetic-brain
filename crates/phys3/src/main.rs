//! PHYS-3 orchestrator (docs/PHYS-POC.md): topological charge. 64 rings of 32 phases, local noise
//! and XY relaxation each step. A bit held as the ring's winding (topological) against the same bit
//! held as its mean phase (local). Then chaos on J (per step, per ring), amplification (J × 2), and
//! D3: counting events as windings against an analog accumulator. Every setting is here.

use std::f64::consts::PI;

const RINGS: usize = 64;
const L: usize = 32;
const J: f64 = 0.2;
const T: usize = 20_000;
const SIGMAS: [f64; 5] = [0.05, 0.1, 0.2, 0.5, 1.0];

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

#[derive(Clone, Copy)]
struct Arm {
    topological: bool,
    j_mult: f64,
    chaos: bool,
}

fn step(theta: &mut [f64], sigma: f64, j: f64, r: &mut Rng) {
    for x in theta.iter_mut() {
        *x += sigma * r.normal();
    }
    xy_relax::relax(theta, j);
}

fn j_at(arm: Arm, seed: u64, ring: usize, t: usize) -> f64 {
    let j = J * arm.j_mult;
    if !arm.chaos {
        return j;
    }
    let key = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (ring as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (t as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    j * (1.0 + 0.5 * (qb_chaos::keyed_unit(key) - 0.5))
}

/// Fraction of bits kept after T steps.
fn retention(arm: Arm, sigma: f64, seed: u64) -> f64 {
    let mut r = Rng(seed);
    let mut ok = 0;
    for ring in 0..RINGS {
        let bit = (r.next() & 1) as i64;
        let mut theta: Vec<f64> = (0..L).map(|i| if arm.topological { 2.0 * PI * bit as f64 * i as f64 / L as f64 } else { PI * bit as f64 }).collect();
        for t in 0..T {
            step(&mut theta, sigma, j_at(arm, seed, ring, t), &mut r);
        }
        let read = if arm.topological {
            winding::winding(&theta)
        } else {
            let (s, c): (f64, f64) = theta.iter().fold((0.0, 0.0), |a, x| (a.0 + x.sin(), a.1 + x.cos()));
            (s.atan2(c).cos() < 0.0) as i64
        };
        ok += (read == bit) as usize;
    }
    ok as f64 / RINGS as f64
}

/// D3: count `events` twists, each added smoothly over 100 steps, spaced evenly over T. Returns
/// (winding counter exact, analog counter exact).
fn counting(events: usize, sigma: f64, seed: u64) -> (f64, f64) {
    let mut r = Rng(seed ^ 0xC0DE);
    let gap = T / events;
    let (mut okw, mut oka) = (0, 0);
    for _ in 0..RINGS {
        let mut theta = vec![0.0; L];
        let mut phi = 0.0f64;
        for t in 0..T {
            let ramp = if std::env::var_os("INSTANT").is_some() { 1 } else { 100 };
            if t % gap < ramp && t / gap < events {
                for (i, x) in theta.iter_mut().enumerate() {
                    *x += 2.0 * PI * i as f64 / L as f64 / ramp as f64;
                }
                phi += 2.0 * PI / ramp as f64;
            }
            step(&mut theta, sigma, J, &mut r);
            phi += sigma * r.normal();
        }
        okw += (winding::winding(&theta) == events as i64) as usize;
        oka += ((phi / (2.0 * PI)).round() as i64 == events as i64) as usize;
    }
    (okw as f64 / RINGS as f64, oka as f64 / RINGS as f64)
}

fn mean(seeds: std::ops::Range<u64>, f: impl Fn(u64) -> f64 + Sync) -> f64 {
    let v: Vec<f64> = std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    v.iter().sum::<f64>() / v.len() as f64
}

fn main() {
    if std::env::var_os("INSTANT").is_some() {
        println!("D3 check (added after the run, labelled): each event added in ONE step (a deliberate phase slip), sigma 0.1");
        for ev in [1usize, 2, 3, 5, 10] {
            let (w, a) = (mean(10..18, |sd| counting(ev, 0.1, sd).0), mean(10..18, |sd| counting(ev, 0.1, sd).1));
            println!("  {ev} events: winding exact {w:.3}, analog exact {a:.3}");
        }
        return;
    }
    let t0 = std::time::Instant::now();
    println!("PHYS-3: {RINGS} rings of {L} phases, J {J}, T {T}; bits kept (test seeds 10-17)");
    let topo = Arm { topological: true, j_mult: 1.0, chaos: false };
    let local = Arm { topological: false, ..topo };
    let mut tk = Vec::new();
    let mut lk = Vec::new();
    for &s in &SIGMAS {
        let (a, b) = (mean(10..18, |sd| retention(topo, s, sd)), mean(10..18, |sd| retention(local, s, sd)));
        println!("  sigma {s:.2}: topological {a:.3}, local {b:.3}");
        tk.push(a);
        lk.push(b);
    }
    let mut chaos_kept = false;
    let mut amp_any = false;
    let mut amp_hi = true;
    for &s in &[0.2, 0.5] {
        let (dp, dc, da) = (mean(0..4, |sd| retention(topo, s, sd)), mean(0..4, |sd| retention(Arm { chaos: true, ..topo }, s, sd)), mean(0..4, |sd| retention(Arm { j_mult: 2.0, ..topo }, s, sd)));
        let (tp, tc, ta) = (mean(10..18, |sd| retention(topo, s, sd)), mean(10..18, |sd| retention(Arm { chaos: true, ..topo }, s, sd)), mean(10..18, |sd| retention(Arm { j_mult: 2.0, ..topo }, s, sd)));
        let (ck, ak) = (dc > dp && tc > tp, da > dp && ta > tp);
        chaos_kept |= ck;
        amp_any |= ak;
        amp_hi &= ak;
        println!("  sigma {s:.1}: chaos on J dev {dp:.3} -> {dc:.3}, test {tp:.3} -> {tc:.3} (kept {ck}); J x 2 dev {dp:.3} -> {da:.3}, test {tp:.3} -> {ta:.3} (kept {ak})");
    }
    let d3 = (mean(10..18, |sd| counting(50, 0.1, sd).0), mean(10..18, |sd| counting(50, 0.1, sd).1));
    println!("  D3 counting 50 events at sigma 0.1 (as precommitted; a 32-site ring holds a winding of at most 15): winding exact {:.3}, analog exact {:.3}", d3.0, d3.1);
    let d3b = (mean(10..18, |sd| counting(10, 0.1, sd).0), mean(10..18, |sd| counting(10, 0.1, sd).1));
    println!("  D3 check (added, labelled): 10 events: winding exact {:.3}, analog exact {:.3}", d3b.0, d3b.1);
    let p2 = tk[..3].iter().all(|&x| x >= 0.95) && (tk[4] - 0.5).abs() <= 0.15 && (lk[4] - 0.5).abs() <= 0.15;
    let v = [
        (format!("P1 sigma 0.1: topological >= 0.99 ({:.3}), local <= 0.75 ({:.3})", tk[1], lk[1]), tk[1] >= 0.99 && lk[1] <= 0.75),
        ("P2 heat melts it past a threshold: >= 0.95 for sigma <= 0.2, both near 0.5 at 1.0".into(), p2),
        ("C1 chaos on J not kept".into(), !chaos_kept),
        ("A1 J x 2 kept at sigma >= 0.2".into(), amp_hi),
        (format!("D3 counting 50: winding >= 0.95 ({:.3}), analog <= 0.6 ({:.3})", d3.0, d3.1), d3.0 >= 0.95 && d3.1 <= 0.6),
    ];
    let _ = amp_any;
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
