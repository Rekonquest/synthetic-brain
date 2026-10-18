# QB-3: a complex stack on a working-memory task (vault Rules 4–7)

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

A task needs several behaviours at once: forget the old target, hold the new one through a noisy
delay, then commit to it on weak evidence. On that task, does a complex quantum-reinforced stack
compete with the best classical design?

The test also asks whether the characters still predict each piece's effect inside a large
composition.

Vault Rule 7 sets the terms: the principal candidate is complex, and simple stacks are controls. Rule 6
requires every domain. Rule 5 requires two chaos mechanisms at several positions. Rule 2 asks for one
to three precommitted orders, with every candidate and control executed. Rule 4 requires the base
controls.

## The task (new)

**Delayed cued tracking**, run as a series of trials:
- N = 16 options on a ring; 40 trials per run, 50 steps each (2,000 steps).
- Each trial has a target c, drawn at random at ring distance ≥ 3 from the last trial's target.
- **The three phases of a trial, and the evidence in each** (one option observed per step):
  1. **Cue,** steps 0–4: the target with p = 0.9, otherwise uniform.
  2. **Delay,** steps 5–24: uniform noise only.
  3. **Response,** steps 25–49: the target with p = 0.25, otherwise uniform.
- The stack chooses an option every step.

**Measures:**
- **acc_resp** (the frozen main metric): accuracy over every response step.
- **acc_early:** accuracy over the first 5 response steps, i.e. whether the memory survived the delay.
- **persev:** the fraction of response steps that choose the *previous* trial's target (trial 1
  excluded).

**Seeds:** development seeds 0–15, used only for tuning λ. Test seeds 100–131 (32), for every
comparison.

## The complex candidate (order A), per step

| # | Domain | Mechanism | Setting |
|---|---|---|---|
| 1 | physics / chaos 1 | frozen logistic disorder: a phase 2π·x_j per site, fixed for the run | QB-2's construction |
| 2 | quantum | ring mixer (unitary transport) | φ = π/12 |
| 3 | information theory | persistence gate: the observed option counts only if it appears ≥ 2 times in the last 3 observations (a k-of-M conjunction) | k = 2, M = 3 |
| 4 | chemistry | Michaelis–Menten catalysed reinforcement: factor 1 + η·(1 + κ·p_o / (p_o + K)), with p_o the observed option's current Born weight used as a concentration | κ = 1, K = 0.1 |
| 5 | cognition | REINFORCE (the factor above, only when the gate passes), DECAY, CONSERVE | η = 0.15; λ tuned |
| 6 | mechanics | inertia: a ← normalize((1 − μ)·a + μ·a_prev), a complex combination | μ = 0.5 |
| 7 | quantum / physics / chaos 2 | cold Born readout with a chaotic temperature: p ∝ (Born)^(1/T_t), T_t = T₀·(0.5 + y_t), y a tent map of slope 1.99, iterated each step | T₀ = 0.25 |

- **Order B** is the same pieces with the mechanics (6) moved before the hooks (3–5), and the mixer
  plus frozen disorder (1–2) moved after them.
- **There is no collapse:** its character is perseveration (QB-1), which this task penalizes.
- The tent slope is 1.99, not 2, because the exact tent map collapses to 0 in floating point (TA1
  recorded the same failure).

## Controls

- **The classical champion:** greedy argmax over plain weights with REINFORCE, DECAY and CONSERVE. The
  persistence gate is on or off, and λ ∈ {0, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2}. The best of the 14 on
  the development seeds by acc_resp is the champion.
- **The candidates' λ** is tuned on the same grid and the same development seeds; ablations keep the
  candidate's chosen λ.
- **Simple stacks:**
  - QB-1's `born` (T = 1, λ = 0.02, no gate);
  - a cold `born` (T = 0.25, λ tuned).
- **Rule 4 base controls:**
  - **base-only:** no hooks;
  - **replacement base:** real weights, with a classical diffusive mixer at the same rate (sin²φ per
    pass), and the same gate, chemistry, inertia and chaotic temperature. Frozen phase disorder cannot
    exist there, so it is dropped.
- **Ablations of candidate A**, one piece at a time:
  - −frozen chaos;
  - −mixer (the mixer and the frozen chaos together; frozen phase is a dead stage without a mixer,
    QB-2 Q4);
  - −gate;
  - −chemistry (plain REINFORCE);
  - −inertia;
  - −chaotic temperature (fixed T₀);
  - warm (T₀ = 1).

## Predictions, each from a character or a law

1. **R1, law 5 (a classical method ties or wins at the target):** the classical champion's acc_resp is
   at least that of candidate A and candidate B, on the test seeds.
2. **R2, the gate suppresses iid noise as a conjunction (persistence filter):**
   - the classical champion selected on the development seeds has the gate on;
   - candidate −gate has acc_resp lower than A's by at least 0.03.
3. **R3, mixing transports evidence away (QB-1, QB-2):** −mixer raises acc_resp over A by at least 0.02.
4. **R4, frozen chaos localizes against the mixer (QB-2):** −frozen chaos lowers acc_resp under A by at
   least 0.05.
5. **R5, chaos on a convex quantity is not neutral, and a chaotic temperature blurs a cold decision
   (catalog rule 35, QB-2):** −chaotic temperature has acc_resp ≥ A's.
6. **R6, catalysed reinforcement is rich-get-richer, a commit character:** −chemistry has persev ≤ A's.
7. **R7, inertia holds the old state (mechanics):** −inertia has persev ≤ A's.
8. **R8, cold commits (QB-2):** warm has acc_resp lower than A's by at least 0.05.
9. **R9, the base alone gives no verdict (law 1):** base-only acc_resp is in [0.03, 0.10] (chance is
   0.0625).
10. **R10, the task's premise (a hold-versus-forget trade-off):** along the classical λ grid (gate on,
    development seeds), acc_early does not increase and persev does not increase as λ rises (slack
    0.02 each). Faster forgetting loses the memory, and holding the memory perseverates.

**Order:** |acc_resp(A) − acc_resp(B)| is reported, with no prediction.

**Kill criterion:** if 4 or more of R1–R10 fail, character-based derivation does not hold at this
complexity.

## Honest expectation, stated before the run

By law 5, I expect the classical champion to win the main metric. If so, the candidate's value has to
show in its pieces' signatures (R2–R8), and in which pieces are antagonistic. Rule 7 keeps those as the
evidence that shapes the next candidate.

## Build

- Mechanisms are added, not changed:
  - `qb-classical::{diffuse, gate}`;
  - `qb-amplitude::State::blend` for inertia;
  - `qb-chaos::tent`.
- The runner is `qb3`.
- **Unit tests before the run:**
  - the gate counts correctly;
  - `blend` with μ = 0 is the identity;
  - the tent map stays in (0, 1) without collapsing to 0 over 10,000 steps;
  - every target is at ring distance ≥ 3 from the one before;
  - the development and test seeds do not overlap.

## Results (2026-10-18, `evidence/qb-3/results.txt`; diagnostics in `results-with-diagnostics.txt`)

The tests passed before the run. The run takes 0.75 s.

**The classical sweep** (development seeds; acc_resp / acc_early / persev):

| λ | gate off | gate on |
|---|---|---|
| 0 | 0.151 / 0.109 / 0.188 | 0.174 / 0.140 / 0.186 |
| 0.005 | 0.355 / 0.178 / 0.412 | 0.379 / 0.225 / 0.406 |
| 0.01 | 0.531 / 0.238 / 0.390 | 0.557 / 0.339 / 0.388 |
| 0.02 | 0.729 / 0.414 / 0.230 | 0.749 / 0.556 / 0.234 |
| **0.05** | 0.796 / 0.540 / 0.051 | **0.804 / 0.709 / 0.060** |
| 0.1 | 0.684 / 0.393 / 0.029 | 0.642 / 0.424 / 0.036 |
| 0.2 | 0.538 / 0.343 / 0.035 | 0.560 / 0.314 / 0.038 |

**Test seeds:**

| Arm | λ | acc_resp | acc_early | persev |
|---|---|---|---|---|
| **classical champion** (gate, λ 0.05) | 0.05 | **0.815** | **0.716** | 0.049 |
| candidate A | 0.05 | 0.112 | 0.101 | 0.079 |
| candidate B | 0.02 | 0.234 | 0.155 | 0.196 |
| simple cold `born` (λ tuned) | 0.02 | 0.640 | 0.356 | 0.209 |
| simple `born` (QB-1) | 0.02 | 0.209 | 0.137 | 0.115 |
| base-only | — | 0.063 | 0.060 | 0.063 |
| replacement base | 0.05 | 0.114 | 0.085 | 0.063 |
| A −frozen | 0.05 | 0.103 | 0.074 | 0.062 |
| A −mixer | 0.05 | 0.402 | 0.266 | 0.137 |
| A −gate | 0.05 | 0.139 | 0.119 | 0.093 |
| A −chemistry | 0.05 | 0.089 | 0.086 | 0.070 |
| A −inertia | 0.05 | 0.295 | 0.204 | 0.076 |
| A −chaotic temperature | 0.05 | 0.113 | 0.099 | 0.079 |
| A warm (T₀ = 1) | 0.05 | 0.083 | 0.079 | 0.068 |

| # | Result | What was measured |
|---|---|---|
| R1 classical ties or wins | MEASURED | champion 0.815 against A 0.112 and B 0.234 |
| R2 the gate suppresses noise | **FAILED** | the champion has the gate on, but A −gate (0.139) beat A (0.112) |
| R3 mixing transports evidence away | MEASURED | −mixer 0.402 against A 0.112 (+0.290) |
| R4 frozen chaos localizes | **FAILED** | A 0.112 against −frozen 0.103 (+0.009; bar +0.05) |
| R5 chaotic temperature blurs | MEASURED (at the floor) | 0.113 against 0.112 |
| R6 catalysis is rich-get-richer | MEASURED | persev 0.070 without chemistry, 0.079 with |
| R7 inertia holds the old state | MEASURED | persev 0.076 without inertia, 0.079 with |
| R8 cold commits | **FAILED** | A 0.112 against warm 0.083 (+0.029; bar +0.05) |
| R9 the base alone gives no verdict | MEASURED | 0.063, which is chance |
| R10 the hold/forget trade-off exists | **FAILED** | acc_early and persev are not monotone in λ |

**6 of 10 hold. The kill criterion (4 fail) FIRES:** at this complexity, the character-based
predictions for the individual pieces did not hold as precommitted. That stands. What follows is the
analysis, and none of it relabels a failure.

### Why: the candidate sat at the floor, which voids most of its comparisons

Candidate A scored 0.112 against chance 0.0625. **A comparison against a stack at the floor cannot
show a piece's effect** (STACK-CATALOG rule 12: an observable must move in the baseline). R2, R5 and R8
compared pieces against a stack that was not working. Their failures are real as precommitted, but
uninformative about the pieces themselves.

### Two antagonisms caused the floor

These are diagnostics added after the run, labelled as such and not predictions. The precommitted
output is byte-identical with them added.

| Diagnostic arm | acc_resp | acc_early | persev |
|---|---|---|---|
| A −inertia (from the table) | 0.295 | 0.204 | 0.076 |
| A −inertia −frozen | 0.124 | 0.093 | 0.048 |
| A −mixer (inertia kept) | 0.402 | 0.266 | 0.137 |
| A −mixer, frozen kept (inertia kept) | 0.190 | 0.162 | 0.114 |
| A −mixer −inertia | **0.491** | 0.334 | 0.067 |
| A −mixer −inertia −chemistry | 0.363 | 0.238 | 0.066 |

1. **Mixing transports the memory away** (character 3, known). It is the largest single loss: R3,
   +0.290.
2. **New: complex inertia is mixing in time, and it fights phase disorder.**
   - Inertia blends the state with the previous step's state, as complex amplitudes. Frozen chaos
     rotates every phase by 2π·x_j between steps, so the blend adds a state to a rotated copy of itself
     and cancels it.
   - Evidence: with the mixer present, removing inertia lets frozen chaos work again (0.295 against
     0.124 without frozen, **+0.171**). R4's character was not wrong; inertia masked it.
   - Evidence: with no mixer, frozen phase should be inert (QB-2 Q4), but with inertia it costs 0.212
     (0.190 against 0.402). **Inertia makes phase load-bearing**, just as a mixer does.
3. With both removed, the stack reaches 0.491, and **chemistry is load-bearing there (+0.128)**,
   consistent with the record's one clean benefit: Born weights consumed as concentrations.

### R10: the task's premise was wrong

The forgetting rate has an *interior optimum* (λ = 0.05), not a monotone trade-off. Below it, old
targets interfere (λ = 0 is stuck at 0.17). Above it, the memory fades. Forgetting helps holding up to
a point, because what it removes is mostly stale evidence. So the trade-off exists only above the
optimum.

### What QB-3 changes

- **New character for the sheet:** complex inertia (a blend with the previous state) is temporal
  mixing. It makes phase load-bearing even with no mixer, and it interferes destructively with
  per-step phase disorder. A complex blend is interference, not smoothing.
- **New rule for Rule-7 composition:** pieces that each carry or rotate phase (mixer, frozen phase
  disorder, complex inertia) **interact pairwise through interference**. Their effects do not add. A
  complex candidate must check its phase-carrying pairs before the full composition, or it lands in a
  broken regime where no ablation can speak.
- **The scoreboard:** classical champion 0.815; best quantum sub-stack 0.491 (a diagnostic, with λ not
  retuned); simple cold `born` 0.640. Law 5 holds again, and strongly.

### Next (QB-3b, to be precommitted)

The next candidate is shaped by these antagonisms (Rule 7: preserved evidence shapes the next
candidate):
- inertia as magnitude-only (no phase), or none;
- the mixer only if paired with frozen disorder that works without complex inertia;
- the gate, chemistry and cold readout kept;
- λ tuned per configuration;
- every ablation run from a stack that is off the floor.
