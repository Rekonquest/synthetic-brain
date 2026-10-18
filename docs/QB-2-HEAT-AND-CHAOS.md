# QB-2: hot, cold and chaos on the quantum-base stacks

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

Do temperature and chaos change these stacks, and do they act differently from each other on a quantum
base?

QB-1 gave one rule, and every prediction here follows it: **derive from the character as recorded,
never from the classical analogue.** In particular, collapse forgets as well as commits, and mixing
transports ballistically rather than diffusing.

## The setup

**The task, settings, seeds and measures are QB-1's, unchanged:**
- 8 options and 400 steps; the best option switches 2 → 5 at t = 200.
- Evidence is p = 0.4; η = 0.15, λ = 0.02, mixer φ = π/12.
- 32 seeds; every arm sees the same streams.
- Measures: acc1, acc2, latency, decisiveness, change rate.

**The four new knobs:**
1. **Decision temperature T:** the choice is drawn with p_i ∝ (|a_i|²)^(1/T). T = 1 is Born, T → 0 is
   argmax, and T = 2 gives p ∝ |a| ∝ w, which is the L1 base.
2. **Phase heat σ (dephasing):** every amplitude gets a random phase kick, uniform in [−σ, σ], fresh
   every step, before the mixer. The levels are σ ∈ {0, 0.3, 1.0, π}.
3. **Occupation heat ε:** every amplitude gets ε·e^{iψ} added (ψ random, fresh every step) before the
   hooks, then the state is renormalized. Empty options get a thermal floor. The levels are
   ε ∈ {0.01, 0.05}.
4. **Chaos:** the logistic map x → 4x(1−x), one value per site, seeded from the seed and warmed up 20
   iterations. The site gets the phase kick 2π·x, before the mixer.
   - **Frozen chaos:** x is fixed per site for the whole run (static disorder).
   - **Changing chaos:** x is iterated every step (deterministic, but always moving).

**The transport probe** (the mechanism, measured directly):
- A ring of 256 sites with the same mixer φ = π/12, starting at site 0, for 100 steps.
- No hooks and no readout.
- Measured: the mean squared ring distance m2 = Σ p_j d_j², under no disorder (ballistic), phase heat
  σ = π, frozen chaos, and changing chaos.

## Predictions, each from a character

1. **Q1, squaring is a cold knob** (Born = L1 cooled): at T = 2 the choices equal `l1`'s on at least 30
   of the 32 seeds.
2. **Q2, colder is sharper and better at the target:**
   - acc1 does not increase from one T to the next, over T = 0.25, 0.5, 1, 2, 4 (slack 0.01);
   - acc1 at T = 0.25 is at least 0.97, approaching `greedy`'s 1.000.
3. **Q3, hotter explores more:** the change rate does not decrease from one T to the next (slack 0.01).
4. **Q4, phase is inert without a mixer** (QB-1's P4a, now for noise):
   - `born` with phase heat, at each σ, makes the same choices as `born` on 32 of 32 seeds;
   - so does `born` with frozen chaos.
5. **Q5, phase heat turns ballistic transport into diffusion, so it rescues the mixer:**
   - `born-mix` at σ = π has acc1 ≥ `born-mix` (σ = 0) acc1 + 0.15, and latency ≤ `born-mix` latency
     − 40;
   - acc1 does not decrease over σ = 0, 0.3, 1, π (slack 0.02).
6. **Q6, heat cannot restore the memory that collapse wipes** (collapse forgets):
   `born-collapse-mix` at σ = π stays at chance, with acc1 in [0.05, 0.25].
7. **Q7, phase heat cannot unlock a collapse, but occupation heat can:**
   - **Q7a:** `born-collapse` at σ = π is pinned on 32 of 32 seeds. A phase kick leaves a zero amplitude
     at zero.
   - **Q7b:** with occupation heat, the change rate is above 0 at ε = 0.01 and larger at ε = 0.05, while
     acc1 stays at chance ([0.05, 0.25]) at both levels. It unlocks, but collapse still forgets, so the
     exploring has no memory.
8. **Q8, frozen chaos traps transport** (static disorder localizes the walk): `born-mix` with frozen
   chaos has acc1 ≥ `born-mix` acc1 + 0.15, and latency ≤ `born-mix` latency − 40.
9. **Q9, changing chaos acts like heat:** acc1 of `born-mix` with changing chaos is within 0.10 of
   `born-mix` at σ = π.
10. **Q10, the probe separates the two mechanisms:**
    - m2 under frozen chaos < 0.25 × m2 under phase heat (localized against diffusive);
    - m2 under phase heat < ballistic m2;
    - m2 under changing chaos is within a factor of 2 of phase heat.

**The physics behind Q8–Q10:** in a quantum walk, static disorder gives Anderson localization, while
noise that changes over time destroys the interference and gives a classical random walk. This is
standard; Schreiber et al., PRL 2011, studied decoherence and disorder in quantum walks. Q8 is the risk
in this test: the task's ring has only 8 sites, and the localization length on it is unknown.

## Kill criterion

If 4 or more of Q1–Q10 fail, the corrected derivation rule is not yet predictive. Each miss gets the
same analysis as in QB-1: was the character wrong, or the derivation from it?

## Build

New mechanisms, additive only, so QB-1's code paths are untouched:
- `qb-classical::temper(weights, T)`;
- `qb-amplitude::State::excite(k, c)`, which adds an amplitude;
- a new crate `qb-chaos` for the logistic map;
- the runner `qb2`.

**Unit tests before the run:**
- `temper` at T = 1 is the identity, and at T = 2 gives √ of the Born weights;
- a phase kick keeps the magnitudes;
- `excite` lifts a zero amplitude;
- the logistic map stays in (0, 1).

## Results (2026-10-18, `evidence/qb-2/results.txt`)

All tests passed before the run (12 in the workspace), and QB-1 still reproduces its 6 of 8 with the
new mechanisms added.

| Arm | acc1 | acc2 | latency | decisiveness | change rate |
|---|---|---|---|---|---|
| `born` (T = 1) | 0.927 | 0.937 | 38.9 | 0.843 | 0.247 |
| `l1` | 0.610 | 0.621 | 46.5 | 0.542 | 0.653 |
| `greedy` | 1.000 | 1.000 | 38.2 | — | 0.010 |
| `born-T0.25` | **1.000** | **1.000** | **37.8** | 0.978 | 0.031 |
| `born-T0.5` | 0.998 | 0.998 | 37.2 | 0.952 | 0.067 |
| `born-T2` | 0.610 | 0.621 | 46.5 | 0.542 | 0.653 |
| `born-T4` | 0.324 | 0.327 | 69.6 | 0.298 | 0.838 |
| `born` + phase heat (each σ), and `born` + frozen chaos | 0.927 | 0.937 | 38.9 | 0.843 | 0.247 |
| `mix`, heat σ = 0 / 0.3 / 1 / π | 0.149 / 0.181 / 0.173 / 0.273 | | 142.1 / 106.2 / 112.4 / 57.9 | | |
| `mix-frozen` (frozen chaos) | **0.671** | **0.752** | **51.6** | 0.666 | 0.476 |
| `mix-changing` (changing chaos) | 0.234 | 0.216 | 69.4 | 0.359 | 0.783 |
| `collapse-mix`, heat π | 0.129 | 0.146 | 66.2 | 0.862 | 0.140 |
| `collapse`, heat π | 0.156 | 0.219 | 156.2 | 0.998 | 0.000 |
| `collapse`, occupation 0.01 / 0.05 | 0.100 / 0.117 | | | | 0.001 / 0.020 |

**The transport probe** (a ring of 256 sites, 100 steps, mean squared distance):

| Condition | m2 |
|---|---|
| ballistic | 1363.0 |
| phase heat | 13.6 |
| changing chaos | 18.3 |
| frozen chaos | **1.5** |

| # | Result | What was measured |
|---|---|---|
| Q1 squaring is a cold knob | MEASURED | T = 2 makes exactly `l1`'s choices on 32 of 32 seeds |
| Q2 colder is sharper | MEASURED | acc1 at T = 0.25 / 0.5 / 1 / 2 / 4: 1.000 / 0.998 / 0.927 / 0.610 / 0.324 |
| Q3 hotter explores more | MEASURED | change rate: 0.031 / 0.067 / 0.247 / 0.653 / 0.838 |
| Q4 phase noise is inert without a mixer | MEASURED | identical to `born` on 32/32 at every heat level, and with frozen chaos |
| Q5 heat rescues the mixer | **FAILED on size** | latency 142.1 → 57.9 (bar ≤ 102.1) and monotone, but acc1 rose only 0.149 → 0.273 (bar ≥ 0.299) |
| Q6 heat cannot restore wiped memory | MEASURED | `collapse-mix` with heat: acc1 0.129, at chance |
| Q7 phase heat cannot unlock a collapse; occupation heat can, without memory | MEASURED | pinned 32/32 under phase heat; occupation heat gives change rate 0.001 → 0.020 with acc1 at chance (0.100, 0.117) |
| Q8 frozen chaos traps transport | MEASURED | acc1 0.149 → **0.671**, latency 142.1 → 51.6 |
| Q9 changing chaos acts like heat | MEASURED | acc1 0.234 against heat's 0.273 |
| Q10 the probe separates the mechanisms | MEASURED | frozen 1.5 < 0.25 × heat's 13.6; heat 13.6 < ballistic 1363; changing 18.3 within 2× of heat |

**10 of 11 checks hold; 1 of 10 predictions fails (Q5); the kill criterion (4) does not fire.**

### The miss, analysed

Q5's direction and mechanism held. The probe shows heat turning ballistic transport (m2 1363) into
diffusion (13.6), and the mixer's latency fell 84 steps. Only the size of the accuracy rescue was
overestimated. I predicted from "heat → diffusion" without working out how far diffusion still smears
the evidence on an 8-site ring within DECAY's forgetting time (1/λ = 50 steps). That is a
quantitative derivation gap, not a wrong character. The next rule to add: **when a character changes
transport, estimate the transport length against the system's size and its memory time before
setting a bar.**

### What QB-2 found

1. **Squaring is temperature.** Born is exactly the L1 base cooled to T = ½ (32/32 identical at
   T = 2). Cooling further gives the best tracker in either test: T = 0.25 reaches 1.000 and the
   shortest latency, 37.8. So "the quantum base sharpens decisions" is the same thing as "it runs
   cold".
2. **On a quantum base, heat and changing chaos are the same tool**: both destroy interference and
   turn transport into diffusion (probe 13.6 and 18.3; acc1 0.273 and 0.234). On these stacks,
   Jesse's sense that hot/cold and chaos are siblings is exact for *changing* chaos.
3. **Frozen chaos is a different tool, one heat cannot be:** static disorder traps the walk
   (Anderson localization). In the probe, the spread is 1.5 against heat's 13.6 and ballistic's 1363.
   In the task, it took the mixing stack from chance to 0.671 (0.752 after the switch). **Chaos used
   to hold a state in place**, a new use of chaos on a quantum base. It is consistent with QB-1's
   surprise (a phase at the evidence site partly re-localized the state) and probably explains it.
4. **Collapse is robust to phase heat:** a zero amplitude stays zero. Only occupation heat, which
   puts energy into empty modes, unlocks it. Because collapse forgets, the unlocked stack explores
   with no memory.

### The three labels for the new arms

| Arm | Load-bearing | Signature | Fit |
|---|---|---|---|
| decision temperature | yes | sharpness dial: cold commits, hot explores | cold for exploiting, hot for exploring |
| phase heat on a mixer | yes | ballistic → diffusive | slows the spread of a signal |
| frozen chaos on a mixer | yes (strongest effect here) | localizes: holds a state against transport | **working memory: holding a state** |
| changing chaos | yes | heat-like | same as heat |
| occupation heat on collapse | yes, weakly | thermal unpinning, memoryless | escaping a stuck decision |

### Next (not yet precommitted)

- **Frozen chaos as memory:** how long does a localized state hold, against disorder strength and ring
  size? Is it a usable short-term memory?
- **The temperature schedule:** annealing (hot early, cold late) on the switching task, a direct test
  of hot/cold as Jesse uses it.
