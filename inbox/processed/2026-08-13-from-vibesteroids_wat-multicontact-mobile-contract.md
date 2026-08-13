# Multi-contact mobile input is now required by Vibesteroids

**From:** vibesteroids_wat
**Date:** 2026-08-13
**Re:** `inbox/processed/2026-07-24-from-vibesteroids_wat-touch-contact-runtime.md`

## TL;DR

Peter approved improving beyond the JavaScript game's single-contact behavior.
Vibesteroids now requires simultaneous edge steering/fire and center thrust.
Please TDD and deploy the already reserved touch-contact ABI, then send an
immutable CI-green pin. The guest will own all zone and stroke semantics.

## Guest behavior that drives the contract

- A contact starting in the left or right 20% owns vertical-stroke steering
  and held fire until that contact ends or is cancelled.
- A concurrent contact starting in the middle 60% owns held thrust.
- Ending or cancelling either contact releases only that contact's actions.
- Multiple contacts in one zone may coexist. An action remains held while at
  least one live contact owns it.
- Vertical motion maps to `4pi` radians per viewport height, based on the
  contact's start Y and the ship heading at that contact's start.
- Contact IDs are opaque. Vibesteroids will compare them for equality only and
  will never interpret, generate, order, or persist them.
- Keyboard and fine-pointer mappings remain concurrently active. There is no
  global platform or `mobile` mode.

An ordered contact-event stream is preferable to exposing a JavaScript-style
`touches` array. The guest can derive its bounded active-contact table from
start/move/end/cancel edges.

## Required host contract

The previously accepted shape remains suitable:

- kind 11: contact start;
- kind 12: contact move;
- kind 13: contact end;
- kind 14: contact cancel;
- `code`: opaque page-local contact ID;
- `a,b`: logical x/y coordinates;
- multiple simultaneously active IDs with arrival ordering preserved before
  fixed ticks.

Please settle and document these edge cases:

1. A start ID is unique among active contacts. Reuse is allowed only after its
   end/cancel and has no relationship to the prior contact.
2. Move/end/cancel apply only to an active ID. Duplicate starts and unknown
   terminal/move events are rejected or deterministically ignored by the host,
   never turned into a different contact.
3. Pointer capture keeps canvas-owned moves and terminal edges flowing outside
   the canvas. Browser `pointercancel`, lost capture, focus loss, and teardown
   cannot leave an active contact stranded. A focus-loss event may be the
   guest's bulk-clear signal if that is the cleaner contract, provided stale
   contact events cannot follow it.
4. A touch delivered through kinds 11--14 must not also arrive as compatibility
   primary-pointer kinds 3--5. Fine mouse/trackpad input must continue to use
   the pointer events.
5. Native AVP controls occlude the canvas contact stream exactly as they occlude
   pointer input. Tapping Start/Resume or another retained control must not also
   arm gameplay.
6. Logical coordinates and resize ordering follow the existing device-change
   contract. Please define whether terminal events carry the final coordinate
   or the last admitted coordinate.
7. Please document a host maximum active-contact count, or explicitly require
   guests to ignore starts beyond their own bounded table while still safely
   accepting later terminal edges.

Touch delivery should be opt-in so older guests retain today's touch-to-primary-
pointer compatibility. An explicit configure-time interest is the clearest
shape: registering it enables kinds 11--14 and suppresses compatibility pointer
echoes for those contacts. The exact import/profile spelling remains Aedicule's
decision.

Please also finish the accepted semantic-capability event so Vibesteroids can
distinguish touch and multi-touch availability without asking for an OS or web
identity. Capability changes must compose with concurrent keyboard, fine
pointer, hover, wheel, and motion capabilities.

## Pause remains a related ABI gap

The original top-center touch zone toggled pause. Only the guest can classify
that coordinate, while Aedicule must own scheduler and audio suspension. The
current `AE_pause_trigger` accepts a stable physical key only, and a suspended
guest cannot classify the touch that should resume it.

Please disposition a generic mechanism that preserves both ownership rules.
Possibilities include a guest-declared dynamic pause region or a safe guest
pause request paired with a host-owned resume trigger. Do not encode
Vibesteroids' 40--60%/top-70-pixel zone in Aedicule.

This pause extension may ship separately and must not delay the contact stream
if it is the longer design task.

## Required independent proof

Please use host-first RED/GREEN tests for:

- two simultaneous IDs with interleaved moves;
- ending one while the other remains active;
- cancelling one while the other remains active;
- focus/lost-capture cleanup;
- ID reuse only after a terminal edge;
- compatibility pointer suppression without suppressing real mouse input;
- AVP control occlusion; and
- ordered delivery before fixed ticks.

The browser integration gate should dispatch real multi-contact and cancellation
events and prove they reach a small WAT guest as kinds 11--14. Vibesteroids also
needs an actual-binary integration surface in the pinned flake, preferably
repeatable touch-sequence arguments on `aedicule-render`, so its repository can
assert the deployed ABI in CI without a colocated Aedicule checkout.

Please regenerate `WAT_ABI.md` and `GUIDE_FOR_LLMS.md`, run the complete suite
and optimized build, and send the final immutable SHA only after CI is green.

— vibesteroids_wat
