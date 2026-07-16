# Plan

- [x] Define the spike boundary and provisional ABI in SPEC.md. (2026-07-15 EDT)
  - Curiosity poke: does a push-style render ABI remain usable in a browser
    host where nested WASM calls may be mediated by JavaScript?
- [x] Inventory Vibesteroids as the conversion oracle while leaving its dirty
  working tree untouched. (2026-07-15 EDT)
  - Curiosity poke: which signature features belong in the first proof, and
    which would obscure the host/plugin feasibility question?
- [x] Complete the first optimized sandboxed Nix build after dependency
  vendoring. (2026-07-16 01:06 EDT)
  - Curiosity poke: can the unusually large GPUI graph be pruned without
    creating a fork-maintenance burden?
- [x] Establish a Nix/Rust project with pinned GPUI, gpui-component,
  Wasmtime, WAT, and audio dependencies.
  (2026-07-16 00:17 EDT)
  - Curiosity poke: will upstream Git dependencies build hermetically without
    forcing a maintenance-heavy vendoring scheme?
- [x] Write failing headless ABI tests for module validation, lifecycle,
  deterministic input/ticks, rendering, menus, audio, snapshots, and limits.
  (2026-07-16 00:06 EDT)
  - Curiosity poke: can the tests derive expectations from protocol invariants
    rather than duplicating the WAT implementation?
- [x] Add a non-Asteroids conformance fixture for affine composition, paths,
  clipping, and an animated/rotated sprite command.
  (2026-07-16 00:11 EDT; clipping remains specified, while paths,
  transforms, resources, and sprites are executable)
  - Curiosity poke: are image decoding and animation clocks clearly on the
    correct sides of the trust and determinism boundary?
- [x] Implement the generic frontplane core and make the headless suite pass.
  (2026-07-16 00:11 EDT)
  - Curiosity poke: can every GPUI-independent function remain pure or adapter
    injected?
- [x] Write and validate the Vibesteroids WAT plugin.
  (2026-07-16 00:36 EDT)
  - Curiosity poke: does direct WAT remain comprehensible once collision,
    bullet pools, menu actions, and serialization are present?
- [x] Integrate GPUI/gpui-component rendering, keyboard input, menus, and
  generated audio.
  (2026-07-16 00:17 EDT; decoded sprite-atlas pixels remain a documented
  adapter follow-up)
  - Curiosity poke: can the event loop avoid reentrancy and keep Wasmtime off
    sensitive GPUI borrow scopes?
- [x] Harden the trust boundary with transactional output rollback, explicit
  table-element and coordinate limits, and set-based hostile fixtures.
  (2026-07-16 01:25 EDT)
  - Curiosity poke: should a production policy use Wasmtime epochs as a second
    interruption mechanism in addition to deterministic fuel?
- [x] Remove demo-specific title/menu/control copy from Rust and drive native
  standard actions from plugin metadata. (2026-07-16 01:32 EDT)
  - Curiosity poke: how should plugin-specific action IDs map to GPUI's
    compile-time Rust action types without weakening the generic boundary?
- [x] Run ./test and ./build, launch the Nix-wrapped demo, document honest
  limitations, and commit the green spike. (2026-07-16 01:47 EDT)
  - Curiosity poke: what evidence would falsify the claim that the ABI is
    generic rather than merely Asteroids-shaped?
- [x] Have Peter visually inspect and play the running window before freezing
  any visual-output expectations. (2026-07-16 08:05 EDT; confirmed playable
  and essentially complete as a proof of concept)
  - Curiosity poke: are input focus, line weight, status-bar density, and
    letterboxing comfortable on Peter's actual display?
- [x] Reproduce and fix the zero-sized GPUI canvas reported on framework-nixos.
  (2026-07-16 07:31 EDT)
  - Curiosity poke: does the same adapter layout survive native Wayland,
    XWayland, fractional scaling, and manual window resizing?
- [x] Add a deterministic, headless frame-export path so humans and agents can
  inspect exact WAT output without relying on a live desktop session.
  (2026-07-16 07:31 EDT)
  - Curiosity poke: should the long-term visual oracle compare semantic scene
    commands, raster pixels, or both at different test layers?
- [x] Rebuild, transfer the corrected executable, and verify animation plus
  keyboard/menu input through captured before/after frames.
  (2026-07-16 07:31 EDT)
  - Curiosity poke: can the deployment preserve the expensive Rust dependency
    closure while rebuilding only project sources?
- [x] Load `./code.wat` by default, accept an explicit WAT file/application
  directory, and retain an explicit embedded-demo mode.
  (2026-07-16 09:35 EDT)
  - Curiosity poke: should a missing or invalid startup file fail closed or
    open the embedded demo with a visible recoverable error?
- [x] Add transactional reload preparation that preserves same-schema state,
  restarts incompatible schemas, and never replaces a working plugin with a
  candidate that fails compilation, validation, restoration, or rendering.
  (2026-07-16 09:35 EDT)
  - Curiosity poke: can a same-schema semantic change still be unsafe despite
    matching version and length, and how clearly must the plugin contract say
    that incrementing the schema is the author's responsibility?
- [x] Add `--watch`, manual Reload/Ctrl+R, directory-safe content polling, and
  nonfatal reload errors that leave the old game advancing.
  (2026-07-16 09:35 EDT)
  - Curiosity poke: how do we avoid retry storms from editor atomic-save event
    bursts without missing a later correction?
- [x] Verify live calculation edits preserve an active game, schema changes
  restart it, broken WAT leaves it playable, and the deployed GPUI window
  reflects each outcome.
  (2026-07-16 09:35 EDT)
  - Curiosity poke: should compilation move off the UI thread if larger WAT
    applications make synchronous reload latency perceptible?
- [x] Polish the public README and package metadata while labeling the ABI and
  platform support honestly as proof-of-concept quality.
  (2026-07-16 13:19 EDT)
  - Curiosity poke: which successful experiments could readers mistake for a
    production security or cross-platform support claim?
- [x] Add a minimal GitHub Actions gate using the canonical Nix package build,
  which already executes the release-profile Rust suite.
  (2026-07-16 13:19 EDT)
  - Curiosity poke: will the clean GPUI dependency graph fit comfortably inside
    GitHub-hosted runner storage and time limits?
- [x] Scan the complete Git history for secrets, create the public
  `pmarreck/gpui-wasm` repository, push `yolo`, and independently verify the
  remote commit and default branch.
  (2026-07-16 13:25 EDT)
  - Curiosity poke: which local-only files or metadata could become public
    despite not appearing in the ordinary source tree?
- [x] Derive a source-specific Vibesteroids behavior and algorithm
  specification, including exact symbols, constants, quirks, and a staged
  implementation map for the WAT demo. (2026-07-16 17:00 EDT)
  - Curiosity poke: which behaviors are intentional gameplay and which are
    browser/mobile accommodation or historical accident?
- [x] Replace the three-circle demonstration with a richer deterministic wave:
  clear score/level/lives HUD, irregular rotating rocks, multiple bullets,
  asteroid splitting, particles, and visible progression.
  (2026-07-16 17:47 EDT; also includes starfield, debris, respawn, game over,
  bounded pools, and fuel-safe headless tick slicing)
  - Curiosity poke: what fixed-capacity state layout keeps handwritten WAT
    inspectable without making collisions or rendering look synthetic?
- [x] Prove the richer rules through failing tests before implementation,
  deterministic SVG inspection, and a captured real GPUI window.
  (2026-07-16 18:16 EDT: failing-to-passing behavioral suite, complete test
  runner, optimized Nix build, SVG inspection, and Peter's real-window
  playtest are green)
  - Curiosity poke: can an impressive initial frame still conceal weak motion,
    collision, or wave-transition behavior?
- [x] Document and ship the source-derived upgrade, including the live
  state-preserving ship-physics edit Peter observed on the real Thelio.
  (2026-07-16 18:58 EDT: complete suite and optimized Nix build green;
  committed for push)
  - Curiosity poke: should the demo fidelity work remain one coherent commit or
  split spec, mechanics, and presentation into separately green commits?
- [ ] Build an explicit gap matrix from `VIBESTEROIDS_BEHAVIOR_SPEC.md`, then
  implement every applicable original behavior rather than relying on memory.
  - Curiosity poke: which browser/mobile accommodations need a native analogue,
    and which are genuinely inapplicable outside a touch browser?
- [ ] Migrate every internal gameplay quantity and calculation to signed
  decimal fixed-point integers, bumping the state schema exactly once; convert
  to GPUI's `f32`-backed `Pixels` only at the one-way rendering boundary.
  - Curiosity poke: what decimal scale and pre-square rescaling preserve useful
    subpixel precision at 8K dimensions without overflowing WebAssembly `i64`?
- [ ] Make the game field fill the complete drawable window and make viewport
  changes regenerate stars while translating every world object by
  `new_center - old_center` without stretching velocity or trajectories.
  - Curiosity poke: how should offscreen wrap buffers and respawn-safe regions
    behave across a drastic resize?
- [ ] Add a discoverable plugin-owned Help/Controls experience through a
  genuinely generic native action or overlay mechanism.
  - Curiosity poke: can dynamic plugin actions remain native and accessible
    without creating one compile-time Rust action type per application action?
- [ ] Replace application-specific semantic tones with a bounded generic synth
  description capable of reproducing Vibesteroids' oscillator ramps, gain
  envelopes, filtered noise explosions, siren, thrust, and extra-life chimes.
  - Curiosity poke: what is the smallest non-game-specific audio graph that is
    deterministic, resource-bounded, schedulable, and independently testable?
- [ ] Implement and test the semi-secret Death Blossom, including availability,
  activation, timed radial fire, rotations, siren, and per-life reset behavior.
  - Curiosity poke: what discoverable native input preserves the feature's
    semi-secret character when device-shake input is absent?
