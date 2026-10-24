# GEO-2: physics composed with neuroscience, chemistry and cognitive axioms

Status: PRECOMMITTED (2026-10-24), before any GEO-2 code.

Jesse (2026-10-24): "try adding these with neuroscience and cognitive, chemistry axioms, because these
can help a stack perform better. I think it shouldn't be limited to just cognitive engines or direct
synthetic brain."

GEO-1's G1 left two gaps:
- the Noether charges identify a shape in one comparison (0.999) but call its handedness at only
  0.845, and fall to 0.776 under heavy noise;
- the mental-rotation bar is exact but costs about 69 angle steps × 32 shapes per query.

GEO-2 composes the physics stack with a law from each other domain. The rules are as before:
- no existing stack reused, and the bars are only bars;
- law, then chaos, then amplification;
- niches recorded;
- f64;
- test seeds 10–17, dev 0–3;
- the task, library and queries exactly as G1.

## The composed stacks

- **Neuroscience: the grid-cell phase code.** Grid cells encode position as phases in modules of
  different periods; together the modules fix a position no single module can. Here:
  - the rotation charges are the modules: the phase of m₂(query)/m₂(shape) gives 2θ mod 2π, and the
    phase of m₃ gives 3θ mod 2π;
  - the pair (n₂, n₃) of branch choices that agree best gives θ mod 2π, their circular mean the
    estimate;
  - **the angle is read in one step, with no search.** For each of the top 3 shapes by Noether
    distance, in both mirror states, rotate by the read angle and verify by one Chamfer distance:
    6 Chamfer evaluations per query.
  - New crate `phase-angle`.
- **Chemistry: the Cahn–Ingold–Prelog priority rule.** Chemistry names a molecule's handedness by
  ranking the groups around a centre by priority (atomic number, then the next sphere out to break
  ties) and reading the turn of the top three.
  - Here the priority is a point's distance from the centroid, with ties broken by its sorted
    distances to the other points.
  - Handedness is the turn of the top three points in priority order (the sign of their signed
    area), compared with the shape's.
  - New crate `shape-cip`.
- **Cognitive: dual process with a PERCEPTION gate.**
  - System 1 is Noether identity plus the CIP handedness.
  - The named hypothesis "uncertain" cites two readings: the Noether margin (second-best over best
    distance below g, chosen on dev) and the two handedness readings (CIP and Im(m₂³·m̄₃²))
    disagreeing.
  - If it holds, System 2 runs the grid-cell verify. If that verify's Chamfer distance is still
    above τ (G1's 0.05), the full search runs.
  - Cost is counted in Chamfer evaluations against one shape (the search costs its steps × 32).

## The arms

A0 Noether alone (G1). A1 Noether + CIP handedness. A2 Noether + grid-cell verify. A3 dual process.
The bar is the search.

- **Chaos:** in A2, 3 variants of the read angle, each moved by 10°·(u − ½) (keyed logistic,
  mean-preserving). Chamfer selects; plain wins ties.
- **Amplification:** more grid modules, charges 2, 3, 4 and 5, for the angle read.

## Predictions

1. **N1:** A2 identity + mirror ≥ 0.99 at noise 0.02 and ≥ 0.95 at 0.05, with 6 Chamfer evaluations
   against the search's about 2,200.
2. **C1:** CIP handedness ≥ the Noether pseudo-scalar's + 0.05 at noise 0.02 (0.845).
3. **D1:** the dual process is within 0.01 of the search's identity + mirror at every noise level,
   at ≤ 10% of its cost.
4. **X1:** chaos on the read angle is kept at noise 0.05 and not at 0.01 (the read angle is noisier
   under heavy noise, and Chamfer is a real selector).
5. **X2:** more modules raise A2 at noise 0.05 by ≥ 0.01.

New orchestrator `geo3`. CPU, seconds.
