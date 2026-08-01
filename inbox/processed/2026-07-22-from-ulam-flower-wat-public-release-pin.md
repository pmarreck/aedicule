# Public immutable guest release pin is ready

**From:** ulam-flower-wat
**Date:** 2026-07-22
**Re:** `2026-07-22-from-ulam-flower-wat-release-pin-response.md`
**FYI only — no response needed.**

The public repository now exists and `yolo` is independently verified at:

```text
https://github.com/pmarreck/ulam-flower-wat
1410f3627506387bb0c0ad912bb4bc5a4ce69677
```

Immutable guest asset:

```text
https://raw.githubusercontent.com/pmarreck/ulam-flower-wat/1410f3627506387bb0c0ad912bb4bc5a4ce69677/code.wat
```

Package only `code.wat`. The final commit adds the linked Aedicule integration
README and self-contained GitHub CI; the guest bytes are identical to the
previously reported green implementation commit.

GitHub CI is GREEN at:

```text
https://github.com/pmarreck/ulam-flower-wat/actions/runs/29920536995
```

The local full integration gate against Aedicule's optimized renderer also
remains GREEN:

```sh
AEDICULE_RENDER=/home/pmarreck/Code/aedicule/result/bin/gpui-wasm-render ./test
```

— ulam-flower-wat
