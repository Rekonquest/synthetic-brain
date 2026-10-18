# QB-4: small stacks orchestrated, on the working-memory task

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

QB-3's monolithic complex stack failed (0.112). Its phase-carrying pieces interfered pairwise. Jesse's
principle, from that result:

> "monolithic complexity doesn't seem how physics tends to work, but rather small complexity, meaning
> many small stacks that do one thing great then orchestrated to work in tandem/together like neurons."

**QB-4 tests that principle on QB-3's task, unchanged** (the same task, settings, development seeds
and test seeds). Each module is a small stack with one job. Modules pass readings to each other the
way neurons pass signals.

## The modules (each a small stack, one job)

| Module | Job | Its one mechanism | The character it uses |
|---|---|---|---|
| **D, detector** | notice a strong new event | persistence gate: the observed option fires an event when it appears ≥ 3 times in the last 4 observations | conjunction suppresses iid noise as p^k (persistence filter) |
| **M, memory** | hold one item through noise | a quantum state written by **destructive projection** onto D's event option; otherwise untouched | collapse commits and stays committed (QB-1 P1), and forgets everything else |
| **T, tracker** | follow the evidence and decide | QB-1's `born` with a cold readout: amplitudes, REINFORCE / DECAY / CONSERVE on each observation, choice p ∝ Born^(1/T₀) | squaring and cold give sharp decisions (QB-1 P3, QB-2 Q2) |

**The wiring:** the orchestrator only connects readings; it holds no decision rule.
1. Each step, the observation goes to D and to T.
2. D's event, if any, goes to M as a write.
3. M's reading (its Born weights) goes to T as a second input: every amplitude j of T is scaled by
   (1 + η_m·p^M_j). This is Born weights consumed as concentrations, the record's one clean benefit.
   **An empty memory (uniform) scales everything equally, so it has no effect after CONSERVE**; no
   rule is needed for "no memory yet".
4. T's readout is the choice.

**Settings:** η = 0.15 (as in QB-3), T₀ = 0.25 (cold), D's window k = 3 of M = 4. λ and η_m are tuned
on the development seeds over λ ∈ {0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.4} × η_m ∈ {0.05, 0.1, 0.2,
0.4, 0.8}.

## Arms (test seeds)

| Arm | What it is |
|---|---|
| `modular-quantum` | D → M (collapse) → T (cold Born) |
| `modular-classical` | the same wiring with classical modules: M is a plain register (one-hot, rewritten on each event), T is QB-3's greedy argmax tracker. Tuned on the same grid |
| `classical-champion` | QB-3's: monolithic greedy with gate, λ = 0.05 (0.815 in QB-3) |
| `monolith-A` | QB-3's candidate A (0.112) |
| `simple-cold-born` | T alone, with no D or M, λ tuned (QB-3: 0.640) |
| `−memory` | modular-quantum with the M → T link cut (η_m = 0) |
| `memory-only` | the choice is M's held item (Born readout of M), with no T |
| `−detector` | M is written on every observation instead of only on D's events |

## Predictions

1. **S1, Jesse's principle (small stacks orchestrated beat one monolith):** `modular-quantum` acc_resp
   ≥ `monolith-A` + 0.30, and ≥ `simple-cold-born`.
2. **S2, collapse is a near-perfect holder:** `modular-quantum` acc_early ≥ 0.75. That is above the
   classical champion's 0.716, because noise cannot touch a collapsed state; only D's rare false events
   can.
3. **S3, modular breaks the hold/forget trade-off:** both modular arms have acc_resp above
   `classical-champion`'s on the test seeds. The tracker is free to forget fast, because the memory
   holds.
4. **S4, law 5 (classical ties or wins):** `modular-classical` acc_resp ≥ `modular-quantum` − 0.02.
5. **S5, the memory carries the delay:** `−memory` has acc_early lower than `modular-quantum`'s by at
   least 0.15.
6. **S6, the detector is what makes the memory hold:** `−detector` has acc_early lower than
   `modular-quantum`'s by at least 0.15. Without D, noise rewrites the memory every step.
7. **S7, one job each:** `memory-only` has acc_early ≥ 0.70 (the memory alone holds the item), and
   its acc_resp is below `modular-quantum`'s (the memory cannot use the response evidence when a cue
   was missed).

**Kill criterion:** if 3 or more of S1–S7 fail, the modular design does not work as derived.

## Build

- New crate `qb4`. It reuses the mechanisms already tested: `State::project`, `born_weights`, `scale`,
  `qb-classical::{gate, temper, draw, argmax}`. The task generator is copied from `qb3`, so the worlds
  are byte-identical (a unit test checks this against `qb3`'s construction).
- **Unit tests:**
  - an empty memory leaves T's choices unchanged;
  - a written memory pulls T toward the held item;
  - D does not fire on pure noise more than 2% of steps on average.

## Results (2026-10-18, `evidence/qb-4/results.txt`)

All tests passed before the run:
- an empty memory leaves T's choices unchanged, and a written one pulls T;
- D fires on fewer than 2% of noise steps;
- the shared `qb-task` crate leaves QB-3's output byte-identical.

The champion reproduces QB-3's 0.815 exactly on these worlds.

| Arm (test seeds) | λ | η_m | acc_resp | acc_early | persev |
|---|---|---|---|---|---|
| **modular-quantum** (D → collapse memory → cold Born) | 0.05 | 0.05 | **0.877** | **0.840** | 0.028 |
| **modular-classical** (D → register → greedy) | 0.05 | 0.05 | **0.887** | **0.871** | 0.021 |
| classical champion (QB-3 monolith) | 0.05 | — | 0.815 | 0.716 | 0.049 |
| simple cold `born` (T alone) | 0.02 | — | 0.640 | 0.356 | 0.209 |
| −memory (link cut) | 0.05 | 0 | 0.523 | 0.303 | 0.060 |
| memory-only (M's item as the choice) | — | — | 0.842 | 0.816 | 0.018 |
| −detector (M written every step) | 0.05 | 0.05 | 0.625 | 0.378 | 0.057 |
| monolith A (QB-3, identical worlds) | 0.05 | — | 0.112 | — | — |

| # | Result | What was measured |
|---|---|---|
| S1 small stacks orchestrated beat one monolith | MEASURED | 0.877 against the monolith's 0.112 (+0.765), and above simple cold `born` (0.640) |
| S2 collapse is a near-perfect holder | MEASURED | acc_early 0.840 (bar 0.75; champion 0.716) |
| S3 modular breaks the hold/forget trade-off | MEASURED | quantum 0.877 and classical 0.887, both above the champion's 0.815 |
| S4 law 5, classical ties or wins | MEASURED | modular-classical 0.887 against modular-quantum 0.877 |
| S5 the memory carries the delay | MEASURED | acc_early 0.840 against 0.303 with the link cut |
| S6 the detector makes the memory hold | MEASURED | acc_early 0.840 against 0.378 without D |
| S7 one job each | MEASURED | memory-only acc_early 0.816; its acc_resp 0.842 is below the full system's 0.877 |

**7 of 7 hold; the kill criterion (3) does not fire.**

**A caveat on record:** η_m was tuned to 0.05, the smallest value on its grid. The best link
strength may be below the grid, so 0.877 and 0.887 are lower bounds for these designs.

### What QB-4 shows

1. **Jesse's principle holds on this task.** The same kinds of pieces, as small one-job stacks wired by
   readings, go from **0.112 as a monolith to 0.877 orchestrated**, and beat the tuned classical
   monolith (0.815).
2. **The orchestration is what beats the classical monolith, in either physics.** Classical modules
   wired the same way score 0.887. Law 5 holds once more: the gain comes from the *architecture*
   (separate holding from tracking), not from quantum maths. That is exactly the point of the
   principle.
3. **The character that was bad for tracking is ideal for holding.** Collapse, the "perseveration" of
   QB-1, is a near-perfect memory: acc_early 0.840. The memory module alone (0.842) beats the classical
   monolith. Each piece's personality is useful once it has the one job that fits it.
4. **Every module is load-bearing for its own job:**
   - without the memory link, early accuracy falls 0.840 → 0.303;
   - without the detector, noise rewrites the memory and it falls to 0.378;
   - without the tracker, the system can't recover a missed cue (0.842 against 0.877).

### Next

- Extend η_m below 0.05 (the grid edge).
- Chaos per module, one module at a time, as Rule 5 asks, now that each module is off the floor.
- A harder task where the quantum characters, not only the architecture, would be expected to
  matter. For example, a memory that must hold several items: a collapse memory per item, against a
  classical register bank.
