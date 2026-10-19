//! GPU-1b (docs/GPU-1B-CAP-F128.md): the contract — IEEE 754 binary128 (sign 1, exponent 15 with
//! bias 16383, fraction 112) held in a `Wide<Wide<u32>>`, four 32-bit words, as a GPU with only
//! 32-bit integers would hold it. Fields, constants and host-side bit conversions; no arithmetic.

pub use gpu_cap_wide::{Wide, Word};

pub type W128 = Wide<Wide<u32>>;

pub const SIG_BITS: u32 = 112;
pub const EXP_BITS: u32 = 15;
pub const MAX_EXP: i32 = 0x7fff;
pub const BIAS: i32 = 16383;

pub const fn word(x: u128) -> W128 {
    Wide { hi: Wide { hi: (x >> 96) as u32, lo: (x >> 64) as u32 }, lo: Wide { hi: (x >> 32) as u32, lo: x as u32 } }
}
pub fn bits(w: W128) -> u128 {
    ((w.hi.hi as u128) << 96) | ((w.hi.lo as u128) << 64) | ((w.lo.hi as u128) << 32) | w.lo.lo as u128
}

pub const SIGN: W128 = word(1 << 127);
pub const ABS: W128 = word(!(1 << 127));
pub const INF: W128 = word(0x7fff << 112);
pub const QUIET: W128 = word(1 << 111);
pub const SIG_MASK: W128 = word((1 << 112) - 1);
pub const IMPLICIT: W128 = word(1 << 112);
pub const ZERO: W128 = word(0);
pub const ONE_BIT: W128 = word(1);
pub const QNAN: W128 = word((0x7fff << 112) | (1 << 111));

pub fn exponent(w: W128) -> i32 {
    ((w >> SIG_BITS) & word(0x7fff)).low_u32() as i32
}
pub fn fraction(w: W128) -> W128 {
    w & SIG_MASK
}
pub fn is_nan(w: W128) -> bool {
    (w & ABS) > INF
}
pub fn normalize(sig: &mut W128) -> i32 {
    let shift = sig.lz() - IMPLICIT.lz();
    *sig = *sig << shift;
    1 - shift as i32
}
pub fn exp_field(e: i32) -> W128 {
    W128::from_u32(e as u32) << SIG_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_round_trip() {
        for x in [0u128, 1, u128::MAX, 0x3ffd_5555_5555_5555_5555_5555_5555_5555] {
            assert_eq!(bits(word(x)), x);
        }
        assert_eq!(exponent(word(0x3fff << 112)), 0x3fff);
        assert!(is_nan(QNAN) && !is_nan(INF));
    }
}
