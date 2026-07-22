---
description: "Archive code and published specification need a pinned compatibility profile."
datetime: 2026-07-19T18:40:45-04:00 # America/New_York (EDT)
tags: [archive, published, specification, pinned, compatibility, profile]
---
When Aedicule considered `blar`/`mini_blar` for packaged WAT media on
2026-07-19, their checked-in reference implementations both used the flat
`MBAR\x02`/`BLAR\x02` archive magic and shared source-compatible archive logic.
The published `blar/BLAR_ARCHIVE_SPEC.md`, however, described a `BLIP\x01`
envelope. Do not advertise generic BLAR support or accept formats by family
name alone.

Until the upstream spec and reference source are reconciled, Aedicule's asset
reader must identify and validate one exact, version-pinned profile. The
initial proposed profile is flat, uncompressed `MBAR\x02`: verified indexed
FILE entries only, no directories, compression, encryption, segments, or
unknown attributes. Retain golden archives from both reference writers before
implementing the Rust parser. `blar`, not Aedicule, owns conversion of source
WAV/AIFF into the selected delivery codec. The current Aedicule decision is
lossless FLAC only; Ogg/MP3 are deferred pending a `blar` design discussion.
Aedicule only decodes the already-selected entry; FLAC and images should
normally be stored without generic archive recompression.
