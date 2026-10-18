//! GPU-1 orchestrator (docs/GPU-1-CAP-F64.md): the CPU validation of the binary64 capability stack
//! the Arc will run — W1/W2 (the doubler, once and twice), F1–F3 (add/sub, mul, compare against
//! native f64, bit for bit), F4 (T1's exchange and T2's bath reproduced bit for bit), and A (the
//! doubler's product against a composed one). Single-threaded.

use gpu_cap_binary64::*;
use gpu_cap_f64_add::{add, sub};
use gpu_cap_f64_cmp::{eq, lt, max};
use gpu_cap_f64_mul::{mul, mul_by};
use thermo_bath::{damp, exchange};
use thermo_lattice::Lattice;

const N: usize = 10_000_000;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

type W128 = Wide<W64>;
fn w128(x: u128) -> W128 {
    Wide { hi: word((x >> 64) as u64), lo: word(x as u64) }
}
fn n128(w: W128) -> u128 {
    ((bits(w.hi) as u128) << 64) | bits(w.lo) as u128
}

/// The edge words for the integer checks.
fn int_edges() -> Vec<u128> {
    let mut v = vec![0, 1, 2, u128::MAX, u128::MAX - 1, 1 << 127, (1 << 127) - 1];
    for k in [31, 32, 33, 63, 64, 65, 95, 96, 97] {
        v.push(1u128 << k);
        v.push((1u128 << k) - 1);
    }
    v.extend([u64::MAX as u128, u32::MAX as u128, 0xFFFF_FFFF_0000_0000, 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000]);
    v
}

/// W1: Wide<u32> against native u64 (and u128 for the full product).
fn w1(r: &mut Rng) -> usize {
    let mut bad = 0;
    let mut check = |x: u64, y: u64, n: u32| {
        let (a, b) = (word(x), word(y));
        let shl = if n >= 64 { 0 } else { x << n };
        let shr = if n >= 64 { 0 } else { x >> n };
        let (lo, hi) = a.mul_wide(b);
        let p = x as u128 * y as u128;
        let ok = bits(a + b) == x.wrapping_add(y)
            && bits(a - b) == x.wrapping_sub(y)
            && bits(Word::shl(a, n)) == shl
            && bits(Word::shr(a, n)) == shr
            && a.lz() == x.leading_zeros()
            && (a < b) == (x < y)
            && (bits(lo), bits(hi)) == (p as u64, (p >> 64) as u64);
        bad += usize::from(!ok);
    };
    let edges: Vec<u64> = int_edges().iter().map(|&x| x as u64).collect();
    for &x in &edges {
        for &y in &edges {
            for n in [0, 1, 31, 32, 33, 63, 64, 70] {
                check(x, y, n);
            }
        }
    }
    for _ in 0..N {
        let (x, y) = (r.next(), r.next());
        check(x, y, (r.next() % 72) as u32);
    }
    bad
}

/// W2: Wide<Wide<u32>> against native u128; the high half of its 256-bit product against a
/// schoolbook over u64 halves with u128 accumulators (an independent recomputation).
fn w2(r: &mut Rng) -> usize {
    let mut bad = 0;
    let hi256 = |x: u128, y: u128| -> u128 {
        let (xl, xh, yl, yh) = (x as u64 as u128, x >> 64, y as u64 as u128, y >> 64);
        let (ll, lh, hl, hh) = (xl * yl, xl * yh, xh * yl, xh * yh);
        let mid = (ll >> 64) + (lh as u64 as u128) + (hl as u64 as u128);
        hh + (lh >> 64) + (hl >> 64) + (mid >> 64)
    };
    let mut check = |x: u128, y: u128, n: u32| {
        let (a, b) = (w128(x), w128(y));
        let shl = if n >= 128 { 0 } else { x << n };
        let shr = if n >= 128 { 0 } else { x >> n };
        let (lo, hi) = a.mul_wide(b);
        let ok = n128(a + b) == x.wrapping_add(y)
            && n128(a - b) == x.wrapping_sub(y)
            && n128(Word::shl(a, n)) == shl
            && n128(Word::shr(a, n)) == shr
            && a.lz() == x.leading_zeros()
            && (a < b) == (x < y)
            && n128(lo) == x.wrapping_mul(y)
            && n128(hi) == hi256(x, y);
        bad += usize::from(!ok);
    };
    let edges = int_edges();
    for &x in &edges {
        for &y in &edges {
            for n in [0, 1, 63, 64, 65, 127, 128, 130] {
                check(x, y, n);
            }
        }
    }
    for _ in 0..N {
        let x = ((r.next() as u128) << 64) | r.next() as u128;
        let y = ((r.next() as u128) << 64) | r.next() as u128;
        check(x, y, (r.next() % 136) as u32);
    }
    bad
}

/// The structured f64 edge set (bits).
fn f_edges() -> Vec<u64> {
    let mut v = Vec::new();
    for x in [0.0f64, 5e-324, 1e-323, f64::MIN_POSITIVE, 1.0, 1.5, 2.0, 3.0, f64::MAX, f64::INFINITY, 0.1, 1.0 + f64::EPSILON] {
        v.push(x.to_bits());
        v.push((-x).to_bits());
    }
    v.push(0x000f_ffff_ffff_ffff); // the largest subnormal
    v.push(0x800f_ffff_ffff_ffff);
    v.push(f64::NAN.to_bits());
    v.push(0x7ff0_0000_0000_0001); // a signalling NaN pattern
    v.push(2f64.powi(-53).to_bits()); // half an ulp of 1: ties against 1.0
    v.push((3.0 * 2f64.powi(-53)).to_bits());
    v.push((1.0 + 2.0 * f64::EPSILON).to_bits());
    v
}

/// One pair class's generator.
fn pair(r: &mut Rng, class: usize) -> (u64, u64) {
    match class {
        0 => (r.next(), r.next()),
        1 => {
            let e = 1 + r.next() % 2046;
            let d = (r.next() % 121) as i64 - 60;
            let e2 = (e as i64 + d).clamp(0, 2046) as u64;
            let f = |r: &mut Rng, e: u64| (r.next() & (1 << 63)) | (e << 52) | (r.next() & ((1 << 52) - 1));
            (f(r, e), f(r, e2))
        }
        _ => {
            let f = |r: &mut Rng| (r.next() & (1 << 63)) | ((r.next() % 3) << 52) | (r.next() & ((1 << 52) - 1));
            (f(r), f(r))
        }
    }
}

fn same(got: W64, want: f64) -> bool {
    if want.is_nan() { is_nan(got) } else { bits(got) == want.to_bits() }
}

/// F1–F3 and A over every set: mismatches per (op, set).
fn floats(r: &mut Rng) -> Vec<(String, usize, usize)> {
    let sets: [(&str, usize); 4] = [("edges x edges", 0), ("random bits", N), ("exponents within 60", N), ("subnormal range", N / 10)];
    let composed = |a: W64, b: W64| -> (W64, W64) {
        let p = bits(a) as u128 * bits(b) as u128;
        (word(p as u64), word((p >> 64) as u64))
    };
    let mut out = Vec::new();
    let edges = f_edges();
    for (si, (name, count)) in sets.iter().enumerate() {
        let pairs: Vec<(u64, u64)> = if si == 0 { edges.iter().flat_map(|&x| edges.iter().map(move |&y| (x, y))).collect() } else { (0..*count).map(|_| pair(r, si - 1)).collect() };
        let (mut f_add, mut f_sub, mut f_mul, mut f_cmp, mut a_amp) = (0, 0, 0, 0, 0);
        for &(x, y) in &pairs {
            let (fx, fy) = (f64::from_bits(x), f64::from_bits(y));
            let (a, b) = (word(x), word(y));
            f_add += usize::from(!same(add(a, b), fx + fy));
            f_sub += usize::from(!same(sub(a, b), fx - fy));
            let m = mul(a, b);
            f_mul += usize::from(!same(m, fx * fy));
            a_amp += usize::from(bits(m) != bits(mul_by(a, b, composed)));
            let both_zero = fx == 0.0 && fy == 0.0;
            let max_ok = both_zero || same(max(a, b), fx.max(fy));
            f_cmp += usize::from(lt(a, b) != (fx < fy) || eq(a, b) != (fx == fy) || !max_ok);
        }
        let n = pairs.len();
        for (op, bad) in [("F1 add", f_add), ("F1 sub", f_sub), ("F2 mul", f_mul), ("F3 cmp", f_cmp), ("A doubler vs composed", a_amp)] {
            out.push((format!("{op:<22} {name}"), bad, n));
        }
    }
    out
}

/// F4: the law recomputed with the capability ops, in the native operation order.
fn law_parity(r: &mut Rng) -> Vec<(String, bool, usize)> {
    let dims = [16usize, 16, 16];
    let mut out = Vec::new();
    for case in ["T1 exchange, random start", "T2 bath exchange + damping, random start", "T2 bath, pulse with leak 8 (subnormal tails)"] {
        let mut native = Lattice::new(&dims);
        if case.contains("pulse") {
            native.e[8 + 16 * 8 + 256 * 8] = 1.0;
        } else {
            for x in native.e.iter_mut() {
                *x = (r.next() >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0;
            }
        }
        let mut cap: Vec<W64> = native.e.iter().map(|&x| from_f64(x)).collect();
        let (kappa, gamma): (f64, f64) = (0.5, if case.contains("pulse") { 8.0 } else { 0.1 });
        // Constants the orchestrator computes on the CPU and passes in.
        let kh = from_f64(kappa / 2.0);
        let a = from_f64((-gamma).exp());
        let sigma = from_f64((0.0f64 * kappa * (2.0 - kappa) / 2.0).sqrt());
        let sq = from_f64((0.0f64 * (1.0 - (-gamma).exp() * (-gamma).exp())).sqrt());
        let (z, h) = (from_f64(0.0), from_f64(0.0));
        let mut ok = true;
        let mut subnormals = 0;
        for _ in 0..100 {
            if case.starts_with("T1") {
                native.exchange(&|_, _| kappa);
                for axis in 0..3 {
                    let old = cap.clone();
                    for site in 0..old.len() {
                        let j = native.forward(site, axis);
                        let flux = mul(kh, sub(old[j], old[site]));
                        cap[site] = add(cap[site], flux);
                        cap[j] = sub(cap[j], flux);
                    }
                }
            } else {
                exchange(&mut native, 0.0, kappa, &|_| 0.0, &|_, _| 0.0);
                for axis in 0..3 {
                    for parity in [0, 1] {
                        for site in 0..cap.len() {
                            if native.coord(site, axis) % 2 != parity {
                                continue;
                            }
                            let j = native.forward(site, axis);
                            let f = add(mul(kh, sub(sub(cap[j], h), sub(cap[site], h))), mul(sigma, z));
                            cap[site] = add(cap[site], f);
                            cap[j] = sub(cap[j], f);
                        }
                    }
                }
                damp(&mut native, 0.0, &|_| gamma, &|_| 0.0, &|_| 0.0);
                for e in cap.iter_mut() {
                    *e = add(add(h, mul(sub(*e, h), a)), mul(sq, z));
                }
            }
            subnormals += native.e.iter().filter(|x| x.is_subnormal()).count();
            ok &= native.e.iter().zip(&cap).all(|(x, c)| x.to_bits() == bits(*c));
        }
        out.push((case.to_owned(), ok, subnormals));
    }
    out
}

fn main() {
    let t0 = std::time::Instant::now();
    let mut r = Rng(0x6750_0001);
    println!("GPU-1: binary64 from 32-bit integer words, validated on the CPU against native f64");
    let b1 = w1(&mut r);
    println!("W1 Wide<u32> vs u64/u128: {b1} mismatches");
    let b2 = w2(&mut r);
    println!("W2 Wide<Wide<u32>> vs u128 (and a schoolbook 256-bit high half): {b2} mismatches");
    let fl = floats(&mut r);
    for (name, bad, n) in &fl {
        println!("  {name:<44} {bad} mismatches of {n}");
    }
    let lp = law_parity(&mut r);
    for (name, ok, sub) in &lp {
        println!("F4 {name}: {} (native subnormal values seen over the run: {sub})", if *ok { "bit-identical" } else { "DIFFERS" });
    }
    let f = |p: &str| fl.iter().filter(|x| x.0.starts_with(p)).all(|x| x.1 == 0);
    let v = [
        ("W1 the doubler works once", b1 == 0),
        ("W2 growth: the doubler applied again", b2 == 0),
        ("F1 add and subtract bit-identical", f("F1")),
        ("F2 multiply bit-identical", f("F2")),
        ("F3 compare identical", f("F3")),
        ("F4 T1/T2 laws reproduced bit for bit", lp.iter().all(|x| x.1)),
        ("A doubler product = composed product", f("A ")),
    ];
    println!();
    for (name, ok) in &v {
        println!("{} {name}", if *ok { "HOLDS " } else { "FAILS " });
    }
    println!("\n{} of {} hold ({:.1} s)", v.iter().filter(|x| x.1).count(), v.len(), t0.elapsed().as_secs_f64());
}
