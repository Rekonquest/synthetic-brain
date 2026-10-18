//! Web-neuron: a link that senses its own contribution (FABRIC-2).
//!
//! The thread starts dormant (g = 0). Each step it senses c = r_src[o] − r_sink[o]: did the reading it
//! carries predict the observation *better than its sink already did*? A source that only agrees
//! with the sink earns c ≈ 0, so agreeing is never credited. The thread is maintained by
//! contribution and erodes without it (the substrate's maintain-or-erase law):
//! g ← clamp(g + α·c, 0, 1) · (1 − δ). Mechanism only: the sink's use of g is the orchestrator's.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LinkNeuron {
    pub g: f64,
}

impl LinkNeuron {
    pub fn sense(&mut self, source: &[f64], sink: &[f64], observed: usize, alpha: f64, delta: f64) {
        let c = source[observed] - sink[observed];
        self.g = (self.g + alpha * c).clamp(0.0, 1.0) * (1.0 - delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A1: an agreeing source stays dormant; a source that contributes when the sink is wrong grows.
    #[test]
    fn contribution_grows_a_thread_and_agreement_does_not() {
        let (mut agree, mut contribute) = (LinkNeuron::default(), LinkNeuron::default());
        for t in 0..400 {
            let observed = if t % 2 == 0 { 1 } else { 2 };
            let sink = if observed == 1 { [0.0, 1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0, 0.0] }; // sink always says 1
            let same = sink;
            let better = [0.0, 0.0, 1.0, 0.0]; // right exactly when the sink is wrong (observed 2)
            agree.sense(&same, &sink, observed, 0.05, 0.01);
            if observed == 2 {
                contribute.sense(&better, &sink, observed, 0.05, 0.01);
            } else {
                contribute.sense(&sink, &sink, observed, 0.05, 0.01);
            }
        }
        assert_eq!(agree.g, 0.0);
        assert!(contribute.g > 0.5, "{}", contribute.g);
    }
}
