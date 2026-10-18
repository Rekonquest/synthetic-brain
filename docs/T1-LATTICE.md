# T1: the d-dimensional lattice and the exchange law

Status: PRECOMMITTED (2026-10-18), before any code.

## The stack

`thermo-lattice` holds a periodic lattice of side L in d dimensions, with one non-negative energy per
site.

**The exchange law** is local diffusion, applied as one pass per axis in turn:
e_i ← e_i + (κ/2)·(e_{i+a} + e_{i−a} − 2e_i), where a is the axis's stride. The law is written once.
The axis count appears only in how many passes run, never in the law.

**Settings** (the orchestrator's): κ = 0.2, and L = 32 for d ≤ 3 and 16 for d = 4.

## Its own checks (predictions)

1. **L1, conservation:** the total energy is constant to a relative 10⁻⁹ over 200 steps, for d = 1, 2, 3
   and 4.
2. **L2, the diffusion law:** from an impulse at the origin, the mean squared spread (by minimum-image
   distance) is m2(t) = d·κ·t to within 1% for t ≤ 40, for each d.
3. **L3, isotropy:** the spread per axis agrees across axes to within 1%.
4. **L4, positivity:** no site ever goes negative for κ ≤ 1 (each axis pass is a convex average).
5. **L5, the same law in every d:** a d = 3 lattice with one axis of length 1 behaves exactly like the
   d = 2 lattice (it is bit-identical). The dimension is the lattice's, not the law's.

## Optimization round (Jesse's protocol), its own check = L2's fidelity

- **Chaos on and off:** a mean-preserving per-edge perturbation of κ, 1 + α(x − ½) with logistic x,
  α ∈ {0.1, 0.3}, per edge and per step.
  - **Prediction L6:** not kept. The plain law is already exact, so chaos can only add error; plain
    wins.
- **Hot and cold:** none at this rung. Temperature enters at T2. The rung is recorded as "no
  temperature", not skipped.
- **Amplification:** off by law. Conservation forbids gain in transport, and growth would create
  energy. Recorded as "amplification excluded by the law".

## The independent recomputation

The m2 check is recomputed in the test with a separate, closed-form random-walk variance (per axis, a
pass adds κ to the variance). The two must agree.

## Results (2026-10-18, `evidence/thermo-1/results.txt`; diagnostic in `results-with-diagnostic.txt`)

The tests passed before the run: neighbours wrap correctly, and the spread matches the closed-form
random-walk variance (κ·t per axis) to 10⁻⁶.

| d | L1 drift | L2 spread error (mean / worst) | L3 anisotropy | L4 | per-axis m2 at t = 40 (want 8.000) |
|---|---|---|---|---|---|
| 1 | 2.3e-16 | 1.4e-8 / 1.8e-7 | 0 | positive | 8.000 |
| 2 | 1.1e-15 | 1.4e-8 / 1.8e-7 | 5.6e-16 | positive | 8.000 × 2 |
| 3 | 1.1e-16 | 1.4e-8 / 1.8e-7 | 2.1e-14 | positive | 8.000 × 3 |
| 4 (side 16) | 4.4e-16 | 3.1e-3 / **1.6e-2** | 3.6e-14 | positive | 7.870 × 4 |

| # | Result |
|---|---|
| L1 conservation | MEASURED (≤ 1.1e-15 in every d) |
| L2 the diffusion law within 1% | **FAILED, in d = 4 only:** worst 1.6% at side 16 |
| L3 isotropy | MEASURED |
| L4 positivity | MEASURED |
| L5 the same law in every d (d = 3 with a unit axis ≡ d = 2, bit for bit) | MEASURED |
| L6 chaos not kept | MEASURED: plain wins on every d (the exact law cannot be improved) |

**5 of 6 hold.**

**L2's miss is the precommitted lattice size, not the law.** After 40 steps the per-axis spread is √8
≈ 2.8 sites. On a side of 16 (a half-width of 8), the tails wrap around and their minimum-image
distance is undercounted.

The diagnostic, added after the run and labelled as such: at side 24 the 4D worst error falls to
**1.2e-4**. At side 16 it reproduces 1.6e-2. The law is exact in every dimension tested, and the miss
was mine (the 4D side chosen too small for t = 40).

**What T1 settles for the rungs above:**
- the exchange law conserves, is isotropic, and is dimension-agnostic, bit for bit;
- chaos has nothing to add to an exact transport law;
- hot and cold enter at T2;
- amplification is excluded by conservation.

**The rule carried up:** size a lattice so that the spread over the run stays well inside the
half-width, or wrap-around is measured as physics.
