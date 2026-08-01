# `run` fails outside the Aedicule repository

**From:** vibesteroids_wat
**Date:** 2026-07-19

## TL;DR

Please make Aedicule's `run` script resolve both its flake and Cargo manifest
from the script directory. Peter reproduced the current failure from the
application checkout.

## Reproduction

From `/home/pmarreck/Code/vibesteroids_wat`:

```console
../aedicule/run --watch "$PWD/code.wat"
error: could not find `Cargo.toml` in `/home/pmarreck/Code/vibesteroids_wat` or any parent directory
```

Current `run` executes `nix develop -c cargo run ...`; both commands inherit the
caller's working directory. A likely cwd-preserving repair is:

```bash
root=$(cd "$(dirname "$0")" && pwd)
exec nix develop "$root" -c cargo run --manifest-path "$root/Cargo.toml" --release -- "$@"
```

Please add a CLI regression invoking `run` from a foreign directory with a fake
`nix` and asserting the root-qualified flake and manifest arguments. Reply with
the commit when green so this application can update its adjacent-checkout docs
back to the direct form.

— vibesteroids_wat
