# ASTRO-R: the three astrophysics laws, composed properly

Status: PRECOMMITTED (2026-10-24), before any ASTRO-R code.

Jesse (2026-10-24): "Was chaos and amplification used on the astrophysics axioms? Were they
logically composed?" They were not (ASTRO-POC, 7 of 15). ASTRO-R redoes each law in his order:
1. **the law as its own stack,** with the fix the POC diagnosed, against plain (dev chooses any
   setting);
2. **chaos, off against on,** on that stack: a small mean-preserving logistic perturbation every
   step, with the stack's own check selecting and plain winning ties;
3. **amplification, on against off,** on what was kept.

Settings are chosen on dev seeds 0–3 and reported on test seeds 10–17. Kept means better on dev
and on test. Built on the POC's crates (`astro-pressure`, `astro-virial`, `astro-jeans`), unchanged;
new orchestrators `astro1r`, `astro2r`, `astro3r`. CPU, 8 threads, time-boxed to a few minutes each.

## ASTRO-1R: Pauli exclusion as degeneracy pressure (the 2-bit content store)

- **The fix:** arm B oscillated because tens of pushes per item were applied at once, each sized for
  one. Each push is now divided by that item's number of colliding rivals that iteration.
- **The stack:** 4 iterations of pressure on the colliding states (γ ∈ {0.5, 1, 2} on dev), measured
  through the quantizer, at N 384, 512, 768, against plain.
- **Chaos:** each iteration, 3 variants with each item's push scaled by 1 + 0.5(u − ½). The selector
  is the writer's own check: the number of collisions left after the iteration (the writer knows its
  content). Plain wins ties. N 512.
- **Amplification:** the push step η × 2.
- **Predictions:**
  - **R1a:** the fixed pressure's collision count falls each iteration (no oscillation).
  - **R1b:** pressure ≥ plain + 0.02 at N 512 and 768.
  - **R1c:** chaos not kept.
  - **R1d:** amplification not kept (the step is already sized to the collisions).

## ASTRO-2R: the virial law as a self-set temperature (FOLD-8's soft cancellation)

- **The fix:** two candidate laws (Rule 2's one to three orders), at the right scale.
  - **L1, the real direction:** the virial theorem (K = −U/2) gives tighter-bound systems the
    higher temperature: T_i = c·m_i, with c ∈ {0.02, 0.05, 0.1}.
  - **L2, the POC's law at the right scale:** T_i = κσ²/m, with κ ∈ {0.1, 0.2, 0.5}.
  - Dev picks the law and its setting.
- **The stack:** 3 rounds of soft cancellation at each item's own temperature, at N 512 and 768,
  against plain, hard K1 (ρ 0.75) and K6 (T₀ 0.005), all from FOLD-8.
- **Chaos:** each round, 3 variants with each item's temperature scaled by 1 + 0.5(u − ½), selected
  by the round's total margin. Plain wins ties.
- **Amplification:** the rounds × 2 (6), more of the same cooling.
- **Predictions:**
  - **R2a:** the chosen law is within ±0.02 of K6 at N 512 (0.884).
  - **R2b:** no arm beats plain at N 768. That is FOLD-8's tipping point, which a temperature law
    cannot move.
  - **R2c:** chaos not kept.
  - **R2d:** rounds × 2 adds ≤ 0.01.

## ASTRO-3R: the Jeans law in a medium where it can act

- **The fix:** the POC's medium was bistable. Its false scars came from the stir's *mean* crossing
  the threshold, which a window cannot remove. The new medium keeps the stir's mean input (q·ε =
  0.001) but makes it **bursty**: rare large kicks, (q, ε) ∈ {(0.002, 0.5), (0.005, 0.2)}. The POC's
  (0.02, 0.05) is kept as the control. The mean then sits below threshold, and false scars can only
  come from spikes, which are short modes that the Jeans window averages away.
- **The stack:** the POC's protocol (64 × 64, blobs of sides 1, 3 and 7). Each law's E_ref is
  matched on dev so that side-7 blobs carve ≥ 0.9. The local law (R 1) is compared against Jeans
  R 3 and 5.
- **Chaos:** the formation rate per site and step, μ·(1 + 0.5(u − ½)). The trap law's own
  maintain-or-erase is the selector. On the winning law, at the burstiest medium.
- **Amplification:** the write gain × 2, on the winning law.
- **Predictions:**
  - **R3a:** in the bursty media, the local law's false rate at match is ≥ 0.05, and Jeans' ≤ 0.5×
    it, at side-7 true ≥ 0.85. The control medium stays as the POC found it, with both false rates
    near 0.
  - **R3b:** chaos not kept.
  - **R3c:** amplification raises side-3 true without raising false by more than 0.01.

## Results (2026-10-24; `evidence/astro-r/`; 67 s, 107 s, 34 s on 8 threads)

### ASTRO-1R, degeneracy pressure, pushes divided by each item's collisions: 3 of 4

| arm (test) | N 384 | N 512 | N 768 |
|---|---|---|---|
| plain | 0.800 | 0.636 | 0.425 |
| **pressure (γ 0.5, chosen on dev)** | **0.898** | **0.728** | 0.430 |
| + amplification (η × 2) | 0.938 | 0.733 | 0.405 |

Collisions per iteration (seed 10) fall every iteration: N 384 1,046 → 608; N 512 3,746 → 2,411;
N 768 13,844 → 10,660.

- **R1a HOLDS:** the oscillation is gone.
- **R1b FAILED at N 768:** +0.098 at N 384 and +0.092 at N 512, but +0.005 at N 768, FOLD-8's
  tipping point again.
- **R1c HOLDS:** chaos (N 512) gives dev 0.722 → 0.717 and test 0.728 → 0.730; not kept.
- **R1d HOLDS:** amplification is not kept on the mean over N. It helps light loads (N 384 0.938)
  and hurts N 768.

### ASTRO-2R, the virial law as each item's own temperature: 2 of 4

Dev chose **L2, T_i = κσ²/m at κ 0.1**. L1 (bound → hot) was identical at every c tried: at
c ≤ 0.1 its temperatures are so low that it acts as hard cancellation.

| arm (test) | N 512 | N 768 |
|---|---|---|
| plain | 0.636 | 0.425 |
| hard K1 (FOLD-8) | 0.894 | 0.407 |
| K6 schedule (FOLD-8) | 0.884 | 0.334 |
| **the virial law** | **0.904** | 0.359 |
| chaos on the temperatures | 0.903 | 0.359 |
| **amplification: rounds × 2** | **0.966** | 0.381 |

The law's median item temperature per round (N 512): **0.0020 → 0.0012 → 0.0006**. It cools itself,
which the POC's version never did.

- **R2a FAILED, upward:** 0.904 is above K6 by more than the predicted ±0.02. It is the best recall
  stack yet at N 512.
- **R2b HOLDS:** nothing beats plain at N 768.
- **R2c HOLDS:** chaos is not kept.
- **R2d FAILED, upward:** doubling the rounds lifts N 512 to **0.966** (+0.062).

### ASTRO-3R, the Jeans law in bursty media: 2 of 3

The bursty stir did not remove the bistability. In every medium the dev sweep shows a cliff
between E_ref 0.6 and 0.9: below it the far field carves, above it nothing false does. Matching on
side-7 blobs puts every law at E_ref 1.35, where every false rate is 0.

At the cliff edge (E_ref 0.6, dev), the Jeans window does help, and more in bursty media:

| medium (q, ε) | local false | Jeans R 3 false |
|---|---|---|
| control (0.02, 0.05) | 0.548 | 0.476 (−13%) |
| bursty (0.005, 0.2) | 0.775 | 0.566 (−27%) |
| burstiest (0.002, 0.5) | 0.949 | 0.737 (−22%) |

- **R3a FAILED:** at the matched threshold there is nothing to remove, and at the edge the
  reduction is short of half.
- **R3b HOLDS:** chaos on μ is not kept.
- **R3c HOLDS:** gain × 2 carves side-3 blobs (0.000 → 1.000) with no false scars.
- **Diagnosis:** what keeps the medium bistable is the trap law's own positive feedback (a carved
  site leaks 100× less, so any spike that carves seals itself), not the noise. A Jeans law needs a
  trap law without that runaway.

### What ASTRO-R settles (7 of 11 hold)

**Composed properly, two of the three laws are load-bearing:**
- **Pauli pressure:** +0.09–0.10 at light and moderate load.
- **The virial self-set temperature:** the best recall stack at N 512, 0.904, and **0.966 with
  rounds × 2**.

Both stop at FOLD-8's tipping point (N 768). The Jeans law has a modest effect only at the edge of a
bistable medium. Chaos was kept nowhere; amplification helped where the stack still had room
(rounds × 2, the gain for small blobs).
