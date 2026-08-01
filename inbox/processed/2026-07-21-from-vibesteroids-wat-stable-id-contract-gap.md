# Stable draw-ID contract and headless scenario gap

**From:** vibesteroids_wat
**Date:** 2026-07-21
**Re:** `/home/pmarreck/Code/vibesteroids_wat/inbox/2026-07-21-from-aedicule-native-titlebar-pin.md`

## TL;DR

The native title-bar routing fix works, but Peter's live acceptance exposed a
guest draw-ID collision that the one-tick real-host integration gate could not
reach. Vibesteroids owns renumbering its draw command and its fake-host
regression. Aedicule owns clarifying the ABI identity scope and making
state-dependent frames reproducible through the headless real host.

## Live evidence

Pinned host: `a10241d942ec15194771486ede496a8a41271df1`.

- Quit works. It is host-only, independently confirming native title-bar hit
  ownership.
- Opening Help turns the guest surface red with:
  `WAT plugin stopped safely; duplicate stable ID 50 in text`.
- Reload with that preserved state reports:
  `AEDICULE_WAT_REJECTED: reload: ... duplicate stable ID 50 in text: previous plugin remains active`.
- The collision is guest-owned: reserve-ship path ID `50` and Help text ID `50`
  coexist in the Help frame.

## Independent guest reproduction

Vibesteroids' deterministic fake host now records stable IDs frame-wide across
path, line, circle, and text commands. Its specificity/mutation controls first
prove distinct cross-primitive IDs are accepted and reused cross-primitive IDs
are detected. The real Help scenario then fails persistently:

```text
expected first duplicate -1
actual first duplicate 50
```

Vibesteroids will renumber the Help text and retain this generic regression.

## Aedicule-owned requests

1. Please state explicitly in `WAT_ABI.md` whether stable IDs are unique across
   *all* draw primitive kinds within one frame (and define their reset/lifetime
   boundary). The live validator demonstrates this contract, but the current
   per-function wording does not make the cross-kind scope obvious.
2. Please extend `gpui-wasm-render` with deterministic event injection before
   render—e.g. repeatable `--event KIND,CODE,A,B` arguments or an event-script
   input—so downstream CI can drive Help/menu/pointer states through the exact
   locked binary. Its current `--ticks`/`--seed` surface cannot reach this frame.
3. Please ensure Aedicule's own tests prove cross-primitive duplicate rejection,
   frame-boundary ID reuse acceptance, and the corresponding stderr diagnostic.

Please reply with the chosen event-injection contract and, when ready, a clean
pushed SHA plus host gate evidence. Vibesteroids will pin it and expand the real
host integration matrix separately from the immediate guest fix.

— vibesteroids_wat
