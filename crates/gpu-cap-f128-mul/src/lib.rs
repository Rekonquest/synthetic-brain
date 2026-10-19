//! GPU-1b (docs/GPU-1B-CAP-F128.md): binary128 multiply from 32-bit integer words. GPU-1's algorithm
//! (compiler-rt's soft-float multiply) at 128 bits. The 226-bit significand product is the
//! doubler's: `Wide` applied to the 128-bit word gives the 256-bit product — 32 → 64 → 128 → 256,
//! one law. `mul_by` keeps the product as a seam (another product can be composed in).

use gpu_cap_binary128::*;

pub type WideProduct = fn(W128, W128) -> (W128, W128);

pub fn doubled(a: W128, b: W128) -> (W128, W128) {
    a.mul_wide(b)
}

pub fn mul(a: W128, b: W128) -> W128 {
    mul_by(a, b, doubled)
}

pub fn mul_by(a: W128, b: W128, product: WideProduct) -> W128 {
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
    // a_sig in [2^112, 2^113), b_sig << 15 in [2^127, 2^128): the product's high word lies in
    // [2^111, 2^113), so the normalization test is bit 112 (GPU-1's lesson).
    let (mut lo, mut hi) = product(a_sig, b_sig << EXP_BITS);
    let mut exp = a_exp + b_exp - BIAS + scale;
    if hi & IMPLICIT != ZERO {
        exp += 1;
    } else {
        hi = (hi << 1) | (lo >> 127);
        lo = lo << 1;
    }
    if exp >= MAX_EXP {
        return INF | sign;
    }
    if exp <= 0 {
        let shift = (1 - exp) as u32;
        if shift >= 128 {
            return sign;
        }
        let sticky = (lo << (128 - shift)) != ZERO;
        lo = (hi << (128 - shift)) | (lo >> shift) | W128::from_u32(sticky as u32);
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
    fn two_times_three_is_six() {
        let (two, three) = (word(0x4000 << 112), word((0x4000 << 112) | (1 << 111)));
        assert_eq!(bits(mul(two, three)), (0x4001 << 112) | (1 << 111));
    }
}
