# FOLD-8: recall with the cognitive roots (cognitive on against off)

Status: PRECOMMITTED (2026-10-23), before any FOLD-8 code.

Jesse (2026-10-23): "did we try recall axioms", then "both, cognitive first". Every recall so far is
one read: one cue, one look, take the best code. Jesse's build rule tests every stack with cognitive
on and off, and folding has not had that test. This applies his roots (`atom-cognitive-engine/docs/
ROOTS.md` v0.2, `ORDERING.md`; PERCEPTION from `a-rag-atom/docs/COGNITIVE-PRINCIPLES.md`) to recall,
**in the orchestrator only**. The functions (`qb-bind`, `qb-quantize`, `qb-overlap-phase`) are
unchanged. The atlas's recall axioms come after, in FOLD-9.

His composition table already names the problem: *"interference: not an operation — the failure
mode GATE exists to govern"*. The crosstalk between folded memories is exactly that interference.

## The store

FOLD-6's content store: one phase store, D 4,096, **2 bits** of phase (1,024 bytes), N byte values
bound to recipe keys. N ∈ {256, 384, 512, 768, 1024}; random bytes and the glossary's bytes. Test
seeds 10–17; dev seeds 0–3 choose every threshold. **Plain (cognitive off)** is FOLD-6's single
read; on random content it gives 0.937, 0.800, 0.636, 0.425 and 0.301.

## The cognitive arms (the query cycle: attend → gate → retrieve → assert-or-abstain)

### K1: explaining away (PREDICT → GATE → REVISE, with CONSERVE and PERCEPTION)

Three rounds after the first read.
- **PERCEPTION:** each item carries a named hypothesis, "item i holds v", which cites its reading
  (the margin between its best and second-best code, and the round). It is revisable: a later round
  can retract and replace it, and the replacement is recorded, not silently overwritten.
- **GATE (thin policy, labelled):** only the most certain hypotheses may act. A hypothesis passes if
  its margin is in the top ρ of this round's margins; ρ ∈ {0.25, 0.5, 0.75} is chosen on dev.
  - The policy-free alternative is ρ = 1 (no gate), and it is run as an ablation.
- **PREDICT:** the expected interference is the sum of the passed hypotheses' bound vectors,
  E = Σ bind(key_j, code_v̂j).
- **CONSERVE:** E is scaled onto the store by least squares (α = Re⟨E, s⟩ / ‖E‖²). The store holds
  the quantized fold, not the raw sum, so this keeps the cancelled mass equal to what is there.
- **REVISE (dissolve first, then replace):** each item is read again from s − α·(E minus its own
  term), so the other memories' predicted interference is dissolved before the re-read. The new
  reading replaces its hypothesis.

### K2: ATTEND with REINFORCE (a prior over the 256 values)

- **REINFORCE:** after a read pass, each passed hypothesis adds 1 to its value's count.
- **ATTEND:** soft relevance weights over the candidates from those counts, w_v = (count_v + 1) /
  (Σ + 256). Each code's reading is weighted, score_v + β·ln w_v, with β ∈ {0.002, 0.005, 0.01,
  0.02} chosen on dev. It never decides; the read still takes the best.
- This is the root that can use the data's redundancy, which FOLD-6 found folding ignores.

### K1 + K2 together, and assert-or-abstain

- **Together:** each K1 round reads with K2's attention.
- **Assert-or-abstain (GATE as the final judgment):** an item is asserted only if its final margin
  is at least g, with g chosen on dev as the smallest giving precision ≥ 0.99. Report coverage and
  precision on test.

### K6: hot and cold in the recall loop (added before any code, 2026-10-23)

Jesse: "Would hot and cold work outside the thermostate". FOLD-5 measured heat as noise on the stored
memories: it only costs. The place where temperature can act outside the thermal state is the recall
loop, which K1 makes iterative.
- **Hot:** each item's predicted interference is soft. It is the bound mixture over all 256 values,
  bind(key_j, Σ_v p_v·code_v), with p = `qb-classical::temper(exp(scores), T)` (Boltzmann weights at
  temperature T).
- **Cooling:** T falls each round (T₀, T₀/2, T₀/4), and the last read is hard. This is O8,
  "exploration decays monotonically; settle before assert".
- T₀ ∈ {0.005, 0.01, 0.02} (the scores' noise scale is 1/√D ≈ 0.016), chosen on dev.
- **No GATE:** a soft hypothesis that is unsure spreads its weight and cancels little by itself.
  CONSERVE scales the soft E as in K1.
- **The cold end** is K1's hard cancellation of every hypothesis without a gate (the ρ = 1
  ablation). Hot-only (T fixed at T₀, no cooling) is run beside it.

### K7: amplification on and off (added before any code, 2026-10-23)

Jesse: "And amplification". His build rule's matrix:
- **plain:** the single read;
- **amplification alone:** write gain × 2, single read;
- **cognitive alone:** the best of K1 and K6;
- **together:** the best cognitive stack, with its predicted interference amplified by G ∈ {1.25, 1.5}
  (over-cancellation, "more of what exists"), G chosen on dev.

### Chaos last, on the kept stack

Each round, each item's cancellation scale is α·(1 + 0.3(u − ½)), with u from
`qb-chaos::keyed_unit` (the logistic map, mean ½). The selector is the stack's own confidence, the
sum of the round's margins, among 3 variants; plain wins ties. Dev, then test.

## Predictions (test seeds)

From the precision of a 2-bit phase (it keeps 0.81 of the power): after perfect cancellation, an
item's signal-to-noise ratio rises from 0.9·√(D/N) to about 2.1·√(D/N), so up to about 5× the
items. Errors in the passed hypotheses take part of that.

1. **K1a, explaining away doubles the depth:** random content, N 512: ≥ 0.95 (plain 0.636); N 768:
   ≥ 0.80 (plain 0.425).
2. **K1b, the GATE is load-bearing:** with ρ = 1 (every hypothesis cancels, wrong ones too), K1 is
   ≥ 0.05 worse at N 768.
3. **K1c, CONSERVE is load-bearing:** with α fixed at 1 (no fit onto the quantized store), K1 is
   worse than plain at N 768.
4. **K2a, attention uses the redundancy:** glossary, N 512: K2 ≥ plain + 0.05.
5. **K2b, and only the redundancy:** random content: K2 within ±0.01 of plain at every N.
6. **K3, together ≥ either alone** on the glossary at N 768.
7. **K4, assert-or-abstain:** random content, N 384 (plain 0.800): precision ≥ 0.99 with coverage
   ≥ 0.6, plain reads.
8. **K5, chaos on the cancellation scale: not kept.** The least-squares α is already the optimum of
   a convex fit.
9. **K6a, annealing beats hard cancellation:** random content, N 768: annealed soft ≥ hard K1 (with
   its gate) + 0.03. Soft decisions keep wrong hypotheses from cancelling at full weight.
10. **K6b, cooling is load-bearing:** annealed ≥ hot-only (fixed T₀) + 0.03 at N 768. A store read
    hot to the end never settles.
11. **K7a, amplification alone is an exact tie at every N.** The 2-bit quantizer keeps phase only,
    and a louder write has the same phase.
12. **K7b, together: not kept** (≤ +0.01). The least-squares α is already the best scale.

Amplification is not rerun (FOLD-5 measured it inert on this store).

A new orchestrator crate, `fold8`. CPU only, 8 threads, a few minutes.

## Results (2026-10-23, `evidence/fold-8/results.txt`, 11 min on 8 threads)

**Dev choices:** ρ 0.75 (dev means 0.649, 0.682, 0.698; at the grid's edge); β 0.01; T₀ 0.005 (also
the grid's edge); best cognitive stack K1 (hard; dev 0.698 against K6's 0.674); G 1.25.

**Random content, test (accuracy at N 256, 384, 512, 768, 1024; the store is 1,024 bytes):**

| arm | 256 | 384 | 512 | 768 | 1024 |
|---|---|---|---|---|---|
| plain (cognitive off; reproduces FOLD-6) | 0.937 | 0.800 | 0.636 | **0.425** | **0.301** |
| amplification alone (write × 2) | 0.937 | 0.800 | 0.636 | 0.425 | 0.301 |
| **K1 explaining away** | 0.999 | 0.983 | **0.894** | 0.407 | 0.219 |
| K1 without GATE (ρ 1) | 1.000 | 0.999 | 0.857 | 0.369 | 0.201 |
| K1 without CONSERVE (α 1) | 0.007 | 0.002 | 0.000 | 0.001 | 0.000 |
| K2 ATTEND + REINFORCE | 0.900 | 0.717 | 0.569 | 0.385 | 0.269 |
| K1 + K2 | 0.997 | 0.952 | 0.794 | 0.312 | 0.157 |
| **K6 annealed (hot → cold)** | **1.000** | **0.999** | 0.884 | 0.334 | 0.170 |
| K6 hot only (no cooling) | 1.000 | 0.999 | 0.798 | 0.285 | 0.134 |
| together: K1 + over-cancellation G 1.25 | 0.997 | 0.965 | 0.788 | 0.285 | 0.142 |
| chaos on K1's cancellation scale | 0.999 | 0.983 | 0.896 | 0.404 | 0.218 |

**Glossary content, test:**

| arm | 256 | 384 | 512 | 768 | 1024 |
|---|---|---|---|---|---|
| plain | 0.945 | 0.798 | 0.637 | 0.425 | 0.305 |
| K1 explaining away | 1.000 | 0.980 | **0.902** | 0.400 | 0.211 |
| **K2 ATTEND + REINFORCE** | 0.960 | 0.883 | 0.802 | 0.664 | **0.547** |
| **K1 + K2** | 0.996 | 0.951 | 0.894 | **0.668** | 0.484 |
| K6 annealed | 1.000 | 0.999 | 0.881 | 0.333 | 0.164 |

**K4, assert-or-abstain** (plain reads, random, N 384; margin gate 0.0139 chosen on dev): test
precision 0.989, coverage 0.362.

| # | Result |
|---|---|
| K1a explaining away: N 512 ≥ 0.95, N 768 ≥ 0.80 | **FAILED:** 0.894, and 0.407 (below plain) |
| K1b GATE load-bearing (≥ 0.05) | **FAILED:** no gate is 0.038 worse at N 768, and *better* at N ≤ 384 |
| K1c CONSERVE load-bearing | MEASURED: without it, recall collapses to 0.000–0.007 |
| K2a attention uses the glossary's redundancy (≥ +0.05) | MEASURED: N 512 0.637 → 0.802; N 1024 0.305 → **0.547** |
| K2b attention a tie on random (±0.01) | **FAILED:** it *costs* up to 0.084; a prior built from noisy hypotheses misleads |
| K3 together ≥ either alone (glossary, N 768) | MEASURED: 0.668 (K1 0.400, K2 0.664) |
| K4 precision ≥ 0.99 at coverage ≥ 0.6 | **FAILED:** 0.989 at 0.362 |
| K5 chaos not kept | MEASURED: a tie (+0.000) |
| K6a annealed ≥ hard + 0.03 at N 768 | **FAILED:** 0.334 against 0.407 (annealed wins at N ≤ 384) |
| K6b cooling load-bearing | MEASURED: 0.334 against hot-only 0.285 |
| K7a amplification alone an exact tie | MEASURED: identical at every N |
| K7b together not kept | MEASURED: G costs 0.065 on average |

**7 of 12 hold.**

### What it settles

- **The cognitive roots raise the fold's depth, below a tipping point.** In the same 1 KB store:
  - at 384 memories, plain 0.800 becomes **0.999** (K6, annealed);
  - at 512, plain 0.636 becomes **0.894** (K1).
  - The store holding 384 values at ≥ 0.95 instead of 256 is 1.5× the memories at no cost in bytes.
- **Above the tipping point they backfire.** When most first reads are wrong (N ≥ 768, plain ≤ 0.43),
  cancelling wrong hypotheses spreads their error and recall falls *below* plain. This is error
  propagation in interference cancellation. The tipping point sits near a first-read accuracy of
  about 0.6. My prediction assumed the gated hypotheses stay mostly right; at depth they do not.
- **ATTEND + REINFORCE is the root that uses the data's redundancy.** On the glossary it lifts the
  deepest loads the most (1,024 values: 0.305 → 0.547). On random bytes it costs, because a prior
  learned from noisy hypotheses is a bias, not knowledge. This answers FOLD-6's finding that folding
  ignores redundancy: **the cognitive layer recovers it, the store does not need to.**
- **Hot → cold annealing is the best stack while the first reads are good** (0.999 at 384), and
  cooling is load-bearing (hot-only is worse). At depth, soft cancellation of wrong hypotheses costs
  more than hard.
- **CONSERVE is the most load-bearing root here.** Unscaled cancellation destroys the store.
- **GATE as written is weak.** Selecting the top ρ by margin barely matters. The ablation is better at
  low N and only 0.04 worse at high N.
- **Amplification and chaos: inert or harmful,** as on every store of this kind.

### The next step this points to (not tested)

A PERCEPTION-level GATE over the whole pass: a named hypothesis "the store is recoverable",
citing the first pass's margin distribution. Cancellation runs only when it holds, so the stack is
never worse than plain. With that gate the best results per depth would combine:

| N | random | glossary |
|---|---|---|
| 384 | 0.999 (K6) | 0.999 (K6) |
| 512 | 0.894 (K1) | 0.902 (K1) |
| 768 | 0.425 (plain) | 0.668 (K1 + K2) |
| 1024 | 0.301 (plain) | 0.547 (K2) |
