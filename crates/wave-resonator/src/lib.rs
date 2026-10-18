//! WAVE-1 (docs/WAVE-1-ANTENNA.md): the receiver. A damped oscillator x of natural angular
//! frequency ω_r and damping ζ, coupled to the medium's displacement u at its site by a spring κ_c
//! in both directions (the medium feels −κ_c(u − x), the oscillator +κ_c(u − x): equal and
//! opposite). Stepped in leapfrog alongside the medium. The energy its damping dissipates,
//! Σ 2ζω_r·ẋ²·dt, is the received energy — TA1's "the receiver absorbs it", in a wave medium.

#[derive(Debug, Clone, Copy)]
pub struct Resonator {
    pub x: f64,
    pub v: f64,
    pub omega: f64,
    pub zeta: f64,
    pub kappa: f64,
    /// Energy dissipated so far.
    pub absorbed: f64,
}

impl Resonator {
    pub fn new(freq: f64, zeta: f64, kappa: f64) -> Resonator {
        Resonator { x: 0.0, v: 0.0, omega: 2.0 * std::f64::consts::PI * freq, zeta, kappa, absorbed: 0.0 }
    }

    /// The force this receiver puts on the medium at its site.
    pub fn force_on_medium(&self, u: f64) -> f64 {
        -self.kappa * (u - self.x)
    }

    /// The kick: the oscillator's own spring, damping and the coupling; the dissipated energy.
    pub fn kick(&mut self, dt: f64, u: f64) {
        let damping = 2.0 * self.zeta * self.omega * self.v;
        self.absorbed += damping * self.v * dt;
        self.v += dt * (-self.omega * self.omega * self.x - damping + self.kappa * (u - self.x));
    }

    pub fn drift(&mut self, dt: f64) {
        self.x += dt * self.v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncoupled_it_absorbs_nothing_and_rings_at_its_frequency() {
        let mut r = Resonator::new(0.1, 0.0, 0.0);
        r.x = 1.0;
        let dt = 0.01;
        let mut crossings = Vec::new();
        let mut last = r.x;
        for n in 0..20_000 {
            r.kick(dt, 0.0);
            r.drift(dt);
            if last > 0.0 && r.x <= 0.0 {
                crossings.push(n as f64 * dt);
            }
            last = r.x;
        }
        let period = (crossings[crossings.len() - 1] - crossings[0]) / (crossings.len() - 1) as f64;
        assert!((period - 10.0).abs() < 0.01 && r.absorbed == 0.0, "{period}");
    }
}
