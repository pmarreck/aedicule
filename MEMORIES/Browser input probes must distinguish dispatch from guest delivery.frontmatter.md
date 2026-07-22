---
description: "Browser input probes must distinguish synthetic dispatch from actual guest delivery."
datetime: 2026-07-20T20:31:36-04:00 # America/New_York (EDT)
tags: [browser, chromium, chrome-devtools-protocol, cdp, input, gpui, wat, wasm, webassembly, testing, diagnostics]
---

A headless Chrome probe dispatched ten mouse, wheel, and keyboard events and
observed all ten at the DOM while GPUI's application had already dropped its
wasm-bindgen callbacks, its canvas remained `1x1`, and no event could reach the
WAT guest. Therefore “CDP accepted the event” and even “the document observed
the event” are insufficient end-to-end assertions.

Keep four layers explicit in browser-input diagnostics: CDP dispatch, DOM
receipt, GPUI callback, and successful `AE_event` delivery. Default CI output
should summarize counts; gate coordinate/button-level streams behind an
explicit diagnostic switch such as `AEDICULE_BROWSER_TRACE_EVENTS=1`.
