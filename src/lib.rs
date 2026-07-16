//! Capability-bounded runtime core for WAT-authored GPUI applications.
//!
//! This crate intentionally keeps WebAssembly execution and immutable frame
//! output independent of GPUI. Native and future web frontplanes are adapters
//! over the same lifecycle, validation, snapshot, and command-buffer logic.

use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use thiserror::Error;
use wasmtime::{
    Caller, Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder, TypedFunc,
};

pub const ABI_MAJOR: i32 = 0;
pub const ABI_MINOR: i32 = 0;
pub const DEFAULT_PLUGIN_FILE: &str = "code.wat";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSource {
    Embedded,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchAction {
    Run { source: PluginSource, watch: bool },
    Help,
    About,
}

/// Resolves the native runner's order-sensitive source switches while treating
/// an application directory as a container for the conventional `code.wat`.
pub fn resolve_launch(
    arguments: impl IntoIterator<Item = OsString>,
    working_directory: &Path,
) -> Result<LaunchAction, String> {
    let mut source = None;
    let mut watch = false;
    let mut positional_count = 0;
    let mut positional_only = false;

    for argument in arguments {
        let recognized = argument.to_str();
        if !positional_only {
            match recognized {
                Some("-h" | "--help") => return Ok(LaunchAction::Help),
                Some("--about") => return Ok(LaunchAction::About),
                Some("--watch") => {
                    watch = true;
                    if source == Some(PluginSource::Embedded) {
                        source = None;
                    }
                    continue;
                }
                Some("--embedded") => {
                    source = Some(PluginSource::Embedded);
                    watch = false;
                    continue;
                }
                Some("--") => {
                    positional_only = true;
                    continue;
                }
                Some(option) if option.starts_with('-') && option != "-" => {
                    return Err(format!("unknown option: {option}"));
                }
                _ => {}
            }
        }

        positional_count += 1;
        if positional_count > 1 {
            return Err("only one WAT file or application directory may be specified".into());
        }
        let mut path = PathBuf::from(argument);
        if path.is_relative() {
            path = working_directory.join(path);
        }
        if path.is_dir() {
            path.push(DEFAULT_PLUGIN_FILE);
        }
        source = Some(PluginSource::File(path));
    }

    let source = source.unwrap_or_else(|| {
        let default = working_directory.join(DEFAULT_PLUGIN_FILE);
        if watch || default.is_file() {
            PluginSource::File(default)
        } else {
            PluginSource::Embedded
        }
    });
    Ok(LaunchAction::Run { source, watch })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRevision {
    Missing,
    Unreadable(String),
    Content(Vec<u8>),
}

impl FileRevision {
    pub fn read(path: &Path) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => Self::Content(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::Missing,
            Err(error) => Self::Unreadable(error.to_string()),
        }
    }
}

#[derive(Debug, Default)]
pub struct RevisionTracker {
    previous: Option<FileRevision>,
}

impl RevisionTracker {
    /// Classifies directory-safe file observations by content, suppressing
    /// duplicate polls and duplicate editor events without relying on time.
    pub fn observe(&mut self, revision: &FileRevision) -> bool {
        if self.previous.as_ref() == Some(revision) {
            false
        } else {
            self.previous = Some(revision.clone());
            true
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Limits {
    pub fuel_per_call: u64,
    pub max_memory_bytes: usize,
    pub max_table_elements: usize,
    pub max_coordinate_abs: f32,
    pub max_commands: usize,
    pub max_string_bytes: usize,
    pub max_menu_items: usize,
    pub max_audio_events: usize,
    pub max_effects: usize,
    pub max_image_bytes: usize,
    pub max_total_image_bytes: usize,
    pub max_path_segments: usize,
    pub max_snapshot_bytes: usize,
    pub max_ticks_per_call: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            fuel_per_call: 2_000_000,
            max_memory_bytes: 4 * 1024 * 1024,
            max_table_elements: 4_096,
            max_coordinate_abs: 1_000_000.0,
            max_commands: 4_096,
            max_string_bytes: 4_096,
            max_menu_items: 128,
            max_audio_events: 256,
            max_effects: 256,
            max_image_bytes: 4 * 1024 * 1024,
            max_total_image_bytes: 16 * 1024 * 1024,
            max_path_segments: 8_192,
            max_snapshot_bytes: 1024 * 1024,
            max_ticks_per_call: 240,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub title: String,
    pub menu_items: Vec<MenuItem>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: "WAT Application".into(),
            menu_items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub id: Option<u32>,
    pub label: String,
    pub shortcut: Option<String>,
    pub separator: bool,
}

impl MenuItem {
    pub fn action(id: u32, label: impl Into<String>, shortcut: Option<&str>) -> Self {
        Self {
            id: Some(id),
            label: label.into(),
            shortcut: shortcut.map(str::to_owned),
            separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: None,
            label: String::new(),
            shortcut: None,
            separator: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameOutput {
    pub background: u32,
    pub commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub tx: f32,
    pub ty: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    Move(Point),
    Line(Point),
    Quadratic {
        control: Point,
        end: Point,
    },
    Cubic {
        control_1: Point,
        control_2: Point,
        end: Point,
    },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Encoded,
    RawRgba,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageResource {
    pub id: u32,
    pub format: ImageFormat,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    PushTransform(Affine),
    PopTransform,
    Line {
        id: u32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        rgba: u32,
    },
    Circle {
        id: u32,
        x: f32,
        y: f32,
        radius: f32,
        width: f32,
        rgba: u32,
        filled: bool,
    },
    Text {
        id: u32,
        text: String,
        x: f32,
        y: f32,
        size: f32,
        rgba: u32,
        centered: bool,
    },
    Path {
        id: u32,
        segments: Vec<PathSegment>,
        width: f32,
        fill_rgba: Option<u32>,
        stroke_rgba: Option<u32>,
    },
    Sprite {
        id: u32,
        image_id: u32,
        source: Rect,
        destination: Rect,
        pivot: Point,
        tint_rgba: u32,
        flags: u32,
    },
}

/// Serializes an immutable scene-command frame as deterministic SVG so visual
/// output can be inspected in headless CI and by agents without a desktop.
pub fn render_svg(frame: &FrameOutput, logical_width: f32, logical_height: f32) -> String {
    let mut svg = String::new();
    let width = svg_number(logical_width);
    let height = svg_number(logical_height);
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    )
    .unwrap();
    let (background, background_opacity) = svg_color(frame.background);
    write!(
        svg,
        "<rect width=\"{width}\" height=\"{height}\" fill=\"{background}\""
    )
    .unwrap();
    write_opacity(&mut svg, "fill", background_opacity);
    svg.push_str("/>\n");

    for command in &frame.commands {
        match command {
            DrawCommand::PushTransform(matrix) => {
                writeln!(
                    svg,
                    "<g transform=\"matrix({} {} {} {} {} {})\">",
                    svg_number(matrix.m11),
                    svg_number(matrix.m12),
                    svg_number(matrix.m21),
                    svg_number(matrix.m22),
                    svg_number(matrix.tx),
                    svg_number(matrix.ty),
                )
                .unwrap();
            }
            DrawCommand::PopTransform => svg.push_str("</g>\n"),
            DrawCommand::Line {
                id,
                x1,
                y1,
                x2,
                y2,
                width,
                rgba,
            } => {
                let (color, opacity) = svg_color(*rgba);
                write!(
                    svg,
                    "<line data-command-id=\"{id}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{}\"",
                    svg_number(*x1),
                    svg_number(*y1),
                    svg_number(*x2),
                    svg_number(*y2),
                    svg_number(*width),
                )
                .unwrap();
                write_opacity(&mut svg, "stroke", opacity);
                svg.push_str(" stroke-linecap=\"round\"/>\n");
            }
            DrawCommand::Circle {
                id,
                x,
                y,
                radius,
                width,
                rgba,
                filled,
            } => {
                let (color, opacity) = svg_color(*rgba);
                write!(
                    svg,
                    "<circle data-command-id=\"{id}\" cx=\"{}\" cy=\"{}\" r=\"{}\"",
                    svg_number(*x),
                    svg_number(*y),
                    svg_number(*radius),
                )
                .unwrap();
                if *filled {
                    write!(svg, " fill=\"{color}\"").unwrap();
                    write_opacity(&mut svg, "fill", opacity);
                } else {
                    write!(
                        svg,
                        " fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\"",
                        svg_number(*width),
                    )
                    .unwrap();
                    write_opacity(&mut svg, "stroke", opacity);
                }
                svg.push_str("/>\n");
            }
            DrawCommand::Text {
                id,
                text,
                x,
                y,
                size,
                rgba,
                centered,
            } => {
                let (color, opacity) = svg_color(*rgba);
                write!(
                    svg,
                    "<text data-command-id=\"{id}\" x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"{}\" fill=\"{color}\"",
                    svg_number(*x),
                    svg_number(*y),
                    svg_number(*size),
                )
                .unwrap();
                write_opacity(&mut svg, "fill", opacity);
                if *centered {
                    svg.push_str(" text-anchor=\"middle\"");
                }
                svg.push('>');
                write_xml_text(&mut svg, text);
                svg.push_str("</text>\n");
            }
            DrawCommand::Path {
                id,
                segments,
                width,
                fill_rgba,
                stroke_rgba,
            } => {
                write!(svg, "<path data-command-id=\"{id}\" d=\"").unwrap();
                for segment in segments {
                    match segment {
                        PathSegment::Move(point) => {
                            write!(svg, "M{} {}", svg_number(point.x), svg_number(point.y))
                                .unwrap();
                        }
                        PathSegment::Line(point) => {
                            write!(svg, "L{} {}", svg_number(point.x), svg_number(point.y))
                                .unwrap();
                        }
                        PathSegment::Quadratic { control, end } => {
                            write!(
                                svg,
                                "Q{} {} {} {}",
                                svg_number(control.x),
                                svg_number(control.y),
                                svg_number(end.x),
                                svg_number(end.y),
                            )
                            .unwrap();
                        }
                        PathSegment::Cubic {
                            control_1,
                            control_2,
                            end,
                        } => {
                            write!(
                                svg,
                                "C{} {} {} {} {} {}",
                                svg_number(control_1.x),
                                svg_number(control_1.y),
                                svg_number(control_2.x),
                                svg_number(control_2.y),
                                svg_number(end.x),
                                svg_number(end.y),
                            )
                            .unwrap();
                        }
                        PathSegment::Close => svg.push('Z'),
                    }
                }
                svg.push('"');
                if let Some(rgba) = fill_rgba {
                    let (color, opacity) = svg_color(*rgba);
                    write!(svg, " fill=\"{color}\"").unwrap();
                    write_opacity(&mut svg, "fill", opacity);
                } else {
                    svg.push_str(" fill=\"none\"");
                }
                if let Some(rgba) = stroke_rgba {
                    let (color, opacity) = svg_color(*rgba);
                    write!(
                        svg,
                        " stroke=\"{color}\" stroke-width=\"{}\"",
                        svg_number(*width),
                    )
                    .unwrap();
                    write_opacity(&mut svg, "stroke", opacity);
                }
                svg.push_str("/>\n");
            }
            DrawCommand::Sprite {
                id,
                image_id,
                source,
                destination,
                pivot,
                tint_rgba,
                flags,
            } => {
                let (color, opacity) = svg_color(*tint_rgba);
                write!(
                    svg,
                    "<rect data-command-id=\"{id}\" data-image-id=\"{image_id}\" data-source=\"{} {} {} {}\" data-pivot=\"{} {}\" data-flags=\"{flags}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{color}\"",
                    svg_number(source.x),
                    svg_number(source.y),
                    svg_number(source.width),
                    svg_number(source.height),
                    svg_number(pivot.x),
                    svg_number(pivot.y),
                    svg_number(destination.x),
                    svg_number(destination.y),
                    svg_number(destination.width),
                    svg_number(destination.height),
                )
                .unwrap();
                write_opacity(&mut svg, "stroke", opacity);
                svg.push_str(" stroke-dasharray=\"4 3\"/>\n");
            }
        }
    }
    svg.push_str("</svg>\n");
    svg
}

fn svg_number(number: f32) -> String {
    if number == 0.0 {
        "0".to_owned()
    } else {
        number.to_string()
    }
}

fn svg_color(rgba: u32) -> (String, u8) {
    (
        format!(
            "#{:02x}{:02x}{:02x}",
            rgba >> 24,
            (rgba >> 16) & 0xff,
            (rgba >> 8) & 0xff,
        ),
        (rgba & 0xff) as u8,
    )
}

fn write_opacity(svg: &mut String, attribute: &str, alpha: u8) {
    if alpha != u8::MAX {
        write!(
            svg,
            " {attribute}-opacity=\"{}\"",
            svg_number(f32::from(alpha) / 255.0),
        )
        .unwrap();
    }
}

fn write_xml_text(svg: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => svg.push_str("&amp;"),
            '<' => svg.push_str("&lt;"),
            '>' => svg.push_str("&gt;"),
            '"' => svg.push_str("&quot;"),
            '\'' => svg.push_str("&apos;"),
            _ => svg.push(character),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioEvent {
    pub id: u32,
    pub volume: f32,
    pub pitch: f32,
    pub flags: u32,
}

impl AudioEvent {
    pub fn fire() -> Self {
        Self {
            id: 1,
            volume: 1.0,
            pitch: 1.0,
            flags: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostEffect {
    Redraw,
    Quit,
    SetCursor(i32),
    PersistSnapshot,
    OpenUrl,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    PointerMove { x: f32, y: f32 },
    PointerDown { button: u32, x: f32, y: f32 },
    PointerUp { button: u32, x: f32, y: f32 },
    Viewport { width: f32, height: f32 },
    MenuAction(u32),
    Focus(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Key {
    Left = 1,
    Right = 2,
    Thrust = 3,
    Fire = 4,
    Pause = 5,
    Restart = 6,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub schema: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluginInit {
    pub seed: u64,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl PluginInit {
    pub const fn new(seed: u64, viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            seed,
            viewport_width,
            viewport_height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransfer {
    Preserved,
    Restarted,
}

#[derive(Debug)]
pub struct PreparedReload {
    pub frontplane: Frontplane,
    pub frame: FrameOutput,
    pub state_transfer: StateTransfer,
}

#[derive(Debug, Error)]
pub enum FrontplaneError {
    #[error("could not parse WAT: {0}")]
    Wat(String),
    #[error("could not compile or instantiate WebAssembly: {0}")]
    Runtime(String),
    #[error("unsupported import {module}.{name}")]
    UnsupportedImport { module: String, name: String },
    #[error("required export {name} is missing or has the wrong type")]
    MissingExport { name: &'static str },
    #[error("ABI major mismatch: host {expected}, plugin {found}")]
    WrongAbi { expected: i32, found: i32 },
    #[error("plugin exhausted its fuel during {operation}")]
    FuelExhausted { operation: &'static str },
    #[error("plugin trapped during {operation}: {message}")]
    Trap {
        operation: &'static str,
        message: String,
    },
    #[error("plugin returned status {status} from {operation}")]
    PluginStatus {
        operation: &'static str,
        status: i32,
    },
    #[error("invalid pointer or length in {operation}")]
    InvalidPointer { operation: &'static str },
    #[error("invalid UTF-8 in {operation}")]
    InvalidUtf8 { operation: &'static str },
    #[error("invalid or non-finite number in {operation}")]
    InvalidNumber { operation: &'static str },
    #[error("{kind} budget exhausted in {operation}")]
    BudgetExhausted {
        operation: &'static str,
        kind: &'static str,
    },
    #[error("invalid frame lifecycle: {0}")]
    InvalidFrame(&'static str),
    #[error("duplicate stable ID {id} in {operation}")]
    DuplicateId { operation: &'static str, id: u32 },
    #[error("unsupported or unavailable capability in {operation}")]
    UnsupportedCapability { operation: &'static str },
    #[error("snapshot schema or size does not match the plugin")]
    SnapshotMismatch,
}

#[derive(Debug, Clone)]
enum PendingError {
    InvalidPointer(&'static str),
    InvalidUtf8(&'static str),
    InvalidNumber(&'static str),
    Budget(&'static str, &'static str),
    InvalidFrame(&'static str),
    DuplicateId(&'static str, u32),
    Unsupported(&'static str),
}

impl PendingError {
    fn into_public(self) -> FrontplaneError {
        match self {
            Self::InvalidPointer(operation) => FrontplaneError::InvalidPointer { operation },
            Self::InvalidUtf8(operation) => FrontplaneError::InvalidUtf8 { operation },
            Self::InvalidNumber(operation) => FrontplaneError::InvalidNumber { operation },
            Self::Budget(operation, kind) => FrontplaneError::BudgetExhausted { operation, kind },
            Self::InvalidFrame(message) => FrontplaneError::InvalidFrame(message),
            Self::DuplicateId(operation, id) => FrontplaneError::DuplicateId { operation, id },
            Self::Unsupported(operation) => FrontplaneError::UnsupportedCapability { operation },
        }
    }
}

struct FrameBuilder {
    background: u32,
    commands: Vec<DrawCommand>,
    ids: HashSet<u32>,
}

struct PathBuilder {
    id: u32,
    segments: Vec<PathSegment>,
}

struct HostState {
    limits: Limits,
    store_limits: StoreLimits,
    metadata: Metadata,
    menu_ids: HashSet<u32>,
    images: Vec<ImageResource>,
    image_ids: HashSet<u32>,
    frame: Option<FrameBuilder>,
    transform_depth: usize,
    current_path: Option<PathBuilder>,
    completed_frame: Option<FrameOutput>,
    audio: Vec<AudioEvent>,
    effects: Vec<HostEffect>,
    audio_checkpoint: usize,
    effect_checkpoint: usize,
    pending_error: Option<PendingError>,
}

impl HostState {
    fn reject(&mut self, error: PendingError, status: i32) -> i32 {
        if self.pending_error.is_none() {
            self.pending_error = Some(error);
        }
        status
    }
}

struct Exports {
    abi_major: TypedFunc<(), i32>,
    abi_minor: TypedFunc<(), i32>,
    configure: TypedFunc<(), i32>,
    init: TypedFunc<(i32, i32, f32, f32), i32>,
    event: TypedFunc<(i32, i32, f32, f32), i32>,
    tick: TypedFunc<i32, i32>,
    render: TypedFunc<(), i32>,
    state_ptr: TypedFunc<(), i32>,
    state_len: TypedFunc<(), i32>,
    state_schema: TypedFunc<(), i32>,
    after_restore: Option<TypedFunc<(), i32>>,
}

/// Owns one untrusted plugin instance and exposes only validated plain data.
pub struct Frontplane {
    store: Store<HostState>,
    _memory: Memory,
    _instance: Instance,
    exports: Exports,
}

impl fmt::Debug for Frontplane {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Frontplane")
            .field("metadata", &self.store.data().metadata)
            .finish_non_exhaustive()
    }
}

impl Frontplane {
    /// Compiles WAT, rejects ambient capabilities, binds the bounded host ABI,
    /// and validates all lifecycle export signatures before returning.
    pub fn from_wat(source: &str, limits: Limits) -> Result<Self, FrontplaneError> {
        let bytes =
            wat::parse_str(source).map_err(|error| FrontplaneError::Wat(error.to_string()))?;
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).map_err(runtime_error)?;
        let module = Module::new(&engine, bytes).map_err(runtime_error)?;

        for import in module.imports() {
            if import.module() != "host.v0" || !SUPPORTED_IMPORTS.contains(&import.name()) {
                return Err(FrontplaneError::UnsupportedImport {
                    module: import.module().to_owned(),
                    name: import.name().to_owned(),
                });
            }
        }

        let mut linker = Linker::new(&engine);
        bind_host_functions(&mut linker)?;
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes)
            .table_elements(limits.max_table_elements)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();
        let state = HostState {
            limits: limits.clone(),
            store_limits,
            metadata: Metadata::default(),
            menu_ids: HashSet::new(),
            images: Vec::new(),
            image_ids: HashSet::new(),
            frame: None,
            transform_depth: 0,
            current_path: None,
            completed_frame: None,
            audio: Vec::new(),
            effects: Vec::new(),
            audio_checkpoint: 0,
            effect_checkpoint: 0,
            pending_error: None,
        };
        let mut store = Store::new(&engine, state);
        store.limiter(|state| &mut state.store_limits);
        store
            .set_fuel(limits.fuel_per_call)
            .map_err(runtime_error)?;
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(runtime_error)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(FrontplaneError::MissingExport { name: "memory" })?;
        let exports = Exports {
            abi_major: required_func(&instance, &mut store, "fp_abi_major")?,
            abi_minor: required_func(&instance, &mut store, "fp_abi_minor")?,
            configure: required_func(&instance, &mut store, "fp_configure")?,
            init: required_func(&instance, &mut store, "fp_init")?,
            event: required_func(&instance, &mut store, "fp_event")?,
            tick: required_func(&instance, &mut store, "fp_tick")?,
            render: required_func(&instance, &mut store, "fp_render")?,
            state_ptr: required_func(&instance, &mut store, "fp_state_ptr")?,
            state_len: required_func(&instance, &mut store, "fp_state_len")?,
            state_schema: required_func(&instance, &mut store, "fp_state_schema")?,
            after_restore: instance
                .get_typed_func::<(), i32>(&mut store, "fp_after_restore")
                .ok(),
        };
        let mut frontplane = Self {
            store,
            _memory: memory,
            _instance: instance,
            exports,
        };
        let abi_major_function = frontplane.exports.abi_major.clone();
        let abi_major = frontplane.call_value("fp_abi_major", abi_major_function)?;
        if abi_major != ABI_MAJOR {
            return Err(FrontplaneError::WrongAbi {
                expected: ABI_MAJOR,
                found: abi_major,
            });
        }
        let abi_minor_function = frontplane.exports.abi_minor.clone();
        let _ = frontplane.call_value("fp_abi_minor", abi_minor_function)?;
        Ok(frontplane)
    }

    /// Runs the plugin's declarative metadata phase transactionally, discarding
    /// partial menus and resources if any host import or export fails.
    pub fn configure(&mut self) -> Result<(), FrontplaneError> {
        self.store.data_mut().metadata = Metadata::default();
        self.store.data_mut().menu_ids.clear();
        self.store.data_mut().images.clear();
        self.store.data_mut().image_ids.clear();
        let function = self.exports.configure.clone();
        let result = self.call_status("fp_configure", function, ());
        if result.is_err() {
            let state = self.store.data_mut();
            state.metadata = Metadata::default();
            state.menu_ids.clear();
            state.images.clear();
            state.image_ids.clear();
        }
        result
    }

    /// Injects the only entropy and initial viewport available to the plugin;
    /// no ambient clock, RNG, filesystem, or process state is exposed.
    pub fn init(
        &mut self,
        seed: u64,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Result<(), FrontplaneError> {
        validate_bounded_numbers(
            "fp_init",
            &[viewport_width, viewport_height],
            self.store.data().limits.max_coordinate_abs,
        )?;
        if viewport_width <= 0.0 || viewport_height <= 0.0 {
            return Err(FrontplaneError::InvalidNumber {
                operation: "fp_init",
            });
        }
        let function = self.exports.init.clone();
        self.call_status(
            "fp_init",
            function,
            (
                seed as i32,
                (seed >> 32) as i32,
                viewport_width,
                viewport_height,
            ),
        )
    }

    pub fn metadata(&self) -> &Metadata {
        &self.store.data().metadata
    }

    pub fn images(&self) -> &[ImageResource] {
        &self.store.data().images
    }

    /// Converts portable typed input into the stable numeric ABI, keeping GPUI
    /// and platform key representations outside the guest boundary.
    pub fn event(&mut self, event: Event) -> Result<(), FrontplaneError> {
        let (kind, code, a, b) = match event {
            Event::KeyDown(key) => (1, key as i32, 0.0, 0.0),
            Event::KeyUp(key) => (2, key as i32, 0.0, 0.0),
            Event::PointerMove { x, y } => (3, 0, x, y),
            Event::PointerDown { button, x, y } => (4, button as i32, x, y),
            Event::PointerUp { button, x, y } => (5, button as i32, x, y),
            Event::Viewport { width, height } => (6, 0, width, height),
            Event::MenuAction(id) => (7, id as i32, 0.0, 0.0),
            Event::Focus(focused) => (8, i32::from(focused), 0.0, 0.0),
        };
        validate_bounded_numbers(
            "fp_event",
            &[a, b],
            self.store.data().limits.max_coordinate_abs,
        )?;
        let function = self.exports.event.clone();
        self.call_status("fp_event", function, (kind, code, a, b))
    }

    /// Advances deterministic simulation time by a bounded integer count.
    pub fn tick(&mut self, ticks: u32) -> Result<(), FrontplaneError> {
        if ticks > self.store.data().limits.max_ticks_per_call {
            return Err(FrontplaneError::BudgetExhausted {
                operation: "fp_tick",
                kind: "tick",
            });
        }
        let function = self.exports.tick.clone();
        self.call_status("fp_tick", function, ticks as i32)
    }

    /// Captures one all-or-nothing immutable command buffer, rejecting partial
    /// frames and balanced-stack violations before an adapter can paint them.
    pub fn render(&mut self) -> Result<FrameOutput, FrontplaneError> {
        {
            let state = self.store.data_mut();
            state.frame = None;
            state.transform_depth = 0;
            state.current_path = None;
            state.completed_frame = None;
        }
        let function = self.exports.render.clone();
        self.call_status("fp_render", function, ())?;
        self.store
            .data_mut()
            .completed_frame
            .take()
            .ok_or(FrontplaneError::InvalidFrame(
                "fp_render did not complete a frame",
            ))
    }

    /// Copies the guest-declared opaque state region after independently
    /// validating its pointer, schema, and configured byte limit.
    pub fn snapshot(&mut self) -> Result<Snapshot, FrontplaneError> {
        let (pointer, length, schema) = self.state_region()?;
        let bytes = self
            ._memory
            .data(&self.store)
            .get(pointer..pointer + length)
            .ok_or(FrontplaneError::InvalidPointer {
                operation: "snapshot",
            })?
            .to_vec();
        Ok(Snapshot { schema, bytes })
    }

    /// Restores only an exact schema/length match so malformed or stale state
    /// cannot partially overwrite the current simulation.
    pub fn restore(&mut self, snapshot: &Snapshot) -> Result<(), FrontplaneError> {
        let (pointer, length, schema) = self.state_region()?;
        if schema != snapshot.schema || length != snapshot.bytes.len() {
            return Err(FrontplaneError::SnapshotMismatch);
        }
        self._memory
            .write(&mut self.store, pointer, &snapshot.bytes)
            .map_err(|_| FrontplaneError::InvalidPointer {
                operation: "restore",
            })?;
        if let Some(after_restore) = self.exports.after_restore.clone() {
            self.call_status("fp_after_restore", after_restore, ())?;
        }
        Ok(())
    }

    pub fn drain_audio(&mut self) -> Vec<AudioEvent> {
        std::mem::take(&mut self.store.data_mut().audio)
    }

    pub fn drain_effects(&mut self) -> Vec<HostEffect> {
        std::mem::take(&mut self.store.data_mut().effects)
    }

    /// Negotiates the opaque snapshot slice while guarding signed conversions,
    /// arithmetic overflow, guest memory bounds, and host policy.
    fn state_region(&mut self) -> Result<(usize, usize, u32), FrontplaneError> {
        let state_ptr = self.exports.state_ptr.clone();
        let state_len = self.exports.state_len.clone();
        let state_schema = self.exports.state_schema.clone();
        let pointer = self.call_value("fp_state_ptr", state_ptr)?;
        let length = self.call_value("fp_state_len", state_len)?;
        let schema = self.call_value("fp_state_schema", state_schema)?;
        if pointer < 0
            || length < 0
            || length as usize > self.store.data().limits.max_snapshot_bytes
            || schema < 0
        {
            return Err(FrontplaneError::InvalidPointer {
                operation: "snapshot",
            });
        }
        let pointer = pointer as usize;
        let length = length as usize;
        if pointer
            .checked_add(length)
            .is_none_or(|end| end > self._memory.data_size(&self.store))
        {
            return Err(FrontplaneError::InvalidPointer {
                operation: "snapshot",
            });
        }
        Ok((pointer, length, schema as u32))
    }

    fn call_value(
        &mut self,
        operation: &'static str,
        function: TypedFunc<(), i32>,
    ) -> Result<i32, FrontplaneError> {
        self.prepare_call()?;
        let result = function.call(&mut self.store, ());
        self.finish_call(operation, result)
    }

    fn call_status<P>(
        &mut self,
        operation: &'static str,
        function: TypedFunc<P, i32>,
        parameters: P,
    ) -> Result<(), FrontplaneError>
    where
        P: wasmtime::WasmParams,
    {
        self.prepare_call()?;
        let result = function.call(&mut self.store, parameters);
        let status = self.finish_call(operation, result)?;
        if status == 0 {
            Ok(())
        } else {
            self.rollback_outputs();
            Err(FrontplaneError::PluginStatus { operation, status })
        }
    }

    /// Establishes fuel and output checkpoints that make each untrusted guest
    /// export an independently bounded transaction.
    fn prepare_call(&mut self) -> Result<(), FrontplaneError> {
        let state = self.store.data_mut();
        state.pending_error = None;
        state.audio_checkpoint = state.audio.len();
        state.effect_checkpoint = state.effects.len();
        self.store
            .set_fuel(self.store.data().limits.fuel_per_call)
            .map_err(runtime_error)
    }

    /// Converts Wasmtime traps and host-import rejections into typed failures,
    /// rolling back semantic side effects from the failed export.
    fn finish_call(
        &mut self,
        operation: &'static str,
        result: Result<i32, wasmtime::Error>,
    ) -> Result<i32, FrontplaneError> {
        let pending_error = self.store.data_mut().pending_error.take();
        let outcome = if let Some(error) = pending_error {
            Err(error.into_public())
        } else {
            match result {
                Ok(value) => Ok(value),
                Err(error) => {
                    if self.store.get_fuel().is_ok_and(|fuel| fuel == 0) {
                        Err(FrontplaneError::FuelExhausted { operation })
                    } else {
                        Err(FrontplaneError::Trap {
                            operation,
                            message: error.to_string(),
                        })
                    }
                }
            }
        };
        if outcome.is_err() {
            self.rollback_outputs();
        }
        outcome
    }

    /// Truncates only events emitted since the current call began, preserving
    /// prior successful output until the adapter drains it.
    fn rollback_outputs(&mut self) {
        let state = self.store.data_mut();
        state.audio.truncate(state.audio_checkpoint);
        state.effects.truncate(state.effect_checkpoint);
    }
}

/// Builds and smoke-renders a replacement instance before exposing it to the
/// caller, preserving exact compatible snapshots while leaving `current`
/// untouched on every candidate failure.
pub fn prepare_reload(
    current: &mut Frontplane,
    source: &str,
    limits: Limits,
    init: PluginInit,
) -> Result<PreparedReload, FrontplaneError> {
    let snapshot = current.snapshot()?;
    let mut candidate = Frontplane::from_wat(source, limits)?;
    candidate.configure()?;
    candidate.init(init.seed, init.viewport_width, init.viewport_height)?;
    let state_transfer = match candidate.restore(&snapshot) {
        Ok(()) => StateTransfer::Preserved,
        Err(FrontplaneError::SnapshotMismatch) => StateTransfer::Restarted,
        Err(error) => return Err(error),
    };
    let frame = candidate.render()?;
    candidate.drain_audio();
    candidate.drain_effects();
    Ok(PreparedReload {
        frontplane: candidate,
        frame,
        state_transfer,
    })
}

const SUPPORTED_IMPORTS: &[&str] = &[
    "title",
    "menu_item",
    "image_define",
    "image_release",
    "frame_begin",
    "transform_push",
    "transform_pop",
    "path_begin",
    "path_move",
    "path_line",
    "path_quad",
    "path_cubic",
    "path_close",
    "path_end",
    "sprite",
    "line",
    "circle",
    "text",
    "frame_end",
    "audio",
    "effect",
    "log",
];

fn required_func<P, R>(
    instance: &Instance,
    store: &mut Store<HostState>,
    name: &'static str,
) -> Result<TypedFunc<P, R>, FrontplaneError>
where
    P: wasmtime::WasmParams,
    R: wasmtime::WasmResults,
{
    instance
        .get_typed_func::<P, R>(store, name)
        .map_err(|_| FrontplaneError::MissingExport { name })
}

fn runtime_error(error: wasmtime::Error) -> FrontplaneError {
    FrontplaneError::Runtime(error.to_string())
}

fn validate_bounded_numbers(
    operation: &'static str,
    values: &[f32],
    max_abs: f32,
) -> Result<(), FrontplaneError> {
    if bounded_finite(values, max_abs) {
        Ok(())
    } else {
        Err(FrontplaneError::InvalidNumber { operation })
    }
}

/// Defines the complete capability allowlist and validates each guest value at
/// the import boundary before adding it to trusted host-owned buffers.
fn bind_host_functions(linker: &mut Linker<HostState>) -> Result<(), FrontplaneError> {
    linker
        .func_wrap(
            "host.v0",
            "title",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| match read_string(
                &mut caller,
                ptr,
                len,
                "title",
            ) {
                Ok(title) => {
                    caller.data_mut().metadata.title = title;
                    0
                }
                Err(error) => caller.data_mut().reject(error, -3),
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "menu_item",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             ptr: i32,
             len: i32,
             shortcut: i32,
             flags: i32| {
                let separator = flags & 1 != 0;
                if flags & !1 != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("unsupported menu flags"), -8);
                }
                if caller.data().metadata.menu_items.len() >= caller.data().limits.max_menu_items {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("menu_item", "menu"), -2);
                }
                if separator {
                    caller
                        .data_mut()
                        .metadata
                        .menu_items
                        .push(MenuItem::separator());
                    return 0;
                }
                let id = id as u32;
                if !caller.data_mut().menu_ids.insert(id) {
                    return caller
                        .data_mut()
                        .reject(PendingError::DuplicateId("menu_item", id), -7);
                }
                let label = match read_string(&mut caller, ptr, len, "menu_item") {
                    Ok(label) => label,
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                let shortcut = match shortcut {
                    0 => None,
                    1 => Some("Ctrl+N"),
                    6 => Some("Ctrl+Q"),
                    _ => None,
                };
                caller
                    .data_mut()
                    .metadata
                    .menu_items
                    .push(MenuItem::action(id, label, shortcut));
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "image_define",
            |mut caller: Caller<'_, HostState>, id: i32, ptr: i32, len: i32, flags: i32| {
                if flags & !1 != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("unsupported image flags"), -8);
                }
                if len < 0 || len as usize > caller.data().limits.max_image_bytes {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("image_define", "image byte"), -2);
                }
                let current_bytes: usize = caller
                    .data()
                    .images
                    .iter()
                    .map(|image| image.bytes.len())
                    .sum();
                if current_bytes.saturating_add(len as usize)
                    > caller.data().limits.max_total_image_bytes
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("image_define", "total image byte"), -2);
                }
                let id = id as u32;
                if !caller.data_mut().image_ids.insert(id) {
                    return caller
                        .data_mut()
                        .reject(PendingError::DuplicateId("image_define", id), -7);
                }
                let max_bytes = caller.data().limits.max_image_bytes;
                let bytes = match read_bytes(&mut caller, ptr, len, max_bytes, "image_define") {
                    Ok(bytes) => bytes,
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                caller.data_mut().images.push(ImageResource {
                    id,
                    format: if flags & 1 == 0 {
                        ImageFormat::Encoded
                    } else {
                        ImageFormat::RawRgba
                    },
                    bytes,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "image_release",
            |mut caller: Caller<'_, HostState>, id: i32| {
                let id = id as u32;
                caller.data_mut().image_ids.remove(&id);
                caller.data_mut().images.retain(|image| image.id != id);
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "frame_begin",
            |mut caller: Caller<'_, HostState>, r: f32, g: f32, b: f32, a: f32| {
                if !normalized(&[r, g, b, a]) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("frame_begin"), -5);
                }
                if caller.data().frame.is_some() {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("nested frame_begin"), -6);
                }
                caller.data_mut().frame = Some(FrameBuilder {
                    background: components_to_rgba(r, g, b, a),
                    commands: Vec::new(),
                    ids: HashSet::new(),
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "transform_push",
            |mut caller: Caller<'_, HostState>,
             m11: f32,
             m12: f32,
             m21: f32,
             m22: f32,
             tx: f32,
             ty: f32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[m11, m12, m21, m22, tx, ty], max_abs) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("transform_push"), -5);
                }
                let command = DrawCommand::PushTransform(Affine {
                    m11,
                    m12,
                    m21,
                    m22,
                    tx,
                    ty,
                });
                let status = push_unkeyed(caller.data_mut(), command, "transform_push");
                if status == 0 {
                    caller.data_mut().transform_depth += 1;
                }
                status
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "transform_pop",
            |mut caller: Caller<'_, HostState>| {
                if caller.data().transform_depth == 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("transform stack underflow"), -6);
                }
                let status = push_unkeyed(
                    caller.data_mut(),
                    DrawCommand::PopTransform,
                    "transform_pop",
                );
                if status == 0 {
                    caller.data_mut().transform_depth -= 1;
                }
                status
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_begin",
            |mut caller: Caller<'_, HostState>, id: i32| {
                if caller.data().frame.is_none() || caller.data().current_path.is_some() {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("nested or out-of-frame path_begin"),
                        -6,
                    );
                }
                caller.data_mut().current_path = Some(PathBuilder {
                    id: id as u32,
                    segments: Vec::new(),
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_move",
            |mut caller: Caller<'_, HostState>, x: f32, y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Move(Point { x, y }),
                    &[x, y],
                    "path_move",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_line",
            |mut caller: Caller<'_, HostState>, x: f32, y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Line(Point { x, y }),
                    &[x, y],
                    "path_line",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_quad",
            |mut caller: Caller<'_, HostState>, cx: f32, cy: f32, x: f32, y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Quadratic {
                        control: Point { x: cx, y: cy },
                        end: Point { x, y },
                    },
                    &[cx, cy, x, y],
                    "path_quad",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_cubic",
            |mut caller: Caller<'_, HostState>,
             c1x: f32,
             c1y: f32,
             c2x: f32,
             c2y: f32,
             x: f32,
             y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Cubic {
                        control_1: Point { x: c1x, y: c1y },
                        control_2: Point { x: c2x, y: c2y },
                        end: Point { x, y },
                    },
                    &[c1x, c1y, c2x, c2y, x, y],
                    "path_cubic",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_close",
            |mut caller: Caller<'_, HostState>| {
                push_path_segment(&mut caller, PathSegment::Close, &[], "path_close")
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "path_end",
            |mut caller: Caller<'_, HostState>, width: f32, fill: i32, stroke: i32, flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[width], max_abs) || width < 0.0 || flags != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("path_end"), -5);
                }
                let Some(path) = caller.data_mut().current_path.take() else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("path_end without path_begin"),
                        -6,
                    );
                };
                push_command(
                    caller.data_mut(),
                    path.id,
                    DrawCommand::Path {
                        id: path.id,
                        segments: path.segments,
                        width,
                        fill_rgba: Some(fill as u32),
                        stroke_rgba: Some(stroke as u32),
                    },
                    "path_end",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "sprite",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             image_id: i32,
             src_x: f32,
             src_y: f32,
             src_w: f32,
             src_h: f32,
             dst_x: f32,
             dst_y: f32,
             dst_w: f32,
             dst_h: f32,
             pivot_x: f32,
             pivot_y: f32,
             tint: i32,
             flags: i32| {
                let values = [
                    src_x, src_y, src_w, src_h, dst_x, dst_y, dst_w, dst_h, pivot_x, pivot_y,
                ];
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&values, max_abs)
                    || [src_w, src_h, dst_w, dst_h]
                        .iter()
                        .any(|value| *value < 0.0)
                    || !(0.0..=1.0).contains(&pivot_x)
                    || !(0.0..=1.0).contains(&pivot_y)
                    || flags != 0
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("sprite"), -5);
                }
                if !caller.data().image_ids.contains(&(image_id as u32)) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("sprite uses unknown image"), -8);
                }
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Sprite {
                        id: id as u32,
                        image_id: image_id as u32,
                        source: Rect {
                            x: src_x,
                            y: src_y,
                            width: src_w,
                            height: src_h,
                        },
                        destination: Rect {
                            x: dst_x,
                            y: dst_y,
                            width: dst_w,
                            height: dst_h,
                        },
                        pivot: Point {
                            x: pivot_x,
                            y: pivot_y,
                        },
                        tint_rgba: tint as u32,
                        flags: flags as u32,
                    },
                    "sprite",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "line",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             x1: f32,
             y1: f32,
             x2: f32,
             y2: f32,
             width: f32,
             rgba: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[x1, y1, x2, y2, width], max_abs) || width < 0.0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("line"), -5);
                }
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Line {
                        id: id as u32,
                        x1,
                        y1,
                        x2,
                        y2,
                        width,
                        rgba: rgba as u32,
                    },
                    "line",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "circle",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             x: f32,
             y: f32,
             radius: f32,
             width: f32,
             rgba: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[x, y, radius, width], max_abs)
                    || radius < 0.0
                    || width < 0.0
                    || flags & !1 != 0
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("circle"), -5);
                }
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Circle {
                        id: id as u32,
                        x,
                        y,
                        radius,
                        width,
                        rgba: rgba as u32,
                        filled: flags & 1 != 0,
                    },
                    "circle",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "text",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             ptr: i32,
             len: i32,
             x: f32,
             y: f32,
             size: f32,
             rgba: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[x, y, size], max_abs) || size < 0.0 || flags & !1 != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("text"), -5);
                }
                let text = match read_string(&mut caller, ptr, len, "text") {
                    Ok(text) => text,
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Text {
                        id: id as u32,
                        text,
                        x,
                        y,
                        size,
                        rgba: rgba as u32,
                        centered: flags & 1 != 0,
                    },
                    "text",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "frame_end",
            |mut caller: Caller<'_, HostState>| {
                if caller.data().transform_depth != 0 || caller.data().current_path.is_some() {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("unbalanced composition stack"),
                        -6,
                    );
                }
                let Some(frame) = caller.data_mut().frame.take() else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("frame_end without frame_begin"),
                        -6,
                    );
                };
                caller.data_mut().completed_frame = Some(FrameOutput {
                    background: frame.background,
                    commands: frame.commands,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "audio",
            |mut caller: Caller<'_, HostState>, id: i32, volume: f32, pitch: f32, flags: i32| {
                if !finite(&[volume, pitch])
                    || !(0.0..=1.0).contains(&volume)
                    || !(0.25..=4.0).contains(&pitch)
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("audio"), -5);
                }
                if caller.data().audio.len() >= caller.data().limits.max_audio_events {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("audio", "audio event"), -2);
                }
                caller.data_mut().audio.push(AudioEvent {
                    id: id as u32,
                    volume,
                    pitch,
                    flags: flags as u32,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "effect",
            |mut caller: Caller<'_, HostState>, kind: i32, a: i32, _b: i32| {
                if caller.data().effects.len() >= caller.data().limits.max_effects {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("effect", "effect"), -2);
                }
                let effect = match kind {
                    1 => HostEffect::Redraw,
                    2 => HostEffect::Quit,
                    3 => HostEffect::SetCursor(a),
                    4 => HostEffect::PersistSnapshot,
                    // open-url (5) requires a manifest capability that v0 does not yet expose.
                    _ => {
                        return caller
                            .data_mut()
                            .reject(PendingError::Unsupported("effect"), -8);
                    }
                };
                caller.data_mut().effects.push(effect);
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            "host.v0",
            "log",
            |mut caller: Caller<'_, HostState>, _level: i32, ptr: i32, len: i32| match read_string(
                &mut caller,
                ptr,
                len,
                "log",
            ) {
                Ok(_) => 0,
                Err(error) => caller.data_mut().reject(error, -3),
            },
        )
        .map_err(runtime_error)?;
    Ok(())
}

fn read_string(
    caller: &mut Caller<'_, HostState>,
    pointer: i32,
    length: i32,
    operation: &'static str,
) -> Result<String, PendingError> {
    let max_bytes = caller.data().limits.max_string_bytes;
    let bytes = read_bytes(caller, pointer, length, max_bytes, operation)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| PendingError::InvalidUtf8(operation))?;
    Ok(text.to_owned())
}

/// Copies a bounded slice out of guest linear memory without exposing borrowed
/// Wasmtime memory beyond the import invocation.
fn read_bytes(
    caller: &mut Caller<'_, HostState>,
    pointer: i32,
    length: i32,
    max_bytes: usize,
    operation: &'static str,
) -> Result<Vec<u8>, PendingError> {
    if pointer < 0 || length < 0 || length as usize > max_bytes {
        return Err(PendingError::InvalidPointer(operation));
    }
    let memory = caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
        .ok_or(PendingError::InvalidPointer(operation))?;
    let pointer = pointer as usize;
    let length = length as usize;
    let bytes = memory
        .data(&*caller)
        .get(pointer..pointer.saturating_add(length))
        .ok_or(PendingError::InvalidPointer(operation))?;
    Ok(bytes.to_vec())
}

/// Adds a keyed scene primitive only inside a live frame, enforcing both the
/// global command budget and stable-ID uniqueness.
fn push_command(
    state: &mut HostState,
    id: u32,
    command: DrawCommand,
    operation: &'static str,
) -> i32 {
    let max_commands = state.limits.max_commands;
    let Some(frame) = state.frame.as_mut() else {
        return state.reject(PendingError::InvalidFrame("draw command outside frame"), -6);
    };
    if frame.commands.len() >= max_commands {
        return state.reject(PendingError::Budget(operation, "draw command"), -2);
    }
    if !frame.ids.insert(id) {
        return state.reject(PendingError::DuplicateId(operation, id), -7);
    }
    frame.commands.push(command);
    0
}

fn push_unkeyed(state: &mut HostState, command: DrawCommand, operation: &'static str) -> i32 {
    let max_commands = state.limits.max_commands;
    let Some(frame) = state.frame.as_mut() else {
        return state.reject(
            PendingError::InvalidFrame("composition command outside frame"),
            -6,
        );
    };
    if frame.commands.len() >= max_commands {
        return state.reject(PendingError::Budget(operation, "draw command"), -2);
    }
    frame.commands.push(command);
    0
}

/// Extends the one active vector path under finite-number and segment-count
/// controls, preventing malformed paths from reaching GPUI.
fn push_path_segment(
    caller: &mut Caller<'_, HostState>,
    segment: PathSegment,
    values: &[f32],
    operation: &'static str,
) -> i32 {
    let max_abs = caller.data().limits.max_coordinate_abs;
    if !bounded_finite(values, max_abs) {
        return caller
            .data_mut()
            .reject(PendingError::InvalidNumber(operation), -5);
    }
    let limit = caller.data().limits.max_path_segments;
    let Some(path) = caller.data_mut().current_path.as_mut() else {
        return caller.data_mut().reject(
            PendingError::InvalidFrame("path segment without path_begin"),
            -6,
        );
    };
    if path.segments.len() >= limit {
        return caller
            .data_mut()
            .reject(PendingError::Budget(operation, "path segment"), -2);
    }
    path.segments.push(segment);
    0
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn bounded_finite(values: &[f32], max_abs: f32) -> bool {
    max_abs.is_finite()
        && max_abs >= 0.0
        && values
            .iter()
            .all(|value| value.is_finite() && value.abs() <= max_abs)
}

fn normalized(values: &[f32]) -> bool {
    finite(values) && values.iter().all(|value| (0.0..=1.0).contains(value))
}

fn components_to_rgba(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let byte = |value: f32| (value * 255.0).round() as u32;
    (byte(r) << 24) | (byte(g) << 16) | (byte(b) << 8) | byte(a)
}
