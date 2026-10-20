# FOLD-3: a containment reading for folded regions

Status: PRECOMMITTED (2026-10-20), before any FOLD-3 code.

FOLD-2 showed that the spectral bridge (whole-profile similarity) caps folded capacity, because a cue
is only 1/F of a folded region's profile. FOLD-3 composes a second reading over the same scars,
**containment**: is the cue's profile present in the region? The bridge is untouched and still read.

## The containment reading (a new crate, `spec-contain`)

1. Take the region's spectrum R. Its noise floor is the median of R over the bands (the stir's flat
   spectrum).
2. Subtract the floor: R′ = max(R − floor, 0).
3. **The score:** containment(c, R) = Σ_b c_b · R′_b / max_b R′_b, with c the normalised cue.
   - It is near the cue's own weight profile when every cue band is present at the strength of the
     region's strongest bands;
   - it is near 0 when the cue's bands are absent.
4. It does not shrink with F, because the region's other memories occupy *other* bands.

## The setup

FOLD-2's exactly (the same simulations, seeds and keys), with both readings computed from the same
spectra, at B ∈ {32, 64, 128} and F ∈ {1, 2, 4, 8, 16}.

## Predictions

1. **FX1, containment raises capacity:** at every B, the containment capacity F*_c is ≥ 2× the
   bridge's (≥ 4, ≥ 8 and ≥ 8 at 32, 64 and 128).
2. **FX2, containment scales with bands:** F*_c(128) ≥ 2·F*_c(32).
3. **FX3, nothing is lost at one per region:** at F 1, containment top-1 ≥ 0.95 at every B.

CPU only, 8 threads, about 15 min.
