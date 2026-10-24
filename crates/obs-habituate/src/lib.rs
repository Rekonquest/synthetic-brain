//! OBS-1 (docs/OBS-1-PRECOMMIT.md): habituation. Each kind of stimulus has a sensitivity in (0, 1].
//! A response is the stimulus times its sensitivity. Each response lowers that kind's sensitivity
//! (× (1 − depress)); every step without it recovers toward 1 (+ recover·(1 − s)). Kinds are
//! independent, so a new kind meets full sensitivity (sensitization by contrast). Mechanism only.

pub struct Habituation {
    pub s: Vec<f64>,
    pub depress: f64,
    pub recover: f64,
}

impl Habituation {
    pub fn new(kinds: usize, depress: f64, recover: f64) -> Habituation {
        Habituation { s: vec![1.0; kinds], depress, recover }
    }
    pub fn step(&mut self) {
        for s in self.s.iter_mut() {
            *s += self.recover * (1.0 - *s);
        }
    }
    pub fn respond(&mut self, kind: usize, stimulus: f64) -> f64 {
        let r = stimulus * self.s[kind];
        self.s[kind] *= 1.0 - self.depress;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_fade_and_a_new_kind_is_fresh() {
        let mut h = Habituation::new(2, 0.2, 0.0);
        let first = h.respond(0, 1.0);
        for _ in 0..10 {
            h.respond(0, 1.0);
        }
        assert!(h.respond(0, 1.0) < 0.2 * first);
        assert_eq!(h.respond(1, 1.0), 1.0);
    }
}
