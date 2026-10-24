//! OBS-1 (docs/OBS-1-PRECOMMIT.md): ideas as phase codes. A memory superposes the codes of the ideas
//! it has seen; a new idea's overlap with it, Re⟨code, memory⟩ / (‖code‖·‖memory‖), is high for
//! familiar ideas and near zero for new ones. Mechanism only.

use qb_amplitude::C;

pub struct Memory {
    pub sum: Vec<C>,
}

impl Memory {
    pub fn new(d: usize) -> Memory {
        Memory { sum: vec![C::new(0.0, 0.0); d] }
    }
    pub fn add(&mut self, code: &[C], weight: f64) {
        for (s, c) in self.sum.iter_mut().zip(code) {
            *s = s.add(c.scale(weight));
        }
    }
    /// Multiply the whole memory by a factor (fading).
    pub fn fade(&mut self, f: f64) {
        for s in self.sum.iter_mut() {
            *s = s.scale(f);
        }
    }
    pub fn overlap(&self, code: &[C]) -> f64 {
        let (mut re, mut nc, mut nm) = (0.0, 0.0, 0.0);
        for (c, m) in code.iter().zip(&self.sum) {
            re += c.re * m.re + c.im * m.im;
            nc += c.norm2();
            nm += m.norm2();
        }
        if nc <= 0.0 || nm <= 0.0 { 0.0 } else { re / (nc * nm).sqrt() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn familiar_high_new_low() {
        let d = 512;
        let code = |s: u64| -> Vec<C> { (0..d).map(|i| C::polar(1.0, ((i as u64 * 2654435761 ^ s * 40503) % 6283) as f64 / 1000.0)).collect() };
        let mut m = Memory::new(d);
        for s in 0..4 {
            m.add(&code(s), 1.0);
        }
        assert!(m.overlap(&code(1)) > 0.4);
        assert!(m.overlap(&code(99)).abs() < 0.2);
    }
}
