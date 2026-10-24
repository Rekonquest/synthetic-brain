//! HYBRID-1 orchestrator (docs/HYBRID-1-PRECOMMIT.md): each memory where it shines. Three stores
//! (co-engines, unchanged): raw (a value at an exact address), association (values bound to keys,
//! 2-bit phase, D 4096) and recognition (patterns superposed, 2-bit phase, D 4096). A stream of 768
//! memories (256 exact, 256 associative, 256 recognition) is routed by a router; then each kind's
//! query is asked. Routers: by kind (fixed), per-memory shine (trial write + readback, the best
//! readback keeps it), learned (shine on the first 20% of each kind, then the reinforced store),
//! and the single-store bars. Every setting is here.

use qb_amplitude::C;
use std::f64::consts::PI;

const D: usize = 4096;
const M: usize = 256;
const BITS: u32 = 2;
const PER_KIND: usize = 256;
const RAW_BYTES: usize = 768;
const CUE: f64 = 0.25;
const LEARN_FRACTION: f64 = 0.2;

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

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Exact,
    Assoc,
    Recog,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Where {
    Raw,
    Assoc,
    Recog,
}

/// One memory: its kind, its key (address/cue source), its value, or its pattern.
struct Memory {
    kind: Kind,
    key: Vec<C>,
    value: usize,
    pattern: Vec<C>,
}

/// The vector a memory adds to a phase store.
fn vector(m: &Memory, codes: &[Vec<C>]) -> Vec<C> {
    match m.kind {
        Kind::Recog => m.pattern.clone(),
        _ => qb_bind::bind(&m.key, &codes[m.value]),
    }
}

/// The raw (unquantized) sums and the members of each phase store, and raw's addresses.
struct Stores {
    assoc: Vec<C>,
    recog: Vec<C>,
    raw: Vec<usize>,
}

fn quantized(s: &[C]) -> Vec<C> {
    let mut q = s.to_vec();
    qb_quantize::quantize(&mut q, BITS);
    q
}

fn add(s: &mut [C], v: &[C], sign: f64) {
    for (a, b) in s.iter_mut().zip(v) {
        *a = a.add(b.scale(sign));
    }
}

/// The value read with a cue from a phase store, and its margin.
fn read_value(store_q: &[C], cue: &[C], codes: &[Vec<C>]) -> (usize, f64) {
    let u = qb_bind::bind(store_q, &conj(cue));
    let mut sc: Vec<(f64, usize)> = codes.iter().enumerate().map(|(v, c)| (qb_overlap_phase::overlap_phase(c, &u), v)).collect();
    sc.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    (sc[0].1, sc[0].0 - sc[1].0)
}

fn partial(key: &[C], seed: u64) -> Vec<C> {
    let mut r = Rng(seed);
    key.iter().map(|k| if r.unit() < CUE { *k } else { C::new(0.0, 0.0) }).collect()
}

/// The readback of memory i in store w (after a trial write): a success flag and a margin.
fn readback(m: &Memory, w: Where, st: &Stores, codes: &[Vec<C>], tau: f64, cue_seed: u64) -> (bool, f64) {
    match (m.kind, w) {
        (Kind::Exact, Where::Raw) => (st.raw.len() <= RAW_BYTES, 1.0),
        (Kind::Exact, Where::Assoc) => {
            let (v, mg) = read_value(&quantized(&st.assoc), &m.key, codes);
            (v == m.value, mg)
        }
        (Kind::Assoc, Where::Assoc) => {
            let (v, mg) = read_value(&quantized(&st.assoc), &partial(&m.key, cue_seed), codes);
            (v == m.value, mg)
        }
        (Kind::Recog, w2) if w2 != Where::Raw => {
            let s = if w2 == Where::Assoc { &st.assoc } else { &st.recog };
            let o = qb_overlap_phase::overlap_phase(&m.pattern, &quantized(s));
            (o > tau, o - tau)
        }
        _ => (false, f64::NEG_INFINITY),
    }
}

fn eligible(k: Kind) -> Vec<Where> {
    match k {
        Kind::Exact => vec![Where::Raw, Where::Assoc],
        Kind::Assoc => vec![Where::Assoc],
        Kind::Recog => vec![Where::Recog, Where::Assoc],
    }
}

fn put(st: &mut Stores, m: &Memory, i: usize, w: Where, codes: &[Vec<C>], sign: f64) {
    match w {
        Where::Raw => {
            if sign > 0.0 {
                st.raw.push(i)
            } else {
                st.raw.retain(|&x| x != i)
            }
        }
        Where::Assoc => add(&mut st.assoc, &vector(m, codes), sign),
        Where::Recog => add(&mut st.recog, &vector(m, codes), sign),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Router {
    Fixed,
    Shine,
    Learn,
    AllRaw,
    AllAssoc,
}

/// (exact accuracy, associative accuracy, recognition accuracy, overall, trial writes).
fn run(router: Router, seed: u64, tau: f64) -> (f64, f64, f64, f64, usize) {
    let codes: Vec<Vec<C>> = (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ 0xC0D)).collect();
    let mut r = Rng(seed ^ 0x5EED);
    let mut mems: Vec<Memory> = Vec::new();
    for k in [Kind::Exact, Kind::Assoc, Kind::Recog] {
        for _ in 0..PER_KIND {
            let id = mems.len() as u64;
            mems.push(Memory { kind: k, key: phasors(seed ^ (id + 1).wrapping_mul(0x9FB2_1C65_1E98_DF25)), value: (r.next() % M as u64) as usize, pattern: phasors(seed ^ (id + 1).wrapping_mul(0xA24B_AED4_963E_E407)) });
        }
    }
    // A random arrival order.
    let mut order: Vec<usize> = (0..mems.len()).collect();
    for i in (1..order.len()).rev() {
        let j = (r.next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    let mut st = Stores { assoc: vec![C::new(0.0, 0.0); D], recog: vec![C::new(0.0, 0.0); D], raw: Vec::new() };
    let mut place = vec![Where::Raw; mems.len()];
    let mut trials = 0usize;
    let mut seen = [0usize; 3];
    let mut wins: [[usize; 3]; 3] = [[0; 3]; 3];
    let kidx = |k: Kind| match k { Kind::Exact => 0, Kind::Assoc => 1, Kind::Recog => 2 };
    let widx = |w: Where| match w { Where::Raw => 0, Where::Assoc => 1, Where::Recog => 2 };
    let wof = |i: usize| [Where::Raw, Where::Assoc, Where::Recog][i];
    let cost = |w: Where, st: &Stores| -> f64 { if w == Where::Raw { 1.0 } else { let _ = st; 4.0 } };
    for &i in &order {
        let m = &mems[i];
        let ki = kidx(m.kind);
        let shine = |st: &mut Stores, trials: &mut usize| -> Where {
            let mut best: Option<(bool, f64, f64, Where)> = None;
            for w in eligible(m.kind) {
                put(st, m, i, w, &codes, 1.0);
                let (ok, mg) = readback(m, w, st, &codes, tau, seed ^ i as u64);
                put(st, m, i, w, &codes, -1.0);
                *trials += 1;
                let c = cost(w, st);
                let better = match best {
                    None => true,
                    Some((bok, bmg, bc, _)) => (ok && !bok) || (ok == bok && ((ok && c < bc) || (!ok && mg > bmg))),
                };
                if better {
                    best = Some((ok, mg, c, w));
                }
            }
            best.unwrap().3
        };
        let w = match router {
            Router::Fixed => match m.kind { Kind::Exact => Where::Raw, Kind::Assoc => Where::Assoc, Kind::Recog => Where::Recog },
            Router::AllRaw => Where::Raw,
            Router::AllAssoc => Where::Assoc,
            Router::Shine => shine(&mut st, &mut trials),
            Router::Learn => {
                if (seen[ki] as f64) < LEARN_FRACTION * PER_KIND as f64 {
                    let w = shine(&mut st, &mut trials);
                    wins[ki][widx(w)] += 1;
                    w
                } else {
                    wof((0..3).max_by_key(|&x| wins[ki][x]).unwrap())
                }
            }
        };
        seen[ki] += 1;
        let w = if w == Where::Raw && m.kind == Kind::Recog { Where::Raw } else { w };
        put(&mut st, m, i, w, &codes, 1.0);
        place[i] = w;
    }
    // The queries.
    let (aq, rq) = (quantized(&st.assoc), quantized(&st.recog));
    let raw_ok = |i: usize| st.raw.iter().position(|&x| x == i).map(|p| p < RAW_BYTES).unwrap_or(false);
    let (mut ex, mut asn, mut rec) = (0usize, 0usize, 0usize);
    for (i, m) in mems.iter().enumerate() {
        match m.kind {
            Kind::Exact => {
                ex += match place[i] {
                    Where::Raw => raw_ok(i),
                    Where::Assoc => read_value(&aq, &m.key, &codes).0 == m.value,
                    Where::Recog => read_value(&rq, &m.key, &codes).0 == m.value,
                } as usize
            }
            Kind::Assoc => {
                let cue = partial(&m.key, seed ^ 0xC0E ^ i as u64);
                asn += match place[i] {
                    Where::Raw => false,
                    Where::Assoc => read_value(&aq, &cue, &codes).0 == m.value,
                    Where::Recog => read_value(&rq, &cue, &codes).0 == m.value,
                } as usize
            }
            Kind::Recog => {
                rec += match place[i] {
                    Where::Raw => false,
                    w => qb_overlap_phase::overlap_phase(&m.pattern, if w == Where::Assoc { &aq } else { &rq }) > tau,
                } as usize
            }
        }
    }
    // Unseen probes: say "not seen" if no store recognizes them.
    let mut rej = 0usize;
    for j in 0..PER_KIND {
        let p = phasors(seed ^ 0xFA15E ^ (j as u64).wrapping_mul(0x9E37_79B9));
        let hit = qb_overlap_phase::overlap_phase(&p, &aq) > tau || qb_overlap_phase::overlap_phase(&p, &rq) > tau;
        rej += (!hit) as usize;
    }
    let n = PER_KIND as f64;
    let (e, a, g) = (ex as f64 / n, asn as f64 / n, (rec + rej) as f64 / (2.0 * n));
    (e, a, g, (e + a + g) / 3.0, trials)
}

fn mean_run(router: Router, seeds: std::ops::Range<u64>, tau: f64) -> [f64; 5] {
    let v: Vec<(f64, f64, f64, f64, usize)> = std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || run(router, s, tau))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let k = v.len() as f64;
    [v.iter().map(|x| x.0).sum::<f64>() / k, v.iter().map(|x| x.1).sum::<f64>() / k, v.iter().map(|x| x.2).sum::<f64>() / k, v.iter().map(|x| x.3).sum::<f64>() / k, v.iter().map(|x| x.4 as f64).sum::<f64>() / k]
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("HYBRID-1: 768 memories (256 exact, 256 associative with a quarter cue, 256 recognition); raw {RAW_BYTES} B, association 1 KB, recognition 1 KB");
    // τ on dev, by the fixed router's recognition accuracy.
    let taus = [0.005, 0.01, 0.015, 0.02, 0.03];
    let dv: Vec<f64> = taus.iter().map(|&t| mean_run(Router::Fixed, 0..4, t)[2]).collect();
    let tau = taus[(0..taus.len()).max_by(|&a, &b| dv[a].partial_cmp(&dv[b]).unwrap()).unwrap()];
    println!("  dev tau {taus:?} -> recognition {dv:.3?}; chosen {tau}");
    let rs = [("all in raw (bar)", Router::AllRaw), ("all in association (bar)", Router::AllAssoc), ("R-fixed, by kind", Router::Fixed), ("R-shine, per memory", Router::Shine), ("R-learn, over time", Router::Learn)];
    let mut res = Vec::new();
    println!("\n  router                         exact  assoc  recog  overall  trial writes (test, seeds 10-17)");
    for (nm, r) in rs {
        let x = mean_run(r, 10..18, tau);
        println!("  {nm:<30} {:.3}  {:.3}  {:.3}  {:.3}    {:.0}", x[0], x[1], x[2], x[3], x[4]);
        res.push(x);
    }
    let (raw, asc, fx, sh, le) = (res[0], res[1], res[2], res[3], res[4]);
    let v = [
        (format!("H1 R-fixed beats both bars by >= 0.15 ({:.3} vs {:.3}, {:.3})", fx[3], raw[3], asc[3]), fx[3] >= raw[3].max(asc[3]) + 0.15),
        (format!("H2 R-shine >= R-fixed - 0.02 ({:.3} vs {:.3})", sh[3], fx[3]), sh[3] >= fx[3] - 0.02),
        (format!("H3 R-learn within 0.02 of R-shine ({:.3} vs {:.3}) with >= 70% fewer trials ({:.0} vs {:.0})", le[3], sh[3], le[4], sh[4]), (le[3] - sh[3]).abs() <= 0.02 && le[4] <= 0.3 * sh[4]),
    ];
    println!();
    for (n, ok) in &v {
        println!("{} {n}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
