---
name: associative-folding
description: What was learned building associative ("folded") memory - many memories superposed in one fixed medium and recalled by association, not address - in Jesse's synthetic brain (FOLD-1..9, HYBRID-1, PROB-1). Covers capacity vs compression (folding costs space, buys damage tolerance and association), the thermal shape code, phase stores read phase-locked, 2-bit phase quantization, the explaining-away recall loop and its tipping point, the perception gate, ATTEND priors, self-calibrated probabilities, and the pitfalls. Use whenever Jesse mentions folding, scars, superposition memory, holographic/vector-symbolic memory, "recall by association", capacity, compression or "saving data"; when building any associative store, hashing/embedding memory, or cache of patterns; and before claiming a memory scheme saves space.
---

> **Drift note (2026-10-24):** much of the work behind this skill was engineered pipelines rather than Atom stacks (see the DRIFT WARNING in `complex-stack-composition`). The measurements are honest; rebuild mechanisms the Atom way before treating them as emergent.


# Associative folding — lessons

## The big three
1. **Capacity is not compression.** Recall that answers "which of 16?" returns ~4 bits; an index
   does that in a fraction of the space. Storing real bytes in a fold costs ~8 stored bytes per
   byte at best (1–2 bits per amplitude), 341× at f64; xz beats it ~19×. Never say folding "saves
   data" without the FOLD-6 measurement.
2. **What folding buys:** recall by association (a quarter of a cue → 0.984; half the cue wrong →
   0.982), records answered in any direction (1.000), damage tolerance (30% of the store destroyed →
   nothing lost), no growth when adding memories, graceful fading.
3. **Read the whole width with signed interference.** Gains over the thermal medium came from
   reading all dimensions with centred/phase-locked overlap, not from "quantum" as such (a
   non-negative classical twin nearly ties).

## Mechanisms and numbers
- **Thermal medium:** a fixed 422 KB; the *shape code* (8 sites × 6 bands) + cooling + gain × 2 →
  128 memories fresh (0.948) or 64 lasting (0.914). More bands raise per-region capacity but grow
  the medium (more bands = more bytes).
- **Phase store:** 4,096 memories in 422 KB at 0.947 read **phase-locked** (Re⟨c,a⟩); the magnitude
  reading |⟨c,a⟩| throws the phase away (0.866).
- **2-bit phase quantization** is the cheap precision; `quantize` keeps phase only (costs π/4 extra
  at any bit count). 4,096 memories in 6.6 KB at 0.838.
- **Explaining away** (PREDICT → GATE → REVISE with CONSERVE scaling; virial temperatures): 384 →
  0.999, 512 → 0.966 in 1 KB; **past ~60% first-read accuracy it backfires** (error propagation).
  CONSERVE (least-squares scale) is essential (without it recall → 0).
- **Perception gate** "store recoverable" (fraction of first margins above the noise floor of
  unwritten keys) keeps the gains and halves the loss; a *verify-after* gate is always fooled
  (cancellation raises margins right or wrong).
- **ATTEND + REINFORCE prior** turns redundancy into recall on real text (0.305 → 0.547 at depth);
  hurts on random data.
- **Maximum entropy probabilities** (T = σ²/μ, self-measured) calibrate without labels.
- **Roche placement:** keep weak memories out of regions with strong ones (capture effect).

## Pitfalls seen
- Chaos before a quantizer is inert when the selector is the quantizer's own objective.
- Precommitted thresholds grids that can never fire (check the reading's range first).
- Age vs occupancy: test which limits depth (diagnostic by write order).
- All f64 so far; 128-bit may matter where long chaotic/critical dynamics run.

Evidence: `~/Desktop/Projects/synthetic-brain/docs/FOLD-*`, `HYBRID-1`, `PROB-1`.

## Self-healing (HOLO-R, 2026-10-24)
Decode the whole store → perceive damaged regions by residual against the decode → mask them →
decode again → proofread (two disjoint halves agree) → replay the recalled values into the
dissolved regions. After losing 6 of 8 regions over three hits: 0.939 recall at N 192 (plain 0.077,
local layout 0.247); at N 256 the load limits rebuilding (0.282 after the third hit).
HOLO-R2: without the proofreading gate the decode-first healer reaches 1.000 (N 192) and 0.995
(N 256) after losing 6 of 8 regions over three hits; load-bearing: perception by residual, replay,
relaxation decoding, erasure masking, holographic layout; annealing repair survives a bad gate.
