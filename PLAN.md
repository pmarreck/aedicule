# Plan

- [ ] Make `.aed` the portable application unit without sacrificing the bare
  WAT edit/run loop.
  - [x] Specify and test deterministic stored-ZIP packaging, safe depackaging,
    and byte-identical application lookup across WAT, directory, and archive.
    (2026-07-22 13:57 EDT)
  - [x] Add `--package` / `--depackage` CLI actions with derived output names,
    spaces in paths, clean diagnostics, and no source-tree mutation.
    (2026-07-22 13:57 EDT)
  - [ ] Add `--appify` / `--deappify` as the platform-delivery layer above
    `.aed`: combine a pinned Aedicule runtime with one exact guest package into
    a normal current-platform application, then losslessly recover its
    constituent runtime/package artifacts and build manifest. Use one file
    where the platform supports it and a native application bundle where it
    does not; preserve the embedded `.aed` bytes exactly.
  - [x] Make `--test` require `tests/main.wast`, run every direct
    `tests/*.wast` suite in deterministic PCG32/Fisher-Yates order with an
    emitted replay seed, and ignore nested WAST composition fragments across
    bare-WAT virtual roots, directories, and `.aed` archives.
    (2026-07-22 13:57 EDT)
  - [ ] Add `--scaffold PROJECT` with a runnable guest, `aedicule.toml`, a
    passing `tests/main.wast`, documented empty asset/lib roots, and strict
    refusal to overwrite or invent licensing.
  - [x] Add a loopback-safe `--web` adapter that serves any application source
    through the pinned browser runtime, prints its URL, and emits COOP/COEP.
    (2026-07-22 13:57 EDT)
  - [x] Carry the browser runtime in native deliveries and test runtime
    discovery without host-specific paths.
    (2026-07-22 13:57 EDT)
  - [x] Preserve the documented downstream tool surface by packaging both
    `aedicule` and `aedicule-render` in the native `frontplane` flake output.
    (2026-07-22 15:33 EDT; caught by Vibesteroids' pinned runtime gate)
  - [x] Make the headless renderer resolve bare WAT, directory, and `.aed`
    inputs through the same bounded virtual application root and asset catalog.
    (2026-07-22 15:40 EDT; caught by Vibesteroids' real packaged FLAC guest)
  - [x] Expose immutable `assets/` FLAC samples through bounded
    `AE_sample_asset` / `AE_sample_play` imports, with identical bare-WAT,
    directory, archive, and reload admission plus native device playback.
    (2026-07-22 14:48 EDT)
  - [x] Carry the same admitted FLAC catalog through dynamic and static browser
    delivery, convert fixed PCM only at the WebAudio boundary, and prove a real
    packaged Vibesteroids request in headless Chrome without audible test
    output. (2026-07-22 22:43 EDT)
  - [ ] Expose packaged image assets by virtual name without granting arbitrary
    virtual-filesystem reads, then add browser image-adapter parity.
  - Curiosity poke: non-loopback browsers require a secure context as well as
    COOP/COEP, so remote/tailnet serving needs an explicit TLS/reverse-proxy
    contract rather than silently printing an unusable HTTP URL.
  - Curiosity poke: platform signing/notarization mutates or wraps appified
    artifacts, so specify the unsigned deterministic core before promising a
    byte-identical appify/deappify/appify round trip.

- [ ] Publish the bundled browser demos as a concise, modern GitHub Pages site.
  - [x] Add a deterministic Pages workflow that deploys the exact
    `delivery-web` output from `yolo`.
    (2026-07-22 11:30 EDT)
  - [x] Preserve shared-Wasm-memory startup on Pages' headerless static host
    through a same-origin service-worker isolation adapter.
    (2026-07-22 11:30 EDT)
  - [x] Prove a Pages-style headerless server reaches a cross-origin-isolated,
    drawable WebGPU canvas in headless Chrome.
    (2026-07-22 11:30 EDT: Ulam Flower and Vibesteroids both passed the full
    synthetic mouse, keyboard, and multi-touch probe.)
  - [x] Ship and verify the public gallery, both direct demos, the package
    catalog/FLAC, and the downloadable `.aed`, then link them from the README.
    (2026-07-22 23:45 EDT; public HTTP and headless WebGPU/input probes passed.)
  - [ ] Upgrade GitHub's Pages actions to the current Node-24 majors:
    `configure-pages@v6`, `upload-pages-artifact@v5`, and `deploy-pages@v5`.
    The v0.1.1 workflow passed, but GitHub annotated the older majors as
    Node-20 compatibility shims. (Official release metadata checked
    2026-07-22 23:23 EDT.)
  - Curiosity poke: does mobile Safari accept the service-worker-controlled
    reload and expose both SharedArrayBuffer and a usable WebGPU adapter?

- [ ] Ship Aedicule and the Ulam Flower/Vibesteroids demos as one coherent
  six-target delivery matrix.
  - [x] Make `./build` produce the optimized current-platform application and
    add `./build_all` for exactly web, macOS/aarch64, Linux/aarch64,
    Windows/aarch64, Linux/x86_64, and Windows/x86_64.
    (2026-07-22 02:57 EDT)
  - [x] Let single-derivation native builds use every detected CPU thread while
    preserving bounded multi-derivation scheduling for the six-target matrix.
    (2026-07-22 15:48 EDT; fake-Nix policy regression covers both paths)
  - [x] Package the Aedicule application/document icon family and native `.wat`
    launch metadata (macOS UTType/document roles, Windows drop-path handling,
    Linux MIME/desktop metadata).
    (2026-07-22 02:58 EDT)
  - [x] Include immutable, independently tested Ulam Flower and Vibesteroids
    WAT snapshots in each release without absorbing their source repositories.
    (2026-07-22 02:58 EDT)
  - [x] Bundle the byte-pinned schema-11 Vibesteroids `.aed` with its FLAC,
    standard WAST entry point, README, and license in every native delivery and
    expose both the identical application as a direct Pages download and its
    exact package-backed WAT/FLAC as the playable browser guest.
    (2026-07-22 22:43 EDT)
  - [x] Produce deterministic archives with manifests/checksums and publish
    them as GitHub Release assets.
    - [x] Audit v0.1.0 with independent Nix rebuilds; remove compile-time
      `ahash` seeds from Linux, canonicalize/sign the Mach-O UUID from content,
      and strip nondeterministic COFF metadata from Windows.
      (2026-07-22 08:57 EDT)
    - [x] Add a fail-accumulating `./check_reproducible` release gate that
      independently rebuilds the raw web and five native targets before any
      tag is published. (2026-07-22 08:57 EDT)
    - [x] Pass that gate on the complete v0.1.1 tree for web, macOS/ARM64,
      Linux/ARM64, Windows/ARM64, Linux/x86_64, and Windows/x86_64.
      (2026-07-22 09:24 EDT)
    - [x] Publish corrective v0.1.1 without erasing the defective v0.1.0 audit
      trail. All six release archives independently reproduced, their uploaded
      digests match the local checksum set, and manifests were published at
      <https://github.com/pmarreck/aedicule/releases/tag/v0.1.1>.
      (2026-07-23 00:23 EDT.)
    - [x] Add a resumable `./publish` adapter that binds a tracked-clean pushed
      revision to its source version/tag, runs local gates, watches the
      independently reproduced GitHub Release, verifies its macOS checksum,
      and optionally installs an ad-hoc-signed private test copy on Peter's
      SSH-reachable tailnet Mac while preserving the previous app in Trash.
      Focused red/green behavior, `./test`, the isolated Nix test derivation,
      `./build`, `./build_all`, ShellCheck/actionlint, and all six artifact
      checksums passed. (2026-07-23 01:26 EDT.)
      - [x] Dogfood both new-publication and existing-release resume paths on
        immutable v0.1.2; all six independently rebuilt GitHub assets and both
        CI systems passed. The first live Mac install then failed safely before
        touching `~/Applications`: Nix's extracted app was read-only, and the
        remote shell's `rm` resolves to the interactive rm-safe adapter.
        (2026-07-23 03:08 EDT.)
      - [x] Reproduce those two Mac-only assumptions under the publisher CLI
        test, make only the private staging tree writable before ad-hoc signing,
        and use `/bin/rm` only for publisher-owned remote temporary paths.
        `./test`, the isolated Nix check, `./build`, `./build_all`, static
        analysis, and all six v0.1.3 archive checksums pass before commit.
        (2026-07-23 03:44 EDT.)
    - [ ] Make the public macOS archive Gatekeeper-clean by adapting
      `../validate_gui`'s external release-keychain, Developer ID hardened
      runtime/timestamp, Apple notarization, stapling, `spctl`, and receipt
      contract. The private tailnet test install remains ad-hoc signed until
      this separate credentialed publication gate is implemented.
  - [x] Expand Mechatron Prime and GitHub Actions gates to falsify the complete
    target/archive matrix, then watch both after pushing.
    - [x] Pass `./test`, the isolated Nix test derivation, `./build`,
      `./build_all`, every emitted checksum, and independent realization of
      all six raw delivery targets. (2026-07-22 10:03 EDT)
    - [x] Reproduce and fix clean-runner-only browser-test failures: remove the
      private capture-helper dependency, include the top-level runner in the
      sandbox source, and suppress first-fetch Nix progress before classifying
      compiler diagnostics. (2026-07-22 04:24 EDT)
    - [x] Poll for Chrome's initial debuggable page after `DevToolsActivePort`
      appears instead of assuming both become ready atomically; cover the CI
      race with an injected no-sleep unit regression and the real Pages-style
      Chrome probe. (2026-07-22 16:26 EDT)
    - [x] Install cross-built release executables directly instead of letting
      nixpkgs duplicate GPUI's entire target tree after compilation.
      (2026-07-22 17:00 EDT; reproduced from GitHub's post-build ENOSPC log and
      covered by the CI-structure gate.)
      Curiosity poke: verify every target keeps the expected executable suffix
      and no delivery closure depends on discarded build artifacts.
    - [x] Hard-cut every runtime, package, launcher, and delivery artifact to
      the canonical `aedicule` / `aedicule-render` executable names, and include
      the launcher in the isolated Nix test source so Mechatron exercises the
      repository-wide legacy-name classifier.
      (2026-07-22 19:08 EDT; exact `checks.x86_64-linux.test` target passed.)
    - [x] Retry whole-tree headless-Chrome profile removal after late helper
      writes outlive Node's internal recursive-delete retry window.
      (2026-07-22 19:17 EDT; deterministic injected-race regression, real
      Pages-style WebGPU/input probe, and complete suite passed.)
    - [x] Make Chromium startup failure bounded, give sandboxed browser helpers
      a private writable home, and constrain Fontconfig to pinned DejaVu rather
      than scanning ambient system/profile roots. The isolated Nix gate, real
      packaged-FLAC browser probe, `./build`, six-target `./build_all`, and all
      emitted checksums passed. (2026-07-22 22:43 EDT)
    - [x] Remove Nix's `DBUS_SESSION_BUS_ADDRESS=disabled:` sentinel only from
      the headless Chrome process while preserving real Unix/abstract/TCP
      addresses. The pure set-classifier and real packaged-FLAC browser probe
      passed after the inherited sentinel had prevented DevTools startup.
      (2026-07-23 00:43 EDT.)
  - [ ] Re-run the browser startup probe and obtain Peter's iPhone visual
    acceptance before describing the web demos as playable.
    (Headless Ulam Flower and Vibesteroids startup/input probes passed
    2026-07-22 03:00 EDT; iPhone visual acceptance remains.)
  - Curiosity poke: native compilation is not sufficient portability proof;
    can each archive be exercised on its target without accidentally retaining
    Nix-store or build-host paths?

- [ ] Generalize the keyed native-control proof into the Aedicule View
  Protocol (AVP), a platform-neutral semantic application UI.
  - [x] Name the protocol and specify the retained view-document model,
    optional canvas composition, event-driven lifecycle, semantic nodes,
    layout, dynamic text/events, accessibility, adapter parity, explicit host
    services, budgets, and staged delivery plan in `VIEW_PROTOCOL.md`.
    (2026-07-21 23:55 EDT)
  - [x] Audit the existing keyed-view kernel and adapters for general-app
    readiness; record correctness, test, boundary, and scaling findings in
    `CODE_REVIEW.md`. (2026-07-22 00:00 EDT)
  - [ ] Extract an adapter-neutral typed view-document module with one
    aggregate node budget while preserving the current v0 native-controls
    profile.
  - [ ] Permit view-only output transactions and event-driven `0/1` guests.
  - [ ] Deliver the general forms slice through desktop, browser, and headless
    adapters before claiming general-application support.
  - [ ] Use `PRINTABLE_BINARY_EDIT_PROPOSAL.md` as the second unrelated AVP
    acceptance client: full-window editable text, user-mediated file handles,
    bounded reads, atomic raw-byte saves, conflict detection, and browser file
    capability parity, while keeping printable-binary semantics guest-owned.
  - [ ] Add explicit asynchronous capability profiles one service at a time.
  - [ ] Specify a bounded headless CLI profile (UTF-8 streams, argv/env
    allowlist, exit status), then an additive semantic TUI profile.
  - [ ] Specify pre-instantiation Wasm-GC policy in application metadata;
    linear-memory guests and short-lived CLI apps default disabled, and no
    guest may toggle the engine collector after instantiation.
  - Curiosity poke: can a separately owned editor guest falsify the protocol
    before packed document encoding or multiple-window support adds complexity?

- [ ] Make Geist Mono Regular a portable numerical-data face.
  - [x] Bundle and register the exact OFL-1.1 font on native and web hosts, then
    expose a versioned integer WAT selector while preserving `AE_text` as the
    default platform face.
    (2026-07-22 17:12 EDT; ABI v0.2 adds float and zero-float Q16.16 imports,
    exact minor-version admission, SVG parity, and release license payloads.)
    Curiosity poke: reject unknown faces atomically instead of silently
    substituting a proportional font that can corrupt numerical alignment.
  - [ ] Specify package-supplied font handles only after `.aed` assets reach the
    browser adapter with the same validation, size limits, and family identity
    rules as native delivery.
    Curiosity poke: internal family-name collisions must not let one guest font
    unpredictably replace another across hot reloads.

- [ ] Add a zero-float WAT profile with exact integer host controls,
  deterministic trig, and Q16.16 vector paths for Ulam/Uzumaki-class clients.
  - [x] Accept paired `AE_init_i32` / `AE_event_i32` lifecycle exports and
    packed-RGBA frame begin without any guest float types or opcodes.
    (2026-07-21 18:58 EDT)
  - [x] Declare bounded exact integer slider lattices and deliver ordered
    change/release values through `AE_control_event`. (2026-07-21 19:03 EDT)
  - [x] Accept Q16.16 move/line/path-end geometry with zero paint disabling
    fill or stroke. (2026-07-21 19:08 EDT)
  - [x] Supply bit-reproducible binary-turn Q1.30 sine/cosine through an
    integer CORDIC implementation. (2026-07-21 19:13 EDT)
  - [x] Independently exercise the real headless renderer from
    `ulam-flower-wat` at controls 0/1050/2400/3600, varied viewports, known-bad
    oracle mutations, and a tick-invariance check. (2026-07-21 19:52 EDT)
  - [x] Replace the host-owned slider layout/value prototype with atomic
    guest-authored `AE_control_panel_q16` and `AE_slider_place_q16` frame
    commands; retain GPUI entities only for native interaction mechanics.
    (2026-07-21 20:34 EDT)
  - [x] Hard-cut the provisional per-frame native controls to a LiveView-style
    `AE_ui_begin(revision)` / declarations / `AE_ui_end()` snapshot: keyed,
    guest-authoritative, host-reconciled, and rollback-safe.
    (2026-07-21 21:11 EDT)
  - [x] Render accepted UI snapshots in the native GPUI host, run all Aedicule
    gates, and send the tested exact contract to `ulam-flower-wat`.
    (2026-07-21 21:16 EDT)
  - [x] Receive the revised real-guest green proof for the final keyed-snapshot
    ABI. (2026-07-21 21:20 EDT)
  - [x] Add guest-authored keyed native buttons that reference declared action
    IDs, use exact Q16.16 bounds, expose a selected state, and deliver the
    existing ordered kind-7 event without embedding playback semantics.
    (2026-07-21 22:31 EDT)
  - [x] Paint asset-independent high-contrast title-bar glyphs over GPUI
    Component's still-functional native control hit regions, preserving host
    pointer occlusion. (2026-07-21 22:31 EDT)
  - [x] Receive Peter's visual acceptance for sliders, playback buttons, and
    the title-bar glyph fallback in the final keyed-snapshot ABI.
    (2026-07-21 22:47 EDT)
  - Curiosity poke: can future integer composition imports share one declarative
    fixed-point type description instead of proliferating suffix-specific docs?

- [ ] Make state-dependent frames reproducible through `aedicule-render` by
  accepting deterministic ordered event injection before the final render.
  - [x] Clarify in the generated ABI that draw IDs are unique across primitive
    kinds for one frame and reusable after the next frame begins.
    (2026-07-21 19:08 EDT)
  - [ ] Test cross-primitive duplicate rejection and next-frame ID reuse.
  - [ ] Add repeatable `--event KIND,CODE,A,B` arguments in command-line order
    after initialization and before ticks/render. Keep kind/code as stable ABI
    integers, parse coordinate/delta fields as logical-pixel decimals, and let
    the host's selected-profile adapter perform legacy-float or Q16.16 conversion.
  - [ ] Prove a menu event reaches a state-dependent frame through the real
    renderer and that rejected events produce stable stderr diagnostics.
  - Curiosity poke: reserve symbolic event aliases for a later additive CLI
    layer rather than making the first machine-facing contract ambiguous.

- [x] Make the launcher CLI regression test self-contained instead of relying
  on a developer's private `$HOME/dotfiles` checkout. (2026-07-20 15:58 EDT)

- [ ] Add web as a first-class Aedicule delivery target: a generic static-site
  bundle that runs the same `aedicule.v0` WAT application through upstream
  GPUI's browser platform, without a JavaScript game fork.
  - Curiosity poke: can the browser guest adapter preserve the native ABI's
    bounded lifecycle and deterministic command-frame semantics despite losing
    Wasmtime's native fuel and interruption facilities?
  - [x] Prove GPUI-web compilation and a browser guest/frame vertical slice.
    (2026-07-22 03:00 EDT: both release-bundled demos reached a 780×437 WebGPU
    canvas without browser exceptions.)
  - [x] Prove the portable browser runtime with synthetic pointer and keyboard
    edges plus exact scheduled WAT updates, without GUI sleeps. (2026-07-19
    18:39 EDT)
  - [x] Package a self-contained site plus a local headers-correct dev server.
    (2026-07-21 21:13 EDT)
  - [x] Verify the optimized browser bundle and its 11 MiB raw-Wasm budget in
    the pure Nix delivery test: 10,118,213-byte raw Wasm. (2026-07-21 21:13 EDT)
  - [x] Refresh the Nix Cargo vendor hash after the browser clock dependency,
    then build, bind, optimize, and header-probe a 10,096,747-byte local Wasm
    bundle containing Vibesteroids as external WAT. (2026-07-20 14:27 EDT)
  - [ ] Receive Peter's visual browser playtest result for that bundle.
  - [x] Replace blank browser startup failures with an accessible loading/error
    surface and capability diagnostics. (2026-07-20 19:32 EDT)
    - Curiosity poke: WebGPU exposure alone does not prove that the browser can
      grant a usable adapter, especially on mobile Safari.
  - [x] Emit a structured browser-console timeline for successful and failed
    startup stages without logging guest source or other application data.
    (2026-07-20 20:14 EDT)
  - [x] Retain GPUI's application handle for the browser document lifetime,
    then make the headless Chrome/CDP startup probe pass with a nontrivial
    canvas and no dropped wasm-bindgen callbacks.
    (2026-07-22 03:00 EDT)
    - Curiosity poke: can we carry upstream GPUI commit `74798c68` as a
      one-commit fork from our existing pin instead of importing 452 unrelated
      Zed commits?
  - [x] Make the headless browser probe synthesize movement, all three pointer
    button pairs, two-axis wheel motion, and a key pair; compact the default
    response and gate the wordy per-event stream behind
    `AEDICULE_BROWSER_TRACE_EVENTS=1`. (2026-07-20 20:30 EDT)
    - Curiosity poke: browser keyboard delivery currently reaches the DOM but
      still needs a focusable GPUI web element and shared key mapping before it
      can claim WAT delivery parity with native.
  - [ ] Preserve identity-bearing multi-touch from browser Pointer Events to
    WAT clients: event kinds 11/12/13/14 are start/move/end/cancel,
    `code = pointerId`, and `a,b = logical x,y`.
    - [x] Prove at the live Chrome/DOM layer that two contacts move
      independently, end separately, and a third contact cancels while three
      opaque IDs remain distinct. (2026-07-20 21:18 EDT)
    - [ ] Prove those same phases and identities reach `AE_event` through core
      WAT tests and the rebuilt browser runtime.
    - [ ] Prevent GPUI Web's current mouse-compatibility conversion from
      duplicating each touch as primary-button input.
    - Curiosity poke: Chrome remapped the probe's requested touch IDs 41/42/43
      to pointer IDs 2/3/4; clients must treat IDs as opaque, page-local
      correlation tokens and never persist or interpret their numeric values.
  - [x] Add an independently runnable downstream Vibesteroids web smoke proof
    sourced from the exact `.aed`, including its declared FLAC asset and a real
    muted WebAudio request. (2026-07-22 22:43 EDT)

- [ ] Standardize cross-platform pointer buttons and two-axis wheel motion for
  WAT clients without exposing platform-specific mouse representations.
  - [x] Map primary/left to ID 1 and secondary/right to ID 2, deliver down/up
    through `AE_event`, and regenerate the checked ABI reference. Portable
    `AE_event` and documentation tests passed. (2026-07-20 11:58 EDT)
  - [x] Map middle/wheel-click to button ID 3 and add kind 10 scroll events
    preserving horizontal/vertical deltas plus their line/logical-pixel unit;
    omit zero-motion phases. Both runtime suites and the native optimized GUI
    build passed. (2026-07-20 13:01 EDT)
  - [x] Send release edges inside and outside the canvas for every mapped
    button, preventing held input from sticking after a drag leaves the app.
    (2026-07-20 12:00 EDT)
  - [x] Verify the WebAssembly GPUI adapter in its `nix develop .#web` host
    toolchain build. (2026-07-20 13:06 EDT)
  - [x] Run the separately deferred pure Nix GUI/browser derivation gate after
    the local Nix-store work. (2026-07-21 21:16 EDT)
  - Curiosity poke: should a future pointer-capture contract distinguish an
    OS-cancelled gesture from an ordinary release without adding guest state?

- [x] Restore the desktop host title bar's input priority over the full-window
  guest pointer plane: Reload, New Game, Help, Quit, and platform window
  controls must not enqueue guest pointer edges, while clicks below the host
  title bar still do.
  - [x] Reproduce the overlap with a headless GPUI hit-routing regression.
  - [x] Occlude the guest hitbox behind the host title-bar layer without
    replacing `TitleBar`'s existing platform drag/control behavior.
  - [x] Run the focused regression, full suite, and optimized desktop build
    after Peter resumes builds following the Nix-store migration.
    (2026-07-21 21:16 EDT; native mouse savepoint `a10241d`)
  - Curiosity poke: should the noninteractive bottom status strip intentionally
    pass pointer input through to the guest, or claim its own native hitbox?

- [x] Establish a bounded Wasmtime ABI and neutral WAT fixtures.
  (2026-07-16 EDT)
  - Curiosity poke: which capability first fails to generalize to a second,
    non-game application?
- [x] Integrate full-window GPUI rendering, native input/menus, generated
  audio, deterministic SVG, and transactional live reload. (2026-07-16 EDT)
  - Curiosity poke: can candidate compilation leave the UI thread without
    introducing an ordering race at swap time?
- [ ] Mirror initial and watched WAT rejections to stderr with a stable
  diagnostic contract while preserving the active guest transactionally.
  - [x] Add the pure `AEDICULE_WAT_REJECTED` formatter and route native
    initial/reload rejections through it; document the renderer as canonical
    validation and deny launcher warnings in the Nix test target.
    (2026-07-20 10:06 EDT)
  - [ ] Run the native pure-Nix verification after the deferred Nix-store work.
  - Curiosity poke: can the pure formatter prove source, full error, and
    survivor status without a GUI timing test?
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
  - [x] Inspect the live reference sources: `blar` and `mini_blar` share the
    flat `MBAR\\x02` profile, while the published BLAR archive spec has drifted.
    Record the strict uncompressed Aedicule proposal in
    `ASSET_PACKAGE_PROPOSAL.md`. (2026-07-19 18:29 EDT)
  - Curiosity poke: can indexed uncompressed entries remain zero-copy while
    compressed entries enforce strict decoded-size and checksum limits?
- [x] Add guest-controlled digitized-audio playback from packaged assets, with
  FLAC as the delivered-media baseline and bounded raw PCM/WAV fallback.
  Ogg and MP3 are deferred pending `blar` design work.
  - [x] Decode bounded RIFF/WAVE integer PCM into device-independent fixed
    samples; malformed, unsupported, contradictory, and over-budget inputs
    have distinct tests. (2026-07-19 18:38 EDT)
  - [x] Decode admitted FLAC assets, preserve transactional guest playback
    requests, and implement native plus browser device adapters without making
    device timing guest-observable. (2026-07-22 22:43 EDT)
  - Curiosity poke: can long tracks stream under a bounded decode-memory budget
    without making audio-device timing observable by the deterministic guest?
- [ ] Load bounded texture assets from the same package into the existing
  content-hashed image-resource cache.
  - Curiosity poke: which decoded-pixel, dimension, and archive-entry limits
    prevent a small compressed asset from becoming an allocation bomb?
- [x] Add a second, unrelated WAT application as the strongest falsification
  test of the generic ABI: [`ulam-flower-wat`](../ulam-flower-wat/) drives
  exact ticks plus pointer/viewport events through only `AE_*` imports.
  Its headless integration test passed. (2026-07-19 18:29 EDT)
- [ ] Build a guest-owned magenta/green anaglyph vector demo for Peter's
  TriOviz glasses, initially as a fullscreen 3D viewer and potentially as a
  Tempest/Vectrex-inspired tube shooter. Keep stereo projection, eye
  separation, palette calibration, and game semantics in WAT; Aedicule should
  supply only generic layered-vector rendering, depth ordering, fullscreen,
  and input capabilities. (Idea captured 2026-07-22 22:01 EDT.)
  - Curiosity poke: photograph or otherwise identify the glasses' actual
    transmission colors before choosing matrices; “magenta/green” anaglyph
    systems are not interchangeable with the more common red/cyan profile.
- [x] Hard-cut the provisional executable, Cargo crate, environment, source,
  Nix, documentation, and test names to canonical `aedicule` and
  `aedicule-render`, with a repository-wide classifier preventing legacy names
  or aliases from returning. (2026-07-22 18:24 EDT: full test suite, optimized
  native build, and all six checksummed delivery archives passed.)
