//! SCAR-1 orchestrator (docs/SCAR-1-PRECOMMIT.md): memories carved as scars into one 3D medium
//! (`scar-write`), recalled by taps (`scar-tap`). Checks S1–S7, then the amplification, chaos and
//! cold-writing rounds. Every setting lives here; T3's kept settings are the plain law.

use scar_write::{run, write, Law};
use thermo_carve::Carving;
use thermo_lattice::Lattice;

const DIMS: [usize; 3] = [16, 16, 16];
const C0: f64 = 0.2;
const LEAK: f64 = 0.01;
const RECALL_STEPS: usize = 300;
const ROUNDS: usize = 2;
const EPISODE: usize = 100;
const POOL: usize = 48;
const M: usize = 6;
const PS: [usize; 5] = [1, 2, 4, 8, 16];

const PLAIN: Law = Law { kappa: 0.3, mu: 0.02, nu: 0.002, c_min: 0.05, p: 3.0, chaos: 0.3, temperature: 0.005 };

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

fn dist(l: &Lattice, a: usize, b: usize) -> f64 {
    (0..3)
        .map(|ax| {
            let d = l.coord(a, ax).abs_diff(l.coord(b, ax));
            (d.min(l.dims[ax] - d) as f64).powi(2)
        })
        .sum::<f64>()
        .sqrt()
}

/// The pool of cells (at least 3 apart) and 16 assemblies of 6 drawn from it.
fn memories(seed: u64) -> (Vec<usize>, Vec<Vec<usize>>) {
    let l = Lattice::new(&DIMS);
    let mut r = Rng(seed.wrapping_mul(0x5CA4_0001) ^ 0x51);
    let mut pool = Vec::new();
    while pool.len() < POOL {
        let s = r.below(l.e.len());
        if pool.iter().all(|&p| dist(&l, p, s) >= 3.0) {
            pool.push(s);
        }
    }
    let assemblies = (0..16)
        .map(|_| {
            let mut a = Vec::new();
            while a.len() < M {
                let c = pool[r.below(POOL)];
                if !a.contains(&c) {
                    a.push(c);
                }
            }
            a
        })
        .collect();
    (pool, assemblies)
}

/// (fidelity, settle step) of recalling `a` from its first `cue` members.
fn fidelity(k: &Carving, pool: &[usize], a: &[usize], cue: usize) -> (f64, usize) {
    let (ranked, settled) = scar_tap::recall(&DIMS, k, PLAIN.kappa, LEAK, pool, &a[..cue], M - cue, RECALL_STEPS);
    let hit = ranked[..M - cue].iter().filter(|s| a.contains(s)).count();
    (hit as f64 / (M - cue) as f64, settled)
}

/// Write the first P assemblies into a fresh medium; the medium and its scars.
fn carve(p: usize, law: &Law, seed: u64) -> (Lattice, Carving, Vec<usize>, Vec<Vec<usize>>) {
    let (pool, asm) = memories(seed);
    let mut l = Lattice::new(&DIMS);
    let mut k = Carving::uniform(l.e.len() * 3, C0);
    write(&mut l, &mut k, &asm[..p], ROUNDS, EPISODE, law, seed);
    (l, k, pool, asm)
}

/// Mean fidelity (cue 3) at each P, and the mean settle step.
fn sweep(law: &Law, seed: u64, ps: &[usize]) -> (Vec<f64>, f64) {
    let mut settle = Vec::new();
    let fids = ps
        .iter()
        .map(|&p| {
            let (_, k, pool, asm) = carve(p, law, seed);
            let v: Vec<(f64, usize)> = asm[..p].iter().map(|a| fidelity(&k, &pool, a, 3)).collect();
            settle.extend(v.iter().map(|x| x.1 as f64));
            v.iter().map(|x| x.0).sum::<f64>() / p as f64
        })
        .collect();
    (fids, settle.iter().sum::<f64>() / settle.len() as f64)
}

struct SeedResult {
    sweep: Vec<f64>,
    settle: f64,
    control: f64,
    completion: f64,
    idle: [f64; 3],
    fresh4: f64,
    bytes: (usize, usize),
}

fn per_seed(seed: u64) -> SeedResult {
    let (sweep_f, settle) = sweep(&PLAIN, seed, &PS);
    let (pool, asm) = memories(seed);
    let uncarved = Carving::uniform(DIMS.iter().product::<usize>() * 3, C0);
    let control = asm.iter().map(|a| fidelity(&uncarved, &pool, a, 3).0).sum::<f64>() / asm.len() as f64;
    let (_, k1, _, _) = carve(1, &PLAIN, seed);
    let completion = fidelity(&k1, &pool, &asm[0], 1).0;
    let (l4, k4, _, _) = carve(4, &PLAIN, seed);
    let bytes = (std::mem::size_of_val(&C0) * DIMS.iter().product::<usize>() * 3, std::mem::size_of_val(&k4.c[..]));
    let fresh4 = asm[..4].iter().map(|a| fidelity(&k4, &pool, a, 3).0).sum::<f64>() / 4.0;
    let mut idle = [0.0; 3];
    for (i, &t) in [0.0, 0.005, 0.02].iter().enumerate() {
        let (mut l, mut k) = (Lattice { dims: l4.dims.clone(), e: vec![0.0; l4.e.len()] }, k4.clone());
        run(&mut l, &mut k, &[], 3000, &Law { temperature: t, ..PLAIN }, PLAIN.kappa / 8.0, seed ^ 0x1D1E, 1_000_000);
        idle[i] = asm[..4].iter().map(|a| fidelity(&k, &pool, a, 3).0).sum::<f64>() / 4.0;
    }
    SeedResult { sweep: sweep_f, settle, control, completion, idle, fresh4, bytes }
}

fn parallel<T: Send>(seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// Diagnostic, added after run 1 (not a prediction): (a) what the writing carved; (b) whether the tap
/// reads ideal scars (conductance 1 on straight axis-paths between an assembly's members).
fn diagnose() {
    let seed = 10;
    let (l, k, pool, asm) = carve(1, &PLAIN, seed);
    let c = &k.c;
    let above = |x: f64| c.iter().filter(|&&v| v > x).count();
    println!("(a) after writing P1: max c {:.3}; edges with c > 0.3: {}, > 0.5: {}, > 0.9: {} (of {})", c.iter().cloned().fold(0.0, f64::max), above(0.3), above(0.5), above(0.9), c.len());
    let _ = l;
    // (b) ideal scars: walk axis by axis from each member to the next, setting c = 1.
    let geo = Lattice::new(&DIMS);
    let mut ideal = Carving::uniform(geo.e.len() * 3, C0);
    let a = &asm[0];
    for w in 0..M {
        let (from, to) = (a[w], a[(w + 1) % M]);
        let mut cur = from;
        for ax in 0..3 {
            while geo.coord(cur, ax) != geo.coord(to, ax) {
                let fwd = (geo.coord(to, ax) + DIMS[ax] - geo.coord(cur, ax)) % DIMS[ax] <= DIMS[ax] / 2;
                if fwd {
                    ideal.c[cur * 3 + ax] = 1.0;
                    cur = geo.forward(cur, ax);
                } else {
                    let back = if geo.coord(cur, ax) == 0 { cur + (DIMS[ax] - 1) * geo.stride(ax) } else { cur - geo.stride(ax) };
                    ideal.c[back * 3 + ax] = 1.0;
                    cur = back;
                }
            }
        }
    }
    for leak in [0.01, 0.001] {
        let (ranked, _) = scar_tap::recall(&DIMS, &ideal, PLAIN.kappa, leak, &pool, &a[..3], 3, RECALL_STEPS);
        let hit = ranked[..3].iter().filter(|s| a.contains(s)).count();
        println!("(b) ideal ring of threads, leak {leak}: recalled {hit} of 3 missing members");
    }
}

/// Diagnostic reading (after run 1, not a prediction): every non-cue cell is a cold receiver held at 0
/// (TA1's absorber); the cue is held at +1; the score is the current each cell absorbs through its
/// own edges after `steps`.
fn absorbed(k: &Carving, pool: &[usize], cue: &[usize], steps: usize) -> Vec<(usize, f64)> {
    use thermo_bath::{damp, exchange_edges};
    let mut l = Lattice::new(&DIMS);
    let is_cue = |s: usize| cue.contains(&s);
    let is_cell = |s: usize| pool.contains(&s);
    let mut flux = vec![0.0; l.e.len() * 3];
    for _ in 0..steps {
        damp(&mut l, 0.0, &|s| if is_cell(s) { 1.0 } else { 0.0 }, &|s| if is_cue(s) { 1.0 } else { 0.0 }, &|_| 0.0);
        flux = exchange_edges(&mut l, 0.0, &|s, a| PLAIN.kappa * k.c[s * 3 + a], &|_| 0.0, &|_, _| 0.0);
    }
    let mut inflow = vec![0.0; l.e.len()];
    for s in 0..l.e.len() {
        for a in 0..3 {
            let f = flux[s * 3 + a]; // > 0: flows into s from its forward neighbour
            inflow[s] += f;
            inflow[l.forward(s, a)] -= f;
        }
    }
    let mut r: Vec<(usize, f64)> = pool.iter().copied().filter(|&c| !is_cue(c)).map(|c| (c, inflow[c])).collect();
    r.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    r
}

fn main() {
    if std::env::var_os("SCAR_ABSORB").is_some() {
        // (c) ideal ring on an eroded background (c_min), (d) the run-1 writing, (e) 3x longer episodes.
        let seed = 10;
        let (pool, asm) = memories(seed);
        let a = &asm[0];
        let hits = |k: &Carving| absorbed(k, &pool, &a[..3], RECALL_STEPS)[..3].iter().filter(|x| a.contains(&x.0)).count();
        let geo = Lattice::new(&DIMS);
        let mut ideal = Carving::uniform(geo.e.len() * 3, PLAIN.c_min);
        for w in 0..M {
            let (mut cur, to) = (a[w], a[(w + 1) % M]);
            for ax in 0..3 {
                while geo.coord(cur, ax) != geo.coord(to, ax) {
                    if (geo.coord(to, ax) + DIMS[ax] - geo.coord(cur, ax)) % DIMS[ax] <= DIMS[ax] / 2 {
                        ideal.c[cur * 3 + ax] = 1.0;
                        cur = geo.forward(cur, ax);
                    } else {
                        let back = if geo.coord(cur, ax) == 0 { cur + (DIMS[ax] - 1) * geo.stride(ax) } else { cur - geo.stride(ax) };
                        ideal.c[back * 3 + ax] = 1.0;
                        cur = back;
                    }
                }
            }
        }
        println!("(c) ideal ring, eroded background, absorbing cells: {} of 3", hits(&ideal));
        let (_, k1, _, _) = carve(1, &PLAIN, seed);
        println!("(d) run-1 writing (100-step episodes), absorbing cells: {} of 3", hits(&k1));
        let mut l = Lattice::new(&DIMS);
        let mut k3 = Carving::uniform(l.e.len() * 3, C0);
        write(&mut l, &mut k3, &asm[..1], ROUNDS, 3 * EPISODE, &PLAIN, seed);
        println!("(e) 300-step episodes, absorbing cells: {} of 3; edges > 0.5: {}", hits(&k3), k3.c.iter().filter(|&&v| v > 0.5).count());
        let mut tot = [0usize; 2];
        for sd in 10..18u64 {
            let (pool, asm) = memories(sd);
            let a = &asm[0];
            let (_, k1, _, _) = carve(1, &PLAIN, sd);
            let mut l = Lattice::new(&DIMS);
            let mut k3 = Carving::uniform(l.e.len() * 3, C0);
            write(&mut l, &mut k3, &asm[..1], ROUNDS, 3 * EPISODE, &PLAIN, sd);
            for (i, k) in [&k1, &k3].iter().enumerate() {
                tot[i] += absorbed(k, &pool, &a[..3], RECALL_STEPS)[..3].iter().filter(|x| a.contains(&x.0)).count();
            }
        }
        println!("over test seeds 10-17, P1, absorbing reading: 100-step episodes {}/24, 300-step episodes {}/24", tot[0], tot[1]);
        return;
    }
    if std::env::var_os("SCAR_DIAG").is_some() {
        return diagnose();
    }
    let t0 = std::time::Instant::now();
    let test: Vec<u64> = (10..18).collect();
    let dev: Vec<u64> = (0..4).collect();
    println!("SCAR-1: scars in a 3D medium (side 16), pool {POOL} cells, assemblies of {M}; plain law {PLAIN:?}");
    let rs = parallel(&test, per_seed);
    let n = rs.len() as f64;
    let mean = |f: &dyn Fn(&SeedResult) -> f64| rs.iter().map(f).sum::<f64>() / n;
    let fold: Vec<f64> = (0..PS.len()).map(|i| mean(&|r| r.sweep[i])).collect();
    let (control, completion, fresh4, settle) = (mean(&|r| r.control), mean(&|r| r.completion), mean(&|r| r.fresh4), mean(&|r| r.settle));
    let idle: Vec<f64> = (0..3).map(|i| mean(&|r| r.idle[i])).collect();
    println!("S1 control (uncarved): fidelity {control:.3} (chance ~0.067)");
    println!("S3 folding: fidelity by P {:?}", PS.iter().zip(&fold).map(|(p, f)| format!("P{p} {f:.3}")).collect::<Vec<_>>());
    let capacity = PS.iter().zip(&fold).filter(|x| *x.1 >= 0.8).map(|x| *x.0).max().unwrap_or(0);
    println!("   capacity (largest P with fidelity >= 0.8): {capacity}");
    println!("S4 completion (P1, cue 1 of 6): {completion:.3}");
    println!("S5/S6 after 3000 idle steps (P4, fresh {fresh4:.3}): T 0 {:.3}, T 0.005 {:.3}, T 0.02 {:.3}", idle[0], idle[1], idle[2]);
    println!("S7 footprint: medium bytes before {} after {} writing; recall settles in {settle:.0} steps (mean)", rs[0].bytes.0, rs[0].bytes.1);
    // Rounds.
    let score = |law: &Law, seeds: &[u64]| -> f64 { parallel(seeds, |s| sweep(law, s, &[1, 4, 16]).0.iter().sum::<f64>() / 3.0).iter().sum::<f64>() / seeds.len() as f64 };
    let plain_dev = score(&PLAIN, &dev);
    let plain_test = score(&PLAIN, &test);
    println!("   rounds: plain capacity score dev {plain_dev:.4}, test {plain_test:.4}");
    let mut kept = Vec::new();
    for (name, law) in [("amplification off (p 1)", Law { p: 1.0, ..PLAIN }), ("chaos off", Law { chaos: 0.0, ..PLAIN }), ("cold writing (T 0)", Law { temperature: 0.0, ..PLAIN })] {
        let d = score(&law, &dev);
        let keep = if d > plain_dev { score(&law, &test) > plain_test } else { false };
        println!("   {name}: dev {d:.4} -> {}", if keep { "KEPT (beats plain on dev and test)" } else { "not kept" });
        kept.push(keep);
    }
    let v = [
        ("S1 control <= 0.2", control <= 0.2),
        ("S2 one scar recalls (P1 >= 0.9)", fold[0] >= 0.9),
        ("S3 folding: >= 0.8 to P4, and P16 < P1", fold[2] >= 0.8 && fold[1] >= 0.8 && fold[4] < fold[0]),
        ("S4 completion from 1 of 6 >= 0.5", completion >= 0.5),
        ("S5 scars fade without use (idle T0 <= half fresh)", idle[0] <= 0.5 * fresh4),
        ("S6 noise keeps them alive (best T >= T0 + 0.1)", idle[1].max(idle[2]) >= idle[0] + 0.1),
        ("S7 zero extra footprint", rs[0].bytes.0 == rs[0].bytes.1),
        ("S8 amplification (p 3) kept over p 1", !kept[0]),
        ("S9 frozen chaos kept over none", !kept[1]),
        ("S10 warm writing kept over cold", !kept[2]),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
