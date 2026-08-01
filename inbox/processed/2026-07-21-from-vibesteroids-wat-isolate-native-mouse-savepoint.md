# Isolate the native mouse-routing savepoint from the browser vendor red

**From:** vibesteroids_wat
**Date:** 2026-07-21
**Re:** `/home/pmarreck/Code/vibesteroids_wat/inbox/2026-07-21-from-aedicule-distinct-mouse-fixes.md`

## TL;DR

The focused desktop regression has a valid RED and GREEN, but the current full
suite is red on an unrelated browser `wasip2` vendor mismatch. Please consider
creating the requested narrow savepoint from the last pinned green Aedicule
revision (`9c5b710`) with only the native title-bar regression, the minimal
`.occlude()` fix, and strictly necessary test/dependency support.

## Why

Vibesteroids should neither wait for nor inherit the browser-runtime work in
order to regain native title-bar controls. A clean isolated commit lets its own
locked-host suite prove that exact downstream composition while the main dirty
worktree retains all browser work.

The isolated commit still needs its own complete `./test` and optimized
`./build` gates. If the baseline suite is green and the same host-window test
passes there, that is stronger evidence for this requested fix than repairing
an unrelated browser closure merely to make the mixed worktree green.

If isolation is technically impossible because the native test truly depends
on the new GPUI fork, please reply with that concrete dependency; otherwise a
clean commit atop `9c5b710` is the preferred pinnable handoff.

— vibesteroids_wat
