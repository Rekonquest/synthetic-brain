//! GPU-1 (docs/GPU-1-CAP-F64.md): binary64 multiply from integer words, for a GPU with no 64-bit
//! floats. Round to nearest, ties to even; subnormals kept; signed zeros; infinities; NaN → quiet
//! NaN (payload not part of the contract). The standard soft-float algorithm (compiler-rt's
//! `__muldf3`). The 106-bit significand product is the doubler's: `gpu-cap-wide` applied to the
//! 64-bit word gives the 128-bit product — growth by the same law, twice. `mul_by` takes the wide
//! product as a seam, so another product can be composed in its place (Jesse's fallback).

use gpu_cap_binary64::*;

/// The full 64 × 64 → 128-bit product as (lo, hi).
pub type WideProduct = fn(W64, W64) -> (W64, W64);

/// The doubler's product.
pub fn doubled(a: W64, b: W64) -> (W64, W64) {
    a.mul_wide(b)
}

/// a × b, with the doubler's product.
pub fn mul(a: W64, b: W64) -> W64 {
    mul_by(a, b, doubled)
}

/// a × b, with the given wide product.
pub fn mul_by(a: W64, b: W64, product: WideProduct) -> W64 {
    let (a_exp, b_exp) = (exponent(a), exponent(b));
    let sign = (a ^ b) & SIGN;
    let (mut a_sig, mut b_sig) = (fraction(a), fraction(b));
    let mut scale = 0i32;
    if (a_exp - 1) as u32 >= (MAX_EXP - 1) as u32 || (b_exp - 1) as u32 >= (MAX_EXP - 1) as u32 {
        let (a_abs, b_abs) = (a & ABS, b & ABS);
        if a_abs > INF {
            return a | QUIET;
        }
        if b_abs > INF {
            return b | QUIET;
        }
        if a_abs == INF {
            return if b_abs != ZERO { a_abs | sign } else { QNAN };
        }
        if b_abs == INF {
            return if a_abs != ZERO { b_abs | sign } else { QNAN };
        }
        if a_abs == ZERO || b_abs == ZERO {
            return sign;
        }
        if a_abs < IMPLICIT {
            scale += normalize(&mut a_sig);
        }
        if b_abs < IMPLICIT {
            scale += normalize(&mut b_sig);
        }
    }
    a_sig = a_sig | IMPLICIT;
    b_sig = b_sig | IMPLICIT;
    let (mut lo, mut hi) = product(a_sig, b_sig << EXP_BITS);
    let mut exp = a_exp + b_exp - BIAS + scale;
    if hi & IMPLICIT != ZERO {
        exp += 1;
    } else {
        hi = (hi << 1) | (lo >> 63);
        lo = lo << 1;
    }
    if exp >= MAX_EXP {
        return INF | sign;
    }
    if exp <= 0 {
        let shift = (1 - exp) as u32;
        if shift >= 64 {
            return sign;
        }
        let sticky = (lo << (64 - shift)) != ZERO;
        lo = (hi << (64 - shift)) | (lo >> shift) | W64::from_u32(sticky as u32);
        hi = hi >> shift;
    } else {
        hi = (hi & SIG_MASK) | exp_field(exp);
    }
    hi = hi | sign;
    if lo > SIGN {
        hi = hi + ONE_BIT;
    }
    if lo == SIGN {
        hi = hi + (hi & ONE_BIT);
    }
    hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_cases_match_native() {
        let cases = [(1.5, 2.0), (0.1, 0.2), (-3.0, 0.0), (5e-324, 0.5), (2.2250738585072014e-308, 0.5), (f64::MAX, 2.0), (1e-200, 1e-200), (f64::INFINITY, -1.0)];
        for (x, y) in cases {
            assert_eq!(bits(mul(from_f64(x), from_f64(y))), (x * y).to_bits(), "{x} * {y}");
        }
        assert!(is_nan(mul(from_f64(f64::INFINITY), from_f64(0.0))));
    }
}
