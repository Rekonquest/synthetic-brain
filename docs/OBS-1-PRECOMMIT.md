# OBS-1: an observer that measures data and ideas, perceives behaviours and reacts

Status: PRECOMMITTED (2026-10-24), before any OBS-1 code.

Jesse (2026-10-24): "let's try making something that can observe, measure ideas or data, and observe
and react to detected behaviors." Asked what "ideas" means: **all of them**:
- the brain's own thoughts (hypotheses and their confidence);
- written ideas (symbols and concepts);
- patterns discovered in data.

Asked what to observe first: **synthetic streams only**, with planted behaviours and ground truth.

**The architecture is his class separation:**
- **sensing:** readings, polarity only, no decisions;
- **perception:** named, revisable hypotheses ("drift", "oscillation", …), each citing the IDs of
  the readings behind it, with no fused score;
- **judgment:** a thin, labelled reaction layer.

**The rules are as before:**
- no existing stack reused (status-quo detectors appear only as bars);
- each law is its own stack, then chaos off/on, then amplification on/off;
- physics composed with neuroscience, chemistry and cognitive axioms;
- niches recorded;
- f64;
- test seeds 10–17, dev 0–3 (every threshold is chosen on dev).

## The axioms (sensing and perception stacks)

- **Predictive coding (neuroscience) / free energy:** an online predictor per stream (an exponential
  forecast of the level and the trend) and its own running error variance. **Surprise** is the error
  divided by its expected size (precision weighting). Behaviours are structure in the surprise.
- **Critical slowing down (physics of tipping points):** before a system tips, its recovery slows.
  The lag-1 autocorrelation and the variance rise. The reading is the trend (Kendall's τ) of both
  over a rolling window. "Approaching a tipping point" is perceived *before* it happens.
- **Resonance (physics):** a bank of 16 damped resonators at log-spaced frequencies, driven by the
  surprise. The energy one resonator absorbs is evidence of an oscillation at its frequency (an
  antenna for rhythms).
- **Thermodynamics:** the stream's temperature (rolling variance of the surprise) and a "heat spike"
  (a sudden rise) as evidence of a burst.
- **Habituation (neuroscience):** a perception that keeps firing for the same behaviour is damped
  (repeated harmless rhythms stop alarming), and a new behaviour is not damped (sensitization).
- **Le Chatelier (chemistry):** the reaction opposes the perceived disturbance, in proportion to it.
- **Ideas as phase codes:** each symbol is a phasor (a recipe). The observer's memory of past ideas is
  their superposition.
  - **novelty** is a low overlap with that memory;
  - **repetition** is a high one;
  - **topic drift** is the movement of the recent ideas' centroid away from the old;
  - **contradiction** is a symbol arriving bound to its "not" role when its positive form is in
    memory.
- **PERCEPTION (cognitive):** each hypothesis names its behaviour and cites its readings, and it can
  be retracted.

## The streams and their planted behaviours (ground truth)

300 streams per family and seed (equal shares of each behaviour). A behaviour starts at a random
time in the stream's second half.

- **F1, data** (2,000 numbers; AR(1) noise around a level):
  - none;
  - drift (a slow linear change of level);
  - step (a sudden change of level);
  - oscillation onset (a sinusoid appears);
  - variance burst;
  - approaching a tipping point (the AR coefficient rises toward 1).
- **F2, written ideas** (1,000 symbols from 256, sampled from a topic of 32):
  - none;
  - repetition loop (a short cycle of ideas recurs);
  - novelty burst (ideas never seen);
  - topic drift (sampling moves to another topic);
  - contradiction (the negated form of a seen idea).
- **F3, the brain's thoughts** (1,000 steps of a probability vector over 8 candidate answers):
  - none (steady confidence);
  - settling (confidence rising: healthy);
  - flip-flopping (the top answer alternates between two);
  - collapse (entropy rising toward uniform);
  - false certainty (sudden certainty on an answer that the evidence later contradicts).

## Measures

- **Detection:** the planted behaviour is named correctly within 200 steps of its start.
- **False alarm:** a behaviour named on a "none" stream.
- **Delay:** steps from the start to detection.
- **The reaction test (F1 tipping):** a system x(t+1) = a(t)·x(t) + noise, with a(t) rising by
  load. If a reaches 1 the system collapses. The observer may react (Le Chatelier: cut the load in
  proportion to the evidence).
  - The score is collapse avoided, and the load kept: reacting too early wastes capacity, too late
    collapses.

## The bars (status quo)

- **F1:** CUSUM on the level (step and drift), a rolling z-score (burst), the FFT peak of the last 256
  (oscillation), and a fixed variance threshold (tipping).
- **F2:** a frequency table (repetition, novelty), and the KL divergence between the old and recent
  windows (drift).
- **F3:** thresholds on the entropy and on top-answer changes.
- **The reaction test:** no reaction (collapse), and a fixed-variance-threshold reaction.

## Chaos and amplification

- **Chaos:** the predictor's learning rate jittered per step by 1 + 0.5(u − ½). The selector is the
  observer's own total surprise on dev (a better model is less surprised). Plain wins ties.
- **Amplification:** surprise precision × 2 (a sharper ear), and the resonator bank with 32 bands.

## Predictions

1. **O1, data:** the observer detects and names every F1 behaviour at ≥ 0.85 with a false-alarm
   rate ≤ 0.05, and beats the combined bars' mean detection by ≥ 0.1 at the same false-alarm rate.
2. **O2, early warning:** critical slowing down names "approaching a tipping point" before the
   tipping in ≥ 0.8 of tipping streams. The variance-threshold bar fires before tipping in ≤ 0.5.
3. **O3, ideas:** F2 behaviours are named at ≥ 0.85 with false alarms ≤ 0.05.
4. **O4, thoughts:** F3 behaviours are named at ≥ 0.8. Healthy settling is *not* flagged as a problem
   in ≥ 0.9 of settling streams.
5. **O5, habituation:** on a long benign oscillation (2,000 extra steps), the habituated observer
   raises ≤ ¼ of the alarms of the unhabituated one, and still detects a new behaviour planted after
   it at ≥ 0.85.
6. **O6, the reaction:** the observer's Le Chatelier reaction avoids collapse in ≥ 0.95 of runs while
   keeping ≥ 10% more load than the variance-threshold bar. With no reaction, ≥ 0.9 collapse.
7. **O7:** chaos is not kept.
8. **O8:** amplification raises oscillation detection at low amplitude by ≥ 0.05, at a false-alarm
   cost of ≤ 0.02.

**New function crates:** `obs-predict`, `obs-slowing`, `obs-resonators`, `obs-novelty`,
`obs-habituate`. Orchestrator `obs1`, holding perception and judgment (labelled). CPU, about a
minute.
