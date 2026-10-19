//! GPU-1b (docs/GPU-1B-CAP-F128.md): binary128 compare and max (IEEE ordering; NaN unordered; −0 =
//! +0; max: a NaN operand yields the other, ±0 against ∓0 unspecified).

use gpu_cap_binary128::*;

pub fn lt(a: W128, b: W128) -> bool {
    if is_nan(a) || is_nan(b) {
        return false;
    }
    if (a & ABS) == ZERO && (b & ABS) == ZERO {
        return false;
    }
    let (an, bn) = (a & SIGN != ZERO, b & SIGN != ZERO);
    if an != bn {
        return an;
    }
    if an { a > b } else { a < b }
}

pub fn eq(a: W128, b: W128) -> bool {
    if is_nan(a) || is_nan(b) {
        return false;
    }
    a == b || ((a | b) & ABS) == ZERO
}

pub fn max(a: W128, b: W128) -> W128 {
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
    fn signs_and_zeros() {
        let (one, neg_one, z, nz) = (word(0x3fff << 112), word((0x3fff << 112) | (1 << 127)), ZERO, SIGN);
        assert!(lt(neg_one, one) && !lt(one, neg_one) && eq(z, nz) && !lt(z, nz) && !eq(QNAN, QNAN));
    }
}
