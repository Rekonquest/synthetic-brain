//! FOLD-4 orchestrator (docs/FOLD-4-PRECOMMIT.md): the best folding in the thermal state at a fixed
//! medium (32 bands, 422,400 bytes). SCAR-1 run 5's laws, as in fold1; every setting is here.
//! - the plain code (FOLD-3's): a k-band profile at every site of its region, read as the region's
//!   32-band spectrum; k 6 reproduces FOLD-3 exactly;
//! - the shape code: 8 of the region's 64 sites, each with its own 6-band profile, read over the
//!   region's 64 × 32 cells;
//! - hot/cold: the stir rate q; rounds: write gain × 2, and per-step chaos on the write gain.
//! `PART` = sparsity | shape | heat | rounds (default: all, in that order).

use spec_bridge::bridge;
use spec_contain::containment;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use thermo_host::Medium;

const DIMS: [usize; 3] = [20, 20, 4];
const BANDS: usize = 32;
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
const SHAPE_SITES: usize = 8;
const SHAPE_BANDS: usize = 6;
const THREADS: usize = 8;

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Code {
    /// k active bands, written at every site of the region (FOLD-3's code).
    Plain(usize),
    /// SHAPE_SITES sites of the region, each with its own SHAPE_BANDS-band profile.
    Shape,
}

/// A key: per written site (local index in the region), its band profile, and the cue it is read by.
struct Key {
    sites: Vec<(usize, Vec<f64>)>,
    cue: Vec<f64>,
}

/// One random profile of `k` bands with weights in [0.5, 1), drawn exactly as fold1's `keys`.
fn profile(r: &mut Rng, k: usize) -> Vec<f64> {
    let mut p = vec![0.0; BANDS];
    let mut on = 0;
    while on < k {
        let b = (r.next() % BANDS as u64) as usize;
        if p[b] == 0.0 {
            p[b] = 0.5 + 0.5 * r.unit();
            on += 1;
        }
    }
    p
}

fn keys(code: Code, seed: u64, n: usize) -> Vec<Key> {
    match code {
        Code::Plain(k) => {
            // fold1's draw (seed ^ 0xF02 as FOLD-2/3), so k 6 reproduces FOLD-3.
            let mut r = Rng((seed ^ 0xF02).wrapping_mul(0x5CA4_F01D) ^ BANDS as u64);
            (0..n)
                .map(|_| {
                    let p = profile(&mut r, k);
                    let t: f64 = p.iter().sum();
                    let p: Vec<f64> = p.iter().map(|x| x / t).collect();
                    Key { sites: (0..64).map(|s| (s, p.clone())).collect(), cue: p }
                })
                .collect()
        }
        Code::Shape => {
            let mut r = Rng((seed ^ 0x5A9E).wrapping_mul(0x5CA4_F01D) ^ BANDS as u64);
            (0..n)
                .map(|_| {
                    let mut chosen: Vec<usize> = Vec::new();
                    while chosen.len() < SHAPE_SITES {
                        let s = (r.next() % 64) as usize;
                        if !chosen.contains(&s) {
                            chosen.push(s);
                        }
                    }
                    let mut sites: Vec<(usize, Vec<f64>)> = chosen.iter().map(|&s| (s, profile(&mut r, SHAPE_BANDS))).collect();
                    let t: f64 = sites.iter().map(|(_, p)| p.iter().sum::<f64>()).sum();
                    // The plain code writes a profile summing to 1 at each of 64 sites per step; the
                    // shape writes the same total (64) over its own sites.
                    for (_, p) in sites.iter_mut() {
                        for x in p.iter_mut() {
                            *x *= 64.0 / t;
                        }
                    }
                    let mut cue = vec![0.0; 64 * BANDS];
                    for (s, p) in &sites {
                        cue[s * BANDS..(s + 1) * BANDS].copy_from_slice(p);
                    }
                    Key { sites, cue }
                })
                .collect()
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Setup {
    code: Code,
    f: usize,
    q: f64,
    gain: f64,
    /// Chaos amplitude on the write gain per (step, site); 0 = off.
    alpha: f64,
}

impl Setup {
    fn new(code: Code, f: usize) -> Setup {
        Setup { code, f, q: Q, gain: GAIN, alpha: 0.0 }
    }
}

/// Each region's 64 × 32 cells (site-major, in the region's site order) at each read time.
type Snaps = Vec<Vec<Vec<f64>>>;

/// Write F keys into each region (key r·F + j into region r), then drain; cells at `reads`.
fn simulate(st: &Setup, ks: &[Key], reads: &[usize], seed: u64) -> Snaps {
    let rs = regions();
    let mut med = Medium::new(&DIMS, BANDS);
    let sites = med.fields[0].e.len();
    let mut material = vec![0.0; sites];
    let tick = |med: &mut Medium, material: &mut Vec<f64>, t: usize| {
        spec_stir::stir(med, st.q, EPS, seed, t);
        spec_step::step_sealed(med, material, KAPPA, DELTA, TAU);
        let energy: Vec<f64> = (0..sites).map(|s| med.fields.iter().map(|x| x.e[s]).sum()).collect();
        spec_trap::form(material, &energy, &|_| MU, NU, E_REF, P);
    };
    let mut t = 0;
    for r in 0..16 {
        for j in 0..st.f {
            let key = &ks[r * st.f + j];
            for _ in 0..WRITE {
                for (local, p) in &key.sites {
                    let s = rs[r][*local];
                    let factor = if st.alpha > 0.0 {
                        let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (s as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                        1.0 + st.alpha * (qb_chaos::keyed_unit(k) - 0.5)
                    } else {
                        1.0
                    };
                    med.inject(s, 0, p, st.gain * factor);
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
            snaps.push(rs.iter().map(|r| r.iter().flat_map(|&s| (0..BANDS).map(move |b| (s, b))).map(|(s, b)| med.fields[b].e[s]).collect()).collect());
        }
        if d < last {
            tick(&mut med, &mut material, t);
            t += 1;
        }
    }
    snaps
}

/// What a region is read as: its 32-band spectrum (plain code, summed over sites in fold1's order)
/// or its cells (shape code).
fn reading(code: Code, cells: &[f64]) -> Vec<f64> {
    match code {
        Code::Plain(_) => (0..BANDS).map(|b| (0..64).map(|s| cells[s * BANDS + b]).sum()).collect(),
        Code::Shape => cells.to_vec(),
    }
}

/// Per key: did containment find its region; did the bridge. Plus set recall (containment).
struct Scores {
    hit_contain: Vec<bool>,
    hit_bridge: Vec<bool>,
    set: f64,
}

fn score(code: Code, cells: &[Vec<f64>], ks: &[Key], f: usize) -> Scores {
    let rd: Vec<Vec<f64>> = cells.iter().map(|c| reading(code, c)).collect();
    let top = |sc: &dyn Fn(&[f64], &[f64]) -> f64| -> Vec<bool> { (0..ks.len()).map(|i| (0..16).max_by(|&a, &b| sc(&ks[i].cue, &rd[a]).partial_cmp(&sc(&ks[i].cue, &rd[b])).unwrap()).unwrap() == i / f).collect() };
    let hit_contain = top(&|c, r| containment(c, r));
    let hit_bridge = top(&|c, r| bridge(c, r));
    let n = ks.len();
    let set = (0..16)
        .map(|r| {
            let sc: Vec<f64> = (0..n).map(|i| containment(&ks[i].cue, &rd[r])).collect();
            let mut rank: Vec<usize> = (0..n).collect();
            rank.sort_by(|&a, &b| sc[b].partial_cmp(&sc[a]).unwrap());
            rank[..f].iter().filter(|&&i| i / f == r).count() as f64 / f as f64
        })
        .sum::<f64>()
        / 16.0;
    Scores { hit_contain, hit_bridge, set }
}

fn frac(h: &[bool]) -> f64 {
    h.iter().filter(|x| **x).count() as f64 / h.len() as f64
}

/// One configuration over seeds: mean containment top-1, bridge top-1, set recall per read, and
/// containment top-1 for the oldest and newest 8 regions at the first read.
#[derive(Clone, Debug)]
struct Row {
    contain: Vec<f64>,
    bridge: Vec<f64>,
    set: Vec<f64>,
    old_new: (f64, f64),
}

/// Run every (setup, seed) job on THREADS workers; rows in setup order.
fn run(setups: &[Setup], seeds: &[u64], reads: &[usize]) -> Vec<Row> {
    let jobs: Vec<(usize, u64)> = (0..setups.len()).flat_map(|i| seeds.iter().map(move |&s| (i, s))).collect();
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<Option<Vec<Scores>>>> = jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (i, sd) = jobs[j];
                let st = &setups[i];
                let ks = keys(st.code, sd, 16 * st.f);
                let snaps = simulate(st, &ks, reads, sd);
                *out[j].lock().unwrap() = Some(snaps.iter().map(|c| score(st.code, c, &ks, st.f)).collect());
            });
        }
    });
    let res: Vec<Vec<Scores>> = out.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect();
    let n = seeds.len() as f64;
    (0..setups.len())
        .map(|i| {
            let mine: Vec<&Vec<Scores>> = (0..seeds.len()).map(|k| &res[i * seeds.len() + k]).collect();
            let f = setups[i].f;
            let per = |g: &dyn Fn(&Scores) -> f64| -> Vec<f64> { (0..reads.len()).map(|r| mine.iter().map(|s| g(&s[r])).sum::<f64>() / n).collect() };
            let half = |lo: usize| mine.iter().map(|s| frac(&s[0].hit_contain[lo * f..(lo + 8) * f])).sum::<f64>() / n;
            Row { contain: per(&|s| frac(&s.hit_contain)), bridge: per(&|s| frac(&s.hit_bridge)), set: per(&|s| s.set), old_new: (half(0), half(8)) }
        })
        .collect()
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

const BYTES: usize = 1600 * (BANDS + 1) * 8;

fn capacity(fs: &[usize], top: &[f64]) -> usize {
    fs.iter().zip(top).filter(|x| *x.1 >= 0.8).map(|x| *x.0).max().unwrap_or(0)
}

fn main() {
    let t0 = std::time::Instant::now();
    let part = std::env::var("PART").unwrap_or_default();
    let all = part.is_empty();
    let test: Vec<u64> = (10..18).collect();
    let dev: Vec<u64> = (0..4).collect();
    let mut verdicts: Vec<(String, bool)> = Vec::new();
    say(format!("FOLD-4: 32 bands, fixed medium {BYTES} bytes, SCAR-1 run 5's laws; test seeds 10-17, read at drain t 250"));

    // Lever 1: sparsity (plain code).
    let mut plain_cap = 2usize;
    if all || part == "sparsity" {
        say("\nLever 1, key sparsity (plain code; containment top-1, bridge, set recall):".into());
        let fs = [2usize, 4, 8];
        let ksz = [3usize, 6, 12];
        let setups: Vec<Setup> = ksz.iter().flat_map(|&k| fs.iter().map(move |&f| Setup::new(Code::Plain(k), f))).collect();
        let rows = run(&setups, &test, &[250]);
        let at = |k: usize, f: usize| rows[ksz.iter().position(|&x| x == k).unwrap() * 3 + fs.iter().position(|&x| x == f).unwrap()].contain[0];
        for (st, r) in setups.iter().zip(&rows) {
            say(format!("  {:?} F {:2}: contain {:.3}, bridge {:.3}, set {:.3}", st.code, st.f, r.contain[0], r.bridge[0], r.set[0]));
        }
        let caps: Vec<usize> = ksz.iter().map(|&k| capacity(&fs, &fs.iter().map(|&f| at(k, f)).collect::<Vec<_>>())).collect();
        say(format!("  capacities (k 3, 6, 12): {caps:?}"));
        plain_cap = *caps.iter().max().unwrap();
        let best4 = ksz.iter().map(|&k| at(k, 4)).fold(0.0, f64::max);
        verdicts.push(("P1 best k follows the Bloom optimum (k12 >= k6 at F2; k3 > k6 + 0.05 at F8; k6 within 0.03 of best at F4)".into(), at(12, 2) >= at(6, 2) && at(3, 8) > at(6, 8) + 0.05 && at(6, 4) >= best4 - 0.03));
        verdicts.push(("P2 sparsity cannot lift the 32-band capacity above 4".into(), plain_cap <= 4));
    }

    // Lever 2: the shape code.
    let mut shape_cap = 0usize;
    if all || part == "shape" {
        say("\nLever 2, the shape code (8 sites x 6 bands of the region's 64 x 32 cells), and the plain code at F 16, 32:".into());
        let fs = [1usize, 2, 4, 8, 16, 32];
        let mut setups: Vec<Setup> = fs.iter().map(|&f| Setup::new(Code::Shape, f)).collect();
        setups.push(Setup::new(Code::Plain(6), 16));
        setups.push(Setup::new(Code::Plain(6), 32));
        let rows = run(&setups, &test, &[250]);
        for (st, r) in setups.iter().zip(&rows) {
            say(format!(
                "  {:?} F {:2}: contain {:.3}, bridge {:.3}, set {:.3}; oldest half {:.3}, newest half {:.3}; {} memories, {} bytes each",
                st.code, st.f, r.contain[0], r.bridge[0], r.set[0], r.old_new.0, r.old_new.1, 16 * st.f, BYTES / (16 * st.f)
            ));
        }
        let top: Vec<f64> = rows[..fs.len()].iter().map(|r| r.contain[0]).collect();
        shape_cap = capacity(&fs, &top);
        let shape_rows: Vec<(usize, Row)> = fs.iter().cloned().zip(rows[..fs.len()].iter().cloned()).collect();
        say(format!("  shape capacity: {shape_cap} per region ({} memories, {} bytes each)", 16 * shape_cap, BYTES / (16 * shape_cap.max(1))));
        let get = |f: usize| shape_rows.iter().find(|x| x.0 == f).map(|x| x.1.clone()).unwrap();
        verdicts.push(("P3 shape capacity >= 16 per region".into(), shape_cap >= 16));
        verdicts.push(("P4 shape at F4 >= 0.95".into(), get(4).contain[0] >= 0.95));
        verdicts.push(("P5 shape set recall >= 0.8 at F16".into(), get(16).set[0] >= 0.8));
        let fail = fs.iter().filter(|&&f| get(f).contain[0] < 0.8).cloned().max();
        let p6 = fail.map(|f| get(f).old_new.0 < get(f).old_new.1);
        say(format!("  P6 deepest failing F: {fail:?}, oldest/newest {:?}", fail.map(|f| get(f).old_new)));
        verdicts.push(("P6 the shape's limit is spread: oldest half worse at the deepest failing F".into(), p6.unwrap_or(false)));
    }

    // The winner for the heat arm: most memories per byte at >= 0.8 (shape if its capacity is larger).
    let (code, depth) = if shape_cap > plain_cap { (Code::Shape, shape_cap.min(16)) } else { (Code::Plain(6), plain_cap.max(2)) };
    let code = match std::env::var("HEAT_CODE").ok().as_deref() {
        Some("shape") => Code::Shape,
        Some("plain") => Code::Plain(6),
        _ => code,
    };
    let depth = std::env::var("HEAT_F").ok().and_then(|x| x.parse().ok()).unwrap_or(depth);
    let mut kept_q = Q;
    if all || part == "heat" || part == "rounds" {
        say(format!("\nLever 3, hot/cold (stir rate q) on {code:?} at F {depth}, read at t 250 and t 20000:"));
        let qs = [0.0, 0.0025, 0.005, 0.01, 0.02, 0.04];
        let setups: Vec<Setup> = qs.iter().map(|&q| Setup { q, ..Setup::new(code, depth) }).collect();
        let rows = run(&setups, &test, &[250, 20_000]);
        for (st, r) in setups.iter().zip(&rows) {
            say(format!("  q {:.4}: t 250 contain {:.3} (set {:.3}); t 20000 contain {:.3} (set {:.3})", st.q, r.contain[0], r.set[0], r.contain[1], r.set[1]));
        }
        let fresh: Vec<f64> = rows.iter().map(|r| r.contain[0]).collect();
        let late: Vec<f64> = rows.iter().map(|r| r.contain[1]).collect();
        // P7: from q 0.04 down to 0.0025 (indices 5 -> 1), top-1 does not fall.
        let p7 = (1..5).all(|i| fresh[i] >= fresh[i + 1]);
        let best = (1..6).max_by(|&a, &b| late[a].partial_cmp(&late[b]).unwrap().then(b.cmp(&a))).unwrap();
        let p8 = late[0] < late[4] - 0.3 && (qs[best] == 0.005 || qs[best] == 0.01);
        verdicts.push(("P7 colder is better fresh (t 250 does not fall from q 0.04 to 0.0025)".into(), p7));
        verdicts.push(("P8 inverted U at t 20000: quiet collapses, best q 0.005 or 0.01".into(), p8));
        // Kept (labelled: chosen on the test seeds' measurement): strictly better than plain q at t 20000, then t 250.
        let plain_i = 4;
        let better = |i: usize| late[i] > late[plain_i] || (late[i] == late[plain_i] && fresh[i] > fresh[plain_i]);
        if better(best) {
            kept_q = qs[best];
        }
        say(format!("  kept q: {kept_q} (best at t 20000: q {})", qs[best]));
    }
    if let Some(q) = std::env::var("KEPT_Q").ok().and_then(|x| x.parse().ok()) {
        kept_q = q;
    }

    if all || part == "rounds" {
        say(format!("\nRounds on {code:?} F {depth} q {kept_q} (dev seeds 0-3, t 20000 then t 250; kept only if better on test; plain wins ties):"));
        let base = Setup { q: kept_q, ..Setup::new(code, depth) };
        let arms = [("amplification: gain x 2", Setup { gain: 2.0 * GAIN, ..base }), ("chaos: write gain x (1 + 0.3 (u - 1/2)) per step and site", Setup { alpha: 0.3, ..base })];
        let key = |r: &Row| (r.contain[1], r.contain[0]);
        let beats = |a: &Row, b: &Row| key(a).0 > key(b).0 || (key(a).0 == key(b).0 && key(a).1 > key(b).1);
        let mut setups = vec![base];
        setups.extend(arms.iter().map(|a| a.1));
        let rows = run(&setups, &dev, &[250, 20_000]);
        say(format!("  plain: t 250 {:.3}, t 20000 {:.3}", rows[0].contain[0], rows[0].contain[1]));
        let mut kept = Vec::new();
        for (i, (name, _)) in arms.iter().enumerate() {
            let r = &rows[i + 1];
            say(format!("  {name}: t 250 {:.3}, t 20000 {:.3}{}", r.contain[0], r.contain[1], if beats(r, &rows[0]) { " (beats plain on dev)" } else { "" }));
            kept.push(beats(r, &rows[0]));
        }
        let mut test_rows: Vec<Option<Row>> = vec![None; 2];
        let confirm: Vec<Setup> = std::iter::once(base).chain(arms.iter().enumerate().filter(|(i, _)| kept[*i]).map(|(_, a)| a.1)).collect();
        if confirm.len() > 1 {
            let tr = run(&confirm, &test, &[250, 20_000]);
            say(format!("  test plain: t 250 {:.3}, t 20000 {:.3}", tr[0].contain[0], tr[0].contain[1]));
            let mut k = 1;
            for i in 0..2 {
                if kept[i] {
                    say(format!("  test {}: t 250 {:.3}, t 20000 {:.3}", arms[i].0, tr[k].contain[0], tr[k].contain[1]));
                    kept[i] = beats(&tr[k], &tr[0]);
                    test_rows[i] = Some(tr[k].clone());
                    k += 1;
                }
            }
        }
        say(format!("  kept: amplification {}, chaos {}", kept[0], kept[1]));
        verdicts.push(("P9 amplification not kept (it substitutes for cooling)".into(), !kept[0]));
        verdicts.push(("P10 chaos not kept".into(), !kept[1]));
    }

    if part == "kept" {
        say("\nAdded after the precommitted rounds (not a prediction): the kept setup (shape, q 0.0025, gain x 2) against the shape at plain settings, by depth, t 250 and t 20000:".into());
        let fs = [4usize, 8, 16];
        let setups: Vec<Setup> = fs.iter().flat_map(|&f| [Setup::new(Code::Shape, f), Setup { q: 0.0025, gain: 2.0 * GAIN, ..Setup::new(Code::Shape, f) }]).collect();
        let rows = run(&setups, &test, &[250, 20_000]);
        for (i, &f) in fs.iter().enumerate() {
            let (p, k) = (&rows[2 * i], &rows[2 * i + 1]);
            say(format!(
                "  F {f:2} ({} memories, {} bytes each): plain {:.3} / {:.3}; kept {:.3} / {:.3} (set {:.3} / {:.3}; oldest half {:.3}, newest {:.3})",
                16 * f, BYTES / (16 * f), p.contain[0], p.contain[1], k.contain[0], k.contain[1], k.set[0], k.set[1], k.old_new.0, k.old_new.1
            ));
        }
    }

    say(String::new());
    for (name, ok) in &verdicts {
        say(format!("{} {name}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", verdicts.iter().filter(|x| x.1).count(), verdicts.len(), t0.elapsed().as_secs_f64()));
}
