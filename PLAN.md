# Plan

- [x] Make the launcher CLI regression test self-contained instead of relying
  on a developer's private `$HOME/dotfiles` checkout. (2026-07-20 15:58 EDT)

- [x] Establish a bounded Wasmtime ABI and neutral WAT fixtures.
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
- [x] Publish the history-preserving repository rename, now canonical at
  `pmarreck/aedicule`.
  (2026-07-17 09:00 EDT: GitHub renames preserved public history and redirects;
  canonical remote and `yolo` commit independently verified, with CI observed
  through the ship gate)
  - Curiosity poke: which existing links and clone URLs need compatibility
    redirects after GitHub's automatic redirect expires or is superseded?
- [x] Rename the local project directory to `aedicule`, matching the canonical
  repository and future organization-neutral identifier. (2026-07-17 09:04 EDT)
  - Curiosity poke: which local tools cache absolute project paths and need a
    clean rebuild or session recreation after a directory rename?
- [x] Configure Mechatron Prime's exact-SHA Nix build/test targets and dynamic
  README badge. (2026-07-17 18:40 EDT: local Nix package and pure test targets
  passed; its first webhook delivery needs the privileged Thelio provisioner.)
  - Curiosity poke: can a later Cargo dependency-artifact layer keep the
    isolated test derivation from recompiling GPUI and Wasmtime unnecessarily?
- [x] Provision the Thelio-signed GitHub webhook for `pmarreck/aedicule`.
  (2026-07-18 14:37 EDT: GitHub hook `654189443` is active, accepts `push`,
  and returned `200` from Thelio.)
- [x] Push the first post-provisioning `yolo` commit and confirm its Mechatron
  build reaches `PASSING` and creates the dynamic badge JSON.
  (2026-07-18 14:39 EDT: signed push `7152ff8` was accepted; the public badge
  endpoint reports `PASSING`.)
- [x] Make `run` resolve the Aedicule flake and Cargo manifest from its own
  script directory when an adjacent application invokes it.
  (2026-07-19 11:07 EDT: fake-Nix foreign-CWD regression, full suite, and
  optimized build passed.)
  - Curiosity poke: does this remain correct when a caller invokes a symlink to
    the launcher rather than the checked-out script directly?
- [x] Standardize native pointer buttons and two-axis wheel motion, including
  primary/secondary/middle down, up, and up-out edges; mirror initial/reload WAT
  rejection diagnostics to stderr while retaining the fallback or previous
  guest. (2026-07-20 15:07 EDT: full core suite, native GUI/unit/CLI tests,
  optimized build, and warning-denied release compile passed.)
  - Curiosity poke: should a future pointer-capture contract distinguish an
    OS-cancelled gesture from an ordinary release without adding guest state?
- [x] Complete the host half of the 120 Hz migration by wiring the exact
  rational accumulator into the live GPUI loop with bounded catch-up. Do not
  call the end-to-end migration complete until `vibesteroids_wat` proves equal
  elapsed-time behavior at 60 and 120 Hz in WAST and Peter playtests it.
  (Host half completed 2026-07-17 10:53 EDT; downstream proof and playtest
  remain open.)
  - Curiosity poke: how should the loop expose dropped catch-up ticks so a
    temporarily overloaded machine cannot hide simulation slowdown?
  - [ ] Confirm equal elapsed-time behavior at 60 and 120 Hz in downstream WAST,
    then complete Peter's live playtest.
- [x] Hard-cut over the WAT boundary to `aedicule.v0` / `AE_*`, negotiate
  rational guest simulation rates after explicit display-refresh events, and
  generate one build-checked WAT ABI reference.
  (2026-07-17 13:00 EDT: host core tests and optimized Nix build passed; the
  downstream WAST guest-observation proof remains separately owned.)
  - Curiosity poke: can a future ABI generator share import type declarations
    with linker bindings, not only the import allowlist and documentation?
- [ ] Add a portable native display-refresh source that reports a window move
  across displays once, then routes it through the completed core event/rate
  negotiation path. GPUI's public display API currently exposes identity and
  bounds but not refresh mode or a change notification.
  - [x] Add the exact `AE_DISPLAY_REFRESH_RATE` process-start fallback, with
    documented canonical 1000/1001 aliases and a build-checked ABI reference.
    (2026-07-17 15:40 EDT)
  - Curiosity poke: should this land as an upstream GPUI API, a small
    platform-adapter trait, or both, without making display mode ambient guest
    authority?
- [ ] Revisit the deferred GPUI display-refresh and presentation-policy
  proposal with Peter before any upstream discussion or implementation.
  See `GPUI_REFRESH_API_PROPOSAL.md`; Zed requires a human-owned contribution.
- [ ] Define a bounded application-asset package and Rust reader, reconciling
  the `blar`/`mini_blar` wire-format versions before choosing a canonical
  profile for independently versioned WAT applications.
  - Curiosity poke: can indexed uncompressed entries remain zero-copy while
    compressed entries enforce strict decoded-size and checksum limits?
- [ ] Add guest-controlled digitized-audio playback from packaged assets, with
  PCM/WAV as the baseline and feature-gated MP3 and Ogg/Vorbis decoding.
  - Curiosity poke: can long tracks stream under a bounded decode-memory budget
    without making audio-device timing observable by the deterministic guest?
- [ ] Load bounded texture assets from the same package into the existing
  content-hashed image-resource cache.
  - Curiosity poke: which decoded-pixel, dimension, and archive-entry limits
    prevent a small compressed asset from becoming an allocation bomb?
- [ ] Add a second, unrelated WAT application as the strongest falsification
  test of the generic ABI.
  - Curiosity poke: would a small editor expose retained widgets, text input,
    focus, accessibility, and persistence gaps more efficiently than another
    canvas application?
- [ ] Decide and test the pre-v1 migration from provisional `gpui-wasm` binary
  names to Mecha Aedicule names.
  - Curiosity poke: can aliases provide a deprecation path without making
    downstream wrappers ambiguous?
