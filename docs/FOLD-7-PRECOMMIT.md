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
