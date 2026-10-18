//! T3 runner (docs/T3-CARVE.md): threads carved by a steady current between two pinned sites. Own
//! checks C1–C5 (with finite-size convergence as the sizing test), then the amplification, chaos and
//! heat rounds judged on diagonal selectivity with a held-out diagonal. Every setting lives in `Cfg`.

use qb_chaos::keyed_unit;
use thermo_bath::{damp, exchange_edges, normal};
use thermo_carve::Carving;
use thermo_lattice::Lattice;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cfg {
    kappa: f64,
    mu: f64,
    nu: f64,
    c_min: f64,
    c0: f64,
    steps: usize,
    p: f64,
    chaos_alpha: f64,
    chaos_frozen: bool,
    temperature: f64,
    carve: bool,
}

fn base() -> Cfg {
    Cfg { kappa: 0.3, mu: 0.02, nu: 0.002, c_min: 0.05, c0: 0.2, steps: 3000, p: 1.0, chaos_alpha: 0.0, chaos_frozen: false, temperature: 0.0, carve: true }
}

fn site_of(l: &Lattice, coords: &[usize]) -> usize {
    coords.iter().enumerate().map(|(a, &c)| c * l.stride(a)).sum()
}

fn uni(key: u64) -> f64 {
    let x = keyed_unit(key);
    (2.0 / std::f64::consts::PI * x.sqrt().asin()).clamp(1e-12, 1.0 - 1e-12)
}

struct Outcome {
    lattice: Lattice,
    carving: Carving,
    current: f64,
}

/// Drive a current from A (+1) to B (−1) and carve.
fn simulate(dims: &[usize], a: &[usize], b: &[usize], cfg: &Cfg, seed: u64) -> Outcome {
    let mut l = Lattice::new(dims);
    let d = dims.len();
    let (sa, sb) = (site_of(&l, a), site_of(&l, b));
    let mut k = Carving::uniform(l.e.len() * d, cfg.c0);
    let mut flux = vec![0.0; l.e.len() * d];
    let h = |s: usize| if s == sa { 1.0 } else if s == sb { -1.0 } else { 0.0 };
    let gamma = |s: usize| if s == sa || s == sb { 1.0 } else { 0.0 };
    let dist = a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).sum::<usize>() as f64;
    let f_ref = cfg.kappa / dist;
    for t in 0..cfg.steps {
        let z = |s: usize, tag: u64| normal(uni(seed ^ (t as u64).wrapping_mul(0x9E37_79B9) ^ (s as u64).wrapping_mul(0xC2B2_AE3D) ^ tag), uni(seed ^ (t as u64).wrapping_mul(0x1656_67B1) ^ (s as u64).wrapping_mul(0xD6E8_FEB8) ^ tag));
        // The reservoirs pin A and B through damping only; the exchange is driven by the raw field
        // differences (h = 0 there). Passing the drive as the exchange's equilibrium field cancels the
        // gradient (the void first run, evidence/thermo-3/results-VOID-drive-bug.txt).
        let hot = cfg.temperature > 0.0;
        damp(&mut l, cfg.temperature, &gamma, &h, &|s| if hot { z(s, 1) } else { 0.0 });
        let c = &k.c;
        flux = exchange_edges(&mut l, cfg.temperature, &|s, axis| cfg.kappa * c[s * d + axis], &|_| 0.0, &|s, axis| if hot { z(s, 10 + axis as u64) } else { 0.0 });
        if cfg.carve {
            let mu = |e: usize| {
                if cfg.chaos_alpha == 0.0 {
                    return cfg.mu;
                }
                let key = (e as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ seed.wrapping_mul(31) ^ if cfg.chaos_frozen { 0 } else { (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) };
                cfg.mu * (1.0 + cfg.chaos_alpha * (keyed_unit(key) - 0.5))
            };
            k.carve(&flux, &mu, cfg.nu, cfg.c_min, cfg.p, f_ref);
        }
    }
    // The current out of A: edges (A, j) carry f into A; edges (i, A) carry f out of A.
    let mut current = 0.0;
    for axis in 0..d {
        current -= flux[sa * d + axis];
        for s in 0..l.e.len() {
            if l.forward(s, axis) == sa {
                current += flux[s * d + axis];
            }
        }
    }
    Outcome { lattice: l, carving: k, current }
}

/// The squared minimum-image distance from an edge's midpoint to the segment from `a` to `b`.
fn edge_distance_to_segment(l: &Lattice, edge: usize, a: &[usize], b: &[usize]) -> f64 {
    let d = l.dims.len();
    let (site, axis) = (edge / d, edge % d);
    let mid: Vec<f64> = (0..d).map(|x| l.coord(site, x) as f64 + if x == axis { 0.5 } else { 0.0 }).collect();
    let (pa, pb): (Vec<f64>, Vec<f64>) = (a.iter().map(|&x| x as f64).collect(), b.iter().map(|&x| x as f64).collect());
    let ab: Vec<f64> = pa.iter().zip(&pb).map(|(x, y)| y - x).collect();
    let wrap = |x: f64, len: f64| { let mut v = x % len; if v > len / 2.0 { v -= len; } if v < -len / 2.0 { v += len; } v };
    let ap: Vec<f64> = (0..d).map(|x| wrap(mid[x] - pa[x], l.dims[x] as f64)).collect();
    let tt = (ap.iter().zip(&ab).map(|(p, q)| p * q).sum::<f64>() / ab.iter().map(|q| q * q).sum::<f64>()).clamp(0.0, 1.0);
    (0..d).map(|x| (ap[x] - tt * ab[x]).powi(2)).sum::<f64>()
}

/// Mean conductance on the segment's edges (within 0.5) against those at distance ≥ 5.
fn thread_ratio(o: &Outcome, a: &[usize], b: &[usize]) -> f64 {
    let (mut on, mut non, mut off, mut noff) = (0.0, 0.0, 0.0, 0.0);
    for (e, c) in o.carving.c.iter().enumerate() {
        let dd = edge_distance_to_segment(&o.lattice, e, a, b);
        if dd <= 0.25 {
            on += c;
            non += 1.0;
        } else if dd >= 25.0 {
            off += c;
            noff += 1.0;
        }
    }
    (on / non) / (off / noff)
}

/// The share of the excess conductance held by the top `k` edges.
fn selectivity(o: &Outcome, cfg: &Cfg, k: usize) -> f64 {
    let mut ex: Vec<f64> = o.carving.c.iter().map(|c| c - cfg.c_min).collect();
    let total: f64 = ex.iter().sum();
    ex.sort_by(|x, y| y.partial_cmp(x).unwrap());
    ex[..k].iter().sum::<f64>() / total
}

fn main() {
    let cfg = base();
    println!("T3: carving; kappa {}, mu {}, nu {}, c_min {}, c0 {}, steps {}", cfg.kappa, cfg.mu, cfg.nu, cfg.c_min, cfg.c0, cfg.steps);
    let (a2, b2) = (vec![27, 32], vec![37, 32]);
    let carved = simulate(&[64, 64], &a2, &b2, &cfg, 1);
    let flat = simulate(&[64, 64], &a2, &b2, &Cfg { carve: false, ..cfg }, 1);
    let r2 = thread_ratio(&carved, &a2, &b2);
    println!("C1 2D straight thread ratio {r2:.2}; C3 current carved {:.6} vs uncarved {:.6} (x{:.2})", carved.current, flat.current, carved.current / flat.current);
    // C2: remove the drive and watch the path erode.
    let mut k = carved.carving.clone();
    let path_edges: Vec<usize> = (0..k.c.len()).filter(|&e| edge_distance_to_segment(&carved.lattice, e, &a2, &b2) <= 0.25).collect();
    let excess = |k: &Carving| path_edges.iter().map(|&e| k.c[e] - cfg.c_min).sum::<f64>();
    let (e0, t_off) = (excess(&k), 300);
    let zero = vec![0.0; k.c.len()];
    for _ in 0..t_off {
        k.carve(&zero, &|_| cfg.mu, cfg.nu, cfg.c_min, cfg.p, 1.0);
    }
    let fitted = -(excess(&k) / e0).ln() / t_off as f64;
    println!("C2 erosion: fitted rate {fitted:.5} vs nu {} ({:+.1}%)", cfg.nu, (fitted / cfg.nu - 1.0) * 100.0);
    let (a3, b3) = (vec![13, 16, 16], vec![19, 16, 16]);
    let c3 = simulate(&[32, 32, 32], &a3, &b3, &cfg, 1);
    let r3 = thread_ratio(&c3, &a3, &b3);
    println!("C4 3D thread ratio {r3:.2}");
    // C5: finite-size convergence.
    let big2 = simulate(&[96, 96], &[43, 48], &[53, 48], &cfg, 1);
    let big3 = simulate(&[48, 48, 48], &[21, 24, 24], &[27, 24, 24], &cfg, 1);
    let (rb2, rb3) = (thread_ratio(&big2, &[43, 48], &[53, 48]), thread_ratio(&big3, &[21, 24, 24], &[27, 24, 24]));
    let conv = [(r2, rb2), (carved.current, big2.current), (r3, rb3), (c3.current, big3.current)];
    let c5 = conv.iter().all(|(x, y)| (x / y - 1.0).abs() < 0.02);
    println!("C5 convergence: 2D ratio {r2:.3} -> {rb2:.3}, current {:.5} -> {:.5}; 3D ratio {r3:.3} -> {rb3:.3}, current {:.5} -> {:.5}", carved.current, big2.current, c3.current, big3.current);

    // Optimization rounds on diagonal selectivity (dev (6,6); held-out (5,7)).
    let (ad, bd, bh) = (vec![26, 26], vec![32, 32], vec![31, 33]);
    let sel = |c: &Cfg, b: &[usize]| selectivity(&simulate(&[64, 64], &ad, b, c, 1), c, 12);
    let mut kept = base();
    let mut verdicts = Vec::new();
    for (name, variants) in [
        ("amplification p", vec![Cfg { p: 2.0, ..kept }, Cfg { p: 3.0, ..kept }]),
        ("chaos", Vec::new()),
        ("heat", Vec::new()),
    ] {
        let variants = match name {
            "chaos" => [0.1, 0.3].iter().flat_map(|&al| [false, true].map(|fz| Cfg { chaos_alpha: al, chaos_frozen: fz, ..kept })).collect(),
            "heat" => [0.005, 0.02].iter().map(|&t| Cfg { temperature: t, ..kept }).collect(),
            _ => variants,
        };
        let dev_off = sel(&kept, &bd);
        let mut best = (dev_off, kept);
        for v in &variants {
            let s = sel(v, &bd);
            println!("   {name} variant (p {}, chaos {} frozen {}, T {}): dev selectivity {s:.4}", v.p, v.chaos_alpha, v.chaos_frozen, v.temperature);
            if s > best.0 {
                best = (s, *v);
            }
        }
        let (h_off, h_on) = (sel(&kept, &bh), sel(&best.1, &bh));
        let keep = best.1 != kept && h_on > h_off;
        println!("   {name}: dev off {dev_off:.4}, best {:.4}; held-out {h_off:.4} -> {h_on:.4} -> {}", best.0, if keep { "KEPT" } else { "not kept" });
        verdicts.push(keep);
        if keep {
            kept = best.1;
        }
    }
    println!("kept: p {}, chaos {} (frozen {}), T {}", kept.p, kept.chaos_alpha, kept.chaos_frozen, kept.temperature);
    let v = [
        ("C1 a thread forms (ratio >= 3)", r2 >= 3.0),
        ("C2 erosion timescale within 10%", (fitted / cfg.nu - 1.0).abs() < 0.10),
        ("C3 carved web conducts better (x1.5)", carved.current / flat.current >= 1.5),
        ("C4 the same in 3D (ratio >= 3)", r3 >= 3.0),
        ("C5 finite-size convergence (< 2%)", c5),
        ("C6 amplification kept", verdicts[0]),
        ("C7 chaos kept", verdicts[1]),
        ("C8 heat not kept", !verdicts[2]),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold", v.iter().filter(|x| x.1).count(), v.len());
}
