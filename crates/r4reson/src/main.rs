//! PHYS3 R4 (docs/PHYS3-POC.md): orbital resonance composed with binding by synchrony. A scene has 3
//! objects (colour, shape). Each active feature oscillator is entrained (coupling K) by its object's
//! location-tagged drive, a phase advancing at the object's own frequency with noise; units of one
//! object lock in phase. Readout: each colour pairs with the shape nearest in phase. Bar: no phase
//! (random pairing).

use std::f64::consts::PI;

const OBJECTS: usize = 3;
const SCENES: usize = 1000;
const STEPS: usize = 500;
const DT: f64 = 0.1;
const K: f64 = 1.0;
const OMEGA: [f64; 3] = [1.0, 1.37, 1.81];

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

fn dphase(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(2.0 * PI);
    d.min(2.0 * PI - d)
}

/// Fraction of scenes with every colour paired to its own shape.
fn run(sigma: f64, k: f64, phase: bool, seed: u64) -> f64 {
    run_c(sigma, k, phase, false, seed)
}

/// `chaos`: K jittered per unit and step by 1 + 0.5(u - 1/2) (logistic, mean 1/2).
fn run_c(sigma: f64, k: f64, phase: bool, chaos: bool, seed: u64) -> f64 {
    let mut r = Rng(seed);
    let mut ok = 0;
    for _ in 0..SCENES {
        let mut drive: Vec<f64> = (0..OBJECTS).map(|_| 2.0 * PI * r.unit()).collect();
        // Units: colour of object o is unit o, shape of object o is unit OBJECTS + o.
        let mut th: Vec<f64> = (0..2 * OBJECTS).map(|_| 2.0 * PI * r.unit()).collect();
        if phase {
            for step in 0..STEPS {
                for (o, d) in drive.iter_mut().enumerate() {
                    *d += OMEGA[o] * DT + 0.05 * DT.sqrt() * r.normal();
                }
                for (u, t) in th.iter_mut().enumerate() {
                    let o = u % OBJECTS;
                    let kk = if chaos {
                        let mut x = 0.01 + 0.98 * ((seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (step as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (u as u64 + 1).wrapping_mul(0x1656_67B1_9E37_79F9)) >> 11) as f64 / (1u64 << 53) as f64;
                        for _ in 0..16 {
                            x = 4.0 * x * (1.0 - x);
                        }
                        k * (1.0 + 0.5 * (x - 0.5))
                    } else {
                        k
                    };
                    *t += DT * (OMEGA[o] + kk * (drive[o] - *t).sin()) + sigma * DT.sqrt() * r.normal();
                }
            }
        }
        let all = (0..OBJECTS).all(|c| {
            let pick = if phase {
                (0..OBJECTS).min_by(|&a, &b| dphase(th[c], th[OBJECTS + a]).partial_cmp(&dphase(th[c], th[OBJECTS + b])).unwrap()).unwrap()
            } else {
                (r.next() % OBJECTS as u64) as usize
            };
            pick == c
        });
        ok += all as usize;
    }
    ok as f64 / SCENES as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS3 R4: binding {OBJECTS} objects (colour + shape) by resonance; {SCENES} scenes per seed");
    let m = |f: &dyn Fn(u64) -> f64| (10..18u64).map(f).sum::<f64>() / 8.0;
    let bar = m(&|s| run(0.0, 0.0, false, s));
    println!("  bar (rate code, random pairing): {bar:.3}");
    let mut law = Vec::new();
    let mut amp = Vec::new();
    for sigma in [0.1, 0.3, 0.6] {
        let a = m(&|s| run(sigma, K, true, s));
        let b = m(&|s| run(sigma, 2.0 * K, true, s));
        println!("  noise {sigma}: resonance K {K}: {a:.3}; K x 2: {b:.3}");
        law.push(a);
        amp.push(b);
    }
    let md = |f: &dyn Fn(u64) -> f64| (0..4u64).map(f).sum::<f64>() / 4.0;
    let (dp, dc) = (md(&|s| run_c(0.6, K, true, false, s)), md(&|s| run_c(0.6, K, true, true, s)));
    let tc = m(&|s| run_c(0.6, K, true, true, s));
    let chaos_kept = dc > dp && tc > law[2];
    println!("  chaos on K (noise 0.6): dev {dp:.3} -> {dc:.3}; test {:.3} -> {tc:.3}; kept {chaos_kept}", law[2]);
    let v = [
        ("R4d chaos not kept".to_string(), !chaos_kept),
        (format!("R4a all pairs right >= 0.95 at noise 0.1 ({:.3}; bar {bar:.3})", law[0]), law[0] >= 0.95),
        (format!("R4b still >= 0.6 at noise 0.6 ({:.3})", law[2]), law[2] >= 0.6),
        (format!("R4c K x 2 kept at noise 0.6 ({:.3} vs {:.3})", amp[2], law[2]), amp[2] > law[2]),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
