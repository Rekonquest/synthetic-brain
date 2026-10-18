//! QB-3 (docs/QB-3-COMPLEX-WORKING-MEMORY.md): a complex quantum-reinforced stack on a delayed cued
//! tracking task, against a tuned classical champion, simple stacks, base controls and one-at-a-time
//! ablations. Every setting lives in `Settings`; λ is tuned on development seeds only.

use qb_amplitude::State;
use qb_chaos::{iterate, tent};
use qb_classical::{argmax, conserve, decay, diffuse, draw, gate, reinforce, temper};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
struct Settings {
    n: usize,
    trials: usize,
    cue: usize,
    delay: usize,
    response: usize,
    p_cue: f64,
    p_response: f64,
    min_jump: usize,
    eta: f64,
    phi: f64,
    gate_k: usize,
    gate_m: usize,
    kappa: f64,
    k_m: f64,
    mu: f64,
    t0: f64,
    tent_slope: f64,
    chaos_warmup: usize,
    lambdas: [f64; 7],
    dev_seeds: std::ops::Range<u64>,
    test_seeds: std::ops::Range<u64>,
}

impl Settings {
    fn qb3() -> Settings {
        Settings {
            n: 16,
            trials: 40,
            cue: 5,
            delay: 20,
            response: 25,
            p_cue: 0.9,
            p_response: 0.25,
            min_jump: 3,
            eta: 0.15,
            phi: PI / 12.0,
            gate_k: 2,
            gate_m: 3,
            kappa: 1.0,
            k_m: 0.1,
            mu: 0.5,
            t0: 0.25,
            tent_slope: 1.99,
            chaos_warmup: 20,
            lambdas: [0.0, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2],
            dev_seeds: 0..16,
            test_seeds: 100..132,
        }
    }
    fn spec(&self) -> qb_task::Spec {
        qb_task::Spec { n: self.n, trials: self.trials, cue: self.cue, delay: self.delay, response: self.response, p_cue: self.p_cue, p_response: self.p_response, min_jump: self.min_jump }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Base {
    Amplitudes,
    Weights,
    Greedy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Order {
    A,
    B,
}

#[derive(Debug, Clone)]
struct Cfg {
    name: String,
    base: Base,
    order: Order,
    hooks: bool,
    mixer: bool,
    frozen: bool,
    gate: bool,
    chem: bool,
    inertia: bool,
    chaos_temp: bool,
    t0: f64,
    lambda: f64,
}

fn candidate(name: &str, order: Order, s: &Settings) -> Cfg {
    Cfg { name: name.to_owned(), base: Base::Amplitudes, order, hooks: true, mixer: true, frozen: true, gate: true, chem: true, inertia: true, chaos_temp: true, t0: s.t0, lambda: 0.0 }
}

use qb_task::{world, Rng, World};

/// Every choice of one configuration on one world.
fn run(cfg: &Cfg, s: &Settings, seed: u64, w: &World) -> Vec<usize> {
    let n = s.n;
    let mut st = State::uniform(n);
    let mut prev = st.clone();
    let mut wt = vec![1.0 / n as f64; n];
    let mut wprev = wt.clone();
    let mut cr = Rng(seed ^ 0x0C0A_05C0_FFEE_0001);
    let xs: Vec<f64> = (0..n).map(|_| iterate(0.05 + 0.9 * cr.unit(), s.chaos_warmup)).collect();
    let mut y = 0.05 + 0.9 * Rng(seed ^ 0x7E47_7E47_0000_0004).unit();
    let mut choices = Vec::with_capacity(w.obs.len());
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        let window = &w.obs[t.saturating_sub(s.gate_m - 1)..=t];
        let passes = !cfg.gate || gate(window, o, s.gate_k);
        let temp = if cfg.chaos_temp {
            y = tent(y, s.tent_slope);
            cfg.t0 * (0.5 + y)
        } else {
            cfg.t0
        };
        match cfg.base {
            Base::Amplitudes => {
                let transport = |st: &mut State| {
                    if cfg.frozen {
                        for (j, x) in xs.iter().enumerate() {
                            st.rotate(j, 2.0 * PI * x);
                        }
                    }
                    if cfg.mixer {
                        st.ring_mix(s.phi);
                    }
                };
                let hooks = |st: &mut State| {
                    if cfg.hooks {
                        if passes {
                            let f = if cfg.chem {
                                let p = st.born_weights()[o];
                                1.0 + s.eta * (1.0 + s.kappa * p / (p + s.k_m))
                            } else {
                                1.0 + s.eta
                            };
                            st.scale(o, f);
                        }
                        st.power_magnitudes(1.0 - cfg.lambda);
                        st.normalize();
                    }
                };
                let inertia = |st: &mut State, prev: &State| {
                    if cfg.inertia {
                        st.blend(prev, s.mu);
                        st.normalize();
                    }
                };
                match cfg.order {
                    Order::A => {
                        transport(&mut st);
                        hooks(&mut st);
                        inertia(&mut st, &prev);
                    }
                    Order::B => {
                        hooks(&mut st);
                        inertia(&mut st, &prev);
                        transport(&mut st);
                    }
                }
                let born = st.born_weights();
                let p = if temp == 1.0 { born } else { temper(&born, temp) };
                choices.push(draw(&p, w.uniforms[t]));
                prev = st.clone();
            }
            Base::Weights => {
                if cfg.mixer {
                    diffuse(&mut wt, s.phi.sin().powi(2));
                }
                if passes {
                    let f = if cfg.chem {
                        let p = wt[o];
                        1.0 + s.eta * (1.0 + s.kappa * p / (p + s.k_m))
                    } else {
                        1.0 + s.eta
                    };
                    wt[o] *= f;
                }
                decay(&mut wt, cfg.lambda);
                conserve(&mut wt);
                if cfg.inertia {
                    for (x, p) in wt.iter_mut().zip(&wprev) {
                        *x = (1.0 - s.mu) * *x + s.mu * p;
                    }
                    conserve(&mut wt);
                }
                let p = temper(&wt, temp);
                choices.push(draw(&p, w.uniforms[t]));
                wprev = wt.clone();
            }
            Base::Greedy => {
                if passes {
                    reinforce(&mut wt, o, s.eta);
                }
                decay(&mut wt, cfg.lambda);
                conserve(&mut wt);
                choices.push(argmax(&wt));
            }
        }
    }
    choices
}

use qb_task::{score, Score};

fn mean_score(cfg: &Cfg, s: &Settings, seeds: std::ops::Range<u64>) -> Score {
    let k = (seeds.end - seeds.start) as f64;
    let mut m = Score::default();
    for seed in seeds {
        let w = world(&s.spec(), seed);
        let sc = score(&s.spec(), &w, &run(cfg, s, seed, &w));
        m.acc_resp += sc.acc_resp / k;
        m.acc_early += sc.acc_early / k;
        m.persev += sc.persev / k;
    }
    m
}

/// The λ from the grid with the best development acc_resp.
fn tune(cfg: &Cfg, s: &Settings) -> Cfg {
    let mut best = (f64::MIN, cfg.clone());
    for &l in &s.lambdas {
        let c = Cfg { lambda: l, ..cfg.clone() };
        let a = mean_score(&c, s, s.dev_seeds.clone()).acc_resp;
        if a > best.0 {
            best = (a, c);
        }
    }
    best.1
}

fn main() {
    let s = Settings::qb3();
    println!("QB-3: {} options, {} trials x (cue {} + delay {} + response {}), dev seeds {:?}, test seeds {:?}", s.n, s.trials, s.cue, s.delay, s.response, s.dev_seeds, s.test_seeds);

    // The classical champion: greedy, gate on/off x lambda grid, chosen on development seeds.
    let mut champ = (f64::MIN, None);
    println!("\nclassical sweep (development seeds): gate, lambda, acc_resp, acc_early, persev");
    let mut sweep_gate_on = Vec::new();
    for g in [false, true] {
        for &l in &s.lambdas {
            let c = Cfg { name: "classical".into(), base: Base::Greedy, order: Order::A, hooks: true, mixer: false, frozen: false, gate: g, chem: false, inertia: false, chaos_temp: false, t0: 1.0, lambda: l };
            let m = mean_score(&c, &s, s.dev_seeds.clone());
            println!("  {g:<5} {l:<6} {:.3} {:.3} {:.3}", m.acc_resp, m.acc_early, m.persev);
            if g {
                sweep_gate_on.push(m);
            }
            if m.acc_resp > champ.0 {
                champ = (m.acc_resp, Some(c));
            }
        }
    }
    let champion = Cfg { name: "classical-champion".into(), ..champ.1.unwrap() };
    let a = tune(&candidate("candidate-A", Order::A, &s), &s);
    let b = tune(&candidate("candidate-B", Order::B, &s), &s);
    let cold = tune(&Cfg { name: "simple-cold-born".into(), mixer: false, frozen: false, gate: false, chem: false, inertia: false, chaos_temp: false, ..candidate("x", Order::A, &s) }, &s);
    let simple = Cfg { name: "simple-born (QB-1)".into(), mixer: false, frozen: false, gate: false, chem: false, inertia: false, chaos_temp: false, t0: 1.0, lambda: 0.02, ..candidate("x", Order::A, &s) };
    let base_only = Cfg { name: "base-only".into(), hooks: false, mixer: false, frozen: false, gate: false, chem: false, inertia: false, chaos_temp: false, t0: 1.0, ..candidate("x", Order::A, &s) };
    let replacement = Cfg { name: "replacement-base".into(), base: Base::Weights, frozen: false, ..a.clone() };
    let abl = |name: &str, f: &dyn Fn(&mut Cfg)| {
        let mut c = a.clone();
        c.name = name.into();
        f(&mut c);
        c
    };
    let ablations = vec![
        abl("A -frozen", &|c| c.frozen = false),
        abl("A -mixer", &|c| {
            c.mixer = false;
            c.frozen = false
        }),
        abl("A -gate", &|c| c.gate = false),
        abl("A -chemistry", &|c| c.chem = false),
        abl("A -inertia", &|c| c.inertia = false),
        abl("A -chaotic-temp", &|c| c.chaos_temp = false),
        abl("A warm (T0=1)", &|c| c.t0 = 1.0),
    ];
    let mut table: Vec<(Cfg, Score)> = Vec::new();
    for c in [champion.clone(), a.clone(), b.clone(), cold, simple, base_only, replacement].into_iter().chain(ablations) {
        let m = mean_score(&c, &s, s.test_seeds.clone());
        table.push((c, m));
    }
    println!("\n{:<22} {:>6} {:>5} {:>8} {:>9} {:>7}", "arm (test seeds)", "lambda", "gate", "acc_resp", "acc_early", "persev");
    for (c, m) in &table {
        println!("{:<22} {:>6} {:>5} {:>8.3} {:>9.3} {:>7.3}", c.name, c.lambda, c.gate, m.acc_resp, m.acc_early, m.persev);
    }
    let get = |name: &str| table.iter().find(|(c, _)| c.name == name).map(|(_, m)| *m).unwrap();
    let (ch, sa, sb) = (get("classical-champion"), get("candidate-A"), get("candidate-B"));
    let mut v: Vec<(&str, bool, String)> = Vec::new();
    v.push(("R1 classical ties or wins", ch.acc_resp >= sa.acc_resp && ch.acc_resp >= sb.acc_resp, format!("champion {:.3} vs A {:.3}, B {:.3}", ch.acc_resp, sa.acc_resp, sb.acc_resp)));
    let g = get("A -gate");
    v.push(("R2 the gate suppresses noise", champion.gate && sa.acc_resp - g.acc_resp >= 0.03, format!("champion gate {}; A {:.3} vs -gate {:.3}", champion.gate, sa.acc_resp, g.acc_resp)));
    let nm = get("A -mixer");
    v.push(("R3 mixing transports evidence away", nm.acc_resp - sa.acc_resp >= 0.02, format!("-mixer {:.3} vs A {:.3}", nm.acc_resp, sa.acc_resp)));
    let nf = get("A -frozen");
    v.push(("R4 frozen chaos localizes", sa.acc_resp - nf.acc_resp >= 0.05, format!("A {:.3} vs -frozen {:.3}", sa.acc_resp, nf.acc_resp)));
    let nt = get("A -chaotic-temp");
    v.push(("R5 chaotic temperature blurs", nt.acc_resp >= sa.acc_resp, format!("-chaotic-temp {:.3} vs A {:.3}", nt.acc_resp, sa.acc_resp)));
    let nc = get("A -chemistry");
    v.push(("R6 catalysis is rich-get-richer", nc.persev <= sa.persev, format!("persev -chemistry {:.3} vs A {:.3}", nc.persev, sa.persev)));
    let ni = get("A -inertia");
    v.push(("R7 inertia holds the old state", ni.persev <= sa.persev, format!("persev -inertia {:.3} vs A {:.3}", ni.persev, sa.persev)));
    let wm = get("A warm (T0=1)");
    v.push(("R8 cold commits", sa.acc_resp - wm.acc_resp >= 0.05, format!("A {:.3} vs warm {:.3}", sa.acc_resp, wm.acc_resp)));
    let bo = get("base-only");
    v.push(("R9 base alone gives no verdict", (0.03..=0.10).contains(&bo.acc_resp), format!("base-only {:.3}", bo.acc_resp)));
    let early_ok = sweep_gate_on.windows(2).all(|p| p[1].acc_early <= p[0].acc_early + 0.02);
    let persev_ok = sweep_gate_on.windows(2).all(|p| p[1].persev <= p[0].persev + 0.02);
    v.push(("R10 the hold/forget trade-off exists", early_ok && persev_ok, format!("acc_early non-increasing: {early_ok}; persev non-increasing: {persev_ok}")));
    // Diagnostics, added after the run and NOT precommitted: is inertia interfering with the
    // per-step phase disorder, and does frozen chaos help once inertia is gone?
    println!("\ndiagnostics (after the run, not predictions):");
    for (name, f) in [
        ("A -inertia -frozen", (|c: &mut Cfg| { c.inertia = false; c.frozen = false }) as fn(&mut Cfg)),
        ("A -mixer -inertia", |c: &mut Cfg| { c.mixer = false; c.frozen = false; c.inertia = false }),
        ("A -mixer -inertia -chem", |c: &mut Cfg| { c.mixer = false; c.frozen = false; c.inertia = false; c.chem = false }),
        ("A -mixer (inertia kept)", |c: &mut Cfg| { c.mixer = false; c.frozen = false }),
        ("A -mixer, frozen kept", |c: &mut Cfg| { c.mixer = false }),
    ] {
        let mut c = a.clone();
        c.name = name.into();
        f(&mut c);
        let m = mean_score(&c, &s, s.test_seeds.clone());
        println!("  {:<26} {:>8.3} {:>9.3} {:>7.3}", name, m.acc_resp, m.acc_early, m.persev);
    }
    println!("\norder: |A - B| acc_resp = {:.3} (reported, no prediction)", (sa.acc_resp - sb.acc_resp).abs());
    println!();
    for (name, ok, d) in &v {
        println!("{} {name}: {d}", if *ok { "HOLDS " } else { "FAILS " });
    }
    let fails = v.iter().filter(|x| !x.1).count();
    println!("\n{} of {} predictions hold; kill criterion (>= 4 fail): {}", v.len() - fails, v.len(), if fails >= 4 { "FIRES" } else { "does not fire" });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_jump_and_seed_sets_do_not_overlap() {
        let s = Settings::qb3();
        for seed in 0..20 {
            let w = world(&s.spec(), seed);
            assert_eq!(w.targets.len(), s.trials);
            assert!(w.targets.windows(2).all(|p| qb_task::ring_distance(p[0], p[1], s.n) >= s.min_jump));
            assert_eq!(w.obs.len(), s.trials * s.spec().trial_len());
        }
        assert!(s.dev_seeds.end <= s.test_seeds.start);
    }

    #[test]
    fn the_cue_is_strong_and_the_delay_is_pure_noise() {
        let s = Settings::qb3();
        let w = world(&s.spec(), 7);
        let (mut cue, mut delay) = (0, 0);
        for (k, &c) in w.targets.iter().enumerate() {
            let base = k * s.spec().trial_len();
            cue += (0..s.cue).filter(|&i| w.obs[base + i] == c).count();
            delay += (s.cue..s.cue + s.delay).filter(|&i| w.obs[base + i] == c).count();
        }
        let cue_rate = cue as f64 / (s.trials * s.cue) as f64;
        let delay_rate = delay as f64 / (s.trials * s.delay) as f64;
        assert!(cue_rate > 0.8 && delay_rate < 0.12, "{cue_rate} {delay_rate}");
    }
}
