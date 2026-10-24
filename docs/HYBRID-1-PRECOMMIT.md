# HYBRID-1: each memory where it shines

Status: PRECOMMITTED (2026-10-24), before any HYBRID-1 code.

Jesse (2026-10-24): "These can all be useful, we can use this as well, but by using a hybrid, where
we can assign memories depending on what memories shine for what memory storage." This is the
field's *complementary learning systems*: no single store is best at everything (FOLD-4 to 9).
HYBRID-1 tests routing between stores. His open question, "should the router learn over time, or
decide fresh for each memory?", is answered by measurement: both are candidate routers.

## The stores (co-engines; each unchanged from where it was measured)

| store | holds | shines at (measured) | cost |
|---|---|---|---|
| **raw** (AtomDB's role) | a value at an exact address | exact recall, 1 byte per byte | an address that must be exact |
| **association** (FOLD-6 to 9) | values bound to keys, 2 bits of phase, D 4,096 (1 KB) | recall from a partial or noisy cue; it can also answer an exact query (the full key) | a tipping point with load |
| **recognition** (FOLD-5's phase store, 2 bits, D 4,096) | patterns superposed | "have I seen this?" | no content |

The thermal scar store is left out of HYBRID-1 for run time (minutes per write); it is the
lasting-memory store for HYBRID-2.

## The workload

A stream of 768 memories in random order, of three kinds, 256 of each. Each kind has its query at
the end:
- **exact:** a value recalled from its exact address;
- **associative:** a value recalled from a cue that has only a quarter of its key (FOLD-7 B);
- **recognition:** was this pattern seen? (256 seen and 256 unseen probes).

**The budget** is equal for every router: raw 768 bytes, association 1 KB, recognition 1 KB.

## The routers (Rule 2: three candidates, each with its reason)

- **R-fixed, by kind:** exact → raw, associative → association, recognition → recognition. The
  designer's assignment.
- **R-shine, per memory (Jesse's idea, deciding fresh):** each memory is written tentatively into
  every store that can answer its kind of query. Its own query is read back from each, and it is
  kept where the readback is best (the readback margin is the reading; ties go to the cheaper
  store). The trial writes are then dissolved (REVISE).
  - A store near its tipping point reads back worse, so the memory goes elsewhere: load balancing
    with no rule.
- **R-learn, over time:** route the first 20% of each kind by shine, REINFORCE the store that won
  for that kind, then route the rest by the reinforced choice, with no more trial writes (cheaper).
- **The bars:** everything in raw (a cue that is not an exact address fails); everything in the
  association store.

**Recall** in every store is its plain single read: phase-locked for association, overlap against a
threshold τ (chosen on dev) for recognition, and lookup for raw. *Amended before code:* the gated
stack of FOLD-9 was first named here, but the shine router reads back every trial write, and the
recall loop on each trial is too slow for a proof of concept. The gated stack is for HYBRID-2.
- **What each store can hold:** raw holds values (1 byte each) and cannot hold a recognition
  pattern; association and recognition hold anything, at the cost of crosstalk.
- **Cost, for ties:** raw 1 byte; association and recognition at their store's bytes divided by the
  items they hold.

## Predictions

1. **H1:** R-fixed beats both single-store bars on the total success rate across the three queries,
   by ≥ 0.15.
2. **H2:** R-shine ≥ R-fixed − 0.02 overall, without being told the kinds. It finds the fixed
   assignment by itself, as the readbacks differ sharply by kind.
3. **H3:** R-learn is within 0.02 of R-shine and makes ≥ 70% fewer trial writes.

*Removed before code:* an H4 on overflow. No other store can answer a partial-cue query, so
associative memories have nowhere to overflow to; the prediction would fail by construction.
Overflow needs a second store with that ability (HYBRID-2, with the thermal store).

A new orchestrator crate, `hybrid1`. CPU, a few minutes.

## Results (2026-10-24, `evidence/hybrid-1/results.txt`, 8 s)

Dev chose τ 0.03 for recognition (dev recognition 0.729 → 0.984 over τ 0.005 → 0.03).

| router (test) | exact | associative (¼ cue) | recognition | overall | trial writes |
|---|---|---|---|---|---|
| all in raw (bar) | 1.000 | 0.000 | 0.498 | 0.499 | 0 |
| all in association (bar) | 0.407 | 0.075 | 0.735 | 0.406 | 0 |
| **R-fixed, by kind** | 1.000 | 0.293 | 0.981 | **0.758** | 0 |
| **R-shine, per memory** | 1.000 | 0.296 | 0.983 | **0.760** | 1,280 |
| **R-learn, over time** | 1.000 | 0.293 | 0.981 | **0.758** | 260 |

| # | Result |
|---|---|
| H1 R-fixed beats both bars by ≥ 0.15 | MEASURED: 0.758 against 0.499 and 0.406 |
| H2 R-shine ≥ R-fixed − 0.02, without being told the kinds | MEASURED: 0.760 |
| H3 R-learn within 0.02 of R-shine with ≥ 70% fewer trials | MEASURED: 0.758 with 260 trials against 1,280 (−80%) |

**3 of 3 hold.**

### What it settles

- **A hybrid beats any single store** by a wide margin (0.76 against 0.50 and 0.41), because each
  store does what it is best at.
- **Jesse's "where it shines" router finds the right assignment by itself,** from readbacks alone,
  with no kinds given. It matches the designer's fixed assignment (0.760 against 0.758).
- **His open question, learn or decide fresh: learning is as good and 80% cheaper.** Once a kind's
  best store is reinforced, trial writes stop with no loss.
- **The weak spot is the associative query from a quarter cue at this load** (0.29). 256 bound
  values plus the crosstalk of everything else is past this store's quarter-cue capacity (FOLD-7 B
  measured 0.984 at N 48). This is the place for FOLD-9's gated recall stack, and for a second
  associative store to overflow into (HYBRID-2).

**Niches:**
- raw: exact recall from an address;
- association: the only store that answers a partial cue;
- recognition: "seen before?" at 0.98;
- the shine router: finds niches by itself;
- the learned router: the same result at a fifth of the cost.

*Precision: f64 throughout (see `docs/TODO.md`, the precision note).*
