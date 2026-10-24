# HOLO-R2: the self-healing store's composition, measured on the order that heals

Status: PRECOMMITTED (2026-10-24), before any HOLO-R2 code change.

HOLO-R found that the decode-first order (C2) heals a holographic store through lifelong damage
(0.939 after losing 6 of 8 regions at N 192), and that every per-domain ablation, repair law, chaos
arm and amplification had been run on the order whose perception was blind (C1). HOLO-R2 runs them
all on C2.

It also tests Jesse's theory, "there is no limit to complexity … more complexity can often lead to
better outcomes": the full composition against a **minimal healer** of the same order.

Everything else is HOLO-R's:
- the task (2-bit, D 4,096, 8 regions; three episodes each erasing 2 fresh regions);
- the frozen metric (recall right after each episode's damage and decode; the final episode is the
  headline);
- the hard invariant (no stage reads true values);
- test seeds 10–17, dev 0–3 (κ re-chosen on dev for C2).

## The arms (all on C2's order: decode the whole store → perceive damaged regions by their residual against the decode → mask → decode again → proofread → replay)

1. The plain holographic store (control); the local layout (control).
2. **C2, the principal** (all eight domains).
3. **Minimal healer** (Rule 7's smaller composition): C2's order with physics, cognition and
   neuroscience only. That is a plain read instead of relaxation, no proofreading, and uniform
   replay weights.
4. **Ablations of C2, one domain each:**
   - cognition: a random mask, the size of the perceived set;
   - information theory: no mask in the second decode;
   - mechanics: a plain read;
   - neuroscience: no replay;
   - biology: no proofreading;
   - chemistry: uniform weights;
   - physics: the local layout.
5. **Chaos on C2:** χ1 (logistic, perception's κ), χ2 (Hénon, replay weights), and χ1 + χ2.
6. **Amplification:** the decoder's rounds × 2.
7. **Repair laws on C2:** autocatalysis (3 cycles) and annealing (3 cooling cycles).

N ∈ {192, 256}. The bounded count is 18 arms × 2 N × 8 seeds.

## Predictions

1. **H1:** C2's final recall is ≥ 0.9 at N 192 (HOLO-R measured 0.939) and ≥ 0.25 at N 256 (0.282).
2. **H2:** removing cognition, information theory, mechanics, neuroscience or biology each lowers the
   final recall at N 192 by ≥ 0.03. Chemistry's weighting ties (±0.02). The local layout is far worse
   (≥ 0.3 lower).
3. **H3, Jesse's complexity theory:** the full C2 ≥ the minimal healer + 0.05 in final recall at both
   N.
4. **H4:** no chaos arm is promoted (strict improvement at both N over no chaos and the single-chaos
   controls).
5. **H5:** the decoder's rounds × 2 raise the final recall at N 256 by ≥ 0.01.
6. **H6:** autocatalysis ≥ replay + 0.02 in final recall at N 256. Repair in cycles rebuilds more
   under load.
7. **H7:** annealing ties autocatalysis (±0.02).
