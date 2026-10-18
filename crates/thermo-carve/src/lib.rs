//! Thermal base, rung T3 (docs/T3-CARVE.md): threads carved into the medium by use.
//!
//! Law (the universum substrate's maintain-or-erase; Tero et al.'s Physarum adaptive network as prior
//! art): each edge's conductance c ∈ [c_min, 1] is maintained by the flux through it and erodes
//! without it:  c ← c + μ_e·g(|f|/f_ref)·(1 − c) − ν·(c − c_min),  g(x) = xᵖ/(1 + xᵖ).
//! p is the amplification (how strongly flow feeds conductance). Erosion is meant to be slower than
//! maintenance (ν < μ), a timescale rule learned in FABRIC-2. Mechanism only: the rates are the
//! caller's.

/// Edge conductances, indexed `site * d + axis` for the edge (site, forward(site, axis)).
#[derive(Debug, Clone, PartialEq)]
pub struct Carving {
    pub c: Vec<f64>,
}

pub fn g(x: f64, p: f64) -> f64 {
    let y = x.max(0.0).powf(p);
    y / (1.0 + y)
}

impl Carving {
    pub fn uniform(edges: usize, c0: f64) -> Carving {
        Carving { c: vec![c0; edges] }
    }

    /// One carving step from the edges' fluxes; `mu(edge)` is the maintenance rate per edge.
    pub fn carve(&mut self, flux: &[f64], mu: &dyn Fn(usize) -> f64, nu: f64, c_min: f64, p: f64, f_ref: f64) {
        for (e, c) in self.c.iter_mut().enumerate() {
            let grow = mu(e) * g(flux[e].abs() / f_ref, p) * (1.0 - *c);
            *c = (*c + grow - nu * (*c - c_min)).clamp(c_min, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_at_the_reference_and_decay_without_flux() {
        assert!((g(1.0, 1.0) - 0.5).abs() < 1e-15 && (g(1.0, 3.0) - 0.5).abs() < 1e-15);
        let mut k = Carving::uniform(1, 0.8);
        for _ in 0..100 {
            k.carve(&[0.0], &|_| 0.02, 0.002, 0.05, 1.0, 1.0);
        }
        let want = 0.05 + 0.75 * (1.0f64 - 0.002).powi(100);
        assert!((k.c[0] - want).abs() < 1e-12);
    }
}
