# QB-8: retesting chaos and hot/cold on the new composed stack

Status: PRECOMMITTED (2026-10-18), before any code.

## Why

Jesse: "Did you try retesting with chaos hot and cold on and off. Because that can change the new
stack composed stack."

He is right: QB-7 carried QB-5's chaos and temperature settings forward without retesting them. The
stack is now a different composition (growth with product inhibition, on top of the gain) in a
different world (40% of cues missing). By his rule, each new composition is re-optimized at the edge
before the next stack goes on top.

## Protocol

QB-5's protocol, unchanged:
- one module at a time, on its own check;
- the development seeds select, and plain wins ties;
- the test seeds verdict, with the winner kept only if it beats off on test;
- each round starts from the kept best.

**The world:** QB-7's (p_miss 0.4). **The base:** QB-7's final kept system (κ 1, ι 1, with QB-6's
gain ×3, T₀ 0.1, amplitude chaos α 0.05 key 2). Test 0.749: cued 0.915, missed 0.492.

**The rounds:** QB-5's variant sets, plus explicit off arms for the chaos the base already carries.
1. **D:** chaos on the votes before the gate, α ∈ {0.1, 0.2, 0.4} × 3 keys.
2. **M:** phase chaos α ∈ {0.1, 0.5, 1.0}; occupation heat ε ∈ {0.005, 0.01, 0.02}.
3. **T, cold and hot:** T₀ ∈ {0.05, 0.1, 0.15, 0.2, 0.3, 0.4}. The grid now goes colder than QB-5's
   edge.
4. **T, the governor:** reheat on D's event, then cool. T_hot ∈ {0.5, 1.0} × τ ∈ {3, 10}.
5. **T, amplitude chaos:** **off** (α 0), plus α ∈ {0.05, 0.1, 0.2} × 3 keys. The base carries
   α 0.05 key 2, so "off" is a variant here.

## Predictions, from the characters, for *this* composition

1. **R1, D:** chaos before the gate is still neutral, so not kept (the noise is still iid).
2. **R2, M:** collapse is still immune to phase chaos (ties exactly), and heat still ties (argmax
   readout). M stays plain.
3. **R3, T: the best temperature moves warmer.** Missed trials need the tracker to escape a stale
   memory, which is exploring, and hot explores. So the best T₀ on dev is above 0.1, and the kept
   change raises acc_missed on test.
4. **R4, T: the governor is still not kept.** It reheats on D's *events*, but a missed trial has no
   cue event: the reheat comes at the wrong time for the problem.
5. **R5, T: amplitude chaos stays neutral.** Every variant, including off, is within 0.003 of the base
   on test acc_resp.

**Kill criterion:** if 3 or more of R1–R5 fail.

## Build

- `qb8` is `qb7`'s runner plus QB-5's knobs: D vote chaos, M phase and occupation chaos, and the
  governor. At the base it reproduces QB-7's 0.749, and a unit test checks this.

## Results (2026-10-18, `evidence/qb-8/results.txt`)

The test passed before the run: the base reproduces QB-7's 0.749.

| Round (check) | dev: off → best | test acc_resp (cued / missed) | verdict |
|---|---|---|---|
| 1 D chaos before the gate (F1) | 0.8286 → 0.8288 (α 0.4 key 1; α ≤ 0.2 ties) | 0.749 → 0.749; F1 0.8062 → 0.8054 | not kept |
| **2 M phase chaos and heat (fidelity)** | 0.6186 → 0.6188 (occupation heat ε 0.01; phase ties exactly) | **0.749 → 0.765** (missed **0.492 → 0.534**) | **KEPT** |
| 3 T cold and hot (acc_resp) | 0.7846 → 0.7868 (**T₀ 0.05**; warmer is worse: 0.15 / 0.2 / 0.3 / 0.4 give 0.7822 / 0.7789 / 0.7661 / 0.7434) | 0.765 → 0.767 | KEPT |
| 4 T governor (acc_resp) | 0.7868 → plain wins (0.7811 / 0.7767 / 0.7594 / 0.7324) | unchanged | not kept |
| 5 T amplitude chaos, off and on (acc_resp) | 0.7868 → **0.7869 (OFF)** | 0.767 → 0.768 | **chaos removed** |

Amplitude chaos on test, from the round-5 base (0.7665):
- OFF: 0.7680;
- α 0.05: 0.7647–0.7665;
- α 0.1: 0.7590–0.7620;
- α 0.2: 0.7437–0.7472.

**The final system:** 0.768 on test (cued 0.917, missed 0.538), up from QB-7's 0.749.

| # | Result | What was measured |
|---|---|---|
| R1 D chaos still neutral | MEASURED | not kept |
| R2 M still immune | **FAILED** | phase ties exactly (held), but occupation heat does **not** tie: it is kept, and lifts missed trials by +0.042 |
| R3 the best temperature moves warmer | **FAILED** | the best is **colder** (T₀ 0.05); missed trials moved only 0.534 → 0.536 |
| R4 the governor still not kept | MEASURED | every reheat is below plain |
| R5 amplitude chaos neutral (within 0.003) | **FAILED** | as worded: α 0.2 deviates by 0.023 (the known too-much edge). In substance, chaos OFF is best, so it is removed |

**2 of 5 hold. The kill criterion (3) FIRES:** my predictions for the new composition did not hold
as precommitted.

### What the retest found (Jesse's point, confirmed)

The composition and the world changed what chaos and temperature do. Predictions carried over from
QB-5's composition failed exactly where the new composition differs:

1. **Heat now has a job, but in the memory, not in the decision.**
   - In QB-5's world (no missing cues), occupation heat on M tied exactly.
   - In this world it softens a stale memory: the thermal floor dilutes the old item's pull on the
     tracker, so the response evidence can overturn it. Missed trials go from 0.492 to 0.534, and cued
     trials are untouched (0.915).
   - My derivation put the needed exploration in T's temperature (R3). The character "heat unpins" was
     right; the module was wrong.
2. **The decision should stay cold, colder still.** T₀ 0.05 beats 0.1, at the grid's edge again.
   Commitment at the decision, softness in the memory: two temperatures in two modules.
3. **The chaos QB-5 kept by the strict rule is removed here.** OFF is best on dev and on test. That
   fits QB-5's own reading that it was selection-luck sized.

### A check gap, on record

M's round was decided on M's check, its argmax hold fidelity, which moved only +0.0002, while the
system gained +0.016. **M's check is blind to M's actual output role**: the concentrations it feeds
T. Heat changes those concentrations without changing the argmax. The variant was kept, but by a
margin the check could not really see. The skill's warning applies: "a wrong check optimizes the
wrong thing". A better check for M would score its reading as T consumes it, for example the Born
weight it assigns to the target. That goes into the next precommitment.

### Next

- Fix M's check: score its concentrations, not only its argmax. Then rerun round 2.
- Extend T₀ below 0.05 (the edge).
- Retest amplification on this composition too, by the same logic.
- Rerun the classical comparator with a matching thermal floor on its register, so law 5 is checked
  fairly on the new composition.
- Then the mismatch module.
