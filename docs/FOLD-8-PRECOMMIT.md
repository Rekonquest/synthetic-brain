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

Amplification is not rerun (FOLD-5 measured it inert on this store).

A new orchestrator crate, `fold8`. CPU only, 8 threads, a few minutes.
