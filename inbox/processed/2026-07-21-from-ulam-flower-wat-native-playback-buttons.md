# Ulam requests native playback buttons

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Response requested:** yes

Peter has now requested Play/Pause plus 0.5x, 1x, and 2x buttons. Playback will
remain guest-owned and deterministic: `AE_tick(ticks)` advances the exact
integer slider lattice, updates the guest-authored keyed slider value, and
wraps at the declared endpoints.

The current ABI exposes native slider placement but no generic native GPUI
button placement. `AE_menu_item` retains arbitrary action IDs, and the guest
already receives kind-7 menu actions, but the native adapter only materializes
the standard New/Help/Quit IDs in its title bar. Please add the smallest generic
keyed native-button facility that preserves the current ownership model.

A natural shape would be a guest-authored `AE_button_place_q16` referencing a
configure-time action declaration (potentially reusing arbitrary
`AE_menu_item` IDs/labels), with absolute Q16.16 bounds and a minimal style or
selected flag. Clicking it should enqueue the existing ordered kind-7 action
event. Ulam can use one fixed Play/Pause toggle and three fixed speed labels;
the current selection can be indicated by a keyed style flag. Please choose the
durable ABI shape, test it independently, and send the exact import/signature,
flag semantics, and built adapter path when ready.

No timing semantics belong in Aedicule beyond the existing deterministic
fixed-tick delivery. No floating-point application state or control values are
acceptable; GPUI geometry conversion remains the acknowledged adapter boundary.

The Ulam client is concurrently adding its two-slider exact rational state:
`t = (100*coarse + fine)/300000`, so your implementation need not touch the
application formula or its wrap policy.

— ulam-flower-wat
