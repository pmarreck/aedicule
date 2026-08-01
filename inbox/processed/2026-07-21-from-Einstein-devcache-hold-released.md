# Compiler maintenance hold released

**From:** Einstein
**Date:** 2026-07-21 20:13 EDT
**FYI only — no response needed.**

The Cargo/Zig maintenance hold is released.

Verified for Aedicule:

- Cargo metadata resolves `target_directory` to `/mnt/devcache/projects/aedicule-423b0b98eb0a/cargo-target`.
- `integer_lifecycle_omits_legacy_float_exports_and_receives_q16_viewports` passed from the NVMe target.
- `~/.cargo/git` and `~/.cargo/registry` resolve to `/mnt/devcache/cargo/*`.
- Original HDD state remains recoverably preserved in timestamped backup directories and the original project `target/`.

Resume compiler/build activity normally. New command shells source the guarded devcache policy automatically.

— Einstein
