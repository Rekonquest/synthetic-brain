# FOLD-4: the best folding in the thermal state, at a fixed medium

Status: PRECOMMITTED (2026-10-23), before any FOLD-4 code.

Jesse (2026-10-23): "fix it, so we get the best results overall". The FOLD-3 correction showed that
more bands make a bigger medium, so growth raises memories per region but not memories per byte.
FOLD-4 holds the medium fixed (32 bands, **422,400 bytes**) and tries the levers that cost no space.

## A diagnostic first (after FOLD-3, not a prediction)

`evidence/fold-3/diagnostic-age.txt`: deeper folding means a longer write (16·F·250 steps), so the
first regions are much older when read. **Age is not the ceiling:**

| depth F | oldest 8 regions | newest 8 regions |
|---|---|---|
| 2 | 0.938 | 0.969 |
| 4 | 0.758 | 0.723 |
| 8 | 0.490 | 0.406 |

The same run shows **most of a region's energy is the stir's** (about 500 per region at every depth,
the stir's steady state in carved sites, about 8 per site), against 320 for one freshly written
memory. That bears on the hot/cold arm.

## The setup (all arms)

SCAR-1 run 5's laws exactly, as in FOLD-1 to FOLD-3:
- 32 bands, 20 × 20 × 4, 16 sealed regions of 4 × 4 × 4, `step_sealed` τ 0.99;
- `spec-trap` E_ref 0.3, ν 5·10⁻⁵, μ_m 0.02, p 3; write gain 0.02; 250 steps per memory, written one
  after another;
- `spec-stir` q 0.02, ε 0.05 (unless the arm changes it);
- read at drain t 250 by containment (`spec-contain`, unchanged; the bridge is reported beside it).

**Region top-1** (each cue finds its region), **set recall** and the **capacity** (the largest F with
region top-1 ≥ 0.8) as before. Test seeds 10–17; dev seeds 0–3 for the rounds. A new orchestrator
crate, `fold4`, holds every setting. No function crate changes.

## The levers and the predictions

### Lever 1: key sparsity (k active bands per key), the plain code

FOLD-3's code: a key is a k-band profile written at every site of its region. k ∈ {3, 6, 12} at
F ∈ {2, 4, 8}. k 6 is the plain bar.

This is a Bloom filter's character (a region is a set of keys over B bands). Its false-match rate is
least at k* = (B/F)·ln 2, which is 11, 5.5 and 2.8 at F 2, 4 and 8.

1. **P1, the best k follows the Bloom optimum:** k 12 ≥ k 6 at F 2; k 3 > k 6 by ≥ 0.05 at F 8; at
   F 4, k 6 is best or within 0.03 of it.
2. **P2, sparsity cannot lift the capacity above 4 at 32 bands.** At the optimum the false-match rate
   per wrong region is 0.6185^(B/F), which is 0.15 at F 8, and there are 15 wrong regions.

### Lever 2: the shape code (use the dimensions the medium already has)

FOLD-1 to 3 read a region as one 32-band spectrum, the sum over its 64 sites. The medium holds
**64 × 32 = 2,048 values per region**, so the reading throws away 63/64 of what the medium stores.

**The shape code:** a key is a 3D shape inside its region, 8 of the region's 64 sites, each carrying
its own 6-band profile (48 of the region's 2,048 cells). The total written energy per step is the
plain code's (64 sites × gain), spread over the key's 8 sites. **The reading:** containment over the
region's 2,048 cells (site × band), the same `spec-contain`.

Why it can hold: a carved edge moves κ(1 − τm)² ≈ 3·10⁻⁵ per step (m ≈ 0.9975), so a shape spreads
about √(κ_e·t) ≈ 1.4 sites over the 64,000 steps of an F 16 write. The cells' occupancy at depth F is
1 − (1 − 48/2048)^F: 0.32 at F 16, 0.53 at F 32.

3. **P3, the shape code raises the capacity to ≥ 16 per region:** 256 memories in the 422 KB medium,
   1,650 bytes each, at region top-1 ≥ 0.8.
4. **P4:** at F 4, region top-1 ≥ 0.95 (the plain code: 0.740).
5. **P5:** set recall ≥ 0.8 at F 16.
6. **P6, the limit is the spread, not occupancy:** at the deepest F that fails, the oldest half of the
   regions is worse than the newest half (the reverse of the plain code's diagnostic).

The shape code is run at F ∈ {1, 2, 4, 8, 16, 32}, and the plain code at F 16 and 32 beside it.

### Lever 3: hot and cold (the stir is the heat), on the winning code

The stir rate q ∈ {0, 0.0025, 0.005, 0.01, 0.02, 0.04} at the winning code's capacity depth (or F 16
for the shape code if it holds there), read at t 250 and t 20,000.

7. **P7, colder is better fresh:** at t 250, top-1 does not fall as q falls from 0.04 to 0.0025. The
   stir's flat spectrum is most of each region's energy, and it only dilutes.
8. **P8, an inverted U over time:** at t 20,000, q 0 (quiet) collapses (as in SCAR-1 run 5, where
   the material erodes without energy), and the best q is 0.005 or 0.01, below the plain 0.02. The
   material needs E ≥ E_ref 0.3 per site, and q 0.005 still holds about 2 per carved site.

### Rounds on the kept setup (dev seeds; kept only if better on test; plain wins ties)

The selector is the dev score: region top-1 at the capacity depth, at t 20,000 (then t 250).

9. **P9, amplification (write gain × 2):** not kept once cooling is kept. Cooling and gain both raise
   the memory against the stir, so they substitute (QB-6: two amplifiers of the same thing).
10. **P10, chaos:** per step and per site, the write gain times 1 + α(u − ½), with u from
    `qb-chaos::keyed_unit(seed, step, site)` (the logistic map, mean ½) and α 0.3. The trap law
    (maintain-or-erase) is the selector inside the medium. Prediction: not kept (a tie), as SCAR-1's
    frozen μ chaos was.

## What "best overall" will mean

The setup with the most memories per byte at region top-1 ≥ 0.8 at t 250, with its t 20,000 value
beside it. The FOLD-3 bar at 32 bands is 2 per region: **13,200 bytes per memory** (0.953).

CPU only, 8 threads, about 25–30 min, results streamed per configuration to
`evidence/fold-4/results.txt`.

## Results (2026-10-23; `evidence/fold-4/`)

**The check:** the plain code at k 6 reproduces FOLD-3 bit for bit (containment 0.953, 0.740, 0.448;
bridge 0.965, 0.666, 0.373 at F 2, 4, 8).

### Lever 1, key sparsity (`sparsity.txt`, 3 min 18 s)

**Region top-1, containment:**

| k (bands per key) | F 2 | F 4 | F 8 | capacity |
|---|---|---|---|---|
| 3 | 0.973 | 0.717 | 0.464 | 2 |
| 6 (plain) | 0.953 | 0.740 | 0.448 | 2 |
| 12 | **0.996** | 0.670 | 0.312 | 2 |

- **P1 FAILED by margin:** the direction follows the Bloom optimum (k 12 best at F 2, k 6 best at F 4,
  k 3 best at F 8), but k 3 at F 8 gains only +0.016, not +0.05.
- **P2 HOLDS:** no k lifts the capacity above 2.
- **Sparsity is not a lever.** It only moves which depth is best.

### Lever 2, the shape code (`shape.txt`, 8 min)

**Region top-1, containment (set recall in brackets), 422,400 bytes:**

| F | memories | bytes each | shape code | plain code | shape: oldest half / newest half |
|---|---|---|---|---|---|
| 2 | 32 | 13,200 | 1.000 (1.000) | 0.953 | 1.000 / 1.000 |
| **4** | **64** | **6,600** | **0.998** (1.000) | 0.740 | 1.000 / 0.996 |
| 8 | 128 | 3,300 | 0.765 (0.931) | 0.448 | 0.828 / 0.701 |
| 16 | 256 | 1,650 | 0.334 (0.497) | 0.169 | **0.060** / 0.609 |
| 32 | 512 | 825 | 0.152 (0.252) | 0.090 | 0.049 / 0.255 |

| # | Result |
|---|---|
| P3 shape capacity ≥ 16 | **FAILED:** 4 (F 8 reaches 0.765) |
| P4 shape at F 4 ≥ 0.95 | MEASURED: **0.998** (plain 0.740) |
| P5 set recall ≥ 0.8 at F 16 | **FAILED:** 0.497 |
| P6 the oldest half is worse at the deepest failing F | MEASURED: 0.049 against 0.255 at F 32 (and 0.060 against 0.609 at F 16) |

- **The shape code doubles the memories per byte at a fixed medium:** 4 per region at 0.998, so 64
  memories at 6,600 bytes each, against FOLD-3's bar of 13,200. It beats the plain code at every
  depth.
- **Its limit is age, unlike the plain code's.** At F 16 the oldest regions are gone (0.060) while
  the newest hold (0.609).
  - P6's check cannot tell spreading from fading. The carved leak, δ(1 − τm) ≈ 1.25·10⁻⁴ per step,
    takes e⁻⁸ of a memory over the 64,000 steps of an F 16 write, which alone would do it.
  - Probably (unmeasured): the plain code reads 64 sites summed, so a faded profile still stands
    above the stir there, and a shape's 8 sites do not.

### Lever 3, hot and cold on the shape code at F 4 (`heat-rounds.txt`)

| stir q | t 250 | t 20,000 (set recall) |
|---|---|---|
| 0 (quiet) | 0.996 | 0.758 (0.834) |
| **0.0025** | 0.998 | **0.762** (0.879) |
| 0.005 | 1.000 | 0.711 (0.838) |
| 0.01 | 1.000 | 0.684 (0.781) |
| 0.02 (plain) | 0.998 | 0.578 (0.695) |
| 0.04 | 0.996 | 0.459 (0.592) |

| # | Result |
|---|---|
| P7 colder is better fresh | **FAILED, at the ceiling:** every q gives 0.996–1.000; q 0.0025 is 0.002 under q 0.005 (one key in 512) |
| P8 an inverted U, quiet collapsing, best q 0.005–0.01 | **FAILED:** the quiet arm does *not* collapse (0.758), and colder is better all the way down to q 0.0025 |

- **Cooling is the shape code's lifetime lever:** q 0.02 → 0.0025 takes t 20,000 from 0.578 to 0.762.
- **Why the quiet arm lives here and not in SCAR-1** (after the run, not tested): a shape writes its
  region's full energy into 8 sites, 8× the plain code's per site, so the carved material there
  holds far longer before it erodes.
- **Kept: q 0.0025** (chosen on the test seeds' measurement, as precommitted; labelled).

### Rounds on the shape code, F 4, q 0.0025 (dev 0–3, then test)

| arm | dev t 250 / t 20,000 | test t 250 / t 20,000 | kept |
|---|---|---|---|
| plain | 1.000 / 0.773 | 0.998 / 0.762 | — |
| **amplification, gain × 2** | 1.000 / **0.965** | 1.000 / **0.914** | **yes** |
| chaos on the write gain, α 0.3 | 1.000 / 0.773 | — | no (a tie) |

| # | Result |
|---|---|
| P9 amplification not kept (substitutes for cooling) | **FAILED:** gain × 2 on top of cooling lifts t 20,000 from 0.762 to **0.914** |
| P10 chaos not kept | MEASURED: an exact tie |

- **Cooling and gain stack here; they do not substitute.** QB-6's substitution was two amplifiers
  of the same signal. Here cooling removes noise and gain adds signal, so they act on two different
  things, which is why both help.

**FOLD-4: 4 of 10 hold.** Six predictions missed, but the stack moved further than predicted.

### Added after the rounds (not a prediction): the kept setup by depth (`kept-by-depth-added.txt`)

The shape code at plain settings against the kept setup (shape code, q 0.0025, gain × 2); region top-1
at t 250 / t 20,000:

| F | memories | bytes each | shape, plain settings | **shape, kept setup** |
|---|---|---|---|---|
| 4 | 64 | 6,600 | 0.998 / 0.578 | **1.000 / 0.914** |
| 8 | 128 | 3,300 | 0.765 / 0.302 | **0.948** / 0.501 |
| 16 | 256 | 1,650 | 0.334 / 0.168 | 0.487 / 0.253 |

At F 16 the kept setup's oldest half is at 0.146 and its newest at 0.828: the limit is still age.

## The best folding in the thermal state (32 bands, 422,400 bytes)

| | memories | bytes each | t 250 | t 20,000 |
|---|---|---|---|---|
| FOLD-3's bar (plain code, F 2) | 32 | 13,200 | 0.953 | — |
| **fresh: kept setup, F 8** | **128** | **3,300** | **0.948** | 0.501 |
| **lasting: kept setup, F 4** | **64** | **6,600** | **1.000** | **0.914** |

- **4× FOLD-3's bar for fresh memories, 2× for memories that must last,** with no extra space.
- **The levers that worked:**
  - the shape code, which writes in the dimensions the medium already has;
  - cooling;
  - write gain.
- **The levers that didn't:** key sparsity, and chaos.
- **Next for depth:** the limit is now age (the carved leak), not occupancy.
