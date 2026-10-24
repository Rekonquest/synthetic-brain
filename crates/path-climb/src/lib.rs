//! GEO-1 (docs/GEO-1-PRECOMMIT.md): climb a field on a w × h grid. From the start, step to the free
//! 4-neighbour with the largest u (ties broken by `tie(step, candidates)` when given) until the goal,
//! with no revisits and at most `cap` steps. Returns the path length, or None. Mechanism only.

pub fn climb(u: &[f64], free: &[bool], w: usize, h: usize, start: usize, goal: usize, cap: usize, tie: Option<&dyn Fn(usize, usize) -> usize>) -> Option<usize> {
    let mut seen = vec![false; u.len()];
    let mut s = start;
    seen[s] = true;
    for step in 0..cap {
        if s == goal {
            return Some(step);
        }
        let (x, y) = (s % w, s / w);
        let mut nb = Vec::new();
        if x > 0 { nb.push(s - 1); }
        if x + 1 < w { nb.push(s + 1); }
        if y > 0 { nb.push(s - w); }
        if y + 1 < h { nb.push(s + w); }
        let cand: Vec<usize> = nb.into_iter().filter(|&t| free[t] && !seen[t]).collect();
        if cand.is_empty() {
            return None;
        }
        let best = cand.iter().map(|&t| u[t]).fold(f64::MIN, f64::max);
        let tied: Vec<usize> = cand.into_iter().filter(|&t| u[t] >= best - best.abs() * 1e-12).collect();
        s = match tie {
            Some(f) if tied.len() > 1 => tied[f(step, tied.len()) % tied.len()],
            _ => tied[0],
        };
        seen[s] = true;
    }
    if s == goal { Some(cap) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn climbs_a_ramp() {
        let u = [0.1, 0.2, 0.3, 1.0];
        assert_eq!(climb(&u, &[true; 4], 4, 1, 0, 3, 10, None), Some(3));
        assert_eq!(climb(&u, &[true, true, false, true], 4, 1, 0, 3, 10, None), None);
        let neg = [-3.0, -2.0, -1.0, 0.0];
        assert_eq!(climb(&neg, &[true; 4], 4, 1, 0, 3, 10, None), Some(3));
        let neg2 = [-4.0, -3.0, -2.0, -1.0];
        assert_eq!(climb(&neg2, &[true; 4], 4, 1, 0, 3, 10, None), Some(3));
    }
}
