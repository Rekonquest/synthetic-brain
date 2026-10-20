//! SCAR-1 run 4 (docs/SCAR-1-PRECOMMIT.md): one step of the band medium. Every band field diffuses
//! by T1's exchange law (band-blind), at κ·w_e with w_e = 1 − ½(m_i + m_j)·τ: carved trap material
//! lowers mobility. Then every site leaks, e ← e·(1 − δ(1 − τ·m_s)): material lowers the leak. A
//! band profile held in carved sites stays; the same profile in uncarved sites spreads and drains.

use thermo_host::Medium;

pub fn step(med: &mut Medium, material: &[f64], kappa: f64, delta: f64, tau: f64) {
    for f in med.fields.iter_mut() {
        let geo = f.clone();
        f.exchange(&|s, a| kappa * (1.0 - 0.5 * (material[s] + material[geo.forward(s, a)]) * tau));
        for (s, x) in f.e.iter_mut().enumerate() {
            *x *= 1.0 - delta * (1.0 - tau * material[s]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_is_conserved_by_exchange_and_lost_only_by_leak() {
        let mut m = Medium::new(&[6, 6], 2);
        m.fields[0].e[7] = 1.0;
        m.fields[1].e[20] = 2.0;
        let mat = vec![0.5; 36];
        step(&mut m, &mat, 0.2, 0.0, 0.9);
        assert!((m.fields[0].total() - 1.0).abs() < 1e-12 && (m.fields[1].total() - 2.0).abs() < 1e-12);
        step(&mut m, &mat, 0.2, 0.01, 0.9);
        assert!((m.fields[0].total() - (1.0 - 0.01 * (1.0 - 0.45))).abs() < 1e-12);
    }
}

/// The sealed variant (SCAR-1 run 5): mobility w_e = (1 − τ·m_i)(1 − τ·m_j), so *either* carved end
/// slows the flow and a carved region is sealed from its uncarved surroundings; the leak is unchanged.
pub fn step_sealed(med: &mut Medium, material: &[f64], kappa: f64, delta: f64, tau: f64) {
    for f in med.fields.iter_mut() {
        let geo = f.clone();
        f.exchange(&|s, a| kappa * (1.0 - tau * material[s]) * (1.0 - tau * material[geo.forward(s, a)]));
        for (s, x) in f.e.iter_mut().enumerate() {
            *x *= 1.0 - delta * (1.0 - tau * material[s]);
        }
    }
}
