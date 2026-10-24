//! ASTRO-2 (docs/ASTRO-POC.md): a hypothesis sets its own temperature from its own binding. Over
//! its readings, the binding is the margin m between the best and the second best, and its noise
//! sigma is the standard deviation of every reading but the best. T = kappa * sigma^2 / m: tightly
//! bound is cold, loosely bound is hot (an unbound reading, m 0, is infinitely hot). Mechanism
//! only: kappa is the caller's.

pub fn temperature(readings: &[f64], kappa: f64) -> f64 {
    let n = readings.len();
    if n < 3 {
        return f64::INFINITY;
    }
    let mut b = 0usize;
    for v in 1..n {
        if readings[v] > readings[b] {
            b = v;
        }
    }
    let (mut second, mut sum, mut sum2) = (f64::NEG_INFINITY, 0.0, 0.0);
    for (v, &x) in readings.iter().enumerate() {
        if v != b {
            second = second.max(x);
            sum += x;
            sum2 += x * x;
        }
    }
    let k = (n - 1) as f64;
    let var = (sum2 / k - (sum / k).powi(2)).max(0.0);
    let m = readings[b] - second;
    if m <= 0.0 { f64::INFINITY } else { kappa * var / m }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound_is_cold_and_loose_is_hot() {
        let noise = [0.1, -0.1, 0.1, -0.1, 0.1, -0.1];
        let mut tight = noise.to_vec();
        tight.push(1.0);
        let mut loose = noise.to_vec();
        loose.push(0.2);
        let (tt, tl) = (temperature(&tight, 4.0), temperature(&loose, 4.0));
        // var of the six (and the 0.1 runner-up) is 0.01; margins 0.9 and 0.1.
        assert!((tt - 4.0 * 0.01 / 0.9).abs() < 1e-12 && (tl - 4.0 * 0.01 / 0.1).abs() < 1e-12);
        assert!(tt < tl);
        assert_eq!(temperature(&[0.3, 0.3, 0.0], 4.0), f64::INFINITY);
    }
}
