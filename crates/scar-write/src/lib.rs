//! SCAR-1 (docs/SCAR-1-PRECOMMIT.md): write memories as scars. A memory is an assembly of cells; it
//! is carved by driven episodes — one member (the hub) held at +1 and the rest at −1 through damping
//! (T3's drive) — under T3's carving law (maintain-or-erase on each edge's conductance). The field
//! carries T2's bath at the writing temperature. Idling is the same law with nothing held. The
//! scar is the conductance change; nothing is added to the medium. Settings are the caller's.

use qb_chaos::keyed_unit;
use thermo_bath::{damp, exchange_edges, normal};
use thermo_carve::Carving;
use thermo_lattice::Lattice;

#[derive(Debug, Clone, Copy)]
pub struct Law {
    pub kappa: f64,
    pub mu: f64,
    pub nu: f64,
    pub c_min: f64,
    pub p: f64,
    /// Frozen per-edge chaos on μ (T3's kept heterogeneity).
    pub chaos: f64,
    pub temperature: f64,
}

fn hash_unit(key: u64) -> f64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 11) as f64 + 0.5) / (1u64 << 53) as f64
}

/// `steps` steps of the medium with the given sites held (site, value) and carving on, from step
/// index `t0` (for the noise keys). `f_ref` is the carving's flux reference.
pub fn run(l: &mut Lattice, k: &mut Carving, holds: &[(usize, f64)], steps: usize, law: &Law, f_ref: f64, seed: u64, t0: usize) {
    let d = l.dims.len();
    // A dense lookup of the held sites (identical results to a search; faster with many holds).
    let mut map: Vec<Option<f64>> = vec![None; l.e.len()];
    for &(s, v) in holds.iter().rev() {
        map[s] = Some(v); // reversed so the first listed hold wins, as the search did
    }
    let held = |s: usize| map[s];
    let hot = law.temperature > 0.0;
    for t in t0..t0 + steps {
        let z = |s: usize, tag: u64| {
            let key = seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (s as u64 * 16 + tag).wrapping_mul(0x1656_67B1_9E37_79F9);
            normal(hash_unit(key), hash_unit(key ^ 0x5555_5555_5555_5555))
        };
        damp(l, law.temperature, &|s| if held(s).is_some() { 1.0 } else { 0.0 }, &|s| held(s).unwrap_or(0.0), &|s| if hot { z(s, 1) } else { 0.0 });
        let c = &k.c;
        let flux = exchange_edges(l, law.temperature, &|s, a| law.kappa * c[s * d + a], &|_| 0.0, &|s, a| if hot { z(s, 2 + a as u64) } else { 0.0 });
        let mu = |e: usize| if law.chaos > 0.0 { law.mu * (1.0 + law.chaos * (keyed_unit((e as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ seed.wrapping_mul(31)) - 0.5)) } else { law.mu };
        k.carve(&flux, &mu, law.nu, law.c_min, law.p, f_ref);
    }
}

/// Carve the assemblies, interleaved round-robin: `rounds` passes of the hub through every member,
/// `episode` steps each, the field reset at each episode. Returns the step count used.
pub fn write(l: &mut Lattice, k: &mut Carving, assemblies: &[Vec<usize>], rounds: usize, episode: usize, law: &Law, seed: u64) -> usize {
    let mut t = 0;
    let geo = Lattice::new(&l.dims.clone());
    let dist = |a: usize, b: usize| -> f64 {
        (0..geo.dims.len())
            .map(|ax| {
                let (x, y) = (geo.coord(a, ax), geo.coord(b, ax));
                let dd = x.abs_diff(y);
                dd.min(geo.dims[ax] - dd) as f64
            })
            .sum()
    };
    let m = assemblies.iter().map(|a| a.len()).max().unwrap_or(0);
    for _ in 0..rounds {
        for h in 0..m {
            for a in assemblies {
                if h >= a.len() {
                    continue;
                }
                let hub = a[h];
                let holds: Vec<(usize, f64)> = a.iter().map(|&s| (s, if s == hub { 1.0 } else { -1.0 })).collect();
                let mean = a.iter().filter(|&&s| s != hub).map(|&s| dist(hub, s)).sum::<f64>() / (a.len() - 1) as f64;
                for x in l.e.iter_mut() {
                    *x = 0.0;
                }
                run(l, k, &holds, episode, law, law.kappa / mean, seed, t);
                t += episode;
            }
        }
    }
    t
}

/// Carve the given cell pairs one at a time, as T3 carved one thread: `a` held at +1 and `b` at −1
/// for `episode` steps, f_ref = κ / their distance, the field reset at each episode. The pairs are
/// carved in the order given (the caller interleaves memories). Returns the steps used.
pub fn write_pairs(l: &mut Lattice, k: &mut Carving, pairs: &[(usize, usize)], episode: usize, law: &Law, seed: u64, t0: usize) -> usize {
    let geo = Lattice::new(&l.dims.clone());
    let dist = |a: usize, b: usize| -> f64 {
        (0..geo.dims.len())
            .map(|ax| {
                let d = geo.coord(a, ax).abs_diff(geo.coord(b, ax));
                d.min(geo.dims[ax] - d) as f64
            })
            .sum()
    };
    let mut t = t0;
    for &(a, b) in pairs {
        for x in l.e.iter_mut() {
            *x = 0.0;
        }
        run(l, k, &[(a, 1.0), (b, -1.0)], episode, law, law.kappa / dist(a, b), seed, t);
        t += episode;
    }
    t - t0
}

/// `run`, with a hook called after every carving step (for a composed stack such as a budget).
pub fn run_with(l: &mut Lattice, k: &mut Carving, holds: &[(usize, f64)], steps: usize, law: &Law, f_ref: f64, seed: u64, t0: usize, after: &dyn Fn(&mut Carving)) {
    for t in t0..t0 + steps {
        run(l, k, holds, 1, law, f_ref, seed, t);
        after(k);
    }
}
