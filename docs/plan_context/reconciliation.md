# Backlog reconciliation, 2026-10-21

The active queue is [PLAN.md](../../PLAN.md). The old queue mixed completed work,
proposals, superseded hypotheses, and private operations. Its completed entries
are in [PLAN_LOG.md](../PLAN_LOG.md); remaining detailed evidence is in
[backlog-evidence.md](backlog-evidence.md). Those records preserve their historical
states and are not instructions to restart old investigations. Private references
were generalized; the exact pre-migration files have a private recovery copy.

## Reconciled work

- Public iPhone touch and shake were physically accepted on 2026-08-26 at host
  `e17562c`. The downstream report of 2026-08-27 pins that host from guest
  `12103b6fd69a30e424358f211dc4edd34d86bfdb` and reports full tests, optimized build,
  Mechatron Prime CI, and GitHub Actions run `33121360721` passing. This is
  downstream-reported evidence, not a newly executed acceptance run.
- The raw-contact opt-in, device-change bit 0, motion capability, and guest
  touch/shake refinements are delivered. Older requests to design them again are
  retired. Richer modality flags, sensor profiles, and suspension regions remain.
- Ulam Play/Pause was physically accepted on 2026-08-03 after the compatibility
  mouse-echo fix. Zero touch slop and duplicate activation are historical causes,
  not open diagnoses. Selection/callout and text-entry keyboard acceptance remain
  distinct checks.
- Editable-at-startup focus ownership and its vacuous test were replaced by
  canvas/text-handler focus policy and real text-field acceptance. The gallery
  zero-stage timeout was later traced to injecting input before listeners existed.
  Earlier proposals to fix key delivery from an unsuitable probe are not tasks.
- Shared Wasm memory and the isolation service worker were removed in August;
  the legacy worker is retired non-blockingly. Do not resume the older 1 GiB
  reservation/backoff experiment or add a new isolation worker as a cache fix.
  Multi-tab Firefox responsiveness and repeated iOS reload acceptance remain.
- The generic file-picker/dropzone runner, gallery previews, web controls, physical
  keys, middle-button/two-axis scroll, and zero-float lifecycle/control/path profile
  are delivered. Pending visual acceptance and broader forms work remain explicit.
- `.aed` uses the shared Rust package reader on native and web. This supersedes
  both the separate JavaScript ZIP reader and the older BLAR-format decision.
  FLAC playback is delivered; broader image/font/service capabilities remain.
- The guide is generated and copied into release demo directories and web demo
  directories by `flake.nix`, with checks in `tests/cli/document_packages`.
  Coverage inside independently published guest `.aed` files needs coordination.
- Native/browser pure-Nix gates ran after the old storage-maintenance hold was
  released. No open build hold was present in the October inbox. New checks still
  use the shared build budget, with one expensive project job at a time.
- Fenix is the accepted Nix Rust provider. The unpushed implementation pins 1.97.1;
  the browser retains its separately pinned nightly. No provider decision is open.

## Next downstream handoff

Ulam owns its five render-oracle cases. Ask its agent for the current revision and
proof of per-worker paths, deterministic failure replay, multiple-failure
accumulation, and serial cross-case comparisons. The old exploratory measurement
was 39.998 seconds sequential versus 9.044 seconds parallel (4.42x); repeat in its
actual checkout before accepting a performance claim. Do not edit the guest tree
or treat an unanswered handoff as permission to do so.

## Retained constraints

Demo freshness is standing work: update the manifest and provenance tests together
from immutable source/package pairs, verify embedded WAT against source, then run
packaged WAST and relevant browser gates. Never silently change published bytes.

For browser regressions, prefer replayed event streams and real-browser in-page
checks. New driver surfaces were explicitly disfavored on 2026-08-05 and remain
last resorts; the established Chromium CDP lane stays. Physical acceptance of one
scenario does not prove Firefox concurrency, all WebKit input, or speaker output.

Fullscreen remains host-controlled on native with an unshadowable Shift-Escape
exit and mandatory temporary notice; native aliases are configurable. The browser
owns web fullscreen entry/exit and shortcuts. The later browser-owned decision
supersedes the earlier proposal for a host-created mobile fullscreen button.

The release adapter binds a pushed source revision, version/tag, independently
reproduced six-target archives, and optional private installation. Signing must
follow bundle assembly and validate nested code, duplicate dylib paths, strict
signatures, startup, notarization, and stapling. A native macOS release lane is the
accepted distribution direction; cross-built macOS output is diagnostic until
that gate is established. Credentialed signing remains separate authorization.

The historical overview's claim of a successful proof of concept is retained in
INTENT.md, with pre-production limitations. Terms moved to TERMINOLOGY.md.
