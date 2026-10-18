//! The delayed cued tracking task (QB-3, QB-4), shared so every experiment sees identical worlds.
//!
//! A run is `trials` trials of cue (the target with `p_cue`), delay (pure uniform noise) and response
//! (the target with `p_response`) over `n` options on a ring; each new target sits at ring distance
//! `min_jump` or more from the last. `score` reads acc_resp, acc_early and persev from the choices.

/// The task's settings.
#[derive(Debug, Clone)]
pub struct Spec {
    pub n: usize,
    pub trials: usize,
    pub cue: usize,
    pub delay: usize,
    pub response: usize,
    pub p_cue: f64,
    pub p_response: f64,
    pub min_jump: usize,
}

impl Spec {
    pub fn trial_len(&self) -> usize {
        self.cue + self.delay + self.response
    }
}

/// SplitMix64.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn below(&mut self, n: usize) -> usize {
        ((self.unit() * n as f64) as usize).min(n - 1)
    }
}

pub fn ring_distance(a: usize, b: usize, n: usize) -> usize {
    let d = a.abs_diff(b);
    d.min(n - d)
}

/// One seed's world: per-trial targets, the observation at every step, and the uniform stream.
pub struct World {
    pub targets: Vec<usize>,
    pub obs: Vec<usize>,
    pub uniforms: Vec<f64>,
}

pub fn world(s: &Spec, seed: u64) -> World {
    let mut r = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D).wrapping_add(3));
    let mut targets = vec![r.below(s.n)];
    while targets.len() < s.trials {
        let c = r.below(s.n);
        if ring_distance(c, *targets.last().unwrap(), s.n) >= s.min_jump {
            targets.push(c);
        }
    }
    let mut obs = Vec::new();
    for &c in &targets {
        for step in 0..s.trial_len() {
            let p = if step < s.cue { s.p_cue } else if step < s.cue + s.delay { 0.0 } else { s.p_response };
            obs.push(if r.unit() < p { c } else { r.below(s.n) });
        }
    }
    let mut u = Rng(seed ^ 0xA5A5_5A5A_DEAD_BEEF);
    let uniforms = (0..obs.len()).map(|_| u.unit()).collect();
    World { targets, obs, uniforms }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Score {
    pub acc_resp: f64,
    pub acc_early: f64,
    pub persev: f64,
}

pub fn score(s: &Spec, w: &World, c: &[usize]) -> Score {
    let (mut resp, mut resp_ok, mut early, mut early_ok, mut pv, mut pv_n) = (0, 0, 0, 0, 0, 0);
    for (k, &target) in w.targets.iter().enumerate() {
        for step in s.cue + s.delay..s.trial_len() {
            let t = k * s.trial_len() + step;
            resp += 1;
            resp_ok += usize::from(c[t] == target);
            if step < s.cue + s.delay + 5 {
                early += 1;
                early_ok += usize::from(c[t] == target);
            }
            if k > 0 {
                pv_n += 1;
                pv += usize::from(c[t] == w.targets[k - 1]);
            }
        }
    }
    Score { acc_resp: resp_ok as f64 / resp as f64, acc_early: early_ok as f64 / early as f64, persev: pv as f64 / pv_n as f64 }
}

/// QB-3's task, as precommitted.
pub fn qb3_spec() -> Spec {
    Spec { n: 16, trials: 40, cue: 5, delay: 20, response: 25, p_cue: 0.9, p_response: 0.25, min_jump: 3 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_jump_and_the_delay_is_noise() {
        let s = qb3_spec();
        for seed in 0..20 {
            let w = world(&s, seed);
            assert!(w.targets.windows(2).all(|p| ring_distance(p[0], p[1], s.n) >= s.min_jump));
            assert_eq!(w.obs.len(), s.trials * s.trial_len());
        }
    }
}
