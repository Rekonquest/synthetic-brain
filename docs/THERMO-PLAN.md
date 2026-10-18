# THERMO: the thermal base, built from scratch, one atom stack at a time

Status: PLAN (2026-10-18).

## Jesse's direction

- **The theory:** the human brain contains a state-like base, perhaps similar to a thermodynamic state
  (hot and cold affect thinking). The thermal base could be "that state, where everything else lives
  in". How would the quantum and atom stacks work inside it, better or worse?
- **The method:** from scratch, "so it can go through the same process of atom stack by atom stack and
  optimizing at each stack as we go along, something I never did for the thermostat" (the universum
  substrate).
- **The dimensions:** "true intelligence doesn't take place in 2D, because when I think, I can see the
  shapes of my ideas, I can rotate those shapes in my thoughts."
  - The behavioural anchor is Shepard & Metzler (1971): the time to judge two 3D shapes the same grows
    linearly with the angle between them. The mind rotates through intermediate angles.

## Rules for every stack

- One function per crate, with zero dependencies, and the orchestrator holding every setting.
- **The axis count never appears inside a law.** Dimensionality is a property of the lattice, not of
  the local physics (the universum substrate's C12 rule). Every law is written once and runs in any
  d.
- Every stack is precommitted, built, and checked against its own law, with an independent
  recomputation. It is then optimized under Jesse's protocol before the next stack goes on top:
  - chaos on and off;
  - hot and cold;
  - amplification on and off, where the stack's law allows it;
  - plain is the bar and wins ties.

## The ladder (each rung depends only on the rungs below)

| Rung | Stack | Its one job | Its own check |
|---|---|---|---|
| T1 | `thermo-lattice` | an d-dimensional periodic lattice, and the exchange law (local diffusion, one pass per axis) | energy conserved; mean squared spread = d·κ·t; isotropic; never negative; the same code for d = 1–4 |
| T2 | `thermo-bath` | temperature: fluctuations and damping from one coupling (fluctuation–dissipation) | equipartition: the stationary variance is proportional to T; the response to a push matches the fluctuations |
| T3 | `thermo-carve` | threads: edge conductances maintained by flow and eroded without it (maintain-or-erase), with the erosion slower than the maintenance | a path that is used becomes conductive; an unused one erodes; the timescales hold |
| T4 | `thermo-rotate` | rotate a shape held in the medium (3D and up), continuously | the rotated pattern matches the target rotation; the time grows linearly with the angle (Shepard–Metzler) |
| T5 | `thermo-host` | the modules (detector, memories, trackers) live at sites and signal *through* the medium | it reproduces the direct-wired system when the medium is ideal |
| T6 | THERMO-1, the experiment | sweep the base temperature, cold → hot; the quantum-module brain against the classical one; a uniform temperature against local temperatures | the working-memory task, against the hand wiring's 0.776 |

## Predictions for T6, stated now and precommitted properly at T6

- Both brains show an inverted U over temperature.
- The quantum one peaks colder, and degrades faster as it gets hot (decoherence).
- Local temperatures beat a uniform one (QB-8/9: a soft memory and a cold decision).
