//! GPU-1b (docs/GPU-1B-CAP-F128.md): host-side conversions between binary64 and binary128 (native
//! integer arithmetic; these run where the f64 values live). Widening is exact; narrowing rounds to
//! nearest, ties to even, with subnormal results and overflow to infinity; NaN stays NaN (quiet).

use gpu_cap_binary128::{bits, word, W128};

/// binary64 → binary128, exactly.
pub fn widen(x: f64) -> W128 {
    let b = x.to_bits();
    let s = ((b >> 63) as u128) << 127;
    let e = ((b >> 52) & 0x7ff) as i64;
    let f = b & ((1 << 52) - 1);
    if e == 0x7ff {
        return word(s | (0x7fff << 112) | ((f as u128) << 60));
    }
    if e == 0 {
        if f == 0 {
            return word(s);
        }
        let shift = f.leading_zeros() as i64 - 11;
        let m = f << shift;
        let e128 = (-1022 - shift) + 16383;
        return word(s | ((e128 as u128) << 112) | (((m & ((1 << 52) - 1)) as u128) << 60));
    }
    word(s | (((e - 1023 + 16383) as u128) << 112) | ((f as u128) << 60))
}

/// binary128 → binary64, round to nearest, ties to even.
pub fn narrow(w: W128) -> f64 {
    let a = bits(w);
    let s = ((a >> 127) as u64) << 63;
    let e = ((a >> 112) & 0x7fff) as i64;
    let frac = a & ((1 << 112) - 1);
    if e == 0x7fff {
        return f64::from_bits(if frac != 0 { s | 0x7ff8_0000_0000_0000 | ((frac >> 60) as u64) } else { s | 0x7ff0_0000_0000_0000 });
    }
    if e == 0 {
        return f64::from_bits(s); // every binary128 subnormal is far below binary64's range
    }
    let sig = frac | (1 << 112);
    let e64 = e - 16383 + 1023;
    if e64 >= 0x7ff {
        return f64::from_bits(s | 0x7ff0_0000_0000_0000);
    }
    // (the kept significand bits, whether to round up) for a right shift by `shift`.
    let round = |shift: u32| -> (u64, bool) {
        if shift >= 128 {
            return (0, false); // below half the smallest subnormal
        }
        let mant = (sig >> shift) as u64;
        let rem = sig & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        (mant, rem > half || (rem == half && mant & 1 == 1))
    };
    if e64 >= 1 {
        // The kept significand carries its leading bit at 52: drop it, add the exponent, then round
        // (a carry out of the fraction lands in the exponent, as it should).
        let (m, up) = round(60);
        return f64::from_bits(s | (((e64 as u64) << 52) + (m - (1 << 52)) + up as u64));
    }
    let (m, up) = round((60 + 1 - e64) as u32);
    f64::from_bits(s | (m + up as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rounds() {
        for x in [1.0f64, -2.5, 5e-324, f64::MIN_POSITIVE, f64::MAX, -0.0, f64::INFINITY, 0.1] {
            assert_eq!(narrow(widen(x)).to_bits(), x.to_bits(), "{x}");
        }
        // 1 + 2^-53 (an exact tie) narrows to 1 (even); 1 + 3·2^-54 narrows up.
        let one = 0x3fffu128 << 112;
        assert_eq!(narrow(word(one | (1 << 59))), 1.0);
        assert_eq!(narrow(word(one | (3 << 58))), 1.0 + f64::EPSILON);
    }
}
