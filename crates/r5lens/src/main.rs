//! PHYS3 R5 orchestrator: gravitational lensing composed with REINFORCE as priming in noisy recognition. (docs/PHYS3-POC.md)

use std::f64::consts::PI;

const POINTS: usize = 8;

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

pub type Shape = Vec<(f64, f64)>;

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


/// A random library of `n` shapes and queries (shape index, noisy rotated shuffled copy, no mirror).
fn world(seed: u64, n: usize, queries: usize, sigma: f64, pick: &dyn Fn(&mut Rng) -> usize) -> (Vec<Shape>, Vec<(usize, Shape)>) {
    let mut r = Rng(seed);
    let lib: Vec<Shape> = (0..n).map(|_| centred(&(0..POINTS).map(|_| (r.unit(), r.unit())).collect::<Vec<_>>())).collect();
    let qs = (0..queries)
        .map(|_| {
            let shape = pick(&mut r);
            let angle = 2.0 * PI * r.unit();
            let mut pts = shape_rotate::rotate(&lib[shape], angle, false);
            for p in pts.iter_mut() {
                *p = (p.0 + sigma * r.normal(), p.1 + sigma * r.normal());
            }
            for i in (1..pts.len()).rev() {
                let j = (r.next() % (i as u64 + 1)) as usize;
                pts.swap(i, j);
            }
            (shape, centred(&pts))
        })
        .collect();
    (lib, qs)
}

const LIB: usize = 32;
const QUERIES: usize = 512;
const SIGMA: f64 = 0.05;

fn zipf_pick(r: &mut Rng) -> usize {
    let w: Vec<f64> = (1..=LIB).map(|k| 1.0 / k as f64).collect();
    let tot: f64 = w.iter().sum();
    let mut u = r.unit() * tot;
    for (k, x) in w.iter().enumerate() {
        if u < *x {
            return k;
        }
        u -= x;
    }
    LIB - 1
}

/// Identity accuracy with lensing beta (0 = none); chaos jitters beta per query.
fn run(seed: u64, zipf: bool, beta: f64, chaos: bool) -> f64 {
    let pick: &dyn Fn(&mut Rng) -> usize = if zipf { &zipf_pick } else { &|r: &mut Rng| (r.next() % LIB as u64) as usize };
    let (lib, qs) = world(seed, LIB, QUERIES, SIGMA, pick);
    let feats: Vec<Vec<f64>> = lib.iter().map(|l| noether(l, false).0).collect();
    let mut mass = vec![1.0f64; LIB];
    let mut ok = 0;
    for (qi, (shape, q)) in qs.iter().enumerate() {
        let f = noether(q, false).0;
        let b = if chaos { beta * (1.0 + 0.5 * (qb_chaos::keyed_unit(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ qi as u64) - 0.5)) } else { beta };
        let j = (0..LIB).min_by(|&a, &c| (l2(&f, &feats[a]) - b * mass[a].ln()).partial_cmp(&(l2(&f, &feats[c]) - b * mass[c].ln())).unwrap()).unwrap();
        mass[j] += 1.0; // REINFORCE: the answer given gains mass.
        ok += (j == *shape) as usize;
    }
    ok as f64 / QUERIES as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS3 R5: lensing (+ REINFORCE) as priming; {LIB} shapes, Noether identity at noise {SIGMA}, {QUERIES} queries per seed");
    let m = |seeds: std::ops::Range<u64>, f: &dyn Fn(u64) -> f64| { let k = (seeds.end - seeds.start) as f64; seeds.map(f).sum::<f64>() / k };
    let betas = [0.002, 0.005, 0.01, 0.02, 0.05];
    let dev: Vec<f64> = betas.iter().map(|&b| m(0..4, &|s| run(s, true, b, false))).collect();
    let beta = betas[(0..betas.len()).max_by(|&a, &b| dev[a].partial_cmp(&dev[b]).unwrap()).unwrap()];
    println!("  dev beta {betas:?} -> {dev:.3?}; chosen {beta}");
    let (z0, z1, z2) = (m(10..18, &|s| run(s, true, 0.0, false)), m(10..18, &|s| run(s, true, beta, false)), m(10..18, &|s| run(s, true, 2.0 * beta, false)));
    let (u0, u1, u2) = (m(10..18, &|s| run(s, false, 0.0, false)), m(10..18, &|s| run(s, false, beta, false)), m(10..18, &|s| run(s, false, 2.0 * beta, false)));
    let (dc, tc) = (m(0..4, &|s| run(s, true, beta, true)), m(10..18, &|s| run(s, true, beta, true)));
    let dl = dev[betas.iter().position(|&b| b == beta).unwrap()];
    println!("  Zipf queries:    no lensing {z0:.3}, lensing {z1:.3}, beta x 2 {z2:.3}");
    println!("  uniform queries: no lensing {u0:.3}, lensing {u1:.3}, beta x 2 {u2:.3}");
    let chaos_kept = dc > dl && tc > z1;
    println!("  chaos on beta (Zipf): dev {dl:.3} -> {dc:.3}; test {z1:.3} -> {tc:.3}; kept {chaos_kept}");
    let v = [
        (format!("R5a Zipf: lensing raises identity >= 0.05 ({z1:.3} vs {z0:.3})"), z1 >= z0 + 0.05),
        (format!("R5b uniform: within +-0.02 ({u1:.3} vs {u0:.3})"), (u1 - u0).abs() <= 0.02),
        (format!("R5c beta x 2 overshoots under uniform (<= -0.02: {:+.3})", u2 - u0), u2 <= u0 - 0.02),
        ("R5d chaos not kept".to_string(), !chaos_kept),
    ];
    println!();
    for (nm, ok) in &v {
        println!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
