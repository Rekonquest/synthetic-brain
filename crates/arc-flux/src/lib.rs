//! SENSE-1 (docs/SENSE-1-RECEIVE.md): perceive the Arc's heat flux. Energy-counter samples (t ns,
//! µJ) become power per window: at each window boundary the latest sample at or before it is taken,
//! and the power is the energy between two boundaries' samples over their own time difference.

/// (window end ns, watts) for windows of `window_ns` from the first sample.
pub fn power(samples: &[(u64, u64)], window_ns: u64) -> Vec<(u64, f64)> {
    let mut out = Vec::new();
    if samples.len() < 2 {
        return out;
    }
    let t0 = samples[0].0;
    let mut i = 0;
    let mut prev = samples[0];
    let mut boundary = t0 + window_ns;
    while boundary <= samples[samples.len() - 1].0 {
        while i + 1 < samples.len() && samples[i + 1].0 <= boundary {
            i += 1;
        }
        let cur = samples[i];
        if cur.0 > prev.0 {
            out.push((boundary, (cur.1 - prev.1) as f64 * 1e-6 / ((cur.0 - prev.0) as f64 * 1e-9)));
        }
        prev = cur;
        boundary += window_ns;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_steady_counter_gives_its_power() {
        // 20 W = 20 µJ per µs = 20_000 µJ per ms.
        let s: Vec<(u64, u64)> = (0..1000).map(|k| (k * 1_000_000, k * 20_000)).collect();
        let p = power(&s, 10_000_000);
        assert!(!p.is_empty() && p.iter().all(|x| (x.1 - 20.0).abs() < 1e-9));
    }
}
