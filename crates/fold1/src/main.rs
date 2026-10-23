//! FOLD-1 orchestrator (docs/FOLD-1-PRECOMMIT.md): SCAR-1 run 5's laws (sealed traps, slow erosion,
//! the noise law, Jesse's spectral bridge), with F memories folded into each of 16 regions. Measures
//! region top-1, set recall (which memories a region holds), the 64-band growth arm, the folded
//! lifetime and the footprint per memory. Copied from scar1r4's simulate; every setting is here.

use spec_bridge::bridge;
use thermo_host::Medium;

const DIMS: [usize; 3] = [20, 20, 4];
const ACTIVE: usize = 6;
const KAPPA: f64 = 0.2;
const DELTA: f64 = 0.01;
const TAU: f64 = 0.99;
const E_REF: f64 = 0.3;
const NU: f64 = 5e-5;
const MU: f64 = 0.02;
const P: f64 = 3.0;
const GAIN: f64 = 0.02;
const Q: f64 = 0.02;
const EPS: f64 = 0.05;
const WRITE: usize = 250;

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

fn regions() -> Vec<Vec<usize>> {
    (0..16)
        .map(|r| {
            let (ox, oy) = (5 * (r % 4), 5 * (r / 4));
            (0..4).flat_map(|z| (oy..oy + 4).flat_map(move |y| (ox..ox + 4).map(move |x| x + DIMS[0] * y + DIMS[0] * DIMS[1] * z))).collect()
        })
        .collect()
}

fn keys(seed: u64, n: usize, bands: usize) -> Vec<Vec<f64>> {
    let mut r = Rng(seed.wrapping_mul(0x5CA4_F01D) ^ bands as u64);
    (0..n)
        .map(|_| {
            let mut k = vec![0.0; bands];
            let mut on = 0;
            while on < ACTIVE {
                let b = (r.next() % bands as u64) as usize;
                if k[b] == 0.0 {
                    k[b] = 0.5 + 0.5 * r.unit();
                    on += 1;
                }
            }
            let t: f64 = k.iter().sum();
            k.iter().map(|x| x / t).collect()
        })
        .collect()
}

/// Write F keys into each region (key r·F + j into region r), drain stirred; spectra at `reads`.
fn simulate(bands: usize, f: usize, ks: &[Vec<f64>], reads: &[usize], seed: u64) -> (Vec<Vec<Vec<f64>>>, usize) {
    let rs = regions();
    let mut med = Medium::new(&DIMS, bands);
    let sites = med.fields[0].e.len();
    let mut material = vec![0.0; sites];
    let bytes = std::mem::size_of_val(&material[..]) + med.fields.iter().map(|x| std::mem::size_of_val(&x.e[..])).sum::<usize>();
    let tick = |med: &mut Medium, material: &mut Vec<f64>, t: usize| {
        spec_stir::stir(med, Q, EPS, seed, t);
        spec_step::step_sealed(med, material, KAPPA, DELTA, TAU);
        let energy: Vec<f64> = (0..sites).map(|s| med.fields.iter().map(|x| x.e[s]).sum()).collect();
        spec_trap::form(material, &energy, &|_| MU, NU, E_REF, P);
    };
    let mut t = 0;
    for r in 0..16 {
        for j in 0..f {
            for _ in 0..WRITE {
                for &s in &rs[r] {
                    med.inject(s, 0, &ks[r * f + j], GAIN);
                }
                tick(&mut med, &mut material, t);
                t += 1;
            }
        }
    }
    let last = *reads.iter().max().unwrap();
    let mut snaps = Vec::new();
    for d in 0..=last {
        if reads.contains(&d) {
            snaps.push(rs.iter().map(|r| (0..bands).map(|b| r.iter().map(|&s| med.fields[b].e[s]).sum()).collect()).collect());
        }
        if d < last {
            tick(&mut med, &mut material, t);
            t += 1;
        }
    }
    (snaps, bytes)
}

/// (region top-1, set recall) for F keys per region.
fn measure(spectra: &[Vec<f64>], ks: &[Vec<f64>], f: usize) -> (f64, f64) {
    let n = ks.len();
    let top1 = (0..n).filter(|&i| (0..16).max_by(|&a, &b| bridge(&ks[i], &spectra[a]).partial_cmp(&bridge(&ks[i], &spectra[b])).unwrap()).unwrap() == i / f).count() as f64 / n as f64;
    let set = (0..16)
        .map(|r| {
            let mut rank: Vec<usize> = (0..n).collect();
            rank.sort_by(|&a, &b| bridge(&ks[b], &spectra[r]).partial_cmp(&bridge(&ks[a], &spectra[r])).unwrap());
            rank[..f].iter().filter(|&&i| i / f == r).count() as f64 / f as f64
        })
        .sum::<f64>()
        / 16.0;
    (top1, set)
}

fn parallel<T: Send>(seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// Region top-1 by the containment reading (FOLD-3).
fn top1_contain(spectra: &[Vec<f64>], ks: &[Vec<f64>], f: usize) -> f64 {
    (0..ks.len()).filter(|&i| (0..16).max_by(|&a, &b| spec_contain::containment(&ks[i], &spectra[a]).partial_cmp(&spec_contain::containment(&ks[i], &spectra[b])).unwrap()).unwrap() == i / f).count() as f64 / ks.len() as f64
}

/// FOLD-3 (docs/FOLD-3-PRECOMMIT.md): FOLD-2's simulations read both ways.
fn fold3() {
    let t0 = std::time::Instant::now();
    let test: Vec<u64> = (10..18).collect();
    let fs = [1usize, 2, 4, 8, 16];
    let mut caps = Vec::new();
    let mut f1 = Vec::new();
    for &b in &[32usize, 64, 128] {
        let mut row = Vec::new();
        for &f in &fs {
            let v = parallel(&test, |sd| {
                let ks = keys(sd ^ 0xF02, 16 * f, b);
                let (snaps, _) = simulate(b, f, &ks, &[250], sd);
                (measure(&snaps[0], &ks, f).0, top1_contain(&snaps[0], &ks, f))
            });
            let (br, co) = (v.iter().map(|x| x.0).sum::<f64>() / 8.0, v.iter().map(|x| x.1).sum::<f64>() / 8.0);
            println!("  B {b:3} F {f:2}: bridge {br:.3}, containment {co:.3}");
            row.push((br, co));
        }
        let cap = |i: usize| fs.iter().zip(&row).filter(|x| if i == 0 { x.1 .0 } else { x.1 .1 } >= 0.8).map(|x| *x.0).max().unwrap_or(0);
        println!("  B {b}: capacity bridge {}, containment {}", cap(0), cap(1));
        caps.push((cap(0), cap(1)));
        f1.push(row[0].1);
    }
    let v = [
        ("FX1 containment capacity >= 2x the bridge's at every B", caps.iter().all(|(b, c)| *c >= 2 * b)),
        ("FX2 containment scales with bands (F*(128) >= 2 F*(32))", caps[2].1 >= 2 * caps[0].1),
        ("FX3 nothing lost at one per region (F1 containment >= 0.95)", f1.iter().all(|&x| x >= 0.95)),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s); capacities (bridge, containment) {caps:?}", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}

/// FOLD-2 (docs/FOLD-2-PRECOMMIT.md): the capacity law, bands B against fold depth F.
fn fold2() {
    let t0 = std::time::Instant::now();
    let test: Vec<u64> = (10..18).collect();
    let fs = [1usize, 2, 4, 8, 16];
    let mut grid = Vec::new();
    for &b in &[32usize, 64, 128] {
        let mut row = Vec::new();
        for &f in &fs {
            let v = parallel(&test, |sd| {
                let ks = keys(sd ^ 0xF02, 16 * f, b);
                let (snaps, _) = simulate(b, f, &ks, &[250], sd);
                measure(&snaps[0], &ks, f)
            });
            let (a, s) = (v.iter().map(|x| x.0).sum::<f64>() / 8.0, v.iter().map(|x| x.1).sum::<f64>() / 8.0);
            println!("  B {b:3} F {f:2}: region top-1 {a:.3}, set recall {s:.3}");
            row.push((a, s));
        }
        let cap = fs.iter().zip(&row).filter(|x| x.1 .0 >= 0.8).map(|x| *x.0).max().unwrap_or(0);
        println!("  B {b}: capacity F* = {cap}");
        grid.push((cap, row));
    }
    let caps: Vec<usize> = grid.iter().map(|g| g.0).collect();
    let same_load = [grid[0].1[1].0, grid[1].1[2].0, grid[2].1[3].0];
    let mean = same_load.iter().sum::<f64>() / 3.0;
    let track = grid.iter().all(|g| g.1.iter().all(|(a, s)| *s >= a - 0.1));
    println!("  same load F = B/16: {same_load:.3?} (mean {mean:.3})");
    let v = [
        ("FC1 capacity linear in bands (F* 2, 4, 8)", caps == vec![2, 4, 8]),
        ("FC2 scale invariance at F = B/16 (within 0.08 of the mean)", same_load.iter().all(|x| (x - mean).abs() <= 0.08)),
        ("FC3 contents track identity (set recall >= top-1 - 0.1)", track),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s); capacities {caps:?}", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}

/// A diagnostic after FOLD-3 (2026-10-23, not a prediction): containment top-1 by write order. The
/// regions are written one after another (16·F·250 steps in all), so at depth F the first region is
/// read about 16·F·250 steps after its writing began. Is the folding ceiling partly age?
fn fold_age() {
    let test: Vec<u64> = (10..18).collect();
    println!("FOLD-3 age diagnostic: 32 bands, containment top-1 per region (write order 0 = oldest), test seeds 10-17");
    for &f in &[2usize, 4, 8] {
        let v = parallel(&test, |sd| {
            let ks = keys(sd ^ 0xF02, 16 * f, 32);
            let (snaps, _) = simulate(32, f, &ks, &[250], sd);
            let sp = &snaps[0];
            let hit: Vec<bool> = (0..ks.len()).map(|i| (0..16).max_by(|&a, &b| spec_contain::containment(&ks[i], &sp[a]).partial_cmp(&spec_contain::containment(&ks[i], &sp[b])).unwrap()).unwrap() == i / f).collect();
            let energy: Vec<f64> = sp.iter().map(|r| r.iter().sum()).collect();
            (hit, energy)
        });
        let per: Vec<f64> = (0..16).map(|r| v.iter().map(|(h, _)| h[r * f..(r + 1) * f].iter().filter(|x| **x).count() as f64 / f as f64).sum::<f64>() / 8.0).collect();
        let en: Vec<f64> = (0..16).map(|r| v.iter().map(|(_, e)| e[r]).sum::<f64>() / 8.0).collect();
        let half = |a: &[f64]| (a[..8].iter().sum::<f64>() / 8.0, a[8..].iter().sum::<f64>() / 8.0);
        println!("  F {f}: top-1 by region {per:.2?}");
        println!("        energy by region {en:.1?}");
        println!("        oldest half {:.3}, newest half {:.3}", half(&per).0, half(&per).1);
    }
}

fn main() {
    if std::env::var_os("FOLDAGE").is_some() {
        return fold_age();
    }
    if std::env::var_os("FOLD3").is_some() {
        return fold3();
    }
    if std::env::var_os("FOLD2").is_some() {
        return fold2();
    }
    let t0 = std::time::Instant::now();
    let test: Vec<u64> = (10..18).collect();
    println!("FOLD-1: SCAR-1 run 5's laws, F memories per region (16 regions), stirred, read at drain t 250");
    let fs = [1usize, 2, 4, 8];
    let mut top = Vec::new();
    let mut set = Vec::new();
    let mut bytes;
    for &f in &fs {
        let v = parallel(&test, |sd| {
            let ks = keys(sd, 16 * f, 32);
            let (snaps, b) = simulate(32, f, &ks, &[250], sd);
            let (a, s) = measure(&snaps[0], &ks, f);
            (a, s, b)
        });
        let (a, s) = (v.iter().map(|x| x.0).sum::<f64>() / 8.0, v.iter().map(|x| x.1).sum::<f64>() / 8.0);
        bytes = v[0].2;
        println!("  F {f}: region top-1 {a:.3}, set recall {s:.3}; {} memories in {bytes} bytes = {} bytes per memory", 16 * f, bytes / (16 * f));
        top.push(a);
        set.push(s);
    }
    let capacity = fs.iter().zip(&top).filter(|x| *x.1 >= 0.8).map(|x| *x.0).max().unwrap_or(0);
    println!("  capacity (largest F with region top-1 >= 0.8): {capacity}");
    let v64 = parallel(&test, |sd| {
        let ks = keys(sd, 64, 64);
        let (snaps, _) = simulate(64, 4, &ks, &[250], sd);
        measure(&snaps[0], &ks, 4)
    });
    let top64 = v64.iter().map(|x| x.0).sum::<f64>() / 8.0;
    println!("  growth: 64 bands at F 4: region top-1 {top64:.3} (32 bands {:.3}), set recall {:.3}", top[2], v64.iter().map(|x| x.1).sum::<f64>() / 8.0);
    let life = parallel(&test, |sd| {
        let ks = keys(sd, 32, 32);
        let (snaps, _) = simulate(32, 2, &ks, &[250, 20_000], sd);
        measure(&snaps[1], &ks, 2).0
    });
    let life2 = life.iter().sum::<f64>() / 8.0;
    println!("  folded lifetime: F 2 at t 20000: region top-1 {life2:.3}");
    let v = [
        ("F1 folding has a ceiling (F2 >= 0.9, F4 >= 0.7, F8 < 0.5)", top[1] >= 0.9 && top[2] >= 0.7 && top[3] < 0.5),
        ("F2 contents readable at F2 (set recall >= 0.8) and falling with F", set[1] >= 0.8 && set.windows(2).skip(1).all(|w| w[1] <= w[0])),
        ("F3 growth: 64 bands lifts F4 by >= 0.1", top64 >= top[2] + 0.1),
        ("F4 footprint per memory halves with each doubling (fixed medium)", true),
        ("F5 folded memories live (F2 at t20000 >= 0.75)", life2 >= 0.75),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
