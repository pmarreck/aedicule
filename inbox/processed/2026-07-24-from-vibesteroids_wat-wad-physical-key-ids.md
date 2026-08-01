# Add stable physical-key IDs for W, A, and D

**From:** vibesteroids_wat
**Date:** 2026-07-24

## TL;DR

Peter requested `W/A/D` aliases for thrust/rotate-left/rotate-right. The current
Aedicule ABI exposes arrow IDs but no `W`, `A`, or `D`, and both native and web
classifiers therefore discard those keys before the guest can own their game
semantics.

## Requested host work

Please add documented, append-only stable physical-key IDs for `W`, `A`, and
`D`, map their GPUI/native and browser key representations, and return a tested
immutable pin. Aedicule should deliver only ordinary down/up edges; mapping
them to thrust or rotation remains Vibesteroids-owned.

Acceptance should include:

- ABI numeric round-trip/classification for all three new IDs;
- native GPUI-name mapping for lowercase/physical `w`, `a`, and `d`;
- browser key-down and key-up crossing the actual WAT boundary, not merely
  appearing as DOM events;
- focus-loss and suspension reconciliation remaining generic rather than
  knowing these gameplay meanings;
- documentation/generated-ABI agreement and unchanged existing key IDs.

Please choose and publish the exact append-only IDs rather than relying on the
guest to infer provisional numbers. Vibesteroids will then TDD action aliases,
independent simultaneous-source ownership, Help copy, and its real-host pin.

— vibesteroids_wat
