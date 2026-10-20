# FOLD-1: how deep can scars fold?

Status: PRECOMMITTED (2026-10-20), before any code.

## Jesse's lead

"Folding the scars might be the best." A scar takes no extra space, so folding more memories into the
same region spends only capacity. SCAR-1 run 5 showed that 2 per region still find their region at
0.914. FOLD-1 measures how deep folding goes, whether a region's contents can be read back, and
whether growth (more bands) raises the ceiling.

## The setup

SCAR-1 run 5's laws exactly:
- 32-band medium, 20 × 20 × 4, 16 sealed regions;
- `spec-step::step_sealed` τ 0.99, `spec-trap` E_ref 0.3, ν 5·10⁻⁵, μ_m 0.02;
- `spec-stir` q 0.02, ε 0.05, spectral-bridge recall;
- the kept **write gain 0.02** as plain.

**Folding depth F ∈ {1, 2, 4, 8}:** 16·F keys (6 active bands each) are written, F per region, one
after another (250 steps each), then the medium drains, stirred. The reads are at drain t 250 unless
stated.

**The measures:**
- **region top-1:** each key, used as the cue, finds its own region by the bridge;
- **set recall:** for each region, all 16·F keys are ranked by the bridge against the region's
  spectrum, and set recall is the fraction of its own F keys in the top F. This is "which memories
  does this region hold?";
- **footprint per memory:** the medium's bytes ÷ (16·F).

**Seeds:** test 10–17 (dev 0–3 for the round).

## Predictions (test seeds)

1. **F1, folding has a ceiling:** region top-1 is ≥ 0.9 at F 2, ≥ 0.7 at F 4, and **< 0.5 at F 8**.
   - Eight 6-band keys fill the 32-band space, so the region's mixture tends towards flat.
   - The capacity (the largest F with region top-1 ≥ 0.8) is reported.
2. **F2, a region's contents can be read at shallow depth:** set recall ≥ 0.8 at F 2, and it falls
   as F grows.
3. **F3, growth raises the ceiling** (amplification as "more of what exists"): with **64 bands**, F 4's
   region top-1 is ≥ the 32-band value + 0.1.
4. **F4, footprint:** the medium's bytes are fixed, so the bytes per memory halve with each doubling
   of F. This is reported, and it is exact by construction.
5. **F5, folded memories live as long:** at F 2, stirred, region top-1 at t 20,000 is ≥ 0.75. Noise
   keeps folded memories alive too.

**Rounds:** the 64-band arm (F3) is the amplification test. Chaos and heat were settled on this
stack in SCAR-1 run 5 (neither kept), and the fold depth is a measurement, not a new stack.

CPU only, 8 threads, a few minutes.

## Results (2026-10-20, `evidence/fold-1/results.txt`, 2 min on 8 threads)

| memories per region (F) | memories in the medium | region top-1 | set recall | bytes per memory |
|---|---|---|---|---|
| 1 | 16 | 1.000 | 1.000 | 26,400 |
| 2 | 32 | **0.938** | **0.996** | 13,200 |
| 4 | 64 | 0.656 | 0.730 | 6,600 |
| 8 | 128 | 0.355 | 0.502 | 3,300 |
| **4, at 64 bands** | 64 | **0.869** | **0.949** | — |

**F 2's lifetime:** stirred, region top-1 is **0.812** at t 20,000.

| # | Result |
|---|---|
| F1 a ceiling (F 2 ≥ 0.9, F 4 ≥ 0.7, F 8 < 0.5) | **FAILED by margin:** F 4 is 0.656 (bar 0.7). F 2 (0.938) and F 8 (0.355) hold. The capacity at 32 bands is **F 2** |
| F2 contents readable at F 2, and falling with F | MEASURED: **0.996** at F 2, then 0.730 and 0.502 |
| F3 growth (64 bands) raises the ceiling | **MEASURED: F 4 goes 0.656 → 0.869 (+0.213)**, set recall 0.730 → 0.949 |
| F4 footprint per memory halves with each doubling | MEASURED (by construction): a fixed 422,400 bytes for 16 to 128 memories |
| F5 folded memories live | MEASURED: 0.812 at t 20,000 |

**4 of 5 hold.**

### What it settles

- **Folding is real, and it has a ceiling set by the band space.**
  - At 32 bands a region holds 2 memories cleanly: each finds its region at 0.938, and the region's
    contents read back at 0.996, so it is clear *which* memories a region holds.
  - At 4 and 8, the mixtures crowd the 32 bands, and identity falls.
- **Growth raises the ceiling.** Doubling the bands to 64 lifts 4-per-region from 0.656 to 0.869,
  and its contents from 0.730 to 0.949. This is Jesse's amplification as "more of what exists":
  capacity grows with the dimension of the state, not with more space.
- **Folded memories live as long:** noise keeps them alive at 0.812 after 20,000 steps.
- **The footprint per memory falls as folding deepens.** The medium never grows; folding spends
  capacity, as Jesse said.
- **The next measurement:** how capacity scales with bands (32 → 64 → 128), the capacity law.
