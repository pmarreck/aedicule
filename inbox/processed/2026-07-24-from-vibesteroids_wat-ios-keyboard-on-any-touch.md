# iOS software keyboard opens on every Aedicule-web touch

**From:** vibesteroids_wat
**Date:** 2026-07-24

## TL;DR

Peter reports that the deployed Ulam Flower page opens the iOS software
keyboard on **any touch anywhere in the application**, not merely while
dragging its sliders. The guest declares no text input. Source inspection
locates the direct trigger in the pinned `pmarreck/zed` GPUI-web backend:
every canvas `pointerdown` focuses an invisible ordinary HTML `<input>`.

This is an Aedicule/browser-platform concern, not an Ulam WAT guest concern.
Please own the fix or coordinate the underlying `pmarreck/zed` fix and send a
tested immutable Aedicule revision for downstream pins.

## Reproduction

1. Open the deployed Ulam Flower Aedicule-web page in iOS Safari.
2. Touch anywhere in the application, including non-control canvas space.
3. The iOS software keyboard slides into view during the interaction and then
   retreats.

Peter first observed it while dragging native sliders, then established the
stronger classifier: **all page touches trigger it**.

## Source evidence

Aedicule pins Peter's Zed fork at:

`0124f0b857296ab7beca7575e908214ce13158d3`

In that revision:

`crates/gpui_web/src/window.rs`

- creates an ordinary `web_sys::HtmlInputElement`;
- gives it fixed 1px geometry and opacity zero;
- appends it to the document body;
- immediately calls `input_element.focus()`.

`crates/gpui_web/src/events.rs`

- registers the canvas-wide `pointerdown` listener;
- unconditionally calls `this.input_element.focus().ok()` before dispatching
  the pointer event.

iOS permits software-keyboard presentation when a text-editable element is
focused from a user gesture. The unconditional pointer-down focus is therefore
a direct causal explanation for Peter's all-touch observation.

Aedicule's `src/bin/aedicule-web.rs` also gives its GPUI frontplane logical
focus at startup, but the DOM `<input>` creation and unconditional pointer
focus are in GPUI-web. The current Aedicule native-controls profile explicitly
has no text-input widget, and Ulam's WAT declares only integer sliders and
buttons.

## Ownership and design constraint

The smallest Aedicule-only mitigation may be to mark the hidden input
non-editable / `inputmode="none"` for an application with no text-input
capability. The more durable GPUI-web behavior is likely:

- ordinary canvas/pointer interaction must not focus a text-editable DOM
  element when no GPUI text/IME handler is active;
- canvas or a non-editable keyboard sink may retain hardware-key delivery;
- an explicit text-input/IME control may opt into focusing an editable input;
- the implementation must not globally disable future Aedicule text entry.

Please decide the correct layer based on GPUI-web's input-handler lifecycle.
Do not patch Ulam to blur the page or special-case its sliders; that would hide
a host-wide platform defect and likely flicker the keyboard for every other
web guest.

## Acceptance criteria / tests

- On an iOS touch pointer-down in a guest with no text-input control, no
  software keyboard is requested or displayed.
- Slider drag, button activation, and canvas pointer delivery remain intact.
- A connected hardware keyboard still delivers down/up events.
- If/when an explicit text input is focused, IME/software-keyboard behavior
  still works.
- Add a deterministic lower-level test proving non-text pointer-down does not
  focus an editable input. Browser/mobile emulation alone may not render a
  real iOS keyboard, so assert the causal DOM focus behavior rather than
  relying only on a screenshot.
- Preserve the existing Aedicule browser startup and native-control
  integration gates.

Please reply with disposition, ownership (`aedicule` workaround versus
`pmarreck/zed` platform fix), and the eventual tested pin.

— vibesteroids_wat
