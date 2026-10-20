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

**A pre-run correction** (the unit test caught it before any result): the precommitted floor, *the
median band*, is only the noise floor while most bands are empty. In a folded region more than half
the bands can carry memories (6F of B), and then the median *is* the memory level, so the reading
collapses. The test's 8-band region with 4 bands active showed exactly that. **The floor is now the
least-filled band**, a band no stored memory uses. A run already started with the median was stopped
before it produced anything. The predictions are unchanged.

## Results (2026-10-20, `evidence/fold-3/results.txt`, 14 min 42 s on 8 threads)

**Region top-1, bridge / containment:**

| bands | F 2 | F 4 | F 8 | F 16 | capacity (bridge, containment) |
|---|---|---|---|---|---|
| 32 | 0.965 / 0.953 | 0.666 / **0.740** | 0.373 / **0.448** | 0.216 / 0.169 | 2, 2 |
| 64 | 0.980 / **1.000** | 0.850 / **0.924** | 0.488 / **0.615** | 0.286 / 0.202 | 4, 4 |
| 128 | 0.996 / **1.000** | 0.943 / **0.988** | 0.609 / **0.797** | 0.345 / 0.211 | 4, 4 |

| # | Result |
|---|---|
| FX1 containment ≥ 2× the bridge's capacity | **FAILED:** the capacities are equal at the 0.8 bar (128 bands at F 8 reach 0.797) |
| FX2 containment scales with bands | MEASURED: 2 → 4 |
| FX3 nothing lost at one per region | MEASURED: 1.000 at every B |

**2 of 3 hold.**

### What the folding line settles (FOLD-1, 2, 3)

- **Containment is the better reading for folded regions up to moderate depth:**
  - +0.07 to +0.19 at F 4 to 8, e.g. 128 bands at F 8: 0.609 → 0.797;
  - both readings fail at F 16;
  - the bridge stays right for one per region, as in C17.
- **The real ceiling is band occupancy,** not the reading. A region holding F keys of 6 bands fills
  about 1 − (1 − 6/B)^F of its bands. When other regions are that full, a cue's bands are "present"
  in them too, so identity blurs whatever the reading. At F 16 even 128 bands are about half full.
- **So capacity is set by how sparse the keys are against the dimension of the state.** The levers,
  in Jesse's terms:
  - growth (more bands, measured: 32 → 64 doubles capacity);
  - sparser keys (fewer active bands per memory);
  - orthogonal codes (vector-symbolic binding), the next FOLD experiment.
- **Measured so far:** 4 memories per region, 64 in a 422 KB medium, at ≥ 0.85 recall (64 or 128
  bands). Footprint per memory: about 6.6 KB.
