use std::{
    collections::HashMap,
    env,
    num::{NonZeroU16, NonZeroU32},
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
    h_flex,
};
use gpui_wasm::{
    Affine, AudioEvent, DrawCommand, Event, FileRevision, FrameOutput, Frontplane, HostEffect, Key,
    LaunchAction, Limits, Metadata, PathSegment, PluginInit, PluginSource, RevisionTracker,
    StateTransfer, SynthFilter, SynthVoice, SynthWaveform, prepare_reload, resolve_launch,
};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, buffer::SamplesBuffer};
use smol::Timer;

actions!(gpui_wasm, [NewGame, HelpControls, ReloadPlugin, Quit]);

const DEMO_WAT: &str = include_str!("../plugins/vibesteroids.wat");
const LOGICAL_WIDTH: f32 = 1024.0;
const LOGICAL_HEIGHT: f32 = 768.0;
const FRAME_INTERVAL: Duration = Duration::from_micros(16_667);
const WATCH_INTERVAL: Duration = Duration::from_millis(250);
const DEFAULT_PLUGIN_INIT: PluginInit = PluginInit::new(0x5eed_cafe, LOGICAL_WIDTH, LOGICAL_HEIGHT);

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
  --seed N      Set the deterministic unsigned 64-bit application seed
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
    Help(String),
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
                Some(7) => Some(StandardMenuEntry::Help(item.label.clone())),
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
            StandardMenuEntry::Help(label) => MenuItem::action(label, HelpControls),
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
    programs: HashMap<u32, Vec<SynthVoice>>,
    last_played: HashMap<u32, Instant>,
}

impl AudioOutput {
    fn new(voices: &[SynthVoice]) -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().ok().map(|mut sink| {
            sink.log_on_drop(false);
            sink
        });
        let mut output = Self {
            sink,
            programs: HashMap::new(),
            last_played: HashMap::new(),
        };
        output.replace_programs(voices);
        output
    }

    fn replace_programs(&mut self, voices: &[SynthVoice]) {
        self.programs.clear();
        self.last_played.clear();
        for voice in voices {
            self.programs
                .entry(voice.program_id)
                .or_default()
                .push(voice.clone());
        }
    }

    fn play(&mut self, event: AudioEvent) {
        let Some(sink) = &self.sink else { return };
        let Some(voices) = self.programs.get(&event.id) else {
            return;
        };
        let cooldown_ms = voices
            .iter()
            .map(|voice| voice.cooldown_ms)
            .max()
            .unwrap_or_default();
        let now = Instant::now();
        if cooldown_ms > 0
            && self.last_played.get(&event.id).is_some_and(|last| {
                now.duration_since(*last) < Duration::from_millis(cooldown_ms.into())
            })
        {
            return;
        }
        self.last_played.insert(event.id, now);
        let samples = render_synth_program_for_host(voices, event, 48_000);
        if samples.is_empty() {
            return;
        }
        sink.mixer().add(SamplesBuffer::new(
            NonZeroU16::new(1).unwrap(),
            NonZeroU32::new(48_000).unwrap(),
            samples,
        ));
    }
}

const DECIMAL_SCALE: i64 = 1_000_000;

/// Converts the frontplane's float audio ABI into the decimal representation;
/// this is an ingress boundary, never part of synthesis or game state.
fn audio_scalar_from_host(value: f32) -> i64 {
    if value.is_finite() {
        (value * DECIMAL_SCALE as f32).round() as i64
    } else {
        0
    }
}

/// Converts completed decimal PCM samples to rodio's required host sample
/// representation only after every oscillator, envelope, and filter is done.
fn audio_samples_to_host(samples: Vec<i32>) -> Vec<f32> {
    samples
        .into_iter()
        .map(|sample| sample as f32 / DECIMAL_SCALE as f32)
        .collect()
}

fn render_synth_program_for_host(
    voices: &[SynthVoice],
    event: AudioEvent,
    sample_rate: u32,
) -> Vec<f32> {
    audio_samples_to_host(render_synth_program_fixed(
        voices,
        event.id,
        audio_scalar_from_host(event.volume),
        audio_scalar_from_host(event.pitch),
        sample_rate,
    ))
}

// FIXED_AUDIO_BEGIN
fn fixed_mul(left: i64, right: i64) -> i64 {
    ((left as i128 * right as i128) / DECIMAL_SCALE as i128) as i64
}

fn interpolate_integer(start: i64, end: i64, index: usize, count: usize) -> i64 {
    if count == 0 {
        return start;
    }
    (start as i128 + (end as i128 - start as i128) * index as i128 / count as i128) as i64
}

fn three_point_integer(start: i64, middle: i64, end: i64, index: usize, count: usize) -> i64 {
    if index.saturating_mul(2) < count {
        interpolate_integer(start, middle, index.saturating_mul(2), count)
    } else {
        interpolate_integer(middle, end, index.saturating_mul(2) - count, count)
    }
}

/// Approximates a sinusoid from a decimal microturn phase using a corrected
/// parabola, preserving exact zeroes and extrema without a float lookup table.
fn fixed_sine(phase: i64) -> i64 {
    let phase = phase.rem_euclid(DECIMAL_SCALE);
    let (half_phase, sign) = if phase < DECIMAL_SCALE / 2 {
        (phase * 2, 1)
    } else {
        ((phase - DECIMAL_SCALE / 2) * 2, -1)
    };
    let parabola = (4_i128 * half_phase as i128 * (DECIMAL_SCALE - half_phase) as i128
        / DECIMAL_SCALE as i128) as i64;
    let corrected = parabola + fixed_mul(225_000, fixed_mul(parabola, parabola) - parabola);
    corrected * sign
}

fn noise_sample(random: u32) -> i64 {
    ((random as i128 - 2_147_483_648_i128) * DECIMAL_SCALE as i128 / 2_147_483_648_i128) as i64
}

fn frequency_phase_step(frequency_millihz: i64, sample_rate: u32) -> i64 {
    (frequency_millihz as i128 * 1_000 / sample_rate.max(1) as i128) as i64
}

fn cutoff_omega(cutoff_millihz: i64, sample_rate: u32) -> i64 {
    let maximum = sample_rate as i64 * 450;
    let cutoff = cutoff_millihz.clamp(1, maximum);
    (6_283_185_i128 * cutoff as i128 / (sample_rate.max(1) as i128 * 1_000)) as i64
}

fn low_pass_coefficient(cutoff_millihz: i64, sample_rate: u32) -> i64 {
    let omega = cutoff_omega(cutoff_millihz, sample_rate);
    (omega as i128 * DECIMAL_SCALE as i128 / (DECIMAL_SCALE + omega) as i128) as i64
}

/// Renders all guest-declared synthesis with signed decimal millionths. PCM
/// conversion happens later at the rodio boundary and cannot feed back here.
fn render_synth_program_fixed(
    voices: &[SynthVoice],
    event_id: u32,
    event_volume: i64,
    event_pitch: i64,
    sample_rate: u32,
) -> Vec<i32> {
    let total_ms = voices
        .iter()
        .map(|voice| voice.delay_ms + voice.duration_ms)
        .max()
        .unwrap_or_default();
    let mut output = vec![0_i64; total_ms as usize * sample_rate as usize / 1_000];

    for (voice_index, voice) in voices.iter().enumerate() {
        let start_sample = voice.delay_ms as usize * sample_rate as usize / 1_000;
        let sample_count = voice.duration_ms as usize * sample_rate as usize / 1_000;
        let mut phase = 0_i64;
        let mut random = event_id ^ (voice_index as u32 + 1).wrapping_mul(0x9e37_79b9);
        let mut brown = 0_i64;
        let mut low = 0_i64;
        let mut band = 0_i64;

        for local_index in 0..sample_count {
            let frequency = fixed_mul(
                three_point_integer(
                    voice.frequency_start_millihz.into(),
                    voice.frequency_mid_millihz.into(),
                    voice.frequency_end_millihz.into(),
                    local_index,
                    sample_count,
                ),
                event_pitch,
            );
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            let white = noise_sample(random);
            brown = fixed_mul(brown + fixed_mul(white, 20_000), 980_392)
                .clamp(-DECIMAL_SCALE, DECIMAL_SCALE);
            let raw = match voice.waveform {
                SynthWaveform::Sine => fixed_sine(phase),
                SynthWaveform::Saw => phase * 2 - DECIMAL_SCALE,
                SynthWaveform::WhiteNoise => white,
                SynthWaveform::BrownNoise => fixed_mul(brown, 3_500_000),
            };
            phase =
                (phase + frequency_phase_step(frequency, sample_rate)).rem_euclid(DECIMAL_SCALE);

            let cutoff = interpolate_integer(
                voice.filter_start_millihz.max(1).into(),
                voice.filter_end_millihz.max(1).into(),
                local_index,
                sample_count,
            );
            let filtered = match voice.filter {
                SynthFilter::None => raw,
                SynthFilter::LowPass => {
                    let alpha = low_pass_coefficient(cutoff, sample_rate);
                    low += fixed_mul(alpha, raw - low);
                    low
                }
                SynthFilter::BandPass => {
                    let coefficient = cutoff_omega(cutoff, sample_rate).clamp(0, 990_000);
                    let high = raw - low - fixed_mul(800_000, band);
                    band += fixed_mul(coefficient, high);
                    low += fixed_mul(coefficient, band);
                    band
                }
            };
            let attack_count = (sample_count / 20).max(1);
            let gain = if local_index < attack_count {
                interpolate_integer(
                    voice.gain_start_ppm.into(),
                    voice.gain_peak_ppm.into(),
                    local_index,
                    attack_count,
                )
            } else {
                interpolate_integer(
                    voice.gain_peak_ppm.into(),
                    voice.gain_end_ppm.into(),
                    local_index - attack_count,
                    sample_count.saturating_sub(attack_count),
                )
            };
            output[start_sample + local_index] +=
                fixed_mul(fixed_mul(filtered, gain), event_volume);
        }
    }

    output
        .into_iter()
        .map(|sample| sample.clamp(-DECIMAL_SCALE, DECIMAL_SCALE) as i32)
        .collect()
}
// FIXED_AUDIO_END

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
    plugin_init: PluginInit,
}

struct FrontplaneView {
    frontplane: Frontplane,
    frame: FrameOutput,
    title: String,
    new_label: Option<String>,
    help_label: Option<String>,
    quit_label: Option<String>,
    focus_handle: FocusHandle,
    window_handle: AnyWindowHandle,
    audio: AudioOutput,
    source: Option<ExternalSource>,
    fatal_error: Option<String>,
    reload_error: Option<String>,
    reload_notice: Option<&'static str>,
    viewport: ViewportTracker,
    plugin_init: PluginInit,
}

#[derive(Default)]
struct ViewportTracker {
    last_bits: Option<(u32, u32)>,
}

impl ViewportTracker {
    /// Coalesces repeated layout passes while preserving fractional logical
    /// pixel sizes exactly at the GPUI-to-guest boundary.
    fn observe(&mut self, width: f32, height: f32) -> Option<(f32, f32)> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        let bits = (width.to_bits(), height.to_bits());
        if self.last_bits == Some(bits) {
            return None;
        }
        self.last_bits = Some(bits);
        Some((width, height))
    }

    fn invalidate(&mut self) {
        self.last_bits = None;
    }
}

impl FrontplaneView {
    fn new(startup: Startup, window: &Window, cx: &mut Context<Self>) -> Self {
        let Startup {
            frontplane,
            frame,
            source,
            reload_error,
            plugin_init,
        } = startup;
        let metadata = frontplane.metadata().clone();
        let entries = standard_menu_entries(&metadata);
        let view = Self {
            frontplane,
            frame,
            title: metadata.title.clone(),
            new_label: entries.iter().find_map(|entry| match entry {
                StandardMenuEntry::New(label) => Some(label.clone()),
                _ => None,
            }),
            help_label: entries.iter().find_map(|entry| match entry {
                StandardMenuEntry::Help(label) => Some(label.clone()),
                _ => None,
            }),
            quit_label: entries.iter().find_map(|entry| match entry {
                StandardMenuEntry::Quit(label) => Some(label.clone()),
                _ => None,
            }),
            focus_handle: cx.focus_handle(),
            window_handle: window.window_handle(),
            audio: AudioOutput::new(&metadata.synth_voices),
            source,
            fatal_error: None,
            reload_error,
            reload_notice: None,
            viewport: ViewportTracker::default(),
            plugin_init,
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
            self.plugin_init,
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
                self.help_label = entries.iter().find_map(|entry| match entry {
                    StandardMenuEntry::Help(label) => Some(label.clone()),
                    _ => None,
                });
                self.quit_label = entries.iter().find_map(|entry| match entry {
                    StandardMenuEntry::Quit(label) => Some(label.clone()),
                    _ => None,
                });
                self.audio.replace_programs(&metadata.synth_voices);
                self.fatal_error = None;
                self.reload_error = None;
                self.reload_notice = Some(match prepared.state_transfer {
                    StateTransfer::Preserved => EN.reload_preserved,
                    StateTransfer::Restarted => EN.reload_restarted,
                });
                self.viewport.invalidate();
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

    fn help_controls(&mut self, _: &HelpControls, _: &mut Window, cx: &mut Context<Self>) {
        if let Err(error) = self
            .frontplane
            .event(Event::MenuAction(7))
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
        if event.is_held
            && matches!(
                key,
                Key::Pause
                    | Key::Restart
                    | Key::AutoFire
                    | Key::KidMode
                    | Key::DeathBlossom
                    | Key::Help
            )
        {
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        if let Some((width, height)) = self
            .viewport
            .observe(f32::from(viewport.width), f32::from(viewport.height))
        {
            if let Err(error) = self
                .frontplane
                .event(Event::Viewport { width, height })
                .and_then(|_| self.frontplane.render().map(|frame| self.frame = frame))
            {
                self.fatal_error = Some(error.to_string());
            }
        }
        let frame = self.frame.clone();
        let fatal_error = self.fatal_error.clone();
        let reload_error = self.reload_error.clone();
        let title = self.title.clone();
        let new_label = self.new_label.clone();
        let help_label = self.help_label.clone();
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
        div()
            .relative()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            .on_action(cx.listener(Self::new_game))
            .on_action(cx.listener(Self::help_controls))
            .on_action(cx.listener(Self::reload_plugin))
            .on_action(cx.listener(Self::quit))
            .child(
                div()
                    .id("frontplane-canvas")
                    .absolute()
                    .inset_0()
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
                TitleBar::new().absolute().top_0().left_0().right_0().child(
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
                                .when_some(help_label, |this, label| {
                                    this.child(Button::new("help-controls").label(label).on_click(
                                        cx.listener(|this, _, window, cx| {
                                            this.help_controls(&HelpControls, window, cx)
                                        }),
                                    ))
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
                h_flex()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .px_3()
                    .py_1()
                    .justify_between()
                    .text_xs()
                    .bg(rgba(0x080b12cc))
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
        "p" | "escape" => Some(Key::Pause),
        "r" => Some(Key::Restart),
        "f" => Some(Key::AutoFire),
        "k" => Some(Key::KidMode),
        "b" => Some(Key::DeathBlossom),
        "h" => Some(Key::Help),
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
    let scale = 1.0;
    let viewport = viewport_transform(bounds);
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

/// Maps guest coordinates directly into the complete canvas instead of
/// imposing a fixed-aspect logical viewport and letterboxing unused pixels.
fn viewport_transform(bounds: Bounds<gpui::Pixels>) -> Affine {
    Affine {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        tx: f32::from(bounds.origin.x),
        ty: f32::from(bounds.origin.y),
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
fn load_demo(plugin_init: PluginInit) -> (Frontplane, FrameOutput) {
    load_frontplane(DEMO_WAT, plugin_init).expect("bundled WAT passed the headless ABI suite")
}

fn load_frontplane(
    source: &str,
    plugin_init: PluginInit,
) -> Result<(Frontplane, FrameOutput), gpui_wasm::FrontplaneError> {
    Frontplane::from_wat(source, Limits::default()).and_then(|mut frontplane| {
        frontplane.configure()?;
        frontplane.init(
            plugin_init.seed,
            plugin_init.viewport_width,
            plugin_init.viewport_height,
        )?;
        let frame = frontplane.render()?;
        frontplane.drain_audio();
        frontplane.drain_effects();
        Ok((frontplane, frame))
    })
}

fn startup(source: PluginSource, watch: bool, seed: Option<u64>) -> Startup {
    let plugin_init = PluginInit::new(
        seed.unwrap_or(DEFAULT_PLUGIN_INIT.seed),
        DEFAULT_PLUGIN_INIT.viewport_width,
        DEFAULT_PLUGIN_INIT.viewport_height,
    );
    match source {
        PluginSource::Embedded => {
            let (frontplane, frame) = load_demo(plugin_init);
            Startup {
                frontplane,
                frame,
                source: None,
                reload_error: None,
                plugin_init,
            }
        }
        PluginSource::File(path) => {
            let revision = FileRevision::read(&path);
            let mut revisions = RevisionTracker::default();
            revisions.observe(&revision);
            let result = source_text(&path, &revision).and_then(|source| {
                load_frontplane(&source, plugin_init)
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
                    plugin_init,
                },
                Err(error) => {
                    let (frontplane, frame) = load_demo(plugin_init);
                    Startup {
                        frontplane,
                        frame,
                        source: Some(ExternalSource {
                            path,
                            watch,
                            revisions,
                        }),
                        reload_error: Some(error),
                        plugin_init,
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
            KeyBinding::new("f1", HelpControls, None),
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
        LaunchAction::Run {
            source,
            watch,
            seed,
        } => run_application(startup(source, watch, seed)),
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{
        DECIMAL_SCALE, StandardMenuEntry, ViewportTracker, fixed_sine, map_key,
        render_synth_program_fixed, standard_menu_entries, viewport_transform,
    };
    use gpui::{Bounds, point, px, size};
    use gpui_wasm::{
        Key, MenuItem as PluginMenuItem, Metadata, SynthFilter, SynthVoice, SynthWaveform,
    };

    #[test]
    fn plugin_metadata_drives_only_recognized_standard_native_actions() {
        let metadata = Metadata {
            title: "Example".into(),
            menu_items: vec![
                PluginMenuItem::action(1, "Begin", Some("Ctrl+N")),
                PluginMenuItem::separator(),
                PluginMenuItem::action(7, "Instructions", Some("F1")),
                PluginMenuItem::action(6, "Leave", Some("Ctrl+Q")),
                PluginMenuItem::action(1024, "Plugin-specific", None),
            ],
            synth_voices: Vec::new(),
        };

        assert_eq!(
            standard_menu_entries(&metadata),
            vec![
                StandardMenuEntry::New("Begin".into()),
                StandardMenuEntry::Separator,
                StandardMenuEntry::Help("Instructions".into()),
                StandardMenuEntry::Quit("Leave".into()),
            ]
        );
    }

    #[test]
    fn viewport_projection_uses_every_drawable_pixel_without_letterboxing() {
        let bounds = Bounds::new(point(px(11.0), px(17.0)), size(px(1600.0), px(900.0)));

        assert_eq!(
            viewport_transform(bounds),
            gpui_wasm::Affine {
                m11: 1.0,
                m12: 0.0,
                m21: 0.0,
                m22: 1.0,
                tx: 11.0,
                ty: 17.0,
            }
        );
    }

    #[test]
    fn viewport_updates_emit_once_per_distinct_valid_size() {
        let mut tracker = ViewportTracker::default();

        assert_eq!(tracker.observe(1600.5, 900.25), Some((1600.5, 900.25)));
        assert_eq!(tracker.observe(1600.5, 900.25), None);
        assert_eq!(tracker.observe(1700.0, 900.25), Some((1700.0, 900.25)));
        assert_eq!(tracker.observe(0.0, 900.0), None);
        assert_eq!(tracker.observe(f32::NAN, 900.0), None);
    }

    #[test]
    fn native_keys_cover_every_desktop_vibesteroids_control() {
        assert_eq!(map_key("escape"), Some(Key::Pause));
        assert_eq!(map_key("f"), Some(Key::AutoFire));
        assert_eq!(map_key("k"), Some(Key::KidMode));
        assert_eq!(map_key("b"), Some(Key::DeathBlossom));
        assert_eq!(map_key("h"), Some(Key::Help));
    }

    #[test]
    fn declared_synth_rendering_is_deterministic_scheduled_and_bounded() {
        let voices = vec![SynthVoice {
            program_id: 9,
            waveform: SynthWaveform::WhiteNoise,
            delay_ms: 10,
            duration_ms: 90,
            frequency_start_millihz: 0,
            frequency_mid_millihz: 0,
            frequency_end_millihz: 0,
            gain_start_ppm: 300_000,
            gain_peak_ppm: 300_000,
            gain_end_ppm: 10_000,
            filter: SynthFilter::LowPass,
            filter_start_millihz: 1_500_000,
            filter_end_millihz: 80_000,
            cooldown_ms: 0,
        }];
        let first = render_synth_program_fixed(&voices, 9, DECIMAL_SCALE, DECIMAL_SCALE, 48_000);
        let second = render_synth_program_fixed(&voices, 9, DECIMAL_SCALE, DECIMAL_SCALE, 48_000);
        assert_eq!(first, second);
        assert_eq!(first.len(), 4_800);
        assert!(first[..480].iter().all(|sample| *sample == 0));
        assert!(first[480..].iter().any(|sample| *sample != 0));
        assert!(
            first
                .iter()
                .all(|sample| (-1_000_000..=1_000_000).contains(sample))
        );
    }

    #[test]
    fn decimal_sine_has_exact_cardinal_points() {
        assert_eq!(fixed_sine(0), 0);
        assert_eq!(fixed_sine(250_000), DECIMAL_SCALE);
        assert_eq!(fixed_sine(500_000), 0);
        assert_eq!(fixed_sine(750_000), -DECIMAL_SCALE);
        assert_eq!(fixed_sine(1_000_000), 0);
    }
}
