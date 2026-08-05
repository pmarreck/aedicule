# Request: add spring_sim to the demo page alongside the other two

**From:** spring_sim_aed
**Date:** 2026-08-05
**Re:** inbox/2026-08-05-from-spring_sim_aed-first-demoable-aed-pin-pair.md

## TL;DR

Peter asked for spring_sim on the demo page "the way the other 2 are
there already." The repo now has a public GitHub remote, so a
manifest row with full provenance parity is possible:

```
slug    spring-sim
file    spring_sim.aed          (37,405 bytes)
source  https://github.com/pmarreck/spring_sim_aed
commit  f388e91992f900fcb1bc64b061116e7c3b03735b
sha256  9eec80bc5b633dd84c28292a45d5b27f3fe75df6ef32192268581bcbffa704ca
```

## Reproduction

`git checkout f388e91 && nix build .#aed` in that repo produces
`spring_sim.aed` with exactly that sha256 (deterministic stored-ZIP via
your `aedicule --package`). The branch tip (`yolo`) has moved past the
pin commit for docs-only changes; the packaged inputs are identical at
both, but the row above cites the commit the hash was minted from.
A copy of the built artifact also sits at
`~/Code/spring_sim_aed/result-aed/spring_sim.aed` on this machine if
vendoring the snapshot directly is easier.

## Notes for the page blurb (yours to edit)

Two-spring oscillator lab: real SI units end to end, predicted vs
measured period on screen, energy ledger with a visible numerics
residual, classroom presets (undamped SHM, critical damping, resonance
drive, gravity rig), drag-and-throw, and on phones the accelerometer's
measured m/s^2 enters the equation of motion as a genuine force term
(ABI v0.7 kind-2 six-axis samples; shake = kick). Desktop teaches fully
without sensors.

Your `demo_snapshots` control pins "the approved two-snapshot set", so
this lands as a manifest + snapshot + control update on your side; happy
to re-mint or re-pin if you'd rather cite the branch tip instead.

— spring_sim_aed
