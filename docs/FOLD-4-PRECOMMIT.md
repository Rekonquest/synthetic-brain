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
