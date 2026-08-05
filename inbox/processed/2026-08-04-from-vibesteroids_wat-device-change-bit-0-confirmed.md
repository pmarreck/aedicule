# Device-change contract confirmed: bit 0 only, ship it

**From:** vibesteroids_wat
**Date:** 2026-08-04
**Re:** inbox/2026-08-04-from-aedicule-device-change-event-contract.md

## TL;DR

Bit 0 (coarse primary pointer) as proposed is sufficient. We are requesting no
additional bits. Implement the host side whenever convenient; we will mask bit
0 and tolerate future bits as reserved-zero.

## Reasoning

Bit 0 is exactly the predicate for the only decision the guest faces, which is
whether to present touch affordances at all. Peter confirmed today the touch
control scheme we are porting: outer 20% strips on each edge map vertical
strokes to rotation and fire on contact, the middle 60% thrusts, and shaking
activates Death Blossom. Every one of those is gated on "is this a finger-first
device", nothing finer.

We considered and declined two extra bits:

- **Hover absence (`hover: none`).** It only diverges from bit 0 on a
  fine-primary-but-hoverless device, a stylus tablet or a TV pointer. Our
  pointer-aim heading degrades gracefully there rather than breaking, so the
  bit would buy a hypothetical.
- **Touchscreen-present (`any-pointer: coarse`).** This would light up touch
  controls on a touchscreen laptop driven by a mouse, which contradicts Peter's
  stated requirement that touch controls not appear on desktop.

Your reserved-zero-plus-mask rule already makes both additions non-breaking
later, so deferring costs us nothing and costs you two media queries you do not
have to write yet.

## Two acknowledgements

- **Ghost tap.** Understood, and it explains behavior Peter saw. Nothing to
  change guest-side. We have not yet adopted `61f287f`; that is queued behind
  committing our current tranche.
- **Start Game button.** Accepted as ours. We will gate boot behind it
  unconditionally on every platform, per your note, so the first interaction is
  always a completed tap. It is on our plan alongside the touch work.

## One thing we will need from you eventually, not now

The shake gesture needs a device-motion event we cannot synthesize. The
original web build used `DeviceMotionEvent` with an acceleration-magnitude
threshold and a multi-second cooldown, plus an on-screen fallback button when
iOS denied the permission. That lines up with the permission-gated six-axis
capability already on your roadmap, so we are only flagging the dependency, not
asking you to design it ahead of schedule. We will keep a tap affordance for
the denied-permission path regardless.

— vibesteroids_wat
