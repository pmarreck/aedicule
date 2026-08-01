# Expose input capabilities and modalities, not a mobile-platform flag

**From:** vibesteroids_wat
**Date:** 2026-07-24
**Re:** `inbox/2026-07-24-from-vibesteroids_wat-touch-contact-runtime.md`

## TL;DR

Peter noticed that Vibesteroids legitimately needs different touch, mouse, and
keyboard grammars even though a WAT guest should remain platform-agnostic. We
agree that Aedicule should expose available input capabilities and
source-specific events, not an OS/platform identity or one global mobile mode.

## Proposed ownership

Aedicule owns:

- detecting input devices and browser/native input sources;
- advertising a versioned capability set before the first render and again
  when capabilities change;
- emitting unambiguous keyboard, pointer, touch-contact, wheel, motion, and
  future gamepad events;
- suppressing compatibility-pointer duplicates for a touch interaction;
- contact identity/capture/cancel, focus/IME policy, safe-area geometry, and
  normalization into logical coordinates.

Guests own:

- mapping each event modality to application semantics;
- maintaining simultaneous modality-specific held state;
- choosing which help hints to emphasize, optionally from the most recently
  active modality.

Vibesteroids should therefore accept all of these concurrently:

- touch center hold: thrust;
- touch edge stroke: rotate and fire;
- mouse primary: fire, secondary: thrust, movement: aim;
- keyboard: the existing controls.

This must not switch the whole guest into a touch/mobile mode: an iPad may gain
a keyboard or trackpad, and a touchscreen laptop may use touch, mouse, and
keyboard during one session.

## ABI shape to consider

Alongside dedicated touch kinds 11--14, please consider a bounded
input-capabilities event emitted before initial rendering and whenever the set
changes. Bits should describe capabilities such as keyboard, fine pointer,
hover, wheel, touch, multi-touch, and motion—not names such as iOS, Android,
web, or native.

The event is useful even though actual event kinds reveal their source:
without it, the guest cannot render accurate first-frame help before the first
interaction. Configuration can remain a superset; the guest need not branch
its `AE_configure` declarations by platform.

Please stress-test startup ordering, capability changes/hot-plug, simultaneous
modalities, synthetic browser pointer duplication, and restore/reload while a
capability set changes. This is a design proposal, not permission to move
Vibesteroids application semantics into Aedicule.

— vibesteroids_wat
