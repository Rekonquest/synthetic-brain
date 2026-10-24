//! OBS-1 (docs/OBS-1-PRECOMMIT.md): an antenna for rhythms. A bank of damped oscillators at
//! log-spaced angular frequencies (from w_lo to w_hi), each driven by the same signal:
//! x'' = −w²x − 2ζw·x' + input. Each band keeps a running average of its energy, ½(x'² + w²x²); a
//! rhythm in the signal pumps the band tuned to it. Mechanism only.

pub struct Bank {
    pub w: Vec<f64>,
    pub x: Vec<f64>,
    pub v: Vec<f64>,
    pub energy: Vec<f64>,
    pub zeta: f64,
    pub rate: f64,
}

impl Bank {
    pub fn new(bands: usize, w_lo: f64, w_hi: f64, zeta: f64, rate: f64) -> Bank {
        let w = (0..bands).map(|k| w_lo * (w_hi / w_lo).powf(k as f64 / (bands - 1).max(1) as f64)).collect();
        Bank { w, x: vec![0.0; bands], v: vec![0.0; bands], energy: vec![0.0; bands], zeta, rate }
    }

    /// One unit time step (semi-implicit Euler).
    pub fn drive(&mut self, input: f64) {
        for k in 0..self.w.len() {
            let w = self.w[k];
            self.v[k] += -w * w * self.x[k] - 2.0 * self.zeta * w * self.v[k] + input;
            self.x[k] += self.v[k];
            let e = 0.5 * (self.v[k] * self.v[k] + w * w * self.x[k] * self.x[k]);
            self.energy[k] += self.rate * (e - self.energy[k]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tuned_band_absorbs() {
        let mut b = Bank::new(8, 0.1, 1.0, 0.05, 0.01);
        for t in 0..3000 {
            b.drive((0.3 * t as f64).sin());
        }
        let best = (0..8).max_by(|&i, &j| b.energy[i].partial_cmp(&b.energy[j]).unwrap()).unwrap();
        assert!((b.w[best] - 0.3).abs() < 0.08, "best band {} at w {}", best, b.w[best]);
    }
}
