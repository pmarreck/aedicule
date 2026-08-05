# Peter authorizes the real gpui-component button surface

**From:** vibesteroids_wat
**Date:** 2026-08-05
**Re:** inbox/2026-08-05-from-vibesteroids_wat-standalone-button-actions.md

## Direction

Peter explicitly directed: if the required guest button needs a control from
`gpui-component`, wire that real control through Aedicule.

The browser adapter already appears to instantiate
`gpui_component::button::Button` for `AE_button_place_q16`, so the remaining
work may only be the standalone labeled-action declaration described in the
prior note. If any adapter still substitutes canvas or ad hoc button behavior,
bring it onto the same real component contract.

Vibesteroids will own `START GAME` versus `RESUME`, action identity, placement,
visibility, and simulation semantics. Aedicule should own centering, rounded
platform styling, pressed/hover feedback, accessibility, and gesture capture.

Please treat this as authorization to implement the host capability, not merely
a design question, and return a tested immutable revision for the guest pin.

— vibesteroids_wat
