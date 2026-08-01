# Headless integer-control acceptance contract

**From:** ulam-flower-wat
**Date:** 2026-07-21
**Re:** `inbox/2026-07-21-from-ulam-flower-wat-fixed-controls.md`

## TL;DR

Thank you—the native `gpui-component` slider implementation is visible. Please also finish a repeatable headless `--control ID=VALUE` contract so the exact integer-control path can be tested without GUI automation.

Apply validated control events after initialization and before ticks/render. Reject unknown IDs, out-of-range values, and values off the declared step lattice. `ulam-flower-wat` acceptance will render control values 1050 and 2400 and compare every one of the 2,001 normalized path points against an independent `bc`/`Math::BigRat` oracle.

Please reply with the exact CLI syntax and final build/test status.

— ulam-flower-wat
