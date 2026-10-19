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

## Results (2026-10-18, `evidence/gpu-1b/`)

The unit tests ran first. They caught one bug before any check ran: the binary64 narrowing OR-ed the
kept significand's leading bit into the exponent field. It was fixed to subtract the leading bit and
then add the exponent. The oracle (`crates/gpu1b/oracle.c`) is built with `-Wall -Wextra -Werror`.

| # | Result | Measured |
|---|---|---|
| Q1 add and subtract | **MEASURED** | 0 mismatches against `__float128` (edges 676; random 10⁶; near exponents 10⁶; subnormal 10⁵) |
| Q2 multiply | **MEASURED** | 0 mismatches on the same sets; the 226-bit product comes from the doubler applied a third time |
| Q3 compare | **MEASURED** | 0 mismatches |
| Q4 conversions | **MEASURED** | widen 0 of 1,000,009; narrow(widen(x)) 0 failures; narrow 0 of 10⁶ against the C `(double)` cast |
| Q5 the linear medium is not precision-limited | **MEASURED** | 128 against 64 bits: at most **2.4·10⁻¹⁵** relative over 2,000 steps |
| Q6 the chaos source is precision-limited | **MEASURED** | ln\|Δ\| grows **0.687 per step** (ln 2 = 0.693, −0.9%) over 1,000 starts |
| Q7 T6's outcome at 128 bits | **MEASURED** | **0 of 64,000 choices differ** over 32 test seeds; acc_resp 0.7786 at both. The chaos values moved by 7.9·10⁻¹² on average (at most 8.8·10⁻⁹) |

**7 of 7 hold.**

### What it settles

- **The doubler grows to 128 bits by the same law.** Binary128 is composed from 32-bit integer words
  and is bit-identical to GCC's `__float128`.
- **Precision is load-bearing exactly where the dynamics amplify it:**
  - in the chaos source, every rounding error doubles each step: the logistic map's Lyapunov exponent,
    ln 2, measured to within 1%;
  - in the linear medium, rounding stays at the 10⁻¹⁵ level forever.
- **At T6's scale, 128 bits do not change the outcome.** The chaos values shift by about 10⁻¹¹ after
  their 16 steps, which is far below anything that moves a choice.
- **Where 128 bits would change the outcome** is any stack where a chaotic or critical part runs for
  many steps before its value is used:
  - a logistic source iterated about 40 steps or more (where 64-bit has lost every digit);
  - a long edge-of-chaos trajectory;
  - near-tie decisions repeated over long runs.

  That is where to use precision as a selected atom.
