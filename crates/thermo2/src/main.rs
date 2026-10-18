//! T2 runner (docs/T2-BATH.md): the bath's own checks B1–B4 and its chaos (B5) and amplification (B6)
//! rounds. Every setting lives in `Settings`.

use qb_chaos::keyed_unit;
use thermo_bath::{damp, exchange, normal};
use thermo_lattice::Lattice;

struct Settings {
    gamma: f64,
    kappa: f64,
    burn: usize,
    measure: usize,
    temps: [f64; 3],
    push: f64,
}

fn settings() -> Settings {
    Settings { gamma: 0.2, kappa: 0.3, burn: 200, measure: 400, temps: [0.5, 1.0, 2.0], push: 0.5 }
}

fn uni(seed: u64, t: usize, a: usize, b: usize) -> f64 {
    keyed_unit(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (a as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ (b as u64 + 7).wrapping_mul(0xD6E8_FEB8_6659_FD93))
}

/// Logistic values are not uniform (arcsine density), so map them to uniform first:
/// u = (2/π)·asin(√x) is uniform for x from the r = 4 logistic map.
fn uniform(seed: u64, t: usize, a: usize, b: usize) -> f64 {
    let x = uni(seed, t, a, b);
    (2.0 / std::f64::consts::PI * x.sqrt().asin()).clamp(1e-12, 1.0 - 1e-12)
}

fn z(seed: u64, t: usize, a: usize, b: usize) -> f64 {
    normal(uniform(seed, t, a, 2 * b), uniform(seed, t, a, 2 * b + 1))
}

/// Run the bath; returns (mean per-site variance, mean at site 0, mean |at the 2d neighbours|).
fn run(dims: &[usize], s: &Settings, temp: f64, gamma_of: &dyn Fn(usize) -> f64, noise_gain: f64, push: f64, seed: u64) -> (f64, f64, f64) {
    let mut l = Lattice::new(dims);
    let h = |site: usize| if site == 0 { push } else { 0.0 };
    let (mut var, mut m0, mut mn) = (0.0, 0.0, 0.0);
    let neighbours: Vec<usize> = (0..dims.len()).map(|a| l.forward(0, a)).collect();
    for t in 0..s.burn + s.measure {
        damp(&mut l, temp, gamma_of, &h, &|site| noise_gain * z(seed, t, site, 0));
        exchange(&mut l, temp, s.kappa, &h, &|site, axis| noise_gain * z(seed, t, site, 1 + axis));
        if t >= s.burn {
            let n = l.e.len() as f64;
            var += l.e.iter().enumerate().map(|(i, x)| (x - h(i)).powi(2)).sum::<f64>() / n / s.measure as f64;
            m0 += l.e[0] / s.measure as f64;
            mn += neighbours.iter().map(|&j| l.e[j]).sum::<f64>() / neighbours.len() as f64 / s.measure as f64;
        }
    }
    (var, m0, mn.abs())
}

fn main() {
    let s = settings();
    println!("T2: the bath (gamma {}, kappa {}); burn {}, measure {}", s.gamma, s.kappa, s.burn, s.measure);
    let shapes: [Vec<usize>; 2] = [vec![32, 32], vec![16, 16, 16]];
    let plain = |_: usize| s.gamma;
    let (mut b1, mut b2) = (true, true);
    for dims in &shapes {
        let mut vars = Vec::new();
        for &t in &s.temps {
            let (v, _, _) = run(dims, &s, t, &plain, 1.0, 0.0, 1);
            println!("d={}: T {t}: variance {v:.4} (error {:+.2}%)", dims.len(), (v / t - 1.0) * 100.0);
            b1 &= (v / t - 1.0).abs() < 0.02;
            vars.push(v);
        }
        b2 &= (vars[2] / vars[1] / 2.0 - 1.0).abs() < 0.03;
    }
    // B3: static FDR, with many seeds averaged for the mean at the pushed site.
    let (mut m0, mut mn) = (0.0, 0.0);
    let seeds = 8;
    for seed in 0..seeds {
        let (_, a, b) = run(&shapes[1], &s, 1.0, &plain, 1.0, s.push, 100 + seed);
        m0 += a / seeds as f64;
        mn += b / seeds as f64;
    }
    let b3 = (m0 / s.push - 1.0).abs() < 0.05 && mn < 0.05;
    println!("B3: push h {} at one site (d=3, T 1): mean there {m0:.4} (h*C00/T = {}), neighbours |mean| {mn:.4}", s.push, s.push);
    // B4: cold, undamped conservation.
    let mut l = Lattice::new(&shapes[1]);
    for (i, x) in l.e.iter_mut().enumerate() {
        *x = (i as f64 * 0.11).cos();
    }
    let t0 = l.total();
    for _ in 0..200 {
        exchange(&mut l, 0.0, s.kappa, &|_| 0.0, &|_, _| 0.0);
    }
    let b4 = (l.total() - t0).abs() < 1e-9;
    println!("B4: cold undamped drift {:.1e}", (l.total() - t0).abs());
    // Rounds, check = B1 equipartition error at T 1, d=3 (lower is better).
    let err = |gamma_of: &dyn Fn(usize) -> f64, gain: f64| (run(&shapes[1], &s, 1.0, gamma_of, gain, 0.0, 1).0 - 1.0).abs();
    let plain_err = err(&plain, 1.0);
    let mut chaos_best = plain_err;
    for alpha in [0.1, 0.3] {
        let g = move |site: usize| 0.2 * (1.0 + alpha * (keyed_unit(site as u64 * 7919 + 13) - 0.5));
        let e = err(&g, 1.0);
        println!("B5 chaos alpha {alpha}: equipartition error {e:.4} (plain {plain_err:.4})");
        chaos_best = chaos_best.min(e);
    }
    // Diagnostic (after the run): the check's own resolution, i.e. plain's error across noise seeds.
    let spread: Vec<f64> = (2..8).map(|seed| (run(&shapes[1], &s, 1.0, &plain, 1.0, 0.0, seed).0 - 1.0).abs()).collect();
    println!("diagnostic: plain equipartition error across noise seeds 2..7: {:?}", spread.iter().map(|e| format!("{e:.4}")).collect::<Vec<_>>());
    let amp = err(&plain, 1.2);
    println!("B6 amplification (noise x1.2): equipartition error {amp:.4} (plain {plain_err:.4})");
    let v = [
        ("B1 equipartition within 2%", b1),
        ("B2 linearity within 3%", b2),
        ("B3 response equals fluctuation", b3),
        ("B4 cold conservation", b4),
        ("B5 chaos not kept", chaos_best >= plain_err),
        ("B6 amplification not kept", amp >= plain_err),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
