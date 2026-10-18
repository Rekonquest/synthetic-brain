# T3: carving threads into the medium (maintain-or-erase)

Status: PRECOMMITTED (2026-10-18), before any code.

## The stack

`thermo-carve` gives each edge of the lattice a conductance c_e ∈ [c_min, 1]. The edge's exchange rate
is κ·c_e, using T2's exchange with per-edge κ and the fluctuation–dissipation noise per edge.

**The law** (the universum substrate's maintain-or-erase, with Tero et al.'s *Physarum* adaptive-network
rule as prior art):
- c ← c + μ·g(|f_e| / f_ref)·(1 − c) − ν·(c − c_min), where g(x) = xᵖ / (1 + xᵖ);
- f_e is the edge's deterministic flux;
- **erosion is 10× slower than maintenance** (ν = μ/10), the rule FABRIC-2 and the substrate's
  compound-c taught;
- p is the amplification: how strongly flow feeds conductance. p = 1 is plain.

**The traffic:** two sites A and B are held at potentials +1 and −1 by strong damping there (T2's exact
OU step, γ 1). Everywhere else γ = 0, so the medium only conducts. A steady current flows from A to B.

**The settings:**
- κ 0.3, μ 0.02, ν 0.002, c_min 0.05, starting c₀ 0.2, f_ref = κ/D;
- 3000 carving steps;
- **sized for expansion:** 2D side **64** with A and B 10 apart (straight) or (6, 6) apart (diagonal);
  3D side **32** with A and B 6 apart.

## Its own checks (predictions)

1. **C1, a thread forms:** in 2D straight, the mean conductance on the edges along the A–B segment is
   at least 3× that of the edges at distance ≥ 5 from the segment.
2. **C2, erosion has the designed timescale:** after the drive is removed, the path's excess
   conductance decays as e^{−νt}. The fitted rate is within 10% of ν.
3. **C3, the carved web conducts better:** the steady A → B current after carving is at least 1.5× the
   current through the uncarved mesh (c₀ everywhere).
4. **C4, the same in 3D:** the thread ratio is ≥ 3.
5. **C5, the mesh is large enough** (Jesse's sizing rule), as **finite-size convergence**:
   - growing the side from 64 to 96 (2D), and from 32 to 48 (3D), changes the thread ratio and the
     A → B current by under 2%;
   - a pre-run correction: the first wording ("the current through a boundary shell") was malformed,
     because the net current through any closed shell is zero by conservation. Convergence under
     enlargement is the real test of "large enough". It was corrected before any T3 code ran.

## Optimization round, judged on the diagonal case's selectivity

**The check:** the share of the total excess conductance Σ(c − c_min) held by the top 12 edges (one
shortest diagonal path has 12 edges). Higher is a sharper thread. Rounds are sequential from the kept
best, plain wins ties, and each is kept only if better on a second, held-out diagonal (the pair (5, 7)).

- **Amplification on and off, p ∈ {2, 3}.**
  - **Prediction C6:** kept. *Physarum* with p > 1 selects a single path among equal ones.
- **Chaos on and off:** a mean-preserving per-edge perturbation of μ, α ∈ {0.1, 0.3}.
  - **Prediction C7:** kept. It breaks the symmetry between equal shortest paths, so one thread wins.
- **Hot and cold:** the medium's temperature T ∈ {0.005, 0.02}. Thermal flux noise carves at random.
  - **Prediction C8:** not kept (cold is best). Its fluctuating fluxes maintain edges off the path.

## Build

- An additive per-edge exchange in `thermo-bath` (`exchange_edges`). T2's runner must reproduce its
  output byte for byte after it is added.
- The new crate `thermo-carve` (the law) and the runner `thermo3`.
- **Unit tests:**
  - with c = 1 everywhere and no carving, `exchange_edges` equals T2's `exchange`;
  - an edge with no flux decays as e^{−νt} towards c_min;
  - g(1) = ½.

## Run 1: VOID (a wiring bug in the runner, kept at `evidence/thermo-3/results-VOID-drive-bug.txt`)

The first run passed the ±1 reservoir drive into the exchange as T2's equilibrium field h. That
makes the exchange see zero potential *difference* at A and B, so no current flowed: the current
was about 0, and every thread ratio was exactly 1.00.

- Like STACK-CATALOG rule 10, the wiring made the comparisons vacuous. None of its verdicts stand.
- **The repair:** the drive enters through the damping only, and the exchange runs on raw differences.
- **Also:** the noise is now not generated at T = 0 (the void run took 5 min 40 s for that reason).
- The predictions are unchanged.

## Run 2: results (2026-10-18, `evidence/thermo-3/results.txt`, 56 s)

The tests passed: g(1) = ½, the zero-flux decay is exactly (1 − ν)ᵗ, and the per-edge exchange equals
T2's at a uniform rate. T2's output is byte-identical after the additive change.

| Check | Measured |
|---|---|
| C1 the 2D straight thread ratio | **6.87** (bar 3) |
| C2 erosion | fitted 0.00200 against ν 0.002 (+0.1%) |
| C3 conduction | carved 0.1560 against uncarved 0.0484: **×3.23** (bar ×1.5) |
| C4 the 3D thread ratio | **10.71** |
| C5 convergence | **current:** 2D 0.15600 → 0.15600, 3D 0.29797 → 0.29794. **Thread ratio:** 2D 6.87 → 8.96, 3D 10.71 → 11.14 |

**The rounds**, on diagonal selectivity: the top-12 share, dev (6, 6), held-out (5, 7).

| Round | dev variants | held-out off → on | verdict |
|---|---|---|---|
| amplification p | p 2: 0.150; **p 3: 0.238** (plain 0.020) | 0.021 → 0.244 | **KEPT (p 3)** |
| chaos (from p 3) | changing 0.1 / 0.3: 0.238 / 0.239; frozen 0.1: 0.246; **frozen 0.3: 0.261** | 0.244 → 0.264 | **KEPT (frozen 0.3)** |
| heat (from the above) | **T 0.005: 0.398**; T 0.02: **0.003** | 0.264 → 0.397 | **KEPT (T 0.005)** |

| # | Result |
|---|---|
| C1, C2, C3, C4 | MEASURED |
| C5 finite-size convergence | **FAILED as written:** the current converges to 5 digits; the thread ratio does not |
| C6 amplification kept | MEASURED (p 3: ×12 selectivity) |
| C7 chaos kept | MEASURED (frozen per-edge heterogeneity breaks the tie between equal paths) |
| C8 heat not kept | **FAILED:** a little heat is kept |

**6 of 8 hold.**

### The two misses, analysed

**C5:** the physically meaningful quantity, the A → B current through the web, converges to 5 digits
in 2D and 3D, so the mesh is large enough. The thread *ratio* does not converge because its
denominator averages *all* edges at distance ≥ 5, which includes a far field whose area grows with
the side and whose edges erode to c_min. The metric is not size-intensive; that is my metric's flaw,
not the mesh's size. **For T4 onward:** off-path baselines are a fixed band (distance 5–10), never
"everything far".

**C8, a real finding:** heat follows an **inverted U**:
- a little (T 0.005) **sharpens** the thread by half again (0.261 → 0.398, held-out 0.264 → 0.397);
- four times more (T 0.02) **melts** it (0.003).

The small thermal fluctuations act as annealing: they let the competing equal paths escape their
tie, and the strongest one is amplified (p 3). Too much heat maintains every edge at random. **This is
Jesse's hot/cold theory, measured in the medium: there is an optimal temperature, and both sides of it
are worse.** My prediction (cold best) was wrong.

**Carried up to T4:** p 3, frozen chaos 0.3, T 0.005, and erosion 10× slower than maintenance.
