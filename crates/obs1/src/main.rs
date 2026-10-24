//! OBS-1 orchestrator (docs/OBS-1-PRECOMMIT.md): an observer that senses readings, perceives named
//! behaviours (each perception cites its evidence: the reading series and the step it crossed) and
//! reacts. Three families of synthetic streams with planted behaviours: F1 data, F2 written ideas
//! (phase-coded symbols), F3 the brain's thoughts (probability vectors over answers). Status-quo
//! detectors are the bars. Every threshold is chosen on dev seeds 0-3 and reported on 10-17.
//!
//! Layout: sensing computes reading series per stream; perception is one rule per behaviour, each
//! firing at the first step its reading crosses its threshold (the evidence is that series and step);
//! the named perception is the first rule to fire; judgment (the reaction) is the thin layer used in
//! the reaction test only.

use obs_novelty::Memory;
use qb_amplitude::C;
use std::f64::consts::PI;

const WINDOW: usize = 200;

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
    fn normal(&mut self) -> f64 {
        (-2.0 * self.unit().max(1e-300).ln()).sqrt() * (2.0 * PI * self.unit()).cos()
    }
}

/// A perception rule's first firing step for each candidate threshold (None = never), after warm-up.
type Fires = Vec<Option<usize>>;

/// The first step from `from` where `pred(t)` holds for `run` consecutive steps (the step it completes).
fn first_run(len: usize, from: usize, run: usize, pred: &dyn Fn(usize) -> bool) -> Option<usize> {
    let mut c = 0;
    for t in from..len {
        if pred(t) {
            c += 1;
            if c >= run {
                return Some(t);
            }
        } else {
            c = 0;
        }
    }
    None
}

/// One labelled stream's perception: per rule, per threshold, the first firing step.
struct Seen {
    kind: usize,
    start: usize,
    rules: Vec<Fires>,
}

/// Score with chosen thresholds: the named perception is the earliest rule to fire (ties: rule order).
/// `credit(rule, kind)` says whether naming `rule` is right for `kind`. (correct, false alarm).
fn verdict(s: &Seen, pick: &[usize], credit: &dyn Fn(usize, usize) -> bool) -> (bool, bool) {
    let first = s.rules.iter().enumerate().filter_map(|(r, f)| f[pick[r]].map(|t| (t, r))).min();
    match first {
        None => (false, false),
        Some((t, r)) => {
            if s.kind == 0 {
                (false, true)
            } else {
                (credit(r, s.kind) && t >= s.start && t <= s.start + WINDOW, t < s.start)
            }
        }
    }
}

/// Choose each rule's threshold on dev: (detections of its own kind in the window) − 2 × (fires on
/// "none" streams), each rule judged alone.
fn choose(dev: &[Seen], credit: &dyn Fn(usize, usize) -> bool) -> Vec<usize> {
    let nr = dev[0].rules.len();
    (0..nr)
        .map(|r| {
            let nk = dev[0].rules[r].len();
            (0..nk)
                .max_by(|&a, &b| {
                    let score = |k: usize| -> f64 {
                        let (mut hit, mut own, mut fa, mut none) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
                        for s in dev {
                            if s.kind == 0 {
                                none += 1.0;
                                fa += s.rules[r][k].is_some() as usize as f64;
                            } else if credit(r, s.kind) {
                                own += 1.0;
                                hit += s.rules[r][k].map(|t| t >= s.start && t <= s.start + WINDOW).unwrap_or(false) as usize as f64;
                            }
                        }
                        hit / own.max(1.0) - 2.0 * fa / none.max(1.0)
                    };
                    score(a).partial_cmp(&score(b)).unwrap()
                })
                .unwrap()
        })
        .collect()
}

/// (per-kind detection, false-alarm rate on "none").
fn score(test: &[Seen], pick: &[usize], credit: &dyn Fn(usize, usize) -> bool, kinds: usize) -> (Vec<f64>, f64) {
    let mut det = vec![(0.0f64, 0.0f64); kinds];
    let (mut fa, mut none) = (0.0f64, 0.0f64);
    for s in test {
        let (ok, alarm) = verdict(s, pick, credit);
        if s.kind == 0 {
            none += 1.0;
            fa += alarm as usize as f64;
        } else {
            det[s.kind].0 += ok as usize as f64;
            det[s.kind].1 += 1.0;
        }
    }
    (det.iter().map(|d| d.0 / d.1.max(1.0)).collect(), fa / none.max(1.0))
}

// ---------------------------------------------------------------- F1: data

const F1_KINDS: [&str; 6] = ["none", "drift", "step", "oscillation", "burst", "tipping"];
const F1_LEN: usize = 2000;
const WARM: usize = 500;

/// A data stream with a planted behaviour; `amp` scales the oscillation (O8 uses 0.5).
fn f1_stream(kind: usize, seed: u64, amp: f64) -> (Vec<f64>, usize) {
    let mut r = Rng(seed);
    let start = 1000 + (r.next() % 700) as usize;
    let mut x = vec![0.0; F1_LEN];
    let mut prev = 0.0;
    for t in 0..F1_LEN {
        let after = t >= start;
        let dt = t.saturating_sub(start) as f64;
        let phi = if kind == 5 && after { (0.5 + 0.49 * dt / 250.0).min(0.995) } else { 0.5 };
        let sigma = if kind == 4 && after && dt < 100.0 { 3.0 } else { 1.0 };
        let level = match kind {
            1 if after => 0.01 * dt,
            2 if after => 2.0,
            _ => 0.0,
        };
        let osc = if kind == 3 && after { amp * (0.3 * t as f64).sin() } else { 0.0 };
        let e = phi * prev + sigma * r.normal();
        prev = e;
        x[t] = level + osc + e;
    }
    (x, start)
}

/// Sensing for F1: surprise, trend/sd, resonator ratio, heat, slowing-down taus. Predictor learning
/// rate jittered by chaos when `chaos` is given; `precision` multiplies the surprise; `bands` sizes
/// the resonator bank.
struct F1Read {
    s: Vec<f64>,
    trend: Vec<f64>,
    osc: Vec<f64>,
    heat: Vec<f64>,
    tau: Vec<(f64, f64)>,
}

fn f1_sense(x: &[f64], chaos: Option<u64>, precision: f64, bands: usize) -> F1Read {
    let mut p = obs_predict::Predictor::new(0.1, 0.01, 0.02);
    let mut bank = obs_resonators::Bank::new(bands, 0.05, 1.5, 0.05, 0.01);
    let n = x.len();
    let (mut s, mut trend, mut osc, mut heat, mut tau) = (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![(0.0, 0.0); n]);
    let (mut acs, mut vars): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    for t in 0..n {
        if let Some(sd) = chaos {
            p.alpha = 0.1 * (1.0 + 0.5 * (qb_chaos::keyed_unit(sd.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ t as u64) - 0.5));
        }
        let (_, _, sur) = p.observe(x[t]);
        s[t] = precision * sur;
        trend[t] = p.trend / p.var.sqrt().max(1e-9);
        bank.drive(s[t]);
        let mut e = bank.energy.clone();
        e.sort_by(|a, b| a.partial_cmp(b).unwrap());
        osc[t] = e[e.len() - 1] / e[e.len() / 2].max(1e-12);
        let lo = t.saturating_sub(30);
        let w = &s[lo..=t];
        let m = w.iter().sum::<f64>() / w.len() as f64;
        heat[t] = w.iter().map(|v| (v - m).powi(2)).sum::<f64>() / w.len() as f64;
        if t >= 100 && t % 10 == 0 {
            let (ac, v) = obs_slowing::indicators(&x[t - 100..t]);
            acs.push(ac);
            vars.push(v);
            if acs.len() > 20 {
                acs.remove(0);
                vars.remove(0);
            }
        }
        tau[t] = if acs.len() >= 20 { (obs_slowing::kendall(&acs), obs_slowing::kendall(&vars)) } else { (0.0, 0.0) };
    }
    F1Read { s, trend, osc, heat, tau }
}

const F1_STEP: [f64; 4] = [3.0, 4.0, 5.0, 6.0];
const F1_DRIFT: [f64; 4] = [0.005, 0.0075, 0.01, 0.015];
const F1_OSC: [f64; 4] = [3.0, 5.0, 8.0, 12.0];
const F1_BURST: [f64; 4] = [2.0, 3.0, 4.0, 6.0];
const F1_TIP: [f64; 4] = [0.3, 0.5, 0.7, 0.85];

/// Perception rules for F1, in rule order: drift, step, oscillation, burst, tipping (rule r names
/// kind r + 1).
fn f1_perceive(rd: &F1Read) -> Vec<Fires> {
    let n = rd.s.len();
    let drift: Fires = F1_DRIFT.iter().map(|&th| first_run(n, WARM, 30, &|t| rd.trend[t].abs() > th)).collect();
    let step: Fires = F1_STEP.iter().map(|&th| first_run(n, WARM, 1, &|t| t >= 2 && rd.s[t].abs() > th && rd.s[t - 1].abs() > th && rd.s[t - 2].abs() > th && rd.s[t].signum() == rd.s[t - 1].signum() && rd.s[t - 1].signum() == rd.s[t - 2].signum())).collect();
    let osc: Fires = F1_OSC.iter().map(|&th| first_run(n, WARM, 20, &|t| rd.osc[t] > th)).collect();
    let burst: Fires = F1_BURST.iter().map(|&th| first_run(n, WARM, 1, &|t| rd.heat[t] > th)).collect();
    let tip: Fires = F1_TIP.iter().map(|&th| first_run(n, WARM, 1, &|t| rd.tau[t].0 > th && rd.tau[t].1 > th)).collect();
    vec![drift, step, osc, burst, tip]
}

const BAR_CUSUM: [f64; 4] = [5.0, 10.0, 20.0, 40.0];
const BAR_Z: [f64; 4] = [3.0, 3.5, 4.0, 5.0];
const BAR_FFT: [f64; 4] = [10.0, 20.0, 40.0, 80.0];
const BAR_VAR: [f64; 4] = [1.5, 2.0, 3.0, 4.0];

/// The status-quo bars for F1, in rule order: CUSUM (names "level": drift or step), z-score (burst),
/// FFT peak (oscillation), variance threshold (tipping).
fn f1_bars(x: &[f64]) -> Vec<Fires> {
    let n = x.len();
    let base = &x[..WARM];
    let mu = base.iter().sum::<f64>() / WARM as f64;
    let sd = (base.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / WARM as f64).sqrt();
    let cusum: Fires = BAR_CUSUM
        .iter()
        .map(|&h| {
            let (mut sp, mut sn) = (0.0f64, 0.0f64);
            for t in WARM..n {
                let z = (x[t] - mu) / sd;
                sp = (sp + z - 0.5).max(0.0);
                sn = (sn - z - 0.5).max(0.0);
                if sp > h || sn > h {
                    return Some(t);
                }
            }
            None
        })
        .collect();
    let zs: Fires = BAR_Z
        .iter()
        .map(|&th| {
            let mut hits: Vec<usize> = Vec::new();
            for t in WARM..n {
                let w = &x[t - 50..t];
                let m = w.iter().sum::<f64>() / 50.0;
                if ((x[t] - m) / sd).abs() > th {
                    hits.push(t);
                    hits.retain(|&h| h + 20 > t);
                    if hits.len() >= 3 {
                        return Some(t);
                    }
                }
            }
            None
        })
        .collect();
    let fft: Fires = BAR_FFT
        .iter()
        .map(|&th| {
            for t in (WARM..n).step_by(32) {
                if t < 256 {
                    continue;
                }
                let w = &x[t - 256..t];
                let m = w.iter().sum::<f64>() / 256.0;
                let pw: Vec<f64> = (1..128)
                    .map(|k| {
                        let (mut re, mut im) = (0.0, 0.0);
                        for (i, v) in w.iter().enumerate() {
                            let a = 2.0 * PI * k as f64 * i as f64 / 256.0;
                            re += (v - m) * a.cos();
                            im += (v - m) * a.sin();
                        }
                        re * re + im * im
                    })
                    .collect();
                let mx = pw.iter().cloned().fold(0.0, f64::max);
                let mean = pw.iter().sum::<f64>() / pw.len() as f64;
                if mx / mean > th {
                    return Some(t);
                }
            }
            None
        })
        .collect();
    let var: Fires = BAR_VAR
        .iter()
        .map(|&th| {
            for t in (WARM..n).step_by(10) {
                let w = &x[t - 100..t];
                let m = w.iter().sum::<f64>() / 100.0;
                if w.iter().map(|v| (v - m).powi(2)).sum::<f64>() / 100.0 > th * sd * sd {
                    return Some(t);
                }
            }
            None
        })
        .collect();
    vec![cusum, zs, fft, var]
}

fn f1_credit_obs(r: usize, kind: usize) -> bool {
    r + 1 == kind
}
fn f1_credit_bar(r: usize, kind: usize) -> bool {
    match r {
        0 => kind == 1 || kind == 2,
        1 => kind == 4,
        2 => kind == 3,
        3 => kind == 5,
        _ => false,
    }
}

const PER_KIND: usize = 50;

fn f1_seen(seeds: std::ops::Range<u64>, bars: bool, chaos: bool, precision: f64, bands: usize, amp: f64) -> Vec<Seen> {
    let jobs: Vec<(usize, u64)> = seeds.flat_map(|sd| (0..6).flat_map(move |k| (0..PER_KIND).map(move |i| (k, sd * 100_000 + (k * 1000 + i) as u64)))).collect();
    std::thread::scope(|sc| {
        let chunks: Vec<_> = jobs
            .chunks(jobs.len().div_ceil(8))
            .map(|ch| {
                sc.spawn(move || {
                    ch.iter()
                        .map(|&(k, s)| {
                            let (x, start) = f1_stream(k, s, amp);
                            let rules = if bars { f1_bars(&x) } else { f1_perceive(&f1_sense(&x, if chaos { Some(s) } else { None }, precision, bands)) };
                            Seen { kind: k, start, rules }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        chunks.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

// ---------------------------------------------------------------- F2: written ideas

const F2_KINDS: [&str; 5] = ["none", "repetition", "novelty", "topic drift", "contradiction"];
const F2_LEN: usize = 1000;
const F2_WARM: usize = 300;
const DI: usize = 512;

fn phasors(seed: u64, d: usize) -> Vec<C> {
    let mut r = Rng(seed);
    (0..d).map(|_| C::polar(1.0, 2.0 * PI * r.unit())).collect()
}

/// (symbol ids, negation flags, start). Topics: symbol / 32.
fn f2_stream(kind: usize, seed: u64) -> (Vec<usize>, Vec<bool>, usize) {
    let mut r = Rng(seed);
    let start = 500 + (r.next() % 350) as usize;
    let cycle: Vec<usize> = (0..4).map(|_| (r.next() % 32) as usize).collect();
    let (mut ids, mut neg) = (Vec::new(), Vec::new());
    let mut seen: Vec<usize> = Vec::new();
    for t in 0..F2_LEN {
        let dt = t as i64 - start as i64;
        let (id, n) = match kind {
            1 if (0..100).contains(&dt) => (cycle[(dt % 4) as usize], false),
            2 if (0..20).contains(&dt) => (5 * 32 + (r.next() % 32) as usize, false),
            3 if dt >= 0 => {
                let p1 = (dt as f64 / 100.0).min(1.0);
                (if r.unit() < p1 { 32 + (r.next() % 32) as usize } else { (r.next() % 32) as usize }, false)
            }
            4 if dt == 0 => (seen[(r.next() % seen.len() as u64) as usize], true),
            _ => ((r.next() % 32) as usize, false),
        };
        seen.push(id);
        ids.push(id);
        neg.push(n);
    }
    (ids, neg, start)
}

const F2_NOV: [f64; 4] = [0.02, 0.04, 0.06, 0.08];
const F2_REP: [f64; 4] = [0.3, 0.4, 0.5, 0.6];
const F2_DRIFT: [f64; 4] = [0.5, 0.6, 0.7, 0.8];
const F2_CON: [f64; 4] = [0.1, 0.15, 0.2, 0.3];

/// Sensing + perception for F2 on phase codes only (no symbol ids): rule order repetition, novelty,
/// topic drift, contradiction.
fn f2_perceive(ids: &[usize], neg: &[bool], seed: u64) -> Vec<Fires> {
    let codes: Vec<Vec<C>> = (0..256).map(|s| phasors(seed ^ (s as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15), DI)).collect();
    let not = phasors(seed ^ 0x0A07, DI);
    let (mut lt, mut st, mut rc) = (Memory::new(DI), Memory::new(DI), Memory::new(DI));
    let n = ids.len();
    let (mut olt, mut ost, mut onot, mut drift) = (vec![1.0; n], vec![0.0; n], vec![0.0; n], vec![1.0; n]);
    for t in 0..n {
        let c: Vec<C> = if neg[t] { codes[ids[t]].iter().zip(&not).map(|(a, b)| a.mul(*b)).collect() } else { codes[ids[t]].clone() };
        if t > 0 {
            olt[t] = lt.overlap(&c);
            ost[t] = st.overlap(&c);
            let un: Vec<C> = c.iter().zip(&not).map(|(a, b)| a.mul(C::new(b.re, -b.im))).collect();
            onot[t] = lt.overlap(&un);
            // Topic drift: the recent memory against the long-term one.
            let (mut re, mut n1, mut n2) = (0.0, 0.0, 0.0);
            for (a, b) in rc.sum.iter().zip(&lt.sum) {
                re += a.re * b.re + a.im * b.im;
                n1 += a.norm2();
                n2 += b.norm2();
            }
            drift[t] = if n1 > 0.0 && n2 > 0.0 { re / (n1 * n2).sqrt() } else { 1.0 };
        }
        lt.add(&c, 1.0);
        st.fade(0.9);
        st.add(&c, 1.0);
        rc.fade(0.97);
        rc.add(&c, 1.0);
    }
    vec![
        F2_REP.iter().map(|&th| first_run(n, F2_WARM, 8, &|t| ost[t] > th)).collect(),
        F2_NOV.iter().map(|&th| first_run(n, F2_WARM, 3, &|t| olt[t] < th)).collect(),
        F2_DRIFT.iter().map(|&th| first_run(n, F2_WARM, 10, &|t| drift[t] < th)).collect(),
        F2_CON.iter().map(|&th| first_run(n, F2_WARM, 1, &|t| onot[t] > th)).collect(),
    ]
}

const BAR_KL: [f64; 4] = [0.5, 1.0, 1.5, 2.0];

/// Bars with symbol ids (the status quo knows ids and a "not" flag).
fn f2_bars(ids: &[usize], neg: &[bool]) -> Vec<Fires> {
    let n = ids.len();
    let rep = first_run(n, F2_WARM, 8, &|t| t >= 4 && ids[t] == ids[t - 4]);
    let nov = first_run(n, F2_WARM, 3, &|t| !ids[..t].contains(&ids[t]));
    let kl: Fires = BAR_KL
        .iter()
        .map(|&th| {
            first_run(n, F2_WARM, 1, &|t| {
                if t < 200 {
                    return false;
                }
                let (mut a, mut b) = (vec![0.5f64; 256], vec![0.5f64; 256]);
                for &i in &ids[t - 50..t] {
                    a[i] += 1.0;
                }
                for &i in &ids[..t - 50] {
                    b[i] += 1.0;
                }
                let (sa, sb): (f64, f64) = (a.iter().sum(), b.iter().sum());
                a.iter().zip(&b).map(|(x, y)| (x / sa) * ((x / sa) / (y / sb)).ln()).sum::<f64>() > th
            })
        })
        .collect();
    let con = first_run(n, F2_WARM, 1, &|t| neg[t] && ids[..t].contains(&ids[t]));
    vec![vec![rep; 4], vec![nov; 4], kl, vec![con; 4]]
}

fn f2_seen(seeds: std::ops::Range<u64>, bars: bool) -> Vec<Seen> {
    let jobs: Vec<(usize, u64)> = seeds.flat_map(|sd| (0..5).flat_map(move |k| (0..PER_KIND).map(move |i| (k, sd * 100_000 + 7_000 + (k * 1000 + i) as u64)))).collect();
    std::thread::scope(|sc| {
        let chunks: Vec<_> = jobs
            .chunks(jobs.len().div_ceil(8))
            .map(|ch| {
                sc.spawn(move || {
                    ch.iter()
                        .map(|&(k, s)| {
                            let (ids, neg, start) = f2_stream(k, s);
                            let rules = if bars { f2_bars(&ids, &neg) } else { f2_perceive(&ids, &neg, s) };
                            Seen { kind: k, start, rules }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        chunks.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

// ---------------------------------------------------------------- F3: the brain's thoughts

const F3_KINDS: [&str; 5] = ["none", "settling", "flip-flop", "collapse", "false certainty"];
const F3_LEN: usize = 1000;
const ANSWERS: usize = 8;

/// Probability vectors over 8 answers (answer 0 is true).
fn f3_stream(kind: usize, seed: u64) -> (Vec<Vec<f64>>, usize) {
    let mut r = Rng(seed);
    let start = 500 + (r.next() % 350) as usize;
    let out = (0..F3_LEN)
        .map(|t| {
            let dt = t as f64 - start as f64;
            let mut l = vec![0.0; ANSWERS];
            l[0] = 1.5;
            match kind {
                1 if dt >= 0.0 => l[0] = 1.5 + 3.5 * (dt / 200.0).min(1.0),
                2 if (0.0..150.0).contains(&dt) => {
                    let a = ((dt as usize) / 8) % 2;
                    l[0] = 0.0;
                    l[1 + a] = 3.0;
                }
                3 if dt >= 0.0 => l[0] = 1.5 * (1.0 - (dt / 100.0).min(1.0)),
                4 if (0.0..60.0).contains(&dt) => {
                    l[0] = 0.0;
                    l[3] = 5.0;
                }
                4 if dt >= 60.0 => l[0] = 3.0,
                _ => {}
            }
            for v in l.iter_mut() {
                *v += 0.3 * r.normal();
            }
            let m = l.iter().cloned().fold(f64::MIN, f64::max);
            let e: Vec<f64> = l.iter().map(|v| (v - m).exp()).collect();
            let s: f64 = e.iter().sum();
            e.iter().map(|v| v / s).collect()
        })
        .collect();
    (out, start)
}

fn entropy(p: &[f64]) -> f64 {
    -p.iter().filter(|&&x| x > 0.0).map(|x| x * x.ln()).sum::<f64>()
}

fn top(p: &[f64]) -> usize {
    (0..p.len()).max_by(|&a, &b| p[a].partial_cmp(&p[b]).unwrap()).unwrap()
}

const F3_FLIP: [f64; 4] = [4.0, 6.0, 8.0, 10.0];
const F3_TREND: [f64; 4] = [0.002, 0.004, 0.008, 0.016];
const F3_DROP: [f64; 4] = [3.0, 4.0, 5.0, 6.0];

/// Perception for F3: rule order settling, flip-flop, collapse, false certainty.
fn f3_perceive(ps: &[Vec<f64>]) -> Vec<Fires> {
    let n = ps.len();
    let h: Vec<f64> = ps.iter().map(|p| entropy(p)).collect();
    let tops: Vec<usize> = ps.iter().map(|p| top(p)).collect();
    let mut pred = obs_predict::Predictor::new(0.1, 0.02, 0.02);
    let (mut sh, mut th) = (vec![0.0; n], vec![0.0; n]);
    for t in 0..n {
        sh[t] = pred.observe(h[t]).2;
        th[t] = pred.trend;
    }
    let changes = |t: usize| -> usize { (t.saturating_sub(40) + 1..=t).filter(|&i| tops[i] != tops[i - 1]).count() };
    let base_h: f64 = h[..WARM.min(n) / 2].iter().sum::<f64>() / (WARM.min(n) / 2) as f64;
    vec![
        F3_TREND.iter().map(|&k| first_run(n, F2_WARM, 30, &|t| th[t] < -k && changes(t) == 0)).collect(),
        F3_FLIP.iter().map(|&k| first_run(n, F2_WARM, 1, &|t| changes(t) as f64 >= k)).collect(),
        F3_TREND.iter().map(|&k| first_run(n, F2_WARM, 30, &|t| th[t] > k && h[t] > base_h)).collect(),
        F3_DROP
            .iter()
            .map(|&k| {
                let mut armed: Option<(usize, usize)> = None;
                for t in F2_WARM..n {
                    if sh[t] < -k {
                        armed = Some((t, tops[t]));
                    }
                    if let Some((t0, a)) = armed {
                        if t > t0 + 150 {
                            armed = None;
                        } else if tops[t] != a && t > t0 + 5 {
                            return Some(t);
                        }
                    }
                }
                None
            })
            .collect(),
    ]
}

const BAR_HI: [f64; 4] = [1.7, 1.8, 1.9, 2.0];
const BAR_LO: [f64; 4] = [0.3, 0.5, 0.7, 0.9];

/// Bars for F3: low entropy (settling), top changes in 40 (flip-flop), high entropy (collapse), low
/// entropy then a top change (false certainty).
fn f3_bars(ps: &[Vec<f64>]) -> Vec<Fires> {
    let n = ps.len();
    let h: Vec<f64> = ps.iter().map(|p| entropy(p)).collect();
    let tops: Vec<usize> = ps.iter().map(|p| top(p)).collect();
    let changes = |t: usize| -> usize { (t.saturating_sub(40) + 1..=t).filter(|&i| tops[i] != tops[i - 1]).count() };
    vec![
        BAR_LO.iter().map(|&k| first_run(n, F2_WARM, 20, &|t| h[t] < k)).collect(),
        F3_FLIP.iter().map(|&k| first_run(n, F2_WARM, 1, &|t| changes(t) as f64 >= k)).collect(),
        BAR_HI.iter().map(|&k| first_run(n, F2_WARM, 20, &|t| h[t] > k)).collect(),
        BAR_LO
            .iter()
            .map(|&k| {
                let mut armed: Option<(usize, usize)> = None;
                for t in F2_WARM..n {
                    if h[t] < k && armed.is_none() {
                        armed = Some((t, tops[t]));
                    }
                    if let Some((t0, a)) = armed {
                        if t > t0 + 150 {
                            armed = None;
                        } else if tops[t] != a {
                            return Some(t);
                        }
                    }
                }
                None
            })
            .collect(),
    ]
}

fn f3_seen(seeds: std::ops::Range<u64>, bars: bool) -> Vec<Seen> {
    let jobs: Vec<(usize, u64)> = seeds.flat_map(|sd| (0..5).flat_map(move |k| (0..PER_KIND).map(move |i| (k, sd * 100_000 + 13_000 + (k * 1000 + i) as u64)))).collect();
    jobs.iter()
        .map(|&(k, s)| {
            let (ps, start) = f3_stream(k, s);
            Seen { kind: k, start, rules: if bars { f3_bars(&ps) } else { f3_perceive(&ps) } }
        })
        .collect()
}

fn f_credit(r: usize, kind: usize) -> bool {
    r + 1 == kind
}

// ---------------------------------------------------------------- O5, O6

/// O5: a benign oscillation from 700 to 2700, a step at 2800. (oscillation alarms, step detected).
fn habituation(seed: u64, habituate: bool, th_osc: f64, th_step: f64) -> (usize, bool) {
    let mut r = Rng(seed);
    let n = 3200;
    let mut prev = 0.0;
    let x: Vec<f64> = (0..n)
        .map(|t| {
            let e = 0.5 * prev + r.normal();
            prev = e;
            e + if (700..2700).contains(&t) { (0.3 * t as f64).sin() } else { 0.0 } + if t >= 2800 { 2.0 } else { 0.0 }
        })
        .collect();
    let rd = f1_sense(&x, None, 1.0, 16);
    let mut h = obs_habituate::Habituation::new(2, 0.1, 0.0005);
    let (mut alarms, mut step) = (0, false);
    for t in WARM..n {
        h.step();
        let resp = if habituate { h.respond(0, rd.osc[t]) } else { rd.osc[t] };
        if (700..2700).contains(&t) && resp > th_osc {
            alarms += 1;
        }
        if (2800..3000).contains(&t) && t >= 2 && rd.s[t].abs() > th_step && rd.s[t - 1].abs() > th_step && rd.s[t - 2].abs() > th_step {
            step = true;
        }
    }
    (alarms, step)
}

/// O6: x(t+1) = a·x + noise, a = 0.3 + load, load rising 0.0008 per step; collapse at a >= 0.99.
/// mode 0 no reaction, 1 the observer's Le Chatelier (slowing-down evidence), 2 the variance bar.
/// (collapsed, mean load over the last 500 steps).
fn reaction(seed: u64, mode: u8, th: f64, var_th: f64) -> (bool, f64) {
    let mut r = Rng(seed ^ 0x2EAC);
    let (mut load, mut x) = (0.0f64, 0.0f64);
    let mut hist: Vec<f64> = Vec::new();
    let (mut acs, mut vars): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    let mut base_var = 0.0;
    let mut kept = 0.0;
    for t in 0..1500 {
        load += 0.0008;
        let a = 0.3 + load;
        if a >= 0.99 {
            return (true, 0.0);
        }
        x = a * x + r.normal();
        hist.push(x);
        if t >= 100 && t % 10 == 0 {
            let (ac, v) = obs_slowing::indicators(&hist[t - 100..t]);
            if t == 200 {
                base_var = v;
            }
            acs.push(ac);
            vars.push(v);
            if acs.len() > 20 {
                acs.remove(0);
                vars.remove(0);
            }
            match mode {
                1 if acs.len() >= 20 => {
                    let e = obs_slowing::kendall(&acs).min(obs_slowing::kendall(&vars));
                    if e > th {
                        load -= 0.05 * e; // Le Chatelier: oppose in proportion to the evidence
                    }
                }
                2 if base_var > 0.0 && v > var_th * base_var => load -= 0.05,
                _ => {}
            }
            load = load.max(0.0);
        }
        if t >= 1000 {
            kept += load;
        }
    }
    (false, kept / 500.0)
}

// ---------------------------------------------------------------- main

fn fmt(name: &str, det: &[f64], fa: f64, kinds: &[&str]) -> String {
    let parts: Vec<String> = (1..kinds.len()).map(|k| format!("{} {:.3}", kinds[k], det[k])).collect();
    format!("  {name:<34} {}; false alarms {fa:.3}", parts.join(", "))
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("OBS-1: an observer (sense -> perceive -> react) on synthetic streams; {PER_KIND} streams per behaviour per seed");
    // F1.
    let (od, bd) = (f1_seen(0..4, false, false, 1.0, 16, 1.0), f1_seen(0..4, true, false, 1.0, 16, 1.0));
    let (op, bp) = (choose(&od, &f1_credit_obs), choose(&bd, &f1_credit_bar));
    let (ot, bt) = (f1_seen(10..18, false, false, 1.0, 16, 1.0), f1_seen(10..18, true, false, 1.0, 16, 1.0));
    let (o1, ofa) = score(&ot, &op, &f1_credit_obs, 6);
    let (b1, bfa) = score(&bt, &bp, &f1_credit_bar, 6);
    println!("\nF1 data (thresholds chosen on dev: observer {op:?}, bars {bp:?}):");
    println!("{}", fmt("observer", &o1, ofa, &F1_KINDS));
    println!("{}", fmt("status-quo bars", &b1, bfa, &F1_KINDS));
    // O2: early warning before tipping (start + 250).
    let early = |set: &[Seen], pick: &[usize], r: usize| -> f64 {
        let tips: Vec<&Seen> = set.iter().filter(|s| s.kind == 5).collect();
        tips.iter().filter(|s| s.rules[r][pick[r]].map(|t| t >= s.start && t < s.start + 250).unwrap_or(false)).count() as f64 / tips.len() as f64
    };
    let (oe, be) = (early(&ot, &op, 4), early(&bt, &bp, 3));
    println!("  early warning before the tip: observer (critical slowing down) {oe:.3}, variance bar {be:.3}");
    // F2.
    let (o2d, b2d) = (f2_seen(0..4, false), f2_seen(0..4, true));
    let (o2p, b2p) = (choose(&o2d, &f_credit), choose(&b2d, &f_credit));
    let (o2, o2fa) = score(&f2_seen(10..18, false), &o2p, &f_credit, 5);
    let (b2, b2fa) = score(&f2_seen(10..18, true), &b2p, &f_credit, 5);
    println!("\nF2 written ideas (observer on phase codes only; bars see symbol ids):");
    println!("{}", fmt("observer", &o2, o2fa, &F2_KINDS));
    println!("{}", fmt("status-quo bars (ids)", &b2, b2fa, &F2_KINDS));
    // F3.
    let (o3d, b3d) = (f3_seen(0..4, false), f3_seen(0..4, true));
    let (o3p, b3p) = (choose(&o3d, &f_credit), choose(&b3d, &f_credit));
    let o3t = f3_seen(10..18, false);
    let (o3, o3fa) = score(&o3t, &o3p, &f_credit, 5);
    let (b3, b3fa) = score(&f3_seen(10..18, true), &b3p, &f_credit, 5);
    println!("\nF3 the brain's thoughts:");
    println!("{}", fmt("observer", &o3, o3fa, &F3_KINDS));
    println!("{}", fmt("status-quo bars", &b3, b3fa, &F3_KINDS));
    let settle: Vec<&Seen> = o3t.iter().filter(|s| s.kind == 1).collect();
    let healthy = settle.iter().filter(|s| (1..4).all(|r| s.rules[r][o3p[r]].is_none())).count() as f64 / settle.len() as f64;
    println!("  settling streams not flagged as a problem: {healthy:.3}");
    // O5 habituation.
    let (tho, ths) = (F1_OSC[op[2]], F1_STEP[op[1]]);
    let hab: Vec<(usize, bool)> = (10..18u64).flat_map(|s| (0..10u64).map(move |i| s * 1000 + i)).map(|s| habituation(s, true, tho, ths)).collect();
    let unh: Vec<(usize, bool)> = (10..18u64).flat_map(|s| (0..10u64).map(move |i| s * 1000 + i)).map(|s| habituation(s, false, tho, ths)).collect();
    let (ha, ua) = (hab.iter().map(|x| x.0).sum::<usize>() as f64, unh.iter().map(|x| x.0).sum::<usize>() as f64);
    let hstep = hab.iter().filter(|x| x.1).count() as f64 / hab.len() as f64;
    println!("\nO5 habituation: benign-oscillation alarms habituated {ha:.0} vs unhabituated {ua:.0}; later step detected {hstep:.3}");
    // O6 reaction.
    let ths_dev: Vec<f64> = vec![0.3, 0.5, 0.7];
    let vth_dev: Vec<f64> = vec![1.5, 2.0, 3.0];
    let runs = |mode: u8, th: f64, vt: f64, seeds: std::ops::Range<u64>| -> (f64, f64) {
        let v: Vec<(bool, f64)> = seeds.flat_map(|s| (0..25u64).map(move |i| s * 1000 + i)).map(|s| reaction(s, mode, th, vt)).collect();
        (v.iter().filter(|x| x.0).count() as f64 / v.len() as f64, mean(&v.iter().map(|x| x.1).collect::<Vec<_>>()))
    };
    let pick_th = |mode: u8, grid: &[f64]| -> f64 {
        *grid.iter().max_by(|a, b| {
            let (ca, la) = if mode == 1 { runs(1, **a, 0.0, 0..4) } else { runs(2, 0.0, **a, 0..4) };
            let (cb, lb) = if mode == 1 { runs(1, **b, 0.0, 0..4) } else { runs(2, 0.0, **b, 0..4) };
            (la - 10.0 * ca).partial_cmp(&(lb - 10.0 * cb)).unwrap()
        }).unwrap()
    };
    let (th1, vt2) = (pick_th(1, &ths_dev), pick_th(2, &vth_dev));
    let (n0, n1, n2) = (runs(0, 0.0, 0.0, 10..18), runs(1, th1, 0.0, 10..18), runs(2, 0.0, vt2, 10..18));
    println!("\nO6 reaction (collapse rate, mean load kept): none {:.3}/{:.3}; observer (Le Chatelier, th {th1}) {:.3}/{:.3}; variance bar (x{vt2}) {:.3}/{:.3}", n0.0, n0.1, n1.0, n1.1, n2.0, n2.1);
    // O7 chaos.
    let surprise_total = |chaos: bool| -> f64 {
        (0..4u64).flat_map(|sd| (0..6).flat_map(move |k| (0..10).map(move |i| sd * 100_000 + (k * 1000 + i) as u64))).map(|s| {
            let (x, _) = f1_stream((s % 6) as usize, s, 1.0);
            f1_sense(&x, if chaos { Some(s) } else { None }, 1.0, 16).s.iter().map(|v| v.abs()).sum::<f64>()
        }).sum()
    };
    let (sp, sc) = (surprise_total(false), surprise_total(true));
    let chaos_kept = if sc < sp {
        let (oc, _) = score(&f1_seen(10..18, false, true, 1.0, 16, 1.0), &op, &f1_credit_obs, 6);
        mean(&oc[1..]) > mean(&o1[1..])
    } else {
        false
    };
    println!("\nO7 chaos on the predictor's rate: dev total surprise {sp:.0} -> {sc:.0}; kept {chaos_kept}");
    // O8 amplification at low-amplitude oscillation.
    let lo = |prec: f64, bands: usize| -> (f64, f64) {
        let set = f1_seen(10..18, false, false, prec, bands, 0.5);
        let (d, fa) = score(&set, &op, &f1_credit_obs, 6);
        (d[3], fa)
    };
    let (l0, l1) = (lo(1.0, 16), lo(2.0, 32));
    println!("O8 low-amplitude oscillation (0.5): plain {:.3} (false alarms {:.3}); amplified (precision x2, 32 bands) {:.3} ({:.3})", l0.0, l0.1, l1.0, l1.1);
    let v = [
        (format!("O1 F1 every behaviour >= 0.85 ({:.3?}), false alarms <= 0.05 ({ofa:.3}), and mean detection >= bars + 0.1 ({:.3} vs {:.3})", &o1[1..], mean(&o1[1..]), mean(&b1[1..])), o1[1..].iter().all(|&x| x >= 0.85) && ofa <= 0.05 && mean(&o1[1..]) >= mean(&b1[1..]) + 0.1),
        (format!("O2 early warning before the tip >= 0.8 ({oe:.3}); variance bar <= 0.5 ({be:.3})"), oe >= 0.8 && be <= 0.5),
        (format!("O3 F2 >= 0.85 ({:.3?}), false alarms <= 0.05 ({o2fa:.3})", &o2[1..]), o2[1..].iter().all(|&x| x >= 0.85) && o2fa <= 0.05),
        (format!("O4 F3 >= 0.8 ({:.3?}); settling not flagged >= 0.9 ({healthy:.3})", &o3[1..]), o3[1..].iter().all(|&x| x >= 0.8) && healthy >= 0.9),
        (format!("O5 habituation <= 1/4 of the alarms ({ha:.0} vs {ua:.0}) and a later step >= 0.85 ({hstep:.3})"), ha <= 0.25 * ua && hstep >= 0.85),
        (format!("O6 reaction avoids collapse >= 0.95 ({:.3}) with >= 10% more load than the bar ({:.3} vs {:.3}); no reaction collapses >= 0.9 ({:.3})", 1.0 - n1.0, n1.1, n2.1, n0.0), 1.0 - n1.0 >= 0.95 && n1.1 >= 1.1 * n2.1 && n0.0 >= 0.9),
        ("O7 chaos not kept".to_string(), !chaos_kept),
        (format!("O8 amplification +0.05 at low amplitude ({:.3} vs {:.3}) at <= 0.02 false-alarm cost ({:+.3})", l1.0, l0.0, l1.1 - l0.1), l1.0 >= l0.0 + 0.05 && l1.1 - l0.1 <= 0.02),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
    let _ = (b2fa, b3fa, b2, b3);
}
