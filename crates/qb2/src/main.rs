//! QB-2 (docs/QB-2-HEAT-AND-CHAOS.md): hot, cold and chaos on the quantum-base stacks.
//!
//! The task, settings, seeds and measures are QB-1's. The new knobs are decision temperature, phase
//! heat (dephasing), occupation heat, and frozen or changing logistic chaos, plus a transport probe on
//! a 256-site ring that measures the mechanism directly. Every setting lives in `Settings`.

use qb_amplitude::{State, C};
use qb_chaos::{iterate, logistic};
use qb_classical::{argmax, conserve, decay, draw, reinforce, temper};
use std::f64::consts::PI;

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
    seeds: u64,
    chaos_warmup: usize,
    probe_sites: usize,
    probe_steps: usize,
}

impl Settings {
    fn qb2() -> Settings {
        Settings { n: 8, steps: 400, switch_at: 200, best_before: 2, best_after: 5, p_best: 0.4, eta: 0.15, lambda: 0.02, phi: PI / 12.0, seeds: 32, chaos_warmup: 20, probe_sites: 256, probe_steps: 100 }
    }
    fn best(&self, t: usize) -> usize {
        if t < self.switch_at { self.best_before } else { self.best_after }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Chaos {
    None,
    Frozen,
    Changing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Base {
    Amplitudes,
    L1,
    Greedy,
}

#[derive(Debug, Clone)]
struct Arm {
    name: String,
    base: Base,
    mixer: bool,
    collapse: bool,
    temp: f64,
    heat: f64,
    occupation: f64,
    chaos: Chaos,
}

fn amp(name: &str) -> Arm {
    Arm { name: name.to_owned(), base: Base::Amplitudes, mixer: false, collapse: false, temp: 1.0, heat: 0.0, occupation: 0.0, chaos: Chaos::None }
}

fn arms() -> Vec<Arm> {
    let mut v = vec![amp("born"), Arm { base: Base::L1, ..amp("l1") }, Arm { base: Base::Greedy, ..amp("greedy") }];
    for t in [0.25, 0.5, 2.0, 4.0] {
        v.push(Arm { temp: t, ..amp(&format!("born-T{t}")) });
    }
    for h in [0.3, 1.0, PI] {
        v.push(Arm { heat: h, ..amp(&format!("born-heat{h:.2}")) });
    }
    v.push(Arm { chaos: Chaos::Frozen, ..amp("born-frozen") });
    for h in [0.0, 0.3, 1.0, PI] {
        v.push(Arm { mixer: true, heat: h, ..amp(&format!("mix-heat{h:.2}")) });
    }
    v.push(Arm { mixer: true, chaos: Chaos::Frozen, ..amp("mix-frozen") });
    v.push(Arm { mixer: true, chaos: Chaos::Changing, ..amp("mix-changing") });
    v.push(Arm { mixer: true, collapse: true, heat: PI, ..amp("collapse-mix-heatpi") });
    v.push(Arm { collapse: true, heat: PI, ..amp("collapse-heatpi") });
    for e in [0.01, 0.05] {
        v.push(Arm { collapse: true, occupation: e, ..amp(&format!("collapse-occ{e}")) });
    }
    v
}

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

/// QB-1's evidence and uniform streams (same construction, so the arms are comparable to QB-1's).
fn streams(s: &Settings, seed: u64) -> (Vec<usize>, Vec<f64>) {
    let mut ev = Rng(seed.wrapping_mul(0x1000_0001).wrapping_add(17));
    let evidence = (0..s.steps).map(|t| if ev.unit() < s.p_best { s.best(t) } else { ((ev.unit() * s.n as f64) as usize).min(s.n - 1) }).collect();
    let mut un = Rng(seed ^ 0xA5A5_5A5A_DEAD_BEEF);
    (evidence, (0..s.steps).map(|_| un.unit()).collect())
}

/// Per-site logistic values, warmed up.
fn chaos_sites(s: &Settings, seed: u64, sites: usize) -> Vec<f64> {
    let mut r = Rng(seed ^ 0x0C0A_05C0_FFEE_0001);
    (0..sites).map(|_| iterate(0.05 + 0.9 * r.unit(), s.chaos_warmup)).collect()
}

/// The disorder or heat applied to every site before the mixer, for one step.
fn kick(state: &mut State, heat: f64, chaos: Chaos, xs: &mut [f64], heat_rng: &mut Rng) {
    if heat > 0.0 {
        for j in 0..state.a.len() {
            state.rotate(j, (2.0 * heat_rng.unit() - 1.0) * heat);
        }
    }
    if chaos != Chaos::None {
        if chaos == Chaos::Changing {
            for x in xs.iter_mut() {
                *x = logistic(*x);
            }
        }
        for (j, x) in xs.iter().enumerate() {
            state.rotate(j, 2.0 * PI * x);
        }
    }
}

fn run(arm: &Arm, s: &Settings, seed: u64, evidence: &[usize], uniforms: &[f64]) -> (Vec<usize>, Vec<f64>) {
    let mut state = State::uniform(s.n);
    let mut w = vec![1.0 / s.n as f64; s.n];
    let mut xs = chaos_sites(s, seed, s.n);
    let mut heat_rng = Rng(seed ^ 0x4EA7_4EA7_0000_0002);
    let mut occ_rng = Rng(seed ^ 0x0CC0_0CC0_0000_0003);
    let (mut choices, mut decisive) = (Vec::new(), Vec::new());
    for t in 0..s.steps {
        let k = evidence[t];
        match arm.base {
            Base::Amplitudes => {
                kick(&mut state, arm.heat, arm.chaos, &mut xs, &mut heat_rng);
                if arm.mixer {
                    state.ring_mix(s.phi);
                }
                if arm.occupation > 0.0 {
                    for j in 0..s.n {
                        state.excite(j, C::polar(arm.occupation, 2.0 * PI * occ_rng.unit()));
                    }
                }
                state.scale(k, 1.0 + s.eta);
                state.power_magnitudes(1.0 - s.lambda);
                state.normalize();
                let born = state.born_weights();
                let p = if arm.temp == 1.0 { born } else { temper(&born, arm.temp) };
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
                decisive.push(w.iter().cloned().fold(0.0, f64::max));
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

fn score(s: &Settings, c: &[usize], d: &[f64]) -> Score {
    let acc = |a: usize, b: usize| (a..b).filter(|&t| c[t] == s.best(t)).count() as f64 / (b - a) as f64;
    let mut latency = 200.0;
    for t in s.switch_at..s.steps {
        if (t - 9..=t).filter(|&u| c[u] == s.best_after).count() >= 5 {
            latency = (t - s.switch_at) as f64;
            break;
        }
    }
    Score {
        acc1: acc(50, 200),
        acc2: acc(300, 400),
        latency,
        decisive: d.iter().sum::<f64>() / d.len() as f64,
        change_rate: c.windows(2).filter(|p| p[0] != p[1]).count() as f64 / (c.len() - 1) as f64,
    }
}

/// The transport probe: mean squared ring distance after `probe_steps` of mixing from site 0.
fn probe(s: &Settings, seed: u64, heat: f64, chaos: Chaos) -> f64 {
    let n = s.probe_sites;
    let mut state = State { a: vec![C::new(0.0, 0.0); n] };
    state.a[0] = C::new(1.0, 0.0);
    let mut xs = chaos_sites(s, seed, n);
    let mut heat_rng = Rng(seed ^ 0x4EA7_4EA7_0000_0002);
    for _ in 0..s.probe_steps {
        kick(&mut state, heat, chaos, &mut xs, &mut heat_rng);
        state.ring_mix(s.phi);
    }
    state.a.iter().enumerate().map(|(j, c)| {
        let d = j.min(n - j) as f64;
        c.norm2() * d * d
    }).sum()
}

fn main() {
    let s = Settings::qb2();
    let arms = arms();
    println!("QB-2: {} options, {} steps, best {} -> {} at t={}, {} seeds", s.n, s.steps, s.best_before, s.best_after, s.switch_at, s.seeds);
    let mut all: Vec<Vec<(Vec<usize>, Score)>> = vec![Vec::new(); arms.len()];
    for seed in 0..s.seeds {
        let (e, u) = streams(&s, seed);
        for (i, a) in arms.iter().enumerate() {
            let (c, d) = run(a, &s, seed, &e, &u);
            let sc = score(&s, &c, &d);
            all[i].push((c, sc));
        }
    }
    let idx = |name: &str| arms.iter().position(|a| a.name == name).unwrap_or_else(|| panic!("no arm {name}"));
    let mean = |name: &str, f: fn(&Score) -> f64| all[idx(name)].iter().map(|(_, x)| f(x)).sum::<f64>() / s.seeds as f64;
    let same_as = |a: &str, b: &str| (0..s.seeds as usize).filter(|&k| all[idx(a)][k].0 == all[idx(b)][k].0).count();
    let pinned = |a: &str| all[idx(a)].iter().filter(|(_, x)| x.change_rate == 0.0).count();

    println!("\n{:<22} {:>6} {:>6} {:>8} {:>8} {:>8}", "arm", "acc1", "acc2", "latency", "decisive", "changes");
    for a in &arms {
        println!("{:<22} {:>6.3} {:>6.3} {:>8.1} {:>8.3} {:>8.3}", a.name, mean(&a.name, |x| x.acc1), mean(&a.name, |x| x.acc2), mean(&a.name, |x| x.latency), mean(&a.name, |x| x.decisive), mean(&a.name, |x| x.change_rate));
    }

    let m2 = |heat: f64, chaos: Chaos| (0..s.seeds).map(|seed| probe(&s, seed, heat, chaos)).sum::<f64>() / s.seeds as f64;
    let (ballistic, dephased, frozen, changing) = (m2(0.0, Chaos::None), m2(PI, Chaos::None), m2(0.0, Chaos::Frozen), m2(0.0, Chaos::Changing));
    println!("\ntransport probe (ring {}, {} steps), mean squared distance: ballistic {ballistic:.1}, phase heat {dephased:.1}, frozen chaos {frozen:.1}, changing chaos {changing:.1}", s.probe_sites, s.probe_steps);

    let chance = |v: f64| (0.05..=0.25).contains(&v);
    let mut v: Vec<(&str, bool, String)> = Vec::new();
    let q1 = same_as("born-T2", "l1");
    v.push(("Q1 squaring is a cold knob", q1 >= 30, format!("T=2 equals l1 on {q1}/{}", s.seeds)));
    let temps = ["born-T0.25", "born-T0.5", "born", "born-T2", "born-T4"];
    let accs: Vec<f64> = temps.iter().map(|t| mean(t, |x| x.acc1)).collect();
    let q2 = accs.windows(2).all(|p| p[1] <= p[0] + 0.01) && accs[0] >= 0.97;
    v.push(("Q2 colder is sharper", q2, format!("acc1 over T 0.25..4: {:?}", accs.iter().map(|a| format!("{a:.3}")).collect::<Vec<_>>())));
    let crs: Vec<f64> = temps.iter().map(|t| mean(t, |x| x.change_rate)).collect();
    v.push(("Q3 hotter explores more", crs.windows(2).all(|p| p[1] >= p[0] - 0.01), format!("change rate over T: {:?}", crs.iter().map(|a| format!("{a:.3}")).collect::<Vec<_>>())));
    let inert: Vec<usize> = ["born-heat0.30", "born-heat1.00", "born-heat3.14", "born-frozen"].iter().map(|a| same_as(a, "born")).collect();
    v.push(("Q4 phase noise inert without a mixer", inert.iter().all(|&n| n == s.seeds as usize), format!("identical to born on {inert:?} of {}", s.seeds)));
    let mix: Vec<f64> = ["mix-heat0.00", "mix-heat0.30", "mix-heat1.00", "mix-heat3.14"].iter().map(|a| mean(a, |x| x.acc1)).collect();
    let q5 = mix[3] >= mix[0] + 0.15 && mean("mix-heat3.14", |x| x.latency) <= mean("mix-heat0.00", |x| x.latency) - 40.0 && mix.windows(2).all(|p| p[1] >= p[0] - 0.02);
    v.push(("Q5 heat rescues the mixer", q5, format!("acc1 over heat 0..pi: {:?}; latency {:.1} -> {:.1}", mix.iter().map(|a| format!("{a:.3}")).collect::<Vec<_>>(), mean("mix-heat0.00", |x| x.latency), mean("mix-heat3.14", |x| x.latency))));
    let q6 = mean("collapse-mix-heatpi", |x| x.acc1);
    v.push(("Q6 heat cannot restore wiped memory", chance(q6), format!("collapse+mix+heat acc1 {q6:.3}")));
    let q7a = pinned("collapse-heatpi");
    v.push(("Q7a phase heat cannot unlock collapse", q7a == s.seeds as usize, format!("pinned on {q7a}/{}", s.seeds)));
    let (o1, o5) = (mean("collapse-occ0.01", |x| x.change_rate), mean("collapse-occ0.05", |x| x.change_rate));
    let q7b = o1 > 0.0 && o5 > o1 && chance(mean("collapse-occ0.01", |x| x.acc1)) && chance(mean("collapse-occ0.05", |x| x.acc1));
    v.push(("Q7b occupation heat unlocks, without memory", q7b, format!("change rate {o1:.3} -> {o5:.3}; acc1 {:.3}, {:.3}", mean("collapse-occ0.01", |x| x.acc1), mean("collapse-occ0.05", |x| x.acc1))));
    let q8 = mean("mix-frozen", |x| x.acc1) >= mix[0] + 0.15 && mean("mix-frozen", |x| x.latency) <= mean("mix-heat0.00", |x| x.latency) - 40.0;
    v.push(("Q8 frozen chaos traps transport", q8, format!("acc1 {:.3} vs {:.3}; latency {:.1} vs {:.1}", mean("mix-frozen", |x| x.acc1), mix[0], mean("mix-frozen", |x| x.latency), mean("mix-heat0.00", |x| x.latency))));
    let q9 = (mean("mix-changing", |x| x.acc1) - mix[3]).abs() <= 0.10;
    v.push(("Q9 changing chaos acts like heat", q9, format!("acc1 {:.3} vs heat {:.3}", mean("mix-changing", |x| x.acc1), mix[3])));
    let q10 = frozen < 0.25 * dephased && dephased < ballistic && changing <= 2.0 * dephased && changing >= 0.5 * dephased;
    v.push(("Q10 the probe separates the mechanisms", q10, format!("frozen {frozen:.1} < 0.25*heat {:.1}; heat {dephased:.1} < ballistic {ballistic:.1}; changing {changing:.1} within 2x of heat", 0.25 * dephased)));

    println!();
    for (name, ok, d) in &v {
        println!("{} {name}: {d}", if *ok { "HOLDS " } else { "FAILS " });
    }
    // Q7a/Q7b count as one prediction (Q7), as in the precommitment.
    let failed: Vec<&str> = v.iter().filter(|x| !x.1).map(|x| x.0.split(' ').next().unwrap_or("")).collect();
    let mut distinct: Vec<&str> = failed.iter().map(|f| if f.starts_with("Q7") { "Q7" } else { f }).collect();
    distinct.dedup();
    println!("\n{} of {} checks hold; predictions failed: {:?}; kill criterion (>= 4 fail): {}", v.len() - failed.len(), v.len(), distinct, if distinct.len() >= 4 { "FIRES" } else { "does not fire" });
}
