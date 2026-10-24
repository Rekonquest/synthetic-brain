//! GEO-2 orchestrator (docs/GEO-2-PRECOMMIT.md), built on GEO-1 G1's world and bars. Original GEO-1 G1 orchestrator (docs/GEO-1-PRECOMMIT.md): name a shape and its handedness after an unknown
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
    #[allow(dead_code)]
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


fn arg(z: (f64, f64)) -> f64 {
    z.1.atan2(z.0)
}

/// Grid-cell read of the angle taking `lib` onto `q` (both centred), from charges `ks`.
fn read_angle(q: &[(f64, f64)], lib: &[(f64, f64)], ks: &[usize]) -> f64 {
    let kmax = *ks.iter().max().unwrap();
    let (mq, ml) = (shape_moments::moments(q, kmax), shape_moments::moments(lib, kmax));
    let modules: Vec<(usize, f64, f64)> = ks
        .iter()
        .map(|&k| {
            let z = mul(mq[k], cj(ml[k]));
            (k, arg(z), (z.0 * z.0 + z.1 * z.1).sqrt())
        })
        .collect();
    phase_angle::read(&modules)
}

/// A2: top-3 shapes by Noether distance; for each, in both mirror states, rotate the library shape by
/// the read angle and verify by Chamfer. (shape, mirror, best Chamfer, Chamfer evaluations).
fn verify(q: &[(f64, f64)], lib: &[Shape], libf: &[(Vec<f64>, f64)], ks: &[usize], chaos: Option<(u64, usize)>) -> (usize, bool, f64, usize) {
    let (f, _) = noether(q, false);
    let mut order: Vec<usize> = (0..SHAPES).collect();
    order.sort_by(|&a, &b| l2(&f, &libf[a].0).partial_cmp(&l2(&f, &libf[b].0)).unwrap());
    let mut best = (f64::MAX, 0usize, false);
    let mut evals = 0;
    for &j in &order[..3] {
        for mirror in [false, true] {
            let base = shape_rotate::rotate(&lib[j], 0.0, mirror);
            let th = read_angle(q, &base, ks);
            let mut cands = vec![th];
            if let Some((sd, qi)) = chaos {
                for v in 1..=3u64 {
                    let key = sd.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((qi as u64) * 64 + j as u64 * 2 + mirror as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ v.wrapping_mul(0x1656_67B1_9E37_79F9);
                    cands.push(th + (10.0f64).to_radians() * (qb_chaos::keyed_unit(key) - 0.5));
                }
            }
            for t in cands {
                let c = shape_chamfer::chamfer(&shape_rotate::rotate(&base, t, false), q);
                evals += 1;
                if c < best.0 {
                    best = (c, j, mirror);
                }
            }
        }
    }
    (best.1, best.2, best.0, evals)
}

const TAU: f64 = 0.05;

/// Per arm: (identity, identity + mirror, mean Chamfer evaluations). Arms: 0 search (bar), 1 Noether,
/// 2 Noether + CIP, 3 grid-cell verify, 4 dual process, 5 verify + chaos, 6 verify with 4 modules.
/// Also the CIP and Noether-pseudo-scalar mirror accuracy on correctly identified shapes.
fn evaluate(seed: u64, sigma: f64, gate: f64) -> (Vec<(f64, f64, f64)>, (f64, f64)) {
    let (lib, qs) = world(seed, sigma);
    let libf: Vec<(Vec<f64>, f64)> = lib.iter().map(|l| noether(l, false)).collect();
    let libcip: Vec<f64> = lib.iter().map(|l| shape_cip::chirality(l)).collect();
    let mut acc = vec![(0usize, 0usize, 0usize); 7];
    let (mut cip_ok, mut im_ok, mut both) = (0, 0, 0);
    for (qi, q) in qs.iter().enumerate() {
        let mut put = |k: usize, s: usize, m: bool, cost: usize| {
            acc[k].0 += (s == q.shape) as usize;
            acc[k].1 += (s == q.shape && m == q.mirror) as usize;
            acc[k].2 += cost;
        };
        let (s, m, steps) = search(&q.pts, &lib, TAU, None, qi);
        put(0, s, m, steps * SHAPES);
        let (f, ch) = noether(&q.pts, false);
        let mut order: Vec<usize> = (0..SHAPES).collect();
        order.sort_by(|&a, &b| l2(&f, &libf[a].0).partial_cmp(&l2(&f, &libf[b].0)).unwrap());
        let j = order[0];
        let im_mirror = (ch > 0.0) != (libf[j].1 > 0.0);
        let cip_mirror = (shape_cip::chirality(&q.pts) > 0.0) != (libcip[j] > 0.0);
        put(1, j, im_mirror, 0);
        put(2, j, cip_mirror, 0);
        if j == q.shape {
            both += 1;
            cip_ok += (cip_mirror == q.mirror) as usize;
            im_ok += (im_mirror == q.mirror) as usize;
        }
        let (vs, vm, vc, ve) = verify(&q.pts, &lib, &libf, &[2, 3], None);
        put(3, vs, vm, ve);
        // Dual process: the PERCEPTION gate "uncertain" cites the Noether margin and the two
        // handedness readings.
        let margin = l2(&f, &libf[order[1]].0) / l2(&f, &libf[j].0).max(1e-12);
        let uncertain = margin < gate || cip_mirror != im_mirror;
        if !uncertain {
            put(4, j, cip_mirror, 0);
        } else if vc <= TAU {
            put(4, vs, vm, ve);
        } else {
            put(4, s, m, ve + steps * SHAPES);
        }
        let (cs, cm, _, ce) = verify(&q.pts, &lib, &libf, &[2, 3], Some((seed, qi)));
        put(5, cs, cm, ce);
        let (as_, am, _, ae) = verify(&q.pts, &lib, &libf, &[2, 3, 4, 5], None);
        put(6, as_, am, ae);
    }
    let n = QUERIES as f64;
    (acc.iter().map(|a| (a.0 as f64 / n, a.1 as f64 / n, a.2 as f64 / n)).collect(), (cip_ok as f64 / both.max(1) as f64, im_ok as f64 / both.max(1) as f64))
}

fn mean_runs(seeds: std::ops::Range<u64>, sigma: f64, gate: f64) -> (Vec<(f64, f64, f64)>, (f64, f64)) {
    let runs: Vec<_> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || evaluate(s, sigma, gate))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let k = runs.len() as f64;
    let arms = (0..7).map(|a| (runs.iter().map(|r| r.0[a].0).sum::<f64>() / k, runs.iter().map(|r| r.0[a].1).sum::<f64>() / k, runs.iter().map(|r| r.0[a].2).sum::<f64>() / k)).collect();
    (arms, (runs.iter().map(|r| r.1 .0).sum::<f64>() / k, runs.iter().map(|r| r.1 .1).sum::<f64>() / k))
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("GEO-2: physics (Noether charges) composed with neuroscience (grid-cell phase code), chemistry (CIP) and cognitive (dual process); G1's task");
    let gates = [1.05, 1.1, 1.2, 1.5];
    let dev: Vec<_> = gates.iter().map(|&g| mean_runs(0..4, 0.02, g).0).collect();
    let gi = (0..gates.len()).find(|&i| dev[i][4].1 >= dev[i][0].1 - 0.01).unwrap_or(gates.len() - 1);
    let gate = gates[gi];
    println!("  dev gate (dual process identity+mirror, cost): {:?}; chosen {gate}", gates.iter().zip(&dev).map(|(g, d)| (*g, (d[4].1 * 1000.0).round() / 1000.0, d[4].2.round())).collect::<Vec<_>>());
    let names = ["search (bar)", "Noether alone", "Noether + CIP (chemistry)", "grid-cell verify (neuroscience)", "dual process (cognitive)", "verify + chaos", "verify, 4 modules (amplified)"];
    let mut t = std::collections::HashMap::new();
    for &sigma in &SIGMAS {
        let (arms, mir) = mean_runs(10..18, sigma, gate);
        println!("\n  noise {sigma}: arm, identity, identity+mirror, Chamfer evaluations per query");
        for (k, a) in arms.iter().enumerate() {
            println!("    {:<34} {:.3}  {:.3}  {:.1}", names[k], a.0, a.1, a.2);
            t.insert((k, (sigma * 1000.0) as i64), *a);
        }
        println!("    mirror accuracy on identified shapes: CIP {:.3}, Noether pseudo-scalar {:.3}", mir.0, mir.1);
        t.insert((90, (sigma * 1000.0) as i64), (mir.0, mir.1, 0.0));
    }
    let g = |k: usize, s: f64| t[&(k, (s * 1000.0) as i64)];
    let d1 = SIGMAS.iter().all(|&s| g(4, s).1 >= g(0, s).1 - 0.01 && g(4, s).2 <= 0.1 * g(0, s).2);
    let x1 = g(5, 0.05).1 > g(3, 0.05).1 && g(5, 0.01).1 <= g(3, 0.01).1;
    let v = [
        (format!("N1 grid-cell verify >= 0.99 at 0.02 ({:.3}), >= 0.95 at 0.05 ({:.3}), {} evaluations vs search {:.0}", g(3, 0.02).1, g(3, 0.05).1, g(3, 0.02).2, g(0, 0.02).2), g(3, 0.02).1 >= 0.99 && g(3, 0.05).1 >= 0.95),
        (format!("C1 CIP mirror >= Noether pseudo-scalar + 0.05 at 0.02 ({:.3} vs {:.3})", g(90, 0.02).0, g(90, 0.02).1), g(90, 0.02).0 >= g(90, 0.02).1 + 0.05),
        (format!("D1 dual process within 0.01 of search at <= 10% cost ({:.3}/{:.3}/{:.3} vs {:.3}/{:.3}/{:.3}; cost {:.0}/{:.0}/{:.0} vs {:.0})", g(4, 0.01).1, g(4, 0.02).1, g(4, 0.05).1, g(0, 0.01).1, g(0, 0.02).1, g(0, 0.05).1, g(4, 0.01).2, g(4, 0.02).2, g(4, 0.05).2, g(0, 0.02).2), d1),
        (format!("X1 chaos kept at 0.05 ({:.3} vs {:.3}), not at 0.01 ({:.3} vs {:.3})", g(5, 0.05).1, g(3, 0.05).1, g(5, 0.01).1, g(3, 0.01).1), x1),
        (format!("X2 4 modules raise verify at 0.05 by >= 0.01 ({:.3} vs {:.3})", g(6, 0.05).1, g(3, 0.05).1), g(6, 0.05).1 >= g(3, 0.05).1 + 0.01),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
