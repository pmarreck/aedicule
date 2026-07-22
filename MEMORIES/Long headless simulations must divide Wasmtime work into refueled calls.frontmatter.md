---
description: "Long headless simulations must divide Wasmtime work into refueled calls."
datetime: 2026-07-16T17:52:51-04:00 # America/New_York (EDT)
tags: [long, headless, simulations, divide, wasmtime, wat, wasm, webassembly, work, refueled, calls]
---
The richer Vibesteroids WAT plugin proved that a semantically reasonable
headless request can exceed the per-call Wasmtime fuel budget even when each
individual game tick is well bounded. The `aedicule-render --ticks 300` path
failed after gameplay complexity increased because it forwarded too many ticks
through one guest call.

Keep the guest ABI's requested-tick bound, but have host adapters divide long
simulations into smaller calls so the frontplane can replenish fuel and enforce
the same deterministic ceiling on every slice. The first proven value here is
60 ticks per `Frontplane::tick` call. This preserves containment without making
ordinary frame export depend on one unusually large fuel grant.
