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
  - **F1:** the asymptotic-freedom law keeps binding ≥ 0.99 with an inside spread ≥ 2× the spring's
    at the same binding.
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
  - **N2:** old class recall stays ≥ 0.9;
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
