# Mecha Aedicule specification

## 1. Purpose and boundary

Mecha Aedicule is a generic native GPUI frontplane that loads applications
authored directly in WebAssembly Text format (WAT). The Rust host owns native
capabilities and resource policy. A guest owns application state, behavior,
rendering intent, menu declarations, audio programs, and effects.

The frontplane repository contains no production guest and no behavioral
oracle for one. It includes only neutral fixtures that test protocol and
adapter behavior. Production guests, WAST scenarios, product research, and
application documentation live in independently versioned repositories.

This boundary has two practical consequences:

1. application-only edits cannot invalidate the large native GPUI/Rust Nix
   derivation; and
2. generic host code cannot quietly absorb one application's business logic.

The provisional package and executable names remain `gpui-wasm`; renaming the
public CLI is a separate pre-v1 compatibility decision.

## 2. Proof-of-concept success criteria

The POC is successful when:

1. an external WAT module compiles at launch and drives a native GPUI window;
2. the host remains generic and exposes only a small versioned capability ABI;
3. guest state, deterministic behavior, rendering intent, menus, audio
   declarations, and application effects remain guest-owned;
4. the same completed scene can render through GPUI or headless SVG;
5. compatible guest edits can replace live code without losing state;
6. malformed or over-budget guests fail without compromising the host or last
   working application;
7. native and headless behavior are covered by deterministic tests; and
8. Nix builds the frontplane independently from mutable downstream WAT.

These criteria are met on the exercised x86_64 NixOS path. ABI stability,
cross-platform parity, rich semantic widgets, and production hostile-code
assurance are not yet claimed.

## 3. Architecture

~~~text
downstream code.wat
       |
       v
wat parser -> Wasmtime instance <-> bounded aedicule.v0 imports
                   |                         |
             opaque snapshot           command frame
                   |                   /      |      \
          transactional reload       GPUI    SVG    audio/effects
~~~

The reusable Rust core is independent of GPUI. It compiles and validates
guests, owns Wasmtime stores, enforces budgets, captures immutable command
frames, snapshots state, and performs transactional candidate replacement.

Adapters provide:

- a full-window GPUI canvas and native lifecycle;
- physical input and standard menu actions;
- fixed-step scheduling from a monotonic clock;
- generated audio at the device boundary; and
- deterministic SVG serialization for headless inspection.

No Wasmtime import may re-enter a live GPUI `Context` or `Window` borrow.

## 4. Trust boundary

ABI v0 grants no WASI imports. A guest has no ambient filesystem, network,
process, environment, or wall-clock access. Any future dangerous capability
must require both explicit declaration and host policy.

Each guest call is bounded by:

- Wasmtime fuel;
- maximum linear memory and table elements;
- pointer, length, UTF-8, number, and lifecycle validation;
- scene-command, stack-depth, resource-byte, decoded-pixel, audio, effect, and
  log budgets; and
- transactional output: partial commands are discarded after a trap or
  validation failure.

Containment is defense in depth, not proof that arbitrary hostile WAT is safe
for production. Wasmtime epoch interruption and asynchronous compilation remain
candidate hardening work.

## 5. ABI principles

The namespace is `aedicule.v0`; every WAT-facing function is named `AE_*`. All
boundary `i32` values are signed in Wasm but
IDs, masks, colors, pointers, and lengths are interpreted as documented
unsigned bit patterns after validation.

- The ABI is numeric and WAT-friendly.
- Every import returns zero on acceptance or a negative typed host error.
- A lifecycle export returns zero on success and propagates required-command
  failures.
- Guest calls emit output into host-owned buffers; GPUI consumes immutable
  completed output afterward.
- Coordinates are logical pixels. Colors are non-premultiplied `0xRRGGBBAA`.
- Rendering does not advance simulation state.
- Stable IDs are guest-scoped and unique within their required scope.
- Unsupported flags and capabilities fail closed.

## 6. Module contract: Aedicule v0

[WAT_ABI.md](WAT_ABI.md) is the single generated client reference. It is built
from the host's declarative ABI table and must match it byte-for-byte during
every build. The summary below deliberately defers import signatures and the
minimal WAT module to that canonical document.

### 6.1 Guest exports

~~~text
memory
AE_abi_major() -> i32
AE_abi_minor() -> i32
AE_configure() -> i32
AE_init(seed_lo: i32, seed_hi: i32,
        viewport_w: f32, viewport_h: f32) -> i32
AE_event(kind: i32, code: i32, a: f32, b: f32) -> i32
AE_tick(ticks: i32) -> i32
AE_tick_rate(current_numerator: i32, current_denominator: i32)
  -> (numerator: i32, denominator: i32) optional; (0, 0) follows display
AE_render() -> i32
AE_state_ptr() -> i32
AE_state_len() -> i32
AE_state_schema() -> i32
AE_after_restore() -> i32           optional in ABI minor 0
~~~

`AE_abi_major` must return 0. A declared rational tick rate must be in
`1..=1000` Hz with a denominator no greater than 1,000,000; a wrong signature
or out-of-range result rejects the guest before initialization. The host
snapshots exactly `AE_state_len` bytes at `AE_state_ptr`.

Restoration is allowed only when schema and byte length match. Guest authors
must increment the schema when an equal-length layout changes meaning. The
optional `AE_after_restore` rebuilds derived caches after bytes are installed.

### 6.2 Metadata and audio imports

The generated ABI reference contains the exact `AE_title`, `AE_menu_item`,
`AE_synth_voice`, `AE_sample_asset`, and `AE_sample_play` declarations,
including bounded flag and scalar meanings.

Audio program IDs are application-owned. Waveforms currently include sine,
sawtooth, white noise, and brown noise. Integer millihertz and parts-per-million
fields keep synthesis declarations portable. Oscillator phase, envelopes,
filters, noise shaping, and mixing use signed decimal fixed point; conversion
to `f32` PCM occurs only at the audio-device adapter.

Sample IDs occupy a namespace separate from synth program IDs. During
`AE_configure`, `AE_sample_asset` may bind a unique ID only to an admitted
`.flac` below the application's immutable `assets/` virtual root. Bare WAT,
directory, and `.aed` launch resolve identical catalogs; declarations are
re-resolved for a transactional reload. Encoded bytes, decoded frames,
channels, sample rate, and aggregate decoded storage are bounded before native
playback. `AE_sample_play` is a one-way transactional output with finite volume
in `0..=1`, pitch in `0.25..=4`, and version-0 flags equal to zero.

### 6.3 Scene imports

The generated ABI reference contains the complete supported `AE_frame_*`,
`AE_transform_*`, `AE_path_*`, `AE_image_*`, `AE_sprite`, `AE_line`,
`AE_circle`, and `AE_text` declarations. It does not promise rectangles,
clipping, or layers, which are future protocol work.

`AE_frame_begin` is first and `AE_frame_end` last. There is at most one completed
frame per `AE_render`. Transform and path stacks must balance.
Underflow, overflow, non-finite or out-of-policy geometry, incomplete paths,
duplicate IDs, and unsupported flags reject the complete frame.

Affine transforms provide nesting, rotation, scale, translation, and
mirroring. Paths provide arbitrary vector art. Images are bounded,
content-hashed, decoded once, and cached under guest-scoped IDs. Animation is
guest state: a guest selects source rectangles, transforms, and opacity from
deterministic ticks. The renderer owns no application clock.

The implemented `AE_ui_*` transaction is the keyed retained kernel of the
proposed Aedicule View Protocol: a guest can atomically publish absolute native
panels, integer sliders, and action buttons today. It is not yet a general
semantic tree. Containers/layout, dynamic text, text inputs, toggles, lists,
dialogs, focus traversal, accessibility, event-driven lifecycle, and browser
parity follow the platform-neutral design in `VIEW_PROTOCOL.md` rather than
painted imitations in the scene stream.

### 6.4 Events

~~~text
1  key-down       code=stable physical-key ID
2  key-up         code=stable physical-key ID
3  pointer-move   a=x, b=y
4  pointer-down   code=button, a=x, b=y
5  pointer-up     code=button, a=x, b=y
6  viewport       a=width, b=height
7  menu-action    code=guest-declared action ID
8  focus          code=0 lost, 1 gained
9  AE_EVENT_DISPLAY_REFRESH
                 code=refresh_numerator_hz, a=refresh_denominator, b=0
~~~

The host suppresses duplicate held key-down notifications uniformly. It does
not infer application meaning or repeat policy. A guest maintains held-input
state and clears it on focus loss.

Standard menu IDs 1–1023 are reserved: New=1, Open=2, Save=3, Preferences=4,
About=5, Quit=6, and Help/Controls=7. Application-specific IDs begin at 1024.
Quit is host-owned and may require native confirmation by policy.

### 6.5 Effects and diagnostics

~~~text
AE_audio(id, volume, pitch, flags) -> i32
AE_effect(kind, a, b) -> i32
AE_log(level, ptr, len) -> i32

effect 1  request-redraw
effect 2  request-quit
effect 3  set-cursor
effect 4  persist-snapshot
effect 5  open-url
~~~

Dangerous effects such as `open-url` require explicit capability and host
confirmation.

Host status codes are:

~~~text
 0  accepted
-1  generic rejection
-2  budget exhausted
-3  invalid pointer or length
-4  invalid UTF-8
-5  invalid or non-finite number
-6  invalid lifecycle order
-7  duplicate command or action ID
-8  unsupported flag, action, or capability
~~~

Guest export errors reserve -1000 through -1999. Traps remain distinct.

## 7. Determinism and scheduling

- Simulation advances only through integer `AE_tick` counts.
- The seed is the sole randomness input; all derived values enter snapshots.
- Snapshot, N ticks, restore, and the same N ticks must yield identical state
  bytes and command frames.
- Rendering is pure with respect to guest simulation state.
- Audio commands are deterministic outputs but are not replayed merely by
  restoring an old snapshot.

The native scheduler accumulates monotonic elapsed nanoseconds as the exact
rational `elapsed_ns * tick_rate_numerator`, carries the remainder modulo one
billion times the rate denominator,
advances all due fixed ticks, and renders once. Each GPUI wake is a freshly
computed one-shot delay to the next ceil-rounded absolute boundary; timer jitter
never becomes the origin of the following tick. The host does not loop on
truncated `Duration::from_nanos(1e9 / hz)`, which introduces fractional-period
drift and adds work duration to the clock.

Native events receive monotonic timestamps on arrival. An event at or before a
tick boundary is delivered before that tick; an event after the boundary cannot
retroactively affect it. A late wake batches adjacent event-free ticks but
splits guest calls at event boundaries, preserving input ordering without one
Wasmtime call per tick.

Catch-up is bounded. The oldest whole excess ticks are dropped and counted in
the visible runtime status rather than retained as debt that can cause a spiral
of death. Events timestamped inside dropped time are still delivered before the
first surviving tick. The clock, elapsed samples, event times, and catch-up
policy are injected for deterministic testing.

Applications should store dimensional quantities in canonical units—such as
logical pixels per second—so a tick-rate change is a policy edit rather than a
rewrite of every velocity constant. Any cross-rate equivalence tolerances and
application-specific mechanics belong to that application's WAST suite.

The display refresh is host input, not a guest query. The initial known display
mode and each changed mode are delivered once as `AE_EVENT_DISPLAY_REFRESH`;
the guest then optionally selects an exact rational simulation rate via
`AE_tick_rate`. The selector sees the prior agreed simulation rate, never the
display mode directly. `(0, 0)` follows the newly delivered display mode.

Until GPUI exposes native display-mode observation, the host may receive an
exact process-start override through `AE_DISPLAY_REFRESH_RATE`. Fractions such
as `60000/1001` are normalized by GCF. The canonical nominal spellings
`23.976`, `29.97`, `59.94`, and `119.88` select their exact
1000/1001-family rationals; every other decimal is parsed exactly as written,
without floating point. Invalid values fail startup. This configuration only
sets the initial host event and never creates a guest display-query capability.

## 8. Transactional loading and reload

Startup resolves, in order, an explicit file or application directory,
`./code.wat`, a packaged downstream default supplied by an environment
contract, and the stable embedded conformance fallback.

Watching compares content identity at the configured path, so in-place writes
and atomic rename saves are both observed. A candidate must:

1. parse, compile, instantiate, and validate under policy;
2. configure and initialize successfully;
3. restore the active snapshot only when schema and length match;
4. run `AE_after_restore` when exported; and
5. emit a valid complete first frame.

Only then is it swapped into the active slot. Every failure leaves the active
module, state, and UI operational and exposes a recoverable diagnostic.

## 9. Testing strategy

Rust tests stop at the frontplane boundary:

- ABI negotiation, lifecycle, import and output validation;
- set-based hostile input and resource-limit classification;
- deterministic clocks, snapshots, and replay;
- transactional candidate success, compatible restore, schema restart, and
  rollback;
- pure scene-to-SVG and GPUI paint-model adaptation;
- generic physical input, menu, CLI, and launch resolution;
- generic audio declaration and integer synthesis; and
- Nix source-closure and repository-boundary classifiers.

Neutral WAT fixtures may exercise these host behaviors, but may not implement a
production application's rules. Downstream behavior belongs in standard WAST
files executed by stock Wasmtime, with an instrumented `aedicule.v0` module when
output assertions are required.

Visual adapters are pure functions wherever practical. Deterministic SVG gives
agents and CI an inspectable artifact; Peter verifies uncertain native visual
appearance before it becomes a frozen expectation.

The canonical commands are:

~~~text
./test
./build
./run [--watch] [path]
~~~

## 10. Nix and downstream composition

`packages.frontplane` is the optimized native derivation and
`packages.default` is its alias. Its filtered source includes Cargo metadata,
`build.rs`, `WAT_ABI.md`, `src/`, and `third_party/`; the ABI reference is
included because the build rejects documentation drift. Other documentation or
downstream WAT changes cannot alter its derivation path. `checks.frontplane`
builds the same artifact.

A downstream application should pin this flake as an input, build its WAT as a
small data derivation, and expose a wrapper that sets the packaged-default
application path before invoking the frontplane. It should run its WAST suite
in its own check derivation. A local input override supports adjacent
development without weakening the committed CI pin.

Binary-cache transport is orthogonal: Mechatron Prime may publish the native
frontplane closure to the private Attic cache so game-only CI jobs substitute
it rather than rebuild it.

## 11. i18n prepare phase

Frontplane-owned fallback titles, standard menus, status, errors, and recovery
copy come from a typed English catalog. Guest labels are UTF-8 presentation
strings in v0. A future ABI may register stable string keys with English
fallbacks so the host can localize standard actions without changing guest
state. No translation corpus is required while the protocol is experimental.

## 12. Non-goals for v0

- a general ECS or game engine;
- application-specific native APIs or behavioral oracles;
- arbitrary WASI authority;
- online services or multiplayer facilities;
- mobile-native GPUI support;
- a complete Aedicule View Protocol implementation beyond the current narrow
  native-controls profile;
- automatic state migration across incompatible schemas;
- complete localization; or
- production security or cross-platform compatibility claims.

## 13. Next falsification tests

1. Implement a non-game application, preferably a small editor, without adding
   its business logic to Rust.
2. Implement the staged retained semantic components, text input, focus, and
   accessibility contract specified in `VIEW_PROTOCOL.md`.
3. Move candidate compilation off the UI thread while proving deterministic
   replacement ordering.
4. Exercise macOS and aarch64 Linux with the same guest and headless artifacts.
5. Profile immediate host imports before considering a packed command buffer.
6. Decide a generated-binding or macro layer without making hand-authored WAT
   opaque.
7. Version the protocol independently before declaring ABI v1.
