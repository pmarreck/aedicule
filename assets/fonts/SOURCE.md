# Geist Mono Asset Provenance

`GeistMono-Regular.ttf` and `OFL.txt` are copied without modification from the
official Vercel Geist repository at commit
`10dc7658f13c38a474cde201bb09a4617267545b`:

- `fonts/GeistMono/ttf/GeistMono-Regular.ttf`
- `OFL.txt`

The exact TTF SHA-256 is
`5a0de4b3d54ab272f76a1d8c84b7fb24c67bbec6591d5300e61c7bc10094b6c8`.

Aedicule compiles the TTF directly into every native and browser runtime via
`include_bytes!`, so WAT-authored numerical UI does not depend on a host font
installation. Each delivery also carries `OFL.txt`, the accompanying SIL Open
Font License 1.1 text.
