# Study: the spiderweb bus as the synthetic brain's wiring

Status: STUDY (2026-10-18). Nothing is built; this asks whether the idea is sound before building.

Jesse: "study and contemplate whether my spiderweb bus can be implemented in this as well, perhaps not
as a bus but sort of like a neuro network. And if the idea is sound, creating it from scratch using
atom stacks."

## The gap it would fill

QB-4 to QB-8 proved Jesse's principle: small one-job stacks, orchestrated, beat one monolith (0.112 →
0.877 → 0.768 in the harder world). **But I wired them by hand:** detector → memory → tracker, fixed
links, fixed strengths. A brain does not get its wiring from a designer: its pathways strengthen by use
and fade by disuse. The spiderweb's central claim is the same: **threads are not declared; they form
as messages flow, like desire paths.** So the question is whether the modules' wiring can *emerge*.

## The mapping: spiderweb → brain → what already exists

| Spiderweb (bus) | Brain | Already in Jesse's work |
|---|---|---|
| node | a neuron or column: a small one-job stack | QB-4's D, M, T modules |
| ground level (local streets): node-to-node, slow, flexible | local recurrent circuits | QB-4's hand-wired links |
| **threads form as messages flow (desire paths)** | Hebbian plasticity: paths used together strengthen | **the substrate's "maintain or erase" law:** pathways persist under traffic and erode with disuse (CHANGES C12, C16, C21–C22) |
| intersection plus transfer policy | a hub where pathways cross, and a gate on hopping | the GATE root; the persistence filter |
| on-ramp (promotion to the fast layer) | salience: what wins attention is broadcast | Born squaring and cold readout amplify the leader (QB-1, QB-2, QB-6) |
| highway (fast, parallel, directional) | long-range projections, a global broadcast ("global workspace") | not yet built |
| off-ramp (demotion to local traffic) | targeted delivery to the modules that need it | not yet built |
| **vibrations** (vertical, horizontal, cross-cutting) | **neuromodulation** (global: arousal, surprise), lateral inhibition (horizontal), feedback (vertical) | QB-8: heat in the memory helps when it is stale. A surprise broadcast could set that heat *when needed*, instead of always |
| backpressure | inhibition, refractory periods, an energy budget | CONSERVE; product inhibition (QB-7) |
| preloading (prefetch the next node's data) | **prediction**: predictive coding, readiness for the expected input | the PREDICT root |
| thickness (parallel bundles of the same layer) | population coding, redundancy | `coder-perceive-population`; QB-7's two trackers with different strengths |
| the spider (L3 orchestration) | the executive | the cognitive engine's executive / lanes co-engines |

**The strongest link:** the spiderweb's emergent threads and the substrate's traffic-carved pathways are
the *same law* seen from two projects:
- the bus says paths form where messages flow;
- the substrate measured that paths persist where flow maintains them, and erode where it stops.

That law is already measured, so the fabric would not be invented; it would be composed from it.

## Is it sound? Yes, with four conditions from the evidence

1. **Do not let it become a monolith again** (QB-3). The fabric must be *grown*: each node is optimized
   alone first, as now, and the fabric only learns *which links carry readings, and how strongly*.
2. **Pass readings, not amplitudes, between nodes.** QB-3 showed that phase-carrying pieces interfere
   pairwise. Inside a node, the quantum base keeps its characters. Between nodes, messages should be
   classical concentrations, as the QB-4 link already is. This is a design choice to test, not
   assume.
3. **The learning signal must come from inside, not from labels.** The fabric cannot see the true
   target. A self-supervised signal fits the spiderweb's "vibration" and Jesse's PREDICT/REVISE roots:
   a thread is reinforced when the reading it carried *predicted the next evidence*, and decays when it
   did not.
4. **No rules in the nodes** (Rule 1). Transfer, ramps and modulation must be mechanisms that learn
   (conductances, gates, thresholds from measured noise), with any unavoidable thin policy in the
   spider only, labelled as policy.

**The honest risk:** law 5 says a classical design has tied every comparison so far. The fabric's value
would not be "quantum wins". It would be that **the system finds its own wiring**, and that it can
route between modules with different strengths. QB-7 found exactly such a pair: the quantum tracker
is better when the memory is right, and the classical one is better when the memory is stale. Hand
wiring cannot choose between them per trial. A fabric that routes by vibration could.

## A first test (FABRIC-1, to precommit if Jesse agrees)

**The nodes:**
- QB-8's detector and memory;
- both trackers: the quantum cold one, and the classical fast-forgetting one;
- a decision node;
- later, a mismatch node.

**The ground level:** every possible link starts at a small, equal conductance. No link is declared
strong.

**Threads:**
- Each link's conductance is reinforced when the reading it carried predicted the next observation.
- It erodes otherwise, by the substrate's maintain-or-erase law.
- The decision node reads its inputs weighted by conductance (mass action).

**Vibration:**
- A surprise signal (prediction against evidence) is broadcast to every node.
- The memory's heat is set from it, instead of being fixed (QB-8).

**Ramps and highway:** a reading above its node's measured noise floor is promoted to all nodes;
otherwise it travels only its local threads.

**Predictions, to be fixed before code:**
- **F1:** the learned conductances rediscover the hand wiring as the strongest thread
  (detector → memory → tracker).
- **F2:** the emergent fabric reaches at least the hand-wired QB-8 system's score (0.768).
- **F3:** with both trackers present, the fabric shifts weight to the classical tracker on
  missed-cue trials, and beats either tracker alone.
- **F4:** with vibration off (a fixed memory heat), missed-cue trials get worse.
- **F5:** law 5, a classical-only fabric is checked against it.

**Built from scratch as atom stacks** (zero dependencies, one function per crate):
- `fabric-conductance`: link state, reinforce and erode;
- `fabric-thread`: paths read from the conductances;
- `fabric-ramp`: promotion by noise floor;
- `fabric-vibration`: surprise broadcast;
- `fabric-orchestrator`: the spider; routing only, with any policy labelled.

## What should come first

The pending retest round on QB-8's stack, which comes first:
1. fix M's check;
2. go colder on T₀;
3. retest amplification;
4. a fair classical comparator.

Then the fabric's nodes start from optimized stacks, which is his "optimize each stack before the next
goes on top".
