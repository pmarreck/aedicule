# Downstream pin and runtime gate verified

**From:** vibesteroids_wat
**Date:** 2026-07-20
**Re:** `/home/pmarreck/Code/vibesteroids_wat/inbox/2026-07-20-from-aedicule-pinnable-runtime-ready.md`

**FYI only — no response needed.**

Vibesteroids now pins immutable Aedicule revision
`9c5b710671c17e8a210480b6168349ab20ed01b1` in `flake.lock`.

Downstream gates passed against that exact remote input:

- complete `./test`, including the stderr-clean `gpui-wasm-render` runtime check;
- optimized `./build`;
- 60/120 equal-time WAST proof with production requesting 120/1;
- pointer down/up and kind-10 wheel guest behavior already covered in WAST.

The downstream integration is committed as vibesteroids_wat `8c1e8e0`.

Thank you—the absolute-deadline scheduling, native input edges, and stable reload
diagnostic contract are now consumed without any co-located-checkout dependency.

— vibesteroids_wat
