use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    env,
    io::Write as _,
    num::{NonZeroU16, NonZeroU32},
    path::{Path, PathBuf},
    process::ExitCode,
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use aedicule::audio::{DECIMAL_SCALE, fixed_sine, render_synth_program_fixed};
use aedicule::{
    AudioEvent, ControlLabelPlacement, ControlPhase, DEFAULT_PLUGIN_ENV, DeviceChangeTracker,
    Event, FALLBACK_WAT,
    FileRevision, FrameOutput, Frontplane, GEIST_MONO_REGULAR, GuestSuspension, HostEffect, Key,
    LaunchAction, Limits, Metadata, PausePhase, PluginInit, PluginSource, PointerButton,
    PointerScrollUnit, Rect, RevisionTracker, SampleAsset, SampleAudioEvent, SimulationCall,
    SimulationScheduler, SliderControl, StateTransfer, SuspensionDisposition, SynthVoice,
    TextField, TextPhase, WatRejectionStage, WebServer,
    audio::{
        render_synth_program as render_synth_program_for_host,
        samples_to_host as audio_samples_to_host,
    },
    depackage_application, discover_web_runtime, display_refresh_rate_from_environment,
    file_url_to_path, format_wat_rejection_diagnostic, initialize_frontplane, package_application,
    prepare_reload_with_assets, read_application_assets, read_application_file, resolve_launch,
    run_application_tests, text_value_events,
};
use gpui::{
    AnyWindowHandle, App, AppContext as _, ClickEvent, Context, Entity, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, KeyDownEvent, KeyUpEvent, Menu, MenuItem,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _,
    PathPromptOptions, PromptLevel, Render, Role, ScrollDelta, ScrollWheelEvent, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, WindowBounds,
    WindowOptions, actions, canvas, div, prelude::FluentBuilder as _, px, rgba, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Selectable as _, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState},
};
use rodio::{
    DeviceSinkBuilder, MixerDeviceSink, Player, Source as _,
    buffer::SamplesBuffer,
    mixer::{Mixer, mixer},
    source::Zero,
};
use smol::Timer;

actions!(
    aedicule,
    [
        AboutAedicule,
        OpenApplication,
        OpenProjectDirectory,
        NewApplication,
        ShowHelp,
        ReloadPlugin,
        Quit
    ]
);

const LOGICAL_WIDTH: f32 = 1024.0;
const LOGICAL_HEIGHT: f32 = 768.0;
const HOST_TITLE_BAR_HEIGHT: f32 = 34.0;
const TITLE_BAR_GLYPH_RGBA: u32 = 0xf6f7f9ff;
const WATCH_INTERVAL: Duration = Duration::from_millis(250);
const MAX_TICKS_PER_WAKE: u32 = 8;
const DEFAULT_PLUGIN_INIT: PluginInit = PluginInit::new(0x5eed_cafe, LOGICAL_WIDTH, LOGICAL_HEIGHT);

struct Strings {
    app_name: &'static str,
    about_app: &'static str,
    about_detail: &'static str,
    file_menu: &'static str,
    open: &'static str,
    open_application: &'static str,
    open_project_directory: &'static str,
    quit_app: &'static str,
    ok: &'static str,
    empty_heading: &'static str,
    empty_detail: &'static str,
    no_application: &'static str,
    file_picker_failed: &'static str,
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
    test_seed: &'static str,
    test_pass: &'static str,
    tests_passed: &'static str,
}

const EN: Strings = Strings {
    app_name: "Aedicule",
    about_app: "About Aedicule",
    about_detail: "A capability-bounded, cross-platform runtime for WAT applications.",
    file_menu: "File",
    open: "Open…",
    open_application: "Open Application…",
    open_project_directory: "Open Project Directory…",
    quit_app: "Quit Aedicule",
    ok: "OK",
    empty_heading: "Open an Aedicule application",
    empty_detail: "Choose a project directory, .aed package, or .wat file to begin.",
    no_application: "no application open",
    file_picker_failed: "Could not open the system file picker",
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
  aedicule [OPTIONS] [PLUGIN.wat|APPLICATION_DIRECTORY|APPLICATION.aed]
  aedicule --package DIRECTORY [OUTPUT.aed]
  aedicule --depackage APPLICATION.aed [OUTPUT_DIRECTORY]
  aedicule --test [--seed N] SOURCE
  aedicule --web SOURCE [--bind ADDRESS] [--port PORT]

Options:
  --watch       Reload after content changes; defaults to ./code.wat
  --embedded    Force the stable ABI conformance fallback
  --package     Create a deterministic .aed from an application directory
  --depackage   Safely expand an .aed into a new directory
  --test        Run direct tests/*.wast suites without opening a window
  --web         Serve an application through the bundled browser runtime
  --bind ADDR   Web listener address; defaults to 127.0.0.1
  --port PORT   Web listener port; defaults to 8080 (0 selects an open port)
  --seed N      Set an application/test seed (decimal or 0x-prefixed)
  -h, --help    Show this help
  --about       Show version and build platform

Without arguments, ./code.wat is loaded when present, followed by any packaged
default application; otherwise Aedicule opens its application chooser. Use
--embedded to run the ABI conformance fallback. An application directory
resolves to its code.wat file. Set AE_DISPLAY_REFRESH_RATE to an exact initial
display rate such as 60000/1001; canonical 59.94, 29.97, 23.976, and 119.88
are also accepted.
",
    about: "generic native GPUI frontplane for capability-bounded WAT applications",
    cli_error: "aedicule",
    test_seed: "test seed",
    test_pass: "PASS",
    tests_passed: "passed",
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

fn native_menus(
    metadata: &Metadata,
    can_reload: bool,
    can_select_mixed_files_and_dirs: bool,
) -> Vec<Menu> {
    let application_items: Vec<_> = standard_menu_entries(metadata)
        .into_iter()
        .map(|entry| match entry {
            StandardMenuEntry::New(label) => MenuItem::action(label, NewApplication),
            StandardMenuEntry::Help(label) => MenuItem::action(label, ShowHelp),
            StandardMenuEntry::Separator => MenuItem::Separator,
            StandardMenuEntry::Quit(label) => MenuItem::action(label, Quit),
        })
        .collect();
    let mut file_items = if can_select_mixed_files_and_dirs {
        vec![MenuItem::action(EN.open, OpenApplication)]
    } else {
        vec![
            MenuItem::action(EN.open_application, OpenApplication),
            MenuItem::action(EN.open_project_directory, OpenProjectDirectory),
        ]
    };
    if can_reload {
        file_items.push(MenuItem::Separator);
        file_items.push(MenuItem::action(EN.reload, ReloadPlugin));
    }
    let mut menus = vec![
        Menu::new(EN.app_name).items([
            MenuItem::action(EN.about_app, AboutAedicule),
            MenuItem::Separator,
            MenuItem::action(EN.quit_app, Quit),
        ]),
        Menu::new(EN.file_menu).items(file_items),
    ];
    if !application_items.is_empty() {
        menus.push(Menu::new(EN.application_menu).items(application_items));
    }
    menus
}

#[derive(Clone, Copy)]
enum OpenPathKind {
    Mixed,
    File,
    Directory,
}

/// Keeps platform-picker limitations out of the application loader by reducing
/// each host menu action to one explicit file/directory capability request.
fn open_path_prompt_options(kind: OpenPathKind) -> PathPromptOptions {
    PathPromptOptions {
        files: matches!(kind, OpenPathKind::Mixed | OpenPathKind::File),
        directories: matches!(kind, OpenPathKind::Mixed | OpenPathKind::Directory),
        multiple: false,
        prompt: Some(EN.open.into()),
    }
}

struct AudioOutput {
    sink: Option<MixerDeviceSink>,
    guest_mixer: Option<Mixer>,
    guest_player: Option<Player>,
    programs: HashMap<u32, Vec<SynthVoice>>,
    samples: HashMap<u32, HostSample>,
    last_played: HashMap<u32, Instant>,
    paused_at: Option<Instant>,
}

#[derive(Clone)]
struct HostSample {
    source: SamplesBuffer,
}

impl AudioOutput {
    fn new(metadata: &Metadata) -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().ok().map(|mut sink| {
            sink.log_on_drop(false);
            sink
        });
        let mut output = Self {
            sink,
            guest_mixer: None,
            guest_player: None,
            programs: HashMap::new(),
            samples: HashMap::new(),
            last_played: HashMap::new(),
            paused_at: None,
        };
        output.replace_metadata(metadata);
        output
    }

    fn replace_metadata(&mut self, metadata: &Metadata) {
        self.reset_transport();
        self.programs.clear();
        self.samples.clear();
        self.last_played.clear();
        for voice in &metadata.synth_voices {
            self.programs
                .entry(voice.program_id)
                .or_default()
                .push(voice.clone());
        }
        for sample in &metadata.sample_assets {
            self.samples
                .insert(sample.id, render_sample_for_host(sample));
        }
    }

    /// Replaces the guest-only mixer bus, deterministically cancelling old
    /// sources on successful reload without touching host chrome audio.
    fn reset_transport(&mut self) {
        self.guest_player = None;
        self.guest_mixer = None;
        let Some(sink) = self.sink.as_ref() else {
            return;
        };
        let channels = sink.config().channel_count();
        let sample_rate = sink.config().sample_rate();
        let (guest_mixer, guest_source) = mixer(channels, sample_rate);
        guest_mixer.add(Zero::new(channels, sample_rate));
        let guest_player = Player::connect_new(sink.mixer());
        guest_player.append(guest_source);
        if self.paused_at.is_some() {
            guest_player.pause();
        }
        self.guest_mixer = Some(guest_mixer);
        self.guest_player = Some(guest_player);
    }

    fn pause_at(&mut self, now: Instant) {
        if self.paused_at.replace(now).is_none()
            && let Some(player) = &self.guest_player
        {
            player.pause();
        }
    }

    fn resume_at(&mut self, now: Instant) {
        let Some(paused_at) = self.paused_at.take() else {
            return;
        };
        shift_audio_cooldowns(&mut self.last_played, now.duration_since(paused_at));
        if let Some(player) = &self.guest_player {
            player.play();
        }
    }

    fn play(&mut self, event: AudioEvent) {
        let Some(guest_mixer) = &self.guest_mixer else {
            return;
        };
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
        guest_mixer.add(SamplesBuffer::new(
            NonZeroU16::new(1).unwrap(),
            NonZeroU32::new(48_000).unwrap(),
            samples,
        ));
    }

    fn play_sample(&mut self, event: SampleAudioEvent) {
        let Some(guest_mixer) = &self.guest_mixer else {
            return;
        };
        let Some(sample) = self.samples.get(&event.id) else {
            return;
        };
        guest_mixer.add(
            sample
                .source
                .clone()
                .speed(event.pitch)
                .amplify(event.volume),
        );
    }
}

/// Rebases guest logical cooldowns by the exact wall duration during which
/// their audio transport was suspended.
fn shift_audio_cooldowns(last_played: &mut HashMap<u32, Instant>, suspension: Duration) {
    for timestamp in last_played.values_mut() {
        *timestamp += suspension;
    }
}

/// Converts admitted fixed PCM into rodio's device-independent source shape;
/// pitch and gain remain event-time transformations rather than stored data.
fn render_sample_for_host(sample: &SampleAsset) -> HostSample {
    HostSample {
        source: SamplesBuffer::new(
            NonZeroU16::new(sample.clip.channels).expect("admitted sample channels are nonzero"),
            NonZeroU32::new(sample.clip.sample_rate).expect("admitted sample rate is nonzero"),
            audio_samples_to_host(sample.clip.samples.clone()),
        ),
    }
}

struct ExternalSource {
    source: PluginSource,
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
    empty_state: bool,
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
    viewport: DeviceChangeTracker,
    plugin_init: PluginInit,
    monotonic_origin: Instant,
    scheduler: SimulationScheduler,
    suspension: GuestSuspension,
    paused_at: Option<Duration>,
    timing_generation: u64,
    open_documents: Rc<RefCell<VecDeque<PathBuf>>>,
    empty_state: bool,
    can_select_mixed_files_and_dirs: bool,
    guest_pointer_buttons: GuestPointerButtons,
    sliders: Vec<NativeSlider>,
    text_fields: Vec<NativeTextField>,
    _slider_subscriptions: Vec<Subscription>,
    _text_field_subscriptions: Vec<Subscription>,
    _focus_subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
struct NativeSlider {
    control: SliderControl,
    state: Entity<SliderState>,
    synced_ui_revision: Option<u32>,
}

/// Retains the platform text widget for one declared field. Unlike a slider,
/// the guest supplies no value in its placement, so the widget owns the edit
/// buffer outright and there is nothing to reconcile back on a new revision.
#[derive(Clone)]
struct NativeTextField {
    id: u32,
    state: Entity<InputState>,
}

/// Tracks presses that began on the guest canvas so a host-control release
/// cannot leak through GPUI's global `on_mouse_up_out` capture listener.
#[derive(Default)]
struct GuestPointerButtons(u8);

impl GuestPointerButtons {
    fn press(&mut self, button: PointerButton) -> bool {
        let bit = 1 << (button as u8 - 1);
        let was_up = self.0 & bit == 0;
        self.0 |= bit;
        was_up
    }

    fn release(&mut self, button: PointerButton) -> bool {
        let bit = 1 << (button as u8 - 1);
        let was_down = self.0 & bit != 0;
        self.0 &= !bit;
        was_down
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

    /// Creates one platform text widget per declared field. Entity creation is
    /// split from subscription because reload happens without a `Window` in
    /// scope and has to reach one through the window handle first.
    fn build_text_field_states(
        fields: &[TextField],
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<(u32, Entity<InputState>)> {
        fields
            .iter()
            .map(|field| {
                let capacity = field.max_scalars as usize;
                let placeholder = field.label.clone();
                let state = cx.new(|cx| {
                    InputState::new(window, cx)
                        .placeholder(placeholder)
                        // The widget refuses over-long edits so the user sees
                        // the bound; the host re-checks it independently,
                        // because a capability bound may never rest on an
                        // adapter's good behavior.
                        .validate(move |text, _| text.chars().count() <= capacity)
                });
                (field.id, state)
            })
            .collect()
    }

    /// Forwards every edit as the adapter-shared scalar sequence, so the guest
    /// observes exactly the value on screen and nothing accrues invisibly in
    /// the host.
    fn subscribe_text_fields(
        states: Vec<(u32, Entity<InputState>)>,
        cx: &mut Context<Self>,
    ) -> (Vec<NativeTextField>, Vec<Subscription>) {
        let mut fields = Vec::with_capacity(states.len());
        let mut subscriptions = Vec::with_capacity(states.len());
        for (id, state) in states {
            subscriptions.push(cx.subscribe(
                &state,
                move |this, state, event: &InputEvent, cx| {
                    let phase = match event {
                        InputEvent::Change => TextPhase::Change,
                        InputEvent::PressEnter { .. } => TextPhase::Commit,
                        InputEvent::Focus | InputEvent::Blur => return,
                    };
                    let value = state.read(cx).value().to_string();
                    for event in text_value_events(id, &value, phase) {
                        this.queue_native_event(event, cx);
                    }
                },
            ));
            fields.push(NativeTextField { id, state });
        }
        (fields, subscriptions)
    }

    fn new(
        startup: Startup,
        open_documents: Rc<RefCell<VecDeque<PathBuf>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let Startup {
            frontplane,
            frame,
            source,
            reload_error,
            plugin_init,
            empty_state,
        } = startup;
        let metadata = frontplane.metadata().clone();
        let monotonic_origin = Instant::now();
        let scheduler = SimulationScheduler::new(
            frontplane.simulation_rate(),
            MAX_TICKS_PER_WAKE,
            Duration::ZERO,
        );
        let suspension = GuestSuspension::new(metadata.pause_triggers.iter().copied());
        let entries = standard_menu_entries(&metadata);
        let (sliders, slider_subscriptions) = Self::build_sliders(&metadata.controls, cx);
        let (text_fields, text_field_subscriptions) = Self::subscribe_text_fields(
            Self::build_text_field_states(&metadata.text_fields, window, cx),
            cx,
        );
        let focus_handle = cx.focus_handle();
        let focus_subscriptions = vec![
            cx.on_focus_in(&focus_handle, window, |this, _, cx| {
                if this.suspension.is_enabled() {
                    this.queue_native_event(Event::Focus(true), cx);
                }
            }),
            cx.on_focus_out(&focus_handle, window, |this, _, _, cx| {
                if this.suspension.is_enabled() {
                    this.queue_native_event(Event::Focus(false), cx);
                }
            }),
        ];
        let mut view = Self {
            frontplane,
            frame,
            title: if empty_state {
                EN.app_name.to_owned()
            } else {
                metadata.title.clone()
            },
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
            focus_handle,
            window_handle: window.window_handle(),
            audio: AudioOutput::new(&metadata),
            source,
            fatal_error: None,
            reload_error,
            reload_notice: None,
            viewport: DeviceChangeTracker::default(),
            plugin_init,
            monotonic_origin,
            scheduler,
            suspension,
            paused_at: None,
            timing_generation: 0,
            open_documents,
            empty_state,
            can_select_mixed_files_and_dirs: cx.can_select_mixed_files_and_dirs(),
            guest_pointer_buttons: GuestPointerButtons::default(),
            sliders,
            text_fields,
            _slider_subscriptions: slider_subscriptions,
            _text_field_subscriptions: text_field_subscriptions,
            _focus_subscriptions: focus_subscriptions,
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
                    if this.open_requested_document(cx) {
                        cx.notify();
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
        if self.suspension.is_suspended() {
            self.paused_at = Some(now);
            return;
        }
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
        let application = source.source.clone();
        let path = source.path.clone();
        let revision = FileRevision::read(&path);
        if !source.revisions.observe(&revision) {
            return false;
        }
        self.apply_revision(&application, &path, revision, cx);
        true
    }

    fn reload_plugin(&mut self, _: &ReloadPlugin, _: &mut Window, cx: &mut Context<Self>) {
        let Some(source) = self.source.as_mut() else {
            return;
        };
        let application = source.source.clone();
        let path = source.path.clone();
        let revision = FileRevision::read(&path);
        source.revisions.observe(&revision);
        self.apply_revision(&application, &path, revision, cx);
        cx.notify();
    }

    /// Compiles and validates a changed file as a separate candidate, swapping
    /// only after state transfer and the candidate's first frame both succeed.
    fn apply_revision(
        &mut self,
        application: &PluginSource,
        path: &Path,
        revision: FileRevision,
        cx: &mut Context<Self>,
    ) {
        let source = match application_source_text(application, path, &revision) {
            Ok(source) => source,
            Err(error) => {
                self.set_reload_error(path, EN.reload_failed, Some(&error));
                return;
            }
        };
        let assets = match read_application_assets(application) {
            Ok(assets) => assets,
            Err(error) => {
                self.set_reload_error(path, EN.reload_failed, Some(&error));
                return;
            }
        };
        match prepare_reload_with_assets(
            &mut self.frontplane,
            &source,
            Limits::default(),
            self.plugin_init,
            assets,
        ) {
            Ok(mut prepared) => {
                if self.suspension.is_suspended()
                    && let Err(error) = prepared.restore_suspension()
                {
                    self.set_reload_error(path, EN.reload_failed, Some(&error.to_string()));
                    return;
                }
                let notice = match prepared.state_transfer {
                    StateTransfer::Preserved => EN.reload_preserved,
                    StateTransfer::Restarted => EN.reload_restarted,
                };
                self.install_frontplane(prepared.frontplane, prepared.frame, notice, cx);
            }
            Err(error) => {
                self.set_reload_error(path, EN.reload_failed, Some(&error.to_string()));
            }
        }
    }

    /// Treats an OS open-document event as a fresh application launch while
    /// preserving the current guest transactionally if the candidate fails.
    fn open_requested_document(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(path) = self.open_documents.borrow_mut().pop_front() else {
            return false;
        };
        let application = plugin_source_for_path(path.clone());
        let revision_path =
            plugin_revision_path(&application).expect("open-document application is external");
        let revision = FileRevision::read(&revision_path);
        let result =
            application_source_text(&application, &revision_path, &revision).and_then(|source| {
                load_application_frontplane(&source, &application, self.plugin_init)
                    .map_err(|error| error.to_string())
            });
        match result {
            Ok((mut frontplane, mut frame)) => {
                if self.suspension.is_suspended() {
                    if let Err(error) = frontplane
                        .event(Event::Pause(PausePhase::Restored))
                        .and_then(|()| frontplane.render())
                        .map(|paused_frame| frame = paused_frame)
                    {
                        self.set_reload_error(
                            &revision_path,
                            EN.reload_failed,
                            Some(&error.to_string()),
                        );
                        return true;
                    }
                    frontplane.drain_audio();
                    frontplane.drain_sample_audio();
                    frontplane.drain_effects();
                }
                let mut revisions = RevisionTracker::default();
                revisions.observe(&revision);
                self.source = Some(ExternalSource {
                    source: application,
                    path: revision_path,
                    watch: false,
                    revisions,
                });
                self.empty_state = false;
                self.install_frontplane(frontplane, frame, EN.reload_restarted, cx);
            }
            Err(error) => self.set_reload_error(&revision_path, EN.reload_failed, Some(&error)),
        }
        true
    }

    /// Reconciles all host adapters from one accepted guest replacement so
    /// metadata, controls, timing, menus, title, and frame change atomically.
    fn install_frontplane(
        &mut self,
        frontplane: Frontplane,
        frame: FrameOutput,
        notice: &'static str,
        cx: &mut Context<Self>,
    ) {
        let metadata = frontplane.metadata().clone();
        let entries = standard_menu_entries(&metadata);
        self.frontplane = frontplane;
        self.accept_frame(frame);
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
        self.suspension
            .replace_triggers(metadata.pause_triggers.iter().copied());
        self.audio.replace_metadata(&metadata);
        let (sliders, subscriptions) = Self::build_sliders(&metadata.controls, cx);
        self.sliders = sliders;
        self._slider_subscriptions = subscriptions;
        // A reload replaces the declarations, so every retained edit buffer is
        // discarded with them rather than surviving into a different guest.
        let text_field_states = cx
            .update_window(self.window_handle, |_, window, app| {
                Self::build_text_field_states(&metadata.text_fields, window, app)
            })
            .unwrap_or_default();
        let (text_fields, text_field_subscriptions) =
            Self::subscribe_text_fields(text_field_states, cx);
        self.text_fields = text_fields;
        self._text_field_subscriptions = text_field_subscriptions;
        self.restart_timing_loop(cx);
        self.fatal_error = None;
        self.reload_error = None;
        self.reload_notice = Some(notice);
        self.viewport.invalidate();
        cx.set_menus(native_menus(
            &metadata,
            self.source.is_some(),
            self.can_select_mixed_files_and_dirs,
        ));
        let title = window_title(&metadata);
        let _ = cx.update_window(self.window_handle, |_, window, _| {
            window.set_window_title(&title);
        });
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
        self.drain_guest_outputs(cx);
        pump.render || pump.dropped_ticks > 0
    }

    fn drain_guest_outputs(&mut self, cx: &mut Context<Self>) {
        for event in self.frontplane.drain_audio() {
            self.audio.play(event);
        }
        for event in self.frontplane.drain_sample_audio() {
            self.audio.play_sample(event);
        }
        for effect in self.frontplane.drain_effects() {
            if effect == HostEffect::Quit {
                cx.quit();
            }
        }
    }

    fn discard_guest_outputs(&mut self) {
        self.frontplane.drain_audio();
        self.frontplane.drain_sample_audio();
        self.frontplane.drain_effects();
    }

    fn render_pause_phase(&mut self, phase: PausePhase) -> Result<(), aedicule::FrontplaneError> {
        self.frontplane.event(Event::Pause(phase))?;
        self.frame = self.frontplane.render()?;
        Ok(())
    }

    /// Stops the guest timeline and its private audio bus at one logical input
    /// barrier, then accepts exactly one guest-authored paused frame.
    fn pause_guest(&mut self, now: Duration, cx: &mut Context<Self>) {
        if self.pump_simulation(now, cx) {
            cx.notify();
        }
        if self.fatal_error.is_some() {
            return;
        }
        for event in self.scheduler.drain_events_through(now) {
            if let Err(error) = self.frontplane.event(event) {
                self.fatal_error = Some(error.to_string());
                cx.notify();
                return;
            }
        }
        self.drain_guest_outputs(cx);
        self.audio.pause_at(Instant::now());
        self.timing_generation = self.timing_generation.wrapping_add(1);
        match self.render_pause_phase(PausePhase::Paused) {
            Ok(()) => {
                self.discard_guest_outputs();
                self.paused_at = Some(now);
                cx.notify();
            }
            Err(error) => {
                self.fatal_error = Some(error.to_string());
                cx.notify();
            }
        }
    }

    /// Reconciles releases before the semantic resumed edge, shifts the exact
    /// scheduler baseline, then restarts both audio and fixed-step wakes.
    fn resume_guest(&mut self, now: Duration, reconciliation: Vec<Event>, cx: &mut Context<Self>) {
        let paused_at = self
            .paused_at
            .take()
            .expect("resume disposition follows a completed native pause");
        self.scheduler.resume_after_pause(paused_at, now);
        let result = reconciliation
            .into_iter()
            .try_for_each(|event| self.frontplane.event(event))
            .and_then(|()| self.render_pause_phase(PausePhase::Resumed));
        if let Err(error) = result {
            self.fatal_error = Some(error.to_string());
            cx.notify();
            return;
        }
        self.audio.resume_at(Instant::now());
        self.drain_guest_outputs(cx);
        self.timing_generation = self.timing_generation.wrapping_add(1);
        self.spawn_timing_loop(cx);
        cx.notify();
    }

    /// Stamps ordinary native input in the scheduler clock while intercepting
    /// configured pause triggers before their raw edges can reach gameplay.
    fn queue_native_event(&mut self, event: Event, cx: &mut Context<Self>) {
        let now = self.monotonic_origin.elapsed();
        match self.suspension.handle(event) {
            SuspensionDisposition::Deliver(event) => {
                self.scheduler.queue_event(now, event);
                if self.pump_simulation(now, cx) {
                    cx.notify();
                }
            }
            SuspensionDisposition::Maintenance(event) => {
                let result = self
                    .frontplane
                    .event(event)
                    .and_then(|()| self.frontplane.render());
                match result {
                    Ok(frame) => {
                        self.frame = frame;
                        self.discard_guest_outputs();
                    }
                    Err(error) => self.fatal_error = Some(error.to_string()),
                }
                cx.notify();
            }
            SuspensionDisposition::Consumed => {}
            SuspensionDisposition::Pause => self.pause_guest(now, cx),
            SuspensionDisposition::Resume { reconciliation } => {
                self.resume_guest(now, reconciliation, cx);
            }
        }
    }

    fn new_application(&mut self, _: &NewApplication, _: &mut Window, cx: &mut Context<Self>) {
        self.fatal_error = None;
        self.queue_native_event(Event::MenuAction(1), cx);
    }

    fn about_aedicule(&mut self, _: &AboutAedicule, window: &mut Window, cx: &mut Context<Self>) {
        let detail = format!(
            "Aedicule {}\n{}",
            env!("CARGO_PKG_VERSION"),
            EN.about_detail
        );
        let _ = window.prompt(PromptLevel::Info, EN.about_app, Some(&detail), &[EN.ok], cx);
    }

    /// Feeds native file-picker selections into the existing OS open-document
    /// queue so drag/drop, Finder associations, and File→Open share one loader.
    fn prompt_for_application(
        &mut self,
        kind: OpenPathKind,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selection = cx.prompt_for_paths(open_path_prompt_options(kind));
        cx.spawn(async move |this, cx| {
            let selected = match selection.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) | Err(_) => None,
                Ok(Err(error)) => {
                    let detail = format!("{}: {error}", EN.file_picker_failed);
                    let _ = this.update(cx, |this, cx| {
                        this.reload_error = Some(detail);
                        cx.notify();
                    });
                    None
                }
            };
            if let Some(path) = selected {
                let _ = this.update(cx, |this, cx| {
                    this.open_documents.borrow_mut().push_back(path);
                    if this.open_requested_document(cx) {
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    fn open_application(
        &mut self,
        _: &OpenApplication,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let kind = if self.can_select_mixed_files_and_dirs {
            OpenPathKind::Mixed
        } else {
            OpenPathKind::File
        };
        self.prompt_for_application(kind, window, cx);
    }

    fn open_project_directory(
        &mut self,
        _: &OpenProjectDirectory,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.prompt_for_application(OpenPathKind::Directory, window, cx);
    }

    fn show_help(&mut self, _: &ShowHelp, _: &mut Window, cx: &mut Context<Self>) {
        self.queue_native_event(Event::MenuAction(7), cx);
    }

    fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    /// Keyboard input belongs to a focused text control, not the guest. This
    /// is the keyboard counterpart of the pointer occlusion that AVP control
    /// layers already provide; without it a guest steals every letter that
    /// happens to map to a guest key, and the text field silently drops it.
    fn text_entry_has_focus(&self, window: &Window, cx: &App) -> bool {
        self.text_fields
            .iter()
            .any(|field| field.state.read(cx).focus_handle(cx).is_focused(window))
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_entry_has_focus(window, cx) {
            return;
        }
        let Some(key) = Key::from_gpui_name(&event.keystroke.key) else {
            return;
        };
        if event.is_held {
            return;
        }
        self.queue_native_event(Event::KeyDown(key), cx);
    }

    fn key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_entry_has_focus(window, cx) {
            return;
        }
        let Some(key) = Key::from_gpui_name(&event.keystroke.key) else {
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
        if !self.guest_pointer_buttons.press(button) {
            return;
        }
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
        if !self.guest_pointer_buttons.release(button) {
            return;
        }
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

/// Presents guest-declared navigation as a focusable semantic link while
/// keeping activation inside the platform's trusted click callback.
fn external_link_control(
    id: u32,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let label = label.into();
    div()
        .id(("external-link-semantic", id as usize))
        .role(Role::Link)
        .aria_label(label.clone())
        .w_full()
        .h_full()
        .child(
            Button::new(("external-link-control", id as usize))
                .link()
                .label(label)
                .w_full()
                .h_full()
                .on_click(on_click),
        )
}

impl Render for FrontplaneView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        // A native window has a fine primary pointer; no device flags apply yet.
        if let Some((width, height, flags)) = self
            .viewport
            .observe(f32::from(viewport.width), f32::from(viewport.height), 0)
        {
            self.queue_native_event(Event::DeviceChange { width, height, flags }, cx);
        }
        let frame = self.frame.clone();
        let ui = self.frontplane.ui_snapshot().cloned();
        let fatal_error = self.fatal_error.clone();
        let reload_error = self.reload_error.clone();
        let title = self.title.clone();
        let new_label = self.new_label.clone();
        let help_label = self.help_label.clone();
        let quit_label = self.quit_label.clone();
        let empty_state = self.empty_state;
        let can_select_mixed_files_and_dirs = self.can_select_mixed_files_and_dirs;
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
        let text_field_layers = ui
            .iter()
            .flat_map(|snapshot| snapshot.text_fields.iter())
            .filter_map(|placement| {
                let field = self
                    .text_fields
                    .iter()
                    .find(|field| field.id == placement.id)?;
                Some(
                    guest_positioned_control_layer(placement.bounds)
                        .id(("text-field", placement.id as usize))
                        .child(Input::new(&field.state).h_full()),
                )
            })
            .collect::<Vec<_>>();
        let external_link_layers = ui
            .iter()
            .flat_map(|snapshot| {
                let revision = snapshot.revision;
                snapshot
                    .external_links
                    .iter()
                    .map(move |placement| (revision, placement))
            })
            .map(|(revision, placement)| {
                let id = placement.id;
                let label = self
                    .frontplane
                    .metadata()
                    .external_links
                    .iter()
                    .find(|link| link.id == id)
                    .expect("validated external-link placement remains declared")
                    .label
                    .clone();
                let link = external_link_control(
                    id,
                    label,
                    cx.listener(move |this, _, _, cx| {
                        if let Ok(request) = this.frontplane.external_link_request(revision, id) {
                            cx.open_url(&request.url);
                        }
                    }),
                );
                guest_positioned_control_layer(placement.bounds)
                    .id(("external-link", placement.id as usize))
                    .child(link)
            })
            .collect::<Vec<_>>();
        let can_reload = self.source.is_some();
        let source_status = match (&self.source, empty_state) {
            (_, true) => EN.no_application.to_owned(),
            (Some(source), false) if source.watch => {
                format!("{} {}", EN.watching, source.path.display())
            }
            (Some(source), false) => format!("{} {}", EN.external_source, source.path.display()),
            (None, false) => EN.embedded_source.to_owned(),
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
            .on_action(cx.listener(Self::about_aedicule))
            .on_action(cx.listener(Self::open_application))
            .on_action(cx.listener(Self::open_project_directory))
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
                                aedicule::gpui_canvas::paint_frame(&frame, bounds, window, cx)
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
            .children(external_link_layers)
            .children(text_field_layers)
            .when(empty_state, |this| {
                this.child(
                    div()
                        .id("empty-state")
                        .absolute()
                        .top(px(HOST_TITLE_BAR_HEIGHT))
                        .bottom(px(28.0))
                        .left_0()
                        .right_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0x0d1117ff))
                        .child(
                            div()
                                .w(px(560.0))
                                .p_8()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_4()
                                .rounded_xl()
                                .bg(rgba(0x161b22ff))
                                .text_color(rgba(0xf6f7f9ff))
                                .child(div().text_xl().child(EN.empty_heading))
                                .child(
                                    div()
                                        .text_center()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(EN.empty_detail),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .when(can_select_mixed_files_and_dirs, |this| {
                                            this.child(
                                                Button::new("open-empty-state")
                                                    .primary()
                                                    .label(EN.open)
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.open_application(
                                                                &OpenApplication,
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    )),
                                            )
                                        })
                                        .when(!can_select_mixed_files_and_dirs, |this| {
                                            this.child(
                                                Button::new("open-file-empty-state")
                                                    .primary()
                                                    .label(EN.open_application)
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.open_application(
                                                                &OpenApplication,
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("open-directory-empty-state")
                                                    .label(EN.open_project_directory)
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.open_project_directory(
                                                                &OpenProjectDirectory,
                                                                window,
                                                                cx,
                                                            )
                                                        },
                                                    )),
                                            )
                                        }),
                                ),
                        ),
                )
            })
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

/// Instantiates the stable conformance fallback through the same lifecycle as
/// external applications when no usable runtime plugin is available.
fn load_fallback(plugin_init: PluginInit) -> (Frontplane, FrameOutput) {
    load_frontplane(FALLBACK_WAT, plugin_init)
        .expect("embedded conformance WAT passed the headless ABI suite")
}

fn load_frontplane(
    source: &str,
    plugin_init: PluginInit,
) -> Result<(Frontplane, FrameOutput), aedicule::FrontplaneError> {
    initialize_loaded_frontplane(
        Frontplane::from_wat(source, Limits::default())?,
        plugin_init,
    )
}

fn load_application_frontplane(
    source: &str,
    application: &PluginSource,
    plugin_init: PluginInit,
) -> Result<(Frontplane, FrameOutput), aedicule::FrontplaneError> {
    let assets =
        read_application_assets(application).map_err(aedicule::FrontplaneError::Application)?;
    initialize_loaded_frontplane(
        Frontplane::from_wat_with_assets(source, Limits::default(), assets)?,
        plugin_init,
    )
}

fn initialize_loaded_frontplane(
    mut frontplane: Frontplane,
    plugin_init: PluginInit,
) -> Result<(Frontplane, FrameOutput), aedicule::FrontplaneError> {
    initialize_frontplane(&mut frontplane, plugin_init)?;
    let frame = frontplane.render()?;
    frontplane.drain_audio();
    frontplane.drain_sample_audio();
    frontplane.drain_effects();
    Ok((frontplane, frame))
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
                empty_state: false,
            }
        }
        application @ (PluginSource::File(_)
        | PluginSource::Directory(_)
        | PluginSource::Archive(_)) => {
            let path = plugin_revision_path(&application)
                .expect("external application has a revision path");
            let revision = FileRevision::read(&path);
            let mut revisions = RevisionTracker::default();
            revisions.observe(&revision);
            let result =
                application_source_text(&application, &path, &revision).and_then(|source| {
                    load_application_frontplane(&source, &application, plugin_init)
                        .map_err(|error| error.to_string())
                });
            match result {
                Ok((frontplane, frame)) => Startup {
                    frontplane,
                    frame,
                    source: Some(ExternalSource {
                        source: application,
                        path,
                        watch,
                        revisions,
                    }),
                    reload_error: None,
                    plugin_init,
                    empty_state: false,
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
                            source: application,
                            path,
                            watch,
                            revisions,
                        }),
                        reload_error: Some(reload_error),
                        plugin_init,
                        empty_state: false,
                    }
                }
            }
        }
    })
}

fn launcher(seed: Option<u64>) -> Result<Startup, String> {
    let mut startup = startup(PluginSource::Embedded, false, seed)?;
    startup.empty_state = true;
    Ok(startup)
}

fn plugin_source_for_path(path: PathBuf) -> PluginSource {
    if path.is_dir() {
        PluginSource::Directory(path)
    } else if path.extension().is_some_and(|extension| extension == "aed") {
        PluginSource::Archive(path)
    } else {
        PluginSource::File(path)
    }
}

fn plugin_revision_path(source: &PluginSource) -> Option<PathBuf> {
    match source {
        PluginSource::Embedded => None,
        PluginSource::File(path) | PluginSource::Archive(path) => Some(path.clone()),
        PluginSource::Directory(root) => Some(root.join(aedicule::DEFAULT_PLUGIN_FILE)),
    }
}

/// Resolves external containers through one virtual-root reader while using a
/// content revision as the transactional trigger for reload decisions.
fn application_source_text(
    application: &PluginSource,
    path: &Path,
    revision: &FileRevision,
) -> Result<String, String> {
    let bytes = match revision {
        FileRevision::Missing => Err(format!("{}: {}", EN.missing_source, path.display())),
        FileRevision::Unreadable(error) => Err(format!(
            "{}: {}: {error}",
            EN.unreadable_source,
            path.display()
        )),
        FileRevision::Content(bytes) => Ok(match application {
            PluginSource::Archive(_) => {
                read_application_file(application, aedicule::DEFAULT_PLUGIN_FILE)?
            }
            _ => bytes.clone(),
        }),
    }?;
    String::from_utf8(bytes)
        .map_err(|error| format!("{}: {}: {error}", EN.invalid_utf8, path.display()))
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
    let open_documents = Rc::new(RefCell::new(VecDeque::new()));
    app.on_open_urls({
        let open_documents = open_documents.clone();
        move |urls| {
            open_documents
                .borrow_mut()
                .extend(urls.iter().filter_map(|url| file_url_to_path(url)));
        }
    });
    app.run(move |cx| {
        gpui_component::init(cx);
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(GEIST_MONO_REGULAR)])
            .expect("register bundled Geist Mono Regular");
        cx.bind_keys([
            KeyBinding::new("cmd-o", OpenApplication, None),
            KeyBinding::new("ctrl-o", OpenApplication, None),
            KeyBinding::new("ctrl-n", NewApplication, None),
            KeyBinding::new("f1", ShowHelp, None),
            KeyBinding::new("cmd-r", ReloadPlugin, None),
            KeyBinding::new("ctrl-r", ReloadPlugin, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);
        let metadata = startup.frontplane.metadata().clone();
        let window_title = if startup.empty_state {
            EN.app_name.to_owned()
        } else {
            window_title(&metadata)
        };
        let can_select_mixed_files_and_dirs = cx.can_select_mixed_files_and_dirs();
        cx.set_menus(native_menus(
            &metadata,
            startup.source.is_some(),
            can_select_mixed_files_and_dirs,
        ));
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.activate(true);
        let options = WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            window_bounds: Some(WindowBounds::centered(size(px(1100.0), px(850.0)), cx)),
            ..Default::default()
        };
        let open_documents = open_documents.clone();
        cx.spawn(async move |cx| {
            cx.open_window(options, move |window, cx| {
                window.activate_window();
                window.set_window_title(&window_title);
                Theme::change(ThemeMode::Dark, Some(window), cx);
                let view = cx.new(|cx| FrontplaneView::new(startup, open_documents, window, cx));
                view.focus_handle(cx).focus(window, cx);
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("open GPUI frontplane window");
        })
        .detach();
    });
}

/// Produces a per-invocation order seed; the printed hexadecimal value is the
/// stable replay interface, while tests inject an explicit seed instead.
fn fresh_test_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    nanos ^ u64::from(std::process::id()).rotate_left(32)
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
            "aedicule {} — {} for {} {}",
            env!("CARGO_PKG_VERSION"),
            EN.about,
            env::consts::OS,
            env::consts::ARCH,
        ),
        LaunchAction::Launcher { seed } => match launcher(seed) {
            Ok(startup) => run_application(startup),
            Err(error) => {
                eprintln!("{}: {error}", EN.cli_error);
                return ExitCode::from(2);
            }
        },
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
        LaunchAction::Package { source, output } => {
            if let Err(error) = package_application(&source, &output) {
                eprintln!("{}: {error}", EN.cli_error);
                return ExitCode::from(2);
            }
            println!("{}", output.display());
        }
        LaunchAction::Depackage { source, output } => {
            if let Err(error) = depackage_application(&source, &output) {
                eprintln!("{}: {error}", EN.cli_error);
                return ExitCode::from(2);
            }
            println!("{}", output.display());
        }
        LaunchAction::Test { source, seed } => {
            let seed = seed.unwrap_or_else(fresh_test_seed);
            println!("{}: 0x{seed:016x}", EN.test_seed);
            match run_application_tests(&source, seed) {
                Ok(report) => {
                    let passed = report.passed.len();
                    for name in report.passed {
                        println!("{} {name}", EN.test_pass);
                    }
                    println!("{passed} {}", EN.tests_passed);
                }
                Err(error) => {
                    eprintln!("{}: {error}", EN.cli_error);
                    return ExitCode::FAILURE;
                }
            }
        }
        LaunchAction::Web { source, bind, port } => {
            let result = discover_web_runtime().and_then(|runtime| {
                let server = WebServer::bind(&source, &runtime, &bind, port)?;
                println!("{}", server.url()?);
                std::io::stdout()
                    .flush()
                    .map_err(|error| format!("flush web URL: {error}"))?;
                server.serve()
            });
            if let Err(error) = result {
                eprintln!("{}: {error}", EN.cli_error);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{
        DECIMAL_SCALE, GuestPointerButtons, OpenPathKind, StandardMenuEntry, TITLE_BAR_GLYPH_RGBA,
        external_link_control, fixed_sine, guest_positioned_control_layer,
        host_title_bar_layer, menu_action_event, menu_action_label, native_menus,
        open_path_prompt_options, render_sample_for_host, render_synth_program_fixed,
        shift_audio_cooldowns, standard_menu_entries, title_bar_control_glyph_overlay,
        title_bar_control_glyphs,
    };
    use aedicule::{
        DEVICE_FLAG_COARSE_POINTER, DeviceChangeTracker, Event, Key, MenuItem as PluginMenuItem, Metadata, SampleAsset, SynthFilter, SynthVoice,
        SynthWaveform, gpui_canvas::viewport_transform,
    };
    use gpui::{
        Bounds, Context, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
        Render, Styled as _, TestApp, Window, div, point, px, size,
    };
    use gpui_component::TitleBar;
    use rodio::{
        Player, Source as _, buffer::SamplesBuffer as TestSamplesBuffer, mixer::mixer as test_mixer,
    };
    use std::{
        collections::HashMap,
        num::{NonZeroU16, NonZeroU32},
        time::{Duration, Instant},
    };

    #[derive(Default)]
    struct HostTitleBarHitProbe {
        guest_pointer_downs: usize,
        host_pointer_downs: usize,
    }

    struct HostControlHitProbe {
        bounds: aedicule::Rect,
        guest_pointer_downs: usize,
        host_pointer_downs: usize,
    }

    impl Default for HostControlHitProbe {
        fn default() -> Self {
            Self {
                bounds: aedicule::Rect {
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

    struct ExternalLinkProbe;

    impl Render for ExternalLinkProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            external_link_control(7, "What is this?", |_, _, cx| {
                cx.open_url("https://example.com/readme#ulam");
            })
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
    fn native_external_link_opens_only_from_its_accessible_click_control() {
        let mut app = TestApp::new();
        app.update(gpui_component::init);
        let mut window = app.open_window(|_, _| ExternalLinkProbe);
        window.draw();

        assert_eq!(app.opened_url(), None);
        window.simulate_click(point(px(40.0), px(16.0)), MouseButton::Left);
        assert_eq!(
            app.opened_url().as_deref(),
            Some("https://example.com/readme#ulam")
        );
    }

    #[test]
    fn guest_pointer_releases_require_a_matching_canvas_press() {
        let mut buttons = GuestPointerButtons::default();

        assert!(!buttons.release(aedicule::PointerButton::Secondary));
        assert!(buttons.press(aedicule::PointerButton::Secondary));
        assert!(!buttons.press(aedicule::PointerButton::Secondary));
        assert!(buttons.press(aedicule::PointerButton::Primary));
        assert!(buttons.release(aedicule::PointerButton::Secondary));
        assert!(!buttons.release(aedicule::PointerButton::Secondary));
        assert!(buttons.release(aedicule::PointerButton::Primary));
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
            bounds: aedicule::Rect {
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
            pause_triggers: Vec::new(),
            controls: Vec::new(),
            external_links: Vec::new(),
            text_fields: Vec::new(),
            synth_voices: Vec::new(),
            sample_assets: Vec::new(),
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
        assert_eq!(
            native_menus(&metadata, false, true)
                .iter()
                .map(|menu| menu.name.as_ref())
                .collect::<Vec<_>>(),
            vec!["Aedicule", "File", "Application"]
        );
    }

    #[test]
    fn host_escape_hatches_exist_without_guest_metadata() {
        let menus = native_menus(&Metadata::default(), false, true);
        assert_eq!(
            menus
                .iter()
                .map(|menu| menu.name.as_ref())
                .collect::<Vec<_>>(),
            vec!["Aedicule", "File"]
        );
        let item_names = |menu: &gpui::Menu| {
            menu.items
                .iter()
                .filter_map(|item| match item {
                    gpui::MenuItem::Action { name, .. } => Some(name.to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            item_names(&menus[0]),
            vec!["About Aedicule", "Quit Aedicule"]
        );
        assert_eq!(item_names(&menus[1]), vec!["Open…"]);
    }

    #[test]
    fn platforms_without_mixed_pickers_expose_file_and_directory_actions() {
        let menus = native_menus(&Metadata::default(), false, false);
        let item_names = menus[1]
            .items
            .iter()
            .filter_map(|item| match item {
                gpui::MenuItem::Action { name, .. } => Some(name.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            item_names,
            vec!["Open Application…", "Open Project Directory…"]
        );
    }

    #[test]
    fn picker_capabilities_classify_every_open_action() {
        let cases = [
            (OpenPathKind::Mixed, true, true),
            (OpenPathKind::File, true, false),
            (OpenPathKind::Directory, false, true),
        ];

        for (kind, files, directories) in cases {
            let options = open_path_prompt_options(kind);
            assert_eq!((options.files, options.directories), (files, directories));
            assert!(!options.multiple);
        }
    }

    #[test]
    fn viewport_projection_uses_every_drawable_pixel_without_letterboxing() {
        let bounds = Bounds::new(point(px(11.0), px(17.0)), size(px(1600.0), px(900.0)));

        assert_eq!(
            viewport_transform(bounds),
            aedicule::Affine {
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
    fn device_flag_flips_emit_even_when_the_size_is_unchanged() {
        let mut tracker = DeviceChangeTracker::default();

        assert_eq!(tracker.observe(800.0, 600.0, 0), Some((800.0, 600.0, 0)));
        assert_eq!(tracker.observe(800.0, 600.0, 0), None);
        assert_eq!(
            tracker.observe(800.0, 600.0, DEVICE_FLAG_COARSE_POINTER),
            Some((800.0, 600.0, DEVICE_FLAG_COARSE_POINTER)),
        );
        assert_eq!(
            tracker.observe(800.0, 600.0, DEVICE_FLAG_COARSE_POINTER),
            None,
        );
        assert_eq!(tracker.observe(800.0, 600.0, 0), Some((800.0, 600.0, 0)));
    }

    #[test]
    fn viewport_updates_emit_once_per_distinct_valid_size() {
        let mut tracker = DeviceChangeTracker::default();

        assert_eq!(tracker.observe(1600.5, 900.25, 0), Some((1600.5, 900.25, 0)));
        assert_eq!(tracker.observe(1600.5, 900.25, 0), None);
        assert_eq!(tracker.observe(1700.0, 900.25, 0), Some((1700.0, 900.25, 0)));
        assert_eq!(tracker.observe(0.0, 900.0, 0), None);
        assert_eq!(tracker.observe(f32::NAN, 900.0, 0), None);
    }

    #[test]
    fn native_key_mapping_exposes_physical_keys_without_guest_semantics() {
        assert_eq!(Key::from_gpui_name("left"), Some(Key::ArrowLeft));
        assert_eq!(Key::from_gpui_name("right"), Some(Key::ArrowRight));
        assert_eq!(Key::from_gpui_name("up"), Some(Key::ArrowUp));
        assert_eq!(Key::from_gpui_name("space"), Some(Key::Space));
        assert_eq!(Key::from_gpui_name("p"), Some(Key::P));
        assert_eq!(Key::from_gpui_name("escape"), Some(Key::Escape));
        assert_eq!(Key::from_gpui_name("r"), Some(Key::R));
        assert_eq!(Key::from_gpui_name("f"), Some(Key::F));
        assert_eq!(Key::from_gpui_name("k"), Some(Key::K));
        assert_eq!(Key::from_gpui_name("b"), Some(Key::B));
        assert_eq!(Key::from_gpui_name("h"), Some(Key::H));
        assert_eq!(Key::from_gpui_name("f1"), Some(Key::F1));
        assert_eq!(Key::from_gpui_name("w"), Some(Key::W));
        assert_eq!(Key::from_gpui_name("a"), Some(Key::A));
        assert_eq!(Key::from_gpui_name("d"), Some(Key::D));
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

    #[test]
    fn admitted_fixed_pcm_is_converted_only_at_the_native_audio_boundary() {
        let sample = SampleAsset {
            id: 77,
            path: "assets/audio/sample.flac".to_owned(),
            clip: aedicule::wav::WavClip {
                channels: 2,
                sample_rate: 8_000,
                frames: 2,
                samples: vec![-1_000_000, 0, 500_000, 1_000_000],
            },
        };

        let rendered = render_sample_for_host(&sample);
        assert_eq!(
            (
                rendered.source.channels().get(),
                rendered.source.sample_rate().get()
            ),
            (2, 8_000)
        );
        assert_eq!(
            rendered.source.collect::<Vec<_>>(),
            vec![-1.0, 0.0, 0.5, 1.0]
        );
    }

    #[test]
    fn native_audio_cooldowns_exclude_wall_time_spent_paused() {
        let anchor = Instant::now();
        let mut last_played = HashMap::from([(7, anchor + Duration::from_millis(25))]);

        shift_audio_cooldowns(&mut last_played, Duration::from_secs(90));

        assert_eq!(last_played[&7], anchor + Duration::from_millis(90_025));
    }

    #[test]
    fn guest_audio_mixer_pause_retains_the_nested_source_cursor() {
        let channels = NonZeroU16::new(1).unwrap();
        let sample_rate = NonZeroU32::new(1).unwrap();
        let (guest_mixer, guest_source) = test_mixer(channels, sample_rate);
        let (guest_player, mut device_source) = Player::new();
        guest_player.append(guest_source);
        guest_mixer.add(TestSamplesBuffer::new(
            channels,
            sample_rate,
            vec![0.25, 0.5, 0.75],
        ));

        assert_eq!(
            device_source
                .by_ref()
                .take(32)
                .find(|sample| *sample != 0.0),
            Some(0.25)
        );
        guest_player.pause();
        assert_eq!(device_source.next(), Some(0.0));
        assert_eq!(device_source.next(), Some(0.0));
        guest_player.play();
        assert_eq!(device_source.next(), Some(0.5));
        assert_eq!(device_source.next(), Some(0.75));
    }
}
