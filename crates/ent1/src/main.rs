//! ENT-1 orchestrator (docs/ENT-1-PRECOMMIT.md): entanglement as coordination between two
//! co-engines that never exchange a message (the CHSH game). Each round: prepare the pair; module A
//! sees only x and its own qubit, module B only y and its own; each rotates its qubit by −angle and
//! measures; they win if α ⊕ β = x ∧ y. Heat = depolarization (the pair replaced by a random basis
//! state); a tap measures B's qubit before B does; chaos moves the four angles, selected by the
//! stack's own win rate. Every setting is here. Quantum-derived maths on a classical PC (Rule 4).

use qb_amplitude::{State, C};
use std::f64::consts::PI;

const ROUNDS: usize = 200_000;
const BATCH: usize = 4_000;
const STEPS: usize = 300;
const ALPHA: f64 = 0.2;

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

#[derive(Clone, Copy)]
struct Arm {
    entangled: bool,
    heat: f64,
    tap: bool,
}

/// Module A owns qubit 0, module B qubit 1. Angles [a0, a1, b0, b1].
fn round(arm: Arm, ang: &[f64; 4], r: &mut Rng) -> bool {
    let (x, y) = ((r.next() & 1) as usize, (r.next() & 1) as usize);
    let h = 0.5f64.sqrt();
    let mut s = if arm.entangled { State { a: vec![C::new(h, 0.0), C::new(0.0, 0.0), C::new(0.0, 0.0), C::new(h, 0.0)] } } else { State { a: vec![C::new(1.0, 0.0), C::new(0.0, 0.0), C::new(0.0, 0.0), C::new(0.0, 0.0)] } };
    if arm.heat > 0.0 && r.unit() < arm.heat {
        s.project((r.next() % 4) as usize);
    }
    if arm.tap {
        qb_measure_qubit::measure(&mut s, 1, r.unit());
    }
    // Module A: its input and its qubit only.
    qb_local_rotate::rotate(&mut s, 0, -ang[x]);
    let alpha = qb_measure_qubit::measure(&mut s, 0, r.unit());
    // Module B: its input and its qubit only.
    qb_local_rotate::rotate(&mut s, 1, -ang[2 + y]);
    let beta = qb_measure_qubit::measure(&mut s, 1, r.unit());
    (alpha ^ beta) == (x & y)
}

fn win_rate(arm: Arm, ang: &[f64; 4], n: usize, seed: u64) -> f64 {
    let mut r = Rng(seed);
    (0..n).filter(|_| round(arm, ang, &mut r)).count() as f64 / n as f64
}

/// The classical best: both always answer 0 (wins unless x = y = 1).
fn classical(n: usize, seed: u64) -> f64 {
    let mut r = Rng(seed);
    (0..n).filter(|_| {
        let (x, y) = (r.next() & 1, r.next() & 1);
        (x & y) == 0
    }).count() as f64 / n as f64
}

/// Chaos from the classical saddle (all angles 0): each step move the angles by α(u − ½), keep the
/// move only if the batch win rate strictly improves. Returns the final angles.
fn climb(arm: Arm, seed: u64, chaos: bool) -> [f64; 4] {
    let mut ang = [0.0; 4];
    let mut best = win_rate(arm, &ang, BATCH, seed ^ 0xBA7C);
    if !chaos {
        return ang;
    }
    for t in 0..STEPS {
        let mut cand = ang;
        for (k, a) in cand.iter_mut().enumerate() {
            let key = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (k as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
            *a += ALPHA * (qb_chaos::keyed_unit(key) - 0.5);
        }
        let sc = win_rate(arm, &cand, BATCH, seed ^ 0xBA7C ^ ((t as u64 + 1) << 20));
        if sc > best {
            best = sc;
            ang = cand;
        }
    }
    ang
}

fn main() {
    let t0 = std::time::Instant::now();
    let ent = Arm { entangled: true, heat: 0.0, tap: false };
    let textbook = [0.0, PI / 4.0, PI / 8.0, -PI / 8.0];
    let w = win_rate(ent, &textbook, ROUNDS, 1);
    let s = 8.0 * (w - 0.5);
    println!("ENT-1: the CHSH game between two co-engines, {ROUNDS} rounds per arm");
    println!("  E1 entangled, textbook angles: win {w:.4}, S {s:.4} (Tsirelson 0.8536, 2.8284)");
    let c = classical(ROUNDS, 2);
    let u = win_rate(Arm { entangled: false, ..ent }, &textbook, ROUNDS, 3);
    println!("  E2 classical best {c:.4}; unentangled pair, same angles {u:.4}");
    let ps = [0.0, 0.1, 0.2, 0.3, 0.5];
    let mut e3 = true;
    for (i, &p) in ps.iter().enumerate() {
        let wp = win_rate(Arm { heat: p, ..ent }, &textbook, ROUNDS, 10 + i as u64);
        let law = 0.5 + 2.0 * 2f64.sqrt() * (1.0 - p) / 8.0;
        e3 &= (wp - law).abs() <= 0.005;
        println!("  E3 heat p {p:.1}: win {wp:.4} (law {law:.4})");
    }
    let wt = win_rate(Arm { tap: true, ..ent }, &textbook, ROUNDS, 20);
    println!("  E4 tapped: win {wt:.4} (law 0.6768)");
    println!("  E5/E6 chaos from the classical saddle (all angles 0), {STEPS} steps, batch {BATCH}, fresh {ROUNDS}-round score:");
    let mut e5 = 0;
    let mut e6 = true;
    let mut plain_stays = true;
    for sd in 10u64..18 {
        let plain = climb(ent, sd, false);
        let wp = win_rate(ent, &plain, ROUNDS, sd ^ 0xF5E5);
        plain_stays &= (wp - 0.75).abs() <= 0.005;
        let on = climb(ent, sd, true);
        let wo = win_rate(ent, &on, ROUNDS, sd ^ 0xF5E5);
        e5 += (wo >= 0.84) as usize;
        let tw = climb(Arm { entangled: false, ..ent }, sd, true);
        let wtw = win_rate(Arm { entangled: false, ..ent }, &tw, ROUNDS, sd ^ 0xF5E5);
        e6 &= wtw <= 0.755;
        println!("    seed {sd}: plain {wp:.4}; chaos {wo:.4} (angles {:.3?}); classical twin under chaos {wtw:.4}", on);
    }
    let v = [
        ("E1 entangled 0.8536 +- 0.004 and S 2.83 +- 0.03", (w - 0.8536).abs() <= 0.004 && (s - 2.8284).abs() <= 0.03),
        ("E2 classical best 0.75 +- 0.004; unentangled with the same angles <= 0.75", (c - 0.75).abs() <= 0.004 && u <= 0.75),
        ("E3 heat follows S = 2 sqrt2 (1 - p) within 0.005", e3),
        ("E4 tapped 0.677 +- 0.005 (worse than classical)", (wt - 0.6768).abs() <= 0.005),
        ("E5 chaos reaches >= 0.84 from the saddle in >= 7 of 8 seeds (plain stays at 0.75)", e5 >= 7 && plain_stays),
        ("E6 the classical twin under chaos stays <= 0.755", e6),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.1} s); chaos reached 0.84 in {e5} of 8", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
