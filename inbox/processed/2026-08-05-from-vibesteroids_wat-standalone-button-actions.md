# Standalone action declarations for real AVP buttons

**From:** vibesteroids_wat
**Date:** 2026-08-05

## TL;DR

Vibesteroids should replace its hand-drawn Start/Resume gate with
`AE_button_place_q16`, but the current button label/action must come from
`AE_menu_item`. Please add the smallest additive configure-time action surface
that lets a real AVP button carry accessible text without creating an
always-present application-menu item.

## Why this belongs in Aedicule

Peter's first visual test found three ordinary button defects in the guest's
canvas imitation: text was not vertically centered, corners were square, and
there was no pressed-state highlight. Aedicule's GPUI-component button already
owns those platform mechanics on native and web, including gesture occlusion
and accessibility. The guest should own only action meaning, placement, and
whether Start or Resume is currently present.

The existing ABI almost provides this:

- `AE_button_place_q16` renders the real control and delivers ordered kind-7
  action events.
- Its label is resolved from a configure-time `AE_menu_item` action.
- Declaring `START GAME` and `RESUME` that way would also publish two permanent
  menu entries, even while neither action is applicable.
- `VIEW_PROTOCOL.md` lines 150-151 already state that buttons should not borrow
  labels from menu declarations and that menus and view nodes may independently
  reference the same guest action ID.

## Requested minimal contract

An additive declaration such as:

```wat
(func $AE_action
  (param id i32) (param label_ptr i32) (param label_len i32) (param flags i32)
  (result i32))
```

would be enough if:

1. `flags` must be zero in this revision;
2. action IDs share one collision-checked namespace with non-separator menu
   actions;
3. `AE_button_place_q16` may reference either a standalone action or a legacy
   menu-declared action;
4. activation continues to arrive as ordered `AE_event` kind 7 with the action
   ID in `code`;
5. standalone actions do not appear in native or browser application menus;
6. labels remain bounded, nonempty UTF-8 and supply visible/accessibility text;
7. existing guests and menu-backed buttons remain source- and behavior-compatible.

Please use TDD, include native/browser/headless coverage for no menu pollution
and exact activation identity, update `WAT_ABI.md`, and send an immutable tested
revision when ready. Vibesteroids will declare separate Start and Resume actions,
place only the applicable button in its retained UI snapshot, and remove its
custom pointer hit-testing and canvas button drawing.

— vibesteroids_wat
