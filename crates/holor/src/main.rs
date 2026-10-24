//! HOLO-R orchestrator (docs/HOLO-R-PRECOMMIT.md): a self-healing holographic store across lifelong
//! damage, under Jesse's Rules 5-7. A 2-bit phase store (D 4096, 8 regions of 512) holds N key -> byte
//! memories; three episodes each erase 2 fresh regions. Per episode the arm may perceive the damage
//! (cognition), read through the healthy amplitudes only (information theory), decode by self-set-
//! temperature relaxation (mechanics/thermodynamics), proofread (biology), and rewrite the damaged
//! regions from what it recalls (neuroscience replay, chemistry Le Chatelier weights; repair laws:
//! replay, autocatalysis, annealing). Two chaos maps (logistic at perception, Henon at repair). The
//! frozen metric is recall right after each episode's damage and decode; no stage reads true values.

use qb_amplitude::C;
use std::f64::consts::PI;

const D: usize = 4096;
const R: usize = 8;
const RW: usize = D / R;
const M: usize = 256;
const BITS: u32 = 2;
const PROBES: usize = 64;
const KAPPA_V: f64 = 0.1;
const EPISODES: usize = 3;

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

fn phasors(seed: u64) -> Vec<C> {
    let mut r = Rng(seed);
    (0..D).map(|_| C::polar(1.0, 2.0 * PI * r.unit())).collect()
}

struct World {
    cre: Vec<Vec<f64>>,
    cim: Vec<Vec<f64>>,
    codes: Vec<Vec<C>>,
    keys: Vec<Vec<C>>,
    vals: Vec<usize>,
    home: Vec<usize>,
}

fn world(seed: u64, n: usize) -> World {
    let codes: Vec<Vec<C>> = (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64 + 1).wrapping_mul(0x1656_67B1_9E37_79F9))).collect();
    let mut r = Rng(seed ^ 0x401D);
    World {
        cre: codes.iter().map(|c| c.iter().map(|x| x.re).collect()).collect(),
        cim: codes.iter().map(|c| c.iter().map(|x| x.im).collect()).collect(),
        keys: (0..n).map(|i| phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64 + 7).wrapping_mul(0x9FB2_1C65_1E98_DF25))).collect(),
        vals: (0..n).map(|_| (r.next() % M as u64) as usize).collect(),
        home: (0..n).map(|i| i % R).collect(),
        codes,
    }
}

/// The amplitude indices item i is read or written through, given a region mask and the layout.
fn indices(i: usize, mask: &[bool; R], local: bool, w: &World) -> Vec<usize> {
    (0..R).filter(|&r| mask[r] && (!local || w.home[i] == r)).flat_map(|r| r * RW..(r + 1) * RW).collect()
}

type Read = (usize, f64, Vec<f64>);

/// Phase-locked reads of `items` through `mask`; with a predicted interference E (and each item's own
/// share) dissolved at scale alpha (explaining away).
fn read(q: &[C], mask: &[bool; R], local: bool, w: &World, items: &[usize], e: Option<(&[C], f64, &[Vec<C>])>) -> Vec<Read> {
    items
        .iter()
        .map(|&i| {
            let idx = indices(i, mask, local, w);
            if idx.is_empty() {
                return (0, 0.0, vec![0.0; M]);
            }
            let (mut ur, mut ui) = (Vec::with_capacity(idx.len()), Vec::with_capacity(idx.len()));
            for &k in &idx {
                let mut x = q[k];
                if let Some((e, a, own)) = e {
                    let o = own[i][k];
                    x = C::new(x.re - a * (e[k].re - o.re), x.im - a * (e[k].im - o.im));
                }
                let kc = w.keys[i][k];
                let u = x.mul(C::new(kc.re, -kc.im));
                ur.push(u.re);
                ui.push(u.im);
            }
            let sc: Vec<f64> = (0..M)
                .map(|v| {
                    let (cr, ci) = (&w.cre[v], &w.cim[v]);
                    let mut acc = 0.0;
                    for (j, &k) in idx.iter().enumerate() {
                        acc += cr[k] * ur[j] + ci[k] * ui[j];
                    }
                    acc / idx.len() as f64
                })
                .collect();
            let (mut b, mut s2) = (0usize, usize::MAX);
            for v in 1..M {
                if sc[v] > sc[b] {
                    s2 = b;
                    b = v;
                } else if s2 == usize::MAX || sc[v] > sc[s2] {
                    s2 = v;
                }
            }
            (b, sc[b] - sc[s2], sc)
        })
        .collect()
}

/// The soft content vector of a read at temperature t: Σ_v p_v code_v over `idx` (full length D).
fn mixture(rd: &Read, t: f64, w: &World, idx: &[usize]) -> Vec<C> {
    let top = rd.2[rd.0];
    let p = qb_classical::temper(&rd.2.iter().map(|x| (x - top).exp()).collect::<Vec<_>>(), t.max(1e-9));
    let mut mix = vec![C::new(0.0, 0.0); D];
    for (v, pv) in p.iter().enumerate() {
        if *pv > 1e-6 {
            for &k in idx {
                mix[k] = mix[k].add(w.codes[v][k].scale(*pv));
            }
        }
    }
    mix
}

/// Decode all items through `mask`: a plain read, or the self-set-temperature relaxation loop.
fn decode(q: &[C], mask: &[bool; R], local: bool, w: &World, relax: bool, rounds: usize) -> Vec<Read> {
    let n = w.keys.len();
    let items: Vec<usize> = (0..n).collect();
    let mut reads = read(q, mask, local, w, &items, None);
    if !relax {
        return reads;
    }
    for _ in 0..rounds {
        let mut own: Vec<Vec<C>> = Vec::with_capacity(n);
        let mut e = vec![C::new(0.0, 0.0); D];
        for i in 0..n {
            let idx = indices(i, mask, local, w);
            let t = astro_virial::temperature(&reads[i].2, KAPPA_V);
            let mix = mixture(&reads[i], t, w, &idx);
            let mut b = vec![C::new(0.0, 0.0); D];
            for &k in &idx {
                b[k] = w.keys[i][k].mul(mix[k]);
                e[k] = e[k].add(b[k]);
            }
            own.push(b);
        }
        let (mut dot, mut nn) = (0.0, 0.0);
        for k in 0..D {
            if mask[k / RW] {
                dot += e[k].re * q[k].re + e[k].im * q[k].im;
                nn += e[k].norm2();
            }
        }
        let a = if nn > 0.0 { dot / nn } else { 0.0 };
        reads = read(q, mask, local, w, &items, Some((&e, a, &own)));
    }
    reads
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Perc {
    /// No perception: read everything, repair nothing.
    None,
    Perceive,
    /// The ablation: as many regions as perceived, chosen at random.
    Random,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Repair {
    None,
    Replay,
    Autocatalysis,
    Anneal,
}

#[derive(Clone, Copy, Debug)]
struct Arm {
    local: bool,
    perc: Perc,
    order2: bool,
    mask_read: bool,
    relax: bool,
    rounds: usize,
    repair: Repair,
    proofread: bool,
    lechat: bool,
    chaos1: bool,
    chaos2: bool,
}

const C1: Arm = Arm { local: false, perc: Perc::Perceive, order2: false, mask_read: true, relax: true, rounds: 6, repair: Repair::Replay, proofread: true, lechat: true, chaos1: false, chaos2: false };
const PLAIN: Arm = Arm { local: false, perc: Perc::None, order2: false, mask_read: false, relax: false, rounds: 0, repair: Repair::None, proofread: false, lechat: false, chaos1: false, chaos2: false };

fn quantize_region(s: &[C], q: &mut [C], r: usize) {
    let mut part = s[r * RW..(r + 1) * RW].to_vec();
    qb_quantize::quantize(&mut part, BITS);
    q[r * RW..(r + 1) * RW].copy_from_slice(&part);
}

/// Perception: the reading h_r per region (the mean margin of PROBES stored keys read through region r
/// alone) and the hypotheses "region r damaged" (h_r < kappa_r * median h).
fn perceive(q: &[C], w: &World, local: bool, kappa: &[f64; R]) -> ([bool; R], [f64; R]) {
    let mut h = [0.0; R];
    for r in 0..R {
        let mut mask = [false; R];
        mask[r] = true;
        let probes: Vec<usize> = (0..w.keys.len()).filter(|&i| !local || w.home[i] == r).take(PROBES).collect();
        let rd = read(q, &mask, local, w, &probes, None);
        h[r] = rd.iter().map(|x| x.1).sum::<f64>() / rd.len().max(1) as f64;
    }
    let mut s = h.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = s[R / 2];
    let mut dmg = [false; R];
    for r in 0..R {
        dmg[r] = h[r] < kappa[r] * med;
    }
    (dmg, h)
}

/// C2's perception: decode through everything, then each region's consistency with the decode.
fn perceive_by_residual(q: &[C], w: &World, local: bool, dec: &[Read], kappa: &[f64; R]) -> [bool; R] {
    let mut cons = [0.0; R];
    for r in 0..R {
        let mut rec = vec![C::new(0.0, 0.0); RW];
        for (i, rd) in dec.iter().enumerate() {
            if local && w.home[i] != r {
                continue;
            }
            for k in 0..RW {
                rec[k] = rec[k].add(w.keys[i][r * RW + k].mul(w.codes[rd.0][r * RW + k]));
            }
        }
        let (mut dot, mut n1, mut n2) = (0.0, 0.0, 0.0);
        for k in 0..RW {
            let qk = q[r * RW + k];
            dot += rec[k].re * qk.re + rec[k].im * qk.im;
            n1 += rec[k].norm2();
            n2 += qk.norm2();
        }
        cons[r] = if n1 > 0.0 && n2 > 0.0 { dot / (n1 * n2).sqrt() } else { 0.0 };
    }
    let mut s = cons.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = s[R / 2];
    let mut dmg = [false; R];
    for r in 0..R {
        dmg[r] = cons[r] < kappa[r] * med;
    }
    dmg
}

/// The Henon map (1.4, 0.3), folded mod 1, as a keyed sequence centred on its own mean.
fn henon(seed: u64, n: usize) -> Vec<f64> {
    let mut r = Rng(seed ^ 0x4E40);
    let (mut x, mut y) = (0.1 * r.unit(), 0.1 * r.unit());
    for _ in 0..100 {
        let nx = 1.0 - 1.4 * x * x + y;
        y = 0.3 * x;
        x = nx;
    }
    let v: Vec<f64> = (0..n)
        .map(|_| {
            let nx = 1.0 - 1.4 * x * x + y;
            y = 0.3 * x;
            x = nx;
            x.rem_euclid(1.0)
        })
        .collect();
    let m = v.iter().sum::<f64>() / n as f64;
    v.iter().map(|h| h - m).collect()
}

/// Rewrite the damaged regions from the recalled values (dissolve first, then write).
#[allow(clippy::too_many_arguments)]
fn rewrite(s: &mut [C], q: &mut [C], w: &World, local: bool, dmg: &[bool; R], dec: &[Read], write: &[f64], soft: Option<f64>) {
    for r in 0..R {
        if !dmg[r] {
            continue;
        }
        for k in r * RW..(r + 1) * RW {
            s[k] = C::new(0.0, 0.0);
        }
        let region: Vec<usize> = (r * RW..(r + 1) * RW).collect();
        for (i, rd) in dec.iter().enumerate() {
            if (local && w.home[i] != r) || write[i] == 0.0 {
                continue;
            }
            let content: Vec<C> = match soft {
                Some(t) if write[i] < 0.0 => mixture(rd, t, w, &region),
                _ => w.codes[rd.0].clone(),
            };
            let wt = write[i].abs();
            for &k in &region {
                s[k] = s[k].add(w.keys[i][k].mul(content[k]).scale(wt));
            }
        }
        quantize_region(s, q, r);
    }
}

/// One arm across the three episodes: recall right after each episode's damage and decode.
fn run(arm: Arm, n: usize, kappa: f64, seed: u64) -> [f64; EPISODES] {
    let w = world(seed, n);
    let mut s = vec![C::new(0.0, 0.0); D];
    for i in 0..n {
        let b = qb_bind::bind(&w.keys[i], &w.codes[w.vals[i]]);
        let all = [true; R];
        for k in indices(i, &all, arm.local, &w) {
            s[k] = s[k].add(b[k]);
        }
    }
    let mut q = s.clone();
    qb_quantize::quantize(&mut q, BITS);
    let mut r = Rng(seed ^ 0xDA4A);
    let mut order: Vec<usize> = (0..R).collect();
    for i in (1..R).rev() {
        let j = (r.next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    let mut out = [0.0; EPISODES];
    for (ep, o) in out.iter_mut().enumerate() {
        for &g in &order[2 * ep..2 * ep + 2] {
            for k in g * RW..(g + 1) * RW {
                let p = C::polar(1.0, 2.0 * PI * r.unit());
                q[k] = p;
                s[k] = p;
            }
        }
        // Perception (with chaos 1 on kappa: 3 logistic variants, the decode's total margin selects).
        let kap = |v: u64| -> [f64; R] {
            let mut k = [kappa; R];
            if v > 0 {
                for (ri, kk) in k.iter_mut().enumerate() {
                    *kk *= 1.0 + 0.5 * (qb_chaos::keyed_unit(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ ((ep * 8 + ri) as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ v) - 0.5);
                }
            }
            k
        };
        let all = [true; R];
        let decode_with = |dmg: &[bool; R]| -> Vec<Read> {
            let mut mask = [true; R];
            if arm.mask_read {
                for rr in 0..R {
                    mask[rr] = !dmg[rr];
                }
            }
            decode(&q, &mask, arm.local, &w, arm.relax, arm.rounds)
        };
        let perceive_with = |k: &[f64; R]| -> [bool; R] {
            match arm.perc {
                Perc::None => [false; R],
                Perc::Perceive if arm.order2 => {
                    let full = decode(&q, &all, arm.local, &w, arm.relax, arm.rounds);
                    perceive_by_residual(&q, &w, arm.local, &full, k)
                }
                Perc::Perceive => perceive(&q, &w, arm.local, k).0,
                Perc::Random => {
                    let cnt = perceive(&q, &w, arm.local, k).0.iter().filter(|x| **x).count();
                    let mut rr = Rng(seed ^ (ep as u64 + 1) * 0xBADD);
                    let mut idx: Vec<usize> = (0..R).collect();
                    for i in (1..R).rev() {
                        let j = (rr.next() % (i as u64 + 1)) as usize;
                        idx.swap(i, j);
                    }
                    let mut d = [false; R];
                    for &i in &idx[..cnt] {
                        d[i] = true;
                    }
                    d
                }
            }
        };
        let total = |d: &[Read]| d.iter().map(|x| x.1).sum::<f64>();
        let mut dmg = perceive_with(&kap(0));
        let mut dec = decode_with(&dmg);
        if arm.chaos1 {
            let mut best = total(&dec);
            for v in 1..=3u64 {
                let dv = perceive_with(&kap(v));
                let dd = decode_with(&dv);
                let t = total(&dd);
                if t > best {
                    best = t;
                    dmg = dv;
                    dec = dd;
                }
            }
        }
        *o = dec.iter().zip(&w.vals).filter(|(d, &v)| d.0 == v).count() as f64 / n as f64;
        if arm.repair == Repair::None || ep + 1 == EPISODES {
            continue;
        }
        // Repair. The healthy regions are split in two halves for proofreading (biology).
        let cycles = if arm.repair == Repair::Replay { 1 } else { 3 };
        let mut cur = dec;
        for cyc in 0..cycles {
            let healthy: Vec<usize> = (0..R).filter(|&rr| !dmg[rr]).collect();
            let pass: Vec<bool> = if arm.proofread && healthy.len() >= 2 {
                let (mut ma, mut mb) = ([false; R], [false; R]);
                for (j, &rr) in healthy.iter().enumerate() {
                    if j % 2 == 0 { ma[rr] = true } else { mb[rr] = true }
                }
                let (da, db) = (decode(&q, &ma, arm.local, &w, arm.relax, arm.rounds), decode(&q, &mb, arm.local, &w, arm.relax, arm.rounds));
                (0..n).map(|i| da[i].0 == cur[i].0 && db[i].0 == cur[i].0).collect()
            } else {
                vec![true; n]
            };
            let mut ms: Vec<f64> = cur.iter().map(|x| x.1).collect();
            ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mref = ms[n / 2].max(1e-12);
            let base_w: Vec<f64> = (0..n).map(|i| if arm.lechat { (1.0 - cur[i].1 / mref).max(0.2) } else { 1.0 }).collect();
            let anneal_t = if arm.repair == Repair::Anneal && cyc + 1 < cycles { Some(0.01 / (1u64 << cyc) as f64) } else { None };
            // Write plan: proofread items hard (positive weight); in annealing, failed items soft (negative marks soft).
            let plan = |jit: Option<&Vec<f64>>| -> Vec<f64> {
                (0..n)
                    .map(|i| {
                        let wi = base_w[i] * jit.map(|h| 1.0 + 0.5 * h[i]).unwrap_or(1.0);
                        if pass[i] { wi } else if anneal_t.is_some() { -wi } else { 0.0 }
                    })
                    .collect()
            };
            let mut plan_w = plan(None);
            if arm.chaos2 {
                // Henon variants of the weights; the readback margin after repair selects.
                let readback = |pw: &Vec<f64>| -> f64 {
                    let (mut s2, mut q2) = (s.clone(), q.clone());
                    rewrite(&mut s2, &mut q2, &w, arm.local, &dmg, &cur, pw, anneal_t);
                    let items: Vec<usize> = (0..n).collect();
                    let mut m = [true; R];
                    for rr in 0..R {
                        m[rr] = true;
                    }
                    read(&q2, &m, arm.local, &w, &items, None).iter().map(|x| x.1).sum::<f64>()
                };
                let mut best = (readback(&plan_w), plan_w.clone());
                for v in 1..=3u64 {
                    let h = henon(seed ^ ((ep * 16 + cyc * 4) as u64 + v).wrapping_mul(0x5851_F42D), n);
                    let pv = plan(Some(&h));
                    let rb = readback(&pv);
                    if rb > best.0 {
                        best = (rb, pv);
                    }
                }
                plan_w = best.1;
            }
            rewrite(&mut s, &mut q, &w, arm.local, &dmg, &cur, &plan_w, anneal_t);
            if cyc + 1 < cycles {
                // The healed regions now help: decode again through every region (autocatalysis).
                dmg = [false; R];
                cur = decode(&q, &all, arm.local, &w, arm.relax, arm.rounds);
            }
        }
    }
    out
}

fn mean_runs(arm: Arm, n: usize, kappa: f64, seeds: std::ops::Range<u64>) -> [f64; EPISODES] {
    let v: Vec<[f64; EPISODES]> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || run(arm, n, kappa, s))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let k = v.len() as f64;
    let mut m = [0.0; EPISODES];
    for x in &v {
        for e in 0..EPISODES {
            m[e] += x[e] / k;
        }
    }
    m
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("HOLO-R: a self-healing holographic store; 2-bit, D {D}, {R} regions; 3 episodes each erasing 2 fresh regions");
    let kappas = [0.5, 0.6, 0.7, 0.8];
    let dev: Vec<f64> = kappas.iter().map(|&k| (mean_runs(C1, 192, k, 0..4)[2] + mean_runs(C1, 256, k, 0..4)[2]) / 2.0).collect();
    let kappa = kappas[(0..4).max_by(|&a, &b| dev[a].partial_cmp(&dev[b]).unwrap()).unwrap()];
    println!("  dev kappa {kappas:?} -> final recall {dev:.3?}; chosen {kappa}");
    let arms: Vec<(&str, Arm)> = vec![
        ("plain holographic (control)", PLAIN),
        ("local layout (control)", Arm { local: true, ..PLAIN }),
        ("C1 principal (perceive first)", C1),
        ("C2 (decode first)", Arm { order2: true, ..C1 }),
        ("ablate cognition: random mask", Arm { perc: Perc::Random, ..C1 }),
        ("ablate information: no mask", Arm { mask_read: false, ..C1 }),
        ("ablate mechanics: plain read", Arm { relax: false, ..C1 }),
        ("ablate neuroscience: no replay", Arm { repair: Repair::None, ..C1 }),
        ("ablate biology: no proofreading", Arm { proofread: false, ..C1 }),
        ("ablate chemistry: uniform weights", Arm { lechat: false, ..C1 }),
        ("ablate physics: local layout in C1", Arm { local: true, ..C1 }),
        ("chaos 1 (logistic at perception)", Arm { chaos1: true, ..C1 }),
        ("chaos 2 (Henon at repair)", Arm { chaos2: true, ..C1 }),
        ("chaos 1 + 2", Arm { chaos1: true, chaos2: true, ..C1 }),
        ("amplification: decoder rounds x 2", Arm { rounds: 12, ..C1 }),
        ("repair law: autocatalysis", Arm { repair: Repair::Autocatalysis, ..C1 }),
        ("repair law: annealing", Arm { repair: Repair::Anneal, ..C1 }),
    ];
    let mut res = std::collections::HashMap::new();
    for &n in &[192usize, 256] {
        println!("\n  N {n}: recall after episodes 1 / 2 / 3");
        for (name, a) in &arms {
            let m = mean_runs(*a, n, kappa, 10..18);
            println!("    {name:<38} {:.3} / {:.3} / {:.3}", m[0], m[1], m[2]);
            res.insert((name.to_string(), n), m);
        }
    }
    let g = |name: &str, n: usize| res[&(name.to_string(), n)];
    let fin = |name: &str, n: usize| g(name, n)[2];
    let c1 = "C1 principal (perceive first)";
    let abl = ["ablate cognition: random mask", "ablate information: no mask", "ablate mechanics: plain read", "ablate neuroscience: no replay", "ablate biology: no proofreading"];
    let h2 = abl.iter().all(|a| fin(a, 256) <= fin(c1, 256) - 0.03) && (fin("ablate chemistry: uniform weights", 256) - fin(c1, 256)).abs() <= 0.02;
    let chaos_promoted = ["chaos 1 (logistic at perception)", "chaos 2 (Henon at repair)", "chaos 1 + 2"].iter().any(|a| fin(a, 192) > fin(c1, 192) && fin(a, 256) > fin(c1, 256));
    let v = [
        (format!("H1 C1 final >= 0.9 at 192 ({:.3}), >= 0.8 at 256 ({:.3}); plain <= 0.3 ({:.3}, {:.3}); local <= 0.3 ({:.3}, {:.3})", fin(c1, 192), fin(c1, 256), fin("plain holographic (control)", 192), fin("plain holographic (control)", 256), fin("local layout (control)", 192), fin("local layout (control)", 256)),
            fin(c1, 192) >= 0.9 && fin(c1, 256) >= 0.8 && fin("plain holographic (control)", 192) <= 0.3 && fin("plain holographic (control)", 256) <= 0.3 && fin("local layout (control)", 192) <= 0.3 && fin("local layout (control)", 256) <= 0.3),
        (format!("H2 cognition, information, mechanics, neuroscience, biology each load-bearing (>= 0.03 at 256: {:.3?} vs {:.3}); Le Chatelier ties ({:.3})", abl.map(|a| fin(a, 256)), fin(c1, 256), fin("ablate chemistry: uniform weights", 256)), h2),
        (format!("H3 C1 >= C2 ({:.3}/{:.3} vs {:.3}/{:.3})", fin(c1, 192), fin(c1, 256), fin("C2 (decode first)", 192), fin("C2 (decode first)", 256)), fin(c1, 192) >= fin("C2 (decode first)", 192) && fin(c1, 256) >= fin("C2 (decode first)", 256)),
        ("H4 no chaos arm promoted".to_string(), !chaos_promoted),
        (format!("H5 rounds x 2 raise episode-1 recall at 256 by >= 0.01 ({:.3} vs {:.3})", g("amplification: decoder rounds x 2", 256)[0], g(c1, 256)[0]), g("amplification: decoder rounds x 2", 256)[0] >= g(c1, 256)[0] + 0.01),
        (format!("H6 autocatalysis >= replay + 0.02 at 256 ({:.3} vs {:.3})", fin("repair law: autocatalysis", 256), fin(c1, 256)), fin("repair law: autocatalysis", 256) >= fin(c1, 256) + 0.02),
        (format!("H7 annealing ties autocatalysis +-0.02 ({:.3} vs {:.3})", fin("repair law: annealing", 256), fin("repair law: autocatalysis", 256)), (fin("repair law: annealing", 256) - fin("repair law: autocatalysis", 256)).abs() <= 0.02),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
