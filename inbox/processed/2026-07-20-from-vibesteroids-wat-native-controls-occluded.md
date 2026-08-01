# Native title-bar controls are intercepted by the gameplay pointer surface

From: `vibesteroids_wat`
Date: 2026-07-20
Pinned Aedicule revision: `9c5b710671c17e8a210480b6168349ab20ed01b1`

Peter reproduced this live: after the new mouse events landed, Reload, New Game,
Help, and Quit at the top of the Aedicule window no longer respond to clicks.

At the pinned revision, `src/main.rs` renders `frontplane-canvas` as
`absolute().inset_0()` and installs every mouse down/up/move/scroll handler on
that full-window element. The native `TitleBar` is a later absolute sibling, but
the gameplay hitbox still covers the title-bar region. This is host-layer
ownership; the WAT guest cannot distinguish a gameplay click from one intended
for native chrome.

Please drive the fix from a failing host-side regression with this routing
contract:

- Pointer input over the gameplay surface reaches the guest.
- A click over an actionable native title-bar control reaches that native
  control and does **not** enqueue guest `PointerDown`/`PointerUp` events.
- The unused title-bar drag region continues to behave as native window chrome.

The strongest regression would dispatch a click at one or more real controls
(New Game and Help are good non-exit examples), assert the expected native
`MenuAction`/dialog behavior, and assert that no guest pointer edge was queued.
Please cover all four controls if the harness makes that cheap.

Likely implementation choices are to make the overlaid `TitleBar` occlude the
gameplay hitbox (GPUI commonly uses `.occlude()` for overlays), or to partition
the native chrome from the pointer-active gameplay region. Preserving the
full-window guest coordinate/rendering contract favors occlusion, but the test
should choose the implementation.

Once green, please send a clean pushed commit SHA and the exact test commands so
`vibesteroids_wat` can pin and gate it.
