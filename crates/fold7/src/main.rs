//! FOLD-7 orchestrator, parts B and C (docs/FOLD-7-PRECOMMIT.md; part A is `fold5` PART=quant).
//! The store is FOLD-6's: one phase store of D amplitudes, held in 2 bits of phase, values bound to
//! recipe vectors and superposed; recall unbinds with the cue and reads the 256 value codes
//! phase-locked. Helpers copied from fold6; every setting is here.
//! - B: recall a byte from a partial cue (a fraction c of the key known) or a noisy cue (a fraction η
//!   of its phases random); B5: chaos on the noisy cue, selected by the reading's own margin;
//! - C: records (x, y, z) stored as (R1⊙code_x)⊙(R2⊙code_y)⊙(R3⊙code_z); any field from the other two.

use qb_amplitude::C;
use std::io::Write;

const D: usize = 4096;
const M: usize = 256;
const BITS: u32 = 2;
const N_B: usize = 48;
const PARTIAL: [f64; 4] = [1.0, 0.5, 0.25, 0.1];
const NOISE: [f64; 5] = [0.0, 0.3, 0.5, 0.7, 0.9];
const CHAOS_ETA: f64 = 0.7;
const CHAOS_ALPHA: f64 = 0.2;
const CHAOS_VARIANTS: usize = 3;
const NS_C: [usize; 4] = [16, 32, 48, 64];

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

fn phasors(seed: u64) -> Vec<C> {
    let mut r = Rng(seed);
    (0..D).map(|_| C::polar(1.0, 2.0 * std::f64::consts::PI * r.unit())).collect()
}

fn recipe(seed: u64, kind: u64, i: usize) -> Vec<C> {
    phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ kind.wrapping_mul(0x6E7))
}

fn codebook(seed: u64) -> Vec<Vec<C>> {
    (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ 0xC0D)).collect()
}

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

fn bytes(seed: u64, salt: u64, n: usize) -> Vec<u8> {
    let mut r = Rng(seed ^ salt);
    (0..n).map(|_| (r.next() % 256) as u8).collect()
}

/// Superpose the bound vectors and hold the sum in BITS bits of phase.
fn store(items: impl Iterator<Item = Vec<C>>) -> Vec<C> {
    let mut s = vec![C::new(0.0, 0.0); D];
    for v in items {
        for (a, b) in s.iter_mut().zip(v) {
            *a = a.add(b);
        }
    }
    qb_quantize::quantize(&mut s, BITS);
    s
}

/// The recalled value and the margin between the best and second-best code.
fn read(u: &[C], codes: &[Vec<C>]) -> (usize, f64) {
    let mut sc: Vec<(f64, usize)> = codes.iter().enumerate().map(|(v, c)| (qb_overlap_phase::overlap_phase(c, u), v)).collect();
    sc.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    (sc[0].1, sc[0].0 - sc[1].0)
}

struct SeedResult {
    partial: Vec<f64>,
    noise: Vec<f64>,
    chaos: (f64, f64),
    records: Vec<[f64; 3]>,
}

fn per_seed(seed: u64) -> SeedResult {
    let codes = codebook(seed);
    // B: the content store.
    let vals = bytes(seed, 0xB17E5, N_B);
    let keys: Vec<Vec<C>> = (0..N_B).map(|i| recipe(seed, 1, i)).collect();
    let st = store((0..N_B).map(|i| qb_bind::bind(&keys[i], &codes[vals[i] as usize])));
    let recall = |cue: &[C]| read(&qb_bind::bind(&st, &conj(cue)), &codes);
    let mut r = Rng(seed ^ 0xC0E);
    let partial: Vec<f64> = PARTIAL
        .iter()
        .map(|&c| (0..N_B).filter(|&i| recall(&keys[i].iter().map(|k| if r.unit() < c { *k } else { C::new(0.0, 0.0) }).collect::<Vec<_>>()).0 == vals[i] as usize).count() as f64 / N_B as f64)
        .collect();
    let noisy = |i: usize, eta: f64, r: &mut Rng| -> Vec<C> { keys[i].iter().map(|k| if r.unit() < eta { C::polar(1.0, 2.0 * std::f64::consts::PI * r.unit()) } else { *k }).collect() };
    let noise: Vec<f64> = NOISE.iter().map(|&eta| (0..N_B).filter(|&i| recall(&noisy(i, eta, &mut r)).0 == vals[i] as usize).count() as f64 / N_B as f64).collect();
    // B5: chaos on the noisy cue at η 0.7, selected by the reading's own margin; plain wins ties.
    let mut plain_ok = 0;
    let mut on_ok = 0;
    for i in 0..N_B {
        let cue = noisy(i, CHAOS_ETA, &mut r);
        let (pv, pm) = recall(&cue);
        let mut best = (pm, pv);
        for v in 1..=CHAOS_VARIANTS {
            let var: Vec<C> = cue
                .iter()
                .enumerate()
                .map(|(j, c)| {
                    let key = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((i * 8 + v) as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (j as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                    c.mul(C::polar(1.0, CHAOS_ALPHA * (qb_chaos::keyed_unit(key) - 0.5) * 2.0 * std::f64::consts::PI))
                })
                .collect();
            let (vv, vm) = recall(&var);
            if vm > best.0 {
                best = (vm, vv);
            }
        }
        plain_ok += (pv == vals[i] as usize) as usize;
        on_ok += (best.1 == vals[i] as usize) as usize;
    }
    let chaos = (plain_ok as f64 / N_B as f64, on_ok as f64 / N_B as f64);
    // C: records of three fields; any one from the other two.
    let roles: Vec<Vec<C>> = (0..3).map(|j| recipe(seed, 7, j)).collect();
    let records = NS_C
        .iter()
        .map(|&n| {
            let f: Vec<[u8; 3]> = (0..n).map(|k| {
                let b = bytes(seed ^ 0x2EC, k as u64 + 1, 3);
                [b[0], b[1], b[2]]
            }).collect();
            let field = |j: usize, v: u8| qb_bind::bind(&roles[j], &codes[v as usize]);
            let st = store(f.iter().map(|x| qb_bind::bind(&qb_bind::bind(&field(0, x[0]), &field(1, x[1])), &field(2, x[2]))));
            std::array::from_fn(|j| {
                let (a, b) = ((j + 1) % 3, (j + 2) % 3);
                f.iter()
                    .filter(|x| {
                        let u = qb_bind::bind(&qb_bind::bind(&qb_bind::bind(&st, &conj(&field(a, x[a]))), &conj(&field(b, x[b]))), &conj(&roles[j]));
                        read(&u, &codes).0 == x[j] as usize
                    })
                    .count() as f64
                    / n as f64
            })
        })
        .collect();
    SeedResult { partial, noise, chaos, records }
}

fn run(seeds: &[u64]) -> Vec<SeedResult> {
    std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || per_seed(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

fn main() {
    let t0 = std::time::Instant::now();
    let test: Vec<u64> = (10..18).collect();
    let dev: Vec<u64> = (0..4).collect();
    say(format!("FOLD-7 B and C: one phase store of D {D} in {BITS} bits ({} bytes); test seeds 10-17", D * BITS as usize / 8));
    let res = run(&test);
    let n = res.len() as f64;
    let mean = |g: &dyn Fn(&SeedResult) -> f64| res.iter().map(g).sum::<f64>() / n;
    let partial: Vec<f64> = (0..PARTIAL.len()).map(|k| mean(&|r| r.partial[k])).collect();
    let noise: Vec<f64> = (0..NOISE.len()).map(|k| mean(&|r| r.noise[k])).collect();
    say(format!("\nB, {N_B} bytes bound to keys; recall from a partial cue (fraction of the key known):"));
    for (c, p) in PARTIAL.iter().zip(&partial) {
        say(format!("  c {c:.2}: {p:.3}"));
    }
    say("  from a noisy cue (fraction of the key's phases random):".into());
    for (e, p) in NOISE.iter().zip(&noise) {
        say(format!("  eta {e:.1}: {p:.3}"));
    }
    let dres = run(&dev);
    let dn = dres.len() as f64;
    let (dp, don) = (dres.iter().map(|r| r.chaos.0).sum::<f64>() / dn, dres.iter().map(|r| r.chaos.1).sum::<f64>() / dn);
    let (tp, ton) = (mean(&|r| r.chaos.0), mean(&|r| r.chaos.1));
    let kept = don > dp && ton > tp;
    say(format!("  B5 chaos on the cue at eta {CHAOS_ETA}: dev plain {dp:.3} on {don:.3}; test plain {tp:.3} on {ton:.3}; kept {kept}"));
    say("\nC, records (x, y, z); recall each field from the other two (x | y | z):".into());
    let rec: Vec<[f64; 3]> = (0..NS_C.len()).map(|k| std::array::from_fn(|j| mean(&|r| r.records[k][j]))).collect();
    for (nn, r) in NS_C.iter().zip(&rec) {
        say(format!("  N {nn:2}: {:.3} | {:.3} | {:.3}", r[0], r[1], r[2]));
    }
    let i48 = NS_C.iter().position(|&x| x == 48).unwrap();
    let spread = rec.iter().map(|r| r.iter().cloned().fold(f64::MIN, f64::max) - r.iter().cloned().fold(f64::MAX, f64::min)).fold(0.0, f64::max);
    let v = [
        (format!("B1 a quarter of the key recalls >= 0.95 ({:.3})", partial[2]), partial[2] >= 0.95),
        (format!("B2 a tenth of the key recalls 0.4-0.85 ({:.3})", partial[3]), (0.4..=0.85).contains(&partial[3])),
        (format!("B3 half the key's phases wrong recalls >= 0.95 ({:.3})", noise[2]), noise[2] >= 0.95),
        (format!("B4 graceful: eta 0.7 in 0.3-0.9 ({:.3}), eta 0.9 <= 0.15 ({:.3})", noise[3], noise[4]), (0.3..=0.9).contains(&noise[3]) && noise[4] <= 0.15),
        (format!("B5 chaos on the cue not kept (test {:+.3})", ton - tp), !kept),
        (format!("C1 each direction >= 0.95 at N 48 ({:.3?})", rec[i48]), rec[i48].iter().all(|&x| x >= 0.95)),
        (format!("C2 the three directions within 0.03 at every N (largest spread {spread:.3})"), spread <= 0.03),
    ];
    say(String::new());
    for (name, ok) in &v {
        say(format!("{} {name}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64()));
}
