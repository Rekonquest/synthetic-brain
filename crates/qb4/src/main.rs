//! QB-4 (docs/QB-4-MODULAR-WORKING-MEMORY.md): small stacks orchestrated, on QB-3's task.
//!
//! Three modules, one job each: D (detector: a persistence gate), M (memory: a quantum state written
//! by destructive projection), T (tracker: cold Born). The orchestrator only routes readings:
//! observation → D and T; D's event → M (write); M's Born weights → T (as concentrations). Every
//! setting lives in `Settings`; λ and η_m are tuned on development seeds only.

use qb_amplitude::State;
use qb_classical::{argmax, conserve, decay, draw, gate, reinforce, temper};
use qb_task::{qb3_spec, score, world, Score, Spec, World};

#[derive(Debug, Clone)]
struct Settings {
    spec: Spec,
    eta: f64,
    t0: f64,
    detector_k: usize,
    detector_m: usize,
    lambdas: [f64; 7],
    eta_ms: [f64; 5],
    dev_seeds: std::ops::Range<u64>,
    test_seeds: std::ops::Range<u64>,
    champion_gate_k: usize,
    champion_gate_m: usize,
    champion_lambda: f64,
}

impl Settings {
    fn qb4() -> Settings {
        Settings {
            spec: qb3_spec(),
            eta: 0.15,
            t0: 0.25,
            detector_k: 3,
            detector_m: 4,
            lambdas: [0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.4],
            eta_ms: [0.05, 0.1, 0.2, 0.4, 0.8],
            dev_seeds: 0..16,
            test_seeds: 100..132,
            champion_gate_k: 2,
            champion_gate_m: 3,
            champion_lambda: 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    /// D → M (collapse) → T (cold Born).
    ModularQuantum,
    /// D → register → greedy tracker.
    ModularClassical,
    /// The choice is M's held item.
    MemoryOnly,
    /// T alone.
    TrackerOnly,
    /// QB-3's monolithic greedy champion (gate 2 of 3, λ = 0.05).
    Champion,
}

#[derive(Debug, Clone)]
struct Cfg {
    name: &'static str,
    kind: Kind,
    lambda: f64,
    eta_m: f64,
    /// M is written by D's events (true) or by every observation (false).
    detector: bool,
}

/// D: does the observation at t fire an event?
fn detect(w: &World, t: usize, k: usize, m: usize) -> Option<usize> {
    let o = w.obs[t];
    gate(&w.obs[t.saturating_sub(m - 1)..=t], o, k).then_some(o)
}

fn run(cfg: &Cfg, s: &Settings, w: &World) -> Vec<usize> {
    let n = s.spec.n;
    let mut memory = State::uniform(n);
    let mut tracker = State::uniform(n);
    let mut register = vec![1.0 / n as f64; n];
    let mut weights = vec![1.0 / n as f64; n];
    let mut choices = Vec::with_capacity(w.obs.len());
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        let event = if cfg.detector { detect(w, t, s.detector_k, s.detector_m) } else { Some(o) };
        match cfg.kind {
            Kind::ModularQuantum | Kind::MemoryOnly | Kind::TrackerOnly => {
                if let Some(e) = event {
                    memory.project(e);
                }
                let pm = memory.born_weights();
                tracker.scale(o, 1.0 + s.eta);
                if cfg.kind == Kind::ModularQuantum {
                    for (j, p) in pm.iter().enumerate() {
                        tracker.scale(j, 1.0 + cfg.eta_m * p);
                    }
                }
                tracker.power_magnitudes(1.0 - cfg.lambda);
                tracker.normalize();
                let read = if cfg.kind == Kind::MemoryOnly { pm } else { tracker.born_weights() };
                choices.push(draw(&temper(&read, s.t0), w.uniforms[t]));
            }
            Kind::ModularClassical => {
                if let Some(e) = event {
                    register = vec![0.0; n];
                    register[e] = 1.0;
                }
                reinforce(&mut weights, o, s.eta);
                for (j, r) in register.iter().enumerate() {
                    weights[j] *= 1.0 + cfg.eta_m * r;
                }
                decay(&mut weights, cfg.lambda);
                conserve(&mut weights);
                choices.push(argmax(&weights));
            }
            Kind::Champion => {
                if gate(&w.obs[t.saturating_sub(s.champion_gate_m - 1)..=t], o, s.champion_gate_k) {
                    reinforce(&mut weights, o, s.eta);
                }
                decay(&mut weights, cfg.lambda);
                conserve(&mut weights);
                choices.push(argmax(&weights));
            }
        }
    }
    choices
}

fn mean(cfg: &Cfg, s: &Settings, seeds: std::ops::Range<u64>) -> Score {
    let k = (seeds.end - seeds.start) as f64;
    let mut m = Score::default();
    for seed in seeds {
        let w = world(&s.spec, seed);
        let sc = score(&s.spec, &w, &run(cfg, s, &w));
        m.acc_resp += sc.acc_resp / k;
        m.acc_early += sc.acc_early / k;
        m.persev += sc.persev / k;
    }
    m
}

/// Tune λ (and η_m where the arm uses it) on the development seeds by acc_resp.
fn tune(cfg: &Cfg, s: &Settings) -> Cfg {
    let uses_eta_m = matches!(cfg.kind, Kind::ModularQuantum | Kind::ModularClassical) && cfg.eta_m > 0.0;
    let mut best = (f64::MIN, cfg.clone());
    for &l in &s.lambdas {
        for &e in if uses_eta_m { &s.eta_ms[..] } else { &[0.0][..] } {
            let c = Cfg { lambda: l, eta_m: if uses_eta_m { e } else { cfg.eta_m }, ..cfg.clone() };
            let a = mean(&c, s, s.dev_seeds.clone()).acc_resp;
            if a > best.0 {
                best = (a, c);
            }
        }
    }
    best.1
}

fn main() {
    let s = Settings::qb4();
    println!("QB-4: QB-3's task (identical worlds via qb-task); dev seeds {:?}, test seeds {:?}", s.dev_seeds, s.test_seeds);
    let mq = tune(&Cfg { name: "modular-quantum", kind: Kind::ModularQuantum, lambda: 0.0, eta_m: 1.0, detector: true }, &s);
    let mc = tune(&Cfg { name: "modular-classical", kind: Kind::ModularClassical, lambda: 0.0, eta_m: 1.0, detector: true }, &s);
    let cold = tune(&Cfg { name: "simple-cold-born", kind: Kind::TrackerOnly, lambda: 0.0, eta_m: 0.0, detector: true }, &s);
    let champion = Cfg { name: "classical-champion", kind: Kind::Champion, lambda: s.champion_lambda, eta_m: 0.0, detector: false };
    let no_memory = Cfg { name: "-memory", eta_m: 0.0, ..mq.clone() };
    let memory_only = Cfg { name: "memory-only", kind: Kind::MemoryOnly, ..mq.clone() };
    let no_detector = Cfg { name: "-detector", detector: false, ..mq.clone() };
    let arms = [mq.clone(), mc.clone(), champion, cold, no_memory, memory_only, no_detector];
    let results: Vec<(Cfg, Score)> = arms.iter().map(|c| (c.clone(), mean(c, &s, s.test_seeds.clone()))).collect();
    println!("\n{:<20} {:>7} {:>6} {:>8} {:>9} {:>7}", "arm (test seeds)", "lambda", "eta_m", "acc_resp", "acc_early", "persev");
    for (c, m) in &results {
        println!("{:<20} {:>7} {:>6} {:>8.3} {:>9.3} {:>7.3}", c.name, c.lambda, c.eta_m, m.acc_resp, m.acc_early, m.persev);
    }
    // QB-3's candidate A on the same worlds and test seeds (evidence/qb-3/results.txt).
    let monolith_a = 0.112;
    println!("{:<20} {:>7} {:>6} {:>8.3}   (QB-3's run, identical worlds)", "monolith-A", 0.05, "-", monolith_a);
    let get = |name: &str| results.iter().find(|(c, _)| c.name == name).map(|(_, m)| *m).unwrap();
    let (q, c, ch, cb, nm, mo, nd) = (get("modular-quantum"), get("modular-classical"), get("classical-champion"), get("simple-cold-born"), get("-memory"), get("memory-only"), get("-detector"));
    let v = [
        ("S1 small stacks orchestrated beat one monolith", q.acc_resp >= monolith_a + 0.30 && q.acc_resp >= cb.acc_resp, format!("modular-quantum {:.3} vs monolith {monolith_a:.3}, simple-cold-born {:.3}", q.acc_resp, cb.acc_resp)),
        ("S2 collapse is a near-perfect holder", q.acc_early >= 0.75, format!("acc_early {:.3} (champion {:.3})", q.acc_early, ch.acc_early)),
        ("S3 modular breaks the hold/forget trade-off", q.acc_resp > ch.acc_resp && c.acc_resp > ch.acc_resp, format!("quantum {:.3}, classical {:.3} vs champion {:.3}", q.acc_resp, c.acc_resp, ch.acc_resp)),
        ("S4 classical ties or wins (law 5)", c.acc_resp >= q.acc_resp - 0.02, format!("modular-classical {:.3} vs modular-quantum {:.3}", c.acc_resp, q.acc_resp)),
        ("S5 the memory carries the delay", q.acc_early - nm.acc_early >= 0.15, format!("acc_early {:.3} vs -memory {:.3}", q.acc_early, nm.acc_early)),
        ("S6 the detector makes the memory hold", q.acc_early - nd.acc_early >= 0.15, format!("acc_early {:.3} vs -detector {:.3}", q.acc_early, nd.acc_early)),
        ("S7 one job each", mo.acc_early >= 0.70 && mo.acc_resp < q.acc_resp, format!("memory-only acc_early {:.3}, acc_resp {:.3} vs {:.3}", mo.acc_early, mo.acc_resp, q.acc_resp)),
    ];
    println!();
    for (name, ok, d) in &v {
        println!("{} {name}: {d}", if *ok { "HOLDS " } else { "FAILS " });
    }
    let fails = v.iter().filter(|x| !x.1).count();
    println!("\n{} of {} predictions hold; kill criterion (>= 3 fail): {}", v.len() - fails, v.len(), if fails >= 3 { "FIRES" } else { "does not fire" });
    println!("champion check: QB-3 measured 0.815 on these worlds; here {:.3}", ch.acc_resp);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(kind: Kind, eta_m: f64) -> Cfg {
        Cfg { name: "t", kind, lambda: 0.05, eta_m, detector: true }
    }

    #[test]
    fn an_empty_memory_has_no_effect_and_a_written_one_pulls() {
        let s = Settings::qb4();
        // A world with no strong repeats never fires D, so M stays uniform: the link must do nothing.
        let mut w = world(&s.spec, 101);
        for (t, o) in w.obs.iter_mut().enumerate() {
            *o = t % s.spec.n;
        }
        assert!((0..w.obs.len()).all(|t| detect(&w, t, s.detector_k, s.detector_m).is_none()));
        assert_eq!(run(&cfg(Kind::ModularQuantum, 0.8), &s, &w), run(&cfg(Kind::ModularQuantum, 0.0), &s, &w));
        // A written memory pulls the tracker toward the held item.
        let mut w2 = world(&s.spec, 101);
        for o in w2.obs.iter_mut().take(4) {
            *o = 7;
        }
        for (t, o) in w2.obs.iter_mut().enumerate().skip(4).take(60) {
            *o = t % s.spec.n;
        }
        let with = run(&cfg(Kind::ModularQuantum, 0.8), &s, &w2);
        let without = run(&cfg(Kind::ModularQuantum, 0.0), &s, &w2);
        let picks = |c: &[usize]| c[4..64].iter().filter(|&&x| x == 7).count();
        assert!(picks(&with) > picks(&without) + 10, "{} vs {}", picks(&with), picks(&without));
    }

    #[test]
    fn the_detector_rarely_fires_on_noise() {
        let s = Settings::qb4();
        let (mut fires, mut steps) = (0, 0);
        for seed in 0..8 {
            let w = world(&s.spec, seed);
            for (k, _) in w.targets.iter().enumerate() {
                for step in s.spec.cue + 3..s.spec.cue + s.spec.delay {
                    let t = k * s.spec.trial_len() + step;
                    steps += 1;
                    fires += usize::from(detect(&w, t, s.detector_k, s.detector_m).is_some());
                }
            }
        }
        assert!((fires as f64 / steps as f64) < 0.02, "{fires}/{steps}");
    }
}
