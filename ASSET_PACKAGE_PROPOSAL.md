# Aedicule Asset Package Proposal (v0)

## Decision

The normal distributable/runnable unit will be one self-contained `*.aed`
application archive. Version 0 is a deterministic ZIP containing a root
`code.wat`, optional immutable assets, and arbitrary project documentation.
Raw `.wat` remains directly runnable for tiny programs, tests, and the
edit-run loop; it is not forced through a package when there is nothing to
package.

The runnable container deliberately does **not** use BLAR yet. ZIP has mature
cross-platform and browser readers, per-entry random access, and ordinary
inspection tools. The first profile stores every entry without generic ZIP
compression: FLAC and normal image formats already compress their own bytes,
while WAT and documentation are small. This keeps package output byte-stable
without making a compressor version part of the format. A later profile may
add deterministic Deflate for text when measured package size justifies it.

Like EPUB, `.aed` retains ZIP's normal `PK` header and reserves its first entry
as an uncompressed `mimetype` sentinel. Its exact content is
`application/vnd.aedicule.app+zip`. Aedicule generates and validates this
container metadata; it is not application-visible and is omitted by
depackaging. A custom pre-ZIP byte prefix is deliberately avoided so standard
ZIP tools remain interoperable.

`blar` owns media preparation. For now, it converts source WAV/AIFF to
lossless FLAC and records whatever reconstruction metadata belongs to the
archive format. Aedicule receives only that finished entry, identifies FLAC,
and decodes it under host limits. It never chooses a codec, transcodes source
media, or treats its archive compressor as a substitute for an audio codec.
Ogg and MP3 are explicitly deferred pending a later `blar` design discussion.

BLAR remains the proposed media-preparation/content-optimization layer. It may
turn authoring WAV/AIFF/MP3 into FLAC before the resulting file enters the ZIP;
it is not required to parse or launch an Aedicule application.

## Accepted archive shape

The source directory and archive share this conventional shape:

```
vibesteroids/
  code.wat
  aedicule.toml
  assets/
    greta.flac
    ...
  lib/
    ... optional future linked WAT modules ...
  tests/
    ... WAST and companion acceptance fixtures ...
  README.md
  LICENSE
  ... arbitrary regular files ...
```

- `code.wat` is the version-0 entry point and is required for directory or
  package launch. A future manifest may select another entry point without
  changing WAT's ABI-version declaration.
- `aedicule.toml` is an optional, host-recognized application manifest. The
  specific name avoids stealing a guest-owned generic `config.toml`.
- `assets/` is the only namespace admitted by initial image/sample declaration
  imports. Other files remain packaged but are not arbitrary guest-readable
  filesystem capabilities.
- `lib/` is reserved for future module-linking. Its WAT files are carried now
  but are not implicitly compiled, imported, or granted authority.
- Direct `tests/*.wast` files are independently executable application suites.
  `tests/main.wast` is the conventional single-suite name. Nested WAST files
  are composition fragments. Normal launch ignores them; `aedicule --test`
  executes direct suites in PCG32/Fisher-Yates order, always emits the seed,
  and accepts that seed again for exact replay.
- The ZIP accepts regular stored files and directory entries only. Symlinks,
  device files, encrypted entries, ZIP64, multipart archives, absolute paths,
  and unknown compression methods are rejected.
- Paths are unique, NFC UTF-8, relative, slash-separated, and contain no
  empty, `.`, or `..` component. Names are never extraction paths.
- Package creation sorts names bytewise, fixes timestamps, normalizes portable
  modes, and rejects filesystem races or unsupported types. Depackaging applies
  the same validation before writing anything and refuses to merge into an
  existing nonempty destination by default.
- The reader validates central-directory bounds, local-header agreement, CRC32,
  entry counts, and configured byte limits before handing bytes to a decoder.

## Limits

The reader receives explicit host limits rather than trusting archive metadata:

| Limit | Initial default | Why |
| --- | ---: | --- |
| Whole package | 64 MiB | bounded startup/download work |
| File entries | 1,024 | bounds index and name scans |
| One stored entry | 16 MiB | caps texture or clip residency |
| Total image bytes | 16 MiB | keeps the existing image budget authoritative |
| Decoded image pixels | 16 million | protects image decoders from pixel bombs |
| Decoded PCM per clip | 32 MiB | bounds mixing memory; long music streaming is deferred |
| Active sampled voices | 64 | prevents unbounded mixer work |

Host configuration may lower these limits. An application cannot raise them.

## Application manifest and profiles

Host policy that must exist before guest instantiation belongs in root
`aedicule.toml`:

```toml
[application]
abi = "0.1"
default_profile = "prod"

[runtime]
wasm_gc = "auto" # auto | disabled | null | collecting

[values]
difficulty = "normal"

[profiles.test.values]
difficulty = "deterministic"

[profiles.dev.runtime]
wasm_gc = "null"
```

Profile selection is deterministic: explicit `--profile NAME`, then
`AEDICULE_PROFILE`, then `application.default_profile`, then `prod`. `--test`
selects `test`; convenience `--dev` and `--prod` aliases may select those
profiles. A later conflicting explicit option follows the CLI's normal
later-argument-wins rule.

The host overlays the selected profile onto base manifest keys before it
validates or instantiates `code.wat`. Guest code may read only typed `[values]`
after overlay. Runtime, capability, path, and security policy remains
host-only, and ambient environment variables or secrets are never reflected
into guest values. Unknown keys and type-changing overrides fail rather than
being silently ignored.

WebAssembly GC does not collect ordinary Wasm linear memory. Collector choice
is immutable engine configuration, so a manifest may request it only before
instantiation; a running guest cannot toggle it. `auto` may inspect validated
module features, `disabled` rejects Wasm-GC types, `null` admits types without
collecting, and `collecting` requires an available host collector. The host
may reject unsupported or policy-forbidden requests.

## WAT-facing capability shape

The eventual ABI additions should operate on validated package names, not WAT
memory payloads:

```wat
;; configure-time, id must be unique
(import "aedicule.v0" "AE_image_asset"
  (func (param id i32 path_ptr i32 path_len i32 flags i32) (result i32)))
(import "aedicule.v0" "AE_sample_asset"
  (func (param id i32 path_ptr i32 path_len i32 flags i32) (result i32)))

;; runtime
(import "aedicule.v0" "AE_sample_play"
  (func (param id i32 volume f32 pitch f32 flags i32) (result i32)))
```

`AE_image_asset` feeds the existing `AE_sprite` resource model. `AE_sample_*`
is distinct from `AE_synth_voice`/`AE_audio`, preserving synthesized programs
and digitized clips as different bounded capabilities. Both declarations happen
during `AE_configure`; runtime calls only choose already admitted IDs. Audio
output is one-way: device buffering or playback position never becomes an
event, timer, or guest-visible clock.

These names are proposed, not yet part of ABI v0.0. They require the normal
hard cutover and generated `WAT_ABI.md` update when implemented.

## Media rollout

1. **Images:** encoded PNG/JPEG/WebP bytes admitted by `AE_image_asset` and
   decoded by the existing native/browser rendering adapters; raw RGBA remains
   available only through the existing bounded memory ABI.
2. **Audio source preparation:** `blar` converts source WAV/AIFF into FLAC.
   This lossless path is the only packaged digitized-audio delivery format for
   now. Ogg and MP3 are deferred; if added, they belong in this same `blar`
   media-preparation surface, not in Aedicule.
3. **Aedicule decoders:** RIFF/WAVE integer PCM remains a bounded fallback for
   deliberately raw, latency-sensitive clips. FLAC is the next package decoder
   and uses the same post-decode channel/rate/duration limits. The packaged
   bytes are stored directly; generic archive compression is skipped for FLAC.
   Unsupported formats, including Ogg and MP3 for now, fail declaration
   atomically; they never silently fall back.
4. **Streaming:** intentionally deferred. It needs a seekable per-entry
   profile, a fixed decode ring budget, and a test proving that no device timing
   leaks back into deterministic WAT execution.

## First concrete package candidate

Vibesteroids schema 10 provides the first end-to-end packaging requirement.
After player-attributed destruction of Voyager, the guest wants to trigger one
immutable sample exactly one simulated second after the reactor explosion
begins. Voyager/asteroid collisions must never schedule it; the guest owns that
attribution and fixed-tick timing. The host owns only declaration, decoding,
and playback. The source is Peter's local authoring file:

```text
~/Downloads/‘How Dare You’ Greta.mp3
```

The untouched source is a 13,172-byte, 48 kHz stereo variable-bitrate MP3 with
duration 1.381604 seconds and SHA-256
`084b45085a03def28987c6d3cf80907c88630b4573e65fa73d0b4cf6a4e5be53`.
Peter authorizes deterministic conversion with Nix-provided FFmpeg when the
`blar` preparation pipeline accepts MP3 input. The source filename is authoring
input, never a runtime path or archive name.
The proposed package contains a normalized logical entry such as
`assets/audio/satellite-destroyed.flac`; `AE_sample_asset` binds that name to a
guest-chosen stable ID during configuration, and `AE_sample_play` triggers it
after the destruction event. The packager must fail clearly until `blar` owns
a tested MP3-to-FLAC preparation path (or receives a preconverted FLAC). It
must never relabel MP3 bytes as FLAC or ask the runtime to infer a codec from a
filename.

The three spoken words are not the provenance concern; the particular YouTube
recording may still carry recording rights even though it captures a public
speech. Local development packaging is approved. Public-demo metadata must
record the exact source URL/license when recovered and the release decision;
the current practical risk assessment is low, not a claim of public-domain
status.

## Delivery

One application-source adapter presents the same virtual root for all three
launch forms:

```text
aedicule path/to/code.wat   # virtual root is path/to/
aedicule vibesteroids/      # virtual root is that directory; entry is code.wat
aedicule vibesteroids.aed   # virtual root is the ZIP root; entry is code.wat
```

The initial authoring commands are:

```text
aedicule --package vibesteroids/ [vibesteroids.aed]
aedicule --depackage vibesteroids.aed [vibesteroids/]
aedicule --test [--seed N] vibesteroids.aed
aedicule --web vibesteroids.aed [--bind 127.0.0.1] [--port 8080]
aedicule --scaffold PROJECT
```

Omitted outputs derive from the input basename. Packaging never mutates its
source tree. `--web` serves the pinned browser runtime and exact application
source in the foreground, writes the usable URL alone to stdout, sends
diagnostics to stderr, and supplies the required COOP/COEP headers. It binds
only to loopback by default; LAN/tailnet exposure requires an explicit
non-loopback `--bind`, and port `0` asks the OS for a free port. The same web
adapter should accept a directory or bare WAT as a development convenience.

`--scaffold` creates a new, nonexisting project directory containing a minimal
ABI-compatible `code.wat`, `aedicule.toml`, `assets/`, `lib/`, a passing
`tests/main.wast`, and a concise `README.md`. The manifest contains safe
defaults plus commented optional settings. It never guesses a license or
merges into an existing destination. Future explicit templates may specialize
the same contract for CLI, canvas, or form applications.

Native file associations should make `.aed` the normal
double-click and drag-and-drop application type while retaining `.wat` as a
developer-facing type. The web launcher fetches the same `.aed` bytes before
creating `BrowserRuntime`. A missing or invalid declared entry fails
configuration rather than producing a partial frame or silent sound.

## Required falsification before implementation

- Golden `.aed` files produced independently by Aedicule and a standard ZIP
  reader/writer, plus byte-for-byte reproduction across supported hosts.
- Rejection tests for truncated headers, central/local disagreement,
  duplicate/escaping names, CRC corruption, unsupported methods/types, ZIP
  bombs, extraction collisions, and every configured limit.
- Round-trip tests proving directory and `.aed` launches expose identical
  `code.wat` and declared asset bytes while arbitrary neighboring host files
  remain inaccessible.
- WAV fixtures covering mono/stereo, supported PCM widths, malformed chunks,
  oversized declared data, and sample-rate conversion boundaries.
- `blar` fixtures proving a WAV source produces a lossless FLAC delivery entry
  with its codec and reconstruction metadata explicit in the archive.
- An image cache test proving duplicate package images are content-hashed once
  while distinct guest IDs remain independently releasable.
- Native and browser tests that a sample request cannot affect simulation rate,
  tick ordering, or observable timestamps.
- Synthetic pointer, keyboard, and tick tests that exercise the same ordered
  `Event`/scheduler path as native and browser input adapters.
