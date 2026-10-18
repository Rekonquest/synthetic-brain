# SENSE-1: the Arc and the dongle feed the thermal state (receive only)

Status: PRECOMMITTED (2026-10-18), before any code.

## Jesse's direction

The second adapter connects to the **existing drivers** and reaches *into the thermal state*. It is
"basically how my dongle feeds into the thermo state":
- **the Arc** (its existing `xe` driver, untouched) becomes a real heat source;
- "we own it", so it can be given **new sensors**, composed from what it already has;
- **the dongle** (the RTL2838 and its existing `rtl2832_sdr` driver, untouched) feeds **more energy**
  into the same state.

Step 1 is **receive only**: no GPU load and no transmitting. The heartbeat (transmit) and the
kernel-side sensors come later.

## The stack (one function per crate; the drivers and Jesse's tools used unchanged)

| crate / tool | its one job |
|---|---|
| `arc-sense` | read one Arc sample from the sysfs paths the orchestrator gives it: the energy counter (µJ), package and VRAM temperature (m°C), the actual GT frequency, and the throttle-reason flags (pl1, pl2, pl4, prochot, ratl, thermal, vr_tdc, vr_thermalert), stamped with CLOCK_MONOTONIC |
| `arc-flux` (perceive) | energy samples → power (W), over fixed windows |
| `arc-rc` (perceive) | the lumped thermal law C·dT/dt = P − (T − T_amb)/R, fitted by least squares on ΔT against (P, T), giving R, C and T_amb: the die's stored heat, its cooling, and the room's temperature |
| `arc-boil` (perceive) | throttle-flag transitions → events (the "boiling point") |
| `sdr-stream` (Jesse's tool, unchanged) | the V4L2 MMAP stream from `/dev/swradio0` as TDR1 frames: a 64-byte header with a CLOCK_MONOTONIC stamp, then CU8 |
| `sdr-frames` | read TDR1 frames, to the format of the observer's `wire.rs` |
| `sdr-band-power` (perceive) | CU8 → mix down to an offset, a boxcar low-pass, then mean power per window, for the channel and for an off-channel comparison |
| `thermo-host` (T5's port) | inject each source's calibrated heat at its own site in the thermal base |
| `sense1` (orchestrator) | record, perceive, feed, and run the checks |

## The recording (one session, 60 s)

**The Arc:** polled every 1 ms on one thread, with no GPU load. Raw samples go to
`evidence/sense-1/arc.csv`.

**The dongle:** `sdr-stream /dev/swradio0 93000000 93300000 2048000 60`.
- This is SDR1's winning channel (93.3 MHz) inside a 93.0 MHz centre.
- The power per 100 ms window is computed for the channel (offset +300 kHz) and for an off-channel
  comparison (offset −500 kHz).
- The summary goes to `evidence/sense-1/sdr.csv`, with the first 1 s of raw frames kept for
  provenance.

`sdr-stream` is built from Jesse's source unchanged, with `cc -O2 -Wall -Wextra -Werror` as SDR1 did.

## The thermal state, and the feed

**The state:** the synthetic brain's thermal base, T5's medium on one heat field.
- 3D, side 16, κ 0.5, 4 substeps, leak γ_L 0.1 per substep, T = 0.
- The base is sized for expansion by T5's H5.

**The world step:** 10 ms, so 6,000 steps over the recording.

**The ports:**
- the Arc at A = (6, 8, 8), the dongle at D = (10, 8, 8), and a probe at P = (8, 8, 8) between them;
- each step, each connected port injects q·(reading / ref);
  - the reading is the Arc's power, or the dongle's channel power, over that step's window (the
    latest window at or before it);
  - ref is that source's mean over the first 5 s (the calibration window);
  - q = 1.

**Calibration rule (the orchestrator's, labelled as policy):** each source contributes the same mean
heat. This stops either source drowning the other by its native units (watts against radio
amplitude).

**The four arms (Jesse's rule, alone and together), over the same recorded data:**

| arm | Arc port | dongle port |
|---|---|---|
| plain (the bar) | disconnected | disconnected |
| Arc alone | connected | disconnected |
| dongle alone | disconnected | connected |
| together | connected | connected |

A disconnected port injects nothing; its reading is never read into the state.

## Predictions

1. **S1, the Arc senses:**
   - the energy counter never decreases and updates at ≥ 500 distinct values per second;
   - package temperature stays in [20, 95] °C;
   - mean idle power is in [5, 75] W.
2. **S2, the dongle senses:**
   - the 93.3 MHz channel's mean power exceeds the off-channel's by ≥ 10 dB (SDR1 measured 13–14 dB);
   - no frame gaps are reported.
3. **S3, zero in gives exactly zero heat:** the plain arm's state is exactly 0 at every site and every
   step, bit for bit.
4. **S4, each source alone delivers heat:** in each single-source arm, the probe's mean heat over the
   run is > 0. And the arm is bit-identical to the "together" arm's contribution from that source
   alone, recomputed independently with the other port disconnected.
5. **S5, the sources add; they do not substitute:** together = Arc alone + dongle alone, at the probe
   and every site, every step, within 10⁻¹² relative.
   - **Why:** this base is linear (T5's H2 superposition, 8.9·10⁻¹⁷).
   - QB-6's "amplifiers of the same thing substitute" needs a nonlinear maintainer. Testing that is
     for when T3's carving is composed in.
6. **S6, the die's thermal law:**
   - if the Arc's power varies by ≥ 3 W during the recording, the fit explains ΔT with R² ≥ 0.5, and
     T_amb lies in [15, 40] °C;
   - if it varies less, R and T_amb cannot be told apart. That is recorded as **unidentifiable at
     idle** (expected for receive-only), and is not counted as a failure.
7. **S7, no boiling:** zero throttle events at idle.

**Chaos is excluded at this step:** sensing has nothing for it to select. **Hot and cold are
deferred:** the base runs at T = 0 here, and heat enters when the Arc's own temperature is coupled
in. **Amplification** is the dongle's extra energy, tested as the four arms.

**Safety:**
- reading sysfs and receiving RF only;
- no GPU work is submitted, no setting is written, and nothing is transmitted;
- the dongle's tuning is restored by `sdr-stream` on exit, as in SDR1;
- the room heat is negligible: one polling thread for 60 s.

## Results (2026-10-18, `evidence/sense-1/`, a 64.7 s session)

`sdr-stream` was built unchanged from Jesse's source with `-Wall -Wextra -Werror`. Its log confirms:
- the `rtl2832_sdr` driver;
- 122,912,768 complex samples;
- sequence gaps 0, and the intentional settle discard of 262,144 samples;
- the original tuning (0.3 MHz / 50 MHz) restored on exit.

| # | Result | Measured |
|---|---|---|
| S1 the Arc senses | **MEASURED** | the counter is monotone; 918 distinct updates/s; 16.13 W idle; package 37.0 °C and VRAM 36.0 °C throughout; act_freq 0–600 MHz |
| S2 the dongle senses | **FAILED** (contrast) | the channel is 9.04 dB over the off-channel, against the 10 dB bar; the gap half holds (below) |
| S3 zero in gives exactly zero heat | **MEASURED** | the plain arm is exactly 0, bit for bit, at every site and step |
| S4 each source alone delivers heat | **MEASURED** | probe mean: Arc 0.009283, dongle 0.004553 |
| S5 the sources add | **MEASURED** | together − (Arc + dongle) is at most 8.9·10⁻¹⁶ of a largest value of 4.09 |
| S6 the die's thermal law | **UNIDENTIFIABLE AT IDLE**, as precommitted | the power varied by only 0.04 W over 64 s, so R and T_amb cannot be separated |
| S7 no boiling at idle | **MEASURED** | 0 events on all 8 flags, and no flag ever raised |

**6 of 7 hold.**

### S2, analysed (diagnostic after the run, not a prediction)

**The "gap marker" is the stream's footer.** `sdr-stream` writes a kind-2 frame once, at the end
(`write footer`, line 111). My precommit read kind 2 as a gap; that was my misreading of the format.
The stream's own counters (sequence gaps 0, reported gaps 0) show **no gaps**.

**The contrast miss is my band filter, not the dongle.** A Hann-windowed FFT of the saved 0.25 s
(`sdr-first-0.25s.cu8`) gives these band powers (±100 kHz, dB relative):

| frequency | dB |
|---|---|
| 93.3 MHz | **52.1** |
| 93.2 / 93.4 MHz | 49.2 / 49.6 |
| the floor | 39–41 |
| 92.5 MHz (the off-channel) | 41.6 |
| another, weaker station at 92.2–92.3 MHz | 44.8 |

- By that measure 93.3 MHz stands **10.5 dB** over the off-channel, and 11–13 dB over the floor.
- The perceive neuron's 8-tap boxcar has sinc sidelobes at only −13 dB and a ±128 kHz main lobe, so
  the strong channel and the band around it leak into the comparison band. That leakage lowered the
  measured contrast to 9.0 dB.
- **The fix is the neuron's selectivity:** a windowed filter or an FFT band power. The result is
  recorded as FAILED as written.

### What SENSE-1 settles

- **The Arc's real heat feeds the thermal state through the existing driver, read only.** The energy
  counter is a fast receiver (918 updates/s). At idle, temperature is flat at 1 °C resolution, so heat
  shows as power, not degrees, until the Arc is loaded.
- **The dongle's received RF feeds the same state, and the two sources add exactly.** The base is
  linear, so there is no substitution. Substitution needs a nonlinear maintainer (T3's carving),
  which is a future step.
- **Zero in gives exactly zero heat**, the TA1 disconnected control, reproduced on real inputs.
- **Receive-only cannot identify the die's thermal law.** The heartbeat (transmit) is what will
  separate R, C and the room's temperature.
