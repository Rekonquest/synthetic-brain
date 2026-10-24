# ENT-1: entanglement as coordination between co-engines (quantum base)

Status: PRECOMMITTED (2026-10-24), before any ENT-1 code.

Jesse (2026-10-24): "lets try an engagement [entanglement] experiment using a quantum base atom
stack". Entanglement's character is already on record (`docs/QUANTUM-BASE-CHARACTERS.md` §8: CHSH
S = 2.8158 against the classical 2; "information lives in the correlation"; a full tap leaves
S = √2). ENT-1 asks what that gives a brain made of co-engines: **can two modules that never exchange
a message coordinate better than any classical pair, by sharing an entangled pair?**

**The claim boundary (vault Rule 4):** this is quantum-derived mathematics on a classical PC, a
four-amplitude register in floats. No quantum computation is claimed. On this machine the "no
message" is enforced by the code's structure: each module's function sees only its own input and its
own half of the register's measurement.

## The task (the CHSH game, as coordination)

Each round, module A gets a private bit x and module B a private bit y (uniform). Each answers a bit,
α and β. They **win if α ⊕ β = x ∧ y**: they must agree, except when both inputs are 1, when they
must disagree.
- **The classical bound:** any strategy without communication, even with shared randomness, wins at
  most 0.75.
- **The quantum bound** (Tsirelson): with a shared Bell pair, cos²(π/8) = 0.8536.

## The stack (one function per crate)

- `qb-amplitude`: the 4-amplitude register (|00⟩, |01⟩, |10⟩, |11⟩).
- **New `qb-local-rotate`:** rotate one qubit of the register by an angle θ in the real plane (a
  local unitary; the other qubit untouched).
- **New `qb-measure-qubit`:** Born-measure one qubit (with a given uniform), collapse the register
  onto the outcome, renormalize, return the outcome. The other qubit's half is left as the collapse
  leaves it.
- **Orchestrator `ent1`:**
  - prepares the Bell pair (|00⟩ + |11⟩)/√2;
  - module A rotates its qubit by −a_x and measures; module B rotates its qubit by −b_y and measures;
  - holds every setting.

Each arm runs 200,000 rounds; seeds as noted.

## Predictions

1. **E1, entanglement beats every classical strategy:** with the textbook angles (a₀ 0, a₁ π/4,
   b₀ π/8, b₁ −π/8), the win rate is 0.8536 ± 0.004, and S = 2√2 ± 0.03.
2. **E2, the bars:**
   - the classical best (both always answer 0) is 0.75 ± 0.004;
   - an unentangled pair (|00⟩) with the same angles is ≤ 0.75.
3. **E3, heat (decoherence):** with probability p per round, the pair is replaced by a random basis
   state (depolarization, via `qb-amplitude::project`). Then S = 2√2(1 − p) and the win rate is
   ½ + S/8.
   - At p ∈ {0, 0.1, 0.2, 0.3, 0.5}, it is within ±0.005 of that law.
   - The quantum edge is gone at p ≥ 0.29.
   - **No inverted U:** cold is best.
4. **E4, a tap:** a third party measures B's qubit in the computational basis before B does. Then
   S = √2 and the win rate is 0.677 ± 0.005, *worse than classical*. A tapped link is detected by the
   coordination failing.
5. **E5, chaos finds the quantum strategy from the classical one.**
   - **Start:** all angles 0. Both modules measure the same basis, so they always agree, which is the
     classical strategy at 0.75. This point is a **saddle**: the win rate's slope is exactly zero
     there, so a plain step never leaves it.
   - **Off (plain):** stays at 0.75.
   - **On:** each step, each of the 4 angles moves by α(u − ½) with α 0.2 rad, u from
     `qb-chaos::keyed_unit` (the logistic map, mean ½).
   - **The selector** is the stack's own win rate on a batch of 4,000 rounds. The move is kept only
     if strictly better (persistence).
   - Over 300 steps, the final angles score ≥ 0.84 on 200,000 fresh rounds in ≥ 7 of 8 test seeds
     (10–17).
6. **E6, the classical twin under the same chaos stays ≤ 0.75 + 0.005.** The same angles drive an
   unentangled pair, with the same chaos and selector. The ceiling belongs to the substrate, not the
   search.

Amplification is not run: for entanglement it means distilling several pairs into one, which this
one-pair stack does not have.

CPU only, one thread, seconds.

## Results (2026-10-24, `evidence/ent-1/results.txt`, 1.3 s on one thread)

| arm | win rate | law |
|---|---|---|
| **entangled pair, textbook angles** | **0.8527** (S 2.822) | 0.8536 (S 2.828) |
| classical best | 0.7501 | 0.75 |
| unentangled pair, same angles | 0.6772 | ≤ 0.75 |
| heat p 0.1 / 0.2 / 0.3 / 0.5 | 0.8178 / 0.7837 / 0.7469 / 0.6777 | 0.8182 / 0.7828 / 0.7475 / 0.6768 |
| tapped | 0.6775 | 0.6768 |

**Chaos from the classical saddle** (fresh 200,000-round scores, test seeds 10–17):

| seed | plain | chaos on | classical twin under chaos |
|---|---|---|---|
| 10 | 0.7494 | 0.8465 | 0.7463 |
| 11 | 0.7521 | 0.8497 | 0.7469 |
| 12 | 0.7486 | 0.8523 | 0.7470 |
| 13 | 0.7491 | 0.8510 | 0.7474 |
| 14 | 0.7513 | 0.8512 | 0.7506 |
| 15 | 0.7513 | 0.7501 (stayed on the saddle) | 0.7512 |
| 16 | 0.7491 | 0.8520 | 0.7474 |
| 17 | 0.7501 | 0.8474 | 0.7466 |

**6 of 6 hold.**

### What it settles

- **Two co-engines that share an entangled pair coordinate better than any classical pair can,
  with no message between them:** 85.3% against the classical ceiling of 75%.
- **Heat erodes it exactly by the law** (S = 2√2(1 − p)). The edge is gone at p ≈ 0.3, and cold is
  best (no inverted U, as in FOLD-5).
- **A tap is detectable by the coordination itself:** a tapped link scores 67.7%, *below* classical.
- **Chaos discovers the quantum strategy on its own,** from the classical one, in 7 of 8 seeds.
  - The classical strategy is a saddle where the win rate's slope is zero, so a plain step never
    leaves it.
  - The angles found differ from the textbook's but are equivalent (the same relative angles,
    rotated or mirrored).
  - Seed 15 drifted too little to leave the saddle in 300 steps.
  - The classical twin under the same chaos never passes 75%, so the gain belongs to the substrate,
    not the search.
- **For the brain:** entanglement is a way for co-engines to agree without talking, and its
  failure flags interference. This is Rule 4's boundary again: on this PC it is the mathematics, and
  the separation is enforced by code structure.
