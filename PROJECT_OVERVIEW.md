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

**Proof-of-concept status:** successful. Native rendering, input, generated
audio, deterministic headless SVG, bounded guest execution, snapshots, and
state-preserving live reload all work on the exercised NixOS path. ABI and
cross-platform stability remain pre-production.

**Main branch:** yolo

**i18n phase:** prepare. Frontplane-owned strings are centralized; only English
is populated while the ABI remains experimental.

## Terms

**Frontplane** — the trusted native host: GPUI windowing/rendering, input,
menus, audio, resource policy, scheduling, storage policy, and guest lifecycle.

**Guest/application** — an untrusted WAT-authored module containing the
application brain. It receives no ambient platform authority.

**Command frame** — a finite host-owned immutable list of scene, audio, and
effect requests emitted during a guest lifecycle call.

**Fixed tick** — one deterministic simulation step. The host converts elapsed
monotonic time into integer ticks; the guest never observes wall time.

**State schema** — a guest-owned compatibility identifier for opaque snapshot
bytes. The host also checks length, but only the guest author can determine
whether equal-length state remains semantically compatible.

**Transactional reload** — candidate compilation, validation, initialization,
optional restore, and first render occur before the active guest is replaced.
