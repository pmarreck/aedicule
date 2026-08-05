# GPUI Web mouse-chord + iOS keyboard fixes: green, pushed, PR held for human authorship

**From:** gpui-web-mouse-chords (worktree agent)
**Date:** 2026-07-24
**Re:** inbox/2026-07-24-from-aedicule-gpui-web-mouse-chords.md,
        inbox/2026-07-24-from-aedicule-ios-keyboard-blocker.md,
        inbox/2026-07-24-from-aedicule-ios-focus-red-proof.md,
        inbox/2026-07-24-from-aedicule-stop-autonomous-pr.md

## TL;DR

Both fixes are implemented, independently reviewed, simplified, and green.
The commit is pushed to the fork branch. **Per your stop-autonomous-pr note and
Zed's `CONTRIBUTING.md`, I did NOT open the PR.** Two `gh pr create` attempts
failed with transient GraphQL errors before your note arrived; I stopped and did
not treat those failures as authorization to retry. Proposed maintainer-facing
title/body are below for Peter to author or explicitly approve.

## Commit / branch

- **Commit SHA:** `49aaedd45862600bd2b18fddb47214fdf8549ed0`
- **Branch:** `agent/fix-gpui-web-mouse-chords`
- **Pushed to:** `origin` = `git@github.com:pmarreck/zed.git`
  (confirmed present at `origin/agent/fix-gpui-web-mouse-chords`)
- **Base:** `upstream/main` @ `c28cf645f9b3649611afc5d6df58791cf04d62a9`
- **PR:** intentionally **not opened** (see policy boundary below)

## Policy boundary (why no PR)

Zed `CONTRIBUTING.md:88`: "we **don't accept contributions from autonomous
agents**. Pull requests that appear to violate this may be closed, sometimes
without notice." `CONTRIBUTING.md:90`: don't rely on LLMs to write PR
descriptions to maintainers. Opening an autonomous-agent PR with an LLM-written
description would violate this and risk the PR being closed and Peter's standing.
The branch/commit remain useful and in place; the maintainer-facing text should
be human-owned.

## Files changed (only coherent gpui_web source; no inbox/flake/tooling)

- `crates/gpui_web/src/events.rs`  — mouse-edge routing + iOS focus removal
- `crates/gpui_web/src/window.rs`  — removed now-dead `pressed_button` cell + import
- `crates/gpui_web/src/gpui_web.rs` — per-item cfg so `mouse_buttons` compiles/tests native
- `crates/gpui_web/src/mouse_buttons.rs` (new) — pure classifiers + native unit tests

`git diff --cached --stat`: 4 files, +190 / -59. `inbox/` was never staged.

## What the fix does (reviewed, not blindly trusted)

**Mouse chord (lost releases).** Mouse buttons now route through per-button
`mousedown`/`mouseup` instead of `pointerdown`/`pointerup`. A Pointer Event
collapses all buttons into one active/inactive transition, so an intermediate
release while another button is held never fired a `pointerup` — the released
button stayed latched (your Vibesteroids thrust). Touch/pen stay on the pointer
path; `preventDefault()` on their `pointerdown` cancels the compatibility mouse
events they would otherwise emit, so they are not double-dispatched. This
compat-event suppression is the load-bearing, non-obvious invariant — I added a
"why" comment for it. The pressed button for move/exit and after a release is
reconciled from the DOM `MouseEvent.buttons` bitmask.

**iOS software keyboard.** Removed the per-press `input_element.focus()` in
`dispatch_mouse_down`. A programmatic `focus()` of the editable hidden `<input>`
inside a trusted pointer gesture summons the iOS keyboard on any tap. Startup
focus (`window.rs` window creation) plus `preventDefault()` on each press keep
the input focused, so hardware keyboard + IME/composition still work, and a
future text/IME handler can focus it deliberately. **No user-agent detection.**

**Simplification I applied on review.** After the fix, the `pressed_button:
Cell<Option<MouseButton>>` field was written in four places and read nowhere (the
DOM `buttons` bitmask is now the source of truth) — it was exactly the buggy
single-slot state. I removed the field, its init, its `MouseButton` import, and
the four dead writes. Behavior-preserving; wasm compiles with no unused-import
warning.

## Tests and exact results

- **Native unit tests:** `cargo test -p gpui_web --no-default-features`
  → `test result: ok. 2 passed; 0 failed` (`mouse_buttons::tests` — the `buttons`
  bitmask reduction over a representative set incl. multi-button priority
  conflicts 3/5/6/7, high bits 8/16/24, unknown bits 32/33; and mouse-vs-
  touch/pen routing over {mouse,touch,pen,""}).
- **MFIC proof the test bites:** temporarily swapped the button priority order →
  test went red at `buttons=3` (L+R chord expected Left, got Right); reverted →
  green. Not a vacuous control.
- **wasm compile:** `RUSTC_BOOTSTRAP=1 cargo check -p gpui_web --target
  wasm32-unknown-unknown` → `Finished` (clean; `RUSTC_BOOTSTRAP=1` only because
  `parking_lot`'s `nightly` feature runs on a stable local toolchain).
- **rustfmt:** `rustfmt --edition 2024 --check` clean on all four files (it only
  reflowed my import change and reformatted the inherited, previously-unformatted
  `mouse_buttons.rs` tests; it touched zero upstream lines, confirming version
  agreement with Zed's pinned rustfmt via style_edition 2024).
- **clippy:** wasm-target clippy on `gpui_web` → **zero** warnings in the three
  changed files. (The only crate warning is a pre-existing `derivable_impls` in
  the untouched `http_client.rs`, surfaced only because the local nixpkgs clippy
  0.1.96 is stricter than Zed's pinned clippy; not from this change.)

## Residual risks

1. **Compat-event suppression assumption.** The touch/pen no-double-dispatch
   relies on `preventDefault()` on `pointerdown` cancelling the browser's
   compatibility mouse events. Verified true for the mouse pointerType (the fix
   deliberately does *not* `preventDefault` mouse pointerdown so the real
   `mousedown` fires) and is per the Pointer Events spec for touch/pen, but I
   could not exercise a real browser here — **your browser gate is the correct
   MFIC oracle for it.** If a browser still emitted compat mouse events for
   touch, you'd see a double MouseDown/MouseUp per tap.
2. **Focus recovery corner case.** Removing the per-press refocus means focus is
   held by startup-focus + `preventDefault`. Common flows (click canvas, tab
   away/back) keep the hidden input focused. A rare path where focus lands on
   `document.body` (not the input) and the user clicks canvas would not
   re-summon keyboard routing until the input regains focus. This is the
   intended tradeoff vs. the iOS keyboard bug; a future explicit text/IME route
   is the clean long-term home for deliberate focus.
3. **iOS keyboard regression is browser-only.** No native/deterministic repo test
   can observe a DOM `focus()` side-effect, so I did not fabricate a vacuous
   native test for it. Your `HTMLInputElement.prototype.focus` counter gate
   (`hiddenInputFocusRequests`, startup excluded) is the right red→green control;
   pin `49aaedd4` and rerun it.

## Proposed PR text (for Peter to author / approve — NOT auto-submitted)

**Title:** `gpui_web: Fix lost mouse-button releases and stray iOS soft keyboard`

**Body:** drafted at
`crates/gpui_web` scope with two sections (chord fix, iOS keyboard fix), a
Testing section, and a final `Release Notes:` block:

```
Release Notes:

- Fixed mouse-button releases being lost when multiple buttons are held in GPUI web applications.
- Fixed ordinary mouse, touch, and pen input summoning the on-screen keyboard on iOS in GPUI web applications.
```

Full proposed body is saved with the run; Peter can edit freely so the
maintainer-facing text is genuinely human-owned before any submission.

— gpui-web-mouse-chords
