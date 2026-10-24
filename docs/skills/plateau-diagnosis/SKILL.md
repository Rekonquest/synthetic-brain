---
name: plateau-diagnosis
description: What to do when a stack, unit, generator or agent loop under chaos tuning stops improving — the plain arm and every chaos variant tie at the same score for several attempts. In Jesse's projects a plateau has never once been weak chaos; every plateau was a gap in what the unit was handed or in how the units were wired (input, contract, scoping, validity, feedback, targeting or architecture), and more renders only burn hours. Use whenever a score repeats across attempts, a repair loop keeps "fixing" without the number moving, all arms tie, a hidden check stays at the same count, or anyone proposes more variants, more rounds or a bigger model; and before changing any unit's code by hand. Pairs with edge-of-chaos (the method) and compose-from-outside (where the fix goes).
---

# Plateau diagnosis — a plateau is a gap, not weak chaos

## The rule

When the plain render and every chaos variant tie at the same score for two or more attempts,
**stop rendering.** Chaos can only re-present what the unit was given; it cannot supply a rule,
a field, a line number or a callee that is not there. Every plateau on record (table below) was
one of six gaps in the pipeline around the unit. Find the gap, fix it as a general mechanism,
then rerun chaos on against off. The fix is never a hand edit to the unit's own code: that
would be a hand-made part, and the next task would hit the same gap.

## The procedure (minutes, not hours)

1. **Score every case, not the first.** Run the kept program over the whole visible suite and
   record each case's *first* failure (event, wrong fields or error with traceback lines, state
   before). Group them into classes: for a crash, the error type plus innermost frame; for a wrong
   record, the event kind plus the sorted wrong fields. **The largest class is the signal.** The
   first case's failure is usually a different, rarer bug.
2. **Recompute the expected number by hand from the spec**, for the cleanest instance of the
   largest class (fewest wrong fields). The arithmetic names the wrong formula. Examples that
   took one line each: `rttvar` too small → variance computed from the already-updated mean;
   timer = t + srtt·9/8 → the loss-delay factor used as the PTO duration.
3. **Find the writer of the wrong field.** Which unit's code assigns that key? Read that code,
   not the unit the traceback or the event handler points at.
4. **Ask what that unit was handed.** Does the rule it got wrong appear in its procedure lines,
   contract or feedback at all? Is it stated the way the code needs it (per entity, unset = None,
   old value vs new)? Did another unit receive those lines too? Was the render that was kept even a
   finished render?
5. **Name the gap from the ladder below, fix it in the pipeline, rerun on vs off.** The bar is the
   score before the fix. Record the arithmetic of step 2 in the notes so the fix is checkable.

## The ladder of gaps (every one has happened; seven kinds)

| gap | how it looks | what fixed it |
|---|---|---|
| **input** | a rule is absent from every render of the unit; the world's node for it has no edge onward | author the missing edge from outside the engine (author tool), never a hand rule in code |
| **contract** | a shared ceiling at an exact count; the failing inputs share a value (`t = 0`, an unset field read as 0, a missing data shape, an unstated precondition) | state the shape and the sentinel in the state contract; `None` for unset, never 0 |
| **scoping** | one unit's code writes fields another unit owns; the owner is never called; the copy carries a bug chaos on either unit cannot reach | give each unit only the lines nearest its own node; a function node is what the function *produces*, so it owns the production of its own past cone (directed, ties shared); neighbours become callees |
| **validity** | the kept unit is a stub, ends in the model's notes and `pass`, or was cut off at the token cap but compiles; its caller crashes on `None` and gets blamed | a unit must return what its docstring promises; a render at the token cap is no render; never accept the last attempt silently |
| **feedback** | the repair prompt shows one failure; each repair fixes one crash and hits the next at the same event; or the fed failure is a rare class while 60 of 68 failures are one crash | feed the largest class, its count, the cleanest instance, the state before it and the failing source lines |
| **architecture** | a unit calls a sibling, so a handler cannot see or sequence that sibling's result; repairs aimed at the handler score far below the kept program | functions never call peers; the handler composes them once each, in order, passing results along (the atom build rule) |
| **targeting** | the right unit is sixth in the target list and never gets a turn; or the frame that *read* a `None` is targeted instead of the unit that produced it | rank targets by who writes the wrong field; on a `None`, blame the unit whose result the failing line uses, before the reader |

## Evidence (Jesse's harness, E35c and its units)

Outcome: sixteen plateaus from 255 to 1456 visible events (2026-10-04 to 2026-10-08), each fixed
as a general rule or one authored worked case, none by touching a unit's code; the general
orchestrator then passed E35c, 40/40 visible and 300/300 hidden, with no hand-made parts.

- 2026-10-04, `update_rtt` stuck at 423/434: all 11 failures were the ack-delay cap; the world's
  cap node had no edge onward. Input gap. One authored edge → 434/434.
- 2026-10-04, timer unit at exactly 1204/1212 for every arm: all 8 failures had a packet sent
  at t = 0 and the contract said "0 = none". Contract gap.
- 2026-10-04, `on_ack` repairs all tied: fed a rare timer mismatch while 60 of 68 failures were
  one crash. Feedback gap.
- 2026-10-06, general orchestrator at 255/300 for 8 attempts: 28 of 40 cases wrong on `rttvar`,
  written inside the acked-packets unit, which had been handed 47 lines (36 of them `UpdateRtt`'s)
  and re-implemented it. Scoping gap, plus a feedback gap (only case 0 shown) and a targeting gap
  (owner sixth in line).
- 2026-10-06, run 10 at 72 for 9 renders of `on_ack`: the acked unit was an abandoned stub that
  compiled; `on_ack` crashed on its `None` and was blamed every time. Validity and targeting gaps.
- 2026-10-06, run 11 at 7 for 12 renders over 4 units: every `get_pto` render used 9/8; the PTO
  formula had been assigned to `UpdateRtt` because the edge's source was its node. Scoping gap
  (ownership must follow the world's direction).
- 2026-10-06, run 12 at 825 for 7 attempts: 9 of 11 failing sends were app-space sends after
  confirmation with no timer, while the state said confirmed. `on_send` had been frozen as
  `on_send(state, ev, handshake_confirmed=False, …)`; the dispatcher never passes the extra
  parameter, so it shadowed the state, and the frozen signature kept it through nine repairs.
  Contract gap: a handler's signature is known in advance (`(state, ev)`); enforce it at render
  time, before anything is frozen. Diagnosed in under ten minutes with steps 1–4.
- 2026-10-08, run 15 at 403 for 6 attempts over 6 units: the timer after an ACK was the PTO
  (0 + 150000 + 4·75000 = 450000) where the loss time was expected (0 + 9/8·150000 = 168750).
  The loss pass never set `loss_time`: the lines defining it had been handed to the getter
  `GetLossTimeAndSpace`, because the ownership rule measured one direction and the loss pass's
  node is a *process* (its outputs lie downstream) while `UpdateRtt`'s is a *quantity* (its work
  lies upstream). Scoping gap, second flavour: ownership direction depends on what the function
  node is, and the author's aliases say which. Recomputing one timer by hand named it.
- 2026-10-08, run 16 at 144 for 6 attempts over 6 units: all four RTT fields untouched after
  every ACK. `update_rtt` had been rendered as `update_rtt(state, ev, t)`, a dispatcher over
  every event kind: nothing could call it, no unit computed the RTT, and since it was not
  reached from the handler it was never a target. Validity gap (a unit that is a handler in
  disguise: refuse an inner function that takes the event or reads `ev["ev"]`) plus a targeting
  gap (a writer nobody calls: the handler is its missing caller and goes first). The tell was
  the target list itself: the unit that owns the wrong fields never appeared in it.
- 2026-10-08, run 17 at 7 for 6 renders on one crash: `TypeError: 'str' // 'int'`. The
  callees returned dicts and the caller unpacked them as tuples, so `loss_time` was the string
  `'loss_time'`. The repair prompt showed the failing line and not the values on it, and every
  render kept the unpack. Feedback gap: a crash trace must carry the values of the names on each
  failing line (`loss_time='loss_time'` ends it in one render).
- 2026-10-08, run 18 at 734 while five inner units took turns on one class: an ACK that newly
  acknowledged nothing reset `pto_count` and re-armed the timer. The handler was first in the
  target list and was skipped every time: its try-budget for that class *key* had been spent
  eighteen attempts earlier, against a different program. Targeting gap (stale budget): a class
  key names a kind of failure; once the kept program improves it is a new failure, so budgets
  reset. The tell: the logged target list named the right unit first and the loop chose another.
- 2026-10-08, run 19 at 765 for 8 attempts, every unit tried: an ACK that newly acknowledged
  nothing still reset `pto_count` and re-armed the timer. The handler did
  `produced = update_rtt(...)` and `update_rtt` returned the whole state dict, always truthy. The
  repair prompt showed the record and the state before the event, not what the callees returned.
  Feedback gap: for a wrong record, list what each written function was given and returned during
  the failing event (`update_rtt(...) -> {'spaces': ...}` says it in one line), the analogue of
  the crash trace's values.
- 2026-10-08, run 20 (resumed at 765 with that call trace) still tied for 8 attempts: the
  handler could not act on "nothing newly acknowledged" because `update_rtt` called the acked
  unit *inside itself*; a second call from the handler removed nothing and skipped the RTT.
  Architecture gap, and the atom build rule names it: functions do not call peers, the
  orchestrator composes them. Refuse an inner unit that calls a sibling; the handler runs each
  once, in order, passing results along. The tell: the failed variants scored far below the kept
  program, the sign of a repair that is structurally impossible from where it was aimed.
- 2026-10-08, run 21 at 111 while every repair of one unit came back refused (`None` for the
  plain arm and all variants, twice): the renders set `state["timer"]` in place and documented
  a numpy-style `Returns / ------- / None`, and the "must return what it promises" check read
  that as a promise. Validity gap in reverse: a check too strict freezes a unit as surely as one
  too loose keeps a stub. The tell: a whole column of `None` for one unit while others render
  fine; tally the refusal reasons before anything else.
- 2026-10-08, run 23 at 91 with no RTT sample after any ACK: the call trace read
  `detect_and_remove_acked_packets('initial', [[0, 0]], …) -> (0, 0)`. The state had `acked = 0`
  and the unit scanned from `acked + 1`, so the first packet was never newly acknowledged.
  "0 = none" once more, this time in the state design, which had been rendered once and frozen,
  so no repair could reach it. Contract gap in the unrepairable part: make the state's values
  repairable with its shape fixed, and pass over a state render whose unset-looking keys hold 0
  (excluding the zeros the task itself states). The tell: a wrong value that every unit's repair
  leaves untouched lives in something no repair is allowed to touch.
- 2026-10-08, run 24 at 657 for 9 attempts, every unit tried, twelve renders of the timer unit:
  an ACK leaving a loss_time set with nothing ack-eliciting in flight; expected the loss time
  (21000), got None. The call trace showed the unit *receiving* loss_time 21000 and cancelling:
  every render tested "nothing in flight" before the loss time, though the procedure line stated
  the order. Input gap, second instance: the world had worked cases for loss-time ties and for
  1-RTT-only in flight, none for this order; one authored `Case:` edge, store rebuilt. The tell:
  the plain render and every dice variant agree on the same wrong order, a prior that only a
  worked case in the unit's own lines overrides.
- 2026-10-08, run 25 at 1241 / 29 for 7 attempts on a four-case class (the timer after a
  timeout): the expected PTO was one space's, the got value another's; the state showed the
  losing space with no packets left and a separate per-space flag `ack_eliciting_in_flight`
  still True. A derived aggregate stored beside the collection it summarises goes stale when
  the collection changes. A sibling class in the same run: `KeyError: 'timer'`, a record field
  the state never had. Contract gap, state kind: the state holds every scalar field of the
  output record (checkable from the visible tests) and stores nothing computable from a
  collection (count, flag, minimum). The tell: a value in the state that contradicts the
  collection next to it.
- 2026-10-08, run 26 at 0 for 5 attempts, every unit tied: the first send's timer was a loss
  time for the packet just sent. The handler computed `loss_delay` and every packet's loss time
  itself, the loss pass's work; the "assigns a key another unit computes" feedback said nothing,
  because under the layering rule the loss pass *returns* loss_time and assigns nothing, so
  code-level ownership saw no overlap. The world's lines did: the identifiers on the left of `=`
  in a unit's `(when …)` clauses are the quantities it defines. Feedback gap: when the code no
  longer shows who owns a quantity, read it from the procedure. The tell: a handler with
  arithmetic in it.
- 2026-10-08, run 29 at 768 for 9 attempts: one missing `pto_count` reset in a 150-line
  handler whose rule was in its own lines. Both of the handler's turns re-rendered the whole
  function and scored 132–523 against the kept 768: the repair lost what already passed. Feedback
  gap of the repair form: a whole-function re-render of a long unit is a lottery; give the repair
  a minimal-edit die ("keep every line not involved in the failure") with the plain repair still
  the bar. The tell: repair candidates far below the kept score on a unit whose fix is one line.

## When the visible suite passes and the hidden check fails

That is not a plateau and not a repair target: it is a **selector gap**. The orchestrator can
only select on what it can score, and a unit can satisfy every visible case while carrying a
branch the hidden sequences reach (2026-10-08, run 30: the timer unit cancelled when nothing at
all was in flight before taking a set loss time; the visible cases only ever had non-ack-eliciting
packets in flight; run 27's program, built on the same world, passed both). Never repair against
the hidden result. Widen the selector instead: the world's worked cases carry exact numbers and
are per-unit checks beside the visible suite; author the missing case into the world and let it
score the unit.

## Signs you are about to waste a day

- "Let's try more variants / more rounds / a higher temperature / a bigger model."
- Reading the traceback's innermost frame and re-rendering it a third time.
- Reasoning from the first case's failure.
- Patching the unit's code by hand "just to get past it".
- Treating agreement across renders as truth: consensus checks the translation, never the fact.

## Relation to the other skills

- `edge-of-chaos` is the method (two arms, the base is the bar, the stack's own selector).
- `compose-from-outside` says where a fix goes: a mechanism around the unit, never inside it.
- `lightcone-worlds` holds the authoring step for an input gap (a missing edge) and the rule
  that worked examples are verified by recomputation before use.

## Instances 12–13 (2026-10-24)
- **OBS-1:** every detector "failed", but the cause was the *sensing*: plain noise looked more
  alarming than the planted behaviours on every reading. A gap in what perception was handed, not
  in its thresholds (measured by comparing the readings' distributions).
- **HOLO-R:** the ablations, repair laws, chaos and amplification all tied, because the principal's
  perception was blind (found 3 of 16 damaged regions). A gap in wiring: one unit never received
  the right target.
