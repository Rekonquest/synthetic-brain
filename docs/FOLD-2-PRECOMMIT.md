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
