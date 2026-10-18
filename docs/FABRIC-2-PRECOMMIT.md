# FABRIC-2: the web built from many small neurons

Status: PRECOMMITTED (2026-10-18), before any FABRIC-2 code.

## Jesse's direction

> "I view the web as one modular part of the whole, I think it should be built like the rest is built,
> many neurons working together to form the web, that is or will be one of many small complex parts."

FABRIC-1 made the web one central mechanism, and it failed on three gaps:
1. credit for agreeing;
2. averaging instead of selecting;
3. a vibration wired against itself.

Plus a declared all-to-all start. **FABRIC-2 builds the web as a population of small web-neurons,** each
a one-job stack, tested alone before wiring. The spider only routes.

**The world, seeds and nodes** are FABRIC-1's: QB-7's world (40% of cues missing); dev 0–15, test
100–131; QB-9's optimized nodes. **The bar:** the hand wiring, 0.776 (cued 0.908, missed 0.572).

## The web-neurons (each its own crate)

| Neuron | One job | Mechanism | Fixes |
|---|---|---|---|
| **link neuron** (`web-link-neuron`), on the MQ → TQ thread | sense its own contribution and keep its thread | starts **dormant** (g = 0). Each step, c = r_src[o] − r_sink[o] (did the memory predict the observation *better than the tracker it feeds*?), then g ← clamp(g + α·c, 0, 1)·(1 − δ). The tracker reads the memory with strength η_max·g | credit for agreeing (agreeing earns c = 0); the all-to-all start (threads grow from flow) |
| **mismatch neuron** (reuses `fabric-vibration`, which already is this function) | compare two readings that do *not* feed each other | an EMA of r_TE[o] − r_MQ[o], where TE is an **evidence-only tracker** with no memory input | the self-cancelling vibration |
| **perception neuron** (`web-perceive`) | hold a named, revisable hypothesis with its evidence, never a score | the hypothesis "the memory is stale" turns on when the mismatch level exceeds θ, and off when it falls below θ/2 or when the memory is rewritten by a new cue event. Its evidence is the level | — |
| **lane neurons** (`web-lane-latch`) | one lane wins and holds; the decision *selects* its reading | QB-7's autocatalytic latch over two lanes, with mutual inhibition. A change of hypothesis pulses the matching lane (2.0). DEC uses the **winning lane's reading only** | averaging |

**The lanes:**
- **fresh:** TQ fed by MQ through the link neuron;
- **stale:** chosen by measurement on the development seeds before the run, as whichever of these has
  the better missed-trial accuracy:
  - **TE**, the evidence-only tracker (classical, gain ×4, κ 1, ι 1, λ 0.1, no memory input);
  - **TC**, fed by MC (QB-9's classical comparator).

**The settings** (36 configurations, tuned on dev):
- θ ∈ {0.2, 0.4, 0.6};
- mismatch rate ρ ∈ {0.05, 0.1, 0.2} (level scale 0.1);
- link α ∈ {0.02, 0.05}, with δ 0.01;
- η_max ∈ {0.05, 0.1}.

**Then, per Jesse's rule, rounds on the web-neurons** (plain wins ties, kept only if better on test):
- **lane heat:** a thermal floor on the latch, ε ∈ {0.01, 0.05};
- **link amplification:** η_max × {2}.

## Stage A: each neuron alone (all before the wired run)

1. **A1, the link neuron alone:** fed a source that only agrees with its sink and a source that
   contributes when the sink is wrong, the contributing link's g ends above the agreeing one's, which
   stays at 0.
2. **A2, the lanes alone select:** the latch's winner switches on a pulse and holds without one, and
   DEC's reading equals one lane's reading exactly at every step (never a mix).
3. **A3, the perception neuron alone:** it turns on above θ, holds between θ/2 and θ (hysteresis),
   and turns off on a rewrite.
4. **A4, choosing the stale lane:** TE against TC + MC, by dev missed-trial accuracy.
5. **A5, the routing ceiling:** an oracle that routes by the true missed flag (fresh lane on cued
   trials, stale lane on missed ones), on test. This is reported as the ceiling of what routing can
   gain.

## Predictions for the wired web (test seeds)

1. **W3, perception senses staleness:** "stale" is on for a larger fraction of missed-trial response
   steps than of cued-trial ones, by at least 0.30.
2. **W4, lanes route by perception:** the stale lane is selected on at least 50% of missed-trial
   response steps, and the fresh lane on at least 70% of cued-trial ones.
3. **W5, the web is at least the hand wiring, and captures the routing:**
   - acc ≥ 0.776;
   - if the oracle's gain over 0.776 is at least 0.01, the web captures at least half of it;
   - if the oracle's gain is under 0.01, the second half is void (routing has nothing to gain), and
     reported so.
4. **W6, lane heat is kept** (collapse-like commitment needs heat to switch).
5. **W7, the link neuron senses contribution:** its mean g over cued-trial response steps is above its
   mean over missed-trial ones (the memory helps when fresh and not when stale).

Ablations reported: −perception (lanes never switch: fresh only), and −link learning (η fixed 0.05).

**Kill criterion:** if 3 or more of W3–W7 fail.

## Build

- New crates: `web-link-neuron`, `web-perceive`, `web-lane-latch`, and the runner `fabric2`, which
  reuses `fabric-vibration`, `qb-*` and `qb-task`.
- A1–A3 are unit tests. A4 and A5 run first in `fabric2`, and their values are printed before the
  wired results.

## Results (2026-10-18, `evidence/fabric-2/results.txt`, 3.6 s)

**Stage A (alone):**
- A1–A3 passed as unit tests: the link neuron stays dormant on an agreeing source and grows on a
  contributing one; the lanes switch on a pulse, hold, and select one lane only; perception shows
  hysteresis and is refuted by a rewrite.
- The wired runner reproduces the hand wiring exactly (0.776) when it is set to the fresh lane with a
  fixed link.

| Stage-A measurement | Result |
|---|---|
| A4 stale lane (dev, each lane alone) | **TE** (evidence only): missed **0.648**, cued 0.705. TC + MC: missed 0.617, cued 0.914. **TE chosen** |
| A5 oracle routing ceiling (test) | **0.796** (cued 0.908, missed 0.623): **+0.021** over the hand wiring, so routing has a real gain available |

**The wired web** (tuned on dev: θ 0.6, ρ 0.05, α 0.05, η_max 0.1):

| Round | Result |
|---|---|
| lane heat (0.01, 0.05) | dev ties exactly (0.7501); not kept |
| link amplification (η_max ×2) | dev 0.7501 → 0.7571; test 0.732 → 0.739; **kept** |

| Arm (test) | acc (cued / missed) |
|---|---|
| hand wiring | **0.776** (0.908 / 0.572) |
| oracle routing (ceiling) | 0.796 (0.908 / 0.623) |
| **web** | 0.739 (0.823 / 0.607) |
| web −perception (fresh lane only, learned link) | 0.745 (0.833 / 0.610) |
| web −link learning (fixed η 0.05) | 0.764 (0.888 / 0.573) |

On the web's response steps:
- "stale" is on for 0.152 of cued steps and 0.325 of missed ones;
- the stale lane is selected on 0.325 of missed steps, and the fresh lane on 0.848 of cued ones;
- the link's g averages 0.035 on cued steps and 0.027 on missed ones.

| # | Result | What was measured |
|---|---|---|
| W3 perception senses staleness (+0.30) | **FAILED** | 0.325 against 0.152 (+0.173) |
| W4 lanes route by perception | **FAILED** | stale lane on 0.325 of missed steps (bar 0.5); fresh lane on 0.848 of cued ones (bar 0.7, held) |
| W5 the web ≥ the hand wiring | **FAILED** | 0.739 against 0.776 |
| W6 lane heat kept | **FAILED** | ties exactly |
| W7 the link neuron senses contribution | MEASURED | g 0.035 cued against 0.027 missed |

**1 of 5 hold; the kill criterion (3) FIRES.**

### Diagnosis: which web-neurons worked, and which failed, each for one reason

1. **The lane neurons worked as designed.** The lane followed perception exactly: the stale lane's
   share on missed steps equals "stale"'s share, 0.325 = 0.325. Selection replaced averaging, which
   fixes one of FABRIC-1's gaps. Lane heat was inert because the latch is only ever switched by
   pulses.
2. **The link neuron starved the fresh lane, because its erosion was too fast for its contribution.**
   - The direction it sensed is right: g is higher when the memory is fresh (W7 held).
   - But with δ = 0.01 per step, erosion outruns the small contribution signal c, which is near 0
     whenever the memory and the tracker agree. So g settles near 0.03.
   - The fresh lane then loses most of its memory input: cued trials fall 0.908 → 0.823, the web's
     largest single loss. Fixing the link (−link learning) recovers 0.764.
   - This is the same timescale gap as QB-7's cascade: **the erosion rate must be slower than the
     contribution's rate.** The universum substrate found it first: its high-load regime needed
     erosion about 7× slower (compound-c). δ was fixed, not tuned, in the precommitment; that is a
     derivation gap.
3. **The perception neuron's sense is too weak.** The mismatch between TE and MQ discriminates
   stale from fresh only modestly (+0.17), and gives 15% false "stale" on cued trials. Its one
   reading cannot carry the hypothesis; it needs more evidence, as the PERCEPTION axiom expects of a
   named hypothesis: several readings, not one.

**What FABRIC-2 established for the next step:**
- routing has a real ceiling (+0.021);
- the right stale lane is the evidence-only tracker;
- lane neurons select correctly;
- the two web-neurons that need work are named, with one cause each: the link neuron's erosion
  timescale, and perception's evidence.
