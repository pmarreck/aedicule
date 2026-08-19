# Plan

- [x] Promote the exact public Vibesteroids release
  `d810415e800eb8ae8c077d3e4b18f7ce7af7a506` on host `b56abf6` or a tested
  descendant. Start RED by advancing the provenance oracle to WAT SHA-256
  `854e2fc1a6c084a2f45a409946929468fdc3c642bde12940851f0e8bd8719292`
  and `.aed` SHA-256
  `0a67be8c46c48a5a22f954f6b2bcad5e05319c19e4b71fc70379a6de566406c0`;
  independently prove extracted package `code.wat` equals Git byte-for-byte.
  Run package WAST, full suite/build, and composed Start-then-multitouch gates
  against the immutable delivery and live port 8911. Atomically promote 8911,
  retire stale port 8910 only after the new endpoint is healthy, push, watch
  exact Mechatron/GitHub CI, and verify public Pages routes plus remote hashes.
  Reply to both Vibesteroids notes with the exact host/guest revisions, hashes,
  gates, CI, and listener evidence. Curiosity poke: reject an outer package
  whose embedded WAT is stale even when the package hash matches a supplied
  pin. The snapshot oracle first failed on the stale manifest, WAT, and package;
  the new archive-reader containment assertion also failed under a deliberate
  stale-WAT mutation before passing on exact `d810415`. Packaged WAST, full
  `./test`, optimized `./build`, immutable delivery, and live-8911 composed
  browser gates pass. Port 8911 now serves the exact hashes above; port 8910's
  tmux session, orphaned Caddy process, and Tailscale proxy are retired, with
  socket and HTTPS probes proving only 8911 remains. Host commit `a20a448`
  matches `origin/yolo`; exact Mechatron Prime and GitHub CI pass, including
  the complete suite, all six delivery targets, web startup proof, and Pages
  deployment. Cache-busted public-origin probes returned the exact committed
  WAT, `.aed`, and manifest hashes on the first attempt, and all three demo
  routes return 200. (Completed 2026-08-19 17:15 EDT.)
- [x] Reproduce the port-8911 Vibesteroids touch-controls regression from the
  exact staged `1e04c70` package with a real-guest behavioral browser test.
  The existing gate proves ordered touch delivery but not visible ship
  response, so add an oracle over the guest's reaction to left/right stroke,
  center thrust, and simultaneous fire before changing implementation. Compare
  the previously accepted and current guest pins to assign ownership, then
  atomically restage and obtain Peter's iPhone acceptance before public Pages
  promotion. Peter reproduced in a different browser, ruling out stale cache;
  the guest diff changes only gift rendering and Help layout, so compare the
  last iPhone-accepted host runtime against the current bridge first. A new
  combined real-Chromium gate now touch-activates Start action 8, proves that
  its UI/occlusion retires, delivers six edge-contact phases with zero pointer
  leakage, and observes a guest firing-audio request; it passes against both
  live staging and the rebuilt delivery. The physical failure is therefore
  iOS/WebKit-specific until phone evidence says otherwise. A `?diag` staging
  build now exposes the negotiated limit plus raw, occluded, queued, disabled,
  enqueued, last-phase, pointer-capture-failure, and guest-delivery counters
  for that classification. A RED bridge test proved that a WebKit exception
  from advisory `setPointerCapture` previously aborted the whole contact;
  pointer capture is now fail-soft because window capture listeners retain
  delivery. Full tests, optimized build, immutable delivery, and the composed
  live-staging browser gate are green. Port 8911 serves this exact build and
  Peter confirmed on a physical iPhone that inputs work again. Because the
  only staged runtime change was making advisory WebKit pointer capture
  fail-soft, a thrown `setPointerCapture` is the strongest causal explanation;
  the exception counter was not observed, so retain that distinction between
  strong evidence and direct proof.
  Curiosity poke: distinguish a retained native-UI panel occluding the whole
  canvas from a bridge never enabled, a Rust drain stall, viewport coordinate
  drift, and guest pause state.
  (Reported 2026-08-14 16:33 EDT; cache ruled out 16:38 EDT; physical iPhone
  acceptance completed 2026-08-15 14:56 EDT.)
- [ ] Narrow the Nix source closures for `webRuntime` and `delivery-web` so a
  browser-test-only or HTML-only edit does not rebuild unrelated Rust/Wasm and
  native frontplane artifacts. Preserve reproducibility by classifying the
  exact runtime inputs rather than excluding filenames ad hoc, and add
  mutation controls proving a runtime source change invalidates the derivation
  while test-only and native-only changes do not. Curiosity poke: the delivery
  still embeds packages produced by the native CLI, so separate the package
  writer dependency from the browser runtime before claiming full decoupling.
  (Observed during touch-diagnostic rebuild, 2026-08-14 16:55 EDT.)
- [x] Eliminate ambiguous Tailscale playtest endpoints. Port 8910 was a stale
  long-lived `aedicule-serve` process with guest `126c173`, while port 8911 is
  current staging. Peter approved retirement. After exact `d810415` passed the
  immutable and live-8911 composed gates, the stale tmux session was retired;
  its Caddy child required an explicit graceful SIGTERM, and the 8910 Tailscale
  Serve handler was removed. Socket status and a failed HTTPS connection prove
  8910 is absent; 8911 routes and exact hashes remain healthy. Curiosity poke:
  a URL must expose both host and guest revisions because either half can
  change observed behavior. (Completed 2026-08-19 16:51 EDT.)
- [x] Audit every Aedicule change made after the guest-owned gift-circle crash
  report. Commit `a9da8e0` changes only the rejected error class for unsupported
  flags from the false `InvalidNumber("circle")/-5` to the precise fail-closed
  `InvalidFrame("unsupported circle flags")/-8`, adds a regression using the
  exact bad packed color, and corrects the single-source ABI description plus
  its generated copies. Valid geometry and guest containment are unchanged.
  Commit `6cb4693` promotes the independently proven corrected guest/package
  and immutable provenance oracle. The complete runtime touch diff from the
  pre-report `a7a9ca1` through `6cb4693` is empty. Both commits are worth
  retaining and neither can cause the physical touch regression. The guest now
  further makes raw flag calls unreachable behind semantic fill/outline
  wrappers; adding host imports is deferred unless another guest reproduces
  the WAT arity hazard. (Completed 2026-08-14 16:46 EDT.)
- [ ] TDD the physical iPhone shake path from browser user activation through
  `DeviceMotionEvent.requestPermission`, bounded sample capture,
  `ShakeDetector`, and guest kind-16/code-1 delivery. Add phone-visible
  diagnostics that distinguish absent/late interest, unavailable or insecure
  API, permission granted/denied/exception, zero samples, below-threshold
  samples, and emitted gestures; add a browser integration test covering the
  permission callback and synthetic motion sequence. Guest action eligibility
  remains Vibesteroids-owned. Curiosity poke: iOS may require permission from
  the completed Start tap rather than its initial pointer edge, and gravity
  inclusion changes the meaningful threshold. (Queued from Vibesteroids
  2026-08-14 16:10 EDT; follows the active gift-bow crash promotion.)
- [x] Reproduce and repair the delayed iPhone Vibesteroids freeze. Peter's
  repeatable-spawn timing and bow suspicion redirected diagnosis from touch
  handling to the gift render. Gift circle ID 915 passed packed cyan color
  `0x5ee7ffff` as `AE_circle` flags; the real ABI accepts only outline `0` or
  fill `1`, so the first active gift frame stopped the guest. Vibesteroids
  forced that frame RED, fixed the argument, tightened its fake host, and
  passed its full WAST/actual-Aedicule suite and Mechatron CI at clean commit
  `1e04c701dcb2709c4441d987fc0fbecaf11333dd`. Aedicule independently rejected
  one stale package, then proved exact inner-source provenance for WAT SHA-256
  `5043234c1c97b4fe27c926e13453f2d24cba5c65475d0145f68d28a1633fb906`
  and AED SHA-256
  `37b14181f2731f8728c862c719ebc152044c2c44001d705f1b675b7d39b9e77b`.
  Its package WAST and real Chromium/WebGPU startup/input gate pass. The exact
  artifacts are live through Tailscale staging. Host commit `a9da8e0` also
  distinguishes unsupported circle flags from non-finite geometry so a future
  violation names the contract it broke. Touch was incidental; the existing
  interrupted/multi-contact controls remain green. Peter's live gift replay
  and public Pages promotion remain release acceptance, not bug ownership.
  (Completed 2026-08-14 16:28 EDT.)
- [x] Repair the fresh-runner Cargo vendor fixed-output hash exposed by GitHub
  Actions after the Rust 1.97 migration. Exact push `c075d1e` independently
  reported `sha256-6CdssA6EJpAFe1HMceLDJCwT9a82+O9x0nZ3g18E2Hg=` in all six
  release jobs while the local warm store retained the former pin. The new pin
  passed cold `release-macos-aarch64`, cold `release-linux-x86_64`, `./test`,
  and `./build`; its focused commit triggers fresh GitHub and Mechatron Prime
  verification. (Completed 2026-08-14 14:50 EDT.)
- [ ] Remove the browser runtime's unnecessary shared-Wasm-memory requirement
  and obtain iPhone acceptance on the Tailscale staging delivery.
  - [x] Bound the imported shared memory from 1 GiB to 256 MiB and prove the
    exact generated maximum with the delivery gate. Native/full tests and the
    real Chromium/WebGPU gates passed; committed as `59e807c`.
    (2026-08-05 19:10 EDT)
  - [x] Peter tested that exact staging build on iOS Safari. The first load
    paused at `isolation-ready` after 290 ms; a reload reached
    `wasm-initializing` after 798 ms and eventually loaded. This refutes a
    permanent deadlock but still leaves an unacceptable apparent hang.
    WebKit bugs 222097, 269777, and 281657 make shared-Wasm reload retention
    and memory pressure the leading mechanism. (2026-08-05 19:28 EDT)
  - [x] Prove GPUI compiles and runs without explicitly enabled atomics or
    shared memory, then write the red delivery/startup tests before removing
    the COI service-worker reload, SharedArrayBuffer/Atomics preflight, shared
    linker/TLS flags, and thread-enabled wasm optimization. The emitted module
    validates, owns ordinary `(memory 81)`, wasm-bindgen accepts it, and
    `wasm-opt -Oz` produces 15,992,441 bytes. Fresh-profile Chromium/WebGPU
    probes reached `settled` for fallback, Ulam, Vibesteroids, and Spring in
    1.75–1.92 seconds with `crossOriginIsolated: false`; Wasm initialization
    took 169–220 ms. (2026-08-05 19:37 EDT)
  - [x] Add a pure set-classifier and non-blocking migration adapter that
    unregisters only old `coi-serviceworker.js` registrations across active,
    waiting, and installing lifecycle slots. It never delays gallery readiness
    or application startup and preserves unrelated origin workers. This is
    required because a prior Safari registration may outlive removal of the
    worker script. (2026-08-05 19:46 EDT)
  - [x] Bring native `aedicule --web` onto the same ordinary-memory delivery:
    stop requiring the deleted COI worker and COOP/COEP response headers, and
    serve the non-blocking retirement module. The HTTP integration test and a
    real Chromium/WebGPU launch both reproduce the old failure and pass the
    corrected path. (2026-08-05 20:28 EDT)
  - [x] Deploy the non-shared exact delivery to Tailscale staging. All three
    direct demos reached `settled` through fresh Chromium/WebGPU profiles in
    1.85–1.95 seconds. Peter's first iPhone attempt still showed the retired
    `isolation-ready` stage, proving Safari retained the old launcher; the next
    load reached the current `wasm-initializing` stage and eventually ran.
    (2026-08-05 20:40 EDT)
  - [x] Fix the Tailscale server's nested runtime delivery before asking for a
    second iPhone acceptance pass. Root cause found: the immutable matcher
    recognized only a site-root runtime, so every demo served its 15.98 MiB
    content-addressed Wasm uncompressed with `Cache-Control: no-store` on every
    reload. The root/nested set now receives one-year immutable caching and is
    excluded from mutable headers; Caddy serves gzip/zstd. A real nested GET
    proved a 6.22 MiB gzip transfer, immutable headers, and continued no-store
    treatment for `bootstrap.js`; the complete `./test` suite passed.
    (2026-08-05 21:09 EDT)
  - [x] Rebuild and restart exact Tailscale staging. The nested runtime now
    serves compressed/immutable over Tailscale; a fresh real Chromium/WebGPU
    run reached `wasm-initialized` at 1.02 seconds and `settled` at 2.19 seconds
    with no controlling service worker. (2026-08-05 21:12 EDT)
  - [x] Replace the session-owned staging process with the transient user
    service `aedicule-staging.service`, configured to restart Caddy after
    failure. Verified active continuously since 2026-08-07 16:54 EDT and the
    Tailscale HTTPS endpoint returned 200 on 2026-08-10 16:39 EDT.
  - [ ] Obtain Peter's second iPhone acceptance pass on the cached/compressed
    delivery, recording the last visible stage and elapsed time if it pauses.
  - Curiosity poke: preserve secure-context and WebGPU checks, multi-tab
    startup serialization, diagnostics, and content-addressed runtime caching
    without retaining any accidental dependency on cross-origin isolation.
- [x] Refresh the staged and shipped Vibesteroids demo from its newest
  immutable sibling release pin. Peter reports the currently pinned package is
  visibly old. Verify the sibling commit/package hash and its WAST acceptance,
  then replace every gallery, delivery, manifest, and release-bundle copy.
  Clean sibling commit `f15a5fe` (three tranches beyond the old `df61ffe` pin)
  passed its complete suite and produced a deterministic 125,044-byte `.aed`;
  source and package hashes were pinned under a red-first provenance test. The
  package's own `tests/main.wast` and the real Chromium/WebGPU startup/input
  gate pass through current Aedicule. This publishes the seeded star field,
  Start Game gate, and viewport-edge projectile lifetime while preserving the
  sibling's newer uncommitted work. (2026-08-05 21:18 EDT.)
  - [x] Supersede this clean interim pin when the Vibesteroids agent finishes
    its already-requested native `AE_action` Start/Resume conversion and sends
    a new public commit/package pair. Commit
    `e8efd62d0a15eafae7c43d0ed745e6c3b4781cd8` includes the native actions,
    raw multi-touch, phone-aware help, shake-triggered Death Blossom, shorter
    gift intervals, parcel bow, and predictive UFO/Voyager spawn safety. The
    guest's complete suite, optimized package, actual Aedicule touch timeline,
    and exact-commit Mechatron Prime CI are green. Peter's visual acceptance of
    the bow and Start/Resume buttons remains open. (Completed 2026-08-14 14:32
    EDT.)
    - [x] Promote Vibesteroids' newest immutable source/package pair through
      Aedicule's gallery and Tailscale staging page, verify the packaged tests
      and browser startup first, then send Peter the exact iPhone test URL.
      Pinned the 214,133-byte WAT at SHA-256 `9af66843...ac4` and the
      139,054-byte `.aed` at SHA-256 `c7faceb9...bf23`; the provenance test was
      observed red against the former pair, then green. The package's
      `tests/main.wast` passed through current Aedicule. Fresh Chromium/WebGPU
      reached every startup stage through `settled` in 2.84 seconds and
      observed all eight synthetic raw-touch deliveries with no pointer leak.
      The Tailscale staging root was replaced atomically with rollback retained;
      both served hashes match and
      `https://thelio-nixos.tail66c90.ts.net:8911/vibesteroids/` returns 200.
      (Completed 2026-08-14 14:32 EDT.)
      - [x] Provisional phone acceptance is live from the sibling's current
        dirty candidate while its immutable pin is pending. Packaged WAST is
        green; the live `.aed` SHA-256 is
        `233fb1ebb537585fd1161a47c1faa23cd851d6cea05e53cfb92bc81666a353ae`;
        nested WAT SHA-256 is
        `33c305a80a84cf961cf4268c5640a870e15dbdadba46ffbb9d0162b6e3b31af9`;
        the ABI-v0.9 browser runtime reaches `settled`; and live Tailscale GETs
        return 200 at
        `https://thelio-nixos.tail66c90.ts.net:8911/vibesteroids/`.
        (2026-08-13 17:47 EDT.)
- [x] Fix the Aedicule-owned source-checkout launcher regression: `./run --web`
  currently builds successfully and then exits because it cannot find a Web
  runtime unless `AEDICULE_WEB_RUNTIME` is supplied. The source runner must
  locate or produce the exact runtime from its own checkout without
  guest-specific setup. Reproduce with the Vibesteroids command from the
  2026-08-11 inbox note, add the failing launcher test first, and preserve
  packaged-install discovery behavior. The wrapper now materializes its exact
  `.#webRuntime` only when the final CLI mode is Web, exports the returned
  store path into the unchanged caller working directory, and preserves an
  explicit `AEDICULE_WEB_RUNTIME`. The set-classifier gate covers every later
  mode override, and the real foreign-source invocation crossed runtime
  discovery before failing at the intentionally missing WAT. Focused test
  green. (2026-08-13 17:37 EDT.)
- [x] Advance Spring Simulator to its layout-v3 and damping-fix release:
  source commit `6e89c4a0d9993540c0f0577b16565c73dccf5438`, 41,009-byte
  `spring_sim.aed`, SHA-256
  `ce2b757e108737ca45fbc2a809d9444ea32327591c462511634fcbd502291ae3`.
  Independently verify the immutable bytes, packaged WAST, headless viewport
  renders, and real Chromium semantic controls before updating every gallery,
  manifest, delivery, and release copy. The immutable artifact matched all
  three pins; its packaged WAST passed; 1024x768, 390x844, and 780x437
  headless renders were clean; and real Chromium/WebGPU at 780x437 settled in
  2.386 seconds with all 7 sliders and 5 buttons visible, delivering semantic
  events without pointer leakage. The manifest and package snapshot now drive
  every generated delivery/release copy. Tailscale staging serves the exact
  package and nested WAT hashes. (Queued 2026-08-10 16:40 EDT; completed
  2026-08-13 17:47 EDT.)
- [x] Add RandomZ-compatible deterministic randomization to the WAT/AED ABI
  after agreeing on the guest contract. Prefer `../random`'s new pure
  `randomr` Rust crate as Aedicule's implementation if it passes an explicit
  `wasm32-unknown-unknown` compile/runtime gate; it already passes the shared
  LuaJIT/Zig pairwise oracle, mutation, cross-target, and `wasm32-wasip1`
  controls. Retain those independent producers and frozen vectors as the
  conformance authority rather than duplicating the algorithm again. Specify
  algorithm/profile versioning, guest-owned serializable stream state, integer
  fixed-point nonlinear distributions, bounded batch sampling, hot-reload
  preservation, and explicitly exclude ambient entropy from this deterministic
  profile; any future entropy capability must be separately named and opted
  into. (Queued 2026-08-11
  08:55 EDT; revised for the completed Rust port at 15:24 EDT; architecture
  discussion first; implementation authorized 2026-08-13.)
  - [x] Prove the pure `randomr` crate compiles unchanged for Aedicule's exact
    `wasm32-unknown-unknown` target under its declared Rust 1.97 MSRV. Its 11
    focused unit tests and doctest pass with default features disabled. It also
    compiles under Aedicule's current Rust 1.96.1 when Cargo's version guard is
    bypassed, isolating the remaining integration decision to declared
    toolchain policy rather than source compatibility. (2026-08-11 15:27 EDT.)
  - [x] Advance Aedicule's native, cross, and development Rust toolchain to
    exact stable 1.97 and pin `randomr` at RandomZ commit
    `346889f1762d421b15186a2b14f105639afef6a2`. GPUI Web retains the exact
    nightly pinned by `flake.lock` because its `wasm_thread` dependency still
    requires `stdarch_wasm_atomic_wait`. Frozen raw/nonlinear vectors, the
    focused toolchain gate, `nix flake check --no-build`, the native Nix build,
    and the complete `./test` suite pass. (2026-08-13 16:57 EDT.)
  - [x] Freeze ABI v0.9's `AE_random_v1_*` contract: 48-byte guest-owned
    serializable streams; raw bytes, inclusive integer ranges, and every
    RandomZ fixed-point nonlinear distribution; bounded batches; canonical
    little-endian wire records; and transactional failure. Native Wasmtime,
    native portable-runtime, and the portable runtime inside real
    `wasm32-unknown-unknown` all match frozen RandomZ seed-42 output. Output and
    source-budget failures, invalid pointers, and state/output overlap leave
    stream state unchanged. A mutation that removed the nonlinear source
    budget made the independent failure control fail. Generated `WAT_ABI.md`
    and `GUIDE_FOR_LLMS.md`, `./test`, and `./build` are green.
    (2026-08-13 17:29 EDT.)
- [ ] Land "option C": one `.aed` implementation for every adapter.
  - [x] `browser_application_from_package(bytes)` in `src/web.rs` expands a
    package to (WAT, assets) through the same validated Rust reader native
    uses; unit-tested natively (compressed expansion, missing guest, non-UTF-8
    guest) and exercised inside a real wasm module by
    `tests/wasm_package.rs::a_browser_package_expands_to_guest_and_assets_in_wasm`.
    (2026-08-02 11:35 EDT)
  - [x] The JavaScript zip parser in `packaging/web/local-application.mjs` is
    DELETED. A visitor-selected `.aed` now crosses into IndexedDB and then into
    the wasm boundary as raw bytes (`__AEDICULE_AED`); JavaScript checks only
    the size bound and the 4-byte zip signature. Legacy stored records with
    `{wat, assets}` still load. (2026-08-02 11:35 EDT)
  - [x] `demos/vibesteroids.aed` regenerated compressed per Peter's explicit
    "Break the SHA" decision: 1,347,921 -> 117,755 bytes (11.4x), content
    proved byte-identical through depackage/repackage/depackage `diff -r`, and
    repacking twice is byte-identical. New sha256 `b5e3b290...` pinned in
    `demos/manifest.tsv` and `tests/cli/demo_snapshots`. (2026-08-02 11:20 EDT)
  - [x] Full serial `./test` + `./test_browser` + `./build` verification: all
    three green, including the new compressed-package browser gate proving a
    zstd `.aed` picked in the gallery runs to `settled` through the Rust
    reader in wasm. (2026-08-02 12:04 EDT)
- [x] ROOT CAUSE FOUND AND FIXED: the gallery gate's intermittent
  zero-stage timeout (~1 failure in 5) was a startup RACE in the gate's own
  readiness check, not the wasm bundle and not the zstd work. The check
  accepted `crossOriginIsolated && picker !== null`, but the picker exists in
  static HTML and isolation is true from document start on the post-reload
  load, while `launcher.mjs` is a module still fetching through the service
  worker — so the gate could inject its `change` event before any listener
  existed, and the event was silently lost. Proven by surfacing the gallery's
  own console stream in the gate: failing runs reach `isolation-ready` and
  never emit `local-application-selected`. Fix is a two-sided readiness
  contract: the launcher sets `picker.dataset.listening = "true"` after
  attaching listeners (plus a `listener-attached` console stage), and the gate
  refuses to inject until it sees it; `web_startup_surface` pins both sides.
  0 failures in 12 consecutive runs after the fix. The wasm-bundle-size
  hypothesis is REFUTED. (2026-08-02 11:55 EDT)
- [ ] Fix the two iOS Safari defects Peter reported 2026-07-28 16:38 EDT on the
  deployed `73891e6` Ulam page. The keyboard defect is CONFIRMED FIXED on
  hardware; these two are what became reachable once it was.
  - [ ] Defect 1 ROOT CAUSE FOUND 2026-07-31: AVP controls have zero touch
    slop. A press that releases even one pixel outside the declared rectangle is
    discarded silently. Measured against Peter's exact device geometry
    (430x775 viewport, button 10 at 70x36 at 24,559, reproduced bit-for-bit
    headlessly): touch drift 0-17px delivers the action, >=18px delivers
    nothing, and 577+18 is exactly the button's bottom edge. A 36 CSS px tall
    target therefore allows +/-18px of finger travel, which a fingertip rolling
    off on release exceeds routinely — matching both the ~30% miss rate and the
    ~30% shortfall in Safari's own `click`, which cancels under the same
    condition. The guest's own comment says "the guest owns this layout and the
    host owns safe activation", so slop belongs in the host.
    - [x] Slop policy decided by Peter: Option A, a fixed 16px release margin,
      not 44x44 hit rects. Shipped in 75697fe: `ArmedControl` with half-open
      `rescues()` in the browser adapter, armed only when
      `(pointer: coarse)` matches, so mouse drag-off-to-cancel is untouched.
      Gate: `web_touch_activation` (centre/near activate once, far never).
      (2026-08-01, EST estimate)
    - [ ] REFUTED EARLIER THEORIES, do not revisit without new evidence: a dead
      rAF loop (frames climb with stalled 0 on hardware); double gesture
      delivery AT THE POINTER/TOUCH LAYER ONLY (pd/pu/ts/te pair 1:1 — but
      those counters never watched mousedown/mouseup, so this refutation
      never covered the compatibility mouse echo that turned out to be the
      real second cause, see below); and a device-pixel-ratio viewport bug
      (hardware reports canvas 1290x2325 for css 430x775, exactly 3x, and the
      guest's own layout math reproduces 70px from a 430px viewport). The DPR
      theory came from a headless reproduction that was an emulator artifact:
      Chromium's device-metrics override reports devicePixelRatio 3 while
      devicePixelContentBox still returns real surface pixels.
  - [ ] Superseded framing of defect 1: pressing Play advances one frame per tap
    and then stops. Read-only evidence so far, not yet a failing test:
    `gpui_web`'s `create_raf_closure` re-schedules the next animation frame
    *after* invoking the frame callback, so a single exception anywhere in a
    frame permanently kills the animation loop, after which only input-driven
    redraws render — one frame per tap exactly. Establish whether an exception
    is actually thrown before changing anything, then make the loop survive one
    regardless, since a brick-the-app-forever failure mode is wrong even if this
    is not today's trigger.
  - [x] Defect 2 FIXED, hardware confirmation still pending. The classifier
    swept every element and found `html`, `head`, `meta`, `title`, `link`,
    `style`, `body`, and `script` selectable while `canvas` alone was not, so
    the selection was the document *around* the application. `web/index.html`
    now applies `-webkit-user-select`/`user-select`, `-webkit-touch-callout`,
    and `-webkit-tap-highlight-color` to `html, body, canvas`, with editable
    elements opting selection back in so text entry keeps it. The browser gate
    enforces the classifier unconditionally for every app; Chromium implements
    neither `-webkit-touch-callout` nor tap-highlight suppression, so those two
    are asserted at the source in `tests/cli/web_startup_surface` and were
    mutation-checked. (2026-07-28 17:35 EDT: `./test`, `./test_browser`, and
    `./build` all green.)
  - [x] Defect 1 SECOND ROOT CAUSE FOUND AND FIXED 2026-08-03, hardware
    confirmation pending: after the slop fix, hardware still showed 9/10 taps
    advancing one frame AND 9/10 failing to pause — the signature of a toggle
    delivered TWICE per tap (play+pause), not of missed taps (a missed tap
    while playing would look like success). iOS Safari ignores
    `preventDefault()` on pointerdown (the spec suppression Chromium honors)
    and synthesizes a compatibility mousedown/mouseup pair at the touch point
    after touchend; `gpui_web` listens on both paths, so every tap dispatched
    twice. Sliders survived because setting a position twice is idempotent;
    Peter's `ck` counter incrementing on hardware was the tell. Red test
    first: `dispatchGhostMousePair` in `web_browser_startup` replays Safari's
    exact stream (ghostActivated=true observed pre-fix). Fix in the fork, rev
    4afe33f254c0 on `aedicule-gpui-web-input-fixes`: touchstart/touchend
    `preventDefault()` (the suppression WebKit actually implements) plus a
    CLOCK-FREE structural guard: every real mouse edge is preceded by its own
    pointer twin (pointerdown, or pointermove for a chord's collapsed second
    button) while a compatibility echo is a bare MouseEvent with none, so a
    two-state device machine (last pointer activity Mouse vs TouchOrPen)
    swallows bare mouse edges after touch/pen deterministically. A first cut
    used a 1500ms/32px time+radius classifier (rev 7c69e44d37ad, superseded,
    kept in fork history): its radius could nondeterministically eat the
    gate's own mouse clicks near a recent touch point. Ghost edges also skip
    `apply_focus_host` so a synthesized gesture cannot summon the keyboard.
    Diag overlay now counts md/mu beside pd/pu/ts/te/ck so ghost pairs are
    visible on hardware. Pin bumped in Cargo.toml/Cargo.lock.
    (2026-08-03 16:45 EDT)
  - [x] Peter retested Play/Pause on iPhone with the ghost fix deployed: "it
    now works." Defect 1 is CLOSED on hardware. (2026-08-03 16:40 EDT)
  - [ ] Classify both as regression vs. newly-reachable pre-existing behavior
    before attributing either to the focus fix. Peter could not get past the
    keyboard on any prior iOS session, so "new" is not established.
  - [ ] Predicted third defect, from the same prior art: iOS raises the software
    keyboard only when `focus()` runs synchronously inside a user-gesture
    handler. Aedicule focuses the editable element from a render/rAF callback,
    so the just-shipped text control likely cannot raise the iOS keyboard at
    all. Test this before believing it either way.
- [ ] Make the public web delivery demonstrably compatible with Firefox,
  Chromium-family browsers, and Safari on macOS/iOS.
  - [ ] Reproduce and fix the Firefox Beta simultaneous-startup race reported
    2026-07-24: starting Vibesteroids and then immediately selecting Ulam in a
    new tab left Vibesteroids at the index HTML's initial “Loading Aedicule…”
    text and froze Ulam on a white page.
  - [ ] Preserve the distinct sequential Firefox observation: with the active
    Vibesteroids tab closed and Ulam reloaded, Ulam rendered successfully with
    its guest-authored controls visible. A fresh isolated Firefox Beta 152
    headless profile initially exposed canvases in both tabs, but Firefox later
    logged “Script terminated by timeout” in Ulam's shared Aedicule runtime,
    followed by `RuntimeError: unreachable executed`. A later instrumented
    replay completed 467 active Ulam frames and 7 throttled background
    Vibesteroids frames while both tabs remained command-responsive; this
    weakens the earlier timeout interpretation because WebDriver itself had
    awaited `requestAnimationFrame` in a throttled tab. Canvas appearance is
    still insufficient acceptance, and neither isolated result reproduces or
    disproves Peter's hardware/profile browser-process wedge. Do not claim a
    permanent one-WebGPU-tab limit, adapter-allocation failure, or Wasm
    watchdog without a hardware trace.
  - [x] Extend diagnostics earlier than the current module bootstrap:
    isolation registration/ready/reload, WebGPU adapter request/completion,
    elapsed time, exception name/cause/stack, browser identity, page
    visibility/focus, service-worker control state, and browser-frame
    started/completed heartbeats. Keep bounded global timelines, concise
    visible progress, and complete console detail. (2026-07-24 08:43 EDT:
    focused gates, complete 11.1-second suite, packaged Chromium acceptance,
    and optimized build passed.)
  - [x] Add an origin-scoped Web Locks critical section from isolation
    readiness through the first committed frame, with feature-detected fallback
    and no fixed delay; trace requested/waiting/acquired/released. Deterministic
    injected-lock tests, the complete suite, packaged Chromium acceptance, and
    optimized build passed. (2026-07-24 09:00 EDT)
    - The locked two-tab Firefox Beta replay initialized and advanced both
      guests, then still crashed the headless SWGL process and lost the
      WebDriver session. The lock mitigates overlapping adapter/Wasm startup
      but is not evidence that Peter's hardware/profile wedge is fixed.
  - [ ] Diagnose Firefox Web Audio separately: Vibesteroids keyboard delivery
    works, but Peter heard no sample audio. Trace user activation, autoplay
    policy, AudioContext creation/state/resume, PCM receipt, source start, and
    rejection without conflating silence with the startup race.
    - [x] Publish bounded Web Audio lifecycle diagnostics for each of those
      stages, with failure class/message and current user-activation/context
      state. (2026-07-24 08:43 EDT)
    - [x] Replace the request-counter-only browser gate with deterministic
      adapter tests proving nonzero PCM reaches the expected channels, the
      graph connects to the destination, resume precedes source start, and
      volume/pitch are applied. The packaged-browser acceptance now uses a
      synth-only WAT and requires trusted activation, a running context,
      nonzero peak/RMS, `source-started`, and `source-ended`; it also caught and
      fixed the native `--web` runtime allowlist omitting `audio.mjs`.
      (2026-07-24 10:51 EDT; focused Rust, Node, native-server, and live muted
      Chromium gates pass. Physical speaker/device-loopback output remains a
      separate acceptance boundary.)
  - [ ] Fix the distinct iOS software-keyboard-on-any-touch defect at the
    GPUI-web input-handler boundary: a guest with no active text/IME control
    must not focus an editable hidden DOM input on canvas or native-control
    pointer-down. Preserve slider/button/canvas touch, hardware-key down/up,
    and future explicit text-input focus. Add a deterministic causal DOM-focus
    test rather than treating mobile emulation as proof that a keyboard did
    not appear.
    - [x] Split input by physical modality without user-agent detection:
      real `mousedown` restores hidden-input focus for hardware keyboard
      delivery, while touch/pen remain on the Pointer Events path and never
      focus that editable input. The instrumented packaged-browser gate first
      caught the over-broad no-refocus implementation breaking W/A/D and synth
      audio, then passed with zero focus calls during the complete multi-touch
      stream while hardware keyboard delivery and audio remained green.
      Pointer cancellation now also releases the corresponding GPUI button
      state so a cancelled touch cannot contaminate the next control gesture.
      (2026-07-24 16:50 EDT; GPUI fork pin `cbea9c7f`.)
    - [x] ROOT CAUSE FOUND (2026-07-25 17:35 EDT). Peter's physical iOS Safari
      acceptance FAILED at ~17:19 EDT: tapping anything on the deployed
      <https://pmarreck.github.io/aedicule/ulam-flower/> still summoned the
      software keyboard, and it persisted through a force reload. Excluded by
      evidence: a stale pin (`Cargo.lock` pins the fixed `cbea9c7f`); a cached
      guest `.wat`/`.aed` (a guest holds no DOM authority and cannot call
      `focus()`, and the service worker's cache-first path covers only
      `aedicule_web_bg.<64-hex>.wasm`); and the delivering agent's recorded
      residual risk #1 — an isolated Playwright probe proved WebKit 26.5,
      Chromium 149, and Firefox 151 all DO suppress the compatibility
      `mousedown` after `preventDefault()` on a touch `pointerdown`.
      The actual cause is `crates/gpui_web/src/window.rs:132`, which focuses
      the hidden editable `<input>` at window creation and never releases it;
      `preventDefault()` on every press is deliberately designed to keep that
      focus. `document.activeElement` is therefore permanently `INPUT`, which
      is precisely what summons the iOS keyboard — the tap is only the gesture
      that lets it present.
    - [ ] Replace the vacuous oracle. The shipped browser gate asserted
      "hidden-input `focus()` calls after startup == 0", which passes while the
      input is already focused from startup, so it could never fail on this
      defect — the check and the code agreed with each other (MFIC: fails the
      Independent axis). The correct oracle is "is any editable element
      focused?", which fails identically in all three engines, so this never
      required an iPhone to catch.
    - [ ] Fix the focus ownership, as a classifier over sets rather than a
      predicate: (a) a touch/pointer on non-text content must leave NO editable
      element focused; (b) an active text-entry control MUST focus the editable
      input so the keyboard still appears (Peter, 2026-07-25). Hardware key
      delivery must survive (a) — the earlier "never refocus" attempt broke
      W/A/D and synthesized audio — so keys likely need a non-editable focus
      host such as the canvas with `tabindex="-1"`, reserving the hidden input
      for genuine text/IME entry. Write the failing test first.
      - [x] Fork fix implemented and pushed: `pmarreck/zed@3c54328c` on
        `aedicule-gpui-web-input-fixes`. Canvas gets `tabindex="-1"` and owns
        focus at startup; `set_input_handler`/`take_input_handler` drive DOM
        focus and blur; `keydown`/`keyup` move to the document so keys arrive
        in either focus state; the decision is the pure
        `focus_policy::focus_host`, exhaustively tested over its complete
        domain and proven non-vacuous by mutation (flipping the mapping reds
        all three tests). 5 native tests pass, wasm compiles clean, rustfmt
        applied. Aedicule repinned with `Cargo.lock` and the Nix vendor hash
        `sha256-MPZMrxoj2N7Q42HaNHLcQVHKy/AHgvu8+hEFAtYE0DA=`.
        (2026-07-25 18:40 EDT)
      - [x] Hardened `listen_document`: it silently registered NO listener when
        the canvas had no owner document, which would have dropped every key
        with no error — the same shape of quiet failure that let this defect
        ship. It now falls back to the canvas so a listener is always attached.
        `pmarreck/zed@42ce92a6`; Aedicule repinned, vendor hash
        `sha256-gftlwIFBxkWc0Z6D70EyEFePPW0U5i0xSoJ6WRFb4/0=`.
        Coverage note: the native suite reported 5 passing on code that did not
        compile, because `events.rs` is `#[cfg(target_family = "wasm")]` and is
        never built natively; `cargo check --target wasm32-unknown-unknown`
        caught the borrow error. Do not read a green native run as covering
        `events.rs`. Key delivery IS asserted, by `web_packaged_audio`'s
        `--require-key-delivery` against `demos/vibesteroids.aed`.
        (2026-07-25 19:02 EDT)
      - [x] Fix VERIFIED against the newly built delivery served over
        `./serve_web`: `editableFocusedAtStartup` is now **false** (was true on
        the live deployed page), with `chordDelivered: true`,
        `hiddenInputTouchFocusRequests: 0`, and the `settled` stage reached.
        The same oracle run against the live pre-fix page
        <https://pmarreck.github.io/aedicule/ulam-flower/> fails, so the red and
        green are on the same harness and differ only by the fix.
        (2026-07-25 18:55 EDT) Peter's physical iPhone re-test is still required:
        the oracle proves "no editable element holds focus", not "no keyboard
        appeared".
      - [ ] SEPARATE PRE-EXISTING DEFECT, do not attribute to the focus fix:
        `web_browser_startup --static-root` times out waiting for the settled
        startup stage and emits ZERO Aedicule stages, so the page never
        bootstraps at all — which no wasm change can cause. The same harness
        reaches `settled` against both the live Pages URL and the identical
        delivery served by `./serve_web`, so the delivery is fine and the
        harness's own static-root server is the suspect. This is what fails
        `run_gallery_local_application` in the aggregate `./test_browser`.
        Failing evidence preserved at
        `scratchpad/gate2-FAILING-timeout-evidence.log`.
      - [ ] `--require-key-delivery` reports `keyFrameChanged: false` for BOTH
        the live pre-fix page and the fixed build when driven ad hoc against
        `/vibesteroids/` over HTTP. Identical before and after, so it is not a
        regression; the assertion is designed for the `web_packaged_audio`
        path that drives `demos/vibesteroids.aed` through the native binary.
        Do not "fix" key delivery on the strength of an ad-hoc invocation.
    - [x] Add the AVP text-entry control (Peter authorized the full slice,
      2026-07-25), built on `gpui-component`'s text input rather than a bespoke
      widget. This is what proves half (b) of the classifier above against a
      real widget instead of an assertion. Follows the `AE_external_link`
      pattern: a configure-time declaration with budget, duplicate-id, and
      validation rejection, plus a `TextFieldPlacement { id, panel_id, bounds }`
      in the retained Q16.16 document. (2026-07-27 08:31 EDT)
      - [x] ABI DESIGN RESOLVED — no strings cross the boundary at all. Peter's
        own proposal: keys stay keys, and text arrives as integers. One complete
        value expands into an ordered run of `AE_text_event(id, index, scalar,
        phase)` calls, one per Unicode scalar, terminated by `index = -1`
        carrying the authoritative scalar count and the edit phase (1 change,
        2 commit). The host therefore never writes into guest memory and the
        guest never decodes UTF-8. Capacity is declared at configure time in
        SCALARS (not bytes, not UTF-16 units), capped at
        `MAX_TEXT_FIELD_SCALARS = 4096`; bounding `index` is exactly equivalent
        to bounding the length and refuses an over-long value before any of it
        is delivered. Lone surrogates and anything above `10FFFF` are refused.
        Declaring a field without exporting `AE_text_event` rolls configure back.
        (2026-07-27 08:31 EDT)
      - [x] Native, browser, and headless adapter parity via one shared pure
        expansion, `text_value_events`, so the three cannot drift into three
        different wire sequences. Native and browser render `gpui_component`'s
        `Input` bound to an `InputState` whose `validate` shows the capacity
        bound, while the host re-checks it independently — an adapter's good
        behavior may never be load-bearing for a capability bound. Headless
        gains `aedicule-render --text ID=VALUE`, splitting at the first `=` so a
        value may contain `=` or be empty. `WAT_ABI.md` and `GUIDE_FOR_LLMS.md`
        regenerated. (2026-07-27 08:31 EDT)
      - [ ] KNOWN AND DOCUMENTED v0.5 LIMITATION: the platform widget owns its
        edit buffer, because `TextFieldPlacement` carries geometry only. A guest
        can therefore read the value but cannot set, clear, or restore it, and a
        reload discards it. This is stated plainly in both generated documents
        rather than left for a guest author to discover. Lifting it needs a
        guest-authored value in the placement plus reconciliation by accepted
        revision, mirroring how sliders already reconcile.
      - [ ] Update `VIEW_PROTOCOL.md` for the delivered text control.
      - [x] Browser gate `web_text_entry` now asserts the classifier's second
        half mechanically in real Chromium: before focus no editable element
        holds focus, after clicking a placed AVP text field one does, and the
        typed scalars reach the guest in exact order (the fixture publishes
        green only for the run 'h','i' with terminating count 2). Registered in
        `./test_browser`. GATE_EXIT=0. (2026-07-27 09:30 EDT)
      - [x] The gate found FOUR real defects that unit tests could not, each
        fixed at its true layer rather than worked around:
        1. `gpui_web` applied the focus policy on every `set_input_handler` /
           `take_input_handler`, which GPUI calls per frame. The resulting
           focus/blur thrash re-entered GPUI. Fixed by making the policy
           idempotent against the document's real `activeElement`
           (`focus_transition`, exhaustive over its 2x3 domain, mutation-proven).
        2. `focus()`/`blur()` fire DOM events synchronously, and this window's
           own focus listeners take the `callbacks` borrow that GPUI's input
           dispatch already holds — every keystroke panicked with "RefCell
           already borrowed". Fixed by running the policy pass in a coalesced
           microtask, which is safe precisely because the pass is idempotent.
        3. `gpui_web` hardcoded `prefer_character_input: false`, so GPUI never
           considered whether a text control was accepting input and swallowed
           any letter that could begin a multi-key binding. Typing "abcdef"
           delivered "ce". Fixed by preferring character input; GPUI honors it
           only when an input handler reports it accepts text, so an
           application with no text control keeps every binding.
        4. THE ACTUAL ROOT CAUSE of the missing characters, and it was ours:
           Aedicule's own root `on_key_down` forwarded keys to the guest even
           while a text field held focus, so every letter that maps to a guest
           `Key` was stolen and `cx.stop_propagation()`'d. This is the keyboard
           counterpart of the pointer occlusion AVP control layers already had.
           Fixed in BOTH the native and browser adapters via
           `text_entry_has_focus`. (2026-07-27 09:30 EDT)
      - [x] Removed a fourth `gpui_web` change that buffered and replayed
        characters arriving while GPUI had no input handler installed. It was
        built on the hypothesis that the missing characters were lost to that
        window; defect 4 above disproved it. With the real cause fixed the path
        is never taken, so it was unexercised speculation rather than a tested
        fix, and it was reverted rather than left standing. Restore it only
        alongside a test that forces the handler-absent window.
        (2026-07-27 09:41 EDT)
    - [ ] Add an automated real-WebCore browser lane so this class of defect
      cannot reach Peter's phone again. Nix `playwright-driver.browsers`
      1.61.1 provides `webkit-2311` (and `firefox-1532`) on Linux x86_64.
      Assert on mechanical signals only — hidden-input `focus()` call counts
      and the exact event sequence — never on "a keyboard appeared", which
      does not exist on the WPE/GTK port. Simulated touch, pointer, and move
      dispatch is the point of the lane.
    - [ ] Serve the HTML entry document uncached, caching only the underlying
      content-hashed assets (Peter, 2026-07-25). GitHub Pages returns
      `cache-control: max-age=600` for every path, so the service worker must
      revalidate navigation/document requests and rewrite their
      `Cache-Control`, while `aedicule_web_bg.<64-hex>.wasm` keeps its
      cache-first immutable path. This also permanently removes the
      stale-HTML-to-stale-wasm confounder from the iOS diagnosis.
  - [x] Preserve independent browser mouse-button release edges under chords.
    Peter reproduced secondary-down (thrust), primary-down (fire), then
    secondary-up while primary remains held leaving thrust latched. Start with
    a failing adapter regression for the exact button-mask transition; classify
    the native desktop adapter separately rather than assuming parity.
    Native is confirmed unaffected. The bold GPUI Web fix now uses independent
    DOM mouse edges plus the authoritative `buttons` mask, and the real guest
    gate proves all three chord transitions reach `AE_event`. The native host
    now emits a release only for a press that began on the guest canvas, so
    GPUI's global mouse-up-out capture cannot leak an AVP control release into
    the guest. The Pages Ulam gate chooses an unoccluded input point and waits
    for semantic-counter quiescence without sleeps before proving button and
    slider occlusion. Native crate tests, wasm compilation, the exact Pages
    Ulam gate, `./test_browser`, `./test`, and `./build` passed.
    (2026-07-24 16:50 EDT; upstream branch `77ea846c`, immutable Aedicule
    backport `cbea9c7f`; upstream PR text remains intentionally Peter-owned
    under Zed's contribution policy.)
  - [ ] Add real acceptance lanes for Firefox, Chromium, macOS Safari, and iOS
    Safari where automation permits; do not use Chromium success as evidence
    for other browser engines.
  - Curiosity poke: simultaneous tabs share an origin, GPU process, service
    worker/cache state, and Web Locks namespace; serialize only initialization
    so normal multi-tab execution is not unnecessarily prohibited. Firefox can
    terminate long-running Wasm after a visible canvas exists, so acceptance
    must observe sustained responsiveness and page lifecycle—not startup alone.

- [ ] Make the public gallery previews respond to deliberate interaction.
  - [x] Add a prominent “About Aedicule” link and concise standalone page
    explaining the human-or-human/agent WAT authoring model, rationale,
    LiveView-like guest-owned update protocol, the exact currently available
    graphics/audio/input/gpui-component surface, and all six delivery targets.
    Distinguish current ABI from proposals and substantiate any “only” or
    “unique” claim; be loud about the exact combination we can prove rather
    than making an unresearched exclusivity claim. (Completed 2026-07-24 09:12
    EDT.)
    - [ ] Receive Peter's visual approval of the desktop/mobile rendering.
  - [x] Publish one build-generated Current + Proposed ABI reference page.
    Generate the current callable surface from the same typed Rust capability
    registry as `WAT_ABI.md`, and generate future cards only from an explicit
    non-callable proposal registry with status, evidence/spec links, and
    adapter targets. Add a conformance test that instantiates every current
    documented import at its exact generated signature so the reference proves
    linker reality rather than only self-consistency. Never present a proposed
    name or signature as callable, and fail the build on generated-page drift.
    Native and portable linker conformance, focused Web delivery, `./test`,
    optimized `./build`, and Nix flake evaluation passed. (Completed 2026-08-13
    18:06 EDT.)
    - Curiosity poke: keep proposal granularity stable enough for durable links
      without prematurely freezing function names or signatures.
  - [x] Add a prominent keyboard-accessible dropzone/file picker to the launch
    page for arbitrary `.wat` and `.aed` applications. Resolve `.aed` through
    the same bounded virtual application root as native, keep selected bytes
    local to the browser, and explain that a lone `.wat` cannot discover
    unselected sibling assets. The browser validates the native stored-ZIP
    package profile, paths, entry/total limits, CRCs, mimetype sentinel, and
    `code.wat`; retains only WAT plus `assets/` in origin-local IndexedDB; and
    launches an opaque-token generic runner without uploading bytes. Real
    Vibesteroids `.aed` parsing and live WAT/`.aed` headless launches pass.
    (2026-07-24 17:25 EDT)
  - [x] Keep the Ulam preview static until pointer rollover or keyboard focus,
    animate only for that interaction, and honor reduced-motion preferences.
    (2026-07-24 17:25 EDT)
  - [x] On each Vibesteroids rollover/focus entry, choose a new random rotation
    direction; rotate and repeatedly fire only while interaction remains
    active. Do not simulate asteroid impacts. Direction selection is
    deterministic under injected randomness and reduced-motion remains
    authoritative. (2026-07-24 17:25 EDT)
  - [ ] Receive Peter's visual and interaction acceptance for the gallery
    dropzone and hover/focus preview animations.
  - Curiosity poke: touch devices have no persistent hover, so provide a clear
    finite touch/focus behavior without leaving animation permanently running.

- [x] Add a capability-bounded, guest-authored external-link primitive to AVP.
  - [x] Let configure-time WAT declarations bind a stable link ID and display
    text to one validated, bounded UTF-8 `https:` URL; expose no general browser
    or network API. (2026-07-24 18:44 EDT: ABI v0.4 accepts only absolute,
    credential-free HTTPS and normalizes it before publication.)
  - [x] Let a retained integer/Q16 UI transaction place that declared link at
    guest-owned viewport geometry. Native and browser adapters activate it only
    from a real human gesture; headless adapters report the exact request
    deterministically without opening a browser. (2026-07-24 18:44 EDT:
    focused core/native/headless gates and wasm32 compilation pass.)
  - [x] Make activation, popup-blocking/platform rejection, accessibility, and
    stale-snapshot behavior explicit and testable, then send the green ABI/SHA
    to `ulam-flower-wat` for its `What is this?` README link. (2026-07-24
    18:57 EDT: standard suite green in 28.2s after doc cleanup; optional
    Chromium/WebGPU suite green in 23.8s, including transient activation,
    `_blank`, `noopener`, and zero canvas pointer leakage. SHA handoff follows
    the green commit.)
  - [x] Refresh the Nix Cargo-vendor fixed-output pin after promoting `url` to
    a direct dependency; the original six-target CI run independently exposed
    the stale hash. (2026-07-24 19:13 EDT: canonical `./test` and optimized
    `./build` pass with the CI-reported replacement hash.)
  - [x] Adopt the contract in the real Ulam guest, pin its public
    `30cc41b4fa6e12b2b691d8eecc9ef490d252d937` snapshot into Aedicule, and make
    the optional trusted-browser gate click that shipped provenance link
    instead of a synthetic fixture. (2026-07-24 20:09 EDT: guest acceptance
    and snapshot/surface classifiers pass. The complete host suite and
    optimized build pass; an initial aggregate Chromium startup timed out,
    then the isolated real-Ulam trusted-click gate and the complete warm
    `./test_browser` rerun both passed.)
  - Curiosity poke: opening a new browsing context must not grant an opener
    reference or let a guest smuggle non-HTTPS schemes through URL parsing.

- [ ] Reduce the complete-suite feedback loop from the measured 62m37s cold
  GitHub run to a bounded build phase plus fast warm test execution.
  - [x] Keep headless native/portable runtime tests from compiling GPUI merely
    because GUI test support is an unconditional development dependency.
    (2026-07-23 19:50 EDT: `cargo tree` proves the headless graph contains no
    GPUI; GUI-only tests activate `gui-test-support` explicitly.)
  - [x] Retain useful Aedicule test backtraces while omitting full debug data
    from third-party dependency artifacts, and remove redundant compilation
    profiles from the complete-suite runner. (2026-07-23 19:50 EDT: cold local
    suite passed in 10m46s and the first unchanged warm run passed in 41.3s;
    a second trace-isolated warm run passed in 25.6s.)
  - [x] Buffer clean checks, replay failures deterministically, overlap every
    dependency-independent suite lane, reuse a project-local browser/font
    cache, close Chromium through CDP, and avoid rebuilding the GUI host solely
    for its browser probe. (2026-07-23 20:25 EDT: two unchanged complete-suite
    runs passed in 10.4s and 10.3s; `./build` passed in 1.9s. On 2026-07-24
    08:43 EDT, a newly observed concurrent Nix evaluation-cache SQLite
    collision received a failing classifier and independent reusable cache
    namespaces; the complete suite again passed in 11.1s.)
  - [x] Cache only reusable Cargo dependencies between trusted `yolo` CI runs,
    with the Nix toolchain included in the cache key and the cache kept below
    GitHub's repository quota. (Implementation and structural gates complete
    2026-07-23 19:50 EDT. The first cold population passed 2026-07-23 20:36
    EDT: 39m04s suite step plus 3m24s upload for a 3.70 GB Rust cache. The
    first warm run passed 2026-07-23 21:05 EDT: 1m40s restore, 18m15s suite,
    47s post step, 21m17s test job, and 27m10s full six-target deployment.)
  - [x] Reclaim only unrelated Android/.NET/Haskell/Docker payloads in the
    ephemeral GitHub test runner and every six-target matrix runner before
    expanding the 3.70 GB compressed Rust cache or cross-building, and fail
    early unless at least 25,000,000 KiB remains. The first audio-fix run had
    filled the test image during restore so completely that the Actions worker
    could not write its own diagnostic log; the first protected follow-up
    crossed that boundary, then proved Windows/x86_64 needed the same policy
    after cross-compilation exhausted its image at 85 MB free.
    (2026-07-24 11:29 EDT; action is pinned to immutable v1.3.1 commit; the
    deliberately failing target-job classifier, focused policy test,
    18.3-second complete suite, and optimized build passed.)
  - [x] Keep `./test` and the standard Nix check browser-free while preserving
    deterministic Web Audio, DOM-startup, WAT-boundary, GUI, and runtime tests.
    Move the real packaged synth-audio plus W/A/D Chromium gate to explicit
    `./test_browser`; record success atomically and remind interactive
    developers only after 48 hours plus a newer commit. Normal CI retains one
    live Pages browser gate rather than duplicating it in the test job.
    (2026-07-24 12:19 EDT; Peter identified the excessive integration boundary;
    partition/reminder classifiers failed first, Chromium moved into a
    dedicated `devShell.browser`, the final fast suite passes in 13.7s, and the
    optional live browser suite passes in 11.3s.)
  - [x] Add `./serve_web` to build and serve the exact static Pages delivery
    locally on loopback by default, with an explicit `--listen` override for
    LAN/tailnet testing. (2026-07-24 12:22 EDT; clean help/error and structural
    delivery tests pass; the gallery and both demos returned HTTP 200 with
    COOP/COEP through the live local server, which then shut down cleanly.
    Default moved from commonly occupied port 8080 to 8910 at 2026-07-24 14:32
    EDT. At 2026-07-24 14:57 EDT, tailnet testing caught Caddy binding every
    interface and matching only the loopback Host: the HTTPS proxy returned an
    empty 200 page. Caddy now binds loopback explicitly while accepting proxy
    Host headers; focused tests, the 12.9-second full suite, optimized build,
    exact tailnet HTTPS assets, and isolated Chromium gallery rendering pass.)
  - [ ] Move the clean-room matrix to a persistent Thelio/self-hosted Nix
    execution path, or cache exact tested Nix derivations: GitHub dependency
    restoration cut the suite step by 2.1×, but transfer/relinking still makes
    hosted CI minutes-scale while unchanged local `./test` is 10–11 seconds.
  - Curiosity poke: measure both cold compile time and warm assertion time so a
    cache hit cannot conceal a pathological clean build.

- [ ] Publish a versioned, self-contained `GUIDE_FOR_LLMS.md` for writing and
  maintaining Aedicule guests without access to the host source.
  - [x] Compose the checked-in guide during the build from an authored teaching
    template plus the typed/generated ABI source of truth, and fail tests on
    drift. (2026-07-23 14:45 EDT)
  - [x] Explain the complete current lifecycle/import/export surface with
    minimal commented WAT patterns, transactional reload/state rules, exact
    arithmetic profiles, capabilities, packaging, testing, and host parity.
    (2026-07-23 14:45 EDT)
  - [x] Clearly fence proposed/future APIs from callable current APIs, with
    links to the canonical online guide and semantic guide versioning.
    (2026-07-23 14:45 EDT)
  - [x] Add a practical LLM mistake checklist and recommend maintained WAT/Wasm
    formatters, validators, analyzers, and deterministic Aedicule test tools.
    (2026-07-23 14:45 EDT)
  - [ ] Include the guide in release demo directories and `.aed` packages,
    coordinating new immutable downstream demo pins rather than silently
    mutating their published snapshots.
  - Curiosity poke: copied guides can outlive both host and guest, so every
    package needs an explicit guide/ABI compatibility statement that remains
    useful offline but points to the canonical current copy when online.

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
    - [ ] Layer appified menus: package metadata owns application
      name/About/icon and selects runner versus single-app mode; AVP declares
      arbitrary hierarchical guest menus with stable action IDs; the host
      retains an unshadowable Quit/recovery floor. Runner mode keeps Open…;
      appified mode exposes document opening only when the package requests
      that capability.
    - [ ] Make menu accelerators typed AVP data: stable key IDs plus semantic
      Primary/Command/Control/Alt/Shift modifiers, atomically reject duplicate
      guest chords, reserve chords by runner/appified capability profile, and
      emit one menu-action event without also leaking a raw key event.
  - [ ] When at least one actionable guest `AE_menu_item` is declared, render
    its menu model through an in-window title-bar hamburger on every native
    platform while retaining platform-native menus where available; both
    surfaces dispatch the same ordered action ID and occlude guest pointer
    input. Omit the hamburger when no actionable items exist (separators alone
    do not count).
    Curiosity poke: preserve declared separator order and keyboard focus while
    bounding menu height/overflow for a hostile or simply very large menu.
  - [ ] Add a semantic window-mode contract: package metadata may request the
    initial mode and WAT may request runtime windowed/fullscreen transitions,
    but Aedicule owns the platform mechanism, may reject the request, and emits
    the actual committed mode once as an ordered event followed by viewport
    geometry. Fullscreen hides the entire custom title bar and its contents;
    the host retains an unshadowable human exit/recovery action.
    - [ ] Default to standard windowed chrome, then resolve safety floor >
      current human override > runtime guest request > package initial
      preference > host default; suppress repeated guest requests while a
      human override is pinned and report the actual mode.
    - [ ] In native Aedicule, reserve Shift–Escape as the non-overridable safety
      toggle. Supply configurable aliases Control–Command–F and
      Command–Shift–F on macOS, plus Alt–Enter, F11, and Control–Shift–F on
      Windows/Linux; consume a recognized semantic chord exactly once without
      leaking raw key events.
    - [ ] On every native fullscreen entry, briefly show an italic,
      noninteractive, click-through adapter-owned safety notice that
      guest/package code cannot suppress: “Press Shift–Escape to toggle
      fullscreen.”
    - [ ] In browsers, leave fullscreen entry, exit, shortcuts, and guidance
      entirely to the browser. Aedicule only observes `fullscreenchange` plus
      viewport geometry and reacts to actual state—for example, hiding its own
      in-page title bar while fullscreen—without adding a competing web
      fullscreen controller or mandatory web safety overlay.
    Curiosity poke: keep title-bar guest-menu invocation distinct from the
    platform window-operations context convention; do not steal canvas
    right-click when fullscreen removes the title bar.
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
  - [x] Re-audit the complete portable-application slice against current
    artifacts: the real Vibesteroids directory depackages/repackages to the
    identical `b22e5002…` `.aed`, its 1+ MiB WAST suite passes with an emitted
    replay seed, bare/directory/archive inputs render byte-identical SVG, both
    public Pages and dynamic `--web` fetch the exact schema-11 WAT and FLAC,
    and the full test/build/six-target/reproducibility gates pass.
    (2026-07-23 08:44 EDT.)
  - [ ] Expose packaged image assets by virtual name without granting arbitrary
    virtual-filesystem reads, then add browser image-adapter parity.
  - [ ] Add an event-driven safe-area inset contract for full-bleed hosts:
    logical-unit left/top/right/bottom insets emitted initially and only when
    native chrome or platform cutouts change, independently of drawable
    viewport geometry.
    Curiosity poke: browser and native adapters have different occlusions, so
    report measured zero/nonzero insets rather than baking the desktop footer
    height into guest layout policy.
  - Curiosity poke: non-loopback browsers require a secure context as well as
    COOP/COEP, so remote/tailnet serving needs an explicit TLS/reverse-proxy
    contract rather than silently printing an unusable HTTP URL.
  - Curiosity poke: platform signing/notarization mutates or wraps appified
    artifacts, so specify the unsigned deterministic core before promising a
    byte-identical appify/deappify/appify round trip.

- [ ] Publish the bundled browser demos as a concise, modern GitHub Pages site.
  - [ ] Add a generic, demo-free browser runner that opens `.wat` or `.aed`
    through an explicit file picker and drag/drop, resolves selected packages
    through the same bounded virtual application root as native, and keeps all
    selected bytes local to the browser.
    - [x] Content-address the optimized Aedicule runtime Wasm by SHA-256, patch
      its loader to the exact immutable URL, retain it cache-first on static
      Pages, and emit one-year immutable headers from Caddy/native `--web`
      while every guest/bootstrap/application asset remains `no-store`.
      (2026-07-23 14:15 EDT; exact hash classifier, real 15 MiB optimized Nix
      bundle, loader reference, mutable/immutable HTTP policy, and service-
      worker cache path pass.)
    Curiosity poke: a lone dropped `code.wat` cannot carry sibling assets, so
    reject or clearly explain incomplete loose-project drops instead of
    pretending asset lookup succeeded.
  - [x] Render the accepted v0 AVP panel/slider/button document in the browser
    adapter with the same values, action/control events, stable IDs, and canvas
    occlusion as native.
    (2026-07-23 14:05 EDT; the optimized Pages bundle exposes the real Ulam
    panel, 4 buttons, and 2 sliders. Headless WebGPU click/drag acceptance
    proves MenuAction/Control delivery and zero raw-pointer leakage.)
    Curiosity poke: browser controls must emit one semantic event and must not
    leak the same gesture through to the underlying canvas.
  - [x] Give the browser frontplane a focusable keyboard surface and deliver
    the same supported physical key-down/key-up IDs as native; Vibesteroids
    previously received no web keyboard events.
    (2026-07-23 13:49 EDT; shared set-classifier, Wasm compile, and real
    Chrome→GPUI→scheduler→WAT→committed-frame regression pass. Opt-in traces
    now distinguish DOM dispatch, frontplane admission, and guest delivery.)
    Curiosity poke: suppress browser defaults only for admitted guest keys and
    preserve both edges so held controls cannot become stuck.
  - [x] Append stable physical-key IDs for W, A, and D without changing any
    existing key number; map native GPUI and browser representations, prove
    down/up edges cross the actual WAT boundary, regenerate ABI/LLM docs, and
    publish the tested SHA for Vibesteroids. Gameplay aliases remain entirely
    guest-owned.
    (2026-07-24 11:29 EDT; append-only IDs 13/14/15, native mapping, portable
    runtime boundary, real Chromium→WAT committed-frame acceptance, generated
    docs, full suite, and optimized build passed at `e8736f9`.)
    Curiosity poke: focus-loss/pause reconciliation must remain a generic held-
    key mechanism and never learn Vibesteroids thrust/rotation semantics.
  - [x] When WebGPU admission fails, show concise current enablement steps for
    Firefox, Chrome, and Safari, with a reload-first/restart-if-still-blocked
    diagnostic path and links to browser-owned documentation.
    (2026-07-23 14:10 EDT; pure i18n tests cover missing-API and null-adapter
    paths plus every browser setting/diagnostic and official help URL.)
    Curiosity poke: distinguish “API disabled by preference” from unsupported
    OS/GPU/driver/blocklist failures instead of prescribing unsafe force flags
    for every rejection.
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
  - [x] Make hosted Pages acceptance select the runner-provided stable Chrome
    explicitly and allow a bounded 60-second cold/contended browser launch,
    after an otherwise-green build timed out before publishing DevTools.
    (2026-07-23 08:54 EDT; policy failed first, then the complete suite passed.)
  - [ ] Upgrade GitHub's Pages actions to the current Node-24 majors:
    `configure-pages@v6`, `upload-pages-artifact@v5`, and `deploy-pages@v5`.
    The v0.1.1 workflow passed, but GitHub annotated the older majors as
    Node-20 compatibility shims. (Official release metadata checked
    2026-07-22 23:23 EDT.)
  - Curiosity poke: does mobile Safari accept the service-worker-controlled
    reload and expose both SharedArrayBuffer and a usable WebGPU adapter?

- [ ] Ship Aedicule and the Ulam Flower/Vibesteroids demos as one coherent
  six-target delivery matrix.
  - [x] Register `.aed` as an Aedicule-owned macOS document type and give
    packaged runnable applications a distinct package document icon rather
    than reusing the editable `.wat` source icon.
    (2026-07-23 13:25 EDT; the full suite and release build passed, macOS
    validated the shipped plist/signature, and opening the installed demo
    `.aed` through Launch Services started Aedicule.)
    Curiosity poke: Finder caches Launch Services declarations and icons, so
    the installed-app acceptance must refresh registration rather than mistake
    stale metadata for a failed bundle.
  - [x] Make a no-argument native launch usable without guest cooperation:
    always expose host-owned About, Open…, and Quit actions; let Open… select
    `.wat`, `.aed`, or an application directory; and render an actionable
    empty state instead of silently launching the black conformance guest.
    (2026-07-23 13:25 EDT; focused tests, the full suite, the macOS release
    build, and Peter's live Mac visual acceptance passed.)
    Curiosity poke: preserve guest-authored application commands in a separate
    menu so a guest cannot rename, shadow, or remove the host escape hatches.
  - [x] Install the two bundled demo applications beside the private Mac test
    install in a stable, visible user directory, and print their exact paths.
    (2026-07-23 13:25 EDT; publisher contract and full suite passed;
    `~/Documents/AediculeDemos/` is populated on Peter's Mac.)
    Curiosity poke: update publisher-owned demo copies transactionally without
    overwriting a user's mutable project directory.
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
      - [x] Publish a v0.1.4 corrective Mac artifact without rewriting the
        immutable v0.1.3 audit trail.
        - [x] Recover the lost v0.1.3 GitHub tag-trigger event by completing
          the independent local rebuild gate, manually uploading only the
          checksum-bound exact-tag artifacts, and resuming `./publish`.
          GitHub CI and Mechatron Prime passed; the public release exists.
          (2026-07-23 04:55 EDT.)
        - [x] Reproduce the installed app's pre-`main` dyld abort: the
          Linux-to-Darwin link retained three `/usr/lib/libobjc.A.dylib` load
          commands even though the Mach-O signature verified. Add a LuaJIT
          set-classifier over every dylib load command and run it before
          signing. (2026-07-23 05:18 EDT.)
        - [x] Pass `-dead_strip_dylibs` through the cross link, prove the real
          Nix artifact has a unique dylib set, copy that exact signed binary to
          Peter's M4 Mac, and run `--about` successfully there. Also require
          the publisher's staged binary to pass the same startup probe before
          touching `~/Applications`. (2026-07-23 05:30 EDT.)
        - [x] Run the complete `./test`, isolated Nix test, `./build`,
          six-target `./build_all`, archive-checksum, independent
          reproducibility, formatting, ShellCheck, actionlint, and diff gates.
          (2026-07-23 06:37 EDT.)
        - [x] Commit and push the corrective source as `8d053096`, publish all
          six independently rebuilt/checksum-bound v0.1.4 archives, and verify
          the public Mac binary and private installed app on Peter's M4. The
          untouched public binary starts and has one `libobjc`; the publisher's
          repaired private bundle has a valid ad-hoc signature and starts from
          `~/Applications`. (2026-07-23 08:17 EDT.)
    - [ ] Make the public macOS archive Gatekeeper-clean by adapting
      `../validate_gui`'s external release-keychain, Developer ID hardened
      runtime/timestamp, Apple notarization, stapling, `spctl`, and receipt
      contract. Independent M4 verification proved that the untouched v0.1.4
      ZIP starts but fails strict bundle-signature verification because its
      resources were assembled after the Mach-O was signed; publish only a
      post-assembly signed/stapled final ZIP once this credentialed gate exists.
      Treat a native-M4 build of the exact release commit as the only supported
      Mac release lane; keep Linux-to-Darwin output diagnostic-only. Before
      signing, reject duplicate dylib load paths in every executable/helper/
      framework; then sign nested code before the outer app, verify
      `codesign --verify --deep --strict`, run the executable `--about`, and
      keep notarization/stapling as a separate credentialed distribution gate.
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
    - [ ] Split the isolated Nix check's release, native-runtime, GUI-test, and
      script gates into reusable derivations so a shell/script-only source
      change does not recompile three Rust feature graphs. The browser-suite
      partition gate measured about 14–20 seconds locally, but the exact cold
      Nix check still took about 9.5 minutes on 2026-07-24.
      Curiosity poke: keep each derived gate exact-commit-bound so finer cache
      reuse cannot accidentally validate source from a different revision.
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
  - [x] Add a configure-time standalone `AE_action` declaration so a real AVP
    button can own bounded visible/accessibility text without publishing an
    always-present application-menu item. Share collision checking with menu
    actions, preserve legacy menu-backed buttons, and prove exact activation
    identity plus absence from native/browser menus in native, browser, and
    headless adapters. Peter explicitly authorized the real GPUI Component
    button surface on every adapter; no canvas or ad hoc substitute. Requested
    by vibesteroids_wat on 2026-08-05. Native, headless, and real
    Chromium/WebGPU tests pass; ABI docs are generated as v0.8 and the full
    canonical suite is green. (2026-08-05 20:32 EDT.) Send the immutable pin
    after committing this savepoint.
    Curiosity poke: a standalone action and a menu declaration must never be
    able to disagree about the label for one ID, regardless of declaration
    order.
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
    - [x] Make delivery configure-time opt-in so legacy guests keep the
      touch-to-primary-pointer compatibility path. The opt-in admits an
      ordered, bounded active-contact stream, suppresses only compatibility
      mouse echoes from those contacts, and retains concurrent real mouse,
      trackpad, keyboard, wheel, hover, and motion input.
      (2026-08-13 18:41 EDT: `AE_touch_interest(max_contacts, flags)` added in
      ABI v0.10; legacy guests retain the prior path.)
    - [x] Enforce active-ID uniqueness and deterministic terminal semantics:
      IDs may be reused only after end/cancel; unknown moves/terminals and
      duplicate starts cannot mutate another contact; focus loss, lost capture,
      pointer cancellation, and teardown cannot strand a held contact.
      Terminal edges carry the last admitted logical coordinate, and resize
      ordering follows the existing device-change barrier.
      (2026-08-13 18:41 EDT)
    - [x] Define and enforce the host active-contact limit, retain ordered
      delivery before fixed ticks, and make retained AVP controls occlude
      contact starts exactly as they occlude gameplay pointer starts.
      (2026-08-13 18:41 EDT: hard host ceiling 16; guest selects 1..=16.)
    - [ ] Add a versioned semantic input-capabilities event before the first
      render and whenever the set changes: keyboard, fine pointer, hover,
      wheel, touch, multi-touch, motion, and later gamepad—not OS/browser names
      or one global mobile mode. Preserve concurrent modality-specific held
      state, startup ordering, hot-plug, and reload/restore across changes.
    - [x] Prove at the live Chrome/DOM layer that two contacts move
      independently, end separately, and a third contact cancels while three
      opaque IDs remain distinct. (2026-07-20 21:18 EDT)
    - [x] Prove those same phases and identities reach `AE_event` through core
      WAT tests and the rebuilt browser runtime.
      (2026-08-13 18:41 EDT: real Chromium delivered 3 starts, 2 moves, 2
      ends, and 1 cancel across three opaque IDs.)
    - [ ] Reproduce Peter's 2026-08-14 iPhone result where no Vibesteroids touch
      control responds, using the exact staged guest and mobile viewport. Add a
      behavioral browser oracle that proves edge stroke rotation, edge-held
      fire, center-held thrust, independent simultaneous contacts, terminal
      release/cancel, and the pause region by observing guest state or rendered
      output rather than merely counting delivered ABI events. Fix the owning
      layer only after the new gate fails RED.
      (2026-08-14 10:37 EDT: RED proved that the raw bridge called
      `preventDefault()` on AVP-owned contacts after Vibesteroids opted into raw
      touch. The bridge now leaves the complete occluded contact lifetime to
      GPUI. A combined 430x775 Chromium gate and deterministic actual-guest SVG
      comparison are green. Peter confirmed working multitouch on iPhone Safari
      at 13:12 EDT. (Completed 2026-08-14 13:12 EDT.)
    - [x] Triage and hand off Peter's post-acceptance Vibesteroids refinements,
      implementing any blocking host capability before guest work proceeds:
      show help while paused on multitouch devices; describe touch controls in
      that conditional help; provide a mobile Death Blossom trigger (shake or
      another discoverable gesture); increase temporary gift frequency; draw a
      bow on the gift; and reject unsafe satellite/enemy-ship spawn regions as
      the game already does for other fairness-delayed spawns. All six are
      guest-owned: DeviceChange bit 0 plus observed raw touch supplies the
      current phone-mode discriminator, while existing `AE_motion_interest`
      kind 1 and event kind 16/code 1 supply host-derived shake. Sent the
      tested contracts and acceptance criteria to Vibesteroids in
      `inbox/2026-08-14-from-aedicule-six-mobile-refinements.md`; the planned
      exact semantic capabilities event remains useful for hybrids but does
      not block this request. (Completed 2026-08-14 13:17 EDT.)
    - [x] Add repeatable actual-binary touch-sequence arguments to
      `aedicule-render` so downstream `.aed` projects can test simultaneous
      IDs, interleaved movement, independent terminal edges, cancellation,
      and ID reuse against an immutable Aedicule package.
      (Done 2026-08-14 10:39 EDT: `--touch PHASE,ID,X,Y` uses the same bounded
      tracker as Web, rejects malformed/non-finite input as a set, requires
      guest opt-in, and preserves ordered opaque IDs. Interleaved `--advance N`
      steps prove held behavior across fixed ticks before later move/end/cancel
      edges; a deliberate reorder remains red. The exact staged Vibesteroids
      guest changed deterministic SVG output only after action/touch/tick
      ordering was corrected.)
    - [x] Restore the Nix CI test runtime closure exposed by touch-savepoint
      shipment `5f68591`. The exact Mechatron commit passed six-platform
      `release-all`, then all native and touch tests passed before all five
      `gui_cli` cases exited 127 because `libxcb.so.1` was absent from the
      check process runtime path. The direct Nix check reproduced the same
      failure. Add the existing Linux GUI library set to the test derivation's
      `LD_LIBRARY_PATH` and rerun the exact check. The corrected direct Nix
      check passed all five GUI CLI cases and its complete check phase.
      (Completed 2026-08-14 11:40 EDT.)
    - [x] Prevent GPUI Web's current mouse-compatibility conversion from
      duplicating each touch as primary-button input.
      (2026-08-13 18:41 EDT: phase-and-coordinate markers survive GPUI's queue
      hop; the browser gate measured zero compatibility pointer deliveries.)
    - [x] Hand the raw-contact contract to Vibesteroids so that guest—not
      Aedicule—maps upward travel on the left half or downward travel on the
      right half to proportional ship rotation on fullscreen mobile.
      (2026-08-13 18:41 EDT: exact opt-in correction sent in
      `../vibesteroids_wat/inbox/2026-08-13-from-aedicule-enable-current-touch-opt-in.md`.)
    - [ ] Make the mobile demo fill the available viewport immediately and
      provide a user-activated fullscreen entry surface where browser policy
      forbids automatic fullscreen; preserve the browser-owned exit path.
    - Curiosity poke: Chrome remapped the probe's requested touch IDs 41/42/43
      to pointer IDs 2/3/4; clients must treat IDs as opaque, page-local
      correlation tokens and never persist or interpret their numeric values.
  - [ ] Separately extend host-owned suspension so a guest may classify a
    dynamic contact region and request pause without encoding any demo zone in
    Aedicule, while resume remains host-observable when ordinary guest ticks
    are stopped. This must not delay the raw multi-contact stream.
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
  - [ ] Make every explicitly loaded native `.wat`, directory, or `.aed` source
    live-watched by default; add an explicit `--no-watch` escape hatch for
    fixed-run workflows while retaining `--watch` for compatibility.
  - [ ] Make `--package` publish through a complete sibling temporary plus an
    atomic same-filesystem replacement, so an active watcher can observe only
    the old or complete new `.aed`, never a half-written ZIP.
  - [ ] Show initial/reload syntax and candidate-admission errors briefly in a
    noninteractive pass-through overlay while retaining the last known-good
    guest, frame, and pre-candidate state; also preserve terminal stderr and an
    optional durable-log sink without making either the only visible report.
  - [ ] Prove candidate events/ticks/rendering cannot mutate the active guest:
    any failed candidate is discarded whole, and the prior runtime-modified
    state resumes byte-for-byte until a fully admitted replacement commits.
  - Curiosity poke: editors and atomic package writers commonly emit rename
    bursts; debounce by source identity and content generation, not sleeps, so
    one complete candidate is evaluated without suppressing a later change.
- [x] Define a host-scheduled guest-suspension ABI that consumes no guest CPU
  while paused but leaves pause presentation and application state guest-owned.
  Use the settled host-converted boundary: the guest declaratively registers
  bounded typed pause triggers during configuration; Aedicule consumes their
  fresh physical edges and emits semantic suspension lifecycle events instead
  of leaking the raw trigger into guest gameplay. A guest that registers no
  pause trigger retains its current lifecycle unchanged.
  (2026-07-23 18:11 EDT: ABI v0.3, native/browser adapters, exact scheduler
  phase, input reconciliation, reload restoration, guest-audio transport,
  generated references, full `./test`, and optimized `./build` are green.)
  - [x] Add `AE_pause_trigger(kind, code, flags)` with a version-0 stable-key
    selector, bounded declarations, generated documentation, and additive ABI
    versioning; reserve the selector kind so controllers and menu actions can
    join without overloading raw key IDs.
  - [x] Specify exact suspend/resume ordering, the one final paused render,
    cached-scene repaint, and monotonic scheduler-baseline reset with no
    accumulated paused-time catch-up.
  - [x] Reconcile releases, cancellation, focus loss, and new presses observed
    while suspended; a held or repeated pause trigger must not self-resume.
  - [x] Keep native window/menu/file-watch/reload machinery live, teach a
    replacement guest that suspension was restored, and define resize as
    viewport event plus at most one paused render without restarting ticks.
  - [x] Freeze a per-guest audio transport at the same logical pause barrier:
    retain FLAC sample cursors, synth phase/envelopes, and cooldown time; resume
    without restart, duplication, truncation, pitch drift, or wall-time expiry.
    Keep host-chrome audio on a separate bus, specify reload preservation versus
    deterministic cancellation, and test/document unavoidable device-buffer
    latency without leaking it into guest simulation time.
  - Curiosity poke: controller, menu, and future network wake sources should use
    typed semantic selectors rather than making raw keyboard IDs the permanent
    suspension policy language.
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

- STANDING POLICY (Peter via vibesteroids_wat's 2026-08-05 note, quoting him:
  "aedicule's manifest should always be updated to the latest ulam-flower,
  vibesteroids, and other demos"): demo freshness is continuous, not
  per-request. Advance `demos/manifest.tsv` + `tests/cli/demo_snapshots`
  together whenever a demo lands a tranche (they send pin pairs unprompted;
  pulling their repo directly is equally valid). This supersedes the earlier
  "no .aed regeneration without explicit say-so" instruction. The b3e1949
  advance to df61ffe already conforms; their note's pin pair independently
  matched ours. They will adopt 2ca03e5 + declare AE_abi_minor 6 with their
  touch tranche, after Peter's current 61f287f playtest.
  - [x] Refresh the Tailscale phone staging demo from Vibesteroids' current
    implementation, validate the packaged guest, and report the exact served
    source/package hashes. The guest's green multi-contact implementation did
    not yet declare the final host opt-in, so the staging candidate adds only
    `AE_touch_interest(8, 0)` and ABI minor 10 while the guest agent lands those
    lines at source. Real Chromium observed all eight ordered touch transitions
    with zero compatibility-pointer duplicates. The Tailscale gallery now
    serves WAT SHA-256 `5ddee65c...b059` and AED SHA-256
    `8bca22da...000b`. (Done 2026-08-13 19:06 EDT.)
- [ ] Vibesteroids on iPhone (Peter, 2026-08-03 hardware pass): three gaps, split
  by responsibility. This is exactly what the demo apps exist to tease out —
  "aedicule apps may need to be client-aware."
  - [ ] Protocol gap, host-side — DESIGN CHOSEN (Peter, 2026-08-04 ~3:41 PM
    EDT): extend the existing viewport event (WAT ABI kind 6) into the
    "device-change event". Its `code` slot is a hardcoded 0 in both ABIs
    (`src/lib.rs` legacy_event_parameters / integer_event_parameters), so it
    becomes a device-class flags bitfield: bit 0 = coarse primary pointer
    (CSS `(pointer: coarse)`, already read host-side for slop arming); other
    bits reserved-zero, guests mask only known bits. Delivery: at least once
    at app start (both adapters already fire on first observation — verified
    2026-08-04), and on any change to dimensions OR flags (dedupe key must
    grow the flags so a size-unchanged class flip emits, e.g. iPad gaining a
    trackpad or a Mac window dragged to a Sidecar iPad). Prose rename in
    VIEW_PROTOCOL.md lands with the implementation.
    - [x] Contract proposal sent to vibesteroids_wat, asking whether bit 1
      (hover absence) or anything else is needed; implementation on BOTH
      sides gated on their reply confirming the bit layout.
      (inbox/2026-08-04-from-aedicule-device-change-event-contract.md,
      2026-08-04 ~3:50 PM EDT)
    - [x] vibesteroids_wat confirmed bit 0 only (their reasoned declines of
      hover-absence and any-pointer bits are in
      inbox/processed/2026-08-04-from-vibesteroids_wat-device-change-bit-0-confirmed.md);
      they also accepted the Start Game gate and flagged the six-axis
      dependency for Death Blossom's shake. (2026-08-04 ~9:32 PM EDT)
    - [x] Host side implemented via strict TDD (2026-08-04 ~10 PM EDT):
      `Event::Viewport` renamed `Event::DeviceChange` with a `flags` field
      carried in the kind-6 code slot (runtime red proved the encoding);
      shared `DeviceChangeTracker` in lib dedupes on size AND flags (red
      proved a size-unchanged flag flip emits) and both adapters use it; the
      browser adapter re-reads `(pointer: coarse)` per observation so slop
      arming follows live flips; ABI minor bumped 5 → 6 (red: a minor-6
      guest was rejected `UnsupportedAbiMinor`) so guests can REQUIRE the
      capability; WAT_ABI.md + GUIDE_FOR_LLMS.md regenerated with the
      device-change row, flag-bit definition, and delivery guarantees;
      surface tripwires guard the adapter wiring.
    - [x] CI green on 2ca03e5; ship notice sent to vibesteroids_wat with pin
      guidance and the "declare AE_abi_minor 6 only if you REQUIRE device
      flags" rule.
      (inbox/2026-08-04-from-aedicule-device-change-host-side-shipped.md,
      2026-08-04 ~10:35 PM EDT)
    - [ ] Optional hardening: end-to-end Chromium gate asserting the boot
      device-change event carries bit 0 under forced `(pointer: coarse)`
      emulated media (CDP Emulation.setEmulatedMedia), once worth the wiring.
    - [x] VIEW_PROTOCOL.md prose renamed: the layout section now says a
      device change (viewport dimensions or device-class flags, kind 6)
      produces the ordered event. (2026-08-04 ~10:05 PM EDT)
  - [x] Six-axis motion capability IMPLEMENTED at ABI v0.7 (Peter authorized
    2026-08-05 ~1 PM EDT "proceed w/any building"; built via strict TDD):
    - Registration: `AE_motion_interest(kind, rate_hz, flags)` host import,
      configure-only, following the AE_pause_trigger precedent. Kind 1 =
      host-derived shake gesture (rate 0); kind 2 = six-axis sample stream
      (rate 0 -> 60 Hz default, else 1..=120). Whole argument domain
      classified in tests/motion.rs; invalid registrations reject the
      configure transaction; a kind-2 interest without an `AE_motion_event`
      export rolls configure back (same rule as sliders/text fields).
    - Delivery: shake = event kind 16 code 1 with peak magnitude (f32
      legacy / Q16.16 integer); samples = dedicated `AE_motion_event(ax ay
      az rx ry rz)` export, Q16.16 in BOTH profiles (m/s2, deg/s), never
      through AE_event. Undeclared motion events are rejected, not
      delivered. Sensor absence / denied permission = silence, never error;
      guests keep a manual fallback (vibesteroids' documented plan).
    - Pure `ShakeDetector` (threshold 15 m/s2, cooldown 1500 ms, injected
      time) unit-tested over synthetic streams; browser adapter drains a
      bounded page-side devicemotion ring per frame and runs detection and
      the guest's registered rate limit in Rust. bootstrap.js requests the
      iOS DeviceMotion permission inside the first user gesture when the
      guest declared interest (`__AEDICULE_MOTION_INTEREST`), recording the
      outcome in `__AEDICULE_MOTION_DIAGNOSTIC`.
    - ABI minor 6 -> 7; generated docs updated (import row, export, kind 16,
      guarantees); surface tripwires guard the browser wiring; native
      adapter stays sensor-silent by design.
    - [x] Shipped as 1b4cde7, CI green across all six jobs (2026-08-05
      ~2 PM EDT); deployed to the Tailscale endpoint; ship notice with the
      full contract, the minor-7-only-if-required rule, and the lifted
      playtest hold sent to vibesteroids_wat.
      (inbox/2026-08-05-from-aedicule-motion-capability-shipped-and-playtest-hold-lifted.md)
    - [ ] Later, unbuilt: DeviceOrientation (attitude) profile, geolocation,
      compass — same registration gate when a demo needs them. Springs demo
      (two springs + weight, shake the phone) remains the six-axis exercise
      app idea, now unblocked.
  - [x] Audio silent on iPhone — root cause found on hardware (Peter,
    2026-08-04 ~1:49 PM EDT): Web Audio unlocks ONLY on a completed tap. Sound
    started the moment Peter tapped (not dragged) and stayed up from then on.
    Mechanism: iOS grants transient user activation for tap-like gestures, not
    pans, and the `bootstrap.js` unlock listeners
    (pointerdown/keydown/touchstart) all fire at gesture START, before WebKit
    grants activation. Earlier "no sound" reports are consistent with
    drag-only interaction plus the then-muted ringer switch. Follow-ups below.
  - [x] Hardware confirmation of the healthy pipeline (Peter, 2026-08-04
    ~2:03 PM EDT): `au running req 87 act false / au-last source-ended` — the
    guest made 87 playback requests, the context is running, and the last
    event is a source finishing. `act false` is normal expiry of transient
    activation. Audio investigation CLOSED.
  - [x] Add end-of-gesture events (touchend/pointerup/click) to the unlock
    listener set so the resume attempt lands inside the freshest activation
    window. TDD: surface tripwire on the full listener array went red, then
    green. (2026-08-04 ~2:15 PM EDT)
  - [x] Tap-for-sound cue DECIDED (Peter, 2026-08-04 ~4:09 PM EDT): no host
    mechanism. The guest presents a "Start Game" button, which guarantees the
    first interaction is a tap (not a drag) — and the host's window-capture
    unlock listeners make any tap unlock audio, so the pattern needs zero new
    code on either side. Guests cannot query audio state, so the pattern is
    "always gate at boot"; a conditional cue would need a capability query we
    have no present requirement for. Drag-never-unlocks is Apple policy and
    cannot be coded around. FYI note sent to vibesteroids_wat.
  - [ ] Guest gaps, vibesteroids_wat's side once the capability signal exists:
    thrust and Death Blossom have no touch affordance, rotation has no
    side-stroking control. Send an LLMsend note when the host side is designed.

- [ ] `?diag=1` startup failure on iPhone (Peter, 2026-08-04): "RangeError:
  Out of memory", reported stage `startup-lock-released`, 306 ms elapsed, all
  capability checks green. Code inspection findings (no fix applied yet):
  - [x] Instrument bug FIXED (2026-08-04 ~2:15 PM EDT): `failedStartupStage`
    in `startup-lock.mjs` pops trailing `startup-lock-released` entries
    before `reportStartupFailure` labels the failure; every other lock stage
    is preserved as a genuine failure position. Failure screen now also shows
    a two-line stack excerpt (message-line filtered so V8 and Safari render
    alike). TDD: unit test over the whole lock-stage domain in
    `web_startup_lock.mjs` went red, then green; surface tripwires added.
  - Prime suspect for the OOM itself: the served glue calls
    `new WebAssembly.Memory({initial: 82, maximum: 16384, shared: true})` —
    a shared memory reserves its full 1 GiB maximum at creation. Adding
    `?diag=1` to a tab that was just running the game forces a same-tab
    reload that races the old instance's teardown (its own 1 GiB reservation
    plus WebGPU buffers still resident), so the new reservation fails.
  - [x] Experiment (a) (Peter, 2026-08-04 ~2:03 PM EDT): the `?diag=1` URL
    boots fine in a FRESH tab — the diag overlay is exonerated as a cause.
  - [ ] Experiment (b), optional: reload the PLAIN URL in a tab that was just
    playing — hypothesis says it sometimes OOMs with no diag anywhere, which
    would positively confirm reload pressure. The next OOM's failure screen
    will now name the true stage and top stack frames either way.
  - [ ] Candidate mitigations once (b) or a truthful failure screen confirms
    (do not build yet): retry `init()` after a short backoff on RangeError;
    and/or lower `--max-memory` below 1 GiB if Aedicule's real ceiling allows
    (capacity decision for Peter).

- [ ] Remove Peter from the regression-verification loop (his question,
  2026-08-03 evening; answered in chat only until now). Peter stays acceptance
  authority at bless time — speaker sound, haptics, feel — but regression
  re-verification can be mechanized in three stages:
  - [ ] Standing policy (already practiced): every hardware finding gets
    encoded as a replayed event stream in the Chromium gate (as the ghost-tap
    fix was). Catches regressions of KNOWN failures only.
  - DISFAVORED (Peter, 2026-08-05 ~12 PM EDT: "i hate using headless browser
    drivers if at all possible to avoid"): the Playwright-WebKit second-engine
    spike and the iOS-Simulator-CI-via-safaridriver roadmap are both headless
    driver machinery — keep them as last resorts only, not planned work. The
    existing Chromium CDP gate stays (established, load-bearing, and the
    replayed-stream policy above depends on it); the aversion applies to
    ADDING driver surfaces. Preferred direction instead: hardware-first bless
    with Peter plus replayed-stream regression, and where WebKit-specific
    behavior must be mechanized, prefer in-page harnesses served to a real
    browser over driver-controlled ones.
- [x] Updated pin recommendation sent to vibesteroids_wat: 61f287f (CI green),
  superseding 73891e6; obsolete compressed-.aed caveat retracted; capability
  signal and audio investigation flagged as coming.
  (inbox/2026-08-03-from-aedicule-ghost-tap-fix-pin-and-capability-heads-up.md,
  2026-08-03 17:41 EDT)

- [x] Deleted the superseded `native-titlebar-routing` branch (local +
  origin) with Peter's approval after a content-level audit: its occlusion
  fix (`host_title_bar_layer`), its headless proof test
  (`host_title_bar_occludes_guest_pointer_edges`), and its gpui
  test-support idea all live on yolo since `1de9822`, which extended the
  pattern to control surfaces. Dead /tmp worktree records pruned in the
  same pass. Commit `a10241d` stays reflog-recoverable ~90 days.
  (2026-08-05 15:05 EDT)

- [x] Add the first demoable `spring_sim_aed` pin to the web gallery with
  the same source/commit/hash provenance and snapshot control as the existing
  demos: `985f7657651e6af7ce331c1fddcf97a23b041235`,
  SHA-256 `d722df0e62d39d5e47bbe2942e8881ff2a648180f426c555f1d88acd5603ddfc`.
  Start from a failing manifest/gallery classifier, verify the vendored bytes
  against the sender's deterministic pin pair, run the focused gates and full
  suite, commit green, then send the spring agent the Aedicule commit.
  Curiosity poke: a third demo must not expose an accidental two-demo cardinality
  assumption in snapshots, page layout, or cache metadata. (2026-08-05 17:29 EDT)
  - [x] Verified the original 37,405-byte artifact and SHA-256 independently;
    its packaged `tests/main.wast` passes through current Aedicule, and the
    complete Nix web delivery builds with matching bytes. Added the manifest,
    responsive third gallery card, direct download, all-release copies, guide,
    provenance classifiers, and localized English strings under a red-first
    focused test tranche. That pin was superseded by the control-geometry fix
    below before publication. (2026-08-05 17:47 EDT)
  - [x] Release blocker found by the real Chromium WebGPU gate: all five spring
    buttons deliver without canvas leakage, but none of its seven 24-pixel-high
    above-label slider rows delivers a semantic control event. The same gate
    passes Ulam's 44-pixel slider rows. Sent the spring agent the full evidence
    and requested a guest-owned geometry fix plus replacement pin pair; do not
    publish the original hash. (2026-08-05 17:50 EDT)
    - [x] Replacement received: commit
      `985f7657651e6af7ce331c1fddcf97a23b041235`, 40,506-byte package,
      SHA-256 `d722df0e62d39d5e47bbe2942e8881ff2a648180f426c555f1d88acd5603ddfc`.
      Verify those bytes independently, replace the staged snapshot and
      provenance, rerun the spring WAST and Chromium control gates, then close
      the blocker only on green. Independent package hash and WAST pass green;
      browser gate now reports seven sliders and five buttons delivered with no
      canvas leakage. (2026-08-05 18:35 EDT)
  - [x] Animate the gallery's spring mass horizontally only while its card is
    hovered or keyboard-focused, matching the other two interactive previews
    and respecting `prefers-reduced-motion`. Start with a failing gallery
    classifier and show Peter the actual rebuilt page before committing.
    Curiosity poke: keep the travel short enough that the fixed spring artwork
    still appears connected to the mass throughout the cycle.
    Focused red/green classifiers and complete `./test` suite passed; rebuilt
    Tailscale staging endpoint serves the new animation and replacement package.
    (2026-08-05 18:36 EDT)
  - [x] Keep the staged gallery available for Peter's iPhone visual acceptance
    over the Thelio's Tailscale HTTPS name on port 8911 while the corrected
    spring pin is under acceptance; verify HTTPS reachability before handing off the URL.
    Tailscale Serve now terminates HTTPS at port 8911 and proxies to the
    loopback-only staged Caddy server; verified gallery and spring route return
    200 and the downloaded package retains its pinned SHA-256.
    (2026-08-05 17:57 EDT)

- [ ] Diagnose and fix the intermittent browser startup stall at
  `wasm-initializing` across demos (Peter, 2026-08-05 18:58 EDT); compare public
  Pages and Tailscale staging and preserve the existing multi-tab admission
  guarantee.
  Curiosity poke: a lock holder that never reaches `wasm-initialized` can make
  later tabs look stuck at `startup-lock-waiting`, while memory pressure or a
  rejected `init()` can leave the first tab stuck at `wasm-initializing`.
  - [x] Encode the concrete resource-pressure finding red-first: the generated
    delivery reserved up to 1 GiB of shared Wasm memory per tab. Cap it at
    256 MiB (4,096 pages), then prove the generated glue carries that ceiling.
    Complete `./test` and real Chromium/WebGPU startups for Vibesteroids, Ulam
    Flower, and Spring Simulator passed. (2026-08-05 19:10 EDT)
  - [ ] Obtain Peter's iPhone/WebKit acceptance from the rebuilt Tailscale
    delivery after closing old tabs that retain the 1 GiB runtime. If it still
    stalls, capture whether the last stage is `wasm-initializing` or
    `startup-lock-waiting` before choosing a second mitigation.
