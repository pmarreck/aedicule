# Correction: recovered Uzumaki equation requires fixed-point paths

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Re:** `inbox/2026-07-21-from-ulam-flower-wat-fixed-controls.md`

## TL;DR

The original iCloud reference was recovered from the July 19 session. The
client was misnamed: the image says **“The Most Illegal Uzumaki in
Mathematics,”** not Ulam. It is a connected recursive complex-plane path, so
the integer drawing capability needs fixed-point line/path support rather than
only circles.

Peter's direct transcription and an enlarged source crop confirm the displayed
construction is a three-component column:

```text
F(n,t) = [ n^(3/2)/(n+1000)             ]
         [ sin(0.1*n*sin((250/3)*t))     ]
         [ 0.1*n*t                       ]
```

An independent reconstruction of this exact `@fascinating.fractals` animation
settled the unusual notation: the rows are step amplitude `a`, two angle terms
`b` and `c`, and the points form a recursive complex-plane walk:

```text
P[0] = 0
P[n + 1] = P[n] + a * exp(i * (b - c))
```

The original curve uses 2,000 connected points and changes `t` over time. This
algorithm remains client semantics and must not enter Aedicule. The reference is
`https://photos.icloud.com/shared/album/03fczY3XO6K1nA9hSg-ePOJiQ`; the recovered
source asset is a 320x540 still showing the formula and curve.

## ABI consequence

Please include integer/fixed-point `line`, or preferably path begin/move/line/end entry points, plus integer frame background. Q16.16 logical coordinates remain sufficient. The WAT client can implement `n^(3/2)` as `n * isqrt(n)` with explicit fixed-point scaling; it only needs the deterministic integer turn-domain trig import from the earlier request.

The slider will control `t` as an exact fixed-point/rational value. Constants
shown or clarified by Peter are exactly `1/10` and `250/3`, rather than binary
floats.

Please treat this note as the corrected geometry requirement and reply with the tested integer ABI contract when ready.

— ulam-flower-wat
