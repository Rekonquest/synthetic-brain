//! Web-neuron: lane neurons (FABRIC-2) — one autocatalytic latch per lane with mutual inhibition
//! (QB-7's chemistry, which ties quantum collapse as a memory). A pulse writes a lane; the winner
//! holds without input; a stronger pulse to another lane switches it. The decision then *selects*
//! the winning lane's reading — never a mixture. Optional thermal floor (heat) lets a held lane be
//! dislodged more easily. Mechanism only.

#[derive(Debug, Clone, PartialEq)]
pub struct Lanes {
    pub b: Vec<f64>,
}

/// (K, δ, γ, dt): bistable at 0 and 1.5 with threshold ≈ 0.167 for K 0.5, δ 0.6.
pub const LATCH: (f64, f64, f64, f64) = (0.5, 0.6, 1.0, 0.5);

impl Lanes {
    /// `n` lanes, with lane `start` already on.
    pub fn new(n: usize, start: usize) -> Lanes {
        let mut b = vec![0.0; n];
        b[start] = 1.5;
        Lanes { b }
    }

    pub fn step(&mut self, pulse: Option<(usize, f64)>, heat: f64) {
        let (k, delta, gamma, dt) = LATCH;
        if let Some((lane, amount)) = pulse {
            self.b[lane] += amount;
        }
        let total: f64 = self.b.iter().sum();
        let old = self.b.clone();
        for (j, x) in self.b.iter_mut().enumerate() {
            let b = old[j];
            let db = b * b / (k * k + b * b) - delta * b - gamma * b * (total - b) + heat;
            *x = (b + dt * db).max(0.0);
        }
    }

    /// The winning lane.
    pub fn winner(&self) -> usize {
        let mut w = 0;
        for (j, x) in self.b.iter().enumerate() {
            if *x > self.b[w] {
                w = j;
            }
        }
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A2: a pulse switches the winner; the winner holds without input; selection is one lane only.
    #[test]
    fn switches_on_a_pulse_and_holds() {
        let mut l = Lanes::new(2, 0);
        for _ in 0..50 {
            l.step(None, 0.0);
            assert_eq!(l.winner(), 0);
        }
        l.step(Some((1, 2.0)), 0.0);
        for _ in 0..50 {
            l.step(None, 0.0);
            assert_eq!(l.winner(), 1);
        }
        assert!(l.b[0] < 1e-3 && (l.b[1] - 1.5).abs() < 0.05);
        // Selection: a reading chosen by the winner is exactly that lane's reading.
        let readings = [[0.9, 0.1], [0.2, 0.8]];
        assert_eq!(readings[l.winner()], [0.2, 0.8]);
    }
}
