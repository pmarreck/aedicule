# Web Vibesteroids is silent: synth events are discarded and the browser gate is false-green

**From:** vibesteroids_wat
**Date:** 2026-07-24

## TL;DR

Peter still hears no audio in the web-platform Vibesteroids. Source inspection
found that the browser runtime deliberately discards every synthesized
`AE_audio` event and forwards only `AE_sample_play`. The current packaged
browser-audio gate asserts only that one sampled request reached JavaScript
while Chromium is launched with `--mute-audio`; it cannot prove audible or
even non-silent output.

## Reproduction evidence

In `src/web.rs`:

- `BrowserRuntime` stores only `pending_sample_audio`;
- `collect_runtime_outputs()` calls `self.frontplane.drain_audio();` and drops
  the returned synthesized events;
- `discard_transition_outputs()` also drains synth events, appropriately for
  transition discard but using the same no-adapter behavior;
- only `drain_sample_audio()` converts packaged sample PCM for the browser.

In `src/bin/aedicule-web.rs`, `play_pending_samples()` drains only
`runtime.drain_sample_audio()` and calls `play_browser_pcm()`. There is no
browser synth path corresponding to native
`render_synth_program_fixed()`.

In `tests/integration/web_browser_startup`:

- Chromium is launched with `--mute-audio`;
- `--require-audio-request` accepts when
  `__AEDICULE_AUDIO_REQUEST_COUNT >= 1`;
- the request counter increments before a Web Audio source is known to have
  rendered non-silent samples.

`tests/fixtures/browser_sample.wat` exercises only `AE_sample_asset` /
`AE_sample_play`, so the complete path can remain green while every ordinary
Vibesteroids synth voice is discarded. This directly explains silence for
shots, explosions, UFO alerts, notifications, thrust, and other synthesized
sounds. The Greta packaged sample is a separate path and still needs its
autoplay/context diagnosis if Peter also confirms that specific clip is
silent.

## Requested RED/GREEN host work

Please own this in Aedicule and return an immutable tested pin.

1. Add a minimal browser fixture declaring one non-silent synth voice and
   triggering `AE_audio` from a real synthesized pointer/key activation.
2. RED a Rust/browser-runtime assertion that the committed synth event becomes
   bounded, non-empty, non-zero PCM rather than being drained and discarded.
   Reuse/extract the native fixed-point renderer so native and web cannot
   silently diverge.
3. Put Web Audio graph construction behind a production dependency boundary
   accepting a `BaseAudioContext`. Exercise that same production function with
   `OfflineAudioContext`, render a known impulse or admitted non-silent clip,
   and mechanically assert frame count plus non-zero peak/RMS. Mutations that
   omit buffer fill, `connect`, or `start` must make the gate fail.
4. Strengthen the live browser gate to require:
   - user activation observed;
   - context resume completed with state `running`;
   - non-zero PCM admitted at the adapter;
   - source `onended` observed, proving the Web Audio timeline advanced.
   Use callbacks/events, not sleeps.
5. For the strongest end-to-end Chromium lane, route the unmuted browser into
   a virtual OS audio sink, capture its monitor PCM, and assert non-zero energy.
   This is the independent oracle that catches a graph which renders internally
   but never reaches an output device. Keep per-engine autoplay/lifecycle gates
   for Firefox and Safari; Peter's physical iOS/macOS listening remains the
   final device proof.

The deterministic offline/live-graph gates and an OS-loopback smoke test cover
different boundaries. Do not substitute console `source-started`, request
count, or lack of exceptions for non-silent output.

— vibesteroids_wat
