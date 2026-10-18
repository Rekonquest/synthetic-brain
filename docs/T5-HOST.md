# T5: the modules live in the medium (thermo-host)

Status: PRECOMMITTED (2026-10-18), before any code.

## The point

Jesse's theory is that the thermal base is "that state, where everything else lives in". So far the
brain's modules (QB-9's D detector → M memory → T tracker, 0.776) are wired directly: D hands M an
event, and M hands T its reading. In T5 they **sit at sites of the 3D medium, and every signal between
them travels through it**. T6 then heats and cools that medium.

## The stack

`thermo-host` is one function, **the port**, where a module meets the medium:
- the medium holds one field per signal species (chemistry's species: each option of each signal is
  its own diffusing substance, on one shared lattice);
- `inject(site, values, gain)` adds gain·values[j] to species j at the site;
- `sense(site)` reads every species there.

Mechanism only. Transport is T2's bath, composed by the orchestrator (`thermo5`):
- k substeps per world step;
- each substep is an exchange at rate κ, then a leak, damping towards 0 at rate γ_L;
- T = 0 at T5 (no noise is generated; T3's lesson), and the heat round adds it.

**The world and the brains:** QB-7's world (p_miss 0.4, n = 16), with dev seeds 0–15 and test 100–131.
- the quantum brain is QB-9's final kept system (0.776);
- the classical brain is QB-9's tuned comparator (0.775);
- both are copied into the runner, with D's event and M's reading routed through ports.

**The layout:** a 3D lattice of side 16, periodic. The sensor, D and T sit at A = (8, 8, 8), where the
observation arrives. M sits at B = A + (r, 0, 0). There are 2 × 16 species: events (D → M) and
readings (M → T).

**One world step:**
1. D decides its event at A, and its port injects the one-hot event at A.
2. M's port senses the event species at B. The event is the argmax species, if it reaches
   ½·G_E(r).
   - G_E(r) is the peak of a unit pulse's response at B, measured once by the orchestrator at
     T = 0 as a calibration.
   - **Thin policy, labelled:** a reading must become a discrete event for M's collapse. The
     policy-free alternative (feeding M the sensed vector) changes what M is, so it is not this
     rung.
3. M updates, and its port injects its reading at B.
4. T's port senses the reading species at A. T consumes p̂ = max(φ, 0)/Σ max(φ, 0), uniform if
   the sum is 0, in place of the wired reading.
5. The transport runs. A signal injected at step t can reach a remote port at step t + 1 at the
   earliest.

**The ideal medium:** r = 0 (co-located), κ = 0, and a full leak (e^{−γ_L} = 0). The port then reads
exactly what was injected this step.

**The remote medium:** κ 0.5, k = 4, γ_L 0.1 per substep. The screening length is √(κ/2/γ_L) ≈ 1.6
sites.

**Sized for expansion:** the modules occupy at most 7 sites along one axis, leaving at least 4.5
screening lengths to the wrap. H5 tests that this is large enough by convergence.

## Its own checks (predictions)

1. **H1, the ideal medium is the wiring:** quantum 0.776 (cued 0.908, missed 0.572) and classical
   0.775 (0.903 / 0.578), with **every choice identical** on all 32 test seeds.
2. **H2, the medium is linear and reciprocal at T = 0,** recomputed independently by separate medium
   runs:
   - the response to two sources equals the sum of the responses (max error ≤ 10⁻¹² of the peak);
   - G(A → B) = G(B → A) to 10⁻¹².
3. **H3, a remote memory works:** at r ∈ {1, 2, 3}, quantum test acc_resp is within 0.01 of 0.776 at
   r = 1 and 2, and within 0.02 at r = 3.
   - **Why:** T integrates the reading slowly (η_m 0.05), so the medium's one-step delay and low-pass
     cost little.
4. **H4, the same for the classical brain,** against 0.775.
5. **H5, the mesh is large enough:** at r = 2, side 16 → 24 changes test acc_resp by ≤ 0.003 and the
   channel check by ≤ 0.01.

## Optimization round, on the remote quantum brain at r = 2

**The check:** the channel's consumer-side log-score. That is M's QB-9 check, measured on p̂ at T's
port: the mean over steps ≥ cue of ln(max(p̂_target, 10⁻⁹)). It follows the law that a module's check
scores what its consumer uses. Plain wins ties, and a variant is kept only if better on test.

- **Amplification on and off,** the send gain g ∈ {2, 4} on both ports.
  - **Prediction H6:** an exact tie, not kept. The medium is linear, and the read normalizes the
    gain away, including in G_E.
- **Hot and cold,** the bath's fluctuation–dissipation noise on every species, T ∈ {10⁻⁶, 10⁻⁵}.
  - **Prediction H7:** not kept. M's own heat is already at its log-score optimum (ε 0.02, QB-9), so
    the channel's noise can only add softening past it.
- **Chaos on and off,** a frozen per-site mean-preserving jitter of γ_L, 1 + α(x − ½), α ∈ {0.1, 0.3}.
  - **Prediction H8:** not kept. It only reshapes a gain the read normalizes away.

## Build

- `thermo-host` (the port), with unit tests: inject then sense round-trips; injecting one species
  leaves every other untouched.
- The runner `thermo5`, which copies QB-9's run (the quantum brain) and classical (the classical
  brain) with the two ports. Its first check is that, with the wired path selected, it reproduces
  0.776 and 0.775.
