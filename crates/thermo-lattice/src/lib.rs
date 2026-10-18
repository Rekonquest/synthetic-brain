//! Thermal base, rung T1 (docs/T1-LATTICE.md): a periodic lattice of any dimension and the local
//! exchange law.
//!
//! Law: diffusion between nearest neighbours, one pass per axis, written once. On every edge
//! (i, i + a) the flux κ_e/2 · (e_{i+a} − e_i) moves energy from the fuller site to the emptier one,
//! computed from the values before the pass, so energy is conserved exactly up to rounding. The
//! number of axes appears only as the number of passes; it never enters the law, so dimensionality is
//! a property of the lattice (Jesse's C12 rule in the universum substrate). Mechanism only: κ is the
//! caller's, per edge and step if it wishes.

/// A periodic lattice with per-axis lengths `dims` and one energy per site.
#[derive(Debug, Clone, PartialEq)]
pub struct Lattice {
    pub dims: Vec<usize>,
    pub e: Vec<f64>,
}

impl Lattice {
    pub fn new(dims: &[usize]) -> Lattice {
        let n = dims.iter().product();
        Lattice { dims: dims.to_vec(), e: vec![0.0; n] }
    }

    /// The stride of `axis` in the flat index (axis 0 varies fastest).
    pub fn stride(&self, axis: usize) -> usize {
        self.dims[..axis].iter().product()
    }

    /// The coordinate of `site` along `axis`.
    pub fn coord(&self, site: usize, axis: usize) -> usize {
        (site / self.stride(axis)) % self.dims[axis]
    }

    /// The neighbour of `site` one step forward along `axis` (periodic).
    pub fn forward(&self, site: usize, axis: usize) -> usize {
        let (s, l) = (self.stride(axis), self.dims[axis]);
        let c = self.coord(site, axis);
        if c + 1 == l { site + s - l * s } else { site + s }
    }

    /// One exchange step: a pass per axis, κ given per (site, axis) for the edge (site, forward).
    pub fn exchange(&mut self, kappa: &dyn Fn(usize, usize) -> f64) {
        for axis in 0..self.dims.len() {
            if self.dims[axis] < 2 {
                continue;
            }
            let old = self.e.clone();
            for site in 0..old.len() {
                let j = self.forward(site, axis);
                let flux = kappa(site, axis) / 2.0 * (old[j] - old[site]);
                self.e[site] += flux;
                self.e[j] -= flux;
            }
        }
    }

    pub fn total(&self) -> f64 {
        self.e.iter().sum()
    }

    /// The energy-weighted mean squared minimum-image distance from `origin`, per axis.
    pub fn spread(&self, origin: usize) -> Vec<f64> {
        let t = self.total();
        (0..self.dims.len())
            .map(|axis| {
                let (l, o) = (self.dims[axis], self.coord(origin, axis));
                self.e.iter().enumerate().map(|(i, x)| {
                    let d = self.coord(i, axis).abs_diff(o);
                    let d = d.min(l - d) as f64;
                    x * d * d
                }).sum::<f64>() / t
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_wrap() {
        let l = Lattice::new(&[4, 3]);
        assert_eq!(l.forward(3, 0), 0);
        assert_eq!(l.forward(1, 1), 5);
        assert_eq!(l.forward(9, 1), 1);
    }

    /// The independent recomputation of L2: per axis, a pass with the kernel [κ/2, 1 − κ, κ/2] adds κ
    /// to the variance, so after t steps an impulse has per-axis variance κ·t (no wrap).
    #[test]
    fn spread_matches_the_random_walk_variance() {
        let mut l = Lattice::new(&[64, 64]);
        l.e[0] = 1.0;
        for _ in 0..30 {
            l.exchange(&|_, _| 0.2);
        }
        for m in l.spread(0) {
            assert!((m - 0.2 * 30.0).abs() < 1e-6, "{m}");
        }
    }
}
