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

## Run 1: STOPPED, unrun (2026-10-18)

Jesse stopped the run 93 min in, at 1,600 of the 1,920 batch-1 jobs, because the
24-thread CPU load had made his room too hot. The runner only averages results when a batch finishes,
so no accuracy was recorded.

| # | Status |
|---|---|
| E0 | **MEASURED:** variance 0.05014 at θ 0.05 (+0.28%, bar 2%), printed before the batch |
| P1–P5 | **UNRUN** |

The predictions stand unchanged for the next run.

**Before rerunning, made-for-speed changes that must not alter any result:**
- at T = 0, the medium's exact linearity (T5's H2, 8.9·10⁻¹⁷) lets a pulse-response convolution
  replace the lattice;
- a specialised bath loop, checked equal to T2's bath number for number;
- optionally, a GPU adapter crate;
- and per-configuration results streamed to disk, so a stopped run keeps what it finished.

## Made ready to rerun (2026-10-18)

The changes are speed and safety only. The precommitted design and predictions are unchanged.
- **`thermo-bath-fast`:** T2's bath in planned form, with precomputed edge sweeps and inlined noise.
  - Its unit test holds it **bit-identical** to `thermo-bath` over 40 steps, hot and cold, including
    signed zeros.
  - T6's own parity mode (`T6_PARITY=1`) runs a hot and a cold job both ways: **0 of 2,000 choices
    differ**. A hot job takes 8.2 s against 34.9 s, and a cold job 1.9 s against 33.5 s.
- **The job log** (`evidence/thermo-6/jobs.csv`) gets one line per finished (configuration, seed),
  flushed as it finishes.
  - A restart skips every logged job, and the means are taken in seed order (deterministic).
  - `T6_SMOKE=1` checks it: a second pass reruns nothing and gives identical results.
- **A thread cap** (`T6_THREADS`, default 8) keeps the room cool. The estimated full run is about
  35 min.
- **Noted while smoke-testing (no defect):** on seeds 100–101 the two brains score identically,
  although their choices differ at 90–102 steps (27 of them scored). Where they disagree, both are
  wrong.

## Run 2: results (2026-10-18, `evidence/thermo-6/results.txt` and `jobs.csv`; 32 min on 8 threads)

**E0:** variance 0.05014 at θ 0.05 (+0.28%).

**Uniform base temperature, test** (acc_resp, with cued / missed):

| θ | 0 | 0.01 | 0.02 | 0.05 | 0.1 | 0.2 |
|---|---|---|---|---|---|---|
| quantum, g 1000 | 0.753 (0.925/0.489) | 0.770 | **0.777** (0.911/0.570) | 0.764 | 0.741 | 0.716 |
| classical, g 1000 | 0.705 (0.926/0.363) | 0.746 | 0.766 | **0.777** (0.909/0.573) | 0.760 | 0.709 |
| quantum, g 1 | **0.753** | 0.727 (0.802/0.610) | 0.727 | 0.727 | 0.724 | 0.709 |
| classical, g 1 | 0.705 | **0.732** (0.811/0.608) | 0.732 | 0.729 | 0.721 | 0.686 |

**Local temperatures, dev, g 1000** (the full grid is in the results file):
- **the rows (θ_A, the decision) barely matter:** for θ_A ≤ 0.1, every column changes by ≤ 0.01;
- **the columns (θ_B, the memory) carry almost all of it:** quantum 0.773 → **0.795 at θ_B 0.02** → 0.755,
  and classical 0.721 → **0.797 at θ_B 0.05** → 0.772.

**The dev-chosen settings, on test:**

| brain | best uniform | best local | drop from the uniform peak to θ 0.2 |
|---|---|---|---|
| quantum | θ 0.02 → **0.777** | (θ_A 0, θ_B 0.02) → **0.779** | 0.061 |
| classical | θ 0.05 → **0.777** | (θ_A 0, θ_B 0.05) → **0.779** | 0.069 |

Frozen chaos 0.3 at the best local pairs: dev 0.7944 against 0.7953 (quantum), and 0.7959 against
0.7973 (classical).

| # | Result |
|---|---|
| E0 the base has its set temperature | MEASURED |
| P1 an inverted U at g 1000, both brains | **MEASURED**: both rise from θ 0, peak, and fall by θ 0.2 (quantum +0.024 / −0.061; classical +0.072 / −0.068) |
| P2 quantum peaks colder and degrades faster | **FAILED (half)**: it peaks colder (0.02 against 0.05), but degrades *slower* (0.061 against 0.069) |
| P3 heat silences unamplified signalling | **FAILED as written**: g 1 is below g 1000 by 0.037–0.050 for quantum, but not ≥ 0.05 at every θ; and classical's g-1 curve peaks at θ 0.01, not 0 |
| P4 local beats uniform by ≥ 0.003 | **FAILED by the bar**: local is better for both brains, but by +0.002 |
| P5 frozen chaos on the temperature field not kept | MEASURED |

**3 of 6 hold.**

### What the misses show (read after the run, not predictions)

1. **Temperature acts on memory, not on the decision.**
   - Both brains' best local pairs put the decision at θ_A 0 (greedy), and the dev rows are nearly
     flat until θ_A 0.2.
   - **The whole inverted U comes from M's temperature:**
     - too cold, and the memory is confidently stale (missed 0.363–0.489);
     - too hot, and it forgets the cue (cued 0.79–0.80);
     - in between, it softens a stale memory without losing a fresh one.
   - This is QB-8/9's finding, now produced by a base temperature rather than a tuned knob.
   - P4's derivation was wrong: I assumed the decision's own temperature mattered (QB-9 had tuned it
     to 0.05). Here it barely does, so one uniform θ set to the memory's optimum is almost as good as
     local control (+0.002).
2. **The quantum memory is the colder one, and it is the more forgiving to heat.**
   - Its optimum is θ 0.02 against classical's 0.05, so the plan's "peaks colder" is right.
   - But the collapse memory re-collapses on every event, which bounds how far heat can drift it. So
     it degrades less from its peak (0.061 against 0.069), and is far less hurt by too much cold
     (−0.024 against −0.072).
   - "Decoherence makes quantum degrade faster" was the classical analogue speaking: the
     QUANTUM-BASE-CHARACTERS rule, broken again by my derivation.
3. **Unamplified signalling under heat:**
   - the channel's noise does not silence the brain, but it turns M's reading into noise, so T runs
     on evidence alone: cued falls to 0.80–0.81, while **missed rises to 0.61**, the best of any
     setting, because a noisy memory cannot be stale;
   - amplification (g 1000) is worth +0.03 to +0.05 at each brain's peak, so it is needed, but not by
     P3's margin.
4. **Both brains reach 0.779** at their best local setting, above QB-9's hand-tuned 0.776 and T5's
   hosted 0.778. **The thermal base's temperature replaces the hand-tuned memory heat and loses
   nothing.**

**Jesse's theory, measured:** a single base temperature changes how well the brain thinks, with an
optimum and both sides worse. Here it works through the memory: cold makes it rigid, heat makes it
forget. The quantum memory wants it colder than the classical one.
