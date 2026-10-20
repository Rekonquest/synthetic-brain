# SCAR-1: memory as scars, recalled by taps

Status: PRECOMMITTED (2026-10-18), before any code.

## Jesse's idea

In the thermodynamic state, memories are held as **scars** (the change the flow leaves in the
medium) and recalled through **taps**:
- **footprint:** a scar takes no extra space, because carving only changes values the medium already
  has;
- **recall:** happens in the same medium;
- **cost:** scars spend *capacity*, not footprint;
- **noise keeps them alive.**

**Folding the scars** (many in one region) is likely the best path. SCAR-1 measures how far this goes,
treating no conventional ceiling as given.

## The stack (one function per crate; T3's laws and kept settings carried up)

**The medium:** T2's bath with T3's per-edge conductances (`exchange_edges`, `thermo-carve`).
- 3D, side 16, κ 0.3, μ 0.02, ν 0.002 (erosion 10× slower than maintenance), c_min 0.05, c₀ 0.2;
- T3's kept settings: amplification p 3, frozen chaos 0.3 on μ, and heat T 0.005 while writing.

**Cells:** a pool of 48 sites at random positions, at least 3 apart. A **memory** is an *assembly* of 6
cells drawn from the pool. Assemblies may overlap, as they would in a brain.

**`scar-write`** (one job: carve an assembly into the medium):
- an episode holds one member (the hub) at +1 and the other five at −1, through damping γ 1 (T3's
  drive), for 100 steps while carving;
- the hub rotates through the members, two rounds, so 12 episodes per assembly;
- the assemblies are interleaved round-robin;
- f_ref = κ / (the mean hub-to-member distance);
- the field is reset to 0 at each episode's start.

**`scar-tap`** (one job: recall by tap):
- the conductances are frozen (no carving during recall) and the field is reset;
- the cue cells are held at +1 (γ 1), and every other site leaks towards 0 at γ_L 0.01, at T 0;
- after 300 steps the field is read at the pool's cells;
- the recalled set is the top (6 − cue size) non-cue cells by field.

**Fidelity** is the fraction of the recalled set that belongs to the assembly. Chance is about
3/45 ≈ 0.07.

**`scar1` is the orchestrator.** Seeds (the pool and assemblies): dev 0–3, test 10–17. Each memory is
recalled from its first 3 members, unless stated otherwise.

## Predictions (test seeds; the mean over memories and seeds)

1. **S1, the control:** in the uncarved medium (c₀ everywhere), recall fidelity ≤ 0.2.
2. **S2, one scar recalls:** P = 1, a cue of 3 of 6, fidelity ≥ 0.9.
3. **S3, folding (many scars in one medium):**
   - with P ∈ {1, 2, 4, 8, 16} memories carved into the same medium, fidelity stays ≥ 0.8 up to P 4;
   - it falls by P 16 (memory is competitive): fidelity at P 16 < fidelity at P 1;
   - the **capacity**, the largest P with fidelity ≥ 0.8, is reported.
4. **S4, pattern completion:** at P 1, a cue of 1 of 6 gives fidelity ≥ 0.5.
5. **S5, scars fade without use:** P 4 written, then 3,000 idle steps (no drive, T 0). Fidelity falls
   to ≤ half its fresh value (erosion e^{−νt} = e^{−6}).
6. **S6, noise keeps them alive (Jesse's claim):** the same idle, with the bath at T ∈ {0.005, 0.02}.
   At the better T, fidelity after the idle is ≥ 0.1 above the T 0 idle.
7. **S7, footprint and cost:**
   - **footprint:** the medium's bytes are the same before and after writing (zero extra by
     construction);
   - **capacity** (S3) gives the medium's bytes per memory held;
   - **recall cost:** the steps for the recalled set to settle (the last change in the top set) are
     reported, not predicted.

## Optimization round (on the dev capacity score: the mean fidelity at P ∈ {1, 4, 16}; kept only if better on test; plain wins ties)

Plain is the setting carried up from T3 (p 3, frozen chaos 0.3, T 0.005 while writing).

- **Amplification off:** p 1.
  - **Prediction S8:** plain (p 3) is kept. A selective carving gives sharper, less overlapping
    threads.
- **Chaos off:** α 0.
  - **Prediction S9:** plain is kept. T3's frozen heterogeneity breaks ties between equal paths.
- **Cold writing:** T 0 while writing.
  - **Prediction S10:** plain is kept. T3's inverted U puts T 0.005 above cold.

**Room and system:** CPU only, 8 threads, a few minutes. No GPU, and no system settings touched.

## Run 1: results (2026-10-18, `evidence/scar-1/results.txt`, 2 min 10 s on 8 threads)

| # | Result | Measured |
|---|---|---|
| S1 control (uncarved) ≤ 0.2 | MEASURED | 0.076 (chance ≈ 0.067) |
| S2 one scar recalls ≥ 0.9 | **FAILED** | **0.167** |
| S3 folding | **FAILED** | P 1 0.167, P 2 0.146, P 4 0.104, P 8 0.052, P 16 0.086; capacity 0 |
| S4 completion from 1 of 6 ≥ 0.5 | **FAILED** | 0.225 |
| S5 scars fade without use | **FAILED** | idle T 0: 0.083 against a fresh 0.104 (there was little to fade) |
| S6 noise keeps them alive | **FAILED** | 0.083 at every T |
| S7 zero extra footprint | MEASURED | 98,304 bytes before and after writing; recall settles in about 45 steps |
| S8–S10 the rounds | hold by the rule, but **vacuous**: every arm is near chance (plain dev 0.052) | |

**5 of 10 hold, and the core failed:** recall is barely above chance, even for one memory.

### Diagnosis (after the run, not predictions)

**Diagnostic 1** (`diagnostic-run1.txt`):
- **(a) The writing carved something:** after P 1, 85 edges have c > 0.5 (2 above 0.9).
- **(b) But even ideal scars read badly:** a hand-made ring of c 1 threads through the six members
  recalls only 1 of 3 missing members under the precommitted tap.
- **So the tap's reading is a bottleneck.** It ranks cells by their *field*, and a cell that a hot
  thread merely passes *beside* is as hot as one at the thread's end. With 48 cells in the medium,
  about four bystanders sit beside any assembly's threads, about as many as the members sought.

**Diagnostic 2** (`diagnostic-run1-absorbing-reading.txt`): the reading changed to Jesse's antenna
receiver, in which every non-cue cell is a cold absorber (held at 0) and the score is the current it
absorbs through its own edges.

| case | recalled |
|---|---|
| the ideal ring, on an eroded background | **2 of 3** |
| the carved scars (P 1), test seeds 10–17 | **8 / 24** with 100-step episodes; 8 / 24 with 300-step episodes |

That is double run 1's 0.167. The longer episodes carved *fewer* strong edges (71).

**So there are two gaps:**
1. **The reading should be absorbed current, not field.** That is the receiver from TA1, and the
   scar's terminus is what conducts into a member.
2. **The writing does not carve connecting threads.** A hub driving five sinks at once, with T3's
   selective p 3, carves stubs near the hub and the sinks rather than the paths between them. T3
   needed about 3,000 steps of one steady pair to carve one 3D thread; here each pair gets a few
   hundred steps, shared with four others.

**Run 2 should:**
- carve **pair by pair**, as T3 did (one held pair at a time, f_ref = κ / its distance, every pair
  of the assembly, a T3-like duration);
- **read absorbed current.**

## Pilot for run 2 (2026-10-20; labelled, not a prediction; `evidence/scar-1/pilot-run2.txt`)

**The setup:**
- one memory per seed, carved pair by pair as a nearest-neighbour ring of 6 threads (1,500 steps
  each, the T3 way);
- read by absorbed current;
- test seeds 10–17.

| erosion ν | recalled | strong edges (c > 0.5) after writing |
|---|---|---|
| T3's 0.002 | **2 / 24** | 5–47: only the last thread survives |
| 0.002 / 100 | **2 / 24** | **all 12,288**: everything saturates |

**The carving law alone cannot store several threads.**
- **Fast erosion:** an unused scar lives about 1/ν = 500 steps, so each thread erodes while the
  next is carved.
- **Slow erosion:** the drive's diffuse flux and the writing heat (T 0.005) maintain *every* edge, so
  the contrast vanishes.
- **What is missing is competition**, a budget that makes threads compete.

### Jesse's own solution, found in universum-substrate (CHANGES.md, C17)

Mother's 106 memories were carved and recalled at 100% top-1 through t = 2,000, under three
conditions SCAR-1 broke:
1. **One region per memory.** Memory is competitive, so each pattern has its own write position (a
   text-key → region), not one shared space.
2. **Compound-c:** strong coupling formation (0.285) with very slow coupling erosion (about 7× slower
   than the default, 5·10⁻⁵), plus tuned dissipation and radiation and the noise law. This is
   validated at 120 facts.
3. **Region-aggregated recall:** cue → route → read the region as a whole, not single sites. The
   per-site identity there was weak too (46–51%).

**So run 2 should follow his proven design**, not my overlapping-assemblies design:
- regions keyed per memory;
- a compound-c-like balance (strong formation, slow erosion, dissipation as the competition);
- region-level reading.

Folding (more than one memory per region) is then the measured question on top.
