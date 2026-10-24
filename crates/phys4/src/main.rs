//! PHYS-4 orchestrator (docs/PHYS-POC.md): a closed string's identity is its vibration. In the 1D
//! periodic wave medium, 32 bits held as standing-mode amplitudes (each read from its own mode
//! against a noiseless reference string, the clock) against 32 bits held as displacements at sites.
//! Then chaos (frozen ±1% disorder in c²), amplification (A × 2), and D4: 8 messages sent at once on
//! 8 harmonics, read by one listener, against the same bits time-shared on one harmonic.

use std::f64::consts::PI;
use thermo_lattice::Lattice;
use wave_lattice::Wave;

const L: usize = 128;
const DT: f64 = 0.5;
const T: usize = 10_000;
const BITS: usize = 32;
const STRINGS: usize = 8;
const SIGMAS: [f64; 3] = [0.0, 0.001, 0.01];

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

fn medium(disorder: bool, seed: u64) -> Vec<f64> {
    let mut r = Rng(seed ^ 0xD15);
    (0..L).map(|_| if disorder { 1.0 + 0.02 * (r.unit() - 0.5) } else { 1.0 }).collect()
}

fn cosmode(n: usize, x: usize) -> f64 {
    (2.0 * PI * n as f64 * x as f64 / L as f64).cos()
}

/// Fraction of bits kept: (mode memory, position memory).
fn run(sigma: f64, amp: f64, disorder: bool, seed: u64) -> (f64, f64) {
    let lat = Lattice::new(&[L]);
    let c2 = medium(disorder, seed);
    let zero = vec![0.0; L];
    let mut r = Rng(seed);
    let (mut okm, mut okp) = (0, 0);
    for _ in 0..STRINGS {
        let bits: Vec<f64> = (0..BITS).map(|_| if r.next() & 1 == 1 { 1.0 } else { -1.0 }).collect();
        let (mut m, mut rf, mut p) = (Wave::new(&lat), Wave::new(&lat), Wave::new(&lat));
        for x in 0..L {
            m.u[x] = (1..=BITS).map(|n| bits[n - 1] * amp * cosmode(n, x)).sum();
            rf.u[x] = (1..=BITS).map(|n| amp * cosmode(n, x)).sum();
        }
        for k in 0..BITS {
            p.u[4 * k] = bits[k] * amp;
        }
        for _ in 0..T {
            for w in [&mut m, &mut rf, &mut p] {
                w.kick(DT, &c2, &zero, &zero);
                w.drift(DT);
            }
            if sigma > 0.0 {
                for x in 0..L {
                    m.v[x] += sigma * r.normal();
                    p.v[x] += sigma * r.normal();
                }
            }
        }
        for n in 1..=BITS {
            let w2 = 4.0 * (PI * n as f64 / L as f64).sin().powi(2);
            let score = w2 * wave_mode::project(&m.u, n) * wave_mode::project(&rf.u, n) + wave_mode::project(&m.v, n) * wave_mode::project(&rf.v, n);
            okm += ((score > 0.0) == (bits[n - 1] > 0.0)) as usize;
        }
        for k in 0..BITS {
            okp += ((p.u[4 * k] > 0.0) == (bits[k] > 0.0)) as usize;
        }
    }
    let tot = (STRINGS * BITS) as f64;
    (okm as f64 / tot, okp as f64 / tot)
}

/// D4: lanes at harmonics `ns`, `symbols` symbols each, W steps per symbol, drive at site 0, listener
/// at site L/2, damping γ; decoded per lane by lock-in over each window's second half against a
/// reference run driven with all +1 symbols. Returns the fraction of bits right.
fn lanes(ns: &[usize], symbols: usize, seed: u64) -> f64 {
    const W: usize = 400;
    const GAMMA: f64 = 0.05;
    let lat = Lattice::new(&[L]);
    let c2 = vec![1.0; L];
    let gamma = vec![GAMMA; L];
    let mut r = Rng(seed ^ 0x1A2E);
    let msgs: Vec<Vec<f64>> = ns.iter().map(|_| (0..symbols).map(|_| if r.next() & 1 == 1 { 1.0 } else { -1.0 }).collect()).collect();
    let omega: Vec<f64> = ns.iter().map(|&n| 2.0 * (PI * n as f64 / L as f64).sin()).collect();
    let listen = |signs: &dyn Fn(usize, usize) -> f64| -> Vec<Vec<(f64, f64)>> {
        let mut w = Wave::new(&lat);
        let mut out = vec![vec![(0.0, 0.0); symbols]; ns.len()];
        for t in 0..symbols * W {
            let (k, time) = (t / W, t as f64 * DT);
            let mut force = vec![0.0; L];
            force[0] = (0..ns.len()).map(|l| signs(l, k) * (omega[l] * time).sin()).sum();
            w.kick(DT, &c2, &gamma, &force);
            w.drift(DT);
            if t % W >= W / 2 {
                let y = w.u[L / 2];
                for l in 0..ns.len() {
                    out[l][k].0 += y * (omega[l] * time).sin();
                    out[l][k].1 += y * (omega[l] * time).cos();
                }
            }
        }
        out
    };
    let rx = listen(&|l, k| msgs[l][k]);
    let rf = listen(&|_, _| 1.0);
    let mut ok = 0;
    for l in 0..ns.len() {
        for k in 0..symbols {
            let s = rx[l][k].0 * rf[l][k].0 + rx[l][k].1 * rf[l][k].1;
            ok += ((s > 0.0) == (msgs[l][k] > 0.0)) as usize;
        }
    }
    ok as f64 / (ns.len() * symbols) as f64
}

fn mean(seeds: std::ops::Range<u64>, f: impl Fn(u64) -> (f64, f64) + Sync) -> (f64, f64) {
    let v: Vec<(f64, f64)> = std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let n = v.len() as f64;
    (v.iter().map(|x| x.0).sum::<f64>() / n, v.iter().map(|x| x.1).sum::<f64>() / n)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS-4: a closed string (L {L}, c2 1, lossless, dt {DT}), {BITS} bits per string, T {T}; test seeds 10-17");
    let mut res = Vec::new();
    for &s in &SIGMAS {
        let (m, p) = mean(10..18, |sd| run(s, 1.0, false, sd));
        println!("  sigma {s:.3}: mode memory {m:.3}, position memory {p:.3}");
        res.push((m, p));
    }
    let (dp, dc) = (mean(0..4, |sd| run(0.001, 1.0, false, sd)).0, mean(0..4, |sd| run(0.001, 1.0, true, sd)).0);
    let tc = mean(10..18, |sd| run(0.001, 1.0, true, sd)).0;
    let chaos_hurts = tc < res[1].0;
    println!("  chaos, frozen +-1% c2 disorder (sigma 0.001): dev {dp:.3} -> {dc:.3}; test {:.3} -> {tc:.3}", res[1].0);
    let (dpa, da) = (mean(0..4, |sd| run(0.01, 1.0, false, sd)).0, mean(0..4, |sd| run(0.01, 2.0, false, sd)).0);
    let ta = mean(10..18, |sd| run(0.01, 2.0, false, sd)).0;
    let amp_kept = da > dpa && ta > res[2].0;
    println!("  amplification A x 2 (sigma 0.01): dev {dpa:.3} -> {da:.3}; test {:.3} -> {ta:.3}; kept {amp_kept}", res[2].0);
    let par: Vec<usize> = (1..=8).map(|i| 4 * i).collect();
    let d4 = mean(10..18, |sd| (lanes(&par, 64, sd), 0.0)).0;
    let ts = mean(10..18, |sd| (lanes(&[16], 512, sd), 0.0)).0;
    println!("  D4 parallel lanes: 8 harmonics x 64 symbols at once: {d4:.3} of bits right; time-shared on one harmonic (8x the time): {ts:.3}");
    let v = [
        (format!("S1 mode >= 0.99 at sigma <= 0.001 ({:.3}, {:.3}); position about 0.5 ({:.3}, {:.3}, {:.3})", res[0].0, res[1].0, res[0].1, res[1].1, res[2].1), res[0].0 >= 0.99 && res[1].0 >= 0.99 && res.iter().all(|x| (x.1 - 0.5).abs() <= 0.1)),
        (format!("S2 mode >= 0.9 at sigma 0.01 ({:.3})", res[2].0), res[2].0 >= 0.9),
        ("C1 frozen chaos harmful".into(), chaos_hurts),
        ("A1 A x 2 kept at sigma 0.01".into(), amp_kept),
        (format!("D4 8 parallel lanes >= 0.95 ({d4:.3})"), d4 >= 0.95),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
