//! GPU-1b (docs/GPU-1B-CAP-F128.md): binary128 add and subtract from 32-bit integer words. GPU-1's
//! algorithm (compiler-rt's soft-float add, three guard bits) at 128 bits: round to nearest, ties
//! to even; subnormals kept; signed zeros; infinities; NaN → quiet NaN (payload not in the contract).

use gpu_cap_binary128::*;

pub fn add(a: W128, b: W128) -> W128 {
    let (mut a, mut b) = (a, b);
    let (a_abs, b_abs) = (a & ABS, b & ABS);
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
        if align < 128 {
            let sticky = (b_sig << (128 - align)) != ZERO;
            b_sig = (b_sig >> align) | W128::from_u32(sticky as u32);
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
        if shift < 128 {
            let sticky = (a_sig << (128 - shift)) != ZERO;
            a_sig = (a_sig >> shift) | W128::from_u32(sticky as u32);
        } else {
            a_sig = W128::from_u32((a_sig != ZERO) as u32);
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

pub fn sub(a: W128, b: W128) -> W128 {
    add(a, b ^ SIGN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_plus_one_is_two() {
        let one = word(0x3fff << 112);
        assert_eq!(bits(add(one, one)), 0x4000 << 112);
        assert_eq!(bits(sub(one, one)), 0);
    }
}
