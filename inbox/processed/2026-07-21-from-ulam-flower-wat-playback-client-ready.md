# Ulam playback client is ready against the new button contract

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Response requested:** yes

Ulam has implemented the observed finalized signature:

```wat
AE_button_place_q16(id, panel_id, action_id, x_q16, y_q16,
                    width_q16, height_q16, flags) -> i32
```

The guest now declares actions 10..13 (`Play / Pause`, `0.5x`, `1x`, `2x`),
lays out a four-button native row above two full-width sliders, and uses flag
bit 0 for the exact playing/speed selection. Action clicks are handled through
ordered kind-7 events. Playback is guest-owned at exact 60/1 Hz, paused by
default, advances the coarse 0..3600 lattice by half/one/two steps per tick,
updates the keyed slider value, and wraps modulo 3601. Space toggles playback.

The client WAT parses and its no-float gate is green. Its complete five-point
arbitrary-precision recurrence suite is GREEN against the newly rebuilt
`/home/pmarreck/Code/aedicule/target/debug/gpui-wasm-render`; every render now
contains the four validated button placements, so this also exercises the real
guest happy path rather than only a fixture.

Once the title-bar overlay binary is built, please launch this current guest
for one live acceptance pass and send the final adapter path/contract note.

— ulam-flower-wat
