# Synthetic brain: to-do list

Started 2026-10-18. Each item is precommitted before it is built, as usual.

## Standing principle (Jesse, 2026-10-24): every stack is a specialist for a hybrid

"Treat all stacks like we intended to use them for memory, a hybrid, use stacks for what they are
strongest for, so a stack that does exceptional in one area but poorly everywhere else can be used
in a hybrid set up. Like the brain, no one thing excels at everything."

- **Never write off a stack for losing overall.** Every write-up records its **niche**: what it is
  best at, with numbers, and the hybrid role it could fill.
- **Routing between specialists** (HYBRID-1 onward) is the integration layer.
- **The niches so far:**
  - thermal shape store: lasting memories that fade unless used;
  - phase recognition store: mass recognition;
  - 2-bit association store: recall from partial or noisy cues, and records in any direction;
  - raw/AtomDB: exact recall;
  - virial recall loop: the deepest clean recall below the tipping point;
  - ATTEND: redundant data at depth;
  - Wilson reading: drift-proof recall and perception;
  - topological ring: noise-proof bits and exact small counts;
  - string harmonics: parallel lanes;
  - entangled pairs: coordination with no message;
  - Zeno attention: holding a state.

## Next session (written 2026-10-24)

Where we stopped: the PHYS-POC precommit (`docs/PHYS-POC.md`) is committed, amended so each law is
built as its own stack, then chaos off/on, then amplification on/off. Its four function crates
(`qb-wilson`, `xy-relax`, `winding`, `wave-mode`) are built and tested. `phys1` (Zeno) is written
but not run. `phys2`–`phys4` have manifests only and are left out of the workspace for now.

1. [x] **Discovery: Jesse chose the four proposed probes (2026-10-24).** Was: "don't limit discovery to memory". Options:
   - open-ended observation of each law's stack;
   - probes Jesse picks;
   - the proposed four (Zeno blockade as a GATE, gauge-invariant perception, topological counting,
     parallel string lanes).
2. [x] **PHYS-POC done** (13 of 21; commit c1ea89e). All four laws are load-bearing for something; chaos was kept nowhere. Discovery used the four probes Jesse chose.
3. **Redo the three astrophysics laws properly:** each as its own stack, then chaos off/on, then
   amplification on/off.
   - Fix ASTRO-1 arm B's step size.
   - Virial in the real direction (bound → hotter), at K6's scale.
   - Jeans in a medium where it can matter.
4. **FOLD-8 follow-up:** a PERCEPTION gate, "store recoverable", so explaining away is never worse
   than plain.
5. **HYBRID-1 precommit:** route each memory to the store where it shines. Open question: should the
   router learn over time, or decide per memory?
6. **The next physics batch:**
   - quark confinement, holographic codes, T-duality, supersymmetric cancellation;
   - the black-hole set;
   - the rest of the astrophysics list.
7. **Add these to `docs/AXIOM-ATLAS.md`.**
8. **Small items:**
   - ENT-1 seed 15 stayed on the saddle;
   - ENT-1 distillation as amplification;
   - background agents get a hard time box.

## Now

- [x] **T6 THERMO-1** (`docs/T6-THERMO-1.md`): done (run 2, 3 of 6). There is an inverted U in both
  brains, through the memory's temperature. The quantum memory peaks colder (θ 0.02 against 0.05),
  and both reach 0.779.

## Next: the real thermal state (the Arc and the dongle)

- [ ] **LONG-1, the long light session:** tens of minutes at low duty, or cooling after a load.
  - Resolves the die's slow heat node and the **room's temperature** (BEAT-1's H4).
  - Needs a two-node trajectory-fit crate (`arc-rc2`) in place of the slope fit.
- [ ] **LOOP-1, the closed loop:** the Arc's real temperature becomes the brain's θ, and the brain's
  activity drives the Arc's load. This is T6 with a real thermometer, in both directions.
- [ ] **MESH-1, the mesh probe:** load different engines of the Arc and see which sensor answers
  first, and how late. Is the silicon one connected mesh?
  - The compute-load path is still open: the compute adapter step stopped.
  - Alternative: fills on different queue families (the copy engine against compute).
- [ ] **GPU-2:** GPU-1's exact binary64 capability running on the Arc itself, checked bit for bit
  against the CPU law.

- [ ] **BW-1, bandwidth to and on the Arc:** measure, then amplify the *effective* bandwidth.
  - **Measure:**
    - the raw PCIe link (it currently runs **Gen4 ×4**; the card supports ×8, and the slot or BIOS lane
      setting is Jesse's to check);
    - fills per engine and queue family;
    - overlapped against one-at-a-time submission (BEAT-1's fills ran 55 GB/s against the rated
      ~186 GB/s of VRAM).
  - **Amplify, in Jesse's generic sense:**
    - send recipes, not data (keys that expand on the card);
    - keep state resident, so only the ports cross;
    - compress, then regrow on the other side;
    - use every engine in parallel;
    - preload (the spiderweb bus).
  - Report bytes delivered per byte sent.

- [ ] **WAVE-1, atom antennas outside the thermodynamic state** (`docs/WAVE-1-ANTENNA.md`,
  precommitted).
  - A wave medium, tuned resonator receivers, and two channels multiplexed.
  - Then a bath dial that turns the wave antenna thermal. Simulated and receive-only.

- [x] **SCAR-1, memory as scars, recalled by taps: done** (run 5, 11 of 12; `docs/SCAR-1-PRECOMMIT.md`).
  - Jesse's C17 rebuilt from scratch: sealed traps, slow erosion, spectral-bridge recall.
  - Fresh 0.953; noise keeps memories alive (0.891 at t 20,000, against 0.062 quiet).
  - Folding 2 per region gives region-level recall 0.914.
  - **Next: FOLD-1**, deeper folding and which memory a folded region holds.
  - The original plan follows.
- **SCAR-1, memory as scars, recalled by taps** (Jesse's thermodynamic memory: the data is held
  as a change in the state, not as data).
  - Carve several patterns as scars (T3's carving inside T5's hosted medium), then recall each from a
    different port with a small tap.
  - **Measure:**
    - recall fidelity, and completion from partial cues;
    - interference between patterns, and decay over time;
    - **noise as the maintainer**, on and off (it maintains only where there is an activity-gated
      coupling state);
    - **the footprint:** bytes of state per recalled pattern against the pattern's own size, and the
      recalled size against the tap size (amplification);
    - how many distinct scars one region holds at a given noise level.
  - **Treat no conventional ceiling as given:** the limits are unknown, and this measures them.

- [x] **FOLD-1: done** (4 of 5; `docs/FOLD-1-PRECOMMIT.md`).
  - At 32 bands a region holds 2 memories cleanly (0.938, contents 0.996).
  - 64 bands lift 4 per region from 0.656 to 0.869. Folded memories live (0.812 at t 20,000).
  - FOLD-2 (the capacity law): sublinear, F* 2, 4, 4 at 32, 64, 128 bands.
  - FOLD-3 (a containment reading): better at moderate depth (128 bands at F 8: 0.609 → 0.797); capacity is set by band occupancy.
  - **Next folding levers:** sparser keys, and orthogonal (vector-symbolic) codes.
- [x] **FOLD-4: done** (4 of 10; `docs/FOLD-4-PRECOMMIT.md`). The best in the thermal state at a fixed 422 KB:
  - the shape code (8 sites × 6 bands) with cooling (q 0.0025) and gain × 2;
  - 128 memories fresh at 0.948 (3,300 bytes each), or 64 lasting at 0.914 after 20,000 steps.
  - Sparsity and chaos are not levers. Depth is now limited by age.
- [x] **FOLD-5: done** (5 of 6; `docs/FOLD-5-PRECOMMIT.md`). Outside the thermal state (the quantum base):
  - 256 per region, so 4,096 memories at 103 bytes each, at 0.947 with the phase-locked reading;
  - no maintenance needed, and heat only costs.
- [x] **FOLD-6: done** (4 of 5; `docs/FOLD-6-PRECOMMIT.md`). **Folding does not save data:**
  - storing bytes exactly takes 8 stored bytes per byte at best (1–2 bits per amplitude), which is
    8× raw and 19× xz; full precision takes 341×.
  - What it buys is damage tolerance (30% of the store destroyed, recall 1.000), association, and
    no growth.
  - Next: quantize the FOLD-5 store to 1–2 bits (3.3–6.6 KB in place of 422 KB, if the 4,096 memories survive it); then cue robustness.
- **FOLD-1, folding data smaller** (Jesse: "can we fold data to make it smaller").
  - Measure each form on the brain's own data (fields, scars, memories), for size against fidelity:
    1. lossless: entropy coding and delta encoding;
    2. quantization (bits matched to need: about 4.7 bits per synapse in real brains);
    3. sparsity;
    4. dimension folding (random projection, the hashing trick, low rank);
    5. superposition folding (vector symbolic architectures: bind, superpose, unbind = tap);
    6. recipes (procedural).
  - Pair it with SCAR-1: how many items fold into one region before recall fails.
  - **Jesse's lead (2026-10-18): folding the scars is likely the best.**
    - A scar takes no *extra* space: carving only changes values the medium already has.
    - A tap's recall happens in the same medium too, costing time and compute, not space.
    - **So scars spend capacity, not footprint.** Measure the capacity (distinct scars per region,
      unfolded against folded) and the recall cost.
- **Precision decision (Jesse, 2026-10-18):** keep 128-bit as the top precision for now. Move to the
  RTX card later, **once Jesse is sure it is safe**.

## Hardware notes (the Arc A380)

- An **ITX card**: very small, one fan, **powered from the motherboard slot only** (no PSU cable).
- **Its power cap is 43 W** (`power2_max` = 43,000,000 µW, averaged over 28 s).
  - BEAT-1's fill load peaked at 34.1 W (100 ms windows), about 79% of the cap.
  - Keep loads light and short. With a single fan, sustained runs heat it faster than a full-size card.
- Resizable BAR is on (an 8 GiB window). Mesa marks DG2 on the `xe` driver as "experimental".

## Later

- [x] **128-bit (binary128): the capability is built and validated on the CPU (GPU-1b, 7 of 7).** Using it in the brain is still to do.
  - Next: wherever a chaotic stack runs long. Original note follows.
- **128-bit (binary128) for the brain.** Jesse: "a great fit later down the road for a synthetic
  brain". It comes from the doubler applied again (`Wide` to 128-bit values and 256-bit products).
  - Uses:
    - long-lived state without drift;
    - colder decisions without T6's greedy thin rule;
    - faint memories that don't round to zero;
    - 64-vs-128 divergence as an edge-of-chaos gauge.
  - **Precision as an atom the engine selects:** promote only the parts that need it.
  - The check is against GCC's libquadmath through a small C helper, an existing independent
    implementation.
- [ ] **Compose T3's carving into the hosted base,** so that substitution between sources (QB-6's
  law) can be tested: the linear base only adds.
- [ ] **A symmetric exchange sweep:** alternate the even/odd order, composed over the bath, for
  rungs that need reciprocal signalling (T5's H2 finding).
- [ ] **T4 stimuli with a mirror margin,** not just exact chirality (T4 run 2's R2 and R3).
- [ ] **The safe memory corpus**, replacing "Mother", near the end of the project.
- [ ] **Bring in the training through episodic memories and identity:** its project is still to be
  located.
