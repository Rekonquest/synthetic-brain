//! Thermal base, rung T4 run 2 (docs/T4-ROTATE.md): a band-limited shear of the field held in the
//! medium.
//!
//! Law: every line along axis `a` is shifted by s·(coord_b − c_b), c_b the centre of axis `b`, so
//! content at (x, y) moves to (x + s·y, y). The shift is band-limited: each line's DFT is multiplied
//! by e^{−ikδ} (the Nyquist bin by cos(πδ), which keeps the line real). So Σe is conserved exactly
//! (the DC bin is untouched) and Σe² loses only the Nyquist bin's share (cos(πδ) is the one factor
//! of modulus below 1); no other detail is lost, unlike interpolating advection. Lines
//! are periodic (the caller sizes the lattice so nothing reaches the wrap). Every other axis is
//! carried unchanged, so the law is the same in any dimension. Rotation is the orchestrator's
//! composition of three shears (Paeth 1986; Unser, Thévenaz & Yaroslavsky 1995), not this crate's.
//! Axis `a` must have a power-of-two length (fails closed otherwise).

use thermo_lattice::Lattice;

/// In-place radix-2 FFT of (re, im); `inverse` uses the conjugate twiddles and divides by n.
fn fft(re: &mut [f64], im: &mut [f64], inverse: bool) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let ang = sign * 2.0 * std::f64::consts::PI / len as f64;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (wr, wi) = ((ang * k as f64).cos(), (ang * k as f64).sin());
                let (u, v) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[v] * wr - im[v] * wi, re[v] * wi + im[v] * wr);
                re[v] = re[u] - tr;
                im[v] = im[u] - ti;
                re[u] += tr;
                im[u] += ti;
            }
        }
        len <<= 1;
    }
    if inverse {
        for i in 0..n {
            re[i] /= n as f64;
            im[i] /= n as f64;
        }
    }
}

/// The field with every line along `a` shifted by `s`·(coord_b − c_b).
pub fn shear(field: &Lattice, a: usize, b: usize, s: f64) -> Vec<f64> {
    let n = field.dims[a];
    assert!(n.is_power_of_two(), "thermo-shear: axis {a} has length {n}, not a power of two");
    let cb = (field.dims[b] as f64 - 1.0) / 2.0;
    let sa = field.stride(a);
    let mut out = field.e.clone();
    let (mut re, mut im) = (vec![0.0; n], vec![0.0; n]);
    for start in 0..field.e.len() {
        if field.coord(start, a) != 0 {
            continue;
        }
        let delta = s * (field.coord(start, b) as f64 - cb);
        if delta == 0.0 {
            continue;
        }
        for i in 0..n {
            re[i] = field.e[start + i * sa];
            im[i] = 0.0;
        }
        fft(&mut re, &mut im, false);
        for k in 0..n {
            let kk = if k <= n / 2 { k as f64 } else { k as f64 - n as f64 };
            let phase = -2.0 * std::f64::consts::PI * kk * delta / n as f64;
            if n % 2 == 0 && k == n / 2 {
                let c = (std::f64::consts::PI * delta).cos();
                re[k] *= c;
                im[k] *= c;
            } else {
                let (c, sn) = (phase.cos(), phase.sin());
                let (r, i) = (re[k] * c - im[k] * sn, re[k] * sn + im[k] * c);
                re[k] = r;
                im[k] = i;
            }
        }
        fft(&mut re, &mut im, true);
        for i in 0..n {
            out[start + i * sa] = re[i];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gauss(dims: &[usize], at: [f64; 3], sigma: f64) -> Lattice {
        let mut l = Lattice::new(dims);
        for s in 0..l.e.len() {
            let d2: f64 = (0..3).map(|a| (l.coord(s, a) as f64 - at[a]).powi(2)).sum();
            l.e[s] = (-d2 / (2.0 * sigma * sigma)).exp();
        }
        l
    }

    #[test]
    fn an_integer_shift_is_an_exact_roll() {
        let mut l = Lattice::new(&[8, 3, 1]);
        for x in 0..8 {
            l.e[x + 8 * 2] = x as f64 + 1.0; // the row y = 2, one above the centre y = 1
        }
        let out = shear(&l, 0, 1, 3.0); // shift 3·(2 − 1) = 3
        for x in 0..8 {
            assert!((out[(x + 3) % 8 + 16] - (x as f64 + 1.0)).abs() < 1e-12);
            assert!(out[x + 8].abs() < 1e-12 && out[x].abs() < 1e-12);
        }
    }

    #[test]
    fn a_fractional_shear_conserves_total_and_energy() {
        let l = gauss(&[16, 16, 4], [9.3, 6.1, 1.5], 1.6);
        let out = shear(&l, 0, 1, 0.137);
        let (t0, t1) = (l.e.iter().sum::<f64>(), out.iter().sum::<f64>());
        let (q0, q1) = (l.e.iter().map(|x| x * x).sum::<f64>(), out.iter().map(|x| x * x).sum::<f64>());
        // Σe exact; Σe² up to the Nyquist share (2.3e-10 relative for σ 1.6).
        assert!((t0 - t1).abs() < 1e-12 * t0 && (q0 - q1).abs() < 1e-8 * q0, "total {t0} -> {t1}, energy {q0} -> {q1}");
    }

    #[test]
    fn three_shears_rotate_a_band_limited_blob() {
        // Rotation by 40° about the centre (15.5, 15.5) as shear_x(−tan 20°) · shear_y(sin 40°) · shear_x(−tan 20°).
        let th = 40f64.to_radians();
        let l = gauss(&[32, 32, 1], [21.0, 15.5, 0.0], 2.0);
        let mut f = l.clone();
        f.e = shear(&f, 0, 1, -(th / 2.0).tan());
        f.e = shear(&f, 1, 0, th.sin());
        f.e = shear(&f, 0, 1, -(th / 2.0).tan());
        let (x, y) = (5.5 * th.cos() + 15.5, 5.5 * th.sin() + 15.5);
        let want = gauss(&[32, 32, 1], [x, y, 0.0], 2.0);
        let err = f.e.iter().zip(&want.e).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        assert!(err < 1e-5, "max error {err}"); // measured 1.6e-6 of a unit peak
    }
}
