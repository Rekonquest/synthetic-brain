# QB-1: the first forward test of the quantum-base characters

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

`QUANTUM-BASE-CHARACTERS.md` explains the recorded "off-target" outcomes from each base axiom's
character, but only after the fact. QB-1 asks the forward question: **on a composition that has never
been built, do the characters predict the outcome before the run?**

Every prediction below comes from a named character, not from the goal. Each arm gets the three labels:
load-bearing, signature, and fit.

## The task (new, nothing on record uses it)

**Tracking the best of N options while the best one changes.**
- N = 8 options and T = 400 steps.
- The best option is b(t) = 2 for t < 200, then b(t) = 5.
- **Evidence at each step:** one option is observed. It is b(t) with probability 0.4, otherwise
  uniform over all 8.
- **The decision at each step:** the stack chooses one option.
- **Score:** how often the choice equals b(t).
- **Seeds:** 32 (SplitMix64). Every arm sees the same evidence stream for a given seed, so the arms are
  paired.

## The stack

**State:** one complex amplitude per option, normalized (Σ|a|² = 1), starting uniform.

**The classical hooks** (Jesse's cognitive roots), the same in every arm that has them, in this order
each step:
1. **REINFORCE:** the observed option's amplitude is multiplied by (1 + η), with η = 0.15.
2. **DECAY:** each |a_i| ← |a_i|^(1−λ), with λ = 0.02, keeping its phase. This is forgetting: small
   magnitudes rise toward the rest.
3. **CONSERVE:** renormalize.

**The base primitives:**
- **Born readout:** choose option i with probability |a_i|².
- **Destructive projection:** after the readout, the state becomes the chosen option alone (phase
  kept).
- **Ring mixer:** a partial unitary between ring neighbours, [[cos φ, i sin φ], [i sin φ, cos φ]] on
  the even pairs, then the odd pairs, with φ = π/12. It runs before REINFORCE.
- **Phase hook:** the observed option's amplitude is also multiplied by e^{iθ}, with θ = π/3.

**The replacement base (L1):** real non-negative weights w with the same three hooks, and CONSERVE as
Σw = 1. The choice is drawn with probability w_i / Σw. Since REINFORCE and DECAY are multiplicative or
power maps, |a| is exactly proportional to w. **So the Born arm and the L1 arm differ only in the
squaring.**

**The classical stack without a base:** the same w, choosing argmax w. That is a greedy choice.

## Arms

| Arm | Base | Hooks | Readout |
|---|---|---|---|
| `born` | amplitudes | R, D, C | Born, non-destructive |
| `born-collapse` | amplitudes | R, D, C | Born + projection |
| `born-collapse-mix` | amplitudes + mixer | R, D, C | Born + projection |
| `born-mix` | amplitudes + mixer | R, D, C | Born |
| `born-phase` | amplitudes + phase | R, D, C | Born |
| `born-mix-phase` | amplitudes + mixer + phase | R, D, C | Born |
| `l1` | replacement weights | R, D, C | w/Σw |
| `greedy` | none (classical only) | R, D, C | argmax |
| `base-only` | amplitudes | none | Born |

## Measures (per arm, the mean over seeds unless stated otherwise)

- **acc1:** accuracy over t = 50–199.
- **acc2:** accuracy over t = 300–399.
- **latency:** the first t ≥ 200 at which at least 5 of the last 10 choices equal 5, minus 200, capped
  at 200.
- **decisiveness:** the mean of the largest choice probability at each readout. Not defined for
  `greedy`.
- **change rate:** the fraction of steps whose choice differs from the step before.

## Predictions, each from a named character

1. **P1, commit and stay committed (character 1):** `born-collapse` pins at step 1.
   - Once projected, every other amplitude is zero, and REINFORCE, DECAY and CONSERVE all map 0 to 0.
   - So after step 1 there are 0 choice changes on every seed, and acc1 and acc2 are about chance (the
     mean is in [0.05, 0.25]).
2. **P2, collapse plus mixing gives Zeno-limited local exploration (characters 1 and 3):**
   - `born-collapse-mix` moves, but slowly: its change rate is in [0.02, 0.25].
   - The amplitude only leaks to ring neighbours before each re-projection, so the choice walks the
     ring, biased by evidence. acc1 is above `born-collapse`'s acc1 + 0.1, and below `born`'s acc1.
3. **P3, squaring peaks (character 2):**
   - `born` decisiveness > `l1` decisiveness on every seed.
   - `born` acc1 > `l1` acc1 (the mean).
   - Their latencies are within 20 steps of each other, because the crossing point of |a| and w is the
     same; only the sharpness around it differs.
4. **P4, phase is inert until something mixes it (character 4):**
   - **P4a:** `born-phase` makes choices bit-identical to `born` on every seed.
   - **P4b:** `born-mix-phase` differs from `born-mix` (in its choice sequence) on at least 30 of the
     32 seeds.
5. **P5, the base alone gives no verdict (law 1):** `base-only` acc1 and acc2 are about chance (the mean
   is in [0.05, 0.25]).
6. **P6, mixing spreads and blurs evidence (character 3):**
   - `born-mix` acc1 < `born` acc1.
   - `born-mix` decisiveness < `born` decisiveness.
   - `born-mix` latency ≤ `born` latency + 5.
7. **P7, a classical method ties or wins at the target (law 5):** `greedy` acc1 ≥ `born` acc1 (the
   mean).

**Fit, stated before the run:** the arm best suited to "track the best option" is predicted to be
`greedy` or `born`. The other arms are predicted to show the base's characters (perseveration, local
exploration, decisiveness), which fit other targets: holding a decision, foraging, and salience.

## Kill criterion

If 3 or more of P1–P7 fail, the character sheet does not predict forwards on this composition. That is
recorded as-is, and the sheet stays retrospective only. Each failed prediction is analysed for whether
the character was wrong, or my derivation of the prediction from it was.

## Build

- **Rust, zero dependencies,** in `synthetic-brain/`:
  - `qb-amplitude`: state, normalize, Born weights, projection, ring mixer, phase;
  - `qb-classical`: reinforce, decay, conserve, L1 weights, argmax;
  - `qb1`: the orchestrator and runner.
- The parameters above are fixed here, and the runner holds them as settings.
- **Unit tests before the run:**
  - the mixer is unitary;
  - projection is idempotent;
  - |a| ∝ w under the hooks (the claim behind P3);
  - the same evidence stream for every arm.

## Results (2026-10-18, `evidence/qb-1/results.txt`)

The tests passed before the run: the mixer is unitary and reaches both ring neighbours, projection
is idempotent (to 1e-12), |a| is proportional to w to 1e-9 over a whole stream, and every arm sees the
same streams. 32 seeds, and every arm ran on the same evidence.

| Arm | acc1 | acc2 | latency | decisiveness | change rate |
|---|---|---|---|---|---|
| `born` | 0.927 | 0.937 | 38.9 | 0.843 | 0.247 |
| `born-collapse` | 0.156 | 0.219 | 156.2 | 0.998 | 0.000 |
| `born-collapse-mix` | 0.129 | 0.146 | 66.2 | 0.862 | 0.140 |
| `born-mix` | 0.149 | 0.149 | 142.1 | 0.238 | 0.855 |
| `born-phase` | 0.927 | 0.937 | 38.9 | 0.843 | 0.247 |
| `born-mix-phase` | 0.202 | 0.182 | 47.9 | 0.350 | 0.791 |
| `l1` | 0.610 | 0.621 | 46.5 | 0.542 | 0.653 |
| `greedy` | **1.000** | **1.000** | 38.2 | — | 0.010 |
| `base-only` | 0.131 | 0.118 | 170.4 | 0.125 | 0.879 |

| # | Result | What was measured |
|---|---|---|
| P1 commit and stay committed | MEASURED | pinned at step 1 on 32 of 32 seeds; accuracy at chance (0.156 / 0.219) |
| P2 collapse plus mixing gives Zeno-limited exploration | **FAILED** | the change rate held at 0.140 (inside its band), but acc1 was 0.129, at chance. The bar was > 0.256 |
| P3 squaring peaks | MEASURED | `born` sharper than `l1` on 32 of 32 seeds; acc1 0.927 against 0.610; latency gap 7.6 |
| P4a phase inert without a mixer | MEASURED | `born-phase` choices identical to `born` on 32 of 32 seeds |
| P4b phase active with a mixer | MEASURED | `born-mix-phase` differs from `born-mix` on 32 of 32 seeds |
| P5 the base alone gives no verdict | MEASURED | `base-only` at chance (0.131 / 0.118) |
| P6 mixing spreads and blurs | **FAILED** | it blurs (decisiveness 0.238 against 0.843) and is less accurate (0.149 against 0.927), but latency was 142.1 against 38.9, outside the bar (≤ 43.9) |
| P7 a classical method ties or wins | MEASURED | `greedy` 1.000 against `born` 0.927 |

**6 of 8 checks hold; 2 of the 7 predictions fail, so the kill criterion (3) does not fire.**

### The two misses, analysed: each is my derivation, not the character

**P2.** I derived "collapse plus mixing = a walk biased by evidence". That treats collapse as commitment
only. But the cognitive engine's record already names collapse's other face: **destructive readout is
forgetting** (BUILD-1b). Every step's projection erases the evidence the hooks had accumulated. So the
walk has no memory of where the evidence pointed, and it is unbiased: at chance, which is what was
measured. Read in full, the character predicts the result; my derivation used half of it.

**P6.** I derived "mixing blurs the peak onto its neighbours". That is diffusion thinking, the
classical picture. The character on record is **ballistic transport** (E2: 29× the classical
variance, α = 2). A unitary walk applied every step does not smear the evidence peak in place; it
carries it away around the ring. So accuracy falls to chance, and recovery never comes. Blurring did
appear (decisiveness 0.238), but transport, not blur, is the dominant term.

**The lesson for the method:** both misses came from swapping the quantum character for its classical
intuition (persistence for collapse, diffusion for mixing). The characters themselves, read literally,
account for all 8 results. That is a post-hoc reading, so it is not claimed as a pass. It becomes the
precommitted derivation rule for QB-2: **derive from the character as recorded, never from the
classical analogue.**

### Off-target behaviour worth keeping (the three labels)

| Arm | Load-bearing | Signature | Fit |
|---|---|---|---|
| `born` against `l1` (squaring) | yes: +0.317 acc1 from squaring alone | sharpens choices around the same crossing point | **good** for decisions and salience |
| `born-collapse` | yes: pinned against `born` | perseveration: holds a decision regardless of evidence | holding a commitment; bad for tracking |
| `born-collapse-mix` | yes | memoryless local exploration (change rate 0.140, Zeno-limited) | foraging or search with no memory |
| `born-mix` | yes | ballistic transport carries evidence away | bad for tracking; possibly good for propagating a signal |
| `born-mix-phase` against `born-mix` | yes: phase changes every seed, and acc1 rises 0.149 → 0.202, latency 142 → 48 | the phase hook partly re-localizes the transported state (interference) | unexplained; worth its own test |

The classical greedy argmax is the best tracker (1.000), as law 5 predicted. The base's value here is
not tracking: it is the other behaviours it supplies.

### Next (QB-2, not yet precommitted)

- A fresh composition, with predictions derived by the corrected rule: collapse forgets as well as
  commits, and mixing transports rather than diffuses.
- `born-mix-phase`'s partial re-localization as its own question: does a phase hook at the evidence
  site counter ballistic transport? That would be interference used to hold a state.
