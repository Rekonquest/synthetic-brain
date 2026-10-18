# T2: the bath, so temperature is real

Status: PRECOMMITTED (2026-10-18), before any code.

## The stack

`thermo-bath` couples a real field φ on T1's lattice (φ may be negative; the energy is φ²/2) to a
temperature T. **The fluctuation–dissipation pairing:** every dissipative channel carries matched
noise, or the medium settles colder than T.

1. **Damping (model A), at each site,** the exact OU step towards the local potential μ_i = φ_i − h_i:
   φ ← h + (φ − h)·e^{−γ} + √(T·(1 − e^{−2γ}))·ξ.
2. **Exchange with conserved noise (model B), on each edge (i, j):**
   - f = (κ/2)·(μ_j − μ_i) + √(T·κ(2 − κ)/2)·ξ_e, then φ_i += f and φ_j −= f;
   - the edges of each axis are updated in two disjoint halves (even, then odd), so that each update
     sees current values;
   - an i.i.d. N(h_i, T) state is then exactly stationary (derived in the code doc).
3. **The noise:** ξ is a standard normal from keyed uniforms (Box–Muller), deterministic per (seed,
   step, site or edge).

The axis count never enters a law. **The lattice is sized for expansion:** for T2 the field is
stationary, so the size only needs enough sites for statistics, 32² in 2D and 16³ in 3D.

## Its own checks (predictions)

1. **B1, equipartition:** the stationary per-site variance is T within 2%, for T ∈ {0.5, 1, 2}, in d = 2
   and d = 3 (after a burn-in of 200 steps, measured over 400).
2. **B2, linearity:** the variance at T = 2 is twice that at T = 1, within 3%.
3. **B3, response equals fluctuation (static FDR):** a field h = 0.5 at one site gives a measured mean
   ⟨φ₀⟩ of h·C₀₀/T within 5% (C₀₀ being the equal-time variance, so the mean is h), and the neighbours
   stay at ≈ 0 (|mean| < 0.05).
4. **B4, conservation:** with no damping (γ = 0) and T = 0, the exchange conserves Σφ to 10⁻⁹.

## Optimization round, the check being B1's equipartition error (lower is better; plain wins ties)

- **Chaos on and off:** a mean-preserving per-site perturbation of γ by the logistic map,
  α ∈ {0.1, 0.3}.
  - **Prediction B5:** not kept. Equipartition holds for any γ (each site's OU step is exact), so chaos
    only adds sampling noise.
- **Hot and cold:** T *is* this rung's dial. The B1 sweep over T is the hot/cold test, and every T is
  checked.
- **Amplification on and off:** noise gain ×1.2 (amplifying the fluctuations).
  - **Prediction B6:** not kept. It breaks fluctuation–dissipation, and the variance becomes 1.44·T.

## Results (2026-10-18, `evidence/thermo-2/results.txt`; diagnostic in `results-with-diagnostic.txt`)

The tests passed before the run: the cold exchange conserves to 10⁻⁹, and Box–Muller is standard
(mean 0 and variance 1, to 10⁻³ and 1%). The runner maps the logistic map's arcsine-distributed values
to uniform before Box–Muller.

| d | T = 0.5 | T = 1 | T = 2 |
|---|---|---|---|
| 2 (32²) | 0.4994 (−0.11%) | 0.9989 (−0.11%) | 1.9977 (−0.11%) |
| 3 (16³) | 0.5001 (+0.03%) | 1.0003 (+0.03%) | 2.0005 (+0.03%) |

| # | Result | What was measured |
|---|---|---|
| B1 equipartition within 2% | MEASURED | ≤ 0.11% in every case |
| B2 linearity within 3% | MEASURED | T = 2 / T = 1 = 2.000 |
| B3 response equals fluctuation | MEASURED | push 0.5 → mean 0.522 at the site (+4.3%, bar 5%); neighbours \|mean\| 0.022 |
| B4 cold conservation | MEASURED | drift 1.4e-14 |
| B5 chaos not kept | **FAILED by the strict rule** | α 0.3 gives an error of 0.0002 against plain's 0.0003 |
| B6 amplification not kept | MEASURED | noise ×1.2 gives an error of 0.440 (variance 1.44·T, as derived) |

**5 of 6 hold.**

**B5, analysed:** equipartition is exact for *any* damping γ (each site's OU step is exact), so chaos on
γ cannot change the physics, only the sampling. The diagnostic, after the run, measured the check's
own resolution: plain's error across noise seeds 2–7 is **0.0001–0.0029**. Chaos's 0.0002 lies inside
that spread. **Decision, labelled:** T2 carries plain up, because a "win" smaller than the check's
seed-to-seed spread is not one. This is the same call QB-8 made on QB-5's selection-luck chaos.

**What T2 settles for the rungs above:**
- the medium has a real temperature, with fluctuations ∝ T exactly;
- response and fluctuation agree;
- fluctuation–dissipation is paired on both dissipative channels;
- amplifying the noise breaks it, as derived.
