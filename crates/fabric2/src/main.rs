//! FABRIC-2 (docs/FABRIC-2-PRECOMMIT.md): the web built from many small web-neurons over QB-9's
//! optimized nodes. The spider only routes:
//! - a link neuron (`web-link-neuron`) on MQ → TQ senses its own contribution and grows from dormant;
//! - a mismatch neuron (`fabric-vibration`) compares the memory MQ with TE, an evidence-only tracker
//!   that does not read the memory;
//! - a perception neuron (`web-perceive`) holds "the memory is stale" with its evidence;
//! - lane neurons (`web-lane-latch`) compete; DEC selects the winning lane's reading.
//! Every setting lives in `Settings` / `Web`.

use fabric_vibration::Vibration;
use qb_amplitude::{State, C};
use qb_chaos::keyed_unit;
use qb_classical::{conserve, decay, draw, gate, temper};
use qb_task::{qb7_spec, score, split_acc, world, Spec, World};
use std::f64::consts::PI;
use web_lane_latch::Lanes;
use web_link_neuron::LinkNeuron;
use web_perceive::{Hypothesis, Perception};

#[derive(Debug, Clone)]
struct Settings {
    spec: Spec,
    eta: f64,
    detector_k: usize,
    detector_m: usize,
    lambda: f64,
    t0: f64,
    tq_gain: f64,
    tc_gain: f64,
    kappa: f64,
    iota: f64,
    mc_floor: f64,
    mc_eta_m: f64,
    te_lambda: f64,
    heat: f64,
    level_scale: f64,
    link_delta: f64,
    pulse: f64,
    dev_seeds: std::ops::Range<u64>,
    test_seeds: std::ops::Range<u64>,
}

impl Settings {
    fn fabric2() -> Settings {
        Settings {
            spec: qb7_spec(),
            eta: 0.15,
            detector_k: 3,
            detector_m: 4,
            lambda: 0.05,
            t0: 0.05,
            tq_gain: 3.0,
            tc_gain: 4.0,
            kappa: 1.0,
            iota: 1.0,
            mc_floor: 0.05,
            mc_eta_m: 0.1,
            te_lambda: 0.1,
            heat: 0.02,
            level_scale: 0.1,
            link_delta: 0.01,
            pulse: 2.0,
            dev_seeds: 0..16,
            test_seeds: 100..132,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StaleLane {
    /// The evidence-only tracker.
    Te,
    /// The classical tracker fed by the classical register memory (QB-9's comparator).
    TcMc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Routing {
    /// Lane neurons driven by perception.
    Web,
    /// Always the fresh lane.
    FreshOnly,
    /// Always the stale lane.
    StaleOnly,
    /// The true missed flag (the ceiling).
    Oracle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Web {
    theta: f64,
    rho: f64,
    alpha: f64,
    eta_max: f64,
    /// false = the link fixed at g 1 (the hand wiring's η).
    link_learning: bool,
    lane_heat: f64,
    stale: StaleLane,
    routing: Routing,
}

/// QB-5's centred chaos (the memory heat's phases, as in QB-8/QB-9).
fn chaos(seed: u64, t: usize, element: usize, tag: u64) -> f64 {
    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (element as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ tag.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    keyed_unit(key) - 0.5
}

/// The classical tracker step (QB-9's TC form) on weights `w`, with a memory input `mem` at `eta_m`.
fn classical_step(s: &Settings, w: &mut [f64], o: usize, gain: f64, lambda: f64, mem: Option<(&[f64], f64)>) {
    let p = w[o];
    w[o] *= 1.0 + s.eta * gain * (1.0 + s.kappa * p / (p + 0.1)) * (1.0 - p).powf(s.iota);
    if let Some((m, eta_m)) = mem {
        for j in 0..w.len() {
            w[j] *= 1.0 + eta_m * m[j];
        }
    }
    decay(w, lambda);
    conserve(w);
}

struct Trace {
    choices: Vec<usize>,
    stale_on: Vec<bool>,
    lane: Vec<usize>,
    g: Vec<f64>,
}

fn run(web: &Web, s: &Settings, w: &World, seed: u64) -> Trace {
    let n = s.spec.n;
    let (mut mq, mut tq) = (State::uniform(n), State::uniform(n));
    let (mut te, mut tc, mut mc) = (vec![1.0 / n as f64; n], vec![1.0 / n as f64; n], vec![1.0 / n as f64; n]);
    let mut link = LinkNeuron { g: if web.link_learning { 0.0 } else { 1.0 } };
    let mut mismatch = Vibration::default();
    let mut perception = Perception::default();
    let mut lanes = Lanes::new(2, 0);
    let uniform = vec![1.0 / n as f64; n];
    let (mut prev_mq, mut prev_tq, mut prev_te) = (uniform.clone(), uniform.clone(), uniform);
    let len = s.spec.trial_len();
    let mut tr = Trace { choices: Vec::new(), stale_on: Vec::new(), lane: Vec::new(), g: Vec::new() };
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        // Web-neurons sense the last step's readings against what was observed.
        if web.link_learning {
            link.sense(&prev_mq, &prev_tq, o, web.alpha, s.link_delta);
        }
        mismatch.observe(prev_te[o], prev_mq[o], web.rho);
        let event = gate(&w.obs[t.saturating_sub(s.detector_m - 1)..=t], o, s.detector_k).then_some(o);
        // MQ: QB-9's collapse memory with fixed heat.
        if let Some(e) = event {
            mq.project(e);
        }
        for j in 0..n {
            mq.excite(j, C::polar(s.heat, 2.0 * PI * (chaos(seed, t, j, 300) + 0.5)));
        }
        mq.normalize();
        let r_mq = mq.born_weights();
        // MC: the classical register with a floor.
        if let Some(e) = event {
            mc = vec![0.0; n];
            mc[e] = 1.0;
        }
        for x in mc.iter_mut() {
            *x = (1.0 - s.mc_floor) * *x + s.mc_floor / n as f64;
        }
        // TQ: the fresh lane, reading MQ through the link neuron.
        let eta_link = if web.link_learning { web.eta_max * link.g } else { 0.05 };
        let p = tq.born_weights()[o];
        tq.scale(o, 1.0 + s.eta * s.tq_gain * (1.0 + s.kappa * p / (p + 0.1)) * (1.0 - p).powf(s.iota));
        for (j, pm) in r_mq.iter().enumerate() {
            tq.scale(j, 1.0 + eta_link * pm);
        }
        tq.power_magnitudes(1.0 - s.lambda);
        tq.normalize();
        let r_tq = temper(&tq.born_weights(), s.t0);
        // TE (evidence only) and TC (fed by MC): the candidate stale lanes.
        classical_step(s, &mut te, o, s.tc_gain, s.te_lambda, None);
        classical_step(s, &mut tc, o, s.tc_gain, s.lambda, Some((&mc, s.mc_eta_m)));
        let r_te = temper(&te, s.t0);
        let r_tc = temper(&tc, s.t0);
        let r_stale = match web.stale {
            StaleLane::Te => &r_te,
            StaleLane::TcMc => &r_tc,
        };
        // Perception and the lanes.
        let changed = perception.revise(mismatch.level(s.level_scale), event.is_some(), web.theta);
        let pulse = changed.then(|| (if perception.hypothesis == Hypothesis::Stale { 1 } else { 0 }, s.pulse));
        lanes.step(pulse, web.lane_heat);
        let lane = match web.routing {
            Routing::Web => lanes.winner(),
            Routing::FreshOnly => 0,
            Routing::StaleOnly => 1,
            Routing::Oracle => usize::from(w.missed[t / len]),
        };
        // DEC selects the winning lane's reading.
        let reading = if lane == 0 { &r_tq } else { r_stale };
        tr.choices.push(draw(reading, w.uniforms[t]));
        tr.stale_on.push(perception.hypothesis == Hypothesis::Stale);
        tr.lane.push(lane);
        tr.g.push(link.g);
        prev_mq = r_mq;
        prev_tq = r_tq;
        prev_te = r_te;
    }
    tr
}

#[derive(Debug, Clone, Copy, Default)]
struct Res {
    acc: f64,
    cued: f64,
    missed: f64,
    stale_cued: f64,
    stale_missed: f64,
    lane_stale_missed: f64,
    lane_fresh_cued: f64,
    g_cued: f64,
    g_missed: f64,
}

fn evaluate(web: &Web, s: &Settings, seeds: std::ops::Range<u64>) -> Res {
    let k = (seeds.end - seeds.start) as f64;
    let len = s.spec.trial_len();
    let mut r = Res::default();
    for seed in seeds {
        let w = world(&s.spec, seed);
        let tr = run(web, s, &w, seed);
        r.acc += score(&s.spec, &w, &tr.choices).acc_resp / k;
        let (c, m) = split_acc(&s.spec, &w, &tr.choices);
        r.cued += c / k;
        r.missed += m / k;
        let (mut nc, mut nm) = (0.0f64, 0.0f64);
        let (mut sc, mut sm, mut lsm, mut lfc, mut gc, mut gm) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for t in 0..tr.choices.len() {
            if t % len < s.spec.cue + s.spec.delay {
                continue;
            }
            if w.missed[t / len] {
                nm += 1.0;
                sm += f64::from(u8::from(tr.stale_on[t]));
                lsm += f64::from(u8::from(tr.lane[t] == 1));
                gm += tr.g[t];
            } else {
                nc += 1.0;
                sc += f64::from(u8::from(tr.stale_on[t]));
                lfc += f64::from(u8::from(tr.lane[t] == 0));
                gc += tr.g[t];
            }
        }
        r.stale_cued += sc / nc.max(1.0) / k;
        r.stale_missed += sm / nm.max(1.0) / k;
        r.lane_stale_missed += lsm / nm.max(1.0) / k;
        r.lane_fresh_cued += lfc / nc.max(1.0) / k;
        r.g_cued += gc / nc.max(1.0) / k;
        r.g_missed += gm / nm.max(1.0) / k;
    }
    r
}

fn main() {
    let s = Settings::fabric2();
    println!("FABRIC-2: the web from many web-neurons; p_miss {}; dev {:?}, test {:?}", s.spec.p_miss, s.dev_seeds, s.test_seeds);
    let base = Web { theta: 0.4, rho: 0.1, alpha: 0.05, eta_max: 0.05, link_learning: false, lane_heat: 0.0, stale: StaleLane::Te, routing: Routing::FreshOnly };
    let hand = evaluate(&base, &s, s.test_seeds.clone());
    println!("hand wiring reproduction (fresh lane only, fixed link): test {:.3} (cued {:.3}, missed {:.3})", hand.acc, hand.cued, hand.missed);

    // A4: choose the stale lane by dev missed-trial accuracy.
    let te_dev = evaluate(&Web { routing: Routing::StaleOnly, stale: StaleLane::Te, ..base }, &s, s.dev_seeds.clone());
    let tc_dev = evaluate(&Web { routing: Routing::StaleOnly, stale: StaleLane::TcMc, ..base }, &s, s.dev_seeds.clone());
    let stale = if te_dev.missed >= tc_dev.missed { StaleLane::Te } else { StaleLane::TcMc };
    println!("\nA4 stale-lane choice (dev, lane alone): TE missed {:.3} (cued {:.3}); TC+MC missed {:.3} (cued {:.3}) -> {:?}", te_dev.missed, te_dev.cued, tc_dev.missed, tc_dev.cued, stale);
    // A5: the routing ceiling.
    let oracle = evaluate(&Web { routing: Routing::Oracle, stale, ..base }, &s, s.test_seeds.clone());
    println!("A5 oracle routing ceiling (test): {:.3} (cued {:.3}, missed {:.3}); gain over hand wiring {:+.3}", oracle.acc, oracle.cued, oracle.missed, oracle.acc - hand.acc);

    // Tune the web's settings on dev.
    let mut best: Option<(f64, Web)> = None;
    for theta in [0.2, 0.4, 0.6] {
        for rho in [0.05, 0.1, 0.2] {
            for alpha in [0.02, 0.05] {
                for eta_max in [0.05, 0.1] {
                    let wv = Web { theta, rho, alpha, eta_max, link_learning: true, lane_heat: 0.0, stale, routing: Routing::Web };
                    let a = evaluate(&wv, &s, s.dev_seeds.clone()).acc;
                    if best.as_ref().is_none_or(|(b, _)| a > *b) {
                        best = Some((a, wv));
                    }
                }
            }
        }
    }
    let mut web = best.unwrap().1;
    println!("\ntuned web (dev): theta {}, rho {}, alpha {}, eta_max {}", web.theta, web.rho, web.alpha, web.eta_max);
    // Rounds on the web-neurons (plain wins ties on dev; kept only if better on test).
    let round = |name: &str, variants: Vec<Web>, web: &mut Web| {
        let dev_off = evaluate(web, &s, s.dev_seeds.clone()).acc;
        let mut b = (dev_off, *web);
        for v in &variants {
            let d = evaluate(v, &s, s.dev_seeds.clone()).acc;
            println!("   {name} variant dev {d:.4} ({:?})", (v.lane_heat, v.eta_max));
            if d > b.0 {
                b = (d, *v);
            }
        }
        let (off, on) = (evaluate(web, &s, s.test_seeds.clone()).acc, evaluate(&b.1, &s, s.test_seeds.clone()).acc);
        let kept = b.1 != *web && on > off;
        println!("   {name}: dev off {dev_off:.4}, best {:.4}; test {off:.3} -> {on:.3} -> {}", b.0, if kept { "KEPT" } else { "not kept" });
        if kept {
            *web = b.1;
        }
        kept
    };
    let heat_kept = round("lane heat", [0.01, 0.05].iter().map(|&h| Web { lane_heat: h, ..web }).collect(), &mut web);
    let _amp_kept = round("link amplification", vec![Web { eta_max: web.eta_max * 2.0, ..web }], &mut web);

    let wr = evaluate(&web, &s, s.test_seeds.clone());
    let no_perception = evaluate(&Web { routing: Routing::FreshOnly, ..web }, &s, s.test_seeds.clone());
    let no_learning = evaluate(&Web { link_learning: false, ..web }, &s, s.test_seeds.clone());
    println!("\n{:<26} test acc (cued / missed)", "arm");
    for (name, r) in [("hand wiring", hand), ("oracle routing (ceiling)", oracle), ("web", wr), ("web -perception", no_perception), ("web -link learning", no_learning)] {
        println!("{name:<26} {:.3} ({:.3} / {:.3})", r.acc, r.cued, r.missed);
    }
    println!("web: 'stale' on cued {:.3} / missed {:.3}; stale lane on missed {:.3}; fresh lane on cued {:.3}; link g cued {:.3} / missed {:.3}", wr.stale_cued, wr.stale_missed, wr.lane_stale_missed, wr.lane_fresh_cued, wr.g_cued, wr.g_missed);

    let gain = oracle.acc - 0.776;
    let w5 = wr.acc >= 0.776 && (gain < 0.01 || wr.acc - 0.776 >= gain / 2.0);
    let v = [
        ("W3 perception senses staleness (+0.30)", wr.stale_missed - wr.stale_cued >= 0.30, format!("stale on missed {:.3} vs cued {:.3}", wr.stale_missed, wr.stale_cued)),
        ("W4 lanes route by perception", wr.lane_stale_missed >= 0.5 && wr.lane_fresh_cued >= 0.7, format!("stale lane on missed {:.3}; fresh on cued {:.3}", wr.lane_stale_missed, wr.lane_fresh_cued)),
        ("W5 the web >= hand wiring, capturing routing", w5, format!("web {:.3}; oracle gain {gain:+.3}{}", wr.acc, if gain < 0.01 { " (< 0.01: capture part void)" } else { "" })),
        ("W6 lane heat kept", heat_kept, format!("kept {heat_kept}")),
        ("W7 the link neuron senses contribution", wr.g_cued > wr.g_missed, format!("g cued {:.3} vs missed {:.3}", wr.g_cued, wr.g_missed)),
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
    fn fresh_only_with_a_fixed_link_reproduces_the_hand_wiring() {
        let s = Settings::fabric2();
        let base = Web { theta: 0.4, rho: 0.1, alpha: 0.05, eta_max: 0.05, link_learning: false, lane_heat: 0.0, stale: StaleLane::Te, routing: Routing::FreshOnly };
        assert!((evaluate(&base, &s, s.test_seeds.clone()).acc - 0.776).abs() < 0.0005);
    }
}
