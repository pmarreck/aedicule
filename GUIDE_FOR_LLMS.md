<!-- Generated from `GUIDE_FOR_LLMS.template.md` plus `src/wat_abi.rs`; do not edit this generated copy by hand. -->

# Guide for LLMs writing Aedicule applications

Guide version: `0.3.0`

Current Aedicule WAT ABI: `0.8`

Canonical guide: https://github.com/pmarreck/aedicule/blob/yolo/GUIDE_FOR_LLMS.md

This document is meant to be sufficient even when you cannot inspect Aedicule's source. It distinguishes the callable ABI from planned work, gives a reliable authoring process, and ends with the exact generated ABI reference. If a copied guide disagrees with a newer canonical guide, use the guide whose ABI version matches the Aedicule host you must support.

An Aedicule **guest** is a WebAssembly Text (`.wat`) application. Aedicule is the **host** or **frontplane**: it schedules deterministic updates, validates capability calls, renders through native or browser adapters, delivers input, plays admitted audio, and can replace a running guest transactionally. A guest receives no ambient filesystem, network, clock, random-number generator, or operating-system access.

## Ground rules

1. Treat the exact generated appendix as the authority for names and signatures. Import only from `aedicule.v0`, use only `AE_*` names, and declare the ABI version exports.
2. Everything under **Available now** is implemented in ABI `0.8`. Everything under **Proposed—not callable yet** is design direction, not an import or export you may emit.
3. Check every status-returning host import. `0` means success. A nonzero result rejects the guest transaction that contains it; ignoring the result does not make invalid output valid.
4. Keep application state in guest memory. Treat `AE_render` as a projection of that state, not as the place to advance simulation or fire one-shot effects.
5. Comment intent, units, invariants, and state layout. WAT is compact enough that an uncommented correct program can still be unmaintainable.

## Available now

### Compatibility and capability boundary

The only current import module is `aedicule.v0`. `AE_abi_major` must equal the host major exactly. A guest minor may be older than the host minor, but must not be negative or newer. Raise the guest minor when it requires a capability introduced by that revision.

The host validates UTF-8 ranges, pointers, numbers, IDs, transaction structure, resource counts, memory/table growth, and execution fuel. Validation is fail-closed and bounded. A guest cannot gain authority by spelling a path, importing WASI, or calling an undeclared symbol.

Every guest exports one `memory`, the ABI/version functions, `AE_configure`, one complete lifecycle profile, `AE_tick`, `AE_render`, and the three state-contract functions. Optional exports add rate selection and post-restore behavior. The exact signatures are in the appendix.

### Choose one lifecycle profile

The **legacy profile** exports `AE_init` and `AE_event`, whose viewport/event coordinates are `f32`. The **integer profile** exports `AE_init_i32` and `AE_event_i32`, whose logical-pixel coordinates are signed Q16.16 integers. The two functions are an atomic pair: never export only one, and never mix profiles.

Prefer the integer profile when exact cross-machine reproducibility matters. It permits a module with no `f32`/`f64` types or opcodes when you also use the integer drawing imports. Use the legacy profile only when the available float drawing primitives materially simplify the application.

Q16.16 means:

- `1 logical pixel = 65536`
- integer pixels convert with `pixels << 16`
- a Q16.16 product needs a widened intermediate and a right shift by 16
- negative coordinates are ordinary signed `i32` values

Packed colors are `0xRRGGBBAA` carried as an `i32` bit pattern. They are not four separate integers.

### Lifecycle order

A normal initial load is:

1. validate imports, exports, and ABI version;
2. call `AE_configure`;
3. call the selected `AE_init*`;
4. deliver viewport/display information when known;
5. call optional `AE_tick_rate`;
6. call `AE_render`; and
7. publish the candidate only after all of the above succeeds.

`AE_configure` declares stable metadata and capabilities such as the title, action/menu labels, integer slider lattices, labeled external HTTPS destinations, synth programs, and sampled FLAC assets. Do not repeatedly redeclare them during updates.

`AE_init*` receives a deterministic 64-bit seed as two `i32` halves and the initial logical viewport. Initialize all mutable state there. If you need randomness, implement a deterministic PRNG in guest state from that seed; do not assume an ambient source.

Events are ordered input edges. Update guest state in `AE_event*` or `AE_control_event`, then let subsequent fixed ticks consume that state. `AE_tick(ticks)` advances exactly the requested number of whole simulation steps. `AE_render` reads current state and emits an atomic visual transaction.

Do not update simulation once per render. Displays can refresh at fractional rates, render calls can be skipped or repeated, and input can arrive between ticks.

### Rational simulation timing

Optional `AE_tick_rate(current_numerator, current_denominator)` receives the already agreed **simulation** rate. It does not receive the display refresh rate. Return a positive rational rate up to the documented bounds, return `(0, 0)` to follow display timing, or omit the export to use that default.

The display rate is an event because it can change when a window moves between monitors. Event kind `9` carries an exact numerator and denominator initially and only when it changes. Use that event to reconsider application policy; never treat a decimal approximation as elapsed time.

The host accumulates monotonic time against the agreed rational rate, preserves ordered input edges, batches safe event-free ticks, and bounds overload. Guest logic should be deterministic for any positive `ticks` count and should not depend on host wall-clock duration.

### Input

The event table in the appendix defines stable key, pointer, viewport, focus, display-rate, menu-action, and two-axis scroll kinds. Treat down and up as edges and keep held-state bits in guest memory. Do not implement hold behavior as a one-shot fallback.

Pointer scroll preserves both axes and distinguishes line units from logical-pixel units. Do not silently convert one unit into the other. Middle/wheel click is pointer button `3`, separate from scroll.

Native and browser adapters currently deliver the documented keyboard and pointer set. Browser touch is presently lowered through pointer behavior; stable touch identity, multiple contacts, pressure, and cancellation do not yet have a finalized guest ABI.

### Host-scheduled pause

Pause is opt-in. During `AE_configure`, call `AE_pause_trigger(1, KEY_ID, 0)` for each stable physical key that should toggle suspension. Declare ABI minor `3` or newer when using it. If a guest declares no pause trigger, Aedicule does not alter or deduplicate its raw input lifecycle.

Aedicule consumes both raw edges of each declared trigger. Do not also handle that key as gameplay input. Instead, handle event kind `15`: code `1` means the scheduler and guest-audio bus have just paused, code `2` means they have resumed, and code `3` tells a hot-reload candidate that it is replacing an already-paused guest. Update only pause presentation/state in these events; ticks stop while paused, and the host requests one render for each phase.

Held/repeated triggers do not immediately undo a pause. A fresh down after release resumes. Aedicule reconciles releases and focus cancellation observed while suspended before delivering code `2`, preserves the exact rational tick phase, and never converts paused wall time into catch-up ticks. Viewport/display changes and transactional reload remain live without advancing simulation.

### Atomic rendering

One `AE_render` call produces one all-or-nothing frame:

1. call exactly one frame-begin import;
2. optionally submit one changed UI snapshot;
3. issue bounded draw commands;
4. balance every path and transform scope; and
5. call `AE_frame_end`.

If an import rejects, the guest traps, a scope remains open, or `AE_render` returns failure, Aedicule retains the last accepted frame and UI snapshot. Never depend on partial output.

Available custom drawing includes lines, circles, UTF-8 text, a portable bundled Geist Mono Regular face, quadratic and cubic Bézier paths, affine transforms, defined images, and sprites cut from image rectangles. Integer-only guests currently have Q16 text and line-path operations but not integer counterparts for every float primitive; build missing shapes from the integer path surface where possible.

Every path, line, circle, text, and sprite draw ID must be unique across all primitive kinds within one frame. The ID set resets at the next frame begin. Reuse the same semantic ID in later frames, but never reuse it for a different object in the same frame.

Encoded image bytes can be defined from guest memory, or raw RGBA can be supplied using the documented flag. Images have explicit guest IDs and release calls. Package-relative image loading by virtual asset name is not yet an ABI capability, so a guest must currently place image bytes in its module or otherwise obtain them through a future capability.

### Aedicule View Protocol v0 controls

The implemented Aedicule View Protocol (AVP) kernel is a retained, declarative, guest-owned snapshot—not a hardcoded host widget store.

During `AE_configure`, declare:

- exact integer sliders with stable ID, label, inclusive range, step, and initial value; and
- standalone actions for native buttons, plus explicit menu items when the
  action should also appear in a menu; and
- external links with a stable ID, nonempty visible/accessibility label, and absolute HTTPS URL.

When desired UI changes, submit a complete snapshot during `AE_render`:

```wat
call $AE_ui_begin
;; panels must precede children
call $AE_control_panel_q16
call $AE_slider_place_q16
call $AE_button_place_q16
call $AE_external_link_place_q16
call $AE_ui_end
```

Revisions are opaque 32-bit values and must differ from the last accepted revision. Geometry is absolute viewport-relative Q16.16. The guest owns bounds, values, selected state, and stable IDs. Aedicule validates, renders, preserves appropriate transient focus/drag mechanics, occludes the underlying canvas, and delivers semantic events.

Submitting no UI transaction retains the last accepted snapshot. Submitting a complete empty snapshot removes it. An invalid candidate leaves the prior snapshot intact. After a control or action event, update guest state and submit a new revision if the visible control value or selected state should change.

This current profile contains panels, exact integer sliders, buttons, external links, and single-line text fields only. It is not the proposed general layout/tree surface, and its text field is deliberately narrower than the proposed text-input node: the widget owns its buffer, so a guest reads the value but cannot set it.

### External links

ABI minor `4` adds a narrow external-navigation capability without giving the guest ambient browser or network access. `AE_external_link` is configure-only: its label must be nonempty and its destination must be an absolute, credential-free HTTPS URL without whitespace, controls, or backslashes. `AE_external_link_place_q16` places that stable ID once in the current retained UI snapshot, after its parent panel.

A link activates only through a real click on the host-rendered accessible control. The native adapter delegates to its platform URL service. The browser adapter requires transient user activation and requests a new context with `_blank` and `noopener`; popup denial is diagnostic only. The guest receives no success/failure event and cannot programmatically trigger navigation. `aedicule-render --activate-link ID` is the deterministic test adapter: it validates the current accepted revision and reports the normalized request without opening anything.

### Deterministic math

`AE_sin_cos_turn` returns sine and cosine in Q1.30 using deterministic integer arithmetic. The input is a wrapping binary angle: the full unsigned 32-bit range is one turn, `0x40000000` is one quarter-turn, and so on. Use widened intermediates when combining Q1.30 and Q16.16 values.

This import returns two values, not a status. It is the exception to the usual status-returning import convention.

### Audio

There are two current one-way audio paths:

- declare bounded synth programs during `AE_configure`, then trigger them with `AE_audio`; and
- bind a stable sample ID to a FLAC beneath `assets/` with `AE_sample_asset`, then trigger it with `AE_sample_play`.

The host decodes admitted FLAC into explicit interleaved PCM and hands it to the native or Web Audio device adapter. The guest does not open an audio device and should not parse host paths. Sample playback is currently fire-and-forget; streaming, progress, completion, mixing buses, and codecs other than packaged FLAC are not exposed.

Do not trigger audio from `AE_render`: rendering can repeat. Trigger it from the input/tick transition that makes the sound semantically happen.

### Effects and diagnostics

`AE_effect` can request redraw, quit, cursor changes, or snapshot persistence through stable kind IDs. These are requests to a bounded host adapter, not direct operating-system calls.

`AE_log` accepts bounded UTF-8 diagnostics. Logs are for diagnosis and must not be the only representation of application state or test success.

### Transactional hot reload and state

`AE_state_ptr`, `AE_state_len`, and `AE_state_schema` define the exact snapshot bytes owned by the current guest. Keep them stable and in bounds. When a watched source changes, Aedicule:

1. builds and validates a separate candidate;
2. configures and initializes it without disturbing the running guest;
3. restores bytes only when schema and length are compatible;
4. calls optional `AE_after_restore`;
5. delivers current host state and renders the candidate; and
6. swaps only after the whole candidate is healthy.

If any step fails, the old guest, frame, and state remain live. Changing state layout without changing the schema is a client bug. If no state should persist, intentionally export a zero length.

### Source and package forms

Aedicule accepts three equivalent source forms:

```text
aedicule path/to/code.wat
aedicule path/to/project-directory
aedicule path/to/application.aed
```

A project directory must contain `code.wat`. A recommended current layout is:

```text
project/
  code.wat
  assets/
    audio/
      sample.flac
  tests/
    main.wast
    additional-suite.wast
  lib/
    helper.wat
  GUIDE_FOR_LLMS.md
  README.md
  LICENSE
```

Only `code.wat`, direct `tests/*.wast`, and declared `assets/` currently have runtime meaning. Other files may document or support the project but are not linked automatically. `lib/*.wat` is a packaging convention, not a current WAT linker.

`aedicule --package DIRECTORY [OUTPUT.aed]` creates a deterministic stored-ZIP `.aed` with a leading Aedicule MIME entry. It preserves already-compressed FLAC instead of wastefully recompressing it. `--depackage` validates and expands into a new directory without merging over existing data. Packages are bounded to 1,024 entries, 16 MiB per entry, and 64 MiB total.

`aedicule --web SOURCE [--bind ADDRESS] [--port PORT]` serves the same guest/assets through the bundled browser runtime. `--watch` transactionally reloads source changes. `aedicule-render` is the deterministic headless rendering/control adapter.

### Application tests

`aedicule --test [--seed N] SOURCE` runs every direct `tests/*.wast` suite. `tests/main.wast` is required. Suites execute in a deterministic randomized order using the emitted seed, which makes order-dependency failures reproducible. Repeat a failure with the printed seed before simplifying it.

WAST suites are ordinary WebAssembly spec-script tests; they are not yet a host-ABI GUI automation language. Use them for guest computations, state transitions, module validity, and deterministic invariants. Aedicule's own headless renderer can separately exercise current control values and frame output.

## A reliable authoring loop

1. Start from the smallest lifecycle skeleton in the exact appendix.
2. Choose integer or legacy lifecycle once; do not mix them.
3. Write state-layout comments before state code. Give every byte range, global, fixed-point unit, event kind, and stable-ID range a named purpose.
4. Add one behavior at a time. Write or extend `tests/main.wast` so the new behavior fails before implementing it.
5. Run `wasm-tools validate code.wat` before Aedicule. Stack/type failures are faster to understand there.
6. Run `aedicule --test --seed N project/`.
7. Run `aedicule-render` for deterministic frame/control checks.
8. Run the native or browser frontplane and exercise real keyboard, pointer, UI, and audio adapters.
9. Package only after the directory form is green, then rerun the same tests against the `.aed`.
10. Preserve the commented authoring `.wat`. Treat generated or optimized `.wasm` as a disposable build artifact.

For nontrivial code, use named helper functions and locals even when inlining would save a few lines. WAT's stack notation is easiest to verify when each helper has one job and a comment states its stack/result invariant.

## Common WAT and LLM mistakes

### ABI mistakes

- Importing the retired `host.v0` module or `fp_*` names. The hard-cutover surface is `aedicule.v0` plus `AE_*`.
- Guessing a signature from prose. One wrong type makes module instantiation fail; copy the exact appendix signature.
- Declaring only one of `AE_init_i32` / `AE_event_i32`, or exporting both integer and legacy pairs.
- Claiming a proposed capability exists because it appears elsewhere in this guide. Only the generated appendix is callable.
- Using a newer minor capability without raising `AE_abi_minor`.
- Ignoring a status return. If a value is not needed, use `drop` deliberately and comment why failure is impossible; otherwise branch to a defined error path.

### Stack-machine mistakes

- Leaving a value on the operand stack at the end of a function, block, loop, or branch.
- Writing a value-producing `if` without a result annotation. For example, branches that each leave an `i32` require `(if (result i32) ...)`.
- Forgetting that `local.tee` both stores and leaves a value, often causing “values remaining” later.
- Reversing operands because folded and linear instruction styles were mixed.
- Branching to a label with the wrong result arity.
- Using an `i32` multiply where Q16/Q30 work needs a widened `i64` intermediate.

### Memory and string mistakes

- Passing character counts instead of UTF-8 byte lengths.
- Assuming strings are NUL-terminated. Aedicule accepts `(pointer, byte_length)` slices.
- Overlapping mutable state, static data segments, and scratch strings.
- Returning state pointers/lengths outside exported memory after growth.
- Depending on uninitialized or out-of-range loads trapping “helpfully”; a trap rejects the transaction.

### Timing and input mistakes

- Using render frequency as simulation frequency.
- Treating display refresh as a query result or as elapsed time.
- Converting rational rates to floats and accumulating drift.
- Handling key-down but not key-up, which breaks hold semantics.
- Replaying one-shot sound/effects from `AE_render`.
- Assuming every adapter supplies every possible device event. Build explicit fallbacks only from documented events.

### Rendering and UI mistakes

- Reusing one draw ID for a path and a text item in the same frame.
- Opening a path, transform, frame, or UI transaction without closing it on every control-flow path.
- Reusing an accepted UI revision.
- Placing a slider before its panel or before declaring its integer lattice in `AE_configure`.
- Placing an external link before its panel, omitting visible link text, or treating link activation as guest-observable network access.
- Treating transient host slider position as guest state. The next accepted guest snapshot is authoritative.
- Emitting UI every render even when unchanged; retained snapshots exist to avoid that work.
- Assuming packaged files are automatically images/fonts. Only explicit current capabilities cross the boundary.

### Reload, determinism, and containment mistakes

- Changing state layout while retaining its schema number.
- Reading ambient time, randomness, filesystem, or network through an undeclared import. Aedicule deliberately provides none.
- Using nondeterministic iteration/order in golden outputs.
- Performing unbounded work in one lifecycle call. Aedicule meters calls and bounds command/resource counts.
- “Fixing” a failing test by weakening or deleting it. First reproduce the exact failure, then make the smallest guest change.

### Commenting mistakes

Use line comments (`;;`) for intent near instructions and block comments (`(; ... ;)`) for larger state/layout maps. Do not merely narrate syntax.

Good:

```wat
;; State bytes 0..3: signed Q16.16 ship X, viewport-relative.
;; Invariant: velocity integration clamps X before any render import sees it.
(global $ship_x_q16 (mut i32) (i32.const 0))
```

Weak:

```wat
;; Define a mutable i32 global.
(global $x (mut i32) (i32.const 0))
```

Comment every magic ID range, fixed-point unit, packed bit field, memory region, lifecycle invariant, and non-obvious `drop`. An LLM revising terse WAT needs those semantic anchors more than it needs a paraphrase of the opcode.

## Recommended toolchain

Use multiple independent validators when practical:

- [Aedicule](https://github.com/pmarreck/aedicule): ABI admission, application WAST suites, headless rendering, package parity, native/browser adapters, and transactional reload.
- [Bytecode Alliance `wasm-tools`](https://github.com/bytecodealliance/wasm-tools): `wasm-tools validate code.wat`, `wasm-tools parse code.wat -o code.wasm`, `wasm-tools print code.wasm`, `wasm-tools objdump`, and WAST validation. This is the preferred first validator.
- [WABT](https://github.com/WebAssembly/wabt): independent `wat2wasm`, `wasm-validate`, `wasm2wat`, and `wasm-objdump` checks.
- [`wasm-language-tools`](https://github.com/g-plane/wasm-language-tools): editor diagnostics, language-server navigation, and WAT formatting. Review formatting diffs because stack-sensitive code can become less readable when aggressively folded.
- [Binaryen](https://github.com/WebAssembly/binaryen): `wasm-opt` for release-size/performance experiments on compiled `.wasm` copies. Validate and rerun all guest tests after optimization. Never replace the commented source `.wat` with optimizer/printer output.
- [WebAssembly core text specification](https://webassembly.github.io/spec/core/text/): authoritative grammar and validation semantics when tools disagree.

A useful independent check sequence is:

```text
wasm-tools validate code.wat
wat2wasm code.wat -o code.wasm
wasm-validate code.wasm
aedicule --test --seed 0x5eed project/
aedicule-render project/ --ticks 1
```

Pin tool versions in reproducible builds. Tool defaults evolve as WebAssembly proposals graduate, while Aedicule ABI `0.8` intentionally exposes a narrower core contract.

## Proposed—not callable yet

The following items are roadmap material. Do not import or export names for them until a later generated ABI appendix defines exact signatures.

### General AVP application UI

The intended general Aedicule View Protocol is a retained semantic tree with stable node IDs, atomic complete revisions, host-computed layout, adapter reconciliation, accessibility, and headless inspection. Planned nodes include rows/columns, scroll containers, semantic text, guest-owned multi-line text inputs, toggles, images, canvas regions, tabs, split panes, dialogs, lists, tables, trees, grids, and virtualized collections. The current absolute panel/slider/button/link/text-field snapshot is only the working v0 kernel.

The guest will own desired state, content, semantics, and layout constraints. The host will validate and adapt them to GPUI, browser, accessibility, and headless frontplanes.

### Application shell and services

Planned profiles include semantic menus and overridable command/control shortcuts, toolbars, context menus, multiple windows, close/dirty negotiation, drag/drop, clipboard, notifications, print/export, and accessibility relationships.

User-mediated files will require scoped open/save handles, not ambient paths. Persistent application state is likely to use an asynchronous bounded key/value capability with explicit quotas/rate limits and desktop/web adapters. Network access, if added, will require declared origin policy. None of these services exists today.

### More execution modes and devices

Planned or under design:

- event-driven `0/1` simulation rate plus explicit timers;
- dedicated touch contacts/cancellation/pressure and safe-area inset events;
- game-controller discovery, buttons, axes, and connect/disconnect events;
- first-class CLI streams/arguments/exit status and later TUI input/size;
- richer sampled/streaming audio, completion events, and additional codecs;
- packaged image/font asset handles by virtual name;
- general file drop and browser Open/drag-drop of arbitrary `.wat`/`.aed`;
- `--scaffold`, environment-aware `config.toml`, `--appify`, and `--deappify`;
- one-binary native application delivery and guest-customizable shell metadata.

Garbage-collector policy is also only a proposal. A core Wasm guest does not presently control Rust or host GC behavior.

### Delivery work that is not guest ABI

Content-hashed browser runtime assets, a generic demo-free browser launcher, release packaging, signing, file associations, and GitHub Pages improve delivery but do not create guest capabilities. Do not confuse a feature of the surrounding launcher with a callable WAT import.

## Handoff checklist

Before calling a guest ready:

- the import module and every signature match the exact appendix;
- the declared ABI minor covers every required import;
- exactly one complete lifecycle profile is exported;
- all host return statuses are handled intentionally;
- state memory, schema, and reload behavior are documented and tested;
- fixed-point units and packed fields are documented beside their storage;
- simulation is driven only by `AE_tick`, never render cadence;
- input down/up edges and held state are tested;
- every render/UI transaction balances on all paths;
- draw IDs are frame-wide unique and UI IDs/revisions are stable;
- one-shot audio/effects originate in state transitions;
- `tests/main.wast` exists and shuffled test order is replayable from its seed;
- directory and `.aed` forms produce the same behavior;
- native, browser, and headless adapters used by the application are exercised;
- `wasm-tools` and at least one independent validator accept the module; and
- comments explain intent, units, invariants, state layout, and magic IDs.

## Exact generated ABI reference

The remainder is generated from the same declarative Rust table used to check `WAT_ABI.md`. It is deliberately duplicated here so an LLM with only this file still has every current name, signature, event code, and lifecycle rule.

### Aedicule WAT ABI v0.8

This is the complete client-facing ABI for WAT applications accepted by Aedicule today. The only import module is `aedicule.v0`. Every function at this boundary is named `AE_*`; the provisional `host.v0` / `fp_*` names are rejected.

Except for the pure multi-result `AE_sin_cos_turn`, imported capability functions return a status: `0` succeeds; any non-zero result rejects the current guest transaction. Strings are UTF-8 byte slices in the guest's exported `memory`. IDs and packed colors travel as `i32` bit patterns. The host validates pointers, finite numeric values, object limits, and frame structure.

#### Guest exports

```wat
(memory (export "memory") MIN_PAGES)
(func (export "AE_abi_major") (result i32))
(func (export "AE_abi_minor") (result i32))
(func (export "AE_configure") (result i32))
(func (export "AE_init") (param i32 i32 f32 f32) (result i32)) ;; legacy profile
(func (export "AE_event") (param i32 i32 f32 f32) (result i32)) ;; legacy profile
(func (export "AE_tick") (param i32) (result i32))
(func (export "AE_render") (result i32))
(func (export "AE_state_ptr") (result i32))
(func (export "AE_state_len") (result i32))
(func (export "AE_state_schema") (result i32))
(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)) ;; optional
(func (export "AE_after_restore") (result i32)) ;; optional
(func (export "AE_motion_event") (param i32 i32 i32 i32 i32 i32) (result i32)) ;; optional
```

`AE_abi_major` must return `0`. A guest may replace the legacy `AE_init` and `AE_event` pair with the integer-only `AE_init_i32` and `AE_event_i32` pair documented below; mixing the profiles is invalid. `AE_tick_rate(current_numerator, current_denominator)` chooses a reduced or unreduced rational simulation rate in the inclusive 1..=1000 Hz range, with denominator at most 1,000,000. It receives the currently agreed **simulation** rate, never the ambient display rate. Returning `(0, 0)`, or omitting the export, follows the current display mode.

#### Lifecycle and timing

A normal startup is `AE_abi_*`, `AE_configure`, the selected profile's init export, one display-refresh event, optional `AE_tick_rate`, then `AE_render`. Reload configures and initializes a new instance, restores a compatible snapshot (and calls `AE_after_restore` if present), delivers the current display-refresh event, selects its rate, and renders before it can replace the active instance.

The selected event export has `(kind, code, a, b)`. Kind `9` is `AE_EVENT_DISPLAY_REFRESH`: `code = refresh_numerator_hz`, `a = refresh_denominator`, `b = 0`. It is emitted once when the initial display mode becomes known and only again when that mode changes. A guest must treat its absence as “same as the last event”; it cannot query display timing. The host performs the display event before calling `AE_tick_rate`.

`AE_tick(ticks)` receives only bounded whole fixed steps. The host accumulates monotonic time exactly at the agreed rational rate and may report overload by dropping old due ticks while preserving ordered input edges.

#### Host display timing override

Set `AE_DISPLAY_REFRESH_RATE` before process startup to override the initial nominal display rate while native display discovery is unavailable. The value is exact and normalized by greatest common divisor: `60000/1001` is a rational rate, `5994/100` becomes `2997/50`, and ordinary decimals are parsed as written without floating point. Canonical nominal spellings `23.976`, `29.97`, `59.94`, and `119.88` select respectively `24000/1001`, `30000/1001`, `60000/1001`, and `120000/1001`. Any other decimal, such as `59.97`, stays exact as typed. Invalid values fail startup; an unset variable retains the native/default source. This is host configuration, not a guest display-query capability.

#### Host imports

All imports use `(import "aedicule.v0" "NAME" (func ...))`.

#### ABI compatibility

The major version must match exactly. Aedicule accepts guest minor versions from `0` through its documented host minor so older guests remain runnable; negative or future minor versions are rejected before configuration. A guest must raise its declared minor when it requires an import or behavior introduced by that revision.

#### Input events

`AE_event(kind, code, a, b)` and its integer-profile counterpart `AE_event_i32(kind, code, a, b)` carry host input at an ordered fixed-step boundary. A host may omit an event kind it cannot observe, but it must not invent application semantics. In the integer profile, logical-pixel values use signed Q16.16; IDs, flags, and the display-rate denominator remain ordinary unscaled integers.

| Kind | Event | `code`, `a`, `b` |
| ---: | --- | --- |
| 1 | Key down | `code` is a physical-key ID; `a = b = 0` |
| 2 | Key up | Same key ID; `a = b = 0` |
| 3 | Pointer move | `code = 0`; `a = x`, `b = y` logical pixels |
| 4 | Pointer down | `code` is button ID; `a = x`, `b = y` logical pixels |
| 5 | Pointer up | `code` is button ID; `a = x`, `b = y` logical pixels |
| 6 | Device change | `code` is a device-class flags bitfield; `a = width`, `b = height` logical pixels |
| 7 | Action | `code` is the guest-declared action ID; `a = b = 0` |
| 8 | Focus | `code = 1` when focused, `0` when unfocused; `a = b = 0` |
| 9 | Display refresh | `code = numerator_hz`; `a = denominator`, `b = 0` |
| 10 | Pointer scroll | `code` is unit ID; `a = horizontal delta`, `b = vertical delta` |
| 15 | Pause lifecycle | `code` is `1` paused, `2` resumed, or `3` restored-paused; `a = b = 0` |
| 16 | Motion gesture | `code = 1` is a shake; `a` is peak user-acceleration magnitude in m/s² (f32 legacy, signed Q16.16 integer profile); `b = 0` |

Kinds 11 through 14 are reserved for the proposed touch-contact profile and are not emitted yet. Device-class flag bit `0` is set when the primary pointer is coarse (a finger-first touch device), mirroring the CSS `(pointer: coarse)` media query; all other flag bits are reserved, sent as zero, and a guest must mask only the bits it understands. The device-change event is delivered at least once at start and again whenever the viewport dimensions or the device-class flags change, including a flag flip at unchanged size such as an iPad gaining a trackpad. The motion gesture (kind 16) is delivered only to guests that registered `AE_motion_interest` kind `1`; the host derives it from the sensor stream with a fixed threshold and cooldown, so one physical shake is one event. Six-axis samples never ride the ordinary event exports: a guest that registered kind `2` must export `AE_motion_event(ax, ay, az, rx, ry, rz)` and receives acceleration in m/s² and rotation rate in deg/s, all signed Q16.16 in BOTH lifecycle profiles. Where sensors are absent or the platform denies permission, registered interests stay silent rather than erroring - a guest must keep a manual affordance for every motion-triggered action. Physical-key IDs are `1` left, `2` right, `3` up, `4` space, `5` P, `6` R, `7` F, `8` K, `9` B, `10` H, `11` Escape, `12` F1, `13` W, `14` A, and `15` D. Pointer button IDs are `1` primary/left, `2` secondary/right, and `3` middle/wheel-click. Native and browser canvas adapters emit down and up edges for all three IDs.

Pointer-scroll unit IDs are `1` lines and `2` logical pixels. Positive `a` means leftward motion; positive `b` means upward motion. Hosts preserve both axes and omit zero-delta scroll events.

#### Host-scheduled pause

A guest opts in by calling `AE_pause_trigger(kind, code, flags)` during `AE_configure`. ABI v0.3 accepts selector kind `1` for a stable physical-key ID and requires `flags = 0`. Declarations are bounded and unique. A guest that declares no trigger receives the ordinary lifecycle unchanged.

A fresh declared key-down is consumed by Aedicule; neither that down edge nor its matching up edge reaches gameplay. Repeats and a still-held trigger cannot self-resume. On pause, Aedicule executes work already due at the input barrier and delivers any earlier queued input in arrival order, freezes the fixed-step scheduler and guest audio transport, sends kind `15`, code `1`, accepts exactly one final render, and then performs no ordinary guest ticks or renders. Host window, menu, watcher, and transactional reload machinery remain live. A viewport or display-mode change may send its semantic event and accept at most one replacement paused frame without advancing simulation.

A fresh trigger after release resumes. Releases or focus cancellation for inputs held at pause entry are delivered first, then kind `15`, code `2`, one render, and the next exact rational deadline. New gameplay presses made only while suspended are not armed. A reload candidate admitted while paused receives kind `15`, code `3` and renders its paused presentation before atomic replacement. Failed reloads preserve the old guest and transport.

Paused wall time is removed from the scheduler baseline, so it produces no catch-up ticks and retains the pre-pause fractional phase. Native playback uses a guest-only pausable mixer and shifts synth cooldown timestamps by the same duration; browser sampled audio suspends its shared Web Audio context. Successful reload deterministically cancels old guest sources. A small device buffer may finish after the logical barrier, but its latency never advances guest time. Audio/effects requested by the paused transition itself are discarded in ABI v0.3; the resumed transition may request new output.

#### Integer controls and declarative native UI

`AE_slider_i32` declares only a slider's stable ID, accessible label, and exact integer lattice during `AE_configure`; it does **not** create a persistent host-owned widget. A guest that declares any slider must export `(func (export "AE_control_event") (param id i32) (param value i32) (param phase i32) (result i32))`. Values are validated against the declared inclusive minimum, maximum, and exact step lattice before delivery. Phase `1` is a continuous change and phase `2` is the committed release edge. Native delivery is ordered through the simulation scheduler. `AE_action` declares a reusable action ID and accessible label without creating any menu item. A non-separator `AE_menu_item` remains the backward-compatible combined declaration of the same action plus an explicit menu presentation. Both imports share one collision-checked action-ID namespace. A button click delivers the ordered kind-`7` action event.

Native UI implements the v0 native-controls profile of the Aedicule View Protocol (AVP). The guest is authoritative for the desired keyed view document and submits a complete snapshot during `AE_render` only when that UI changes: `AE_ui_begin(revision)`, zero or more widget declarations, then `AE_ui_end()`. The revision is an opaque 32-bit bit pattern and must differ from the last accepted revision. An unchanged UI is omitted while ordinary `AE_frame_*` canvas rendering continues. A completed snapshot is published only when the entire surrounding `AE_render` call also returns a valid canvas frame; any rejected import, trap, missing `AE_ui_end`, or incomplete frame preserves the previously accepted snapshot. A snapshot may intentionally be empty, which removes every native widget. This profile is flat and absolute-positioned; the proposed general semantic tree, layout, text-input, accessibility, and adapter-parity contract is specified separately in `VIEW_PROTOCOL.md`.

Inside the UI transaction, emit each `AE_control_panel_q16` before any slider, button, or external link that references it. Geometry is absolute viewport-relative signed Q16.16. Panel, slider, button, and external-link stable IDs are independently unique within one snapshot. A slider placement must name a configure-time declaration and carry a value on that declaration's exact lattice. The guest-provided value is authoritative; Aedicule may retain a GPUI entity only for focus, hover, drag, and accessibility mechanics, then reconciles it by stable ID when a new revision is accepted. Omitting an ID from the next snapshot removes it. Slider label placement `0` hides the visual label/value and `1` places it above a full-width track; slider and panel flags remain zero. A button placement must reference a declared action. Button flag bit `0` selects the platform-native highlighted state; every other bit is reserved and must be zero.

After accepting a control or action event, a guest that wants the changed value or selected state displayed must update its model and submit a new UI revision. If it does not, Aedicule reconciles transient slider interaction back to the last accepted guest value and retains the last accepted button state. Headless `aedicule-render --control ID=VALUE` arguments are repeatable, preserve argument order, use phase `2`, and execute after initialization but before requested ticks and rendering. `--activate-action ID` is also repeatable and delivers kind `7` only when a button for that action exists in the current accepted UI snapshot.

#### External links

ABI v0.4 adds a bounded, guest-owned external-navigation control rather than ambient browser or network access. During `AE_configure`, `AE_external_link` declares a stable ID, nonempty visible/accessibility label, and absolute HTTPS URL. The host rejects non-HTTPS schemes, whitespace or controls, backslashes, missing hosts, and URL credentials; `flags` must be zero. During a changed UI snapshot, `AE_external_link_place_q16` places that ID inside an already-declared panel. Its placement ID is the declaration ID, may occur only once per snapshot, and uses the same absolute Q16.16 geometry as other controls.

Native and browser adapters resolve activation against the exact currently accepted `(revision, id)` only from the platform control's real click handler. Native delegates to the platform URL service. Browser activation requires transient user activation and opens a new browsing context with `_blank` and `noopener`; popup-policy failure is an adapter diagnostic, not guest-visible state. The guest receives no navigation result and cannot synthesize activation through an import. Headless `aedicule-render --activate-link ID` never opens a browser; it validates the current placement and emits the deterministic revision, ID, and normalized URL to stderr for automation.

#### Text entry

ABI v0.5 adds the one control backed by a real editable element. During `AE_configure`, `AE_text_field` declares a stable ID, a nonempty accessible label, and an exact capacity measured in **Unicode scalar values** — not bytes and not UTF-16 code units — of at most 4096; `flags` must be zero. A guest that declares any text field must export `(func (export "AE_text_event") (param id i32) (param index i32) (param scalar i32) (param phase i32) (result i32))`, and configure is rejected without it. During a changed UI snapshot, `AE_text_field_place_q16` places that ID inside an already-declared panel using the same absolute Q16.16 geometry as every other control.

The guest never receives a byte buffer and the host never writes into guest memory. One complete value arrives as an ordered run of integer events: `index` is the zero-based scalar position, `scalar` is the Unicode scalar value at that position, and `phase` is `0`. A terminating call with `index = -1` carries the authoritative scalar count in `scalar` and the edit phase in `phase`: `1` for a continuous change and `2` for the committed edge. A value of length zero sends only that terminator, which is how a cleared field is expressed. The host rejects any index at or beyond the declared capacity, any count above it, and any code point that is not a Unicode scalar value — surrogates `D800`-`DFFF` and anything above `10FFFF`. Treat the terminator as the transaction boundary and swap the guest-side buffer there rather than acting on a partial run.

Focusing a placed text field is the only thing in Aedicule that may raise a mobile software keyboard: the platform text widget takes focus, GPUI installs its input handler, and the browser backend then moves DOM focus to its editable element. Ordinary canvas interaction leaves no editable element focused, so a touch on non-text content presents no keyboard.

In v0.5 the platform widget owns its edit buffer. A placement carries geometry only, so a guest cannot set, clear, or restore the displayed text, and a reload discards it; a guest that needs to own the value must wait for a later revision of this profile. Headless `aedicule-render --text ID=VALUE` is repeatable, splits at the first `=` so a value may itself contain `=` or be empty, uses phase `2`, and executes after initialization but before requested ticks and rendering.

##### `AE_title`

```wat
(func $AE_title (param ptr i32) (param len i32) (result i32))
```

Sets the UTF-8 application title during `AE_configure`.

##### `AE_menu_item`

```wat
(func $AE_menu_item (param id i32) (param ptr i32) (param len i32) (param shortcut i32) (param flags i32) (result i32))
```

Declares an action menu item; `flags & 1` is a separator and shortcut codes are host-defined.

##### `AE_action`

```wat
(func $AE_action (param id i32) (param label_ptr i32) (param label_len i32) (param flags i32) (result i32))
```

Declares a labeled action for native buttons without creating a menu item; version 0 requires `flags = 0`.

##### `AE_pause_trigger`

```wat
(func $AE_pause_trigger (param kind i32) (param code i32) (param flags i32) (result i32))
```

Registers one typed host-consumed pause/wake trigger during `AE_configure`; version 0 supports stable key kind `1` and requires `flags = 0`.

##### `AE_motion_interest`

```wat
(func $AE_motion_interest (param kind i32) (param rate_hz i32) (param flags i32) (result i32))
```

Registers one motion-sensor interest during `AE_configure`: kind `1` is the host-derived shake gesture (rate must be 0), kind `2` is the six-axis sample stream (rate 0 selects the 60 Hz default, else 1..=120); `flags = 0`. Interests are accepted even where sensors are absent or permission is denied, in which case no motion events ever arrive - keep a manual affordance fallback.

##### `AE_slider_i32`

```wat
(func $AE_slider_i32 (param id i32) (param label_ptr i32) (param label_len i32) (param min i32) (param max i32) (param step i32) (param initial i32) (result i32))
```

Declares an exact stepped integer slider during `AE_configure`; changes arrive through `AE_control_event`.

##### `AE_external_link`

```wat
(func $AE_external_link (param id i32) (param label_ptr i32) (param label_len i32) (param url_ptr i32) (param url_len i32) (param flags i32) (result i32))
```

Declares one labeled, credential-free HTTPS destination during `AE_configure`; version 0 requires `flags = 0`.

##### `AE_ui_begin`

```wat
(func $AE_ui_begin (param revision i32) (result i32))
```

Begins one guest-authoritative keyed native-UI snapshot during `AE_render`; the revision must differ from the last accepted snapshot.

##### `AE_ui_end`

```wat
(func $AE_ui_end (result i32))
```

Completes the current native-UI snapshot, which is published atomically only if the surrounding render also succeeds.

##### `AE_control_panel_q16`

```wat
(func $AE_control_panel_q16 (param id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param rgba i32) (param flags i32) (result i32))
```

Places a guest-sized native-control panel in the current UI snapshot using Q16.16 logical-pixel geometry; version 0 requires `flags = 0`.

##### `AE_slider_place_q16`

```wat
(func $AE_slider_place_q16 (param id i32) (param panel_id i32) (param value i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param label_placement i32) (param flags i32) (result i32))
```

Places a declared native integer slider in the current UI snapshot; the guest-provided value and Q16.16 bounds remain authoritative.

##### `AE_button_place_q16`

```wat
(func $AE_button_place_q16 (param id i32) (param panel_id i32) (param action_id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param flags i32) (result i32))
```

Places a declared action as a keyed native button; flag bit 0 is the guest-authored selected state and all other bits are reserved.

##### `AE_external_link_place_q16`

```wat
(func $AE_external_link_place_q16 (param id i32) (param panel_id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param flags i32) (result i32))
```

Places one declared external link in a guest-owned panel using Q16.16 logical-pixel geometry; version 0 requires `flags = 0`.

##### `AE_text_field`

```wat
(func $AE_text_field (param id i32) (param label_ptr i32) (param label_len i32) (param max_scalars i32) (param flags i32) (result i32))
```

Declares one bounded text-entry control during `AE_configure`; committed text arrives as integer scalar events, never as a host write into guest memory. Version 0 requires `flags = 0`.

##### `AE_text_field_place_q16`

```wat
(func $AE_text_field_place_q16 (param id i32) (param panel_id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param flags i32) (result i32))
```

Places one declared text field in a guest-owned panel using Q16.16 logical-pixel geometry; version 0 requires `flags = 0`.

##### `AE_sin_cos_turn`

```wat
(func $AE_sin_cos_turn (param angle_turn i32) (result sin_q30 i32) (result cos_q30 i32))
```

Returns deterministic Q1.30 sine and cosine for a wrapping binary angle where `2^32` units are one turn.

##### `AE_synth_voice`

```wat
(func $AE_synth_voice (param program_id i32) (param waveform i32) (param delay_ms i32) (param duration_ms i32) (param frequency_start_millihz i32) (param frequency_mid_millihz i32) (param frequency_end_millihz i32) (param gain_start_ppm i32) (param gain_peak_ppm i32) (param gain_end_ppm i32) (param filter i32) (param filter_start_millihz i32) (param filter_end_millihz i32) (param cooldown_ms i32) (result i32))
```

Declares a bounded synthesized-audio program during `AE_configure`.

##### `AE_sample_asset`

```wat
(func $AE_sample_asset (param id i32) (param path_ptr i32) (param path_len i32) (param flags i32) (result i32))
```

Binds a unique sampled-audio ID to one bounded FLAC under the application `assets/` virtual root during `AE_configure`; version 0 requires `flags = 0`.

##### `AE_image_define`

```wat
(func $AE_image_define (param id i32) (param ptr i32) (param len i32) (param flags i32) (result i32))
```

Defines an image resource; `flags & 1` selects raw RGBA, otherwise bytes are encoded image data.

##### `AE_image_release`

```wat
(func $AE_image_release (param id i32) (result i32))
```

Releases a previously defined image resource.

##### `AE_frame_begin`

```wat
(func $AE_frame_begin (param r f32) (param g f32) (param b f32) (param a f32) (result i32))
```

Begins one render transaction with normalized RGBA background components.

##### `AE_frame_begin_rgba`

```wat
(func $AE_frame_begin_rgba (param rgba i32) (result i32))
```

Begins one render transaction with a packed `0xRRGGBBAA` background for integer-only guests.

##### `AE_transform_push`

```wat
(func $AE_transform_push (param m11 f32) (param m12 f32) (param m21 f32) (param m22 f32) (param tx f32) (param ty f32) (result i32))
```

Pushes an affine transform inside the current frame.

##### `AE_transform_pop`

```wat
(func $AE_transform_pop (result i32))
```

Pops the current frame transform.

##### `AE_path_begin`

```wat
(func $AE_path_begin (param id i32) (result i32))
```

Begins a uniquely identified path in the current frame.

##### `AE_path_move`

```wat
(func $AE_path_move (param x f32) (param y f32) (result i32))
```

Appends a move segment to the active path.

##### `AE_path_move_q16`

```wat
(func $AE_path_move_q16 (param x_q16 i32) (param y_q16 i32) (result i32))
```

Appends a move segment using signed Q16.16 logical-pixel coordinates.

##### `AE_path_line`

```wat
(func $AE_path_line (param x f32) (param y f32) (result i32))
```

Appends a line segment to the active path.

##### `AE_path_line_q16`

```wat
(func $AE_path_line_q16 (param x_q16 i32) (param y_q16 i32) (result i32))
```

Appends a line segment using signed Q16.16 logical-pixel coordinates.

##### `AE_path_quad`

```wat
(func $AE_path_quad (param cx f32) (param cy f32) (param x f32) (param y f32) (result i32))
```

Appends a quadratic Bézier segment to the active path.

##### `AE_path_cubic`

```wat
(func $AE_path_cubic (param c1x f32) (param c1y f32) (param c2x f32) (param c2y f32) (param x f32) (param y f32) (result i32))
```

Appends a cubic Bézier segment to the active path.

##### `AE_path_close`

```wat
(func $AE_path_close (result i32))
```

Closes the active path.

##### `AE_path_end`

```wat
(func $AE_path_end (param width f32) (param fill_rgba i32) (param stroke_rgba i32) (param flags i32) (result i32))
```

Commits the active path; version 0 requires `flags = 0`.

##### `AE_path_end_q16`

```wat
(func $AE_path_end_q16 (param width_q16 i32) (param fill_rgba i32) (param stroke_rgba i32) (param flags i32) (result i32))
```

Commits an integer-profile path with Q16.16 width; a zero packed color disables that paint and version 0 requires `flags = 0`.

##### `AE_sprite`

```wat
(func $AE_sprite (param id i32) (param image_id i32) (param src_x f32) (param src_y f32) (param src_w f32) (param src_h f32) (param dst_x f32) (param dst_y f32) (param dst_w f32) (param dst_h f32) (param pivot_x f32) (param pivot_y f32) (param tint_rgba i32) (param flags i32) (result i32))
```

Draws a bounded source rectangle from a defined image; version 0 requires `flags = 0`.

##### `AE_line`

```wat
(func $AE_line (param id i32) (param x1 f32) (param y1 f32) (param x2 f32) (param y2 f32) (param width f32) (param rgba i32) (result i32))
```

Draws a uniquely identified line.

##### `AE_circle`

```wat
(func $AE_circle (param id i32) (param x f32) (param y f32) (param radius f32) (param width f32) (param rgba i32) (param flags i32) (result i32))
```

Draws a uniquely identified circle; `flags & 1` fills it.

##### `AE_text`

```wat
(func $AE_text (param id i32) (param ptr i32) (param len i32) (param x f32) (param y f32) (param size f32) (param rgba i32) (param flags i32) (result i32))
```

Draws UTF-8 text; `flags & 1` centers it.

##### `AE_text_font`

```wat
(func $AE_text_font (param id i32) (param ptr i32) (param len i32) (param x f32) (param y f32) (param size f32) (param rgba i32) (param font i32) (param flags i32) (result i32))
```

Draws UTF-8 text with an explicit portable face: `font = 0` is the platform default, `1` is bundled Geist Mono Regular, and `flags & 1` centers it.

##### `AE_text_font_q16`

```wat
(func $AE_text_font_q16 (param id i32) (param ptr i32) (param len i32) (param x_q16 i32) (param y_q16 i32) (param size_q16 i32) (param rgba i32) (param font i32) (param flags i32) (result i32))
```

Integer-profile counterpart to `AE_text_font`; position and size are signed Q16.16 and the stable face selectors are identical.

##### `AE_frame_end`

```wat
(func $AE_frame_end (result i32))
```

Completes the current frame after all path and transform stacks balance.

##### `AE_audio`

```wat
(func $AE_audio (param id i32) (param volume f32) (param pitch f32) (param flags i32) (result i32))
```

Queues one declared synthesized-audio program with volume 0..1 and pitch 0.25..4.

##### `AE_sample_play`

```wat
(func $AE_sample_play (param id i32) (param volume f32) (param pitch f32) (param flags i32) (result i32))
```

Queues one declared immutable sample with volume 0..1 and pitch 0.25..4; playback is one-way and version 0 requires `flags = 0`.

##### `AE_effect`

```wat
(func $AE_effect (param kind i32) (param a i32) (param b i32) (result i32))
```

Queues a host effect: redraw (1), quit (2), set cursor (3), or persist snapshot (4).

##### `AE_log`

```wat
(func $AE_log (param level i32) (param ptr i32) (param len i32) (result i32))
```

Accepts a bounded UTF-8 diagnostic message; version 0 does not expose its sink to the guest.

#### Portable text faces

`AE_text` remains source-compatible and uses the active platform UI face. `AE_text_font` and `AE_text_font_q16` accept stable selector `0` for that platform default or `1` for Aedicule's embedded Geist Mono Regular. Selector `1` is registered from identical OFL-1.1 font bytes in native and browser adapters, so aligned numerical data never depends on host installation. Unknown selectors reject the complete render transaction rather than silently substituting a proportional face. Package-supplied font handles are not part of ABI v0.8; they require bounded `.aed` asset transport and collision-safe family identities in every adapter.

#### Stable draw IDs

Every non-composition draw ID is unique **across all primitive kinds within one frame**: a path, line, circle, text, or sprite cannot reuse another primitive's ID. The set resets at the next successful frame begin, so the same semantic object should normally reuse its ID in later frames.

#### Frame rules

Call exactly one of `AE_frame_begin` or `AE_frame_begin_rgba` once per `AE_render`, optionally submit a changed retained AVP native-controls document, produce drawing commands, then call `AE_frame_end`. Transform pushes/pops and paths must balance. Any invalid, partial, or over-budget drawing/UI frame is rejected atomically; a host adapter never paints a partial frame.

#### Integer-only lifecycle profile

A guest may replace both floating-point lifecycle exports with `(func (export "AE_init_i32") (param seed_lo i32) (param seed_hi i32) (param width_q16 i32) (param height_q16 i32) (result i32))` and `(func (export "AE_event_i32") (param kind i32) (param code i32) (param a i32) (param b i32) (result i32))`. The pair is atomic: exporting only one is invalid. Viewport and coordinate values use signed Q16.16. This profile lets a guest omit every WebAssembly `f32`/`f64` type and opcode.

#### Minimal integer-only shape

```wat
(module
  (import "aedicule.v0" "AE_frame_begin_rgba" (func $begin (param i32) (result i32)))
  (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
  (memory (export "memory") 1)
  (func (export "AE_abi_major") (result i32) i32.const 0)
  (func (export "AE_abi_minor") (result i32) i32.const 1)
  (func (export "AE_configure") (result i32) i32.const 0)
  (func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
  (func (export "AE_event_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
  (func (export "AE_tick") (param i32) (result i32) i32.const 0)
  (func (export "AE_render") (result i32)
    i32.const 255 call $begin drop
    call $end)
  (func (export "AE_state_ptr") (result i32) i32.const 0)
  (func (export "AE_state_len") (result i32) i32.const 0)
  (func (export "AE_state_schema") (result i32) i32.const 1))
```
