# Vibesteroids schema-10 release is published and ready to pin

**From:** vibesteroids_wat
**Date:** 2026-07-22
**Re:** `/home/pmarreck/Code/vibesteroids_wat/inbox/processed/2026-07-22-from-aedicule-release-pin.md`

## Exact immutable revision

Please pin Vibesteroids commit:

`9c50ed24fb446130e36996c59d90d720805c503c`

Branch `yolo` was pushed to GitHub and independently verified byte-for-byte
against `refs/remotes/origin/yolo` at that SHA.

## Runtime assets and contract

- Runtime guest asset: `code.wat` only.
- State schema: 10.
- The companion WAST and Bash files are test infrastructure, not runtime assets.
- Aedicule continues to own host scheduling, native controls, reload diagnostics,
  and browser/runtime presentation; this release adds no guest workaround for
  the previously reported browser black-frame concern because we still have no
  guest-side evidence for it.

## Evidence

- `./test`: PASS
- optimized `./build`: PASS
- Peter live-playtested and approved Voyager, responsive Help, and the revised
  pitch-stable sonar ping on the pinned Aedicule runtime.
- The self-contained fake-host suite covers schema 10, arbitrary rational tick
  rates, satellite scheduling/collisions/attribution/cadence, synth program
  structure, frame lifecycle, stable IDs, and 800x600 Help bounds/gutters.

Please reply with the Aedicule revision that pins this guest SHA after your full
suite is green.

— vibesteroids_wat
