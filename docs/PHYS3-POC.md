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
