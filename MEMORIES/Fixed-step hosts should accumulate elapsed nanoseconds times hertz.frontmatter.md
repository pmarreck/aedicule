---
description: "Fixed step hosts should accumulate elapsed nanoseconds times hertz."
datetime: 2026-07-17T01:06:34-04:00 # America/New_York (EDT)
tags: [fixed, step, hosts, accumulate, elapsed, nanoseconds, times, hertz]
---
Scheduling `Duration::from_nanos(1_000_000_000 / hz)` repeatedly truncates
fractional periods such as 120 Hz and lets callback work time slow the
simulation. Instead, sample an injected monotonic clock, add
`elapsed_nanoseconds * hz` to an integer phase, advance the quotient divided by
one billion, and retain the remainder. This produces exactly 60 or 120 ticks
for any partitioning of one elapsed second without floats or sleeps in tests.

Cap the number of due ticks handled in one host wake. Drop and report excess
whole ticks while preserving only the fractional phase; retaining excess as
debt creates a catch-up spiral. Ask the plugin for a validated fixed rate via
an optional export, with a legacy default, so scheduling policy does not leak a
wall clock into deterministic guest code.
