# ASTRO-POC: three astrophysics laws as axioms for folding and recall (proof of concept)

Status: PRECOMMITTED (2026-10-23), before any ASTRO code.

Three small proofs of concept. Each borrows one law from astrophysics, maps it onto a part of the
brain that has a measured ceiling, and is run against the plain unit on the same seeds (plain wins
ties). Dev seeds 0–3 choose every setting that is chosen; test seeds 10–17 report. CPU only,
at most 4 threads, seconds to about two minutes per run. Evidence goes to `evidence/astro-poc/`.

New crates (one function each, no policy): `astro-exclude`, `astro-pressure`, `astro-virial`,
`astro-jeans`; orchestrators `astro1`, `astro2`, `astro3` hold every setting. No existing crate is
changed.

## A model of the plain 2-bit store (used by ASTRO-1 and ASTRO-2)

FOLD-6's content store: D 4,096, N byte values bound to recipe keys, b_i = key_i ⊙ code_{v_i},
s = Σ b_i, quantized to 2 bits of phase, read as the 256 scores z_{i,v} = Re⟨key_i ⊙ code_v, q⟩ / D.

- **Bussgang:** the 2-bit phase quantizer gives q = α·s + d with d uncorrelated with s. Phase-only
  quantization keeps a share 8/π² · π/4 ≈ 0.64 of the power (FOLD-7's diagnosis), so α·‖s‖ = 0.8·√D
  and ‖d‖² = 0.36·D.
- **In units of the reading noise** σ = 1/√(2D) (a random unit-magnitude q against any code):
  - the correct reading is S + n₀, with S = 0.8·√(2D/N);
  - n₀ is the crosstalk plus the quantizer's noise, of sd 1;
  - the 255 wrong readings are independent of sd 1.
- **Recall** = E over n₀ of Φ(S + n₀)^255.

This model gives 0.79, 0.635 and 0.42 at N 384, 512 and 768. The measured values are 0.800, 0.636
and 0.425, so the model is used for the derivations below.

---

## ASTRO-1: Pauli exclusion (degeneracy pressure) against crosstalk

**The law (quantum statistics; in astrophysics, white dwarfs and neutron stars):** no two fermions
occupy the same quantum state. Packing them closer costs energy: degeneracy pressure pushes them
apart, and that pressure holds a star up against gravity.

**The mapping:**
- an item's memory occupies its readout state (key_i, code_{v_i});
- any other memory's amplitude found in that state is a second occupant;
- at write time (the writer knows the content), exclusion pushes the second occupants out of the
  store before it is quantized and stored.

### Arm A, the specified law: exclusion of the shared component (pre-quantization)

- **The step (`astro-exclude`):** iterate s ← s − η·Σ_i r_i·b_i, with r_i = ⟨b_i, s⟩/‖b_i‖² − 1.
  - r_i is item i's crosstalk: the overlap of everything but its own unit component with b_i.
  - Each item's own component is kept.
- K_A 8 iterations; η ∈ {0.05, 0.1, 0.25, 0.5}, chosen per N on dev. Then quantize to 2 bits and
  read plainly.
- **Ablation:** η 0, which is plain.

**Derivation.** This is a Richardson iteration toward zero-forcing precoding, s = B·G⁻¹·1, with
G = BᴴB/D. G's eigenvalues lie in [(1 − √β)², (1 + √β)²] with β = N/D, so η 0.5 converges for
N ≤ 768.
- **What exact exclusion removes:** the crosstalk on the correct reading only. n₀ keeps just the
  quantizer's part, of sd 0.6. The wrong readings Re⟨b_{i,v}, q⟩ are unchanged in law.
- **What it costs:** E[G⁻¹] = I/(1 − β), so ‖s‖² = DN/(1 − β). After quantizing, the correct
  reading falls to S′ = 0.8·√(2D/N − 2).
- **Recall** = E over n₀ ~ N(0, 0.6²) of Φ(S′ + n₀)^255:
  - N 384: S′ 3.52, giving 0.835 (model plain 0.79);
  - N 512: S′ 2.99, giving 0.60 (model plain 0.635);
  - N 768: S′ 2.36, giving 0.26 (model plain 0.42).
- **Why it can lose:** recall is set by the maximum of 255 wrong readings (about 2.75σ). When the
  mean signal is below that maximum, a correct reading's own spread (n₀) is what lets the lucky
  items through. Exclusion equalizes the items toward the mean, and so removes that luck.

**Predictions (test seeds, random bytes, against plain on the same seeds):**
1. **X1:** full exclusion (η 0.5, K 8) gains at the light load: N 384, +0.02 to +0.07.
2. **X2:** at N 512 it does not gain: the difference is between −0.07 and +0.02.
3. **X3:** at N 768 it loses: worse than plain by ≥ 0.08.
4. **X4:** with η chosen on dev, the test gain at N 768 is ≤ +0.01 (dev picks a small η, or plain).

### Arm B, exclusion on the colliding states, felt through the quantizer

Arm A's derivation says the second occupants that cause errors are not in the correct state: they
are in item i's *competing* states (i, v ≠ v_i), whose readings beat the correct one.
- **Arm B applies the exclusion there,** and measures it on the stored (quantized) state, since that
  is where the memories live.
- **Each of K_B 4 iterations:**
  - q = Q(s);
  - read every z_{i,v} from q;
  - σ̂ is the rms of the wrong readings; α_q = Re⟨q, s⟩ / ‖s‖² is the measured Bussgang gain;
  - every competing state with z_{i,v} > z_{i,v_i} − γσ̂ is a collision, with deficit
    δ = z_{i,v} − z_{i,v_i} + γσ̂;
  - **the degeneracy pressure (`astro-pressure`):** s ← s + x·(b_i − b_{i,v}), with x = η_B·δ/(2α_q).
    Pushing by x moves the gap by 2α_q·x in the linearized store.
- Then the final q = Q(s) is stored and read plainly.
- η_B 0.5. γ ∈ {0.5, 1, 2} is chosen on dev: one γ, the best dev mean over N.
- **Ablation:** the same pushes (the same number per item, the same x), but each against a wrong
  code drawn at random instead of the colliding one.

**Derivation (weaker; there is no closed form for the converged state).**
- Every iteration reads the stored state itself, so a collision that the last iteration removes is
  an error removed.
- The quantizer re-adds noise of 0.6σ on every reading each round, which caps the convergence.
- Content is N·8 bits against 8,192 stored bits: 0.5 bits per bit at N 512 and 0.75 at N 768.
  FOLD-6's plain store reaches 0.266, so there is room, more so at 512.
5. **X5:** B ≥ plain + 0.10 at N 512.
6. **X6:** B ≥ plain + 0.05 at N 768.
7. **X7, the law is load-bearing:** B ≥ its random-rival ablation + 0.08 at N 512, and the ablation
   is within ±0.05 of plain.

**Kill:** if neither A nor B beats plain by ≥ 0.02 at any N, exclusion at write time is not a lever
on this store.

---

## ASTRO-2: the virial theorem (negative heat capacity) as a self-regulating hot/cold governor

**The law (stellar dynamics):**
- **Virial theorem:** a bound self-gravitating system in equilibrium has 2K + U = 0, so its
  temperature is set by its own binding, not by a heat bath.
- **Negative heat capacity:** as it loses energy it contracts and its kinetic temperature changes by
  itself. There is no external thermostat and no schedule.

**The mapping (as specified):** each hypothesis's temperature is set by its own binding.
- **Binding:** the margin m_i between its best and second-best code.
- **Tightly bound → cold (hard). Loosely bound → hot (soft, spread, so it cancels little).**
- As cancellation raises margins over the rounds, the system cools itself.
- **The functional form, derived:** the reading noise is Gaussian of sd σ_i, measured from item i's
  own 255 non-best readings. The odds that the runner-up is the true value are then about
  exp(m_i²/(4σ_i²)). Setting the Boltzmann weight's odds exp(m_i/T_i) equal to them gives
  **T_i = κ·σ_i²/m_i with κ ≈ 4.**
- **The governor (`astro-virial`):** κ ∈ {2, 4, 8}, chosen on dev; nothing else is set.

**Fidelity note:** in real self-gravitating systems the virial theorem makes the *more* bound system
the *hotter* one (T ∝ GM/R). The specified mapping inverts that direction. The literal direction,
T_i = c·m_i (c ∈ {0.25, 0.5, 1} on dev), is run beside it as an extra arm.

**The setup:** FOLD-8's recall loop (its code copied):
- 3 rounds;
- the soft predicted interference bind(key_j, Σ_v p_v·code_v), with p = `qb-classical::temper`;
- CONSERVE's least-squares scale;
- REVISE re-reads with the others' interference dissolved.

Random bytes, N ∈ {512, 768}, test seeds 10–17.

**Arms:**
- plain;
- hard cancellation (K1, gate ρ 0.75, FOLD-8's dev choice);
- hard without a gate (ρ 1);
- the fixed schedule (K6: T₀ 0.005, FOLD-8's dev choice, then T₀/2 and T₀/4);
- virial (T_i = κσ_i²/m_i);
- **ablation:** one temperature for every item, each round's median of the virial T_i, so the
  rounds still cool but the per-item binding is removed;
- literal virial (T_i = c·m_i).

**The check:** plain, K1 and K6 reproduce FOLD-8's test numbers:
- plain: 0.636 / 0.425;
- K1: 0.894 / 0.407;
- no gate: 0.857 / 0.369;
- K6: 0.884 / 0.334 at N 512 / 768.

**Why it should help:** K6's temperature (0.005 ≈ 0.45σ) is the same for every item. An unsure item,
with a margin under σ, still puts most of its weight on its top one or two codes, and so cancels a
wrong value nearly at full weight. That is why every cognitive arm in FOLD-8 lost to plain at N 768,
where about 58% of the first reads are wrong. The virial governor makes those items hot: their
mixture spreads over many codes, its norm falls, and they cancel little. The bound items stay hard.

**Predictions:**
8. **V1:** at N 512, virial ≥ max(K6, K1) + 0.01.
9. **V2:** at N 768 it no longer collapses: virial ≥ plain + 0.02 and ≥ K6 + 0.08.
10. **V3, the per-item binding is load-bearing:** virial ≥ the one-temperature ablation + 0.03 at
    N 768.
11. **V4, it cools itself:** at N 512 the median T_i falls from round 1 to round 3 (MEASURED and
    reported, with no external schedule).
12. **V5, the literal direction (bound → hot) is worse:** at N 768, ≤ virial − 0.03.

**Kill:** virial ≤ plain at both N, and ≤ K6 at N 512.

---

## ASTRO-3: Jeans instability as the scar-formation law in the thermal medium

**The law (star formation):**
- In a self-gravitating gas, a perturbation of wavenumber k obeys ω² = c_s²k² − 4πGρ.
- Modes with k > k_J (shorter than the Jeans length λ_J = c_s·√(π/(Gρ))) are held up by pressure.
  They oscillate and disperse; they never form structure.
- Only perturbations larger than λ_J collapse.

**The mapping:**
- `spec-trap::form` grows material from the *local* energy at each site, x = e_s/E_ref, blind to
  scale, so a lone noise kick drives the trap law exactly as a written memory does.
- Read in Fourier space, Jeans' criterion says the structure-forming drive is the energy field with
  its modes above k_J removed.
- **The Jeans drive (`astro-jeans`)** is therefore the energy averaged over an R × R window (R ~ λ_J,
  the simplest lattice low-pass). The unchanged trap law then runs on x = ⟨e⟩_R / E_ref.
- R 1 is the identity, the local law. R ∈ {3, 5}, chosen on dev.

**The setup:**
- SCAR-1 run 5 / FOLD-4's medium laws:
  - `step_sealed` κ 0.2, δ 0.01, τ 0.99;
  - `spec-trap` μ 0.02, ν 5·10⁻⁵, p 3;
  - `spec-stir` ε 0.05.
- A 2D periodic 64 × 64 lattice, 1 band. The trap law and the diffusion see only the total energy,
  so the bands do not matter here.
- **Three noise levels,** each a condition: stir q ∈ {0.02, 0.05, 0.1}.
- **Writes:** square blobs of side 1, 3 and 7, six of each per seed, at random positions at least 6
  sites apart. Gain 0.02 per site per step for 300 steps, then 3,000 drain steps with the stir on.
- **Read:** material m > 0.5 is a scar.
  - A blob is scarred if at least half its sites are scarred. **True-scar rate = the fraction of
    blobs scarred, per side.**
  - **False-scar rate = the fraction of far-field sites** (Chebyshev distance > 5 from every blob)
    that are scarred.
- **Matched true rate:** for each law and q, E_ref is the largest value in
  {0.05, 0.075, 0.1, 0.15, 0.2, 0.3, 0.45, 0.6, 0.9, 1.35, 2.0, 3.0} whose dev side-7 true rate is
  ≥ 0.9. R is the one with the lower dev false rate, pooled over q.

**Derivation:**
- The stirred field is already smoothed by the medium's own diffusion over
  ℓ ≈ √(D_diff/δ) = √(0.1/0.01) ≈ 3 sites.
- So a 3–5 site window lowers the noise's site-to-site spread only moderately, about 1.5–2× in sd.
- The trap law is steep (p 3) and self-reinforcing: a carved site leaks 100× less and seals itself,
  so false scars come from the far tail of the noise. Halving the sd at a fixed threshold cuts that
  tail by much more than half.
- A side-7 blob keeps its full drive at its interior under the window. A side-1 write is diluted
  about R²-fold before diffusion, so the Jeans law should refuse small writes. That is its
  signature, and also its cost.

**Predictions:**
13. **J1, fewer false scars at the same true-scar rate:** at every q where the local law's test false
    rate is ≥ 0.01, the Jeans false rate is ≤ 0.5 × the local one. Both laws' test side-7 true rate
    is ≥ 0.85.
14. **J2, the scale law (small perturbations do not collapse):** side-1 true rate under Jeans
    ≤ 0.5 × local's at every q (or both ≤ 0.1).
15. **J3, the code check:** R 1 reproduces the local law bit for bit.

**Kill:** if the Jeans false rate ≥ local at matched true rate for every q, the scale law is not
load-bearing in this medium.

---

## Results

(Added below after the runs; predictions above are not edited.)

### ASTRO-1 (2026-10-24, `evidence/astro-poc/astro1.txt`, 133 s on 4 threads)

**The check:** plain reproduces FOLD-6/8 exactly: 0.800, 0.636 and 0.425.

**Test (seeds 10–17), accuracy:**

| arm | N 384 | N 512 | N 768 |
|---|---|---|---|
| plain (η 0, A's ablation) | 0.800 | 0.636 | 0.425 |
| A: full exclusion (η 0.5, K 8) | **0.829** | 0.600 | 0.267 |
| A at dev-chosen η (0.5, 0.05, plain) | 0.829 | **0.650** | 0.425 |
| B: pressure on colliding states (γ 2) | 0.524 | 0.480 | 0.399 |
| B ablation: random rivals | 0.498 | 0.469 | 0.396 |

**The model predicted** 0.835, 0.60 and 0.26 for full exclusion; it measured 0.829, 0.600 and
0.267.

| # | Result |
|---|---|
| X1 full exclusion at N 384, +0.02..+0.07 | MEASURED: +0.029 |
| X2 no gain at N 512 (−0.07..+0.02) | MEASURED: −0.036 (the dev-chosen small η 0.05 gives +0.014) |
| X3 loses ≥ 0.08 at N 768 | MEASURED: −0.158 |
| X4 dev-chosen η gains ≤ +0.01 at N 768 | MEASURED: dev chose plain, +0.000 |
| X5 B ≥ plain + 0.10 at N 512 | **FAILED:** −0.156 |
| X6 B ≥ plain + 0.05 at N 768 | **FAILED:** −0.025 |
| X7 B ≥ ablation + 0.08 | **FAILED:** +0.010 |

**4 of 7 hold.**

**Diagnosis:**
- **Arm A (the specified law) behaves exactly as derived.** Exclusion zero-forces the crosstalk on
  each item's own state, but the store pays 1/(1 − N/D) in power for it. It also strips each item's
  luck: the spread of its own reading is what lifts items past the maximum of 255 wrong readings
  when the mean signal sits below that maximum. So exclusion is a **light-load law**: +0.03 at N
  384, harmful from about N 512.
- **Arm B is broken as built,** not refuted. The collision count *rises* after the first iteration
  (N 512: 25,742 → 93,414 → 55,000 → 87,793). Each item has tens of collisions at γ 2, and every
  push is applied at once (Jacobi) with the single-pair step size, so the update overshoots and
  oscillates. The random-rival ablation fails the same way, which confirms that the step is the
  fault. A Gauss–Seidel or normalized step (divide by the item's collision count) would be the
  first fix. It was not attempted (the PoC was stopped there).

### ASTRO-2 (2026-10-24, `evidence/astro-poc/astro2.txt`, 124 s on 4 threads)

**The check:** plain, K1, no gate and K6 reproduce FOLD-8 exactly: 0.636 / 0.425, 0.894 / 0.407,
0.857 / 0.369 and 0.884 / 0.334.

Dev chose κ 2 and c 0.25. Every virial κ was below plain on dev.

**Test (seeds 10–17):**

| arm | N 512 | N 768 | median item T per round (N 512) |
|---|---|---|---|
| plain | 0.636 | **0.425** | — |
| hard K1 (ρ 0.75) | **0.894** | 0.407 | — |
| hard, no gate | 0.857 | 0.369 | — |
| fixed schedule K6 | 0.884 | 0.334 | 0.0050, 0.0025, 0.0013 |
| virial T_i = κσ_i²/m_i | 0.631 | 0.382 | 0.040, 0.049, 0.053 |
| ablation: one T (round median) | 0.320 | 0.211 | 0.040, 0.072, 0.064 |
| literal virial T_i = c·m_i | 0.818 | 0.363 | 0.0015, 0.0022, 0.0042 |

| # | Result |
|---|---|
| V1 N 512: virial ≥ max(K6, K1) + 0.01 | **FAILED:** 0.631 against 0.894 |
| V2 N 768: virial ≥ plain + 0.02 and ≥ K6 + 0.08 | **FAILED:** 0.382 against 0.425 (it does beat K6's 0.334, by +0.048) |
| V3 virial ≥ one-T ablation + 0.03 at N 768 | MEASURED: 0.382 against 0.211 |
| V4 the median T falls round 1 → 3 | **FAILED:** it rises, 0.040 → 0.053 |
| V5 literal ≤ virial − 0.03 at N 768 | **FAILED:** 0.363 against 0.382 (−0.019) |

**1 of 5 hold.**

**Diagnosis: the derived scale was wrong by about 10×, and the loop never cooled.**
- **The derivation used a two-way runner-up odds** for an item's temperature. But a margin between
  the top two of 256 readings is typically well under σ even for a correct item. So T = κσ²/m came
  out around 0.04–0.08, 8–16× hotter than K6's 0.005.
- **Hot mixtures cancel nothing useful** and add a little noise, so the arm lands just under plain.
- Because nothing is cancelled, the margins do not grow and the temperatures rise instead of
  falling. **The self-cooling feedback (V4) needs a starting point cold enough for cancellation to
  bite,** and this one was not.
- **The per-item binding does carry information** (V3: one shared temperature at the same level is
  far worse). The literal "bound → hot" arm, which dev made cold (c 0.25), behaves like a softened
  hard cancel (0.818).
- **Still open:** a per-item temperature at K6's scale (κ ≈ 0.2), or normalized against the
  255-way maximum instead of the runner-up. The best version can at most trade blows with hard K1;
  **no arm beats plain at N 768,** which is FOLD-8's open problem and is unchanged.

### ASTRO-3 (2026-10-24, `evidence/astro-poc/astro3.txt`, 65 s on 4 threads)

**Test (seeds 10–17), at each law's dev-matched E_ref.** The true rate is given per blob side
[1, 3, 7]; the false rate is the far field's.

| q | E_ref | local (R 1) | Jeans R 3 (dev choice) | Jeans R 5 |
|---|---|---|---|---|
| 0.02 | 1.35 | [0, 0, 1.00], false 0.0000 | [0, 0, 1.00], false 0.0000 | [0, 0, 1.00], false 0.0000 |
| 0.05 | 2.0 | [0, 0, 1.00], false 0.0000 | [0, 0, 1.00], false 0.0000 | [0, 0, 1.00], false 0.0000 |
| 0.1 | 3.0 | [0, 0.042, 1.00], false 0.0013 | [0, 0.021, 1.00], false 0.0012 | [0, 0.021, 1.00], false 0.0012 |

| # | Result |
|---|---|
| J1 Jeans false ≤ 0.5 × local at matched true rate | **FAILED, untestable as set up:** at the matched E_ref the local law's false rate is already < 0.01 at every q (0.0000, 0.0000, 0.0013), so there is nothing to remove |
| J2 side-1 true rate under Jeans ≤ 0.5 × local | HOLDS, but vacuously: both are 0 at every q |
| J3 R 1 reproduces the local law bit for bit | MEASURED: true |

**Diagnosis: the medium is bistable, so a scale law has nothing to act on.** The dev sweep shows a
cliff in E_ref:
- below it, the stir alone carves the **whole** far field (false rate 1.000);
- above it, nothing but the side-7 blobs carves (false rate 0.000).
- There is only a narrow band in between, at q 0.02 and E_ref 0.6: false 0.548 (local), 0.476
  (R 3) and 0.678 (R 5).

False scars here are not tail fluctuations. They come from the bath's **mean** level crossing the
trap law's threshold, and then from the self-reinforcing seal (a carved site leaks 100× less and
fills up). A box mean leaves the mean unchanged, so the Jeans window moves the cliff by almost
nothing. The precommit's derivation assumed tail-driven false scars; that was wrong for this
medium. Matching on side-7 blobs also put both laws above the cliff, where even the local law
refuses side-1 and side-3 writes.

A real test of the Jeans idea needs a medium where the noise is spatially rough and the background
mean sits clearly below threshold (smaller κ, or bursty stir). As it stands, it is **not worth a
full experiment**.

### Summary (ASTRO-POC: 7 of 15 hold)

| experiment | holds | what it settles |
|---|---|---|
| ASTRO-1 Pauli exclusion | 4 of 7 | A, the specified law: matches its derivation to ±0.01; a light-load gain only (+0.029 at N 384), harmful at N 768. B (pressure on colliding states): broken step (oscillates), not refuted |
| ASTRO-2 virial governor | 1 of 5 | The derived temperature scale was ~10× too hot, so the loop never cooled itself; below plain and below hard K1 |
| ASTRO-3 Jeans scar law | 2 of 3 (both vacuous or a code check) | The medium is bistable (the noise carves all or nothing), so a scale law has nothing to act on |

**Worth a full precommitted experiment:** only ASTRO-1 arm B, after its step is normalized. It is
the one arm that attacks the errors themselves: write-time pressure on each item's *colliding*
states, measured on the stored 2-bit state. Arm A is now understood (a closed form predicts it), and
it is not worth more.
