//! QB-5 (docs/QB-5-CHAOS-AND-TEMPERATURE-PER-MODULE.md): chaos and hot/cold, one module at a time,
//! on QB-4's modular working memory (D detector → M collapse memory → T cold Born tracker).
//!
//! Each round: the module's own check selects the best variant on development seeds (plain wins ties);
//! the test seeds report on against off; the winner is kept only if it beats off on the module's
//! check on the test seeds. Every setting lives in `Settings` and `Cfg`.

use qb_amplitude::{State, C};
use qb_chaos::keyed_unit;
use qb_classical::{draw, temper};
use qb_task::{qb3_spec, score, world, Spec, World};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
struct Settings {
    spec: Spec,
    eta: f64,
    detector_k: usize,
    detector_m: usize,
    dev_seeds: std::ops::Range<u64>,
    test_seeds: std::ops::Range<u64>,
}

impl Settings {
    fn qb5() -> Settings {
        Settings { spec: qb3_spec(), eta: 0.15, detector_k: 3, detector_m: 4, dev_seeds: 0..16, test_seeds: 100..132 }
    }
}

/// One configuration of the three modules.
#[derive(Debug, Clone, PartialEq)]
struct Cfg {
    lambda: f64,
    eta_m: f64,
    t0: f64,
    d_alpha: f64,
    d_key: u64,
    m_phase: f64,
    m_occ: f64,
    governor: Option<(f64, f64)>,
    t_alpha: f64,
    t_key: u64,
}

impl Cfg {
    /// QB-4's `modular-quantum`, as tuned there.
    fn qb4() -> Cfg {
        Cfg { lambda: 0.05, eta_m: 0.05, t0: 0.25, d_alpha: 0.0, d_key: 0, m_phase: 0.0, m_occ: 0.0, governor: None, t_alpha: 0.0, t_key: 0 }
    }
}

/// A chaotic value for (seed, step, element, tag), centred: x − ½ with x from the logistic map.
fn chaos(seed: u64, t: usize, element: usize, tag: u64) -> f64 {
    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (element as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ tag.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    keyed_unit(key) - 0.5
}

/// What one run produced: the choices, D's events, and M's held item each step.
struct Trace {
    choices: Vec<usize>,
    events: Vec<Option<usize>>,
    held: Vec<Option<usize>>,
}

fn run(cfg: &Cfg, s: &Settings, w: &World, seed: u64) -> Trace {
    let n = s.spec.n;
    let mut memory = State::uniform(n);
    let mut tracker = State::uniform(n);
    let mut last_event: Option<usize> = None;
    let mut tr = Trace { choices: Vec::new(), events: Vec::new(), held: Vec::new() };
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        // D: the persistence gate, with chaos on the vote weights before its threshold.
        let lo = t.saturating_sub(s.detector_m - 1);
        let votes: f64 = (lo..=t).filter(|&i| w.obs[i] == o).map(|i| 1.0 + cfg.d_alpha * chaos(seed, t, i - lo, 100 + cfg.d_key)).sum();
        let event = (votes >= s.detector_k as f64 - 0.5).then_some(o);
        // M: written by destructive projection; then its own chaos and heat.
        if let Some(e) = event {
            memory.project(e);
            last_event = Some(t);
        }
        if cfg.m_phase > 0.0 {
            for j in 0..n {
                memory.rotate(j, cfg.m_phase * 2.0 * PI * chaos(seed, t, j, 200));
            }
        }
        if cfg.m_occ > 0.0 {
            for j in 0..n {
                memory.excite(j, C::polar(cfg.m_occ, 2.0 * PI * (chaos(seed, t, j, 300) + 0.5)));
            }
            memory.normalize();
        }
        let pm = memory.born_weights();
        let top = (0..n).max_by(|&a, &b| pm[a].partial_cmp(&pm[b]).unwrap()).unwrap();
        tr.held.push((pm[top] > 1.0 / n as f64 + 1e-9).then_some(top));
        // T: evidence, the memory's reading, amplitude chaos, decay, readout at T_t.
        tracker.scale(o, 1.0 + s.eta);
        for (j, p) in pm.iter().enumerate() {
            tracker.scale(j, 1.0 + cfg.eta_m * p);
        }
        if cfg.t_alpha > 0.0 {
            for j in 0..n {
                tracker.scale(j, 1.0 + cfg.t_alpha * chaos(seed, t, j, 400 + cfg.t_key));
            }
        }
        tracker.power_magnitudes(1.0 - cfg.lambda);
        tracker.normalize();
        let temp = match (cfg.governor, last_event) {
            (Some((hot, tau)), Some(te)) => cfg.t0 + (hot - cfg.t0) * (-((t - te) as f64) / tau).exp(),
            _ => cfg.t0,
        };
        tr.choices.push(draw(&temper(&tracker.born_weights(), temp), w.uniforms[t]));
        tr.events.push(event);
    }
    tr
}

/// The three module checks for one run.
#[derive(Debug, Clone, Copy, Default)]
struct Checks {
    d_f1: f64,
    m_fidelity: f64,
    acc_resp: f64,
}

fn checks(s: &Settings, w: &World, tr: &Trace) -> Checks {
    let len = s.spec.trial_len();
    let target_at = |t: usize| w.targets[t / len];
    let fired: Vec<usize> = (0..tr.events.len()).filter(|&t| tr.events[t].is_some()).collect();
    let precision = if fired.is_empty() { 0.0 } else { fired.iter().filter(|&&t| tr.events[t] == Some(target_at(t))).count() as f64 / fired.len() as f64 };
    let recall = w.targets.iter().enumerate().filter(|(k, &c)| (0..s.spec.cue).any(|i| tr.events[k * len + i] == Some(c))).count() as f64 / w.targets.len() as f64;
    let d_f1 = if precision + recall > 0.0 { 2.0 * precision * recall / (precision + recall) } else { 0.0 };
    let (mut held, mut steps) = (0, 0);
    for (k, &c) in w.targets.iter().enumerate() {
        for step in s.spec.cue..len {
            steps += 1;
            held += usize::from(tr.held[k * len + step] == Some(c));
        }
    }
    Checks { d_f1, m_fidelity: held as f64 / steps as f64, acc_resp: score(&s.spec, w, &tr.choices).acc_resp }
}

fn mean(cfg: &Cfg, s: &Settings, seeds: std::ops::Range<u64>) -> Checks {
    let k = (seeds.end - seeds.start) as f64;
    let mut m = Checks::default();
    for seed in seeds {
        let w = world(&s.spec, seed);
        let c = checks(s, &w, &run(cfg, s, &w, seed));
        m.d_f1 += c.d_f1 / k;
        m.m_fidelity += c.m_fidelity / k;
        m.acc_resp += c.acc_resp / k;
    }
    m
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Module {
    D,
    M,
    T,
}

fn check_of(m: Module, c: &Checks) -> f64 {
    match m {
        Module::D => c.d_f1,
        Module::M => c.m_fidelity,
        Module::T => c.acc_resp,
    }
}

/// One round: select on development seeds (plain wins ties), report on the test seeds, keep or not.
struct Round {
    name: &'static str,
    module: Module,
    base: Cfg,
    best: Cfg,
    dev_off: f64,
    dev_on: f64,
    test_off: Checks,
    test_on: Checks,
    kept: bool,
    dev_variants: Vec<(String, f64)>,
}

fn round(name: &'static str, module: Module, base: &Cfg, variants: Vec<(String, Cfg)>, s: &Settings) -> Round {
    let dev_off = check_of(module, &mean(base, s, s.dev_seeds.clone()));
    let mut best = (dev_off, base.clone());
    let mut dev_variants = Vec::new();
    for (label, v) in variants {
        let d = check_of(module, &mean(&v, s, s.dev_seeds.clone()));
        dev_variants.push((label, d));
        if d > best.0 {
            best = (d, v);
        }
    }
    let test_off = mean(base, s, s.test_seeds.clone());
    let test_on = mean(&best.1, s, s.test_seeds.clone());
    let kept = best.1 != *base && check_of(module, &test_on) > check_of(module, &test_off);
    Round { name, module, base: base.clone(), best: best.1, dev_off, dev_on: best.0, test_off, test_on, kept, dev_variants }
}

fn main() {
    let s = Settings::qb5();
    println!("QB-5: chaos and hot/cold per module on QB-4's modular working memory; dev {:?}, test {:?}", s.dev_seeds, s.test_seeds);
    let mut base = Cfg::qb4();
    let base0 = mean(&base, &s, s.test_seeds.clone());
    println!("base (QB-4 modular-quantum) on test: D F1 {:.3}, M fidelity {:.3}, acc_resp {:.3}", base0.d_f1, base0.m_fidelity, base0.acc_resp);

    let mut rounds = Vec::new();
    let mut d_vars = Vec::new();
    for a in [0.1, 0.2, 0.4] {
        for k in 0..3 {
            d_vars.push((format!("D alpha {a} key {k}"), Cfg { d_alpha: a, d_key: k, ..base.clone() }));
        }
    }
    let r = round("D: chaos before the gate", Module::D, &base, d_vars, &s);
    if r.kept {
        base = r.best.clone();
    }
    rounds.push(r);

    let mut m_vars = Vec::new();
    for a in [0.1, 0.5, 1.0] {
        m_vars.push((format!("M phase chaos {a}"), Cfg { m_phase: a, ..base.clone() }));
    }
    for e in [0.005, 0.01, 0.02] {
        m_vars.push((format!("M occupation heat {e}"), Cfg { m_occ: e, ..base.clone() }));
    }
    let r = round("M: phase chaos and heat", Module::M, &base, m_vars, &s);
    if r.kept {
        base = r.best.clone();
    }
    rounds.push(r);

    let t_vars = [0.1, 0.15, 0.2, 0.3, 0.4].iter().map(|&t| (format!("T0 {t}"), Cfg { t0: t, ..base.clone() })).collect();
    let r = round("T1: cold and hot (the knob)", Module::T, &base, t_vars, &s);
    if r.kept {
        base = r.best.clone();
    }
    let t1_base = base.clone();
    rounds.push(r);

    let mut g_vars = Vec::new();
    for hot in [0.5, 1.0] {
        for tau in [3.0, 10.0] {
            g_vars.push((format!("governor hot {hot} tau {tau}"), Cfg { governor: Some((hot, tau)), ..base.clone() }));
        }
    }
    let r = round("T2: governor (reheat on event, cool)", Module::T, &base, g_vars, &s);
    if r.kept {
        base = r.best.clone();
    }
    rounds.push(r);

    let mut a_vars = Vec::new();
    for a in [0.05, 0.1, 0.2] {
        for k in 0..3 {
            a_vars.push((format!("T amplitude chaos {a} key {k}"), Cfg { t_alpha: a, t_key: k, ..base.clone() }));
        }
    }
    let r = round("T3: amplitude chaos", Module::T, &base, a_vars, &s);
    if r.kept {
        base = r.best.clone();
    }
    rounds.push(r);

    for r in &rounds {
        println!("\n== {} (check: {:?})", r.name, r.module);
        for (label, d) in &r.dev_variants {
            println!("   dev {label:<30} {d:.4}");
        }
        println!("   dev: off {:.4}, best {:.4} ({})", r.dev_off, r.dev_on, if r.best == r.base { "plain wins".to_owned() } else { format!("{:?}", diff(&r.base, &r.best)) });
        println!(
            "   test: off check {:.4} / acc_resp {:.3}; on check {:.4} / acc_resp {:.3} -> {}",
            check_of(r.module, &r.test_off),
            r.test_off.acc_resp,
            check_of(r.module, &r.test_on),
            r.test_on.acc_resp,
            if r.kept { "KEPT" } else { "not kept" }
        );
    }
    let fin = mean(&base, &s, s.test_seeds.clone());
    println!("\nfinal kept system on test: D F1 {:.3}, M fidelity {:.3}, acc_resp {:.3} (QB-4 base {:.3})", fin.d_f1, fin.m_fidelity, fin.acc_resp, base0.acc_resp);

    let [rd, rm, rt1, rt2, rt3] = [&rounds[0], &rounds[1], &rounds[2], &rounds[3], &rounds[4]];
    let phase_ties = rm.dev_variants.iter().filter(|(l, _)| l.contains("phase")).all(|(_, d)| (*d - rm.dev_off).abs() < 1e-12);
    let occ_lower = rm.dev_variants.iter().filter(|(l, _)| l.contains("occupation")).all(|(_, d)| *d < rm.dev_off);
    let v = [
        ("P1 D chaos before the gate is neutral (not kept)", !rd.kept, format!("kept: {}", rd.kept)),
        ("P2 M immune to phase chaos, harmed by heat", phase_ties && occ_lower && !rm.kept, format!("phase ties exactly: {phase_ties}; heat lowers fidelity: {occ_lower}; kept: {}", rm.kept)),
        ("P3 T colder helps up to a point", rt1.best.t0 <= 0.25 && rt1.test_on.acc_resp >= rt1.test_off.acc_resp, format!("best T0 {}; test {:.3} -> {:.3}", rt1.best.t0, rt1.test_off.acc_resp, rt1.test_on.acc_resp)),
        ("P4 T governor adds nothing (<= +0.01)", rt2.test_on.acc_resp - mean(&t1_base, &s, s.test_seeds.clone()).acc_resp <= 0.01, format!("governor test {:.3} vs fixed {:.3}", rt2.test_on.acc_resp, rt2.test_off.acc_resp)),
        ("P5 T amplitude chaos not kept", !rt3.kept, format!("kept: {}", rt3.kept)),
    ];
    println!();
    for (name, ok, d) in &v {
        println!("{} {name}: {d}", if *ok { "HOLDS " } else { "FAILS " });
    }
    let fails = v.iter().filter(|x| !x.1).count();
    println!("\n{} of {} predictions hold; kill criterion (>= 3 fail): {}", v.len() - fails, v.len(), if fails >= 3 { "FIRES" } else { "does not fire" });
}

/// The fields that differ between two configurations (for the report).
fn diff(a: &Cfg, b: &Cfg) -> Vec<String> {
    let mut out = Vec::new();
    if a.t0 != b.t0 {
        out.push(format!("t0 {}", b.t0));
    }
    if a.d_alpha != b.d_alpha {
        out.push(format!("d_alpha {} key {}", b.d_alpha, b.d_key));
    }
    if a.m_phase != b.m_phase {
        out.push(format!("m_phase {}", b.m_phase));
    }
    if a.m_occ != b.m_occ {
        out.push(format!("m_occ {}", b.m_occ));
    }
    if a.governor != b.governor {
        out.push(format!("governor {:?}", b.governor));
    }
    if a.t_alpha != b.t_alpha {
        out.push(format!("t_alpha {} key {}", b.t_alpha, b.t_key));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plain_configuration_reproduces_qb4() {
        let s = Settings::qb5();
        let m = mean(&Cfg::qb4(), &s, s.test_seeds.clone());
        assert!((m.acc_resp - 0.877).abs() < 0.0005, "{}", m.acc_resp);
    }

    #[test]
    fn checks_recompute_independently() {
        let s = Settings::qb5();
        let w = world(&s.spec, 3);
        let tr = run(&Cfg::qb4(), &s, &w, 3);
        let c = checks(&s, &w, &tr);
        // Recompute M fidelity by walking trials and steps directly.
        let len = s.spec.trial_len();
        let mut held = 0;
        let mut steps = 0;
        for t in 0..tr.held.len() {
            if t % len >= s.spec.cue {
                steps += 1;
                held += usize::from(tr.held[t] == Some(w.targets[t / len]));
            }
        }
        assert!((c.m_fidelity - held as f64 / steps as f64).abs() < 1e-12);
        // Recompute precision from the event list.
        let ev: Vec<(usize, usize)> = tr.events.iter().enumerate().filter_map(|(t, e)| e.map(|e| (t, e))).collect();
        let p = ev.iter().filter(|(t, e)| *e == w.targets[t / len]).count() as f64 / ev.len() as f64;
        let r = (0..s.spec.trials).filter(|&k| ev.iter().any(|(t, e)| t / len == k && t % len < s.spec.cue && *e == w.targets[k])).count() as f64 / s.spec.trials as f64;
        assert!((c.d_f1 - 2.0 * p * r / (p + r)).abs() < 1e-12);
    }

    #[test]
    fn the_weighted_gate_at_zero_alpha_is_the_plain_gate() {
        let s = Settings::qb5();
        let w = world(&s.spec, 9);
        let tr = run(&Cfg::qb4(), &s, &w, 9);
        for t in 0..w.obs.len() {
            let plain = qb_classical::gate(&w.obs[t.saturating_sub(s.detector_m - 1)..=t], w.obs[t], s.detector_k);
            assert_eq!(tr.events[t].is_some(), plain);
        }
    }
}
