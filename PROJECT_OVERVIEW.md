# Mecha Aedicule

Mecha Aedicule explores a generic, cross-platform GPUI “frontplane” that loads an
application authored in WebAssembly Text Format (WAT). The frontplane owns
platform integration; the plugin owns application state and behavior.

The working tree, package, and binaries retain the provisional `gpui-wasm`
identifier until the generic frontplane and its Vibesteroids application are
split into adjacent repositories. Product naming must not obscure that
technical extraction or silently break existing launch paths.

The initial feasibility demonstration is an Asteroids-style game:

- one WAT module owns the ship, asteroids, bullets, score, lives, and seeded
  deterministic placement;
- a native Rust host embeds Wasmtime;
- a GPUI/gpui-component view presents the window, menu, HUD, and canvas;
- a headless adapter exports the same immutable command buffer as deterministic
  SVG so visual frames remain inspectable without a live desktop;
- the host passes input and fixed ticks into the plugin;
- the plugin calls a small versioned host ABI to emit draw/audio/effect
  commands; and
- transactional live reloads preserve compatible snapshots while rejecting
  broken candidates without interrupting the running application.

The project is not intended to become a full game engine during the spike. It
is intended to discover whether a clean GPUI-facing WASM application ABI is
pleasant, deterministic, containable, and sufficiently expressive.

**POC status:** successful. On 2026-07-16 Peter confirmed that the deployed
Vibesteroids conversion is a playable game. Further work is refinement and
productization rather than proof of basic feasibility. The same-day live-edit
extension also proved that an external `code.wat` can be watched and replaced
while a real GPUI window is running: compatible edits preserve game state,
schema changes restart deliberately, and malformed edits leave the previous
game playable.

**Main branch:** yolo

**i18n phase:** prepare. English is the only populated locale during the
feasibility spike; visible strings are centralized so translation can be
evaluated later without changing the plugin ABI.

## Terms

**Frontplane**

The trusted native host: windowing, GPUI rendering, menus, input, audio,
storage, resource limits, and plugin lifecycle.

**Plugin/application**

The untrusted WAT-authored module. It is called a plugin because the frontplane
loads it generically, even though it may contain the entire application brain.

**Command buffer**

A finite host-owned list of rendering, audio, and effect requests emitted by
the plugin during a lifecycle call.

**Fixed tick**

One deterministic simulation step. Wall-clock sampling and frame pacing belong
to the host; the plugin receives integer tick counts.

**State schema**

A plugin-owned integer compatibility declaration for its opaque snapshot
bytes. The host also checks snapshot length, but only the plugin author can
know whether an equal-length layout or semantic change requires incrementing
the schema and starting fresh state.
