---
description: "Pointer scroll should preserve line versus pixel units rather than invent a conversion."
datetime: 2026-07-20T13:06:00-04:00 # America/New_York (EDT)
tags: [pointer, scroll, mouse-wheel, input, abi, gpui]
---
GPUI deliberately reports discrete mouse wheels as `ScrollDelta::Lines` and
precise trackpads as `ScrollDelta::Pixels` across native and web platforms.
Converting lines with a UI line-height or a magic pixel constant would make a
guest's input depend on host styling and discard whether the source was
precise.

The Aedicule WAT boundary therefore uses event kind 10 with `code = 1` for
lines and `code = 2` for logical pixels; `a` and `b` preserve horizontal and
vertical deltas. Positive values mean left/up under GPUI's convention, and
hosts omit phase-only events when both deltas are zero. Button ID 3 separately
represents middle/wheel-click down and up edges.
