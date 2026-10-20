//! SCAR-1 run 4 (docs/SCAR-1-PRECOMMIT.md): the noise law as sparse kicks (the edge-of-chaos
//! maintainer: "sparse kicks plus activity-gated erosion"). Each step, each site with probability q
//! gains ε in one band chosen at random; keyed by (seed, step, site), so deterministic.

use thermo_host::Medium;

fn unit(key: u64) -> f64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

pub fn stir(med: &mut Medium, q: f64, eps: f64, seed: u64, t: usize) {
    let bands = med.fields.len();
    let sites = med.fields[0].e.len();
    for s in 0..sites {
        let key = seed.wrapping_mul(0xA24B_AED4_963E_E407) ^ (t as u64).wrapping_mul(0x9FB2_1C65_1E98_DF25) ^ (s as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
        if unit(key) < q {
            let b = ((unit(key ^ 0x5555_5555) * bands as f64) as usize).min(bands - 1);
            med.fields[b].e[s] += eps;
        }
    }
}
