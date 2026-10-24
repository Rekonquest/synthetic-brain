# PHYS3-POC: the remaining planned laws (astrophysics and black holes), composed where they fit

Status: PRECOMMITTED (2026-10-24), before any PHYS3 code.

Jesse (2026-10-24): "Let's continue with the remaining planned laws." The rules are as before:
- the mechanism works, which does not make the theory true;
- each law is its own stack against a status-quo bar (no existing stack reused), then chaos off/on,
  then amplification on/off;
- composed with neuroscience, chemistry or cognitive axioms where one fits;
- niches recorded;
- f64;
- test seeds 10–17, dev 0–3.

**Laws from the queue with no clean mapping to a testable stack yet** (recorded, not forced):
T-duality, supersymmetric cancellation, the black-hole area law, fast scrambling (the fold already
scrambles), the Eddington limit, inflation, and recombination.

## R1: the Roche limit (a weak body near a strong one is torn apart) as memory placement

- **The store:** 16 regions, each a 2-bit association store of D 1,024, holding 512 memories
  (key → byte). A quarter of them are **strong** (written at amplitude 3), the rest weak
  (amplitude 1).
- **Plain:** random placement, 32 per region.
- **The Roche law:** no weak memory shares a region with a body more than r × its strength (r 2), so
  strong memories fill 4 regions of their own and weak ones the other 12. In a 2-bit store the
  strong drown the weak through the quantizer: radio's capture effect.
- *Amended before code* (Jesse, 2026-10-24: "don't forget chaos and amplification"). The first
  plan had no chaos arm here, and its "amplification" (r 4) was really an ablation.
- **Chaos:** 3 placements, each shuffled by keyed logistic chaos within its strength class. The
  selector is the writer's own readback (the total margin of its memories, which the writer knows);
  plain wins ties.
- **Amplification:** weak memories written at amplitude 2.
- **Ablation:** a gentler limit, r 4 (strong and weak may mix, which equals random placement).
- **Predictions:**
  - **R1a:** weak recall rises ≥ 0.1 over plain.
  - **R1b:** strong recall stays ≥ plain − 0.02.
  - **R1c:** chaos not kept.
  - **R1d:** amplification lifts weak recall under random placement (less capture) and is a tie under
    the Roche law (a region of equals quantizes the same).

## R2: hierarchical merging (galaxies grow by merging) as a memory index

- **The task:** G1's recognition with 1,024 library shapes (identity by Noether features, noise
  0.02, no mirror).
- **Bar:** a linear scan (1,024 distances).
- **The merging law:** repeatedly merge the two nearest clusters (feature centroids) into one,
  building a binary tree bottom-up. A query descends the tree, keeping the best b children at each
  level by distance to their centroids. b 1 is greedy; **amplification** is b 4.
- **Chaos:** one extra keyed-random child explored per level; the final distance selects.
- **Predictions:**
  - **R2a:** greedy ≥ 0.9 of the scan's accuracy at ≤ 3% of its comparisons.
  - **R2b:** b 4 is within 0.01 of the scan at ≤ 10% of its comparisons.
  - **R2c:** chaos not kept.

## R3: mass segregation (heavy bodies sink to a cluster's core) as a memory cache

- **The task:** 4,096 memories, a Zipf(1.0) access stream of 200,000, a core of 256. A core access
  costs 1 and a halo access 20.
- **Bars:** random static placement, and **LRU** (the status quo).
- **The segregation law (dynamical friction):**
  - each memory's mass decays and grows with use, m ← m·γ + 1 on access (and m·γ per step
    otherwise, applied lazily);
  - every 1,000 accesses the 256 heaviest settle into the core;
  - γ is chosen on dev.
- **Chaos:** the settling interval jittered by 1 + 0.5(u − ½).
- **Amplification:** settle every 250 accesses.
- **The test streams:** stationary Zipf, and a **shifting** Zipf (the popularity ranking reshuffled
  every 50,000).
- **Predictions:**
  - **R3a:** under stationary Zipf, segregation's mean cost ≤ LRU's − 5%.
  - **R3b:** under the shifting stream it is within 10% of LRU's.
  - **R3c:** chaos not kept.

## R4: orbital resonance, composed with neuroscience's binding by synchrony

- **The binding problem:** a scene has 3 objects, each with a colour (of 8) and a shape (of 8). The
  task is which colour goes with which shape.
- **Feature units:** 16 oscillators (phases θ). An active feature receives its object's
  location-tagged drive, a phase that drifts with noise σ at the object's own frequency (the common
  input of one receptive field).
- **The resonance law:** active units are pulled by Kuramoto coupling K toward the drive and toward
  each other, and lock in phase with the units that share their drive.
- **Readout:** each colour is paired with the shape nearest it in phase.
- **Bar:** rate coding with no phase (pairing at random: 1/6 of scenes fully right; the expected
  pairs right is 1 of 3).
- 1,000 scenes; σ ∈ {0.1, 0.3, 0.6}.
- **Chaos** (*amended before code*; the first plan had none): the entrainment K jittered per unit
  and step by 1 + 0.5(u − ½), mean-preserving; kept only if better on dev (seeds 0–3) and on test.
- **Amplification:** K × 2.
- **Predictions:**
  - **R4a:** every pairing right in ≥ 0.95 of scenes at σ 0.1, against the bar's 0.167.
  - **R4b:** binding degrades gracefully with noise, still ≥ 0.6 at σ 0.6.
  - **R4c:** K × 2 is kept at σ 0.6.
  - **R4d:** chaos not kept.

## R5: gravitational lensing, composed with the cognitive REINFORCE, as priming

- **The task:** G1's recognition by Noether identity at noise 0.05 (where it scored 0.776), with the
  queries' shapes drawn Zipf(1.0).
- **The lensing law:** a heavy memory bends recall toward itself. The effective distance is
  d_j − β·ln m_j, where the mass m_j is REINFORCE's count of past answers that named shape j (+1
  each, online, starting at 1). β is chosen on dev.
- **Bar:** no lensing.
- **Chaos:** β jittered per query, selected by dev.
- **Amplification:** β × 2.
- **Predictions:**
  - **R5a:** under Zipf queries, identity rises ≥ 0.05.
  - **R5b:** under uniform queries, it changes by no more than ±0.02 (the masses stay even).
  - **R5c:** β × 2 overshoots under uniform queries (≤ −0.02).
  - **R5d:** chaos not kept.

## Results (2026-10-24, `evidence/phys3-poc/`; each run 0–7 s)

### R1, the Roche limit as placement: 4 of 4

| placement | weak recall | strong recall |
|---|---|---|
| random (bar) | 0.793 | 1.000 |
| **Roche law, r 2** | **1.000** | 1.000 |
| Roche + chaos placement | 1.000 | 0.999 |
| gentler limit r 4 (ablation, = random) | 0.793 | 1.000 |
| weak × 2, random placement | 0.995 | 1.000 |
| weak × 2, Roche | 1.000 | 1.000 |

All four hold. In a 2-bit store the strong memories capture the quantizer's phase and drown the
weak (radio's capture effect). Keeping them apart removes that, and so does making the weak ones
louder. Chaos is not kept.

### R2, hierarchical merging as an index: 0 of 3

| (1,024 shapes) | identity | comparisons |
|---|---|---|
| linear scan (bar) | 0.979 | 1,024 |
| merge tree, greedy | 0.263 | 43 |
| merge tree, beam 4 (amplified) | 0.696 | 211 |
| greedy + chaos child | **0.310** | 84 |
| beam 4 + chaos child | **0.722** | 254 |

- **R2a and R2b FAILED:** centroids of merged clusters describe their members poorly in this
  feature space, so the descent goes wrong early.
- **R2c FAILED, and chaos was kept:** one keyed-random extra child per level lifts both (+0.047,
  +0.026). Exploration helps where the plain rule misjudges.

### R3, mass segregation (dynamical friction) as a cache: 3 of 3

Dev chose γ 0.9999.

| mean access cost | stationary Zipf | shifting Zipf |
|---|---|---|
| random core (bar) | 18.317 | 18.811 |
| LRU (status quo) | 9.046 | 9.058 |
| **segregation** | **7.134** | **7.681** |
| segregation + chaos interval | 7.134 | 7.645 |
| segregation, settle every 250 (amplified) | 7.091 | 7.541 |

All three hold: **21% cheaper than LRU** on the stationary stream, and 15% cheaper even when
popularity shifts. Chaos is a tie or near-tie (not kept by the rule). Settling more often helps a
little.

### R4, orbital resonance with binding by synchrony: 2 of 4

The bar (no phase, independent random pairing) is 0.037 (all three right by chance: (1/3)³; the
precommit's 1/6 assumed a permutation).

| noise | resonance, K 1 | K × 2 (amplified) |
|---|---|---|
| 0.1 | 0.877 | 0.909 |
| 0.3 | 0.652 | 0.742 |
| 0.6 | 0.379 | 0.522 |

- **R4a FAILED by margin** (0.877, bar 0.95). **R4b FAILED** (0.379 at noise 0.6).
- **R4c HOLDS:** K × 2 is kept, +0.14 at noise 0.6.
- **R4d HOLDS:** chaos is not kept.
- Phase locking does bind features by synchrony (24× the chance level at low noise), but it needs
  strong entrainment against noise.

### R5, lensing with REINFORCE (priming): 2 of 4

Dev chose β 0.02.

| | no lensing | lensing | β × 2 |
|---|---|---|---|
| Zipf queries | 0.787 | 0.807 | 0.810 |
| uniform queries | 0.781 | 0.771 | 0.750 |

- **R5a FAILED by size:** +0.020 (bar +0.05).
- **R5b and R5c HOLD:** uniform queries are within ±0.02, and β × 2 overshoots there (−0.032).
- **R5d:** chaos was "kept" by the strict rule on differences below the printed precision (dev and
  test both 0.799 and 0.807 to three places). That is selection-luck size, recorded as kept but not
  meaningful.

**PHYS3: 11 of 18 hold.**

### Niches

- **The Roche law:** placement in any shared quantized store with mixed-strength contents.
- **Mass segregation:** the cache, beating LRU under stationary and shifting popularity.
- **Resonance (binding by synchrony):** feature binding, given strong entrainment.
- **The merge tree with chaos exploration:** a cheap first guess (0.72 at a fifth of the cost),
  where a verify step follows.
- **Lensing:** a small priming gain when the query distribution is skewed; off when it is not.
