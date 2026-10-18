//! GPU-1 (docs/GPU-1-CAP-F64.md): the contract — IEEE 754 binary64 held in a `Wide<u32>` (two
//! 32-bit words, as the Arc stores it): its fields, its constants, and the conversions to and from
//! host f64 bits (host-side only: constants and checks). Vocabulary only, no arithmetic policy.

pub use gpu_cap_wide::{Wide, Word};

/// A binary64 value's bits, as the GPU holds them.
pub type W64 = Wide<u32>;

pub const SIG_BITS: u32 = 52;
pub const EXP_BITS: u32 = 11;
pub const MAX_EXP: i32 = 0x7ff;
pub const BIAS: i32 = 1023;

/// A 64-bit constant as two 32-bit words.
pub const fn word(x: u64) -> W64 {
    Wide { hi: (x >> 32) as u32, lo: x as u32 }
}
/// The bits back as a host u64.
pub fn bits(w: W64) -> u64 {
    ((w.hi as u64) << 32) | w.lo as u64
}
pub fn from_f64(x: f64) -> W64 {
    word(x.to_bits())
}
pub fn to_f64(w: W64) -> f64 {
    f64::from_bits(bits(w))
}

pub const SIGN: W64 = word(1 << 63);
pub const ABS: W64 = word(!(1 << 63));
pub const INF: W64 = word(0x7ff0_0000_0000_0000);
pub const QUIET: W64 = word(1 << 51);
pub const SIG_MASK: W64 = word((1 << 52) - 1);
pub const IMPLICIT: W64 = word(1 << 52);
pub const ZERO: W64 = word(0);
pub const ONE_BIT: W64 = word(1);
/// The canonical quiet NaN this stack produces.
pub const QNAN: W64 = word(0x7ff8_0000_0000_0000);

/// The biased exponent field.
pub fn exponent(w: W64) -> i32 {
    ((w >> SIG_BITS) & word(0x7ff)).low_u32() as i32
}
/// The fraction field.
pub fn fraction(w: W64) -> W64 {
    w & SIG_MASK
}
pub fn is_nan(w: W64) -> bool {
    (w & ABS) > INF
}
/// Shift a subnormal significand up so its leading one sits at the implicit bit; the exponent it
/// then carries (1 − shift).
pub fn normalize(sig: &mut W64) -> i32 {
    let shift = sig.lz() - IMPLICIT.lz();
    *sig = *sig << shift;
    1 - shift as i32
}
/// An exponent as a word placed in the exponent field.
pub fn exp_field(e: i32) -> W64 {
    W64::from_u32(e as u32) << SIG_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip() {
        for x in [1.5f64, -2.0, f64::MIN_POSITIVE, 5e-324, f64::MAX, -0.0] {
            let w = from_f64(x);
            assert_eq!(to_f64(w).to_bits(), x.to_bits());
            assert_eq!(exponent(w) as u64, (x.to_bits() >> 52) & 0x7ff);
            assert_eq!(bits(fraction(w)), x.to_bits() & ((1 << 52) - 1));
        }
        assert!(is_nan(from_f64(f64::NAN)) && !is_nan(INF));
    }
}
