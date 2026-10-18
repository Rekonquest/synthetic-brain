//! QB-8 (docs/QB-8-RETEST-CHAOS-TEMPERATURE.md): chaos and hot/cold retested, one module at a time,
//! on QB-7's composed stack in QB-7's world. Built from QB-7's runner (copied) plus QB-5's knobs.
//!
//! Modules (QB-4's architecture): D detector → M memory → T tracker. The chemistry amplifiers:
//! cooperative binding (Hill) on D, an autocatalytic latch as a rival M, an enzyme cascade replacing
//! T's plain gain, and growth with product inhibition on T. Protocol: QB-5's (the module's own check
//! selects on dev seeds, plain wins ties, kept only if it beats off on test). Every setting lives in
//! `Settings` and `Cfg`.

use qb_amplitude::State;
use qb_chaos::keyed_unit;
use qb_classical::{argmax, conserve, decay, draw, gate, reinforce, temper};
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
}

fn run(cfg: &Cfg, s: &Settings, w: &World, seed: u64) -> Trace {
    let n = s.spec.n;
    let mut memory = State::uniform(n);
    let mut latch = Latch::new(n);
    let mut tracker = State::uniform(n);
    let (mut x1, mut x2) = (vec![0.0; n], vec![0.0; n]);
    let mut hill_rng = Rng(seed ^ 0x4111_4111_0000_0009);
    let mut tr = Trace { choices: Vec::new(), events: Vec::new(), held: Vec::new() };
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
    Checks { d_f1, m_fidelity: held as f64 / steps as f64, acc_resp: score(&s.spec, w, &tr.choices).acc_resp, acc_cued, acc_missed }
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
        m.acc_cued += c.acc_cued / k;
        m.acc_missed += c.acc_missed / k;
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

/// The classical modular comparator (QB-4's, with the evidence gain ×3), λ and η_m tuned on dev.
#[allow(dead_code)]
fn classical(s: &Settings, w: &World, lambda: f64, eta_m: f64) -> Vec<usize> {
    let n = s.spec.n;
    let (mut register, mut weights) = (vec![1.0 / n as f64; n], vec![1.0 / n as f64; n]);
    let mut out = Vec::new();
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        if gate(&w.obs[t.saturating_sub(s.detector_m - 1)..=t], o, s.detector_k) {
            register = vec![0.0; n];
            register[o] = 1.0;
        }
        reinforce(&mut weights, o, s.eta * 3.0);
        for (j, r) in register.iter().enumerate() {
            weights[j] *= 1.0 + eta_m * r;
        }
        decay(&mut weights, lambda);
        conserve(&mut weights);
        out.push(argmax(&weights));
    }
    out
}

#[allow(dead_code)]
fn classical_mean(s: &Settings, seeds: std::ops::Range<u64>, lambda: f64, eta_m: f64) -> (f64, f64, f64) {
    let k = (seeds.end - seeds.start) as f64;
    let (mut a, mut c, mut m) = (0.0, 0.0, 0.0);
    for seed in seeds {
        let w = world(&s.spec, seed);
        let ch = classical(s, &w, lambda, eta_m);
        a += score(&s.spec, &w, &ch).acc_resp / k;
        let (x, y) = split_acc(&s.spec, &w, &ch);
        c += x / k;
        m += y / k;
    }
    (a, c, m)
}

fn main() {
    let s = Settings::qb7();
    println!("QB-8: retest chaos and hot/cold per module on QB-7's composed stack; p_miss {}; dev {:?}, test {:?}", s.spec.p_miss, s.dev_seeds, s.test_seeds);
    let mut base = Cfg::qb7_kept();
    let b0 = mean(&base, &s, s.test_seeds.clone());
    println!("base (QB-7 kept) on test: D F1 {:.3}, M fidelity {:.3}, acc_resp {:.3} (cued {:.3}, missed {:.3})", b0.d_f1, b0.m_fidelity, b0.acc_resp, b0.acc_cued, b0.acc_missed);
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
    let mut dv = Vec::new();
    for a in [0.1, 0.2, 0.4] {
        for k in 0..3 {
            dv.push((format!("D chaos {a} key {k}"), Cfg { d_alpha: a, d_key: k, ..base.clone() }));
        }
    }
    go!("1 D chaos before the gate", Module::D, dv);
    let mut mv: Vec<(String, Cfg)> = [0.1, 0.5, 1.0].iter().map(|&a| (format!("M phase chaos {a}"), Cfg { m_phase: a, ..base.clone() })).collect();
    mv.extend([0.005, 0.01, 0.02].iter().map(|&e| (format!("M occupation heat {e}"), Cfg { m_occ: e, ..base.clone() })));
    go!("2 M phase chaos and heat", Module::M, mv);
    go!("3 T cold and hot", Module::T, [0.05, 0.15, 0.2, 0.3, 0.4].iter().map(|&t| (format!("T0 {t}"), Cfg { t0: t, ..base.clone() })).collect());
    let t3 = base.clone();
    let mut gv = Vec::new();
    for hot in [0.5, 1.0] {
        for tau in [3.0, 10.0] {
            gv.push((format!("governor hot {hot} tau {tau}"), Cfg { governor: Some((hot, tau)), ..base.clone() }));
        }
    }
    go!("4 T governor", Module::T, gv);
    let mut av = vec![("amplitude chaos OFF".to_owned(), Cfg { t_alpha: 0.0, ..base.clone() })];
    for a in [0.05, 0.1, 0.2] {
        for k in 0..3 {
            av.push((format!("amplitude chaos {a} key {k}"), Cfg { t_alpha: a, t_key: k, ..base.clone() }));
        }
    }
    let amp_base = base.clone();
    go!("5 T amplitude chaos (off and on)", Module::T, av);
    for r in &rounds {
        println!("\n== {} (check: {:?})", r.name, r.module);
        for (label, d) in &r.dev_variants {
            println!("   dev {label:<28} {d:.4}");
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
    println!("\nfinal kept system on test: acc_resp {:.3} (cued {:.3}, missed {:.3}); QB-7 kept {:.3}", fin.acc_resp, fin.acc_cued, fin.acc_missed, b0.acc_resp);
    // The amplitude-chaos variants' test scores (all of them), for R5.
    let mut amp_tests = Vec::new();
    for (label, c) in std::iter::once(("amplitude chaos OFF".to_owned(), Cfg { t_alpha: 0.0, ..amp_base.clone() })).chain([0.05, 0.1, 0.2].iter().flat_map(|&a| (0..3).map(move |k| (a, k))).map(|(a, k)| (format!("amplitude chaos {a} key {k}"), Cfg { t_alpha: a, t_key: k, ..amp_base.clone() }))) {
        amp_tests.push((label, mean(&c, &s, s.test_seeds.clone()).acc_resp));
    }
    let amp_ref = mean(&amp_base, &s, s.test_seeds.clone()).acc_resp;
    println!("amplitude chaos on test (round-5 base {amp_ref:.4}): {:?}", amp_tests.iter().map(|(l, a)| format!("{l}: {a:.4}")).collect::<Vec<_>>());
    let r = |i: usize| &rounds[i];
    let phase_ties = r(1).dev_variants.iter().filter(|(l, _)| l.contains("phase")).all(|(_, d)| (*d - r(1).dev_off).abs() < 1e-12);
    let heat_ties = r(1).dev_variants.iter().filter(|(l, _)| l.contains("occupation")).all(|(_, d)| (*d - r(1).dev_off).abs() < 1e-12);
    let v = [
        ("R1 D chaos still neutral (not kept)", !r(0).kept, format!("kept {}", r(0).kept)),
        ("R2 M still immune (phase and heat tie)", phase_ties && heat_ties && !r(1).kept, format!("phase ties {phase_ties}, heat ties {heat_ties}")),
        ("R3 the best temperature moves warmer, raising missed", r(2).best.t0 > 0.1 && r(2).kept && r(2).test_on.acc_missed > r(2).test_off.acc_missed, format!("best T0 {}; kept {}; missed {:.3} -> {:.3}", r(2).best.t0, r(2).kept, r(2).test_off.acc_missed, r(2).test_on.acc_missed)),
        ("R4 the governor still not kept", !r(3).kept, format!("kept {}", r(3).kept)),
        ("R5 amplitude chaos neutral (within 0.003)", amp_tests.iter().all(|(_, a)| (a - amp_ref).abs() <= 0.003), format!("max deviation {:.4}", amp_tests.iter().map(|(_, a)| (a - amp_ref).abs()).fold(0.0, f64::max))),
    ];
    let _ = t3;
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
    fn the_base_reproduces_qb7() {
        let s = Settings::qb7();
        let m = mean(&Cfg::qb7_kept(), &s, s.test_seeds.clone());
        assert!((m.acc_resp - 0.749).abs() < 0.0005, "{}", m.acc_resp);
    }
}
