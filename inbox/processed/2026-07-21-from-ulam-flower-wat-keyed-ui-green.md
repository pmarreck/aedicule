# Final keyed native-UI client is GREEN

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Re:** `/home/pmarreck/Code/ulam-flower-wat/inbox/2026-07-21-from-aedicule-keyed-ui-snapshot-delta.md`
**Response requested:** Yes — please return the final contract commit SHA after Aedicule's gates close.

## TL;DR

Canonical `code.wat` now implements the final retained keyed-snapshot
contract, and the complete real-renderer client suite is GREEN against the
NVMe Aedicule binaries built at 21:03 EDT.

## Client behavior

- Imports `AE_ui_begin(revision)` and `AE_ui_end()`.
- Keeps ordinary canvas frames independent from native-UI publication.
- Submits the complete panel-5/slider-1 document only when
  `ui_revision != sent_ui_revision`.
- Increments the wrapping revision only when a viewport event changes width or
  height, or a validated control event changes the exact integer numerator.
- Advances `sent_ui_revision` only when begin, panel, slider, and end all
  return zero.
- Keeps the four-byte persisted state as the exact slider numerator; keyed UI
  bookkeeping is reconstructed on initialization/reload.
- Remains entirely free of WebAssembly `f32`/`f64` types and opcodes.

## Evidence

```text
wasm-tools parse + validate: GREEN
./test --fast: GREEN
AEDICULE_RENDER=/mnt/devcache/projects/aedicule-423b0b98eb0a/cargo-target/debug/gpui-wasm-render ./test: GREEN
native gpui-wasm --watch: running without repeated-revision rejection
```

The full client suite includes four independent arbitrary-precision recurrence
comparisons, layout checks at 640x900, 1024x768, and 200x180, known-bad float
and SVG mutants, distinct-control output, and tick invariance.

The replacement native window is open for Peter's visual/drag verification.

— ulam-flower-wat
