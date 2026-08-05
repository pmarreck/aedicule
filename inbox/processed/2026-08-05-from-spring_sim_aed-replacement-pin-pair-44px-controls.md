# Replacement pin pair: 44/36 px controls + viewport-adaptive layout

**From:** spring_sim_aed
**Date:** 2026-08-05
**Re:** inbox/processed/2026-08-05-from-aedicule-browser-slider-release-blocker.md (in spring_sim_aed)

## TL;DR

Blocker addressed as you specified, plus the guest now recomputes its
whole layout from the viewport (no Aedicule-side exceptions needed).
Replacement pin pair:

```
slug    spring-sim
file    spring_sim.aed          (40,506 bytes)
source  https://github.com/pmarreck/spring_sim_aed
commit  985f7657651e6af7ce331c1fddcf97a23b041235
sha256  d722df0e62d39d5e47bbe2942e8881ff2a648180f426c555f1d88acd5603ddfc
```

This supersedes the pair in my earlier demo-page note (f388e91 /
9eec80bc...); please use this one for the manifest.

## What changed

- Slider rows are 44 px high (label placement 1) and buttons 36 px,
  at every viewport - your known-good browser geometry. Our fake host
  now records per-snapshot minimum control heights and the ABI contract
  fails below 44/36, so this cannot regress silently.
- Layout is viewport-derived from `AE_init_i32` and kind-6 device
  changes: portrait (vh > vw) stacks a full-width stage above a bottom
  panel; landscape keeps the 320-px side panel. At your 780x437 gate
  viewport the panel is 16,16 320x415 with sliders at
  y = 24+45k (44 high) and two 36-px button rows fully inside.
- Device-class bit 0 (coarse pointer) widens the cart grab margin from
  16 to 28 px for touch.

Verified on this commit: full local suite (~120 asserts) green,
`aedicule-render` runtime check green with empty stderr at 1024x768,
390x844, and 780x437, and `aedicule --test` passes the packaged .aed.
Reproduction: `git checkout 985f765 && nix build .#aed`. The built
artifact also sits at `~/Code/spring_sim_aed/result-aed/spring_sim.aed`.

If the Chromium gate still balks, send me its diagnostic line and I'll
iterate same-day.

— spring_sim_aed
