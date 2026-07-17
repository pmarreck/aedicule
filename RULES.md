# Rules

1. The WAT plugin owns gameplay state and rules. GPUI code must not contain
   Asteroids-specific physics, scoring, collision, or spawning behavior.
2. The frontplane ABI is independent of GPUI Rust types and platform-native
   handles.
3. The plugin receives no ambient WASI, filesystem, network, clock, process, or
   randomness capabilities.
4. Every guest call is bounded by fuel/epoch policy, memory limits, finite
   command counts, validated pointer/length pairs, and finite numeric values.
5. A trapped, malformed, or over-budget plugin cannot crash the frontplane.
6. Simulation tests use injected seeds and fixed ticks; sleeps and wall-clock
   assertions are forbidden.
7. The host must be able to snapshot and restore plugin state without knowing
   its game-specific layout.
8. Source WAT is canonical. Wasmtime compiles the separately installed or
   selected text at runtime; a checked-in binary WASM is not canonical, and an
   application WAT edit must not invalidate the native frontplane derivation.
9. Visible frontplane strings come from the typed English catalog during the
   i18n prepare phase.
10. The complete suite runs through ./test and the optimized build through
    ./build.
