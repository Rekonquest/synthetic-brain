---
name: physics-axiom-stacks
description: How to turn laws of theoretical physics, astrophysics and black-hole physics into working stacks (mechanisms) and which mappings proved load-bearing in Jesse's synthetic-brain work - Noether conserved charges, least action as a sum over paths, gauge-invariant (Wilson-line) reading, topological winding, string modes, quantum Zeno, the virial self-set temperature, Jaynes maximum entropy, holography, no-hair consolidation, the Roche limit, mass segregation, resonance binding. Use whenever Jesse asks to "try physics axioms", "quarks", "strings", "black holes", "astrophysics", "theoretical physics", "is it load bearing", or wants a new mechanism for memory, recall, perception, planning, calibration or caching; and whenever designing a stack where a physics law might be the mechanism. Includes the mapping method, the measured catalogue, the laws still unmapped, and the honesty rule (a load-bearing law proves a mechanism, not the theory).
---

# Physics laws as stacks

Jesse (2026-10-24): "if they are load bearing, then we know there is a there, there" and "keep
reaching for theoretical physics axioms, because it's turning out to be rather fruitful".

## The honesty rule (say it every time)
A law that is load-bearing in a stack proves **its mathematics works as a mechanism**. It does not
test whether the theory is true of nature (epicycles predicted planets). Say this when reporting.

## How to map a law (before code)
1. Name the law's **state, invariant and dynamics** (what it conserves, what it extremizes).
2. Find the stack whose *input and output types* match (Rule 2): a conserved quantity → an
   invariant reading; an extremum → a planner or decoder; a phase → interference; a
   threshold or bistability → a gate.
3. Pick a **status-quo bar** (an existing method) and a falsifier (an ablation).
4. Derive the prediction from the law's character, not from a classical analogy (derivations from
   analogy failed repeatedly). Fix derivations *before* code; record the fix.
5. Add chaos (two maps, two positions) and a real amplification arm (more of what exists).
6. Record the **niche**: what it is best at (see `hybrid-specialists`).
7. If no clean mapping exists, record the law as unmapped instead of forcing an analogy.

## Measured catalogue (synthetic-brain, all f64, 2026-10-23/24)
| law | stack | result |
|---|---|---|
| **Noether** (symmetry ↔ conserved charge) | complex moments m_k = Σz^k carry rotation charge k; neutral products are invariant; Im of a neutral product flips under reflection | shape identity in one comparison (0.999); weak alone at handedness (0.845) |
| Noether **+ grid-cell phase code** (neuroscience) | each charge is a phase module (k branches); together they read the rotation angle in one step; verify once | identity + handedness 1.000 with 6 checks vs a search's ~2,200; more modules resist noise |
| **least action = Feynman sum over paths** | relax u_s = (λ/4)Σ neighbours (goal = 1), climb u | 100% path success within 0.14% of optimal (greedy 1.8%) |
| **gauge invariance** (U(1), QCD's principle) | read through Wilson lines Re Σ u_i·conj(u_{i+1}) | memory/perception immune to smooth drift: 1.000 vs 0.02 |
| **topological charge** | winding number of a ring of phases | noise-proof bit (1.000 vs 0.53); exact counter within capacity (∝ ring length); refuses smooth changes |
| **closed-string modes** | harmonics as memory / lanes | 32 bits at 1.000; 8 parallel lanes at 0.98 |
| **quantum Zeno** | frequent measurement | attention holds a state (0.16 → 0.96); classical twin flat |
| **virial theorem** (self-set temperature T_i = κσ²/m, κ≈0.1) | per-item temperature in an explaining-away recall loop | best recall stack: 0.904, 0.966 with rounds × 2; cools itself |
| **Jaynes maximum entropy** | T = σ²/μ measured by the store (noise from unwritten keys, signal by method of moments) | calibrated probabilities with no labels (ECE 0.015 vs tuned softmax 0.044) |
| **holography** | spread every memory over all regions | survives losing a region (0.96 vs 0.75) |
| **black-hole no-hair** | consolidate old memories to their class | new recall 0.65 → 0.94 |
| **Roche limit** | keep weak memories away from strong ones in a quantized store | weak recall 0.79 → 1.00 |
| **mass segregation** (dynamical friction) | decaying use-mass; heaviest settle into a fast core | 21% cheaper than LRU |
| **orbital resonance + binding by synchrony** | phase-locking to an object's drive | binding 0.88–0.91 vs 0.04 |
| **Pauli exclusion** | write-time pressure on colliding states (push ÷ collisions) | +0.09–0.10 at light/moderate load |
| colour confinement | closed-loop consistency check | error flag (0.928 → 0.956 precision) |
| asymptotic freedom | free inside r₀, constant pull beyond | concepts bound with 1.77× inner freedom |
| weak or failed | Jeans (bistable medium), lensing (+0.02), renormalization group (cost 0.44, not 0.25), merge tree (0.72), Born as probability (miscalibrated) | — |

**Unmapped so far:** T-duality, supersymmetric cancellation, black-hole area law, fast scrambling
(the fold already scrambles), Eddington limit, inflation, recombination.

## Patterns that recur
- **Neutral combinations are invariants** (colour confinement, gauge invariance, Noether charges
  are one idea): combine charged quantities so the charges cancel.
- **Self-measured temperatures beat hand-set ones** (virial in recall, maximum entropy in
  probability).
- **Spread beats local** for damage (holography, folding) — and costs space.
- **A tipping point appears in iterative cancellation** past ~60% first-read accuracy; gate it.

Evidence: `~/Desktop/Projects/synthetic-brain/docs/` PHYS-POC, PHYS2-POC, PHYS3-POC, ASTRO-R,
GEO-1, GEO-2, PROB-1, FOLD-8, FOLD-9, and `docs/AXIOM-ATLAS.md`.
