//! GEO-1 G1 orchestrator (docs/GEO-1-PRECOMMIT.md): name a shape and its handedness after an unknown
//! rotation, translation, possible mirror, shuffle and noise. Bars: plain Chamfer, mental rotation
//! (search outward from 0° in 5° steps until a match). New stacks from physics: Noether (conserved
//! rotation charges, neutral products of complex moments; handedness from the sign of an imaginary
//! part) and spontaneous symmetry breaking (a canonical frame). Every setting is here.

use std::f64::consts::PI;

const SHAPES: usize = 32;
const POINTS: usize = 8;
const QUERIES: usize = 256;
const STEP_DEG: f64 = 5.0;
const SIGMAS: [f64; 3] = [0.01, 0.02, 0.05];

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
    fn normal(&mut self) -> f64 {
        (-2.0 * self.unit().max(1e-300).ln()).sqrt() * (2.0 * PI * self.unit()).cos()
    }
}

type Shape = Vec<(f64, f64)>;

fn centred(p: &[(f64, f64)]) -> Shape {
    let n = p.len() as f64;
    let (cx, cy) = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    p.iter().map(|q| (q.0 - cx, q.1 - cy)).collect()
}

/// Complex product helpers on (re, im).
fn mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
fn cj(a: (f64, f64)) -> (f64, f64) {
    (a.0, -a.1)
}
fn pw(a: (f64, f64), k: usize) -> (f64, f64) {
    (0..k).fold((1.0, 0.0), |acc, _| mul(acc, a))
}

/// Noether: mirror-invariant features (|m_k|, Re and |Im| of neutral products) plus the chirality
/// pseudo-scalar Im(m2^3 conj(m3)^2). `amp` adds charges up to 10 and more neutral products.
fn noether(p: &[(f64, f64)], amp: bool) -> (Vec<f64>, f64) {
    let kmax = if amp { 10 } else { 6 };
    let m = shape_moments::moments(p, kmax);
    let mut f: Vec<f64> = (2..=kmax).map(|k| (m[k].0.powi(2) + m[k].1.powi(2)).sqrt()).collect();
    let mut neutral = vec![mul(pw(m[2], 2), cj(m[4])), mul(pw(m[2], 3), pw(cj(m[3]), 2)), mul(mul(m[2], m[3]), cj(m[5]))];
    if amp {
        neutral.push(mul(mul(m[2], m[4]), cj(m[6])));
        neutral.push(mul(pw(m[3], 2), cj(m[6])));
        neutral.push(mul(pw(m[4], 2), cj(m[8])));
        neutral.push(mul(mul(m[2], m[6]), cj(m[8])));
    }
    for z in &neutral {
        let mag = (z.0 * z.0 + z.1 * z.1).sqrt().max(1e-300);
        // Scale to a comparable size: the cube root of the magnitude keeps units near a length^2.
        let s = mag.powf(1.0 / 3.0) / mag;
        f.push(z.0 * s);
        f.push(z.1.abs() * s);
    }
    f.extend(shape_distances::sorted_distances(p));
    let chir = mul(pw(m[2], 3), pw(cj(m[3]), 2)).1;
    (f, chir)
}

fn l2(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

struct Query {
    shape: usize,
    mirror: bool,
    angle: f64,
    pts: Shape,
}

fn world(seed: u64, sigma: f64) -> (Vec<Shape>, Vec<Query>) {
    let mut r = Rng(seed);
    let lib: Vec<Shape> = (0..SHAPES).map(|_| centred(&(0..POINTS).map(|_| (r.unit(), r.unit())).collect::<Vec<_>>())).collect();
    let qs = (0..QUERIES)
        .map(|_| {
            let shape = (r.next() % SHAPES as u64) as usize;
            let mirror = r.next() & 1 == 1;
            let angle = 2.0 * PI * r.unit();
            let mut pts = shape_rotate::rotate(&lib[shape], angle, mirror);
            let (tx, ty) = (r.unit() * 5.0, r.unit() * 5.0);
            for p in pts.iter_mut() {
                *p = (p.0 + tx + sigma * r.normal(), p.1 + ty + sigma * r.normal());
            }
            for i in (1..pts.len()).rev() {
                let j = (r.next() % (i as u64 + 1)) as usize;
                pts.swap(i, j);
            }
            Query { shape, mirror, angle, pts: centred(&pts) }
        })
        .collect();
    (lib, qs)
}

/// Mental rotation: undo angles 0, +5, -5, +10, ... until a Chamfer below tau. (shape, mirror, steps).
fn search(q: &[(f64, f64)], lib: &[Shape], tau: f64, chaos: Option<u64>, qi: usize) -> (usize, bool, usize) {
    let mut best = (f64::MAX, 0usize, false);
    let mut steps = 0;
    let mut k = 0usize;
    let mut a_pos = 0.0;
    let mut a_neg = 0.0;
    while a_pos <= 180.0 + 1e-9 {
        let mut step = STEP_DEG;
        if let Some(sd) = chaos {
            let key = sd.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (qi as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (k as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
            step *= 1.0 + 0.5 * (qb_chaos::keyed_unit(key) - 0.5);
        }
        let angles: Vec<f64> = if k == 0 { vec![0.0] } else { vec![a_pos, a_neg] };
        for a in angles {
            for mirror in [false, true] {
                steps += 1;
                // Undo: un-rotate, then un-mirror.
                let r0 = shape_rotate::rotate(q, -a.to_radians(), false);
                let r1 = if mirror { shape_rotate::rotate(&r0, 0.0, true) } else { r0 };
                for (j, l) in lib.iter().enumerate() {
                    let c = shape_chamfer::chamfer(&r1, l);
                    if c < best.0 {
                        best = (c, j, mirror);
                    }
                }
            }
        }
        if best.0 < tau {
            break;
        }
        k += 1;
        a_pos += step;
        a_neg -= step;
    }
    (best.1, best.2, steps)
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx).powi(2);
        syy += (b - my).powi(2);
    }
    sxy / (sxx * syy).sqrt()
}

/// Per strategy: (identity accuracy, mirror accuracy, mean steps). Strategy 0 plain, 1 search,
/// 2 Noether, 3 canon, 4 search+chaos, 5 Noether+amp. Also returns (|angle|, steps) for the search
/// on unmirrored queries, and canon failures by isotropy tercile.
fn evaluate(seed: u64, sigma: f64, tau: f64) -> (Vec<(f64, f64, f64)>, Vec<(f64, f64)>, [usize; 3], usize) {
    let (lib, qs) = world(seed, sigma);
    let libf: Vec<(Vec<f64>, f64)> = lib.iter().map(|l| noether(l, false)).collect();
    let libfa: Vec<(Vec<f64>, f64)> = lib.iter().map(|l| noether(l, true)).collect();
    let libc: Vec<(Shape, bool, f64)> = lib.iter().map(|l| shape_canon::canon(l)).collect();
    let mut ratios: Vec<f64> = libc.iter().map(|c| c.2).collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (t1, t2) = (ratios[SHAPES / 3], ratios[2 * SHAPES / 3]);
    let mut acc = vec![(0usize, 0usize, 0usize); 6];
    let mut rt = Vec::new();
    let mut canon_fail = [0usize; 3];
    let mut canon_fail_total = 0;
    for (qi, q) in qs.iter().enumerate() {
        // Plain.
        let mut best = (f64::MAX, 0, false);
        for (j, l) in lib.iter().enumerate() {
            for mirror in [false, true] {
                let c = shape_chamfer::chamfer(&q.pts, &shape_rotate::rotate(l, 0.0, mirror));
                if c < best.0 {
                    best = (c, j, mirror);
                }
            }
        }
        let mut record = |k: usize, s: usize, m: bool, steps: usize| {
            acc[k].0 += (s == q.shape) as usize;
            acc[k].1 += (s == q.shape && m == q.mirror) as usize;
            acc[k].2 += steps;
        };
        record(0, best.1, best.2, 1);
        // Mental rotation, and with chaos.
        let (s, m, steps) = search(&q.pts, &lib, tau, None, qi);
        record(1, s, m, steps);
        if !q.mirror {
            let a = q.angle.to_degrees();
            rt.push((a.min(360.0 - a), steps as f64));
        }
        let (s, m, steps) = search(&q.pts, &lib, tau, Some(seed), qi);
        record(4, s, m, steps);
        // Noether.
        for (k, lf, amp) in [(2usize, &libf, false), (5, &libfa, true)] {
            let (f, ch) = noether(&q.pts, amp);
            let j = (0..SHAPES).min_by(|&a, &b| l2(&f, &lf[a].0).partial_cmp(&l2(&f, &lf[b].0)).unwrap()).unwrap();
            let mirror = (ch > 0.0) != (lf[j].1 > 0.0);
            record(k, j, mirror, 1);
        }
        // Symmetry breaking.
        let (qc, qm, _) = shape_canon::canon(&q.pts);
        let j = (0..SHAPES).min_by(|&a, &b| shape_chamfer::chamfer(&qc, &libc[a].0).partial_cmp(&shape_chamfer::chamfer(&qc, &libc[b].0)).unwrap()).unwrap();
        let mirror = qm != libc[j].1;
        record(3, j, mirror, 1);
        if j != q.shape {
            canon_fail_total += 1;
            let r = libc[q.shape].2;
            canon_fail[if r >= t2 { 0 } else if r >= t1 { 1 } else { 2 }] += 1;
        }
    }
    let n = QUERIES as f64;
    (acc.iter().map(|a| (a.0 as f64 / n, a.1 as f64 / n, a.2 as f64 / n)).collect(), rt, canon_fail, canon_fail_total)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("GEO-1 G1: {SHAPES} shapes of {POINTS} points, {QUERIES} queries per seed (random rotation, translation, mirror 1/2, shuffle, noise)");
    let taus = [0.02, 0.03, 0.05, 0.08];
    let dev: Vec<(f64, f64)> = taus.iter().map(|&t| {
        let v: Vec<_> = (0..4u64).map(|s| evaluate(s, 0.02, t).0[1]).collect();
        (v.iter().map(|x| x.1).sum::<f64>() / 4.0, v.iter().map(|x| x.2).sum::<f64>() / 4.0)
    }).collect();
    let ti = (0..taus.len()).max_by(|&a, &b| dev[a].0.partial_cmp(&dev[b].0).unwrap().then(dev[b].1.partial_cmp(&dev[a].1).unwrap())).unwrap();
    let tau = taus[ti];
    println!("  dev tau (identity+mirror, steps): {:?} -> chosen {tau}", taus.iter().zip(&dev).map(|(t, d)| (t, (d.0 * 1000.0).round() / 1000.0, d.1.round())).collect::<Vec<_>>());
    let names = ["plain (Chamfer, no rotation)", "mental rotation (search)", "Noether charges", "symmetry breaking (canon)", "search + chaos", "Noether + amplification"];
    let mut table = std::collections::HashMap::new();
    let mut rt_all = Vec::new();
    let mut cf = [0usize; 3];
    let mut cft = 0;
    for &sigma in &SIGMAS {
        let runs: Vec<_> = std::thread::scope(|sc| {
            let hs: Vec<_> = (10..18u64).map(|s| sc.spawn(move || evaluate(s, sigma, tau))).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        println!("\n  noise {sigma}: strategy, identity, identity+mirror, cost (comparisons per query / steps)");
        for k in 0..6 {
            let m = |f: &dyn Fn(&(f64, f64, f64)) -> f64| runs.iter().map(|r| f(&r.0[k])).sum::<f64>() / runs.len() as f64;
            let row = (m(&|x| x.0), m(&|x| x.1), m(&|x| x.2));
            println!("    {:<30} {:.3}  {:.3}  {:.1}", names[k], row.0, row.1, row.2);
            table.insert((k, (sigma * 1000.0) as i64), row);
        }
        if sigma == 0.02 {
            for r in &runs {
                rt_all.extend(r.1.iter().cloned());
                for i in 0..3 {
                    cf[i] += r.2[i];
                }
                cft += r.3;
            }
        }
    }
    let rho = pearson(&rt_all.iter().map(|x| x.0).collect::<Vec<_>>(), &rt_all.iter().map(|x| x.1).collect::<Vec<_>>());
    println!("\n  Shepard-Metzler signature: correlation of search steps with |angle| (noise 0.02, unmirrored): {rho:.3}");
    println!("  canon failures by isotropy tercile (most isotropic first): {cf:?} of {cft}");
    let g = |k: usize, s: f64| table[&(k, (s * 1000.0) as i64)];
    let chaos_better = |s: f64| g(4, s).1 > g(1, s).1 || (g(4, s).1 == g(1, s).1 && g(4, s).2 < g(1, s).2);
    let v = [
        (format!("G1a plain identity <= 0.2 ({:.3})", g(0, 0.02).0), g(0, 0.02).0 <= 0.2),
        (format!("G1b search identity >= 0.95 ({:.3}), mirror >= 0.95 ({:.3}), steps-angle corr >= 0.8 ({rho:.3})", g(1, 0.02).0, g(1, 0.02).1, ), g(1, 0.02).0 >= 0.95 && g(1, 0.02).1 >= 0.95 && rho >= 0.8),
        (format!("G1c Noether identity >= 0.95 ({:.3}), identity+mirror >= 0.9 ({:.3}), cost 1", g(2, 0.02).0, g(2, 0.02).1), g(2, 0.02).0 >= 0.95 && g(2, 0.02).1 >= 0.9),
        (format!("G1d canon identity >= 0.8 ({:.3}), failures concentrated in the most isotropic third ({} of {})", g(3, 0.02).0, cf[0], cft), g(3, 0.02).0 >= 0.8 && cf[0] * 2 >= cft),
        ("G1e chaos on the search not kept (noise 0.02)".to_string(), !chaos_better(0.02)),
        (format!("G1f amplification kept at 0.01 (+{:.3}) and not at 0.05 ({:+.3})", g(5, 0.01).1 - g(2, 0.01).1, g(5, 0.05).1 - g(2, 0.05).1), g(5, 0.01).1 >= g(2, 0.01).1 + 0.01 && g(5, 0.05).1 <= g(2, 0.05).1),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
