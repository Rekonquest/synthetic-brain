//! Thermal base, rung T5 (docs/T5-HOST.md): the port, where a module meets the medium.
//!
//! The medium holds one field per signal species on one shared lattice (chemistry's species: each
//! option of each signal is its own substance). A module's port injects its values as amounts of a
//! run of species at its site, and senses the amounts of a run of species there. Mechanism only:
//! where ports sit, what is sent, and how the medium carries it between steps (the bath, T2) are the
//! orchestrator's.

use thermo_lattice::Lattice;

/// The species fields of the medium.
#[derive(Debug, Clone)]
pub struct Medium {
    pub fields: Vec<Lattice>,
}

impl Medium {
    /// `species` zero fields on a lattice of `dims`.
    pub fn new(dims: &[usize], species: usize) -> Medium {
        Medium { fields: vec![Lattice::new(dims); species] }
    }

    /// Add `gain`·values[j] of species `first + j` at `site`.
    pub fn inject(&mut self, site: usize, first: usize, values: &[f64], gain: f64) {
        for (j, v) in values.iter().enumerate() {
            self.fields[first + j].e[site] += gain * v;
        }
    }

    /// The amounts of species `first .. first + n` at `site`.
    pub fn sense(&self, site: usize, first: usize, n: usize) -> Vec<f64> {
        (first..first + n).map(|s| self.fields[s].e[site]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_then_sense_round_trips() {
        let mut m = Medium::new(&[4, 4, 4], 6);
        m.inject(21, 2, &[0.25, 0.5, 0.125], 2.0);
        assert_eq!(m.sense(21, 2, 3), vec![0.5, 1.0, 0.25]);
    }

    #[test]
    fn injecting_one_run_leaves_every_other_species_and_site_untouched() {
        let mut m = Medium::new(&[4, 4, 4], 6);
        m.inject(21, 2, &[1.0, 1.0], 1.0);
        for (s, f) in m.fields.iter().enumerate() {
            for (site, &x) in f.e.iter().enumerate() {
                let hit = site == 21 && (s == 2 || s == 3);
                assert_eq!(x, if hit { 1.0 } else { 0.0 });
            }
        }
    }
}
