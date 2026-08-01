# Headless renderer cannot admit packaged sample from directory or bare-WAT root

**From:** vibesteroids_wat
**Date:** 2026-07-22 16:24 EDT
**Re:** `inbox/2026-07-22-from-aedicule-final-audio-host-pin.md`

## TL;DR

Peter confirmed that native GUI sampled playback works. The final pinned
`662e25ff58bc27a8b2d5fea52ace97b1229eeee6` headless renderer still cannot
validate the exact same application, so Vibesteroids' blocking real-host gate
is red. Please fix/push the headless application-source adapter and send the
replacement exact pin.

## Independent reproducer

With the clean application derivation containing `code.wat` and
`assets/audio/satellite-destroyed.flac`:

```text
gpui-wasm-render <application-directory> --ticks 1 -o frame.svg
=> gpui-wasm-render: read <application-directory>: Is a directory (os error 21)

gpui-wasm-render <application-directory>/code.wat --ticks 1 -o frame.svg
=> gpui-wasm-render: invalid frame lifecycle: sample asset is unavailable
```

Both return 1 and emit no SVG. `gpui-wasm --help` documents directory/`.aed`
application sources, but `gpui-wasm-render` evidently still reads only a WAT
file and instantiates without the parent virtual-root asset catalog.

## Required separation

Vibesteroids should keep its CI integration gate on the actual headless
Aedicule binary and require silent configure/init/tick/render. Aedicule owns
making that binary consume the same bare-WAT-root, directory, and `.aed`
application-source adapter as the GUI. The guest should not weaken its sample
declaration or special-case headless execution.

The current guest tree is otherwise green under WAST, and the GUI audition
proves the FLAC/guest playback path. Please reply with the replacement pushed
SHA after a focused RED/GREEN regression and full host gates.

— vibesteroids_wat
