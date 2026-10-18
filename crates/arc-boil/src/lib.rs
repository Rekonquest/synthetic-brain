//! SENSE-1 (docs/SENSE-1-RECEIVE.md): perceive the boiling point. A throttle flag going from lowered
//! to raised is an event — the moment the chip meets one of its limits, like a phase change.

/// Per flag (bit k of `flags`, `n` flags): the number of lowered → raised transitions over a series.
pub fn events(flags: &[u32], n: usize) -> Vec<usize> {
    let mut out = vec![0; n];
    for w in flags.windows(2) {
        for (k, e) in out.iter_mut().enumerate() {
            if w[0] & (1 << k) == 0 && w[1] & (1 << k) != 0 {
                *e += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_rising_edges_count() {
        assert_eq!(events(&[0b00, 0b01, 0b01, 0b00, 0b11, 0b10], 2), vec![2, 1]);
    }
}
