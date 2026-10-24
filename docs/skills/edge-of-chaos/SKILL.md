---
name: edge-of-chaos
description: Jesse's chaos technique — the most important step in building Atom stacks. Every stack or function is optimized with chaos (off vs on; the plain stack is the bar) BEFORE the next stack is built on top of it; in a platform built without it, apply it wherever a stack or function fails or works poorly. Chaos = a small deterministic chaos-theory perturbation applied every step inside ONE stack or function; the stack's own check selects; the winner is kept. Tests take minutes. Use whenever you build, fix or tune any stack, engine, function or pipeline step in Jesse's projects; whenever he says chaos, controlled chaos, edge of chaos, "a bit of chaos", "add chaos", "chaos to the top", "chaos at the end", "see what emerges", "self-optimize", "with and without", logistic/Hénon, anneal, temperature knob; and BEFORE rewriting anything that underperforms. Also use when a stack stalls, abstains, gets stuck, forgets over many steps, or a generator's output plateaus.
---

# Edge of chaos — Jesse's technique

## Why it matters to Atom

An Atom stack is built from inert, deterministic atoms. It has no weights and no training, so
it does exactly what it was built to do, even when that isn't good enough. Chaos is how such a
stack improves itself:

- **chaos** supplies small variations near its current behaviour;
- **the stack's own law** decides which variations survive;
- **the survivors are kept.**

That's why he calls it "the main ingredient in most stacks". His words:

> "when I added a bit of chaos, is what ended up making it work the way it does now"
> (atom-harness, 2026-08-21)

In atom-harness with chaos off, the stack abstained on every case. With chaos on, it answered
every case, 1024/1024 correct over 32 held-out seeds.

**It is a building step, not a tuning step.**

> "normally it's done at the beginning of building a platform, not after, every stack gets
> optimized before moving to the next stack and so on, it's why I told you this is the most
> important step, otherwise we are basically starting from scratch again" (2026-10-04)

- Each stack is optimized at the edge before the next one goes on top. Stacks built on
  un-optimized stacks inherit and compound their weaknesses.
- In a platform built without this step, don't restart. Use it "where things aren't working or
  working well", one stack or function at a time.

"Chaos" means chaos theory ("No, I mean chaos, as in chaos theory", 2026-09-03). It is always
tried before rewriting code or stacks:

> "edge of chaos, not destructive chaos, can self optimize a stack alone" (2026-10-04)

Other uses of the word are not this technique:
- a stress test;
- fault injection;
- privacy scrambling;
- "promote chaos", which is security work only.

## The protocol (do exactly this)

1. **Pick ONE stack or function.**
   - While building: the stack you just built.
   - On an existing platform: the unit that fails or works poorly.

   Never a chain: "I use chaos to optimize one stack/function, not an entire chain of
   stacks."
2. **Give it its own check** and judge it on that unit's output alone.
   - Use the stack's own law where it has one: its checker, acceptance rule, erosion, or a
     threshold over trials.
   - Otherwise build a check from data the stack legitimately has. For example, the
     task's visible tests, decomposed down to this function.
   - **Validate the check first:** an independent oracle must reproduce it 100%. A wrong
     check optimizes the wrong thing.
3. **Off = the plain unit. Its score is the bar.**

   > "you test with it on and off. The base is the bar"

   Run exactly two arms. Add no extra control arms (seed-only, keep-best-only,
   perturbation-only) unless he asks.
4. **On = the same unit with chaos inside it.**
   - A seeded, chaos-theory perturbation: small, and preserving meaning or the mean.
   - Applied per element, step or variant.
   - It fires when the plain result isn't perfect, or doesn't **strictly** beat the
     current best.
   - The check selects. **Plain wins ties.** Invalid candidates are refused (fail closed).
5. **Keep it short:** 3–5 seeds and roughly 20 calls, in **minutes, not hours**. A long test
   is aimed at a chain; shrink it.
6. **Decide.**
   - Keep chaos if "on" beats the bar.
   - **Keep the winner** (persistence) as the unit's new base.
   - If the unit isn't perfect yet, run another round from the kept best, feeding it its own
     first failure, with the same rule: the plain repair is the bar.
   - If "on" does nothing, first check the four ingredients and the position: "maybe it's
     just in the wrong position". Then, if it still does nothing, "remove them for now".
7. **Move to the next stack or function** and repeat.

**Tuning toward the edge** is optional and happens inside the "on" arm. Push the strength
(temperature, number of variants or dice, amplitude) "toward the upper limits … but nothing
too strong". Stop where the unit's own "too much" signal appears:
- false assertions;
- wrong answers;
- scrambled identity;
- degenerate output;
- destroyed code.

Use short grids.

## Worked example: the template for generate-and-check (a-rag-atom, 2026-10-04)

The unit was **one rendered function**: `update_rtt_on_ack`, the RTT part of QUIC's
OnAckReceived. The model rendered it from the engine-composed procedure, with thinking off.

**The check:** 434 vectors, one per ACK in the task's 40 visible sequences. An oracle written
from the RFC text matched all 434.

**The arms:**
- **Off:** one plain render.
- **On:** if the plain render isn't perfect, 3 variants, each with 3 seeded presentation dice
  plus its own sampling seed. The dice:
  - cases first;
  - trace the cases;
  - steps as words;
  - reverse the steps;
  - contract first;
  - one step at a time.

  The check picks; plain wins ties.

**Results:** on beat the bar on **5/5 seeds**, mean **335.8 vs 237.6** of 434 (+41%), using 20
renders in 5.4 minutes. Seed 4242 went from 130 to 414. The winning dice most often included
cases first, trace, and reverse.

**Contrast:** the same idea applied to the whole compose → render → check → repair chain was a
1.5-hour test with a muddy answer. It was stopped, and it was the wrong scope.

## The four ingredients (why it works; check these when a test shows nothing)

Across his builds, chaos helped only when all four held together. Every failure lacked at
least one (see `references/evidence.md`).

1. **Small, and preserving the mean, norm or meaning.**
   - Use a bounded factor such as `1 + α(u − ū)`, where ū is the source's invariant mean.
   - Or use content-preserving dice for a generator.
   - A mean shift is a hidden second input: it once produced a fake 34% effect.
2. **Every step, access, trial or variant — never once,** and per element.
   - One uniform factor across the whole state is invisible to a rank or threshold readout.
   - BUILD-32/33 applied chaos once at the end, and all 19 arms tied.
3. **The stack's own law selects.** This is what does the optimizing. Examples:
   - carving plus disuse erosion;
   - a novelty-gated sum over annealed trials;
   - Metropolis acceptance plus cooling;
   - a "k in a row" gate;
   - a checker with strict selection where plain wins ties.

   No selector means chaos only reshuffles.
4. **The survivors persist:** carved coupling, reinforced confidence, committed laws, the kept
   best.

Reference result, atom-harness controlled-chaos v5:

| arm | coverage | false assertions |
|---|---|---|
| off | 0 | — |
| raw, strength 0.4–0.55 | about 0.87 | 0 |
| raw, strength 0.7 / 0.85 / 1.0 | — | 12 / 18 / 24 ("too much") |
| annealed | full | 0 |

## Choosing the source, position and dial

- **Source:**
  - logistic map at r = 4 (invariant mean ½);
  - or Hénon (1.4, 0.3), folded by x mod 1, mean about 0.486, so centre on the measured mean.
  - Avoid the tent map: it dies in f64.
  - Seed everything, and index by (item, step, trial).
  - The **edge is the amplitude or coupling**, not the map's r. He runs r = 4 with a small
    amplitude.
- **Position.** Any order is allowed; when unsure, try another position.
  - early ("the first two primitives can benefit from chaos … then build on top of that
    behavior");
  - before a threshold, where it acts as a filter;
  - **never after a threshold**, where it acts as an eraser;
  - at the end of a stack;
  - "to the top": every layer or axis;
  - several positions at once.
- **On a quantum base,** temperature is the knob ("temperature acts like a knob for quantum,
  kind of like chaos does for classical"), and classical chaos does not go first.
- **Holding the edge at runtime:** the governor.
  - It reheats on coherent surprise and cools on incoherent noise.
  - It keeps acceptance in the band 0.20–0.48 under a chaos budget of 1.25.
  - "Construction-time only" is agent drift.

Patterns for each kind of stack are in `references/patterns.md`:
- voting and fusion;
- search and walks;
- generate-and-check;
- memory and fields;
- runtime regulation;
- quantum stacks.

## Order of operations when something underperforms

**Chaos first** (this skill) → **compose from outside** (the compose-from-outside and
sealed-engine-composition skills) → only then consider rewriting. Never patch a sealed engine
to add chaos: compose the perturbation and the selector around it.

## Watch-outs

When the plain arm and every variant tie for two attempts, stop and use the `plateau-diagnosis`
skill: every plateau on record was a gap in what the unit was handed, never weak chaos.

- **A plateau in a single failure class means missing input, not weak chaos.** Chaos can only
  re-present what the unit is given. On 2026-10-04 the RTT function plateaued at 423/434, and
  all 11 failures were the ack-delay cap. The world's cap node had no edge onward, so the
  composed procedure never carried the rule. Fix the input (here, a world edge), then rerun on
  against off.
- **A ceiling shared by the plain render and every variant usually points to your check or
  contract, not the model.** On 2026-10-04 every timer render stopped at exactly 1204/1212.
  All 8 failures involved a packet sent at t = 0, which my contract had called "0 = none".
  Inspect the shared failures before anything else.
- **Repair feedback must show the most frequent failure classes, not just the first failure in
  order.** On 2026-10-04 the on_ack repair was fed a rare timer mismatch. Meanwhile 60 of its 68
  failures were one crash, so every variant tied the base. Rank the classes, and show one example
  of each of the top three.
- **A unit that re-implements a neighbour's work is a scoping gap in what it was handed, and no
  amount of chaos on either unit repairs it.** On 2026-10-06 the general orchestrator plateaued
  at 255/300 for eight attempts: 28 of 40 cases failed on one field computed inside a unit that
  had been given its neighbour's procedure lines too (its cone held 47 lines, 36 of them another
  function's), so it inlined that work with a bug and nothing ever called the real unit. Give
  each unit only the lines nearest its own node (partition the composed cones by nearest
  function node; ties shared) and let the neighbours be callees. Check for this whenever one
  unit's code writes fields that another unit owns.
- **Greedy chaos is an oxymoron.** A generator needs sampling at fixed seeds or presentation
  dice.
- **The off arm must be the plain unit itself:** the same prompt or input and the same seed
  as the on arm's plain candidate.
- **Maintainers interfere.** Noise plus coupling did worse than either alone. Test
  combinations as their own runs.
- **The cost of the edge depends on load.** Scrambling appeared only under heavy crowding.
- **Per-read dissipation erases build-up.** Noise maintains memory only where there is an
  activity-gated coupling state.
- **A perturbation can help one workload and hurt another.** Report results per workload.
- **One dial can carry two effects** (v5 strength also turned on coupling). Keep it bundled,
  his way.

## Mistakes I made (don't repeat them)

- **S1-C:** a strength sweep on a single walk with no selector and nothing kept. That was a
  bad test, not a verdict.
- **S1-R:** chaos on a whole chain, three arms of 6 rounds, about 1.5 hours. Wrong scope,
  too long, and too many arms.
- **Treating chaos as tuning after the build,** instead of a step in building each stack.
- **Believing agent-written notes over his words.**

## Kit

`scripts/chaos_kit.py` and `scripts/chaos_kit.rs` are zero-dependency and deterministic. Their
`unit()` values are bit-identical across the two languages. Copy what you need into the
stack's own module. Contents:

- **the main entry point** (Python): `on_off(run, seeds)`;
- **the generator selector:** `chaos_select`;
- **sources:** `unit`, `factor`, `logistic_lyapunov`;
- **other selectors:** `phase_mix` (v5), `anneal`, `Governor`;
- **memory maintainer:** `sparse_noise`;
- **optional tuning inside "on":** `sweep`.

Self-tests:

```bash
python3 ~/.claude/skills/edge-of-chaos/scripts/chaos_kit.py
```

```bash
rustc --edition 2021 --test ~/.claude/skills/edge-of-chaos/scripts/chaos_kit.rs -o /tmp/chaos_kit_test && /tmp/chaos_kit_test
```

## Vocabulary: his term → what to build

| His term | Build this |
|---|---|
| chaos, controlled chaos, "a bit of chaos" | a seeded chaotic-map perturbation, small, preserving meaning or the mean, per element |
| edge of chaos, "upper limits … nothing too strong" | the strongest setting before the unit's "too much" signal |
| "tunes", "self-optimize", "see what emerges" | perturbation plus the stack's own selector plus persistence |
| "with and without", "on and off", "the base is the bar" | two arms; keep chaos only if on beats the plain unit |
| "to the top" / "at the end" | every layer or axis / the terminal stage |
| "first two primitives … build on top" | early chaos for discovery, then build on the emergent behaviour |
| "wrong position" | move it |
| temperature (quantum), hot/cold | the annealing temperature as the dial |
| noise as maintainer, "traffic" | sparse kicks plus activity-gated erosion of a coupling state |

## Evidence discipline

Rank evidence as **his words > his code and measured results > agent-written notes**. His
Obsidian and AtomDB-MCP notes are kept by AI agents: a newer note isn't ground truth, and a
change is often drift. Follow his words and code, flag the drift, and ask him when a term is
ambiguous. His dated quotes, and where chaos helped and failed, are in
`references/evidence.md`.

## Evidence from 2026-10-23/24 (synthetic brain, about 20 stacks)
- **Chaos was inert wherever the plain step is already optimal for the stack's own selector**
  (quantizer + readback, least-squares scales, invariant readings): it tied by construction.
- **Kept only where a real selector judged a noisy or misjudging step:** the grid-cell angle read at
  high noise (+0.007), the merge-tree descent (+0.026 to +0.047, exploration of one extra branch),
  and at selection-luck size elsewhere.
- **Rule 5 needs two maps of different dynamics** (logistic and Hénon) at two positions, with
  none/single/both controls. "Chaos: none" is never an acceptable plan.
