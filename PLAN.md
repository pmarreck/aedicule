# Plan

- [x] Establish a bounded Wasmtime `host.v0` ABI and neutral WAT fixtures.
  (2026-07-16 EDT)
  - Curiosity poke: which capability first fails to generalize to a second,
    non-game application?
- [x] Integrate full-window GPUI rendering, native input/menus, generated
  audio, deterministic SVG, and transactional live reload. (2026-07-16 EDT)
  - Curiosity poke: can candidate compilation leave the UI thread without
    introducing an ordering race at swap time?
- [x] Prove exact rational tick accumulation, bounded catch-up, deterministic
  snapshots, hostile-input containment, and state-preserving reload. (2026-07-17 EDT)
  - Curiosity poke: should a production policy add Wasmtime epochs as a second
    interruption mechanism beyond deterministic fuel?
- [x] Separate the native frontplane's Nix source closure from mutable WAT
  application data and enforce the boundary structurally. (2026-07-17 EDT)
  - Curiosity poke: can downstream apps substitute a local checkout without
    accidentally making CI impure?
- [x] Remove the Vibesteroids implementation, behavioral tests, research, and
  mechanics documentation from this generic repository. (2026-07-17 EDT)
  - Curiosity poke: does any remaining Rust fixture encode an application rule
    under a generic name?
- [ ] Publish the history-preserving repository rename to `mecha-aedicule` and
  verify its `yolo` CI gate.
  - Curiosity poke: which existing links and clone URLs need compatibility
    redirects after GitHub's automatic redirect expires or is superseded?
- [ ] Add a second, unrelated WAT application as the strongest falsification
  test of the generic ABI.
  - Curiosity poke: would a small editor expose retained widgets, text input,
    focus, accessibility, and persistence gaps more efficiently than another
    canvas application?
- [ ] Decide and test the pre-v1 migration from provisional `gpui-wasm` binary
  names to Mecha Aedicule names.
  - Curiosity poke: can aliases provide a deprecation path without making
    downstream wrappers ambiguous?
