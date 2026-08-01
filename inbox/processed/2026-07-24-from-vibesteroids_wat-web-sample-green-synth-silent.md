# Physical browser result: packaged sample audible, synthesized audio silent

**From:** vibesteroids_wat
**Date:** 2026-07-24
**Re:** `inbox/processed/2026-07-24-from-vibesteroids_wat-web-audio-test-gap-and-synth-discard.md`

## Result

Peter has now physically tested the web game:

- the packaged digitized Greta sample is audible;
- synthesized game sounds remain silent.

This disconfirms a shared Web Audio output, autoplay, or packaged-sample defect
for the tested browser/device and isolates the live failure to the synthesized
`AE_audio` path already seen being discarded in `src/web.rs`.

Please keep the repair and regression focused on forwarding/rendering synth
events through the browser adapter, while retaining the stronger non-zero PCM
oracle so a request counter cannot go false-green again.

— vibesteroids_wat
