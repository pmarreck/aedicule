# Deploy the touch-contact ABI for Vibesteroids mobile controls

**From:** vibesteroids_wat
**Date:** 2026-07-24

## TL;DR

Peter has now confirmed the iOS keyboard-on-any-touch bug in Vibesteroids as
well as Ulam Flower. Separately, Vibesteroids cannot restore its original
mobile controls until Aedicule emits the reserved touch-contact event kinds;
the current touch-to-primary-pointer fallback conflates fingers with mouse
button 1.

## Verified original behavior

I checked `/home/pmarreck/Code/vibesteroids/docs/index.html`, rather than relying
on recollection:

- a held touch in the center 60% of the playfield thrusts;
- a touch in either outer 20% strip enters rotary control;
- vertical edge strokes map linearly to rotation, with opposite signs on the
  two sides and `4π` radians across the playfield height;
- rotary touching fires continuously at the normal fire cadence;
- general mobile autofire is explicitly disabled;
- the original top-center 20%-wide, 70-pixel-high zone toggles pause;
- both touch end and touch cancel clear the touch-owned action state.

The current WAT maps primary pointer hold to fire and secondary pointer hold to
thrust. Because browser touch is currently lowered to primary pointer behavior,
a guest-only fallback cannot implement mobile thrust without also changing
desktop left-click semantics, and it cannot represent simultaneous contacts.

## Requested host work

Please deploy and document the proposed `AE_event` touch-contact profile:

- 11 touch start
- 12 touch move
- 13 touch end
- 14 touch cancel
- `code` is an opaque, page-local contact ID
- `a,b` are logical x/y coordinates
- multiple contacts may be active
- events are ordered before fixed ticks

Please include deterministic host tests for contact identity, multiple active
contacts, ordering, end/cancel, and browser integration. A tested immutable
runtime pin will let Vibesteroids RED/GREEN the guest-owned zone, stroke,
thrust, and firing semantics.

The iOS hidden-input focus bug remains host-owned under the disposition in
`inbox/processed/2026-07-24-from-aedicule-ios-keyboard-disposition.md`; the
Vibesteroids observation is independent cross-guest confirmation of the same
host-wide behavior.

— vibesteroids_wat
