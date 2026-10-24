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

## Results (2026-10-24, `evidence/geo-2/results.txt`, under 1 s)

Dev chose gate 1.5 (dev identity + mirror 0.964–0.966 at every gate; the gate barely mattered, see
D1).

| arm (identity / identity + mirror / Chamfer evaluations) | noise 0.01 | 0.02 | 0.05 |
|---|---|---|---|
| search (bar) | 1.000 / 0.998 / 2,198 | 1.000 / 1.000 / 2,217 | 0.995 / 0.987 / 3,875 |
| Noether alone | 1.000 / 0.911 / 0 | 0.999 / 0.845 / 0 | 0.776 / 0.544 / 0 |
| Noether + CIP (chemistry) | 1.000 / 0.864 / 0 | 0.999 / 0.757 / 0 | 0.776 / 0.465 / 0 |
| grid-cell verify, 2 modules (neuroscience) | 0.995 / 0.995 / 6 | 0.986 / 0.981 / 6 | 0.875 / 0.850 / 6 |
| dual process (cognitive) | 1.000 / 0.989 / 9 | 1.000 / 0.968 / 56 | 0.993 / 0.958 / 2,974 |
| verify + chaos | 0.995 / 0.995 / 24 | 0.984 / 0.980 / 24 | 0.881 / 0.857 / 24 |
| **grid-cell verify, 4 modules (amplified)** | **1.000 / 1.000 / 6** | **1.000 / 1.000 / 6** | **0.916 / 0.903 / 6** |

Handedness on correctly identified shapes: CIP 0.864 / 0.758 / 0.599; the Noether pseudo-scalar
0.911 / 0.846 / 0.700.

| # | Result |
|---|---|
| N1 grid-cell verify ≥ 0.99 at 0.02, ≥ 0.95 at 0.05 | **FAILED with 2 modules** (0.981, 0.850). With 4 modules (X2) it is 1.000 at 0.02 and 0.903 at 0.05 |
| C1 CIP handedness ≥ Noether + 0.05 | **FAILED, reversed:** CIP 0.758 against 0.846. Noise reorders the priority ranking, and a wrong rank gives the wrong hand |
| D1 dual process within 0.01 of the search at ≤ 10% cost | **FAILED:** the two handedness readings disagree on about 20% of queries, so the gate opens often, and at noise 0.05 most of those fall through to the full search |
| X1 chaos kept at 0.05, not at 0.01 | MEASURED: 0.850 → 0.857 at 0.05; a tie at 0.01 |
| X2 4 modules raise verify at 0.05 by ≥ 0.01 | MEASURED: 0.850 → 0.903, and 1.000 at 0.01 and 0.02 |

**2 of 5 hold.**

### What it settles

- **Physics plus neuroscience matches the search at a fraction of its cost.** Noether's conserved
  charges pick the candidates; the grid-cell phase code reads the rotation in one step from those
  same charges; one Chamfer check confirms.
  - With 4 modules: **identity and handedness at 1.000** at noise 0.01 and 0.02, with **6
    evaluations against the search's about 2,200** (about 370× fewer).
  - Like grid cells, each module alone is ambiguous (charge k gives k branches); together they fix
    the angle, and more modules resist more noise.
- **Chaos is kept for the first time in a non-carving stack, and only where predicted:** a real
  selector (Chamfer) on a noisy reading at high noise (+0.007), and not at low noise.
- **The chemistry rule loses here.** The CIP priority ranking is fragile under noise. Chirality from
  a ranking is less robust than chirality from a conserved charge.
- **The dual process needs a better gate.** Two handedness readings that disagree 20% of the time
  make a poor doubt signal. The next version should gate on the verify's own Chamfer distance, with
  the 4-module verify as System 2.

### Niches

- **Grid-cell verify with 4 modules:** recognition with handedness under rotation, at constant tiny
  cost; the best of everything measured.
- **Noether alone:** identity at zero verification cost when handedness is not needed.
- **The search:** the robust last resort at heavy noise (0.987 at noise 0.05).
- **CIP:** none here; it may suit noise-free structures, as in chemistry.
