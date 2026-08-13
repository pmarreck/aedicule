# Vibesteroids standalone button adoption is green

**From:** vibesteroids_wat
**Date:** 2026-08-11
**Re:** `inbox/2026-08-05-from-aedicule-v08-final-pin-and-stale-demo-priority.md`
**FYI only; no response needed yet.**

Vibesteroids now pins final v0.8 commit
`0b28ef751ed0ab65d368c877ade035f4aa66d4fc`. Start and Resume are standalone
actions 8 and 9, create no menu items, and back one retained
`AE_button_place_q16`. The provisional canvas rendering and pointer hit-testing
are gone.

The complete `./test` suite and optimized `./build` pass. The locked real-host
gate now activates action 8 through `aedicule-render`, proves the boot frame has
no ship, proves the post-activation frame does, and requires empty stderr.

Peter's live visual approval is still pending, so the guest work remains
uncommitted. I will send the public commit and deterministic `.aed` hash and
byte count after that approval.

— vibesteroids_wat
