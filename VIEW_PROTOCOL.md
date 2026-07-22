# Aedicule View Protocol

Status: design proposal; not yet a WAT ABI contract.

The **Aedicule View Protocol (AVP)** is the platform-neutral semantic UI plane
for Aedicule applications. A guest publishes an immutable **view document**;
the host validates it, commits it atomically, and reconciles stable node IDs
into GPUI desktop controls, browser controls, accessibility nodes, and
headless test models.

The existing `AE_ui_begin` / `AE_ui_end` transaction is the working kernel of
this design. Its current panels, sliders, and buttons form a narrow
**v0 native-controls profile**, not the completed general application
protocol. “LiveView-like” describes its inspiration, but is not its name or a
claim that Aedicule uses Phoenix LiveView's server, transport, or diff model.

## Goals

- Let a WAT guest define complete applications, not only games or custom
  graphical demos.
- Keep application state, behavior, text, layout intent, and action semantics
  guest-owned.
- Render one semantic document through native desktop, browser, accessibility,
  and headless adapters without application-specific Rust.
- Preserve the existing all-or-nothing validation, stable identity, rollback,
  bounded execution, and transactional hot-reload guarantees.
- Keep the ABI legible enough to author and debug directly in WAT.
- Grant platform services only as explicit, bounded capabilities; never add
  ambient WASI authority.

“Any application” means any application expressible through the capabilities
the host deliberately grants. It does not mean that an untrusted guest gains
arbitrary operating-system access.

## Canonical model

AVP is **declarative and retained**, not immediate-mode UI:

1. The guest owns the desired semantic document.
2. It submits a complete revision only when that document changes.
3. The host validates the complete candidate before publishing any part.
4. The host retains the last accepted revision when a submission is omitted or
   rejected.
5. Stable IDs let each adapter preserve appropriate transient mechanics such
   as focus, selection handles, hover, scroll position, and accessibility
   identity.
6. Guest-authored values remain authoritative. Host transient state never
   silently becomes application state.

AVP initially uses complete documents, not guest-authored patches. Host-side
reconciliation may calculate a diff internally. A wire-level patch protocol is
deferred until profiling proves that complete bounded documents are a real
bottleneck.

## Application output transaction

`AE_render` should produce one visual-output transaction containing zero or
more independently optional sections:

- a custom drawing frame;
- a changed AVP view document.

Every section emitted by one call commits atomically only if the export returns
success and every opened section is complete. A view-only application must not
manufacture a dummy canvas frame. Omitting a section retains its previous
accepted state; explicitly submitting an empty view document removes every
view node.

The first successful render must produce at least one visible section or an
explicitly empty view document. This preserves useful startup validation while
allowing forms, documents, and other non-canvas applications.

Audio, effects, and future service requests remain bounded outputs of the
lifecycle call that caused them. They are not replayed merely because a visual
document is rendered again, and `AE_render` remains pure with respect to guest
application state.

## View document

A document is an ordered, rooted tree. Every node has:

- a nonzero stable ID unique across the complete document;
- a node kind;
- a parent ID, except for the root;
- sibling order determined by declaration order;
- semantic state and validated node-specific properties; and
- layout constraints, not adapter-computed absolute positions.

Parents must be declared before children. The host rejects duplicate IDs,
unknown parents, cycles, excessive depth, excessive children, invalid UTF-8,
unsupported properties, and aggregate budget exhaustion before publishing the
revision.

The current absolute Q16.16 panel/control declarations remain a compatibility
profile during development. The general profile uses guest-authored layout
constraints and host-computed physical geometry so the same document can
adapt to desktop fonts, browser metrics, localization, accessibility scaling,
and narrow viewports.

## Initial node vocabulary

The first useful general-application slice should remain deliberately small:

- root/window content;
- row and column containers;
- overlay/stack container;
- scroll container;
- text: label, heading, paragraph, and status roles;
- button;
- checkbox or toggle;
- exact integer slider;
- single-line text input;
- image; and
- canvas region containing the existing custom scene frame.

Multiline editing, tabs, split panes, menus, popovers, dialogs, lists, tables,
trees, and virtualized collections follow as separately testable profiles.
Canvas is one node kind in a general application, not the root abstraction.

## Layout

Phase one uses a bounded flex-style layout rather than reproducing all of CSS:

- row or column direction;
- start, center, end, space-between, and stretch alignment;
- gap and per-edge padding;
- fixed, content-sized, fill, and fractional main/cross-axis sizing;
- minimum and maximum dimensions;
- grow and shrink factors;
- clipping and scrolling; and
- hidden/visible state.

Lengths are signed Q16.16 logical pixels where a scalar is required. Size modes
are enums rather than magic sentinel numbers. The guest specifies constraints;
the adapter performs measurement and placement. Layout results are not sent
back every frame. A viewport or meaningful constraint change produces one
ordered event, after which the guest may publish a different document.

Grid and free absolute positioning are later additive profiles. Absolute
positioning remains useful for overlays but must not be the only way to create
ordinary application UI.

## Text, localization, and styling

Snapshot-owned UTF-8 text may change on any revision. Buttons and controls do
not borrow their labels from configure-time menu declarations. Menus and view
nodes may independently reference the same guest action ID.

The first styling contract is semantic:

- text roles instead of arbitrary font files;
- primary, secondary, destructive, selected, disabled, busy, invalid, and
  read-only states;
- foreground/background accent roles;
- density and emphasis tokens; and
- host theme, contrast, text-scale, and reduced-motion events.

This keeps native and browser adapters coherent. Arbitrary CSS or GPUI style
objects are not ABI types. Packaged fonts and richer appearance can be added
later under explicit resource budgets.

Guest-owned strings are the application's source text. Frontplane-owned
fallback and policy strings remain in Aedicule's locale catalogue. A later
localization profile may add stable message keys and parameters without making
the host the owner of application prose.

## Events and dynamic payloads

General controls need one typed event path rather than a growing mixture of
menu events and widget-specific exports. The proposed shape is:

~~~wat
(func (export "AE_view_event")
  (param node_id i32) (param kind i32)
  (param a i32) (param b i32)
  (param payload_token i32) (param payload_len i32)
  (result i32))
~~~

Integer fields carry toggles, exact scalar values, selection ranges, phases,
and modifier masks. UTF-8 text and other bounded data use an ephemeral payload
token. During the callback the guest may copy slices through a bounded
`AE_event_payload_read` import. A token is invalid outside that callback. This
avoids fixed guest scratch-buffer sizes and keeps host ownership explicit.

Initial event kinds include activate, value-changing, value-committed,
text-edited, text-committed, focus-gained/lost, selection-changed,
scroll-changed, and close-requested. IME composition is a required part of the
text-input profile, not a later key-code emulation. Adapters preserve ordered
delivery and must not duplicate a semantic control event as a canvas pointer
event.

Exact numbers, flags, ranges, and identifiers stay integer-only. This lets the
general view profile remain usable by zero-float guests.

## Event-driven applications

General applications must be able to idle. The proposed simulation-rate result
`0/1` means **event-driven**: no `AE_tick` calls and no periodic wake merely to
keep the application alive. `(0, 0)` retains its existing meaning of following
the current display rate.

Input or a completed host-service request wakes the guest, delivers the ordered
event batch, then requests one render. An application that needs simulation may
continue selecting any supported positive rational rate. A later timer
capability supplies explicit bounded timer events; it does not reintroduce an
ambient wall clock.

## Focus and accessibility

Accessibility is part of the semantic contract from the first general slice.
Each interactive node provides or derives:

- role;
- accessible name and optional description;
- value, checked/selected/expanded/invalid/busy state;
- enabled and focusable state;
- deterministic traversal order; and
- relationships such as label-for and described-by.

The host implements keyboard traversal, platform focus rings, screen-reader
nodes, high-contrast behavior, and native text editing mechanics. The guest
owns meaningful state and responds to semantic events. Pointer-only controls
are invalid in the general profile.

## Application shell

After the first forms slice, AVP needs adapter-neutral declarations for:

- arbitrary application menus and shortcuts;
- toolbars and status regions;
- tabs and split panes;
- dialogs, sheets, popovers, and context menus;
- lists, tables, trees, and virtualized collections;
- multiple windows and close/dirty-state negotiation; and
- drag/drop and file-drop targets.

These are semantic structures. A guest does not draw imitations of native
controls merely to receive ordinary application behavior.

## Host-service capabilities

UI alone does not make a general application platform. Nontrivial apps also
need an asynchronous request/result protocol with guest request IDs and opaque
host handles. Candidate capability profiles are:

- packaged read-only assets;
- user-mediated open/save pickers and scoped file handles;
- application-scoped persistent storage;
- clipboard read/write;
- drag/drop payloads;
- bounded HTTP requests under declared origin policy;
- notifications;
- print/export; and
- explicit timers.

Every service is denied unless both the guest manifest and host policy grant
it. Results return as ordered events. Handles are guest-scoped, unforgeable at
the ABI boundary, bounded, revoked on reload unless deliberately transferred,
and never expose native pointers or paths unnecessarily.

The host must reject an unavailable capability when requested. It must not
accept a request and silently discard it.

## Adapter parity

The adapter-neutral core owns document validation and produces one immutable
accepted model. Desktop GPUI, browser, accessibility, and headless adapters
consume that same model.

A profile is not considered delivered until:

1. core validation and rollback tests pass;
2. native reconciliation and event tests pass;
3. browser behavior is equivalent or explicitly reported unsupported before
   launch;
4. a deterministic headless model can inspect document structure and inject
   semantic events; and
5. a downstream non-game guest exercises it without application logic in
   Aedicule.

## Budgets and versioning

The view document needs aggregate limits for node count, depth, total UTF-8
bytes, per-string bytes, child count, event payload bytes, retained adapter
entities, decoded resources, and service requests. One aggregate node budget
must not multiply silently as new node kinds are added.

Capability/profile negotiation belongs in the ABI minor contract. A guest must
be able to require the profiles it uses and fail before initialization when an
adapter lacks one. Optional Wasm exports with the wrong type are errors, not
silently absent features.

During the pre-v1 period, the proposed hard cutover is:

- `AE_ui_*` becomes `AE_view_*`;
- Rust `UiSnapshot` becomes `ViewDocument`;
- the current flat absolute declarations become an explicitly named
  compatibility profile; and
- exact signatures remain generated into the single build-checked
  `WAT_ABI.md` reference.

No hard cutover should land until downstream guests and adapter conformance
tests are ready to move in the same green savepoint.

## Proposed implementation sequence

1. **Name and model:** extract an adapter-neutral `view` module, rename the
   accepted snapshot to `ViewDocument`, use one typed node collection and one
   aggregate budget, and retain the current control profile unchanged.
2. **Output lifecycle:** allow view-only renders and event-driven `0/1` guests;
   test atomic publication after every surrounding-render failure class.
3. **General forms slice:** add root, row/column, text, button, toggle, integer
   slider, text input, canvas region, dynamic UTF-8, generic view events, focus,
   IME, and accessibility.
4. **Parity:** implement the same document in desktop, browser, and headless
   adapters with semantic event injection and reconciliation tests.
5. **Application shell:** arbitrary menus, scrolling collections, dialogs,
   tabs/splits, and window lifecycle.
6. **Services:** add explicit async capability handles for assets, storage,
   files, clipboard, HTTP, timers, and export one profile at a time.
7. **Falsification:** build a small editor or structured-data application in a
   separate WAT repository before claiming general-purpose readiness.

## Decisions still requiring evidence

- Keep call-oriented WAT imports or move large documents to a packed binary
  stream. Start with calls; profile before sacrificing hand-authored clarity.
- Whether layout measurement feedback is ever guest-visible. Prefer constraint
  events over a synchronous measure/re-render loop.
- Which minimum text-input and IME behavior GPUI web can provide consistently
  across desktop browsers and mobile Safari.
- Whether multiple windows belong in the first stable protocol major or an
  additive capability profile.
- How hot reload transfers outstanding service requests and opaque handles
  without violating transactional replacement.
