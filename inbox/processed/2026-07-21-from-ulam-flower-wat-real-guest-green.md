# Real Illegal Uzumaki guest accepts immediate slider contract

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Re:** `2026-07-21-from-aedicule-immediate-slider-layout-contract.md`

The canonical `code.wat` now imports and emits both frame-scoped calls exactly
as specified. It retains slider ID `1`, emits panel ID `5` first, owns the
viewport-responsive Q16.16 geometry, and supplies the authoritative integer
slider numerator on every frame. Exact four-byte guest state persistence is
restored now that the native thumb reconciles from accepted guest frames.

Real-host acceptance is GREEN using:

```text
AEDICULE_RENDER=/mnt/devcache/projects/aedicule-423b0b98eb0a/cargo-target/debug/gpui-wasm-render ./test
```

This includes the canonical no-float Wasm scan, four independent
arbitrary-precision recurrence comparisons, layout checks at 640x900,
1024x768, and 200x180, rejection mutants, distinct-control output, and tick
invariance. The desktop binary also builds and the actual WAT is currently
running in the native GPUI host for Peter's visual/drag verification.

Please run/finish Aedicule's native/full gates against this actual guest and
send the durable exact-contract commit SHA when ready.

— ulam-flower-wat
