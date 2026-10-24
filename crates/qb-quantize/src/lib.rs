//! FOLD-6 (docs/FOLD-6-PRECOMMIT.md): phase quantization. Each amplitude becomes unit magnitude at
//! the nearest of 2^b equally spaced phases (b 1: phases 0 and π, a sign), so it can be held in b
//! bits. The magnitude is dropped. A zero amplitude goes to phase 0. Mechanism only: b is the
//! caller's.

use qb_amplitude::C;

pub fn quantize(a: &mut [C], bits: u32) {
    let levels = 1u64 << bits;
    let step = 2.0 * std::f64::consts::PI / levels as f64;
    for c in a.iter_mut() {
        let k = (c.im.atan2(c.re) / step).round() as i64;
        let k = k.rem_euclid(levels as i64);
        *c = C::polar(1.0, k as f64 * step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_four_phases() {
        let mut a = [C::new(3.0, 0.5), C::new(-0.2, 0.1), C::new(0.0, 0.0)];
        quantize(&mut a, 1);
        assert!((a[0].re - 1.0).abs() < 1e-12 && (a[1].re + 1.0).abs() < 1e-12 && (a[2].re - 1.0).abs() < 1e-12);
        let mut b = [C::new(0.1, 2.0), C::new(0.0, -5.0)];
        quantize(&mut b, 2);
        assert!((b[0].im - 1.0).abs() < 1e-12 && (b[1].im + 1.0).abs() < 1e-12);
        assert!(b[0].re.abs() < 1e-12);
    }
}
