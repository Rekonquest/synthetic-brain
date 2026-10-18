//! QB-9 (docs/QB-9-RETEST-ROUND.md): the retest round on QB-8's stack — M judged by its log-score
//! (a proper scoring rule over its concentrations), a colder decision, amplification retested, and a
//! fully tunable classical comparator. Built from QB-8's runner (copied).
//!
//! Modules (QB-4's architecture): D detector → M memory → T tracker. The chemistry amplifiers:
//! cooperative binding (Hill) on D, an autocatalytic latch as a rival M, an enzyme cascade replacing
//! T's plain gain, and growth with product inhibition on T. Protocol: QB-5's (the module's own check
//! selects on dev seeds, plain wins ties, kept only if it beats off on test). Every setting lives in
//! `Settings` and `Cfg`.

use qb_amplitude::State;
use qb_chaos::keyed_unit;
use qb_classical::{argmax, conserve, decay, draw, gate, temper};
use qb_task::{qb7_spec, score, split_acc, world, Rng, Spec, World};

#[derive(Debug, Clone)]
struct Settings {
    spec: Spec,
    eta: f64,
    detector_k: usize,
    detector_m: usize,
    hill_k: f64,
    latch: (f64, f64, f64, f64, f64),
    cascade: (f64, f64),
    dev_seeds: std::ops::Range<u64>,
    test_seeds: std::ops::Range<u64>,
}

impl Settings {
    fn qb7() -> Settings {
        Settings {
            spec: qb7_spec(),
            eta: 0.15,
            detector_k: 3,
            detector_m: 4,
            hill_k: 2.5,
            // (K, δ, γ, dt, write pulse)
            latch: (0.5, 0.6, 1.0, 0.5, 2.0),
            // (activation rate, deactivation rate)
            cascade: (0.5, 0.2),
            dev_seeds: 0..16,
            test_seeds: 100..132,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Cfg {
    lambda: f64,
    eta_m: f64,
    t0: f64,
    t_alpha: f64,
    t_key: u64,
    t_gain: f64,
    kappa: f64,
    iota: f64,
    hill: Option<(f64, usize)>,
    latch: bool,
    cascade: bool,
    d_alpha: f64,
    d_key: u64,
    m_phase: f64,
    m_occ: f64,
    governor: Option<(f64, f64)>,
}

impl Cfg {
    /// QB-8's final kept system.
    fn qb8_kept() -> Cfg {
        Cfg { t0: 0.05, m_occ: 0.01, t_alpha: 0.0, ..Cfg::qb7_kept() }
    }

    /// QB-7's final kept system (QB-6's plus growth κ 1 with product inhibition ι 1).
    fn qb7_kept() -> Cfg {
        Cfg { lambda: 0.05, eta_m: 0.05, t0: 0.1, t_alpha: 0.05, t_key: 2, t_gain: 3.0, kappa: 1.0, iota: 1.0, hill: None, latch: false, cascade: false, d_alpha: 0.0, d_key: 0, m_phase: 0.0, m_occ: 0.0, governor: None }
    }
}

/// QB-5's centred chaos for (seed, step, element, tag).
fn chaos(seed: u64, t: usize, element: usize, tag: u64) -> f64 {
    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (element as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ tag.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    keyed_unit(key) - 0.5
}

/// The autocatalytic latch: per-option concentrations, bistable, with mutual inhibition.
#[derive(Debug, Clone)]
struct Latch {
    b: Vec<f64>,
}

impl Latch {
    fn new(n: usize) -> Latch {
        Latch { b: vec![0.0; n] }
    }
    fn step(&mut self, pulse: Option<usize>, p: (f64, f64, f64, f64, f64)) {
        let (k, delta, gamma, dt, amount) = p;
        if let Some(e) = pulse {
            self.b[e] += amount;
        }
        let total: f64 = self.b.iter().sum();
        let old = self.b.clone();
        for (j, x) in self.b.iter_mut().enumerate() {
            let b = old[j];
            let db = b * b / (k * k + b * b) - delta * b - gamma * b * (total - b);
            *x = (b + dt * db).max(0.0);
        }
    }
    fn reading(&self, n: usize) -> Vec<f64> {
        let t: f64 = self.b.iter().sum();
        if t > 0.0 { self.b.iter().map(|x| x / t).collect() } else { vec![1.0 / n as f64; n] }
    }
    fn held(&self, threshold: f64) -> Option<usize> {
        let top = argmax(&self.b);
        (self.b[top] > threshold).then_some(top)
    }
}

/// The latch's unstable threshold for K = 0.5, δ = 0.6 (the lower root of 0.6b² − b + 0.15).
#[allow(dead_code)]
const LATCH_THRESHOLD: f64 = 0.1667;

struct Trace {
    choices: Vec<usize>,
    events: Vec<Option<usize>>,
    held: Vec<Option<usize>>,
    /// M's reading (its Born weights, as T consumes them) at every step.
    pm: Vec<Vec<f64>>,
}

fn run(cfg: &Cfg, s: &Settings, w: &World, seed: u64) -> Trace {
    let n = s.spec.n;
    let mut memory = State::uniform(n);
    let mut latch = Latch::new(n);
    let mut tracker = State::uniform(n);
    let (mut x1, mut x2) = (vec![0.0; n], vec![0.0; n]);
    let mut hill_rng = Rng(seed ^ 0x4111_4111_0000_0009);
    let mut tr = Trace { choices: Vec::new(), events: Vec::new(), held: Vec::new(), pm: Vec::new() };
    let mut last_event: Option<usize> = None;
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        // D.
        let event = match cfg.hill {
            None if cfg.d_alpha > 0.0 => {
                let lo = t.saturating_sub(s.detector_m - 1);
                let votes: f64 = (lo..=t).filter(|&i| w.obs[i] == o).map(|i| 1.0 + cfg.d_alpha * chaos(seed, t, i - lo, 100 + cfg.d_key)).sum();
                (votes >= s.detector_k as f64 - 0.5).then_some(o)
            }
            None => gate(&w.obs[t.saturating_sub(s.detector_m - 1)..=t], o, s.detector_k).then_some(o),
            Some((nh, m)) => {
                let c = w.obs[t.saturating_sub(m - 1)..=t].iter().filter(|&&x| x == o).count() as f64;
                let a = c.powf(nh) / (s.hill_k.powf(nh) + c.powf(nh));
                (hill_rng.unit() < a).then_some(o)
            }
        };
        // M.
        let pm = if cfg.latch {
            latch.step(event, s.latch);
            tr.held.push(latch.held(LATCH_THRESHOLD));
            latch.reading(n)
        } else {
            if let Some(e) = event {
                memory.project(e);
            }
            if cfg.m_phase > 0.0 {
                for j in 0..n {
                    memory.rotate(j, cfg.m_phase * 2.0 * std::f64::consts::PI * chaos(seed, t, j, 200));
                }
            }
            if cfg.m_occ > 0.0 {
                for j in 0..n {
                    memory.excite(j, qb_amplitude::C::polar(cfg.m_occ, 2.0 * std::f64::consts::PI * (chaos(seed, t, j, 300) + 0.5)));
                }
                memory.normalize();
            }
            let pm = memory.born_weights();
            let top = argmax(&pm);
            tr.held.push((pm[top] > 1.0 / n as f64 + 1e-9).then_some(top));
            pm
        };
        tr.pm.push(pm.clone());
        // T.
        let eta = s.eta * cfg.t_gain;
        if cfg.cascade {
            let (ka, kd) = s.cascade;
            for j in 0..n {
                let input = if j == o { 1.0 } else { 0.0 };
                x1[j] = (x1[j] + ka * input * (1.0 - x1[j]) - kd * x1[j]).clamp(0.0, 1.0);
                x2[j] = (x2[j] + ka * x1[j] * (1.0 - x2[j]) - kd * x2[j]).clamp(0.0, 1.0);
                tracker.scale(j, 1.0 + eta * x2[j]);
            }
        } else {
            let p = tracker.born_weights()[o];
            let growth = if cfg.kappa > 0.0 { 1.0 + cfg.kappa * p / (p + 0.1) } else { 1.0 };
            let brake = if cfg.iota > 0.0 { (1.0 - p).powf(cfg.iota) } else { 1.0 };
            tracker.scale(o, 1.0 + eta * growth * brake);
        }
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
        if event.is_some() {
            last_event = Some(t);
        }
        let temp = match (cfg.governor, last_event) {
            (Some((hot, tau)), Some(te)) => cfg.t0 + (hot - cfg.t0) * (-((t - te) as f64) / tau).exp(),
            _ => cfg.t0,
        };
        tr.choices.push(draw(&temper(&tracker.born_weights(), temp), w.uniforms[t]));
        tr.events.push(event);
    }
    tr
}

#[derive(Debug, Clone, Copy, Default)]
struct Checks {
    d_f1: f64,
    m_fidelity: f64,
    m_log: f64,
    acc_resp: f64,
    acc_cued: f64,
    acc_missed: f64,
}

fn checks(s: &Settings, w: &World, tr: &Trace) -> Checks {
    let len = s.spec.trial_len();
    let target_at = |t: usize| w.targets[t / len];
    let fired: Vec<usize> = (0..tr.events.len()).filter(|&t| tr.events[t].is_some()).collect();
    let precision = if fired.is_empty() { 0.0 } else { fired.iter().filter(|&&t| tr.events[t] == Some(target_at(t))).count() as f64 / fired.len() as f64 };
    let horizon = s.spec.cue + s.spec.delay + s.spec.response / 2;
    let recall = w.targets.iter().enumerate().filter(|(k, &c)| (0..horizon).any(|i| tr.events[k * len + i] == Some(c))).count() as f64 / w.targets.len() as f64;
    let d_f1 = if precision + recall > 0.0 { 2.0 * precision * recall / (precision + recall) } else { 0.0 };
    let (mut held, mut steps) = (0, 0);
    for (k, &c) in w.targets.iter().enumerate() {
        for step in s.spec.cue..len {
            steps += 1;
            held += usize::from(tr.held[k * len + step] == Some(c));
        }
    }
    let (acc_cued, acc_missed) = split_acc(&s.spec, w, &tr.choices);
    let mut log = 0.0;
    for (k, &c) in w.targets.iter().enumerate() {
        for step in s.spec.cue..len {
            log += tr.pm[k * len + step][c].max(1e-9).ln();
        }
    }
    Checks { d_f1, m_log: log / steps as f64, m_fidelity: held as f64 / steps as f64, acc_resp: score(&s.spec, w, &tr.choices).acc_resp, acc_cued, acc_missed }
}

fn mean(cfg: &Cfg, s: &Settings, seeds: std::ops::Range<u64>) -> Checks {
    let k = (seeds.end - seeds.start) as f64;
    let mut m = Checks::default();
    for seed in seeds {
        let w = world(&s.spec, seed);
        let c = checks(s, &w, &run(cfg, s, &w, seed));
        m.d_f1 += c.d_f1 / k;
        m.m_fidelity += c.m_fidelity / k;
        m.m_log += c.m_log / k;
        m.acc_resp += c.acc_resp / k;
        m.acc_cued += c.acc_cued / k;
        m.acc_missed += c.acc_missed / k;
    }
    m
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Module {
    D,
    M,
    MLog,
    T,
}

fn check_of(m: Module, c: &Checks) -> f64 {
    match m {
        Module::D => c.d_f1,
        Module::M => c.m_fidelity,
        Module::MLog => c.m_log,
        Module::T => c.acc_resp,
    }
}

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

/// The classical modular comparator, every knob tunable: a gate detector, a register memory with a
/// classical thermal floor, and a greedy tracker with evidence gain and growth with a brake.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Classical {
    floor: f64,
    gain: f64,
    kappa: f64,
    iota: f64,
    lambda: f64,
    eta_m: f64,
}

fn classical(s: &Settings, w: &World, c: &Classical) -> Vec<usize> {
    let n = s.spec.n;
    let (mut register, mut weights) = (vec![1.0 / n as f64; n], vec![1.0 / n as f64; n]);
    let mut out = Vec::new();
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        if gate(&w.obs[t.saturating_sub(s.detector_m - 1)..=t], o, s.detector_k) {
            register = vec![0.0; n];
            register[o] = 1.0;
        }
        if c.floor > 0.0 {
            for r in register.iter_mut() {
                *r = (1.0 - c.floor) * *r + c.floor / n as f64;
            }
        }
        let p = weights[o];
        let growth = if c.kappa > 0.0 { 1.0 + c.kappa * p / (p + 0.1) } else { 1.0 };
        let brake = if c.iota > 0.0 { (1.0 - p).powf(c.iota) } else { 1.0 };
        weights[o] *= 1.0 + s.eta * c.gain * growth * brake;
        for (j, r) in register.iter().enumerate() {
            weights[j] *= 1.0 + c.eta_m * r;
        }
        decay(&mut weights, c.lambda);
        conserve(&mut weights);
        out.push(argmax(&weights));
    }
    out
}

fn classical_mean(s: &Settings, seeds: std::ops::Range<u64>, c: &Classical) -> (f64, f64, f64) {
    let k = (seeds.end - seeds.start) as f64;
    let (mut a, mut cu, mut m) = (0.0, 0.0, 0.0);
    for seed in seeds {
        let w = world(&s.spec, seed);
        let ch = classical(s, &w, c);
        a += score(&s.spec, &w, &ch).acc_resp / k;
        let (x, y) = split_acc(&s.spec, &w, &ch);
        cu += x / k;
        m += y / k;
    }
    (a, cu, m)
}

fn main() {
    let s = Settings::qb7();
    println!("QB-9: the retest round on QB-8's stack; p_miss {}; dev {:?}, test {:?}", s.spec.p_miss, s.dev_seeds, s.test_seeds);
    let mut base = Cfg::qb8_kept();
    let b0 = mean(&base, &s, s.test_seeds.clone());
    println!("base (QB-8 kept) on test: M log-score {:.3}, acc_resp {:.3} (cued {:.3}, missed {:.3})", b0.m_log, b0.acc_resp, b0.acc_cued, b0.acc_missed);
    let mut rounds = Vec::new();
    macro_rules! go {
        ($name:expr, $module:expr, $vars:expr) => {{
            let r = round($name, $module, &base, $vars, &s);
            if r.kept {
                base = r.best.clone();
            }
            rounds.push(r);
        }};
    }
    go!("1 M heat, judged by its log-score", Module::MLog, [0.0, 0.0025, 0.005, 0.02, 0.04].iter().map(|&e| (format!("M heat {e}"), Cfg { m_occ: e, ..base.clone() })).collect());
    let after_m = base.clone();
    go!("2 T colder", Module::T, [0.02, 0.03, 0.04, 0.07].iter().map(|&t| (format!("T0 {t}"), Cfg { t0: t, ..base.clone() })).collect());
    let before_amp = base.clone();
    go!("3 T gain", Module::T, [2.0, 4.0].iter().map(|&g| (format!("T gain {g}"), Cfg { t_gain: g, ..base.clone() })).collect());
    go!("4 T growth and brake", Module::T, [(0.5, 1.0), (1.5, 1.0), (1.0, 0.5), (1.0, 2.0)].iter().map(|&(k, i)| (format!("kappa {k} iota {i}"), Cfg { kappa: k, iota: i, ..base.clone() })).collect());
    go!("5 M->T link gain", Module::T, [0.025, 0.1].iter().map(|&e| (format!("eta_m {e}"), Cfg { eta_m: e, ..base.clone() })).collect());
    for r in &rounds {
        println!("\n== {} (check: {:?})", r.name, r.module);
        for (label, d) in &r.dev_variants {
            println!("   dev {label:<22} {d:.4}");
        }
        println!("   dev: off {:.4}, best {:.4} ({})", r.dev_off, r.dev_on, if r.best == r.base { "plain wins".to_owned() } else { "variant".to_owned() });
        println!(
            "   test: off check {:.4} / acc {:.3} (cued {:.3}, missed {:.3}); on check {:.4} / acc {:.3} (cued {:.3}, missed {:.3}) -> {}",
            check_of(r.module, &r.test_off), r.test_off.acc_resp, r.test_off.acc_cued, r.test_off.acc_missed,
            check_of(r.module, &r.test_on), r.test_on.acc_resp, r.test_on.acc_cued, r.test_on.acc_missed,
            if r.kept { "KEPT" } else { "not kept" }
        );
    }
    let fin = mean(&base, &s, s.test_seeds.clone());
    println!("\nfinal kept quantum system on test: acc_resp {:.3} (cued {:.3}, missed {:.3}); QB-8 kept {:.3}", fin.acc_resp, fin.acc_cued, fin.acc_missed, b0.acc_resp);
    let no_heat = mean(&Cfg { m_occ: 0.0, ..base.clone() }, &s, s.test_seeds.clone());
    println!("same system with M heat 0: acc_resp {:.3} (cued {:.3}, missed {:.3})", no_heat.acc_resp, no_heat.acc_cued, no_heat.acc_missed);
    // The fair classical comparator, tuned on dev over the full grid.
    let mut best: Option<(f64, Classical)> = None;
    let mut grid = 0;
    for floor in [0.0, 0.01, 0.02, 0.05] {
        for gain in [2.0, 3.0, 4.0] {
            for kappa in [0.0, 1.0] {
                for iota in [0.0, 1.0] {
                    for lambda in [0.02, 0.05, 0.1, 0.2] {
                        for eta_m in [0.025, 0.05, 0.1] {
                            grid += 1;
                            let c = Classical { floor, gain, kappa, iota, lambda, eta_m };
                            let a = classical_mean(&s, s.dev_seeds.clone(), &c).0;
                            if best.as_ref().is_none_or(|(b, _)| a > *b) {
                                best = Some((a, c));
                            }
                        }
                    }
                }
            }
        }
    }
    let (_, cbest) = best.unwrap();
    let (ca, cc, cm) = classical_mean(&s, s.test_seeds.clone(), &cbest);
    println!("fair classical comparator ({grid} configs tuned on dev; best {cbest:?}): test acc_resp {ca:.3} (cued {cc:.3}, missed {cm:.3})");
    let r = |i: usize| &rounds[i];
    let colder_delta = r(1).test_on.acc_resp - r(1).test_off.acc_resp;
    let amp_delta = fin.acc_resp - mean(&before_amp, &s, s.test_seeds.clone()).acc_resp;
    let _ = after_m;
    let v = [
        ("Q1 the fixed check sees heat", r(0).best.m_occ > 0.0 && fin.acc_missed - no_heat.acc_missed >= 0.02, format!("best eps {}; missed {:.3} vs no-heat {:.3}", r(0).best.m_occ, fin.acc_missed, no_heat.acc_missed)),
        ("Q2 cold has saturated (<= 0.003)", colder_delta.abs() <= 0.003, format!("best T0 {}; test change {colder_delta:+.4}", r(1).best.t0)),
        ("Q3 amplification transfers (< 0.005)", amp_delta.abs() < 0.005, format!("rounds 3-5 change test by {amp_delta:+.4}")),
        ("Q4 law 5: classical within 0.01", ca >= fin.acc_resp - 0.01, format!("classical {ca:.3} vs quantum {:.3}", fin.acc_resp)),
        ("Q5 the profiles persist", fin.acc_cued > cc && cm > fin.acc_missed, format!("cued q {:.3} / c {cc:.3}; missed q {:.3} / c {cm:.3}", fin.acc_cued, fin.acc_missed)),
    ];
    println!();
    for (name, ok, d) in &v {
        println!("{} {name}: {d}", if *ok { "HOLDS " } else { "FAILS " });
    }
    let fails = v.iter().filter(|x| !x.1).count();
    println!("\n{} of {} predictions hold; kill criterion (>= 3 fail): {}", v.len() - fails, v.len(), if fails >= 3 { "FIRES" } else { "does not fire" });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_reproduces_qb8_and_the_classical_reproduces_qb7() {
        let s = Settings::qb7();
        assert!((mean(&Cfg::qb8_kept(), &s, s.test_seeds.clone()).acc_resp - 0.768).abs() < 0.0005);
        let c = Classical { floor: 0.0, gain: 3.0, kappa: 0.0, iota: 0.0, lambda: 0.1, eta_m: 0.05 };
        assert!((classical_mean(&s, s.test_seeds.clone(), &c).0 - 0.747).abs() < 0.0005);
    }

    #[test]
    fn the_log_score_recomputes() {
        let s = Settings::qb7();
        let w = world(&s.spec, 4);
        let tr = run(&Cfg::qb8_kept(), &s, &w, 4);
        let c = checks(&s, &w, &tr);
        let len = s.spec.trial_len();
        let vals: Vec<f64> = (0..tr.pm.len()).filter(|t| t % len >= s.spec.cue).map(|t| tr.pm[t][w.targets[t / len]].max(1e-9).ln()).collect();
        assert!((c.m_log - vals.iter().sum::<f64>() / vals.len() as f64).abs() < 1e-9);
    }
}
