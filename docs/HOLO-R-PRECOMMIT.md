# HOLO-R: holographic memory redone under Rules 5, 6 and 7 (a self-healing store)

Status: PRECOMMITTED (2026-10-24), before any HOLO-R code.

Jesse (2026-10-24): "Did you try chaos and amplification with the holography experiments? And did
you try adding cognitive, neuroscience, chemistry etc. Remember my complexity rules." I had not.
PHYS2 Q4 had no chaos, its "amplification" was only a lighter load, and it was physics alone. Under
his rules a single-law stack is a control, not the principal candidate. HOLO-R builds the principal
candidate:
- **Rule 6:** genuine mechanisms from every required domain, each ablated;
- **Rule 7:** compositional complexity, with one to three declared orders, every control run;
- **Rule 5:** two chaos mechanisms of different dynamics at two positions, with no-chaos,
  single-chaos and multi-chaos controls.

## The task: lifelong damage

- A 2-bit phase store of D 4,096 in 8 regions of 512 holds N key → byte memories (N ∈ {192, 256}).
- **Three damage episodes:** each erases 2 regions not erased before (their amplitudes set to random
  phases).
- After each episode the store may perceive, decode and repair before the next.
- **The frozen metric:** recall (the exact value) after each episode, with the final episode the
  headline.
- **Hard invariant:** no stage may read the true values. Repair uses only what the store recalls.

Test seeds 10–17; every threshold is chosen on dev seeds 0–3.

## The mechanisms (state · invariant · interaction · falsifier · why)

| domain | mechanism | state | invariant | interacts with | falsifier (ablation) | why chosen |
|---|---|---|---|---|---|---|
| **quantum** (Rule 4 base) | phase superposition, 2-bit phase quantization | complex amplitudes | unit modulus after quantization | every write and read | n/a: the base | FOLD-5 to 9 measured its capacity and phase-locked reading |
| **physics** | holography: every memory spread over all regions | the layout | a memory's energy is equal in every region | erasure masks, repair targets | **local layout** (each memory in one region) | PHYS2 Q4: 0.96 against 0.75 after losing a region |
| **cognition** | PERCEPTION of damage, then GATE, then REVISE | per region, the reading h_r (the mean best-code margin of 64 stored keys read from that region alone) and the named hypothesis "region r damaged" (h_r < κ·median h, κ on dev), citing h_r | hypotheses are retractable and cite their readings | chooses the mask (GATE) and the regions to rebuild (REVISE: dissolve, then rewrite) | **random mask** of the same size | the class separation; damage must be perceived, not told |
| **information theory** | erasure decoding: read only through amplitudes perceived healthy | the mask | the masked read uses the healthy subset only | uses perception; feeds the decoder | **no mask** (read everything, erased noise included) | an erasure costs half what an error does |
| **mechanics / thermodynamics** | relaxation decoding: ASTRO-2R's self-set-temperature explaining-away loop (6 rounds) on the masked store, CONSERVE-scaled | per-item readings and temperatures | the least-squares scale conserves the store's mass | refines what replay writes | **plain single read** | ASTRO-2R: the best recall stack (0.904; 0.966 with rounds × 2) |
| **neuroscience** | replay (consolidation in sleep): the decoded memories are rewritten into the dissolved regions | the rebuilt regions | only recalled values are replayed (the hard invariant) | needs the decoder's output and perception's target regions | **no replay** (no repair) | the hippocampus repairs and consolidates by replay |
| **biology** (added before code; Jesse: "and try biological axioms") | kinetic proofreading (Hopfield 1974): a memory is replayed only if two independent decodes agree, one from each of two disjoint halves of the healthy regions, so errors are cut multiplicatively before anything is written | a per-memory pass or fail | nothing is written that failed a check | gates replay | **no proofreading** (replay every decode) | replay could otherwise write wrong values back and make an error permanent; biochemistry's answer is to discriminate twice |
| **chemistry** | Le Chatelier: each memory's replay strength is proportional to its displacement from equilibrium, its confidence deficit 1 − m_i/m_ref (floored at 0.2), so the least-held memories are restored hardest | replay weights | the total replayed weight is fixed | shapes replay | **uniform replay weight** | a system pushed from equilibrium shifts to oppose the push |

## The two declared orders (Rule 2 and Rule 7)

- **C1, perceive first:** perceive → mask → relax-decode → proofread → replay (Le Chatelier).
- **C2, decode first:** relax-decode on the whole store → perceive damage from each region's
  residual against the decode → mask → relax-decode again → proofread → replay.

## Chaos (Rule 5): two mechanisms with different dynamics, at two positions

- **χ1, the logistic map** (r = 4, mean ½) **at perception:** each region's threshold κ is scaled by
  1 + 0.5(u − ½), 3 variants. The selector is the decode's total margin; plain wins ties.
- **χ2, the Hénon map** (1.4, 0.3; folded mod 1, centred on its measured mean) **at repair:** each
  memory's replay weight is scaled by 1 + 0.5(h − h̄), 3 variants. The selector is the total readback
  margin after repair; plain wins ties.
- **Controls:** no chaos, χ1 alone, χ2 alone, χ1 + χ2.
- **Promotion** only on strict improvement of final recall over no-chaos and the single-chaos
  controls (Rule 5).

## Amplification

The decoder's rounds × 2 (12): more of the same relaxation.

## The arms (all on matched inputs)

1. Plain holographic store (the PHYS2 Q4 control).
2. Local layout (control).
3. C1, the principal candidate.
4. C2.
5. Ablations of C1, one domain each: random mask, no mask, plain read, no replay, no
   proofreading, uniform replay weight, local layout under C1.
6. Chaos: χ1, χ2, χ1 + χ2 on C1.
7. Amplification on C1.

The bounded count is 15 arms × 2 N × 8 test seeds, plus dev.

## Predictions

1. **H1:** C1's final recall (after 3 episodes, 6 of 8 regions lost in all, each healed in between)
   is ≥ 0.9 at N 192 and ≥ 0.8 at N 256. The plain holographic store is ≤ 0.3 at both (6 of 8 regions
   gone with no repair), and the local layout ≤ 0.3.
2. **H2, load-bearing domains:** removing perception (random mask), erasure decoding, the relaxation
   decoder, replay, or proofreading each lowers C1's final recall by ≥ 0.03. Without proofreading,
   wrong values written in episode 1 compound in episodes 2 and 3. **The Le Chatelier weighting ties**
   uniform replay (±0.02), because every memory loses the same regions, so the deficits are similar.
3. **H3:** C1 ≥ C2 (perceiving first avoids decoding through erased noise).
4. **H4:** no chaos arm is promoted (each ties or loses).
5. **H5:** the decoder's rounds × 2 raise C1's episode-1 recall at N 256 by ≥ 0.01.

A new orchestrator crate, `holor`. CPU, 8 threads, a few minutes.

## Amendment before code: repair and healing laws from different domains (Jesse, 2026-10-24)

"And try repair/healing axioms from different domains." The repair stage gets three candidate laws
(Rule 2: one to three, each with its reason). Each runs inside C1, with biology's proofreading gate
and chemistry's Le Chatelier rate law unchanged.

| law (domain) | mechanism | invariant | falsifier | why |
|---|---|---|---|---|
| **replay** (neuroscience; C1 as declared) | one pass: decode, proofread, rewrite the damaged regions | only proofread values are written | no replay | hippocampal replay consolidates and repairs |
| **autocatalysis** (chemistry) | repair in 3 cycles per episode: each cycle's healed regions improve the next decode, so more memories pass proofreading and are written; the product catalyses its own production | the same gate in every cycle | one cycle (= replay) | a reaction whose product speeds it up runs to completion where a single pass stalls |
| **annealing** (physics, materials) | 3 cycles with a cooling temperature. In early cycles, memories that fail proofreading are written as soft mixtures of their likely values (Boltzmann at T), hedging instead of omitting; T halves each cycle; in the last cycle only proofread values are written | the final cycle writes only proofread values | autocatalysis (hard writes only) | heating then slow cooling lets a lattice settle its defects instead of freezing them in |

**Predictions:**
- **H6:** autocatalysis ≥ replay + 0.02 in final recall at N 256 (proofreading pass rates rise
  across cycles).
- **H7:** annealing ties autocatalysis (±0.02): soft writes help the decode but add crosstalk.

**The arms** gain 2 (autocatalysis, annealing); the bounded count is 17.

## Results (2026-10-24, `evidence/holo-r/results.txt`, 963 s on 8 threads)

Dev chose κ 0.8 by C1's final recall (dev 0.055, 0.055, 0.074, 0.097: already a sign C1 was not
healing).

| recall after episodes 1 / 2 / 3 | N 192 | N 256 |
|---|---|---|
| plain holographic (control) | 0.863 / 0.445 / 0.077 | 0.710 / 0.329 / 0.054 |
| local layout (control) | 0.742 / 0.495 / 0.247 | 0.705 / 0.470 / 0.234 |
| C1 (perceive first) | 0.979 / 0.483 / 0.107 | 0.872 / 0.323 / 0.070 |
| **C2 (decode first)** | **1.000 / 0.994 / 0.939** | **1.000 / 0.564 / 0.282** |
| C1 ablations, all seven domains | 0.867–0.979 / 0.360–0.538 / 0.064–0.252 | 0.728–0.872 / 0.255–0.486 / 0.054–0.236 |
| C1 + chaos 1 (logistic, perception) | 0.965 / 0.500 / 0.198 | 0.888 / 0.275 / 0.061 |
| C1 + chaos 2 (Hénon, repair) | 0.979 / 0.485 / 0.113 | 0.872 / 0.324 / 0.072 |
| C1 + chaos 1 + 2 | 0.965 / 0.515 / 0.199 | 0.888 / 0.278 / 0.064 |
| C1, decoder rounds × 2 | 0.977 / 0.476 / 0.105 | 0.871 / 0.323 / 0.066 |
| C1, autocatalysis | 0.979 / 0.483 / 0.107 | 0.872 / 0.323 / 0.070 |
| C1, annealing | 0.979 / 0.514 / 0.128 | 0.872 / 0.342 / 0.085 |

**1 of 7 hold** (H7, a tie between two repair laws on a stack that was not healing).

### The cause, measured (`evidence/holo-r/diagnostic-perception.txt`; after the run, labelled)

In episode 1, against the 2 truly erased regions (4 dev seeds, 8 erased regions each N):

| perception | N 192: erased found / healthy flagged | N 256 |
|---|---|---|
| C1, perceive first (each region probed alone) | 2 / 8, 1 | 1 / 8, 0 |
| **C2, residual against the whole-store decode** | **8 / 8, 0** | **8 / 8, 0** |

- **C1's perception was blind.** Read through one region alone (512 amplitudes for 192–256
  memories), an erased region's margins look almost like a healthy one's (seed 0, N 192: 0.0077
  erased against 0.0106–0.0121 healthy), so κ could not separate them.
- Its repair therefore rebuilt the wrong regions, or none, and every C1-based comparison (all seven
  ablations, the repair laws, chaos, amplification) measures variations of a stack that was not
  healing. Those comparisons are **inconclusive**, not negative.
- **C2 perceives damage the way a whole system does: as disagreement between a part and the decode
  of everything.** That reading found every erased region with no false alarms.

### What it settles

- **A self-healing holographic store works** (C2, the declared second order): **after losing 6 of
  its 8 regions over three hits it still recalls 0.939 at N 192**, where the same store without
  healing falls to 0.077 and the local layout to 0.247. At N 256 it heals the first hit fully
  (0.564 after the second, 0.282 after the third), so the load limits how much it can rebuild.
- **The order mattered more than any single domain:** the same mechanisms in another order
  (C1) failed.
- **Next (HOLO-R2, a new precommit):** run the per-domain ablations, the repair laws (replay,
  autocatalysis, annealing), both chaos maps and the amplification **on C2**. Also give C1 a
  whole-system perception, to separate perception from order.

**Niche:** C2 is the self-healing memory for lifelong damage, at moderate load.
