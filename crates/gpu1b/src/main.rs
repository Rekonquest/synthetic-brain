//! GPU-1b orchestrator (docs/GPU-1B-CAP-F128.md): binary128 from 32-bit integer words, checked
//! against GCC's `__float128` through the C oracle (Q1–Q4); then precision as a diagnostic: the
//! linear medium (Q5) and the chaos source (Q6). Q7 lives in thermo6 (`T6_F128=1`).
//! Usage: gpu1b ORACLE_BINARY.

use gpu_cap_binary128::*;
use gpu_cap_f128_add::{add, sub};
use gpu_cap_f128_cmp::{eq, lt, max};
use gpu_cap_f128_conv::{narrow, widen};
use gpu_cap_f128_mul::mul;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use thermo_lattice::Lattice;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn u128(&mut self) -> u128 {
        ((self.next() as u128) << 64) | self.next() as u128
    }
}

/// Run records (op, a, b) through the oracle; 16 result bytes each.
fn oracle(bin: &str, recs: &[(u8, u128, u128)]) -> Vec<[u8; 16]> {
    let mut child = Command::new(bin).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("oracle");
    let mut stdin = child.stdin.take().unwrap();
    let payload: Vec<u8> = recs.iter().flat_map(|&(op, a, b)| std::iter::once(op).chain(a.to_le_bytes()).chain(b.to_le_bytes())).collect();
    let writer = std::thread::spawn(move || {
        stdin.write_all(&payload).unwrap();
    });
    let mut out = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut out).unwrap();
    writer.join().unwrap();
    assert!(child.wait().unwrap().success());
    out.chunks(16).map(|c| c.try_into().unwrap()).collect()
}

fn is_nan_bits(x: u128) -> bool {
    (x & !(1 << 127)) > (0x7fff << 112)
}

fn edges() -> Vec<u128> {
    let one = 0x3fffu128 << 112;
    let mut v = vec![0, 1, (1 << 112) - 1, 1 << 112, one, 0x4000 << 112, (0x7ffe << 112) | ((1 << 112) - 1), 0x7fff << 112, (0x7fff << 112) | (1 << 111), (0x7fff << 112) | 1, one | 1, (0x3f8e) << 112, (0x3f8e << 112) | (1 << 111)];
    let neg: Vec<u128> = v.iter().map(|x| x | (1 << 127)).collect();
    v.extend(neg);
    v
}

fn pairs(r: &mut Rng, class: usize, n: usize) -> Vec<(u128, u128)> {
    (0..n)
        .map(|_| match class {
            0 => (r.u128(), r.u128()),
            1 => {
                let e = 1 + (r.next() % 0x7ffe) as i64;
                let e2 = (e + (r.next() % 241) as i64 - 120).clamp(0, 0x7ffe) as u128;
                let f = |r: &mut Rng, e: u128| ((r.next() as u128 & 1) << 127) | (e << 112) | (r.u128() & ((1 << 112) - 1));
                (f(r, e as u128), f(r, e2))
            }
            _ => {
                let f = |r: &mut Rng| ((r.next() as u128 & 1) << 127) | (((r.next() % 3) as u128) << 112) | (r.u128() & ((1 << 112) - 1));
                (f(r), f(r))
            }
        })
        .collect()
}

fn main() {
    let bin = std::env::args().nth(1).expect("usage: gpu1b ORACLE");
    let t0 = std::time::Instant::now();
    let mut r = Rng(0x6750_1B01);
    println!("GPU-1b: binary128 from 32-bit integer words, against GCC __float128");
    let e = edges();
    let sets: Vec<(&str, Vec<(u128, u128)>)> = vec![
        ("edges x edges", e.iter().flat_map(|&a| e.iter().map(move |&b| (a, b))).collect()),
        ("random bits", pairs(&mut r, 0, 1_000_000)),
        ("exponents within 120", pairs(&mut r, 1, 1_000_000)),
        ("subnormal range", pairs(&mut r, 2, 100_000)),
    ];
    let mut bad = [0usize; 3];
    for (name, ps) in &sets {
        let recs: Vec<(u8, u128, u128)> = ps.iter().flat_map(|&(a, b)| [(0u8, a, b), (1, a, b), (2, a, b), (3, a, b)]).collect();
        let res = oracle(&bin, &recs);
        let (mut fa, mut fs, mut fm, mut fc) = (0, 0, 0, 0);
        for (k, &(a, b)) in ps.iter().enumerate() {
            let (wa, wb) = (word(a), word(b));
            let want = |i: usize| u128::from_le_bytes(res[4 * k + i]);
            let ok = |got: W128, w: u128| if is_nan_bits(w) { is_nan(got) } else { bits(got) == w };
            fa += usize::from(!ok(add(wa, wb), want(0)));
            fs += usize::from(!ok(sub(wa, wb), want(1)));
            fm += usize::from(!ok(mul(wa, wb), want(2)));
            let (olt, oeq) = (res[4 * k + 3][0] == 1, res[4 * k + 3][1] == 1);
            let both_zero = (a | b) & !(1 << 127) == 0;
            let want_max = if is_nan_bits(a) { b } else if is_nan_bits(b) { a } else if olt { b } else { a };
            let max_ok = both_zero || (if is_nan_bits(want_max) { is_nan(max(wa, wb)) } else { bits(max(wa, wb)) == want_max });
            fc += usize::from(lt(wa, wb) != olt || eq(wa, wb) != oeq || !max_ok);
        }
        println!("  {name:<22} {} pairs: add {fa}, sub {fs}, mul {fm}, cmp {fc} mismatches", ps.len());
        bad[0] += fa + fs;
        bad[1] += fm;
        bad[2] += fc;
    }
    // Q4: conversions.
    let mut xs: Vec<f64> = (0..1_000_000).map(|_| f64::from_bits(r.next())).collect();
    xs.extend([0.0, -0.0, 5e-324, f64::MIN_POSITIVE, f64::MAX, f64::INFINITY, f64::NAN, 1.0, 0.1]);
    let wrecs: Vec<(u8, u128, u128)> = xs.iter().map(|x| (5u8, x.to_bits() as u128, 0)).collect();
    let wres = oracle(&bin, &wrecs);
    let widen_bad = xs.iter().zip(&wres).filter(|(x, o)| { let w = u128::from_le_bytes(**o); if x.is_nan() { !is_nan(widen(**x)) } else { bits(widen(**x)) != w } }).count();
    let trip_bad = xs.iter().filter(|x| { let y = narrow(widen(**x)); if x.is_nan() { !y.is_nan() } else { y.to_bits() != x.to_bits() } }).count();
    let ns: Vec<u128> = (0..1_000_000).map(|_| r.u128()).collect();
    let nres = oracle(&bin, &ns.iter().map(|&a| (4u8, a, 0)).collect::<Vec<_>>());
    let narrow_bad = ns.iter().zip(&nres).filter(|(a, o)| { let want = f64::from_bits(u64::from_le_bytes(o[..8].try_into().unwrap())); let got = narrow(word(**a)); if want.is_nan() { !got.is_nan() } else { got.to_bits() != want.to_bits() } }).count();
    println!("Q4 conversions: widen {widen_bad} mismatches of {}; narrow(widen(x)) {trip_bad} failures; narrow {narrow_bad} mismatches of {}", xs.len(), ns.len());
    // Q5: the linear medium at 64 and 128 bits.
    let mut l64 = Lattice::new(&[16, 16, 16]);
    for x in l64.e.iter_mut() {
        *x = (r.next() >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0;
    }
    let mut e128: Vec<W128> = l64.e.iter().map(|&x| widen(x)).collect();
    let kh = widen(0.5 / 2.0);
    let mut worst = 0.0f64;
    for _ in 0..2000 {
        l64.exchange(&|_, _| 0.5);
        for axis in 0..3 {
            let old = e128.clone();
            for s in 0..old.len() {
                let j = l64.forward(s, axis);
                let flux = mul(kh, sub(old[j], old[s]));
                e128[s] = add(e128[s], flux);
                e128[j] = sub(e128[j], flux);
            }
        }
        let scale = l64.e.iter().map(|x| x.abs()).fold(0.0, f64::max);
        worst = worst.max(l64.e.iter().zip(&e128).map(|(a, b)| (a - narrow(*b)).abs()).fold(0.0, f64::max) / scale);
    }
    println!("Q5 linear medium: 128-bit against 64-bit, worst relative difference over 2000 steps {worst:.2e}");
    // Q6: the chaos source at 64 and 128 bits.
    let (four, one) = (widen(4.0), widen(1.0));
    let mut slopes = Vec::new();
    for _ in 0..1000 {
        let x0 = 0.01 + 0.98 * ((r.next() >> 11) as f64 / (1u64 << 53) as f64);
        let (mut a, mut b) = (x0, widen(x0));
        let mut pts = Vec::new();
        for n in 1..=60 {
            a = 4.0 * a * (1.0 - a);
            b = mul(mul(four, b), sub(one, b));
            let d = (a - narrow(b)).abs();
            if d > 1e-15 && d < 1e-3 {
                pts.push((n as f64, d.ln()));
            }
        }
        if pts.len() >= 5 {
            let k = pts.len() as f64;
            let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / k, pts.iter().map(|p| p.1).sum::<f64>() / k);
            slopes.push(pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / pts.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>());
        }
    }
    let lyap = slopes.iter().sum::<f64>() / slopes.len() as f64;
    println!("Q6 chaos source: 64-bit against 128-bit logistic, ln|difference| grows {lyap:.4} per step over {} starts (ln 2 = {:.4})", slopes.len(), std::f64::consts::LN_2);
    let v = [
        ("Q1 add and subtract bit-identical to __float128", bad[0] == 0),
        ("Q2 multiply bit-identical", bad[1] == 0),
        ("Q3 compare identical", bad[2] == 0),
        ("Q4 conversions (widen exact, round trip, narrow = C cast)", widen_bad == 0 && trip_bad == 0 && narrow_bad == 0),
        ("Q5 the linear medium is not precision-limited (<= 1e-13)", worst <= 1e-13),
        ("Q6 the chaos source is precision-limited (ln 2 per step, within 10%)", ((lyap - std::f64::consts::LN_2) / std::f64::consts::LN_2).abs() <= 0.1),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.0} s); Q7 is thermo6's T6_F128 mode", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
