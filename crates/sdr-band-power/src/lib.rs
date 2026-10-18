//! SENSE-1 (docs/SENSE-1-RECEIVE.md): perceive a band's power from raw CU8. Each complex sample
//! (I, Q bytes, centred at 127.5, scaled by 1/128) is mixed down by the band's offset with a phase
//! from its absolute sample index (continuous across frames), averaged over `taps` consecutive
//! samples (a boxcar low-pass, about rate/taps wide), and the block's |mean|² summed.

/// The band: its offset from the tuned centre, the sample rate, and the boxcar length.
#[derive(Debug, Clone, Copy)]
pub struct Band {
    pub offset_hz: f64,
    pub rate_hz: f64,
    pub taps: usize,
}

/// (sum of block powers, number of blocks) for the CU8 bytes starting at sample `first`.
pub fn power(cu8: &[u8], first: u64, band: &Band) -> (f64, usize) {
    let n = cu8.len() / 2;
    let w = -2.0 * std::f64::consts::PI * band.offset_hz / band.rate_hz;
    let (mut sum, mut blocks) = (0.0, 0);
    let mut k = 0;
    while k + band.taps <= n {
        let (mut re, mut im) = (0.0, 0.0);
        for j in k..k + band.taps {
            let (i, q) = ((cu8[2 * j] as f64 - 127.5) / 128.0, (cu8[2 * j + 1] as f64 - 127.5) / 128.0);
            let ph = w * ((first + j as u64) % (band.rate_hz as u64)) as f64;
            let (c, s) = (ph.cos(), ph.sin());
            re += i * c - q * s;
            im += i * s + q * c;
        }
        re /= band.taps as f64;
        im /= band.taps as f64;
        sum += re * re + im * im;
        blocks += 1;
        k += band.taps;
    }
    (sum, blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tone_at_the_offset_is_seen_there_and_not_elsewhere() {
        let rate = 2_048_000.0;
        let f = 300_000.0;
        let bytes: Vec<u8> = (0..65536)
            .flat_map(|n| {
                let ph = 2.0 * std::f64::consts::PI * f * n as f64 / rate;
                [(127.5 + 100.0 * ph.cos()).round() as u8, (127.5 + 100.0 * ph.sin()).round() as u8]
            })
            .collect();
        let on = power(&bytes, 0, &Band { offset_hz: f, rate_hz: rate, taps: 8 });
        let off = power(&bytes, 0, &Band { offset_hz: -500_000.0, rate_hz: rate, taps: 8 });
        assert!(on.0 / on.1 as f64 > 100.0 * off.0 / off.1 as f64);
    }
}
