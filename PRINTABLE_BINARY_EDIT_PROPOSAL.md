# Printable Binary Edit — WAT Client Proposal

## Purpose

Printable Binary Edit is a deliberately non-game Aedicule client: a full-window
text editor for arbitrary binary files. Opening a file decodes its bytes into
the canonical printable-binary UTF-8 representation. Saving validates and
encodes the edited printable-binary text back into raw bytes; the textual
projection is never written as the target binary format.

Besides being useful for inspecting embedded ASCII and changing headers, this
client is an acceptance test for Aedicule's claim to support general
applications. The frontplane must provide generic editor and user-authorized
file capabilities without learning printable-binary semantics.

## Initial user contract

1. **Open** uses a native file picker or document-open event. Aedicule grants a
   capability for only the selected file and supplies bounded bytes; WAT never
   receives ambient filesystem access.
2. The client decodes every input byte into canonical printable-binary text and
   opens that text in one full-window editor.
3. **Save** validates the complete edited Unicode-scalar set as a classifier.
   If any scalar is outside the printable-binary alphabet, saving is rejected
   without touching the target and every invalid location can be selected.
4. **Strip invalid characters and save** is a separate, explicit action. It
   reports the number of scalars that would be removed before proceeding, then
   validates the resulting complete set again.
5. Successful save encodes printable-binary text back to raw bytes and replaces
   the selected target atomically. **Save As** grants a new destination through
   the native picker. Encoding, permission, short-write, and replacement
   failures preserve the original file.
6. If the file changed externally since it was opened or last saved, ordinary
   Save refuses to overwrite it and offers Reload or Save As. The first version
   may use a content digest rather than platform-specific timestamps.
7. Undo/redo, selection, clipboard, search, and keyboard navigation operate on
   the editable text projection. The status surface reports text position,
   decoded byte offset where defined, decoded byte length, and modified state.

Strict rejection is the default because silent stripping can alter bytes. The
strip action is useful, but it is intentionally conspicuous and independently
undoable before any disk write.

## Ownership boundary

The WAT application owns:

- printable-binary decode, validation, stripping, and encode semantics;
- the document model, undo history, dirty state, search, and status values;
- decisions about when to request Open, Save, Save As, or conflict resolution;
  and
- all client-specific tests and presentation choices.

Aedicule owns generic capabilities:

- semantic multiline editable text with selection, composition/IME,
  accessibility, clipboard, keyboard navigation, and viewport virtualization;
- user-mediated file-open and file-save handles;
- bounded/chunked reads and atomic writes with stable error results;
- external-change/conflict evidence tied to a granted handle;
- drag/drop and platform document associations; and
- native, browser, and headless-test adapters for the same observable contract.

The printable-binary transform does **not** belong in Aedicule's host ABI. It
can live in the guest, or in a guest-side `lib/` module compiled or translated
from the existing printable-binary specification/library.

## Browser mapping

On the web, Open maps to the File System Access API where available and to a
file input plus download-based Save As otherwise. The client must be told
whether an opened capability supports in-place atomic replacement. IndexedDB
may retain drafts, but it must not masquerade as authority to overwrite a local
file.

## Bounded first slice

The first implementation may set an explicit whole-document byte limit rather
than pretending to handle multi-gigabyte files. The UI must reject oversized
inputs before allocating their decoded expansion and report both the input
limit and maximum printable expansion. A later chunked editor can use a rope or
piece table plus a virtualized viewport after measurements justify it.

## Mechanically falsifiable acceptance

- Decode then encode is byte-identical for all 256 single-byte values, boundary
  sequences, mixed ASCII/control bytes, and seeded arbitrary byte arrays.
- Encode then decode is canonical for every accepted printable-binary string.
- Validation classifies complete mixed sets, not isolated hand-picked scalars.
- A mutant that accepts one non-alphabet Unicode scalar fails the classifier.
- Strip removes every and only invalid scalar, reports the exact count, and
  leaves the remaining sequence in order.
- Invalid text, malformed printable sequences, encoding failure, short writes,
  replacement failure, and an external-change conflict leave the original
  bytes unchanged.
- Successful Save writes the decoded raw bytes, never the visible UTF-8 editor
  representation.
- Open/save paths with spaces and non-ASCII names pass on every delivery target.
- Native and browser adapter tests drive synthetic edit, selection, save, and
  conflict events without sleeps; headless tests inject file capabilities and
  assert returned bytes entirely in memory.

## Architectural value

This client should become the second unrelated AVP acceptance application. If
it requires Aedicule to hardcode printable-binary widgets or file semantics,
the protocol is still an application-specific engine. If it can be built from
generic text, command, dialog, and capability nodes, AVP has crossed an
important line toward a real cross-platform application frontplane.
