# Generic GPUI Frontplane for WAT Applications

## 1. Purpose

Prove or falsify this architecture:

> A single generic GPUI frontplane can load a WAT-authored WASM application,
> give it bounded platform capabilities, and render an interactive application
> without application-specific Rust.

The demonstrator is an Asteroids-style game because it exercises more than a
form or static widget gallery:

- continuous keyboard input;
- deterministic simulation and game state;
- collision and object lifecycle;
- custom vector rendering;
- HUD text;
- native menu actions;
- audio events;
- state snapshot/restore;
- restart and quit effects;
- frame/tick pacing; and
- hostile-plugin containment.

The spike succeeds only if the host remains generic. A beautiful Asteroids game
implemented partly in Rust would fail the architectural experiment.

### 1.1 Vibesteroids conversion source

The demo is a behavioral conversion of Peter Marreck's original MIT-licensed
[Vibesteroids](../vibesteroids/README.md), not an unrelated Asteroids clone.
The source repository remains unchanged and acts as an independent reference
implementation. The first WAT milestone preserves its recognizable core:

- deterministic seeded simulation;
- rotation, thrust, friction, firing, and screen wrapping;
- asteroid motion and bullet collisions;
- score, lives, pause, restart, and level progression;
- semantic shoot, thrust, and explosion sounds; and
- the VIBESTEROIDS title and authorship.

Kid Mode, Death Blossom, touch/shake controls, split asteroids, particles,
safe respawn, and its full generated-sound palette are explicit conversion
backlog, not host features. Adding them must require only WAT changes or a
genuinely generic ABI revision. This is a useful falsification test: if a
Vibesteroids feature forces application-specific Rust into the frontplane, the
ABI boundary is wrong.

## 2. Done criteria

The spike is complete when:

1. ./test runs a clean deterministic headless suite.
2. ./build produces an optimized native frontplane composed with the canonical
   Asteroids WAT as a separately replaceable Nix data artifact.
3. The native window uses GPUI and gpui-component.
4. The WAT module owns all Asteroids state and gameplay rules.
5. Keyboard input reaches the plugin through the generic event ABI.
6. The plugin emits bounded rendering commands that the host paints.
7. New Game is routed through a generic menu/action mechanism.
8. Quit is requested through a generic host effect and handled by the
   frontplane.
9. Fire/explosion events request audible host-generated sounds.
10. Host snapshots can restore the exact deterministic game state.
11. Fuel, memory, command, pointer, string, and numeric validation failures are
    tested and reported without terminating the host process.
12. SPEC.md records any portability or API obstacles discovered.

## 3. Architecture

~~~text
                      vibesteroids.wat
                             │
                        wat → wasm
                             │
                  ┌──────────▼──────────┐
                  │ Frontplane core     │
                  │ Wasmtime lifecycle  │
                  │ ABI validation      │
                  │ command buffers     │
                  │ snapshot/recovery   │
                  └──────────┬──────────┘
                             │ immutable FrameOutput
                  ┌──────────▼──────────┐
                  │ GPUI adapter        │
                  │ components/canvas   │
                  │ keyboard/menu       │
                  │ audio/window        │
                  └─────────────────────┘
~~~

The core has no GPUI dependency. It accepts adapters for module bytes, limits,
and host effects and returns plain values. The GPUI layer consumes those
values. Headless tests exercise the exact core used by the GUI.

## 4. Trust boundary

The WAT application is untrusted even when generated locally. The host:

- enables no WASI imports;
- provides only the imports in this specification;
- rejects unknown imports/exports;
- limits linear memory and tables;
- limits execution fuel per configure/init/event/tick/render call;
- places a maximum on commands, UTF-8 bytes, and audio/effect events;
- validates every guest pointer/length before reading memory;
- rejects NaN and infinite coordinates, sizes, colors, or audio values;
- resets per-call command buffers before entry;
- rolls back incomplete frame, audio, effect, and configuration output after a
  failed call;
- preserves the last known-good snapshot outside guest memory; and
- converts failures into typed FrontplaneError values.

Fuel and memory safety do not prove gameplay correctness. Deterministic replay,
state invariants, and an independent host-side collision/reference scenario
provide the initial mechanically falsifiable controls.

## 5. ABI principles

1. **Versioned:** module and host agree on major/minor ABI versions.
2. **Small:** lifecycle plus generic events, draw commands, audio, effects, and
   state bytes.
3. **Not GPUI-shaped:** no GPUI entity, context, element, callback, or native
   handle crosses the boundary.
4. **Push rendering:** render invokes host imports that append commands.
5. **Retained-ready:** commands contain stable IDs so a later ABI can send
   create/update/remove patches without changing application semantics.
6. **Integer actions:** menu/input/effect IDs are stable numeric values;
   localized labels remain presentation data.
7. **Deterministic:** no wall clock or ambient randomness is imported.
8. **Explicit failure:** zero means success; negative status codes are
   standardized plugin failures.
9. **Bounded:** every variable-size transfer carries and obeys a limit.
10. **WIT-shaped:** although the spike uses a core WASM ABI for WAT simplicity,
    concepts map directly to a future Component Model/WIT interface.

## 6. Core module contract: gpui-frontplane-v0

The namespace is host.v0. All i32 values are signed at the Wasm boundary but
IDs, colors, masks, pointers, and lengths are interpreted as their documented
unsigned bit patterns after validation.

### 6.1 Required exports

~~~text
memory                         linear memory
fp_abi_major() -> i32          must return 0
fp_abi_minor() -> i32          current module minor version
fp_configure() -> i32          declare title, menus, resources, and synth voices
fp_init(seed_lo, seed_hi,
        viewport_w, viewport_h) -> i32
fp_event(kind, code, a, b) -> i32
fp_tick(ticks) -> i32          advance fixed simulation ticks
fp_tick_hz() -> i32            optional preferred fixed rate (1..=1000)
fp_render() -> i32             emit one complete command frame
fp_state_ptr() -> i32
fp_state_len() -> i32
fp_state_schema() -> i32
~~~

viewport_w/viewport_h and event a/b use f32 where declared by the actual Wasm
signature. ticks is an i32 count and is bounded by host policy. A missing
`fp_tick_hz` preserves ABI-v0 behavior at 60 Hz; an exported zero, negative,
greater-than-1000 value, or wrong signature rejects the plugin before init.

The Vibesteroids application converts viewport scalars to signed decimal
millionths exactly once on ingress and converts completed draw scalars exactly
once on egress. A mechanically checked adapter is the only region containing
floating-point arithmetic; snapshots and gameplay calculations contain no
IEEE-754 state.

The host snapshots exactly state_len bytes beginning at state_ptr. Restore
writes a validated snapshot back to that region only when schema and length
match, then invokes:

~~~text
fp_after_restore() -> i32
~~~

This callback is optional in ABI minor version 0 and required once derived
caches exist.

### 6.2 Required host imports

Lifecycle metadata:

~~~text
host.v0.title(ptr: i32, len: i32) -> i32
host.v0.menu_item(id: i32, label_ptr: i32, label_len: i32,
                  shortcut: i32, flags: i32) -> i32
host.v0.synth_voice(program_id: i32, waveform: i32,
                    delay_ms: i32, duration_ms: i32,
                    frequency_start_millihz: i32,
                    frequency_mid_millihz: i32,
                    frequency_end_millihz: i32,
                    gain_start_ppm: i32, gain_peak_ppm: i32,
                    gain_end_ppm: i32, filter: i32,
                    filter_start_millihz: i32,
                    filter_end_millihz: i32,
                    cooldown_ms: i32) -> i32
~~~

Frame:

~~~text
host.v0.frame_begin(r: f32, g: f32, b: f32, a: f32) -> i32
host.v0.line(id: i32, x1: f32, y1: f32, x2: f32, y2: f32,
             width: f32, rgba: i32) -> i32
host.v0.circle(id: i32, x: f32, y: f32, radius: f32,
               width: f32, rgba: i32, flags: i32) -> i32
host.v0.text(id: i32, ptr: i32, len: i32, x: f32, y: f32,
             size: f32, rgba: i32, flags: i32) -> i32
host.v0.frame_end() -> i32
~~~

Scene composition and resources are designed as the next v0 minor capability,
not as Asteroids-specific calls:

~~~text
host.v0.capability(id: i32) -> i32

host.v0.transform_push(m11: f32, m12: f32, m21: f32, m22: f32,
                       tx: f32, ty: f32) -> i32
host.v0.transform_pop() -> i32
host.v0.clip_rect_push(x: f32, y: f32, w: f32, h: f32) -> i32
host.v0.clip_pop() -> i32
host.v0.layer_push(opacity: f32, blend: i32, flags: i32) -> i32
host.v0.layer_pop() -> i32

host.v0.rect(id: i32, x: f32, y: f32, w: f32, h: f32,
             radius: f32, width: f32, rgba: i32, flags: i32) -> i32
host.v0.path_begin(id: i32) -> i32
host.v0.path_move(x: f32, y: f32) -> i32
host.v0.path_line(x: f32, y: f32) -> i32
host.v0.path_quad(cx: f32, cy: f32, x: f32, y: f32) -> i32
host.v0.path_cubic(c1x: f32, c1y: f32, c2x: f32, c2y: f32,
                   x: f32, y: f32) -> i32
host.v0.path_close() -> i32
host.v0.path_end(width: f32, fill_rgba: i32,
                 stroke_rgba: i32, flags: i32) -> i32

host.v0.image_define(id: i32, ptr: i32, len: i32, flags: i32) -> i32
host.v0.image_release(id: i32) -> i32
host.v0.sprite(id: i32, image_id: i32,
               src_x: f32, src_y: f32, src_w: f32, src_h: f32,
               dst_x: f32, dst_y: f32, dst_w: f32, dst_h: f32,
               pivot_x: f32, pivot_y: f32, rgba: i32, flags: i32) -> i32
~~~

Encoded images are bounded, content-hashed, decoded once by the host, and
cached under a plugin-scoped resource ID. Raw RGBA is an explicit flag with
validated dimensions; SVG is excluded until its parser and external-resource
policy are specified. A sprite-sheet animation is simply repeated sprite calls
with changing source rectangles. Rotation, scaling, translation, mirroring,
and parent/child composition come from the affine transform stack, so the ABI
does not need special "rotatable sprite" variants.

Effects:

~~~text
host.v0.audio(id: i32, volume: f32, pitch: f32, flags: i32) -> i32
host.v0.effect(kind: i32, a: i32, b: i32) -> i32
host.v0.log(level: i32, ptr: i32, len: i32) -> i32
~~~

Every import returns zero when accepted or a negative host error. The plugin
must propagate a required-command failure from its current export.

### 6.3 Generic event kinds

~~~text
1  key-down       code = frontplane key ID; a/b unused
2  key-up         code = frontplane key ID; a/b unused
3  pointer-move   a=x, b=y
4  pointer-down   code=button, a=x, b=y
5  pointer-up     code=button, a=x, b=y
6  viewport       a=width, b=height
7  menu-action    code=plugin-declared action ID
8  focus          code=0 lost, 1 gained
~~~

The Asteroids mapping is:

~~~text
Left/Right      rotate
Up              thrust
Space           fire
P/Escape        pause
F               toggle auto-fire
K               toggle Kid Mode
B               activate Death Blossom
H/F1            Help / Controls
R               new game
~~~

The keyboard adapter uses stable physical-key IDs rather than GPUI names or
application actions: ArrowLeft=1, ArrowRight=2, ArrowUp=3, Space=4, P=5, R=6,
F=7, K=8, B=9, H=10, Escape=11, and F1=12. Vibesteroids—not Rust—maps those
codes to the actions above. A later textual input ABI must not reinterpret
these physical IDs as localized characters. The native adapter suppresses
duplicate held key-down notifications uniformly; it does not select a repeat
policy by guessing what any key means to the guest.

Input state is maintained by the plugin from down/up events. Focus loss must
clear held controls to prevent stuck input.

### 6.4 Menu flags and standard actions

Plugins declare menu items during fp_configure. IDs 1–1023 are reserved:

~~~text
1  New
2  Open
3  Save
4  Preferences
5  About
6  Quit
7  Help / Controls
~~~

The demo declares:

- New Game using standard action 1;
- separator;
- Help / Controls using standard action 7; and
- Quit using standard action 6.

The host may render these through native menus or gpui-component menus. New is
delivered to fp_event(menu-action, 1, 0, 0). Quit is host-owned: the
frontplane requests confirmation when policy/UI requires it and closes without
requiring the plugin to cooperate.

Application-specific action IDs begin at 1024.

### 6.5 Effect kinds

~~~text
1  request-redraw
2  request-quit
3  set-cursor
4  persist-snapshot
5  open-url
~~~

Dangerous effects such as open-url require a manifest capability and host
confirmation. The Asteroids module uses request-redraw and persist-snapshot;
Quit normally uses the standard host menu action.

### 6.6 Drawing semantics

- Coordinates are logical pixels with origin at the content area's top-left.
- Colors are non-premultiplied 0xRRGGBBAA.
- Width/radius/size must be finite and nonnegative.
- frame_begin must be first and frame_end last.
- At most one completed frame exists per fp_render call.
- Commands before a trap or missing frame_end are discarded.
- Stable command IDs are unique within a frame.
- flags bit 0 on circle means filled.
- flags bit 0 on text means horizontally centered.
- Unsupported flags fail closed in ABI v0.
- Every transform, clip, layer, and path stack must balance before frame_end;
  underflow, overflow, or an incomplete path rejects the entire frame.
- Transform matrices and transformed bounds are finite and bounded.
- Resource IDs and command IDs are plugin-scoped unsigned integers.
- Image bytes count against both per-resource and total-decoded-pixel budgets.
- A sprite's pivot is normalized relative to its destination rectangle.

The composable scene model is deliberately smaller than GPUI but larger than
Asteroids. Its basis is:

1. affine transforms for arbitrary nesting, rotation, scale, and translation;
2. balanced clip/layer stacks for composition and effects;
3. convenience primitives for common shapes;
4. general paths for arbitrary vector art;
5. host-cached image resources plus sprite-sheet source rectangles; and
6. host-shaped text with explicit alignment/wrapping flags.

Animation time is never owned by the renderer. The plugin advances animation
state from deterministic ticks and selects the current sprite rectangle,
transform, opacity, or path. The host may interpolate presentation only when a
plugin explicitly supplies two states and an interpolation fraction; it may
not invent a wall-clock state transition.

The immediate host-import form is chosen because it is exceptionally clear in
hand-authored WAT. Measurements determine whether a later compatible entry
point accepts a packed command buffer for fewer guest/host crossings. Stable
IDs make that path naturally retained-mode-ready without making v0 a mutable
scene graph.

### 6.6.1 Semantic component plane

Menus are the first semantic/native component capability. Buttons, toggles,
labels, text inputs, lists, dialogs, layout containers, focus traversal, and
accessibility nodes belong to a separate retained component tree—not simulated
with painted rectangles. Nodes use stable IDs, role/type, properties, child
order, and generic action/value/focus events. Canvas scene nodes may expose an
accessibility/hit-test overlay by stable ID.

The spike specifies this separation but does not pretend to solve a complete
cross-platform widget protocol. Conflating semantic UI with the high-rate scene
stream would make both inefficient: game frames would churn widget trees, and
painted "buttons" would lose keyboard, screen-reader, and platform behavior.

### 6.7 Guest-declared audio synthesis

Audio program IDs are application-owned integers, not host-known semantics or
file names. During `fp_configure`, a plugin composes each program from bounded
`synth_voice` declarations. Waveforms are sine, sawtooth, white noise, or
brown noise. Each voice declares scheduling, a three-point frequency envelope,
an attack/decay gain envelope, an optional low-pass or band-pass filter sweep,
and a playback cooldown. Integer millihertz and parts-per-million fields keep
the declaration portable and deterministic.

`host.v0.audio(id, volume, pitch, flags)` plays a previously declared program.
Volume and pitch are finite normalized values with bounded pitch range. The
host mixes deterministic mono samples and owns the audio device; it has no
Vibesteroids-specific switch or sound names. The demo composes the original
shot, thrust, two explosion textures, three-part Death Blossom siren, and
five-note extra-life chime without audio assets.

Oscillator phase, frequency/gain interpolation, noise shaping, filters, and
mixing use signed decimal millionths. The `audio` call's host scalars convert
once on ingress and completed PCM converts once for rodio; these adapters are
mechanically excluded from the integer-only synthesis region.

### 6.8 Host status codes

~~~text
 0   accepted
-1   generic rejection
-2   command budget exhausted
-3   invalid pointer or length
-4   invalid UTF-8
-5   invalid/non-finite number
-6   invalid lifecycle order
-7   duplicate command/action ID
-8   unsupported flag/action/capability
~~~

Plugin export errors reserve -1000 through -1999. A trap is not converted into
a plugin status.

## 7. Asteroids state

The demo plugin exposes a fixed 16,384-byte little-endian state region. Its
schema-3 layout begins:

~~~text
offset  size  field
0       4     simulation tick
4       4     configured seed
8       16    viewport width and height as signed decimal millionths
24      48    ship position, velocity, and direction
72      56    score, lives, level, flags, lifecycle, and blossom rotation
256     3072  64 bullet records
3328    2560  32 asteroid records
5888    7200  150 particle records
13088   320   four debris records
13408   1600  100 deterministic star records
~~~

The exact schema is documented beside vibesteroids.wat. Tests decode only
independent invariants; the host treats the entire region as opaque for normal
snapshots.

V0 capacity is fixed to keep direct WAT bounded and inspectable:

- one ship;
- 64 bullets;
- 32 asteroids;
- 150 particles;
- four ship-debris pieces; and
- 100 stars.

Fixed capacity is an intentional direct-WAT spike constraint, not an ABI
constraint. The plugin can grow these pools without changing the host.

The plugin performs:

- rotation, thrust, drag, and wraparound;
- bullet spawn/lifetime;
- asteroid motion and wraparound;
- circle collision;
- deterministic asteroid spawning;
- score and lives;
- paused/game-over/new-game states; and
- audio/effect emission.

## 8. Determinism and state

- Simulation advances only through fp_tick integer ticks.
- A plugin declares its fixed rate with optional `fp_tick_hz`; legacy plugins
  default to 60 Hz.
- The host caps catch-up ticks per rendered frame.
- The seed is the only randomness input.
- Every seed-derived value is inside the snapshot.
- Render calls do not mutate simulation state.
- Snapshot → N ticks → restore → same N ticks must yield identical state bytes
  and render command buffers.
- Audio commands are deterministic outputs but are not replayed when restoring
  an old snapshot unless the corresponding tick is executed again.

Gameplay and synth values are signed decimal fixed-point integers. Structural
tests reject floating-point guest arithmetic outside the marked host-scalar
adapter, so snapshot/replay equality does not depend on an IEEE-754 engine.

### 8.1 Rate-independent simulation contract

Gameplay state stores dimensional quantities in canonical units rather than
per-tick units:

- position: decimal-fixed logical pixels;
- linear velocity: decimal-fixed logical pixels per second;
- acceleration: decimal-fixed logical pixels per second squared;
- angular velocity: decimal-fixed radians per second; and
- authored durations: seconds or milliseconds converted through named helpers.

Only integration, damping, and duration helpers know the declared tick rate.
Changing 60 to 120 must therefore be a one-value policy edit, not a search for
velocity constants. Integer division may lose less than one millionth of a
logical pixel per integration, so the guest either carries a quotient
remainder or stays within the explicit accumulated tolerance below. No float
may be introduced to hide rate conversion.

The native adapter does not schedule `Duration::from_nanos(1e9 / hz)` in a
loop, because that truncates fractional periods and adds work time to the game
clock. It accumulates monotonic elapsed nanoseconds as the exact rational
`elapsed_ns * hz`, carries the remainder modulo one billion, advances all due
fixed ticks, renders once, and drops whole excess ticks beyond a bounded
catch-up budget. The dropped count is diagnostic state, not deferred debt that
could create a spiral of death.

Before 120 Hz becomes the packaged default, the same seed and timestamped
input script run for equal simulated seconds at 60 and 120 Hz must satisfy:

- score, lives, level, flags, pool occupancy, and ordered semantic effects are
  exact for fixtures that do not deliberately exercise collision tunneling;
- constant-velocity positions differ by at most 2,000 decimal microunits after
  ten seconds; accelerated/damped paths differ by at most 0.1 percent of path
  length plus 100,000 microunits;
- velocity magnitude and facing direction differ by at most 0.2 percent after
  ten seconds of equivalent controls;
- authored lifecycle boundaries differ by no more than one 60-Hz tick; and
- each collision-focused difference is classified explicitly as desirable
  reduced tunneling or a regression—never absorbed into a broad tolerance.

The equivalence matrix covers inertial coast and drag, held thrust, both
rotations, manual and automatic fire, bullet expiry, asteroid motion,
particles/debris, explosion and respawn, Death Blossom, and level-scaled
fragment impulses. Render interpolation is out of scope until fixed simulation
equivalence is green; 120 fixed updates with one render per host wake is the
first implementation.

## 9. GPUI frontplane

The initial window contains:

- a gpui-component application/menu bar;
- File or Game menu with New Game and Quit;
- a full remaining-area custom canvas;
- compact status text for FPS/tick/plugin state;
- a visible error overlay after a guest trap; and
- a restart-last-snapshot action.

The adapter:

1. loads a configured or packaged runtime module, with a stable embedded
   conformance module only as the final emergency fallback;
2. validates ABI and calls configure/init;
3. maps GPUI key events to generic event IDs;
4. schedules fixed ticks using the GPUI executor;
5. asks the core for an immutable completed frame;
6. paints commands without re-entering Wasmtime;
7. drains audio/effect commands after the guest call; and
8. requests another animation frame while active.

No Wasmtime host import may call back into a live GPUI Context or Window borrow.

## 10. Headless test plan

Application behavior tests are standard WebAssembly Script (`.wast`) files.
The runner composes the canonical production WAT with a deterministic,
instrumented WAST implementation of `host.v0`; no custom Rust test runner or
test-only export is added to the game. `wasmtime wast` executes the resulting
script directly.

Rust tests stop at the generic frontplane boundary: ABI validation, budgets,
transactional host output, adapters, hot reload, scheduling, snapshots, and
independent metamorphic controls. A test that knows a Vibesteroids state
offset, score, weapon, level rule, or expected game drawing belongs in WAST.
Structural source policies that WAST cannot observe, such as excluding float
opcodes outside the marked ABI adapter, use a dedicated WAT lint rather than a
Rust gameplay oracle.

### 10.1 ABI and lifecycle

- valid module negotiates v0;
- wrong major version rejects;
- missing/incorrect export rejects;
- unknown import rejects;
- configure produces title plus New Game/Quit;
- invalid lifecycle ordering rejects;
- second frame_begin or missing frame_end rejects.

### 10.2 Behavior

- fixed seed produces known initial state invariants;
- left/right rotate in opposite directions;
- thrust changes velocity in facing direction;
- fire allocates one bullet and emits fire audio once;
- key-up stops the action;
- focus loss clears held input;
- positions wrap at all four edges;
- collision removes/splits objects, changes score, and emits audio;
- New Game resets state with the configured seed;
- pause prevents simulation mutation;
- render produces finite, in-bounds-enough vector commands and does not mutate
  state.

Expected behavior comes from public game invariants and independently decoded
state fields, not from reimplementing the entire plugin algorithm in tests.

### 10.3 State/replay

- snapshot length/schema bounds;
- exact snapshot restore;
- deterministic replay after restore;
- rejected wrong-length/schema snapshot leaves current state unchanged;
- last-known-good snapshot survives a plugin trap.

### 10.4 Containment

Separate malicious WAT fixtures attempt:

- infinite loop;
- memory growth beyond policy;
- invalid pointer/length;
- invalid UTF-8;
- NaN/infinite drawing values;
- command flood;
- duplicate command IDs;
- audio flood;
- effect without capability; and
- render trap after partial commands.

Each must produce the specified typed error and leave the host usable.

### 10.5 GUI adapter

Pure renderer tests transform FrameOutput plus viewport/theme into a paint
model. Key/menu mapping is a pure function. Visual appearance is not frozen
until Peter sees the running window or captured output.

The repository also ships a headless `gpui-wasm-render` adapter. It loads the
packaged default, stable fallback, or an external WAT file, deterministically
advances a requested number of fixed ticks, and serializes the immutable
command buffer as SVG.
This is both a human-viewable report and an agent/CI visual artifact; it does
not replace a native-window capture, because the latter independently covers
GPUI layout, focus, compositor, and platform-adapter behavior.

## 11. Build and dependency strategy

The flake supplies:

- stable Rust/Cargo;
- pkg-config and native GPUI Linux dependencies;
- Wasmtime/Cranelift build dependencies;
- audio dependencies;
- wat/wasm-tools validation where appropriate; and
- development diagnostics.

Cargo pins exact GPUI and gpui-component revisions in Cargo.lock. Nix produces:

- the optimized frontplane binary;
- the canonical WAT in a separate data derivation, compiled by Wasmtime when
  the plugin is instantiated without invalidating the frontplane binary; and
- a check derivation running the full headless suite.

Top-level scripts hide Nix invocation:

~~~text
./test
./build
./run
~~~

./run is allowed for this GUI spike and launches an optimized frontplane from
the Nix development environment. ./build remains the reproducible package
path.

## 12. i18n prepare phase

Frontplane-owned strings—window title fallback, menus, status, errors, restart,
and plugin failure messages—come from a typed English catalog.

Plugin-declared labels are UTF-8 presentation strings in v0. A future ABI can
register stable string keys plus an English fallback, allowing the host to
localize standard actions and applications without changing gameplay state.

No translation corpus is required for the spike.

## 13. Feasibility questions to answer

1. Is direct WAT still legible after a nontrivial deterministic game?
2. Is the numeric core ABI clearer than hand-authoring Component Model WAT?
3. Can one completed command buffer isolate Wasmtime from GPUI borrow/reentrancy
   concerns?
4. Does GPUI provide a sufficiently efficient custom vector canvas?
5. Can generated audio be integrated without platform-specific game code?
6. Can the same core module and ABI be hosted in a browser without recompiling
   gameplay?
7. Are state snapshots stable and cheap enough for reload/recovery?
8. Which missing primitive first makes the ABI feel Asteroids-specific?
9. How much binary/build complexity comes from GPUI, Wasmtime, and audio?
10. Does resource containment remain responsive under malicious WAT?

## 14. Explicit non-goals

- production-quality game content or art;
- online multiplayer;
- mobile-native GPUI support;
- a general ECS;
- arbitrary native APIs for plugins;
- full retained component-tree ABI;
- WASI access;
- hot code migration across different state schemas;
- complete localization;
- accessibility semantics beyond documenting the required future ABI; and
- optimizing before profiling.

## 15. Likely next step if successful

Promote gpui-frontplane-v0 into a separately versioned protocol crate/spec,
replace ad hoc numeric binding code with generated bindings or a small
WAT-friendly macro layer, add a browser host, and test a second application
(preferably an editor or non-game utility). A second unrelated application is
the strongest practical test that the ABI is genuinely generic.

### 15.1 Live-edit development mode

The native frontplane should load `./code.wat` when present, otherwise use a
packaged runtime default supplied by the adapter environment, and finally a
stable embedded conformance application if neither exists. A positional file
or application-directory argument selects another source; `--embedded` forces
the conformance fallback, `--seed N` supplies a validated deterministic `u64`
initialization seed, while `--watch` polls the selected path's contents and
transactionally reloads after saves. Watching follows the path rather than an
open inode so editor replace/rename saves remain visible.

Every candidate is compiled, configured, initialized, optionally restored, and
rendered before replacing the active instance. Matching snapshot schema and
length preserve state. A schema/length mismatch starts the candidate from its
freshly initialized state. Any other failure leaves the old instance running
and reports a nonfatal reload error. Plugin authors must increment the schema
when the byte layout or meaning changes; equal untyped linear-memory bytes
cannot prove semantic compatibility.

## 16. Spike implementation status

**Feasibility verdict: successful.** Peter played the deployed native build on
2026-07-16 and confirmed that it is an operational game. The remaining items
below are refinements and production-hardening boundaries, not blockers to the
core WAT-plugin/GPUI-frontplane concept.

Implemented and covered headlessly:

- no WASI, an explicit import allowlist, per-call fuel, and Wasmtime memory,
  table, instance, and transfer budgets;
- pointer, UTF-8, numeric, stable-ID, lifecycle, command, audio, effect, image,
  path, tick, and snapshot validation, with transactional failure rollback;
- exact snapshot/restore, deterministic tick and input delivery, typed
  metadata, standard menu actions, guest-declared bounded synth programs, and
  host effects;
- lines, circles, text, balanced affine transforms, general vector paths,
  image resources, and sprite-sheet source/destination commands;
- an independent non-gameplay composition fixture proving a rotated path,
  plugin-clocked atlas animation, and restoration of the selected frame;
- a WAT-owned Vibesteroids conversion with schema-4 decimal-fixed state and
  canonical per-second velocities,
  seeded placement, ship motion, firing, collisions, scoring, lives, levels,
  pause, restart, wrapping, auto-fire, Kid Mode, Death Blossom, particles,
  debris, safe respawn, guest-declared audio, and vector output; and
- a GPUI/gpui-component window adapter with plugin-declared title and standard
  native menu actions, complete desktop keyboard input, fixed simulation
  ticks, full-window resize delivery, vector painting, and rodio rendering of
  guest-declared synth programs; and
- deterministic headless SVG export for arbitrary requested ticks, including
  external WAT input and stdout/file output suitable for CI visual inspection.
- conventional `code.wat` discovery, explicit file/directory, packaged
  default, and embedded-fallback source selection; content-based path polling
  that survives atomic saves,
  manual Reload/Ctrl+R, and transactional live replacement. Real-window
  verification covered compatible state preservation, schema-triggered fresh
  state, malformed-source rollback while the old game continued advancing,
  and automatic recovery after the file was repaired.

Honest boundaries discovered by the spike:

- GPUI's simple image painter accepts a decoded image and destination bounds
  but does not expose an atlas source rectangle. The core sprite command is
  executable and tested; the current native adapter renders its destination
  outline. A production follow-up should add a clipped/scaled image primitive,
  pre-slice atlases at resource load, or extend GPUI's image element.
- Rectangles, clip stacks, layer opacity/blending, and the retained semantic
  component plane are specified but not yet executable. Affine transforms,
  paths, resources, and sprites were the smallest set needed to prove that the
  scene ABI is not Asteroids-shaped.
- Native menus currently map only reserved standard action IDs. Arbitrary
  plugin action IDs await the retained semantic component/action plane because
  GPUI actions are compile-time Rust types.
- Browser-only device-motion permission, shake activation, touch zones, Eruda,
  and URL test parameters remain outside the desktop proof. A future generic
  pointer and motion capability can add parity without hard-coding mobile
  Vibesteroids behavior into the host.
- GPUI currently brings a very large Zed dependency graph. The first sandboxed
  Nix vendor pass took roughly 25 minutes. The native source and mutable WAT
  application are now separate derivations, so plugin edits reuse the local
  native result; a cold hosted runner still needs a binary cache or a future
  crane/cargo-chef dependency split to avoid rebuilding GPUI from scratch.
- Candidate compilation currently runs synchronously on the GPUI thread.
  Transactional replacement prevents corruption, but larger applications may
  require background compilation plus UI-thread handoff to avoid a visible
  pause.
