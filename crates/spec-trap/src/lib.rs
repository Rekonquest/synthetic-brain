//! SCAR-1 run 4 (docs/SCAR-1-PRECOMMIT.md): the trap material's maintain-or-erase. Held energy forms
//! material, m ← m + μ·g(E/E_ref)·(1 − m) − ν·m, with E the site's total band energy and g T3's
//! x^p/(1 + x^p). Without energy it erodes (ν). Per-site μ is the caller's (frozen chaos).

pub fn form(material: &mut [f64], energy: &[f64], mu: &dyn Fn(usize) -> f64, nu: f64, e_ref: f64, p: f64) {
    for (s, m) in material.iter_mut().enumerate() {
        let grow = mu(s) * thermo_carve::g(energy[s] / e_ref, p) * (1.0 - *m);
        *m = (*m + grow - nu * *m).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_with_energy_and_erodes_without() {
        let mut m = vec![0.0, 0.5];
        form(&mut m, &[1.0, 0.0], &|_| 0.02, 0.002, 1.0, 3.0);
        assert!((m[0] - 0.01).abs() < 1e-12 && (m[1] - 0.499).abs() < 1e-12);
    }
}
