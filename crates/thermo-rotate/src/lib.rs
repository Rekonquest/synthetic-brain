//! Thermal base, rung T4 (docs/T4-ROTATE.md): rotate the field held in the medium.
//!
//! Law: advection by a rigid rotation of angle θ in the plane of axes (a, b) about the lattice
//! centre, semi-Lagrangian — each site takes the bilinearly interpolated value from the point the
//! rotation carries onto it — with every other axis carried unchanged. So the law is the same in any
//! dimension: a rotation acts in one plane. The total is renormalized after each step (interpolation
//! conserves it only approximately). Values from outside the lattice are 0 (the caller sizes the
//! lattice so none are needed).

use thermo_lattice::Lattice;

/// The field rotated by `angle` (radians) in the plane of axes `a` and `b`.
pub fn rotate(field: &Lattice, a: usize, b: usize, angle: f64) -> Vec<f64> {
    let (la, lb) = (field.dims[a] as f64, field.dims[b] as f64);
    let (ca, cb) = ((la - 1.0) / 2.0, (lb - 1.0) / 2.0);
    let (cos, sin) = (angle.cos(), angle.sin());
    let (sa, sb) = (field.stride(a), field.stride(b));
    let mut out = vec![0.0; field.e.len()];
    for (site, o) in out.iter_mut().enumerate() {
        let (x, y) = (field.coord(site, a) as f64 - ca, field.coord(site, b) as f64 - cb);
        // The source point: rotate the destination back by −angle.
        let (px, py) = (cos * x + sin * y + ca, -sin * x + cos * y + cb);
        let (x0, y0) = (px.floor(), py.floor());
        let (fx, fy) = (px - x0, py - y0);
        let base = site - field.coord(site, a) * sa - field.coord(site, b) * sb;
        let at = |i: f64, j: f64| -> f64 {
            if i < 0.0 || j < 0.0 || i >= la || j >= lb {
                0.0
            } else {
                field.e[base + i as usize * sa + j as usize * sb]
            }
        };
        *o = at(x0, y0) * (1.0 - fx) * (1.0 - fy) + at(x0 + 1.0, y0) * fx * (1.0 - fy) + at(x0, y0 + 1.0) * (1.0 - fx) * fy + at(x0 + 1.0, y0 + 1.0) * fx * fy;
    }
    let (before, after): (f64, f64) = (field.e.iter().sum(), out.iter().sum());
    if after != 0.0 {
        for o in out.iter_mut() {
            *o *= before / after;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quarter_turn_moves_a_point_and_a_full_turn_returns_it() {
        let mut l = Lattice::new(&[9, 9, 3]);
        let site = |x: usize, y: usize, z: usize| x + 9 * y + 81 * z;
        l.e[site(6, 4, 1)] = 1.0;
        let q = rotate(&l, 0, 1, std::f64::consts::FRAC_PI_2);
        // (6,4) about (4,4) by +90° in the (x, y) plane goes to (4,6); z is carried.
        assert!((q[site(4, 6, 1)] - 1.0).abs() < 1e-9, "{}", q[site(4, 6, 1)]);
        let mut f = l.clone();
        for _ in 0..4 {
            f.e = rotate(&f, 0, 1, std::f64::consts::FRAC_PI_2);
        }
        assert!((f.e[site(6, 4, 1)] - 1.0).abs() < 1e-9);
    }
}
