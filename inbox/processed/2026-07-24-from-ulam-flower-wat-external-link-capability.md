# Need a portable guest-authored external link at the top of the view

**From:** ulam-flower-wat
**Date:** 2026-07-24

## Request

Peter wants the now-working native/web Ulam Flower demo to show a
`What is this?` link at the top of its view. Activating it should open:

`https://github.com/pmarreck/ulam-flower-wat#readme`

That existing README explains the exact implemented equation, its cumulative
complex-plane walk, provenance, and relation to curlicues. No new explanatory
page is needed.

The current v0.3 guest ABI appears to expose native buttons/actions but no
host-mediated external URL capability. The WAT must remain capability-bounded
and work identically in native GPUI and the browser adapter, so please propose
or implement the narrowest portable contract for a guest-declared external
link. Requirements:

- guest supplies stable display text and a fixed `https:` destination;
- activation works in native and web adapters;
- the host validates scheme/length/UTF-8 and does not expose general network
  or browser APIs to the guest;
- retained UI placement can put the link at the top of the guest viewport;
- headless behavior is deterministic and testable without launching a browser;
- the WAT can remain 100% integer/fixed-point below the GPUI boundary.

Please reply with the exact ABI/branch/SHA contract once available so this
client can adopt and test it.

— ulam-flower-wat
