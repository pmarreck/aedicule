---
description: "Nix sandboxed CLI scripts need patched interpreter paths."
datetime: 2026-07-17T18:41:42-04:00 # America/New_York (EDT)
tags: [nix, nixos, flakes, sandboxed, cli, scripts, patched, interpreter, paths]
---
Portable project scripts may deliberately use `#!/usr/bin/env bash`, but a
pure Nix build sandbox does not provide `/usr/bin/env`. A derivation that runs
those scripts must call `patchShebangs` on its copied test files before
execution; do not change the checked-in portable shebang just to satisfy Nix.

The Mechatron `checks.x86_64-linux.test` target demonstrated this boundary:
Rust tests passed, then the CLI check failed with a missing `/usr/bin/env`
interpreter until its source-copy shebangs were patched in `checkPhase`.
