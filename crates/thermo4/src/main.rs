//! T4 runner (docs/T4-ROTATE.md): Shepard–Metzler mental rotation in the medium. A chiral 3D chain of
//! cubes is held as a field, rotated continuously about the vertical axis, and compared with a target
//! that is either the same shape rotated by θ or its mirror rotated by θ. Own checks R1–R4, then the
//! amplification (R5), heat (R6) and chaos (R7) rounds. Every setting lives in `Cfg`.

use qb_chaos::keyed_unit;
use thermo_bath::{exchange_edges, normal};
use thermo_lattice::Lattice;
use thermo_rotate::rotate;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cfg {
    side: usize,
    cubes: usize,
    sigma: f64,
    step_deg: f64,
    sweep_deg: f64,
    gamma: f64,
    temperature: f64,
    chaos: f64,
}

fn base() -> Cfg {
    Cfg { side: 32, cubes: 8, sigma: 0.8, step_deg: 3.0, sweep_deg: 180.0, gamma: 1.0, temperature: 0.0, chaos: 0.0 }
}

const ANGLES: [f64; 7] = [0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0];

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

/// A chiral Shepard–Metzler-like chain of `n` unit cubes.
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
        if !ok {
            continue;
        }
        let mirror: Vec<Cube> = cubes.iter().map(|c| [-c[0], c[1], c[2]]).collect();
        let m = normalize(&mirror);
        if (0..4).all(|k| normalize(&cubes.iter().map(|c| rot90(*c, k)).collect::<Vec<_>>()) != m) {
            return cubes;
        }
    }
}

/// Render cubes (size 2) as Gaussian splats, mirrored if asked, rotated by `deg` about the vertical
/// axis through the lattice centre, centroid at the centre.
fn render(cubes: &[Cube], mirror: bool, deg: f64, cfg: &Cfg) -> Lattice {
    let mut l = Lattice::new(&[cfg.side; 3]);
    let centre = (cfg.side as f64 - 1.0) / 2.0;
    let mut pts = Vec::new();
    for c in cubes {
        for dx in [-0.5, 0.5] {
            for dy in [-0.5, 0.5] {
                for dz in [-0.5, 0.5] {
                    let x = 2.0 * c[0] as f64 + dx;
                    pts.push([if mirror { -x } else { x }, 2.0 * c[1] as f64 + dy, 2.0 * c[2] as f64 + dz]);
                }
            }
        }
    }
    let n = pts.len() as f64;
    let mean: Vec<f64> = (0..3).map(|a| pts.iter().map(|p| p[a]).sum::<f64>() / n).collect();
    let (cs, sn) = (deg.to_radians().cos(), deg.to_radians().sin());
    for p in &pts {
        let (x, y, z) = (p[0] - mean[0], p[1] - mean[1], p[2] - mean[2]);
        let (rx, ry) = (cs * x - sn * y, sn * x + cs * y);
        let q = [rx + centre, ry + centre, z + centre];
        let lo: Vec<i64> = q.iter().map(|v| (v - 3.0).floor() as i64).collect();
        for i in lo[0]..=lo[0] + 7 {
            for j in lo[1]..=lo[1] + 7 {
                for k in lo[2]..=lo[2] + 7 {
                    if i < 0 || j < 0 || k < 0 || i as usize >= cfg.side || j as usize >= cfg.side || k as usize >= cfg.side {
                        continue;
                    }
                    let dd = (i as f64 - q[0]).powi(2) + (j as f64 - q[1]).powi(2) + (k as f64 - q[2]).powi(2);
                    let site = i as usize + cfg.side * j as usize + cfg.side * cfg.side * k as usize;
                    l.e[site] += (-dd / (2.0 * cfg.sigma * cfg.sigma)).exp();
                }
            }
        }
    }
    l
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

/// Max distance from the vertical axis of any site holding more than 1e-6 of the peak.
fn radius(l: &Lattice) -> f64 {
    let peak = l.e.iter().cloned().fold(0.0, f64::max);
    let c = (l.dims[0] as f64 - 1.0) / 2.0;
    l.e.iter().enumerate().filter(|(_, x)| **x > 1e-6 * peak).map(|(s, _)| ((l.coord(s, 0) as f64 - c).powi(2) + (l.coord(s, 1) as f64 - c).powi(2)).sqrt()).fold(0.0, f64::max)
}

struct Sweep {
    sims: Vec<f64>,
    max_radius: f64,
}

/// Rotate the held reference through the sweep, recording the similarity to the target each step.
fn sweep(cubes: &[Cube], target: &Lattice, cfg: &Cfg, seed: u64) -> Sweep {
    let mut held = render(cubes, false, 0.0, cfg);
    let steps = (cfg.sweep_deg / cfg.step_deg).round() as usize;
    let mut sims = Vec::with_capacity(steps + 1);
    let mut max_radius = radius(&held);
    for t in 0..=steps {
        sims.push(cosine(&held.e, &target.e));
        if t == steps {
            break;
        }
        let jitter = if cfg.chaos > 0.0 { 1.0 + cfg.chaos * (keyed_unit(seed.wrapping_mul(0x9E37_79B9) ^ t as u64) - 0.5) } else { 1.0 };
        held.e = rotate(&held, 0, 1, (cfg.step_deg * jitter).to_radians());
        if cfg.gamma != 1.0 {
            let total: f64 = held.e.iter().sum();
            for x in held.e.iter_mut() {
                *x = x.max(0.0).powf(cfg.gamma);
            }
            let after: f64 = held.e.iter().sum();
            for x in held.e.iter_mut() {
                *x *= total / after;
            }
        }
        if cfg.temperature > 0.0 {
            let z = |s: usize, axis: usize| {
                let u = |tag: u64| { let x = keyed_unit(seed ^ (t as u64).wrapping_mul(0xC2B2_AE3D) ^ (s as u64).wrapping_mul(0x1656_67B1) ^ tag); (2.0 / std::f64::consts::PI * x.sqrt().asin()).clamp(1e-12, 1.0 - 1e-12) };
                normal(u(2 * axis as u64 + 1), u(2 * axis as u64 + 2))
            };
            exchange_edges(&mut held, cfg.temperature, &|_, _| 0.05, &|_| 0.0, &z);
        }
        max_radius = max_radius.max(radius(&held));
    }
    Sweep { sims, max_radius }
}

struct Trial {
    theta: f64,
    same: bool,
    max_sim: f64,
    sims: Vec<f64>,
    max_radius: f64,
}

fn trials(seeds: std::ops::Range<u64>, cfg: &Cfg) -> Vec<Trial> {
    let mut out = Vec::new();
    for seed in seeds {
        let cubes = shape(seed, cfg.cubes);
        for &theta in &ANGLES {
            for same in [true, false] {
                let target = render(&cubes, !same, theta, cfg);
                let s = sweep(&cubes, &target, cfg, seed);
                out.push(Trial { theta, same, max_sim: s.sims.iter().cloned().fold(f64::MIN, f64::max), sims: s.sims, max_radius: s.max_radius });
            }
        }
    }
    out
}

fn threshold(tr: &[Trial]) -> f64 {
    let mean = |same: bool| { let v: Vec<f64> = tr.iter().filter(|t| t.same == same).map(|t| t.max_sim).collect(); v.iter().sum::<f64>() / v.len() as f64 };
    (mean(true) + mean(false)) / 2.0
}

struct Score {
    accuracy: f64,
    same_mean_max: f64,
    min_same_by_theta: f64,
    r2: f64,
    slope: f64,
    max_radius: f64,
}

fn score(tr: &[Trial], s_star: f64, cfg: &Cfg) -> Score {
    let correct = tr.iter().filter(|t| (t.max_sim >= s_star) == t.same).count() as f64 / tr.len() as f64;
    let same: Vec<&Trial> = tr.iter().filter(|t| t.same).collect();
    let same_mean_max = same.iter().map(|t| t.max_sim).sum::<f64>() / same.len() as f64;
    let min_same_by_theta = ANGLES.iter().map(|&a| { let v: Vec<f64> = same.iter().filter(|t| t.theta == a).map(|t| t.max_sim).collect(); v.iter().sum::<f64>() / v.len() as f64 }).fold(f64::MAX, f64::min);
    // RT: the first step at which the similarity reaches s*, for same pairs that reach it.
    let pts: Vec<(f64, f64)> = same.iter().filter_map(|t| t.sims.iter().position(|&s| s >= s_star).map(|rt| (t.theta, rt as f64))).collect();
    let n = pts.len() as f64;
    let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(|p| p.1).sum::<f64>() / n);
    let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let syy: f64 = pts.iter().map(|p| (p.1 - my).powi(2)).sum();
    let _ = cfg;
    Score { accuracy: correct, same_mean_max, min_same_by_theta, r2: sxy * sxy / (sxx * syy), slope: sxy / sxx, max_radius: tr.iter().map(|t| t.max_radius).fold(0.0, f64::max) }
}

/// Diagnostic, added after run 1 (not a prediction): how faithfully the held field tracks the exact
/// render at the same angle, and at which step the plain same-pair maximum falls.
fn diagnose(cfg: &Cfg) {
    for seed in 0..4u64 {
        let cubes = shape(seed, cfg.cubes);
        let mut held = render(&cubes, false, 0.0, cfg);
        let peak = held.e.iter().cloned().fold(0.0, f64::max);
        let mut line = format!("shape {seed}: peak {peak:.3}; fidelity to the exact render at");
        for t in 1..=60usize {
            held.e = rotate(&held, 0, 1, cfg.step_deg.to_radians());
            if t % 10 == 0 {
                let exact = render(&cubes, false, t as f64 * cfg.step_deg, cfg);
                line += &format!(" {}deg {:.4}", t as f64 * cfg.step_deg, cosine(&held.e, &exact.e));
            }
        }
        println!("{line}");
        let target = render(&cubes, false, 90.0, cfg);
        let s = sweep(&cubes, &target, cfg, seed);
        let (arg, max) = s.sims.iter().enumerate().fold((0, f64::MIN), |b, (i, &v)| if v > b.1 { (i, v) } else { b });
        println!("  same 90deg: max {max:.4} at step {arg} (aligned is step 30; step-0 sim {:.4}, step-30 sim {:.4})", s.sims[0], s.sims[30]);
    }
}

fn main() {
    let cfg = base();
    if std::env::var("T4_DIAG").is_ok() {
        diagnose(&cfg);
        return;
    }
    println!("T4: mental rotation in the medium; side {}, {} cubes, step {} deg", cfg.side, cfg.cubes, cfg.step_deg);
    let dev = trials(0..10, &cfg);
    let s_star = threshold(&dev);
    let test = trials(100..120, &cfg);
    let sc = score(&test, s_star, &cfg);
    println!("threshold s* (dev) {s_star:.4}");
    for &a in &ANGLES {
        let f = |same: bool| { let v: Vec<f64> = test.iter().filter(|t| t.same == same && t.theta == a).map(|t| t.max_sim).collect(); v.iter().sum::<f64>() / v.len() as f64 };
        println!("  theta {a:>5}: same max sim {:.4}, mirror max sim {:.4}", f(true), f(false));
    }
    println!("test: accuracy {:.3}; same mean max {:.4} (min over theta {:.4}); RT vs theta R^2 {:.4}, slope {:.4} steps/deg (want {:.4}); max radius {:.2} (limit {:.1})", sc.accuracy, sc.same_mean_max, sc.min_same_by_theta, sc.r2, sc.slope, 1.0 / cfg.step_deg, sc.max_radius, cfg.side as f64 / 2.0 - 3.0);
    // Rounds: check = dev accuracy, tie-break dev same-mean-max; kept only if better on test.
    let rank = |c: &Cfg| { let d = trials(0..10, c); let sc = score(&d, threshold(&d), c); (sc.accuracy, sc.same_mean_max) };
    let test_rank = |c: &Cfg| { let d = trials(0..10, c); let t = trials(100..120, c); let sc = score(&t, threshold(&d), c); (sc.accuracy, sc.same_mean_max) };
    let mut kept = cfg;
    let mut verdicts = Vec::new();
    for (name, variants) in [("amplification", vec![Cfg { gamma: 1.03, ..kept }, Cfg { gamma: 1.08, ..kept }]), ("heat", vec![]), ("chaos", vec![])] {
        let variants: Vec<Cfg> = match name {
            "heat" => [1e-7, 1e-6].iter().map(|&t| Cfg { temperature: t, ..kept }).collect(),
            "chaos" => [0.1, 0.3].iter().map(|&a| Cfg { chaos: a, ..kept }).collect(),
            _ => variants,
        };
        let off = rank(&kept);
        let mut best = (off, kept);
        for v in &variants {
            let r = rank(v);
            println!("   {name} variant (gamma {}, T {:e}, chaos {}): dev accuracy {:.3}, same mean max {:.4}", v.gamma, v.temperature, v.chaos, r.0, r.1);
            if r > best.0 {
                best = (r, *v);
            }
        }
        let (toff, ton) = (test_rank(&kept), test_rank(&best.1));
        let keep = best.1 != kept && ton > toff;
        println!("   {name}: test off ({:.3}, {:.4}) -> on ({:.3}, {:.4}) -> {}", toff.0, toff.1, ton.0, ton.1, if keep { "KEPT" } else { "not kept" });
        verdicts.push(keep);
        if keep {
            kept = best.1;
        }
    }
    let v = [
        ("R1 the shape survives rotation (>= 0.90 at every theta)", sc.min_same_by_theta >= 0.90),
        ("R2 Shepard-Metzler: RT linear (R^2 >= 0.98), slope within 10%", sc.r2 >= 0.98 && (sc.slope * cfg.step_deg - 1.0).abs() < 0.10),
        ("R3 same vs mirror accuracy >= 0.90", sc.accuracy >= 0.90),
        ("R4 the mesh holds the rotation", sc.max_radius <= cfg.side as f64 / 2.0 - 3.0),
        ("R5 amplification kept", verdicts[0]),
        ("R6 heat not kept", !verdicts[1]),
        ("R7 chaos not kept", !verdicts[2]),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
