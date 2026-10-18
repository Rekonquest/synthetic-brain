# T6, THERMO-1: thinking hot and cold

Status: PRECOMMITTED (2026-10-18), before any code.

## Jesse's question

"The human brain itself contains a state-like base, perhaps similar to a thermodynamic state. Cold
and hot affect a person's ability to think, so maybe the thermo base can be that state, where
everything else lives in. I also wonder how the quantum and atom stacks would work in that state,
better or worse."

T6 puts T5's hosted brains into a base with a real temperature and sweeps it.

## The setup

**The brains:** T5's hosted quantum and classical brains, with M remote at r = 2 in T5's medium (side
16, κ 0.5, k 4, γ_L 0.1). The world, seeds and ports are T5's.

**The base temperature field θ(site):**
- **uniform:** one θ everywhere;
- **local:** two regions, each site belonging to the nearer port along x (the minimum image, ties to
  A). The A-region (sensor, D, T) is at θ_A, and the B-region (M) at θ_B.

**Everything takes its temperature from the base at its own site.** The couplings are identities, so
no constant is fitted:
- **the medium:** T2's bath at the local θ on every species. The exchange noise on an edge uses the
  mean of its two ends.
- **M:**
  - quantum: the occupation heat is ε = θ_B (QB-9's tuned value was 0.02);
  - classical: the register floor is ε_c = θ_B (QB-9 tuned 0.05).
- **T's decision:** both brains draw from temper(weights, θ_A), the Boltzmann choice at the base
  temperature (QB-9 tuned 0.05 for quantum, and argmax for classical).
  - **Thin policy, labelled:** θ_A < 0.01 is taken as the greedy limit, argmax, because temper's
    1/θ overflows.

So QB-9's two tuned brains are points of the local grid: quantum at (θ_A, θ_B) = (0.05, 0.02), and
classical at (< 0.01, 0.05).

**Amplification:** the send gain g on both ports, g ∈ {1, 1000}.
- At r 2 the received signal is about 0.005·g against a noise of √θ (T5).
- g 1000 makes the channels clean up to θ 0.2. At g 1 the channels are noise from θ ≈ 10⁻⁶.

**The noise, for speed (labelled):** a uniform kick on [−√3, √3] from a hash, which has unit variance,
replaces Box–Muller. The bath's variance law needs only zero mean and unit variance. Check E0 tests
that this holds.

**The grid:** θ ∈ {0, 0.01, 0.02, 0.05, 0.1, 0.2}.
- **Uniform:** every θ, both gains, both brains, on test. The best uniform θ for each brain is chosen
  on dev at g 1000.
- **Local:** every (θ_A, θ_B) pair, both brains, at g 1000, chosen on dev. The uniform settings are
  the grid's diagonal.

**The check is acc_resp,** the brain's own output, because T6 judges the whole system.

## Predictions (on test, seeds 100–131)

0. **E0, the base has the temperature it is set to:** with no signals, the stationary per-site
   variance of a species field at uniform θ 0.05 is θ within 2% (100 burn-in steps, 100 measured).
1. **P1, an inverted U (Jesse's theory):** at g 1000 and uniform θ, for both brains, the best interior
   θ (0.01–0.1) beats both θ 0 and θ 0.2 by ≥ 0.005.
2. **P2, quantum peaks colder and degrades faster (the plan's prediction):**
   - the dev-chosen θ* is lower for quantum than for classical, or equal;
   - and the drop from acc(θ*) to acc(0.2) is larger for quantum.
3. **P3, heat silences unamplified signalling:**
   - at g 1, for every θ ≥ 0.01 and both brains, acc_resp is ≥ 0.05 below g 1000 at the same θ;
   - and the g-1 curve peaks at θ 0.
   - **This is amplification kept.**
4. **P4, local beats uniform, for both brains:** the dev-chosen local pair beats the dev-chosen
   uniform θ by ≥ 0.003 on test.
   - **The derivation is opposite for the two:**
     - quantum wants its memory *colder* than its decision (ε 0.02 against 0.05);
     - classical wants its memory *warmer* than its decision (floor 0.05 against argmax).
   - No single θ serves both modules of either brain.
5. **P5, frozen chaos on the temperature field is not kept:** a per-site θ·(1 + α(x − ½)) with α 0.3
   (keyed logistic, frozen), at each brain's best local pair, does not beat plain on dev, or does not
   hold on test. The modules read θ only at their own site, so heterogeneity elsewhere only moves
   the channel noise.

**Kill criterion:** none. T6 is the experiment the ladder was built for. Every outcome is recorded as
measured.

## Build

The runner `thermo6` copies T5's run and changes three things:
- the module temperatures and the decision come from the base;
- the bath runs at the local θ: it is called at temperature 1, with the per-site (or per-edge) noise
  scaled by √θ, which is identical to T2's law;
- the uniform noise.

Jobs are (configuration, seed) pairs run on a 24-thread pool.
