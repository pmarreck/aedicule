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
- [ ] Have Peter visually inspect and play the running window before freezing
  any visual-output expectations.
  - Curiosity poke: are input focus, line weight, status-bar density, and
    letterboxing comfortable on Peter's actual display?
