//! The single declarative source for the WAT-facing ABI reference. The build
//! compares its rendered Markdown with `WAT_ABI.md` so client documentation
//! cannot silently diverge from the names that the host admits.

use std::fmt::Write as _;

pub const ABI_MAJOR: i32 = 0;
pub const ABI_MINOR: i32 = 0;
pub const IMPORT_MODULE: &str = "aedicule.v0";

/// One capability import exposed to an Aedicule WAT application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatAbiImport {
    pub name: &'static str,
    pub signature: &'static str,
    pub summary: &'static str,
}

pub const WAT_ABI_IMPORTS: &[WatAbiImport] = &[
    WatAbiImport {
        name: "AE_title",
        signature: "(param ptr i32) (param len i32) (result i32)",
        summary: "Sets the UTF-8 application title during `AE_configure`.",
    },
    WatAbiImport {
        name: "AE_menu_item",
        signature: "(param id i32) (param ptr i32) (param len i32) (param shortcut i32) (param flags i32) (result i32)",
        summary: "Declares an action menu item; `flags & 1` is a separator and shortcut codes are host-defined.",
    },
    WatAbiImport {
        name: "AE_synth_voice",
        signature: "(param program_id i32) (param waveform i32) (param delay_ms i32) (param duration_ms i32) (param frequency_start_millihz i32) (param frequency_mid_millihz i32) (param frequency_end_millihz i32) (param gain_start_ppm i32) (param gain_peak_ppm i32) (param gain_end_ppm i32) (param filter i32) (param filter_start_millihz i32) (param filter_end_millihz i32) (param cooldown_ms i32) (result i32)",
        summary: "Declares a bounded synthesized-audio program during `AE_configure`.",
    },
    WatAbiImport {
        name: "AE_image_define",
        signature: "(param id i32) (param ptr i32) (param len i32) (param flags i32) (result i32)",
        summary: "Defines an image resource; `flags & 1` selects raw RGBA, otherwise bytes are encoded image data.",
    },
    WatAbiImport {
        name: "AE_image_release",
        signature: "(param id i32) (result i32)",
        summary: "Releases a previously defined image resource.",
    },
    WatAbiImport {
        name: "AE_frame_begin",
        signature: "(param r f32) (param g f32) (param b f32) (param a f32) (result i32)",
        summary: "Begins one render transaction with normalized RGBA background components.",
    },
    WatAbiImport {
        name: "AE_transform_push",
        signature: "(param m11 f32) (param m12 f32) (param m21 f32) (param m22 f32) (param tx f32) (param ty f32) (result i32)",
        summary: "Pushes an affine transform inside the current frame.",
    },
    WatAbiImport {
        name: "AE_transform_pop",
        signature: "(result i32)",
        summary: "Pops the current frame transform.",
    },
    WatAbiImport {
        name: "AE_path_begin",
        signature: "(param id i32) (result i32)",
        summary: "Begins a uniquely identified path in the current frame.",
    },
    WatAbiImport {
        name: "AE_path_move",
        signature: "(param x f32) (param y f32) (result i32)",
        summary: "Appends a move segment to the active path.",
    },
    WatAbiImport {
        name: "AE_path_line",
        signature: "(param x f32) (param y f32) (result i32)",
        summary: "Appends a line segment to the active path.",
    },
    WatAbiImport {
        name: "AE_path_quad",
        signature: "(param cx f32) (param cy f32) (param x f32) (param y f32) (result i32)",
        summary: "Appends a quadratic Bézier segment to the active path.",
    },
    WatAbiImport {
        name: "AE_path_cubic",
        signature: "(param c1x f32) (param c1y f32) (param c2x f32) (param c2y f32) (param x f32) (param y f32) (result i32)",
        summary: "Appends a cubic Bézier segment to the active path.",
    },
    WatAbiImport {
        name: "AE_path_close",
        signature: "(result i32)",
        summary: "Closes the active path.",
    },
    WatAbiImport {
        name: "AE_path_end",
        signature: "(param width f32) (param fill_rgba i32) (param stroke_rgba i32) (param flags i32) (result i32)",
        summary: "Commits the active path; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_sprite",
        signature: "(param id i32) (param image_id i32) (param src_x f32) (param src_y f32) (param src_w f32) (param src_h f32) (param dst_x f32) (param dst_y f32) (param dst_w f32) (param dst_h f32) (param pivot_x f32) (param pivot_y f32) (param tint_rgba i32) (param flags i32) (result i32)",
        summary: "Draws a bounded source rectangle from a defined image; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_line",
        signature: "(param id i32) (param x1 f32) (param y1 f32) (param x2 f32) (param y2 f32) (param width f32) (param rgba i32) (result i32)",
        summary: "Draws a uniquely identified line.",
    },
    WatAbiImport {
        name: "AE_circle",
        signature: "(param id i32) (param x f32) (param y f32) (param radius f32) (param width f32) (param rgba i32) (param flags i32) (result i32)",
        summary: "Draws a uniquely identified circle; `flags & 1` fills it.",
    },
    WatAbiImport {
        name: "AE_text",
        signature: "(param id i32) (param ptr i32) (param len i32) (param x f32) (param y f32) (param size f32) (param rgba i32) (param flags i32) (result i32)",
        summary: "Draws UTF-8 text; `flags & 1` centers it.",
    },
    WatAbiImport {
        name: "AE_frame_end",
        signature: "(result i32)",
        summary: "Completes the current frame after all path and transform stacks balance.",
    },
    WatAbiImport {
        name: "AE_audio",
        signature: "(param id i32) (param volume f32) (param pitch f32) (param flags i32) (result i32)",
        summary: "Queues one declared synthesized-audio program with volume 0..1 and pitch 0.25..4.",
    },
    WatAbiImport {
        name: "AE_effect",
        signature: "(param kind i32) (param a i32) (param b i32) (result i32)",
        summary: "Queues a host effect: redraw (1), quit (2), set cursor (3), or persist snapshot (4).",
    },
    WatAbiImport {
        name: "AE_log",
        signature: "(param level i32) (param ptr i32) (param len i32) (result i32)",
        summary: "Accepts a bounded UTF-8 diagnostic message; version 0 does not expose its sink to the guest.",
    },
];

/// Renders the exact checked-in WAT client reference from the ABI declarations.
pub fn wat_abi_markdown() -> String {
    let mut document = format!(
        "<!-- Generated by `AE_UPDATE_WAT_ABI=1 cargo run --bin generate-wat-abi`; do not edit by hand. -->\n\n# Aedicule WAT ABI v{ABI_MAJOR}.{ABI_MINOR}\n\nThis is the complete client-facing ABI for WAT applications accepted by Aedicule today. The only import module is `{IMPORT_MODULE}`. Every function at this boundary is named `AE_*`; the provisional `host.v0` / `fp_*` names are rejected.\n\nAll imported functions return a status: `0` succeeds; any non-zero result rejects the current guest transaction. Strings are UTF-8 byte slices in the guest's exported `memory`. IDs and packed colors travel as `i32` bit patterns. The host validates pointers, finite numeric values, object limits, and frame structure.\n\n## Guest exports\n\n```wat\n(memory (export \"memory\") MIN_PAGES)\n(func (export \"AE_abi_major\") (result i32))\n(func (export \"AE_abi_minor\") (result i32))\n(func (export \"AE_configure\") (result i32))\n(func (export \"AE_init\") (param i32 i32 f32 f32) (result i32))\n(func (export \"AE_event\") (param i32 i32 f32 f32) (result i32))\n(func (export \"AE_tick\") (param i32) (result i32))\n(func (export \"AE_render\") (result i32))\n(func (export \"AE_state_ptr\") (result i32))\n(func (export \"AE_state_len\") (result i32))\n(func (export \"AE_state_schema\") (result i32))\n(func (export \"AE_tick_rate\") (param i32 i32) (result i32 i32)) ;; optional\n(func (export \"AE_after_restore\") (result i32)) ;; optional\n```\n\n`AE_abi_major` must return `{ABI_MAJOR}`. `AE_tick_rate(current_numerator, current_denominator)` chooses a reduced or unreduced rational simulation rate in the inclusive 1..=1000 Hz range, with denominator at most 1,000,000. It receives the currently agreed **simulation** rate, never the ambient display rate. Returning `(0, 0)`, or omitting the export, follows the current display mode.\n\n## Lifecycle and timing\n\nA normal startup is `AE_abi_*`, `AE_configure`, `AE_init`, one display-refresh event, optional `AE_tick_rate`, then `AE_render`. Reload configures and initializes a new instance, restores a compatible snapshot (and calls `AE_after_restore` if present), delivers the current display-refresh event, selects its rate, and renders before it can replace the active instance.\n\n`AE_event` has `(kind, code, a, b)`. Kind `9` is `AE_EVENT_DISPLAY_REFRESH`: `code = refresh_numerator_hz`, `a = refresh_denominator`, `b = 0`. It is emitted once when the initial display mode becomes known and only again when that mode changes. A guest must treat its absence as “same as the last event”; it cannot query display timing. The host performs the display event before calling `AE_tick_rate`.\n\n`AE_tick(ticks)` receives only bounded whole fixed steps. The host accumulates monotonic time exactly at the agreed rational rate and may report overload by dropping old due ticks while preserving ordered input edges.\n\n## Host display timing override\n\nSet `AE_DISPLAY_REFRESH_RATE` before process startup to override the initial nominal display rate while native display discovery is unavailable. The value is exact and normalized by greatest common divisor: `60000/1001` is a rational rate, `5994/100` becomes `2997/50`, and ordinary decimals are parsed as written without floating point. Canonical nominal spellings `23.976`, `29.97`, `59.94`, and `119.88` select respectively `24000/1001`, `30000/1001`, `60000/1001`, and `120000/1001`. Any other decimal, such as `59.97`, stays exact as typed. Invalid values fail startup; an unset variable retains the native/default source. This is host configuration, not a guest display-query capability.\n\n## Host imports\n\nAll imports use `(import \"{IMPORT_MODULE}\" \"NAME\" (func ...))`.\n\n"
    );

    document.push_str(
        "## Input events\n\n`AE_event(kind, code, a, b)` carries host input at an ordered fixed-step boundary. A host may omit an event kind it cannot observe, but it must not invent application semantics.\n\n| Kind | Event | `code`, `a`, `b` |\n| ---: | --- | --- |\n| 1 | Key down | `code` is a physical-key ID; `a = b = 0` |\n| 2 | Key up | Same key ID; `a = b = 0` |\n| 3 | Pointer move | `code = 0`; `a = x`, `b = y` logical pixels |\n| 4 | Pointer down | `code` is button ID; `a = x`, `b = y` logical pixels |\n| 5 | Pointer up | `code` is button ID; `a = x`, `b = y` logical pixels |\n| 6 | Viewport | `code = 0`; `a = width`, `b = height` logical pixels |\n| 7 | Menu action | `code` is the guest-declared action ID; `a = b = 0` |\n| 8 | Focus | `code = 1` when focused, `0` when unfocused; `a = b = 0` |\n| 9 | Display refresh | `code = numerator_hz`; `a = denominator`, `b = 0` |\n| 10 | Pointer scroll | `code` is unit ID; `a = horizontal delta`, `b = vertical delta` |\n\nPhysical-key IDs are `1` left, `2` right, `3` up, `4` space, `5` P, `6` R, `7` F, `8` K, `9` B, `10` H, `11` Escape, and `12` F1. Pointer button IDs are `1` primary/left, `2` secondary/right, and `3` middle/wheel-click. Native canvas adapters emit down and up edges for all three IDs.\n\nPointer-scroll unit IDs are `1` lines and `2` logical pixels. Positive `a` means leftward motion; positive `b` means upward motion. Hosts preserve both axes and omit zero-delta scroll events.\n\n",
    );

    for import in WAT_ABI_IMPORTS {
        let _ = writeln!(
            document,
            "### `{}`\n\n```wat\n(func ${} {})\n```\n\n{}\n",
            import.name, import.name, import.signature, import.summary
        );
    }

    document.push_str(
        "## Frame rules\n\nCall `AE_frame_begin` once per `AE_render`, produce drawing commands, then call `AE_frame_end`. Transform pushes/pops and paths must balance. Any invalid, partial, or over-budget frame is rejected atomically; a host adapter never paints a partial frame.\n\n## Minimal shape\n\n```wat\n(module\n  (import \"aedicule.v0\" \"AE_frame_begin\" (func $begin (param f32 f32 f32 f32) (result i32)))\n  (import \"aedicule.v0\" \"AE_frame_end\" (func $end (result i32)))\n  (memory (export \"memory\") 1)\n  (func (export \"AE_abi_major\") (result i32) i32.const 0)\n  (func (export \"AE_abi_minor\") (result i32) i32.const 0)\n  (func (export \"AE_configure\") (result i32) i32.const 0)\n  (func (export \"AE_init\") (param i32 i32 f32 f32) (result i32) i32.const 0)\n  (func (export \"AE_event\") (param i32 i32 f32 f32) (result i32) i32.const 0)\n  (func (export \"AE_tick\") (param i32) (result i32) i32.const 0)\n  (func (export \"AE_render\") (result i32)\n    f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop\n    call $end)\n  (func (export \"AE_state_ptr\") (result i32) i32.const 0)\n  (func (export \"AE_state_len\") (result i32) i32.const 0)\n  (func (export \"AE_state_schema\") (result i32) i32.const 1))\n```\n"
    );
    document
}
