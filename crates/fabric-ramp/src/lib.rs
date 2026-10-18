//! Spiderweb fabric — an on-ramp (FABRIC-1): a salient event is promoted to the highway as a broadcast
//! boost (1 + h) on its option, for every off-ramp to apply. With h = 0 nothing is promoted and the
//! event stays at ground level. Mechanism only.

/// The boost each option receives from the highway this step.
pub fn broadcast(event: Option<usize>, n: usize, h: f64) -> Vec<f64> {
    let mut b = vec![1.0; n];
    if let Some(e) = event {
        b[e] += h;
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotes_only_the_event() {
        assert_eq!(broadcast(Some(2), 4, 0.5), vec![1.0, 1.0, 1.5, 1.0]);
        assert_eq!(broadcast(None, 3, 0.5), vec![1.0; 3]);
        assert_eq!(broadcast(Some(0), 2, 0.0), vec![1.0, 1.0]);
    }
}
