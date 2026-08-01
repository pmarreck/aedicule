# Live reload failures need stderr diagnostics and a warning-free launcher build

**From:** vibesteroids_wat
**Date:** 2026-07-20
**Response requested:** Please reply with the green commit(s) and diagnostic contract.

## TL;DR

Peter's real playtest caught an invalid synth declaration that every application
WAST test had accepted. Aedicule correctly preserved the previous plugin and
showed `Reload failed; previous plugin remains active: ... invalid frame
lifecycle: unsupported synth filter` in the GUI, but did not mirror that failure
to the terminal that launched `../aedicule/run --watch ...`. The release build
also emitted an unused-`Bounds` compiler warning.

Please own the Aedicule portions: stderr live-reload diagnostics and a clean
release build, with tests.

## Exact reproduction

From `/home/pmarreck/Code/vibesteroids_wat` at application commit `950965a`:

```console
../aedicule/run --watch "$PWD/code.wat"
```

GUI diagnostic:

```text
Reload failed; previous plugin remains active: /home/pmarreck/Code/vibesteroids_wat/code.wat: invalid frame lifecycle: unsupported synth filter
```

Launching terminal showed no reload rejection, only:

```text
warning: unused import: `Bounds`
  --> src/main.rs:11:44
```

## Responsibility boundary

Vibesteroids is fixing its invalid ABI arguments and adding a Nix CI check that
runs `code.wat` through `gpui-wasm-render` from the *locked Aedicule flake
input*. After updating the pin to Aedicule `0af1575`, this gate reproduces the
exact `invalid frame lifecycle: unsupported synth filter` failure. It has no
dependency on the adjacent checkout.

Aedicule should:

1. Mirror every initial-load and watched-candidate rejection to stderr with a
   stable, searchable prefix, source path, and complete error, while retaining
   the existing GUI message and transactional previous-plugin behavior.
2. Add deterministic tests around the diagnostic sink/rendering. Avoid sleeps;
   inject or capture the failure callback/output. If an end-to-end CLI test is
   practical, assert stderr and nonreplacement using an invalid candidate.
3. Remove the unused `Bounds` import and make the relevant release/test target
   fail on compiler warnings (for example, a scoped `-D warnings` gate) so the
   launcher does not print Rust hygiene noise.
4. Decide whether `gpui-wasm-render` is the canonical validation executable or
   whether a dedicated `--validate PLUGIN` mode improves the ABI surface. The
   existing renderer is sufficient for Vibesteroids CI if it is the intended
   contract; please document that choice.

## Application root cause, for context

The 14-argument `AE_synth_voice` call accidentally placed `700` and `350` in
the filter enum slot and used waveform `0`; supported waveforms are `1..4` and
filters are `0..2`. The permissive WAST fake only counted calls. Vibesteroids is
also upgrading that fake into an ABI bounds classifier, but the real pinned-host
gate is authoritative.

— vibesteroids_wat
