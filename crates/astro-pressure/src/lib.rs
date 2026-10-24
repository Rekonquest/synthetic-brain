//! ASTRO-1 arm B (docs/ASTRO-POC.md): degeneracy pressure. A state occupied by its own memory
//! repels a second occupant: the store moves along (own - rival) by x, s <- s + x * (own - rival),
//! which raises the own state's reading and lowers the rival's. Mechanism only: which states
//! collide and how hard they are pushed are the caller's.

use qb_amplitude::C;

pub fn push(s: &mut [C], own: &[C], rival: &[C], x: f64) {
    for ((a, o), r) in s.iter_mut().zip(own).zip(rival) {
        *a = C::new(a.re + x * (o.re - r.re), a.im + x * (o.im - r.im));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raises_own_and_lowers_rival() {
        let own = [C::new(1.0, 0.0), C::new(0.0, 0.0)];
        let rival = [C::new(0.0, 0.0), C::new(0.0, 1.0)];
        let mut s = [C::new(0.2, 0.0), C::new(0.0, 0.5)];
        push(&mut s, &own, &rival, 0.25);
        assert!((s[0].re - 0.45).abs() < 1e-12 && (s[1].im - 0.25).abs() < 1e-12);
        let mut t = s;
        push(&mut t, &own, &rival, 0.0);
        assert_eq!(t, s);
    }
}
