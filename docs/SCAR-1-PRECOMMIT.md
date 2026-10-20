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

## Run 2: PRECOMMITTED (2026-10-20), before any run-2 code. Jesse's C17 design, from scratch

Jesse left the choice to me. It is built from scratch, stack by stack, carrying his proven laws:
- one region per memory;
- a compound-c-like balance;
- region-level recall.

**The medium:** 3D, 24 × 16 × 16, κ 0.3, c₀ 0.2, c_min 0.05.
- **Keys:** the input face x = 0 has 256 *bands* (sites). A memory's key is 16 random bands.
- **Regions:** the output face x = 23 is split into 16 patches of 4 × 4. Memory m is written to
  patch m, so each memory has its own region.

**Writing (`scar-write`, unchanged):**
- one episode holds the key's bands at +1 and its patch's 16 sites at −1 (γ 1), for 300 steps while
  carving;
- 4 rounds, with the memories interleaved round-robin (rehearsal);
- f_ref = κ / 23.

**The balance (compound-c-like):**
- formation μ 0.05, with erosion ν 2·10⁻⁴: 250× slower than formation, against T3's 10×;
- **the new stack `thermo-budget`** (synaptic scaling, borrowed from neuroscience): after each
  carving step, each site's total incident conductance is capped at a budget B (1.5 = 1.25 × 6c₀) by
  scaling its edges (each edge takes the smaller of its two ends' factors, clamped at c_min). Used
  routes then grow at the expense of unused ones. This is the competition the pilot lacked;
- T3's other kept settings: p 3, frozen chaos 0.3, writing heat T 0.005.

**Recall (`scar-tap`, the new absorbed-current reading):**
- the cue's bands are held at +1;
- every patch site is a cold absorber (held at 0);
- after 300 steps, each patch's absorbed current is summed (region-level reading);
- **the recalled memory** is the patch with the most current (top-1).

**The measure:** top-1 accuracy over the stored memories. Chance is 1/16.

**Seeds:** keys from dev 0–3 and test 10–17. **The cue**, unless stated otherwise, is half the key
(8 of 16 bands).

### Predictions (test seeds)

1. **S1, the control:** in the uncarved medium, top-1 ≤ 0.15.
2. **S2, capacity, one per region:** P ∈ {1, 2, 4, 8, 16}; top-1 ≥ 0.9 at **P 16** (every region
   used). The curve is reported.
3. **S3, completion:** at P 16, a cue of 4 of 16 bands gives top-1 ≥ 0.6.
4. **S4, noisy cue:** at P 16, half the key plus 8 random non-key bands gives top-1 ≥ 0.8.
5. **S5, the budget is load-bearing:** with `thermo-budget` off, the P-16 top-1 falls by ≥ 0.2 (no
   competition: saturation, as in the pilot).
6. **S6, fading:** P 16, then 10,000 idle steps at T 0 (erosion e^{−2}). Top-1 falls below 0.5.
7. **S7, noise keeps them alive:** the same idle at T 0.005. Top-1 ends ≥ 0.2 above the T-0 idle.
8. **S8, zero extra footprint:** the medium's bytes are unchanged by writing.

### Rounds (dev: the P-16 top-1, then the mean absorbed-current margin; kept only if better on test; plain wins ties)

- **Amplification off (p 1):**
  - **Prediction S9:** not kept.
- **Chaos off:**
  - **Prediction S10:** not kept.
- **Cold writing (T 0):**
  - **Prediction S11:** not kept. T3's inverted U puts T 0.005 above cold.

CPU only, 8 threads, a few minutes.

### Run 2, first attempt: VOID (a geometry bug; `evidence/scar-1-run2/results-VOID-periodic-wrap.txt`)

Everything came out at chance (P 1 0.25, P 16 0.094), and the correct region's current was far below
the others (margin −6.7). The lattice is **periodic**, so the output face at x = 23 is one wrap-step
from the bands at x = 0:
- every band sat beside the patch with its own (y, z);
- carving and recall both went across the wrap, regardless of the memory.

Like T3's run 1, the wiring made the comparison vacuous, so none of its verdicts stand. **The repair:**
the output face moves to x = 12, halfway round, 12 steps from the bands either way, with f_ref = κ / 12.
The predictions are unchanged.

## Run 2: results (2026-10-20, `evidence/scar-1-run2/results.txt`, 4 min 39 s on 8 threads)

| # | Result | Measured |
|---|---|---|
| S1 control | MEASURED | 0.078 |
| S2 capacity | **FAILED** | **P 1 1.000**, P 2 0.562, P 4 0.188, P 8 0.141, P 16 0.078 |
| S3 completion | FAILED | 0.086 |
| S4 noisy cue | FAILED | 0.070 |
| S5 the budget is load-bearing | FAILED | off 0.086, on 0.078 (both at chance at P 16) |
| S6 fading | holds, but vacuously (nothing to fade at P 16) | |
| S7 noise keeps them alive | FAILED | |
| S8 zero extra footprint | MEASURED | |
| S9–S11 rounds | all at chance; cold writing "kept" by tie-break margin only | |

**5 of 11 hold.** **One scar now recalls perfectly (P 1 = 1.000).** Two or more do not.

### Diagnosis (`diagnostic-picks.txt`; after the run, not a prediction)

- **Every region's entrance is carved:** a summed conductance of 4.9–7.5 into each of the 16 regions
  at P 16.
- **But recall picks region 0 or 2 for almost every cue:** at P 16 the picks were 0, 2, 0, 2, 2, 1,
  2, 0, 5, 2, 3, 0, 1, 2, 0, 0.
- **The carving built one shared network**, the slime-mould effect: many source–sink pairs grow a
  single connected web, a Steiner-tree-like network. From any key the current reaches every region
  through the web, and geometry, not the memory, picks the winner.

**What is missing is inhibition.** While memory m is written, nothing says "not the other regions".
**Run 3** adds one change: during memory m's episodes, the other 15 regions are held at the key's
potential (+1). No current is carved from key m toward them, and erosion plus the budget prune the
shared routes. This is *lateral inhibition* from neuroscience, the "not this" signal of contrastive
Hebbian learning.

## Run 3: PRECOMMITTED (2026-10-20), before any run-3 code

Run 3 is identical to run 2 except for the writing: during memory m's episode, the patches of every
other region are held at +1 (inhibited).

**Predictions S1–S11 as in run 2, with these re-derived:**
- **S2:** P 16 top-1 ≥ 0.9.
- **S5:** inhibition is the load-bearing change. Run 2 (no inhibition) is the arm against it, and run
  3 must beat it at P 16 by ≥ 0.3.

## Run 3: results (2026-10-20, `evidence/scar-1-run3/results.txt`, 4 min 18 s)

**S2 capacity:** P 1 **0.000**, P 2 0.000, P 4 0.031, P 8 0.031, P 16 0.109. **4 of 11 hold**, and
it is worse than run 2.

**My inhibition design was wrong.** Holding the other 15 regions at +1 did not make them "not this";
it made them **sources**:
- current flowed from all of them into the region being written;
- that carved region-to-region channels along the output face and starved the key's own route, so
  even a single memory is lost.

Run 2 had already left the other regions *free* (neutral). **So run 2's merging was not a missing
inhibition. It comes from the architecture:** every memory shares one input face and one medium, so
their routes must share one network.

### What Jesse's C17 actually did (CHANGES.md, C17 design and amendment)

- Each memory's key spectrum was **carved inside its own region**, so the scars are local and no
  network is shared between memories.
- Recall re-derived the cue and **compared it against each region's carved key** through the
  *spectral bridge* routing law (`battery.rs`). The top region won.
- So the tap reads each region's scar, and the bridge does the matching.

**Run 4 should follow that:**
- local carving per region;
- recall by matching the cue against what each region's scars hold.

The bridge law's exact form is Jesse's (`battery.rs`), so read it, or ask him, before building.

## Run 4: PRECOMMITTED (2026-10-20), before any run-4 code. C17 rebuilt from scratch

Jesse confirmed it ("as long as it works and stays true to the architecture"): a C17 memory is a
**band spectrum held in its own region**, kept by carved material and the noise law, and recalled by
matching the cue against each region's spectrum with his **spectral bridge**:
match = 1 − ½ · L1(a/Σa, b/Σb), from `universum-substrate/src/field.rs`.

### The stack (one function per crate; T5's multi-species medium carries the bands)

**The medium:**
- 32 band fields (species) on a 3D lattice of 20 × 20 × 4;
- 16 regions of 4 × 4 × 4, on a 4 × 4 grid at spacing 5, so a one-site **moat** separates regions;
- moat sites are uncarved, so they are leaky and mobile, and act as absorbers between regions.

**The four new crates:**

| crate | its one job |
|---|---|
| `spec-step` | one medium step: band-blind diffusion (T1's exchange) at κ·w_e with w_e = 1 − ½(m_i + m_j)·τ, then a leak e ← e·(1 − δ(1 − τ·m_s)). Carved material **traps**: low mobility and low leak |
| `spec-trap` | the material's maintain-or-erase: m ← m + μ_m·g(E_s/E_ref)·(1 − m) − ν_m·m, with E_s the site's total band energy and g T3's x^p/(1 + x^p) |
| `spec-stir` | the noise law as sparse kicks: each step, each site with probability q gains ε in one random band (keyed, deterministic) |
| `spec-bridge` | Jesse's spectral bridge, as a crate |

**Settings:** κ 0.2, δ 0.01, τ 0.9, μ_m 0.02, ν_m 0.002, E_ref 1, p 3; stir q 0.02, ε 0.05; write
gain 0.01 per step.

**Keys:** 6 active bands of 32, with random weights, normalised.

**Writing:** memory m's key spectrum, times the gain, is injected at every site of region m for 250
steps. The memories are written one after another (C17's write protocol), then the medium drains
with no input.
- the **stirred** arm runs the noise during the write and the drain;
- the **quiet** arm has no noise.

**Recall:** each region's spectrum is the sum of its sites' band energies. The cue is matched against
every region by the bridge, and the top-1 region is the recalled one. *Self-match* is the bridge
between a region's spectrum and its own key.

**Reads:** at drain t = 0, 250 and 2,000.

**Seeds:** keys from dev 0–3 and test 10–17. The cue is the exact key unless stated otherwise (C17
re-derives the same cue).

### Predictions (test seeds, P 16, one memory per region)

1. **R1, the control:** with nothing written (stirred), top-1 ≤ 0.15.
2. **R2, fresh recall:** at drain t = 0, top-1 ≥ 0.95.
3. **R3, maintain-or-erase:** the quiet arm at t = 2,000 falls at least 0.3 below fresh. (C17's quiet
   collapse was 3%.)
4. **R4, noise maintains (C16):** the stirred arm at t = 2,000 is ≥ 0.75, and ≥ 0.3 above quiet.
5. **R5, completion:** at t = 250 stirred, a cue of 3 of the key's 6 bands gives ≥ 0.8.
6. **R6, noisy cue:** at t = 250 stirred, the key plus 30% flat noise gives ≥ 0.9.
7. **R7, folding, 2 memories per region** (32 keys; each region written with two, one after the
   other):
   - **(a)** at t = 250 stirred, region-level top-1 (each cue finds its region) ≥ 0.8;
   - **(b)** the mean self-match falls by ≥ 0.2 from the one-per-region value. A region's spectrum
     becomes a mixture, as Jesse's law "competitive, not superposable" says.
8. **R8, zero extra footprint:** the medium's bytes are unchanged by writing.

### Rounds (dev: stirred t = 2,000 top-1, then mean self-match; kept only if better on test; plain wins ties)

- **Amplification:** write gain × 2.
  - **Prediction R9:** kept. More held energy, and material that saturates sooner.
- **Chaos:** frozen per-site jitter of μ_m, α 0.3.
  - **Prediction R10:** not kept.
- **Heat:** stir × 2 (q 0.04).
  - **Prediction R11:** not kept. A flat noise spectrum dilutes the profiles.

The cold side of hot/cold is R3/R4's quiet arm. CPU only, small, about a minute.

## Run 4: results (2026-10-20, `evidence/scar-1-run4/results.txt`, 81 s)

**4 of 12 hold.** Fresh recall is at chance: stirred 0.094, quiet 0.125. Self-match is 0.19, folding
0.070, and the rounds are all vacuous at chance.

**Diagnosis** (`diagnostic-regions.txt`; after the run):
- **The last-written region is perfect** (self-match 0.987, energy 14.6).
- **Every earlier region has lost its own memory.** Its energy (0.5–3.6) is mostly spillover from
  later writes, and self-match is 0.14–0.46.

My precommitted physics was too weak in two places:
1. **Regions were not sealed.** The mobility w_e = 1 − ½(m_i + m_j)τ only slows flow where *both*
   ends are carved, so the edge from a carved region into its moat kept half its mobility.
2. **The trap faded.** E_ref 1 sat above the held energy (about 0.9 per site), so formation was weak,
   and erosion ν 0.002 is far faster than C17's compound-c (5·10⁻⁵).

**The pilot** (`pilot-run5-sealed.txt`; labelled) uses:
- the **sealed** mobility, w_e = (1 − τm_i)(1 − τm_j), so *either* carved end slows the flow;
- τ 0.99, E_ref 0.3, ν 5·10⁻⁵ (C17's coupling erosion).

| arm | drain t 0 | t 250 | t 2,000 |
|---|---|---|---|
| quiet | 1.000 | 1.000 | 0.992 |
| stirred | 0.953 | 0.953 | 0.953 |

**The memories hold.** At ν 5·10⁻⁵ the material lives about 20,000 steps, so maintain-or-erase needs
a much longer drain to show.

## Run 5: PRECOMMITTED (2026-10-20), before the run

Run 5 is run 4 with:
- the **sealed law** (`spec-step::step_sealed`), τ 0.99, E_ref 0.3, ν 5·10⁻⁵;
- reads at drain t = 0, 250, 2,000 and **20,000**.

All else is unchanged: the gain, μ_m, the stir, the keys, the seeds and the cues.

**Predictions** (test seeds):
1. **R1, the control:** ≤ 0.15.
2. **R2, fresh:** ≥ 0.95.
3. **R3, maintain-or-erase:** quiet at t 20,000 ≤ fresh − 0.3 (the material's lifetime is about
   1/ν).
4. **R4, noise maintains:** stirred at t 20,000 ≥ 0.75, and ≥ quiet + 0.3.
5. **R5, completion** (3 of 6 bands, t 250): ≥ 0.8.
6. **R6, noisy cue** (+30% flat): ≥ 0.9.
7. **R7, folding** (2 per region, t 250):
   - **(a)** region top-1 ≥ 0.8;
   - **(b)** self-match falls by ≥ 0.2 from one per region.
8. **R8, zero extra footprint.**
9. **Rounds** (dev: stirred t 20,000 top-1, then self-match):
   - **R9:** write gain × 2 kept;
   - **R10:** frozen μ chaos not kept;
   - **R11:** stir × 2 not kept.
