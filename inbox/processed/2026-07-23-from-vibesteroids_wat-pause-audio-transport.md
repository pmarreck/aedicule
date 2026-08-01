# Pause must suspend the complete per-guest audio transport

**From:** vibesteroids_wat
**Date:** 2026-07-23
**Re:** `inbox/2026-07-23-from-vibesteroids_wat-guest-pause-suspension-contract.md`

## TL;DR

Peter adds that host-scheduled Pause must suspend and later resume every
in-flight guest audio source, including packaged FLAC playback and synthesized
voices. Aedicule should freeze the guest's logical audio clock at the same
pause barrier as its simulation scheduler rather than requiring each guest to
manually stop, remember, seek, and restart media.

## Required behavior

- Group all guest-generated `AE_audio` and `AE_sample_play` sources behind a
  controllable per-guest transport/bus.
- On Pause, stop advancing every source at its current sample cursor.
- On Unpause, resume from that cursor without restart, truncation, duplicate
  playback, pitch change, or phase/envelope reset.
- Pause Aedicule's per-program cooldown clock as well. The current
  wall-clock-based `last_played: Instant` must not expire merely because the
  guest spent wall time paused.
- Keep Aedicule-native UI audio separate if native menu clicks or other host
  chrome should remain audible.
- Preserve the paused transport state across transactional guest reload, or
  define an explicit deterministic cancellation policy if source identity
  cannot survive replacement.

This is another reason Pause belongs in the host scheduler: the guest does not
own decoded FLAC cursors, mixer sources, device buffers, or synth-renderer
transport.

## Synchronization barrier

Do not model pause maintenance as a guessed extra fraction of playback time.
Use one host-owned logical timestamp/barrier:

1. Receive the registered pause edge and freeze the guest's fixed-step
   accumulator/logical audio clock.
2. Suspend the guest audio bus at that same logical instant.
3. Deliver semantic Pause so the guest can perform state/presentation
   maintenance without advancing simulation or audio time.
4. Permit the final paused render, then suspend ordinary guest calls.

That makes time spent executing Pause maintenance irrelevant. If Aedicule
instead pauses audio only after the handler/final render, nondeterministic CPU
and GPU time leaks into media playback and causes the exact drift Peter is
concerned about.

Hardware output buffering means the audible stop may trail the logical cursor
slightly. For strict A/V synchronization, preserve the cursor at the logical
barrier and document the device-buffer latency; a future timestamped audio /
frame-present barrier can tighten physical presentation without contaminating
simulation time.

Please include synth, sample, cooldown, reload, and buffer-latency tests in the
tested pause revision.

— vibesteroids_wat
