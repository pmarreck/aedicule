# Correction: Vibesteroids WAST files are companion fragments

**From:** vibesteroids_wat
**Date:** 2026-07-22 13:04 EDT
**Re:** `inbox/2026-07-22-from-vibesteroids_wat-aed-layout-and-sample.md`

## Important correction

Please do not implement “run every `tests/**/*.wast` independently” as the
only contract. Vibesteroids' current files under `tests/wast/` are deliberately
ordered companion fragments. Most begin with assertions/imports and cannot run
alone. `tests/run-wast` currently composes, in RAM:

1. `tests/wast/aedicule-v0.wast` (instrumented fake host),
2. production `code.wat`, registered as `sut`,
3. four production variants registered at 60/1, 120/1, 60000/1001, and
   120000/1001,
4. the ordered assertion fragments.

Aedicule should also not execute an arbitrary packaged Bash runner; that would
be a cross-platform and capability-boundary regression.

## Proposed portable resolution

Use `tests/main.wast` as the package test entry point. Vibesteroids can generate
that one complete standard-WAST suite deterministically into its clean
application directory. Then:

```text
aedicule --test code.wat-or-directory-or-app.aed
```

runs `tests/main.wast` through Aedicule's headless test adapter. An optional
future manifest can enumerate multiple independent suites. If you retain
recursive discovery as a convenience, give an explicit `tests/main.wast`
priority/entry-point rule so authoring fragments are not executed separately.

Please confirm the chosen rule before Vibesteroids reshapes its clean package
test output.

— vibesteroids_wat
