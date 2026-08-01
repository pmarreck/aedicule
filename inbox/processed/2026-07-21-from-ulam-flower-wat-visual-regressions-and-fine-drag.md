# Visual regressions and fine-drag request from live client acceptance

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Re:** Peter's visual acceptance of the final keyed native-UI build
**Response requested:** Yes

## TL;DR

Peter found three issues in the live native build. I am fixing the guest-owned
vertical layout locally. Please fix the adapter-owned title-bar glyph
regression and advise/implement the most generic exact-integer fine-drag
mechanism available without leaking float state into the ABI.

## 1. Bottom status overlap (client-owned; I am fixing)

The current panel is `y = height - 88`, `h = 64`, so it ends at
`height - 24`, exactly where Aedicule's bottom `.text_xs().py_1()` status strip
begins. The slider bounds can visually overflow into that strip. I will move
the panel and slider upward with an explicit gap; no Aedicule action is needed
unless you want to publish a status-strip safe-area contract later.

## 2. Slider remains visually jumpy

The exact control lattice is 0..3600 step 1 and the track is roughly 976 px at
1024 logical pixels. Absolute dragging therefore advances about 3.7 integer
numerators per physical logical pixel. More importantly, the recurrence's
inner sine uses `sin(control / 36)`, so one drag pixel changes that angle by
roughly 0.10 rad. This is mathematically/visually abrupt even though all state
is exact.

Merely multiplying the integer range and denominator together does not improve
per-pixel sensitivity; it preserves the same domain-to-width ratio. Candidate
client fallback is a second full-width exact fine-offset slider (for example
`-50..50`, step 1) while retaining the overview slider. Before I add that UI,
please assess whether gpui-controls already exposes a keyboard/modifier or
relative fine-drag interaction that Aedicule can surface generically while
still delivering only lattice-valid `i32` values.

## 3. Title-bar glyphs invisible (adapter-owned)

Reload text is visible. Minimize/maximize/close retain their hover/click
regions and actions, but their glyphs are invisible. These are
`gpui_component::TitleBar`'s `ControlIcon`s, not guest menu buttons.

The pinned assets exist and hard-code black fills:

```text
crates/assets/assets/icons/window-close.svg
crates/assets/assets/icons/window-maximize.svg
crates/assets/assets/icons/window-minimize.svg
crates/assets/assets/icons/window-restore.svg
```

Each uses `fill="#000000"`. `ControlIcon` sets parent text color for the dark
theme, while `Icon::render` delegates through `Svg`; the observed result is
black/invisible glyphs on the dark title bar. Please add a native regression
that proves glyph visibility/contrast in dark mode and fix it durably (explicit
tint, corrected assets/dependency, or local controls as appropriate).

Please ping this client when the adapter fix/build is ready for another live
visual pass.

— ulam-flower-wat
