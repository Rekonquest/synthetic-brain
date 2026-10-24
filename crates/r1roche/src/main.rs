//! PHYS3 R1 (docs/PHYS3-POC.md): the Roche limit as memory placement. 16 regions, each a 2-bit
//! association store of D 1024; 512 memories, a quarter strong (amplitude 3). Plain places them at
//! random; the Roche law keeps weak memories out of regions holding bodies more than r x stronger.

use qb_amplitude::C;
use std::f64::consts::PI;

const D: usize = 1024;
const REGIONS: usize = 16;
const PER: usize = 32;
const STRONG: f64 = 3.0;

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

/// (weak recall, strong recall) for a placement rule: roche_r 0 = random, else the Roche law with
/// limit r (strong memories may only share with bodies within r x their strength).
fn run(roche_r: f64, seed: u64) -> (f64, f64) {
    run_full(roche_r, 1.0, 0, seed).0
}

/// (weak, strong) recall and the writer's readback margin; `weak_amp` scales the weak memories;
/// `variant` > 0 shuffles each strength class by keyed chaos.
fn run_full(roche_r: f64, weak_amp: f64, variant: u64, seed: u64) -> ((f64, f64), f64) {
    let n = REGIONS * PER;
    let codes: Vec<Vec<C>> = (0..256).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64 + 1) * 0x9E37)).collect();
    let mut r = Rng(seed ^ 0x0C4E);
    let strength: Vec<f64> = (0..n).map(|i| if i < n / 4 { STRONG } else { weak_amp }).collect();
    let strong = |i: usize| i < n / 4;
    let vals: Vec<usize> = (0..n).map(|_| (r.next() % 256) as usize).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| phasors(seed ^ (i as u64 + 7).wrapping_mul(0xA24B_AED4_963E_E407))).collect();
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (r.next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    if variant > 0 {
        let mut cr = Rng(seed ^ variant.wrapping_mul(0x5851_F42D_4C95_7F2D));
        for i in (1..n).rev() {
            let j = (cr.next() % (i as u64 + 1)) as usize;
            if strong(order[i]) == strong(order[j]) {
                order.swap(i, j);
            }
        }
    }
    // Placement.
    let region: Vec<usize> = if roche_r > 0.0 && STRONG > roche_r {
        let mut reg = vec![0; n];
        let (mut s, mut w) = (0, 0);
        for &i in &order {
            if strong(i) {
                reg[i] = s / PER;
                s += 1;
            } else {
                reg[i] = n / 4 / PER + w / PER;
                w += 1;
            }
        }
        reg
    } else {
        let mut reg = vec![0; n];
        for (k, &i) in order.iter().enumerate() {
            reg[i] = k / PER;
        }
        reg
    };
    let mut stores = vec![vec![C::new(0.0, 0.0); D]; REGIONS];
    for i in 0..n {
        for (a, b) in stores[region[i]].iter_mut().zip(qb_bind::bind(&keys[i], &codes[vals[i]])) {
            *a = a.add(b.scale(strength[i]));
        }
    }
    for s in stores.iter_mut() {
        qb_quantize::quantize(s, 2);
    }
    let (mut wk, mut wn, mut sk, mut sn) = (0, 0, 0, 0);
    let mut margin = 0.0;
    for i in 0..n {
        let u: Vec<C> = stores[region[i]].iter().zip(&keys[i]).map(|(a, k)| a.mul(C::new(k.re, -k.im))).collect();
        let sc: Vec<f64> = (0..256).map(|v| qb_overlap_phase::overlap_phase(&codes[v], &u)).collect();
        let best = (0..256).max_by(|&a, &b| sc[a].partial_cmp(&sc[b]).unwrap()).unwrap();
        let other = (0..256).filter(|&v| v != vals[i]).map(|v| sc[v]).fold(f64::MIN, f64::max);
        margin += sc[vals[i]] - other;
        let ok = (best == vals[i]) as usize;
        if strong(i) {
            sk += ok;
            sn += 1;
        } else {
            wk += ok;
            wn += 1;
        }
    }
    ((wk as f64 / wn as f64, sk as f64 / sn as f64), margin)
}

fn mean(f: impl Fn(u64) -> (f64, f64) + Sync) -> (f64, f64) {
    let v: Vec<(f64, f64)> = std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = (10..18u64).map(|s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    (v.iter().map(|x| x.0).sum::<f64>() / 8.0, v.iter().map(|x| x.1).sum::<f64>() / 8.0)
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PHYS3 R1: the Roche limit; {REGIONS} regions x {PER} memories (2-bit, D {D}); a quarter strong (x{STRONG})");
    let p = mean(|s| run(0.0, s));
    let ro = mean(|s| run(2.0, s));
    let am = mean(|s| run(4.0, s));
    println!("  random placement (bar): weak {:.3}, strong {:.3}", p.0, p.1);
    println!("  Roche law, r 2:         weak {:.3}, strong {:.3}", ro.0, ro.1);
    println!("  gentler limit, r 4:     weak {:.3}, strong {:.3}", am.0, am.1);
    // Chaos on the Roche placement: 3 shuffled variants, the writer's readback margin selects.
    let pick = |s: u64| -> u64 {
        let mut best = (run_full(2.0, 1.0, 0, s).1, 0u64);
        for v in 1..=3 {
            let m = run_full(2.0, 1.0, v, s).1;
            if m > best.0 {
                best = (m, v);
            }
        }
        best.1
    };
    let ch = mean(|s| run_full(2.0, 1.0, pick(s), s).0);
    let chosen: Vec<u64> = (10..18u64).map(pick).collect();
    println!("  Roche + chaos placement:  weak {:.3}, strong {:.3} (variants chosen {chosen:?})", ch.0, ch.1);
    let (amp_rand, amp_roche) = (mean(|s| run_full(0.0, 2.0, 0, s).0), mean(|s| run_full(2.0, 2.0, 0, s).0));
    println!("  amplification, weak x 2:  random weak {:.3}, strong {:.3}; Roche weak {:.3}, strong {:.3}", amp_rand.0, amp_rand.1, amp_roche.0, amp_roche.1);
    let chaos_kept = ch.0 + ch.1 > ro.0 + ro.1;
    let v = [
        ("R1c chaos not kept".to_string(), !chaos_kept),
        (format!("R1d weak x 2 lifts random-placement weak ({:.3} vs {:.3}), ties under Roche ({:.3} vs {:.3})", amp_rand.0, p.0, amp_roche.0, ro.0), amp_rand.0 > p.0 && (amp_roche.0 - ro.0).abs() <= 0.01),
        (format!("R1a weak recall rises >= 0.1 ({:.3} vs {:.3})", ro.0, p.0), ro.0 >= p.0 + 0.1),
        (format!("R1b strong recall >= plain - 0.02 ({:.3} vs {:.3})", ro.1, p.1), ro.1 >= p.1 - 0.02),
    ];
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
