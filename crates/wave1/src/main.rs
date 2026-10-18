//! WAVE-1 orchestrator (docs/WAVE-1-ANTENNA.md): atom antennas outside the thermodynamic state.
//! The medium is `wave-lattice` (the wave equation, leapfrog); receivers are `wave-resonator`s;
//! transmitters are forces this orchestrator applies. Checks W1–W8, then the amplification and chaos
//! rounds. Every setting lives here.

use qb_chaos::keyed_unit;
use thermo_bath::normal;
use thermo_lattice::Lattice;
use wave_lattice::Wave;
use wave_resonator::Resonator;

const DT: f64 = 0.5;
const SIDE: usize = 48;
const SPONGE: usize = 8;
const BIT_STEPS: usize = 400;
const SKIP: usize = 100;
const F1: f64 = 0.06;
const F2: f64 = 0.10;

fn hash_unit(key: u64) -> f64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 11) as f64 + 0.5) / (1u64 << 53) as f64
}

fn bits(seed: u64) -> Vec<u8> {
    (0..16).map(|k| if k < 4 { (1 - k % 2) as u8 } else { (hash_unit(seed * 1000 + k as u64) < 0.5) as u8 }).collect()
}

#[derive(Clone)]
struct Tx {
    site: usize,
    freq: f64,
    amp: f64,
    /// None = always on.
    bits: Option<Vec<u8>>,
}

#[derive(Clone, Copy)]
struct Setup {
    zeta: f64,
    temperature: f64,
    gamma_b: f64,
    chaos: f64,
    noise_seed: u64,
}

const PLAIN: Setup = Setup { zeta: 0.05, temperature: 0.0, gamma_b: 0.0, chaos: 0.0, noise_seed: 1 };

fn lattice() -> Lattice {
    Lattice::new(&[SIDE; 3])
}

fn site(x: i64, y: i64, z: i64) -> usize {
    let c = (SIDE / 2) as i64;
    ((c + x) as usize) + SIDE * ((c + y) as usize) + SIDE * SIDE * ((c + z) as usize)
}

/// The sponge: γ rising quadratically to 0.5 over the outer SPONGE sites on every face.
fn sponge(l: &Lattice) -> Vec<f64> {
    (0..l.e.len())
        .map(|s| {
            let depth = (0..3).map(|a| {
                let x = l.coord(s, a);
                let from_edge = x.min(SIDE - 1 - x);
                if from_edge < SPONGE { (SPONGE - from_edge) as f64 / SPONGE as f64 } else { 0.0 }
            });
            0.5 * depth.fold(0.0, f64::max).powi(2)
        })
        .collect()
}

/// Run transmitters and receivers for `steps`; the receivers' absorbed energy recorded at every step.
fn run(txs: &[Tx], rxs: &mut [(usize, Resonator)], steps: usize, s: &Setup) -> Vec<Vec<f64>> {
    let l = lattice();
    let mut w = Wave::new(&l);
    let c2: Vec<f64> = (0..l.e.len()).map(|i| if s.chaos > 0.0 { 1.0 + s.chaos * (keyed_unit(0x3A3E_0001 ^ (i as u64).wrapping_mul(0x9E37_79B9)) - 0.5) } else { 1.0 }).collect();
    let gamma: Vec<f64> = sponge(&l).iter().map(|g| g + s.gamma_b).collect();
    let mut force = vec![0.0; l.e.len()];
    let sigma = (2.0 * s.gamma_b * s.temperature * DT).sqrt();
    let mut trace = vec![Vec::with_capacity(steps); rxs.len()];
    for n in 0..steps {
        let t = n as f64 * DT;
        for f in force.iter_mut() {
            *f = 0.0;
        }
        for tx in txs {
            let on = tx.bits.as_ref().map(|b| b.get(n / BIT_STEPS).copied().unwrap_or(0) == 1).unwrap_or(true);
            if on {
                force[tx.site] += tx.amp * (2.0 * std::f64::consts::PI * tx.freq * t).sin();
            }
        }
        for (site, r) in rxs.iter() {
            force[*site] += r.force_on_medium(w.u[*site]);
        }
        w.kick(DT, &c2, &gamma, &force);
        if sigma > 0.0 {
            for (i, v) in w.v.iter_mut().enumerate() {
                let k = s.noise_seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (n as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (i as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                *v += sigma * normal(hash_unit(k), hash_unit(k ^ 0x5555_5555_5555_5555));
            }
        }
        for (k, (site, r)) in rxs.iter_mut().enumerate() {
            r.kick(DT, w.u[*site]);
            trace[k].push(r.absorbed);
        }
        w.drift(DT);
        for (_, r) in rxs.iter_mut() {
            r.drift(DT);
        }
    }
    trace
}

/// Energy per bit (skipping each bit's first SKIP steps), and decoding by the preamble's midpoint.
fn decode(trace: &[f64], b: &[u8]) -> (f64, Vec<f64>) {
    let e: Vec<f64> = (0..b.len()).map(|k| trace[(k + 1) * BIT_STEPS - 1] - trace[k * BIT_STEPS + SKIP]).collect();
    let thr = (e[0] + e[2]) / 4.0 + (e[1] + e[3]) / 4.0;
    let ok = (4..b.len()).filter(|&k| (e[k] > thr) == (b[k] == 1)).count();
    (ok as f64 / (b.len() - 4) as f64, e)
}

/// W7's two-channel link under a setup: (mean accuracy, accuracies, total absorbed by both receivers).
fn link(s: &Setup, amp: f64) -> (f64, [f64; 2], f64) {
    let (b1, b2) = (bits(1), bits(2));
    let txs = [Tx { site: site(-6, 0, 0), freq: F1, amp, bits: Some(b1.clone()) }, Tx { site: site(6, 0, 0), freq: F2, amp, bits: Some(b2.clone()) }];
    let mut rxs = [(site(0, 6, 0), Resonator::new(F1, s.zeta, 0.05)), (site(0, -6, 0), Resonator::new(F2, s.zeta, 0.05))];
    let tr = run(&txs, &mut rxs, 16 * BIT_STEPS, s);
    let (a1, _) = decode(&tr[0], &b1);
    let (a2, _) = decode(&tr[1], &b2);
    ((a1 + a2) / 2.0, [a1, a2], tr[0].last().unwrap() + tr[1].last().unwrap())
}

/// Diagnostic, added after the run (not a prediction): absorbed power at r 8 against the receiver's
/// nominal frequency, for the transmitter at F1, with the precommitted coupling and a weak one.
fn diagnose() {
    for kappa in [0.05, 0.005] {
        let mut row = Vec::new();
        for f in [0.045, 0.048, 0.052, 0.056, 0.060, 0.065, 0.070] {
            let mut rx = [(site(8, 0, 0), Resonator::new(f, 0.05, kappa))];
            let tr = run(&[Tx { site: site(0, 0, 0), freq: F1, amp: 1.0, bits: None }], &mut rx, 1600, &PLAIN);
            row.push(format!("{f}: {:.3e}", (tr[0][1599] - tr[0][799]) / (800.0 * DT)));
        }
        let shifted = ((2.0 * std::f64::consts::PI * F1).powi(2) - kappa).sqrt() / (2.0 * std::f64::consts::PI);
        println!("kappa {kappa}: {} | the nominal frequency that resonates at {F1} once coupled: {shifted:.4}", row.join("  "));
    }
}

fn main() {
    if std::env::var_os("WAVE_DIAG").is_some() {
        return diagnose();
    }
    let t0 = std::time::Instant::now();
    println!("WAVE-1: atom antennas in a wave medium; 3D side {SIDE}, sponge {SPONGE}, dt {DT}, c 1");
    // W1: the invariant, d = 1, 2, 3.
    let mut w1 = true;
    for dims in [vec![256usize], vec![64, 64], vec![24, 24, 24]] {
        let l = Lattice::new(&dims);
        let mut w = Wave::new(&l);
        for (i, u) in w.u.iter_mut().enumerate() {
            *u = hash_unit(i as u64) - 0.5;
        }
        let (c2, g, f) = (vec![1.0; l.e.len()], vec![0.0; l.e.len()], vec![0.0; l.e.len()]);
        let (mut h0, mut worst) = (None, 0.0f64);
        for _ in 0..2000 {
            let vm = w.v.clone();
            w.kick(DT, &c2, &g, &f);
            let h = 0.5 * vm.iter().zip(&w.v).map(|(a, b)| a * b).sum::<f64>() + w.strain(1.0);
            let h0v = *h0.get_or_insert(h);
            worst = worst.max(((h - h0v) / h0v).abs());
            w.drift(DT);
        }
        println!("W1 d={}: invariant drift {worst:.2e}", dims.len());
        w1 &= worst <= 1e-12;
    }
    // W2: dispersion in 1D.
    let mut w2 = true;
    for m in [1usize, 4, 8] {
        let l = Lattice::new(&[64]);
        let mut w = Wave::new(&l);
        let k = 2.0 * std::f64::consts::PI * m as f64 / 64.0;
        let basis: Vec<f64> = (0..64).map(|x| (k * x as f64).cos()).collect();
        w.u.clone_from(&basis);
        let (c2, g, f) = (vec![1.0; 64], vec![0.0; 64], vec![0.0; 64]);
        let norm: f64 = basis.iter().map(|b| b * b).sum();
        let mut a = Vec::new();
        for _ in 0..400 {
            a.push(w.u.iter().zip(&basis).map(|(u, b)| u * b).sum::<f64>() / norm);
            w.kick(DT, &c2, &g, &f);
            w.drift(DT);
        }
        let est: Vec<f64> = (1..a.len() - 1).filter(|&n| a[n].abs() > 0.5).map(|n| ((a[n + 1] + a[n - 1]) / (2.0 * a[n])).acos() / DT).collect();
        let omega = est.iter().sum::<f64>() / est.len() as f64;
        let want = (2.0 / DT) * (DT * (k / 2.0).sin()).asin();
        let err = ((omega - want) / want).abs();
        println!("W2 m={m}: omega {omega:.9} vs discrete law {want:.9} (rel {err:.2e})");
        w2 &= err <= 1e-6;
    }
    // W3: wave speed in 3D from a pulse's first arrivals.
    let l = lattice();
    let mut w = Wave::new(&l);
    for s in 0..l.e.len() {
        let d2: f64 = (0..3).map(|a| (l.coord(s, a) as f64 - (SIDE / 2) as f64).powi(2)).sum();
        w.u[s] = (-d2 / (2.0 * 1.5f64.powi(2))).exp();
    }
    let (c2, g, f) = (vec![1.0; l.e.len()], sponge(&l), vec![0.0; l.e.len()]);
    let probes = [6i64, 10, 14];
    let mut hist = vec![Vec::new(); 3];
    for _ in 0..160 {
        for (k, &r) in probes.iter().enumerate() {
            hist[k].push(w.u[site(r, 0, 0)].abs());
        }
        w.kick(DT, &c2, &g, &f);
        w.drift(DT);
    }
    let arrivals: Vec<f64> = hist.iter().map(|h| { let peak = h[10..].iter().cloned().fold(0.0, f64::max); h.iter().position(|&x| x >= 0.1 * peak).unwrap() as f64 * DT }).collect();
    let (mr, mt) = (probes.iter().map(|&r| r as f64).sum::<f64>() / 3.0, arrivals.iter().sum::<f64>() / 3.0);
    let speed = probes.iter().zip(&arrivals).map(|(&r, t)| (r as f64 - mr) * (t - mt)).sum::<f64>() / arrivals.iter().map(|t| (t - mt).powi(2)).sum::<f64>();
    println!("W3 arrivals at r {probes:?}: {arrivals:?}; speed {speed:.3} (c 1)");
    let w3 = (speed - 1.0).abs() <= 0.1;
    // W4: zero in, zero out.
    let mut rx = [(site(0, 8, 0), Resonator::new(F1, 0.05, 0.05))];
    let tr = run(&[], &mut rx, 800, &PLAIN);
    let off_zero = tr[0].iter().all(|&e| e.to_bits() == 0) && rx[0].1.x.to_bits() == 0;
    let mut rx = [(site(0, 8, 0), Resonator::new(F1, 0.05, 0.0))];
    let tr = run(&[Tx { site: site(0, 0, 0), freq: F1, amp: 1.0, bits: None }], &mut rx, 800, &PLAIN);
    let uncoupled_zero = tr[0].iter().all(|&e| e.to_bits() == 0);
    println!("W4 transmitter off: everything exactly 0 {off_zero}; uncoupled receiver absorbs exactly 0 {uncoupled_zero}");
    let w4 = off_zero && uncoupled_zero;
    // W5: far field.
    let rs = [6i64, 8, 10, 12];
    let powers: Vec<f64> = rs
        .iter()
        .map(|&r| {
            let mut rx = [(site(r, 0, 0), Resonator::new(F1, 0.05, 0.05))];
            let tr = run(&[Tx { site: site(0, 0, 0), freq: F1, amp: 1.0, bits: None }], &mut rx, 1600, &PLAIN);
            (tr[0][1599] - tr[0][799]) / (800.0 * DT)
        })
        .collect();
    let lx: Vec<f64> = rs.iter().map(|&r| (r as f64).ln()).collect();
    let ly: Vec<f64> = powers.iter().map(|p| p.ln()).collect();
    let (mx, my) = (lx.iter().sum::<f64>() / 4.0, ly.iter().sum::<f64>() / 4.0);
    let slope = lx.iter().zip(&ly).map(|(x, y)| (x - mx) * (y - my)).sum::<f64>() / lx.iter().map(|x| (x - mx).powi(2)).sum::<f64>();
    println!("W5 received power at r {rs:?}: {:?}; exponent {slope:.3} (want -2)", powers.iter().map(|p| format!("{p:.3e}")).collect::<Vec<_>>());
    let w5 = ((slope + 2.0) / 2.0).abs() <= 0.15;
    // W6: tuning.
    let tuned = |f: f64| {
        let mut rx = [(site(8, 0, 0), Resonator::new(f, 0.05, 0.05))];
        let tr = run(&[Tx { site: site(0, 0, 0), freq: F1, amp: 1.0, bits: None }], &mut rx, 1600, &PLAIN);
        tr[0][1599] - tr[0][799]
    };
    let (on, off) = (tuned(F1), tuned(F1 * 1.3));
    println!("W6 tuned {on:.3e} vs detuned by 30% {off:.3e}: x{:.1}", on / off);
    let w6 = on >= 10.0 * off;
    // W7: two channels.
    let (acc, accs, _) = link(&PLAIN, 1.0);
    let single = |which: usize| -> [f64; 2] {
        let (b1, b2) = (bits(1), bits(2));
        let mut txs = vec![Tx { site: site(-6, 0, 0), freq: F1, amp: 1.0, bits: Some(b1) }, Tx { site: site(6, 0, 0), freq: F2, amp: 1.0, bits: Some(b2) }];
        txs.remove(1 - which);
        let mut rxs = [(site(0, 6, 0), Resonator::new(F1, 0.05, 0.05)), (site(0, -6, 0), Resonator::new(F2, 0.05, 0.05))];
        let tr = run(&txs, &mut rxs, 16 * BIT_STEPS, &PLAIN);
        [*tr[0].last().unwrap(), *tr[1].last().unwrap()]
    };
    let (only1, only2) = (single(0), single(1));
    let (sel1, sel2) = (only1[0] / only2[0], only2[1] / only1[1]);
    println!("W7 decoding: channel 1 {:.1}%, channel 2 {:.1}%; own/other energy: receiver 1 x{sel1:.1}, receiver 2 x{sel2:.1}", accs[0] * 100.0, accs[1] * 100.0);
    let w7 = accs[0] >= 0.95 && accs[1] >= 0.95 && sel1 >= 10.0 && sel2 >= 10.0;
    // W8: the bath dial.
    let temps = [0.0, 1e-6, 1e-5, 1e-4, 1e-3];
    let hot: Vec<f64> = temps.iter().map(|&t| link(&Setup { temperature: t, gamma_b: 0.01, ..PLAIN }, 1.0).0).collect();
    println!("W8 decoding vs bath temperature {temps:?}: {:?}", hot.iter().map(|a| format!("{:.1}%", a * 100.0)).collect::<Vec<_>>());
    let w8 = hot.windows(2).all(|p| p[1] <= p[0]) && *hot.last().unwrap() < 0.75;
    // Rounds (check: mean accuracy, then total absorbed; plain wins ties).
    let better = |a: (f64, f64), b: (f64, f64)| a.0 > b.0 || (a.0 == b.0 && a.1 > b.1);
    let base = link(&PLAIN, 1.0);
    let mut zeta_kept = false;
    for z in [0.02, 0.1] {
        let r = link(&Setup { zeta: z, ..PLAIN }, 1.0);
        println!("   amplification (receiver zeta {z}): {:.1}% (energy {:.3e}) vs plain {:.1}% ({:.3e})", r.0 * 100.0, r.2, base.0 * 100.0, base.2);
        zeta_kept |= better((r.0, r.2), (base.0, base.2));
    }
    let hot_setup = Setup { temperature: 1e-3, gamma_b: 0.01, ..PLAIN };
    let hot_base = link(&hot_setup, 1.0);
    let mut amp_kept = false;
    for a in [2.0, 4.0] {
        let r = link(&hot_setup, a);
        println!("   amplification (drive x{a} at T 1e-3): {:.1}% vs plain {:.1}%", r.0 * 100.0, hot_base.0 * 100.0);
        amp_kept |= better((r.0, r.2), (hot_base.0, hot_base.2));
    }
    let mut chaos_kept = false;
    for a in [0.1, 0.3] {
        let r = link(&Setup { chaos: a, ..PLAIN }, 1.0);
        println!("   chaos (frozen c^2 jitter {a}): {:.1}% (energy {:.3e}) vs plain", r.0 * 100.0, r.2);
        chaos_kept |= better((r.0, r.2), (base.0, base.2));
    }
    let _ = acc;
    let v = [
        ("W1 the invariant is conserved (<= 1e-12, d = 1, 2, 3)", w1),
        ("W2 the discrete dispersion law (<= 1e-6)", w2),
        ("W3 waves travel at c (within 10%)", w3),
        ("W4 zero in gives exactly zero", w4),
        ("W5 the far field falls as 1/r^2 (within 15%)", w5),
        ("W6 tuning (>= 10x)", w6),
        ("W7 two channels at once (>= 95% each, >= 10x selectivity)", w7),
        ("W8 a bath turns it thermal (non-increasing; hottest < 75%)", w8),
        ("W9 a sharper receiver not kept, a stronger drive kept under heat", !zeta_kept && amp_kept),
        ("W10 frozen disorder not kept", !chaos_kept),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
