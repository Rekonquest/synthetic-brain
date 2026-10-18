# QB-7: chemistry axioms as amplifiers, in a world where cues go missing

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

Jesse: chemistry axioms "can really come into play here with amplification."

Chemistry's amplifiers differ from QB-6's plain gain:
- cooperative binding gives ultrasensitivity;
- enzyme cascades give multistage gain with deactivation;
- autocatalysis with degradation gives a bistable latch;
- product inhibition acts as a brake.

**The world gives growth a real job.** QB-3's task is unchanged except that **each trial's cue is
missing with probability 0.4**: its cue steps are pure noise. In a missed trial:
- the memory still holds the *previous* target, and pulls the wrong way;
- the target must be found by growing the weak response evidence (p = 0.25) into a decision.

The QB-3 worlds are byte-identical at p_miss = 0, and that is checked.

## Protocol (QB-5's, unchanged)

- One module at a time, judged on its own check.
- The development seeds (0–15) select, and plain wins ties.
- The test seeds (100–131) report on against off.
- The winner is kept only if it beats off on the check on the test seeds.
- Each round starts from the kept best.

**The base is QB-6's kept system:** collapse memory, cold T₀ 0.1, evidence gain ×3, λ 0.05,
η_m 0.05, amplitude chaos α 0.05.

**The checks:**
- **D:** F1. Recall is now the fraction of trials with a target event *before the response midpoint*,
  whether the trial was cued or not.
- **M:** hold fidelity.
- **T:** acc_resp, reported split into cued and missed trials.

## The chemistry amplifiers

1. **D, cooperative binding (Hill).**
   - Activation a = cⁿ / (Kⁿ + cⁿ), where c is the count of the observed option in the last M
     observations and K = 2.5.
   - The event fires *stochastically* with probability a (binding is stochastic). As n → ∞ with M = 4,
     this is the plain gate.
   - Variants: n ∈ {2, 4, 8} × M ∈ {4, 8}.
2. **M, an autocatalytic latch (a chemical rival to collapse).**
   - A concentration b_j per option: db = b²/(K² + b²) − δ·b − γ·b·Σ_{k≠j} b_k, with K = 0.5,
     δ = 0.6, γ = 1, dt = 0.5.
   - It is bistable: off at 0, on at 1.5, with a threshold of 0.167. Mutual inhibition gives
     winner-take-all.
   - An event adds a pulse of 2.0 to b_e.
   - T reads b/Σb as concentrations. The held item is the argmax when it is above the threshold.
3. **T, an enzyme cascade (replacing the plain gain).**
   - Two stages per option: x₁ += 0.5·in·(1 − x₁) − 0.2·x₁, then x₂ += 0.5·x₁·(1 − x₂) − 0.2·x₂.
     `in` is 1 for the observed option.
   - Every option is reinforced by 1 + η·G·x₂.
   - Variants: G ∈ {1, 2, 3}, with the plain observed-only gain removed (the substitution law).
4. **T, growth with product inhibition (the brake).**
   - The factor is 1 + ηG·(1 + κ·p/(p + 0.1))·(1 − p)^ι, on top of the kept gain.
   - The product inhibits its own synthesis.
   - Variants: κ ∈ {0.5, 1} × ι ∈ {1, 2}.
   - κ 1, ι 0 (QB-6's overshooting form) is reported as a reference, not as a variant.

## Predictions

1. **C1, cooperative binding is kept on D.** A longer window with ultrasensitivity catches target
   repeats in the response phase of missed trials (recall up), while a sharp n keeps single noise
   repeats below K (precision held). F1 improves on test.
2. **C2, the latch ties collapse.** Both are bistable commitments, immune to sub-threshold noise.
   M's fidelity is within 0.02 of collapse's, whether or not it is kept.
3. **C3, the cascade is kept and beats the plain gain.** It amplifies persistent signals
   multiplicatively, and its deactivation filters one-off noise: the weak, persistent response
   evidence is exactly its job. Test acc_resp ≥ +0.005 over the round's base.
4. **C4, product inhibition lets growth coexist with gain.** The best κ, ι variant is kept, with test
   acc_resp ≥ +0.003. Without the brake, QB-6 found that growth on top of gain overshot.
5. **C5, law 5:** the classical modular comparator (a register memory and a greedy tracker with
   evidence gain ×3, λ and η_m retuned on dev) has test acc_resp ≥ the final kept system's − 0.01.
6. **C6, the world's premise:** on the base, acc_resp on missed trials is lower than on cued trials by
   at least 0.20.

**Kill criterion:** if 3 or more of C1–C6 fail, the chemistry characters do not predict at the module
level here.

## Build

- `qb-task` gains `p_miss` and a per-trial `missed` flag, drawn from a separate stream. At p_miss = 0,
  QB-3's output must stay byte-identical.
- The runner is `qb7`, built from `qb6`'s.
- **Unit tests:**
  - the base configuration with p_miss = 0 reproduces QB-6's 0.915;
  - the latch writes, holds for 40 steps with no input, and switches on a new pulse;
  - Hill at n = 64, M = 4 matches the plain gate on more than 99% of steps;
  - the cascade's activations stay in [0, 1];
  - in missed trials the cue steps are noise.

## Results (2026-10-18, `evidence/qb-7/results.txt`, 1.4 s)

Tests passed before the run:
- at p_miss = 0, the base reproduces QB-6's 0.915;
- the latch writes, holds for 40 steps, and switches;
- Hill n = 64 matches the plain gate on more than 99% of steps;
- the cascade stays in [0, 1].

With the new world fields, QB-3 to QB-6's outputs are byte-identical.

**The hard world:** the base (QB-6 kept) scores 0.723 on test, **0.915 on cued trials and 0.427 on
missed ones**.

| Round (check) | dev: off → best | test acc_resp (cued / missed) | verdict |
|---|---|---|---|
| 1 D cooperative binding (F1) | 0.8286 → plain wins (Hill n 2–8, window 4/8: 0.542–0.766) | unchanged | not kept |
| 2 M autocatalytic latch (fidelity) | 0.6186 → plain wins (latch 0.6177) | unchanged | not kept (a tie) |
| 3 T enzyme cascade, replacing the gain (acc_resp) | 0.7455 → plain wins (G 1/2/3: 0.637/0.651/0.638) | unchanged | not kept |
| **4 T growth with product inhibition (acc_resp)** | 0.7455 → **0.7719 (κ 1, ι 1)** | **0.723 → 0.749** (cued 0.915 / missed 0.427 → **0.492**) | **KEPT** |

The reference arm, growth κ 1 *without* the brake, scored 0.7430 on dev, **below** the base (0.7455).

**The classical modular comparator** (λ 0.1, η_m 0.05, tuned on dev) scored **0.747** on test (cued
0.868, missed 0.561). The final kept system scored **0.749** (cued 0.915, missed 0.492).

| # | Result | What was measured |
|---|---|---|
| C1 cooperative binding kept on D | **FAILED** | every Hill variant is below the plain gate (best 0.766 against 0.829) |
| C2 the latch ties collapse | MEASURED | fidelity 0.6034 against 0.6049 |
| C3 the cascade beats the plain gain | **FAILED** | every cascade variant is below plain (best 0.651 against 0.746) |
| C4 product inhibition lets growth coexist | MEASURED | kept: +0.026 on test, all of it on missed trials (+0.065) |
| C5 classical ties or wins | MEASURED | 0.747 against 0.749 |
| C6 missed trials are much harder | MEASURED | 0.915 against 0.427 |

**4 of 6 hold; the kill criterion (3) does not fire.**

### The misses, analysed (both derivations, not characters)

**C1:** I made binding *stochastic*: it fires with probability a. Ultrasensitivity's benefit is a
sharp switch. Stochastic firing at c = 2 still fires with probability 2ⁿ/(2.5ⁿ + 2ⁿ) (0.14 at
n = 8), so noise repeats leak through as false events. A deterministic Hill threshold is exactly the
plain gate. So ultrasensitivity has nothing to add to a gate that is already a hard threshold.

**C3:** the cascade amplifies *sustained* input, while deactivation (0.2 per step) forgets between
inputs. The response evidence is *intermittent*: the target appears on about 31% of steps, a gap of
about 3 steps. So each hit decays before the next arrives, and the cascade passes on less than the
direct gain. I treated "weak and persistent" as if it were "weak and intermittent". **The rule: a
cascade's deactivation time has to exceed the input's inter-arrival time.**

### What QB-7 shows

1. **Product inhibition is the chemistry piece that works here,** as the brake. Growth alone on top of
   the gain overshoots (dev 0.7430, below the base). With the product inhibiting its own synthesis,
   growth and gain coexist, and **the gain lands entirely on the missed trials (0.427 → 0.492)**, where
   weak evidence has to be grown into a decision. That was the job it was given.
2. **The chemical latch and quantum collapse are the same memory** (fidelity 0.603 against 0.605):
   two physics, one character, a bistable commitment.
3. **For the first time, quantum and classical tie with different strengths:**
   - the quantum system is better when the memory is right (cued 0.915 against 0.868), from its cold,
     committed readout;
   - the classical system is better when the memory is stale (missed 0.561 against 0.492), because it
     forgets faster (λ 0.1 against 0.05).

   Equal totals, different strengths: a reason to orchestrate them *together*, routing each kind of
   trial to the module that is good at it, rather than choosing one.
4. **Every module's weak spot is the stale memory.** On missed trials, M still holds the previous
   target (fidelity falls to 0.605 overall). The next module the architecture needs is one that
   notices the memory is stale, a surprise or mismatch signal, and releases it.

### Next

- A **mismatch module**: when the tracker's evidence persistently disagrees with the memory, release
  the memory. This is PREDICT/REVISE from Jesse's cognitive roots, as its own small stack.
- **Orchestrating the two trackers together** (quantum cold for committed trials, classical fast-forget
  for uncertain ones), with the routing decided by readings, not by a rule.
- Hill as a *graded* amplifier into T (not a stochastic gate on D), and a cascade with deactivation
  slower than the inter-arrival time. Both remain untested.
