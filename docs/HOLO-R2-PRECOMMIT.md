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

## Results (2026-10-24, `evidence/holo-r2/results.txt`, 1,174 s on 8 threads)

Dev chose κ 0.8 (dev final 0.570–0.604: C2 heals on dev too).

| recall after episodes 1 / 2 / 3 | N 192 | N 256 |
|---|---|---|
| plain holographic (control) | 0.863 / 0.445 / 0.077 | 0.710 / 0.329 / 0.054 |
| local layout (control) | 0.742 / 0.495 / 0.247 | 0.705 / 0.470 / 0.234 |
| **C2, all eight domains** | 1.000 / 0.994 / **0.939** | 1.000 / 0.564 / **0.282** |
| minimal healer (physics, cognition, neuroscience) | 0.945 / 0.906 / 0.869 | 0.854 / 0.769 / 0.712 |
| − cognition (random mask) | 0.800 / 0.238 / **0.075** | 0.625 / 0.168 / 0.053 |
| − information (no mask) | 0.969 / 0.886 / **0.579** | 0.846 / 0.421 / 0.232 |
| − mechanics (plain read) | 0.945 / 0.671 / **0.454** | 0.854 / 0.497 / 0.292 |
| − neuroscience (no replay) | 1.000 / 0.999 / **0.066** | 1.000 / 0.734 / 0.049 |
| **− biology (no proofreading)** | **1.000 / 1.000 / 1.000** | **1.000 / 0.999 / 0.995** |
| − chemistry (uniform weights) | 1.000 / 0.995 / 0.962 | 1.000 / 0.571 / 0.282 |
| − physics (local layout) | 0.751 / 0.502 / 0.252 | 0.751 / 0.501 / 0.251 |
| chaos 1 / chaos 2 / both | 0.939 / 0.932 / 0.932 (final) | 0.225 / 0.290 / 0.239 |
| decoder rounds × 2 | 1.000 / 0.994 / 0.951 | 1.000 / 0.542 / 0.269 |
| autocatalysis | 1.000 / 0.994 / 0.939 | 1.000 / 0.564 / 0.282 |
| **annealing** | 1.000 / 0.998 / **0.980** | 1.000 / 0.951 / **0.594** |

| # | Result |
|---|---|
| H1 C2 final ≥ 0.9 at 192, ≥ 0.25 at 256 | MEASURED: 0.939, 0.282 (reproduces HOLO-R) |
| H2 five domains load-bearing, chemistry ties, local ≥ 0.3 worse | **FAILED on two of seven:** cognition (−0.864), neuroscience (−0.873), mechanics (−0.485) and information (−0.360) are load-bearing; chemistry ties (+0.023); local is 0.687 worse. But **biology is harmful**: removing proofreading gives 1.000 |
| H3 complexity: full ≥ minimal + 0.05 at both N | **FAILED:** 0.939 against 0.869 at 192, but 0.282 against 0.712 at 256. The full stack carried a harmful mechanism |
| H4 no chaos arm promoted | MEASURED |
| H5 rounds × 2 raise the final at 256 by ≥ 0.01 | **FAILED:** 0.269 against 0.282 |
| H6 autocatalysis ≥ replay + 0.02 | **FAILED:** identical. The extra cycles found nothing new to write, because proofreading still blocked it |
| H7 annealing ties autocatalysis | **FAILED, upward:** 0.594 against 0.282 at 256, and 0.980 at 192. Its soft writes rebuild the memories proofreading had blocked |

**2 of 7 hold.**

### What it settles

- **Kinetic proofreading, as built, sabotages the healer.** Each half of the healthy regions (3 of 8)
  is too weak to decode on its own, so two halves rarely agree. Most *correct* memories fail the
  check and are never rewritten, and the rebuilt regions hold too few memories. The whole-store
  decode is already near perfect (1.000 at episode 1), so the gate removes good writes, not bad ones.
- **Without it, the store heals almost perfectly through lifelong damage:** 1.000 at N 192 and
  **0.995 at N 256 after losing 6 of 8 regions**, against 0.054 for the same store unhealed.
  (Observed here as an ablation. It must be confirmed as a precommitted principal on fresh seeds:
  HOLO-R3.)
- **Load-bearing domains, measured:** cognition (perceiving where the damage is), neuroscience
  (replay), mechanics (relaxation decoding), information theory (erasure masking), physics (the
  holographic layout). Chemistry's Le Chatelier weighting ties.
- **Jesse's complexity theory, with the right reading:** more complexity helps when each added
  mechanism is load-bearing. The seven-domain healer without the harmful gate beats the
  three-domain minimal healer at both loads (1.000 against 0.869; 0.995 against 0.712). The
  eight-domain stack lost at N 256 because one mechanism was harmful. **Complexity needs its
  ablations to find the harmful parts.**
- **Annealing is the repair law that survives a bad gate:** soft writes rebuild what the gate
  blocked (0.594 against 0.282 at 256).
- **Chaos and amplification:** none promoted, no gain.

**Niche:** C2 without proofreading is the lifelong self-healing memory. Proofreading may still suit
a store where the halves can decode (a lighter load or more regions), or a weaker decode (where
errors to stop exist).

### Next (HOLO-R3, a new precommit)

- Confirm C2 without proofreading as the principal on fresh seeds.
- Push the load (N 320, 384) to find where healing fails.
- Test biology mechanisms that fit:
  - proofreading on a whole-store re-decode instead of halves;
  - apoptosis (remove memories that keep failing);
  - immune self/non-self (verify a rebuilt region against the healthy ones).
