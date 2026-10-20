//! SCAR-1 run 2 orchestrator (docs/SCAR-1-PRECOMMIT.md, run 2): Jesse's C17 design built from
//! scratch. A memory's key (16 bands on the input face) is carved into its own region (a 4×4 patch
//! on the output face) by `scar-write`, with a compound-c-like balance and the `thermo-budget`
//! competition; recall reads the absorbed current per region (`scar-tap::recall_regions`). Checks
//! S1–S8, then the rounds. Every setting lives here.

use scar_write::{run_with, Law};
use thermo_budget::{cap, Incidence};
use thermo_carve::Carving;
use thermo_lattice::Lattice;

const DIMS: [usize; 3] = [24, 16, 16];
const C0: f64 = 0.2;
const BUDGET: f64 = 1.5;
const EPISODE: usize = 300;
const ROUNDS: usize = 4;
const RECALL: usize = 300;
const KEY: usize = 16;
const PS: [usize; 5] = [1, 2, 4, 8, 16];

const PLAIN: Law = Law { kappa: 0.3, mu: 0.05, nu: 2e-4, c_min: 0.05, p: 3.0, chaos: 0.3, temperature: 0.005 };

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn site(x: usize, y: usize, z: usize) -> usize {
    x + DIMS[0] * y + DIMS[0] * DIMS[1] * z
}

/// The 256 bands of the input face.
fn bands() -> Vec<usize> {
    (0..16).flat_map(|z| (0..16).map(move |y| site(0, y, z))).collect()
}

/// The output face sits halfway round the periodic x axis (x = 12): 12 steps from the bands either
/// way. Run 2's first attempt put it at x = 23, one wrap-step from the bands (VOID).
const OUT_X: usize = DIMS[0] / 2;

/// The 16 regions: 4×4 patches of the output face.
fn regions() -> Vec<Vec<usize>> {
    (0..16).map(|r| { let (py, pz) = (4 * (r % 4), 4 * (r / 4)); (0..4).flat_map(move |dz| (0..4).map(move |dy| site(OUT_X, py + dy, pz + dz))).collect() }).collect()
}

/// 16 keys of 16 bands each.
fn keys(seed: u64) -> Vec<Vec<usize>> {
    let b = bands();
    let mut r = Rng(seed.wrapping_mul(0x5CA4_0002) ^ 0x77);
    (0..16)
        .map(|_| {
            let mut k = Vec::new();
            while k.len() < KEY {
                let x = b[r.below(b.len())];
                if !k.contains(&x) {
                    k.push(x);
                }
            }
            k
        })
        .collect()
}

/// Write the first P memories (key m → region m), interleaved, with or without the budget.
fn write(p: usize, law: &Law, budget: bool, seed: u64) -> Carving {
    let (ks, rs) = (keys(seed), regions());
    let mut l = Lattice::new(&DIMS);
    let inc = Incidence::new(&l);
    let mut k = Carving::uniform(l.e.len() * 3, C0);
    let hook = |k: &mut Carving| if budget { cap(&mut k.c, &inc, BUDGET, law.c_min) };
    let mut t = 0;
    for _ in 0..ROUNDS {
        for m in 0..p {
            let holds: Vec<(usize, f64)> = ks[m].iter().map(|&s| (s, 1.0)).chain(rs[m].iter().map(|&s| (s, -1.0))).collect();
            for x in l.e.iter_mut() {
                *x = 0.0;
            }
            run_with(&mut l, &mut k, &holds, EPISODE, law, law.kappa / OUT_X as f64, seed, t, &hook);
            t += EPISODE;
        }
    }
    k
}

/// Top-1 accuracy and mean margin for recalling the first P memories with cues made by `cue`.
fn recall(k: &Carving, p: usize, seed: u64, cue: &dyn Fn(&[usize], &mut Rng) -> Vec<usize>) -> (f64, f64) {
    let (ks, rs) = (keys(seed), regions());
    let mut r = Rng(seed ^ 0xC0E);
    let (mut hit, mut margin) = (0usize, 0.0);
    for m in 0..p {
        let c = cue(&ks[m], &mut r);
        let cur = scar_tap::recall_regions(&DIMS, k, PLAIN.kappa, &c, &rs, RECALL);
        let best = (0..16).max_by(|&a, &b| cur[a].partial_cmp(&cur[b]).unwrap()).unwrap();
        hit += usize::from(best == m);
        let other = (0..16).filter(|&j| j != m).map(|j| cur[j]).fold(f64::MIN, f64::max);
        margin += (cur[m] - other) / cur[m].abs().max(1e-300);
    }
    (hit as f64 / p as f64, margin / p as f64)
}

fn half(k: &[usize], _: &mut Rng) -> Vec<usize> {
    k[..KEY / 2].to_vec()
}

struct Seed {
    sweep: Vec<f64>,
    control: f64,
    completion: f64,
    noisy: f64,
    no_budget: f64,
    idle: [f64; 2],
    fresh: f64,
    bytes: (usize, usize),
}

fn per_seed(seed: u64) -> Seed {
    let sweep: Vec<f64> = PS.iter().map(|&p| recall(&write(p, &PLAIN, true, seed), p, seed, &half).0).collect();
    let uncarved = Carving::uniform(DIMS.iter().product::<usize>() * 3, C0);
    let control = recall(&uncarved, 16, seed, &half).0;
    let k16 = write(16, &PLAIN, true, seed);
    let fresh = recall(&k16, 16, seed, &half).0;
    let completion = recall(&k16, 16, seed, &|k, _| k[..4].to_vec()).0;
    let all = bands();
    let noisy = recall(&k16, 16, seed, &|k, r| {
        let mut c = k[..KEY / 2].to_vec();
        while c.len() < KEY / 2 + 8 {
            let x = all[r.below(all.len())];
            if !k.contains(&x) && !c.contains(&x) {
                c.push(x);
            }
        }
        c
    })
    .0;
    let no_budget = recall(&write(16, &PLAIN, false, seed), 16, seed, &half).0;
    let mut idle = [0.0; 2];
    let inc = Incidence::new(&Lattice::new(&DIMS));
    for (i, &t) in [0.0, 0.005].iter().enumerate() {
        let (mut l, mut k) = (Lattice::new(&DIMS), k16.clone());
        run_with(&mut l, &mut k, &[], 10_000, &Law { temperature: t, ..PLAIN }, PLAIN.kappa / OUT_X as f64, seed ^ 0x1D1E, 1_000_000, &|k: &mut Carving| cap(&mut k.c, &inc, BUDGET, PLAIN.c_min));
        idle[i] = recall(&k, 16, seed, &half).0;
    }
    let n = DIMS.iter().product::<usize>() * 3 * 8;
    Seed { sweep, control, completion, noisy, no_budget, idle, fresh, bytes: (n, std::mem::size_of_val(&k16.c[..])) }
}

fn parallel<T: Send>(seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn main() {
    let t0 = std::time::Instant::now();
    let (test, dev): (Vec<u64>, Vec<u64>) = ((10..18).collect(), (0..4).collect());
    println!("SCAR-1 run 2: keys carved into their own regions; medium {DIMS:?}; plain law {PLAIN:?}, budget {BUDGET}");
    let rs = parallel(&test, per_seed);
    let n = rs.len() as f64;
    let mean = |f: &dyn Fn(&Seed) -> f64| rs.iter().map(f).sum::<f64>() / n;
    let curve: Vec<f64> = (0..PS.len()).map(|i| mean(&|r| r.sweep[i])).collect();
    let (control, completion, noisy, no_budget, fresh) = (mean(&|r| r.control), mean(&|r| r.completion), mean(&|r| r.noisy), mean(&|r| r.no_budget), mean(&|r| r.fresh));
    let idle = [mean(&|r| r.idle[0]), mean(&|r| r.idle[1])];
    println!("S1 control (uncarved): top-1 {control:.3} (chance 0.0625)");
    println!("S2 capacity (half-key cue): {:?}", PS.iter().zip(&curve).map(|(p, f)| format!("P{p} {f:.3}")).collect::<Vec<_>>());
    println!("S3 completion (4 of 16 bands, P16): {completion:.3}");
    println!("S4 noisy cue (8 of 16 + 8 wrong bands, P16): {noisy:.3}");
    println!("S5 budget off (P16): {no_budget:.3} against on {fresh:.3}");
    println!("S6/S7 after 10,000 idle steps (P16, fresh {fresh:.3}): T 0 {:.3}, T 0.005 {:.3}", idle[0], idle[1]);
    println!("S8 footprint: {} bytes before, {} after", rs[0].bytes.0, rs[0].bytes.1);
    let score = |law: &Law, seeds: &[u64]| -> (f64, f64) {
        let v = parallel(seeds, |s| recall(&write(16, law, true, s), 16, s, &half));
        (v.iter().map(|x| x.0).sum::<f64>() / v.len() as f64, v.iter().map(|x| x.1).sum::<f64>() / v.len() as f64)
    };
    let (pd, pt) = (score(&PLAIN, &dev), score(&PLAIN, &test));
    println!("   rounds: plain dev {:.3} (margin {:.4}), test {:.3} (margin {:.4})", pd.0, pd.1, pt.0, pt.1);
    let better = |a: (f64, f64), b: (f64, f64)| a.0 > b.0 || (a.0 == b.0 && a.1 > b.1);
    let mut kept = Vec::new();
    for (name, law) in [("amplification off (p 1)", Law { p: 1.0, ..PLAIN }), ("chaos off", Law { chaos: 0.0, ..PLAIN }), ("cold writing (T 0)", Law { temperature: 0.0, ..PLAIN })] {
        let d = score(&law, &dev);
        let keep = better(d, pd) && better(score(&law, &test), pt);
        println!("   {name}: dev {:.3} (margin {:.4}) -> {}", d.0, d.1, if keep { "KEPT" } else { "not kept" });
        kept.push(keep);
    }
    let v = [
        ("S1 control <= 0.15", control <= 0.15),
        ("S2 capacity: P16 top-1 >= 0.9", curve[4] >= 0.9),
        ("S3 completion from 4 of 16 >= 0.6", completion >= 0.6),
        ("S4 noisy cue >= 0.8", noisy >= 0.8),
        ("S5 the budget is load-bearing (off falls >= 0.2)", fresh - no_budget >= 0.2),
        ("S6 fading: idle T0 < 0.5", idle[0] < 0.5),
        ("S7 noise keeps them alive (T 0.005 >= T0 + 0.2)", idle[1] >= idle[0] + 0.2),
        ("S8 zero extra footprint", rs[0].bytes.0 == rs[0].bytes.1),
        ("S9 amplification not dropped", !kept[0]),
        ("S10 chaos not dropped", !kept[1]),
        ("S11 warm writing not dropped", !kept[2]),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
