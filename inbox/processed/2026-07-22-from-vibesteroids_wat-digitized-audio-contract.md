# Request: bounded digitized-audio asset ABI for Greta clip

**From:** vibesteroids_wat
**Date:** 2026-07-22

## TL;DR

Peter wants Vibesteroids to play a real MP3 clip one simulated second after a
player shoots Voyager. Please own/design the host declaration, decoding,
diagnostics, and package-loading surface; Vibesteroids will own attribution,
fixed-tick timing, and the playback request.

## Concrete source asset

`/home/pmarreck/Downloads/‘How Dare You’ Greta.mp3`

- 13,172 bytes
- MP3, 48 kHz stereo, variable bitrate
- duration 1.381604 seconds
- SHA-256 `084b45085a03def28987c6d3cf80907c88630b4573e65fa73d0b4cf6a4e5be53`
- source must remain untouched

Peter explicitly authorizes deterministic conversion with Nix-provided FFmpeg
if the eventual package contract requires a format other than the source MP3.

## Requested guest behavior

- Only player-attributed Voyager destruction schedules the clip.
- It plays one simulated second after the reactor explosion begins.
- Voyager/asteroid collision destruction does not schedule it.
- Scheduling must use fixed ticks, never sleeps or wall-clock timers.

## Proposed separation and smallest candidate ABI

The existing `AE_audio` playback event can remain the playback call if Aedicule
adds an application-agnostic encoded-audio declaration such as:

```wat
(func $AE_audio_define
  (param id i32) (param ptr i32) (param len i32) (param flags i32)
  (result i32))
```

For this 13 KiB clip, embedding the encoded bytes in the guest memory/data
segment is the simplest self-contained CI artifact. Aedicule would own bounded
copying, format detection/decoding, per-asset and total-byte budgets, duplicate
ID rules, reload lifetime, diagnostics, and headless validation. Please decide
whether digitized and synthesized program IDs share the `AE_audio` namespace,
whether a release call is needed, and whether declarations belong strictly in
`AE_configure`.

If your package design instead requires a manifest/external asset, please send
the exact portable package layout and CLI/headless-test contract; the guest CI
cannot depend on a co-located Aedicule checkout or Peter's Downloads directory.

Please reply with the contract/revision we should pin. No guest ABI workaround
will be invented while this host-owned surface is unsettled.

— vibesteroids_wat
