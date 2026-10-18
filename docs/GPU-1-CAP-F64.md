# GPU-1: 64-bit floats for the Arc, composed from outside (CPU validation first)

Status: PRECOMMITTED (2026-10-18), before any code.

## Why

Jesse wants a two-way thermo adapter for his Intel Arc A380, his experiment GPU. Its first direction
is to run the thermal base on the Arc. Every law of the base is written in binary64 (f64), but the
driver reports:
- `shaderFloat64 = false`;
- `shaderInt64 = true` (32-bit integers are native).

Jesse's direction: compose from outside, as Vello's `cap-*` crates do, and **give it the tools
needed for 64-bit**. The Arc, its driver and the existing laws are never touched.

**His amplification idea:** growth by doubling the bits, through a separate amplification adapter.
If that does not work, compose the wide product directly over it instead.

## The capability stack (one function per crate, zero dependencies)

| crate | its one job |
|---|---|
| `gpu-cap-wide` | **the amplification adapter (growth):** `Wide<W>` makes a 2w-bit word from two w-bit words (add with carry, subtract with borrow, the full product w × w → 2w, shifts, leading zeros, compare). It is written once and applies to itself: `Wide<u32>` = 64 bits from the Arc's native 32-bit words, and `Wide<Wide<u32>>` = 128 bits. The base word's only primitive is u32 × u32 → (lo, hi), which is SPIR-V's `OpUMulExtended`. |
| `gpu-cap-binary64` | the contract: the binary64 format held in a `Wide<u32>` (its sign, exponent and fraction fields, and its constants), and conversion to and from host f64 bits |
| `gpu-cap-f64-add` | add and subtract, with round to nearest even, subnormals (no flush to zero), signed zeros, infinities and NaN |
| `gpu-cap-f64-mul` | multiply. Its 106-bit significand product is `gpu-cap-wide`'s doubling of the 64-bit word, and the same rounding rules apply |
| `gpu-cap-f64-cmp` | less-than, equal, and max (Rust's `f64::max` rule: a NaN operand yields the other) |
| `gpu1` (orchestrator) | the checks below |

**What stays on the CPU:** transcendental and constant inputs (e^{−γ}, √T, κ/2, the FFT twiddles) are
computed by the orchestrator and passed in. Division and normalization stay at the ports. So the GPU
needs only add, subtract, multiply and compare.

## Its own checks (predictions), all on the CPU

1. **W1, the doubler works once:** `Wide<u32>` matches native u64 (wrapping add and sub, shifts by
   0–63, leading zeros, compare) and native u128 (the full product) on 10⁷ random pairs plus the
   edges (0, 1, MAX, and carry chains). Zero mismatches.
2. **W2, growth by applying it again:** `Wide<Wide<u32>>` matches native u128 for the same operations
   on 10⁷ random pairs plus the edges. Zero mismatches.
3. **F1, add and subtract are bit-identical** to native f64 on:
   - 10⁷ random bit patterns, which cover every class;
   - 10⁷ pairs within ±60 in exponent (to exercise alignment and cancellation);
   - 10⁶ pairs in the subnormal and near-subnormal range;
   - a structured edge set, crossed with itself: ±0, ±min subnormal, ±max subnormal, ±min normal,
     ±1, ±max, ±∞, NaN, and exact rounding ties.

   Zero mismatches. **The one stated exception:** a NaN result must be NaN, but its payload is not
   compared, because x86's default NaN is not IEEE-specified.
4. **F2, multiply is bit-identical** on the same sets. Zero mismatches.
5. **F3, compare is identical** (<, ==, and max excluding the ±0 pair) on the same sets.
6. **F4, the laws are reproduced bit for bit:** T1's exchange law and T2's damping and exchange at
   T = 0, recomputed with the capability ops in the same operation order, reproduce the native
   crates' fields **bit for bit** on a 16³ lattice for 100 steps, from a random start and from a
   pulse (the pulse drives the tails into subnormals).
7. **A, amplification on and off:** the multiply using the doubler (`Wide<Wide<u32>>`) against a
   direct schoolbook 106-bit product composed inside the test.
   - **Prediction:** identical results.
   - The doubler is kept, because it is the one law reused for both widths.
   - If W2 fails, the composed product replaces it (Jesse's fallback), labelled.

**Chaos and hot/cold are excluded by the law.** Exact arithmetic has one correct answer, so any
perturbation is an error. Recorded as excluded, not skipped (as T1 did with amplification).

**Not predicted at this stage:** speed. CPU emulation of the GPU's arithmetic is a validation path,
not a performance one. GPU-2 moves the same integer code onto the Arc.

**Room heat:** the checks are single-threaded, and take seconds to minutes.
