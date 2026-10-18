# WAVE-1: atom antennas outside the thermodynamic state

Status: PRECOMMITTED (2026-10-18), before any code.

## Jesse's question

"What if we made regular atom antennas? I only tried making it in the thermodynamic state, never
tried to see if it would work outside of it."

The thermal antenna (TA1, universum-substrate) moves energy by diffusion: it is slow, smoothing and
noise-limited. A real antenna launches **waves**. WAVE-1 builds atom antennas in a wave medium, then
turns a bath on to see how a wave antenna becomes a thermal one.

## The stack (one function per crate, zero dependencies)

**`wave-lattice`, the medium's law.** The scalar wave equation on T1's d-dimensional lattice, in
leapfrog (symplectic) form, with the velocity at half steps:
- v ← v + dt·(c²·Σ_axes(u₊ + u₋ − 2u) − γ(site)·v + f(site));
- then u ← u + dt·v.
- The axis count never enters the law.
- γ(site) and the force f(site) are the caller's: a sponge layer at the boundary absorbs outgoing
  waves (sized for expansion: nothing wraps), and the transmitters supply f.
- **Its invariant**, with no loss and no force: H = ½·Σ v₋·v₊ + ½·c²·Σ_edges (Δu)², where v₋ and v₊ are
  the half steps either side of u.

**`wave-resonator`, the receiver.** A damped oscillator (natural frequency f_r, damping ζ) coupled to
the medium at its site by a spring κ_c in both directions (equal and opposite forces), stepped in
leapfrog. The energy it dissipates, ∫2ζω_r·ẋ² dt, is **the received energy**. That is TA1's "the
receiver absorbs it", in a wave medium.

**The transmitter is the orchestrator's force:** f = A·sin(2πf_t·t)·key(t) at its site. key(t) is the
on-off bit stream.

**`wave1` is the orchestrator.**

**Settings:**
- c 1, dt 0.5 (stable in 3D: c·dt < 1/√3);
- **3D side 48**, with a sponge 8 sites thick (γ rising quadratically to 0.5), leaving a 32³
  interior;
- frequencies 0.06 and 0.10 (wavelengths 16.7 and 10 sites, ≥ 8 sites per wavelength);
- receivers ζ 0.05, κ_c 0.05, A 1.

## Its own checks (predictions)

1. **W1, the medium conserves its invariant:** with no sponge, force or receiver, H is constant to
   10⁻¹² relative over 2,000 steps in d = 1, 2 and 3, from a random start.
2. **W2, it follows the discrete dispersion law:** a standing mode k = 2πm/L in 1D (L 64,
   m ∈ {1, 4, 8}) oscillates at ω = (2/dt)·asin(c·dt·sin(k/2)), within 10⁻⁶ relative.
3. **W3, waves travel at their speed:** in 3D, a pulse's first arrival (|u| reaching 10% of its later
   peak) at r ∈ {6, 10, 14} gives a speed within 10% of c.
4. **W4, zero in gives exactly zero (TA1's control):** with the transmitter off, the medium and the
   receiver stay exactly 0. With κ_c = 0, the receiver absorbs exactly 0 while the transmitter runs.
5. **W5, the far field falls off as 1/r²:** the received power at r ∈ {6, 8, 10, 12} fits an exponent
   within 15% of −2.
6. **W6, tuning:** a receiver tuned to the transmitter absorbs ≥ 10× more than one detuned by 30%.
7. **W7, two channels at once (multiplexing):**
   - two transmitters (0.06 and 0.10) each send their own 16-bit on-off stream (bit time 400 steps);
   - two receivers, each tuned to one, decode their own stream at ≥ 95%;
   - each absorbs ≥ 10× more from its own channel than from the other.
8. **W8, a bath turns the wave antenna thermal:** T2's damping-plus-noise on every site, in Langevin
   form (v ← v − dt·γ_b·v + √(2γ_b·T·dt)·ξ), γ_b 0.01, over T ∈ {0, 10⁻⁶, 10⁻⁵, 10⁻⁴, 10⁻³}:
   - the W7 decoding accuracy does not increase with T;
   - at the hottest, it falls below 75%.

   That is the transition from a clear wave channel to a noisy, diffusive one.

## Optimization round (each on W7's mean decoding accuracy, then the receivers' own-channel energy; plain wins ties)

- **Amplification on and off:**
  - the receiver's sharpness, ζ ∈ {0.02, 0.1} against 0.05 (a higher Q is resonant amplification);
  - and the drive, A ∈ {2, 4} at the hottest bath, where noise makes amplitude matter.
  - **Prediction W9:** a sharper receiver is not kept at T = 0, because a longer ring-down smears the
    bits. A stronger drive is kept under heat.
- **Chaos on and off:** a frozen per-site jitter of c², 1 + α(x − ½), α ∈ {0.1, 0.3}.
  - **Prediction W10:** not kept. Frozen disorder localizes waves (QB-2's Anderson character) and cuts
    far transfer.
- **Hot and cold:** W8 itself.

**Room heat:** single-threaded, about 10⁹ site updates in total, so minutes.

## Run 1: results (2026-10-18, `evidence/wave-1/results.txt`, 2 min 10 s on one thread)

The unit tests passed first: the neighbours wrap both ways, and an uncoupled resonator rings at its
period and absorbs exactly 0.

| # | Result | Measured |
|---|---|---|
| W1 the invariant is conserved | **MEASURED** | drift 1.8·10⁻¹⁵ (1D), 7.0·10⁻¹⁵ (2D), 2.2·10⁻¹⁴ (3D) over 2,000 steps |
| W2 the discrete dispersion law | **MEASURED** | m 1, 4, 8: agreement to 4·10⁻¹⁶ – 2·10⁻¹⁵ |
| W3 waves travel at c | **MEASURED** | arrivals at 1.5, 5.5 and 9.5 for r 6, 10, 14: speed **1.000** |
| W4 zero in gives exactly zero | **MEASURED** | transmitter off: everything exactly 0; uncoupled receiver: exactly 0 absorbed |
| W5 the far field falls as 1/r² | **FAILED** | exponent −2.53; the power is not monotone in log r (r 10 → 12 is shallower) |
| W6 tuning ≥ 10× | **FAILED** | ×6.2 |
| W7 two channels | **FAILED (selectivity)** | **decoding 100% / 100%**; own against other energy ×4.4 (receiver 1) and ×58 (receiver 2) |
| W8 a bath turns it thermal | **FAILED** | 100% decoding at every T up to 10⁻³; no transition was reached |
| W9 amplification | **FAILED** (a flawed check) | |
| W10 frozen disorder not kept | **FAILED** (a flawed check) | |

**4 of 10 hold.** The medium is exact. Every miss is at the antenna and experiment level, and each is
diagnosed below.

### The misses, diagnosed (after the run, not predictions)

1. **W6 and W7: the coupling detunes the receiver.** This is confirmed
   (`diagnostic-coupling-detunes.txt`).
   - The coupling spring κ_c adds stiffness, so ω_eff² = ω_r² + κ_c. At κ_c 0.05 against ω_r² 0.142,
     the "0.06" receiver really resonates at 0.0697.
   - **The scan:** with κ_c 0.05, absorption peaks at a nominal 0.048 (the law predicts 0.0483),
     17× above the nominal 0.06. With κ_c 0.005, it peaks at 0.06 (the law predicts 0.0589).
   - Receiver 1 was pushed toward channel 2, hence its ×4.4. Receiver 2, being stiffer (ω_r² 0.395),
     shifted less, and still gave ×58.
   - **My derivation miss:** I tuned the bare oscillator, not the coupled one.
2. **W5: the box is too small for the wavelength.** This is likely, not confirmed.
   - The 32-site interior holds under two wavelengths at 0.06 (λ 16.7).
   - An 8-site quadratic sponge reflects part of each wave, so the receivers sit in a standing-wave
     pattern rather than a clean outgoing field.
   - That is Jesse's sizing rule again, here for waves: the mesh must be many wavelengths across, with
     an absorber gentler than one wavelength.
3. **W8: the temperatures were too low, and tuning rejects noise.**
   - A resonant receiver integrates only the noise within its narrow band (Q ≈ 10), so its
     signal-to-noise ratio rises with tuning. That is why real antennas are tuned.
   - The transition must be looked for at higher T (10⁻² to 1).
4. **W9 and W10: the round's tie-break was meaningless.**
   - With decoding saturated at 100%, the tie-break (total absorbed energy) decided the rounds.
   - Raw energy grows with drive (∝ A²), with receiver bandwidth and with disorder's focusing,
     regardless of whether the link is any better.
   - **The check must score what the consumer uses:** a decoding margin, meaning the separation
     between 1-bit and 0-bit energy relative to their spread. That is the law learned in QB-9,
     forgotten here.

### What run 1 settles

- **The atom wave medium is exact.**
  - Its invariant holds to 10⁻¹⁴ in 1–3D, it follows the discrete dispersion law to rounding, and
    waves travel at exactly c.
  - Zero in gives exactly zero, TA1's control.
- **An atom antenna works outside the thermodynamic state.** Two transmitters on two frequencies
  sent two bit streams at once, and both were decoded at 100%. The thermal antenna can only do one
  slow channel.
- **Run 2's fixes are known:**
  - tune the coupled resonator (nominal frequency √(ω_t² − κ_c));
  - a larger box (several wavelengths) with a gentler sponge;
  - a hotter T grid;
  - a decoding-margin check for the rounds.
