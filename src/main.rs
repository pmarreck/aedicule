use std::{
    collections::HashMap,
    env,
    num::{NonZeroU16, NonZeroU32},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use gpui::{
    AnyWindowHandle, App, AppContext as _, Context, Entity, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, KeyDownEvent, KeyUpEvent, Menu, MenuItem,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Render,
    ScrollDelta, ScrollWheelEvent, Styled as _, Subscription, Window, WindowBounds, WindowOptions,
    actions, canvas, div, prelude::FluentBuilder as _, px, rgba, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Selectable as _, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants as _},
    h_flex,
    slider::{Slider, SliderEvent, SliderState},
};
use gpui_wasm::{
    AudioEvent, ControlLabelPlacement, ControlPhase, DEFAULT_PLUGIN_ENV, Event, FALLBACK_WAT,
    FileRevision, FrameOutput, Frontplane, HostEffect, Key, LaunchAction, Limits, Metadata,
    PluginInit, PluginSource, PointerButton, PointerScrollUnit, Rect, RevisionTracker,
    SimulationCall, SimulationScheduler, SliderControl, StateTransfer, SynthFilter, SynthVoice,
    SynthWaveform, WatRejectionStage, display_refresh_rate_from_environment,
    format_wat_rejection_diagnostic, initialize_frontplane, prepare_reload, resolve_launch,
};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, buffer::SamplesBuffer};
use smol::Timer;

actions!(gpui_wasm, [NewApplication, ShowHelp, ReloadPlugin, Quit]);

const LOGICAL_WIDTH: f32 = 1024.0;
const LOGICAL_HEIGHT: f32 = 768.0;
const HOST_TITLE_BAR_HEIGHT: f32 = 34.0;
const TITLE_BAR_GLYPH_RGBA: u32 = 0xf6f7f9ff;
const WATCH_INTERVAL: Duration = Duration::from_millis(250);
const MAX_TICKS_PER_WAKE: u32 = 8;
const DEFAULT_PLUGIN_INIT: PluginInit = PluginInit::new(0x5eed_cafe, LOGICAL_WIDTH, LOGICAL_HEIGHT);

struct Strings {
    fallback_title: &'static str,
    application_menu: &'static str,
    abi_status: &'static str,
    abi_version: &'static str,
    hertz: &'static str,
    capability_bounded: &'static str,
    dropped_ticks: &'static str,
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
    abi_status: "WAT owns behavior • GPUI owns the window",
    abi_version: "ABI v0",
    hertz: "Hz",
    capability_bounded: "capability-bounded",
    dropped_ticks: "dropped ticks",
    plugin_stopped: "WAT plugin stopped safely",
    reload: "Reload",
    reload_failed: "Reload failed; previous plugin remains active",
    reload_preserved: "Reloaded with live state preserved",
    reload_restarted: "Reloaded with fresh state (schema changed)",
    embedded_source: "embedded conformance fallback",
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
  --embedded    Force the stable ABI conformance fallback
  --seed N      Set the deterministic unsigned 64-bit application seed
  -h, --help    Show this help
  --about       Show version and build platform

Without arguments, ./code.wat is loaded when present, followed by any packaged
default plugin, then the embedded fallback. An application directory resolves
to its code.wat file. Set AE_DISPLAY_REFRESH_RATE to an exact initial display
rate such as 60000/1001; canonical 59.94, 29.97, 23.976, and 119.88 are also
accepted.
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

/// Resolves a generic guest action to its configure-time accessible label so
/// native widgets never invent or duplicate application text.
fn menu_action_label(metadata: &Metadata, action_id: u32) -> Option<&str> {
    metadata
        .menu_items
        .iter()
        .find(|item| item.id == Some(action_id))
        .map(|item| item.label.as_str())
}

/// Maps a native guest-authored button click onto the same ordered action
/// event used by menus, preserving one application-independent input path.
fn menu_action_event(action_id: u32) -> Event {
    Event::MenuAction(action_id)
}

fn native_menus(metadata: &Metadata, can_reload: bool) -> Vec<Menu> {
    let mut items: Vec<_> = standard_menu_entries(metadata)
        .into_iter()
        .map(|entry| match entry {
            StandardMenuEntry::New(label) => MenuItem::action(label, NewApplication),
            StandardMenuEntry::Help(label) => MenuItem::action(label, ShowHelp),
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
    monotonic_origin: Instant,
    scheduler: SimulationScheduler,
    timing_generation: u64,
    sliders: Vec<NativeSlider>,
    _slider_subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct NativeSlider {
    control: SliderControl,
    state: Entity<SliderState>,
    synced_ui_revision: Option<u32>,
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
    /// Adapts exact integer control lattices to GPUI step indices, translating
    /// every emitted index back through `min + index * step` before scheduling.
    fn build_sliders(
        controls: &[SliderControl],
        cx: &mut Context<Self>,
    ) -> (Vec<NativeSlider>, Vec<Subscription>) {
        let mut sliders = Vec::with_capacity(controls.len());
        let mut subscriptions = Vec::with_capacity(controls.len());
        for control in controls {
            let initial_index =
                (i64::from(control.initial) - i64::from(control.min)) / i64::from(control.step);
            let state = cx.new(|_| {
                SliderState::new()
                    .min(0.0)
                    .max(control.step_count() as f32)
                    .step(1.0)
                    .default_value(initial_index as f32)
            });
            let subscribed_control = control.clone();
            subscriptions.push(
                cx.subscribe(&state, move |this, _, event: &SliderEvent, cx| {
                    let (slider_value, phase) = match event {
                        SliderEvent::Change(value) => (*value, ControlPhase::Change),
                        SliderEvent::Release(value) => (*value, ControlPhase::Release),
                    };
                    let index = slider_value.end().round();
                    if !index.is_finite() || index < 0.0 {
                        return;
                    }
                    let index = index as u32;
                    if let Some(value) = subscribed_control.value_at_step(index) {
                        if let Some(slider) = this
                            .sliders
                            .iter_mut()
                            .find(|slider| slider.control.id == subscribed_control.id)
                        {
                            slider.synced_ui_revision = None;
                        }
                        this.queue_native_event(
                            Event::Control {
                                id: subscribed_control.id,
                                value,
                                phase,
                            },
                            cx,
                        );
                    }
                }),
            );
            sliders.push(NativeSlider {
                control: control.clone(),
                state,
                synced_ui_revision: None,
            });
        }
        (sliders, subscriptions)
    }

    fn new(startup: Startup, window: &Window, cx: &mut Context<Self>) -> Self {
        let Startup {
            frontplane,
            frame,
            source,
            reload_error,
            plugin_init,
        } = startup;
        let metadata = frontplane.metadata().clone();
        let monotonic_origin = Instant::now();
        let scheduler = SimulationScheduler::new(
            frontplane.simulation_rate(),
            MAX_TICKS_PER_WAKE,
            Duration::ZERO,
        );
        let entries = standard_menu_entries(&metadata);
        let (sliders, slider_subscriptions) = Self::build_sliders(&metadata.controls, cx);
        let mut view = Self {
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
            monotonic_origin,
            scheduler,
            timing_generation: 0,
            sliders,
            _slider_subscriptions: slider_subscriptions,
        };

        view.spawn_timing_loop(cx);
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

    /// Runs a single cancellable-by-generation timer loop whose delay is
    /// recomputed from the scheduler's next absolute rational boundary.
    fn spawn_timing_loop(&mut self, cx: &mut Context<Self>) {
        let generation = self.timing_generation;
        cx.spawn(async move |this, cx| {
            loop {
                let delay = match this.update(cx, |this, _| {
                    (this.timing_generation == generation).then(|| {
                        this.scheduler
                            .time_until_next_wake(this.monotonic_origin.elapsed())
                    })
                }) {
                    Ok(Some(delay)) => delay,
                    Ok(None) | Err(_) => break,
                };
                Timer::after(delay).await;
                let keep_running = match this.update(cx, |this, cx| {
                    if this.timing_generation != generation {
                        return false;
                    }
                    let now = this.monotonic_origin.elapsed();
                    if this.pump_simulation(now, cx) {
                        cx.notify();
                    }
                    true
                }) {
                    Ok(keep_running) => keep_running,
                    Err(_) => false,
                };
                if !keep_running {
                    break;
                }
            }
        })
        .detach();
    }

    /// Restarts the rational timeline at the candidate swap instant so a
    /// changed guest-declared frequency cannot inherit an obsolete wake.
    fn restart_timing_loop(&mut self, cx: &mut Context<Self>) {
        let now = self.monotonic_origin.elapsed();
        self.scheduler =
            SimulationScheduler::new(self.frontplane.simulation_rate(), MAX_TICKS_PER_WAKE, now);
        self.timing_generation = self.timing_generation.wrapping_add(1);
        self.spawn_timing_loop(cx);
    }

    /// Publishes one fully validated guest canvas transaction; native UI has
    /// its own guest-authored revision lifecycle inside the frontplane.
    fn accept_frame(&mut self, frame: FrameOutput) {
        self.frame = frame;
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
                self.accept_frame(prepared.frame);
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
                let (sliders, subscriptions) = Self::build_sliders(&metadata.controls, cx);
                self.sliders = sliders;
                self._slider_subscriptions = subscriptions;
                self.restart_timing_loop(cx);
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
        let rendered = match detail {
            Some(detail) => format!("{message}: {}: {detail}", path.display()),
            None => format!("{message}: {}", path.display()),
        };
        eprintln!(
            "{}",
            format_wat_rejection_diagnostic(
                path,
                WatRejectionStage::Reload,
                detail.unwrap_or(message),
            )
        );
        self.reload_error = Some(rendered);
    }

    /// Executes one pure scheduler plan, renders at most once, and drains
    /// semantic effects only after Wasmtime releases its store borrow.
    fn pump_simulation(&mut self, now: Duration, cx: &mut Context<Self>) -> bool {
        let pump = self.scheduler.pump(now);
        if self.fatal_error.is_some() {
            return pump.dropped_ticks > 0;
        }
        for call in pump.calls {
            let result = match call {
                SimulationCall::Event(event) => self.frontplane.event(event),
                SimulationCall::Tick(ticks) => self.frontplane.tick(ticks),
            };
            if let Err(error) = result {
                self.fatal_error = Some(error.to_string());
                return true;
            }
        }
        if pump.render {
            match self.frontplane.render() {
                Ok(frame) => self.accept_frame(frame),
                Err(error) => {
                    self.fatal_error = Some(error.to_string());
                    return true;
                }
            }
        }
        for event in self.frontplane.drain_audio() {
            self.audio.play(event);
        }
        for effect in self.frontplane.drain_effects() {
            if effect == HostEffect::Quit {
                cx.quit();
            }
        }
        pump.render || pump.dropped_ticks > 0
    }

    /// Stamps native input in the scheduler's monotonic clock domain before
    /// pumping, preventing a late wake from applying it to an overdue tick.
    fn queue_native_event(&mut self, event: Event, cx: &mut Context<Self>) {
        let now = self.monotonic_origin.elapsed();
        self.scheduler.queue_event(now, event);
        if self.pump_simulation(now, cx) {
            cx.notify();
        }
    }

    fn new_application(&mut self, _: &NewApplication, _: &mut Window, cx: &mut Context<Self>) {
        self.fatal_error = None;
        self.queue_native_event(Event::MenuAction(1), cx);
    }

    fn show_help(&mut self, _: &ShowHelp, _: &mut Window, cx: &mut Context<Self>) {
        self.queue_native_event(Event::MenuAction(7), cx);
    }

    fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = map_key(&event.keystroke.key) else {
            return;
        };
        if event.is_held {
            return;
        }
        self.queue_native_event(Event::KeyDown(key), cx);
    }

    fn key_up(&mut self, event: &KeyUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(key) = map_key(&event.keystroke.key) else {
            return;
        };
        self.queue_native_event(Event::KeyUp(key), cx);
    }

    /// Forwards canvas-relative GPUI mouse motion through the scheduler so a
    /// guest observes it before the next exact fixed-step boundary.
    fn pointer_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.queue_native_event(
            Event::PointerMove {
                x: f32::from(event.position.x),
                y: f32::from(event.position.y),
            },
            cx,
        );
    }

    /// Converts a GPUI press into a shared pointer edge before its next exact
    /// simulation boundary, keeping native platform button names out of WAT.
    fn pointer_down(
        &mut self,
        button: PointerButton,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.queue_native_event(
            button.down(f32::from(event.position.x), f32::from(event.position.y)),
            cx,
        );
    }

    /// Pairs a native pointer release with the same stable button ID so WAT
    /// guests cannot retain a stuck fire or thrust state after a click ends.
    fn pointer_up(
        &mut self,
        button: PointerButton,
        event: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.queue_native_event(
            button.up(f32::from(event.position.x), f32::from(event.position.y)),
            cx,
        );
    }

    /// Preserves GPUI's horizontal/vertical scroll axes and native unit kind,
    /// then stamps the non-zero movement in the scheduler's input timeline.
    fn pointer_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let event = match event.delta {
            ScrollDelta::Lines(delta) => PointerScrollUnit::Lines.event(delta.x, delta.y),
            ScrollDelta::Pixels(delta) => {
                PointerScrollUnit::LogicalPixels.event(f32::from(delta.x), f32::from(delta.y))
            }
        };
        if let Some(event) = event {
            self.queue_native_event(event, cx);
        }
    }
}

impl Focusable for FrontplaneView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Keeps the desktop host title bar in one explicit layer so its hit-routing
/// policy can be tested independently from WAT execution and audio setup.
fn host_title_bar_layer() -> gpui::Div {
    div().absolute().top_0().left_0().right_0().occlude()
}

/// Supplies an asset-independent fallback for GPUI Component's invisible
/// title-bar SVGs while its underlying native control hit regions stay active.
fn title_bar_control_glyphs(maximized: bool) -> [&'static str; 3] {
    ["−", if maximized { "❐" } else { "□" }, "×"]
}

/// Paints non-interactive high-contrast glyphs over the adapter's existing
/// minimize/maximize/close controls without taking over their event handling.
fn title_bar_control_glyph_overlay(maximized: bool) -> gpui::Div {
    if cfg!(target_os = "macos") || cfg!(target_family = "wasm") {
        return div();
    }
    h_flex()
        .absolute()
        .top_0()
        .right_0()
        .h(px(HOST_TITLE_BAR_HEIGHT))
        .text_color(rgba(TITLE_BAR_GLYPH_RGBA))
        .children(
            title_bar_control_glyphs(maximized)
                .into_iter()
                .map(|glyph| {
                    div()
                        .flex()
                        .h_full()
                        .w(px(HOST_TITLE_BAR_HEIGHT))
                        .items_center()
                        .justify_center()
                        .text_lg()
                        .child(glyph)
                }),
        )
}

/// Claims pointer ownership for host-provided guest controls so manipulating a
/// slider cannot simultaneously enqueue a pointer edge on the WAT canvas.
fn host_control_layer() -> gpui::Div {
    div().occlude()
}

/// Converts one validated guest-authored control rectangle into an occluding
/// GPUI layer without applying a host width cap or positional policy.
fn guest_positioned_control_layer(bounds: Rect) -> gpui::Div {
    host_control_layer()
        .absolute()
        .left(px(bounds.x))
        .top(px(bounds.y))
        .w(px(bounds.width))
        .h(px(bounds.height))
}

impl Render for FrontplaneView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        if let Some((width, height)) = self
            .viewport
            .observe(f32::from(viewport.width), f32::from(viewport.height))
        {
            self.queue_native_event(Event::Viewport { width, height }, cx);
        }
        let frame = self.frame.clone();
        let ui = self.frontplane.ui_snapshot().cloned();
        let fatal_error = self.fatal_error.clone();
        let reload_error = self.reload_error.clone();
        let title = self.title.clone();
        let new_label = self.new_label.clone();
        let help_label = self.help_label.clone();
        let quit_label = self.quit_label.clone();
        let control_panels = ui
            .iter()
            .flat_map(|snapshot| snapshot.control_panels.iter())
            .map(|panel| {
                guest_positioned_control_layer(panel.bounds)
                    .id(("control-panel", panel.id as usize))
                    .rounded_lg()
                    .bg(rgba(panel.rgba))
            })
            .collect::<Vec<_>>();
        let mut slider_layers =
            Vec::with_capacity(ui.as_ref().map_or(0, |snapshot| snapshot.sliders.len()));
        for placement in ui.iter().flat_map(|snapshot| snapshot.sliders.iter()) {
            let Some(slider) = self
                .sliders
                .iter_mut()
                .find(|slider| slider.control.id == placement.id)
            else {
                continue;
            };
            let ui_revision = ui
                .as_ref()
                .expect("slider placement belongs to an accepted UI snapshot")
                .revision;
            if slider.synced_ui_revision != Some(ui_revision) {
                let index = slider
                    .control
                    .step_index(placement.value)
                    .expect("validated slider placement stays on its declared lattice");
                slider.synced_ui_revision = Some(ui_revision);
                slider.state.update(cx, |state, slider_cx| {
                    state.set_value(index as f32, window, slider_cx);
                });
            }
            let track = div().w_full().child(Slider::new(&slider.state));
            let content = match placement.label_placement {
                ControlLabelPlacement::Hidden => div().size_full().flex().child(track),
                ControlLabelPlacement::Above => div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .w_full()
                            .text_sm()
                            .child(format!("{}: {}", slider.control.label, placement.value)),
                    )
                    .child(track),
            };
            slider_layers.push(
                guest_positioned_control_layer(placement.bounds)
                    .id(("native-slider", placement.id as usize))
                    .child(content),
            );
        }
        let button_layers = ui
            .iter()
            .flat_map(|snapshot| snapshot.buttons.iter())
            .map(|placement| {
                let action_id = placement.action_id;
                let label = menu_action_label(self.frontplane.metadata(), action_id)
                    .expect("validated button action remains declared")
                    .to_owned();
                let button = Button::new(("native-button-control", placement.id as usize))
                    .label(label)
                    .selected(placement.selected)
                    .w_full()
                    .h_full()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.queue_native_event(menu_action_event(action_id), cx)
                    }));
                guest_positioned_control_layer(placement.bounds)
                    .id(("native-button", placement.id as usize))
                    .child(button)
            })
            .collect::<Vec<_>>();
        let can_reload = self.source.is_some();
        let source_status = match &self.source {
            Some(source) if source.watch => {
                format!("{} {}", EN.watching, source.path.display())
            }
            Some(source) => format!("{} {}", EN.external_source, source.path.display()),
            None => EN.embedded_source.to_owned(),
        };
        let timing_status = format!(
            "{} • {} {} • {}",
            EN.abi_version,
            self.frontplane.simulation_rate(),
            EN.hertz,
            EN.capability_bounded,
        );
        let timing_status = match self.scheduler.total_dropped_ticks() {
            0 => timing_status,
            dropped => format!("{timing_status} • {dropped} {}", EN.dropped_ticks),
        };
        let runtime_status = match self.reload_notice {
            Some(notice) => format!("{notice} • {timing_status} • {source_status}"),
            None => format!("{timing_status} • {source_status}"),
        };
        div()
            .relative()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            .on_action(cx.listener(Self::new_application))
            .on_action(cx.listener(Self::show_help))
            .on_action(cx.listener(Self::reload_plugin))
            .on_action(cx.listener(Self::quit))
            .child(
                div()
                    .id("frontplane-canvas")
                    .absolute()
                    .inset_0()
                    .bg(rgba(frame.background))
                    .on_mouse_move(cx.listener(Self::pointer_move))
                    .on_scroll_wheel(cx.listener(Self::pointer_scroll))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, window, cx| {
                            this.pointer_down(PointerButton::Primary, event, window, cx);
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseUpEvent, window, cx| {
                            this.pointer_up(PointerButton::Primary, event, window, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseUpEvent, window, cx| {
                            this.pointer_up(PointerButton::Primary, event, window, cx);
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, event: &MouseDownEvent, window, cx| {
                            this.pointer_down(PointerButton::Secondary, event, window, cx);
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Right,
                        cx.listener(|this, event: &MouseUpEvent, window, cx| {
                            this.pointer_up(PointerButton::Secondary, event, window, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Right,
                        cx.listener(|this, event: &MouseUpEvent, window, cx| {
                            this.pointer_up(PointerButton::Secondary, event, window, cx);
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Middle,
                        cx.listener(|this, event: &MouseDownEvent, window, cx| {
                            this.pointer_down(PointerButton::Middle, event, window, cx);
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Middle,
                        cx.listener(|this, event: &MouseUpEvent, window, cx| {
                            this.pointer_up(PointerButton::Middle, event, window, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Middle,
                        cx.listener(|this, event: &MouseUpEvent, window, cx| {
                            this.pointer_up(PointerButton::Middle, event, window, cx);
                        }),
                    )
                    .child(
                        canvas(
                            move |bounds, _, _| (bounds, frame),
                            move |_, (bounds, frame), window, cx| {
                                gpui_wasm::gpui_canvas::paint_frame(&frame, bounds, window, cx)
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
            .children(control_panels)
            .children(slider_layers)
            .children(button_layers)
            .child(
                host_title_bar_layer()
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
                                                Button::new("reload-plugin")
                                                    .label(EN.reload)
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.reload_plugin(
                                                                &ReloadPlugin,
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    )),
                                            )
                                        })
                                        .when_some(new_label, |this, label| {
                                            this.child(
                                                Button::new("new-game")
                                                    .primary()
                                                    .label(label)
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.new_application(
                                                                &NewApplication,
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    )),
                                            )
                                        })
                                        .when_some(help_label, |this, label| {
                                            this.child(
                                                Button::new("help-controls").label(label).on_click(
                                                    cx.listener(|this, _, window, cx| {
                                                        this.show_help(&ShowHelp, window, cx)
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
                    .child(title_bar_control_glyph_overlay(window.is_maximized())),
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

/// Normalizes GPUI's portable key names into versioned physical-key IDs,
/// leaving all application meaning inside the guest.
fn map_key(key: &str) -> Option<Key> {
    match key {
        "left" => Some(Key::ArrowLeft),
        "right" => Some(Key::ArrowRight),
        "up" => Some(Key::ArrowUp),
        "space" | " " => Some(Key::Space),
        "p" => Some(Key::P),
        "escape" => Some(Key::Escape),
        "r" => Some(Key::R),
        "f" => Some(Key::F),
        "k" => Some(Key::K),
        "b" => Some(Key::B),
        "h" => Some(Key::H),
        "f1" => Some(Key::F1),
        _ => None,
    }
}

/// Instantiates the stable conformance fallback through the same lifecycle as
/// external applications when no usable runtime plugin is available.
fn load_fallback(plugin_init: PluginInit) -> (Frontplane, FrameOutput) {
    load_frontplane(FALLBACK_WAT, plugin_init)
        .expect("embedded conformance WAT passed the headless ABI suite")
}

fn load_frontplane(
    source: &str,
    plugin_init: PluginInit,
) -> Result<(Frontplane, FrameOutput), gpui_wasm::FrontplaneError> {
    Frontplane::from_wat(source, Limits::default()).and_then(|mut frontplane| {
        initialize_frontplane(&mut frontplane, plugin_init)?;
        let frame = frontplane.render()?;
        frontplane.drain_audio();
        frontplane.drain_effects();
        Ok((frontplane, frame))
    })
}

fn startup(source: PluginSource, watch: bool, seed: Option<u64>) -> Result<Startup, String> {
    let mut plugin_init = PluginInit::new(
        seed.unwrap_or(DEFAULT_PLUGIN_INIT.seed),
        DEFAULT_PLUGIN_INIT.viewport_width,
        DEFAULT_PLUGIN_INIT.viewport_height,
    );
    if let Some(display_refresh) =
        display_refresh_rate_from_environment().map_err(|error| error.to_string())?
    {
        plugin_init = plugin_init.with_display_refresh(display_refresh);
    }
    Ok(match source {
        PluginSource::Embedded => {
            let (frontplane, frame) = load_fallback(plugin_init);
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
                load_frontplane(&source, plugin_init).map_err(|error| error.to_string())
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
                    let reload_error = format!("{}: {}: {error}", EN.reload_failed, path.display());
                    eprintln!(
                        "{}",
                        format_wat_rejection_diagnostic(
                            &path,
                            WatRejectionStage::InitialLoad,
                            &error,
                        )
                    );
                    let (frontplane, frame) = load_fallback(plugin_init);
                    Startup {
                        frontplane,
                        frame,
                        source: Some(ExternalSource {
                            path,
                            watch,
                            revisions,
                        }),
                        reload_error: Some(reload_error),
                        plugin_init,
                    }
                }
            }
        }
    })
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
            KeyBinding::new("ctrl-n", NewApplication, None),
            KeyBinding::new("f1", ShowHelp, None),
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
    let packaged_default = env::var_os(DEFAULT_PLUGIN_ENV).map(PathBuf::from);
    let action = match resolve_launch(
        env::args_os().skip(1),
        &working_directory,
        packaged_default.as_deref(),
    ) {
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
        } => match startup(source, watch, seed) {
            Ok(startup) => run_application(startup),
            Err(error) => {
                eprintln!("{}: {error}", EN.cli_error);
                return ExitCode::from(2);
            }
        },
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{
        DECIMAL_SCALE, StandardMenuEntry, TITLE_BAR_GLYPH_RGBA, ViewportTracker, fixed_sine,
        guest_positioned_control_layer, host_title_bar_layer, map_key, menu_action_event,
        menu_action_label, render_synth_program_fixed, standard_menu_entries,
        title_bar_control_glyph_overlay, title_bar_control_glyphs,
    };
    use gpui::{
        Bounds, Context, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
        Render, Styled as _, TestApp, Window, div, point, px, size,
    };
    use gpui_component::TitleBar;
    use gpui_wasm::{
        Event, Key, MenuItem as PluginMenuItem, Metadata, SynthFilter, SynthVoice, SynthWaveform,
        gpui_canvas::viewport_transform,
    };

    #[derive(Default)]
    struct HostTitleBarHitProbe {
        guest_pointer_downs: usize,
        host_pointer_downs: usize,
    }

    struct HostControlHitProbe {
        bounds: gpui_wasm::Rect,
        guest_pointer_downs: usize,
        host_pointer_downs: usize,
    }

    impl Default for HostControlHitProbe {
        fn default() -> Self {
            Self {
                bounds: gpui_wasm::Rect {
                    x: 100.0,
                    y: 100.0,
                    width: 200.0,
                    height: 40.0,
                },
                guest_pointer_downs: 0,
                host_pointer_downs: 0,
            }
        }
    }

    impl Render for HostControlHitProbe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .relative()
                .size_full()
                .child(div().absolute().inset_0().on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.guest_pointer_downs += 1),
                ))
                .child(guest_positioned_control_layer(self.bounds).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.host_pointer_downs += 1),
                ))
        }
    }

    impl Render for HostTitleBarHitProbe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .relative()
                .size_full()
                .child(div().absolute().inset_0().on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| this.guest_pointer_downs += 1),
                ))
                .child(
                    host_title_bar_layer()
                        .child(TitleBar::new().child(div().size_full().on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, _| this.host_pointer_downs += 1),
                        )))
                        .child(title_bar_control_glyph_overlay(false)),
                )
        }
    }

    #[test]
    fn host_title_bar_occludes_guest_pointer_edges() {
        let mut app = TestApp::new();
        app.update(gpui_component::init);
        let mut window = app.open_window(|_, _| HostTitleBarHitProbe::default());
        window.draw();

        window.simulate_click(point(px(100.0), px(17.0)), MouseButton::Left);
        window.read(|probe, _| {
            assert_eq!(probe.host_pointer_downs, 1);
            assert_eq!(probe.guest_pointer_downs, 0);
        });

        window.simulate_click(point(px(100.0), px(100.0)), MouseButton::Left);
        window.read(|probe, _| {
            assert_eq!(probe.host_pointer_downs, 1);
            assert_eq!(probe.guest_pointer_downs, 1);
        });
    }

    #[test]
    fn title_bar_control_fallback_has_opaque_distinct_glyphs() {
        assert_eq!(title_bar_control_glyphs(false), ["−", "□", "×"]);
        assert_eq!(title_bar_control_glyphs(true), ["−", "❐", "×"]);
        assert_eq!(TITLE_BAR_GLYPH_RGBA & 0xff, 0xff);
    }

    #[test]
    fn host_control_surface_occludes_guest_pointer_edges() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, _| HostControlHitProbe::default());
        window.draw();

        window.simulate_click(point(px(150.0), px(120.0)), MouseButton::Left);
        window.read(|probe, _| {
            assert_eq!(probe.host_pointer_downs, 1);
            assert_eq!(probe.guest_pointer_downs, 0);
        });

        window.simulate_click(point(px(50.0), px(50.0)), MouseButton::Left);
        window.read(|probe, _| {
            assert_eq!(probe.host_pointer_downs, 1);
            assert_eq!(probe.guest_pointer_downs, 1);
        });
    }

    #[test]
    fn arbitrary_declared_actions_supply_native_button_labels_and_events() {
        let metadata = Metadata {
            menu_items: vec![
                PluginMenuItem::separator(),
                PluginMenuItem::action(42, "Play/Pause", None),
            ],
            ..Metadata::default()
        };

        assert_eq!(menu_action_label(&metadata, 42), Some("Play/Pause"));
        assert_eq!(menu_action_label(&metadata, 99), None);
        assert_eq!(menu_action_event(42), Event::MenuAction(42));
    }

    #[test]
    fn guest_positioned_control_surface_can_span_the_full_viewport_width() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, _| HostControlHitProbe {
            bounds: gpui_wasm::Rect {
                x: 12.0,
                y: 100.0,
                width: 776.0,
                height: 80.0,
            },
            ..HostControlHitProbe::default()
        });
        window.draw();

        window.simulate_click(point(px(780.0), px(140.0)), MouseButton::Left);
        window.read(|probe, _| {
            assert_eq!(probe.host_pointer_downs, 1);
            assert_eq!(probe.guest_pointer_downs, 0);
        });

        window.simulate_click(point(px(4.0), px(140.0)), MouseButton::Left);
        window.read(|probe, _| {
            assert_eq!(probe.host_pointer_downs, 1);
            assert_eq!(probe.guest_pointer_downs, 1);
        });
    }

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
            controls: Vec::new(),
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
    fn native_key_mapping_exposes_physical_keys_without_guest_semantics() {
        assert_eq!(map_key("left"), Some(Key::ArrowLeft));
        assert_eq!(map_key("right"), Some(Key::ArrowRight));
        assert_eq!(map_key("up"), Some(Key::ArrowUp));
        assert_eq!(map_key("space"), Some(Key::Space));
        assert_eq!(map_key("p"), Some(Key::P));
        assert_eq!(map_key("escape"), Some(Key::Escape));
        assert_eq!(map_key("r"), Some(Key::R));
        assert_eq!(map_key("f"), Some(Key::F));
        assert_eq!(map_key("k"), Some(Key::K));
        assert_eq!(map_key("b"), Some(Key::B));
        assert_eq!(map_key("h"), Some(Key::H));
        assert_eq!(map_key("f1"), Some(Key::F1));
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
