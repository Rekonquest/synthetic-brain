# GEO-1: geometric and spatial reasoning stacks, with the next physics laws

Status: PRECOMMITTED (2026-10-24), before any GEO-1 code.

Jesse (2026-10-24): "lets explore atom stack reasoning types, and limit it to geometric and spatial
reasoning for now. And do the next physics laws."

**Amended before code** (Jesse, 2026-10-24): "let's not use existing stacks for discovery, because
we are looking for better, not the status quo." Existing methods appear only as **bars to beat**
(plain comparison, mental-rotation search, greedy walking). Every reasoning stack below is new,
built from a physics law. His BUILD-8 and BUILD-9 (2026-09-25, an integer torus) are history here,
not a basis.

The physics laws as reasoning strategies:
- **Noether's theorem:** a symmetry has a conserved quantity, so reason with the conserved
  quantities;
- **spontaneous symmetry breaking:** commit to one frame out of a symmetric set;
- **least action, as Feynman's sum over paths:** weight every path by e^(−action), and the
  stationary path dominates;
- **the renormalization group:** coarse-grain, solve, then refine.

The rules are the same as before:
- each law is its own stack against plain, then chaos off/on, then amplification on/off;
- discovery probes, with each stack's niche recorded;
- f64;
- test seeds 10–17 and dev seeds 0–3.

## G1: recognition and handedness under rotation (Shepard–Metzler)

- **The library:** 32 shapes, each 8 random points in the unit square, centred.
- **A query:** a library shape rotated by a uniform random angle, translated, mirrored with
  probability ½, with its points shuffled (the correspondence is unknown) and noise σ_p added.
- **The task:** name the shape and say whether it is mirrored.
- 256 queries per seed; σ_p ∈ {0.01, 0.02, 0.05} (the heat sweep).

**The strategies:**
- **Plain:** compare the centred query with each library shape (and its mirror) by the symmetric
  Chamfer distance, with no rotation.
- **Mental rotation (search; the human strategy):** rotate the query in 5° steps outward from 0°
  (both mirror states). Stop at the first angle where the best Chamfer distance is below a threshold
  τ (chosen on dev), otherwise take the best over all angles. Steps are counted.
- **Noether (conserved charges; new):** treat the centred points as complex numbers z_i and take
  the moments m_k = Σ z_i^k.
  - A rotation by θ multiplies m_k by e^{ikθ}: m_k carries rotation "charge" k. **Any product of
    moments whose charges sum to zero is conserved** by every rotation (the same neutrality rule as
    colour confinement and gauge invariance).
  - **Identity:** the neutral invariants |m_k| (k = 2…6) and Re(m₂³·m̄₃²) and Re(m₃²·m̄₂³·…)
    families (the conserved charges), each scaled by the shape's size, compared by L2. The sorted
    pairwise-distance multiset is added as a second conserved family.
  - **Handedness:** a reflection conjugates every z_i, so the imaginary part of a neutral product
    changes sign. Mirror = sign Im(m₂³·m̄₃²) against the library shape's. No point correspondence is
    needed.
  - No rotation and no search.
- **Symmetry breaking (a canonical frame):** break the rotational symmetry by the shape's principal
  axis (its second moment). Fix the axis's sign by the third moment along it. The mirror is fixed by
  the third moment's sign along the second axis. Then compare the canonical poses by Chamfer.

**New function crates:** `shape-chamfer`, `shape-rotate`, `shape-moments` (the complex moments
m_k), `shape-distances` (the sorted distance multiset) and `shape-canon` (principal-axis frame).
Orchestrator `geo1`.

**Predictions:**
1. **G1a:** plain identity ≤ 0.2 (a random rotation defeats it).
2. **G1b:** mental rotation identity ≥ 0.95 and mirror ≥ 0.95 at σ_p 0.02. **Its steps grow with the
   rotation angle** (the Shepard–Metzler signature): the correlation of steps with |angle| is
   ≥ 0.8.
3. **G1c:** Noether identity ≥ 0.95 at σ_p 0.02 at a constant cost (no search). Mirror ≥ 0.9; it
   fails only where Im(m₂³·m̄₃²) is near zero (a nearly mirror-symmetric shape).
4. **G1d:** symmetry breaking identity ≥ 0.8, failing mostly on shapes whose principal axes are
   nearly degenerate (an eigenvalue ratio near 1). Those failures concentrate in the third of shapes
   with the most isotropic second moment.
5. **G1e, chaos** (the search step jittered by 1 + 0.5(u − ½), Chamfer selects): not kept.
6. **G1f, amplification** (more charges: moments up to k = 10 and more neutral products): kept at
   σ_p 0.01 (+≥ 0.01) and not at σ_p 0.05, where high-charge moments are noise-dominated.

## G2: planning a path (least action, the renormalization group)

- **The world:** a 64 × 64 grid with random obstacles at density ρ, and a start and goal in opposite
  corners that are connected. 64 worlds per seed, ρ ∈ {0.15, 0.25, 0.35}.
- **The ground truth:** BFS shortest path length.

**The strategies:**
- **Plain (greedy):** step to the free neighbour nearest the goal (Euclidean), never revisiting. A
  dead end is a failure; the step cap is 4 × BFS.
- **Least action, as Feynman's sum over paths:**
  - a field u is held at 1 on the goal and relaxed on free cells as u_s = (λ/4)·Σ neighbours u
    (λ 0.5, until fixed);
  - then u_s = Σ over paths of (λ/4)^length, so ln u ≈ −length·ln(4/λ) + ln(number of paths), and
    the shortest paths dominate;
  - the walker climbs u from the start.
- **The renormalization group (coarse → fine):**
  - coarse-grain 4 × 4 blocks (a block is free if at least half its cells are);
  - run least action on the 16 × 16 coarse grid;
  - refine with least action restricted to the coarse path's blocks and their neighbours;
  - if the restricted refinement fails, fall back to the full grid (counted).
- **Cost:** relaxation updates, summed.

**New function crates:** `path-sum` (one relaxation sweep of the sum over paths) and `path-climb`
(follow the field uphill). Orchestrator `geo2`.

**Predictions:**
7. **G2a:** greedy success ≤ 0.6 at ρ 0.25.
8. **G2b:** least action success 1.000, with path length ≤ 1.02 × BFS on average.
9. **G2c:** the renormalization group's cost ≤ ¼ of full least action, success ≥ 0.95 (with the
   fallback), and length ≤ 1.1 × BFS.
10. **G2d, chaos** (the climb's ties broken by keyed logistic chaos, path length selects): not
    kept.
11. **G2e, amplification** (λ 0.25: a stronger weighting of short paths): length closer to BFS by
    ≥ 0.005 at ρ 0.35, at no loss of success.

## Results (2026-10-24, `evidence/geo-1/g1.txt`, `g2.txt`; 3 s and 1 s)

### G1, recognition and handedness under rotation: 3 of 6

Dev chose τ 0.05 for the search.

| strategy | noise 0.01: identity / + mirror | 0.02 | 0.05 | cost |
|---|---|---|---|---|
| plain (bar) | 0.218 / 0.131 | 0.211 / 0.129 | 0.187 / 0.121 | 1 |
| mental rotation (bar) | 1.000 / 0.998 | **1.000 / 1.000** | **0.995 / 0.987** | 69–121 angle steps (each against all 32 shapes) |
| **Noether charges (new)** | **1.000** / 0.911 | **0.999** / 0.845 | 0.776 / 0.544 | **1** |
| symmetry breaking (new) | 0.942 / 0.919 | 0.896 / 0.861 | 0.731 / 0.673 | 1 |
| search + chaos | as search | as search | 0.995 / 0.987 (120.7 steps) | |
| Noether + more charges | 1.000 / 0.911 | 1.000 / 0.845 | 0.801 / 0.561 | 1 |

- **The Shepard–Metzler signature holds:** the search's steps grow with the rotation angle at a
  correlation of **0.999**.
- **Symmetry breaking fails where predicted:** 120 of its 214 failures are shapes in the most
  isotropic third.

| # | Result |
|---|---|
| G1a plain ≤ 0.2 | **FAILED by margin:** 0.211 |
| G1b search ≥ 0.95, steps ∝ angle | MEASURED: 1.000, correlation 0.999 |
| G1c Noether identity ≥ 0.95 at cost 1, + mirror ≥ 0.9 | **FAILED on the mirror:** identity 0.999 in one comparison, but handedness 0.845 |
| G1d canon ≥ 0.8, failures in the isotropic third | MEASURED: 0.896; 120 of 214 |
| G1e chaos not kept | MEASURED |
| G1f more charges kept at 0.01, not at 0.05 | **FAILED, reversed:** a tie at 0.01, +0.025 identity at 0.05 |

### G2, path planning: 3 of 5

| strategy | ρ 0.15: success / length ÷ BFS | ρ 0.25 | ρ 0.35 | cost (updates per world) |
|---|---|---|---|---|
| greedy (bar) | 0.416 / 1.037 | 0.018 / 1.084 | 0.000 | 0 |
| **least action, sum over paths (new)** | **1.000 / 1.0001** | **1.000 / 1.0014** | **1.000 / 1.0013** | 557k–678k |
| renormalization group (new) | 1.000 / 1.0001 | 1.000 / 1.0051 | 1.000 / 1.0066 | 248k / 255k / 602k |
| least action, λ 0.25 | 1.000 / 1.0000 | 1.000 / 1.0005 | 1.000 / 1.0005 | 473k–550k |

RG fell back to the full grid 0, 37 and 410 times (of 512) at ρ 0.15, 0.25 and 0.35.

| # | Result |
|---|---|
| G2a greedy ≤ 0.6 at 0.25 | MEASURED: 0.018 |
| G2b least action 1.000, ≤ 1.02 × BFS | MEASURED: 1.000, ≤ 1.0014 |
| G2c RG ≤ ¼ cost, ≥ 0.95, ≤ 1.1 | **FAILED on cost:** 0.44 at ρ 0.25 (success 1.000, length 1.005). In dense worlds the coarse blocks close narrow passages and force the fallback |
| G2d chaos ties not kept | MEASURED: identical |
| G2e λ 0.25 closer by ≥ 0.005 | **FAILED by size:** 1.0013 → 1.0005 (and 15% cheaper) |

**GEO-1: 6 of 11 hold.**

### Niches

- **Least action as a sum over paths:** the planner. It never fails, it is near-optimal, and it is
  built from one relaxation law, with no search algorithm written in.
- **Noether charges:** identity in one comparison, the fastest recognizer, weak at handedness and
  under heavy noise.
- **Mental rotation (the bar):** exact but slow, with a human-like cost that grows with the angle.
- **Symmetry breaking:** fast, except for nearly isotropic shapes.
- **The renormalization group:** a cheap planner in sparse worlds.
