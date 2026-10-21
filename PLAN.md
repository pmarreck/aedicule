# PLAN

Purpose: [INTENT.md](INTENT.md). Completed work: [log](docs/PLAN_LOG.md). Historical constraints and reconciliation: [context](docs/plan_context/reconciliation.md).

## Current cleanup and delivery

- [ ] Audit previously published private operational references, coordinate downstream immutable-pin impact before any public history rewrite, and preserve authorized attribution.
- [ ] Await the requested downstream Ulam render-runner status and real-checkout evidence for isolated concurrent cases with deterministic failure replay (context: docs/plan_context/reconciliation.md).
- [ ] Narrow Nix source closures for web runtime, delivery packaging, and script checks; prove each input classifier with relevant and irrelevant mutations (context: docs/plan_context/backlog-evidence.md).
- [ ] Split the isolated Nix check into reusable feature/script derivations without weakening exact-revision coverage.
- [ ] Reconcile current demo release pins and packaged-guide coverage with downstream owners; independently verify source/package bytes and WAST before promotion.
- [ ] Update the Pages action versions after checking current upstream compatibility; retain exact-artifact deployment gates.

## Browser acceptance and regressions

- [ ] Recheck the Firefox simultaneous-tab freeze with sustained foreground/background responsiveness and a fresh profile; keep startup lock evidence separate from a proven fix.
- [ ] Confirm synthesized and sampled Firefox audio with trusted activation and physical output; retain independent PCM/graph tests.
- [ ] Verify iOS text-entry focus raises the keyboard from a trusted gesture while non-text controls never do.
- [ ] Obtain explicit iOS acceptance for selection/callout suppression and repeated cached reloads; August touch/shake acceptance does not prove every browser scenario.
- [ ] Obtain visual acceptance of the gallery About page, file dropzone, and hover/focus/reduced-motion previews.
- [ ] Prefer real-browser in-page checks and replayed event regressions; add new WebKit/Safari driver machinery only after revisiting the recorded last-resort policy.
- [ ] Reconcile stale HTML cache policy with the delivered content-addressed module graph; do not reintroduce the retired isolation service worker.
- [ ] Update VIEW_PROTOCOL.md to describe the delivered text field and its remaining guest-value/reload limitation.
- [ ] Add guest-authoritative text-field values and accepted-revision reconciliation so guests can set, clear, and restore edit buffers.

## Guest lifecycle and input

- [ ] Watch explicitly loaded native WAT, directories, and AED packages by default, with an explicit --no-watch override.
- [ ] Make --package replace its destination atomically through a complete same-filesystem temporary file.
- [ ] Show bounded, click-through reload-error feedback while preserving stderr diagnostics and the last accepted guest/state/frame.
- [ ] Prove failed candidate event/tick/render work cannot mutate the active guest and test rename/content-generation reload races.
- [ ] Add ordered general --event injection to aedicule-render, prove menu-driven output, and cover cross-primitive ID rejection and next-frame reuse.
- [ ] Obtain downstream 60/120 Hz equal-elapsed-time WAST proof and live acceptance without changing guest-owned simulation semantics.
- [ ] Add a portable native refresh observation source that emits changes once through the existing rational-rate negotiation.
- [ ] Revisit GPUI_REFRESH_API_PROPOSAL.md before any upstream contribution or presentation-policy implementation.
- [ ] Specify richer input-capability changes for concurrent modalities, hot-plug, and reload; retain the existing kind-6 coarse-pointer flag.
- [ ] Add bounded safe-area inset change events independently of viewport dimensions.
- [ ] Extend semantic suspension triggers to guest-declared regions/actions while keeping resume observable with guest ticks stopped.
- [ ] Add a coarse-pointer boot-event browser regression if the existing device-change tests leave an adapter gap.

## Application UI and delivery

- [ ] Extract the adapter-neutral AVP document with an aggregate node budget and permit view-only transactions/event-driven guests (context: VIEW_PROTOCOL.md).
- [ ] Deliver the general forms slice across native, browser, and headless adapters before claiming general-application completeness.
- [ ] Exercise AVP with the separately owned printable-binary editor and user-mediated file capabilities (context: PRINTABLE_BINARY_EDIT_PROPOSAL.md).
- [ ] Specify asynchronous services, persistent storage, headless CLI/TUI profiles, and pre-instantiation Wasm-GC policy in bounded slices (context: VIEW_PROTOCOL.md).
- [ ] Implement --scaffold with a runnable guest, configuration, tests/main.wast, documented asset/lib roots, and no overwrite or invented license.
- [ ] Implement --appify/--deappify with exact constituent recovery and a build manifest; separate deterministic unsigned content from signing.
- [ ] Add package identity, runner/appified menu policy, hierarchical guest menus, and typed semantic accelerators with an unshadowable host recovery floor.
- [ ] Show a native title-bar hamburger only for actionable guest menus, preserving separator order, bounded overflow, focus, and pointer occlusion.
- [ ] Implement the accepted native fullscreen contract, safety notice, and shortcut precedence; browsers retain their own fullscreen controls (context: docs/plan_context/backlog-evidence.md).
- [ ] Obtain the credentialed native-macOS signing/notarization lane before describing public archives as Gatekeeper-clean; verify nested code and final bundles.
- [ ] Evaluate persistent CI dependency caching after measuring cold compilation separately from warm test execution.

## Asset and client proposals

- [ ] Specify bounded packaged PNG admission and an integer/fixed-point draw contract for source rectangles, pivot, rotation, scale, flip, opacity, and sampling before implementation.
- [ ] Keep sprite animation guest-owned and prove PNG native/browser/headless parity, alpha/clipping behavior, and compressed/decoded/draw budgets.
- [ ] Specify package font IDs, byte limits, and collision-safe family identity; bundled Geist Mono already works.
- [ ] Retain the guest-owned magenta/green anaglyph viewer/game proposal; verify filter characteristics before choosing projection/color matrices.
- [ ] Add orientation, compass, geolocation, or gamepad capabilities only when a concrete client needs a bounded profile.

## Recent accepted work

- [x] Reconcile seven inbox notes, retire 66 processed envelopes, consolidate intent/terminology, and reduce the plan after full tests and an optimized build passed (done 2026-10-21 17:01 EDT).
- [x] Remove private publisher defaults; prove no unconfigured SSH probe and preserve SSH-configured users (done 2026-10-21 17:01 EDT; d845fb4).
- [x] Public iPhone touch and shake physically accepted on host e17562c; downstream guest 12103b6 reports matching green pin acceptance (done 2026-08-26 10:12 EDT; downstream report 2026-08-27).
- [x] Pin native/cross Fenix Rust 1.97.1 without changing the 1.97 MSRV or browser nightly (done 2026-08-27 16:46 EDT; 59432d0).
