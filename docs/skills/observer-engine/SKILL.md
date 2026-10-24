---
name: observer-engine
description: How to build an observer that senses data or ideas, perceives named behaviours (citing evidence) and reacts - Jesse's sensing → perception → judgment separation applied to streams of numbers, symbols ("ideas") and the brain's own hypotheses ("thoughts"). Records what OBS-1 measured, including why it failed (the sensing did not separate behaviours) and the fixes. Use whenever Jesse asks for something that can "observe", "measure ideas or data", "detect behaviours", "react", monitor a stack, watch for drift, tipping points, oscillation, novelty, contradiction, false certainty or collapse; and for any monitoring/anomaly/change-detection design in any project.
---

> **Drift note (2026-10-24):** much of the work behind this skill was engineered pipelines rather than Atom stacks (see the DRIFT WARNING in `complex-stack-composition`). The measurements are honest; rebuild mechanisms the Atom way before treating them as emergent.


# Observer engines

## Architecture (his class separation)
Sensing (readings, no decisions) → PERCEPTION (named, retractable hypotheses citing their readings,
no fused score) → judgment (a thin labelled reaction layer). "Ideas" means all of: patterns in data,
symbols/concepts (phase codes, no symbol table), and the brain's own thoughts (hypotheses and their
confidence).

## OBS-1 (1 of 8) — the lesson: sensing must separate behaviours first
Measured: plain noise looked *more* alarming than planted behaviours on every reading (max
surprise 3.56 none vs 3.11 step; resonator ratio 10.0 vs 7.1; slowing evidence 0.68 vs 0.68).
Before tuning thresholds, check that each reading's distribution on "none" is separated from its
behaviour's. Gaps found:
1. Single-step surprise is swamped by chance maxima over long quiet streams → use **evidence
   accumulation** (a drift-diffusion / leaky integrator of signed surprise, neuroscience).
2. A resonator bank under red noise is dominated by slow bands → **per-band adaptation** (each
   band against its own baseline: sensory adaptation).
3. Kendall τ over 20 points is too noisy for early warning → longer baseline and contrast.
4. Rules chosen one at a time add their false alarms → choose thresholds **jointly** under one
   false-alarm budget.
5. Amplifying the surprise under fixed thresholds makes everything fire → re-choose thresholds.

## What worked
- **Thoughts observer:** false certainty (abrupt entropy drop followed by a top-answer change)
  0.998 where entropy thresholds score 0.000; healthy settling left unflagged (0.998).
- **Ideas on phase codes (no ids):** novelty 0.985, contradiction (unbind a NOT role) 0.950;
  repetition 0.875.
- **Habituation** silences a repeated benign rhythm completely; keep kinds independent.
- Critical slowing down (rising autocorrelation and variance) is the right physics for tipping
  points, but needs the fixes above to beat a variance threshold.

Evidence: `~/Desktop/Projects/synthetic-brain/docs/OBS-1-PRECOMMIT.md`.

## Perceive damage as part-versus-whole disagreement (HOLO-R)
Probing each part alone is noisy (found 3 of 16 erased regions). Comparing each part against the
decode of the whole found 16 of 16 with no false alarms. A system knows its damaged parts by their
disagreement with everything else.
