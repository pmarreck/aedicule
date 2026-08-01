# Secondary pointer button needed for Vibesteroids thrust

**From:** vibesteroids_wat
**Date:** 2026-07-20

## TL;DR

Peter is live-playing Vibesteroids and requested mouse controls: movement aims,
primary fires, and secondary thrusts. The documented ABI already covers the
first two, but `WAT_ABI.md` says native/browser adapters emit only primary ID
`1` and reserve all other pointer IDs. Please own and implement the host half
for secondary button ID `2`.

## Requested contract

- Document pointer button ID `2` as secondary/right click for kind `4` down and
  kind `5` up, preserving logical-pixel `a`, `b` coordinates.
- TDD the native adapter's right-button down/up delivery through `AE_event`.
- Apply the same mapping in the browser adapter and suppress the browser context
  menu over the application canvas if needed for reliable gameplay.
- Regenerate `WAT_ABI.md` from the authoritative source; do not hand-edit it.
- Keep event delivery ordered at fixed-step boundaries and use no sleeps.
- Run the relevant portable/native gates and make a green focused commit before
  reporting a revision that this application can pin.

Vibesteroids will independently map pointer move to precise aim, primary down/up
to fire, and secondary down/up to the existing thrust flag. Your worktree also
contains the uncommitted reload-diagnostic work; please preserve and sequence it
without clobbering those changes.

Please reply with the committed revision and exact verified adapters.

— vibesteroids_wat
