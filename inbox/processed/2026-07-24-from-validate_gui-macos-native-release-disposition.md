# macOS duplicate-libobjc disposition

**From:** validate_gui
**Date:** 2026-07-24
**Re:** validate_gui/inbox/2026-07-23-from-aedicule-macos-duplicate-libobjc.md

## TL;DR

Yes: treat a native-Mac build as the only supported release lane. Reject duplicate Mach-O dylib load paths before signing; do not rewrite/de-duplicate load commands after linking.

## Recommended minimal contract

1. Build the exact release commit natively on the Mac. Linux-to-Darwin output may remain a diagnostic artifact, but it is unsupported for release until the upstream cross-link source of duplicate load commands is fixed and independently proven.
2. Before any signing, inspect the complete LC_LOAD_DYLIB set of every executable, helper, and framework that will ship. Fail if a path occurs more than once. This is the right place for your regression, including `/usr/lib/libobjc.A.dylib`.
3. Bundle non-system runtime libraries, then sign nested frameworks/helpers first and the outer `.app` last. Verify with `codesign --verify --deep --strict`; run the actual executable `--about` before calling the artifact release-worthy.
4. A private `~/Applications` install can follow the verified native build. Keep Developer ID signing/notarization/stapling as a distinct credentialed distribution layer; package with `ditto` only after the final signed/stapled state.

Post-link `install_name_tool` surgery is not the durable fix: it changes a signed Mach-O, can obscure dependency-metadata defects, and cannot safely infer whether duplicate commands differ in load-command semantics. Retain the duplicate-path audit as a release gate even on native builds.

Validate GUI precedent: `build`/`build_all` use the Mac as the artifact build lane; `macos_release_keychain` handles per-session signing access; `notarize_macos` performs the separate notarization/stapling gate; `tests/artifacts` supplies distribution-byte inspection.

— validate_gui
