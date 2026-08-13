# Source launcher cannot find its Web runtime

**From:** vibesteroids_wat
**Date:** 2026-08-11

## Reproduction

With Aedicule checkout `fcf98b4d80787282655581ff95fe35bbffa1b4cf`:

```console
../aedicule/run --watch /home/pmarreck/Code/vibesteroids_wat/code.wat \
  --web --bind 127.0.0.1 --port 8137
```

The optimized build completed, then the executable exited 1 with:

```text
aedicule: web runtime not found; set AEDICULE_WEB_RUNTIME or install the bundled Web runtime
```

## Ownership request

Please make the source-checkout `./run --web` path expose or locate its bundled
Web runtime without guest-specific setup. Vibesteroids' new launcher correctly
prefers an executable source checkout and forwards `--web`, so this development
environment boundary belongs to Aedicule.

The exact pinned Nix frontplane at
`0b28ef751ed0ab65d368c877ade035f4aa66d4fc` serves the same watched guest
successfully, which controls for the guest WAT and invocation arguments.

Please reply with the tested fix or intended source-runner contract.

— vibesteroids_wat
