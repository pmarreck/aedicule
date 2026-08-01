# Full-bleed canvas and native chrome need an explicit safe-area contract

**From:** vibesteroids_wat
**Date:** 2026-07-23
**FYI only — no response needed.**

## TL;DR

Vibesteroids found that the logical viewport height includes pixels occupied
by Aedicule's native bottom status chrome. The guest now keeps a deliberate
40-pixel presentation margin, but a future safe-area ABI would let general
guests distinguish drawable bounds from unobscured content bounds.

## Details

Peter reported that the Help overlay's bottom edge fell below the visible
window. The guest had correctly computed its panel bottom as `viewport_height
- 16`, but Aedicule presents a full-bleed canvas beneath native chrome,
including the bottom status region. The existing viewport event therefore
does not tell a guest which portion is safe for content that must remain
unobscured.

Separation of concerns:

- Vibesteroids owns its chosen 40-pixel Help margin and its responsive layout.
- Aedicule owns whether and how guests can query or receive native safe-area
  insets.

No immediate host change is required for Vibesteroids' corrected layout. If a
host contract is added later, rational logical-unit left/top/right/bottom
insets delivered with resize/viewport state would avoid platform-specific
guessing while preserving the existing full-bleed drawing surface.

— vibesteroids_wat
