# Proposed simulation timing and input-order contract

**From:** vibesteroids_wat
**Date:** 2026-07-17 10:00 EDT
**Re:** Peter's report that the live adapter still uses a fixed 16.667 ms sleep

## TL;DR

Keep the existing guest-declared `fp_tick_hz()` and separate `fp_event(...)`
from `fp_tick(...)`. Treat monotonic elapsed time—not timer wake count—as
authoritative. Drive the adapter with an OS-backed one-shot deadline wake,
advance all due fixed steps with the existing rational accumulator, interleave
timestamped events at deterministic tick boundaries, render once, and arm the
next absolute deadline without a fixed sleep or busy wait.

Please reply with any GPUI constraint that makes this contract impractical, or
confirm the resulting implementation/test plan.

## Ownership

- The guest declares an exact required fixed simulation frequency through
  `fp_tick_hz()`. The host honors it or rejects the guest; it must not silently
  substitute another frequency. Legacy absence may continue to mean 60 Hz.
- Aedicule owns the monotonic clock, wake mechanism, catch-up limit, dropped-tick
  diagnostics, native event timestamps, and render scheduling.
- The OS/compositor owns physical display refresh. Simulation Hz and display Hz
  are deliberately independent.
- The guest never receives wall-clock time or a variable delta.

`fp_tick_hz()` specifies simulation steps per elapsed second, not Wasm function
invocations per second. `fp_tick(n)` may batch adjacent steps when no event
boundary falls between them.

## Deadline-driven host loop

Maintain:

- the existing rational `FixedStepClock` phase/remainder;
- the last sampled monotonic instant;
- the absolute deadline of the next simulation boundary; and
- a FIFO of native events stamped from the same monotonic clock domain.

On every timer or native-event wake:

1. Sample monotonic `now`; never infer elapsed time from how many timers fired.
2. Determine which fixed boundaries are due from actual elapsed time.
3. For each due boundary, deliver queued events whose timestamp is at or before
   that boundary, preserving arrival order, then advance that tick. Define the
   equality case explicitly as event-before-tick.
4. Batch consecutive event-free boundaries into `fp_tick(n)`; split the call at
   every event boundary.
5. Bound catch-up. Drop excess whole tick debt and count it diagnostically;
   retain only the rational sub-tick remainder. Never spin to repay unlimited
   debt.
6. Preserve every queued event in order even across dropped time. Deliver
   events from a dropped interval before the next surviving tick; do not
   coalesce away press/release edges.
7. Render at most once after the pump has reached its latest state. Rendering
   need not occur once per simulation step and must not mutate simulation.
8. Compute the next absolute fixed-step deadline. Arm an OS/GPUI one-shot timer
   for `deadline.saturating_duration_since(Instant::now())`. The timer is only a
   wake request: early, late, and spurious wakes are harmless because the next
   monotonic sample decides whether any tick is due.

If GPUI exposes only `Timer::after(Duration)`, it remains suitable as the
one-shot wake adapter when the duration is freshly computed from the absolute
deadline after work. A repeating `Timer::after(FRAME_INTERVAL)` loop is not.
If work has already crossed the deadline, schedule an immediate cooperative
wake/yield and let bounded catch-up handle it; do not busy-wait.

## Input semantics

Do not pass an unordered bag of inputs into `fp_tick`. Retain the two existing
ports:

```text
fp_event(kind, code, a, b)
fp_tick(ticks)
```

The observable deterministic contract is the ordered sequence of those calls.
An event received between boundaries N and N+1 is delivered immediately before
tick N+1. An event received after a boundary that the host has not processed
yet must not retroactively affect that overdue tick; timestamped queuing is what
prevents this under timer lateness.

Non-simulation rendering may be requested after event delivery, but event
arrival must not manufacture a simulation tick.

## Mechanically falsifiable tests

Please start red and keep the scheduler core independent of GPUI/real clocks:

- arbitrary elapsed-time partitions yield exactly the same tick boundaries as
  one whole interval at both 60 and 120 Hz;
- simulated wake lateness does not shift later boundaries (no work-time drift);
- early/spurious wakes produce zero ticks and a corrected next deadline;
- inputs immediately before, exactly on, and immediately after a boundary are
  interleaved with ticks according to the stated rule;
- a late wake spanning several boundaries batches only event-free runs and
  splits around queued events;
- bounded overload reports dropped ticks, retains fractional phase, preserves
  event order, and never creates unbounded debt;
- render count is independent of the number of catch-up ticks; and
- a fake wake driver proves the loop arms computed deadlines without sleeps or
  timing-dependent tests.

After the host adapter is green, `vibesteroids_wat` will provide the downstream
60/120 equal-elapsed-time WAST matrix and Peter's playtest before declaring 120
Hz.

— vibesteroids_wat
