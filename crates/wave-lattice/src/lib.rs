//! WAVE-1 (docs/WAVE-1-ANTENNA.md): the wave medium. The scalar wave equation on T1's
//! d-dimensional periodic lattice, in leapfrog form with the velocity at half steps:
//!   kick:  v ← v + dt·(c²(site)·Σ_axes(u₊ + u₋ − 2u) − γ(site)·v + f(site))
//!   drift: u ← u + dt·v
//! The axis count never enters the law. Loss γ, force f and the per-site c² are the caller's (a
//! sponge layer, the transmitters, frozen disorder). With no loss and no force and a uniform c², the
//! leapfrog invariant H = ½·Σ v₋·v₊ + ½·c²·Σ_edges (Δu)² (v₋, v₊ the half steps either side of u) is
//! exactly conserved.

use thermo_lattice::Lattice;

/// The medium: displacement u and half-step velocity v, with a neighbour table (forward, backward
/// per axis) built once from the lattice.
pub struct Wave {
    pub u: Vec<f64>,
    pub v: Vec<f64>,
    pub nb: Vec<u32>,
    pub d: usize,
}

impl Wave {
    pub fn new(l: &Lattice) -> Wave {
        let d = l.dims.len();
        let mut nb = Vec::with_capacity(l.e.len() * 2 * d);
        for s in 0..l.e.len() {
            for a in 0..d {
                let st = l.stride(a);
                let back = if l.coord(s, a) == 0 { s + (l.dims[a] - 1) * st } else { s - st };
                nb.push(l.forward(s, a) as u32);
                nb.push(back as u32);
            }
        }
        Wave { u: vec![0.0; l.e.len()], v: vec![0.0; l.e.len()], nb, d }
    }

    /// Σ_axes(u₊ + u₋ − 2u) at a site.
    pub fn laplacian(&self, s: usize) -> f64 {
        let mut lap = 0.0;
        for a in 0..self.d {
            let (f, b) = (self.nb[s * 2 * self.d + 2 * a] as usize, self.nb[s * 2 * self.d + 2 * a + 1] as usize);
            lap += self.u[f] + self.u[b] - 2.0 * self.u[s];
        }
        lap
    }

    pub fn kick(&mut self, dt: f64, c2: &[f64], gamma: &[f64], force: &[f64]) {
        for s in 0..self.u.len() {
            let lap = self.laplacian(s);
            self.v[s] += dt * (c2[s] * lap - gamma[s] * self.v[s] + force[s]);
        }
    }

    pub fn drift(&mut self, dt: f64) {
        for (u, v) in self.u.iter_mut().zip(&self.v) {
            *u += dt * v;
        }
    }

    /// ½·c²·Σ_edges (Δu)², each edge once (forward neighbours only).
    pub fn strain(&self, c2: f64) -> f64 {
        let mut e = 0.0;
        for s in 0..self.u.len() {
            for a in 0..self.d {
                let f = self.nb[s * 2 * self.d + 2 * a] as usize;
                e += (self.u[f] - self.u[s]).powi(2);
            }
        }
        0.5 * c2 * e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_wrap_both_ways() {
        let l = Lattice::new(&[4, 3]);
        let w = Wave::new(&l);
        // Site 0 = (0, 0): forward x is 1, backward x is 3; forward y is 4, backward y is 8.
        assert_eq!(&w.nb[0..4], &[1, 3, 4, 8]);
    }
}
