# QB-6: amplification on and off, one module at a time

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

Jesse's build rule tests every stack with amplification on and off. His definition (2026-10-18):

> amplification "is a general generic term, it can be used to make a signal stronger or make more of
> something that already exists, like growth."

So each module gets an amplifier in one or both senses:
- **gain:** a stronger signal;
- **growth:** more of what exists.

Each is tested on against off, under QB-5's protocol:
- the module's own check selects on the development seeds, and plain wins ties;
- the test seeds report on against off;
- the winner is kept only if it beats off on the check on the test seeds;
- each round starts from the kept best.

**The base is QB-5's kept system:** T₀ 0.1, amplitude chaos α 0.05 key 2; 0.889 on test.

## The amplifiers, per module

| Round | Module | Sense | Amplifier | Levels | Check |
|---|---|---|---|---|---|
| 1 | D, detector | gain | each *repeat* vote in the window counts G (the current observation counts 1); fires at ≥ k − ½ | G ∈ {1.2, 1.5, 2.0} | D's F1 |
| 2 | M, memory | growth | the held item's amplitude ×G each step, then renormalize | G ∈ {1.5, 3} | M's fidelity |
| 3 | M → T link | gain | η_m × G (the memory's signal strength into the tracker) | G ∈ {2, 4, 8} | acc_resp |
| 4 | T, tracker | gain | the evidence reinforcement η × G | G ∈ {1.5, 2, 3} | acc_resp |
| 5 | T, tracker | growth | catalysed reinforcement (Michaelis–Menten): factor 1 + η·(1 + κ·p_o/(p_o + 0.1)), p_o the tracker's Born weight on the observed option | κ ∈ {0.5, 1, 2} | acc_resp |

The link's job is to carry the memory into the decision, so its check is the system's acc_resp.

## Predictions, from the characters and the earlier runs

1. **A1, D's gain is not kept.**
   - A gain on repeats is a lower threshold: at G ≥ 1.5, one repeat fires, which acts as k = 2.
   - A conjunction suppresses iid noise as p^k, so lowering k raises false events faster than it
     recovers missed cues. Recall is already high (cue p = 0.9).
   - F1 does not improve.
2. **A2, M's growth is a dead stage (it ties exactly).** A collapsed memory is one-hot, already at full
   strength. Growing its only non-zero amplitude, then renormalizing, changes nothing (QB-1 P1, QB-5
   P2).
3. **A3, the link's gain is not kept.** QB-4's tuning already chose the smallest η_m on its grid.
   Amplifying the memory pushes the system toward memory-only (QB-4: 0.842), which is weaker than the
   combination.
4. **A4, T's gain is kept.** A stronger reinforcement widens the gap in log-weight between target and
   noise (∝ ln(1 + η)/λ), which acts like further cooling. QB-5's cooling gain sat at the colder edge
   of its grid, so more separation should still help. Prediction: kept, with test acc_resp ≥ +0.003.
5. **A5, T's growth is kept.** Rich-get-richer commits faster to what the evidence already favours,
   and the memory handles the switch, so slower switching costs less. It was load-bearing in QB-3's
   diagnostics (+0.128). Prediction: kept, with test acc_resp ≥ +0.003.

**Kill criterion:** if 3 or more of A1–A5 fail, the characters do not predict amplification at the
module level.

## Build

- `qb6` reuses `qb5`'s runner and checks, with the five amplifiers added; each is off at G = 1 or
  κ = 0.
- **Unit tests:**
  - with every amplifier off, `qb6` reproduces QB-5's kept system (test 0.889);
  - M's growth leaves a one-hot state unchanged;
  - D's gain at G = 1 equals the plain gate.

## Results (2026-10-18, `evidence/qb-6/results.txt`; diagnostics in `results-with-diagnostics.txt`)

Tests passed before the run:
- with every amplifier off, the runner reproduces QB-5's 0.889;
- growth leaves a one-hot state unchanged;
- gain 1 equals the plain gate.

| Round (module check) | dev: off → best | test: off → on (check / acc_resp) | verdict |
|---|---|---|---|
| 1 D gain (F1) | 0.9585 → 0.9585 (G 1.2 ties; G 1.5 and 2 give 0.7643) | 0.9526 / 0.889 → unchanged | plain wins |
| 2 M growth (fidelity) | 0.8857 → 0.8857 (both tie exactly) | unchanged | plain wins (dead stage) |
| 3 M → T link gain (acc_resp) | 0.9078 → 0.9078 (G 2 / 4 / 8: 0.8913 / 0.8840 / 0.8814) | unchanged | plain wins |
| **4 T gain (acc_resp)** | 0.9078 → **0.9271** (G 1.5 / 2 / 3: 0.9193 / 0.9251 / 0.9271) | **0.889 → 0.915** | **KEPT (G 3)** |
| 5 T growth (acc_resp), on top of the kept gain | 0.9271 → 0.9271 (κ 0.5 / 1 / 2: 0.9163 / 0.9001 / 0.8659) | unchanged | plain wins |

| # | Result | What was measured |
|---|---|---|
| A1 D gain not kept | MEASURED | G ≥ 1.5 acts as k = 2: F1 falls 0.9585 → 0.7643 (false events) |
| A2 M growth is a dead stage | MEASURED | ties exactly, on dev and on test |
| A3 link gain not kept | MEASURED | every gain is below plain |
| A4 T gain kept, ≥ +0.003 | MEASURED | kept at G 3: **+0.026** on test |
| A5 T growth kept, ≥ +0.003 | **FAILED** | on top of the kept gain, every κ is below plain |

**4 of 5 hold; the kill criterion (3) does not fire.**

### Diagnostics (added after the run, not predictions)

| Diagnostic | dev | test |
|---|---|---|
| growth alone (κ 0.5 / 1 / 2) on QB-5's base, without the gain | 0.9144 / 0.9178 / 0.9115 | 0.8967 / **0.9014** / 0.8994 |
| T gain beyond the grid (G 4 / 6) | 0.9209 / 0.9067 | 0.9121 / 0.8992 |
| modular-*classical* with η × 1.5 / 2 / 3 (λ and η_m retuned on dev) | — | 0.901 / 0.911 / **0.917** |

### What QB-6 shows

1. **Amplification is real here, and it is the largest single gain since the modular redesign.**
   Amplifying the tracker's evidence ×3 takes the test score from 0.889 to 0.915. G 3 is near the
   edge: G 4 is lower on dev, and G 6 is the "too much" point.
2. **A5's character was right; its position was wrong.** Growth helps on its own (+0.012, κ 1). On
   top of the gain it overshoots. **Two amplifiers of the same thing (both strengthen the leader)
   substitute; they do not add.** This is the amplifier version of the substrate's
   "maintainers interfere" law (C16, C22).
3. **Amplification helps both physics equally (law 5 again).** Classical modules given the same
   evidence gain reach 0.917, against the quantum system's 0.915. The gain belongs to the
   amplification, not to the quantum base. The honest scoreboard is a tie.
4. **Where amplification does not belong:**
   - on a threshold, the detector's gain acts as a lower threshold, which lets noise through;
   - on a saturated memory, there is nothing left to grow;
   - on the memory link, a stronger memory signal pushes the system toward memory-only, which is weaker.

### Next

- Jesse's full matrix: amplification × cognitive, on and off, together and alone, with chaos last.
- Amplification in a *growth* role where it has a job: a world where the cue is sometimes missing, so
  weak evidence has to be grown into a decision. The same task would give hot/cold an exploring job.
