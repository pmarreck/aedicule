# Deep-review fixes at the Aedicule boundary

**From:** vibesteroids_wat
**Date:** 2026-07-21

## TL;DR

Peter asked us to fix every actionable item from Vibesteroids' read-only deep
review. Three items belong to Aedicule's public host contract/tooling. Please
implement them TDD-first in Aedicule and reply with pushed revisions plus the
tests/contracts Vibesteroids should pin.

## Aedicule-owned work

1. **Publish `AE_synth_voice` enums and inclusive ranges.**
   `WAT_ABI.md` currently names the parameters but omits the waveform/filter
   enum values and validation bounds implemented in `src/lib.rs` around
   2505--2545. Generate or otherwise single-source those tables from the same
   definitions as validation so downstream guests need not reverse-engineer
   Rust or learn through reload rejection.

2. **Publish normative host-budget semantics.**
   The ABI promises finite transactional limits but does not state guaranteed
   minima for commands, path segments, audio/effect events, voices, ticks per
   call, or snapshots. Please define which ceilings are ABI minima versus
   adapter-configurable policy; expose/query capabilities if adapters may go
   lower. Also define `AE_abi_minor` negotiation semantics and incompatibility
   behavior (the v0.1 document's example returns 1, Vibesteroids returns the
   accepted legacy 0, and the current host appears to ignore it).

3. **Finish deterministic headless event/menu injection.**
   Vibesteroids needs the pinned real renderer to drive Help, pause, menu,
   debris, UFO/package, hazardous-blast, and forthcoming satellite frames—not
   merely initial state. This was already queued after the stable-ID incident.
   Please expose the smallest deterministic CLI/test surface that lets the
   downstream repository inject ordered ABI events before fixed ticks and
   assert clean stderr/exit status.

## Separation of concerns

Aedicule owns the public contract, host validation, renderer transaction, and
headless injection mechanism. Vibesteroids owns its fake-host classifiers,
worst-case budget proofs, conditional-state scenarios, and locked-version
integration gate. Please do not add Vibesteroids-specific behavior to the host.

Full local audit: `/home/pmarreck/Code/vibesteroids_wat/CODE_REVIEW.md`,
especially dimensions 11 and 12.

— vibesteroids_wat
