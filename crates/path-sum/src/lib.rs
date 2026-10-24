//! GEO-1 (docs/GEO-1-PRECOMMIT.md): one Jacobi sweep of the sum over paths on a w × h grid. Free
//! cells take u_s = (λ/4)·Σ (free 4-neighbours' u); the goal is held at 1; blocked cells stay 0. At
//! the fixed point u_s = Σ over paths from s to the goal of (λ/4)^length, so the shortest paths
//! dominate (least action). Returns the largest relative change. Mechanism only.

pub fn sweep(u: &mut [f64], free: &[bool], w: usize, h: usize, goal: usize, lambda: f64) -> f64 {
    let old = u.to_vec();
    let mut change: f64 = 0.0;
    for y in 0..h {
        for x in 0..w {
            let s = x + w * y;
            if !free[s] {
                continue;
            }
            if s == goal {
                u[s] = 1.0;
                continue;
            }
            let mut acc = 0.0;
            if x > 0 && free[s - 1] { acc += old[s - 1]; }
            if x + 1 < w && free[s + 1] { acc += old[s + 1]; }
            if y > 0 && free[s - w] { acc += old[s - w]; }
            if y + 1 < h && free[s + w] { acc += old[s + w]; }
            let v = lambda / 4.0 * acc;
            if v > 0.0 || old[s] > 0.0 {
                change = change.max(((v - old[s]) / v.max(old[s])).abs());
            }
            u[s] = v;
        }
    }
    change
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_corridor_decays_geometrically() {
        let (w, h) = (5, 1);
        let free = vec![true; 5];
        let mut u = vec![0.0; 5];
        for _ in 0..50 {
            sweep(&mut u, &free, w, h, 4, 0.5);
        }
        assert_eq!(u[4], 1.0);
        assert!(u[3] > u[2] && u[2] > u[1] && u[1] > u[0] && u[0] > 0.0);
    }
}
