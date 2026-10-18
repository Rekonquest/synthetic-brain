//! Spiderweb fabric — threads form by use (FABRIC-1).
//!
//! Law (borrowed from the universum substrate's measured "maintain or erase"): a pathway persists
//! where traffic maintains it and erodes where it does not. Here the traffic that maintains a link is
//! a reading that *predicted* the next observation. Mechanism only:
//! - `Link::observe`: the score s ← (1 − ρ)·s + ρ·n·r[next], where n·r[next] is 1 at chance;
//! - `conductances`: a sink's inputs compete, g_e = s_eᵞ / Σ sᵞ (γ amplifies differences).
//!
//! Which links exist and which sinks they feed is the orchestrator's; nothing here decides.

/// One link's running predictive score (1 = chance).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Link {
    pub score: f64,
}

impl Default for Link {
    fn default() -> Self {
        Link { score: 1.0 }
    }
}

impl Link {
    /// Fold in how well the reading the link carried predicted `next`.
    pub fn observe(&mut self, reading: &[f64], next: usize, rate: f64) {
        let n = reading.len() as f64;
        self.score = (1.0 - rate) * self.score + rate * n * reading[next];
    }
}

/// The competitive conductances of a sink's inputs.
pub fn conductances(links: &[Link], gamma: f64) -> Vec<f64> {
    let w: Vec<f64> = links.iter().map(|l| l.score.max(1e-12).powf(gamma)).collect();
    let t: f64 = w.iter().sum();
    w.iter().map(|x| x / t).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_predictive_link_gains_against_one_that_is_not() {
        let (mut good, mut bad) = (Link::default(), Link::default());
        let sharp = [0.0, 1.0, 0.0, 0.0];
        let flat = [0.25; 4];
        for _ in 0..50 {
            good.observe(&sharp, 1, 0.1);
            bad.observe(&flat, 1, 0.1);
        }
        let g = conductances(&[good, bad], 2.0);
        assert!(g[0] > 0.9 && (g.iter().sum::<f64>() - 1.0).abs() < 1e-12, "{g:?}");
        assert!((bad.score - 1.0).abs() < 1e-9);
    }
}
