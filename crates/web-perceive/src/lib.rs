//! Web-neuron: perception (FABRIC-2) — a named, revisable hypothesis with its evidence, never a score
//! or a decision (Jesse's PERCEPTION axiom). Here the hypothesis is "the memory is stale".
//! It turns on when the evidence (a mismatch level) exceeds θ; it is revised off when the evidence
//! falls below θ/2 or when the memory is rewritten (a fresh write refutes it). Hysteresis between θ/2
//! and θ keeps it from flickering. What the hypothesis drives is the orchestrator's.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hypothesis {
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perception {
    pub hypothesis: Hypothesis,
    /// The evidence behind the current hypothesis: the last level seen.
    pub evidence: f64,
}

impl Default for Perception {
    fn default() -> Self {
        Perception { hypothesis: Hypothesis::Fresh, evidence: 0.0 }
    }
}

impl Perception {
    /// Revise from one step's evidence. Returns true when the hypothesis changed.
    pub fn revise(&mut self, level: f64, rewritten: bool, theta: f64) -> bool {
        let before = self.hypothesis;
        self.evidence = level;
        self.hypothesis = match self.hypothesis {
            _ if rewritten => Hypothesis::Fresh,
            Hypothesis::Fresh if level > theta => Hypothesis::Stale,
            Hypothesis::Stale if level < theta / 2.0 => Hypothesis::Fresh,
            h => h,
        };
        self.hypothesis != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A3: on above θ, holds between θ/2 and θ, off below θ/2 or on a rewrite.
    #[test]
    fn hysteresis_and_refutation() {
        let mut p = Perception::default();
        assert!(!p.revise(0.3, false, 0.4));
        assert!(p.revise(0.5, false, 0.4) && p.hypothesis == Hypothesis::Stale);
        assert!(!p.revise(0.3, false, 0.4));
        assert_eq!(p.hypothesis, Hypothesis::Stale);
        assert!(p.revise(0.1, false, 0.4) && p.hypothesis == Hypothesis::Fresh);
        p.revise(0.9, false, 0.4);
        assert!(p.revise(0.9, true, 0.4) && p.hypothesis == Hypothesis::Fresh);
    }
}
