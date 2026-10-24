//! ASTRO-3 orchestrator (docs/ASTRO-POC.md): Jeans instability as the scar-formation law. SCAR-1
//! run 5 / FOLD-4's medium laws (spec-stir, spec-step::step_sealed, spec-trap::form) unchanged; the
//! trap law is driven either by the local energy (R 1) or by the Jeans drive (astro-jeans: the energy
//! with modes shorter than R removed). Blobs of side 1, 3, 7 are written; true scars (blobs) are
//! counted against false scars (far-field sites carved by the stir alone). Every setting is here.

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use thermo_host::Medium;

const DIMS: [usize; 2] = [64, 64];
const KAPPA: f64 = 0.2;
const DELTA: f64 = 0.01;
const TAU: f64 = 0.99;
const NU: f64 = 5e-5;
const MU: f64 = 0.02;
const P: f64 = 3.0;
/// ASTRO-R media (q, ε): the POC's control, then bursty stirs with the same mean input q·ε.
const MEDIA: [(f64, f64); 3] = [(0.02, 0.05), (0.005, 0.2), (0.002, 0.5)];
const GAIN: f64 = 0.02;
const WRITE: usize = 300;
const DRAIN: usize = 3000;
const SIDES: [usize; 3] = [1, 3, 7];
const PER_SIDE: usize = 6;
const GAP: usize = 6;
const FAR: usize = 5;
const SCAR: f64 = 0.5;
const TRUE_TARGET: f64 = 0.9;
const E_REFS: [f64; 12] = [0.05, 0.075, 0.1, 0.15, 0.2, 0.3, 0.45, 0.6, 0.9, 1.35, 2.0, 3.0];
const RS: [usize; 3] = [1, 3, 5];
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
}

fn pdist(a: usize, b: usize, l: usize) -> usize {
    let d = a.abs_diff(b);
    d.min(l - d)
}

/// Blobs: per side, PER_SIDE squares (as lists of sites) at random corners, every pair of blobs at
/// least GAP apart (Chebyshev, periodic) between their nearest sites.
fn blobs(seed: u64) -> Vec<(usize, Vec<usize>)> {
    let (w, h) = (DIMS[0], DIMS[1]);
    let mut r = Rng(seed ^ 0x81_0B5);
    let mut out: Vec<(usize, Vec<usize>)> = Vec::new();
    for &side in SIDES.iter().rev() {
        let mut placed = 0;
        while placed < PER_SIDE {
            let (x0, y0) = ((r.next() % w as u64) as usize, (r.next() % h as u64) as usize);
            let sites: Vec<usize> = (0..side).flat_map(|dy| (0..side).map(move |dx| ((x0 + dx) % w) + w * ((y0 + dy) % h))).collect();
            let ok = out.iter().all(|(_, o)| o.iter().all(|&a| sites.iter().all(|&b| pdist(a % w, b % w, w).max(pdist(a / w, b / w, h)) >= GAP)));
            if ok {
                out.push((side, sites));
                placed += 1;
            }
        }
    }
    out
}

/// Final material after the write and drain. `r` 0 = the local law called directly (no astro-jeans).
fn simulate(r: usize, e_ref: f64, medium: usize, mode: u8, seed: u64, bl: &[(usize, Vec<usize>)]) -> Vec<f64> {
    let (q, eps) = MEDIA[medium];
    let gain = if mode == 2 { 2.0 * GAIN } else { GAIN };
    let mut med = Medium::new(&DIMS, 1);
    let sites = med.fields[0].e.len();
    let mut material = vec![0.0; sites];
    for t in 0..WRITE + DRAIN {
        if t < WRITE {
            for (_, b) in bl {
                for &s in b {
                    med.inject(s, 0, &[1.0], gain);
                }
            }
        }
        spec_stir::stir(&mut med, q, eps, seed, t);
        spec_step::step_sealed(&mut med, &material, KAPPA, DELTA, TAU);
        let energy = &med.fields[0].e;
        let mu = |site: usize| -> f64 {
            if mode != 1 {
                return MU;
            }
            let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (site as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
            MU * (1.0 + 0.5 * (qb_chaos::keyed_unit(k) - 0.5))
        };
        if r == 0 {
            spec_trap::form(&mut material, energy, &mu, NU, e_ref, P);
        } else {
            let d = astro_jeans::drive(energy, &DIMS, r);
            spec_trap::form(&mut material, &d, &mu, NU, e_ref, P);
        }
    }
    material
}

/// (true rate per side, false rate on the far field).
fn score(m: &[f64], bl: &[(usize, Vec<usize>)]) -> (Vec<f64>, f64) {
    let w = DIMS[0];
    let tr: Vec<f64> = SIDES
        .iter()
        .map(|&side| {
            let bs: Vec<&Vec<usize>> = bl.iter().filter(|b| b.0 == side).map(|b| &b.1).collect();
            bs.iter().filter(|b| 2 * b.iter().filter(|&&s| m[s] > SCAR).count() >= b.len()).count() as f64 / bs.len() as f64
        })
        .collect();
    let all: Vec<usize> = bl.iter().flat_map(|b| b.1.iter().copied()).collect();
    let (mut far, mut hit) = (0usize, 0usize);
    for s in 0..m.len() {
        if all.iter().all(|&a| pdist(a % w, s % w, w).max(pdist(a / w, s / w, DIMS[1])) > FAR) {
            far += 1;
            hit += (m[s] > SCAR) as usize;
        }
    }
    (tr, hit as f64 / far as f64)
}

/// (R, E_ref, medium, mode 0 plain / 1 chaos / 2 amplification, seed)
type Job = (usize, f64, usize, u8, u64);

fn run(jobs: &[Job]) -> Vec<(Vec<f64>, f64)> {
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<(Vec<f64>, f64)>> = jobs.iter().map(|_| std::sync::Mutex::new((Vec::new(), 0.0))).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (r, e, m, mode, sd) = jobs[j];
                let bl = blobs(sd);
                *out[j].lock().unwrap() = score(&simulate(r, e, m, mode, sd, &bl), &bl);
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap()).collect()
}

/// Mean (true per side, false) over seeds for one (r, e_ref, q), from results laid out seed-fastest.
fn avg(res: &[(Vec<f64>, f64)]) -> (Vec<f64>, f64) {
    let k = res.len() as f64;
    ((0..SIDES.len()).map(|i| res.iter().map(|x| x.0[i]).sum::<f64>() / k).collect(), res.iter().map(|x| x.1).sum::<f64>() / k)
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

fn main() {
    let t0 = std::time::Instant::now();
    let dev: Vec<u64> = (0..4).collect();
    let test: Vec<u64> = (10..18).collect();
    say(format!("ASTRO-3R: Jeans drive vs local drive in bursty media {MEDIA:?} (same mean input); {DIMS:?}; blobs {SIDES:?} x {PER_SIDE}"));
    let jobs: Vec<Job> = RS.iter().flat_map(|&r| (0..MEDIA.len()).flat_map(move |m| E_REFS.iter().flat_map(move |&e| (0..4u64).map(move |s| (r, e, m, 0u8, s))))).collect();
    let res = run(&jobs);
    let k = dev.len();
    let mut chosen = vec![vec![0.0; MEDIA.len()]; RS.len()];
    for (ri, &r) in RS.iter().enumerate() {
        for m in 0..MEDIA.len() {
            let mut line = format!("  dev R {r} medium {:?}:", MEDIA[m]);
            let mut pick = E_REFS[0];
            for (ei, &e) in E_REFS.iter().enumerate() {
                let base = ((ri * MEDIA.len() + m) * E_REFS.len() + ei) * k;
                let (t, f) = avg(&res[base..base + k]);
                line += &format!(" {e}:{:.2}/{:.3}", t[2], f);
                if t[2] >= TRUE_TARGET {
                    pick = e;
                }
            }
            chosen[ri][m] = pick;
            say(line);
        }
    }
    say(format!("  matched E_ref per (R, medium): {chosen:?}"));
    let tj: Vec<Job> = (0..RS.len()).flat_map(|ri| (0..MEDIA.len()).flat_map(move |m| (10..18u64).map(move |s| (ri, m, s)))).map(|(ri, m, s)| (RS[ri], chosen[ri][m], m, 0u8, s)).collect();
    let tres = run(&tj);
    let kt = test.len();
    let mut tab = vec![vec![(Vec::new(), 0.0); MEDIA.len()]; RS.len()];
    for ri in 0..RS.len() {
        for m in 0..MEDIA.len() {
            let base = (ri * MEDIA.len() + m) * kt;
            tab[ri][m] = avg(&tres[base..base + kt]);
            say(format!("  test R {} medium {:?} E_ref {}: true {:.3?} false {:.4}", RS[ri], MEDIA[m], chosen[ri][m], tab[ri][m].0, tab[ri][m].1));
        }
    }
    let mut r3a = true;
    for m in 1..MEDIA.len() {
        let (lt, lf) = &tab[0][m];
        let best_j = (1..RS.len()).min_by(|&a, &b| tab[a][m].1.partial_cmp(&tab[b][m].1).unwrap()).unwrap();
        let (jt, jf) = &tab[best_j][m];
        r3a &= *lf >= 0.05 && *jf <= 0.5 * lf && jt[2] >= 0.85 && lt[2] >= 0.85;
    }
    // Chaos and amplification on the winning law in the burstiest medium.
    let mb = MEDIA.len() - 1;
    let win = (0..RS.len()).min_by(|&a, &b| tab[a][mb].1.partial_cmp(&tab[b][mb].1).unwrap()).unwrap();
    let (rw, ew) = (RS[win], chosen[win][mb]);
    let one = |mode: u8, seeds: &[u64]| avg(&run(&seeds.iter().map(|&s| (rw, ew, mb, mode, s)).collect::<Vec<_>>()));
    let (dp, dc, da) = (one(0, &dev), one(1, &dev), one(2, &dev));
    let (tc, ta) = (one(1, &test), one(2, &test));
    let tp = tab[win][mb].clone();
    let better = |x: &(Vec<f64>, f64), y: &(Vec<f64>, f64)| x.1 < y.1 && x.0[2] >= y.0[2] - 0.02;
    let chaos_kept = better(&dc, &dp) && better(&tc, &tp);
    say(format!("  winner in the burstiest medium: R {rw} at E_ref {ew}"));
    say(format!("  chaos on mu: dev false {:.4} -> {:.4} (true {:.3?} -> {:.3?}); test false {:.4} -> {:.4} (true {:.3?}); kept {chaos_kept}", dp.1, dc.1, dp.0, dc.0, tp.1, tc.1, tc.0));
    say(format!("  amplification gain x 2: dev true {:.3?} false {:.4}; test true {:.3?} -> {:.3?}, false {:.4} -> {:.4}", da.0, da.1, tp.0, ta.0, tp.1, ta.1));
    let r3c = ta.0[1] > tp.0[1] && ta.1 <= tp.1 + 0.01;
    let v = [
        ("R3a bursty media: local false >= 0.05 and Jeans <= 0.5 x local at side-7 true >= 0.85".to_string(), r3a),
        ("R3b chaos not kept".to_string(), !chaos_kept),
        (format!("R3c gain x 2 raises side-3 true ({:.3} -> {:.3}) without false +0.01 ({:.4} -> {:.4})", tp.0[1], ta.0[1], tp.1, ta.1), r3c),
    ];
    say(String::new());
    for (nm, ok) in &v {
        say(format!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
