# PROB-1: a probability stack from physics axioms (calibrated candidate answers)

Status: PRECOMMITTED (2026-10-24), before any PROB-1 code.

Jesse (2026-10-24): "lets try making a better probability stack, and remember to keep reaching for
theoretical physics axioms. Because it's turning out to be rather fruitful." Asked which one: "both,
brain first", then (2026-10-24) **"No, leave the creativity lane out for now."** So this is the
synthetic brain's probability stack only; no port is planned.

**The target borrows his creativity lane's form (the lane itself is left out):** up to 5 candidate answers, each with an independent
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

## Results (2026-10-24, `evidence/prob-1/results.txt`, 18 s)

Dev chose softmax T 0.003 and virial κ 0.1. ECE is over the 5 shown candidates; "70% right" is the
accuracy of candidates scored in [0.6, 0.8).

| random, test | N | ECE | Brier | log-loss | top-5 | top-1 | "70%" right |
|---|---|---|---|---|---|---|---|
| softmax, tuned T (bar) | 384 | 0.004 | 0.105 | 0.750 | 0.949 | 0.800 | 0.719 |
| **maximum entropy (no labels)** | 384 | 0.008 | 0.109 | 0.764 | 0.949 | 0.800 | 0.768 |
| softmax, tuned T | 512 | 0.017 | 0.160 | 1.460 | 0.858 | 0.636 | 0.634 |
| **maximum entropy** | 512 | **0.009** | 0.160 | 1.466 | 0.858 | 0.636 | 0.664 |
| **maximum entropy, refined readings** | 512 | **0.006** | **0.023** | **0.131** | **0.997** | **0.966** | 0.678 |
| virial, refined readings | 512 | 0.008 | 0.027 | 0.199 | 0.997 | 0.966 | 0.562 |
| softmax, tuned T | 768 | 0.044 | 0.216 | 2.652 | 0.692 | 0.425 | 0.456 |
| **maximum entropy** | 768 | **0.015** | **0.205** | **2.558** | 0.692 | 0.425 | 0.634 |
| Born | 768 | 0.077 | 0.244 | 4.322 | 0.481 | 0.259 | (none scored there) |
| virial (own T) | 768 | 0.072 | 0.253 | 3.409 | 0.692 | 0.425 | 0.397 |

Born's ECE at N 384 and 512 is 0.149 and 0.120.

**Glossary, N 768:** maximum entropy ECE 0.008, log-loss 2.883, top-1 0.372. With the ATTEND
prior: log-loss **1.859**, top-1 **0.556**, top-5 0.802, "70%" right 0.767.

| # | Result |
|---|---|
| P1 maximum entropy ECE ≤ 0.03 at every N, within 0.01 of tuned softmax | MEASURED: 0.008, 0.009, 0.015, and **better than the tuned softmax at N 512 and 768** (0.017, 0.044) |
| P2 Born ECE ≥ 0.1 at N 768 | **FAILED:** 0.077 at N 768 (0.149 and 0.120 at N 384 and 512) |
| P3 virial lowest log-loss at N 768 | **FAILED:** 3.409, the worst but Born. A per-item temperature distorts the probabilities, though it sharpens recall (ASTRO-2R) |
| P4 top-5 coverage ≥ top-1 + 0.1 at N 768 | MEASURED: 0.692 against 0.425 |
| P5 maximum entropy's "70%" in [0.65, 0.75] | **FAILED at N 768:** 0.634 (N 384 0.768, N 512 0.664) |
| P6 chaos not kept | MEASURED: no variant beat plain on dev |
| P7 refined readings cut log-loss ≥ 10% at N 512 | MEASURED: 1.466 → 0.131 (−91%) |
| P8 the ATTEND prior cuts the glossary's log-loss ≥ 10% | MEASURED: 2.883 → 1.859 (−36%) |

**5 of 8 hold.**

### What it settles

- **Jaynes' maximum entropy gives a self-calibrating probability stack.** The store measures its own
  noise (keys never written) and its own signal (the method of moments), and the Boltzmann law at
  T = σ²/μ needs no labels and no tuning. It is better calibrated than temperature scaling tuned on
  labelled dev data once the store is loaded (N 512 and 768).
- **With the recall loop's refined readings it becomes the best probability stack measured:** at
  N 512, top-1 0.966, top-5 0.997, ECE 0.006, log-loss 0.131.
- **With ATTEND on real data** it uses the redundancy: top-1 0.372 → 0.556 at N 768, with its 70%
  scores right 77% of the time.
- **The "70%" check drifts low at the deepest load** (0.634), so the label-free calibration is good
  in aggregate (ECE 0.015) but slightly overconfident in that band.
- **Born and the per-item virial temperature are not probability laws for this store.** The virial
  temperature is a *recall* law: it sharpens readings in the loop (ASTRO-2R) but miscalibrates when
  used for the final scores.

**Niches:**
- **maximum entropy:** the calibrated probability layer, with no labels;
- **maximum entropy on refined readings:** the best scored candidate answers below the tipping
  point;
- **maximum entropy with the ATTEND prior:** scored answers on redundant data;
- **the virial temperature:** inside the recall loop, not at the output.
