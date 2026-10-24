# PROB-1: a probability stack from physics axioms (calibrated candidate answers)

Status: PRECOMMITTED (2026-10-24), before any PROB-1 code.

Jesse (2026-10-24): "lets try making a better probability stack, and remember to keep reaching for
theoretical physics axioms. Because it's turning out to be rather fruitful." Asked which one: "both,
brain first". This is the brain's. The winner is then ported to the creativity lane
(`atom-creative-paths`).

**The target is his creativity lane's form:** up to 5 candidate answers, each with an independent
probability score (like 70 and 20), with no winner declared; the user chooses. **Honest means
calibrated:** of the answers scored 70%, about 70% are right.

## The setting

FOLD-6 to 9's 2-bit content store (D 4,096), random bytes and the glossary, N ∈ {384, 512, 768}
(where recall is uncertain). Each item's readings are its 256 phase-locked scores z_v. Test seeds
10–17; dev seeds 0–3.

## The bar (plain)

A softmax over the readings at one temperature T, chosen on dev (labels allowed there) to minimize
log-loss: *temperature scaling*, the field's standard calibration. It is a strong bar.

## The candidate laws (Rule 2: three, each with its reason)

- **Born:** p_v ∝ |⟨code_v, u⟩|², the quantum base's probability rule as it stands, with no
  temperature. QB-2 recorded that squaring *is* a temperature (Born = L1 cooled to ½), and a fixed
  one.
- **Virial (ASTRO-2R's winner):** Boltzmann at each item's own temperature, T_i = κσ_i²/m_i, with κ
  chosen on dev.
- **Maximum entropy (Jaynes), with no labels and no tuning:**
  - **σ²**, the noise variance, is read from 64 keys never written (sensing; FOLD-9's noise floor);
  - **μ**, the signal, by the method of moments: E[Σ_v z_v²] = μ² + 256σ², so μ² = mean over items
    of Σ_v z_v² − 256σ²;
  - the maximum-entropy model with these two moments (Gaussian noise, one shifted mean) gives the
    likelihood ratio p_v ∝ exp(μ·z_v/σ²): **a Boltzmann law at T = σ²/μ that the store measures
    for itself.**

**Output:** each item's top 5 candidates and their probabilities (independent; they need not sum to
1 over the shown five).

## The stack's order (Jesse's)

1. The law as its own stack against plain.
2. **Chaos:** each item's temperature jittered by 1 + 0.5(u − ½), 3 variants. The selector is the
   dev log-loss, the stack's own check on data it may see. Plain wins ties.
3. **Amplification:** the probabilities computed from FOLD-9's refined readings (the virial recall
   loop, 6 rounds) instead of the first read: more of the same evidence.

## Measures (test)

- **ECE:** the expected calibration error over all shown candidates' probabilities, in 10 bins.
- **The Brier score** of the top candidate, and the **log-loss** of the true value (its probability,
  floored at 1e-6).
- **Top-5 coverage:** is the true value among the 5?
- **The 70% check:** the accuracy of candidates scored in [0.6, 0.8).

## Predictions

1. **P1:** maximum entropy, with no labels, has ECE ≤ 0.03 at every N, within 0.01 of the tuned
   softmax.
2. **P2:** Born is badly calibrated: ECE ≥ 0.1 at N 768.
3. **P3:** virial has the lowest log-loss at N 768. A per-item temperature suits per-item noise.
4. **P4:** top-5 coverage is ≥ plain's top-1 accuracy + 0.1 at N 768. The truth is often among the
   candidates even when not first.
5. **P5:** maximum entropy's 70% check lands in [0.65, 0.75].
6. **P6:** chaos is not kept.
7. **P7:** amplification (refined readings) lowers the log-loss at N 512 by ≥ 10%.
8. **P8, the glossary:** maximum entropy with FOLD-8's ATTEND prior (the prior multiplies the
   likelihood) beats it without the prior on log-loss by ≥ 10% at N 768.

A new orchestrator crate, `prob1` (from `fold9`). Function crates: `qb-classical::temper` (Boltzmann),
`qb-amplitude` (Born), and a new `prob-moments` (μ by the method of moments).
