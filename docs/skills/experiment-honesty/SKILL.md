---
name: experiment-honesty
description: A transferable discipline for running experiments and reporting results honestly in any project - precommit predictions before code, choose every threshold on dev seeds and report on held-out test seeds, keep a status-quo bar, correct derivations only before code (and record the correction), label after-the-run checks as diagnostics, measure the cause of a failure before claiming it, report misses plainly with numbers, and save/commit as you go. Use for any benchmark, A/B test, ML or algorithm experiment, performance study, research prototype or claim of improvement, in Jesse's projects or elsewhere; and whenever tempted to tune after seeing results, cherry-pick, or explain a failure without measuring.
---

# Experiment honesty

## Before code
- Write the precommit: task, frozen metric, hard invariants, arms, bars, predictions with numbers,
  what each failure would mean. Commit it before any code.
- **Check every prediction's arithmetic and every threshold grid's range against what the reading
  can actually take** (seen: grids that could never fire; capacities impossible by construction).
  Fixing them before code is legitimate — record the correction in the doc.
- Keep a **status-quo bar** (the existing method) and an **ablation** for each claimed mechanism.

## Running
- Dev seeds choose every threshold; test seeds only report. Never re-tune on test.
- If something changes mid-run (a bug, an unspecified choice), say so: rerun as a **labelled check**
  and keep the precommitted result too.
- Look for bugs when results are "too clean" or "too identical" (exact ties four times in a row was
  once a real property, once worth checking).

## After
- Report holds and misses as counts ("7 of 12 hold") with the numbers. A miss in the good direction
  is still a miss of the prediction.
- **Measure the cause before explaining a failure** (a diagnostic run), and label it as after the run.
- Separate "the mechanism works" from "the theory is true".
- Save, commit and push as each piece finishes; never leave results only in scrollback.
- Background agents get a hard time box and must run before writing up.
