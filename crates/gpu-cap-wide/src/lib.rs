//! GPU-1 (docs/GPU-1-CAP-F64.md): the amplification adapter — growth by doubling a word's bits.
//!
//! `Wide<W>` is a 2w-bit word made of two w-bit words (`hi`, `lo`). It implements `Word` itself, so
//! the same law applies again: `Wide<u32>` is 64 bits from the Arc's native 32-bit words, and
//! `Wide<Wide<u32>>` is 128 bits. The base word's only non-trivial primitive is the full product
//! u32 × u32 → (lo, hi), SPIR-V's `OpUMulExtended`; everything wider is composed from it.
//! Arithmetic wraps (carries out of the top are returned by `adc`/`sbb`, dropped by the operators).
//! Shifts by ≥ the width give zero. Mechanism only: no word knows what its bits mean.

use std::ops::{Add, BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

/// A fixed-width unsigned word.
pub trait Word: Copy + Eq + Ord + std::fmt::Debug {
    const BITS: u32;
    fn zero() -> Self;
    fn from_u32(x: u32) -> Self;
    fn low_u32(self) -> u32;
    /// self + o + carry, and the carry out.
    fn adc(self, o: Self, carry: bool) -> (Self, bool);
    /// self − o − borrow, and the borrow out.
    fn sbb(self, o: Self, borrow: bool) -> (Self, bool);
    /// The full product, as (lo, hi).
    fn mul_wide(self, o: Self) -> (Self, Self);
    fn shl(self, n: u32) -> Self;
    fn shr(self, n: u32) -> Self;
    fn and(self, o: Self) -> Self;
    fn or(self, o: Self) -> Self;
    fn xor(self, o: Self) -> Self;
    fn not(self) -> Self;
    /// Leading zeros (BITS for zero).
    fn lz(self) -> u32;
}

impl Word for u32 {
    const BITS: u32 = 32;
    fn zero() -> u32 {
        0
    }
    fn from_u32(x: u32) -> u32 {
        x
    }
    fn low_u32(self) -> u32 {
        self
    }
    fn adc(self, o: u32, carry: bool) -> (u32, bool) {
        let (s, c1) = self.overflowing_add(o);
        let (s, c2) = s.overflowing_add(carry as u32);
        (s, c1 || c2)
    }
    fn sbb(self, o: u32, borrow: bool) -> (u32, bool) {
        let (s, b1) = self.overflowing_sub(o);
        let (s, b2) = s.overflowing_sub(borrow as u32);
        (s, b1 || b2)
    }
    fn mul_wide(self, o: u32) -> (u32, u32) {
        // OpUMulExtended on the GPU.
        let p = self as u64 * o as u64;
        (p as u32, (p >> 32) as u32)
    }
    fn shl(self, n: u32) -> u32 {
        if n >= 32 { 0 } else { self << n }
    }
    fn shr(self, n: u32) -> u32 {
        if n >= 32 { 0 } else { self >> n }
    }
    fn and(self, o: u32) -> u32 {
        self & o
    }
    fn or(self, o: u32) -> u32 {
        self | o
    }
    fn xor(self, o: u32) -> u32 {
        self ^ o
    }
    fn not(self) -> u32 {
        !self
    }
    fn lz(self) -> u32 {
        self.leading_zeros()
    }
}

/// Two words as one of twice the width. Field order makes the derived ordering numeric.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Wide<W> {
    pub hi: W,
    pub lo: W,
}

impl<W: Word> Word for Wide<W> {
    const BITS: u32 = 2 * W::BITS;
    fn zero() -> Self {
        Wide { hi: W::zero(), lo: W::zero() }
    }
    fn from_u32(x: u32) -> Self {
        Wide { hi: W::zero(), lo: W::from_u32(x) }
    }
    fn low_u32(self) -> u32 {
        self.lo.low_u32()
    }
    fn adc(self, o: Self, carry: bool) -> (Self, bool) {
        let (lo, c) = self.lo.adc(o.lo, carry);
        let (hi, c) = self.hi.adc(o.hi, c);
        (Wide { hi, lo }, c)
    }
    fn sbb(self, o: Self, borrow: bool) -> (Self, bool) {
        let (lo, b) = self.lo.sbb(o.lo, borrow);
        let (hi, b) = self.hi.sbb(o.hi, b);
        (Wide { hi, lo }, b)
    }
    fn mul_wide(self, o: Self) -> (Self, Self) {
        // Schoolbook over the half words: four partial products, then three carry-propagating limbs.
        let z = W::zero();
        let (p0l, p0h) = self.lo.mul_wide(o.lo);
        let (p1l, p1h) = self.lo.mul_wide(o.hi);
        let (p2l, p2h) = self.hi.mul_wide(o.lo);
        let (p3l, p3h) = self.hi.mul_wide(o.hi);
        let (s, c1) = p0h.adc(p1l, false);
        let (r1, c2) = s.adc(p2l, false);
        let (s, d1) = p1h.adc(p2h, false);
        let (s, d2) = s.adc(p3l, c1);
        let (r2, d3) = s.adc(z, c2);
        let (s, _) = p3h.adc(z, d1);
        let (s, _) = s.adc(z, d2);
        let (r3, _) = s.adc(z, d3);
        (Wide { hi: r1, lo: p0l }, Wide { hi: r3, lo: r2 })
    }
    fn shl(self, n: u32) -> Self {
        let w = W::BITS;
        if n >= 2 * w {
            Self::zero()
        } else if n == 0 {
            self
        } else if n >= w {
            Wide { hi: self.lo.shl(n - w), lo: W::zero() }
        } else {
            Wide { hi: self.hi.shl(n).or(self.lo.shr(w - n)), lo: self.lo.shl(n) }
        }
    }
    fn shr(self, n: u32) -> Self {
        let w = W::BITS;
        if n >= 2 * w {
            Self::zero()
        } else if n == 0 {
            self
        } else if n >= w {
            Wide { hi: W::zero(), lo: self.hi.shr(n - w) }
        } else {
            Wide { hi: self.hi.shr(n), lo: self.lo.shr(n).or(self.hi.shl(w - n)) }
        }
    }
    fn and(self, o: Self) -> Self {
        Wide { hi: self.hi.and(o.hi), lo: self.lo.and(o.lo) }
    }
    fn or(self, o: Self) -> Self {
        Wide { hi: self.hi.or(o.hi), lo: self.lo.or(o.lo) }
    }
    fn xor(self, o: Self) -> Self {
        Wide { hi: self.hi.xor(o.hi), lo: self.lo.xor(o.lo) }
    }
    fn not(self) -> Self {
        Wide { hi: self.hi.not(), lo: self.lo.not() }
    }
    fn lz(self) -> u32 {
        if self.hi != W::zero() { self.hi.lz() } else { W::BITS + self.lo.lz() }
    }
}

// Operators, so code over a wide word reads like code over a native one (wrapping).
impl<W: Word> Add for Wide<W> {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        self.adc(o, false).0
    }
}
impl<W: Word> Sub for Wide<W> {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self.sbb(o, false).0
    }
}
impl<W: Word> BitAnd for Wide<W> {
    type Output = Self;
    fn bitand(self, o: Self) -> Self {
        Word::and(self, o)
    }
}
impl<W: Word> BitOr for Wide<W> {
    type Output = Self;
    fn bitor(self, o: Self) -> Self {
        Word::or(self, o)
    }
}
impl<W: Word> BitXor for Wide<W> {
    type Output = Self;
    fn bitxor(self, o: Self) -> Self {
        Word::xor(self, o)
    }
}
impl<W: Word> Not for Wide<W> {
    type Output = Self;
    fn not(self) -> Self {
        Word::not(self)
    }
}
impl<W: Word> Shl<u32> for Wide<W> {
    type Output = Self;
    fn shl(self, n: u32) -> Self {
        Word::shl(self, n)
    }
}
impl<W: Word> Shr<u32> for Wide<W> {
    type Output = Self;
    fn shr(self, n: u32) -> Self {
        Word::shr(self, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w64(x: u64) -> Wide<u32> {
        Wide { hi: (x >> 32) as u32, lo: x as u32 }
    }
    fn n64(w: Wide<u32>) -> u64 {
        ((w.hi as u64) << 32) | w.lo as u64
    }

    #[test]
    fn carries_ripple_through_the_doubled_word() {
        assert_eq!(n64(w64(u64::MAX) + w64(1)), 0);
        assert_eq!(n64(w64(0xFFFF_FFFF) + w64(1)), 0x1_0000_0000);
        assert_eq!(n64(w64(0) - w64(1)), u64::MAX);
        let (lo, hi) = w64(u64::MAX).mul_wide(w64(u64::MAX));
        let p = u64::MAX as u128 * u64::MAX as u128;
        assert_eq!((n64(lo), n64(hi)), (p as u64, (p >> 64) as u64));
    }
}
