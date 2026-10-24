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
const EPS: f64 = 0.05;
const QS: [f64; 3] = [0.02, 0.05, 0.1];
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
const THREADS: usize = 4;

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
fn simulate(r: usize, e_ref: f64, q: f64, seed: u64, bl: &[(usize, Vec<usize>)]) -> Vec<f64> {
    let mut med = Medium::new(&DIMS, 1);
    let sites = med.fields[0].e.len();
    let mut material = vec![0.0; sites];
    for t in 0..WRITE + DRAIN {
        if t < WRITE {
            for (_, b) in bl {
                for &s in b {
                    med.inject(s, 0, &[1.0], GAIN);
                }
            }
        }
        spec_stir::stir(&mut med, q, EPS, seed, t);
        spec_step::step_sealed(&mut med, &material, KAPPA, DELTA, TAU);
        let energy = &med.fields[0].e;
        if r == 0 {
            spec_trap::form(&mut material, energy, &|_| MU, NU, e_ref, P);
        } else {
            let d = astro_jeans::drive(energy, &DIMS, r);
            spec_trap::form(&mut material, &d, &|_| MU, NU, e_ref, P);
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

type Job = (usize, f64, f64, u64);

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
                let (r, e, q, sd) = jobs[j];
                let bl = blobs(sd);
                *out[j].lock().unwrap() = score(&simulate(r, e, q, sd, &bl), &bl);
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
    say(format!("ASTRO-3: Jeans drive vs local drive for scar formation; {DIMS:?} lattice; q {QS:?}; blobs {SIDES:?} x {PER_SIDE}; {THREADS} threads"));

    // J3, the code check: R 1 through astro-jeans equals the local law called directly.
    let bl = blobs(0);
    let same = simulate(0, 0.3, 0.05, 0, &bl) == simulate(1, 0.3, 0.05, 0, &bl);
    say(format!("J3 code check (R 1 == local, bit for bit): {same}"));

    // Dev: every (R, E_ref, q).
    let jobs: Vec<Job> = RS.iter().flat_map(|&r| QS.iter().flat_map(move |&q| E_REFS.iter().flat_map(move |&e| (0..4u64).map(move |s| (r, e, q, s))))).collect();
    let res = run(&jobs);
    let k = dev.len();
    let mut chosen = vec![vec![0.0; QS.len()]; RS.len()]; // E_ref per (R, q)
    let mut dev_false = vec![0.0; RS.len()];
    say("\nDev (seeds 0-3): per R and q, E_ref -> side-7 true / side-1 true / false".into());
    for (ri, &r) in RS.iter().enumerate() {
        for (qi, &q) in QS.iter().enumerate() {
            let mut line = format!("  R {r} q {q:<5}:");
            let mut pick = (E_REFS[0], 0usize);
            for (ei, &e) in E_REFS.iter().enumerate() {
                let base = ((ri * QS.len() + qi) * E_REFS.len() + ei) * k;
                let (t, f) = avg(&res[base..base + k]);
                line += &format!(" {e}:{:.2}/{:.2}/{:.3}", t[2], t[0], f);
                if t[2] >= TRUE_TARGET {
                    pick = (e, ei);
                }
            }
            chosen[ri][qi] = pick.0;
            let base = ((ri * QS.len() + qi) * E_REFS.len() + pick.1) * k;
            dev_false[ri] += avg(&res[base..base + k]).1 / QS.len() as f64;
            say(line);
        }
    }
    let rj = 1 + (1..RS.len()).min_by(|&a, &b| dev_false[a].partial_cmp(&dev_false[b]).unwrap()).unwrap() - 1;
    say(format!("  matched E_ref per (R, q): {chosen:?}; dev false at match per R {dev_false:.4?}; Jeans R chosen {}  ({:.0} s)", RS[rj], t0.elapsed().as_secs_f64()));

    // Test: every R at its dev-matched E_ref.
    let tj: Vec<Job> = (0..RS.len()).flat_map(|ri| (0..QS.len()).flat_map(move |qi| (10..18u64).map(move |s| (ri, qi, s)))).map(|(ri, qi, s)| (RS[ri], chosen[ri][qi], QS[qi], s)).collect();
    let tres = run(&tj);
    let kt = test.len();
    let mut tab = vec![vec![(Vec::new(), 0.0); QS.len()]; RS.len()];
    say("\nTest (seeds 10-17): true rate per side [1, 3, 7] and far-field false rate".into());
    for ri in 0..RS.len() {
        for qi in 0..QS.len() {
            let base = (ri * QS.len() + qi) * kt;
            tab[ri][qi] = avg(&tres[base..base + kt]);
            say(format!("  R {} q {:<5} E_ref {:<5}: true {:.3?}  false {:.4}", RS[ri], QS[qi], chosen[ri][qi], tab[ri][qi].0, tab[ri][qi].1));
        }
    }
    let mut j1 = true;
    let mut tested = 0;
    let mut j1s = String::new();
    let mut j2 = true;
    let mut j2s = String::new();
    for qi in 0..QS.len() {
        let (lt, lf) = &tab[0][qi];
        let (jt, jf) = &tab[rj][qi];
        if *lf >= 0.01 {
            tested += 1;
            let ok = *jf <= 0.5 * lf && lt[2] >= 0.85 && jt[2] >= 0.85;
            j1 &= ok;
            j1s += &format!(" q {}: {jf:.4} vs {lf:.4}, side-7 {:.2}/{:.2};", QS[qi], jt[2], lt[2]);
        }
        let ok2 = jt[0] <= 0.5 * lt[0] || (jt[0] <= 0.1 && lt[0] <= 0.1);
        j2 &= ok2;
        j2s += &format!(" q {}: {:.2} vs {:.2};", QS[qi], jt[0], lt[0]);
    }
    let v = [
        (format!("J1 Jeans false <= 0.5 x local at matched true rate ({tested} q with local false >= 0.01):{j1s}"), j1 && tested > 0),
        (format!("J2 side-1 true under Jeans <= 0.5 x local:{j2s}"), j2),
        (format!("J3 R 1 reproduces the local law bit for bit: {same}"), same),
    ];
    say(String::new());
    for (nm, ok) in &v {
        say(format!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
