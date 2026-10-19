# GPU-1b: 128-bit floats from the doubler, and whether 128 bits change the outcome

Status: PRECOMMITTED (2026-10-18), before any code.

## Why now

Jesse: "the next logical step should be 128 bits, because that can change the outcome." He doesn't
mind the GPU being damaged, but he does care about the rest of the system.

**The GPU compute-kernel step is on hold** (it stopped earlier), so this rung runs on the CPU only:
- at most 8 threads;
- no system setting is touched;
- nothing runs on the GPU.

It is the same validation path GPU-1 took for 64-bit.

## The stack (composed from outside, zero Rust dependencies; GPU-1's crates untouched)

| crate | its one job |
|---|---|
| `gpu-cap-binary128` | the contract: IEEE binary128 (1 sign bit, a 15-bit exponent with bias 16383, a 112-bit fraction) held in a `Wide<Wide<u32>>` (four 32-bit words), with its constants |
| `gpu-cap-f128-add` | add and subtract, with round to nearest even, subnormals, signed zeros, infinities and NaN (GPU-1's algorithm at 128 bits) |
| `gpu-cap-f128-mul` | multiply. The 226-bit significand product comes from the doubler applied to the 128-bit word (→ 256 bits): **growth by the same law, a third time** |
| `gpu-cap-f128-cmp` | <, == and max |
| `gpu-cap-f128-conv` | widen binary64 → binary128 (exact), and narrow binary128 → binary64 (round to nearest even) |
| `gpu1b` (orchestrator) | the checks below |

**The oracle** is GCC's `__float128`. That is libgcc's soft-float, an existing, independent
implementation, reached through a small C helper built with `-Wall -Wextra -Werror`, the way
`sdr-capture` is used.

## Predictions

1. **Q1, add and subtract are bit-identical** to `__float128`. The sets:
   - 10⁶ random bit patterns;
   - 10⁶ pairs within ±120 in exponent;
   - 10⁵ pairs in the subnormal range;
   - an edge set crossed with itself (±0, ±min subnormal, ±max subnormal, ±min normal, ±1, ±max, ±∞,
     NaN, and exact ties).

   Zero mismatches. A NaN result must be NaN, but its payload is not compared.
2. **Q2, multiply is bit-identical** on the same sets.
3. **Q3, compare is identical** on the same sets (max excluding the ±0 pair).
4. **Q4, conversion:**
   - widening is exact, and narrow(widen(x)) = x for 10⁶ random f64 values and the edges;
   - narrowing is bit-identical to the C cast `(double)` on 10⁶ random binary128 values.

### Does 128-bit change the outcome? (the instrument)

5. **Q5, the linear medium is not precision-limited:**
   - T1's exchange law runs in binary128 and in binary64 from the same random start (16³, 2,000
     steps, κ 0.5);
   - narrowed, the binary128 field differs from the binary64 one by ≤ 10⁻¹³ relative at every step;
   - **why:** diffusion is linear and contracting, so rounding errors do not grow.
6. **Q6, the chaos source is precision-limited:**
   - the logistic map x → 4x(1 − x) runs in binary64 and binary128 from the same 1,000 starts;
   - log |Δ| grows by **ln 2 = 0.693 per step** (its Lyapunov exponent), within 10%, while
     10⁻¹⁵ < |Δ| < 10⁻³;
   - **why:** this is where precision is load-bearing. The chaos source doubles every rounding error
     each step.
7. **Q7, T6's outcome at 128 bits:**
   - T6's best quantum configuration (θ_A 0, θ_B 0.02, g 1000), on all 32 test seeds;
   - its chaos source (`keyed_unit`'s 16 logistic steps) is computed in binary128, then narrowed;
   - **prediction:** 0 choices differ from binary64, and acc_resp is unchanged;
   - **why:** after 16 steps the chaos values differ by only about 10⁻¹¹ (2¹⁶ × 10⁻¹⁶), and they only
     set M's excitation phases. So the outcome does not depend on the extra precision.
   - If choices do differ, 128-bit changes the outcome here, and that is recorded as such.

**Chaos and hot/cold are excluded for the arithmetic itself** (exact arithmetic has one answer).
The precision-as-a-diagnostic results (Q5–Q7) are the experiment.
