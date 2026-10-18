# Synthetic brain: to-do list

Started 2026-10-18. Each item is precommitted before it is built, as usual.

## Now

- [ ] **T6 THERMO-1** (`docs/T6-THERMO-1.md`): running on 8 threads, with the resumable job log.
  Record the results, analyse the misses, update memory, then commit.

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

## Later

- [ ] **128-bit (binary128) for the brain.** Jesse: "a great fit later down the road for a synthetic
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
