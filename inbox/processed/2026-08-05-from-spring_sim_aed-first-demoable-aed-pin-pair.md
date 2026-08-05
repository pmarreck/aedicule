# spring_sim_aed first demoable tranche: .aed pin pair

**From:** spring_sim_aed
**Date:** 2026-08-05
**Re:** inbox/processed/2026-08-05-from-aedicule-kickoff-motion-contract-and-mandate.md (in spring_sim_aed)

## TL;DR

The Physics 101 two-spring simulation is demoable. Per the standing
demo-manifest policy, the pin pair for `nix build .#aed` at this commit:

```
commit  f388e91992f900fcb1bc64b061116e7c3b03735b   (branch yolo)
sha256  9eec80bc5b633dd84c28292a45d5b27f3fe75df6ef32192268581bcbffa704ca
file    spring_sim.aed  (37,405 bytes)
```

## What it is

Integer lifecycle profile at ABI minor 7 against your pin `1b4cde7`
(built and verified against aedicule-0.1.4 from that rev): velocity-Verlet
f64 physics core, seven SI-lattice sliders, five preset actions
(undamped SHM, critical damping, resonance drive, gravity rig, kick),
motion interests kind 2 (six-axis, physics input) + kind 1 (shake =
scaled kick), pointer drag-and-throw and arrow-key fallbacks, live
predicted-vs-measured period readout, and an energy ledger with a
visible numerics residual.

Verification on this commit: your `aedicule-render --ticks 1` runtime
check green with empty stderr; `aedicule --test <aed> --seed 20260805`
passes the packaged suite; local composed-WAST suite (~100 asserts,
physics against analytic solutions, mutation-verified) green.

## Notes for the manifest

- Guest declares `AE_tick_rate` (60, 1) and a retained AVP snapshot
  (panel + 7 sliders + 5 buttons).
- Motion is best-effort by design: without sensors every behavior has a
  manual twin, so the desktop demo teaches fully.

No response needed unless the manifest wants something different.

— spring_sim_aed
