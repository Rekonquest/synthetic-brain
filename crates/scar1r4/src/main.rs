//! SCAR-1 run 4 orchestrator (docs/SCAR-1-PRECOMMIT.md, run 4): Jesse's C17 rebuilt from scratch.
//! A memory is a band spectrum held in its own region by carved trap material (`spec-trap`), in a
//! 32-band medium (`spec-step`), maintained by the noise law (`spec-stir`), and recalled by matching
//! the cue against every region's spectrum with Jesse's spectral bridge (`spec-bridge`). Checks
//! R1–R8, then the rounds. Every setting lives here.

use qb_chaos::keyed_unit;
use spec_bridge::bridge;
use thermo_host::Medium;
use thermo_lattice::Lattice;

const DIMS: [usize; 3] = [20, 20, 4];
const BANDS: usize = 32;
const ACTIVE: usize = 6;
const KAPPA: f64 = 0.2;
const DELTA: f64 = 0.01;
const TAU: f64 = 0.9;
const NU: f64 = 0.002;
const E_REF: f64 = 1.0;
const P: f64 = 3.0;
const WRITE: usize = 250;
/// Run 5 (run 4 is this file at commit 37a28e4 plus its diagnostics): the long drain.
const DRAIN: usize = 20_000;
const READS: [usize; 4] = [0, 250, 2000, 20_000];

#[derive(Clone, Copy)]
struct Cfg {
    /// Run 5's law: sealed mobility, τ, E_ref and ν (None = run 4's law and constants).
    sealed: Option<(f64, f64, f64)>,
    gain: f64,
    mu: f64,
    chaos: f64,
    q: f64,
    eps: f64,
    stirred: bool,
}

const PLAIN: Cfg = Cfg { sealed: Some((0.99, 0.3, 5e-5)), gain: 0.01, mu: 0.02, chaos: 0.0, q: 0.02, eps: 0.05, stirred: true };

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
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// The 16 regions: 4×4×4 blocks on a 4×4 grid at spacing 5 (a one-site moat between regions).
fn regions() -> Vec<Vec<usize>> {
    (0..16)
        .map(|r| {
            let (ox, oy) = (5 * (r % 4), 5 * (r / 4));
            let mut v = Vec::new();
            for z in 0..4 {
                for y in oy..oy + 4 {
                    for x in ox..ox + 4 {
                        v.push(x + DIMS[0] * y + DIMS[0] * DIMS[1] * z);
                    }
                }
            }
            v
        })
        .collect()
}

/// `n` keys: 6 active bands of 32 with weights in (0.5, 1], normalised.
fn keys(seed: u64, n: usize) -> Vec<Vec<f64>> {
    let mut r = Rng(seed.wrapping_mul(0x5CA4_0004) ^ 0x99);
    (0..n)
        .map(|_| {
            let mut k = vec![0.0; BANDS];
            let mut on = 0;
            while on < ACTIVE {
                let b = r.below(BANDS);
                if k[b] == 0.0 {
                    k[b] = 0.5 + 0.5 * r.unit();
                    on += 1;
                }
            }
            let t: f64 = k.iter().sum();
            k.iter().map(|x| x / t).collect()
        })
        .collect()
}

/// Write `plan` (region, key) one after another, then drain; the region spectra at each read time.
fn simulate(cfg: &Cfg, plan: &[(usize, Vec<f64>)], seed: u64) -> (Vec<Vec<Vec<f64>>>, usize) {
    let rs = regions();
    let mut med = Medium::new(&DIMS, BANDS);
    let sites = med.fields[0].e.len();
    let mut material = vec![0.0; sites];
    let mu: Vec<f64> = (0..sites).map(|s| if cfg.chaos > 0.0 { cfg.mu * (1.0 + cfg.chaos * (keyed_unit(0x5BEC_0004 ^ (s as u64).wrapping_mul(0x9E37_79B9) ^ seed) - 0.5)) } else { cfg.mu }).collect();
    let bytes = std::mem::size_of_val(&material[..]) + med.fields.iter().map(|f| std::mem::size_of_val(&f.e[..])).sum::<usize>();
    let mut t = 0;
    let tick = |med: &mut Medium, material: &mut Vec<f64>, t: usize| {
        if cfg.stirred {
            spec_stir::stir(med, cfg.q, cfg.eps, seed, t);
        }
        let energy_of = |med: &Medium| -> Vec<f64> { (0..sites).map(|s| med.fields.iter().map(|f| f.e[s]).sum()).collect() };
        match cfg.sealed {
            None => {
                spec_step::step(med, material, KAPPA, DELTA, TAU);
                spec_trap::form(material, &energy_of(med), &|s| mu[s], NU, E_REF, P);
            }
            Some((tau, e_ref, nu)) => {
                spec_step::step_sealed(med, material, KAPPA, DELTA, tau);
                spec_trap::form(material, &energy_of(med), &|s| mu[s], nu, e_ref, P);
            }
        }
    };
    for (r, key) in plan {
        for _ in 0..WRITE {
            for &s in &rs[*r] {
                med.inject(s, 0, key, cfg.gain);
            }
            tick(&mut med, &mut material, t);
            t += 1;
        }
    }
    let read = |med: &Medium| -> Vec<Vec<f64>> { rs.iter().map(|r| (0..BANDS).map(|b| r.iter().map(|&s| med.fields[b].e[s]).sum()).collect()).collect() };
    let mut snaps = Vec::new();
    for d in 0..=DRAIN {
        if READS.contains(&d) {
            snaps.push(read(&med));
        }
        if d < DRAIN {
            tick(&mut med, &mut material, t);
            t += 1;
        }
    }
    let after = std::mem::size_of_val(&material[..]) + med.fields.iter().map(|f| std::mem::size_of_val(&f.e[..])).sum::<usize>();
    (snaps, if after == bytes { bytes } else { 0 })
}

/// Top-1 accuracy of cues against region spectra: (cue, true region).
fn top1(spectra: &[Vec<f64>], cues: &[(Vec<f64>, usize)]) -> f64 {
    cues.iter().filter(|(c, truth)| (0..16).max_by(|&a, &b| bridge(c, &spectra[a]).partial_cmp(&bridge(c, &spectra[b])).unwrap()).unwrap() == *truth).count() as f64 / cues.len() as f64
}

struct Seed {
    stirred: [f64; 4],
    quiet: [f64; 4],
    control: f64,
    completion: f64,
    noisy: f64,
    selfmatch: f64,
    fold_region: f64,
    fold_selfmatch: f64,
    bytes: usize,
}

fn one_per_region(cfg: &Cfg, seed: u64) -> (Vec<Vec<Vec<f64>>>, Vec<Vec<f64>>) {
    let ks = keys(seed, 16);
    let plan: Vec<(usize, Vec<f64>)> = ks.iter().enumerate().map(|(r, k)| (r, k.clone())).collect();
    (simulate(cfg, &plan, seed).0, ks)
}

fn per_seed(seed: u64) -> Seed {
    let ks = keys(seed, 16);
    let exact: Vec<(Vec<f64>, usize)> = ks.iter().enumerate().map(|(r, k)| (k.clone(), r)).collect();
    let plan: Vec<(usize, Vec<f64>)> = ks.iter().enumerate().map(|(r, k)| (r, k.clone())).collect();
    let (snaps, bytes) = simulate(&PLAIN, &plan, seed);
    let (qsnaps, _) = simulate(&Cfg { stirred: false, ..PLAIN }, &plan, seed);
    let (csnaps, _) = simulate(&PLAIN, &[], seed);
    let stirred = [0, 1, 2, 3].map(|i| top1(&snaps[i], &exact));
    let quiet = [0, 1, 2, 3].map(|i| top1(&qsnaps[i], &exact));
    let mut r = Rng(seed ^ 0xC0E);
    let half: Vec<(Vec<f64>, usize)> = ks
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let on: Vec<usize> = (0..BANDS).filter(|&b| k[b] > 0.0).collect();
            let mut c = vec![0.0; BANDS];
            for &b in on.iter().take(ACTIVE / 2) {
                c[b] = k[b];
            }
            (c, i)
        })
        .collect();
    let noisy: Vec<(Vec<f64>, usize)> = ks.iter().enumerate().map(|(i, k)| (k.iter().map(|x| x + 0.3 / BANDS as f64 * (0.5 + r.unit())).collect(), i)).collect();
    let selfmatch = ks.iter().enumerate().map(|(i, k)| bridge(k, &snaps[1][i])).sum::<f64>() / 16.0;
    // Folding: 32 keys, two per region, written one after the other.
    let ks32 = keys(seed ^ 0xF01D, 32);
    let fplan: Vec<(usize, Vec<f64>)> = (0..32).map(|i| (i / 2, ks32[i].clone())).collect();
    let (fsnaps, _) = simulate(&PLAIN, &fplan, seed);
    let fcues: Vec<(Vec<f64>, usize)> = (0..32).map(|i| (ks32[i].clone(), i / 2)).collect();
    let fold_selfmatch = (0..32).map(|i| bridge(&ks32[i], &fsnaps[1][i / 2])).sum::<f64>() / 32.0;
    Seed { stirred, quiet, control: top1(&csnaps[1], &exact), completion: top1(&snaps[1], &half), noisy: top1(&snaps[1], &noisy), selfmatch, fold_region: top1(&fsnaps[1], &fcues), fold_selfmatch, bytes }
}

fn parallel<T: Send>(seeds: &[u64], f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    std::thread::scope(|sc| {
        let f = &f;
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// Diagnostic, added after run 4 (not a prediction): per region, total energy and self-match at
/// drain t 0, quiet arm, seed 10.
fn diagnose() {
    if std::env::var_os("SCAR_PILOT").is_some() {
        // Pilot for run 5 (labelled): the sealed law, τ 0.99, E_ref 0.3, ν 5e-5, stirred and quiet,
        // fresh and t 2000 top-1 over the test seeds.
        let cfg = Cfg { sealed: Some((0.99, 0.3, 5e-5)), ..PLAIN };
        for stirred in [false, true] {
            let c = Cfg { stirred, ..cfg };
            let v: Vec<[f64; 3]> = parallel(&(10..18).collect::<Vec<_>>(), |sd| {
                let ks = keys(sd, 16);
                let plan: Vec<(usize, Vec<f64>)> = ks.iter().enumerate().map(|(r, k)| (r, k.clone())).collect();
                let cues: Vec<(Vec<f64>, usize)> = ks.iter().enumerate().map(|(r, k)| (k.clone(), r)).collect();
                let (snaps, _) = simulate(&c, &plan, sd);
                [0, 1, 2].map(|i| top1(&snaps[i], &cues))
            });
            let m = [0, 1, 2].map(|i| v.iter().map(|x| x[i]).sum::<f64>() / v.len() as f64);
            println!("pilot sealed, stirred {stirred}: top-1 at drain t 0 / 250 / 2000 = {:.3} / {:.3} / {:.3}", m[0], m[1], m[2]);
        }
        return;
    }
    let ks = keys(10, 16);
    let plan: Vec<(usize, Vec<f64>)> = ks.iter().enumerate().map(|(r, k)| (r, k.clone())).collect();
    let (snaps, _) = simulate(&Cfg { stirred: false, ..PLAIN }, &plan, 10);
    for r in 0..16 {
        let tot: f64 = snaps[0][r].iter().sum();
        println!("region {r:2} (written {} steps before the drain): energy {tot:.3e}, self-match {:.3}", (15 - r) * WRITE, bridge(&ks[r], &snaps[0][r]));
    }
}

fn main() {
    if std::env::var_os("SCAR_DIAG").is_some() {
        return diagnose();
    }
    let t0 = std::time::Instant::now();
    let (test, dev): (Vec<u64>, Vec<u64>) = ((10..18).collect(), (0..4).collect());
    println!("SCAR-1 run 5 (sealed law): C17 from scratch - spectra in regions; medium {DIMS:?}, {BANDS} bands; reads at drain {READS:?}");
    let rs = parallel(&test, per_seed);
    let n = rs.len() as f64;
    let m = |f: &dyn Fn(&Seed) -> f64| rs.iter().map(f).sum::<f64>() / n;
    let st = [0, 1, 2, 3].map(|i| m(&|r| r.stirred[i]));
    let qu = [0, 1, 2, 3].map(|i| m(&|r| r.quiet[i]));
    let (control, completion, noisy, selfmatch, fold_region, fold_self) = (m(&|r| r.control), m(&|r| r.completion), m(&|r| r.noisy), m(&|r| r.selfmatch), m(&|r| r.fold_region), m(&|r| r.fold_selfmatch));
    println!("R1 control (nothing written, stirred, t 250): top-1 {control:.3}");
    println!("R2-R4 top-1 at drain t 0 / 250 / 2000 / 20000: stirred {:.3} / {:.3} / {:.3} / {:.3}; quiet {:.3} / {:.3} / {:.3} / {:.3}", st[0], st[1], st[2], st[3], qu[0], qu[1], qu[2], qu[3]);
    println!("R5 completion (3 of 6 bands, t 250): {completion:.3}; R6 noisy cue (+30% flat): {noisy:.3}");
    println!("R7 folding (2 per region, t 250): region top-1 {fold_region:.3}; self-match {fold_self:.3} against one-per-region {selfmatch:.3}");
    println!("R8 footprint: {} bytes, unchanged by writing: {}", rs[0].bytes, rs.iter().all(|r| r.bytes > 0));
    let score = |cfg: &Cfg, seeds: &[u64]| -> (f64, f64) {
        let v = parallel(seeds, |s| {
            let (snaps, ks) = one_per_region(cfg, s);
            let cues: Vec<(Vec<f64>, usize)> = ks.iter().enumerate().map(|(r, k)| (k.clone(), r)).collect();
            (top1(&snaps[3], &cues), ks.iter().enumerate().map(|(i, k)| bridge(k, &snaps[3][i])).sum::<f64>() / 16.0)
        });
        (v.iter().map(|x| x.0).sum::<f64>() / v.len() as f64, v.iter().map(|x| x.1).sum::<f64>() / v.len() as f64)
    };
    let better = |a: (f64, f64), b: (f64, f64)| a.0 > b.0 || (a.0 == b.0 && a.1 > b.1);
    let (pd, pt) = (score(&PLAIN, &dev), score(&PLAIN, &test));
    println!("   rounds: plain dev {:.3} (self-match {:.4}), test {:.3} ({:.4})", pd.0, pd.1, pt.0, pt.1);
    let mut kept = Vec::new();
    for (name, cfg) in [("amplification (write gain x2)", Cfg { gain: 0.02, ..PLAIN }), ("chaos (frozen mu jitter 0.3)", Cfg { chaos: 0.3, ..PLAIN }), ("heat (stir x2)", Cfg { q: 0.04, ..PLAIN })] {
        let d = score(&cfg, &dev);
        let keep = better(d, pd) && better(score(&cfg, &test), pt);
        println!("   {name}: dev {:.3} (self-match {:.4}) -> {}", d.0, d.1, if keep { "KEPT" } else { "not kept" });
        kept.push(keep);
    }
    let v = [
        ("R1 control <= 0.15", control <= 0.15),
        ("R2 fresh recall >= 0.95", st[0] >= 0.95),
        ("R3 maintain-or-erase: quiet t20000 <= fresh - 0.3", qu[3] <= st[0] - 0.3),
        ("R4 noise maintains: stirred t20000 >= 0.75 and >= quiet + 0.3", st[3] >= 0.75 && st[3] >= qu[3] + 0.3),
        ("R5 completion >= 0.8", completion >= 0.8),
        ("R6 noisy cue >= 0.9", noisy >= 0.9),
        ("R7a folding: region top-1 >= 0.8", fold_region >= 0.8),
        ("R7b folding: self-match falls >= 0.2", selfmatch - fold_self >= 0.2),
        ("R8 zero extra footprint", rs.iter().all(|r| r.bytes > 0)),
        ("R9 amplification kept", kept[0]),
        ("R10 chaos not kept", !kept[1]),
        ("R11 heat x2 not kept", !kept[2]),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
    let _ = Lattice::new(&[1]);
}
