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
