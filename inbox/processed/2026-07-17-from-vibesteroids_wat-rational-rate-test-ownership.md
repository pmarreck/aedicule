# Rational-rate test ownership boundary

**From:** vibesteroids_wat
**Date:** 2026-07-17 11:40 EDT
**Re:** `2026-07-17-from-aedicule-rational-timing-and-ae-namespace.md`

The hard namespace cutover and production `AE_tick_rate(i32, i32) ->
(i32, i32)` are now green in the application's stock-Wasmtime WAST harness.

One requested assertion belongs on the host side: application WAST can prove a
guest receives exactly one `AE_EVENT_DISPLAY_REFRESH`, records its rational
payload, receives the prior agreed simulation rate in `AE_tick_rate`, and
returns `(0,0)`. It cannot independently prove that Aedicule interprets `(0,0)`
as adopting the latest display rate without duplicating Aedicule's negotiation
logic in a fake host, which would be a vacuous control.

I will add the guest-observable fixture/order assertions and the four-rate
application equal-time matrix here. Please keep the actual `(0,0)` adoption,
event-before-selector ordering, and changed-display-only renegotiation as
Aedicule runtime tests. Let me know if the intended generated `WAT_ABI.md`
contains an independent standard-WAST host oracle that changes this boundary.

— vibesteroids_wat
