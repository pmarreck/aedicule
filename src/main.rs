use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use gpui::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, FocusHandle, Focusable, Hsla,
    InteractiveElement as _, IntoElement, KeyBinding, KeyDownEvent, KeyUpEvent, Menu, MenuItem,
    ParentElement as _, PathBuilder, Render, SharedString, Styled as _, TextAlign, TextRun, Window,
    WindowBounds, WindowOptions, actions, canvas, div, point, prelude::FluentBuilder as _, px,
    rgba, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_wasm::{
    Affine, AudioEvent, DrawCommand, Event, FileRevision, FrameOutput, Frontplane, HostEffect, Key,
    LaunchAction, Limits, Metadata, PathSegment, PluginInit, PluginSource, RevisionTracker,
    StateTransfer, prepare_reload, resolve_launch,
};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Source as _, source::SineWave};
use smol::Timer;

actions!(gpui_wasm, [NewGame, ReloadPlugin, Quit]);

const DEMO_WAT: &str = include_str!("../plugins/vibesteroids.wat");
const LOGICAL_WIDTH: f32 = 1024.0;
const LOGICAL_HEIGHT: f32 = 768.0;
const FRAME_INTERVAL: Duration = Duration::from_micros(16_667);
const WATCH_INTERVAL: Duration = Duration::from_millis(250);
const PLUGIN_INIT: PluginInit = PluginInit::new(0x5eed_cafe, LOGICAL_WIDTH, LOGICAL_HEIGHT);

struct Strings {
    fallback_title: &'static str,
    application_menu: &'static str,
    abi_status: &'static str,
    runtime_status: &'static str,
    plugin_stopped: &'static str,
    reload: &'static str,
    reload_failed: &'static str,
    reload_preserved: &'static str,
    reload_restarted: &'static str,
    embedded_source: &'static str,
    watching: &'static str,
    external_source: &'static str,
    missing_source: &'static str,
    unreadable_source: &'static str,
    invalid_utf8: &'static str,
    help: &'static str,
    about: &'static str,
    cli_error: &'static str,
}

const EN: Strings = Strings {
    fallback_title: "WAT Application — GPUI Frontplane",
    application_menu: "Application",
    abi_status: "WAT owns gameplay • GPUI owns the window",
    runtime_status: "ABI v0 • 60 Hz • capability-bounded",
    plugin_stopped: "WAT plugin stopped safely",
    reload: "Reload",
    reload_failed: "Reload failed; previous plugin remains active",
    reload_preserved: "Reloaded with live state preserved",
    reload_restarted: "Reloaded with fresh state (schema changed)",
    embedded_source: "embedded demo",
    watching: "watching",
    external_source: "external WAT",
    missing_source: "WAT source does not exist",
    unreadable_source: "could not read WAT source",
    invalid_utf8: "WAT source is not valid UTF-8",
    help: "\
Run a capability-bounded WAT application in the native GPUI frontplane.

Usage:
  gpui-wasm [OPTIONS] [PLUGIN.wat|APPLICATION_DIRECTORY]

Options:
  --watch       Reload after content changes; defaults to ./code.wat
  --embedded    Force the bundled Vibesteroids demonstration
  -h, --help    Show this help
  --about       Show version and build platform

Without arguments, ./code.wat is loaded when present; otherwise the embedded
demo runs. An application directory resolves to its code.wat file.
",
    about: "generic native GPUI frontplane for capability-bounded WAT applications",
    cli_error: "gpui-wasm",
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum StandardMenuEntry {
    New(String),
    Separator,
    Quit(String),
}

/// Reduces plugin metadata to the standard actions the native GPUI adapter can
/// represent without synthesizing application-specific Rust action types.
fn standard_menu_entries(metadata: &Metadata) -> Vec<StandardMenuEntry> {
    metadata
        .menu_items
        .iter()
        .filter_map(|item| {
            if item.separator {
                return Some(StandardMenuEntry::Separator);
            }
            match item.id {
                Some(1) => Some(StandardMenuEntry::New(item.label.clone())),
                Some(6) => Some(StandardMenuEntry::Quit(item.label.clone())),
                _ => None,
            }
        })
        .collect()
}

fn native_menus(metadata: &Metadata, can_reload: bool) -> Vec<Menu> {
    let mut items: Vec<_> = standard_menu_entries(metadata)
        .into_iter()
        .map(|entry| match entry {
            StandardMenuEntry::New(label) => MenuItem::action(label, NewGame),
            StandardMenuEntry::Separator => MenuItem::Separator,
            StandardMenuEntry::Quit(label) => MenuItem::action(label, Quit),
        })
        .collect();
    if can_reload {
        if !items.is_empty() {
            items.push(MenuItem::Separator);
        }
        items.push(MenuItem::action(EN.reload, ReloadPlugin));
    }
    if items.is_empty() {
        Vec::new()
    } else {
        vec![Menu {
            name: EN.application_menu.into(),
            items,
            disabled: false,
        }]
    }
}

struct AudioOutput {
    sink: Option<MixerDeviceSink>,
    last_thrust: Option<Instant>,
}

impl AudioOutput {
    fn new() -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().ok().map(|mut sink| {
            sink.log_on_drop(false);
            sink
        });
        Self {
            sink,
            last_thrust: None,
        }
    }

    fn play(&mut self, event: AudioEvent) {
        let Some(sink) = &self.sink else { return };
        if event.id == 4 {
            let now = Instant::now();
            if self
                .last_thrust
                .is_some_and(|last| now.duration_since(last) < Duration::from_millis(55))
            {
                return;
            }
            self.last_thrust = Some(now);
        }
        let (base_frequency, milliseconds) = match event.id {
            1 => (760.0, 90),
            2 => (180.0, 170),
            3 => (85.0, 420),
            4 => (58.0, 65),
            5 => return,
            6 => (1046.5, 260),
            _ => (440.0, 70),
        };
        let tone = SineWave::new(base_frequency * event.pitch)
            .take_duration(Duration::from_millis(milliseconds))
            .amplify(event.volume * 0.18)
            .fade_out(Duration::from_millis(milliseconds / 2));
        sink.mixer().add(tone);
    }
}

struct ExternalSource {
    path: PathBuf,
    watch: bool,
    revisions: RevisionTracker,
}

struct Startup {
    frontplane: Frontplane,
    frame: FrameOutput,
    source: Option<ExternalSource>,
    reload_error: Option<String>,
}

struct FrontplaneView {
    frontplane: Frontplane,
    frame: FrameOutput,
    title: String,
    new_label: Option<String>,
    quit_label: Option<String>,
    focus_handle: FocusHandle,
    window_handle: AnyWindowHandle,
    audio: AudioOutput,
    source: Option<ExternalSource>,
    fatal_error: Option<String>,
    reload_error: Option<String>,
    reload_notice: Option<&'static str>,
}

impl FrontplaneView {
    fn new(startup: Startup, window: &Window, cx: &mut Context<Self>) -> Self {
        let Startup {
            frontplane,
            frame,
            source,
            reload_error,
        } = startup;
        let metadata = frontplane.metadata().clone();
        let entries = standard_menu_entries(&metadata);
        let view = Self {
            frontplane,
            frame,
            title: metadata.title,
            new_label: entries.iter().find_map(|entry| match entry {
                StandardMenuEntry::New(label) => Some(label.clone()),
                _ => None,
            }),
            quit_label: entries.iter().find_map(|entry| match entry {
                StandardMenuEntry::Quit(label) => Some(label.clone()),
                _ => None,
            }),
            focus_handle: cx.focus_handle(),
            window_handle: window.window_handle(),
            audio: AudioOutput::new(),
            source,
            fatal_error: None,
            reload_error,
            reload_notice: None,
        };

        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(FRAME_INTERVAL).await;
                if this
                    .update(cx, |this, cx| {
                        this.advance(cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        if view.source.as_ref().is_some_and(|source| source.watch) {
            cx.spawn(async move |this, cx| {
                loop {
                    Timer::after(WATCH_INTERVAL).await;
                    if this
                        .update(cx, |this, cx| {
                            if this.poll_reload(cx) {
                                cx.notify();
                            }
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .detach();
        }
        view
    }

    fn poll_reload(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(source) = self.source.as_mut() else {
            return false;
        };
        let path = source.path.clone();
        let revision = FileRevision::read(&path);
        if !source.revisions.observe(&revision) {
            return false;
        }
        self.apply_revision(&path, revision, cx);
        true
    }

    fn reload_plugin(&mut self, _: &ReloadPlugin, _: &mut Window, cx: &mut Context<Self>) {
        let Some(source) = self.source.as_mut() else {
            return;
        };
        let path = source.path.clone();
        let revision = FileRevision::read(&path);
        source.revisions.observe(&revision);
        self.apply_revision(&path, revision, cx);
        cx.notify();
    }

    /// Compiles and validates a changed file as a separate candidate, swapping
    /// only after state transfer and the candidate's first frame both succeed.
    fn apply_revision(&mut self, path: &Path, revision: FileRevision, cx: &mut Context<Self>) {
        let bytes = match revision {
            FileRevision::Missing => {
                self.set_reload_error(path, EN.missing_source, None);
                return;
            }
            FileRevision::Unreadable(error) => {
                self.set_reload_error(path, EN.unreadable_source, Some(&error));
                return;
            }
            FileRevision::Content(bytes) => bytes,
        };
        let source = match String::from_utf8(bytes) {
            Ok(source) => source,
            Err(error) => {
                self.set_reload_error(path, EN.invalid_utf8, Some(&error.to_string()));
                return;
            }
        };
        match prepare_reload(
            &mut self.frontplane,
            &source,
            Limits::default(),
            PLUGIN_INIT,
        ) {
            Ok(prepared) => {
                let metadata = prepared.frontplane.metadata().clone();
                let entries = standard_menu_entries(&metadata);
                self.frontplane = prepared.frontplane;
                self.frame = prepared.frame;
                self.title = metadata.title.clone();
                self.new_label = entries.iter().find_map(|entry| match entry {
                    StandardMenuEntry::New(label) => Some(label.clone()),
                    _ => None,
                });
                self.quit_label = entries.iter().find_map(|entry| match entry {
                    StandardMenuEntry::Quit(label) => Some(label.clone()),
                    _ => None,
                });
                self.fatal_error = None;
                self.reload_error = None;
                self.reload_notice = Some(match prepared.state_transfer {
                    StateTransfer::Preserved => EN.reload_preserved,
                    StateTransfer::Restarted => EN.reload_restarted,
                });
                cx.set_menus(native_menus(&metadata, true));
                let title = window_title(&metadata);
                let _ = cx.update_window(self.window_handle, |_, window, _| {
                    window.set_window_title(&title);
                });
            }
            Err(error) => {
                self.set_reload_error(path, EN.reload_failed, Some(&error.to_string()));
            }
        }
    }

    fn set_reload_error(&mut self, path: &Path, message: &str, detail: Option<&str>) {
        self.reload_notice = None;
        self.reload_error = Some(match detail {
            Some(detail) => format!("{message}: {}: {detail}", path.display()),
            None => format!("{message}: {}", path.display()),
        });
    }

    /// Runs one fixed guest tick, atomically replaces the last good frame, and
    /// drains semantic effects only after Wasmtime releases its store borrow.
    fn advance(&mut self, cx: &mut Context<Self>) {
        if self.fatal_error.is_some() {
            return;
        }
        if let Err(error) = self
            .frontplane
            .tick(1)
            .and_then(|_| self.frontplane.render().map(|frame| self.frame = frame))
        {
            self.fatal_error = Some(error.to_string());
            return;
        }
        for event in self.frontplane.drain_audio() {
            self.audio.play(event);
        }
        for effect in self.frontplane.drain_effects() {
            if effect == HostEffect::Quit {
                cx.quit();
            }
        }
    }

    fn new_game(&mut self, _: &NewGame, _: &mut Window, cx: &mut Context<Self>) {
        self.fatal_error = None;
        if let Err(error) = self
            .frontplane
            .event(Event::MenuAction(1))
            .and_then(|_| self.frontplane.render().map(|frame| self.frame = frame))
        {
            self.fatal_error = Some(error.to_string());
        }
        cx.notify();
    }

    fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = map_key(&event.keystroke.key) else {
            return;
        };
        if event.is_held && matches!(key, Key::Pause | Key::Restart) {
            return;
        }
        if let Err(error) = self.frontplane.event(Event::KeyDown(key)) {
            self.fatal_error = Some(error.to_string());
        }
        cx.notify();
    }

    fn key_up(&mut self, event: &KeyUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = map_key(&event.keystroke.key) else {
            return;
        };
        if let Err(error) = self.frontplane.event(Event::KeyUp(key)) {
            self.fatal_error = Some(error.to_string());
        }
        cx.notify();
    }
}

impl Focusable for FrontplaneView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FrontplaneView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let frame = self.frame.clone();
        let fatal_error = self.fatal_error.clone();
        let reload_error = self.reload_error.clone();
        let title = self.title.clone();
        let new_label = self.new_label.clone();
        let quit_label = self.quit_label.clone();
        let can_reload = self.source.is_some();
        let source_status = match &self.source {
            Some(source) if source.watch => {
                format!("{} {}", EN.watching, source.path.display())
            }
            Some(source) => format!("{} {}", EN.external_source, source.path.display()),
            None => EN.embedded_source.to_owned(),
        };
        let runtime_status = match self.reload_notice {
            Some(notice) => format!("{notice} • {source_status}"),
            None => format!("{} • {source_status}", EN.runtime_status),
        };
        v_flex()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            .on_action(cx.listener(Self::new_game))
            .on_action(cx.listener(Self::reload_plugin))
            .on_action(cx.listener(Self::quit))
            .child(
                TitleBar::new().child(
                    h_flex()
                        .w_full()
                        .pr_3()
                        .gap_2()
                        .justify_between()
                        .child(title)
                        .child(
                            h_flex()
                                .gap_2()
                                .when(can_reload, |this| {
                                    this.child(
                                        Button::new("reload-plugin").label(EN.reload).on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.reload_plugin(&ReloadPlugin, window, cx)
                                            }),
                                        ),
                                    )
                                })
                                .when_some(new_label, |this, label| {
                                    this.child(
                                        Button::new("new-game").primary().label(label).on_click(
                                            cx.listener(|this, _, window, cx| {
                                                this.new_game(&NewGame, window, cx)
                                            }),
                                        ),
                                    )
                                })
                                .when_some(quit_label, |this, label| {
                                    this.child(Button::new("quit").label(label).on_click(
                                        cx.listener(|this, _, window, cx| {
                                            this.quit(&Quit, window, cx)
                                        }),
                                    ))
                                }),
                        ),
                ),
            )
            .child(
                div()
                    .id("frontplane-canvas")
                    .relative()
                    .flex_1()
                    .w_full()
                    .bg(rgba(frame.background))
                    .child(
                        canvas(
                            move |bounds, _, _| (bounds, frame),
                            move |_, (bounds, frame), window, cx| {
                                paint_frame(&frame, bounds, window, cx)
                            },
                        )
                        .size_full(),
                    )
                    .when_some(fatal_error, |this, error| {
                        this.child(
                            div()
                                .absolute()
                                .inset_4()
                                .p_4()
                                .rounded_lg()
                                .bg(rgba(0x6b1020ee))
                                .text_color(rgba(0xffffffff))
                                .child(format!("{}: {error}", EN.plugin_stopped)),
                        )
                    })
                    .when_some(reload_error, |this, error| {
                        this.child(
                            div()
                                .absolute()
                                .top_4()
                                .left_4()
                                .right_4()
                                .p_3()
                                .rounded_lg()
                                .bg(rgba(0x6b4510ee))
                                .text_color(rgba(0xffffffff))
                                .child(error),
                        )
                    }),
            )
            .child(
                h_flex()
                    .px_3()
                    .py_1()
                    .justify_between()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(EN.abi_status)
                    .child(runtime_status),
            )
    }
}

/// Normalizes GPUI's portable key names into the versioned gameplay key IDs.
fn map_key(key: &str) -> Option<Key> {
    match key {
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "up" => Some(Key::Thrust),
        "space" | " " => Some(Key::Fire),
        "p" => Some(Key::Pause),
        "r" => Some(Key::Restart),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Matrix(Affine);

impl Matrix {
    const IDENTITY: Self = Self(Affine {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        tx: 0.0,
        ty: 0.0,
    });

    /// Composes parent and child affine transforms without leaking GPUI matrix
    /// types into the frontplane ABI.
    fn then(self, next: Affine) -> Self {
        let parent = self.0;
        Self(Affine {
            m11: parent.m11 * next.m11 + parent.m21 * next.m12,
            m12: parent.m12 * next.m11 + parent.m22 * next.m12,
            m21: parent.m11 * next.m21 + parent.m21 * next.m22,
            m22: parent.m12 * next.m21 + parent.m22 * next.m22,
            tx: parent.m11 * next.tx + parent.m21 * next.ty + parent.tx,
            ty: parent.m12 * next.tx + parent.m22 * next.ty + parent.ty,
        })
    }

    /// Projects one logical plugin point into its current composed viewport.
    fn apply(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.0.m11 * x + self.0.m21 * y + self.0.tx,
            self.0.m12 * x + self.0.m22 * y + self.0.ty,
        )
    }
}

/// Adapts a validated immutable scene command buffer into GPUI paths and text;
/// no guest execution occurs while GPUI window/context borrows are live.
fn paint_frame(
    frame: &FrameOutput,
    bounds: Bounds<gpui::Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let scale = (width / LOGICAL_WIDTH).min(height / LOGICAL_HEIGHT);
    let offset_x = f32::from(bounds.origin.x) + (width - LOGICAL_WIDTH * scale) / 2.0;
    let offset_y = f32::from(bounds.origin.y) + (height - LOGICAL_HEIGHT * scale) / 2.0;
    let viewport = Affine {
        m11: scale,
        m12: 0.0,
        m21: 0.0,
        m22: scale,
        tx: offset_x,
        ty: offset_y,
    };
    let mut matrices = vec![Matrix(viewport)];

    for command in &frame.commands {
        match command {
            DrawCommand::PushTransform(transform) => {
                matrices.push(
                    matrices
                        .last()
                        .copied()
                        .unwrap_or(Matrix::IDENTITY)
                        .then(*transform),
                );
            }
            DrawCommand::PopTransform => {
                if matrices.len() > 1 {
                    matrices.pop();
                }
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                rgba: color,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                let (x1, y1) = matrix.apply(*x1, *y1);
                let (x2, y2) = matrix.apply(*x2, *y2);
                let mut path = PathBuilder::stroke(px(*width * scale));
                path.move_to(point(px(x1), px(y1)));
                path.line_to(point(px(x2), px(y2)));
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgba(*color));
                }
            }
            DrawCommand::Circle {
                x,
                y,
                radius,
                width,
                rgba: color,
                filled,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                let mut points = Vec::with_capacity(32);
                for index in 0..32 {
                    let angle = index as f32 * std::f32::consts::TAU / 32.0;
                    let (x, y) = matrix.apply(*x + radius * angle.cos(), *y + radius * angle.sin());
                    points.push(point(px(x), px(y)));
                }
                let mut path = if *filled {
                    PathBuilder::fill()
                } else {
                    PathBuilder::stroke(px(*width * scale))
                };
                path.add_polygon(&points, true);
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgba(*color));
                }
            }
            DrawCommand::Text {
                text,
                x,
                y,
                size,
                rgba: color,
                centered,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                let (x, y) = matrix.apply(*x, *y);
                let text = SharedString::from(text.clone());
                let run = TextRun {
                    len: text.len(),
                    font: window.text_style().font(),
                    color: Hsla::from(rgba(*color)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let font_size = px(*size * scale);
                let line = window
                    .text_system()
                    .shape_line(text, font_size, &[run], None);
                let x = if *centered {
                    x - f32::from(line.width()) / 2.0
                } else {
                    x
                };
                let _ = line.paint(
                    point(px(x), px(y - f32::from(font_size) / 2.0)),
                    font_size,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
            DrawCommand::Path {
                segments,
                width,
                fill_rgba,
                stroke_rgba,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                if let Some(color) = fill_rgba {
                    paint_path(segments, matrix, PathBuilder::fill(), *color, window);
                }
                if let Some(color) = stroke_rgba {
                    paint_path(
                        segments,
                        matrix,
                        PathBuilder::stroke(px(*width * scale)),
                        *color,
                        window,
                    );
                }
            }
            DrawCommand::Sprite { destination, .. } => {
                // Image atlas decoding/cropping is the remaining v0 adapter spike.
                let matrix = *matrices.last().unwrap();
                let corners = [
                    (destination.x, destination.y),
                    (destination.x + destination.width, destination.y),
                    (
                        destination.x + destination.width,
                        destination.y + destination.height,
                    ),
                    (destination.x, destination.y + destination.height),
                ];
                let points: Vec<_> = corners
                    .into_iter()
                    .map(|(x, y)| {
                        let (x, y) = matrix.apply(x, y);
                        point(px(x), px(y))
                    })
                    .collect();
                let mut path = PathBuilder::stroke(px(scale));
                path.add_polygon(&points, true);
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgba(0xff00ffff));
                }
            }
        }
    }
}

/// Replays the generic quadratic/cubic path model through GPUI after applying
/// the current affine composition matrix.
fn paint_path(
    segments: &[PathSegment],
    matrix: Matrix,
    mut path: PathBuilder,
    color: u32,
    window: &mut Window,
) {
    for segment in segments {
        match segment {
            PathSegment::Move(point_) => {
                let (x, y) = matrix.apply(point_.x, point_.y);
                path.move_to(point(px(x), px(y)));
            }
            PathSegment::Line(point_) => {
                let (x, y) = matrix.apply(point_.x, point_.y);
                path.line_to(point(px(x), px(y)));
            }
            PathSegment::Quadratic { control, end } => {
                let (cx, cy) = matrix.apply(control.x, control.y);
                let (x, y) = matrix.apply(end.x, end.y);
                path.curve_to(point(px(x), px(y)), point(px(cx), px(cy)));
            }
            PathSegment::Cubic {
                control_1,
                control_2,
                end,
            } => {
                let (c1x, c1y) = matrix.apply(control_1.x, control_1.y);
                let (c2x, c2y) = matrix.apply(control_2.x, control_2.y);
                let (x, y) = matrix.apply(end.x, end.y);
                path.cubic_bezier_to(
                    point(px(x), px(y)),
                    point(px(c1x), px(c1y)),
                    point(px(c2x), px(c2y)),
                );
            }
            PathSegment::Close => path.close(),
        }
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, rgba(color));
    }
}

/// Instantiates the bundled demonstration through the same generic lifecycle
/// used by any future externally selected WAT plugin.
fn load_demo() -> (Frontplane, FrameOutput) {
    load_frontplane(DEMO_WAT).expect("bundled WAT passed the headless ABI suite")
}

fn load_frontplane(source: &str) -> Result<(Frontplane, FrameOutput), gpui_wasm::FrontplaneError> {
    Frontplane::from_wat(source, Limits::default()).and_then(|mut frontplane| {
        frontplane.configure()?;
        frontplane.init(
            PLUGIN_INIT.seed,
            PLUGIN_INIT.viewport_width,
            PLUGIN_INIT.viewport_height,
        )?;
        let frame = frontplane.render()?;
        frontplane.drain_audio();
        frontplane.drain_effects();
        Ok((frontplane, frame))
    })
}

fn startup(source: PluginSource, watch: bool) -> Startup {
    match source {
        PluginSource::Embedded => {
            let (frontplane, frame) = load_demo();
            Startup {
                frontplane,
                frame,
                source: None,
                reload_error: None,
            }
        }
        PluginSource::File(path) => {
            let revision = FileRevision::read(&path);
            let mut revisions = RevisionTracker::default();
            revisions.observe(&revision);
            let result = source_text(&path, &revision).and_then(|source| {
                load_frontplane(&source)
                    .map_err(|error| format!("{}: {}: {error}", EN.reload_failed, path.display()))
            });
            match result {
                Ok((frontplane, frame)) => Startup {
                    frontplane,
                    frame,
                    source: Some(ExternalSource {
                        path,
                        watch,
                        revisions,
                    }),
                    reload_error: None,
                },
                Err(error) => {
                    let (frontplane, frame) = load_demo();
                    Startup {
                        frontplane,
                        frame,
                        source: Some(ExternalSource {
                            path,
                            watch,
                            revisions,
                        }),
                        reload_error: Some(error),
                    }
                }
            }
        }
    }
}

fn source_text(path: &Path, revision: &FileRevision) -> Result<String, String> {
    match revision {
        FileRevision::Missing => Err(format!("{}: {}", EN.missing_source, path.display())),
        FileRevision::Unreadable(error) => Err(format!(
            "{}: {}: {error}",
            EN.unreadable_source,
            path.display()
        )),
        FileRevision::Content(bytes) => String::from_utf8(bytes.clone())
            .map_err(|error| format!("{}: {}: {error}", EN.invalid_utf8, path.display())),
    }
}

fn window_title(metadata: &Metadata) -> String {
    if metadata.title.is_empty() {
        EN.fallback_title.to_owned()
    } else {
        metadata.title.clone()
    }
}

fn run_application(startup: Startup) {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
    app.run(move |cx| {
        gpui_component::init(cx);
        cx.bind_keys([
            KeyBinding::new("ctrl-n", NewGame, None),
            KeyBinding::new("ctrl-r", ReloadPlugin, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);
        let metadata = startup.frontplane.metadata().clone();
        let window_title = window_title(&metadata);
        cx.set_menus(native_menus(&metadata, startup.source.is_some()));
        cx.on_action(|_: &Quit, cx| cx.quit());
        let options = WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            window_bounds: Some(WindowBounds::centered(size(px(1100.0), px(850.0)), cx)),
            ..Default::default()
        };
        cx.spawn(async move |cx| {
            cx.open_window(options, move |window, cx| {
                window.activate_window();
                window.set_window_title(&window_title);
                Theme::change(ThemeMode::Dark, Some(window), cx);
                let view = cx.new(|cx| FrontplaneView::new(startup, window, cx));
                view.focus_handle(cx).focus(window, cx);
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("open GPUI frontplane window");
        })
        .detach();
    });
}

fn main() -> ExitCode {
    if cfg!(debug_assertions) && env::var_os("MUTE_DEBUG_STATUS").is_none() {
        eprintln!("\x1b[33mDEBUG BUILD!\x1b[0m");
    }
    let working_directory = match env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            eprintln!(
                "{}: could not determine working directory: {error}",
                EN.cli_error
            );
            return ExitCode::FAILURE;
        }
    };
    let action = match resolve_launch(env::args_os().skip(1), &working_directory) {
        Ok(action) => action,
        Err(error) => {
            eprintln!("{}: {error}", EN.cli_error);
            return ExitCode::from(2);
        }
    };
    match action {
        LaunchAction::Help => print!("{}", EN.help),
        LaunchAction::About => println!(
            "gpui-wasm {} — {} for {} {}",
            env!("CARGO_PKG_VERSION"),
            EN.about,
            env::consts::OS,
            env::consts::ARCH,
        ),
        LaunchAction::Run { source, watch } => run_application(startup(source, watch)),
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{StandardMenuEntry, standard_menu_entries};
    use gpui_wasm::{MenuItem as PluginMenuItem, Metadata};

    #[test]
    fn plugin_metadata_drives_only_recognized_standard_native_actions() {
        let metadata = Metadata {
            title: "Example".into(),
            menu_items: vec![
                PluginMenuItem::action(1, "Begin", Some("Ctrl+N")),
                PluginMenuItem::separator(),
                PluginMenuItem::action(6, "Leave", Some("Ctrl+Q")),
                PluginMenuItem::action(1024, "Plugin-specific", None),
            ],
        };

        assert_eq!(
            standard_menu_entries(&metadata),
            vec![
                StandardMenuEntry::New("Begin".into()),
                StandardMenuEntry::Separator,
                StandardMenuEntry::Quit("Leave".into()),
            ]
        );
    }
}
