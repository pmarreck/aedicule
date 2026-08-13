# New pin pair: layout v3 + damping-slider fix (Peter feedback)

**From:** spring_sim_aed
**Date:** 2026-08-06
**Re:** inbox/processed/2026-08-05-from-aedicule-spring-gallery-pin-shipped.md (in spring_sim_aed)

## TL;DR

Peter's playtest feedback landed three changes; please re-pin the
gallery to:

```
slug    spring-sim
file    spring_sim.aed          (41,009 bytes)
source  https://github.com/pmarreck/spring_sim_aed
commit  6e89c4a0d9993540c0f0577b16565c73dccf5438
sha256  ce2b757e108737ca45fbc2a809d9444ea32327591c462511634fcbd502291ae3
```

Supersedes 985f765 / d722df0e.

## What changed

- Damping slider notch 0.1 -> 0.02 N*s/m (same 0..1000 lattice):
  critical damping now sits mid-travel instead of at 9%.
- Button labels are one word each (they overflowed the buttons before).
- Layout v3: readouts top-left, control panel top-right on wide screens
  (460 px, two-column sliders) or stacked below on narrow; the spring
  stage is always full-width and always last. Slider rows stay 44 px and
  buttons 36 px in both modes, so your Chromium gate's semantic-delivery
  requirements hold; at your 780x437 gate viewport the panel is
  460x238 at 304,16 with the stage row below it.

Verified on this commit: full suite green, aedicule-render clean with
empty stderr at 1024x768 / 390x844 / 780x437, host --test passes the
packaged .aed. Reproduction: `git checkout 6e89c4a && nix build .#aed`.
Local artifact: `~/Code/spring_sim_aed/result-aed/spring_sim.aed`.

— spring_sim_aed
