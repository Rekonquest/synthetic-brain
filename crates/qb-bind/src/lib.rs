//! FOLD-6 (docs/FOLD-6-PRECOMMIT.md): binding, the elementwise product a_i·b_i of two amplitude
//! vectors. Binding a key to a value gives a vector unlike either; binding again with the key's
//! conjugate gives the value back (unit phasors). Superposed bindings are folded content. Mechanism
//! only: which vectors are bound, and conjugation, are the caller's.

use qb_amplitude::C;

pub fn bind(a: &[C], b: &[C]) -> Vec<C> {
    a.iter().zip(b).map(|(x, y)| x.mul(*y)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_then_unbind_with_the_conjugate() {
        let key = [C::polar(1.0, 0.7), C::polar(1.0, -2.1)];
        let value = [C::polar(1.0, 1.3), C::polar(1.0, 0.4)];
        let bound = bind(&key, &value);
        let conj: Vec<C> = key.iter().map(|c| C::new(c.re, -c.im)).collect();
        let back = bind(&bound, &conj);
        for (x, y) in back.iter().zip(&value) {
            assert!((x.re - y.re).abs() < 1e-12 && (x.im - y.im).abs() < 1e-12);
        }
    }
}
