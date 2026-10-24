//! PHYS3 R2 orchestrator: hierarchical merging as a memory index for 1,024 shapes, against a linear scan. (docs/PHYS3-POC.md)

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

const LIB: usize = 1024;
const QUERIES: usize = 256;

/// Build the merge tree: rounds of mutual-nearest-centroid merging. Nodes: (centroid, children, leaf).
struct Node {
    c: Vec<f64>,
    kids: Vec<usize>,
    leaf: Option<usize>,
}

fn build(feats: &[Vec<f64>]) -> (Vec<Node>, Vec<usize>) {
    let mut nodes: Vec<Node> = feats.iter().enumerate().map(|(i, f)| Node { c: f.clone(), kids: vec![], leaf: Some(i) }).collect();
    let mut level: Vec<usize> = (0..feats.len()).collect();
    let size = |nodes: &Vec<Node>, i: usize| -> usize { fn s(nodes: &Vec<Node>, i: usize) -> usize { if nodes[i].leaf.is_some() { 1 } else { nodes[i].kids.iter().map(|&k| s(nodes, k)).sum() } } s(nodes, i) };
    while level.len() > 1 {
        let nn: Vec<usize> = level.iter().map(|&a| *level.iter().filter(|&&b| b != a).min_by(|&&x, &&y| l2(&nodes[a].c, &nodes[x].c).partial_cmp(&l2(&nodes[a].c, &nodes[y].c)).unwrap()).unwrap()).collect();
        let mut used = vec![false; nodes.len()];
        let mut next = Vec::new();
        for (k, &a) in level.iter().enumerate() {
            if used[a] {
                continue;
            }
            let b = nn[k];
            let kb = level.iter().position(|&x| x == b).unwrap();
            if nn[kb] == a && !used[b] {
                used[a] = true;
                used[b] = true;
                let (sa, sb) = (size(&nodes, a) as f64, size(&nodes, b) as f64);
                let c = nodes[a].c.iter().zip(&nodes[b].c).map(|(x, y)| (x * sa + y * sb) / (sa + sb)).collect();
                nodes.push(Node { c, kids: vec![a, b], leaf: None });
                next.push(nodes.len() - 1);
            }
        }
        for &a in &level {
            if !used[a] {
                next.push(a);
            }
        }
        level = next;
    }
    let root = level[0];
    (nodes, vec![root])
}

/// Descend with beam b (and chaos: one extra keyed-random child per level). (leaf, comparisons).
fn descend(nodes: &[Node], roots: &[usize], f: &[f64], beam: usize, chaos: Option<u64>) -> (usize, usize) {
    let mut frontier: Vec<usize> = roots.to_vec();
    let mut comps = 0;
    let mut leaves: Vec<usize> = Vec::new();
    let mut depth = 0u64;
    while !frontier.is_empty() {
        let mut kids: Vec<usize> = Vec::new();
        for &n in &frontier {
            if let Some(l) = nodes[n].leaf {
                leaves.push(l);
            } else {
                kids.extend(nodes[n].kids.iter().cloned());
            }
        }
        if kids.is_empty() {
            break;
        }
        comps += kids.len();
        let mut scored: Vec<(f64, usize)> = kids.iter().map(|&k| (l2(f, &nodes[k].c), k)).collect();
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        frontier = scored.iter().take(beam).map(|x| x.1).collect();
        if let Some(sd) = chaos {
            if scored.len() > beam {
                let u = qb_chaos::keyed_unit(sd.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ depth.wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (f[0].to_bits()));
                let extra = beam + ((u * (scored.len() - beam) as f64) as usize).min(scored.len() - beam - 1);
                frontier.push(scored[extra].1);
            }
        }
        depth += 1;
    }
    let best = *leaves.iter().min_by(|&&a, &&b| l2(f, &nodes[a].c).partial_cmp(&l2(f, &nodes[b].c)).unwrap()).unwrap();
    (nodes[best].leaf.unwrap(), comps + leaves.len())
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS3 R2: hierarchical merging as an index; {LIB} shapes, Noether features, noise 0.02, {QUERIES} queries per seed");
    let mut rows = vec![(0.0, 0.0); 5];
    for seed in 10..18u64 {
        let (lib, qs) = world(seed, LIB, QUERIES, 0.02, &|r: &mut Rng| (r.next() % LIB as u64) as usize);
        let feats: Vec<Vec<f64>> = lib.iter().map(|l| noether(l, false).0).collect();
        let (nodes, roots) = build(&feats);
        for (shape, q) in &qs {
            let f = noether(q, false).0;
            let scan = (0..LIB).min_by(|&a, &b| l2(&f, &feats[a]).partial_cmp(&l2(&f, &feats[b])).unwrap()).unwrap();
            let r = [(scan, LIB), descend(&nodes, &roots, &f, 1, None), descend(&nodes, &roots, &f, 4, None), descend(&nodes, &roots, &f, 1, Some(seed)), descend(&nodes, &roots, &f, 4, Some(seed))];
            for (k, (got, c)) in r.iter().enumerate() {
                rows[k].0 += (*got == *shape) as usize as f64;
                rows[k].1 += *c as f64;
            }
        }
    }
    let n = (8 * QUERIES) as f64;
    let names = ["linear scan (bar)", "merge tree, greedy (b 1)", "merge tree, beam 4 (amplified)", "greedy + chaos child", "beam 4 + chaos child"];
    for (nm, r) in names.iter().zip(&rows) {
        println!("  {nm:<32} identity {:.3}, comparisons {:.0}", r.0 / n, r.1 / n);
    }
    let a = |k: usize| rows[k].0 / n;
    let c = |k: usize| rows[k].1 / n;
    let v = [
        (format!("R2a greedy >= 0.9 x scan ({:.3} vs {:.3}) at <= 3% comparisons ({:.1}%)", a(1), a(0), 100.0 * c(1) / c(0)), a(1) >= 0.9 * a(0) && c(1) <= 0.03 * c(0)),
        (format!("R2b beam 4 within 0.01 of scan ({:.3}) at <= 10% ({:.1}%)", a(2), 100.0 * c(2) / c(0)), a(2) >= a(0) - 0.01 && c(2) <= 0.10 * c(0)),
        (format!("R2c chaos not kept (greedy {:.3} -> {:.3}; beam {:.3} -> {:.3})", a(1), a(3), a(2), a(4)), !(a(3) > a(1) && a(4) > a(2))),
    ];
    println!();
    for (nm, ok) in &v {
        println!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
