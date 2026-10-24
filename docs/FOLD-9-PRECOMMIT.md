# FOLD-9: a PERCEPTION gate, so recall is never worse than plain

Status: PRECOMMITTED (2026-10-24), before any FOLD-9 code.

FOLD-8 and ASTRO-R found one tipping point. In the 2-bit store (D 4,096, 1,024 bytes), the
cognitive recall loop lifts N 512 from 0.636 to 0.966 (ASTRO-2R: the virial self-set temperature,
6 rounds). Past about N 768, where most first reads are wrong, it falls below plain. FOLD-9 gives
the loop a judgment over the whole store, so it runs only where it helps.

## The best stack, from ASTRO-2R

Soft cancellation at each item's own temperature T_i = κσ_i²/m_i (κ 0.1), 6 rounds, with PREDICT,
CONSERVE and REVISE as in FOLD-8.

## Two candidate gates (Rule 2: one to three, each with its reason)

- **G-P, a PERCEPTION gate (thin policy in the orchestrator, labelled):**
  - after the first read, the store forms the named hypothesis "recoverable", citing one reading:
    the fraction f of items whose first margin is above g, the noise floor (the median first margin
    of 64 keys never written, read from the same store);
  - the loop runs only if f ≥ θ, with θ chosen on dev from {0.6, 0.7, 0.8, 0.9}.
  - *Amended before code:* the first grid, {0.3 … 0.6}, could never close the gate. About half the
    wrong items clear a median floor, so f ≥ about 0.5 even for a store that fails entirely.
    Expected f is about 0.90, 0.82, 0.71 and 0.65 at N 384, 512, 768 and 1024.
  - this is the class separation: the margins are sensing, "recoverable" is the perception, and
    running the loop is the judgment.
  - **The policy-free alternative is G-V.**
- **G-V, verify at the boundary (O9, no threshold):**
  - run the loop, then keep its result only if its total margin beats the first pass's;
  - otherwise dissolve it and keep plain (REVISE's dissolve-first).

**ATTEND + REINFORCE** (FOLD-8 K2, β 0.01, counts from the top half of first margins) is added on
the glossary, where it lifted the deepest loads.

## The setup

- Random bytes and the glossary; N ∈ {384, 512, 768, 1024}; test seeds 10–17; dev seeds 0–3.
- **Arms:** plain; the stack ungated; G-P; G-V. On the glossary: plain; ATTEND alone; G-P + ATTEND.
- Chaos and amplification were settled on this stack in ASTRO-2R (chaos not kept; rounds × 2 is
  the amplification already inside). The gate is one judgment per store, not an iterated step, so
  it gets no chaos arm.

## Predictions

1. **P1, G-P is never worse than plain:** ≥ plain − 0.01 at every N, on both contents.
2. **P2, G-P keeps the gains:** ≥ the ungated stack − 0.01 at N 384 and 512.
3. **P3, G-V is weaker at depth:** at N 768, G-V ≤ G-P − 0.02. A wrong cancellation can raise
   margins too (confident wrong answers), so a margin check is fooled where the perception gate,
   which looks before acting, is not.
4. **P4, the best at every load:** on the glossary, G-P + ATTEND ≥ the better of plain and ATTEND
   alone − 0.01 at every N, and ≥ 0.95 at N 512.

A new orchestrator crate, `fold9`, copied from `astro2r`. CPU, 8 threads, a few minutes.

## Results (2026-10-24, `evidence/fold-9/results.txt`, 148 s on 8 threads)

Dev chose **θ 0.6**, the edge of the grid (dev means 0.657, 0.610, 0.578, 0.537 for θ 0.6 to 0.9).
The reading f (test mean) is 0.811, 0.702, 0.607 and 0.582 at N 384, 512, 768 and 1024, lower than
my estimate.

| random content (test) | 384 | 512 | 768 | 1024 |
|---|---|---|---|---|
| plain | 0.800 | 0.636 | **0.425** | **0.301** |
| the stack, ungated | 0.998 | 0.966 | 0.381 | 0.244 |
| **G-P, the perception gate** | **0.998** | **0.966** | 0.409 | 0.268 |
| G-V, the verify gate | 0.998 | 0.966 | 0.381 | 0.244 |

| glossary (test) | 384 | 512 | 768 | 1024 |
|---|---|---|---|---|
| plain | 0.798 | 0.637 | 0.425 | 0.305 |
| ATTEND alone | 0.883 | 0.802 | 0.664 | 0.547 |
| the stack, ungated | 0.999 | 0.972 | 0.377 | 0.243 |
| **G-P + ATTEND** | **0.997** | **0.981** | **0.777** | **0.552** |

| # | Result |
|---|---|
| P1 G-P never worse than plain − 0.01 | **FAILED on random content:** 0.409 against 0.425 at N 768, and 0.268 against 0.301 at N 1024. The glossary holds |
| P2 G-P keeps the gains at N 384 and 512 | MEASURED: 0.998 and 0.966, as ungated |
| P3 G-V weaker at N 768 | MEASURED: 0.381 against 0.409; the verify gate never rejected (margins after cancellation always look better) |
| P4 glossary: G-P + ATTEND the best at every N, ≥ 0.95 at 512 | MEASURED: 0.981 at N 512, and the best at every N |

**3 of 4 hold.**

### What it settles

- **The perception gate works in kind but is too coarse in degree.** It halves the loss at N 768–1024
  on random content, but θ's grid stopped at 0.6 and the reading f at N 768 (0.607) sits on that
  edge. A finer θ (about 0.65), or a reading with more contrast than "above the noise median", is the
  fix.
- **The verify gate is fooled every time:** cancellation always raises the total margin, right or
  wrong. Confidence after acting is not evidence. The perception gate, which looks before acting, is
  the right class.
- **On real data, gate + ATTEND is the best recall at every depth:** 0.981 at N 512, and 0.777 at
  N 768 against plain's 0.425.

**Niches** (Jesse's standing principle):
- **G-P + ATTEND:** the recall stack for redundant data at any load.
- **The ungated virial stack:** the deepest clean recall on random data below the tipping point
  (0.966 at N 512).
- **Plain reading:** past the tipping point on random data.
