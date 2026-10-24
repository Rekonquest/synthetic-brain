//! ASTRO-3 (docs/ASTRO-POC.md): the Jeans drive. Jeans' dispersion relation w^2 = c^2 k^2 - 4 pi G rho
//! lets only modes longer than the Jeans length grow; shorter ones are held up by pressure and
//! disperse. The structure-forming drive is therefore the field with its short modes removed: here
//! the mean over an r^d box (r odd, centred) on a periodic lattice of `dims` (axis 0 fastest, as
//! thermo-lattice). r 1 is the identity (the local drive). Mechanism only: r is the caller's.

pub fn drive(field: &[f64], dims: &[usize], r: usize) -> Vec<f64> {
    assert!(r % 2 == 1, "the window is odd and centred");
    let mut out = field.to_vec();
    if r == 1 {
        return out;
    }
    let h = (r / 2) as isize;
    let mut stride = 1usize;
    for &l in dims {
        let old = out.clone();
        for (site, o) in out.iter_mut().enumerate() {
            let c = (site / stride) % l;
            let base = site - c * stride;
            let mut acc = 0.0;
            for d in -h..=h {
                let cc = (c as isize + d).rem_euclid(l as isize) as usize;
                acc += old[base + cc * stride];
            }
            *o = acc / r as f64;
        }
        stride *= l;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_at_one_and_a_point_spreads_over_the_box() {
        let mut f = vec![0.0; 36];
        f[7] = 9.0;
        assert_eq!(drive(&f, &[6, 6], 1), f);
        let g = drive(&f, &[6, 6], 3);
        // site 7 = (1, 1): its 3x3 box covers (0..2, 0..2) and (5, *) wraps; each gets 1.
        assert!((g.iter().sum::<f64>() - 9.0).abs() < 1e-12);
        assert!((g[0] - 1.0).abs() < 1e-12 && (g[14] - 1.0).abs() < 1e-12 && g[21].abs() < 1e-12);
        // a uniform field is unchanged (no Jeans swindle)
        let u = drive(&vec![0.4; 36], &[6, 6], 5);
        assert!(u.iter().all(|x| (x - 0.4).abs() < 1e-12));
    }
}
