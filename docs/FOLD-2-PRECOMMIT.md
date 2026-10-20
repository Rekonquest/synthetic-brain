# FOLD-2: the capacity law (memories per region against bands)

Status: PRECOMMITTED (2026-10-20), before any FOLD-2 code.

FOLD-1 found that a region holds 2 memories cleanly at 32 bands, and that 64 bands lift 4 per region
from 0.656 to 0.869. FOLD-2 asks how capacity scales with the dimension of the state.

**The setup:** FOLD-1's exactly (SCAR-1 run 5's laws, stirred, read at drain t 250, test seeds
10–17, keys of 6 active bands), for bands B ∈ {32, 64, 128} and fold depth F ∈ {1, 2, 4, 8, 16}.
The capacity F*(B) is the largest F with region top-1 ≥ 0.8.

## Predictions

1. **FC1, capacity grows linearly with bands:** F*(32) = 2, F*(64) = 4, F*(128) = 8.
   - **Why:** a region's mixture of F six-band keys stays distinguishable while 6F stays near B/2.5.
2. **FC2, scale invariance:** at the same load F/B (F = B/16, which is 2, 4 and 8), region top-1 at
   the three band counts lies within ±0.08 of their mean.
3. **FC3, contents track identity:** at every (B, F), set recall ≥ region top-1 − 0.1.

**Room:** CPU only, 8 threads, about 15–20 min. 128 bands cost 4× per step.

## Results (2026-10-20, `evidence/fold-2/results.txt`, 15 min on 8 threads)

**Region top-1 (set recall in brackets):**

| bands B | F 1 | F 2 | F 4 | F 8 | F 16 | capacity F* |
|---|---|---|---|---|---|---|
| 32 | 1.000 | 0.965 (0.992) | 0.666 (0.771) | 0.373 (0.524) | 0.216 (0.231) | **2** |
| 64 | 1.000 | 0.980 (1.000) | 0.850 (0.939) | 0.488 (0.672) | 0.286 (0.347) | **4** |
| 128 | 1.000 | 0.996 (1.000) | 0.943 (0.990) | 0.609 (0.866) | 0.345 (0.457) | **4** |

| # | Result |
|---|---|
| FC1 capacity linear in bands | **FAILED:** 2, 4, 4 (128 bands reach only 0.609 at F 8) |
| FC2 scale invariance at F = B/16 | **FAILED:** 0.965, 0.850, 0.609 (it falls with scale) |
| FC3 contents track identity | MEASURED |

**1 of 3 hold.**

### Why (after the run, not a prediction)

**The bridge compares whole profiles.** In a region holding F memories, a cue matches only 1/F of the
region's normalised profile, so its score falls like 1/F *whatever B is*. More bands only reduce the
chance overlaps with other regions, so the gain is sublinear.

The bridge is the right reading for *one memory per region*, its design case in C17. For a folded
region the question is different: *is this memory contained in the mixture?* That is a
**containment** reading, the membership test of Bloom filters or the unbinding of vector symbolic
memory.
