# PHYS-POC: theoretical-physics axioms as atom stacks (proofs of concept)

Status: PRECOMMITTED (2026-10-24), before any PHYS code.

Jesse (2026-10-24): "lets try quarks and string axioms and lets dig into theoretical physics as
well... if they are load bearing, then we know there is a there, there."

**What load-bearing will and will not show:** a law that is load-bearing here does real work as a
mechanism in the stack. That does not test whether the theory is true of nature. A theory's
mathematics can be useful even if the theory is wrong, as epicycles were.

**For every test:** plain is the bar, with the same seeds. Each runs in seconds, on CPU only.

**Amended before any code (2026-10-24).** Jesse: "Was chaos and amplification used on the
astrophysics axioms? Were they logically composed?" They were not, and this doc first deferred them
too. Each test is now composed in Jesse's order:
1. **The law as its own stack:** the plain unit against the law's unit, the law kept only if it wins
   on its own check.
2. **Chaos, off against on,** on that stack: a small mean-preserving logistic perturbation every
   step, with the stack's own check selecting and plain winning ties (dev seeds 0–3, then test 10–17).
3. **Amplification, on against off,** on what was kept.

Heat (the noise level) is swept where it is the natural knob.

## PHYS-1: the quantum Zeno effect (attention holds a thought)

- **The law:** a quantum state watched often enough does not evolve. Repeated projective
  measurement resets its leakage, which starts quadratically in time.
- **The stack:** `qb-amplitude` with 16 options, starting in option 0.
  - **Drift:** `ring_mix(φ 0.01)` each step, for T = 200 steps.
  - **Attention:** every k steps, a Born measurement (`born_weights` + `project`). It survives only if
    every measurement, including the final one, finds option 0.
  - k ∈ {200 (final only), 20, 10, 5, 2, 1}, with 4,000 trials each.
- **The classical twin:** the same chain with classical ring diffusion (`qb-classical::diffuse`,
  s = φ²) and the same observations (a draw from the weights, held at the observed option).
- **Predictions:**
  - **Z1:** quantum survival rises with the attention rate, from ≤ 0.25 at k 200 to ≥ 0.95 at k 1.
  - **Z2:** classical survival varies by ≤ 0.03 across all k. Observing a classical drift does not
    slow it.

## PHYS-2: gauge invariance (the quark rule, here in its U(1) form)

- **The law:** in a gauge theory (QCD for quarks, here the simpler U(1)), a local phase can be
  changed freely at every point. Only *relative* quantities around links (Wilson lines) are real.
- **The stack:** FOLD-6's content store at full precision, D 4,096, N ∈ {8, 16} random bytes. After
  writing, the medium drifts by a smooth local phase φ_i (a random walk along i with step σ, the
  same for every memory): s_i ← s_i·e^{iφ_i}.
  - **Plain reading:** unbind, then Re⟨code, u⟩.
  - **Gauge-invariant reading:** with u(v) = s ⊙ conj(key ⊙ code_v), the score is
    Re Σ_i u_i(v)·conj(u_{i+1}(v)), a Wilson line of length 1. The drift enters only as
    φ_i − φ_{i+1}, a step of size σ.
  - σ ∈ {0, 0.02, 0.1, 0.3}.
- **Predictions:**
  - **G1:** with no drift, plain is 1.000. Gauge-invariant is ≥ 0.95 at N 8 and ≥ 0.7 at N 16 (it
    pays in crosstalk, because the noise of a product grows as N rather than √N).
  - **G2:** at σ ≥ 0.1, plain is ≤ 0.05 (chance is 1/256). The phase spans many turns over 4,096
    sites.
  - **G3:** gauge-invariant at N 8 is ≥ 0.95 at σ 0.1 and ≥ 0.9 at σ 0.3 (cos 0.3 = 0.955).

## PHYS-3: topological charge (a memory that local noise cannot erase)

- **The law:** a winding number, Σ wrap(θ_{i+1} − θ_i)/2π around a closed loop, is an integer. No
  smooth local change can alter it. It changes only by a phase slip, where one neighbour step
  crosses π.
- **The stack:** 64 rings of L = 32 phases (closed loops). Each step adds local noise θ_i += σ·ξ,
  then relaxes each phase toward its neighbours, θ_i += J·Σ sin(θ_nb − θ_i) with J 0.2.
  - **Topological memory:** a bit is the winding, 0 or 1 (written as θ_i = 2πW·i/L).
  - **Plain (local) memory:** a bit is the ring's mean phase, 0 or π, read as the nearer of the
    two.
  - T = 20,000 steps; σ ∈ {0.05, 0.1, 0.2, 0.5, 1.0} (the heat sweep); seeds 10–17.
- **Predictions:**
  - **P1:** at σ 0.1, the topological memory keeps ≥ 0.99 of its bits, and the local one ≤ 0.75.
    The mean phase is an unprotected zero mode; it random-walks with variance σ²T/L ≈ 6 rad².
  - **P2:** heat melts it, but only past a threshold: topological ≥ 0.95 for σ ≤ 0.2, then falling
    as phase slips set in. At σ 1.0 both are near chance (0.5).

## PHYS-4: string modes (a closed string's identity is its vibration)

- **The law:** in string theory a string's state, which particle it is, is its vibrational mode.
  The modes of a closed string are integer harmonics, and each keeps its own amplitude in a lossless
  string.
- **The stack:** the wave medium (`wave-lattice`, 1D, periodic = a closed string), L 128, c² 1, no
  loss, dt 0.5.
  - **Mode memory:** K = 32 bits written as standing-mode amplitudes ±A on cos(2πn·x/L), n = 1..32.
    Each is read from its own mode alone: project u and v onto it, and take the sign of the overlap
    with a noiseless reference string (all bits +A, evolved alongside, as a clock). The reference
    carries the mode's known phase, so no dispersion formula is needed.
  - **Plain (position) memory:** 32 bits as displacements ±A at 32 sites, read as the sign at those
    sites.
  - Small kicks each step (noise σ), T = 10,000 steps, σ ∈ {0, 0.001, 0.01}.
- **Predictions:**
  - **S1:** mode memory keeps ≥ 0.99 at σ ≤ 0.001, and position memory ≈ 0.5 (chance) at every σ.
    A displacement in a wave medium leaves at speed c.
  - **S2:** at σ 0.01, mode memory is still ≥ 0.9. Noise spreads over 128 modes, while each bit's
    energy stays in one.
- **The honest limit:** the medium is linear and lossless, so a full inverse of its evolution would
  decode position bits too. The test is whether a bit can be read from its own channel alone. The
  modes are the channels that hold still.

## The chaos and amplification arms (amended before code)

| test | chaos (on against off; the stack's own check selects) | amplification (on against off) |
|---|---|---|
| PHYS-1 Zeno | the attention interval jittered per measurement, k·(1 + 0.5(u − ½)) (mean-preserving); selected by survival on a dev batch | Born power 2 → 4 in the measurement (`power_magnitudes`, "more of what is there") |
| PHYS-2 gauge | 3 variants of the cue's phases jittered by 0.2·(u − ½), selected by the reading's own margin | the Wilson line summed over lags 1 and 2 (more of the invariant) |
| PHYS-3 topology | J jittered per step and per ring, J·(1 + 0.5(u − ½)); the relaxation itself is the selector (a smooth flow that keeps what is stable) | J × 2 (stronger coupling) |
| PHYS-4 strings | frozen disorder in c², ±1% per site (frozen chaos; QB-2: it localizes) | the write amplitude A × 2 |

**Predictions:**
- **C1, chaos:**
  - not kept in PHYS-1, PHYS-2 and PHYS-3;
  - **harmful** in PHYS-4, because frozen disorder localizes the waves and so mixes the modes.
- **A1, amplification:**
  - kept in PHYS-1 (a power above Born biases each measurement toward the held option);
  - not kept in PHYS-2 (the lag-2 step drifts twice as far);
  - kept in PHYS-3 at σ ≥ 0.2 (stiffer rings slip less);
  - kept in PHYS-4 at σ 0.01 (signal against noise).

## Discovery beyond memory (amended before code, 2026-10-24; Jesse chose these four probes)

Jesse: "And don't limit discovery to memory. Key word 'discovery'." (Rule 3: from the known into the
unknown.) Each law's stack is also probed for one behaviour outside memory, chosen by what it
consumes and produces (Rule 2), against a plain or classical twin.

| law | probe | capacity (atlas) | the bar |
|---|---|---|---|
| Zeno | **blockade as a GATE:** a walker starts at option 0 of a 16-ring and spreads by `ring_mix`; attention on option 4 (measured every step: "is it here?"; on a yes, the run ends there) | 4 attention; 10 control | the classical twin, a diffusing walker watched the same way |
| gauge | **invariant perception:** classify 64 random phasor shapes (length 512) received through an unknown smooth phase channel (step σ) | 1 perception | the plain correlation reading |
| topology | **robust counting:** 50 events, each a full twist added smoothly over 100 steps, then rest, under noise σ; the count is the winding | 8 number | an analog accumulator gaining 2π per event under the same noise, read as round(φ/2π) |
| strings | **parallel lanes:** 8 messages (64 one-bit symbols each) sent at once on one closed string, each at its own harmonic; one listener decodes all 8 from its site | 9 composition; signalling | the same messages time-shared on one harmonic (8× the time) |

**Predictions:**
- **D1:** with attention on option 4 every step, the walker reaches options 6–10 by T 200 ≥ 50% less
  often than with no attention; the classical twin ≤ 10% less (only what is caught at option 4).
- **D2:** at σ 0.1, Wilson classification ≥ 0.95 and plain ≤ 0.1.
- **D3:** at σ 0.1, the winding counter is exact in ≥ 0.95 of rings, and the analog counter in ≤ 0.6.
- **D4:** all 8 lanes decode ≥ 0.95 of their bits, in 1/8 of the time-shared duration.

## Results (2026-10-24, `evidence/phys-poc/`; each run 1–33 s)

### PHYS-1, the quantum Zeno effect (`phys1.txt`): 4 of 5

| attention every k steps | 200 | 20 | 10 | 5 | 2 | 1 |
|---|---|---|---|---|---|---|
| quantum survival | 0.158 | 0.453 | 0.666 | 0.818 | 0.921 | **0.960** |
| classical twin | 0.963 | 0.965 | 0.959 | 0.961 | 0.963 | 0.960 |

- **Z1 and Z2 HOLD:** attention holds a quantum state, and in the classical twin attention changes
  nothing.
- **C1 HOLDS:** chaos on the attention interval was not kept.
- **A1 HOLDS, with a flag:** Born power 4 lifts survival 0.67 → 0.996. That power is the quantum
  base's hook (`power_magnitudes`), not quantum mechanics; it biases each measurement toward the held
  option.
- **D1 FAILED:** the Zeno blockade cut the walker's reach by 60%, but the classical twin's fell 61%.
  Catching a walker at the watched site blocks it either way, so this setup shows no quantum
  blockade.

### PHYS-2, gauge invariance (`phys2.txt`): 3 of 6

| N, σ | 0 | 0.02 | 0.1 | 0.3 |
|---|---|---|---|---|
| plain, N 8 | 0.750 | 0.500 | 0.469 | 0.047 |
| **Wilson, N 8** | 1.000 | 1.000 | 1.000 | **1.000** |
| plain, N 16 | 0.750 | 0.500 | 0.367 | 0.023 |
| **Wilson, N 16** | 0.992 | 0.992 | 0.992 | **0.992** |

D2, perception of 64 shapes: Wilson 1.000 at every σ; plain 0.520 → 0.361.

- **G1 and D2 FAILED on my implementation choice:** the drift started at a random global phase,
  which alone defeats the plain phase-locked reading (0.750 with no drift). The precommit did not fix
  the start.
- **The labelled check** with the drift starting at 0 (`phys2-check-phi0-zero.txt`): plain 1.000 at
  σ 0, and G1 holds. The plain reading still holds more than predicted at σ 0.1 (0.500, and 0.738 in
  D2), so G2 and D2 fail on their plain-side thresholds.
- **G3, C1 and A1 HOLD:** the Wilson reading is unaffected by every drift; chaos and amplification
  had nothing to add.
- **Load-bearing:** read through Wilson lines, memory and perception are immune to any smooth local
  phase. Plain recall falls to 0.02–0.05 at σ 0.3.

### PHYS-3, topological charge (`phys3.txt`): 2 of 5

| σ | 0.05 | 0.1 | 0.2 | 0.5 | 1.0 |
|---|---|---|---|---|---|
| **topological (winding)** | 1.000 | **1.000** | 0.998 | 0.342 | 0.252 |
| local (mean phase) | 0.785 | **0.533** | 0.535 | 0.512 | 0.496 |

- **P1 HOLDS:** a winding memory keeps every bit through noise that erases a local one.
- **P2 FAILED on its "near 0.5" clause:** heat melts it past a threshold between σ 0.2 and 0.5, but a
  melted winding wanders to other integers, so it falls *below* chance (0.25).
- **C1 HOLDS:** chaos on J was not kept.
- **A1 FAILED by its wording:** J × 2 is kept at σ 0.5 (0.342 → 0.410), but at σ 0.2 the result is a
  tie at the ceiling.
- **D3 FAILED, and that is itself the topological law at work:** I added each event as a smooth
  twist over 100 steps, and the ring counted none of them (0.000). No smooth change can alter a
  winding, so the relaxation undid each gradual twist. Also, 50 events cannot fit: a 32-site ring
  holds a winding of at most 15.
- **The labelled check** with each event as a one-step slip (`phys3-check-instant-events.txt`):
  - the winding counter is **exact (1.000) for 1, 2 and 3 events**, where the analog counter manages
    0.164;
  - at 5 or more events it fails (0.049, 0.000), because a winding of 5 on 32 sites decays under
    this noise (measured separately: windings 1–3 held 20 of 20, winding 5 held 0 of 20);
  - so the counting capacity is set by the ring's length.

### PHYS-4, closed-string modes (`phys4.txt`): 4 of 5

| σ | 0 | 0.001 | 0.01 |
|---|---|---|---|
| **mode memory** | 1.000 | 1.000 | 0.980 |
| position memory | 0.580 | 0.506 | 0.506 |

- **S1 and S2 HOLD:** a string's harmonics hold 32 bits for 10,000 steps; displacements leave at the
  wave speed.
- **A1 HOLDS:** A × 2 at σ 0.01 gives 0.980 → 0.995.
- **C1 FAILED:** frozen ±1% disorder did no harm (1.000 → 1.000). The reference string shares the
  disorder, so the mode mixing is shared too.
- **D4 HOLDS:** 8 messages at once on 8 harmonics, decoded by one listener at **0.980**, in an
  eighth of the time. Time-shared on one harmonic it gives 1.000.

### What PHYS-POC settles

**13 of 21 hold as precommitted.** Every law tested is load-bearing for some behaviour, against a
plain or classical twin:

| law | load-bearing for | the measured difference |
|---|---|---|
| quantum Zeno | attention holding a state | 0.158 → 0.960 with attention rate; classical flat at 0.96 |
| gauge invariance | memory and perception immune to drift | 1.000 against 0.02–0.05 |
| topological charge | noise-proof memory; exact small counts | 1.000 against 0.533; 1.000 against 0.164 |
| string modes | memory in a wave medium; parallel lanes | 1.000 against 0.51; 8 lanes at 0.980 |

- **Chaos was kept nowhere,** as in FOLD-5 to 8 for stores without a carving law.
- **Amplification helped where signal fought noise:** strings and topology at high heat. The Zeno
  win comes from a non-physical Born power and is flagged.
- **What this does not show:** whether these theories are true of nature (see the top of this
  doc). The mathematics does work as mechanism.
