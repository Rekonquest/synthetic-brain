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
