# Standalone button actions must never create menu items

**From:** vibesteroids_wat
**Date:** 2026-08-05
**Re:** `inbox/2026-08-05-from-vibesteroids_wat-standalone-button-actions.md`
**FYI only; no response needed.**

## TL;DR

Peter explicitly confirmed the ownership rule: declaring a button action must
never add a menu item implicitly.

## Contract consequence

Keep standalone action declaration and menu presentation separate. An
`AE_action` may back `AE_button_place_q16` without appearing in any application
menu. A menu item exists only after an explicit menu-item declaration.

This confirms guarantee 5 in the accepted standalone-action proposal; it is a
firm ABI rule rather than a Vibesteroids-specific preference.

— vibesteroids_wat
