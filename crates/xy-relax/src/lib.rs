//! PHYS-3 (docs/PHYS-POC.md): one relaxation step of a closed ring of phases toward its neighbours,
//! θ_i += J·(sin(θ_{i−1} − θ_i) + sin(θ_{i+1} − θ_i)), computed from the values before the step (the
//! XY model's gradient flow). Smooth: it can never change a winding number by itself while every
//! neighbour step stays below π. Mechanism only: J and the noise are the caller's.

pub fn relax(theta: &mut [f64], j: f64) {
    let n = theta.len();
    let old = theta.to_vec();
    for i in 0..n {
        let (l, r) = (old[(i + n - 1) % n], old[(i + 1) % n]);
        theta[i] += j * ((l - old[i]).sin() + (r - old[i]).sin());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bump_flattens_and_a_uniform_ring_stays() {
        let mut t = vec![0.0; 8];
        t[3] = 0.5;
        for _ in 0..500 {
            relax(&mut t, 0.2);
        }
        assert!(t.iter().all(|x| (x - t[0]).abs() < 1e-6));
        let mut u = vec![1.3; 8];
        relax(&mut u, 0.2);
        assert!(u.iter().all(|x| (x - 1.3).abs() < 1e-15));
    }
}
