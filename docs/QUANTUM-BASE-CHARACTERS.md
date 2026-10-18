# Quantum-base characters: what each base axiom does to the stack above it

Status: OBSERVATION (2026-10-18). This compiles what is already on record. No new experiment was run
for it, and nothing here is promoted.

## Why this exists

Jesse's quantum-reinforced stacks (vault Rule 4) put quantum-derived mathematics at the base of a
classical stack. **This is not quantum computing: everything runs on a classical PC.**

Every outcome on that base has differed from the one he aimed at, and an earlier model labelled those
runs failures. That label is wrong. In almost every case the base was **load-bearing**: removing or
replacing it changed the behaviour. It was just load-bearing for a different action than the one
aimed at.

The working idea:
- **The outcome is set by the base's character, not by the goal.**
- Catalogue each base axiom's character from the record, and new stacks can be predicted from those
  characters.
- Then they can be steered: put a base where its character is the behaviour wanted.

Every run gets **three labels instead of pass/fail:**
1. **Load-bearing:** does the ablation or replacement change the behaviour?
2. **Signature:** what did the base make the stack *do*?
3. **Fit:** does that suit this target?

## Sources

All paths are under `~/Desktop/Projects/` unless stated otherwise.

| Source | What it holds |
|---|---|
| `atom-quantum-stack` | The five postulates as inert modules, plus a measuring engine. E1 interference, E2 walk, E3 CHSH, E5 lightcone, E6 teleport, E13 resources |
| `Quantum-hardened-atom-stacks` | `atom-wave` cache stacks, and the security-burst Python forecast (its Rust gates are UNRUN) |
| `hybrid-radio-waves-main/atom-quantum-signal-stack` | A Walsh/Born signal base: v1, and Correction 1, which is canonical |
| `hybrid-radio-waves-main/atom-no-copy-stack` | v0 and Corrections 1–5 |
| `hybrid-radio-waves-main/atom-hybrid-lora-stack` | Dual-receiver trial 1 |
| `quantum-network` | `docs/STACK-CATALOG.md` (20 stacks), `QUANTUM-PRIMITIVE-THREAD.md`, `PERSISTENCE-FILTER.md`, 24 experiments (all re-run on 2026-10-18), 19/19 crate tests |
| `a-rag-atom/atom-cognitive-engine` | `weave.rs`, `interference_field.rs`. BUILD-1/1b/2/3/34/35, and E1R to E1R-4 |
| `hybrid-radio-waves-main/universum-substrate` | Thermal antenna TA1, drive D1, SDR-1, CAP1, quantum receiver Q1/Q2, parametric gain P1, C1 |
| Vault: `Atom_Architecture_Rules/` | Rules 4 to 7 and the primitive-centric framing |

## The characters

### 1. Born sampling and destructive projection: *commit, and stay committed*

- **What it does:** it picks one outcome and zeroes the rest, and the choice persists. A read state is
  definite: 1,000 later reads all equal the first.
- **To the axioms above:**
  - **It turns a stock into a quota.** Every further read needs a re-preparation, which is Landauer's
    erasure cost rediscovered: 319,493 re-preparations in `qm_coupled_stack` G3.
  - **It gives any threshold a computable false-positive floor** (negative binomial).
  - **It creates the need for corroboration.**
- **With no mixer to undo it, the stack pins its contents.** The cache's no-mixers arm scored
  7,542 / 2,184 / 2,488 hits on hot / scan / shifting, against intact 6,527 / 343 / 9,547. It froze the
  other 7 pages.
- **Interleaved with a unitary walk, it freezes transport** (Zeno-like). Spread falls to 0.49% of the
  unmeasured spread (E5, the veil dial).
- **A joint projection ties coupled branches to one fate** (the dual cache).
- **In the cognitive engine:**
  - Destructive readout is **forgetting**: phantom mass 0.0 against 4.8e9 without it (BUILD-1b).
  - It is also what lets the derivation loop chain: support 10 against 1 (BUILD-2, BUILD-34).
  - Born definiteness **freezes revision**: supersessions 0 against 68 (E1R-3). It turns M readings
    into one effective sample, which defeats statistical floors unless the erasure is paid. The
    noise-naming rate falls to about 0.497 under Never/PerWindow (E1R-4).
- **Load-bearing:** yes, everywhere it was ablated.

### 2. Born squaring as evidence: *sharpen into peaks*

- **What it does:** squaring amplitudes into weights concentrates evidence into peaks.
- **Signal Correction 1:** best range was 0.405 with the Born hook, against:
  - 0.118 with the Born hook removed;
  - 0.048 for the L1 replacement base, which has no squaring and cannot peak.
- **The replacement is smoother and more erasure-stable:** best data 0.983, against the quantum stack's
  0.964.
- **A smoother above it fights it:** removing the median smoother raised range to 0.421.
- **No-copy Correction 3, the one clean benefit on record:** when a mass-action chemistry atom used
  Born weights as concentrations, staleness reached 32.03. Born-only gave 31.50 and classical 31.33.
- **Fit:** good wherever the target is salience or concentration; bad where the target is spread or
  stability.

### 3. Unitary mixing (local mixers, Walsh): *spread, and undo commitment*

- **What it does:** it spreads mass ballistically. Variance is 28.98× the classical walk's, and the
  spread exponent α is 2.000 against heat's 1.000 (E2, E5).
- **Undoing commitment:** in the cache it erases collapse pinning and pulls behaviour toward
  random-but-structured. Intact scored hot 6,527 (random 6,484) and scan 343 (random 169, LRU 0). That
  is scan resistance, but no hot-set retention.
- **It is the dominant term of the cache base:** stage order moves the result only weakly, with
  spreads of about 181 / 8.7 hits.
- **Coherent before Born** favours spread-out ("data") evidence: 0.964 (signal v1 and Correction 1).
- **Removing the Walsh hook costs both metrics:** range 0.405 → 0.297, data 0.964 → 0.904.
- **In no-copy** it changes every trajectory, but pulls staleness toward 32, the decorrelated midpoint
  (see law 3).

### 4. Phase: *inert until something mixes it*

- **What it does:** phase does nothing to an outcome on its own.
- **E1:** the phase-tag-only coin stays flat at 0.49–0.51. Interference fringes need a mixer after the
  phase.
- **Cache:** the per-access phase is nearly weightless. The no-phase arm scored 6,639 / 341.6 / 9,535
  against intact. The cognitive engine's unitary epoch carrier is inert too: support 10 = 10 (BUILD-34).
- **Seeded phases make outcomes seed-dependent:** security-burst flood peaks range 7.4–47.0.
- **A phase-carried parameter bounds extraction:** Fisher information F = 1 in any basis, so a stolen
  copy's error is at least 1/√B. Measured: error·√B → 1.02–1.03, exponent −0.5232
  (`qm_policy_extraction`).

### 5. Interference (phase plus mixer): *the base makes the structure*

- **In the cognitive engine:** adding intensities gives a numerically flat field, so all structure
  comes from interference (BUILD-1).
  - Coherent base 4 against classical 0 (BUILD-34); the coherent base wins 24 of 24 paired instances
    (BUILD-35).
  - **But the base produces state, never a verdict:** base-only gives identical readouts for two
    different targets (BUILD-34).
- **Direction depends on the input's shape (security-burst, a Python forecast):** it attenuates a
  single impulse (0.63–0.77× the peak), but **amplifies sustained input at the source**:
  - 1.17–3.09× the peak, in 24 of 24 orders on every sustained envelope;
  - this looks like refocusing or revivals.
- **Cache:** it bought scan resistance, not retention.

### 6. Coherence as a state-relation: *a read leaves a trace*

- **Reading it costs:** the disturbance is between 1× and 2× the attacker's gain, about 2× in the
  gentle limit (`qm_no_free_peek`).
- **The trace grows linearly while fidelity loss is quadratic:** the trace channel is 133× more
  sensitive.
- **Detection is decoupled from consumption:** a coherence decoy carries the trace, while the real
  secret stays bit-identical (`qm_same_trace_decoy`).
- **Blind spots:** a basis-encoded value is read for free (fidelity 1.0000000000), and memory access
  is silent.

### 7. Complementarity (V² + D² ≤ 1): *a bound no tuning escapes*

- **What it does:** interference against which-way information obeys V² + D² ≤ 1, tight (maximum
  overshoot 0.00140).
- **Why it matters:** it is the only sum-bound (conjugate) constraint found, and it underlies the
  no-free-peek guarantee (`qm_complementarity`).

### 8. Entanglement and tensor product: *information lives in the correlation*

- **The correlation exceeds any local bound:** CHSH S = 2.8158 against 2.
- **No part holds the information:** a local spy scores 0.2481 against chance 0.25, while the joint
  readout scores 1.0000.
- **The channel stays silent:** Bob's marginal is 0.496–0.506 (E3, E6).
- **Structured entanglement compresses, generic does not:** GHZ needs a 1-bit hidden variable, while
  random circuits leave the product model at TVD 0.44–0.54 (E13).
- **Under tapping it degrades in trend but cannot localize:** a full tap leaves a product state at
  S = √2, which equals a classical bit at every angle. A 25% duty-cycle tap still reads healthy
  (e19b, qstack).

### 9. Monogamy (XOR shares): *a single holder and no reuse*

- **What it covers:** a one-share holder, and the reuse of a spent share.
- **What it does not cover:** the copier (`qm_axiom_sweep`, e27). None of the 4 axioms covers the
  copier; it is a substrate problem, not an axiom problem.

### 10. Coherent complex field with coupling: *hold the outcome steady*

- **No-copy Correction 5:** coupling through the coherent field made outcomes nearly independent of
  the input.
- **The spread across corpora fell from 4.14 bits uncoupled to 0.06 bits coupled.** The mean did not
  rise: coherent coupled 31.51 / 31.45 / 31.47, against classical 31.49 / 31.46 / 31.45.
- **Gates V6–V8, which wanted the mean to rise, were labelled FAILED.** The character is
  stabilization.

### 11. Density coherence (Im ρ): *turn phase slope into modulation*

- **Hybrid LoRa trial:** for chirps, Im ρ is in effect the phase step. Its hook becomes
  self-phase-modulation, which widens the spectrum.
- **Its effect on out-of-band leakage dwarfs everything else:**
  - it causes about 93–98% of the added leakage;
  - that is about 15–45× the whole classical stack's effect, while chaos changes OOB by about 1e-7;
  - full stack 1.832% against identity 1.798% (plain).
- **Phase-free base = no-base.**
- **R8 and R9 were labelled FAILED.** This is the strongest lever in that stack, pointed the wrong way
  for OOB.

### 12. Planck occupation: *a spectral gate*

- **TA1:** the Planck factor rejects about 44% of excitation and transmits less than the classical
  q = 1:
  - outgoing/incident 2.30e-3 against 4.19e-3 (aggregated from `thermal.csv` by a reader; not stated in
    the docs);
  - base-only and no-base give exactly 0 outgoing.
- **It is inert at zero drive** (SDR-1, CAP1 byte-identical).
- **It is useful only when paired with reservoir damping:** Q2 coherence 0.5415, against 0.4454 with no
  base. There, an independent classical exponential cooling matches it within 1.86e-9.
- **D1:** its local law holds (0.44% error), and it strongly activates the field (retained field
  3.7e-6 → 2.1).

### 13. Other physics bases in the substrate

- **Lindblad collective damping (Q1):** destructive. Coherence 0.307 against 0.445.
- **Bosonic three-wave Hamiltonian (P1):** real gain of +2.46 to 3.12 dB, but only after the body.
  A classical RK4 reference matches it (ratio 1.0001–1.0003).

### 14. Complex inertia (a blend with the previous state): *mixing in time* (QB-3)

- **What it does:** a ← (1 − μ)a + μ·a_prev on complex amplitudes is interference across time. It
  makes phase load-bearing even with no mixer: frozen phase disorder costs 0.212 with inertia (0.190
  against 0.402), where it is inert without it (QB-2 Q4).
- **It fights per-step phase disorder:** it blends a state with a rotated copy of itself, so it cancels.
  Removing it restores frozen chaos's localization (+0.171).
- **The general form:** phase-carrying pieces interact pairwise through interference; their effects do
  not add.

## Classical hooks observed on a quantum base

- **Chaos placement decides what it is.**
  - Before a threshold or k-fold gate it is a **filter**: selectivity 4.7 → 151.1, and false positives
    down about 100×.
  - After a decision it is an **eraser**: rates converge to 2⁻ᵏ, ratio 1.0, 85× worse
    (`qm_chaos_twice`).
  - At the end of an assembly it erases: separation 1.0000 → −0.0025.
  - On a convex quantity (exp(−Ea/kT)) it amplifies, by Jensen's inequality.
  - Chaos first averaged 30.05 and chaos last 32.20 (no-copy Correction 4).
- **Logistic chaos on the cache base:** a small directional bias (scan +13.3, 28/32; hot −124, 0/32),
  which a linear control partly reproduces.
- **Chemistry as a consumer of Born weights** (mass action, concentrations): the only clean benefit
  pass (law 2 above).
- **A chemistry bond as a coupling between branches** averages the branches and never selects the
  better one. Mean g is about 0.69–0.77.
- **A catalyst's allegiance is set by its position:**
  - at the defender's repair barrier: 6.58× rounds to break;
  - at the attacker's break barrier: 0.56×.
- **Crypto goes last:** the damage equals the downstream dependency closure, and crypto placed last
  costs exactly 0. The rule only applies where something depends on what crypto hides.

## Laws that cut across the record

1. **A base alone is inert, and produces state, never a verdict.** Base-only equals identity in
   signal Correction 1 and the LoRa trial, moved zero shares in no-copy, and gave identical readouts
   for different targets in BUILD-34. The behaviour exists only through the axioms hooked above.
2. **The base imposes its character; it does not average away** (catalog stack 1). This is the core
   reason outcomes were "off-target": the target came from the goal, but the result came from the
   character.
3. **The base often behaves as an attractor, not a booster.**
   - On the no-copy staleness metric, quantum and coherent bases sit at about 31.4–31.7 bits, the
     decorrelated midpoint of a 64-bit token. Classical and zero-base arms scatter from 31.3 to 33.7.
   - The coherent field with coupling holds the outcome steady across inputs (4.14 → 0.06 bits).
   - A gate that asked the base to "raise the mean" was asking it to do the opposite of its character.
4. **Position decides the effect:**
   - chaos before or after a decision;
   - coherent before or after Born;
   - a catalyst at repair or at break;
   - crypto first or last;
   - the cognitive engine's InjectThenMeasure order collapses support to 1, against 10.
5. **Where a classical algorithm competes, it ties.** BUILD-34/35, Q2 cooling, P1 RK4, the tuned
   threshold in `sec_roc`. The engine's own record calls the base "a PATH, not an information
   advantage". Its value is in what it makes the stack *do* (its character), not in beating a
   classical method at the same job.

## Retro-prediction: the "failures", read from the characters

| Off-target outcome | Character that predicts it |
|---|---|
| Cache intact stack is random-like on hot/shifting and resistant on scan | Mixing dominates (3): random-but-structured |
| Cache no-mixers arm pins 7 pages | Commit with no mixer (1) |
| No-copy staleness stuck near 32; "raise the mean" gates fail | Mixing plus Born pull toward the decorrelated midpoint (law 3) |
| Correction 5's coupled field doesn't raise the mean, but steadies it | Coherent coupling stabilizes (10) |
| LoRa OOB gets worse | Density coherence → self-phase-modulation widens the spectrum (11) |
| Signal C1: replacement wins on data, quantum wins on range | Born squaring peaks (2); the replacement smooths |
| E1R-3/4: statistical floors fail under Never/PerWindow | Definiteness turns M readings into one sample (1) |
| Security-burst amplifies sustained input | Interference refocuses repeated injection (5) |
| TA1 transmits less than classical q = 1 | Planck occupation gates the spectrum (12) |

**Most of the outcomes recorded as failures follow from the base's character once it is named.** So the
next step is to test this forwards, not only backwards.

## Caveats

- **Not evidence about a base:**
  - `qm_chaos_pair` fails its own control;
  - `qm_interleave` runs in a broken regime, and its "base" is `0.10·rng.unit()`;
  - `qm_self_assembling` has only a nominal base.
- **The cache experiment has no no-measurement control:** Born sampling is always how the victim is
  chosen. Its measurement character is inferred from the no-mixers arm and the veil dial.
- **The security-burst numbers are Python forecasts:** the Rust gates G0–G6 are UNRUN.
- **Stale or inconsistent records:**
  - `PERSISTENCE-FILTER.md` still lists the imitator as unmeasured (it is measured);
  - `qstack-simulated.json`'s stored verdict contradicts the thread's conclusion;
  - `qm_onetake_assembly`'s prose ("P4 failing", "~240 uses") contradicts its own table.
- **On a PC, Born sampling is a seeded Bernoulli draw** ("chaos-like in kind"). Every result here is
  quantum-influenced classical behaviour, never quantum computation (Rule 4).

## Toward the synthetic brain (to discuss, not decided)

The cognitive engine already runs on this base, with interference, destructive readout and
definiteness. Read as characters, these look like brain functions:

| Character | Possible brain role |
|---|---|
| Destructive readout | Forgetting; clearing working memory |
| Definiteness | Commitment; a decision that holds |
| Mixing | Exploration; spreading activation |
| Born squaring | Salience, attention, peaking |
| Coherent coupling | Stabilizing a state against input noise |
| Interference | Building a scene's structure |
| Phase-carried parameters | Guarding a policy against extraction |

**The forward test:** choose a composition, precommit the outcome predicted from the characters (not
from the goal), and run it. If the prediction holds, the base can be steered on purpose.
