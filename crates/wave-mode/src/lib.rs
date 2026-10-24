//! PHYS-4 (docs/PHYS-POC.md): the projection of a 1D periodic field onto the standing mode
//! cos(2πn·x/L), Σ_x f(x)·cos(2πn·x/L)·2/L (the mode's amplitude). Mechanism only.

pub fn project(f: &[f64], n: usize) -> f64 {
    let l = f.len() as f64;
    f.iter().enumerate().map(|(x, v)| v * (2.0 * std::f64::consts::PI * n as f64 * x as f64 / l).cos()).sum::<f64>() * 2.0 / l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_out_its_own_mode() {
        let l = 64;
        let f: Vec<f64> = (0..l).map(|x| 0.7 * (2.0 * std::f64::consts::PI * 3.0 * x as f64 / l as f64).cos() - 0.2 * (2.0 * std::f64::consts::PI * 5.0 * x as f64 / l as f64).cos()).collect();
        assert!((project(&f, 3) - 0.7).abs() < 1e-12);
        assert!((project(&f, 5) + 0.2).abs() < 1e-12);
        assert!(project(&f, 4).abs() < 1e-12);
    }
}
