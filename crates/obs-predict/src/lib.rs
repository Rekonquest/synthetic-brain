//! OBS-1 (docs/OBS-1-PRECOMMIT.md): predictive coding. A forecaster keeps a level and a trend (each
//! updated toward the evidence at rates alpha and beta) and the running variance of its own errors.
//! Each new reading returns (the forecast it made, the error, the surprise = error / √variance): the
//! error weighted by its expected precision. Mechanism only: the rates are the caller's.

pub struct Predictor {
    pub level: f64,
    pub trend: f64,
    pub var: f64,
    pub alpha: f64,
    pub beta: f64,
    pub var_rate: f64,
    started: bool,
}

impl Predictor {
    pub fn new(alpha: f64, beta: f64, var_rate: f64) -> Predictor {
        Predictor { level: 0.0, trend: 0.0, var: 1.0, alpha, beta, var_rate, started: false }
    }

    pub fn observe(&mut self, x: f64) -> (f64, f64, f64) {
        if !self.started {
            self.level = x;
            self.started = true;
            return (x, 0.0, 0.0);
        }
        let forecast = self.level + self.trend;
        let err = x - forecast;
        let surprise = err / self.var.sqrt().max(1e-12);
        let new_level = forecast + self.alpha * err;
        self.trend += self.beta * (new_level - self.level - self.trend);
        self.level = new_level;
        self.var += self.var_rate * (err * err - self.var);
        (forecast, err, surprise)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learns_a_ramp_and_is_surprised_by_a_step() {
        let mut p = Predictor::new(0.3, 0.1, 0.05);
        let mut last = 0.0;
        for t in 0..300 {
            last = p.observe(0.5 * t as f64).2;
        }
        assert!(last.abs() < 0.5);
        let s = p.observe(0.5 * 300.0 + 50.0).2;
        assert!(s > 5.0);
    }
}
