# Browser delivery

Aedicule supplies a browser **runtime**, not a JavaScript reimplementation of
an application. The host compiles to WebAssembly once; a delivery bundle adds
an ordinary `code.wat` file that the browser fetches before starting that host.
The WAT stays application data and follows the same `aedicule.v0` ABI as native
delivery.

## Build and serve the conformance bundle

```sh
nix build .#webFallback
nix run .#webServe -- ./result
```

Then open <http://localhost:8080>. `webServe` deliberately sends
Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers; opening
the directory with `file://` or a generic server without those headers will
not work because GPUI's browser backend uses shared WebAssembly memory.

## Headless startup diagnosis

Once a bundle is being served, a dependency-free Chrome DevTools Protocol
probe can stream its structured startup stages and verify that initialization
produced a drawable canvas:

```sh
./tests/integration/web_browser_startup http://localhost:8080/
```

The probe requires Node 22 or newer plus a Chromium-family browser. Set
`AEDICULE_CHROME` when Chromium is not discoverable on `PATH`. Successful
startup reports `bootstrap`, browser capabilities, the WebGPU adapter, WAT
fetch, Wasm initialization, the first animation frame, and the settled canvas
snapshot as JSON lines. An uncaught browser exception, visible failure status,
missing stage, or canvas whose backing buffer remains `1x1` makes the probe
exit nonzero.

After startup, the probe synthesizes pointer movement, primary/middle/secondary
button edges, two-axis wheel motion, and an Arrow Left key edge through Chrome's
DevTools input domain. It also drives a simultaneous left-up/right-down
two-finger gesture, releases both contacts, and cancels a third contact. Its
default output is one compact `input-summary` JSON record. Set
`AEDICULE_BROWSER_TRACE_EVENTS=1` to print every dispatched and
document-observed event while diagnosing identity, phase, ordering, or
coordinate problems.

## Compose an application bundle

A downstream Nix flake can make its own immutable deployment artifact without
forking Aedicule's host:

```nix
packages.${system}.web = aedicule.lib.${system}.webBundle {
  wat = ./code.wat;
  title = "Vibesteroids";
};
```

The resulting directory contains `index.html`, `bootstrap.js`,
`aedicule_web.js`, `aedicule_web_bg.wasm`, and the supplied `code.wat`. The
bootstrap loads and validates the UTF-8 WAT document before invoking the
browser host; a missing or blank document is an error, never a silent fallback
to Aedicule's conformance guest.

## Payload budget

The release Wasm passes through `wasm-opt --enable-threads -Oz` after
`wasm-bindgen`; shared-memory support remains explicit. The delivery smoke
test rejects an uncompressed `aedicule_web_bg.wasm` larger than 11 MiB. On the
current host, the optimized runtime measures 10.0 MB raw and 3.61 MB with
gzip -9, before the tiny application-specific `code.wat` document.

## Current boundary

This delivery slice runs initialization, exact rational scheduled ticks, and
validated command frames through the portable, fuel-metered Wasmi runtime.
GPUI Web animation frames supply advisory monotonic timestamps; the shared
Aedicule scheduler still determines fixed ticks and input ordering. Pointer
movement, primary-button presses, and changed logical viewport sizes travel
through the shared `AE_event` ABI. Keyboard/focus forwarding, PCM/audio output,
and packaged images remain the next browser adapter slices; none will become
browser-only JavaScript behavior.
