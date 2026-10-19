# Synthetic brain: to-do list

Started 2026-10-18. Each item is precommitted before it is built, as usual.

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

- [ ] **SCAR-1, memory as scars, recalled by taps** (Jesse's thermodynamic memory: the data is held
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

- [ ] **FOLD-1, folding data smaller** (Jesse: "can we fold data to make it smaller").
  - Measure each form on the brain's own data (fields, scars, memories), for size against fidelity:
    1. lossless: entropy coding and delta encoding;
    2. quantization (bits matched to need: about 4.7 bits per synapse in real brains);
    3. sparsity;
    4. dimension folding (random projection, the hashing trick, low rank);
    5. superposition folding (vector symbolic architectures: bind, superpose, unbind = tap);
    6. recipes (procedural).
  - Pair it with SCAR-1: how many items fold into one region before recall fails.
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
