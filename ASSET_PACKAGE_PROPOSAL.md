# Aedicule Asset Package Proposal (v0)

## Decision

An Aedicule application may carry an optional `assets.mbar` beside `code.wat`.
Version 0 accepts one deliberately narrow, version-pinned `mini_blar` profile:
a flat, uncompressed `MBAR\x02` archive. The host loads and validates this
package before instantiating WAT; WAT code names immutable package entries and
never receives filesystem, URL, or archive-parser authority.

`blar` owns media preparation. For now, it converts source WAV/AIFF to
lossless FLAC and records whatever reconstruction metadata belongs to the
archive format. Aedicule receives only that finished entry, identifies FLAC,
and decodes it under host limits. It never chooses a codec, transcodes source
media, or treats its archive compressor as a substitute for an audio codec.
Ogg and MP3 are explicitly deferred pending a later `blar` design discussion.

This is a compatibility profile, not a claim that every archive described as
“BLAR” is accepted. The current `blar` and `mini_blar` reference source trees
share this wire profile, while `BLAR_ARCHIVE_SPEC.md` presently describes a
different magic/version. Aedicule must keep its profile pinned until those
upstream documents and reference implementations are reconciled.

## Accepted archive shape

The archive must be a `mini_blar` flat-file archive with magic `MBAR\x02`:

```
ARRAY [ DATA("MBAR\\x02"), ARRAY [ FILE, FILE, ... ] ]
```

- Only the outer array, body array, and FILE entries are accepted. `DIR`,
  segments, encryption, signatures, container expansion, and unknown
  container or attribute forms are rejected.
- Every entry is an uncompressed FILE with a UTF-8 `pa` path and a DATA
  payload. The host rejects `COMP` even when a decoder happens to be linked.
- Paths are unique, NFC UTF-8, relative, slash-separated, and contain no
  empty, `.` or `..` component. Archive paths are names, never extraction
  paths.
- The reader validates all declared lengths, index offsets, element counts,
  index ordering, and required container types before slicing. It verifies the
  outer BLAKE3-128 checksum and each selected FILE/DATA xxHash64 checksum
  before handing bytes to an image or audio decoder.

The initial profile intentionally excludes zstd. That preserves direct,
zero-copy access to encoded texture bytes and prevents a small downloaded
archive from expanding into an unbounded allocation. It must not become a
delivery-size assumption: `blar` prepares WAV/AIFF as FLAC before packaging;
that already-compressed entry normally bypasses archive compression. A future
compressed profile needs its own magic/profile version, decoded-size fields
checked before allocation, content-aware “store versus zstd” selection for
non-media data, and independent corruption fixtures.

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

## Delivery

Native launchers default to a sibling `assets.mbar` when present and may later
expose an explicit package argument. The web bundle copies the same bytes as a
static asset and fetches them before creating the `BrowserRuntime`; Caddy's
existing cross-origin-isolation headers remain unchanged. A missing optional
package is valid only until a WAT declaration requests an asset, which then
fails configuration rather than producing a partial frame or silent sound.

## Required falsification before implementation

- Golden archives produced by the pinned `mini_blar` writer, including an
  independently generated cross-check from `blar`.
- Rejection tests for truncated lengths, bad indexes, duplicate/escaping names,
  checksum corruption, unsupported attributes, and every configured limit.
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
