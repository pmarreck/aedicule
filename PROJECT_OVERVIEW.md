# Mecha Aedicule

Mecha Aedicule is a generic native GPUI frontplane for applications authored
directly in WebAssembly Text format (WAT). The trusted Rust host owns platform
integration and bounded capabilities. An untrusted guest owns application
state, rules, rendering intent, menus, audio declarations, and effects.

**Project identifier:** `aedicule`

The project's goal is a stable native appliance into which independently
versioned WAT applications can be loaded, tested, watched, and transactionally
replaced. Application-only edits must not rebuild the GPUI/Rust dependency
graph, and no specific application's rules or behavioral oracle may leak into
this repository.

The first substantial application is the separate `vibesteroids_wat`
repository. A second non-game application is required before claiming that the
v0 capability model is broadly reusable.

**Proof-of-concept status:** successful. Native rendering, timestamped input,
guest-rate absolute-deadline scheduling, generated audio, deterministic
headless SVG, bounded guest execution, snapshots, and state-preserving live
reload all work on the exercised NixOS path. ABI and cross-platform stability
remain pre-production.

**Main branch:** yolo

**i18n phase:** prepare. Frontplane-owned strings are centralized; only English
is populated while the ABI remains experimental. The canonical 50-locale
baseline, RTL set, selection rationale, and browser locale behavior are recorded
in `docs/I18N.md`.

## Terms

**Frontplane** — the trusted native host: GPUI windowing/rendering, input,
menus, audio, resource policy, scheduling, storage policy, and guest lifecycle.

**Guest/application** — an untrusted WAT-authored module containing the
application brain. It receives no ambient platform authority.

**Command frame** — a finite host-owned immutable list of scene, audio, and
effect requests emitted during a guest lifecycle call.

**Aedicule View Protocol (AVP)** — the proposed platform-neutral semantic UI
plane. Its immutable keyed unit is a **view document**. The implemented
`AE_ui_*` panels/sliders/buttons prove the transactional kernel as a narrow v0
native-controls profile; hierarchy, general widgets, accessibility, browser
parity, and application services remain specified work. See
`VIEW_PROTOCOL.md`.

**Fixed tick** — one deterministic simulation step. The host converts elapsed
monotonic time into integer ticks against absolute rational boundaries; the
guest never observes wall time. Display mode is instead an explicit `AE_event`
that can trigger the guest's optional rational-rate policy.

**Display observation limit** — the core delivers and tests the exact
display-refresh event contract, but GPUI's public display object does not yet
provide a refresh mode. The native adapter therefore uses deterministic 60/1
Hz at startup until GPUI exposes a portable change notification or an explicit
platform adapter is added. For development and deterministic testing,
`AE_DISPLAY_REFRESH_RATE` supplies an exact process-start override; it accepts
fractions such as `60000/1001`, canonical `59.94`-style aliases, and otherwise
exactly parsed decimal input.

**State schema** — a guest-owned compatibility identifier for opaque snapshot
bytes. The host also checks length, but only the guest author can determine
whether equal-length state remains semantically compatible.

**Transactional reload** — candidate compilation, validation, initialization,
optional restore, and first render occur before the active guest is replaced.
