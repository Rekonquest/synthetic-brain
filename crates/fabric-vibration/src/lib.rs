//! Spiderweb fabric — a vibration (FABRIC-1): the mismatch between how well two readings predicted
//! what was observed, as an EMA, mapped to a level in [0, 1]. Mechanism only; what the level drives
//! (here, a memory's heat) is the orchestrator's.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vibration {
    pub v: f64,
}

impl Vibration {
    /// Fold in one observation: `ahead` and `behind` are the probabilities two readings gave it.
    pub fn observe(&mut self, ahead: f64, behind: f64, rate: f64) {
        self.v = (1.0 - rate) * self.v + rate * (ahead - behind);
    }

    /// The level: v / scale, clamped to [0, 1].
    pub fn level(&self, scale: f64) -> f64 {
        (self.v / scale).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_predictions_do_not_vibrate() {
        let mut v = Vibration::default();
        for _ in 0..100 {
            v.observe(0.3, 0.3, 0.1);
        }
        assert_eq!(v.level(0.1), 0.0);
        for _ in 0..100 {
            v.observe(0.4, 0.1, 0.1);
        }
        assert!(v.level(0.1) > 0.99);
    }
}
