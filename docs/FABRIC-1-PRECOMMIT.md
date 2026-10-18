# FABRIC-1: the spiderweb as the synthetic brain's wiring, first test

Status: PRECOMMITTED (2026-10-18), before any fabric code. The design follows `FABRIC-STUDY.md`.

## The question

Can the modules' wiring *emerge* the spiderweb's way, with threads that form where readings flow and
maintain themselves by use? And can such a fabric do something hand wiring cannot: route each moment
to the module that is currently good at it, by vibration?

**The world and seeds** are QB-7's (40% of cues missing; dev 0–15, test 100–131). **The nodes** are
QB-9's optimized ones, so each stack is optimized before the fabric goes on top.

## The nodes (small stacks, each already optimized)

| Node | What it is | Its reading (a distribution over the 16 options) |
|---|---|---|
| **MQ** | a memory column: QB-9's gate detector writing a quantum collapse memory, with heat (ε from the vibration, below) | its Born weights |
| **MC** | a memory column: the same detector writing a classical register with a floor (QB-9's comparator: 0.05) | the register |
| **TQ** | the quantum tracker (QB-9: T₀ 0.05, gain ×3, κ 1, ι 1, λ 0.05) | its tempered Born weights |
| **TC** | the classical tracker (QB-9's comparator: gain ×4, κ 1, ι 1, λ 0.05) | its weights, tempered at the same T₀ |
| **DEC** | the decision node | it draws the choice directly from its mixture of inputs. The trackers' readings are already tempered at T₀ 0.05, so it does not temper again (a clarification written before any run, so that the frozen hand-wired fabric reproduces QB-9 exactly) |

## The fabric (built from scratch as atom stacks; nothing declared strong)

1. **Ground level (local streets).**
   - **The candidate links:** MQ → TQ, MQ → TC, MC → TQ, MC → TC (a memory feeding a tracker), and
     MQ, MC, TQ, TC → DEC.
   - **All start equal.** A tracker reads its memory inputs as concentrations,
     1 + η_link·Σ g·r_mem[j]. DEC reads the mixture Σ g·r_source.
2. **Threads form by use (the substrate's maintain-or-erase law, `fabric-conductance`).**
   - Each link keeps a predictive score s: an EMA, at rate ρ, of n·r_source[o_{t+1}], meaning how
     much better than chance the reading it carried predicted the *next* observation.
   - A sink's input conductances are g_e = s_eᵞ / Σ sᵞ: competition among a sink's inputs, with γ
     amplifying the differences.
   - **Labels are never used; the signal is self-supervised.**
3. **Vibration (`fabric-vibration`).**
   - The mismatch v is an EMA of (r_TQ[o_t] − r_MQ[o_t]): the tracker predicting the world better
     than the memory means the memory is stale.
   - MQ's heat is ε_t = ε_max·clamp(v / 0.1, 0, 1), set by the vibration instead of fixed.
   - During the delay both predict at chance, so v ≈ 0 and the memory is not heated while it holds.
4. **Highway and ramps (`fabric-ramp`).**
   - A detector event is a salient signal. It is promoted to the highway and broadcast to both
     trackers, whose off-ramp scales the event's option by (1 + h).
   - With h = 0 it travels only the ground level, into the memories.
5. **The spider (`fabric-orchestrator`):** it routes readings and applies the fabric's settings. It
   holds no rule about which module to trust; trust is the conductances.

**The fabric's settings**, tuned on dev over 324 configurations:
- ρ ∈ {0.02, 0.05, 0.1, 0.2};
- γ ∈ {1, 2, 4};
- η_link ∈ {0.025, 0.05, 0.1};
- ε_max ∈ {0.02, 0.04, 0.08};
- h ∈ {0, 0.5, 1.0}.

## Arms (test seeds)

| Arm | What it is |
|---|---|
| `fabric` | the full fabric, tuned |
| `hand-wired` | QB-9's final quantum system (0.776), on identical worlds |
| `fabric-quantum-only` | MQ and TQ only, retuned |
| `fabric-classical-only` | MC and TC only, retuned |
| `−vibration` | the fabric with fixed heat ε 0.02 (QB-9's best fixed value) |
| `−highway` | the fabric with h = 0 |

## Predictions

1. **F1, threads emerge with the hand wiring's shape:** averaged over response steps, the trackers
   (TQ and TC) carry at least 60% of DEC's conductance. The decision learns to trust trackers, which
   are fed by memories, over raw memories.
2. **F2, the emergent fabric is at least as good as the hand wiring:** `fabric` test acc_resp ≥ 0.776.
3. **F3, routing by vibration:**
   - on missed-cue trials' response steps, DEC's conductance on TC is above its conductance on TQ;
   - on cued trials, TQ's is above TC's (QB-7's profiles: classical better when the memory is stale);
   - and `fabric` beats both single trackers' systems, the hand-wired 0.776 and the classical 0.775.
4. **F4, the vibration earns its place:** `−vibration` has acc_missed below `fabric`'s.
5. **F5, diversity pays:** `fabric` acc_resp is at least 0.005 above both `fabric-quantum-only` and
   `fabric-classical-only`. Mixing the two physics' profiles beats either alone.
6. **F6, the highway earns its place:** `−highway` has acc_cued at or below `fabric`'s.

**Kill criterion:** if 3 or more of F1–F6 fail, the spiderweb fabric does not work as derived at this
scale.

## Build (zero dependencies, one function per crate)

- `fabric-conductance`: link scores (EMA of predictive score) and competitive conductances.
- `fabric-vibration`: the mismatch EMA → a heat level.
- `fabric-ramp`: promotion of an event to a broadcast boost.
- `fabric-thread`: reads the strongest path from the conductances (reporting).
- `fabric1`: the spider and runner, reusing the node mechanics already tested (`qb-amplitude`,
  `qb-classical`, `qb-task`).

**Unit tests:**
- a link that predicts better gains conductance against one that does not;
- the conductances over a sink's inputs sum to 1;
- the vibration is 0 when both readings predict equally;
- with the fabric frozen at hand-wired weights (MQ → TQ only, TQ → DEC only, heat 0.02, h 0),
  `fabric1` reproduces QB-9's 0.776.

## Results (2026-10-18, `evidence/fabric-1/results.txt`, 43 s)

Tests passed before the run:
- frozen at QB-9's hand wiring, the fabric reproduces **0.776** exactly;
- every sink's conductances sum to 1.

The four fabric crates pass their own tests.

| Arm (tuned on dev) | ρ | γ | η_link | ε_max | h | test acc (cued / missed) |
|---|---|---|---|---|---|---|
| hand wiring (QB-9, frozen) | — | — | 0.05 | fixed 0.02 | 0 | **0.776** (0.908 / 0.572) |
| `fabric` | 0.05 | 4 | 0.025 | 0.02 | 1 | 0.719 (0.862 / 0.498) |
| `fabric-quantum-only` | 0.1 | 4 | 0.025 | 0.02 | 1 | 0.745 (0.902 / 0.502) |
| `fabric-classical-only` | 0.02 | 4 | 0.05 | 0.02 | 1 | 0.690 (0.800 / 0.519) |
| `−vibration` (fixed heat 0.02) | 0.05 | 4 | 0.025 | — | 1 | 0.695 (0.812 / 0.513) |
| `−highway` | 0.05 | 4 | 0.025 | 0.02 | 0 | 0.705 (0.834 / 0.505) |

**DEC's conductances on response steps (fabric, test):**

| | MQ | MC | TQ | TC |
|---|---|---|---|---|
| all | 0.261 | 0.113 | 0.318 | 0.308 |
| cued trials | 0.293 | 0.085 | 0.319 | 0.302 |
| missed trials | 0.207 | 0.154 | 0.318 | 0.320 |

**The trackers' memory inputs:** MQ 0.669 and MC 0.331, for both trackers. **The strongest thread into
DEC:** MQ → DEC (0.261).

| # | Result | What was measured |
|---|---|---|
| F1 threads with the hand wiring's shape | MEASURED | the trackers carry 0.627 of DEC (bar 0.60) |
| F2 at least the hand wiring | **FAILED** | 0.719 against 0.776 |
| F3 routing by vibration | **FAILED** | missed: TC 0.320 against TQ 0.318, i.e. no routing. Accuracy is below both single systems |
| F4 the vibration earns its place | **FAILED** | missed 0.498 against 0.513 with fixed heat |
| F5 diversity pays | **FAILED** | the full fabric (0.719) is below quantum-only (0.745) |
| F6 the highway earns its place | MEASURED | cued 0.862 against 0.834 |

**2 of 6 hold; the kill criterion (3) FIRES.** The fabric as built does not work at this scale.

### Why: three gaps in what the fabric was handed (diagnosis, not new claims)

None of these is a failure of the nodes, which reproduce exactly when frozen. All three are in the
fabric's learning signal and readout, the "feedback and wiring" class of the plateau-diagnosis skill.

1. **The learning signal cannot tell agreeing sources apart.**
   - A link is scored by n·r[next]. The memories and trackers usually point at the *same* option, so
     every link pointing there earns the *same* score.
   - The scores separate only in the brief moments when sources disagree, so even γ = 4 leaves the
     conductances near uniform (0.11–0.32).
   - The signal measured "predicts the world", not "**adds** prediction the others lack".
2. **DEC averages; it does not select.**
   - A conductance-weighted mixture gives the stale memories about 37% of the vote on missed trials.
     The hand wiring never let a memory vote; memories only nudge a tracker (η 0.05).
   - The record already named this character: the dual-cache bond "averages the branches, never
     selects" (`QUANTUM-BASE-CHARACTERS.md`, classical hooks). A mixture readout is that averaging
     character, and it should have been predicted.
3. **The vibration was wired against itself.**
   - It compared TQ's prediction with MQ's, but TQ *consumes* MQ, so they agree, the mismatch stays
     near 0, and the stale memory is not heated.
   - Fixed heat beat it on missed trials (0.513 against 0.498). A mismatch signal needs two readings
     that do not feed each other.

Two more, structural:
- **Declaring every candidate link at equal strength is the all-to-all start the spiderweb rules
  out** ("do not declare the fabric upfront"). Threads should *form* from flow, not be pruned from a
  full mesh, which is QB-3's monolith risk again.
- **Adding nodes hurt** (quantum-only 0.745 > full 0.719). With an averaging readout, every extra
  source dilutes the best one.

### For FABRIC-2 (to precommit if Jesse wants to continue)

- **Credit by unique contribution:** score a link by r_source[o] − mix[o], its prediction *beyond*
  what the sink already had. A source that only agrees earns nothing (PREDICT/REVISE as prediction
  error).
- **Select at DEC:** attend to the strongest thread, with a cold choice over the conductances (ATTEND),
  instead of a mixture.
- **A vibration from decoupled readings:** compare the memory with evidence it does not shape, for
  example a tracker with no memory input.
- **Grow threads:** start with no links. A link is created when readings flow along it, and erodes
  without use.

### The scoreboard

The hand wiring (0.776) is still the best system in this world. The fabric's first form loses to it,
and the reasons are specific and testable.
