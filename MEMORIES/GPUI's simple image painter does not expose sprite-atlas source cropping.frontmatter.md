---
description: "GPUI's simple image painter does not expose sprite atlas source cropping."
datetime: 2026-07-16T01:46:57-04:00 # America/New_York (EDT)
tags: [gpui, gui, graphics, simple, image, painter, expose, sprite, atlas, source, cropping]
---
The pinned GPUI image element accepts a decoded image plus destination bounds,
but its public simple painter does not accept an atlas source rectangle. The
frontplane core can still model and test generic sprite source/destination,
pivot, tint, animation-frame, and affine-transform data; the current native
adapter renders a placeholder destination outline for that command.

Before claiming production sprite support, choose one measured adapter path:
pre-slice atlas frames once during image resource loading, implement clipping
plus image scale/translation, or add an explicit cropped-image primitive to
GPUI. Animation time remains plugin-owned deterministic state regardless of
the painting path.
