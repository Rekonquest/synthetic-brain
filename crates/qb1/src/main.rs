//! QB-1 (docs/QB-1-FORWARD-TEST.md): the first forward test of the quantum-base characters.
//!
//! The orchestrator composes the base (`qb-amplitude`) and the classical hooks (`qb-classical`) per
//! arm, runs every arm on the same evidence and the same uniform stream per seed, and scores the
//! seven precommitted predictions. Every setting lives in `Settings`.

use qb_amplitude::State;
use qb_classical::{argmax, conserve, decay, draw, reinforce};
use std::f64::consts::PI;

/// Every setting, fixed by the precommitment.
#[derive(Debug, Clone)]
struct Settings {
    n: usize,
    steps: usize,
    switch_at: usize,
    best_before: usize,
    best_after: usize,
    p_best: f64,
    eta: f64,
    lambda: f64,
    phi: f64,
    theta: f64,
    seeds: u64,
}

impl Settings {
    fn qb1() -> Settings {
        Settings { n: 8, steps: 400, switch_at: 200, best_before: 2, best_after: 5, p_best: 0.4, eta: 0.15, lambda: 0.02, phi: PI / 12.0, theta: PI / 3.0, seeds: 32 }
    }
    fn best(&self, t: usize) -> usize {
        if t < self.switch_at { self.best_before } else { self.best_after }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Base {
    Amplitudes,
    L1,
    Greedy,
}

#[derive(Debug, Clone, Copy)]
struct Arm {
    name: &'static str,
    base: Base,
    hooks: bool,
    mixer: bool,
    phase: bool,
    collapse: bool,
}

const fn amp(name: &'static str, mixer: bool, phase: bool, collapse: bool) -> Arm {
    Arm { name, base: Base::Amplitudes, hooks: true, mixer, phase, collapse }
}

const ARMS: [Arm; 9] = [
    amp("born", false, false, false),
    amp("born-collapse", false, false, true),
    amp("born-collapse-mix", true, false, true),
    amp("born-mix", true, false, false),
    amp("born-phase", false, true, false),
    amp("born-mix-phase", true, true, false),
    Arm { name: "l1", base: Base::L1, hooks: true, mixer: false, phase: false, collapse: false },
    Arm { name: "greedy", base: Base::Greedy, hooks: true, mixer: false, phase: false, collapse: false },
    Arm { name: "base-only", base: Base::Amplitudes, hooks: false, mixer: false, phase: false, collapse: false },
];

/// SplitMix64.
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

/// The evidence stream (observed option per step) and the uniform stream for draws, for one seed.
fn streams(s: &Settings, seed: u64) -> (Vec<usize>, Vec<f64>) {
    let mut ev = Rng(seed.wrapping_mul(0x1000_0001).wrapping_add(17));
    let evidence = (0..s.steps)
        .map(|t| if ev.unit() < s.p_best { s.best(t) } else { ((ev.unit() * s.n as f64) as usize).min(s.n - 1) })
        .collect();
    let mut un = Rng(seed ^ 0xA5A5_5A5A_DEAD_BEEF);
    let uniforms = (0..s.steps).map(|_| un.unit()).collect();
    (evidence, uniforms)
}

/// One arm on one seed: the choice and the largest choice probability at each step.
fn run(arm: &Arm, s: &Settings, evidence: &[usize], uniforms: &[f64]) -> (Vec<usize>, Vec<f64>) {
    let mut state = State::uniform(s.n);
    let mut w = vec![1.0 / s.n as f64; s.n];
    let mut choices = Vec::with_capacity(s.steps);
    let mut decisive = Vec::with_capacity(s.steps);
    for t in 0..s.steps {
        let k = evidence[t];
        match arm.base {
            Base::Amplitudes => {
                if arm.mixer {
                    state.ring_mix(s.phi);
                }
                if arm.hooks {
                    if arm.phase {
                        state.rotate(k, s.theta);
                    }
                    state.scale(k, 1.0 + s.eta);
                    state.power_magnitudes(1.0 - s.lambda);
                    state.normalize();
                }
                let p = state.born_weights();
                let c = draw(&p, uniforms[t]);
                decisive.push(p.iter().cloned().fold(0.0, f64::max));
                if arm.collapse {
                    state.project(c);
                }
                choices.push(c);
            }
            Base::L1 | Base::Greedy => {
                reinforce(&mut w, k, s.eta);
                decay(&mut w, s.lambda);
                conserve(&mut w);
                let c = if arm.base == Base::L1 { draw(&w, uniforms[t]) } else { argmax(&w) };
                decisive.push(if arm.base == Base::L1 { w.iter().cloned().fold(0.0, f64::max) } else { f64::NAN });
                choices.push(c);
            }
        }
    }
    (choices, decisive)
}

#[derive(Debug, Clone, Default)]
struct Score {
    acc1: f64,
    acc2: f64,
    latency: f64,
    decisive: f64,
    change_rate: f64,
}

fn score(s: &Settings, choices: &[usize], decisive: &[f64]) -> Score {
    let acc = |a: usize, b: usize| (a..b).filter(|&t| choices[t] == s.best(t)).count() as f64 / (b - a) as f64;
    let mut latency = 200.0;
    for t in s.switch_at..s.steps {
        if t >= 9 && (t - 9..=t).filter(|&u| choices[u] == s.best_after).count() >= 5 {
            latency = (t - s.switch_at) as f64;
            break;
        }
    }
    let changes = choices.windows(2).filter(|p| p[0] != p[1]).count();
    Score {
        acc1: acc(50, 200),
        acc2: acc(300, 400),
        latency,
        decisive: decisive.iter().sum::<f64>() / decisive.len() as f64,
        change_rate: changes as f64 / (choices.len() - 1) as f64,
    }
}

fn main() {
    let s = Settings::qb1();
    println!("QB-1: {} options, {} steps, best {} -> {} at t={}, {} seeds", s.n, s.steps, s.best_before, s.best_after, s.switch_at, s.seeds);
    // per arm: per seed (choices, decisive, score)
    let mut all: Vec<Vec<(Vec<usize>, Score)>> = vec![Vec::new(); ARMS.len()];
    for seed in 0..s.seeds {
        let (evidence, uniforms) = streams(&s, seed);
        for (i, arm) in ARMS.iter().enumerate() {
            let (c, d) = run(arm, &s, &evidence, &uniforms);
            let sc = score(&s, &c, &d);
            all[i].push((c, sc));
        }
    }
    let mean = |i: usize, f: fn(&Score) -> f64| all[i].iter().map(|(_, sc)| f(sc)).sum::<f64>() / s.seeds as f64;
    let idx = |name: &str| ARMS.iter().position(|a| a.name == name).unwrap();
    println!("\n{:<18} {:>6} {:>6} {:>8} {:>8} {:>8}", "arm", "acc1", "acc2", "latency", "decisive", "changes");
    for (i, arm) in ARMS.iter().enumerate() {
        println!(
            "{:<18} {:>6.3} {:>6.3} {:>8.1} {:>8.3} {:>8.3}",
            arm.name,
            mean(i, |x| x.acc1),
            mean(i, |x| x.acc2),
            mean(i, |x| x.latency),
            mean(i, |x| x.decisive),
            mean(i, |x| x.change_rate)
        );
    }

    let (born, bc, bcm, bm, bp, bmp, l1, greedy, base) =
        (idx("born"), idx("born-collapse"), idx("born-collapse-mix"), idx("born-mix"), idx("born-phase"), idx("born-mix-phase"), idx("l1"), idx("greedy"), idx("base-only"));
    let chance = |v: f64| (0.05..=0.25).contains(&v);
    let mut verdicts: Vec<(&str, bool, String)> = Vec::new();

    let pinned = all[bc].iter().filter(|(_, sc)| sc.change_rate == 0.0).count();
    let p1 = pinned == s.seeds as usize && chance(mean(bc, |x| x.acc1)) && chance(mean(bc, |x| x.acc2));
    verdicts.push(("P1 commit and stay committed", p1, format!("pinned on {pinned}/{} seeds; acc1 {:.3}, acc2 {:.3}", s.seeds, mean(bc, |x| x.acc1), mean(bc, |x| x.acc2))));

    let cr = mean(bcm, |x| x.change_rate);
    let p2 = (0.02..=0.25).contains(&cr) && mean(bcm, |x| x.acc1) > mean(bc, |x| x.acc1) + 0.1 && mean(bcm, |x| x.acc1) < mean(born, |x| x.acc1);
    verdicts.push(("P2 collapse+mix = Zeno-limited exploration", p2, format!("change rate {cr:.3}; acc1 {:.3} (collapse {:.3}, born {:.3})", mean(bcm, |x| x.acc1), mean(bc, |x| x.acc1), mean(born, |x| x.acc1))));

    let sharper = (0..s.seeds as usize).filter(|&k| all[born][k].1.decisive > all[l1][k].1.decisive).count();
    let lat_gap = (mean(born, |x| x.latency) - mean(l1, |x| x.latency)).abs();
    let p3 = sharper == s.seeds as usize && mean(born, |x| x.acc1) > mean(l1, |x| x.acc1) && lat_gap <= 20.0;
    verdicts.push(("P3 squaring peaks", p3, format!("born sharper on {sharper}/{}; acc1 born {:.3} vs l1 {:.3}; latency gap {lat_gap:.1}", s.seeds, mean(born, |x| x.acc1), mean(l1, |x| x.acc1))));

    let same = (0..s.seeds as usize).filter(|&k| all[bp][k].0 == all[born][k].0).count();
    verdicts.push(("P4a phase inert without a mixer", same == s.seeds as usize, format!("born-phase identical to born on {same}/{}", s.seeds)));
    let differ = (0..s.seeds as usize).filter(|&k| all[bmp][k].0 != all[bm][k].0).count();
    verdicts.push(("P4b phase active with a mixer", differ >= 30, format!("born-mix-phase differs from born-mix on {differ}/{}", s.seeds)));

    let p5 = chance(mean(base, |x| x.acc1)) && chance(mean(base, |x| x.acc2));
    verdicts.push(("P5 base alone gives no verdict", p5, format!("acc1 {:.3}, acc2 {:.3}", mean(base, |x| x.acc1), mean(base, |x| x.acc2))));

    let p6 = mean(bm, |x| x.acc1) < mean(born, |x| x.acc1) && mean(bm, |x| x.decisive) < mean(born, |x| x.decisive) && mean(bm, |x| x.latency) <= mean(born, |x| x.latency) + 5.0;
    verdicts.push(("P6 mixing spreads and blurs", p6, format!("acc1 {:.3} vs {:.3}; decisive {:.3} vs {:.3}; latency {:.1} vs {:.1}", mean(bm, |x| x.acc1), mean(born, |x| x.acc1), mean(bm, |x| x.decisive), mean(born, |x| x.decisive), mean(bm, |x| x.latency), mean(born, |x| x.latency))));

    let p7 = mean(greedy, |x| x.acc1) >= mean(born, |x| x.acc1);
    verdicts.push(("P7 classical ties or wins at the target", p7, format!("greedy acc1 {:.3} vs born {:.3}", mean(greedy, |x| x.acc1), mean(born, |x| x.acc1))));

    println!();
    for (name, ok, detail) in &verdicts {
        println!("{} {name}: {detail}", if *ok { "HOLDS " } else { "FAILS " });
    }
    let fails = verdicts.iter().filter(|v| !v.1).count();
    println!("\n{} of {} predictions hold; kill criterion (>= 3 fail): {}", verdicts.len() - fails, verdicts.len(), if fails >= 3 { "FIRES" } else { "does not fire" });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amplitude_magnitudes_stay_proportional_to_the_l1_weights() {
        // The premise of P3: under REINFORCE, DECAY and CONSERVE, |a| is proportional to w, so Born and
        // L1 differ only in the squaring.
        let s = Settings::qb1();
        let (evidence, _) = streams(&s, 3);
        let mut state = State::uniform(s.n);
        let mut w = vec![1.0 / s.n as f64; s.n];
        for &k in &evidence {
            state.scale(k, 1.0 + s.eta);
            state.power_magnitudes(1.0 - s.lambda);
            state.normalize();
            reinforce(&mut w, k, s.eta);
            decay(&mut w, s.lambda);
            conserve(&mut w);
            let ratio = state.a[0].abs() / w[0];
            for i in 0..s.n {
                assert!((state.a[i].abs() / w[i] / ratio - 1.0).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn every_arm_sees_the_same_streams() {
        let s = Settings::qb1();
        assert_eq!(streams(&s, 5), streams(&s, 5));
        assert_ne!(streams(&s, 5).0, streams(&s, 6).0);
        let (e, _) = streams(&s, 1);
        let hits = e.iter().enumerate().filter(|(t, &k)| k == s.best(*t)).count() as f64 / e.len() as f64;
        assert!((0.40..0.55).contains(&hits), "{hits}");
    }
}
