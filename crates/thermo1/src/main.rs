//! T1 runner (docs/T1-LATTICE.md): the lattice's own checks L1–L5 for d = 1–4, then its chaos round
//! (L6). Every setting lives in `Settings`.

use qb_chaos::keyed_unit;
use thermo_lattice::Lattice;

struct Settings {
    kappa: f64,
    steps_conserve: usize,
    steps_spread: usize,
    side_low: usize,
    side_4d: usize,
    chaos_alphas: [f64; 2],
    chaos_seeds: u64,
}

fn settings() -> Settings {
    Settings { kappa: 0.2, steps_conserve: 200, steps_spread: 40, side_low: 32, side_4d: 16, chaos_alphas: [0.1, 0.3], chaos_seeds: 5 }
}

fn dims(s: &Settings, d: usize) -> Vec<usize> {
    vec![if d == 4 { s.side_4d } else { s.side_low }; d]
}

/// Mean relative error of the total spread against d·κ·t over t = 1..=steps (the L2 fidelity).
fn spread_error(d: usize, s: &Settings, kappa: &dyn Fn(usize, usize, usize) -> f64) -> (f64, f64, bool) {
    let mut l = Lattice::new(&dims(s, d));
    l.e[0] = 1.0;
    let (mut err, mut worst, mut positive) = (0.0, 0.0f64, true);
    for t in 1..=s.steps_spread {
        l.exchange(&|site, axis| kappa(t, site, axis));
        positive &= l.e.iter().all(|x| *x >= 0.0);
        let m2: f64 = l.spread(0).iter().sum();
        let want = d as f64 * s.kappa * t as f64;
        let e = (m2 - want).abs() / want;
        err += e / s.steps_spread as f64;
        worst = worst.max(e);
    }
    (err, worst, positive)
}

fn main() {
    let s = settings();
    println!("T1: the d-dimensional lattice, kappa {}", s.kappa);
    let mut ok = [true; 5];
    for d in 1..=4 {
        // L1 conservation.
        let mut l = Lattice::new(&dims(&s, d));
        for (i, x) in l.e.iter_mut().enumerate() {
            *x = ((i * 2654435761) % 1000) as f64 / 1000.0;
        }
        let t0 = l.total();
        for _ in 0..s.steps_conserve {
            l.exchange(&|_, _| s.kappa);
        }
        let drift = (l.total() - t0).abs() / t0;
        ok[0] &= drift < 1e-9;
        // L2, L4.
        let (mean_err, worst, positive) = spread_error(d, &s, &|_, _, _| s.kappa);
        ok[1] &= worst < 0.01;
        ok[3] &= positive;
        // L3 isotropy.
        let mut li = Lattice::new(&dims(&s, d));
        li.e[0] = 1.0;
        for _ in 0..s.steps_spread {
            li.exchange(&|_, _| s.kappa);
        }
        let per = li.spread(0);
        let (lo, hi) = (per.iter().cloned().fold(f64::MAX, f64::min), per.iter().cloned().fold(0.0, f64::max));
        let iso = (hi - lo) / hi;
        ok[2] &= iso < 0.01;
        println!("d={d}: L1 drift {drift:.1e}; L2 spread error mean {mean_err:.2e}, worst {worst:.2e}; L3 anisotropy {iso:.1e}; L4 positive {positive}; per-axis m2 at t={}: {:?}", s.steps_spread, per.iter().map(|x| format!("{x:.3}")).collect::<Vec<_>>());
    }
    // L5: d=3 with one axis of length 1 is bit-identical to d=2.
    let (mut a, mut b) = (Lattice::new(&[s.side_low, s.side_low]), Lattice::new(&[s.side_low, s.side_low, 1]));
    a.e[5] = 1.0;
    b.e[5] = 1.0;
    for _ in 0..s.steps_spread {
        a.exchange(&|_, _| s.kappa);
        b.exchange(&|_, _| s.kappa);
    }
    ok[4] = a.e == b.e;
    println!("L5: d=3 with a unit axis equals d=2 bit for bit: {}", ok[4]);

    // L6: the chaos round, judged on L2's fidelity (mean relative error; lower is better).
    println!("\nL6 chaos round (check: spread fidelity, mean relative error; plain wins ties)");
    let mut chaos_wins = 0;
    for d in 1..=4 {
        let (plain, _, _) = spread_error(d, &s, &|_, _, _| s.kappa);
        let mut best = (plain, "plain".to_owned());
        for &alpha in &s.chaos_alphas {
            for seed in 0..s.chaos_seeds {
                let k = |t: usize, site: usize, axis: usize| {
                    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (site as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ (axis as u64 + 1).wrapping_mul(0xD6E8_FEB8_6659_FD93);
                    s.kappa * (1.0 + alpha * (keyed_unit(key) - 0.5))
                };
                let (e, _, _) = spread_error(d, &s, &k);
                if e < best.0 {
                    best = (e, format!("alpha {alpha} seed {seed}"));
                }
            }
        }
        if best.1 != "plain" {
            chaos_wins += 1;
        }
        println!("d={d}: plain {plain:.2e}; best {:.2e} ({})", best.0, best.1);
    }
    // Diagnostic (after the run, not a prediction): the 4D shortfall against the lattice side.
    for side in [16usize, 24] {
        let mut l = Lattice::new(&[side; 4]);
        l.e[0] = 1.0;
        let mut worst = 0.0f64;
        for t in 1..=s.steps_spread {
            l.exchange(&|_, _| s.kappa);
            let m2: f64 = l.spread(0).iter().sum();
            worst = worst.max((m2 - 4.0 * s.kappa * t as f64).abs() / (4.0 * s.kappa * t as f64));
        }
        println!("diagnostic: d=4, side {side}: worst spread error over t<=40 {worst:.2e}");
    }
    let v = [
        ("L1 conservation", ok[0]),
        ("L2 the diffusion law (within 1%)", ok[1]),
        ("L3 isotropy (within 1%)", ok[2]),
        ("L4 positivity", ok[3]),
        ("L5 the same law in every d", ok[4]),
        ("L6 chaos not kept (plain wins on every d)", chaos_wins == 0),
    ];
    println!();
    for (name, good) in &v {
        println!("{} {name}", if *good { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
