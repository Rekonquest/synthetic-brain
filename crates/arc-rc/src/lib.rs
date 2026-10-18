//! SENSE-1 (docs/SENSE-1-RECEIVE.md): perceive the die's thermal law. The lumped law (one heat
//! capacity C, one resistance R to the surroundings at T_amb), C·dT/dt = P − (T − T_amb)/R, is
//! dT/dt = a·P + b·T + c with a = 1/C, b = −1/(RC), c = T_amb/(RC). It is fitted by least squares
//! over consecutive points (t s, T °C, P W); the law's parameters are read back from (a, b, c).
//! Whether the fit is trustworthy (the identifiability rule) is the orchestrator's call.

#[derive(Debug, Clone, Copy)]
pub struct Fit {
    pub c_j_per_k: f64,
    pub r_k_per_w: f64,
    pub t_amb_c: f64,
    pub r2: f64,
    pub points: usize,
}

/// Solve the 3×3 system m·x = v by Cramer's rule.
fn solve(m: [[f64; 3]; 3], v: [f64; 3]) -> Option<[f64; 3]> {
    let det = |m: [[f64; 3]; 3]| m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let d = det(m);
    if d.abs() < 1e-12 {
        return None;
    }
    let mut x = [0.0; 3];
    for (k, xk) in x.iter_mut().enumerate() {
        let mut mk = m;
        for row in 0..3 {
            mk[row][k] = v[row];
        }
        *xk = det(mk) / d;
    }
    Some(x)
}

/// Fit the law to points (t s, T °C, P W); None if the system is singular.
pub fn fit(points: &[(f64, f64, f64)]) -> Option<Fit> {
    let rows: Vec<([f64; 3], f64)> = points.windows(2).map(|w| ([w[0].2, w[0].1, 1.0], (w[1].1 - w[0].1) / (w[1].0 - w[0].0))).collect();
    if rows.len() < 3 {
        return None;
    }
    let mut m = [[0.0; 3]; 3];
    let mut v = [0.0; 3];
    for (x, y) in &rows {
        for i in 0..3 {
            v[i] += x[i] * y;
            for j in 0..3 {
                m[i][j] += x[i] * x[j];
            }
        }
    }
    let [a, b, c] = solve(m, v)?;
    let mean = rows.iter().map(|r| r.1).sum::<f64>() / rows.len() as f64;
    let (mut ss_res, mut ss_tot) = (0.0, 0.0);
    for (x, y) in &rows {
        let f = a * x[0] + b * x[1] + c * x[2];
        ss_res += (y - f).powi(2);
        ss_tot += (y - mean).powi(2);
    }
    let cap = 1.0 / a;
    Some(Fit { c_j_per_k: cap, r_k_per_w: -1.0 / (b * cap), t_amb_c: -c / b, r2: if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 }, points: rows.len() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_simulated_die_is_recovered() {
        // C 50 J/K, R 0.8 K/W, T_amb 24 °C, power stepping 15 ↔ 40 W, integrated finely.
        let (c, r, ta) = (50.0, 0.8, 24.0);
        let mut t = 30.0;
        let mut pts = Vec::new();
        for k in 0..2000 {
            let p = if (k / 200) % 2 == 0 { 15.0 } else { 40.0 };
            pts.push((k as f64 * 0.1, t, p));
            for _ in 0..100 {
                t += 0.001 * (p - (t - ta) / r) / c;
            }
        }
        let f = fit(&pts).unwrap();
        assert!((f.c_j_per_k - c).abs() < 1.0 && (f.r_k_per_w - r).abs() < 0.02 && (f.t_amb_c - ta).abs() < 0.3, "{f:?}");
    }
}
