# Proposed PR text (DRAFT — for Peter to author/edit; not submitted)

Branch `agent/fix-gpui-web-mouse-chords` @ commit `12e6583a5e05103569e25749d797891e7b09585b`
on `pmarreck/zed`, base `zed-industries/zed:main`. **Not opened** per Zed
CONTRIBUTING (no autonomous-agent PRs; maintainer-facing text must be human-owned).

---

**Title:** gpui_web: Fix lost mouse-button releases and stray iOS soft keyboard

**Body:**

This fixes two user-visible GPUI Web input bugs, both in the pointer/mouse dispatch path of `crates/gpui_web`.

## Lost mouse-button releases during chords

`gpui_web` registered mouse buttons through `pointerdown`/`pointerup`. A Pointer Event represents the transition into and out of the *active* mouse-pointer state rather than individual button edges, so when one button is released while another stays held, no `pointerup` fires and that release is silently dropped. A browser app holding two buttons at once (for example a right-button thrust plus a left-button fire in a canvas game) keeps the released button latched.

Mouse input now flows through per-button `mousedown`/`mouseup` listeners, which fire for every individual button edge. Touch and pen keep using the pointer path; `preventDefault()` on their `pointerdown` cancels the compatibility mouse events they would otherwise emit, so they are never dispatched twice. The pressed button reported on move/exit and after a release is reconciled from the DOM `MouseEvent.buttons` bitmask, replacing the single `pressed_button` cell that became `None` on any intermediate release.

## On-screen keyboard summoned by non-text input (iOS Safari)

`dispatch_mouse_down` called `input_element.focus()` on every pointer press. A programmatic `focus()` of an editable input inside a trusted pointer gesture summons the iOS software keyboard on any tap, even when no text field is shown. The shared per-pointer focus is removed. A real `mousedown` still restores the hidden input's focus so hardware keyboard delivery recovers after focus has moved elsewhere. Touch and pen remain on the Pointer Events path, where `preventDefault()` suppresses their compatibility mouse event; they therefore never focus the editable input merely because the user touched the canvas. A future text/IME handler can still focus it deliberately when text entry is actually requested. No user-agent detection is used.

## Testing

- `cargo test -p gpui_web --no-default-features` — the pure button/pointer classifiers are extracted into `mouse_buttons` so they compile and unit-test on native hosts (2 passing tests, covering the `buttons` bitmask reduction and the mouse-vs-touch/pen routing over representative sets).
- Compiles cleanly for `wasm32-unknown-unknown`.
- rustfmt and clippy clean on the changed files.
- A downstream instrumented browser gate moves focus to `body`, proves a real mouse press restores W/A/D delivery, delivers the exact right-down/left-down/right-up chord to the guest, and observes zero hidden-input focus calls during a multi-touch stream.

Release Notes:

- Fixed mouse-button releases being lost when multiple buttons are held in GPUI web applications.
- Fixed touch and pen input summoning the on-screen keyboard on iOS in GPUI web applications.
