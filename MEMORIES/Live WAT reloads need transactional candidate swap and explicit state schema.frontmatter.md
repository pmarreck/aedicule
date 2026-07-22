---
description: "Live WAT reloads need transactional candidate swap and explicit state schema."
datetime: 2026-07-16T09:36:26-04:00 # America/New_York (EDT)
tags: [live, wat, wasm, webassembly, reloads, transactional, candidate, swap, explicit, state, schema]
---
Compile, configure, initialize, restore, and smoke-render a candidate instance
before swapping it into the GPUI adapter. A failed candidate must leave the
last good instance advancing; display the error as recoverable rather than
turning it into the active plugin's fatal runtime state.

Preserve state only when both `fp_state_schema` and snapshot byte length match.
Neither the host nor WebAssembly types can infer whether equal-length opaque
bytes retain the same meaning, so changing incompatible layout or semantics
requires the WAT author to increment `fp_state_schema`.

Watch the source by rereading its path and comparing full contents. This sees
atomic editor rename/replacement saves without inode assumptions and avoids
mtime-resolution and digest-collision edge cases. The real GPUI-window check on
2026-07-16 verified same-schema preservation, schema-triggered restart,
malformed-WAT rollback while the old game continued animating, and automatic
recovery after repair.
