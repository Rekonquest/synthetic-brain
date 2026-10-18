//! GPU-1 (docs/GPU-1-CAP-F64.md): binary64 add and subtract from integer words, for a GPU with no
//! 64-bit floats. IEEE 754 round to nearest, ties to even; subnormals kept (no flush to zero);
//! signed zeros; infinities; NaN in → quiet NaN out (payload not part of the contract). The
//! algorithm is the standard soft-float one (three guard bits: guard, round, sticky), the same as
//! compiler-rt's `__adddf3`, written over `Wide<u32>` so only 32-bit integer operations remain.

use gpu_cap_binary64::*;

/// a + b.
pub fn add(a: W64, b: W64) -> W64 {
    let (mut a, mut b) = (a, b);
    let (a_abs, b_abs) = (a & ABS, b & ABS);
    // Zero, infinity or NaN on either side (the wrapping test catches 0 too).
    if a_abs - ONE_BIT >= INF - ONE_BIT || b_abs - ONE_BIT >= INF - ONE_BIT {
        if a_abs > INF {
            return a | QUIET;
        }
        if b_abs > INF {
            return b | QUIET;
        }
        if a_abs == INF {
            return if (a ^ b) == SIGN { QNAN } else { a };
        }
        if b_abs == INF {
            return b;
        }
        if a_abs == ZERO {
            return if b_abs == ZERO { a & b } else { b };
        }
        if b_abs == ZERO {
            return a;
        }
    }
    if b_abs > a_abs {
        std::mem::swap(&mut a, &mut b);
    }
    let (mut a_exp, mut b_exp) = (exponent(a), exponent(b));
    let (mut a_sig, mut b_sig) = (fraction(a), fraction(b));
    if a_exp == 0 {
        a_exp = normalize(&mut a_sig);
    }
    if b_exp == 0 {
        b_exp = normalize(&mut b_sig);
    }
    let sign = a & SIGN;
    let subtract = (a ^ b) & SIGN != ZERO;
    a_sig = (a_sig | IMPLICIT) << 3;
    b_sig = (b_sig | IMPLICIT) << 3;
    let align = (a_exp - b_exp) as u32;
    if align != 0 {
        if align < 64 {
            let sticky = (b_sig << (64 - align)) != ZERO;
            b_sig = (b_sig >> align) | W64::from_u32(sticky as u32);
        } else {
            b_sig = ONE_BIT;
        }
    }
    if subtract {
        a_sig = a_sig - b_sig;
        if a_sig == ZERO {
            return ZERO;
        }
        if a_sig < IMPLICIT << 3 {
            let shift = a_sig.lz() - (IMPLICIT << 3).lz();
            a_sig = a_sig << shift;
            a_exp -= shift as i32;
        }
    } else {
        a_sig = a_sig + b_sig;
        if a_sig & (IMPLICIT << 4) != ZERO {
            let sticky = a_sig & ONE_BIT;
            a_sig = (a_sig >> 1) | sticky;
            a_exp += 1;
        }
    }
    if a_exp >= MAX_EXP {
        return INF | sign;
    }
    if a_exp <= 0 {
        let shift = (1 - a_exp) as u32;
        if shift < 64 {
            let sticky = (a_sig << (64 - shift)) != ZERO;
            a_sig = (a_sig >> shift) | W64::from_u32(sticky as u32);
        } else {
            a_sig = W64::from_u32((a_sig != ZERO) as u32);
        }
        a_exp = 0;
    }
    let rgs = (a_sig & word(7)).low_u32();
    let mut result = ((a_sig >> 3) & SIG_MASK) | exp_field(a_exp) | sign;
    if rgs > 4 {
        result = result + ONE_BIT;
    }
    if rgs == 4 {
        result = result + (result & ONE_BIT);
    }
    result
}

/// a − b.
pub fn sub(a: W64, b: W64) -> W64 {
    add(a, b ^ SIGN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_cases_match_native() {
        let cases = [(1.0, 2.0), (0.1, 0.2), (1.0, -1.0), (-0.0, -0.0), (5e-324, 5e-324), (f64::MAX, f64::MAX), (1.0, 1e-17), (f64::INFINITY, -1.0)];
        for (x, y) in cases {
            assert_eq!(bits(add(from_f64(x), from_f64(y))), (x + y).to_bits(), "{x} + {y}");
            assert_eq!(bits(sub(from_f64(x), from_f64(y))), (x - y).to_bits(), "{x} - {y}");
        }
        assert!(is_nan(add(from_f64(f64::INFINITY), from_f64(f64::NEG_INFINITY))));
    }
}
