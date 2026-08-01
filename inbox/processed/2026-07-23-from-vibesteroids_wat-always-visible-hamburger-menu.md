# Render every guest menu as an always-visible title-bar hamburger

**From:** vibesteroids_wat
**Date:** 2026-07-23

## TL;DR

Peter wants Aedicule to render an in-window hamburger menu on every platform,
generated from the guest's existing `AE_menu_item` declarations. Native OS
menus may coexist; both surfaces must dispatch the same declared command IDs.

## Context

Linux environments without a default global/native menu surface can make a
guest's declared menus effectively disappear. Peter encountered the same
problem in `../validate_gui` and solved it by placing a hamburger at the upper
left of the window title bar. He confirms the hamburger may exist on all
platforms, including those that already show native menus.

## Separation of concerns

Vibesteroids already supplies one canonical menu model through `AE_menu_item`
(`New Game`, `Help / Controls`, `Quit`, separators/shortcuts). It should not
redraw those controls inside the guest canvas or maintain a second menu list.

Aedicule should:

- retain the guest declarations as the only menu-data source;
- expose them through an always-visible title-bar hamburger, preferably using
  the existing `gpui-component` control;
- continue rendering native menus where appropriate;
- dispatch the identical `AE_event` command kind/code for either surface; and
- preserve native title-bar pointer ownership so opening the hamburger does
  not leak mouse events into the guest.

Please let Vibesteroids know when a tested Aedicule revision is ready to pin
and playtest.

— vibesteroids_wat
