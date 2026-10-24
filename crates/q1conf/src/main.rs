//! Q1 (docs/PHYS2-POC.md): colour confinement as a closed-loop check on FOLD-7 records. An answer z
//! recalled from (x, y) is accepted only if the loop closes: x comes back from (y, z) and y from
//! (x, z). Plain accepts every answer. Chaos re-tries a rejected answer with jittered cues;
//! amplification closes the loop once more. Every setting is here.

use qb_amplitude::C;
use std::f64::consts::PI;

const D: usize = 4096;
const BITS: u32 = 2;

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
    (0..D).map(|_| C::polar(1.0, 2.0 * PI * r.unit())).collect()
}

fn recipe(seed: u64, kind: u64, i: usize) -> Vec<C> {
    phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ kind.wrapping_mul(0x6E7))
}

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

fn add(s: &mut [C], v: &[C], k: f64) {
    for (a, b) in s.iter_mut().zip(v) {
        *a = a.add(b.scale(k));
    }
}

fn quantized(s: &[C]) -> Vec<C> {
    let mut q = s.to_vec();
    qb_quantize::quantize(&mut q, BITS);
    q
}

/// (best index, margin) of u read against a codebook, phase-locked.
fn read(u: &[C], codes: &[Vec<C>]) -> (usize, f64) {
    let mut sc: Vec<(f64, usize)> = codes.iter().enumerate().map(|(v, c)| (qb_overlap_phase::overlap_phase(c, u), v)).collect();
    sc.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    (sc[0].1, sc[0].0 - sc[1].0)
}

fn par<T: Send>(seeds: std::ops::Range<u64>, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn keyed(seed: u64, a: u64, b: u64) -> f64 {
    qb_chaos::keyed_unit(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ a.wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ b.wrapping_mul(0x1656_67B1_9E37_79F9))
}

const M: usize = 256;
const NS: [usize; 4] = [64, 128, 192, 256];

/// (plain accuracy, accepted precision, coverage) for one store; mode 0 plain-check, 1 chaos, 2 amp.
fn run(n: usize, mode: u8, seed: u64) -> (f64, f64, f64) {
    let codes: Vec<Vec<C>> = (0..M).map(|v| recipe(seed, 3, v)).collect();
    let roles: Vec<Vec<C>> = (0..3).map(|j| recipe(seed, 7, j)).collect();
    let mut r = Rng(seed ^ 0x2EC);
    let recs: Vec<[usize; 3]> = (0..n).map(|_| [(r.next() % 256) as usize, (r.next() % 256) as usize, (r.next() % 256) as usize]).collect();
    let field = |j: usize, v: usize| qb_bind::bind(&roles[j], &codes[v]);
    let mut s = vec![C::new(0.0, 0.0); D];
    for x in &recs {
        add(&mut s, &qb_bind::bind(&qb_bind::bind(&field(0, x[0]), &field(1, x[1])), &field(2, x[2])), 1.0);
    }
    let q = quantized(&s);
    // Recall field j given the other two values (a, b at fields ja, jb); `jitter` perturbs the cue.
    let recall = |j: usize, known: [(usize, usize); 2], jitter: Option<(u64, u64)>| -> usize {
        let mut cue = qb_bind::bind(&qb_bind::bind(&field(known[0].0, known[0].1), &field(known[1].0, known[1].1)), &roles[j]);
        if let Some((a, b)) = jitter {
            for (k, c) in cue.iter_mut().enumerate() {
                *c = c.mul(C::polar(1.0, 0.2 * (keyed(seed ^ a, b, k as u64) - 0.5)));
            }
        }
        read(&qb_bind::bind(&q, &conj(&cue)), &codes).0
    };
    let closes = |x: usize, y: usize, z: usize| -> bool {
        let ok = recall(0, [(1, y), (2, z)], None) == x && recall(1, [(0, x), (2, z)], None) == y;
        if mode == 2 && ok {
            return recall(2, [(0, x), (1, y)], None) == z;
        }
        ok
    };
    let (mut plain_ok, mut acc, mut acc_ok) = (0, 0, 0);
    for (k, x) in recs.iter().enumerate() {
        let z = recall(2, [(0, x[0]), (1, x[1])], None);
        plain_ok += (z == x[2]) as usize;
        let mut answer = if closes(x[0], x[1], z) { Some(z) } else { None };
        if answer.is_none() && mode == 1 {
            for v in 1..=3u64 {
                let zv = recall(2, [(0, x[0]), (1, x[1])], Some((k as u64, v)));
                if closes(x[0], x[1], zv) {
                    answer = Some(zv);
                    break;
                }
            }
        }
        if let Some(a) = answer {
            acc += 1;
            acc_ok += (a == x[2]) as usize;
        }
    }
    (plain_ok as f64 / n as f64, acc_ok as f64 / acc.max(1) as f64, acc as f64 / n as f64)
}

fn mean3(v: &[(f64, f64, f64)]) -> (f64, f64, f64) {
    let k = v.len() as f64;
    (v.iter().map(|x| x.0).sum::<f64>() / k, v.iter().map(|x| x.1).sum::<f64>() / k, v.iter().map(|x| x.2).sum::<f64>() / k)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("Q1 colour confinement: records (x, y, z) in a 2-bit store (D {D}); an answer is accepted only if its loop closes");
    let mut at192 = (0.0, 0.0, 0.0);
    for &n in &NS {
        let t = mean3(&par(10..18, |s| run(n, 0, s)));
        println!("  N {n:3}: plain accuracy {:.3}; closed-loop precision {:.3} at coverage {:.3}", t.0, t.1, t.2);
        if n == 192 {
            at192 = t;
        }
    }
    let keep = |base: (f64, f64, f64), on: (f64, f64, f64), cov: bool| if cov { on.2 > base.2 && on.1 >= base.1 - 0.005 } else { on.1 > base.1 };
    let (db, dc, da) = (mean3(&par(0..4, |s| run(192, 0, s))), mean3(&par(0..4, |s| run(192, 1, s))), mean3(&par(0..4, |s| run(192, 2, s))));
    let (tc, ta) = (mean3(&par(10..18, |s| run(192, 1, s))), mean3(&par(10..18, |s| run(192, 2, s))));
    let chaos_kept = keep(db, dc, true) && keep(at192, tc, true);
    let amp_kept = keep(db, da, false) && keep(at192, ta, false);
    println!("  chaos (N 192): dev prec/cov {:.3}/{:.3} -> {:.3}/{:.3}; test {:.3}/{:.3} -> {:.3}/{:.3}; kept {chaos_kept}", db.1, db.2, dc.1, dc.2, at192.1, at192.2, tc.1, tc.2);
    println!("  amplification, close twice (N 192): dev {:.3}/{:.3} -> {:.3}/{:.3}; test {:.3}/{:.3} -> {:.3}/{:.3}; kept {amp_kept}", db.1, db.2, da.1, da.2, at192.1, at192.2, ta.1, ta.2);
    let v = [
        (format!("C1 N 192: precision >= 0.98 ({:.3}) where plain <= 0.9 ({:.3}), coverage >= 0.5 ({:.3})", at192.1, at192.0, at192.2), at192.1 >= 0.98 && at192.0 <= 0.9 && at192.2 >= 0.5),
        ("C2 chaos not kept".to_string(), !chaos_kept),
        ("C3 amplification not kept".to_string(), !amp_kept),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
