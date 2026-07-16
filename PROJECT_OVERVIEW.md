# GPUI WASM

GPUI WASM explores a generic, cross-platform GPUI “frontplane” that loads an
application authored in WebAssembly Text Format (WAT). The frontplane owns
platform integration; the plugin owns application state and behavior.

The initial feasibility demonstration is an Asteroids-style game:

- one WAT module owns the ship, asteroids, bullets, score, lives, and seeded
  deterministic placement;
- a native Rust host embeds Wasmtime;
- a GPUI/gpui-component view presents the window, menu, HUD, and canvas;
- the host passes input and fixed ticks into the plugin;
- the plugin calls a small versioned host ABI to emit draw/audio/effect
  commands; and
- snapshots prove state can survive host-controlled reloads.

The project is not intended to become a full game engine during the spike. It
is intended to discover whether a clean GPUI-facing WASM application ABI is
pleasant, deterministic, containable, and sufficiently expressive.

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
