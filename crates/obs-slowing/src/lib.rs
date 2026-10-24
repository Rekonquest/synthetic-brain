//! OBS-1 (docs/OBS-1-PRECOMMIT.md): critical slowing down. Before a system tips, its recovery from
//! small kicks slows: the lag-1 autocorrelation and the variance of its fluctuations rise.
//! `indicators` gives both for a window; `kendall` gives the trend of a series (Kendall's τ in
//! [−1, 1]). Mechanism only.

pub fn indicators(w: &[f64]) -> (f64, f64) {
    let n = w.len() as f64;
    let m = w.iter().sum::<f64>() / n;
    let var = w.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
    if var <= 0.0 {
        return (0.0, 0.0);
    }
    let cov = w.windows(2).map(|p| (p[0] - m) * (p[1] - m)).sum::<f64>() / (n - 1.0);
    (cov / var, var)
}

pub fn kendall(s: &[f64]) -> f64 {
    let n = s.len();
    if n < 2 {
        return 0.0;
    }
    let mut c = 0i64;
    for i in 0..n {
        for j in i + 1..n {
            c += (s[j] - s[i]).signum() as i64;
        }
    }
    c as f64 / (n * (n - 1) / 2) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rising_and_correlated() {
        assert_eq!(kendall(&[1.0, 2.0, 3.0, 4.0]), 1.0);
        assert_eq!(kendall(&[4.0, 3.0, 2.0, 1.0]), -1.0);
        let (ac, _) = indicators(&[1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
        assert!(ac > 0.3);
        let (ac2, _) = indicators(&[1.0, -1.0, 1.0, -1.0, 1.0, -1.0]);
        assert!(ac2 < -0.5);
    }
}
