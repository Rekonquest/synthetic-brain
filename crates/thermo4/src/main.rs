//! T4 runner, run 2 (docs/T4-ROTATE.md): Shepard–Metzler mental rotation in the medium. A chiral 3D
//! chain of cubes is held as a field, rotated continuously about the vertical axis by three
//! band-limited shears (`thermo-shear`), and compared with a target that is either the same shape
//! rotated by θ or its mirror rotated by θ. Own checks R1–R4 and R8, then the amplification (R5),
//! heat (R6) and chaos (R7) rounds. Every setting lives in `Cfg`. Run 1 (bilinear advection,
//! `thermo-rotate`) is the code at commit 1de482e.

use qb_chaos::keyed_unit;
use thermo_bath::normal;
use thermo_lattice::Lattice;
use thermo_shear::shear;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cfg {
    side: usize,
    cube: f64,
    sigma: f64,
    cubes: usize,
    step_deg: f64,
    sweep_deg: f64,
    gamma: f64,
    temperature: f64,
    chaos: f64,
}

fn base() -> Cfg {
    Cfg { side: 32, cube: 2.0, sigma: 0.8, cubes: 8, step_deg: 3.0, sweep_deg: 180.0, gamma: 1.0, temperature: 0.0, chaos: 0.0 }
}

const ANGLES: [f64; 7] = [0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];
const PEAK: f64 = 0.05;

/// SplitMix64 for shape generation.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

type Cube = [i32; 3];

fn rot90(c: Cube, k: usize) -> Cube {
    let mut c = c;
    for _ in 0..k {
        c = [-c[1], c[0], c[2]];
    }
    c
}

fn normalize(set: &[Cube]) -> Vec<Cube> {
    let m: Vec<i32> = (0..3).map(|a| set.iter().map(|c| c[a]).min().unwrap()).collect();
    let mut v: Vec<Cube> = set.iter().map(|c| [c[0] - m[0], c[1] - m[1], c[2] - m[2]]).collect();
    v.sort();
    v
}

/// Sub-voxel centres in side-32 units (cube size 2, offsets ±0.5), centred on their centroid.
fn points(cubes: &[Cube]) -> Vec<[f64; 3]> {
    let mut pts = Vec::new();
    for c in cubes {
        for dx in [-0.5, 0.5] {
            for dy in [-0.5, 0.5] {
                for dz in [-0.5, 0.5] {
                    pts.push([2.0 * c[0] as f64 + dx, 2.0 * c[1] as f64 + dy, 2.0 * c[2] as f64 + dz]);
                }
            }
        }
    }
    let n = pts.len() as f64;
    let mean: Vec<f64> = (0..3).map(|a| pts.iter().map(|p| p[a]).sum::<f64>() / n).collect();
    pts.iter().map(|p| [p[0] - mean[0], p[1] - mean[1], p[2] - mean[2]]).collect()
}

/// A chiral Shepard–Metzler-like chain of `n` unit cubes whose sub-voxel centres lie within 9 of the
/// centroid (side-32 units).
fn shape(seed: u64, n: usize) -> Vec<Cube> {
    let mut r = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ 0x51);
    let dirs: [Cube; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    loop {
        let mut cubes = vec![[0, 0, 0]];
        let mut d = dirs[r.below(6) as usize];
        let mut ok = true;
        while cubes.len() < n {
            if r.below(10) < 4 {
                let choices: Vec<Cube> = dirs.iter().copied().filter(|x| x[0] * d[0] + x[1] * d[1] + x[2] * d[2] == 0).collect();
                d = choices[r.below(choices.len() as u64) as usize];
            }
            let last = *cubes.last().unwrap();
            let next = [last[0] + d[0], last[1] + d[1], last[2] + d[2]];
            if cubes.contains(&next) {
                ok = false;
                break;
            }
            cubes.push(next);
        }
        if !ok || points(&cubes).iter().any(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() > 9.0) {
            continue;
        }
        let mirror: Vec<Cube> = cubes.iter().map(|c| [-c[0], c[1], c[2]]).collect();
        let m = normalize(&mirror);
        if (0..4).all(|k| normalize(&cubes.iter().map(|c| rot90(*c, k)).collect::<Vec<_>>()) != m) {
            return cubes;
        }
    }
}

/// Render as Gaussian splats (window ±6σ), mirrored if asked, rotated by `deg` about the vertical
/// axis through the lattice centre, scaled to `cfg.cube / 2` per side-32 unit, peak scaled to PEAK.
fn render(cubes: &[Cube], mirror: bool, deg: f64, cfg: &Cfg) -> Lattice {
    let mut l = Lattice::new(&[cfg.side; 3]);
    let centre = (cfg.side as f64 - 1.0) / 2.0;
    let (cs, sn) = (deg.to_radians().cos(), deg.to_radians().sin());
    let scale = cfg.cube / 2.0;
    let w = (6.0 * cfg.sigma).ceil() as i64;
    for p in points(cubes) {
        let x = if mirror { -p[0] } else { p[0] } * scale;
        let (y, z) = (p[1] * scale, p[2] * scale);
        let q = [cs * x - sn * y + centre, sn * x + cs * y + centre, z + centre];
        let c: Vec<i64> = q.iter().map(|v| v.round() as i64).collect();
        for i in c[0] - w..=c[0] + w {
            for j in c[1] - w..=c[1] + w {
                for k in c[2] - w..=c[2] + w {
                    if i < 0 || j < 0 || k < 0 || i as usize >= cfg.side || j as usize >= cfg.side || k as usize >= cfg.side {
                        continue;
                    }
                    let dd = (i as f64 - q[0]).powi(2) + (j as f64 - q[1]).powi(2) + (k as f64 - q[2]).powi(2);
                    l.e[i as usize + cfg.side * j as usize + cfg.side * cfg.side * k as usize] += (-dd / (2.0 * cfg.sigma * cfg.sigma)).exp();
                }
            }
        }
    }
    let peak = l.e.iter().cloned().fold(0.0, f64::max);
    for x in l.e.iter_mut() {
        *x *= PEAK / peak;
    }
    l
}

/// Rotation by `rad` in the (x, y) plane as three band-limited shears (Paeth).
fn rotate3(field: &mut Lattice, rad: f64) {
    field.e = shear(field, 0, 1, -(rad / 2.0).tan());
    field.e = shear(field, 1, 0, rad.sin());
    field.e = shear(field, 0, 1, -(rad / 2.0).tan());
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let (mut ab, mut aa, mut bb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        ab += x * y;
        aa += x * x;
        bb += y * y;
    }
    ab / (aa.sqrt() * bb.sqrt())
}

/// R4: the share of Σe² farther than L/2 − 3 from the vertical axis.
fn outer_share(l: &Lattice) -> f64 {
    let c = (l.dims[0] as f64 - 1.0) / 2.0;
    let limit = l.dims[0] as f64 / 2.0 - 3.0;
    let (mut out, mut all) = (0.0, 0.0);
    for (s, x) in l.e.iter().enumerate() {
        let r = ((l.coord(s, 0) as f64 - c).powi(2) + (l.coord(s, 1) as f64 - c).powi(2)).sqrt();
        all += x * x;
        if r > limit {
            out += x * x;
        }
    }
    out / all
}

/// A uniform from a keyed logistic value (arcsine-distributed) for Box–Muller, as in T2.
fn uniform(key: u64) -> f64 {
    (2.0 / std::f64::consts::PI * keyed_unit(key).sqrt().asin()).clamp(1e-12, 1.0 - 1e-12)
}

struct Sweep {
    sims: Vec<f64>,
    outer: f64,
}

/// Rotate the held reference through the sweep, recording the similarity to the target each step.
fn sweep(cubes: &[Cube], target: &Lattice, cfg: &Cfg, seed: u64) -> Sweep {
    let mut held = render(cubes, false, 0.0, cfg);
    let steps = (cfg.sweep_deg / cfg.step_deg).round() as usize;
    let mut sims = Vec::with_capacity(steps + 1);
    let mut outer = outer_share(&held);
    for t in 0..=steps {
        sims.push(cosine(&held.e, &target.e));
        if t == steps {
            break;
        }
        let jitter = if cfg.chaos > 0.0 { 1.0 + cfg.chaos * (keyed_unit(seed.wrapping_mul(0x9E37_79B9) ^ t as u64) - 0.5) } else { 1.0 };
        rotate3(&mut held, (cfg.step_deg * jitter).to_radians());
        if cfg.gamma != 1.0 {
            let total: f64 = held.e.iter().sum();
            for x in held.e.iter_mut() {
                *x = x.signum() * x.abs().powf(cfg.gamma);
            }
            let after: f64 = held.e.iter().sum();
            for x in held.e.iter_mut() {
                *x *= total / after;
            }
        }
        if cfg.temperature > 0.0 {
            // Conserved kicks only: f = √(T/2)·ξ on every edge (the bath's κ(2−κ)/2 at κ = 1).
            let amp = (cfg.temperature / 2.0).sqrt();
            for axis in 0..3 {
                for s in 0..held.e.len() {
                    let key = seed ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (s as u64 * 3 + axis as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                    let f = amp * normal(uniform(key), uniform(key ^ 0xA5A5_A5A5));
                    let j = held.forward(s, axis);
                    held.e[s] += f;
                    held.e[j] -= f;
                }
            }
        }
        outer = outer.max(outer_share(&held));
    }
    Sweep { sims, outer }
}

struct Trial {
    theta: f64,
    same: bool,
    max_sim: f64,
    sims: Vec<f64>,
    outer: f64,
}

/// All trials of the given shapes, one thread per shape.
fn trials(seeds: std::ops::Range<u64>, cfg: &Cfg) -> Vec<Trial> {
    let seeds: Vec<u64> = seeds.collect();
    let per: Vec<Vec<Trial>> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                sc.spawn(move || {
                    let cubes = shape(seed, cfg.cubes);
                    let mut out = Vec::new();
                    for &theta in &ANGLES {
                        for same in [true, false] {
                            let target = render(&cubes, !same, theta, cfg);
                            let s = sweep(&cubes, &target, cfg, seed);
                            out.push(Trial { theta, same, max_sim: s.sims.iter().cloned().fold(f64::MIN, f64::max), sims: s.sims, outer: s.outer });
                        }
                    }
                    out
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    per.into_iter().flatten().collect()
}

fn threshold(tr: &[Trial]) -> f64 {
    let mean = |same: bool| {
        let v: Vec<f64> = tr.iter().filter(|t| t.same == same).map(|t| t.max_sim).collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    (mean(true) + mean(false)) / 2.0
}

struct Score {
    accuracy: f64,
    same_mean_max: f64,
    min_same_by_theta: f64,
    r2: f64,
    slope: f64,
    outer: f64,
    reached: usize,
    same_n: usize,
}

fn score(tr: &[Trial], s_star: f64) -> Score {
    let accuracy = tr.iter().filter(|t| (t.max_sim >= s_star) == t.same).count() as f64 / tr.len() as f64;
    let same: Vec<&Trial> = tr.iter().filter(|t| t.same).collect();
    let same_mean_max = same.iter().map(|t| t.max_sim).sum::<f64>() / same.len() as f64;
    let min_same_by_theta = ANGLES
        .iter()
        .map(|&a| {
            let v: Vec<f64> = same.iter().filter(|t| t.theta == a).map(|t| t.max_sim).collect();
            v.iter().sum::<f64>() / v.len() as f64
        })
        .fold(f64::MAX, f64::min);
    // RT: the first step at which the similarity reaches s*, for same pairs that reach it.
    let pts: Vec<(f64, f64)> = same.iter().filter_map(|t| t.sims.iter().position(|&s| s >= s_star).map(|rt| (t.theta, rt as f64))).collect();
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let syy: f64 = pts.iter().map(|p| (p.1 - my).powi(2)).sum();
    Score {
        accuracy,
        same_mean_max,
        min_same_by_theta,
        r2: sxy * sxy / (sxx * syy),
        slope: sxy / sxx,
        outer: tr.iter().map(|t| t.outer).fold(0.0, f64::max),
        reached: pts.len(),
        same_n: same.len(),
    }
}

fn report(label: &str, dev: &[Trial], test: &[Trial], cfg: &Cfg) -> Score {
    let s_star = threshold(dev);
    let sc = score(test, s_star);
    println!("{label}: side {}, cube {}, sigma {}; threshold s* (dev) {s_star:.4}", cfg.side, cfg.cube, cfg.sigma);
    for &a in &ANGLES {
        let f = |same: bool| {
            let v: Vec<f64> = test.iter().filter(|t| t.same == same && t.theta == a).map(|t| t.max_sim).collect();
            v.iter().sum::<f64>() / v.len() as f64
        };
        let rts: Vec<f64> = test.iter().filter(|t| t.same && t.theta == a).filter_map(|t| t.sims.iter().position(|&s| s >= s_star)).map(|x| x as f64).collect();
        println!("  theta {a:>5}: same max sim {:.4}, mirror max sim {:.4}, mean RT {:.2} steps ({} of 20 reach s*)", f(true), f(false), rts.iter().sum::<f64>() / rts.len().max(1) as f64, rts.len());
    }
    println!(
        "  test: accuracy {:.3}; same mean max {:.4} (min over theta {:.4}); RT vs theta R^2 {:.4}, slope {:.4} steps/deg (want {:.4}), {} of {} same pairs reach s*; max outer energy share {:.2e}",
        sc.accuracy, sc.same_mean_max, sc.min_same_by_theta, sc.r2, sc.slope, 1.0 / cfg.step_deg, sc.reached, sc.same_n, sc.outer
    );
    sc
}

/// Diagnostic, added after run 2 (not a prediction): per test shape, the mirror pairs that pass s*
/// and the same-pair RT at 180 deg, against the shape's own C2 self-similarity (A vs A rotated 180).
fn diagnose(cfg: &Cfg) {
    let s_star = threshold(&trials(0..10, cfg));
    let test = trials(100..120, cfg);
    for (k, seed) in (100..120u64).enumerate() {
        let tr = &test[k * 14..(k + 1) * 14];
        let mirror_pass = tr.iter().filter(|t| !t.same && t.max_sim >= s_star).count();
        let rt180 = tr.iter().find(|t| t.same && t.theta == 180.0).and_then(|t| t.sims.iter().position(|&x| x >= s_star));
        let cubes = shape(seed, cfg.cubes);
        let c2 = cosine(&render(&cubes, false, 0.0, cfg).e, &render(&cubes, false, 180.0, cfg).e);
        let mm = tr.iter().filter(|t| !t.same).map(|t| t.max_sim).fold(0.0, f64::max);
        println!("shape {seed}: mirror pairs passing s* {mirror_pass}/7 (max {mm:.4}); RT at 180 {rt180:?}; C2 self-similarity {c2:.4}");
    }
}

fn main() {
    let cfg = base();
    if std::env::var("T4_DIAG").is_ok() {
        diagnose(&cfg);
        return;
    }
    println!("T4 run 2: mental rotation by three band-limited shears; {} cubes, step {} deg", cfg.cubes, cfg.step_deg);
    // Fidelity to the exact render after the full sweep (the R1 note's sub-prediction: >= 0.99).
    for seed in 0..4u64 {
        let cubes = shape(seed, cfg.cubes);
        let mut held = render(&cubes, false, 0.0, &cfg);
        let mut line = format!("fidelity, dev shape {seed}:");
        for t in 1..=60usize {
            rotate3(&mut held, cfg.step_deg.to_radians());
            if t % 20 == 0 {
                line += &format!(" {}deg {:.6}", t as f64 * cfg.step_deg, cosine(&held.e, &render(&cubes, false, t as f64 * cfg.step_deg, &cfg).e));
            }
        }
        println!("{line}");
    }
    let dev = trials(0..10, &cfg);
    let test = trials(100..120, &cfg);
    let sc = report("base", &dev, &test, &cfg);
    // R8: the same shapes at twice the resolution.
    let big = Cfg { side: 64, cube: 4.0, sigma: 1.6, ..cfg };
    let sc_big = report("R8 enlarged", &trials(0..10, &big), &trials(100..120, &big), &big);
    // Rounds: check = dev accuracy, tie-break dev same-mean-max; kept only if better on test.
    let rank = |c: &Cfg| {
        let d = trials(0..10, c);
        let s = score(&d, threshold(&d));
        (s.accuracy, s.same_mean_max)
    };
    let test_rank = |c: &Cfg| {
        let d = trials(0..10, c);
        let s = score(&trials(100..120, c), threshold(&d));
        (s.accuracy, s.same_mean_max)
    };
    let mut kept = cfg;
    let mut verdicts = Vec::new();
    for name in ["amplification", "heat", "chaos"] {
        let variants: Vec<Cfg> = match name {
            "amplification" => [1.03, 1.08].iter().map(|&g| Cfg { gamma: g, ..kept }).collect(),
            "heat" => [1e-7, 1e-6].iter().map(|&t| Cfg { temperature: t, ..kept }).collect(),
            _ => [0.1, 0.3].iter().map(|&a| Cfg { chaos: a, ..kept }).collect(),
        };
        let off = rank(&kept);
        println!("   {name}: plain dev accuracy {:.3}, same mean max {:.4}", off.0, off.1);
        let mut best = (off, kept);
        for v in &variants {
            let r = rank(v);
            println!("   {name} variant (gamma {}, T {:e}, chaos {}): dev accuracy {:.3}, same mean max {:.4}", v.gamma, v.temperature, v.chaos, r.0, r.1);
            if r > best.0 {
                best = (r, *v);
            }
        }
        let keep = if best.1 == kept {
            println!("   {name}: plain wins on dev -> not kept");
            false
        } else {
            let (toff, ton) = (test_rank(&kept), test_rank(&best.1));
            let k = ton > toff;
            println!("   {name}: test off ({:.3}, {:.4}) -> on ({:.3}, {:.4}) -> {}", toff.0, toff.1, ton.0, ton.1, if k { "KEPT" } else { "not kept" });
            k
        };
        verdicts.push(keep);
        if keep {
            kept = best.1;
        }
    }
    let v = [
        ("R1 the shape survives rotation (>= 0.90 at every theta)", sc.min_same_by_theta >= 0.90),
        ("R2 Shepard-Metzler: RT linear (R^2 >= 0.98), slope within 10%", sc.r2 >= 0.98 && (sc.slope * cfg.step_deg - 1.0).abs() < 0.10),
        ("R3 same vs mirror accuracy >= 0.90", sc.accuracy >= 0.90),
        ("R4 the mesh holds the rotation (outer energy share < 1e-6)", sc.outer < 1e-6),
        ("R5 amplification not kept", !verdicts[0]),
        ("R6 heat not kept", !verdicts[1]),
        ("R7 chaos not kept", !verdicts[2]),
        ("R8 converges at side 64 (accuracy within 0.03, same mean max within 0.01)", (sc.accuracy - sc_big.accuracy).abs() <= 0.03 && (sc.same_mean_max - sc_big.same_mean_max).abs() <= 0.01),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
