//! BEAT-1 (docs/BEAT-1-HEARTBEAT.md): perceive band power from raw CU8 with a Hann-windowed FFT.
//! SENSE-1's boxcar leaked the strong channel into its comparison band (sinc sidelobes at −13 dB);
//! a Hann window's sidelobes fall to −31 dB and below, so bands stay apart. Each whole segment of
//! `n` samples (I, Q centred at 127.5, scaled by 1/128) is windowed and transformed, and for each
//! band (offset from the tuned centre, half-width) the bins inside it are summed.

/// In-place radix-2 FFT (forward), n a power of two.
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f64::consts::PI / len as f64;
        for s in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (wr, wi) = ((ang * k as f64).cos(), (ang * k as f64).sin());
                let (u, v) = (s + k, s + k + len / 2);
                let (tr, ti) = (re[v] * wr - im[v] * wi, re[v] * wi + im[v] * wr);
                re[v] = re[u] - tr;
                im[v] = im[u] - ti;
                re[u] += tr;
                im[u] += ti;
            }
        }
        len <<= 1;
    }
}

/// Per band, the summed bin power over all whole segments; and the number of segments.
pub fn band_powers(cu8: &[u8], n: usize, rate_hz: f64, bands: &[(f64, f64)]) -> (Vec<f64>, usize) {
    assert!(n.is_power_of_two());
    let win: Vec<f64> = (0..n).map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos()).collect();
    let bin = |k: usize| if k < n / 2 { k as f64 } else { k as f64 - n as f64 } * rate_hz / n as f64;
    let members: Vec<Vec<usize>> = bands.iter().map(|&(c, hw)| (0..n).filter(|&k| (bin(k) - c).abs() <= hw).collect()).collect();
    let mut out = vec![0.0; bands.len()];
    let (mut re, mut im) = (vec![0.0; n], vec![0.0; n]);
    let segs = cu8.len() / 2 / n;
    for s in 0..segs {
        for i in 0..n {
            let j = 2 * (s * n + i);
            re[i] = (cu8[j] as f64 - 127.5) / 128.0 * win[i];
            im[i] = (cu8[j + 1] as f64 - 127.5) / 128.0 * win[i];
        }
        fft(&mut re, &mut im);
        for (b, m) in members.iter().enumerate() {
            out[b] += m.iter().map(|&k| re[k] * re[k] + im[k] * im[k]).sum::<f64>();
        }
    }
    (out, segs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tone_stays_in_its_band() {
        let rate = 2_048_000.0;
        let bytes: Vec<u8> = (0..8192)
            .flat_map(|t| {
                let ph = 2.0 * std::f64::consts::PI * 300_000.0 * t as f64 / rate;
                [(127.5 + 100.0 * ph.cos()).round() as u8, (127.5 + 100.0 * ph.sin()).round() as u8]
            })
            .collect();
        let (p, segs) = band_powers(&bytes, 4096, rate, &[(300_000.0, 100_000.0), (-500_000.0, 100_000.0)]);
        assert_eq!(segs, 2);
        assert!(10.0 * (p[0] / p[1]).log10() > 40.0, "{p:?}");
    }
}
