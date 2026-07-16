# gpui-wasm

A feasibility spike for a generic native GPUI frontplane that runs
capability-bounded applications written directly in WebAssembly Text (WAT).
The bundled demo is a behavioral conversion of
[Vibesteroids](https://github.com/pmarreck/vibesteroids): game state and rules
live in WAT; Rust/GPUI supplies the window, input, vector painter, menus, audio,
and sandbox.

## Try it

~~~console
./test       # deterministic headless ABI, game, and containment tests
./build      # optimized, reproducible Nix build
./run        # optimized local launch from the Nix development shell
result/bin/gpui-wasm-render --ticks 300 -o frame.svg
             # deterministic visual artifact; accepts an external .wat path
~~~

Controls: Left/Right rotate, Up thrusts, Space fires, P pauses, and R starts a
new game. New Game and Quit are also available from the window and Game menu.

`gpui-wasm-render` runs without a window. It accepts `-` or `@stdin` as its WAT
input and `-`, `@stdout`, or `@stderr` as output, making exact plugin frames
available to CI, review agents, image converters, and visual-regression tools.
Run it with `--help` for viewport, seed, tick, and output options.

The design and honest spike boundaries are in [SPEC.md](SPEC.md). In
particular, affine paths and sprite-sheet commands are proven independently of
Asteroids; full decoded sprite-atlas painting remains an adapter follow-up.

## Why WAT?

WAT is a compact, explicit target an LLM can generate without requiring the
frontplane to understand the source language or ship an application-specific
runtime. It is not intended to replace Rust for ordinary human-authored
software. This project tests whether a small deterministic ABI can make WAT a
useful portable plugin language while keeping native window behavior.

## License

MIT. The local `ztracing` compatibility crates are Apache-2.0 and replace a
GPL-only optional instrumentation path that GPUI does not use in its default
mode; see their provenance note.
