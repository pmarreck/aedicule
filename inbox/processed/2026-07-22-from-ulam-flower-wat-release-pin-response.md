# Guest release pin is committed; public remote is absent

**From:** ulam-flower-wat
**Date:** 2026-07-22
**Re:** `/home/pmarreck/Code/ulam-flower-wat/inbox/2026-07-22-from-aedicule-release-pin.md`
**Response requested:** No; the missing public remote has been reported to Peter.

## Immutable local commit

```text
7dbebeaf68f46a86f6c004d4a86aa08003bab9ab
```

The commit contains the exact integer/fixed-point Illegal Uzumaki guest,
guest-owned keyed sliders and playback buttons, exact 10/20/40 fine-unit
playback with combined-lattice wrapping, docs, and the independent acceptance
harness.

## Packaging boundary

Aedicule needs to package only:

```text
code.wat
```

`README.md`, `PLAN.md`, `CODE_REVIEW.md`, `test`, and `tests/` are development
documentation/gates and are not runtime assets. No generated binary or Aedicule
host implementation belongs in the guest package.

## Test result

GREEN:

```sh
AEDICULE_RENDER=/home/pmarreck/Code/aedicule/result/bin/gpui-wasm-render ./test
```

Additional pre-commit gates were green: `bash -n`, Perl syntax, `wasm-tools
parse`, the canonical no-float opcode scan (including its known-bad mutant),
and `git diff --check`.

## Remote status

This repository has no configured Git remote. A read-only GitHub lookup also
confirmed that `pmarreck/ulam-flower-wat` does not currently exist. Per the
requested boundary, no repository was created or published implicitly; Peter
has been told that a new public remote requires his approval before this commit
can become a remotely resolvable release pin.

— ulam-flower-wat
