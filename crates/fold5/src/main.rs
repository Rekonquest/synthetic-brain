//! FOLD-5 orchestrator (docs/FOLD-5-PRECOMMIT.md): folding outside the thermal state. Sixteen
//! regions, F memories folded into each, recall by cue, in the same 422,400 bytes as the 32-band
//! thermal medium:
//! - the phase store (the quantum base): each region one `qb-amplitude::State` of D 1,650 complex
//!   amplitudes; keys are dense phasor codes; the reading is interference (`qb-overlap`);
//! - the fair classical comparator: D 3,300 non-negative reals per region; keys uniform in (0, 1);
//!   the reading is the centred correlation (`spec-correlate`).
//! Writing: 250 steps at gain g per memory, one memory after another. Heat: each step, each element
//! with probability q gets a kick of size ε (a random phase in the phase store, +ε in the classical
//! one). There is no flow, so writes and kicks commute; each element's kicks are drawn as a Bernoulli
//! process in time (geometric gaps), streamed in time order. Every setting is here.
//! `PART` = grid | heat | rounds (default: all).

use qb_amplitude::{State, C};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

const D_PHASE: usize = 1650;
const D_REAL: usize = 3300;
const GAIN: f64 = 0.02;
const Q: f64 = 0.02;
const EPS: f64 = 0.05;
const WRITE: usize = 250;
const THREADS: usize = 8;
const CHAOS_ALPHA: f64 = 0.1;
const CHAOS_VARIANTS: usize = 3;

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Phase,
    Classical,
}

#[derive(Clone, Copy, Debug)]
struct Setup {
    kind: Kind,
    f: usize,
    gain: f64,
    eps: f64,
    chaos: bool,
    /// The phase-locked reading (Re⟨c, a⟩) in place of the magnitude (FOLD-5's diagnosis).
    lock: bool,
}

impl Setup {
    fn new(kind: Kind, f: usize) -> Setup {
        Setup { kind, f, gain: GAIN, eps: EPS, chaos: false, lock: false }
    }
}

/// A memory key: phasors (phase store) or non-negative weights (classical).
enum Keys {
    Phase(Vec<Vec<C>>),
    Real(Vec<Vec<f64>>),
}

fn keys(kind: Kind, seed: u64, n: usize) -> Keys {
    let mut r = Rng(seed.wrapping_mul(0x5CA4_F01D) ^ 0xF05);
    match kind {
        Kind::Phase => Keys::Phase((0..n).map(|_| (0..D_PHASE).map(|_| C::polar(1.0, 2.0 * std::f64::consts::PI * r.unit())).collect()).collect()),
        Kind::Classical => Keys::Real((0..n).map(|_| (0..D_REAL).map(|_| r.unit()).collect()).collect()),
    }
}

/// One element's heat: a Bernoulli(q) process in time, streamed by geometric gaps.
struct Kicks {
    rng: Rng,
    next: u64,
}

impl Kicks {
    fn new(seed: u64, element: usize) -> Kicks {
        let mut k = Kicks { rng: Rng(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (element as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ 0x4B1C), next: 0 };
        k.next = k.gap();
        k
    }
    /// Steps from one kick to the next (≥ 1); the first kick lands at gap − 1.
    fn gap(&mut self) -> u64 {
        let u = 1.0 - self.rng.unit();
        ((u.ln() / (1.0 - Q).ln()).floor() as u64) + 1
    }
    /// The kicks at steps < t, each as a uniform for its phase (phase store) — call `f` per kick.
    fn advance(&mut self, t: u64, f: &mut dyn FnMut(f64)) {
        while self.next - 1 < t {
            let u = self.rng.unit();
            f(u);
            self.next += self.gap();
        }
    }
}

/// Each region's state.
enum Regions {
    Phase(Vec<State>),
    Real(Vec<Vec<f64>>),
}

/// The score of every key against every region at each read, as (keys × 16).
fn simulate(st: &Setup, ks: &Keys, reads: &[u64], seed: u64) -> Vec<Vec<[f64; 16]>> {
    let d = match st.kind {
        Kind::Phase => D_PHASE,
        Kind::Classical => D_REAL,
    };
    let mut regs = match st.kind {
        Kind::Phase => Regions::Phase((0..16).map(|_| State { a: vec![C::new(0.0, 0.0); d] }).collect()),
        Kind::Classical => Regions::Real(vec![vec![0.0; d]; 16]),
    };
    let mut kicks: Vec<Kicks> = (0..16 * d).map(|e| Kicks::new(seed, e)).collect();
    let eps = st.eps;
    let mut heat_to = |regs: &mut Regions, t: u64| {
        if eps == 0.0 {
            return;
        }
        for (e, k) in kicks.iter_mut().enumerate() {
            let (r, i) = (e / d, e % d);
            match regs {
                Regions::Phase(s) => k.advance(t, &mut |u| s[r].excite(i, C::polar(eps, 2.0 * std::f64::consts::PI * u))),
                Regions::Real(v) => k.advance(t, &mut |_| v[r][i] += eps),
            }
        }
    };
    let score = |regs: &Regions, m: usize, r: usize| -> f64 {
        match (regs, ks) {
            (Regions::Phase(s), Keys::Phase(k)) => if st.lock { qb_overlap_phase::overlap_phase(&k[m], &s[r].a) } else { qb_overlap::overlap(&k[m], &s[r].a) },
            (Regions::Real(v), Keys::Real(k)) => spec_correlate::correlate(&k[m], &v[r]),
            _ => unreachable!(),
        }
    };
    for r in 0..16 {
        for j in 0..st.f {
            let m = r * st.f + j;
            let t0 = (m * WRITE) as u64;
            heat_to(&mut regs, t0);
            // The written vector per step (plain), and chaos variants selected by readback.
            let factors: Vec<f64> = if st.chaos {
                let cands: Vec<Vec<f64>> = std::iter::once(vec![1.0; d])
                    .chain((0..CHAOS_VARIANTS).map(|v| {
                        (0..d)
                            .map(|i| {
                                let key = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (m as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ ((v * d + i) as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                                1.0 + CHAOS_ALPHA * (qb_chaos::keyed_unit(key) - 0.5)
                            })
                            .collect()
                    }))
                    .collect();
                let readback = |fac: &[f64]| -> f64 {
                    let total = WRITE as f64 * st.gain;
                    let trial = match (&regs, ks) {
                        (Regions::Phase(s), Keys::Phase(k)) => Regions::Phase(vec![State { a: s[r].a.iter().zip(&k[m]).zip(fac).map(|((a, c), f)| a.add(c.scale(total * f))).collect() }]),
                        (Regions::Real(v), Keys::Real(k)) => Regions::Real(vec![v[r].iter().zip(&k[m]).zip(fac).map(|((a, c), f)| a + c * total * f).collect()]),
                        _ => unreachable!(),
                    };
                    (r * st.f..=m).map(|mm| score(&trial, mm, 0)).sum::<f64>() / (j + 1) as f64
                };
                let base = readback(&cands[0]);
                let mut best = (base, 0);
                for (c, fac) in cands.iter().enumerate().skip(1) {
                    let s = readback(fac);
                    if s > best.0 {
                        best = (s, c);
                    }
                }
                cands[best.1].clone()
            } else {
                vec![1.0; d]
            };
            match (&mut regs, ks) {
                (Regions::Phase(s), Keys::Phase(k)) => {
                    let w: Vec<C> = k[m].iter().zip(&factors).map(|(c, f)| c.scale(st.gain * f)).collect();
                    for _ in 0..WRITE {
                        for (i, x) in w.iter().enumerate() {
                            s[r].excite(i, *x);
                        }
                    }
                }
                (Regions::Real(v), Keys::Real(k)) => {
                    let w: Vec<f64> = k[m].iter().zip(&factors).map(|(c, f)| c * st.gain * f).collect();
                    for _ in 0..WRITE {
                        for (a, x) in v[r].iter_mut().zip(&w) {
                            *a += x;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    let end = (16 * st.f * WRITE) as u64;
    let n = 16 * st.f;
    reads
        .iter()
        .map(|&d| {
            heat_to(&mut regs, end + d);
            (0..n).map(|m| std::array::from_fn(|r| score(&regs, m, r))).collect()
        })
        .collect()
}

/// (region top-1, set recall, oldest-half top-1, newest-half top-1) from a score matrix.
fn measure(sc: &[[f64; 16]], f: usize) -> (f64, f64) {
    let n = sc.len();
    let top = (0..n).filter(|&m| (0..16).max_by(|&a, &b| sc[m][a].partial_cmp(&sc[m][b]).unwrap()).unwrap() == m / f).count() as f64 / n as f64;
    let set = (0..16)
        .map(|r| {
            let mut rank: Vec<usize> = (0..n).collect();
            rank.sort_by(|&a, &b| sc[b][r].partial_cmp(&sc[a][r]).unwrap());
            rank[..f].iter().filter(|&&m| m / f == r).count() as f64 / f as f64
        })
        .sum::<f64>()
        / 16.0;
    (top, set)
}

/// Mean (top-1, set recall) per read for each setup over seeds, on THREADS workers.
fn run(setups: &[Setup], seeds: &[u64], reads: &[u64]) -> Vec<Vec<(f64, f64)>> {
    let jobs: Vec<(usize, u64)> = (0..setups.len()).flat_map(|i| seeds.iter().map(move |&s| (i, s))).collect();
    let next = AtomicUsize::new(0);
    let out: Vec<std::sync::Mutex<Option<Vec<(f64, f64)>>>> = jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..THREADS {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (i, sd) = jobs[j];
                let st = &setups[i];
                let ks = keys(st.kind, sd, 16 * st.f);
                let res = simulate(st, &ks, reads, sd);
                *out[j].lock().unwrap() = Some(res.iter().map(|m| measure(m, st.f)).collect());
            });
        }
    });
    let res: Vec<Vec<(f64, f64)>> = out.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect();
    let n = seeds.len() as f64;
    (0..setups.len())
        .map(|i| (0..reads.len()).map(|r| ((0..seeds.len()).map(|k| res[i * seeds.len() + k][r].0).sum::<f64>() / n, (0..seeds.len()).map(|k| res[i * seeds.len() + k][r].1).sum::<f64>() / n)).collect())
        .collect()
}

fn say(s: String) {
    println!("{s}");
    std::io::stdout().flush().unwrap();
}

const BYTES: usize = 16 * D_PHASE * 16;

fn main() {
    assert_eq!(BYTES, 16 * D_REAL * 8);
    let t0 = std::time::Instant::now();
    let part = std::env::var("PART").unwrap_or_default();
    let all = part.is_empty();
    let test: Vec<u64> = (10..18).collect();
    let dev: Vec<u64> = (0..4).collect();
    let mut verdicts: Vec<(String, bool)> = Vec::new();
    say(format!("FOLD-5: outside the thermal state, {BYTES} bytes (phase: 16 x {D_PHASE} complex; classical: 16 x {D_REAL} reals); test seeds 10-17"));

    let fs = [1usize, 4, 16, 64, 128, 256, 512];
    let mut cap = std::env::var("CAP").ok().and_then(|x| x.parse().ok()).unwrap_or(0usize);
    if all || part == "grid" {
        say("\nThe grid, read at drain t 250 (region top-1, set recall):".into());
        let setups: Vec<Setup> = [Kind::Phase, Kind::Classical].iter().flat_map(|&k| fs.iter().map(move |&f| Setup::new(k, f))).collect();
        let rows = run(&setups, &test, &[250]);
        for (st, r) in setups.iter().zip(&rows) {
            say(format!("  {:?} F {:3}: top-1 {:.3}, set {:.3}; {} memories, {} bytes each", st.kind, st.f, r[0].0, r[0].1, 16 * st.f, BYTES / (16 * st.f)));
        }
        let top = |k: usize| -> Vec<f64> { rows[k * fs.len()..(k + 1) * fs.len()].iter().map(|r| r[0].0).collect() };
        let capf = |t: &[f64]| fs.iter().zip(t).filter(|x| *x.1 >= 0.8).map(|x| *x.0).max().unwrap_or(0);
        cap = capf(&top(0));
        let ccap = capf(&top(1));
        say(format!("  capacity: phase {cap} per region ({} memories, {} bytes each); classical {ccap}", 16 * cap, BYTES / (16 * cap.max(1))));
        verdicts.push(("Q1 phase capacity >= 64 and <= 256 per region".into(), (64..=256).contains(&cap)));
        let gap = top(0).iter().zip(top(1)).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        verdicts.push((format!("Q2 the classical comparator ties within 0.05 at every F (largest gap {gap:.3})"), gap <= 0.05));
    }
    let depth = cap.max(1);

    if all || part == "heat" {
        say(format!("\nLifetime and heat, phase store at F {depth} (t 250, t 20000):"));
        let mults = [0.0, 0.5, 1.0, 2.0, 4.0];
        let setups: Vec<Setup> = mults.iter().map(|&m| Setup { eps: EPS * m, ..Setup::new(Kind::Phase, depth) }).collect();
        let rows = run(&setups, &test, &[250, 20_000]);
        for (m, r) in mults.iter().zip(&rows) {
            say(format!("  eps x {m}: t 250 top-1 {:.3} (set {:.3}); t 20000 top-1 {:.3} (set {:.3})", r[0].0, r[0].1, r[1].0, r[1].1));
        }
        let plain = &rows[2];
        verdicts.push(("Q3 no maintenance needed: t 20000 within 0.02 of t 250".into(), (plain[1].0 - plain[0].0).abs() <= 0.02));
        let fresh: Vec<f64> = rows.iter().map(|r| r[0].0).collect();
        verdicts.push(("Q4 heat: no inverted U (never rises with heat), x4 costs <= 0.05".into(), fresh.windows(2).all(|w| w[1] <= w[0]) && fresh[2] - fresh[4] <= 0.05));
    }

    if all || part == "rounds" {
        say(format!("\nRounds, phase store at F {depth} (dev seeds 0-3, then test; t 250 top-1, then set recall; plain wins ties):"));
        let base = Setup::new(Kind::Phase, depth);
        let arms = [("amplification: gain x 2", Setup { gain: 2.0 * GAIN, ..base }), ("chaos: 3 variants per write, alpha 0.1, readback selects", Setup { chaos: true, ..base })];
        let mut setups = vec![base];
        setups.extend(arms.iter().map(|a| a.1));
        let beats = |a: &(f64, f64), b: &(f64, f64)| a.0 > b.0 || (a.0 == b.0 && a.1 > b.1);
        let dv = run(&setups, &dev, &[250]);
        let tv = run(&setups, &test, &[250]);
        say(format!("  plain: dev {:.3} (set {:.3}), test {:.3} (set {:.3})", dv[0][0].0, dv[0][0].1, tv[0][0].0, tv[0][0].1));
        let mut kept = [false; 2];
        for (i, (name, _)) in arms.iter().enumerate() {
            let (d, t) = (&dv[i + 1][0], &tv[i + 1][0]);
            kept[i] = beats(d, &dv[0][0]) && beats(t, &tv[0][0]);
            say(format!("  {name}: dev {:.3} (set {:.3}), test {:.3} (set {:.3}); kept {}", d.0, d.1, t.0, t.1, kept[i]));
        }
        let dgain = tv[1][0].0 - tv[0][0].0;
        let dchaos = tv[2][0].0 - tv[0][0].0;
        verdicts.push((format!("Q5 amplification a tie (test change {dgain:+.3}, |.| <= 0.01)"), dgain.abs() <= 0.01));
        verdicts.push((format!("Q6 chaos not kept or <= 0.01 (test change {dchaos:+.3})"), !kept[1] || dchaos <= 0.01));
    }

    if part == "phaselock" {
        say("\nDiagnosis after the run (not a prediction): the phase-locked reading Re<c, a> against the magnitude |<c, a>|, t 250:".into());
        let ff = [128usize, 256, 512];
        let setups: Vec<Setup> = ff.iter().flat_map(|&f| [Setup::new(Kind::Phase, f), Setup { lock: true, ..Setup::new(Kind::Phase, f) }, Setup::new(Kind::Classical, f)]).collect();
        let rows = run(&setups, &test, &[250]);
        for (i, &f) in ff.iter().enumerate() {
            say(format!("  F {f:3}: magnitude {:.3} (set {:.3}), phase-locked {:.3} (set {:.3}), classical {:.3} (set {:.3})", rows[3 * i][0].0, rows[3 * i][0].1, rows[3 * i + 1][0].0, rows[3 * i + 1][0].1, rows[3 * i + 2][0].0, rows[3 * i + 2][0].1));
        }
    }

    say(String::new());
    for (name, ok) in &verdicts {
        say(format!("{} {name}", if *ok { "HOLDS " } else { "FAILS " }));
    }
    say(format!("\n{} of {} hold ({:.0} s)", verdicts.iter().filter(|x| x.1).count(), verdicts.len(), t0.elapsed().as_secs_f64()));
}
