//! ASTRO-2 orchestrator (docs/ASTRO-POC.md): the virial theorem as a self-regulating hot/cold
//! governor in FOLD-8's recall loop (PREDICT -> CONSERVE -> REVISE, soft interference at
//! temperature T). FOLD-8's loop is copied; only who sets T changes:
//! - fixed schedule (FOLD-8 K6): T0, T0/2, T0/4 for every item;
//! - virial: each item's own T_i = kappa sigma_i^2 / m_i (astro-virial), no schedule;
//! - ablation: every item at the round's median virial T;
//! - literal virial (bound -> hot): T_i = c m_i.
//! Hard cancellation (K1, gate rho) and plain are the bars. Every setting is here.

use qb_amplitude::C;

const D: usize = 4096;
const M: usize = 256;
const BITS: u32 = 2;
const NS: [usize; 4] = [384, 512, 768, 1024];
const THREADS: usize = 8;

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
    (0..D).map(|_| C::polar(1.0, 2.0 * std::f64::consts::PI * r.unit())).collect()
}

fn key(seed: u64, i: usize) -> Vec<C> {
    phasors(seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (i as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ 0x6E7)
}

fn codebook(seed: u64) -> Vec<Vec<C>> {
    (0..M).map(|v| phasors(seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ (v as u64).wrapping_mul(0x1656_67B1_9E37_79F9) ^ 0xC0D)).collect()
}

fn random_content(seed: u64, n: usize) -> Vec<u8> {
    let mut r = Rng(seed ^ 0xB17E5);
    (0..n).map(|_| (r.next() % 256) as u8).collect()
}

struct Store<'a> {
    keys: &'a [Vec<C>],
    cre: &'a [Vec<f64>],
    cim: &'a [Vec<f64>],
    /// ATTEND: β·ln(prior_v) added to each code's reading (β 0 = off).
    prior: Vec<f64>,
    beta: f64,
}

/// Per item: (best, margin, the 256 phase-locked readings) of the store w_i = s - alpha (E - own_i).
fn read_pass(st: &Store, s: &[C], e: Option<(&[C], f64, &[Option<Vec<C>>])>) -> Vec<(usize, f64, Vec<f64>)> {
    let n = st.keys.len();
    let mut out = Vec::with_capacity(n);
    const BLK: usize = 8;
    for b0 in (0..n).step_by(BLK) {
        let b1 = (b0 + BLK).min(n);
        let us: Vec<(Vec<f64>, Vec<f64>, f64)> = (b0..b1)
            .map(|i| {
                let (mut ur, mut ui, mut na) = (vec![0.0; D], vec![0.0; D], 0.0);
                for k in 0..D {
                    let mut w = s[k];
                    if let Some((e, a, own)) = e {
                        let mut p = e[k];
                        if let Some(o) = &own[i] {
                            p = C::new(p.re - o[k].re, p.im - o[k].im);
                        }
                        w = C::new(w.re - a * p.re, w.im - a * p.im);
                    }
                    let kc = st.keys[i][k];
                    let u = w.mul(C::new(kc.re, -kc.im));
                    ur[k] = u.re;
                    ui[k] = u.im;
                    na += u.norm2();
                }
                (ur, ui, na)
            })
            .collect();
        let mut sc = vec![vec![0.0; M]; b1 - b0];
        for v in 0..M {
            let (cr, ci) = (&st.cre[v], &st.cim[v]);
            for (j, (ur, ui, na)) in us.iter().enumerate() {
                let mut acc = 0.0;
                for k in 0..D {
                    acc += cr[k] * ur[k] + ci[k] * ui[k];
                }
                sc[j][v] = if *na > 0.0 { acc / (D as f64 * na).sqrt() } else { 0.0 } + if st.beta > 0.0 { st.beta * st.prior[v].ln() } else { 0.0 };
            }
        }
        for x in sc {
            let (mut b, mut s2) = (0usize, usize::MAX);
            for v in 1..M {
                if x[v] > x[b] {
                    s2 = b;
                    b = v;
                } else if s2 == usize::MAX || x[v] > x[s2] {
                    s2 = v;
                }
            }
            let m = x[b] - x[s2];
            out.push((b, m, x));
        }
    }
    out
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

const KAPPAS_P: [f64; 4] = [0.05, 0.1, 0.2, 0.5];
const TS: [f64; 7] = [0.002, 0.003, 0.005, 0.0075, 0.01, 0.015, 0.02];
const LOOP_ROUNDS: usize = 6;
const KAPPA_LOOP: f64 = 0.1;
const PROBES: usize = 64;
const SHOW: usize = 5;

fn conj(a: &[C]) -> Vec<C> {
    a.iter().map(|c| C::new(c.re, -c.im)).collect()
}

fn gloss() -> Vec<u8> {
    std::fs::read("docs/GLOSSARY.md").expect("run from the repository root")
}

/// One store's evidence: per item the 256 readings (first pass and refined), the Born weights, the
/// true value; plus the measured noise variance and the ATTEND prior.
struct Evidence {
    z: Vec<Vec<f64>>,
    z_refined: Vec<Vec<f64>>,
    born: Vec<Vec<f64>>,
    truth: Vec<usize>,
    noise_var: f64,
    prior: Vec<f64>,
}

fn evidence(text: bool, n: usize, seed: u64, refine: bool) -> Evidence {
    let content: Vec<u8> = if text { gloss()[..n].to_vec() } else { random_content(seed, n) };
    let codes = codebook(seed);
    let cre: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.re).collect()).collect();
    let cim: Vec<Vec<f64>> = codes.iter().map(|c| c.iter().map(|x| x.im).collect()).collect();
    let keys: Vec<Vec<C>> = (0..n).map(|i| key(seed, i)).collect();
    let mut s = vec![C::new(0.0, 0.0); D];
    for (i, &v) in content.iter().enumerate() {
        for (a, b) in s.iter_mut().zip(qb_bind::bind(&keys[i], &codes[v as usize])) {
            *a = a.add(b);
        }
    }
    qb_quantize::quantize(&mut s, BITS);
    let uniform = vec![1.0 / M as f64; M];
    let st = Store { keys: &keys, cre: &cre, cim: &cim, prior: uniform.clone(), beta: 0.0 };
    let first = read_pass(&st, &s, None);
    // Born: |<code, u>|^2 per code.
    let born: Vec<Vec<f64>> = keys
        .iter()
        .map(|k| {
            let u = qb_bind::bind(&s, &conj(k));
            codes.iter().map(|c| {
                let (mut re, mut im) = (0.0, 0.0);
                for (a, b) in c.iter().zip(&u) {
                    re += a.re * b.re + a.im * b.im;
                    im += a.re * b.im - a.im * b.re;
                }
                (re * re + im * im) / (D as f64 * D as f64)
            }).collect()
        })
        .collect();
    // SENSING: noise variance from keys never written.
    let probe_keys: Vec<Vec<C>> = (0..PROBES).map(|i| key(seed ^ 0x9E0B_E5, i)).collect();
    let probe = read_pass(&Store { keys: &probe_keys, cre: &cre, cim: &cim, prior: uniform.clone(), beta: 0.0 }, &s, None);
    let noise_var = probe.iter().flat_map(|x| x.2.iter()).map(|z| z * z).sum::<f64>() / (PROBES * M) as f64;
    // ATTEND prior (FOLD-8 K2): counts from the top half of first margins.
    let mut prior = vec![1.0; M];
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| first[b].1.partial_cmp(&first[a].1).unwrap());
    for &i in &order[..n / 2] {
        prior[first[i].0] += 1.0;
    }
    let tot: f64 = prior.iter().sum();
    prior.iter_mut().for_each(|x| *x /= tot);
    // Amplification: the refined readings of the virial recall loop.
    let mut reads = first.clone();
    if refine {
        for _ in 0..LOOP_ROUNDS {
            let t_item: Vec<f64> = reads.iter().map(|x| astro_virial::temperature(&x.2, KAPPA_LOOP)).collect();
            let mut own: Vec<Option<Vec<C>>> = vec![None; n];
            let mut e = vec![C::new(0.0, 0.0); D];
            for j in 0..n {
                let p = qb_classical::temper(&reads[j].2.iter().map(|x| (x - reads[j].2[reads[j].0]).exp()).collect::<Vec<_>>(), t_item[j]);
                let mut mix = vec![C::new(0.0, 0.0); D];
                for (v, pv) in p.iter().enumerate() {
                    if *pv > 1e-6 {
                        for (m, c) in mix.iter_mut().zip(&codes[v]) {
                            *m = m.add(c.scale(*pv));
                        }
                    }
                }
                let b = qb_bind::bind(&keys[j], &mix);
                for (x, y) in e.iter_mut().zip(&b) {
                    *x = x.add(*y);
                }
                own[j] = Some(b);
            }
            let (mut dot, mut nn) = (0.0, 0.0);
            for (x, y) in e.iter().zip(&s) {
                dot += x.re * y.re + x.im * y.im;
                nn += x.norm2();
            }
            let alpha = if nn > 0.0 { dot / nn } else { 0.0 };
            reads = read_pass(&st, &s, Some((&e, alpha, &own)));
        }
    }
    Evidence { z: first.iter().map(|x| x.2.clone()).collect(), z_refined: reads.iter().map(|x| x.2.clone()).collect(), born, truth: content.iter().map(|&v| v as usize).collect(), noise_var, prior }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Law {
    /// Softmax at a fixed temperature (temperature scaling; the bar).
    Softmax(f64),
    Born,
    /// Boltzmann at each item's own temperature kappa sigma^2 / m.
    Virial(f64),
    /// Boltzmann at T = sigma^2 / mu, measured by the store itself (no labels).
    MaxEnt,
}

/// Per item, the 256 probabilities under a law. `jitter` (chaos) scales each item's temperature.
fn probs(ev: &Evidence, law: Law, refined: bool, attend: bool, jitter: Option<u64>, seed: u64) -> Vec<Vec<f64>> {
    let z = if refined { &ev.z_refined } else { &ev.z };
    let mu = prob_moments::signal(z.iter().map(|r| r.iter().map(|x| x * x).sum::<f64>()).sum::<f64>() / z.len() as f64, ev.noise_var, M);
    z.iter()
        .enumerate()
        .map(|(i, r)| {
            let mut t = match law {
                Law::Softmax(t) => t,
                Law::Born => 0.0,
                Law::Virial(k) => astro_virial::temperature(r, k),
                Law::MaxEnt => if mu > 0.0 { ev.noise_var / mu } else { f64::INFINITY },
            };
            if let Some(v) = jitter {
                let k = seed.wrapping_mul(0xD1B5_4A32_D192_ED03) ^ v.wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (i as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
                t *= 1.0 + 0.5 * (qb_chaos::keyed_unit(k) - 0.5);
            }
            let mut w: Vec<f64> = if law == Law::Born {
                let b = &ev.born[i];
                b.to_vec()
            } else {
                let top = r.iter().cloned().fold(f64::MIN, f64::max);
                r.iter().map(|x| ((x - top) / t.max(1e-12)).exp()).collect()
            };
            if attend {
                for (x, p) in w.iter_mut().zip(&ev.prior) {
                    *x *= p;
                }
            }
            let tot: f64 = w.iter().sum();
            w.iter().map(|x| x / tot).collect()
        })
        .collect()
}

/// (ECE over the shown candidates, Brier of the top, log-loss of the truth, top-5 coverage,
/// top-1 accuracy, accuracy of candidates scored in [0.6, 0.8), and how many there were).
fn measure(p: &[Vec<f64>], truth: &[usize]) -> [f64; 7] {
    let mut bins = vec![(0.0, 0.0, 0usize); 10];
    let (mut brier, mut ll, mut cov, mut top1, mut c70, mut n70) = (0.0, 0.0, 0usize, 0usize, 0usize, 0usize);
    for (pi, &t) in p.iter().zip(truth) {
        let mut idx: Vec<usize> = (0..M).collect();
        idx.sort_by(|&a, &b| pi[b].partial_cmp(&pi[a]).unwrap());
        for &v in &idx[..SHOW] {
            let b = ((pi[v] * 10.0) as usize).min(9);
            bins[b].0 += pi[v];
            bins[b].1 += (v == t) as usize as f64;
            bins[b].2 += 1;
            if (0.6..0.8).contains(&pi[v]) {
                n70 += 1;
                c70 += (v == t) as usize;
            }
        }
        brier += (pi[idx[0]] - (idx[0] == t) as usize as f64).powi(2);
        ll += -pi[t].max(1e-6).ln();
        cov += idx[..SHOW].contains(&t) as usize;
        top1 += (idx[0] == t) as usize;
    }
    let total: usize = bins.iter().map(|b| b.2).sum();
    let ece = bins.iter().map(|b| if b.2 > 0 { (b.0 - b.1).abs() } else { 0.0 }).sum::<f64>() / total as f64;
    let n = p.len() as f64;
    [ece, brier / n, ll / n, cov as f64 / n, top1 as f64 / n, if n70 > 0 { c70 as f64 / n70 as f64 } else { f64::NAN }, n70 as f64]
}

/// Pool evidence over seeds; measure a law on the pooled items.
fn pooled(evs: &[Evidence], law: Law, refined: bool, attend: bool, jitter: Option<u64>) -> [f64; 7] {
    let mut p = Vec::new();
    let mut t = Vec::new();
    for (k, ev) in evs.iter().enumerate() {
        p.extend(probs(ev, law, refined, attend, jitter, k as u64));
        t.extend(ev.truth.iter().cloned());
    }
    measure(&p, &t)
}

fn gather(text: bool, n: usize, seeds: std::ops::Range<u64>, refine: bool) -> Vec<Evidence> {
    std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.map(|s| sc.spawn(move || evidence(text, n, s, refine))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn line(name: &str, m: &[f64; 7]) -> String {
    format!("  {name:<34} ECE {:.3}  Brier {:.3}  log-loss {:.3}  top-5 {:.3}  top-1 {:.3}  '70%' right {:.3} (n {})", m[0], m[1], m[2], m[3], m[4], m[5], m[6])
}

fn main() {
    let t0 = std::time::Instant::now();
    println!("PROB-1: a probability stack from physics axioms, 2-bit store D {D}, top-{SHOW} candidates with independent scores");
    let ns = [384usize, 512, 768];
    // Dev: softmax T and virial kappa by log-loss (random content, pooled over N).
    let dev: Vec<(usize, Vec<Evidence>)> = ns.iter().map(|&n| (n, gather(false, n, 0..4, false))).collect();
    let dll = |law: Law| dev.iter().map(|(_, e)| pooled(e, law, false, false, None)[2]).sum::<f64>();
    let t_star = *TS.iter().min_by(|a, b| dll(Law::Softmax(**a)).partial_cmp(&dll(Law::Softmax(**b))).unwrap()).unwrap();
    let k_star = *KAPPAS_P.iter().min_by(|a, b| dll(Law::Virial(**a)).partial_cmp(&dll(Law::Virial(**b))).unwrap()).unwrap();
    println!("  dev choices: softmax T {t_star}, virial kappa {k_star}");
    let laws = [("softmax, tuned T (bar)", Law::Softmax(t_star)), ("Born", Law::Born), ("virial (own T)", Law::Virial(k_star)), ("maximum entropy (no labels)", Law::MaxEnt)];
    let mut res = std::collections::HashMap::new();
    for &n in &ns {
        let ev = gather(false, n, 10..18, n == 512);
        println!("\nRandom content, N {n} (test):");
        for (nm, l) in laws {
            let m = pooled(&ev, l, false, false, None);
            println!("{}", line(nm, &m));
            res.insert((n, format!("{l:?}")), m);
        }
        if n == 512 {
            for (nm, l) in [("maximum entropy, refined readings", Law::MaxEnt), ("virial, refined readings", Law::Virial(k_star))] {
                let m = pooled(&ev, l, true, false, None);
                println!("{}", line(nm, &m));
                res.insert((n, format!("{l:?}-refined")), m);
            }
        }
    }
    // Chaos on the best law by dev log-loss.
    let best = *[Law::Born, Law::Virial(k_star), Law::MaxEnt].iter().min_by(|a, b| dll(**a).partial_cmp(&dll(**b)).unwrap()).unwrap();
    let dbase = dll(best);
    let mut bv = (dbase, None);
    for v in 1..=3u64 {
        let d = dev.iter().map(|(_, e)| pooled(e, best, false, false, Some(v))[2]).sum::<f64>();
        if d < bv.0 {
            bv = (d, Some(v));
        }
    }
    let chaos_kept = if let Some(v) = bv.1 {
        let tests: Vec<Vec<Evidence>> = ns.iter().map(|&n| gather(false, n, 10..18, false)).collect();
        let tb = tests.iter().map(|e| pooled(e, best, false, false, None)[2]).sum::<f64>();
        let tc = tests.iter().map(|e| pooled(e, best, false, false, Some(v))[2]).sum::<f64>();
        println!("\n  chaos on {best:?}: dev log-loss {dbase:.4} -> {:.4} (variant {v}); test {tb:.4} -> {tc:.4}", bv.0);
        tc < tb
    } else {
        println!("\n  chaos on {best:?}: no variant beat plain on dev ({dbase:.4})");
        false
    };
    // Glossary with the ATTEND prior.
    let ge = gather(true, 768, 10..18, false);
    let g0 = pooled(&ge, Law::MaxEnt, false, false, None);
    let g1 = pooled(&ge, Law::MaxEnt, false, true, None);
    println!("\nGlossary, N 768 (test):");
    println!("{}", line("maximum entropy", &g0));
    println!("{}", line("maximum entropy x ATTEND prior", &g1));
    let r = |n: usize, k: &str| res[&(n, k.to_string())];
    let me = |n: usize| r(n, "MaxEnt");
    let sm = |n: usize| r(n, &format!("{:?}", Law::Softmax(t_star)));
    let vi = |n: usize| r(n, &format!("{:?}", Law::Virial(k_star)));
    let p1 = ns.iter().all(|&n| me(n)[0] <= 0.03 && me(n)[0] <= sm(n)[0] + 0.01);
    let p3 = vi(768)[2] <= me(768)[2].min(sm(768)[2]).min(r(768, "Born")[2]);
    let v = [
        (format!("P1 max-ent ECE <= 0.03 at every N and within 0.01 of tuned softmax ({:.3?} vs {:.3?})", ns.map(|n| me(n)[0]), ns.map(|n| sm(n)[0])), p1),
        (format!("P2 Born ECE >= 0.1 at N 768 ({:.3})", r(768, "Born")[0]), r(768, "Born")[0] >= 0.1),
        (format!("P3 virial lowest log-loss at N 768 ({:.3}; max-ent {:.3}, softmax {:.3}, Born {:.3})", vi(768)[2], me(768)[2], sm(768)[2], r(768, "Born")[2]), p3),
        (format!("P4 top-5 coverage >= top-1 + 0.1 at N 768 ({:.3} vs {:.3})", me(768)[3], me(768)[4]), me(768)[3] >= me(768)[4] + 0.1),
        (format!("P5 max-ent '70%' check in [0.65, 0.75] (N 768: {:.3}, n {})", me(768)[5], me(768)[6]), (0.65..=0.75).contains(&me(768)[5])),
        ("P6 chaos not kept".to_string(), !chaos_kept),
        (format!("P7 refined readings cut max-ent log-loss >= 10% at N 512 ({:.3} -> {:.3})", me(512)[2], r(512, "MaxEnt-refined")[2]), r(512, "MaxEnt-refined")[2] <= 0.9 * me(512)[2]),
        (format!("P8 glossary: ATTEND prior cuts log-loss >= 10% ({:.3} -> {:.3})", g0[2], g1[2]), g1[2] <= 0.9 * g0[2]),
    ];
    println!();
    for (nm, ok) in &v {
        println!("{} {nm}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
