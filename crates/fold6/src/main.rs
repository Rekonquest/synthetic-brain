//! FOLD-6 orchestrator (docs/FOLD-6-PRECOMMIT.md): does folding save data? N byte values are bound
//! to random keys and superposed in one phase store of D amplitudes, held at full precision or at
//! b bits of phase; each value is recalled by unbinding with its key and reading the 256 value codes
//! phase-locked. Counted against raw storage (1 byte per byte) and xz. Keys and codes are recipes
//! (made from the seed), like raw storage's implicit index. Every setting is here.
//! Run from the repository root (it reads docs/GLOSSARY.md).

use qb_amplitude::C;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

const D: usize = 4096;
const M: usize = 256;
const THREADS: usize = 8;
const NS: [usize; 12] = [32, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048];
/// Bits per amplitude: 0 = full precision (complex f64, 128 bits).
const PRECISIONS: [u32; 5] = [0, 8, 4, 2, 1];
const DAMAGE: f64 = 0.3;
const XZ_GLOSSARY: f64 = 5200.0 / 12203.0;

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

fn key(seed: u64, i: usize) -> Vec<C> {
    phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ 0x6E7)
}

fn codebook(seed: u64) -> Vec<Vec<C>> {
    (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ 0xC0D)).collect()
}

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

/// Fold `content` into one store (quantized to `bits` if > 0), optionally damage a fraction of it,
/// and return how many values come back exactly.
fn fold_and_recall(content: &[u8], bits: u32, damage: f64, seed: u64, codes: &[Vec<C>]) -> usize {
    let mut state = vec![C::new(0.0, 0.0); D];
    for (i, &v) in content.iter().enumerate() {
        for (s, b) in state.iter_mut().zip(qb_bind::bind(&key(seed, i), &codes[v as usize])) {
            *s = s.add(b);
        }
    }
    if bits > 0 {
        qb_quantize::quantize(&mut state, bits);
    }
    if damage > 0.0 {
        let mut r = Rng(seed ^ 0xDA4A_6E);
        let levels = 1u64 << bits.max(1);
        for s in state.iter_mut() {
            if r.unit() < damage {
                let k = (r.next() % levels) as f64;
                *s = C::polar(1.0, k * 2.0 * std::f64::consts::PI / levels as f64);
            }
        }
    }
    content
        .iter()
        .enumerate()
        .filter(|(i, &v)| {
            let u = qb_bind::bind(&state, &conj(&key(seed, *i)));
            let best = (0..M).max_by(|&a, &b| qb_overlap_phase::overlap_phase(&codes[a], &u).partial_cmp(&qb_overlap_phase::overlap_phase(&codes[b], &u)).unwrap()).unwrap();
            best == v as usize
        })
        .count()
}

fn random_content(seed: u64, n: usize) -> Vec<u8> {
    let mut r = Rng(seed ^ 0xB17E5);
    (0..n).map(|_| (r.next() % 256) as u8).collect()
}

/// The M-ary symmetric channel's information per item at accuracy p.
fn info(p: f64) -> f64 {
    let m = M as f64;
    let mut i = m.log2();
    if p > 0.0 {
        i += p * p.log2();
    }
    if p < 1.0 {
        i += (1.0 - p) * ((1.0 - p) / (m - 1.0)).log2();
    }
    i.max(0.0)
}

fn stored_bits(bits: u32) -> f64 {
    D as f64 * if bits == 0 { 128.0 } else { bits as f64 }
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

/// Run jobs (content kind, precision, N, seed) on THREADS workers; correct counts per job.
fn run(jobs: &[(bool, u32, usize, u64, f64)], glossary: &[u8]) -> Vec<usize> {
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<usize>> = jobs.iter().map(|_| std::sync::Mutex::new(0)).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (text, bits, n, sd, damage) = jobs[j];
                let codes = codebook(sd);
                let content = if text { glossary[..n].to_vec() } else { random_content(sd, n) };
                *out[j].lock().unwrap() = fold_and_recall(&content, bits, damage, sd, &codes);
            });
        }
    });
    out.into_iter().map(|m| m.into_inner().unwrap()).collect()
}

fn main() {
    let t0 = std::time::Instant::now();
    let glossary = std::fs::read("docs/GLOSSARY.md").expect("run from the repository root");
    let seeds: Vec<u64> = (10..18).collect();
    say(format!("FOLD-6: one phase store of D {D}, byte values (M {M}) bound to recipe keys; test seeds 10-17; raw = 1 byte per byte, xz on the glossary = {XZ_GLOSSARY:.3}"));
    let mut jobs = Vec::new();
    for text in [false, true] {
        for &b in &PRECISIONS {
            for &n in &NS {
                for &sd in &seeds {
                    jobs.push((text, b, n, sd, 0.0));
                }
            }
        }
    }
    let res = run(&jobs, &glossary);
    let mut verdicts: Vec<(String, bool)> = Vec::new();
    // exact[text][b index], best efficiency[b index] (random content)
    let mut exact = [[0usize; 5]; 2];
    let mut eff = [0.0f64; 5];
    let mut k = 0;
    for (ti, text) in [false, true].iter().enumerate() {
        say(format!("\n{} content: accuracy p(N) per precision", if *text { "Glossary (real)" } else { "Random" }));
        for (bi, &b) in PRECISIONS.iter().enumerate() {
            let mut line = format!("  {:>4}:", if b == 0 { "f64".to_string() } else { format!("{b} b") });
            let mut prefix = true;
            for &n in &NS {
                let c = &res[k..k + seeds.len()];
                k += seeds.len();
                let p = c.iter().sum::<usize>() as f64 / (n * seeds.len()) as f64;
                let all = c.iter().all(|&x| x == n);
                if prefix && all {
                    exact[ti][bi] = n;
                } else {
                    prefix = false;
                }
                if !text {
                    eff[bi] = eff[bi].max(n as f64 * info(p) / stored_bits(b));
                }
                line += &format!(" {n}:{p:.3}");
            }
            say(line);
            let bytes = stored_bits(b) / 8.0;
            let ne = exact[ti][bi];
            say(format!(
                "        lossless N_exact {ne}: {} stored bytes per content byte{}",
                if ne > 0 { format!("{:.1}", bytes / ne as f64) } else { "-".into() },
                if *text { String::new() } else { format!("; best lossy efficiency {:.4} recalled bits per stored bit", eff[bi]) }
            ));
        }
    }
    let cost = |ti: usize, bi: usize| if exact[ti][bi] > 0 { stored_bits(PRECISIONS[bi]) / 8.0 / exact[ti][bi] as f64 } else { f64::INFINITY };
    let best_b = (3..5).min_by(|&a, &b| cost(0, a).partial_cmp(&cost(0, b)).unwrap()).unwrap();
    let best_eff = eff.iter().cloned().fold(0.0, f64::max);
    verdicts.push((format!("R1 full precision lossless cost >= 100 bytes per byte ({:.1})", cost(0, 0)), cost(0, 0) >= 100.0));
    verdicts.push((
        format!("R2 best of 1-2 bits: lossless 2-10 bytes per byte ({:.2} at {} b) and lossy 0.1-0.5 bits per bit ({:.3})", cost(0, best_b), PRECISIONS[best_b], eff[3].max(eff[4])),
        (2.0..=10.0).contains(&cost(0, best_b)) && (0.1..=0.5).contains(&eff[3].max(eff[4])),
    ));
    verdicts.push((format!("R3 never beats raw on random data (best {best_eff:.3} < 1)"), best_eff < 1.0));
    let r4a = (0..5).all(|bi| exact[0][bi] == 0 || ((exact[1][bi] as f64 - exact[0][bi] as f64).abs() <= 0.2 * exact[0][bi] as f64));
    let best_text = (0..5).map(|bi| cost(1, bi)).fold(f64::INFINITY, f64::min);
    verdicts.push((format!("R4 redundancy ignored (glossary N_exact within 20% of random) and xz beats the best fold by >= 5x ({:.1}x)", best_text / XZ_GLOSSARY), r4a && best_text >= 5.0 * XZ_GLOSSARY));

    // R5: damage tolerance in the 1-bit store at N_exact / 2.
    let n5 = (exact[0][4] / 2).max(1);
    let djobs: Vec<(bool, u32, usize, u64, f64)> = seeds.iter().map(|&sd| (false, 1, n5, sd, DAMAGE)).collect();
    let dres = run(&djobs, &glossary);
    let p5 = dres.iter().sum::<usize>() as f64 / (n5 * seeds.len()) as f64;
    say(format!("\nDamage: 1-bit store, N {n5}, {:.0}% of amplitudes overwritten: accuracy {p5:.3} (raw storage keeps about {:.2})", DAMAGE * 100.0, 1.0 - DAMAGE * 255.0 / 256.0));
    verdicts.push((format!("R5 30% damage, recall >= 0.95 ({p5:.3})"), p5 >= 0.95));

    say(String::new());
    for (name, ok) in &verdicts {
        say(format!("{} {name}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", verdicts.iter().filter(|x| x.1).count(), verdicts.len(), t0.elapsed().as_secs_f64()));
}
