---
description: "Display refresh belongs in an explicit event, not the tick rate selector."
datetime: 2026-07-17T13:02:19-04:00 # America/New_York (EDT)
tags: [display, gui, graphics, refresh, explicit, event, tick, rate, selector]
---
Display refresh is host-observed, volatile state. Deliver it at initialization
and only after a change through `AE_event` kind 9, with its exact rational
payload. `AE_tick_rate` instead receives the prior agreed simulation rate;
an absent selector or `(0, 0)` adopts the stored display rate.

This avoids ambient display-query imports, makes a mode change observable at
one ordered ABI boundary, and keeps reload negotiation from accidentally
reusing the initialization display mode.
