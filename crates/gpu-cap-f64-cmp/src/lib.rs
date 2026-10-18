//! GPU-1 (docs/GPU-1-CAP-F64.md): binary64 compare and max from integer words. IEEE ordering: NaN
//! is unordered (every comparison false), −0 equals +0. `max` follows Rust's `f64::max`: a NaN
//! operand yields the other; for −0 against +0 either may be returned (Rust leaves it unspecified).

use gpu_cap_binary64::*;

/// a < b.
pub fn lt(a: W64, b: W64) -> bool {
    if is_nan(a) || is_nan(b) {
        return false;
    }
    let (a_abs, b_abs) = (a & ABS, b & ABS);
    if a_abs == ZERO && b_abs == ZERO {
        return false;
    }
    let (a_neg, b_neg) = (a & SIGN != ZERO, b & SIGN != ZERO);
    if a_neg != b_neg {
        return a_neg;
    }
    if a_neg { a > b } else { a < b }
}

/// a == b.
pub fn eq(a: W64, b: W64) -> bool {
    if is_nan(a) || is_nan(b) {
        return false;
    }
    a == b || ((a | b) & ABS) == ZERO
}

/// The larger of a and b.
pub fn max(a: W64, b: W64) -> W64 {
    if is_nan(a) {
        return b;
    }
    if is_nan(b) {
        return a;
    }
    if lt(a, b) { b } else { a }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_matches_native() {
        let v = [-f64::INFINITY, -1.0, -5e-324, -0.0, 0.0, 5e-324, 1.0, f64::INFINITY, f64::NAN];
        for &x in &v {
            for &y in &v {
                assert_eq!(lt(from_f64(x), from_f64(y)), x < y, "{x} < {y}");
                assert_eq!(eq(from_f64(x), from_f64(y)), x == y, "{x} == {y}");
            }
        }
    }
}
