---
description: "Mutable WAT belongs outside the native Nix source closure."
datetime: 2026-07-17T00:40:03-04:00 # America/New_York (EDT)
tags: [mutable, wat, wasm, webassembly, outside, native, nix, nixos, flakes, source, closure]
---
The generic frontplane's fallback WAT may be embedded because it is part of the
stable native ABI conformance boundary. A real application such as
Vibesteroids must instead be a runtime-loaded data artifact in its own Nix
derivation. If `plugins/vibesteroids.wat` enters the `buildRustPackage.src`
closure—even through an unused `include_str!`—every gameplay edit invalidates
and relinks the large GPUI Rust derivation.

Keep `packages.frontplane` sourced from an explicit allowlist containing the
Rust manifests, `src/`, and pinned native dependencies; package mutable WAT
under `share/aedicule/plugins/`; and compose them with tiny launcher wrappers
that set `AEDICULE_DEFAULT_APPLICATION`. A structural test must inspect the native
source derivation and reject the application WAT. This protects the boundary
more reliably than timing a build or trusting a source comment.

This does not cache ordinary Rust changes. A separate Cargo dependency-artifact
layer such as crane is still needed for fast clean-sandbox Rust iteration.
