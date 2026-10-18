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
