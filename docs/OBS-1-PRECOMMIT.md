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

## Results (2026-10-24, `evidence/obs-1/results.txt`, 24 s)

| F1 data (test) | drift | step | oscillation | burst | tipping | false alarms |
|---|---|---|---|---|---|---|
| observer | 0.623 | 0.000 | 0.010 | 0.772 | 0.035 | 0.297 |
| status-quo bars | 0.795 | 0.825 | 0.415 | 0.637 | 0.235 | 0.205 |

Early warning before the tip: the observer 0.215, the variance bar 0.858.

| F2 written ideas | repetition | novelty | topic drift | contradiction | false alarms |
|---|---|---|---|---|---|
| observer (phase codes only) | 0.875 | 0.985 | 0.050 | 0.950 | 0.048 |
| bars (with symbol ids) | 1.000 | 1.000 | 0.432 | 1.000 | 0.000 |

| F3 thoughts | settling | flip-flop | collapse | false certainty | false alarms |
|---|---|---|---|---|---|
| **observer** | **0.998** | 0.270 | 0.010 | **0.998** | 0.007 |
| bars | 0.995 | 0.000 | 0.000 | 0.000 | 0.000 |

Settling was not flagged as a problem in 0.998 of streams.

- **O5:** benign-oscillation alarms 0 habituated against 7,258 unhabituated, but the later step was
  detected 0.000 (the step rule never fires; see below).
- **O6** (collapse rate / load kept): no reaction 1.000 / 0; observer 0.015 / 0.304; variance bar
  0.010 / 0.335.
- **O7:** chaos not kept (dev surprise 388,143 → 388,166).
- **O8:** amplification is harmful: oscillation 0.000 with a false-alarm rate of 1.000.

**1 of 8 hold** (O7).

### The cause, measured (`evidence/obs-1/diagnostic.txt`; after the run, labelled)

Median over 40 dev streams of each reading's maximum (on "none" over the whole stream after
warm-up; on the others within the 200 steps after the start):

| stream | max \|surprise\| | max resonator ratio | max slowing evidence |
|---|---|---|---|
| none | **3.56** | **10.0** | **0.68** |
| step | 3.11 | 7.7 | 0.46 |
| oscillation | 2.97 | 7.1 | 0.62 |
| tipping | 3.08 | 6.4 | 0.68 |

**The sensing does not separate the behaviours: plain noise looks more alarming than the planted
behaviours on every reading.** It is a gap in what perception was handed, not in its thresholds:
1. **A single-step surprise** is swamped by chance maxima over a long quiet stream. A moderate step
   (2 units against noise of 1.15) never stands out in one step, because the predictor adapts within
   a few steps. It needs **evidence accumulation**.
2. **The resonator bank is not adapted per band.** With red (AR) noise, the slow bands always hold
   the most energy, so "the loudest band against the median" is always high. Each band needs its own
   baseline (**sensory adaptation**).
3. **Kendall's τ over 20 points** is too noisy to separate tipping from chance at the needed false-
   alarm rate. It needs a longer baseline and a contrast against it.
4. **Rules judged alone on dev add their false alarms** (0.297 in all). The thresholds need choosing
   jointly, against one false-alarm budget.
5. **O8's amplification doubled the surprise under fixed thresholds,** so every rule fired.

**What worked:**
- the thoughts observer (F3): it detects false certainty at **0.998**, where the entropy bars score
  0.000, and leaves healthy settling unflagged (0.998);
- idea novelty and contradiction on phase codes alone (0.985, 0.950), close to bars that see exact
  symbol ids;
- habituation silenced a benign rhythm completely.

### Niches

- **The thoughts observer:** catches false certainty, which the status quo cannot.
- **The idea observer:** novelty and contradiction with no symbol table.
- **Habituation:** silences repeated harmless signals.

OBS-1b fixes the five gaps above under a new precommit.
