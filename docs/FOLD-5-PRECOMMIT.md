# FOLD-5: folding outside the thermal state (the quantum base)

Status: PRECOMMITTED (2026-10-23), before any FOLD-5 code.

Jesse (2026-10-23): "I want you to try folding outside the thermo state."

## Why the quantum base

The thermal medium holds **energy**, which is never negative. Every memory folded into a region adds
energy to its bands, so overlaps only accumulate, and the ceiling is occupancy (FOLD-3). The quantum
base (QB-1 to QB-9) holds **amplitudes with phase**. Two random phases cancel on average, so folded
memories should interfere away from each other instead of piling up. This is superposition folding,
item 5 of the folding list in `docs/TODO.md` (vector-symbolic memory: superpose, then unbind = tap).

The wave medium (WAVE-1) is the other substrate outside the thermal state that carries phase. It is
not used here; it is the next step if this one teaches something.

## The store

- **The same task as FOLD-1 to 4:** 16 regions, F memories folded into each, recall by cue.
- **The same bytes:** 422,400, the 32-band thermal medium. 16 regions × D complex amplitudes at
  16 bytes each gives **D = 1,650 per region**. Each region is one `qb-amplitude::State`.
- **No flow between regions,** because QB-3/QB-4 measured that the mixer transports a memory away;
  QB-4's modular design keeps each module's state separate.
- **Keys:** dense phasor codes, D unit amplitudes with random phases.
- **Writing:** memory j of region r is written for 250 steps at gain g 0.02 per step
  (`excite(i, g·key_i)` per amplitude), one memory after another, as in the thermal medium.
- **Heat (the noise):** each step, each amplitude with probability q 0.02 receives `excite` of
  magnitude ε 0.05 at a random phase, the stir's rate and size. (In the quantum base, heat is
  occupation; QB-2.)
- The store has no flow, so the writes and the kicks commute: the state after T steps is the sum of
  all writes plus all kicks. It is computed that way, with the kicks drawn per amplitude for the
  whole run (the same law; only the rounding order differs).
- **The reading (interference):** score(cue, region) = |⟨cue, a_r⟩| / (‖cue‖·‖a_r‖), a new function
  crate `qb-overlap`. Region top-1, set recall and capacity as in FOLD-1.

**The fair classical comparator** (as in every QB experiment): the same store with non-negative real
weights and the same bytes (D_c = 3,300 per region, 8 bytes each).
- Keys: uniform in (0, 1). Heat: kicks of +ε (the stir's always-positive kicks).
- The reading: centred correlation (Pearson), a new function crate `spec-correlate`.

Test seeds 10–17; dev 0–3 for the rounds. Depth F ∈ {1, 4, 16, 64, 128, 256}. Reads at drain t 250,
and t 20,000 at the capacity depth. A new orchestrator crate, `fold5`.

## Predictions

1. **Q1, the phase store's capacity is ≥ 64 and ≤ 256 per region** (1,024 to 4,096 memories in
   422 KB).
   - The correct region scores about D (times the written amplitude); the other F − 1 memories in it,
     and every wrong region, add random-phase sums of size about √(F·D). The signal-to-crosstalk ratio
     is √(D/F).
   - Picking the right region among 16 needs that ratio at about 3 or more, so F ≲ D/9 ≈ 180.
2. **Q2, the classical comparator ties** within ±0.05 at every F.
   - Centred, the non-negative keys are zero-mean random vectors with the same √(D_c/F) law, and the
     two stores have the same real degrees of freedom (2 × 1,650 = 3,300).
   - So any gain over the thermal medium is the **reading of the dimensions with signed (centred)
     interference**, not phase as such. This is QB-4's and QB-9's character: the gain is the
     architecture.
3. **Q3, no maintenance is needed:** at the capacity depth, top-1 at t 20,000 is within 0.02 of
   t 250. There is no leak, and 20,000 steps of kicks add about ε√(qt) ≈ 1 per amplitude, against a
   written 5.
4. **Q4, heat: no inverted U.** With ε × {0, 0.5, 1, 2, 4} at the capacity depth, top-1 never rises
   with heat, and × 4 costs ≤ 0.05.
   - Noise maintains a memory only where there is an activity-gated coupling (the edge-of-chaos
     record), and this store has none. Crosstalk, not noise, sets its limit.
5. **Q5, amplification (gain × 2): a tie** (≤ 0.01). The crosstalk scales with the gain.
6. **Q6, chaos: not kept** (a tie, or ≤ 0.01).
   - On: at each write, 3 variants of the written vector, each amplitude scaled by 1 + α(u − ½), with
     u from `qb-chaos::keyed_unit` and α 0.1 (mean-preserving).
   - The selector is the region's own readback: the mean score of its memories written so far. Plain
     wins ties.
   - Random codes are already near-orthogonal, so there is little for selection to find.

## Against the thermal medium

The comparison is memories per byte at region top-1 ≥ 0.8, against FOLD-3's 32-band bar (13,200
bytes per memory) and FOLD-4's best. If Q1 holds, the phase store holds about 30× more per byte.

**What it gives up, to be weighed and not hidden:**
- the store has no medium: no flow, no scars, no maintain-or-erase;
- it never forgets, which the thermal medium does (SCAR-1's quiet collapse);
- a read costs D complex products per region, against 32 numbers.

CPU only, a few minutes.

## Results (2026-10-23; `evidence/fold-5/grid.txt`, `heat.txt`, `rounds.txt`; about 5 min on 8 threads)

**Region top-1 at drain t 250 (set recall in brackets), the same 422,400 bytes:**

| memories per region (F) | memories | bytes each | phase store | classical comparator |
|---|---|---|---|---|
| 16 | 256 | 1,650 | 1.000 (1.000) | 1.000 (1.000) |
| 64 | 1,024 | 412 | 1.000 (0.999) | 1.000 (1.000) |
| 128 | 2,048 | 206 | 0.993 (0.961) | 0.996 (0.973) |
| **256** | **4,096** | **103** | **0.866** (0.793) | **0.931** (0.861) |
| 512 | 8,192 | 51 | 0.574 (0.536) | 0.730 (0.665) |

**Lifetime and heat (phase store, F 256):**

| heat ε × | t 250 | t 20,000 |
|---|---|---|
| 0 | 0.870 | 0.870 |
| 0.5 | 0.869 | 0.868 |
| 1 (plain) | 0.866 | 0.867 |
| 2 | 0.857 | 0.857 |
| 4 | 0.826 | 0.824 |

**Rounds (phase store, F 256; dev, then test):** gain × 2 gives test 0.869 (+0.002); chaos gives test
0.867 (+0.001). Both pass the strict rule on dev and test, but both are selection-luck size.

| # | Result |
|---|---|
| Q1 phase capacity 64–256 | MEASURED: **256 per region** (0.866), 4,096 memories at 103 bytes each |
| Q2 the classical comparator ties (±0.05) | **FAILED: the classical store is better at depth** (0.931 against 0.866 at F 256; 0.730 against 0.574 at F 512) |
| Q3 no maintenance needed | MEASURED: t 20,000 equals t 250 (0.867 / 0.866) |
| Q4 heat: no inverted U | MEASURED: heat only costs (0.870 cold → 0.826 at × 4) |
| Q5 amplification a tie | MEASURED: +0.002 |
| Q6 chaos ≤ 0.01 | MEASURED: +0.001 |

**5 of 6 hold.**

### What it settles

- **Outside the thermal state, folding goes about 64× deeper per byte.**
  - The thermal medium's best is the shape code at 4 per region: 64 memories, 6,600 bytes each
    (FOLD-4).
  - The phase store holds 256 per region: 4,096 memories, **103 bytes each**, at 0.866.
  - The square-root law held: √(D/F) is about 2.5 at F 256, where the region is still found, and
    1.8 at F 512, where it is not.
- **Folded memories there need nothing to stay alive.** There is no leak, so 20,000 steps change
  nothing, and heat only costs.
  - This is the opposite of the thermal medium, where noise is what keeps the scars (SCAR-1).
  - It is the edge-of-chaos record again: noise maintains only where there is an activity-gated
    coupling, and this store has none.
- **Amplification and chaos do nothing here** (+0.002, +0.001). Crosstalk sets the limit, and both
  scale with it.
- **Q2's miss (a diagnosis after the run, not a prediction):** the phase store's reading takes the
  *magnitude* |⟨c, a⟩|, so it throws the phase away.
  - Every wrong region's crosstalk then counts in both of its components (re and im), while the
    classical correlation counts one signed component.
  - Each memory was written with the key's own phase, so its overlap lands on the real axis. A
    phase-locked reading (Re⟨c, a⟩) should close the gap. That is the next check.

### The diagnosis tested (after the run, not a prediction; `diagnostic-phaselock.txt`)

A new function crate, `qb-overlap-phase`, gives the phase-locked reading Re⟨c, a⟩/(‖c‖‖a‖). It is
the same store, same seeds and same states; only the reading changes.

| F | magnitude reading | **phase-locked reading** | classical comparator |
|---|---|---|---|
| 128 | 0.993 | **0.998** | 0.996 |
| 256 | 0.866 | **0.947** | 0.931 |
| 512 | 0.574 | **0.758** | 0.730 |

- **Confirmed:** the magnitude reading threw the phase away. Read in phase, the phase store is the
  best store at every depth, slightly above the classical comparator.
  - Its edge over the classical store is small (+0.016 at F 256), so the ±0.05 tie of Q2 holds
    *with the right reading*.
  - The gain over the thermal medium is still mostly the dimensions read with signed interference.
- **Best outside the thermal state:** 256 per region at 0.947, which is 4,096 memories in 422,400
  bytes at 103 bytes each. That is 64× more per byte than the thermal medium's best for lasting
  memories (6,600 bytes), and 32× more than its best fresh (3,300 bytes).

### What these memories are (a caution, 2026-10-23)

Recall here answers "which of 16 regions holds this key?": about 4 bits per memory, with the key
itself as the cue. So FOLD-1 to 5 measure **capacity** (patterns recognised before they blur), not
data compression. A plain index of the same 4,096 answers takes about 2 KB.

Whether folding saves *data* needs content stored in the fold: key → value binding, the value
recalled, and the bits recalled per byte measured against the raw and compressed size. That is
proposed as FOLD-6.
