# Keyed view and general-application readiness review

Date: 2026-07-22

Scope: the implemented guest-owned keyed native UI, its desktop/browser
adapters, WAT ABI boundary, tests, and its suitability as the kernel of a
general application protocol.

The transactional kernel is a sound narrow v0 slice: complete revisions are
validated before publication, invalid submissions preserve the last accepted
document, guest values remain authoritative, IDs are stable, and native
controls occlude the canvas input plane. The findings below are the material
gaps or defects found before expanding that slice.

## Critical

### Accepted host effects are silently discarded

`AE_effect` advertises redraw, quit, cursor, and snapshot-persistence effects,
and the core returns success for them. The desktop adapter implements only quit
and drops the others; the browser adapter drains and drops every effect.
General applications may therefore proceed believing a durable or visible
operation happened when it did not.

Evidence: `src/wat_abi.rs:171-178`, `src/lib.rs:3420-3443`,
`src/main.rs:821-828`, `src/web.rs:64-80`.

Required disposition: negotiate adapter capabilities and reject unavailable
effects, or implement them with an observable completion/failure result.

## High

### Browser delivery accepts view documents but never renders them

The shared frontplane accepts and publishes `AE_ui_*` snapshots in browser
builds, but `BrowserRuntime` exposes only `FrameOutput`; the browser adapter
paints only that canvas frame. A guest can receive successful import statuses
and launch as a blank or control-less application.

Evidence: `src/web.rs:31-35`, `src/web.rs:83-87`,
`src/bin/aedicule-web.rs:132-225`.

Required disposition: render the same adapter-neutral document in the browser,
or reject/diagnose an unsupported required view profile before launch.

### Reusing a revision after omission can display a stale slider value

Only the currently accepted revision is forbidden, so `A -> B -> A` is valid.
Native slider entities cache only their last synchronized guest revision. If A
contains a slider, B omits it, and the next A restores it with a different
value, the entity still says it synchronized to A and skips the update.

Evidence: `src/lib.rs:3726-3734`, `src/main.rs:470-475`,
`src/main.rs:1021-1041`.

Required disposition: add a host-owned monotonic acceptance generation or
cache the accepted value as well as revision; invalidate omitted IDs. Regress
`A(value 1) -> B(omitted) -> A(value 2)` through the actual native adapter.

### Lifecycle phase and rollback ownership are not enforced at the ABI boundary

Imports documented for configure or render are callable from every guest
export. Failed calls roll back audio/effects and pending UI, but not title,
menu, control, synth, or image mutations. A guest can mutate configuration from
an event/tick and retain part of a mutation after the surrounding call fails.

Evidence: `src/lib.rs:1636-1655`, `src/lib.rs:2447-2454`,
`src/lib.rs:2246-2254`, `WAT_ABI.md:148-154`.

Required disposition: track the active lifecycle phase, reject imports outside
their admitted phase, and stage every phase-owned mutation until its export
commits successfully.

### The advertised keyed tree is a flat per-widget structure with a multiplying budget

`UiSnapshot` and `UiBuilder` contain parallel panel/slider/button vectors and
ID sets. The adapter renders one pass per kind. This cannot represent nested or
interleaved semantic children, layout/focus order, or exhaustive adapter
handling. `max_controls` is independently allowed for every kind, so adding a
kind silently increases the aggregate resource limit.

Evidence: `src/lib.rs:905-913`, `src/lib.rs:1621-1629`,
`src/lib.rs:3525-3610`, `src/main.rs:1009-1085`.

Required disposition: introduce one ordered typed node arena/tree, one global
stable-ID index, parent-before-child validation, and aggregate node/depth/text
budgets before adding general widgets.

### Native widget behavior is not tested through the real widget path

Existing hit tests use synthetic occluding divs, and the button event test
calls the one-line event constructor directly. They do not click the actual
GPUI button, exercise slider subscriptions/phases, prove scheduler delivery,
or prove that a native control does not also emit a canvas pointer edge.

Evidence: `src/main.rs:502-559`, `src/main.rs:1019-1085`,
`src/main.rs:1541-1575`, `src/main.rs:1625-1684`.

Required disposition: add real GPUI button/slider event tests with a
guest-observed oracle and no test-only production bypass.

## Medium

### UI atomicity lacks surrounding-render failure coverage

Tests cover import-level invalid slider/button declarations, but not a valid
new document that reaches `AE_ui_end` and is followed by a trap, nonzero render
status, missing/invalid frame end, or incomplete UI transaction. Empty-document
removal, adapter reconciliation/removal, and hot reload also lack direct tests.

Evidence: `src/lib.rs:2054-2085`, `tests/integer_profile.rs:257-345`.

Required disposition: classify the complete failure set against a previously
accepted revision and assert both retained view and retained paint output.

### General applications are forced to tick and manufacture a canvas frame

Simulation rates below 1 Hz are rejected, events are rendered through the
simulation scheduler, and `Frontplane::render` requires a completed canvas
frame before publishing a view document. An idle form/document therefore wakes
periodically and emits a dummy frame.

Evidence: `src/lib.rs:479-527`, `src/lib.rs:2054-2085`,
`src/lib.rs:2269-2283`.

Required disposition: add event-driven `0/1` mode and make view/canvas sections
independently optional within one atomic visual-output transaction.

### Dynamic text, text input, and generic application events do not exist

Button and slider labels come from immutable configure-time menu/control
metadata. The key vocabulary is twelve game-oriented physical keys, and there
is no UTF-8 edit payload, IME composition, selection, clipboard, generic
modifiers, or semantic event envelope.

Evidence: `src/lib.rs:954-963`, `src/lib.rs:1463-1478`,
`src/main.rs:1043-1079`, `src/main.rs:1303-1320`.

Required disposition: follow `VIEW_PROTOCOL.md`: snapshot-owned strings,
semantic nodes, generic keyed events with bounded ephemeral payloads, and
native/browser IME behavior.

### Resource and frame budgets do not bound aggregate host allocations

Images have no count budget, accept zero-byte entries, and are not rolled back
on failed guest calls. Text and path limits are per object rather than aggregate
frame/document limits. A small guest-memory footprint can induce much larger
retained Rust allocations.

Evidence: `src/lib.rs:710-750`, `src/lib.rs:2846-2890`,
`src/lib.rs:3464-3498`, `src/lib.rs:3630-3655`.

Required disposition: add resource-count, total frame payload/segment, and
view-document node/text/depth budgets; stage resource mutations transactionally.

### ABI documentation and runtime bindings are separate sources of truth

The generated catalog stores signatures as strings while the Wasmtime linker
spells types again in Rust. Current document tests prove the Markdown matches
the catalog, not that the catalog matches executable bindings. Wrongly typed
`AE_after_restore` is also silently treated as absent, and the ABI minor export
is called but not negotiated.

Evidence: `src/wat_abi.rs:13-19`, `src/lib.rs:1728-1745`,
`src/lib.rs:1803-1820`, `src/lib.rs:2329-2345`,
`tests/wat_abi_doc.rs:3-33`.

Required disposition: derive docs and binding registration from a typed schema
or mechanically instantiate every declared signature; reject present-but-wrong
optional exports and define minor/profile negotiation before expanding AVP.

### Diagnostics can report success without producing a diagnostic

`AE_log` validates then discards the message. Several runtime adapter failures
are visible only in an overlay, not stderr/browser console, and an impossible
native slider lookup silently omits the widget.

Evidence: `src/lib.rs:3447-3458`, `src/main.rs:797-818`,
`src/main.rs:1021-1028`, `src/bin/aedicule-web.rs:68-76`.

Required disposition: retain bounded structured diagnostics and route identical
severity/code/message semantics to native stderr, browser console, and visible
recovery UI.

## Low

- The native adapter deep-clones complete frames/view snapshots and rebuilds
  widget layers on every GPUI render, even for an unchanged retained revision
  (`src/main.rs:992-1085`, `src/gpui_canvas.rs:121-144`). Event-driven idle,
  `Arc`-owned accepted outputs, indexed entities, and virtualization should
  precede large documents.
- `SPEC.md`, README status, and generated ABI prose had drifted over whether
  retained controls existed. Exact current wire facts should remain generated;
  architecture/status documents should link rather than duplicate tables.
- The database/storage review dimension is not applicable: Aedicule currently
  has no database layer. Future persistence is explicitly a capability profile,
  not ambient host data access.

## Suggested next green units

1. Fix the retained-slider `A -> B -> A` reconciliation defect with a real
   adapter regression.
2. Reject or implement every currently silent accepted capability, beginning
   with browser view documents and `AE_effect`.
3. Add lifecycle-phase tracking and transactional resource/metadata staging.
4. Strengthen atomic publication and actual-widget tests.
5. Begin AVP extraction and event-driven/view-only output only after those
   correctness savepoints are green.

## Pause lifecycle addendum

Date: 2026-07-23

Scope: ABI v0.3 pause declaration and semantic events, exact scheduler
rebasing, raw-input ownership/reconciliation, native and browser adapters,
hot reload, and the guest-only audio transport. This focused milestone pass
rechecked the applicable functionality, coverage, duplication, complexity,
resource lifetime, ABI, and error-handling dimensions above. FFI and database
dimensions remain inapplicable.

### Resolved warning: no-opt-in guests lost repeated raw edges

The first classifier tracked every guest even when it had no pause trigger, so
a repeated key-down or pointer-down was consumed. That contradicted the
additive contract and could have changed Ulam or any older guest merely by
running on ABI v0.3.

Resolution: `GuestSuspension::handle` now bypasses all classification when the
feature is disabled. A set-based regression proves repeated key, pointer, and
focus edges are delivered unchanged without an opt-in.

### Resolved warning: pre-barrier input could replay after resume

An ordinary edge already queued for the next tick could be followed by an
immediate pause trigger. Without an explicit barrier drain, its physical
release could be reconciled before the stale down edge finally reached the
guest after resume.

Resolution: both adapters execute due work and drain every earlier queued edge
before emitting the semantic paused event. Native scheduler and browser
runtime regressions prove the order.

### Resolved advisory: audio cursor preservation relied only on dependency prose

Resolution: the native test suite now drives Aedicule's nested
guest-mixer/player topology directly. It consumes one sample, observes silence
while paused, and then resumes at the next two samples; the separate cooldown
test proves paused wall time is excluded.

### Remaining manual boundary

The Web Audio context is browser- and device-owned. Unit/source contracts prove
that the Rust adapter stops animation work and requests context suspend/resume,
while browser startup integration proves the callable bridge exists. Audible
device-buffer tail and browser autoplay policy remain real-browser acceptance
checks, as documented in the ABI, rather than deterministic unit-test claims.
