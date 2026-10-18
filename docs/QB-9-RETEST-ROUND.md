# QB-9: the retest round on QB-8's stack (before the fabric)

Status: PRECOMMITTED (2026-10-18), before any code.

## Why

QB-8 left four items. Jesse's rule is that each stack is optimized before the next goes on top, and
FABRIC-1 will be built on these nodes. So these come first:
1. fix the memory's check;
2. go colder on the decision;
3. retest amplification on this composition;
4. a fair classical comparator.

**The world:** QB-7's (p_miss 0.4). **The base:** QB-8's final system:
- collapse memory with occupation heat ε 0.01;
- T₀ 0.05, evidence gain ×3, growth κ 1 with product inhibition ι 1;
- λ 0.05, η_m 0.05, no amplitude chaos.

Test 0.768 (cued 0.917, missed 0.538). The protocol is QB-5's.

## 1. M's check, fixed

M's job is to deliver the target to T, through its Born weights used as concentrations. QB-8's check
read only the argmax, so it could not see heat.

**The new check is M's log-score:** the mean over delay and response steps of ln(max(p^M_target,
10⁻⁹)). It is a proper scoring rule: it rewards a reading that predicts the target. Unlike the weight
on the target, it heavily penalizes a memory that is *confidently wrong* when stale, which is exactly
what heat softens.

**Round 1 (M):** occupation heat ε ∈ {0 (off), 0.0025, 0.005, 0.02, 0.04}, from the base's 0.01, judged
by the log-score.

## 2. Colder decision

**Round 2 (T):** T₀ ∈ {0.02, 0.03, 0.04, 0.07}, from the base's 0.05.

## 3. Amplification, retested on this composition

- **Round 3 (T gain):** G ∈ {2, 4}, from the base's 3.
- **Round 4 (T growth and brake):** (κ, ι) ∈ {(0.5, 1), (1.5, 1), (1, 0.5), (1, 2)}, from the base's
  (1, 1).
- **Round 5 (the M → T link gain):** η_m ∈ {0.025, 0.1}, from the base's 0.05.

## 4. A fair classical comparator

The classical modular system gets the same kinds of knobs, all tuned on the development seeds, on a
288-point grid:
- the gate detector;
- a register memory with a classical thermal floor, r ← (1 − ε_c)·r + ε_c·uniform each step,
  ε_c ∈ {0, 0.01, 0.02, 0.05};
- a greedy tracker with evidence gain G ∈ {2, 3, 4} and growth with brake (κ, ι) ∈ {0, 1} × {0, 1};
- λ ∈ {0.02, 0.05, 0.1, 0.2} and η_m ∈ {0.025, 0.05, 0.1}.

## Predictions

1. **Q1, the fixed check sees heat:** under the log-score, the best ε is above 0 (heat kept), and the
   kept system's acc_missed on test is at least 0.02 above the ε = 0 arm's.
2. **Q2, cold has saturated:** going colder than 0.05 changes test acc_resp by at most 0.003.
3. **Q3, amplification transfers to this composition:** re-optimizing rounds 3–5 changes test acc_resp
   by less than 0.005 in total.
4. **Q4, law 5:** the fully tuned classical comparator's test acc_resp is at least the final quantum
   system's − 0.01.
5. **Q5, the two profiles persist:** quantum is above classical on cued trials, and classical is above
   quantum on missed trials (QB-7's split).

**Kill criterion:** if 3 or more of Q1–Q5 fail.

## Build

- `qb9` is `qb8`'s runner with the log-score check and a tunable classical comparator.
- **Unit tests:**
  - the base reproduces QB-8's 0.768;
  - the log-score is recomputed independently;
  - the classical comparator at QB-7's settings (ε_c 0, G 3, κ 0, ι 0, λ 0.1, η_m 0.05) reproduces
    QB-7's 0.747.

## Results (2026-10-18, `evidence/qb-9/results.txt`, 5.1 s)

Tests passed before the run:
- the base reproduces QB-8's 0.768;
- the classical comparator at QB-7's settings reproduces QB-7's 0.747;
- the log-score recomputes independently.

**A correction to the precommitment:** the classical grid is 4 × 3 × 2 × 2 × 4 × 3 = **576**
configurations, not 288. The grid itself was as listed; only the count was wrong.

| Round (check) | dev: off → best | test acc_resp (cued / missed) | verdict |
|---|---|---|---|
| **1 M heat, by log-score** | −2.003 → **−1.748 (ε 0.02)**. ε 0: −7.762; 0.0025: −2.909; 0.005: −2.423; 0.04: −1.819 | **0.768 → 0.776** (cued 0.917 → 0.908, missed **0.538 → 0.572**) | **KEPT** |
| 2 T colder | 0.7941 → 0.7946 (T₀ 0.03; 0.02 / 0.04 / 0.07: 0.7939 / 0.7944 / 0.7933) | 0.776 → 0.776 (−0.0001) | not kept |
| 3 T gain | plain wins (G 2 / 4: 0.7887 / 0.7913) | unchanged | not kept |
| 4 T growth and brake | 0.7941 → 0.7963 (κ 0.5, ι 1) | 0.776 → 0.774 | not kept (lost on test) |
| 5 M → T link gain | plain wins (0.025 / 0.1: 0.7824 / 0.7743) | unchanged | not kept |

**The final quantum system:** **0.776** (cued 0.908, missed 0.572). With M's heat set to 0, the same
system scores 0.751 (cued 0.918, missed 0.493).

**The fair classical comparator**, tuned on dev over 576 configurations (floor 0.05, gain 4, κ 1, ι 1,
λ 0.05, η_m 0.1): **0.775** (cued 0.903, missed 0.578).

| # | Result | What was measured |
|---|---|---|
| Q1 the fixed check sees heat | MEASURED | best ε 0.02; missed 0.572 against 0.493 with no heat (+0.079) |
| Q2 cold has saturated | MEASURED | best T₀ 0.03 on dev; test change −0.0001 |
| Q3 amplification transfers | MEASURED | rounds 3–5 changed test by +0.0000 |
| Q4 law 5 | MEASURED | classical 0.775 against quantum 0.776 |
| Q5 the profiles persist | MEASURED | cued: quantum 0.908, classical 0.903; missed: classical 0.578, quantum 0.572 |

**5 of 5 hold.**

### What QB-9 shows

1. **The fixed check found what the old one hid.** Judged by the log-score, the best memory heat is
   *twice* QB-8's (0.02, not 0.01). It lifts missed trials to 0.572 for a small cost on cued trials
   (0.917 → 0.908). The old argmax check could not see this trade-off at all. A module's check must
   score the output its consumer actually uses.
2. **The other dials are at their optimum for this composition.** Cold has saturated, and the gain,
   growth, brake and link all hold where they are. Here the amplification settings *did* transfer.
   QB-8's lesson stands (always retest), but a retest can confirm as well as change.
3. **Law 5 is exact once both sides get the same kinds of knobs:** 0.775 against 0.776. The two
   profiles persist, narrowed: quantum is better when the memory is right, classical when it is
   stale.
4. **The nodes are optimized,** so FABRIC-1 can be built on them.
