# BEAT-1: the heartbeat (the Arc transmits through real heat)

Status: PRECOMMITTED (2026-10-18), before any code.

## The question

SENSE-1 showed that the Arc's real heat and the dongle's RF can feed the thermal state (receive only).
BEAT-1 is TA1's question asked of real silicon: **can a known pattern be put into a real thermal
state and recovered from it?**
- The Arc transmits: load on for a 1, idle for a 0.
- The existing sensors and the composed ones receive.
- The same signal is fed through the synthetic brain's thermal base and recovered there too.

Jesse asked for very light loads only.

## The transmitter, composed over the existing driver

`arc-vk-fill` is the adapter crate. It has one job: **fill a device-local buffer on the Arc, over and
over, for a given time.**
- It goes through the system's Vulkan loader and Mesa's Intel driver (ANV), both untouched. The loader
  is found by `dlopen` at run time, so there are no crate dependencies.
- The work is `vkCmdFillBuffer` on a 256 MiB device-local buffer. It is memory traffic: there is no
  shader, and no compiler is needed.
- Each submission is fenced, so the transmitter can stop within one submission.
- **Its own check:** a 1 MiB host-visible buffer is filled with 0xA5A5A5A5 and read back. This proves
  the Arc ran the command.
- It selects the device whose vendor ID is 0x8086 and whose name contains "Arc".
- **Nothing is written to any setting,** and no power or clock control is touched. The card's own
  limits stay in charge.

## The receiver

The receiver is SENSE-1's stack:
- `arc-sense`, polled every 1 ms;
- `arc-flux` for power;
- `arc-rc` and `arc-boil`;
- the dongle through Jesse's unchanged `sdr-stream`.

**The radio fix (S2's diagnosis):** the band power comes from a new perceive neuron,
`sdr-band-fft`, a Hann-windowed 4096-point FFT summing the bins within ±100 kHz.
`sdr-band-power` (the boxcar) stays as SENSE-1's record.

## The schedule (one session, about 112 s)

The pattern is 64 bits from SplitMix64, seed 0xBEA7. The first 8 bits are a preamble, 10101010.

| part | duration | what happens |
|---|---|---|
| A, the power channel | 16 s | 64 bits × 250 ms |
| C, the disconnected control | 16 s | the same 64-bit clock, transmitter off |
| B, the temperature channel | 80 s | 10 bits × 8 s (pattern 1100101101) |

**Decoding:**
- **The power channel:** a bit's value is its mean power, skipping the first 50 ms of each bit. The
  threshold is the midpoint of the preamble's 1-mean and 0-mean. The 56 bits after the preamble are
  scored.
- **The temperature channel:** a bit is 1 if the package temperature rises over it (its last reading
  above its first), and 0 otherwise.

**The thermal base:** as in SENSE-1 (T5's medium, 16³, a 10 ms step), with the Arc port at A, the
dongle at D and the probe at P, in four arms. The probe's heat is decoded the same way as the power
channel (the bit's mean probe heat, threshold from the preamble).

## Predictions

0. **H0, the adapter runs:**
   - the Arc is found;
   - the readback is 0xA5A5A5A5 at every word;
   - every fenced submission completes.
1. **H1, the heartbeat is received:**
   - the power channel decodes ≥ 95% of the 56 scored bits;
   - the mean power of 1-bits exceeds that of 0-bits by ≥ 3 W.
2. **H2, no signal when disconnected:** in part C, the mean power over the clock's 1-slots and
   0-slots differs by < 0.5 W.
3. **H3, the heartbeat is in real temperature:**
   - the package temperature rises ≥ 1 °C over at least one 8 s 1-bit;
   - the temperature channel decodes ≥ 8 of 10 bits.
4. **H4, the die's thermal law, now identifiable:** the power varies by ≥ 3 W, and the fit over the
   session gives R² ≥ 0.5 and T_amb in [15, 40] °C. The fitted room temperature is reported.
5. **H5, the heartbeat crosses the synthetic thermal state:** decoding the probe's heat in the
   Arc-alone arm recovers ≥ 95% of the 56 bits. And again, together = Arc + dongle within 10⁻¹².
6. **H6, the radio:**
   - with the FFT neuron, 93.3 MHz stands ≥ 10 dB over the off-channel;
   - **exploratory:** the Arc's load leaves the channel's power unchanged within 0.5 dB between 1-
     and 0-bits. No RF from the GPU's memory traffic is expected at 93 MHz.
7. **H7, no boiling:** zero throttle events. Light load stays under every limit.

**Arms and dials:** amplification is the dongle's extra energy, the four arms. Hot and cold is the
Arc's own heat, the experiment itself. Chaos is excluded, because a heartbeat is a fixed code with
nothing to select.

**Safety and room heat:**
- buffer fills only, for at most 56 s of on-time in total;
- the card's own power and thermal limits govern it;
- one CPU polling thread;
- no settings are written and nothing is transmitted by radio;
- the dongle's tuning is restored on exit.

## Results (2026-10-18, `evidence/beat-1/`, a 2 min 44 s session)

**Before the session, a one-second smoke test** of the transmitter (`arc-vk-fill`'s `smoke` example,
not one of the checks):
- the self-check read back all 262,144 words;
- the load went from 16.17 W idle to 28.34 W, at 55 GB/s of fills.

Mesa prints a warning that DG2 on the Xe kernel driver is "experimental". It is recorded, and every
fenced submission completed regardless.

| # | Result | Measured |
|---|---|---|
| H0 the adapter runs | **MEASURED** | the A380 found (0x8086 / 0x56a5); the readback exact; all 41 transmit slots completed their fenced submissions |
| H1 the heartbeat is received | **MEASURED** | **100% of the 56 scored bits**; 1-bits averaged 35.85 W, **+16.67 W** over 0-bits |
| H2 no signal when disconnected | **MEASURED** | +0.001 W between the clock's 1- and 0-slots with the transmitter off |
| H3 the heartbeat is in real temperature | **MEASURED** | 9 of 10 slow bits; rises of +4 to +5 °C over a 1-bit; the package went 37 → 46 °C |
| H4 the die's law identifiable | **FAILED** as written | the slope fit gives R² 0.046 (power range 17.8 W) |
| H5 the heartbeat crosses the thermal base | **MEASURED** | the probe's heat decodes **98.2%**; together − (Arc + dongle) at most 4.4·10⁻¹⁶; plain exactly 0 |
| H6 radio | **MEASURED** | FFT contrast **10.79 dB** (SENSE-1's boxcar gave 9.04); the GPU's load moves 93.3 MHz by only −0.036 dB |
| H7 no boiling | **MEASURED** | 0 events on all 8 flags |

**7 of 8 hold.**

**H3's one miss** is bit 7, a 1 following a 1. The temperature read 0.0 change over it: after the
previous 1-bit the die was already near its fast equilibrium, and the sensor's step is 1 °C.

### H4, analysed (diagnostic after the run, `evidence/beat-1/diagnostic-die-law-trajectory-fit.txt`)

**The precommitted fit was the wrong tool.** It regresses the slope dT/dt over 1 s steps, but the
temperature reads in whole degrees, so almost every slope is 0 or ±1 °C/s, and the slope regression
sees mostly quantization.

Fitting the whole **trajectory** instead (output error, at 0.1 s):

| model | R² | parameters |
|---|---|---|
| one node | 0.694 | τ 1.7 s, R 0.28 K/W, "T_amb" 34.8 °C |
| **two nodes** (a fast die on a slow sink) | **0.953** | **τ_fast 0.4 s (R 0.21 K/W)**; τ_slow at the grid's 595 s edge (R 0.97 K/W) |

- **The die has two time constants:**
  - a fast one, under a second: the die and its spreader;
  - a slow one, minutes or longer: the heatsink and the air.
- A 2-minute session cannot resolve the slow one. So the **room temperature is not identifiable
  yet**: the 34 °C is the sink's working point, not the room.
- **To measure the room,** a session must run for several slow time constants: tens of minutes of
  light duty, or cooling observed after a load.

### What BEAT-1 settles

**A known pattern put into real silicon heat is recovered:**
- perfectly, in power;
- 9 of 10 times, in raw 1 °C temperature;
- at 98% after crossing the synthetic brain's thermal state.

This is TA1's proof of concept, on real hardware, through the existing driver, with no driver
rewritten.

**The other measured facts:**
- the disconnected control is clean (0.001 W);
- the two real sources still add exactly in the linear base;
- the GPU's memory traffic leaves no trace at 93.3 MHz;
- the radio neuron's fix (Hann FFT) restored the 10 dB contrast.

**Carried forward:**
- the die is a two-node thermal system, with a fast node of about 0.4 s and a slow one of minutes or
  more;
- the fast node is the channel for signalling, and the slow node is the "state" that integrates
  everything over minutes.
