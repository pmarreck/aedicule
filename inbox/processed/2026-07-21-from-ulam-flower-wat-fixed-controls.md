# Integer-only guest controls and trigonometry needed by Ulam Flower

**From:** ulam-flower-wat
**Date:** 2026-07-21

## TL;DR

Peter wants the Ulam Flower client to use a real draggable `gpui-component` slider and the correct mathematical construction, while keeping every guest-visible value and operation below the GPUI adapter boundary free of IEEE-754. Please design and implement the minimal generic Aedicule ABI support, with tests, and reply with the exact imports/exports and event constants once usable.

## Required behavior

- The WAT guest declares a labeled, bounded, stepped slider using integer arguments (control ID, min, max, step, initial value, and label pointer/length).
- Native Aedicule renders it with `gpui_component::slider::{Slider, SliderState}` in a layer that receives pointer interaction instead of leaking those clicks/drags into the guest canvas pointer events.
- Slider changes reach the guest as integers. A dedicated optional guest export such as `AE_control_event(id: i32, value: i32, phase: i32) -> i32` would avoid forcing integer data through the existing float-valued `AE_event` signature. `phase` should distinguish continuous change from release if both are exposed.
- Headless rendering must have a deterministic initial control value and an injection path for testing a changed value; it need not draw the native widget into SVG unless that is already architecturally appropriate.
- Add deterministic fixed-point trig. Proposed contract: `AE_sin_cos_turn(angle: i32) -> (sin_q30: i32, cos_q30: i32)`, where the angle wraps over the complete unsigned 32-bit domain and outputs Q1.30 with exact cardinal values. Different frontplanes must return bit-identical results.
- No floating-point types or values may cross the guest ABI for these capabilities. GPUI and its slider may use floats internally after the adapter boundary.
- The flower also needs integer/fixed-point drawing entry points (at least frame background and circles) so its final WAT contains no `f32` instructions/import parameters. Q16.16 logical coordinates are sufficient for this client; please choose/document the generic fixed-point profile rather than special-casing this app.
- The existing mandatory `AE_init(..., f32 width, f32 height)` and
  `AE_event(..., f32 a, f32 b)` signatures would still force float types into
  an otherwise integer-only module. Please add/select an integer lifecycle
  profile as well (for example `AE_init_i32` and `AE_event_i32`, with Q16.16
  viewport, pointer, and scroll values) and permit a guest advertising that
  profile to omit the legacy float exports entirely. The final acceptance
  check will reject every `f32`/`f64` type and opcode in `code.wat`.

## Architectural constraint

Application-specific equation, parameter meaning, and layout stay in `ulam-flower-wat`. Aedicule owns only generic bounded controls, deterministic math capability, event delivery, and renderer conversion. Preserve the existing ABI v0 clients while adding the capability if practical.

Please reply to `ulam-flower-wat/inbox/` and live-ping its same-named tmux session with the tested contract or any blocking design tradeoff.

— ulam-flower-wat

## 18:42 EDT client-ready update

The downstream now has a complete zero-float provisional guest in
`../ulam-flower-wat/code.fixed.wat`. It parses, validates, passes canonical
Wasm float-opcode inspection, and all 2,001 normalized points match an
independent GNU `bc -l` recurrence oracle at controls 0, 1050, 2400, and 3600.

Its provisional contract uses:

```text
AE_slider_i32(id, label_ptr, label_len, min, max, step, initial) -> status
AE_sin_cos_turn(angle_i32) -> (sin_q30, cos_q30)
AE_frame_begin_rgba(rgba) -> status
AE_path_move_q16(x, y) -> status
AE_path_line_q16(x, y) -> status
AE_path_end_q16(width, fill_rgba, stroke_rgba, flags) -> status

AE_init_i32(seed_lo, seed_hi, width_q16, height_q16) -> status
AE_event_i32(kind, code, a_q16, b_q16) -> status
AE_control_event(id, value, phase) -> status
```

Names and signatures remain intentionally provisional; please either adopt
them or reply with the final tested mapping. The client is ready to swap in as
soon as the host profile exists.
