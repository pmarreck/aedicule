# Aedicule intent

Aedicule is a cross-platform application runner for programs authored in
WebAssembly Text (WAT), by people working alone or with coding agents. A trusted
Rust/GPUI frontplane supplies bounded platform capabilities; an untrusted guest
owns application state, domain rules, rendering intent, menus, and audio requests.

## Desired outcomes

- Independently versioned guests run on web, macOS/aarch64, Linux/aarch64,
  Linux/x86_64, Windows/aarch64, and Windows/x86_64.
- WAT edits can be tested and transactionally reloaded without rebuilding the
  native Rust/GPUI dependency graph. Failed edits retain the last accepted guest.
- Bare WAT, project directories, and portable `.aed` packages share one bounded
  application-root model, with deterministic testing and asset admission.
- The host supports general applications as well as games and graphical demos.
  Guest-owned UI documents and explicit services should make that possible
  without putting application-specific policy in the host.
- A source-independent, versioned [guide](GUIDE_FOR_LLMS.md) and generated
  [ABI reference](WAT_ABI.md) distinguish callable capabilities from proposals.

## Boundaries

Guests receive no ambient filesystem, network, process, clock, or entropy access.
New capabilities require explicit contracts and budgets. Deterministic seeded
randomization is an admitted capability, not ambient entropy. The host validates
memory accesses, command counts, resource sizes, numeric values, and guest work.

Application behavior and its WAST oracles belong in separate client repositories.
Host tests use neutral conformance fixtures; pinned demonstration packages are
delivery inputs. Application-only changes must not rebuild the native frontplane.
The six delivery targets are an intended capability, not a claim that every
adapter or platform has identical acceptance coverage.

Appification is a proposed delivery layer combining an exact package and runtime;
it does not replace portable packages. General forms, accessibility, file/storage
services, and broader hardware access remain staged work in
[VIEW_PROTOCOL.md](VIEW_PROTOCOL.md), not implied by the current small control set.

## Verification and current limits

The project is a working proof of concept with a pre-production ABI. Exercised
capabilities include native and browser scenes, timestamped input, exact rational
ticks, synthesized/FLAC audio, bundled Geist Mono text, deterministic headless
SVG, snapshots, and transactional reload. Independent guests include Vibesteroids,
Ulam Flower, and Spring Simulator; an unrelated editor/forms client remains a
stronger test of general-application scope.

`./test` is the required suite and `./build` the optimized native build.
`./test_browser` preserves the explicitly separate real-browser acceptance lane;
`./build_all` and `./check_reproducible` verify delivery and reproducibility.
Hardware acceptance remains necessary for sound, touch feel, and platform UI
behavior that automated checks do not establish. Public macOS signing and
notarization remain a separate distribution gate.

The i18n phase is prepare: frontplane strings are centralized, with English
populated while the UI/ABI is experimental. See [docs/I18N.md](docs/I18N.md).

These intentions consolidate the former project overview and the accepted
cross-platform/general-application direction. [RULES.md](RULES.md) states the
engineering invariants, [TERMINOLOGY.md](TERMINOLOGY.md) defines project terms,
and [PLAN.md](PLAN.md) orders remaining work.
