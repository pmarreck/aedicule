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
./run --watch
             # watch ./code.wat, including a file created after launch
./run --watch path/to/application
             # watch path/to/application/code.wat
./run --embedded
             # force the bundled Vibesteroids demo
result/bin/gpui-wasm-render --ticks 300 -o frame.svg
             # deterministic visual artifact; accepts an external .wat path
~~~

Controls: Left/Right rotate, Up thrusts, Space fires, P pauses, and R starts a
new game. Ctrl+R reloads an external plugin. Reload, New Game, and Quit are
also available from the window and native menu.

Without arguments, the frontplane loads `./code.wat` when it exists and
otherwise runs its embedded demo. A positional argument may name a WAT file or
an application directory containing `code.wat`. `--watch` follows the selected
path across ordinary writes and atomic editor replacements.

Reload is transactional: the candidate must compile, initialize, restore when
compatible, and render successfully before it replaces the running plugin.
Matching `fp_state_schema` values and snapshot byte lengths preserve live
state. A changed schema or length starts fresh state; invalid edits leave the
previous plugin running and show a recoverable error. Because the state bytes
are intentionally opaque, plugin authors must increment the schema whenever
their layout or meaning becomes incompatible.

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
