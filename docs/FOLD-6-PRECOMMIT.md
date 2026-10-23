# FOLD-6: does folding save data?

Status: PRECOMMITTED (2026-10-23), before any FOLD-6 code.

Jesse (2026-10-23), on FOLD-5's 4,096 memories in 422 KB: "So that's pretty huge savings on data?"
FOLD-1 to 5 measured **capacity**: each recall answers "which of 16 regions holds this key?", about
4 bits, with the key as the cue. FOLD-6 stores **content** in the fold and counts the bits that come
back, against the bytes the fold costs, raw storage and ordinary compression.

## The store (FOLD-5's winner, now holding content)

- **One phase store** (a `qb-amplitude` state) of **D = 4,096** amplitudes. The efficiency below
  barely depends on D; D is kept small so the runs take minutes.
- **The content:** N items, item i = a byte value v_i ∈ {0 … 255}.
- **Writing (binding, superposition folding):** item i adds key_i ⊙ code_{v_i} to the state.
  - key_i and code_v are random phasor vectors made from a seed (a recipe), so they are not stored,
    just as raw storage stores no index.
  - ⊙ is the elementwise product, a new function crate `qb-bind`.
- **Precision (the quantization lever, glossary 22):** the folded state is kept either at full
  precision (complex f64, 128 bits per amplitude) or reduced to b bits of phase per amplitude
  (b ∈ {1, 2, 4, 8}; the magnitude is dropped, a new function crate `qb-quantize`).
  - b 1 is a sign: phases 0 and π.
  - **Stored bits = D × (128 or b).**
- **Reading:** unbind with the key (state ⊙ conj(key_i)), then read every value's code with the
  phase-locked overlap (`qb-overlap-phase`, from FOLD-5); the best one is the recalled value.
- **No heat or flow:** FOLD-5 measured that this store needs no maintenance and that heat only
  costs, so it is left out.

**Two contents:**
1. **random bytes,** uniform (8 bits of entropy each, incompressible);
2. **real data:** the bytes of `docs/GLOSSARY.md` in order (12,203 bytes). The reference
   compressors on the whole file:
   - xz −9e: 5,200 bytes (0.43 bytes per byte);
   - gzip −9: 5,278 bytes.

N ∈ {32, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048}. Test seeds 10–17 (keys, codes and
random content per seed).

## The measures

- **Accuracy p(N):** the fraction of items recalled exactly.
- **Lossless size:** N_exact is the largest N with every item correct on all 8 seeds. The cost is
  **stored bytes per content byte = D·bits/8 ÷ N_exact**. Raw storage is 1.0; xz is 0.43 on the
  glossary.
- **Lossy efficiency** (random content): recalled bits per stored bit, at the best N. For each
  item, the recalled information is that of an M-ary symmetric channel,
  I = log₂M + p·log₂p + (1 − p)·log₂((1 − p)/(M − 1)) with M 256. Raw storage is 1.0.

## Predictions

1. **R1, full precision is far bigger than raw:** at complex f64, the lossless cost is ≥ 100 stored
   bytes per content byte.
   - Recall of one of 256 values needs a signal-to-crosstalk ratio √(D/N) of about 4, so
     N ≈ D/16 ≈ 256 items. They are held in 64 KB.
2. **R2, quantizing helps a lot, but not past raw:** the best b (1 or 2) gives a lossless cost
   between 2 and 10 stored bytes per content byte, and a best lossy efficiency between 0.1 and 0.5
   bits per bit.
   - A sign keeps about 2/π of the signal, and costs 1 bit instead of 128.
3. **R3, folding never beats raw on random data:** the best lossy efficiency at any b is < 1 bit per
   bit. That is the entropy floor (glossary 22): no method stores more bits than it holds.
4. **R4, folding ignores the data's redundancy:** on the glossary, N_exact is within ±20% of the
   random-content N_exact at the same b.
   - The keys are random, so the crosstalk does not depend on what the values are.
   - xz, which uses the redundancy, beats the best fold by ≥ 5× in bytes.
5. **R5, what folding gives instead, damage tolerance:** in the 1-bit store at N = N_exact/2,
   overwrite 30% of the stored amplitudes with random ones. Recall stays ≥ 0.95.
   - Every item is spread over all D amplitudes, so damage lowers every item's signal a little
     instead of erasing some items outright.
   - Raw storage with 30% of its bytes overwritten keeps about 70%.

## The thermal medium, for the record (no new run)

FOLD-4's best fresh setup holds 128 region memories (about 4 bits each, at 0.948) in 3,379,200
stored bits: about 0.0001 recalled bits per stored bit.

CPU only, 8 threads, a few minutes.
