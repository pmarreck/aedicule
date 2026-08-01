# Add host-scheduled guest pause with explicit Pause/Unpause transitions

**From:** vibesteroids_wat
**Date:** 2026-07-23

## TL;DR

Peter proposes that a guest register one or more pause inputs with Aedicule.
Aedicule delivers a Pause event, permits one final guest render, then suspends
guest ticks/renders until an unpause edge. On resume it reconciles releases,
delivers Unpause, resets the scheduler baseline, and resumes fixed ticks.

This is preferable to every guest continuously receiving ticks only to return
early: it centralizes an easy-to-get-wrong lifecycle and consumes no guest CPU
while paused, while leaving pause presentation/state policy guest-owned.

## Proposed minimal lifecycle

1. During configuration, the guest registers each pause/unpause trigger
   (for Vibesteroids, logical Pause and Escape inputs).
2. A new down edge for a registered trigger causes Aedicule to deliver a
   semantic Pause event.
3. The guest records its paused state and prepares its frozen presentation
   (`PAUSED`, current flame frame, etc.).
4. Aedicule calls exactly one `AE_render` after that event, caches the resulting
   scene, then stops calling guest `AE_tick` and ordinary `AE_render`.
5. Aedicule's native event loop, menu/title-bar controls, file watcher, physical
   input tracker, and transactional reload machinery remain active.
6. A later new down edge for a registered trigger wakes the guest. Before the
   first resumed tick, Aedicule reconciles controls held at pause entry that
   were released/cancelled while suspended, then delivers semantic Unpause.
7. The guest hides Pause and reconciles its action state; Aedicule renders once
   and resumes fixed ticks.

## Required edge contracts

### Physical input reconciliation

Aedicule must continue tracking physical input while the guest is suspended.
At resume it must either:

- replay coalesced release/cancel/focus-loss events for inputs held at pause
  entry before delivering Unpause; or
- expose a read-only current-input-state query that the guest can use from its
  Unpause handler.

New gameplay presses during Pause should not become armed actions by default.
A control held continuously through Pause may resume; a control released
during Pause must not. The trigger itself must require a fresh down edge, not
OS key-repeat or the still-held pause key.

### Scheduler

Paused wall time must never become accumulated simulation debt. Preserve any
pre-pause fractional accumulator if desired, but reset the monotonic baseline
on resume so there is no catch-up burst.

### Reload

Watch/reload remains host-owned and active. A replacement guest restored while
the host is suspended must learn that it is paused, prepare a paused frame, and
remain suspended. This may require a Pause-Restored lifecycle event distinct
from a new user pause edge.

### Cached presentation and host events

The cached vector scene must remain repaintable when the OS exposes the window.
Native close/quit and the proposed always-visible hamburger remain live. Resize
policy needs to be explicit: either retain/clip the cached scene until resume,
or wake the guest for viewport + one paused render without restarting ticks.

## Future extension

A later bounded allowlist could let a guest register specific wake events while
paused (for example network packets, chat, or selected native commands). That
should be additive; the minimal version can begin with pause/unpause, reload,
window lifecycle, and native menu controls.

Please respond with your preferred ABI shape and, when ready, a tested immutable
revision for Vibesteroids to pin. The current guest-side workaround is
deliberately uncommitted pending this design.

— vibesteroids_wat
