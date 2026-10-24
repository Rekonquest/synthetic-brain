# FOLD-7: a smaller fold, and recall by association

Status: PRECOMMITTED (2026-10-23), before any FOLD-7 code.

Jesse (2026-10-23), on FOLD-6: "I think that's huge, recalling from association not by address. OK
do the next steps." FOLD-6 found that folding costs space but survives damage, and that 1–2 bits per
amplitude is the cheap precision. FOLD-7 has three parts:
- **A** shrinks FOLD-5's capacity store to 1–2 bits;
- **B** tests recall from partial and noisy cues;
- **C** recalls any part of a record from the rest of it.

Quantization is the new knob, and it is a threshold, so chaos is tested where Jesse's rule puts it,
**before the threshold** (as a filter), with the store's own check selecting. Jesse (2026-10-23):
"Did you try it with chaos". Amplification is not rerun: FOLD-5 measured it inert on this store, and
heat only costs.

## A. The FOLD-5 store at 1–2 bits

FOLD-5 exactly (16 regions × D 1,650 phase amplitudes, writes, heat, seeds), read phase-locked
(`qb-overlap-phase`), with each region's state quantized to b bits of phase at read time
(`qb-quantize`, from FOLD-6). b ∈ {f64, 8, 4, 2, 1}; F ∈ {16, 64, 128, 256, 512}. The store's bytes
are 16 × 1,650 × b / 8.

Quantizing to b bits keeps a share of the signal-to-crosstalk power: about 0.81 at 2 bits (8/π²) and
0.64 at 1 bit (2/π). So a 1-bit store at F behaves like the full one at about F/0.64.

1. **A1:** 2 bits at F 256: region top-1 ≥ 0.90 (f64: 0.947).
2. **A2, the capacity survives at 1 bit:** F 256 is ≥ 0.80. That is 4,096 memories in **3,300
   bytes**, about 0.8 bytes per memory, against FOLD-5's 422,400.
3. **A3:** 8 bits is within 0.01 of f64 at every F.

**A4, chaos before the quantizer** (on against off, plain wins ties):
- **On:** before quantizing a region, make 3 variants of its state. Each amplitude's phase is moved
  by α·(u − ½)·2π/2^b, with u from `qb-chaos::keyed_unit(seed, region, variant, amplitude)` (the
  logistic map, mean ½) and α 0.5: a mean-preserving dither of up to a quarter of a level.
- **The selector** is the region's own readback: the mean phase-locked score of its own memories in
  the quantized variant. The best variant is kept if it strictly beats plain quantization.
- Run at b 1 and 2 and F 256, dev seeds 0–3, then test.
- **Prediction:** kept at 1 bit with a test gain of 0.005–0.03, and a tie or ≤ 0.005 at 2 bits.
  - A 1-bit sign of an amplitude near the threshold is a coin flip that dithering can turn.
  - The selector scores the memories themselves, so it can only help or tie.
  - With 3 variants the gain is small.

## B. Recall from a partial or noisy cue

FOLD-6's content store: D 4,096, quantized to 2 bits, N 48 random bytes (half of N_exact 96), bound
to recipe keys, recalled by unbinding with the cue and reading the 256 codes phase-locked. Test seeds
10–17.
- **Partial cue:** only a fraction c of the key's amplitudes is known (the rest are 0), with
  c ∈ {1, 0.5, 0.25, 0.1}.
- **Noisy cue:** a fraction η of the key's phases is replaced by random ones, with
  η ∈ {0, 0.3, 0.5, 0.7, 0.9}.

A cue keeps a signal proportional to what is right in it (c, or 1 − η), against the same crosstalk.
From FOLD-6's own curve, recall among 256 values is about 0.98 at a signal-to-crosstalk ratio of 4.2,
and about 0.65 at 2.6.

4. **B1:** a quarter of the key (c 0.25) recalls ≥ 0.95.
5. **B2:** a tenth of the key (c 0.1) recalls between 0.4 and 0.85: weakened, not gone.
6. **B3:** with half the key's phases wrong (η 0.5), recall is ≥ 0.95.
7. **B4, graceful:** at η 0.7, recall is between 0.3 and 0.9; at η 0.9 it is ≤ 0.15.

**B5, chaos on the noisy cue** (at η 0.7, where recall is intermediate):
- **On:** 3 variants of the cue, each phase moved by α·(u − ½)·2π with α 0.2 (keyed logistic,
  mean-preserving).
- **The selector** is the reading's own confidence: the margin between the best and the second-best
  code. Plain wins ties.
- Dev seeds 0–3, then test.
- **Prediction: not kept.** A perturbation adds no information about the key, and a confidence
  margin can be raised by a wrong peak as easily as a right one.

For the record (not measured): an address must be exact. One wrong bit of an address fetches a
different item.

## C. Any part of a record recalls the rest

Records of three random bytes (x, y, z). The fields are bound to role vectors (recipes) and the
record is their product, (R₁⊙code_x)⊙(R₂⊙code_y)⊙(R₃⊙code_z). N records are superposed in one store
(D 4,096, 2 bits), N ∈ {16, 32, 48, 64}.

**Recall in each of the three directions:** unbind the two known fields (and the missing field's
role), then read the 256 codes. So "given x and y, what is z?", and the same for y and for x, all
from one store with no address.

8. **C1:** at N 48, each of the three directions recalls ≥ 0.95. The two known fields together act as
   a key, so the law is FOLD-6's.
9. **C2:** the three directions are within 0.03 of each other at every N; the store has no preferred
   direction.

A new orchestrator crate, `fold7`, for B and C; part A is a `PART=quant` arm of `fold5`. CPU only,
8 threads, about 10 min.

## Results (2026-10-23; `evidence/fold-7/a-quant.txt` 7 min, `bc-association.txt` 4 s)

### A. The FOLD-5 store held in fewer bits (region top-1, read phase-locked)

| precision | bytes | F 64 | F 128 | F 256 | F 512 |
|---|---|---|---|---|---|
| f64 | 422,400 | 1.000 | 0.998 | 0.947 | 0.758 |
| 8 bits | 26,400 | 1.000 | 0.992 | 0.895 | 0.675 |
| 4 bits | 13,200 | 1.000 | 0.991 | 0.891 | 0.671 |
| **2 bits** | **6,600** | 1.000 | 0.980 | **0.838** | 0.602 |
| 1 bit | 3,300 | 0.979 | 0.841 | 0.606 | 0.395 |

| # | Result |
|---|---|
| A1 2 bits at F 256 ≥ 0.90 | **FAILED:** 0.838 |
| A2 1 bit at F 256 ≥ 0.80 | **FAILED:** 0.606 (1 bit holds 128 per region at 0.841) |
| A3 8 bits within 0.01 of f64 | **FAILED:** 0.895 against 0.947 at F 256 |
| A4 chaos before the quantizer | **FAILED:** an exact tie at 1 and 2 bits, dev and test |

**0 of 4 hold, and both causes are mine:**
- **A1–A3: the quantizer keeps phase only.** Dropping the magnitude costs a further π/4 of the power
  (for a sum of many phasors) at any number of phase bits. So 8 bits is no better than 4. The
  prediction counted only the phase rounding.
- **A4: the selector cannot be beaten.** The region's readback (the mean overlap of its own memories
  with the quantized store) equals, apart from the heat, Re⟨Σ keys, q⟩. Plain rounding to the
  nearest phase maximizes that exactly, amplitude by amplitude, so no dithered variant can win.
  - Chaos before this quantizer needs a selector that is not the quantizer's own objective. Recorded
    for the edge-of-chaos record: **a selector that the plain step already optimizes makes chaos
    inert by construction.**

**What it gives anyway:** 2 bits hold **4,096 memories in 6,600 bytes at 0.838** (1.6 bytes per
memory), or 2,048 at 0.980. That is 64× smaller than the f64 store for −0.11 at the deepest load,
and the same size at 2,048 memories gives 0.980.

### B. Recall from a partial or noisy cue (2-bit content store, 48 bytes)

| part of the key known | recall | | key phases random | recall |
|---|---|---|---|---|
| all | 1.000 | | 0 | 1.000 |
| half | 1.000 | | 30% | 1.000 |
| **a quarter** | **0.984** | | **half** | **0.982** |
| a tenth | 0.654 | | 70% | 0.607 |
| | | | 90% | 0.070 |

- B5: chaos on the noisy cue gives dev 0.646 → 0.604 and test 0.581 → 0.560; not kept.

### C. Any part of a record recalls the rest (2-bit store, 1,024 bytes)

At N 16, 32, 48 and 64 records, each field from the other two: **1.000 in all three directions.**

| # | Result |
|---|---|
| B1 a quarter of the key ≥ 0.95 | MEASURED: 0.984 |
| B2 a tenth of the key 0.4–0.85 | MEASURED: 0.654 |
| B3 half the key wrong ≥ 0.95 | MEASURED: 0.982 |
| B4 graceful (0.3–0.9 at 70%; ≤ 0.15 at 90%) | MEASURED: 0.607; 0.070 |
| B5 chaos on the cue not kept | MEASURED: −0.021 |
| C1 each direction ≥ 0.95 at N 48 | MEASURED: 1.000 |
| C2 no preferred direction | MEASURED: spread 0.000 |

**FOLD-7: 7 of 11 hold** (A 0 of 4; B and C 7 of 7).

### What it settles

- **Recall by association works, from a little of the cue.** A quarter of a key, or a key with half
  its phases wrong, still recalls 98%. An address with one wrong bit fetches the wrong item.
- **One store answers in every direction.** "Given x and y, what is z?", and every permutation of
  it, all at 1.000. Nothing is indexed by position.
- **It degrades gracefully.** Recall weakens as less of the cue is right, with no cliff until about
  three-quarters is gone.
- **The cheap store is 2 bits of phase.** Magnitude matters at depth, so the bit count beyond 2 buys
  little and the phase-only form loses about 0.05–0.1 at the deepest loads.
