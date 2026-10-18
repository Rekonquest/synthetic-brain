//! T2's bath (docs/T2-BATH.md), planned for speed. The same law, in the same floating-point
//! operation order, as `thermo-bath`'s `exchange` and `damp` with h = 0: the per-axis even/odd edge
//! sweeps become precomputed edge lists (no per-site coordinate division), and the noise is a
//! generic closure the compiler can inline (no dynamic call). `thermo-bath` stays the law; the tests
//! here hold this copy to it bit for bit, hot and cold. Composed from outside: nothing in the bath
//! changes.

use thermo_lattice::Lattice;

/// The edge sweeps of a lattice: per axis, the even-parity then odd-parity (site, forward) pairs,
/// each in increasing site order — the order `thermo_bath::exchange` visits them.
pub struct Plan {
    pub sweeps: Vec<Vec<(u32, u32)>>,
    pub sites: usize,
}

impl Plan {
    pub fn new(l: &Lattice) -> Plan {
        let mut sweeps = Vec::new();
        for axis in 0..l.dims.len() {
            if l.dims[axis] < 2 {
                continue;
            }
            for parity in [0, 1] {
                sweeps.push((0..l.e.len()).filter(|&s| l.coord(s, axis) % 2 == parity).map(|s| (s as u32, l.forward(s, axis) as u32)).collect());
            }
        }
        Plan { sweeps, sites: l.e.len() }
    }
}

/// `thermo_bath::exchange(field, temperature, kappa, &|_| 0.0, noise)`, planned. `noise(site, axis)`.
pub fn exchange<N: Fn(usize, usize) -> f64>(e: &mut [f64], plan: &Plan, temperature: f64, kappa: f64, noise: N) {
    let sigma = (temperature * kappa * (2.0 - kappa) / 2.0).sqrt();
    for (k, sweep) in plan.sweeps.iter().enumerate() {
        let axis = k / 2;
        for &(s, j) in sweep {
            let (s, j) = (s as usize, j as usize);
            let f = kappa / 2.0 * ((e[j] - 0.0) - (e[s] - 0.0)) + sigma * noise(s, axis);
            e[s] += f;
            e[j] -= f;
        }
    }
}

/// `thermo_bath::damp(field, temperature, &|_| gamma, &|_| 0.0, noise)`, planned, one γ for all sites.
pub fn damp<N: Fn(usize) -> f64>(e: &mut [f64], temperature: f64, gamma: f64, noise: N) {
    let a = (-gamma).exp();
    let c = (temperature * (1.0 - a * a)).sqrt();
    for (s, x) in e.iter_mut().enumerate() {
        *x = 0.0 + (*x - 0.0) * a + c * noise(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kick(key: u64) -> f64 {
        let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0) * 3f64.sqrt()
    }

    #[test]
    fn hot_and_cold_match_the_bath_bit_for_bit() {
        for (temp, scale) in [(0.0, 0.0), (1.0, 0.2)] {
            let mut slow = Lattice::new(&[16, 16, 16]);
            for (i, x) in slow.e.iter_mut().enumerate() {
                *x = if i % 7 == 0 { -0.0 } else { kick(i as u64) };
            }
            let plan = Plan::new(&slow);
            let mut fast = slow.e.clone();
            let sd: Vec<f64> = (0..slow.e.len() * 3).map(|k| scale * (1.0 + (k % 5) as f64)).collect();
            for t in 0..40u64 {
                let nz2 = |s: usize, ax: usize| sd[s * 3 + ax] * kick(t ^ ((s * 4 + ax) as u64).wrapping_mul(0x1656_67B1_9E37_79F9));
                let nz1 = |s: usize| sd[s * 3] * kick(t ^ ((s * 4 + 3) as u64).wrapping_mul(0x1656_67B1_9E37_79F9));
                thermo_bath::exchange(&mut slow, temp, 0.5, &|_| 0.0, &nz2);
                thermo_bath::damp(&mut slow, temp, &|_| 0.1, &|_| 0.0, &nz1);
                exchange(&mut fast, &plan, temp, 0.5, nz2);
                damp(&mut fast, temp, 0.1, nz1);
                assert!(slow.e.iter().zip(&fast).all(|(a, b)| a.to_bits() == b.to_bits()), "differs at step {t}, T {temp}");
            }
        }
    }
}
