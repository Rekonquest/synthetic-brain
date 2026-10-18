# QB-5: chaos and hot/cold, one module at a time (Jesse's edge-of-chaos protocol)

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

Do chaos and temperature improve QB-4's modules, each judged on its own job?

This follows Jesse's protocol:
- one unit at a time, with its own validated check;
- off (the plain unit) is the bar, and plain wins ties;
- the check selects the variant, and the winner is kept before moving on;
- the runs are short;
- on a quantum base, temperature is the knob, and classical chaos does not go first.

The task, worlds and seeds are QB-4's (`qb-task`). **The base is QB-4's `modular-quantum`** (λ = 0.05,
η_m = 0.05, cold T₀ = 0.25; 0.877 on the test seeds).

## The modules' own checks

Computed from the development worlds' known targets, the way a function's visible tests are used.
Each is checked twice: computed from the choices, and recomputed independently in a unit test.

| Module | Its own check |
|---|---|
| **D, detector** | event F1. Precision is the fraction of events whose option is the current trial's target. Recall is the fraction of trials with at least one target event during the cue |
| **M, memory** | hold fidelity: the fraction of delay and response steps where M's held item (argmax of its Born weights, when it is not uniform) is the target |
| **T, tracker** | acc_resp, the system's choice accuracy over the response steps |

## Chaos and temperature, per module

The source is the logistic map at r = 4, keyed per (seed, step, element), with 16 iterations from a
hashed start. Its invariant mean is ½. Each perturbation is mean-preserving, as 1 + α(x − ½).

- **D:** chaos on the vote weights *before* the gate's threshold (the filter position). Each
  observation in the window counts 1 + α(x − ½), and the gate fires at a weighted count ≥ k − ½. At
  α = 0 this is exactly the plain gate. Variants: α ∈ {0.1, 0.2, 0.4} × 3 chaos keys.
- **M:**
  - phase chaos on the memory state, each step: a rotation of α·2π(x − ½) per element, α ∈
    {0.1, 0.5, 1.0};
  - occupation heat: ε·e^{iψ} added to every element, each step, ε ∈ {0.005, 0.01, 0.02}.
- **T, in three rounds, each started from the kept best:**
  1. **Cold and hot (the knob):** T₀ ∈ {0.1, 0.15, 0.2, 0.3, 0.4}.
  2. **A governor:** reheat on D's event and cool after it.
     T_t = T₀ + (T_hot − T₀)·e^{−(t − t_event)/τ}, with T_hot ∈ {0.5, 1.0} and τ ∈ {3, 10}.
  3. **Amplitude chaos:** each amplitude is scaled by 1 + α(x − ½), per element, each step, before
     the readout. α ∈ {0.05, 0.1, 0.2} × 3 chaos keys.

**The rule each round:** the best variant on the development seeds by the module's check replaces the
current base only if it is strictly better. The test seeds then report on against off, on the module's
check and on acc_resp. Chaos is **kept** only if on beats off on the module's check on the test seeds.

## Predictions, from the characters

1. **P1, D: chaos before the gate is neutral on iid noise.** A decorrelator has nothing to destroy
   when the noise is already independent (STACK-CATALOG, chemistry positions: chaos on an iid stream
   is neutral). D's chaos is not kept.
2. **P2, M: a collapsed memory is immune to phase chaos, and heat only harms it.**
   - Phase chaos on a one-hot state changes no Born weight (QB-2 Q4, Q7a): every phase variant ties
     plain exactly.
   - Occupation heat unpins the memory (QB-2 Q7b) and lowers hold fidelity.
   - M stays plain.
3. **P3, T: colder helps up to a point.** The best T₀ on the development seeds is ≤ 0.25 (the cold
   side), and on the test seeds the kept T is at or above off on acc_resp.
4. **P4, T: the governor adds nothing.** The architecture already gave switching to D and M, so
   reheating T on events does not beat the best fixed temperature by more than 0.01 acc_resp on the
   test seeds.
5. **P5, T: amplitude chaos is not kept.** It feeds a convex readout (p ∝ |a|^(2/T)), where chaos
   amplifies rather than averages (catalog rule 35), and on quantum stacks temperature is the knob. No
   amplitude-chaos variant beats the kept T strictly on the development seeds, or it loses on the test
   seeds.

**The overall prediction:** temperature (hot/cold) is the only dial that matters here, and classical
chaos is kept on no module.

**Kill criterion:** if 3 or more of P1–P5 fail, the characters do not predict chaos and temperature
at the module level.

## Build

- `qb-chaos::keyed_unit(key)`: a hashed start, then 16 logistic iterations.
- The runner `qb5` reuses QB-4's modules unchanged at every α = 0.
- **Unit tests:**
  - at α = 0, ε = 0, fixed T₀ and no governor, `qb5` reproduces QB-4's `modular-quantum` choices
    exactly;
  - the weighted gate at α = 0 equals the plain gate;
  - `keyed_unit`'s mean over 10⁵ keys is within 0.01 of ½;
  - each module check is recomputed independently.

## Results (2026-10-18, `evidence/qb-5/results.txt`, 1.25 s)

Tests passed before the run:
- the plain configuration reproduces QB-4's 0.877;
- the checks recompute independently;
- the weighted gate at α = 0 equals the plain gate;
- `keyed_unit`'s mean is within 0.01 of ½.

| Round (module check) | dev: off → best | test check: off → on | test acc_resp | verdict |
|---|---|---|---|---|
| D: chaos before the gate (F1) | 0.9585 → 0.9588 (α 0.4) | 0.9526 → 0.9522 | 0.877 → 0.877 | not kept |
| M: phase chaos and heat (fidelity) | 0.8857 → 0.8857 | 0.8682 → 0.8682 | 0.877 | plain wins |
| **T1: cold and hot (acc_resp)** | 0.8966 → **0.9073 (T₀ 0.1)** | **0.877 → 0.889** | **0.889** | **KEPT** |
| T2: the governor (acc_resp) | 0.9073 → 0.9073 | 0.889 → 0.889 | 0.889 | plain wins |
| T3: amplitude chaos (acc_resp) | 0.9073 → 0.9078 (α 0.05, key 2) | 0.8887 → 0.8890 | 0.889 | kept by the rule (see below) |

**Dev values per variant:**
- D: every α ≤ 0.2 variant ties plain exactly (0.9585); α 0.4 gives 0.9578–0.9588.
- M: all six variants tie plain exactly (0.8857).
- T₀ = 0.1 / 0.15 / 0.2 / 0.3 / 0.4: 0.9073 / 0.9068 / 0.9035 / 0.8807 / 0.8254.
- Governor variants: 0.8973 / 0.8863 / 0.8524 / 0.8029, all below plain.
- Amplitude chaos: α 0.05 and 0.1 give 0.9059–0.9078 (around plain); α 0.2 gives 0.8838–0.8871.

| # | Result | What was measured |
|---|---|---|
| P1 D's chaos is neutral, not kept | MEASURED | at α ≤ 0.2 the weighted votes can never cross k − ½, so it is a dead stage; α 0.4 moved a few events, and lost on test |
| P2 M is immune to phase chaos and harmed by heat | **FAILED** | phase chaos tied exactly (immune: held). Heat **also tied exactly**; it did not lower fidelity |
| P3 colder T helps up to a point | MEASURED | best T₀ 0.1 (≤ 0.25); test 0.877 → 0.889 |
| P4 the governor adds nothing | MEASURED | every reheat variant was below plain on dev; test unchanged |
| P5 amplitude chaos not kept | **FAILED** | kept by the strict rule: +0.0005 dev, +0.0003 test |

**3 of 5 hold; the kill criterion (3) does not fire.**

### The misses, analysed

**P2, a derivation miss: I ignored how M is read.** QB-2's "occupation heat unlocks a collapse" was
measured on a state that is *sampled* each step. M is never sampled: the check reads its argmax, and T
reads its Born weights as small concentrations (η_m = 0.05). A thermal floor of ε ≤ 0.02 never
changes the argmax, and barely changes the concentrations, so fidelity ties exactly. The character
(heat unpins) needs the readout it was measured through. **Rule added: derive from the character
*and the readout it was measured through*.**

**P5, kept by the rule, but the effect is at the size of selection luck.** The best of 9 variants
gained 0.0005 on dev, and 0.0003 on test (about 10 of 32,000 response steps). That is the size a
best-of-9 pick produces by chance, so it is **kept by Jesse's strict rule and recorded as practically
neutral**. α 0.2 is the "too much" edge: −0.02 on dev for all three keys.

### What QB-5 shows

1. **Temperature is the knob here, as Jesse said for quantum stacks.** The one real gain is cooling the
   tracker: T₀ 0.25 → 0.1, test 0.877 → **0.889**. That ties the classical modular system (0.887).
   T₀ 0.1 is the edge of the grid, so colder may be better still.
2. **Classical chaos did nothing useful on any module:**
   - on D it is either below the gate's resolution (a dead stage) or noise;
   - on M it is invisible (the collapse is immune to phase);
   - on T it is neutral at small α and harmful at α 0.2.

   That matches the skill's rule for quantum bases ("classical chaos does not go first; temperature is
   the knob") and the catalog's record of chaos rarely being kept on a quantum base.
3. **Reheating on events hurts.** The architecture already gives switching to D and M, so heating T
   only adds wrong choices. Hot/cold belongs where the job needs exploring, and here no module's job
   does.
4. **Each module is near its own optimum:**
   - D: F1 0.953;
   - M: fidelity 0.868, which is bounded by D's missed cues and false events, not by M;
   - T: 0.889 at T₀ 0.1.

   The next gains are in D's recall (cues it misses) and in a harder task, not in more chaos on these.

### Next

- Extend T₀ below 0.1, and η_m below 0.05 (both kept values sit on their grid edges).
- D is the bottleneck for M. Its own optimization would be the gate's k and M, judged on D's F1.
- A task whose modules *need* exploring (hot): one where the cue is sometimes missing, so the system
  must search. That would let hot/cold do a real job.
