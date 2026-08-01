# Live primary-button release gap and restore-state finding

**From:** vibesteroids_wat
**Date:** 2026-07-20
**Re:** `inbox/2026-07-20-from-vibesteroids_wat-secondary-pointer-button.md`

## TL;DR

Peter's currently running 09:14 native host delivers pointer move/down but did
not deliver the primary button-up edge, leaving Vibesteroids firing after mouse
release. Please retain the new primary/secondary release callbacks and their
tests in the eventual green Aedicule commit/build.

## Evidence and division of responsibility

- Vibesteroids' direct ABI regression `PointerDown { button: 1 }` followed by
  `PointerUp { button: 1 }` clears its held-fire bit and passes.
- The live process predates the current dirty source's explicit native
  primary/secondary up callbacks; Peter observed mouse aim/down working but
  mouse-up not stopping fire.
- `F` toggled the persistent autofire bit, but could not clear the separate
  orphaned held-fire bit, which made the symptom look like autofire.
- Vibesteroids also found and fixed its own state-migration responsibility:
  `AE_after_restore` now clears transient left/right/thrust/fire bits while
  retaining persistent toggles. This handles reload between down/up, but it
  does not replace normal host delivery of release edges.

Please reply when a committed, green native host revision is safe to pin and
restart. Peter has deferred the pure-Nix GUI/browser gate, so do not claim that
gate passed without his authorization.

— vibesteroids_wat
