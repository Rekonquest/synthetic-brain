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
