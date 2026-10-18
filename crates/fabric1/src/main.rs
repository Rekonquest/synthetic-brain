//! FABRIC-1 (docs/FABRIC-1-PRECOMMIT.md): the spiderweb as the synthetic brain's wiring.
//!
//! The spider (this orchestrator) routes readings between QB-9's optimized nodes — MQ (gate +
//! quantum collapse memory with heat), MC (gate + classical register with a floor), TQ (quantum
//! tracker), TC (classical tracker), DEC (decision) — over links whose conductances form by use
//! (`fabric-conductance`: a link is maintained by readings that predicted the next observation).
//! A vibration (`fabric-vibration`: the tracker out-predicting the memory) sets MQ's heat; an on-ramp
//! (`fabric-ramp`) promotes detector events to a broadcast for the trackers. The spider holds no rule
//! about which module to trust: trust is the conductances. Every setting lives in `Fab` / `Settings`.

use fabric_conductance::{conductances, Link};
use fabric_ramp::broadcast;
use fabric_thread::strongest;
use fabric_vibration::Vibration;
use qb_amplitude::{State, C};
use qb_chaos::keyed_unit;
use qb_classical::{conserve, decay, draw, gate, temper};
use qb_task::{qb7_spec, score, split_acc, world, Spec, World};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
struct Settings {
    spec: Spec,
    eta: f64,
    detector_k: usize,
    detector_m: usize,
    // QB-9's optimized node settings.
    lambda: f64,
    t0: f64,
    tq_gain: f64,
    tc_gain: f64,
    kappa: f64,
    iota: f64,
    mc_floor: f64,
    heat_fixed: f64,
    vibration_scale: f64,
    dev_seeds: std::ops::Range<u64>,
    test_seeds: std::ops::Range<u64>,
}

impl Settings {
    fn fabric1() -> Settings {
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
            heat_fixed: 0.02,
            vibration_scale: 0.1,
            dev_seeds: 0..16,
            test_seeds: 100..132,
        }
    }
}

/// Which nodes take part (the arm's population).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Population {
    Both,
    QuantumOnly,
    ClassicalOnly,
}

/// The fabric's own settings (tuned on dev), or a frozen hand wiring.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fab {
    rho: f64,
    gamma: f64,
    eta_link: f64,
    eps_max: f64,
    h: f64,
    vibration: bool,
    population: Population,
    /// Frozen at QB-9's hand wiring: MQ → TQ → DEC, fixed heat, no highway, no learning.
    frozen: bool,
}

/// QB-5's centred chaos (the heat's random phases use the same keys as QB-8/QB-9).
fn chaos(seed: u64, t: usize, element: usize, tag: u64) -> f64 {
    let key = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (t as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ (element as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ tag.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    keyed_unit(key) - 0.5
}

/// Sources into DEC, in this order.
const MQ: usize = 0;
const MC: usize = 1;
const TQ: usize = 2;
const TC: usize = 3;

struct Trace {
    choices: Vec<usize>,
    /// DEC's conductances per step, over [MQ, MC, TQ, TC].
    g_dec: Vec<[f64; 4]>,
    /// The trackers' memory-input conductances per step: [MQ→TQ, MC→TQ, MQ→TC, MC→TC].
    g_mem: Vec<[f64; 4]>,
}

fn run(f: &Fab, s: &Settings, w: &World, seed: u64) -> Trace {
    let n = s.spec.n;
    let present = |src: usize| match f.population {
        Population::Both => true,
        Population::QuantumOnly => src == MQ || src == TQ,
        Population::ClassicalOnly => src == MC || src == TC,
    };
    let (mut mq, mut tq) = (State::uniform(n), State::uniform(n));
    let (mut mc, mut tc) = (vec![1.0 / n as f64; n], vec![1.0 / n as f64; n]);
    let mut dec_links = [Link::default(); 4];
    let mut tq_links = [Link::default(); 2];
    let mut tc_links = [Link::default(); 2];
    let mut vib = Vibration::default();
    let uniform = vec![1.0 / n as f64; n];
    let mut prev: [Vec<f64>; 4] = [uniform.clone(), uniform.clone(), uniform.clone(), uniform.clone()];
    let mut tr = Trace { choices: Vec::new(), g_dec: Vec::new(), g_mem: Vec::new() };
    for t in 0..w.obs.len() {
        let o = w.obs[t];
        // Threads form by use: each link is scored by how well its last reading predicted o.
        if !f.frozen {
            for (k, l) in dec_links.iter_mut().enumerate() {
                l.observe(&prev[k], o, f.rho);
            }
            for (k, l) in tq_links.iter_mut().enumerate() {
                l.observe(&prev[k], o, f.rho);
            }
            for (k, l) in tc_links.iter_mut().enumerate() {
                l.observe(&prev[k], o, f.rho);
            }
            vib.observe(prev[TQ][o], prev[MQ][o], f.rho);
        }
        let event = gate(&w.obs[t.saturating_sub(s.detector_m - 1)..=t], o, s.detector_k).then_some(o);
        // MQ: collapse memory, heat from the vibration (or fixed).
        if let Some(e) = event {
            mq.project(e);
        }
        let heat = if f.frozen || !f.vibration { s.heat_fixed } else { f.eps_max * vib.level(s.vibration_scale) };
        if heat > 0.0 {
            for j in 0..n {
                mq.excite(j, C::polar(heat, 2.0 * PI * (chaos(seed, t, j, 300) + 0.5)));
            }
            mq.normalize();
        }
        let r_mq = mq.born_weights();
        // MC: classical register with a floor.
        if let Some(e) = event {
            mc = vec![0.0; n];
            mc[e] = 1.0;
        }
        for x in mc.iter_mut() {
            *x = (1.0 - s.mc_floor) * *x + s.mc_floor / n as f64;
        }
        let r_mc = mc.clone();
        // Conductances (competition among each sink's present inputs).
        let masked = |links: &[Link], srcs: &[usize]| -> Vec<f64> {
            let present_links: Vec<Link> = links.iter().zip(srcs).map(|(l, &sc)| if present(sc) { *l } else { Link { score: 0.0 } }).collect();
            let g = conductances(&present_links, f.gamma);
            g.iter().zip(srcs).map(|(x, &sc)| if present(sc) { *x } else { 0.0 }).collect()
        };
        let (g_tq, g_tc, g_dec): (Vec<f64>, Vec<f64>, Vec<f64>) = if f.frozen {
            (vec![1.0, 0.0], vec![0.0, 1.0], vec![0.0, 0.0, 1.0, 0.0])
        } else {
            let z = |v: Vec<f64>| { let t: f64 = v.iter().sum(); v.iter().map(|x| if t > 0.0 { x / t } else { 0.0 }).collect::<Vec<_>>() };
            (z(masked(&tq_links, &[MQ, MC])), z(masked(&tc_links, &[MQ, MC])), z(masked(&dec_links, &[MQ, MC, TQ, TC])))
        };
        let eta_link = if f.frozen { 0.05 } else { f.eta_link };
        let boost = broadcast(event, n, if f.frozen { 0.0 } else { f.h });
        // TQ: QB-9's quantum tracker, fed by its memory inputs and the highway.
        let p = tq.born_weights()[o];
        tq.scale(o, 1.0 + s.eta * s.tq_gain * (1.0 + s.kappa * p / (p + 0.1)) * (1.0 - p).powf(s.iota));
        for j in 0..n {
            tq.scale(j, 1.0 + eta_link * (g_tq[0] * r_mq[j] + g_tq[1] * r_mc[j]));
            if boost[j] != 1.0 {
                tq.scale(j, boost[j]);
            }
        }
        tq.power_magnitudes(1.0 - s.lambda);
        tq.normalize();
        let r_tq = temper(&tq.born_weights(), s.t0);
        // TC: QB-9's classical tracker, fed the same way.
        let p = tc[o];
        tc[o] *= 1.0 + s.eta * s.tc_gain * (1.0 + s.kappa * p / (p + 0.1)) * (1.0 - p).powf(s.iota);
        for j in 0..n {
            tc[j] *= (1.0 + eta_link * (g_tc[0] * r_mq[j] + g_tc[1] * r_mc[j])) * boost[j];
        }
        decay(&mut tc, s.lambda);
        conserve(&mut tc);
        let r_tc = temper(&tc, s.t0);
        // DEC: the conductance-weighted mixture of its present inputs.
        let readings = [&r_mq, &r_mc, &r_tq, &r_tc];
        let mix: Vec<f64> = (0..n).map(|j| (0..4).map(|k| g_dec[k] * readings[k][j]).sum()).collect();
        tr.choices.push(draw(&mix, w.uniforms[t]));
        tr.g_dec.push([g_dec[0], g_dec[1], g_dec[2], g_dec[3]]);
        tr.g_mem.push([g_tq[0], g_tq[1], g_tc[0], g_tc[1]]);
        prev = [r_mq, r_mc, r_tq, r_tc];
    }
    tr
}

#[derive(Debug, Clone, Copy, Default)]
struct Result {
    acc: f64,
    cued: f64,
    missed: f64,
    /// Mean DEC conductances over response steps: all, cued trials, missed trials.
    g_all: [f64; 4],
    g_cued: [f64; 4],
    g_missed: [f64; 4],
    g_mem: [f64; 4],
}

fn evaluate(f: &Fab, s: &Settings, seeds: std::ops::Range<u64>) -> Result {
    let k = (seeds.end - seeds.start) as f64;
    let len = s.spec.trial_len();
    let mut r = Result::default();
    for seed in seeds {
        let w = world(&s.spec, seed);
        let tr = run(f, s, &w, seed);
        r.acc += score(&s.spec, &w, &tr.choices).acc_resp / k;
        let (c, m) = split_acc(&s.spec, &w, &tr.choices);
        r.cued += c / k;
        r.missed += m / k;
        let (mut na, mut nc, mut nm) = (0.0f64, 0.0f64, 0.0f64);
        let (mut ga, mut gc, mut gm, mut gmem) = ([0.0; 4], [0.0; 4], [0.0; 4], [0.0; 4]);
        for t in 0..tr.choices.len() {
            if t % len < s.spec.cue + s.spec.delay {
                continue;
            }
            na += 1.0;
            for i in 0..4 {
                ga[i] += tr.g_dec[t][i];
                gmem[i] += tr.g_mem[t][i];
            }
            if w.missed[t / len] {
                nm += 1.0;
                for i in 0..4 {
                    gm[i] += tr.g_dec[t][i];
                }
            } else {
                nc += 1.0;
                for i in 0..4 {
                    gc[i] += tr.g_dec[t][i];
                }
            }
        }
        for i in 0..4 {
            r.g_all[i] += ga[i] / na / k;
            r.g_cued[i] += gc[i] / nc.max(1.0) / k;
            r.g_missed[i] += gm[i] / nm.max(1.0) / k;
            r.g_mem[i] += gmem[i] / na / k;
        }
    }
    r
}

fn tune(pop: Population, s: &Settings) -> Fab {
    let mut best: Option<(f64, Fab)> = None;
    for rho in [0.02, 0.05, 0.1, 0.2] {
        for gamma in [1.0, 2.0, 4.0] {
            for eta_link in [0.025, 0.05, 0.1] {
                for eps_max in [0.02, 0.04, 0.08] {
                    for h in [0.0, 0.5, 1.0] {
                        let f = Fab { rho, gamma, eta_link, eps_max, h, vibration: true, population: pop, frozen: false };
                        let a = evaluate(&f, s, s.dev_seeds.clone()).acc;
                        if best.as_ref().is_none_or(|(b, _)| a > *b) {
                            best = Some((a, f));
                        }
                    }
                }
            }
        }
    }
    best.unwrap().1
}

fn main() {
    let s = Settings::fabric1();
    println!("FABRIC-1: the spiderweb fabric over QB-9's optimized nodes; p_miss {}; dev {:?}, test {:?}", s.spec.p_miss, s.dev_seeds, s.test_seeds);
    let hand = Fab { rho: 0.0, gamma: 1.0, eta_link: 0.05, eps_max: 0.0, h: 0.0, vibration: false, population: Population::QuantumOnly, frozen: true };
    let hw = evaluate(&hand, &s, s.test_seeds.clone());
    println!("frozen hand wiring (QB-9 reproduction): test {:.3} (cued {:.3}, missed {:.3})", hw.acc, hw.cued, hw.missed);
    let fab = tune(Population::Both, &s);
    let qo = tune(Population::QuantumOnly, &s);
    let co = tune(Population::ClassicalOnly, &s);
    let arms = [
        ("fabric", fab),
        ("fabric-quantum-only", qo),
        ("fabric-classical-only", co),
        ("-vibration", Fab { vibration: false, ..fab }),
        ("-highway", Fab { h: 0.0, ..fab }),
    ];
    println!("\n{:<22} {:>6} {:>6} {:>6} {:>8} {:>6}   test acc (cued / missed)", "arm", "rho", "gamma", "eta_l", "eps_max", "h");
    let mut res = Vec::new();
    for (name, f) in &arms {
        let r = evaluate(f, &s, s.test_seeds.clone());
        println!("{name:<22} {:>6} {:>6} {:>6} {:>8} {:>6}   {:.3} ({:.3} / {:.3})", f.rho, f.gamma, f.eta_link, f.eps_max, f.h, r.acc, r.cued, r.missed);
        res.push((*name, r));
    }
    let get = |name: &str| res.iter().find(|(n, _)| *n == name).unwrap().1;
    let fr = get("fabric");
    let names = ["MQ", "MC", "TQ", "TC"];
    let fmt = |g: &[f64; 4]| names.iter().zip(g).map(|(n, x)| format!("{n} {x:.3}")).collect::<Vec<_>>().join(", ");
    println!("\nDEC conductances on response steps (fabric, test): all [{}]", fmt(&fr.g_all));
    println!("   cued trials [{}]", fmt(&fr.g_cued));
    println!("   missed trials [{}]", fmt(&fr.g_missed));
    println!("tracker memory inputs: MQ->TQ {:.3}, MC->TQ {:.3}, MQ->TC {:.3}, MC->TC {:.3}", fr.g_mem[0], fr.g_mem[1], fr.g_mem[2], fr.g_mem[3]);
    let edges = [(0, 2, fr.g_mem[0]), (1, 2, fr.g_mem[1]), (0, 3, fr.g_mem[2]), (1, 3, fr.g_mem[3]), (0, 4, fr.g_all[0]), (1, 4, fr.g_all[1]), (2, 4, fr.g_all[2]), (3, 4, fr.g_all[3])];
    let (path, strength) = strongest(&edges, 5, 4);
    let label = ["MQ", "MC", "TQ", "TC", "DEC"];
    println!("strongest thread into DEC: {} (strength {strength:.3})", path.iter().map(|&i| label[i]).collect::<Vec<_>>().join(" -> "));

    let tracker_share = fr.g_all[TQ] + fr.g_all[TC];
    let (qr, cr, nv, nh) = (get("fabric-quantum-only"), get("fabric-classical-only"), get("-vibration"), get("-highway"));
    let v = [
        ("F1 threads emerge with the hand wiring's shape", tracker_share >= 0.60, format!("trackers carry {tracker_share:.3} of DEC")),
        ("F2 the fabric is at least the hand wiring", fr.acc >= 0.776, format!("fabric {:.3} vs hand-wired 0.776 (reproduced {:.3})", fr.acc, hw.acc)),
        ("F3 routing by vibration", fr.g_missed[TC] > fr.g_missed[TQ] && fr.g_cued[TQ] > fr.g_cued[TC] && fr.acc > 0.776 && fr.acc > 0.775, format!("missed TC {:.3} vs TQ {:.3}; cued TQ {:.3} vs TC {:.3}; acc {:.3}", fr.g_missed[TC], fr.g_missed[TQ], fr.g_cued[TQ], fr.g_cued[TC], fr.acc)),
        ("F4 the vibration earns its place", nv.missed < fr.missed, format!("missed: fabric {:.3} vs -vibration {:.3}", fr.missed, nv.missed)),
        ("F5 diversity pays (+0.005 over each)", fr.acc >= qr.acc + 0.005 && fr.acc >= cr.acc + 0.005, format!("fabric {:.3}; quantum-only {:.3}; classical-only {:.3}", fr.acc, qr.acc, cr.acc)),
        ("F6 the highway earns its place", nh.cued <= fr.cued, format!("cued: fabric {:.3} vs -highway {:.3}", fr.cued, nh.cued)),
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
    fn frozen_at_the_hand_wiring_it_reproduces_qb9() {
        let s = Settings::fabric1();
        let hand = Fab { rho: 0.0, gamma: 1.0, eta_link: 0.05, eps_max: 0.0, h: 0.0, vibration: false, population: Population::QuantumOnly, frozen: true };
        let r = evaluate(&hand, &s, s.test_seeds.clone());
        assert!((r.acc - 0.776).abs() < 0.0005, "{}", r.acc);
    }

    #[test]
    fn conductances_sum_to_one_per_sink() {
        let s = Settings::fabric1();
        let w = world(&s.spec, 2);
        let f = Fab { rho: 0.1, gamma: 2.0, eta_link: 0.05, eps_max: 0.04, h: 0.5, vibration: true, population: Population::Both, frozen: false };
        let tr = run(&f, &s, &w, 2);
        for g in &tr.g_dec {
            assert!((g.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        }
        for g in &tr.g_mem {
            assert!((g[0] + g[1] - 1.0).abs() < 1e-9 && (g[2] + g[3] - 1.0).abs() < 1e-9);
        }
    }
}
