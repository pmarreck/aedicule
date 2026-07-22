---
description: "Guest declared audio can remain application agnostic and decimal fixed through the PCM boundary."
datetime: 2026-07-16T20:43:38-04:00 # America/New_York (EDT)
tags: [guest, declared, audio, remain, application, agnostic, decimal, fixed, pcm, boundary]
---
The generic frontplane must not map application-owned audio program IDs to
host-known sound names. Let WAT declare bounded voices by waveform, schedule,
frequency/gain envelopes, filter sweep, and cooldown, then group them by the
opaque program ID in the host.

Integer descriptor units make this natural: millihertz for frequency,
parts-per-million for gain, and milliseconds for schedule. Render oscillator
phase, interpolation, deterministic noise, filters, and mixing as signed
decimal millionths. Convert the WAT audio call's host scalars once on ingress
and the finished integer PCM samples once to rodio's `f32` buffer on egress.
Structural tests should scan a marked fixed-audio region for `f32`/`f64`, and
behavior tests should prove scheduling silence, deterministic output, nonzero
body samples, exact sine cardinal points, and bounded amplitude.

For stable integer filters, derive a decimal angular coefficient from
`6.283185 * cutoff / sample_rate`; use `omega / (1 + omega)` for a bounded
one-pole low-pass coefficient and a clamped omega for the state-variable
band-pass. The exact Web Audio transfer function is platform-specific anyway;
this retains the source sound's shape while making replay deterministic.
