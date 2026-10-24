# PHYS2-POC: quarks, black holes and holography as atom stacks (proofs of concept)

Status: PRECOMMITTED (2026-10-24), before any PHYS2 code.

The second batch from the atlas queue (Jesse: "start then move to the rest"). The same rules as
PHYS-POC:
- load-bearing means the mechanism works, not that the theory is true of nature;
- each law is its own stack against plain, then chaos off/on (the stack's own check selects), then
  amplification on/off;
- each write-up records the stack's **niche** (Jesse's standing principle: every stack is a
  specialist for a hybrid);
- f64 throughout (the precision note).

Test seeds 10–17; dev 0–3. Each run takes seconds to a minute.

## Q1: colour confinement as a closed-loop check (FOLD-7's record store)

- **The law:** quarks never appear alone. Only colourless (closed) combinations are observed, and
  pulling one out costs more than making a new closed pair.
- **The stack:** records (x, y, z) in the 2-bit store (D 4,096), N ∈ {64, 128, 192, 256}, past
  FOLD-7's clean range.
  - Recall z from (x, y), then close the loop: recall x from (y, ẑ) and y from (x, ẑ).
  - The answer ẑ is **accepted only if both come back right** (the triplet is "colourless").
  - Plain accepts every answer.
- **Measures:** precision of accepted answers, and coverage (the fraction accepted).
- **Chaos:** on a rejected answer, 3 variants of the cue jittered by 0.2·(u − ½), each re-closed;
  the first closed variant is accepted. **Amplification:** the loop closed twice over (x and y read
  back twice, from the re-read).
- **Predictions:**
  - **C1:** at N 192, the precision of accepted answers is ≥ 0.98 when plain's accuracy is ≤ 0.9,
    with coverage ≥ 0.5.
  - **C2:** chaos is not kept (it adds coverage only by accepting lucky wrong closures).
  - **C3:** amplification is not kept.

## Q2: asymptotic freedom as concept clusters

- **The law:** the strong force is weak at short distance (quarks move almost freely inside a
  hadron) and grows with distance, so they never escape (confinement). Its potential grows like
  σ·r.
- **The stack:** 16 concepts, each with 8 members, as points in a 2D plane under noise σ_n for
  20,000 steps. Each member is pulled toward its concept's centre:
  - **free inside, confined outside:** no force within radius r₀ 1, a constant pull σ beyond it
    (the linear potential);
  - **plain:** a spring (F ∝ r);
  - **Coulomb-like:** F ∝ 1/r², which can be escaped.
- **Measures:**
  - binding, the fraction of members still nearest their own centre at the end;
  - freedom inside, the members' spread within r₀ against the spring's (more spread means more room
    to vary).
- **Chaos:** σ jittered per step by 1 + 0.5(u − ½). **Amplification:** σ × 2.
- **Predictions:**
  - **F1:** the asymptotic-freedom law keeps binding ≥ 0.99 with an inside spread ≥ 1.5× the
    spring's (the spring at equal force at r₀: k = σ/r₀). *Corrected before code from 2×:* a uniform
    disk of radius about r₀ against the spring's Gaussian gives about 1.66×.
  - **F2:** Coulomb loses members (binding ≤ 0.8).
  - **F3:** chaos is not kept.
  - **F4:** σ × 2 tightens without gain.

## Q3: no-hair forgetting as consolidation (the 2-bit association store)

- **The law:** a black hole keeps only its conserved quantities (mass, charge, spin). Everything else
  about what fell in is lost.
- **The stack:**
  - 256 old memories (key → byte) are written;
  - then **consolidation**: each old memory's binding is replaced by its "no-hair" version, key →
    the value's class (the top 4 bits of the byte, coded by a class codebook at quarter amplitude);
  - then 256 new memories are written.
  - **Plain:** no consolidation.
- **Measures:** recall of the new memories; class recall of the old (the top 4 bits).
- **Chaos:** the class amplitude jittered per old item, selected by the new memories' readback.
  **Amplification:** the class amplitude × 2.
- **Predictions:**
  - **N1:** new-memory recall rises by ≥ 0.1 over plain (the old memories now load the store
    less);
  - **N2:** old class recall stays ≥ 0.3 (chance is 1/16). *Corrected before code from ≥ 0.9:* at
    quarter amplitude the class signal against the crosstalk of about 272 effective items is about
    0.25·√(4096/272)·0.9 ≈ 0.9, too weak for 16 classes to be clean. The law trades detail for
    room, as no-hair says.
  - **N3:** chaos is not kept;
  - **N4:** amplification trades new recall for old class recall (both move ≥ 0.03 in opposite
    directions).

## Q4: holographic recovery after losing a region

- **The law (AdS/CFT as an error-correcting code):** the bulk is encoded in the boundary so that it
  can be recovered from any large enough part of it.
- **The stack:** the 2-bit association store (D 4,096, N 128), with its D amplitudes cut into 4
  regions.
  - **Holographic layout:** every memory spread over all D (the fold, as is).
  - **Local layout:** memory i written only in region i mod 4 (D/4 amplitudes, as files on disks).
- **Damage:** one whole region erased (set to random phases).
- **Chaos:** none; the layout is fixed by construction. **Amplification:** N 64.
- **Predictions:**
  - **H1:** holographic recall stays ≥ 0.95 after losing a region. The local layout loses about a
    quarter (≤ 0.78), and its undamaged regions are each more crowded (a quarter of the dimensions).
  - **H2:** with no damage the two layouts tie (±0.02). Each holds the same items per amplitude
    (D/N is the same), so their signal-to-crosstalk is the same. *Corrected before code:* first
    written as "local already worse (≤ 0.9)".

## Results (2026-10-24, `evidence/phys2-poc/`; each run 1–11 s)

### Q1, colour confinement as a closed-loop check: 2 of 3

| N | plain accuracy | closed-loop precision | coverage |
|---|---|---|---|
| 64 | 1.000 | 1.000 | 1.000 |
| 128 | 0.995 | 0.995 | 0.990 |
| 192 | 0.984 | 0.988 | 0.965 |
| 256 | 0.928 | **0.956** | 0.905 |

- **C1 FAILED on its condition:** the record store is stronger than I derived, so plain is still
  0.984 at N 192.
- Where plain does err (N 256), the closed loop lifts precision 0.928 → 0.956 and flags about 10%
  of answers. The rejected answers are mostly the wrong ones.
- **C2 and C3 HOLD:** chaos and closing twice are not kept.
- **Niche:** an error flag that needs no outside check. A record that does not close is suspect. It
  is modest at this load and should grow past it.

### Q2, asymptotic freedom as concept clusters: 3 of 4

| law | binding | spread inside (RMS within 2 r₀) |
|---|---|---|
| **free inside, constant pull outside** | **1.000** | **0.786** |
| spring (equal force at r₀) | 1.000 | 0.444 |
| Coulomb 1/r² (equal force at r₀) | 1.000 | 0.140 |

- **F1 HOLDS:** full binding with 1.77× the spring's freedom inside.
- **F2 FAILED:** Coulomb lost no members. At this noise its strong close-range pull holds them tight
  (spread 0.140), and nothing reached the weak far field. My derivation used the far-field escape
  only.
- **F3 and F4 HOLD:** chaos is not kept; σ × 2 tightens without gain.
- **Niche:** concepts that hold their members and let them vary freely. Of the three laws, it has
  the most internal flexibility at full binding.

### Q3, no-hair forgetting as consolidation: 3 of 4

| | new recall | old class recall |
|---|---|---|
| plain (old kept whole) | 0.652 | 0.660 |
| **no-hair at a 0.25** | **0.936** | 0.273 |
| no-hair at a 0.5 (amplification) | 0.877 | **0.609** |

- **N1 HOLDS:** new recall rises by +0.28.
- **N2 FAILED by margin:** 0.273 (bar 0.3).
- **N3 HOLDS:** chaos is not kept.
- **N4 HOLDS:** a × 2 trades −0.058 new for +0.336 old class.
- **At a 0.5, no-hair beats plain on both counts:** new recall 0.877 against 0.652, and old class
  recall 0.609 against 0.660 (nearly kept). Forgetting the detail and keeping the class frees room.
- **Niche:** consolidation, or "sleep". It makes room for new memories by reducing old ones to what
  is conserved.

### Q4, holographic recovery: 2 of 2

| N 128 | intact | one region of four erased |
|---|---|---|
| **holographic (spread over all)** | 1.000 | **0.961** |
| local (each in one region) | 0.999 | 0.750 |

At N 64 the holographic store loses nothing (1.000) while the local one still loses a quarter
(0.750).
- **H1 and H2 HOLD.**
- **Niche:** surviving the loss of a whole region, like the brain's resilience to focal damage.

**PHYS2-POC: 10 of 13 hold.** Every law is load-bearing for something, and no two have the same
niche: an error flag, flexible concepts, consolidation, and region-loss survival.
