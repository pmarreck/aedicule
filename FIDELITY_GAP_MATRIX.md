# Vibesteroids Fidelity Gap Matrix

This is the implementation ledger between the source-derived behavior in
`VIBESTEROIDS_BEHAVIOR_SPEC.md` and the WAT application. A behavior is not
complete merely because it looks similar: its state, transitions, rendering,
audio event, snapshot behavior, and boundary cases must be covered where
applicable.

## Deliberate platform and product decisions

| Source behavior | GPUI/WAT decision | Reason |
|---|---|---|
| Browser URL parameters | Provide validated native launch options for seed and initial level | A native process has no URL; deterministic launch remains useful. |
| Browser test-mode URL and in-page test UI | Keep the Rust/WAT headless test binaries | The native independent harness is stronger and already canonical. |
| Eruda mobile console and HTTPS development server | Not applicable | These exist solely to support a browser/mobile runtime. |
| Device-motion permission and shake activation | Defer until a host motion capability exists | A desktop keyboard action remains available; the generic ABI must not pretend to have sensors. |
| Browser touch controls | Implement pointer/touch semantics only after the generic pointer ABI is bounded and tested | GPUI can eventually supply pointer input, but this is separate from fixed-decimal and desktop parity. |
| Resize by proportional coordinate scaling | Translate positions by `new_center - old_center` | Peter explicitly selected center-relative preservation so resize does not stretch trajectories. |
| Ship drag `0.99` | Use `0.995` per simulation tick | Peter selected the gentler coefficient during a live, state-preserving WAT edit. |
| Refresh-rate-dependent browser delta timing | Use deterministic 60 Hz integer ticks | Replay, tests, and snapshots must not depend on display timing. |
| Ambient randomness for thrust/audio | Keep gameplay randomness seeded; keep audio noise outside gameplay state | Rendering/audio adapters must never feed nondeterminism back into simulation. |

## Numeric architecture — blocking prerequisite

| Requirement | Current state | Completion evidence |
|---|---|---|
| Decimal fixed-point world values | **Missing:** state and physics use `f32` | Schema 3 stores positions, velocities, directions, radii, angles, timers expressed as spatial/time values, and derived physics values as signed `i64` integers at a named power-of-ten scale. |
| Integer-only simulation | **Missing:** movement, collision, wrapping, rotation, and damping use `f32` | A structural classifier rejects floating-point arithmetic, loads, and stores outside explicitly marked host-scalar conversion adapters. |
| Safe fixed-point multiplication and distance tests | **Missing** | Boundary tests cover negative values, `0.995` damping exactly, 8K viewports, collision equality, and overflow-safe squared distances. |
| One-way host conversion | **Partial:** GPUI consumes `f32`, but the guest also computes with it | Host `f32` viewport values convert once on ingress; final guest draw scalars convert once on egress; no converted value re-enters gameplay state. |
| Explicit incompatible schema transition | **Missing** | `fp_state_schema` changes from 2 to 3 once; the watcher restarts instead of restoring schema-2 bytes. |

## Gameplay and state

| Requirement | Current state | Remaining work / proof |
|---|---|---|
| Seeded deterministic restart | **Implemented, partial launch surface** | Add validated native seed/level launch options and retain exact snapshot/replay tests. |
| Five initial parent rocks, +1 per wave | **Implemented** | Retain capacity and progression tests after the schema migration. |
| Level difficulty formulas | **Approximate** | Port the specified acceleration, rotation, bullet, asteroid, fire-delay, and cap formulas to fixed decimal; test levels 1, 10, 25, and caps. |
| Ship rotation, thrust, wrap, and `0.995` drag | **Approximate** | Use fixed-decimal direction/rotation and exact integer damping; preserve Peter's chosen drag deviation. |
| Bullet origin, velocity inheritance, cadence, lifetime, and cap | **Partial** | Derive lifetime from half the viewport diagonal, port exact level cadence/speed, and test full-pool failure. |
| Asteroid construction and irregular outlines | **Partial** | Generate 8–12 seeded points with source-derived radii, speed, angular motion, spawn edges, and limits. |
| Bullet/ship collision and splitting | **Implemented, approximate constants** | Port all thresholds/impulses to fixed decimal and test boundary equality plus 60% child radii. |
| Scoring and repeated 20,000-point extra ships | **Implemented** | Preserve threshold-crossing and semantic-audio tests after migration. |
| Ship death, debris, safe respawn, bomb, and game over | **Implemented, approximate timing/geometry** | Port exact state transitions and fixed-decimal safe-region tests. |
| Pause, focus release, restart | **Implemented** | Add Escape alongside P and ensure every simulation quantity freezes. |
| Auto-fire toggle (`F`) | **Missing** | Add a generic key code, state latch, visible Help entry, and deterministic cadence tests. |
| Kid Mode (`K`) | **Missing** | Suppress score/life loss and ordinary HUD while retaining explosions, respawn behavior, and Death Blossom indicator. |
| Death Blossom (`B`) | **Missing** | Add per-life availability, eligibility rules, 7.2-rad/s rotation, 12 rotations, two-tick effective fire cadence, base bullet speed, cancellation/reset, HUD/message, siren, and mid-effect snapshot tests. |

## Viewport, rendering, and controls

| Requirement | Current state | Remaining work / proof |
|---|---|---|
| Game fills drawable window | **Missing:** host letterboxes a fixed 1024×768 scene | Remove fixed-aspect projection; make gameplay the full window content with controls/status overlaid rather than consuming the field. |
| Runtime viewport events | **Missing:** plugin starts at a fixed logical size | Detect actual canvas size changes and dispatch one bounded viewport event per distinct size. |
| Center-relative resize | **Missing:** only width/height are replaced | Translate ship, bullets, rocks, particles, and debris by the center delta; preserve velocities and directions exactly. |
| 100 deterministic stars regenerated on resize | **Partial:** 48 render-derived stars | Keep a separate fixed seed 42, regenerate exactly 100 fixed-decimal positions for each viewport, and prove gameplay RNG is unchanged. |
| Recognizable vector ship/rocks/debris/particles | **Implemented, approximate** | Retain semantic scene-command tests while moving all geometry calculations to fixed decimal before draw conversion. |
| Neon title/byline splash and timing | **Partial** | Implement the source-derived show/hold/fade timing at new game and respawn; do not let rendering mutate state. |
| HUD, pause, game-over, and Play Again presentation | **Partial** | Complete original information hierarchy, Kid Mode hiding, Death Blossom status/message, and native restart affordance. |
| Discoverable Help/Controls | **Missing** | Add a standard generic Help action plus plugin-owned overlay listing keyboard controls, including F/K/B. |
| Pointer/touch overlays | **Deferred** | Specify and test a generic pointer capability before implementing desktop click/mobile parity. |

## Audio

| Requirement | Current state | Remaining work / proof |
|---|---|---|
| Application-independent host audio | **Missing:** Rust maps Vibesteroids IDs to sine tones | Define a bounded guest-declared synth graph/command format; host validates and renders it without knowing game semantics. |
| Shot | **Approximate** | Reproduce sine 800→200 Hz and 0.1 s gain decay. |
| Thrust | **Approximate** | Reproduce 60 Hz sawtooth through the low-pass envelope and rate-limit events. |
| Asteroid explosion | **Approximate** | Reproduce shaped white noise, bass peak, low-pass sweep, and 0.5 s gain envelope. |
| Ship explosion | **Approximate** | Reproduce layered white/brown noise, impulse, band-pass/low-pass shaping, and 1.2 s envelope. |
| Death Blossom siren | **Missing** | Reproduce three scheduled 400→800→400 Hz whoops. |
| Extra life | **Approximate** | Reproduce five scheduled sawtooth chimes with the specified ratios and filters. |
| Failure isolation and bounds | **Partial** | Invalid graphs, excessive voices/duration/events, or unavailable devices fail silently to gameplay while yielding diagnostic host state. |

## Completion gate

The fidelity phase is complete only when:

1. every non-deferred row above is covered by a deterministic test;
2. simulation snapshots contain no IEEE-754 gameplay values;
3. the float-boundary classifier passes;
4. `./test` and the optimized `./build` pass;
5. Peter visually verifies the full-window presentation, Help, sounds, and
   Death Blossom in the real GPUI window; and
6. documentation names every intentional difference from browser
   Vibesteroids.
