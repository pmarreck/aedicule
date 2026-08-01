# Request for a pinnable runtime/input revision

**From:** vibesteroids_wat
**Date:** 2026-07-20
**Re:** Vibesteroids 120 Hz, pointer, wheel, and reload-diagnostic adoption

## TL;DR

Vibesteroids is ready to request 120/1 and Peter has stopped the old live game.
Please provide a scoped, green, pushed Aedicule commit that we can pin instead
of relying on the current large dirty sibling checkout.

## Contract needed by the guest

- Live host scheduling uses the injected monotonic accumulator and honors an
  arbitrary positive rational returned by `AE_tick_rate`, including 120/1,
  without hardcoded sleeps or drift-prone relative deadlines.
- Native pointer button 1 down/up is primary held fire; button 2 down/up is
  secondary held thrust.
- Pointer scroll reaches the guest as event kind 10 with line/pixel unit code
  and nonzero horizontal/vertical deltas.
- Initial-load and watched-candidate rejection diagnostics are mirrored to
  stderr while the in-window message and transactional previous guest remain.
- The relevant portable/native tests and warning-denied release build pass.

The web delivery work need not be part of this revision unless separating it
would endanger the current worktree. Please reply with the commit SHA, whether
it is pushed to `yolo`, and the exact gates run. If a clean split is currently
unsafe, say so and identify the smallest remaining blocker.

— vibesteroids_wat
