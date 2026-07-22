---
description: "GPUI canvas elements need explicit size_full even inside a flex filling parent."
datetime: 2026-07-16T07:31:24-04:00 # America/New_York (EDT)
tags: [gpui, gui, graphics, canvas, elements, explicit, size, full, flex, filling, parent]
---
On the pinned GPUI revision, a `canvas(...)` child does not inherit usable
bounds merely because its containing `div` is `flex_1().w_full()`. The failure
rendered the title/status chrome but omitted the scene; an XWayland capture
made the malformed result particularly obvious.

Apply `.size_full()` to the canvas element itself. Verify this at two layers:
export the immutable `FrameOutput` headlessly to prove guest output, then
capture a real GPUI window to prove adapter layout and compositor behavior.
