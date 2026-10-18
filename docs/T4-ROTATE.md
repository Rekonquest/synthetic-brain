# T4: rotating a shape held in the medium (mental rotation)

Status: PRECOMMITTED (2026-10-18), before any code.

## Jesse's point, and the anchor

"When I think, I can see the shapes of my ideas, I can rotate those shapes in my thoughts." Shepard &
Metzler (1971) asked people whether two 3D cube-chain shapes were the same (one rotated) or mirror
images. **The decision time grew linearly with the angle between them:** the mind rotates through the
intermediate angles. T4 makes the medium do the same task.

## The stack

`thermo-rotate` advects the 3D field by a rigid rotation about the vertical axis through the lattice
centre:
- semi-Lagrangian, each site taking the trilinear-interpolated value from where the flow brings it;
- Δθ = 3° per step;
- the total is renormalized each step (interpolation conserves it only approximately);
- the axis count is not in the law: the rotation acts in the plane of two axes and carries the rest.

**The shapes:**
- Shepard–Metzler-like chains of 8 cubes (cube size 2 lattice units), with random bends;
- each rendered as Gaussian splats (σ 0.8) at its voxel centres;
- **chiral only:** a shape whose mirror image equals one of its 90° rotations about the vertical axis
  is rejected.

**The trial:** a reference A, and a target B that is either A rotated by θ (**same**) or the mirror of A
rotated by θ (**different**), with θ ∈ {0°, 30°, …, 180°}. B is rendered exactly, by rotating the voxel
centres and then splatting.

- **The medium rotates A** forward through 180°. At each step it takes the cosine similarity with B.
- **It answers "same"** when the similarity reaches the threshold s*. The threshold is chosen on the
  development shapes as the midpoint between the mean maxima of same and different pairs.
- **The response time** is the number of steps to reach s*.

**Sized for expansion (the rule from T1):**
- a lattice of 32³;
- the shape's centroid at the centre, and its radius ≤ 9;
- a check that no energy above 10⁻⁶ of the peak ever lies farther than L/2 − 3 from the axis during
  any rotation.

**Seeds:** development shapes 0–9, test shapes 100–119.

## Its own checks (predictions, on the test shapes)

1. **R1, the shape survives rotation:** the mean maximum similarity for *same* pairs is ≥ 0.90 at every
   θ.
2. **R2, Shepard–Metzler:** the response time for *same* pairs is linear in θ (R² ≥ 0.98), with a slope
   within 10% of 1 step per 3°.
3. **R3, same and different are told apart:** accuracy ≥ 90% over all pairs and angles.
4. **R4, the mesh holds the rotation** (Jesse's sizing rule): the radius check above holds for every
   trial.

## Optimization round

**The check:** R3's accuracy on the development shapes, with the mean same-pair maximum similarity as
the tie-break. Plain wins ties, and a variant is kept only if it is better on the test shapes.

- **Amplification on and off:** contrast sharpening after each step, e ← e^γ renormalized,
  γ ∈ {1.03, 1.08}. This counteracts the advection's blur.
  - **Prediction R5:** kept, because it raises the large-angle similarity.
- **Hot and cold:** the bath's conserved noise on the held field, T ∈ {10⁻⁷, 10⁻⁶} (the field peaks at
  about 0.05).
  - **Prediction R6:** not kept. A held shape has no competition for heat to break; noise only
    corrupts it. T3's heat helped because threads compete, and nothing competes here.
- **Chaos on and off:** a mean-preserving jitter of the angular step, 3°·(1 + α(x − ½)), α ∈ {0.1, 0.3}.
  - **Prediction R7:** not kept. Rotation needs a steady rate; jitter breaks R2's linearity, and it
    cannot raise accuracy.

## Run 1: results (2026-10-18, `evidence/thermo-4/results-run1-bilinear.txt`, 7 min 24 s)

| θ | 0 | 30 | 60 | 90 | 120 | 150 | 180 |
|---|---|---|---|---|---|---|---|
| same, mean max similarity | 1.000 | 0.971 | 0.935 | 0.901 | 0.873 | 0.849 | 0.850 |
| mirror, mean max similarity | 0.821 | 0.850 | 0.837 | 0.827 | 0.822 | 0.816 | 0.839 |

| # | Result |
|---|---|
| R1 the shape survives rotation (≥ 0.90 at every θ) | **FAILED**: 0.849 at 150° |
| R2 RT linear, slope within 10% | **FAILED**: R² 0.654, slope 0.215 steps/° (want 0.333) |
| R3 accuracy ≥ 0.90 | **FAILED**: 0.707 (s* 0.881) |
| R4 the mesh holds the rotation | **FAILED**: radius 19.9 against the limit 13 |
| R5 amplification kept | **FAILED**: γ 1.03 / 1.08 lower dev accuracy (0.586 / 0.536) |
| R6 heat not kept | holds, but **VOID** (wiring confound, below) |
| R7 chaos not kept | holds (test unchanged) |

**2 of 7 hold.**

### Diagnostic, after the run (not a prediction; `evidence/thermo-4/diagnostic-run1-fidelity.txt`)

Fidelity is the cosine between the held field after t steps and the exact render at t·3°, over dev shapes 0–3.

- **The rotation is right:** the same-pair maximum at 90° falls at step 30 (29 for one shape), the
  aligned step, rising from a step-0 similarity of 0.25–0.76.
- **The advection blurs:** fidelity falls with every step. At 30° it is 0.964–0.988; at 180° it is
  0.800–0.903.
  - Semi-Lagrangian bilinear interpolation adds roughly fx(1 − fx) of variance per step. Far from
    the axis a 3° step moves a site by about half a cell, so a σ 0.8 shape is smeared within tens of
    steps.
  - **The same-pair maximum is exactly this fidelity curve.** Once the curve falls below the
    mirror's ≈ 0.83, same and mirror cannot be told apart, which explains the failures of R1, R2 and
    R3 together.
- **R4:** the pointwise "10⁻⁶ of the peak" radius measures the same smearing, in the tails. It is not
  a size limit being reached.
- **R5:** sharpening cannot undo blur, because it distorts the profile as well.
- **R6 is VOID:** the heat arm called `exchange_edges` with κ 0.05. That adds deterministic diffusion,
  which plain lacks, besides the noise. Its 0.579 / 0.851 is diffusion, not temperature.
- **The scale:** the render's peak is about 6, not the precommit's 0.05, so T 10⁻⁷ / 10⁻⁶ were far
  weaker than intended.
- **Also:** the precommit's shape radius ≤ 9 was not enforced.

**The miss is mine:** I chose a diffusive advection for a sharp field. Chaos (R7) did not help, and
nothing it could select would restore lost detail. By Jesse's order (chaos → compose from outside →
rewrite), the next step composes a non-diffusive rotation from outside. `thermo-rotate` stays as the
run-1 record.

## Run 2: PRECOMMITTED (2026-10-18), before any run-2 code

**The new stack, `thermo-shear`:**
- shifts every line along axis a by s·(coord_b − c_b), with the shift **band-limited** (Fourier: each
  line's DFT is multiplied by e^{−ikδ});
- a Fourier shift conserves Σe exactly and loses no detail except the Nyquist bin (cos(πδ) there keeps the line real; corrected when the unit test measured Σe² −2.3e-10 relative, before any run);
- the radix-2 FFT is written in the crate, with zero dependencies.

**The orchestrator** composes rotation from three shears (Paeth 1986, Unser et al. 1995): shear_a by
−tan(θ/2), then shear_b by sin θ, then shear_a by −tan(θ/2). This is exact for band-limited fields.

**The repairs:**
- shapes are rejected unless every sub-voxel centre lies within 9 of the centroid;
- the render is scaled so its peak is 0.05, as the precommit stated (cosine is unchanged);
- the heat arm is **noise only:** a conserved kick f = √(T/2)·ξ on every edge, i.e. the bath's
  κ(2 − κ)/2 at κ = 1, with no deterministic flux;
- amplification is sign-preserving, sign(e)·|e|^γ, because Fourier fields ring slightly negative and
  clipping would be a hidden filter.

**R4 rewritten (labelled change):** the share of Σe² lying farther than L/2 − 3 from the axis stays
below 10⁻⁶ in every trial. A pointwise threshold counts any interpolation's tails; the share is what
"the shape reaches the edge" means.

**Sized for expansion, both ways:**
- **resolution:** σ 0.8 still carries about 4% of its spectrum at Nyquist, which a Fourier shift turns
  into ringing;
- **so, R8, convergence:** the base check is repeated at **side 64, cube size 4, σ 1.6**, the same
  shape at twice the resolution. Test accuracy must move by ≤ 0.03 and the same-pair mean maximum by
  ≤ 0.01 against side 32. The rounds run at side 32.

**Predictions** (test shapes, side 32). The precommitted bars are unchanged; R5's direction is
re-derived:
- R1: ≥ 0.90 at every θ. Fidelity after 180° should be ≥ 0.99.
- R2: R² ≥ 0.98, slope within 10% of 1/3. The crossing comes a fixed few steps before alignment, and
  the θ = 0 point is clipped at 0.
- R3: accuracy ≥ 0.90.
- R4: holds.
- **R5: amplification not kept.** Run 1's rationale (counteracting blur) is gone, and sharpening
  distorts a faithful shape.
- R6: heat not kept.
- R7: chaos not kept.
- R8: converges.

## Run 2: results (2026-10-18, `evidence/thermo-4/results.txt`, 3 min 41 s on 24 threads)

The tests passed before the run:
- an integer shear is an exact roll;
- a fractional shear conserves Σe to 10⁻¹² and Σe² up to the Nyquist share;
- three shears rotate a σ 2 blob by 40° to within 1.6·10⁻⁶ of a unit peak.

**Fidelity** to the exact render, dev shapes 0–3: **0.99997–0.99999** at 60°, 120° and 180°. Run 1 had
0.80–0.90 at 180°.

| θ | 0 | 30 | 60 | 90 | 120 | 150 | 180 |
|---|---|---|---|---|---|---|---|
| same, mean max | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| mirror, mean max | 0.824 | 0.851 | 0.839 | 0.839 | 0.842 | 0.837 | 0.855 |
| mean RT (steps), s* 0.938 | 0.00 | 5.60 | 15.45 | 25.45 | 35.45 | 45.45 | 52.60 |

| # | Result |
|---|---|
| R1 the shape survives rotation | **MEASURED**: 1.0000 at every θ |
| R2 RT linear, R² ≥ 0.98, slope within 10% | **FAILED**: R² 0.924, slope 0.307 (−8%, inside the band) |
| R3 accuracy ≥ 0.90 | **FAILED**: 0.886 |
| R4 outer energy share < 10⁻⁶ at side 32 | **FAILED**: 1.06·10⁻⁵ (at side 64: 3.8·10⁻¹²) |
| R5 amplification not kept | MEASURED: γ 1.03 / 1.08 give dev 0.686 / 0.543 against plain 0.829 |
| R6 heat not kept | MEASURED: T 10⁻⁷ / 10⁻⁶ give dev 0.521 / 0.514 |
| R7 chaos not kept | MEASURED: α 0.1 / 0.3 tie on accuracy and lower the same-max (0.9999 / 0.9996) |
| R8 converges at side 64 | **MEASURED**: accuracy 0.886 → 0.886, same-max 1.0000 → 1.0000, every RT identical |

**5 of 8 hold.**

### The three misses, from a diagnostic after the run (`evidence/thermo-4/diagnostic-run2-per-shape.txt`)

**R2 and R3 are the stimuli, not the medium.** All 32 errors are mirror pairs passing s*, and they
fall on 8 of the 20 shapes.

| shape | mirror pairs passing s* |
|---|---|
| 111 | 7 of 7 |
| 100, 110, 112 | 6 of 7 each |
| 105 | 3 |
| 119 | 2 |
| 108, 116 | 1 each |

- These chains' mirror images are within cosine 0.95–0.98 of a rotation of the original.
- My chirality filter rejects only an *exact* mirror-equals-a-90°-turn. Shepard and Metzler
  hand-picked shapes whose mirrors no rotation approaches; my random chains include near-achiral ones.
- **R2's bend comes from the same shapes and from the clip at 0°:**
  - shape 111 is exactly C2-symmetric about the vertical axis (its self-similarity at 180° is
    1.0000), so its 180° target is matched at step 0;
  - shapes 110, 119 and 105 are partly so, with RT 47, 50 and 53 at 180°;
  - and at θ = 0 the RT cannot go below 0, against the fitted offset of −4.55.
  - **From 60° to 150° the RT is exactly linear:** 15.45, 25.45, 35.45, 45.45, which is 1 step per 3°
    with a constant 4.55-step lead (the similarity crosses s* about 14° before alignment).

**R4 is resolution.** At σ 0.8 the splats carry about 4% of their spectrum at Nyquist, and the
Fourier shear rings it out to the edge, at 10⁻⁵. At side 64 (σ 1.6), the same shape leaves
3.8·10⁻¹² there. The mesh was wide enough but too coarse. **Jesse's sizing rule has a second half:**
a mesh must be large enough in *extent* (T1) and fine enough in *resolution* (T4) for the operation
done on it.

### What T4 settles

- **The medium can hold a 3D shape and turn it with no loss**, through any angle, in 3° steps. The
  rotation is composed of three band-limited shears, a law written once for any plane in any
  dimension.
- **The time to match grows linearly with the angle, 1 step per 3°.** That is Shepard–Metzler's
  signature, exact wherever the stimulus has no self-symmetry.
- **Rotation is a conservative operation:**
  - heat only corrupts it;
  - sharpening distorts it;
  - chaos on the rate only blurs the peak.

  Plain wins all three rounds. Contrast T3, where carving is a competitive operation and heat helped.
- **Carried up:** Fourier three-shear rotation. The resolution σ ≥ 1.6 sites for shapes that will be
  rotated. Stimuli need a mirror margin, not just exact chirality.
