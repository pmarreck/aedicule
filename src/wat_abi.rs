//! The single declarative source for the WAT-facing ABI reference. The build
//! compares its rendered Markdown with `WAT_ABI.md` so client documentation
//! cannot silently diverge from the names that the host admits.

use std::fmt::Write as _;

pub const ABI_MAJOR: i32 = 0;
pub const ABI_MINOR: i32 = 10;
pub const IMPORT_MODULE: &str = "aedicule.v0";
pub const LLM_GUIDE_VERSION: &str = "0.3.0";
const LLM_GUIDE_CANONICAL_URL: &str =
    "https://github.com/pmarreck/aedicule/blob/yolo/GUIDE_FOR_LLMS.md";
const LLM_GUIDE_ABI_MARKER: &str = "{{GENERATED_ABI_REFERENCE}}";

/// One capability import exposed to an Aedicule WAT application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatAbiImport {
    pub name: &'static str,
    pub signature: &'static str,
    pub summary: &'static str,
}

/// One explicitly non-callable capability direction shown beside the current
/// ABI without assigning it an import name or prematurely freezing a signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatAbiProposal {
    pub slug: &'static str,
    pub title: &'static str,
    pub status: &'static str,
    pub summary: &'static str,
    pub adapters: &'static str,
    pub evidence_url: &'static str,
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
        name: "AE_action",
        signature: "(param id i32) (param label_ptr i32) (param label_len i32) (param flags i32) (result i32)",
        summary: "Declares a labeled action for native buttons without creating a menu item; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_pause_trigger",
        signature: "(param kind i32) (param code i32) (param flags i32) (result i32)",
        summary: "Registers one typed host-consumed pause/wake trigger during `AE_configure`; version 0 supports stable key kind `1` and requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_motion_interest",
        signature: "(param kind i32) (param rate_hz i32) (param flags i32) (result i32)",
        summary: "Registers one motion-sensor interest during `AE_configure`: kind `1` is the host-derived shake gesture (rate must be 0), kind `2` is the six-axis sample stream (rate 0 selects the 60 Hz default, else 1..=120); `flags = 0`. Interests are accepted even where sensors are absent or permission is denied, in which case no motion events ever arrive - keep a manual affordance fallback.",
    },
    WatAbiImport {
        name: "AE_touch_interest",
        signature: "(param max_contacts i32) (param flags i32) (result i32)",
        summary: "Opts into ordered raw touch-contact event kinds `11` through `14` during `AE_configure`, with a guest-selected simultaneous-contact bound from 1 through 16; version 0 requires `flags = 0`. Guests that omit this import retain touch-to-primary-pointer compatibility.",
    },
    WatAbiImport {
        name: "AE_slider_i32",
        signature: "(param id i32) (param label_ptr i32) (param label_len i32) (param min i32) (param max i32) (param step i32) (param initial i32) (result i32)",
        summary: "Declares an exact stepped integer slider during `AE_configure`; changes arrive through `AE_control_event`.",
    },
    WatAbiImport {
        name: "AE_external_link",
        signature: "(param id i32) (param label_ptr i32) (param label_len i32) (param url_ptr i32) (param url_len i32) (param flags i32) (result i32)",
        summary: "Declares one labeled, credential-free HTTPS destination during `AE_configure`; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_ui_begin",
        signature: "(param revision i32) (result i32)",
        summary: "Begins one guest-authoritative keyed native-UI snapshot during `AE_render`; the revision must differ from the last accepted snapshot.",
    },
    WatAbiImport {
        name: "AE_ui_end",
        signature: "(result i32)",
        summary: "Completes the current native-UI snapshot, which is published atomically only if the surrounding render also succeeds.",
    },
    WatAbiImport {
        name: "AE_control_panel_q16",
        signature: "(param id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param rgba i32) (param flags i32) (result i32)",
        summary: "Places a guest-sized native-control panel in the current UI snapshot using Q16.16 logical-pixel geometry; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_slider_place_q16",
        signature: "(param id i32) (param panel_id i32) (param value i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param label_placement i32) (param flags i32) (result i32)",
        summary: "Places a declared native integer slider in the current UI snapshot; the guest-provided value and Q16.16 bounds remain authoritative.",
    },
    WatAbiImport {
        name: "AE_button_place_q16",
        signature: "(param id i32) (param panel_id i32) (param action_id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param flags i32) (result i32)",
        summary: "Places a declared action as a keyed native button; flag bit 0 is the guest-authored selected state and all other bits are reserved.",
    },
    WatAbiImport {
        name: "AE_external_link_place_q16",
        signature: "(param id i32) (param panel_id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param flags i32) (result i32)",
        summary: "Places one declared external link in a guest-owned panel using Q16.16 logical-pixel geometry; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_text_field",
        signature: "(param id i32) (param label_ptr i32) (param label_len i32) (param max_scalars i32) (param flags i32) (result i32)",
        summary: "Declares one bounded text-entry control during `AE_configure`; committed text arrives as integer scalar events, never as a host write into guest memory. Version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_text_field_place_q16",
        signature: "(param id i32) (param panel_id i32) (param x_q16 i32) (param y_q16 i32) (param width_q16 i32) (param height_q16 i32) (param flags i32) (result i32)",
        summary: "Places one declared text field in a guest-owned panel using Q16.16 logical-pixel geometry; version 0 requires `flags = 0`.",
    },
    WatAbiImport {
        name: "AE_sin_cos_turn",
        signature: "(param angle_turn i32) (result sin_q30 i32) (result cos_q30 i32)",
        summary: "Returns deterministic Q1.30 sine and cosine for a wrapping binary angle where `2^32` units are one turn.",
    },
    WatAbiImport {
        name: "AE_random_v1_seed_u64",
        signature: "(param state_ptr i32) (param seed_low i32) (param seed_high i32) (result i32)",
        summary: "Initializes one 48-byte guest-owned RandomZ v1 stream from the unsigned 64-bit seed supplied to `AE_init[_i32]`.",
    },
    WatAbiImport {
        name: "AE_random_v1_seed_bytes",
        signature: "(param state_ptr i32) (param seed_ptr i32) (result i32)",
        summary: "Initializes one 48-byte guest-owned RandomZ v1 stream from exactly 32 seed bytes.",
    },
    WatAbiImport {
        name: "AE_random_v1_fill",
        signature: "(param state_ptr i32) (param output_ptr i32) (param output_len i32) (result i32)",
        summary: "Writes deterministic raw stream bytes and advances the serialized stream only after the complete bounded output succeeds.",
    },
    WatAbiImport {
        name: "AE_random_v1_range_i64",
        signature: "(param state_ptr i32) (param start i64) (param end i64) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch of unbiased inclusive-range integers as little-endian `i64` values.",
    },
    WatAbiImport {
        name: "AE_random_v1_uniform",
        signature: "(param state_ptr i32) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch of exact uniform `[0,1)` RandomZ fixed values.",
    },
    WatAbiImport {
        name: "AE_random_v1_normal",
        signature: "(param state_ptr i32) (param mean_m i64) (param mean_e i32) (param stddev_m i64) (param stddev_e i32) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch from the integer-only RandomZ normal distribution.",
    },
    WatAbiImport {
        name: "AE_random_v1_normal_i64",
        signature: "(param state_ptr i32) (param start i64) (param end i64) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch of range-scaled normal integers as little-endian `i64` values.",
    },
    WatAbiImport {
        name: "AE_random_v1_exponential",
        signature: "(param state_ptr i32) (param rate_m i64) (param rate_e i32) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch from the integer-only RandomZ exponential distribution.",
    },
    WatAbiImport {
        name: "AE_random_v1_poisson",
        signature: "(param state_ptr i32) (param lambda_m i64) (param lambda_e i32) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch of RandomZ Poisson variates as little-endian `i64` values.",
    },
    WatAbiImport {
        name: "AE_random_v1_log_normal",
        signature: "(param state_ptr i32) (param mean_m i64) (param mean_e i32) (param stddev_m i64) (param stddev_e i32) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch from the integer-only RandomZ log-normal distribution.",
    },
    WatAbiImport {
        name: "AE_random_v1_beta",
        signature: "(param state_ptr i32) (param alpha_m i64) (param alpha_e i32) (param beta_m i64) (param beta_e i32) (param output_ptr i32) (param count i32) (result i32)",
        summary: "Writes a bounded batch from the integer-only RandomZ beta distribution.",
    },
    WatAbiImport {
        name: "AE_synth_voice",
        signature: "(param program_id i32) (param waveform i32) (param delay_ms i32) (param duration_ms i32) (param frequency_start_millihz i32) (param frequency_mid_millihz i32) (param frequency_end_millihz i32) (param gain_start_ppm i32) (param gain_peak_ppm i32) (param gain_end_ppm i32) (param filter i32) (param filter_start_millihz i32) (param filter_end_millihz i32) (param cooldown_ms i32) (result i32)",
        summary: "Declares a bounded synthesized-audio program during `AE_configure`.",
    },
    WatAbiImport {
        name: "AE_sample_asset",
        signature: "(param id i32) (param path_ptr i32) (param path_len i32) (param flags i32) (result i32)",
        summary: "Binds a unique sampled-audio ID to one bounded FLAC under the application `assets/` virtual root during `AE_configure`; version 0 requires `flags = 0`.",
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
        name: "AE_frame_begin_rgba",
        signature: "(param rgba i32) (result i32)",
        summary: "Begins one render transaction with a packed `0xRRGGBBAA` background for integer-only guests.",
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
        name: "AE_path_move_q16",
        signature: "(param x_q16 i32) (param y_q16 i32) (result i32)",
        summary: "Appends a move segment using signed Q16.16 logical-pixel coordinates.",
    },
    WatAbiImport {
        name: "AE_path_line",
        signature: "(param x f32) (param y f32) (result i32)",
        summary: "Appends a line segment to the active path.",
    },
    WatAbiImport {
        name: "AE_path_line_q16",
        signature: "(param x_q16 i32) (param y_q16 i32) (result i32)",
        summary: "Appends a line segment using signed Q16.16 logical-pixel coordinates.",
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
        name: "AE_path_end_q16",
        signature: "(param width_q16 i32) (param fill_rgba i32) (param stroke_rgba i32) (param flags i32) (result i32)",
        summary: "Commits an integer-profile path with Q16.16 width; a zero packed color disables that paint and version 0 requires `flags = 0`.",
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
        name: "AE_text_font",
        signature: "(param id i32) (param ptr i32) (param len i32) (param x f32) (param y f32) (param size f32) (param rgba i32) (param font i32) (param flags i32) (result i32)",
        summary: "Draws UTF-8 text with an explicit portable face: `font = 0` is the platform default, `1` is bundled Geist Mono Regular, and `flags & 1` centers it.",
    },
    WatAbiImport {
        name: "AE_text_font_q16",
        signature: "(param id i32) (param ptr i32) (param len i32) (param x_q16 i32) (param y_q16 i32) (param size_q16 i32) (param rgba i32) (param font i32) (param flags i32) (result i32)",
        summary: "Integer-profile counterpart to `AE_text_font`; position and size are signed Q16.16 and the stable face selectors are identical.",
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
        name: "AE_sample_play",
        signature: "(param id i32) (param volume f32) (param pitch f32) (param flags i32) (result i32)",
        summary: "Queues one declared immutable sample with volume 0..1 and pitch 0.25..4; playback is one-way and version 0 requires `flags = 0`.",
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

pub const WAT_ABI_PROPOSALS: &[WatAbiProposal] = &[
    WatAbiProposal {
        slug: "semantic-view-tree",
        title: "Semantic view tree and layout",
        status: "Specified direction",
        summary: "General guest-owned composition, layout, scrolling, tabs, grids, editable values, and accessibility semantics beyond the current flat AVP controls.",
        adapters: "Native, Web, headless",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/VIEW_PROTOCOL.md",
    },
    WatAbiProposal {
        slug: "persistent-key-value",
        title: "Persistent key/value storage",
        status: "Policy design",
        summary: "Bounded, rate-limited application state with desktop and browser adapters, transactional writes, quotas, and no ambient filesystem access.",
        adapters: "Native, Web, headless test double",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/PLAN.md",
    },
    WatAbiProposal {
        slug: "file-capabilities",
        title: "User-mediated file capabilities",
        status: "Policy design",
        summary: "Explicit open/save grants for general applications, with browser file-picker parity and no path authority beyond the selected resource.",
        adapters: "Native, Web",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/PLAN.md",
    },
    WatAbiProposal {
        slug: "game-controllers",
        title: "Game-controller input",
        status: "Adapter research",
        summary: "Stable device, button, axis, connection, and disconnection events with deterministic dead-zone and ordering rules.",
        adapters: "Web, macOS, Linux, Windows",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/PLAN.md",
    },
    WatAbiProposal {
        slug: "cli-tui",
        title: "CLI and TUI application profiles",
        status: "Architecture proposal",
        summary: "Non-windowed lifecycle, structured standard streams, terminal capability negotiation, and deterministic headless execution for WAT applications.",
        adapters: "Terminal, headless",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/PLAN.md",
    },
    WatAbiProposal {
        slug: "package-fonts",
        title: "Package-supplied fonts",
        status: "Asset-contract proposal",
        summary: "Bounded font assets with stable family handles and identical native/browser shaping behavior beyond bundled Geist Mono.",
        adapters: "Native, Web, headless",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/PLAN.md",
    },
    WatAbiProposal {
        slug: "appify",
        title: "Appify and deappify",
        status: "Delivery proposal",
        summary: "Combine one pinned Aedicule runtime and one exact .aed package into a normal platform application, then recover both constituents losslessly.",
        adapters: "macOS, Linux, Windows",
        evidence_url: "https://github.com/pmarreck/aedicule/blob/yolo/PLAN.md",
    },
];

/// Produces one neutral guest that imports every declared capability at the
/// registry's exact value types, so both runtime linkers can prove the docs.
pub fn wat_abi_conformance_module() -> String {
    let mut module = String::from("(module\n");
    for import in WAT_ABI_IMPORTS {
        let (parameters, results) = signature_value_types(import.signature);
        let _ = write!(
            module,
            "  (import \"{IMPORT_MODULE}\" \"{}\" (func",
            import.name
        );
        if !parameters.is_empty() {
            let _ = write!(module, " (param {})", parameters.join(" "));
        }
        if !results.is_empty() {
            let _ = write!(module, " (result {})", results.join(" "));
        }
        module.push_str("))\n");
    }
    let _ = write!(
        module,
        "  (memory (export \"memory\") 1)\n\
         \t(func (export \"AE_abi_major\") (result i32) i32.const {ABI_MAJOR})\n\
         \t(func (export \"AE_abi_minor\") (result i32) i32.const {ABI_MINOR})\n\
         \t(func (export \"AE_configure\") (result i32) i32.const 0)\n\
         \t(func (export \"AE_init_i32\") (param i32 i32 i32 i32) (result i32) i32.const 0)\n\
         \t(func (export \"AE_event_i32\") (param i32 i32 i32 i32) (result i32) i32.const 0)\n\
         \t(func (export \"AE_tick\") (param i32) (result i32) i32.const 0)\n\
         \t(func (export \"AE_render\") (result i32) i32.const 0)\n\
         \t(func (export \"AE_state_ptr\") (result i32) i32.const 0)\n\
         \t(func (export \"AE_state_len\") (result i32) i32.const 0)\n\
         \t(func (export \"AE_state_schema\") (result i32) i32.const 1))"
    );
    module
}

fn signature_value_types(signature: &str) -> (Vec<&str>, Vec<&str>) {
    let mut parameters = Vec::new();
    let mut results = Vec::new();
    for clause in signature.split(')') {
        let clause = clause.trim();
        let Some(clause) = clause.strip_prefix('(') else {
            continue;
        };
        let mut words = clause.split_whitespace();
        let Some(kind) = words.next() else {
            continue;
        };
        let value_type = words
            .last()
            .unwrap_or_else(|| panic!("ABI signature clause has no value type: {clause}"));
        assert!(
            matches!(value_type, "i32" | "i64" | "f32" | "f64"),
            "unknown ABI value type in {clause}"
        );
        match kind {
            "param" => parameters.push(value_type),
            "result" => results.push(value_type),
            _ => panic!("unknown ABI signature clause: {clause}"),
        }
    }
    (parameters, results)
}

/// Renders the Web reference from the same current/proposed registries used by
/// checked Markdown and runtime-linker conformance tests.
pub fn abi_reference_html() -> String {
    let mut document = String::from(
        r##"<!doctype html>
<html lang="en" dir="ltr" data-i18n-title="abiReferenceTitle">
	<head>
		<meta charset="utf-8">
		<meta name="viewport" content="width=device-width, initial-scale=1">
		<title>Aedicule ABI reference</title>
		<link rel="icon" href="./icon.png">
		<link rel="manifest" href="./manifest.webmanifest">
		<style>
			:root { color-scheme: dark; font: 16px/1.55 Inter, ui-sans-serif, system-ui, sans-serif; --ink: #f7f9ff; --muted: #aeb8d4; --cyan: #6ee7ff; --violet: #c89aff; --panel: #11182bdc; }
			* { box-sizing: border-box; }
			html { scroll-behavior: smooth; }
			body { min-height: 100vh; margin: 0; background: radial-gradient(circle at 8% 0%, #183d69 0, transparent 34rem), radial-gradient(circle at 92% 18%, #4b1f65 0, transparent 32rem), linear-gradient(155deg, #080b14, #060810 70%); color: var(--ink); }
			body::before { position: fixed; inset: 0; background-image: linear-gradient(#ffffff08 1px, transparent 1px), linear-gradient(90deg, #ffffff08 1px, transparent 1px); background-size: 3rem 3rem; mask-image: linear-gradient(to bottom, #000, transparent 72%); content: ""; pointer-events: none; }
			main { position: relative; width: min(82rem, calc(100% - 2rem)); margin-inline: auto; padding: 1.25rem 0 4rem; }
			nav, .jump { display: flex; align-items: center; gap: 1rem; }
			nav { justify-content: space-between; }
			a { color: var(--cyan); text-underline-offset: .24rem; }
			.brand { display: flex; align-items: center; gap: .65rem; color: var(--ink); font-weight: 850; text-decoration: none; }
			.brand img { width: 2.5rem; height: 2.5rem; }
			.back, .jump a { color: var(--muted); font-weight: 750; }
			.hero { padding: clamp(3.5rem, 8vw, 7rem) 0 2.5rem; }
			.eyebrow, .status, .adapters { font: 800 .72rem/1.3 ui-monospace, SFMono-Regular, Menlo, monospace; letter-spacing: .09em; text-transform: uppercase; }
			.eyebrow { color: var(--cyan); }
			h1 { max-width: 12ch; margin: .8rem 0 1.3rem; background: linear-gradient(100deg, #fff 15%, var(--cyan) 60%, var(--violet)); background-clip: text; color: transparent; font-size: clamp(3rem, 9vw, 7rem); line-height: .87; letter-spacing: -.07em; }
			.lead { max-width: 50rem; color: #d4dcf2; font-size: clamp(1.05rem, 2.2vw, 1.35rem); }
			.jump { margin-top: 1.5rem; flex-wrap: wrap; }
			section { padding-top: 2.5rem; }
			section > header { display: grid; grid-template-columns: minmax(0, 1fr) minmax(18rem, 38rem); gap: 2rem; align-items: end; margin-bottom: 1.25rem; }
			h2 { margin: 0; font-size: clamp(2rem, 5vw, 3.6rem); letter-spacing: -.055em; }
			section > header p { margin: 0; color: var(--muted); }
			.grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: .85rem; }
			.card { min-width: 0; padding: 1.15rem; border: 1px solid #ffffff18; border-radius: 1.1rem; background: linear-gradient(145deg, #151d34e8, var(--panel)); box-shadow: inset 0 1px #ffffff10; }
			.card h3 { margin: .65rem 0 .7rem; font-size: 1rem; overflow-wrap: anywhere; }
			.card p { margin: 0; color: var(--muted); }
			.card code { color: #f5e7ff; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
			.signature { display: block; max-width: 100%; margin: .7rem 0; padding: .72rem; overflow-x: auto; border-radius: .7rem; background: #060913cc; color: #cfefff; font-size: .76rem; }
			.status { color: var(--cyan); }
			.status.proposed { color: #ffca84; }
			.adapters { margin-top: 1rem !important; color: var(--violet) !important; }
			footer { display: flex; flex-wrap: wrap; gap: 1.2rem; padding-top: 3rem; }
			@media (max-width: 62rem) { .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } section > header { grid-template-columns: 1fr; gap: .5rem; } }
			@media (max-width: 42rem) { .grid { grid-template-columns: 1fr; } .jump { align-items: flex-start; flex-direction: column; } }
			@media (prefers-reduced-motion: reduce) { html { scroll-behavior: auto; } *, *::before, *::after { animation: none !important; transition: none !important; } }
		</style>
	</head>
	<body>
		<main>
			<nav>
				<a class="brand" href="./"><img src="./icon.png" alt=""><span>Aedicule</span></a>
				<a class="back" href="./" data-i18n="abiReferenceBack"></a>
			</nav>
			<header class="hero">
				<p class="eyebrow" data-i18n="abiReferenceEyebrow"></p>
				<h1 data-i18n="abiReferenceHeadline"></h1>
				<p class="lead" data-i18n="abiReferenceLead"></p>
				<div class="jump"><a href="#current" data-i18n="abiCurrentTitle"></a><a href="#proposed" data-i18n="abiProposedTitle"></a></div>
			</header>
			<section id="current">
				<header><h2 data-i18n="abiCurrentTitle"></h2><p data-i18n="abiCurrentDescription"></p></header>
				<div class="grid">
"##,
    );
    for import in WAT_ABI_IMPORTS {
        let _ = writeln!(
            document,
            "\t\t\t\t\t<article class=\"card\" data-abi-status=\"callable\"><span class=\"status\" data-i18n=\"abiCallable\"></span><h3><code>{}</code></h3><code class=\"signature\">{}</code><p>{}</p></article>",
            html_escape(import.name),
            html_escape(import.signature),
            inline_code_html(import.summary),
        );
    }
    document.push_str(
        r#"				</div>
			</section>
			<section id="proposed">
				<header><h2 data-i18n="abiProposedTitle"></h2><p data-i18n="abiProposedDescription"></p></header>
				<div class="grid">
"#,
    );
    for proposal in WAT_ABI_PROPOSALS {
        let _ = writeln!(
            document,
            "\t\t\t\t\t<article class=\"card\" id=\"{}\" data-abi-status=\"not-callable\"><span class=\"status proposed\" data-i18n=\"abiNotCallable\"></span><h3>{}</h3><p>{}</p><p class=\"adapters\">{} · {}</p><p><a href=\"{}\" data-i18n=\"abiEvidenceLink\"></a></p></article>",
            html_escape(proposal.slug),
            html_escape(proposal.title),
            html_escape(proposal.summary),
            html_escape(proposal.status),
            html_escape(proposal.adapters),
            html_escape(proposal.evidence_url),
        );
    }
    document.push_str(
        r#"				</div>
			</section>
			<footer><a href="./about.html" data-i18n="aboutLink"></a><a href="https://github.com/pmarreck/aedicule/blob/yolo/WAT_ABI.md" data-i18n="abiMarkdownLink"></a><a href="https://github.com/pmarreck/aedicule/blob/yolo/GUIDE_FOR_LLMS.md" data-i18n="aboutGuideLink"></a></footer>
		</main>
		<script type="module" src="./launcher.mjs"></script>
	</body>
</html>
"#,
    );
    document
}

fn inline_code_html(text: &str) -> String {
    let mut output = String::new();
    for (index, part) in text.split('`').enumerate() {
        if index % 2 == 0 {
            output.push_str(&html_escape(part));
        } else {
            output.push_str("<code>");
            output.push_str(&html_escape(part));
            output.push_str("</code>");
        }
    }
    output
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Renders the exact checked-in WAT client reference from the ABI declarations.
pub fn wat_abi_markdown() -> String {
    let mut document = format!(
        "<!-- Generated by `AE_UPDATE_WAT_ABI=1 cargo run --bin generate-wat-abi`; do not edit by hand. -->\n\n# Aedicule WAT ABI v{ABI_MAJOR}.{ABI_MINOR}\n\nThis is the complete client-facing ABI for WAT applications accepted by Aedicule today. The only import module is `{IMPORT_MODULE}`. Every function at this boundary is named `AE_*`; the provisional `host.v0` / `fp_*` names are rejected.\n\nExcept for the pure multi-result `AE_sin_cos_turn`, imported capability functions return a status: `0` succeeds; any non-zero result rejects the current guest transaction. Strings are UTF-8 byte slices in the guest's exported `memory`. IDs and packed colors travel as `i32` bit patterns. The host validates pointers, finite numeric values, object limits, and frame structure.\n\n## Guest exports\n\n```wat\n(memory (export \"memory\") MIN_PAGES)\n(func (export \"AE_abi_major\") (result i32))\n(func (export \"AE_abi_minor\") (result i32))\n(func (export \"AE_configure\") (result i32))\n(func (export \"AE_init\") (param i32 i32 f32 f32) (result i32)) ;; legacy profile\n(func (export \"AE_event\") (param i32 i32 f32 f32) (result i32)) ;; legacy profile\n(func (export \"AE_tick\") (param i32) (result i32))\n(func (export \"AE_render\") (result i32))\n(func (export \"AE_state_ptr\") (result i32))\n(func (export \"AE_state_len\") (result i32))\n(func (export \"AE_state_schema\") (result i32))\n(func (export \"AE_tick_rate\") (param i32 i32) (result i32 i32)) ;; optional\n(func (export \"AE_after_restore\") (result i32)) ;; optional\n(func (export \"AE_motion_event\") (param i32 i32 i32 i32 i32 i32) (result i32)) ;; optional\n```\n\n`AE_abi_major` must return `{ABI_MAJOR}`. A guest may replace the legacy `AE_init` and `AE_event` pair with the integer-only `AE_init_i32` and `AE_event_i32` pair documented below; mixing the profiles is invalid. `AE_tick_rate(current_numerator, current_denominator)` chooses a reduced or unreduced rational simulation rate in the inclusive 1..=1000 Hz range, with denominator at most 1,000,000. It receives the currently agreed **simulation** rate, never the ambient display rate. Returning `(0, 0)`, or omitting the export, follows the current display mode.\n\n## Lifecycle and timing\n\nA normal startup is `AE_abi_*`, `AE_configure`, the selected profile's init export, one display-refresh event, optional `AE_tick_rate`, then `AE_render`. Reload configures and initializes a new instance, restores a compatible snapshot (and calls `AE_after_restore` if present), delivers the current display-refresh event, selects its rate, and renders before it can replace the active instance.\n\nThe selected event export has `(kind, code, a, b)`. Kind `9` is `AE_EVENT_DISPLAY_REFRESH`: `code = refresh_numerator_hz`, `a = refresh_denominator`, `b = 0`. It is emitted once when the initial display mode becomes known and only again when that mode changes. A guest must treat its absence as “same as the last event”; it cannot query display timing. The host performs the display event before calling `AE_tick_rate`.\n\n`AE_tick(ticks)` receives only bounded whole fixed steps. The host accumulates monotonic time exactly at the agreed rational rate and may report overload by dropping old due ticks while preserving ordered input edges.\n\n## Host display timing override\n\nSet `AE_DISPLAY_REFRESH_RATE` before process startup to override the initial nominal display rate while native display discovery is unavailable. The value is exact and normalized by greatest common divisor: `60000/1001` is a rational rate, `5994/100` becomes `2997/50`, and ordinary decimals are parsed as written without floating point. Canonical nominal spellings `23.976`, `29.97`, `59.94`, and `119.88` select respectively `24000/1001`, `30000/1001`, `60000/1001`, and `120000/1001`. Any other decimal, such as `59.97`, stays exact as typed. Invalid values fail startup; an unset variable retains the native/default source. This is host configuration, not a guest display-query capability.\n\n## Host imports\n\nAll imports use `(import \"{IMPORT_MODULE}\" \"NAME\" (func ...))`.\n\n"
    );

    document.push_str(
        "## ABI compatibility\n\nThe major version must match exactly. Aedicule accepts guest minor versions from `0` through its documented host minor so older guests remain runnable; negative or future minor versions are rejected before configuration. A guest must raise its declared minor when it requires an import or behavior introduced by that revision.\n\n",
    );

    document.push_str(
        "## Input events\n\n`AE_event(kind, code, a, b)` and its integer-profile counterpart `AE_event_i32(kind, code, a, b)` carry host input at an ordered fixed-step boundary. A host may omit an event kind it cannot observe, but it must not invent application semantics. In the integer profile, logical-pixel values use signed Q16.16; IDs, flags, and the display-rate denominator remain ordinary unscaled integers.\n\n| Kind | Event | `code`, `a`, `b` |\n| ---: | --- | --- |\n| 1 | Key down | `code` is a physical-key ID; `a = b = 0` |\n| 2 | Key up | Same key ID; `a = b = 0` |\n| 3 | Pointer move | `code = 0`; `a = x`, `b = y` logical pixels |\n| 4 | Pointer down | `code` is button ID; `a = x`, `b = y` logical pixels |\n| 5 | Pointer up | `code` is button ID; `a = x`, `b = y` logical pixels |\n| 6 | Device change | `code` is a device-class flags bitfield; `a = width`, `b = height` logical pixels |\n| 7 | Action | `code` is the guest-declared action ID; `a = b = 0` |\n| 8 | Focus | `code = 1` when focused, `0` when unfocused; `a = b = 0` |\n| 9 | Display refresh | `code = numerator_hz`; `a = denominator`, `b = 0` |\n| 10 | Pointer scroll | `code` is unit ID; `a = horizontal delta`, `b = vertical delta` |\n| 11 | Touch start | `code` is an opaque contact ID; `a = x`, `b = y` logical pixels |\n| 12 | Touch move | Same contact ID and coordinates |\n| 13 | Touch end | Same contact ID; coordinates are its last admitted position |\n| 14 | Touch cancel | Same contact ID; coordinates are its last admitted position |\n| 15 | Pause lifecycle | `code` is `1` paused, `2` resumed, or `3` restored-paused; `a = b = 0` |\n| 16 | Motion gesture | `code = 1` is a shake; `a` is peak user-acceleration magnitude in m/s² (f32 legacy, signed Q16.16 integer profile); `b = 0` |\n\nRaw touch is configure-time opt-in through `AE_touch_interest`; guests that omit it retain touch-to-primary-pointer compatibility. Contact IDs are opaque and meaningful only from one admitted start through its end or cancel. The guest selects a simultaneous-contact capacity from 1 through the host ceiling of 16. Duplicate starts, moves and terminals for unknown IDs, and starts beyond that capacity are ignored deterministically; an overflowed ID remains suppressed until its terminal edge. A terminal uses the last admitted coordinates rather than untrusted terminal coordinates. Loss of focus or capture cancels admitted contacts in start order. Contacts beginning over a guest-authored AVP control belong to that control and never enter the raw stream. For an opted-in guest, a raw touch's browser-generated primary-pointer compatibility echo is suppressed while real mouse input remains available. Admitted contact edges keep arrival order and are delivered before the following fixed tick.\n\nDevice-class flag bit `0` is set when the primary pointer is coarse (a finger-first touch device), mirroring the CSS `(pointer: coarse)` media query; all other flag bits are reserved, sent as zero, and a guest must mask only the bits it understands. The device-change event is delivered at least once at start and again whenever the viewport dimensions or the device-class flags change, including a flag flip at unchanged size such as an iPad gaining a trackpad. The motion gesture (kind 16) is delivered only to guests that registered `AE_motion_interest` kind `1`; the host derives it from the sensor stream with a fixed threshold and cooldown, so one physical shake is one event. Six-axis samples never ride the ordinary event exports: a guest that registered kind `2` must export `AE_motion_event(ax, ay, az, rx, ry, rz)` and receives acceleration in m/s² and rotation rate in deg/s, all signed Q16.16 in BOTH lifecycle profiles. Where sensors are absent or the platform denies permission, registered interests stay silent rather than erroring - a guest must keep a manual affordance for every motion-triggered action. Physical-key IDs are `1` left, `2` right, `3` up, `4` space, `5` P, `6` R, `7` F, `8` K, `9` B, `10` H, `11` Escape, `12` F1, `13` W, `14` A, and `15` D. Pointer button IDs are `1` primary/left, `2` secondary/right, and `3` middle/wheel-click. Native and browser canvas adapters emit down and up edges for all three IDs.\n\nPointer-scroll unit IDs are `1` lines and `2` logical pixels. Positive `a` means leftward motion; positive `b` means upward motion. Hosts preserve both axes and omit zero-delta scroll events.\n\n",
    );

    document.push_str(
        "## Host-scheduled pause\n\nA guest opts in by calling `AE_pause_trigger(kind, code, flags)` during `AE_configure`. ABI v0.3 accepts selector kind `1` for a stable physical-key ID and requires `flags = 0`. Declarations are bounded and unique. A guest that declares no trigger receives the ordinary lifecycle unchanged.\n\nA fresh declared key-down is consumed by Aedicule; neither that down edge nor its matching up edge reaches gameplay. Repeats and a still-held trigger cannot self-resume. On pause, Aedicule executes work already due at the input barrier and delivers any earlier queued input in arrival order, freezes the fixed-step scheduler and guest audio transport, sends kind `15`, code `1`, accepts exactly one final render, and then performs no ordinary guest ticks or renders. Host window, menu, watcher, and transactional reload machinery remain live. A viewport or display-mode change may send its semantic event and accept at most one replacement paused frame without advancing simulation.\n\nA fresh trigger after release resumes. Releases or focus cancellation for inputs held at pause entry are delivered first, then kind `15`, code `2`, one render, and the next exact rational deadline. New gameplay presses made only while suspended are not armed. A reload candidate admitted while paused receives kind `15`, code `3` and renders its paused presentation before atomic replacement. Failed reloads preserve the old guest and transport.\n\nPaused wall time is removed from the scheduler baseline, so it produces no catch-up ticks and retains the pre-pause fractional phase. Native playback uses a guest-only pausable mixer and shifts synth cooldown timestamps by the same duration; browser sampled audio suspends its shared Web Audio context. Successful reload deterministically cancels old guest sources. A small device buffer may finish after the logical barrier, but its latency never advances guest time. Audio/effects requested by the paused transition itself are discarded in ABI v0.3; the resumed transition may request new output.\n\n",
    );

    document.push_str(
        "## Integer controls and declarative native UI\n\n`AE_slider_i32` declares only a slider's stable ID, accessible label, and exact integer lattice during `AE_configure`; it does **not** create a persistent host-owned widget. A guest that declares any slider must export `(func (export \"AE_control_event\") (param id i32) (param value i32) (param phase i32) (result i32))`. Values are validated against the declared inclusive minimum, maximum, and exact step lattice before delivery. Phase `1` is a continuous change and phase `2` is the committed release edge. Native delivery is ordered through the simulation scheduler. `AE_action` declares a reusable action ID and accessible label without creating any menu item. A non-separator `AE_menu_item` remains the backward-compatible combined declaration of the same action plus an explicit menu presentation. Both imports share one collision-checked action-ID namespace. A button click delivers the ordered kind-`7` action event.\n\nNative UI implements the v0 native-controls profile of the Aedicule View Protocol (AVP). The guest is authoritative for the desired keyed view document and submits a complete snapshot during `AE_render` only when that UI changes: `AE_ui_begin(revision)`, zero or more widget declarations, then `AE_ui_end()`. The revision is an opaque 32-bit bit pattern and must differ from the last accepted revision. An unchanged UI is omitted while ordinary `AE_frame_*` canvas rendering continues. A completed snapshot is published only when the entire surrounding `AE_render` call also returns a valid canvas frame; any rejected import, trap, missing `AE_ui_end`, or incomplete frame preserves the previously accepted snapshot. A snapshot may intentionally be empty, which removes every native widget. This profile is flat and absolute-positioned; the proposed general semantic tree, layout, text-input, accessibility, and adapter-parity contract is specified separately in `VIEW_PROTOCOL.md`.\n\nInside the UI transaction, emit each `AE_control_panel_q16` before any slider, button, or external link that references it. Geometry is absolute viewport-relative signed Q16.16. Panel, slider, button, and external-link stable IDs are independently unique within one snapshot. A slider placement must name a configure-time declaration and carry a value on that declaration's exact lattice. The guest-provided value is authoritative; Aedicule may retain a GPUI entity only for focus, hover, drag, and accessibility mechanics, then reconciles it by stable ID when a new revision is accepted. Omitting an ID from the next snapshot removes it. Slider label placement `0` hides the visual label/value and `1` places it above a full-width track; slider and panel flags remain zero. A button placement must reference a declared action. Button flag bit `0` selects the platform-native highlighted state; every other bit is reserved and must be zero.\n\nAfter accepting a control or action event, a guest that wants the changed value or selected state displayed must update its model and submit a new UI revision. If it does not, Aedicule reconciles transient slider interaction back to the last accepted guest value and retains the last accepted button state. Headless `aedicule-render --control ID=VALUE` arguments are repeatable, preserve argument order, use phase `2`, and execute after initialization but before requested ticks and rendering. `--activate-action ID` is also repeatable and delivers kind `7` only when a button for that action exists in the current accepted UI snapshot.\n\n## External links\n\nABI v0.4 adds a bounded, guest-owned external-navigation control rather than ambient browser or network access. During `AE_configure`, `AE_external_link` declares a stable ID, nonempty visible/accessibility label, and absolute HTTPS URL. The host rejects non-HTTPS schemes, whitespace or controls, backslashes, missing hosts, and URL credentials; `flags` must be zero. During a changed UI snapshot, `AE_external_link_place_q16` places that ID inside an already-declared panel. Its placement ID is the declaration ID, may occur only once per snapshot, and uses the same absolute Q16.16 geometry as other controls.\n\nNative and browser adapters resolve activation against the exact currently accepted `(revision, id)` only from the platform control's real click handler. Native delegates to the platform URL service. Browser activation requires transient user activation and opens a new browsing context with `_blank` and `noopener`; popup-policy failure is an adapter diagnostic, not guest-visible state. The guest receives no navigation result and cannot synthesize activation through an import. Headless `aedicule-render --activate-link ID` never opens a browser; it validates the current placement and emits the deterministic revision, ID, and normalized URL to stderr for automation.\n\n## Text entry\n\nABI v0.5 adds the one control backed by a real editable element. During `AE_configure`, `AE_text_field` declares a stable ID, a nonempty accessible label, and an exact capacity measured in **Unicode scalar values** — not bytes and not UTF-16 code units — of at most 4096; `flags` must be zero. A guest that declares any text field must export `(func (export \"AE_text_event\") (param id i32) (param index i32) (param scalar i32) (param phase i32) (result i32))`, and configure is rejected without it. During a changed UI snapshot, `AE_text_field_place_q16` places that ID inside an already-declared panel using the same absolute Q16.16 geometry as every other control.\n\nThe guest never receives a byte buffer and the host never writes into guest memory. One complete value arrives as an ordered run of integer events: `index` is the zero-based scalar position, `scalar` is the Unicode scalar value at that position, and `phase` is `0`. A terminating call with `index = -1` carries the authoritative scalar count in `scalar` and the edit phase in `phase`: `1` for a continuous change and `2` for the committed edge. A value of length zero sends only that terminator, which is how a cleared field is expressed. The host rejects any index at or beyond the declared capacity, any count above it, and any code point that is not a Unicode scalar value — surrogates `D800`-`DFFF` and anything above `10FFFF`. Treat the terminator as the transaction boundary and swap the guest-side buffer there rather than acting on a partial run.\n\nFocusing a placed text field is the only thing in Aedicule that may raise a mobile software keyboard: the platform text widget takes focus, GPUI installs its input handler, and the browser backend then moves DOM focus to its editable element. Ordinary canvas interaction leaves no editable element focused, so a touch on non-text content presents no keyboard.\n\nIn v0.5 the platform widget owns its edit buffer. A placement carries geometry only, so a guest cannot set, clear, or restore the displayed text, and a reload discards it; a guest that needs to own the value must wait for a later revision of this profile. Headless `aedicule-render --text ID=VALUE` is repeatable, splits at the first `=` so a value may itself contain `=` or be empty, uses phase `2`, and executes after initialization but before requested ticks and rendering.\n\n",
    );

    document.push_str(
        "## Deterministic RandomZ v1\n\nABI v0.9 exposes the RandomZ v1 deterministic byte stream and its integer-only nonlinear distributions. The `v1` import names freeze the algorithm and call-consumption behavior: a future incompatible generator must use new import names rather than changing an existing sequence. There is no ambient or system-entropy import in this profile.\n\nEvery stream occupies 48 guest-owned bytes. Bytes `0..4` are `AER\\x01`; bytes `4..8` are zero; bytes `8..40` are the derived 256-bit stream key; bytes `40..48` are the unsigned byte position in little-endian order. Initialize the region once with `AE_random_v1_seed_u64` or `AE_random_v1_seed_bytes`, keep distinct streams in non-overlapping regions, and include every live region inside `AE_state_ptr` / `AE_state_len` if it must survive transactional reload. `seed_u64` joins the two `AE_init*` seed halves as one unsigned value and places its big-endian encoding in the final eight bytes of a zero-filled 32-byte seed. `seed_bytes` consumes exactly 32 bytes as supplied.\n\nRandomZ fixed values are canonical `(mantissa i64, exponent i32)` pairs representing `(mantissa / 2^62) * 2^exponent`. Zero is exactly `(0, 0)`; a nonzero mantissa has magnitude from `2^62` through `2^63 - 1`. Fixed batch output uses a 12-byte little-endian record: mantissa first, exponent second. Integer batch output is packed little-endian `i64`. `range_i64` is inclusive and requires an ordered range whose cardinality is at most `2^53`; normal standard deviation, exponential rate, Poisson lambda, and both beta parameters must be positive canonical fixed values. Other distribution-specific numeric-domain limits fail closed.\n\n`count = 0` and zero-length raw fills are valid no-ops. A normal host admits at most 65,536 output bytes and 65,536 source bytes per import; callers should split larger work and check every status. State and output regions may not overlap. The host computes into private buffers and commits output plus the advanced state only after the full batch succeeds. Invalid pointers, state headers, parameters, output budgets, source-consumption budgets, or position overflow leave both regions unchanged.\n\n",
    );

    for import in WAT_ABI_IMPORTS {
        let _ = writeln!(
            document,
            "### `{}`\n\n```wat\n(func ${} {})\n```\n\n{}\n",
            import.name, import.name, import.signature, import.summary
        );
    }

    let _ = write!(
        document,
        "## Portable text faces\n\n`AE_text` remains source-compatible and uses the active platform UI face. `AE_text_font` and `AE_text_font_q16` accept stable selector `0` for that platform default or `1` for Aedicule's embedded Geist Mono Regular. Selector `1` is registered from identical OFL-1.1 font bytes in native and browser adapters, so aligned numerical data never depends on host installation. Unknown selectors reject the complete render transaction rather than silently substituting a proportional face. Package-supplied font handles are not part of ABI v{ABI_MAJOR}.{ABI_MINOR}; they require bounded `.aed` asset transport and collision-safe family identities in every adapter.\n\n## Stable draw IDs\n\nEvery non-composition draw ID is unique **across all primitive kinds within one frame**: a path, line, circle, text, or sprite cannot reuse another primitive's ID. The set resets at the next successful frame begin, so the same semantic object should normally reuse its ID in later frames.\n\n",
    );

    document.push_str(
        "## Frame rules\n\nCall exactly one of `AE_frame_begin` or `AE_frame_begin_rgba` once per `AE_render`, optionally submit a changed retained AVP native-controls document, produce drawing commands, then call `AE_frame_end`. Transform pushes/pops and paths must balance. Any invalid, partial, or over-budget drawing/UI frame is rejected atomically; a host adapter never paints a partial frame.\n\n## Integer-only lifecycle profile\n\nA guest may replace both floating-point lifecycle exports with `(func (export \"AE_init_i32\") (param seed_lo i32) (param seed_hi i32) (param width_q16 i32) (param height_q16 i32) (result i32))` and `(func (export \"AE_event_i32\") (param kind i32) (param code i32) (param a i32) (param b i32) (result i32))`. The pair is atomic: exporting only one is invalid. Viewport and coordinate values use signed Q16.16. This profile lets a guest omit every WebAssembly `f32`/`f64` type and opcode.\n\n## Minimal integer-only shape\n\n```wat\n(module\n  (import \"aedicule.v0\" \"AE_frame_begin_rgba\" (func $begin (param i32) (result i32)))\n  (import \"aedicule.v0\" \"AE_frame_end\" (func $end (result i32)))\n  (memory (export \"memory\") 1)\n  (func (export \"AE_abi_major\") (result i32) i32.const 0)\n  (func (export \"AE_abi_minor\") (result i32) i32.const 1)\n  (func (export \"AE_configure\") (result i32) i32.const 0)\n  (func (export \"AE_init_i32\") (param i32 i32 i32 i32) (result i32) i32.const 0)\n  (func (export \"AE_event_i32\") (param i32 i32 i32 i32) (result i32) i32.const 0)\n  (func (export \"AE_tick\") (param i32) (result i32) i32.const 0)\n  (func (export \"AE_render\") (result i32)\n    i32.const 255 call $begin drop\n    call $end)\n  (func (export \"AE_state_ptr\") (result i32) i32.const 0)\n  (func (export \"AE_state_len\") (result i32) i32.const 0)\n  (func (export \"AE_state_schema\") (result i32) i32.const 1))\n```\n"
    );
    document
}

/// Combines authored teaching material with the exact generated ABI reference,
/// keeping a source-independent LLM guide readable without duplicating symbols.
pub fn guide_for_llms_markdown() -> String {
    let template = include_str!("../GUIDE_FOR_LLMS.template.md");
    assert_eq!(
        template.matches(LLM_GUIDE_ABI_MARKER).count(),
        1,
        "LLM guide template must contain exactly one generated ABI marker"
    );
    let abi_reference = demote_markdown_headings(&strip_generated_comment(&wat_abi_markdown()));
    let abi_reference = abi_reference.trim_end();
    template
        .replace("{{GUIDE_VERSION}}", LLM_GUIDE_VERSION)
        .replace("{{ABI_VERSION}}", &format!("{ABI_MAJOR}.{ABI_MINOR}"))
        .replace("{{CANONICAL_URL}}", LLM_GUIDE_CANONICAL_URL)
        .replace(LLM_GUIDE_ABI_MARKER, abi_reference)
}

fn strip_generated_comment(markdown: &str) -> String {
    markdown
        .strip_prefix("<!--")
        .and_then(|rest| rest.split_once("-->\n\n").map(|(_, body)| body.to_owned()))
        .unwrap_or_else(|| markdown.to_owned())
}

fn demote_markdown_headings(markdown: &str) -> String {
    let mut output = String::with_capacity(markdown.len() + 2048);
    for line in markdown.lines() {
        if line.starts_with('#') {
            output.push_str("##");
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}
