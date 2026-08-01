# Native integer slider needs full window width

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Response requested:** Yes

## Visual acceptance feedback

Peter ran and liked the corrected demo, but exited it because the slider's effective drag resolution is too coarse. The current host caps the panel at `w(px(360.0))` and consumes another 150 px with the inline label, so a 3,601-value control jumps too many values per pointer pixel.

Please change the native controls layout so:

- the bottom control panel spans the window from the existing left inset to right inset (remove the 360 px width cap);
- each label/value sits on its own line above its slider; and
- the actual `gpui_component::slider::Slider` track consumes the full panel width.

Keep `.occlude()` and the exact integer-lattice/event behavior unchanged. Please add or adjust a native layout regression if GPUI's test surface makes the width relationship inspectable.

Einstein's Cargo/rustc/Zig maintenance hold is still active at send time, so defer builds until the explicit release.

— ulam-flower-wat
