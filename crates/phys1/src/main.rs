//! PHYS-1 orchestrator (docs/PHYS-POC.md): the quantum Zeno effect as attention holding a thought.
//! A 16-option state starts in option 0 and drifts by ring_mix(φ) each step; every k steps it is
//! Born-measured and must be found in option 0 every time. The classical twin drifts by ring
//! diffusion and is observed the same way. Then chaos (jittered attention intervals, selected by dev
//! survival) and amplification (Born power 4). Every setting is here.

use qb_amplitude::{State, C};

const N: usize = 16;
const PHI: f64 = 0.01;
const T: usize = 200;
const TRIALS: usize = 4000;
const KS: [usize; 6] = [200, 20, 10, 5, 2, 1];
const K_ROUND: usize = 10;

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

/// Measurement times: every k (plain), or chaos-jittered intervals k(1 + 0.5(u − ½)) for variant v.
fn schedule(k: usize, variant: usize) -> Vec<usize> {
    let mut times = Vec::new();
    let mut t = 0usize;
    let mut i = 0u64;
    loop {
        let step = if variant == 0 { k as f64 } else { k as f64 * (1.0 + 0.5 * (qb_chaos_unit(variant as u64, i) - 0.5)) };
        t += (step.round() as usize).max(1);
        i += 1;
        if t >= T {
            break;
        }
        times.push(t);
    }
    times.push(T);
    times
}

fn qb_chaos_unit(v: u64, i: u64) -> f64 {
    let key = v.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ i.wrapping_mul(0x9FB2_1C65_1E98_DF25);
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let mut x = 0.01 + 0.98 * ((z >> 11) as f64 / (1u64 << 53) as f64);
    for _ in 0..16 {
        x = 4.0 * x * (1.0 - x);
    }
    x.clamp(1e-9, 1.0 - 1e-9)
}

fn quantum(times: &[usize], power: f64, trials: usize, seed: u64) -> f64 {
    let mut r = Rng(seed);
    let mut ok = 0;
    for _ in 0..trials {
        let mut s = State { a: vec![C::new(0.0, 0.0); N] };
        s.a[0] = C::new(1.0, 0.0);
        let mut alive = true;
        let mut next = 0;
        for t in 1..=T {
            s.ring_mix(PHI);
            if next < times.len() && times[next] == t {
                next += 1;
                let mut w = s.clone();
                w.power_magnitudes(power / 2.0); // |a|^(power/2), so the weights are |a|^power
                let out = qb_classical::draw(&w.born_weights(), r.unit());
                s.project(out);
                if out != 0 {
                    alive = false;
                    break;
                }
            }
        }
        ok += alive as usize;
    }
    ok as f64 / trials as f64
}

fn classical(times: &[usize], trials: usize, seed: u64) -> f64 {
    let mut r = Rng(seed);
    let mut ok = 0;
    for _ in 0..trials {
        let mut w = vec![0.0; N];
        w[0] = 1.0;
        let mut alive = true;
        let mut next = 0;
        for t in 1..=T {
            qb_classical::diffuse(&mut w, PHI * PHI);
            if next < times.len() && times[next] == t {
                next += 1;
                let out = qb_classical::draw(&w, r.unit());
                w = vec![0.0; N];
                w[out] = 1.0;
                if out != 0 {
                    alive = false;
                    break;
                }
            }
        }
        ok += alive as usize;
    }
    ok as f64 / trials as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS-1: the quantum Zeno effect (16 options, ring_mix {PHI}, T {T}, {TRIALS} trials per arm)");
    let mut q = Vec::new();
    let mut c = Vec::new();
    for &k in &KS {
        let times = schedule(k, 0);
        q.push(quantum(&times, 2.0, TRIALS, 10 + k as u64));
        c.push(classical(&times, TRIALS, 10 + k as u64));
        println!("  attention every {k:3} steps: quantum survival {:.3}, classical twin {:.3}", q.last().unwrap(), c.last().unwrap());
    }
    let rises = q.windows(2).all(|w| w[1] >= w[0] - 0.02);
    let cspread = c.iter().cloned().fold(f64::MIN, f64::max) - c.iter().cloned().fold(f64::MAX, f64::min);
    // Chaos on the attention interval at k = K_ROUND: 3 variants, dev survival selects, plain wins ties.
    let plain_t = schedule(K_ROUND, 0);
    let dev_plain = quantum(&plain_t, 2.0, TRIALS, 1);
    let mut best = (dev_plain, 0usize);
    for v in 1..=3 {
        let d = quantum(&schedule(K_ROUND, v), 2.0, TRIALS, 1);
        if d > best.0 {
            best = (d, v);
        }
    }
    let test_plain = quantum(&plain_t, 2.0, TRIALS, 2);
    let test_chaos = quantum(&schedule(K_ROUND, best.1), 2.0, TRIALS, 2);
    let chaos_kept = best.1 > 0 && test_chaos > test_plain;
    println!("  chaos on the attention interval (k {K_ROUND}): dev plain {dev_plain:.3}, best variant {} ({:.3}); test plain {test_plain:.3}, chaos {test_chaos:.3}; kept {chaos_kept}", best.1, best.0);
    // Amplification: Born power 4.
    let (dp, da) = (quantum(&plain_t, 2.0, TRIALS, 3), quantum(&plain_t, 4.0, TRIALS, 3));
    let (tp, ta) = (quantum(&plain_t, 2.0, TRIALS, 4), quantum(&plain_t, 4.0, TRIALS, 4));
    let amp_kept = da > dp && ta > tp;
    println!("  amplification (Born power 4, k {K_ROUND}): dev {dp:.3} -> {da:.3}; test {tp:.3} -> {ta:.3}; kept {amp_kept}");
    let v = [
        (format!("Z1 quantum survival rises with attention, <= 0.25 at k 200 ({:.3}) to >= 0.95 at k 1 ({:.3})", q[0], q[5]), rises && q[0] <= 0.25 && q[5] >= 0.95),
        (format!("Z2 classical survival varies <= 0.03 across k (spread {cspread:.3})"), cspread <= 0.03),
        ("C1 chaos not kept".to_string(), !chaos_kept),
        ("A1 amplification (Born power 4) kept".to_string(), amp_kept),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.1} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
