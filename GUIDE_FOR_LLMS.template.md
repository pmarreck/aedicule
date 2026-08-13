<!-- Generated from `GUIDE_FOR_LLMS.template.md` plus `src/wat_abi.rs`; do not edit this generated copy by hand. -->

# Guide for LLMs writing Aedicule applications

Guide version: `{{GUIDE_VERSION}}`

Current Aedicule WAT ABI: `{{ABI_VERSION}}`

Canonical guide: {{CANONICAL_URL}}

This document is meant to be sufficient even when you cannot inspect Aedicule's source. It distinguishes the callable ABI from planned work, gives a reliable authoring process, and ends with the exact generated ABI reference. If a copied guide disagrees with a newer canonical guide, use the guide whose ABI version matches the Aedicule host you must support.

An Aedicule **guest** is a WebAssembly Text (`.wat`) application. Aedicule is the **host** or **frontplane**: it schedules deterministic updates, validates capability calls, renders through native or browser adapters, delivers input, plays admitted audio, and can replace a running guest transactionally. A guest receives no ambient filesystem, network, clock, random-number generator, or operating-system access.

## Ground rules

1. Treat the exact generated appendix as the authority for names and signatures. Import only from `aedicule.v0`, use only `AE_*` names, and declare the ABI version exports.
2. Everything under **Available now** is implemented in ABI `{{ABI_VERSION}}`. Everything under **Proposed—not callable yet** is design direction, not an import or export you may emit.
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

`AE_init*` receives a deterministic 64-bit seed as two `i32` halves and the initial logical viewport. Initialize all mutable state there. ABI minor `9` adds the explicit `AE_random_v1_*` deterministic stream and distribution imports described below. Older hosts still require a guest-side PRNG; no host supplies ambient entropy.

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

### Deterministic RandomZ v1

ABI minor `9` provides cross-platform-identical RandomZ v1 bytes, inclusive integer ranges, uniform fixed values, and normal, range-scaled normal, exponential, Poisson, log-normal, and beta distributions. Import names include `v1` because their sequence and byte-consumption behavior are permanent compatibility promises.

Each stream uses a 48-byte guest-owned state region. Seed it from the two `AE_init*` halves with `AE_random_v1_seed_u64`, or from exactly 32 bytes with `AE_random_v1_seed_bytes`. Keep that state inside the exported snapshot region if hot reload must continue the same stream. Multiple independent streams need distinct non-overlapping 48-byte regions.

Batch integer output is packed little-endian `i64`. Batch fixed output uses 12-byte little-endian records containing a canonical signed `i64` mantissa and `i32` binary exponent. Check every import status before reading output. A rejected pointer, parameter, batch, source-consumption limit, or invalid stream leaves output and serialized stream state unchanged. Use bounded batches rather than one host call per particle or sample.

This profile is deterministic. It has no system-entropy capability, and a seed is replay material rather than a password. Do not use it for secrets or security tokens.

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
- Reading ambient time, entropy, filesystem, or network through an undeclared import. Aedicule provides only explicit bounded capabilities; RandomZ v1 is deterministic and replayable.
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

Pin tool versions in reproducible builds. Tool defaults evolve as WebAssembly proposals graduate, while Aedicule ABI `{{ABI_VERSION}}` intentionally exposes a narrower core contract.

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

{{GENERATED_ABI_REFERENCE}}
