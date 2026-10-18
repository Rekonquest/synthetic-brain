//! QB-7 (docs/QB-7-CHEMISTRY-AMPLIFICATION.md): chemistry axioms as amplifiers, one module at a
//! time, in QB-3's world with 40% of cues missing.
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
}

impl Cfg {
    /// QB-6's kept system.
    fn qb6_kept() -> Cfg {
        Cfg { lambda: 0.05, eta_m: 0.05, t0: 0.1, t_alpha: 0.05, t_key: 2, t_gain: 3.0, kappa: 0.0, iota: 0.0, hill: None, latch: false, cascade: false }
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
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        // D.
        let event = match cfg.hill {
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
        tr.choices.push(draw(&temper(&tracker.born_weights(), cfg.t0), w.uniforms[t]));
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
    println!("QB-7: chemistry amplifiers per module; QB-3's world with p_miss {}; dev {:?}, test {:?}", s.spec.p_miss, s.dev_seeds, s.test_seeds);
    let mut base = Cfg::qb6_kept();
    let b0 = mean(&base, &s, s.test_seeds.clone());
    println!("base (QB-6 kept) on test: D F1 {:.3}, M fidelity {:.3}, acc_resp {:.3} (cued {:.3}, missed {:.3})", b0.d_f1, b0.m_fidelity, b0.acc_resp, b0.acc_cued, b0.acc_missed);
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
    let mut hv = Vec::new();
    for nh in [2.0, 4.0, 8.0] {
        for m in [4, 8] {
            hv.push((format!("Hill n {nh} window {m}"), Cfg { hill: Some((nh, m)), ..base.clone() }));
        }
    }
    go!("1 D cooperative binding (Hill)", Module::D, hv);
    go!("2 M autocatalytic latch vs collapse", Module::M, vec![("latch".to_owned(), Cfg { latch: true, ..base.clone() })]);
    go!("3 T enzyme cascade (replacing the gain)", Module::T, [1.0, 2.0, 3.0].iter().map(|&g| (format!("cascade G {g}"), Cfg { cascade: true, t_gain: g, ..base.clone() })).collect());
    let mut gv = Vec::new();
    for k in [0.5, 1.0] {
        for i in [1.0, 2.0] {
            gv.push((format!("growth kappa {k} brake iota {i}"), Cfg { kappa: k, iota: i, ..base.clone() }));
        }
    }
    let reference = mean(&Cfg { kappa: 1.0, iota: 0.0, ..base.clone() }, &s, s.dev_seeds.clone()).acc_resp;
    go!("4 T growth with product inhibition", Module::T, gv);
    for r in &rounds {
        println!("\n== {} (check: {:?})", r.name, r.module);
        for (label, d) in &r.dev_variants {
            println!("   dev {label:<32} {d:.4}");
        }
        println!("   dev: off {:.4}, best {:.4} ({})", r.dev_off, r.dev_on, if r.best == r.base { "plain wins".to_owned() } else { "variant".to_owned() });
        println!(
            "   test: off check {:.4} / acc {:.3} (cued {:.3}, missed {:.3}); on check {:.4} / acc {:.3} (cued {:.3}, missed {:.3}) -> {}",
            check_of(r.module, &r.test_off), r.test_off.acc_resp, r.test_off.acc_cued, r.test_off.acc_missed,
            check_of(r.module, &r.test_on), r.test_on.acc_resp, r.test_on.acc_cued, r.test_on.acc_missed,
            if r.kept { "KEPT" } else { "not kept" }
        );
    }
    println!("   reference (round 4 base + growth kappa 1 with no brake, dev): {reference:.4}");
    let fin = mean(&base, &s, s.test_seeds.clone());
    println!("\nfinal kept system on test: D F1 {:.3}, M fidelity {:.3}, acc_resp {:.3} (cued {:.3}, missed {:.3})", fin.d_f1, fin.m_fidelity, fin.acc_resp, fin.acc_cued, fin.acc_missed);
    // Classical comparator, tuned on dev.
    let mut best = (f64::MIN, 0.0, 0.0);
    for l in [0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.4] {
        for e in [0.05, 0.1, 0.2, 0.4, 0.8] {
            let a = classical_mean(&s, s.dev_seeds.clone(), l, e).0;
            if a > best.0 {
                best = (a, l, e);
            }
        }
    }
    let (ca, cc, cm) = classical_mean(&s, s.test_seeds.clone(), best.1, best.2);
    println!("classical modular comparator (lambda {}, eta_m {}): test acc_resp {ca:.3} (cued {cc:.3}, missed {cm:.3})", best.1, best.2);

    let r = |i: usize| &rounds[i];
    let latch_fid = mean(&Cfg { latch: true, ..r(1).base.clone() }, &s, s.test_seeds.clone()).m_fidelity;
    let v = [
        ("C1 cooperative binding kept on D", r(0).kept, format!("F1 {:.4} -> {:.4}", check_of(Module::D, &r(0).test_off), check_of(Module::D, &r(0).test_on))),
        ("C2 the latch ties collapse (within 0.02)", (latch_fid - r(1).test_off.m_fidelity).abs() <= 0.02, format!("latch {latch_fid:.4} vs collapse {:.4}", r(1).test_off.m_fidelity)),
        ("C3 the cascade beats the plain gain (+0.005)", r(2).kept && r(2).test_on.acc_resp - r(2).test_off.acc_resp >= 0.005, format!("kept {}; {:.3} -> {:.3}", r(2).kept, r(2).test_off.acc_resp, r(2).test_on.acc_resp)),
        ("C4 product inhibition lets growth coexist (+0.003)", r(3).kept && r(3).test_on.acc_resp - r(3).test_off.acc_resp >= 0.003, format!("kept {}; {:.3} -> {:.3}", r(3).kept, r(3).test_off.acc_resp, r(3).test_on.acc_resp)),
        ("C5 classical ties or wins (law 5)", ca >= fin.acc_resp - 0.01, format!("classical {ca:.3} vs final {:.3}", fin.acc_resp)),
        ("C6 missed trials are much harder (>= 0.20)", b0.acc_cued - b0.acc_missed >= 0.20, format!("cued {:.3} vs missed {:.3}", b0.acc_cued, b0.acc_missed)),
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
    fn the_base_at_no_missing_cues_reproduces_qb6() {
        let mut s = Settings::qb7();
        s.spec.p_miss = 0.0;
        let m = mean(&Cfg::qb6_kept(), &s, s.test_seeds.clone());
        assert!((m.acc_resp - 0.915).abs() < 0.0005, "{}", m.acc_resp);
    }

    #[test]
    fn the_latch_writes_holds_and_switches() {
        let s = Settings::qb7();
        let mut l = Latch::new(16);
        l.step(Some(3), s.latch);
        for _ in 0..40 {
            l.step(None, s.latch);
        }
        assert_eq!(l.held(LATCH_THRESHOLD), Some(3));
        assert!((l.b[3] - 1.5).abs() < 0.05, "{}", l.b[3]);
        l.step(Some(9), s.latch);
        for _ in 0..40 {
            l.step(None, s.latch);
        }
        assert_eq!(l.held(LATCH_THRESHOLD), Some(9));
        assert!(l.b[3] < 1e-3);
    }

    #[test]
    fn a_sharp_hill_matches_the_plain_gate() {
        let s = Settings::qb7();
        let w = world(&s.spec, 5);
        let plain = run(&Cfg::qb6_kept(), &s, &w, 5);
        let hill = run(&Cfg { hill: Some((64.0, 4)), ..Cfg::qb6_kept() }, &s, &w, 5);
        let same = plain.events.iter().zip(&hill.events).filter(|(a, b)| a == b).count();
        assert!(same as f64 / plain.events.len() as f64 > 0.99, "{same}");
    }

    #[test]
    fn cascade_activations_stay_in_range() {
        let (ka, kd) = Settings::qb7().cascade;
        let (mut x1, mut x2) = (0.0f64, 0.0f64);
        for t in 0..1000 {
            let input = if t % 3 == 0 { 1.0 } else { 0.0 };
            x1 = (x1 + ka * input * (1.0 - x1) - kd * x1).clamp(0.0, 1.0);
            x2 = (x2 + ka * x1 * (1.0 - x2) - kd * x2).clamp(0.0, 1.0);
            assert!((0.0..=1.0).contains(&x1) && (0.0..=1.0).contains(&x2));
        }
    }
}
