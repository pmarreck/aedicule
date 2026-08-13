//! Browser GPUI entrypoint for the portable, fuel-metered Aedicule runtime.

#[cfg(target_family = "wasm")]
use std::{borrow::Cow, cell::OnceCell};

#[cfg_attr(not(target_family = "wasm"), allow(unused_imports))]
use aedicule::{
    ApplicationAssets, ControlLabelPlacement, ControlPhase, DEVICE_FLAG_COARSE_POINTER,
    DeviceChangeTracker, Event, ExternalLinkRequest, MotionInterestKind, ShakeDetector,
    GEIST_MONO_REGULAR, Key, PluginInit, PointerButton, PointerScrollUnit, Rect, SliderControl,
    TextField, TextPhase, TouchContactPhase, TouchContactTracker, UiSnapshot,
    gpui_canvas::paint_frame,
    text_value_events,
    web::{
        BROWSER_ASSETS_GLOBAL, BROWSER_WAT_GLOBAL, BrowserDeliveredEventCounts,
        BrowserInputOutcome, BrowserRuntime, required_browser_wat,
    },
};
#[cfg_attr(not(target_family = "wasm"), allow(unused_imports))]
use gpui::{
    App, AppContext as _, Bounds, ClickEvent, Context, Entity, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyDownEvent, KeyUpEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement as _, Render, Role, ScrollDelta, ScrollWheelEvent,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window, WindowBounds,
    WindowOptions, canvas, div, prelude::FluentBuilder as _, px, rgba, size,
};
#[cfg_attr(not(target_family = "wasm"), allow(unused_imports))]
use gpui_component::{
    Root, Selectable as _, Theme, ThemeMode,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState},
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsCast as _, JsValue};
use web_time::Instant;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = globalThis, js_name = __AEDICULE_PLAY_PCM)]
    fn play_browser_pcm(
        sample_rate: u32,
        channels: u32,
        samples: &js_sys::Float32Array,
        volume: f32,
        pitch: f32,
    );

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = globalThis, js_name = __AEDICULE_SET_AUDIO_PAUSED)]
    fn set_browser_audio_paused(paused: bool);

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = globalThis, js_name = __AEDICULE_REPORT_FRAME_STARTED)]
    fn report_browser_frame_started(guest_elapsed_ms: f64);

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = globalThis, js_name = __AEDICULE_REPORT_FRAME_COMPLETED)]
    fn report_browser_frame_completed(guest_elapsed_ms: f64, background: u32);

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console, js_name = info)]
    fn browser_console_info(prefix: &str, phase: &str, detail: &str);
}

#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
const LOGICAL_WIDTH: f32 = 1024.0;
#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
const LOGICAL_HEIGHT: f32 = 768.0;
#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
const DEFAULT_PLUGIN_INIT: PluginInit = PluginInit::new(0x5eed_cafe, LOGICAL_WIDTH, LOGICAL_HEIGHT);

#[cfg(target_family = "wasm")]
thread_local! {
    static BROWSER_APPLICATION: OnceCell<gpui::ApplicationHandle> = const { OnceCell::new() };
}

/// Keeps GPUI's strong application handle for the browser document lifetime;
/// the externally driven WebAssembly run loop returns immediately after setup.
#[cfg(target_family = "wasm")]
fn retain_browser_application(application: gpui::ApplicationHandle) {
    BROWSER_APPLICATION.with(|retained| {
        assert!(
            retained.set(application).is_ok(),
            "Aedicule browser application started more than once"
        );
    });
}

/// Extends the opt-in DOM input trace across the Rust adapter boundary so a
/// browser probe can distinguish dispatch, frontplane admission, and guest
/// behavior without making production consoles noisy.
#[cfg(target_family = "wasm")]
fn trace_browser_detail(phase: &str, detail: &str) {
    let enabled = js_sys::Reflect::get(
        &js_sys::global(),
        &JsValue::from_str("__AEDICULE_TRACE_EVENTS"),
    )
    .ok()
    .and_then(|value| value.as_bool())
    .unwrap_or(false);
    if enabled {
        browser_console_info("[Aedicule input]", phase, detail);
    }
}

#[cfg(not(target_family = "wasm"))]
fn trace_browser_detail(_: &str, _: &str) {}

fn trace_browser_input(event: &Event) {
    trace_browser_detail("frontplane", &format!("{event:?}"));
}

#[cfg(not(target_family = "wasm"))]
fn set_browser_audio_paused(_: bool) {}

#[cfg(not(target_family = "wasm"))]
fn report_browser_frame_started(_: f64) {}

#[cfg(not(target_family = "wasm"))]
fn report_browser_frame_completed(_: f64, _: u32) {}

/// Publishes the last accepted guest background as a compact browser
/// diagnostic; unlike headless GPU screenshots, this observes the committed
/// Aedicule frame without depending on Chromium's shared-image readback.
#[cfg(target_family = "wasm")]
fn publish_browser_frame_background(background: u32) {
    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("__AEDICULE_FRAME_BACKGROUND"),
        &JsValue::from_f64(f64::from(background)),
    )
    .expect("publish accepted browser frame background");
}

#[cfg(not(target_family = "wasm"))]
fn publish_browser_frame_background(_: u32) {}

#[cfg(target_family = "wasm")]
fn set_js_property(target: &js_sys::Object, name: &str, value: &JsValue) {
    js_sys::Reflect::set(target, &JsValue::from_str(name), value)
        .expect("publish Aedicule browser diagnostic property");
}

/// Publishes guest-delivery progress independently of DOM event observation so
/// browser acceptance can wait on semantic WAT delivery without wall-clock
/// sleeps.
#[cfg(target_family = "wasm")]
fn publish_browser_delivered_events(counts: BrowserDeliveredEventCounts) {
    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("__AEDICULE_DELIVERED_EVENT_COUNT"),
        &JsValue::from_f64(counts.total as f64),
    )
    .expect("publish browser guest event count");
    let diagnostic = js_sys::Object::new();
    for (name, value) in [
        ("total", counts.total),
        ("controls", counts.controls),
		("menuActions", counts.menu_actions),
		("pointer", counts.pointer),
		("touch", counts.touch),
	] {
        set_js_property(&diagnostic, name, &JsValue::from_f64(value as f64));
    }
    set_js_property(
        &diagnostic,
        "lastMenuActionId",
        &counts
            .last_menu_action_id
            .map_or(JsValue::NULL, |id| JsValue::from_f64(f64::from(id))),
    );
    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("__AEDICULE_DELIVERED_EVENT_COUNTS"),
        &diagnostic,
    )
    .expect("publish browser semantic event counts");
}

#[cfg(not(target_family = "wasm"))]
fn publish_browser_delivered_events(_: BrowserDeliveredEventCounts) {}

/// Publishes the exact accepted AVP placement bounds as read-only diagnostics;
/// the adapter still renders from the typed snapshot rather than this JS copy.
#[cfg(target_family = "wasm")]
fn publish_browser_ui_snapshot(ui: &UiSnapshot) {
    let diagnostic = js_sys::Object::new();
    set_js_property(
        &diagnostic,
        "revision",
        &JsValue::from_f64(f64::from(ui.revision)),
    );
    let panels = js_sys::Array::new();
    let sliders = js_sys::Array::new();
    let buttons = js_sys::Array::new();
    let external_links = js_sys::Array::new();
    let text_fields = js_sys::Array::new();
    for (id, bounds, output) in ui
        .control_panels
        .iter()
        .map(|panel| (panel.id, panel.bounds, &panels))
        .chain(
            ui.sliders
                .iter()
                .map(|slider| (slider.id, slider.bounds, &sliders)),
        )
        .chain(
            ui.buttons
                .iter()
                .map(|button| (button.id, button.bounds, &buttons)),
        )
        .chain(
            ui.external_links
                .iter()
                .map(|link| (link.id, link.bounds, &external_links)),
        )
        .chain(
            ui.text_fields
                .iter()
                .map(|field| (field.id, field.bounds, &text_fields)),
        )
    {
        let placement = js_sys::Object::new();
        set_js_property(&placement, "id", &JsValue::from_f64(f64::from(id)));
        set_js_property(&placement, "x", &JsValue::from_f64(f64::from(bounds.x)));
        set_js_property(&placement, "y", &JsValue::from_f64(f64::from(bounds.y)));
        set_js_property(
            &placement,
            "width",
            &JsValue::from_f64(f64::from(bounds.width)),
        );
        set_js_property(
            &placement,
            "height",
            &JsValue::from_f64(f64::from(bounds.height)),
        );
        output.push(&placement);
    }
    for (index, button) in ui.buttons.iter().enumerate() {
        let placement = buttons.get(index as u32).unchecked_into::<js_sys::Object>();
        set_js_property(
            &placement,
            "actionId",
            &JsValue::from_f64(f64::from(button.action_id)),
        );
    }
    set_js_property(&diagnostic, "panels", &panels);
    set_js_property(&diagnostic, "sliders", &sliders);
    set_js_property(&diagnostic, "buttons", &buttons);
    set_js_property(&diagnostic, "externalLinks", &external_links);
    set_js_property(&diagnostic, "textFields", &text_fields);
    js_sys::Reflect::set(
        &js_sys::global(),
        &JsValue::from_str("__AEDICULE_UI_SNAPSHOT"),
        &diagnostic,
    )
    .expect("publish accepted browser UI snapshot");
}

#[cfg(not(target_family = "wasm"))]
fn publish_browser_ui_snapshot(_: &UiSnapshot) {}

/// Opens a validated destination only while the browser still reports a
/// transient user activation, then exposes adapter outcome for diagnostics.
#[cfg(target_family = "wasm")]
fn activate_browser_external_link(request: &ExternalLinkRequest) {
    let global = js_sys::global();
    let active = js_sys::Reflect::get(&global, &JsValue::from_str("navigator"))
        .ok()
        .and_then(|navigator| {
            js_sys::Reflect::get(&navigator, &JsValue::from_str("userActivation")).ok()
        })
        .and_then(|activation| {
            js_sys::Reflect::get(&activation, &JsValue::from_str("isActive")).ok()
        })
        .and_then(|active| active.as_bool())
        .unwrap_or(false);
    let status = if !active {
        "rejected-no-user-activation"
    } else {
        match js_sys::Reflect::get(&global, &JsValue::from_str("open"))
            .ok()
            .and_then(|open| open.dyn_into::<js_sys::Function>().ok())
        {
            Some(open) => match open.call3(
                &global,
                &JsValue::from_str(&request.url),
                &JsValue::from_str("_blank"),
                &JsValue::from_str("noopener"),
            ) {
                Ok(value) if value.is_null() => "blocked",
                Ok(_) => "opened",
                Err(_) => "failed",
            },
            None => "failed",
        }
    };
    let diagnostic = js_sys::Object::new();
    for (name, value) in [
        ("revision", JsValue::from_f64(f64::from(request.revision))),
        ("id", JsValue::from_f64(f64::from(request.id))),
        ("url", JsValue::from_str(&request.url)),
        ("status", JsValue::from_str(status)),
        ("userActivation", JsValue::from_bool(active)),
    ] {
        set_js_property(&diagnostic, name, &value);
    }
    js_sys::Reflect::set(
        &global,
        &JsValue::from_str("__AEDICULE_EXTERNAL_LINK_ACTIVATION"),
        &diagnostic,
    )
    .expect("publish browser external-link activation");
}

#[cfg(not(target_family = "wasm"))]
fn activate_browser_external_link(_: &ExternalLinkRequest) {}

/// Bridges GPUI Web animation/input callbacks to Aedicule's tested rational
/// scheduler, keeping guest execution outside the canvas paint callback.
#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
struct WebFrontplane {
    runtime: BrowserRuntime,
    origin: Instant,
    focus_handle: FocusHandle,
    reported_delivered_events: u64,
    device_change: DeviceChangeTracker,
    fatal_error: Option<String>,
    pressed_pointer_buttons: [u16; 3],
    coarse_pointer: bool,
    shake_interest: bool,
    motion_sample_interval_ms: Option<f64>,
    shake_detector: ShakeDetector,
    last_motion_sample_ms: f64,
    armed_control: Option<ArmedControl>,
    touch_contacts: Option<TouchContactTracker>,
    sliders: Vec<WebSlider>,
    text_fields: Vec<WebTextField>,
    _slider_subscriptions: Vec<Subscription>,
    _text_field_subscriptions: Vec<Subscription>,
    _focus_subscriptions: Vec<Subscription>,
}

/// How far past a control's declared edge a finger may travel between press and
/// release and still activate it. A fingertip covers far more than a mouse
/// cursor and its reported centroid shifts as it lifts, so a touch platform
/// that demanded an exact in-bounds release would discard ordinary taps; a
/// 36 px tall control only tolerates 18 px of travel without this.
const TOUCH_RELEASE_SLOP: f32 = 16.0;

/// A press that landed inside a control and may still activate it from slightly
/// outside. Only touch-shaped input arms one; a mouse keeps the exact
/// drag-away-to-cancel semantics a pointer user expects.
#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
#[derive(Clone, Copy)]
struct ArmedControl {
    action_id: u32,
    bounds: Rect,
}

impl ArmedControl {
    /// Whether a release at this point should still activate the control.
    /// Releases *inside* the declared bounds are deliberately excluded: those
    /// already activate through the control's own click handling, and admitting
    /// them here would deliver the action twice.
    fn rescues(&self, x: f32, y: f32) -> bool {
        // Half-open, matching GPUI's own hit test. Treating the far edge as
        // inside left a one-pixel dead band exactly on it: GPUI declined the
        // click because it was outside, and this declined the rescue because it
        // was inside, so the press reached neither path.
        let inside = x >= self.bounds.x
            && x < self.bounds.x + self.bounds.width
            && y >= self.bounds.y
            && y < self.bounds.y + self.bounds.height;
        if inside {
            return false;
        }
        x >= self.bounds.x - TOUCH_RELEASE_SLOP
            && x <= self.bounds.x + self.bounds.width + TOUCH_RELEASE_SLOP
            && y >= self.bounds.y - TOUCH_RELEASE_SLOP
            && y <= self.bounds.y + self.bounds.height + TOUCH_RELEASE_SLOP
    }
}

#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
#[derive(Clone)]
struct WebSlider {
    control: SliderControl,
    state: Entity<SliderState>,
    synced_ui_revision: Option<u32>,
}

/// Retains one browser text widget per declared field. Focusing it is what
/// legitimately raises a mobile software keyboard: GPUI installs its input
/// handler, and the web backend then moves DOM focus to the editable element.
#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
#[derive(Clone)]
struct WebTextField {
    id: u32,
    state: Entity<InputState>,
}

#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
impl WebFrontplane {
    /// Adapts exact guest integer lattices to GPUI Component step indices while
    /// scheduling semantic values, never toolkit floats, back into WAT.
    fn build_sliders(
        controls: &[SliderControl],
        cx: &mut Context<Self>,
    ) -> (Vec<WebSlider>, Vec<Subscription>) {
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
                    if let Some(value) = subscribed_control.value_at_step(index as u32) {
                        if let Some(slider) = this
                            .sliders
                            .iter_mut()
                            .find(|slider| slider.control.id == subscribed_control.id)
                        {
                            slider.synced_ui_revision = None;
                        }
                        this.queue_event(Event::Control {
                            id: subscribed_control.id,
                            value,
                            phase,
                        });
                        cx.notify();
                    }
                }),
            );
            sliders.push(WebSlider {
                control: control.clone(),
                state,
                synced_ui_revision: None,
            });
        }
        (sliders, subscriptions)
    }

    /// Builds one browser text widget per declared field and forwards every
    /// edit as the adapter-shared scalar sequence, so browser, native, and
    /// headless delivery cannot drift apart.
    fn build_text_fields(
        fields: &[TextField],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (Vec<WebTextField>, Vec<Subscription>) {
        let mut text_fields = Vec::with_capacity(fields.len());
        let mut subscriptions = Vec::with_capacity(fields.len());
        for field in fields {
            let capacity = field.max_scalars as usize;
            let placeholder = field.label.clone();
            let state = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    // The widget shows the bound; the host enforces it again,
                    // because a capability bound may never rest on an
                    // adapter's good behavior.
                    .validate(move |text, _| text.chars().count() <= capacity)
            });
            let id = field.id;
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
                        this.queue_event(event);
                    }
                    cx.notify();
                },
            ));
            text_fields.push(WebTextField { id, state });
        }
        (text_fields, subscriptions)
    }

    fn new(
        runtime: BrowserRuntime,
        origin: Instant,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let shake_interest = runtime
            .metadata()
            .motion_interests
            .iter()
            .any(|interest| interest.kind == MotionInterestKind::ShakeGesture);
        let motion_sample_interval_ms = runtime
            .metadata()
            .motion_interests
            .iter()
            .find(|interest| interest.kind == MotionInterestKind::SixAxisSample)
            .map(|interest| 1000.0 / f64::from(interest.rate_hz.max(1)));
        if shake_interest || motion_sample_interval_ms.is_some() {
            Self::publish_motion_interest();
        }
        let touch_contacts = runtime
            .metadata()
            .touch_max_contacts
            .and_then(TouchContactTracker::new);
        if let Some(max_contacts) = runtime.metadata().touch_max_contacts {
            Self::publish_touch_interest(max_contacts);
        }
        let (sliders, slider_subscriptions) = Self::build_sliders(&runtime.metadata().controls, cx);
        let (text_fields, text_field_subscriptions) =
            Self::build_text_fields(&runtime.metadata().text_fields, window, cx);
        let focus_handle = cx.focus_handle();
        let focus_subscriptions = vec![
            cx.on_focus_in(&focus_handle, window, |this, _, cx| {
                if this.runtime.pause_is_enabled() {
                    this.queue_event(Event::Focus(true));
                    cx.notify();
                }
            }),
            cx.on_focus_out(&focus_handle, window, |this, _, _, cx| {
                this.cancel_touch_contacts();
                if this.runtime.pause_is_enabled() {
                    this.queue_event(Event::Focus(false));
                }
                cx.notify();
            }),
        ];
        Self {
            runtime,
            origin,
            focus_handle,
            reported_delivered_events: 0,
            device_change: DeviceChangeTracker::default(),
            fatal_error: None,
            pressed_pointer_buttons: [0; 3],
            coarse_pointer: primary_pointer_is_coarse(),
            shake_interest,
            motion_sample_interval_ms,
            shake_detector: ShakeDetector::default(),
            last_motion_sample_ms: -f64::INFINITY,
            armed_control: None,
            touch_contacts,
            sliders,
            text_fields,
            _slider_subscriptions: slider_subscriptions,
            _text_field_subscriptions: text_field_subscriptions,
            _focus_subscriptions: focus_subscriptions,
        }
    }

    /// Tells the page a guest registered motion interest so bootstrap can
    /// request the iOS DeviceMotion permission inside a user gesture.
    #[cfg(target_family = "wasm")]
    fn publish_motion_interest() {
        let global = js_sys::global();
        let _ = js_sys::Reflect::set(
            &global,
            &JsValue::from_str("__AEDICULE_MOTION_INTEREST"),
            &JsValue::TRUE,
        );
    }

    #[cfg(not(target_family = "wasm"))]
    fn publish_motion_interest() {}

    #[cfg(target_family = "wasm")]
    fn publish_touch_interest(max_contacts: u32) {
        let _ = js_sys::Reflect::set(
            &js_sys::global(),
            &JsValue::from_str("__AEDICULE_TOUCH_MAX_CONTACTS"),
            &JsValue::from_f64(f64::from(max_contacts)),
        );
    }

    #[cfg(not(target_family = "wasm"))]
    fn publish_touch_interest(_: u32) {}

    fn cancel_touch_contacts(&mut self) {
        let events = self
            .touch_contacts
            .as_mut()
            .map(TouchContactTracker::cancel_all)
            .unwrap_or_default();
        for event in events {
            self.queue_event(event);
        }
    }

    /// Drains the bounded page bridge before the scheduler advances. The pure
    /// tracker validates identity and preserves last-admitted terminal points.
    #[cfg(target_family = "wasm")]
    fn drain_touch_contacts(&mut self) {
        if self.touch_contacts.is_none() {
            return;
        }
        let global = js_sys::global();
        let Ok(queue) =
            js_sys::Reflect::get(&global, &JsValue::from_str("__AEDICULE_TOUCH_EVENTS"))
        else {
            return;
        };
        let Ok(queue) = queue.dyn_into::<js_sys::Array>() else {
            return;
        };
        let field = |entry: &JsValue, name: &str| {
            js_sys::Reflect::get(entry, &JsValue::from_str(name)).ok()
        };
        let mut events = Vec::new();
        for entry in queue.splice(0, queue.length(), &JsValue::UNDEFINED).iter() {
            let phase = field(&entry, "phase")
                .and_then(|value| value.as_string())
                .unwrap_or_default();
            if phase == "cancel-all" {
                events.extend(
                    self.touch_contacts
                        .as_mut()
                        .expect("touch interest was checked")
                        .cancel_all(),
                );
                continue;
            }
            let Some(id) = field(&entry, "id")
                .and_then(|value| value.as_f64())
                .filter(|id| *id >= 0.0 && *id <= f64::from(u32::MAX) && id.fract() == 0.0)
                .map(|id| id as u32)
            else {
                continue;
            };
            let Some(x) = field(&entry, "x").and_then(|value| value.as_f64()) else {
                continue;
            };
            let Some(y) = field(&entry, "y").and_then(|value| value.as_f64()) else {
                continue;
            };
            let phase = match phase.as_str() {
                "start" => TouchContactPhase::Start,
                "move" => TouchContactPhase::Move,
                "end" => TouchContactPhase::End,
                "cancel" => TouchContactPhase::Cancel,
                _ => continue,
            };
            if let Some(event) = self
                .touch_contacts
                .as_mut()
                .expect("touch interest was checked")
                .observe(phase, id, x as f32, y as f32)
            {
                events.push(event);
            }
        }
        for event in events {
            self.queue_event(event);
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn drain_touch_contacts(&mut self) {}

    /// Drains the page-captured devicemotion ring (`__AEDICULE_MOTION_SAMPLES`)
    /// and applies the guest.s registered pacing: the unit-tested ShakeDetector
    /// collapses one physical shake into one gesture, and six-axis samples are
    /// throttled to the registered rate before crossing the guest boundary.
    #[cfg(target_family = "wasm")]
    fn drain_motion_samples(&mut self) {
        if !self.shake_interest && self.motion_sample_interval_ms.is_none() {
            return;
        }
        let global = js_sys::global();
        let Ok(ring) =
            js_sys::Reflect::get(&global, &JsValue::from_str("__AEDICULE_MOTION_SAMPLES"))
        else {
            return;
        };
        let Ok(ring) = ring.dyn_into::<js_sys::Array>() else {
            return;
        };
        if ring.length() == 0 {
            return;
        }
        let field = |sample: &JsValue, name: &str| -> f64 {
            js_sys::Reflect::get(sample, &JsValue::from_str(name))
                .ok()
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0)
        };
        for sample in ring.splice(0, ring.length(), &JsValue::UNDEFINED).iter() {
            let elapsed_ms = field(&sample, "elapsedMs");
            let acceleration = [
                field(&sample, "ax") as f32,
                field(&sample, "ay") as f32,
                field(&sample, "az") as f32,
            ];
            let rotation = [
                field(&sample, "rx") as f32,
                field(&sample, "ry") as f32,
                field(&sample, "rz") as f32,
            ];
            if self.shake_interest {
                let magnitude = (acceleration[0] * acceleration[0]
                    + acceleration[1] * acceleration[1]
                    + acceleration[2] * acceleration[2])
                    .sqrt();
                if let Some(magnitude) = self.shake_detector.observe(elapsed_ms as u64, magnitude)
                {
                    self.queue_event(Event::MotionGesture { magnitude });
                }
            }
            if let Some(interval) = self.motion_sample_interval_ms {
                if elapsed_ms - self.last_motion_sample_ms >= interval {
                    self.last_motion_sample_ms = elapsed_ms;
                    self.queue_event(Event::MotionSample {
                        acceleration,
                        rotation,
                    });
                }
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn drain_motion_samples(&mut self) {}

    /// Samples the browser's monotonic animation timeline; a rendering error
    /// freezes the last complete frame instead of escaping into a paint call.
    fn advance(&mut self) {
        if self.fatal_error.is_some() {
            return;
        }
        match self.runtime.advance_to(self.origin.elapsed()) {
            Ok(_) => {
                let delivered = self.runtime.delivered_event_count();
                if delivered != self.reported_delivered_events {
                    trace_browser_detail("guest", &format!("delivered-events={delivered}"));
                    publish_browser_delivered_events(self.runtime.delivered_event_counts());
                    self.reported_delivered_events = delivered;
                }
                self.play_pending_audio();
            }
            Err(error) => self.fatal_error = Some(error.to_string()),
        }
    }

    /// Hands decoded immutable PCM to Web Audio after the deterministic runtime
    /// has committed its tick; device state never feeds back into guest time.
    fn play_pending_audio(&mut self) {
        let mut pending = self.runtime.drain_audio();
        pending.extend(self.runtime.drain_sample_audio());
        #[cfg(target_family = "wasm")]
        for playback in pending {
            let samples = js_sys::Float32Array::from(playback.samples.as_slice());
            play_browser_pcm(
                playback.sample_rate,
                u32::from(playback.channels),
                &samples,
                playback.volume,
                playback.pitch,
            );
        }
        #[cfg(not(target_family = "wasm"))]
        let _ = pending;
    }

    /// Gives GPUI input a monotonic timestamp before the shared scheduler
    /// decides which fixed-step boundary must observe it.
    fn queue_event(&mut self, event: Event) {
        if self.fatal_error.is_none() {
            trace_browser_input(&event);
            match self.runtime.handle_input(self.origin.elapsed(), event) {
                Ok(BrowserInputOutcome::Paused) => {
                    self.play_pending_audio();
                    set_browser_audio_paused(true);
                }
                Ok(BrowserInputOutcome::Resumed) => {
                    set_browser_audio_paused(false);
                    self.play_pending_audio();
                }
                Ok(
                    BrowserInputOutcome::Queued
                    | BrowserInputOutcome::Rendered
                    | BrowserInputOutcome::Consumed,
                ) => {}
                Err(error) => self.fatal_error = Some(error.to_string()),
            }
        }
    }

    /// Delivers browser keyboard presses through the same physical-key
    /// classifier as native and suppresses browser actions only for admitted
    /// guest keys.
    /// Delivers a control's action when a finger pressed inside it but lifted
    /// just outside. The control's own click handling covers every in-bounds
    /// release and never reaches here, because a control layer occludes the
    /// root; the two paths are disjoint, so an action cannot be delivered twice.
    fn rescue_armed_control(&mut self, event: &MouseUpEvent) {
        let Some(armed) = self.armed_control.take() else {
            return;
        };
        if armed.rescues(f32::from(event.position.x), f32::from(event.position.y)) {
            self.queue_event(Event::MenuAction(armed.action_id));
        }
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
        window.prevent_default();
        cx.stop_propagation();
        if !event.is_held {
            self.queue_event(Event::KeyDown(key));
            cx.notify();
        }
    }

    /// Pairs every admitted browser key press with a release edge so a guest
    /// cannot retain a stuck control after focus remains inside the canvas.
    fn key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.text_entry_has_focus(window, cx) {
            return;
        }
        let Some(key) = Key::from_gpui_name(&event.keystroke.key) else {
            return;
        };
        window.prevent_default();
        cx.stop_propagation();
        self.queue_event(Event::KeyUp(key));
        cx.notify();
    }

    /// Gives browser button presses the same numeric ABI identity as native
    /// GPUI before the shared scheduler orders the edge before a tick.
    fn queue_pointer_down(&mut self, button: PointerButton, event: &MouseDownEvent) {
        let x = f32::from(event.position.x);
        let y = f32::from(event.position.y);
        let index = button as usize - 1;
        self.pressed_pointer_buttons[index] = self.pressed_pointer_buttons[index].saturating_add(1);
        self.queue_event(button.down(x, y));
    }

    /// Delivers the matching browser release edge; GPUI Web itself prevents
    /// `contextmenu` on its event element, so right-click gameplay stays in
    /// this callback rather than opening the browser menu.
    fn queue_pointer_up(&mut self, button: PointerButton, event: &MouseUpEvent) {
        let x = f32::from(event.position.x);
        let y = f32::from(event.position.y);
        let index = button as usize - 1;
        if self.pressed_pointer_buttons[index] == 0 {
            return;
        }
        self.pressed_pointer_buttons[index] -= 1;
        self.queue_event(button.up(x, y));
    }

    /// Preserves precise browser/trackpad pixels and discrete wheel lines as
    /// distinct ABI units while omitting gesture phases with no movement.
    fn queue_pointer_scroll(&mut self, event: &ScrollWheelEvent) {
        let event = match event.delta {
            ScrollDelta::Lines(delta) => PointerScrollUnit::Lines.event(delta.x, delta.y),
            ScrollDelta::Pixels(delta) => {
                PointerScrollUnit::LogicalPixels.event(f32::from(delta.x), f32::from(delta.y))
            }
        };
        if let Some(event) = event {
            self.queue_event(event);
        }
    }

    /// Emits a device-change event only when the logical size or the device
    /// class actually changed, avoiding per-frame guest work while letting a
    /// live pointer-class flip (an iPad gaining a trackpad) emit at constant
    /// size. Re-reads the media query so release-slop arming follows the flip.
    fn observe_device_change(&mut self, window: &Window) {
        let viewport = window.viewport_size();
        self.coarse_pointer = primary_pointer_is_coarse();
        let flags = if self.coarse_pointer {
            DEVICE_FLAG_COARSE_POINTER
        } else {
            0
        };
        if let Some((width, height, flags)) = self.device_change.observe(
            f32::from(viewport.width),
            f32::from(viewport.height),
            flags,
        ) {
            self.queue_event(Event::DeviceChange { width, height, flags });
        }
    }
}

impl Focusable for WebFrontplane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Whether the device's primary input is a finger rather than a pointer. GPUI
/// carries no touch origin on its mouse events, so the release-slop policy is
/// decided per device here instead of per event; a desktop pointer keeps exact
/// drag-away-to-cancel behaviour because this reports false for it.
#[cfg(target_family = "wasm")]
fn primary_pointer_is_coarse() -> bool {
    let global = js_sys::global();
    let Ok(match_media) = js_sys::Reflect::get(&global, &JsValue::from_str("matchMedia")) else {
        return false;
    };
    let Ok(match_media) = match_media.dyn_into::<js_sys::Function>() else {
        return false;
    };
    let Ok(query) = match_media.call1(&global, &JsValue::from_str("(pointer: coarse)")) else {
        return false;
    };
    js_sys::Reflect::get(&query, &JsValue::from_str("matches"))
        .ok()
        .and_then(|matches| matches.as_bool())
        .unwrap_or(false)
}

#[cfg(not(target_family = "wasm"))]
fn primary_pointer_is_coarse() -> bool {
    false
}

/// Claims browser pointer ownership for a native control so its gesture cannot
/// also reach the guest drawing canvas underneath.
fn browser_control_layer(bounds: Rect) -> gpui::Div {
    div()
        .occlude()
        .absolute()
        .left(px(bounds.x))
        .top(px(bounds.y))
        .w(px(bounds.width))
        .h(px(bounds.height))
}

/// Gives browser-rendered guest navigation the same link semantics and trusted
/// click boundary as the native adapter.
fn browser_external_link_control(
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

impl Render for WebFrontplane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        report_browser_frame_started(self.origin.elapsed().as_secs_f64() * 1000.0);
        self.observe_device_change(window);
        self.drain_touch_contacts();
        self.drain_motion_samples();
        self.advance();
        if self.fatal_error.is_none() && !self.runtime.is_suspended() {
            window.request_animation_frame();
        }
        let frame = self.runtime.frame().clone();
        let frame_background = frame.background;
        publish_browser_frame_background(frame.background);
        let fatal_error = self.fatal_error.clone();
        let ui = self.runtime.ui_snapshot().cloned();
        if let Some(ui) = ui.as_ref() {
            publish_browser_ui_snapshot(ui);
        }
        let control_panels = ui
            .iter()
            .flat_map(|snapshot| snapshot.control_panels.iter())
            .map(|panel| {
                browser_control_layer(panel.bounds)
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
                browser_control_layer(placement.bounds)
                    .id(("native-slider", placement.id as usize))
                    .child(content),
            );
        }
        let button_layers = ui
            .iter()
            .flat_map(|snapshot| snapshot.buttons.iter())
            .filter_map(|placement| {
                let action_id = placement.action_id;
                let label = self.runtime.metadata().action_label(action_id)?.to_owned();
                let button = Button::new(("native-button-control", placement.id as usize))
                    .label(label)
                    .selected(placement.selected)
                    .w_full()
                    .h_full()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.queue_event(Event::MenuAction(action_id));
                        cx.notify();
                    }));
                let bounds = placement.bounds;
                Some(
                    browser_control_layer(bounds)
                        .id(("native-button", placement.id as usize))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _: &MouseDownEvent, _, _| {
                                this.armed_control = this
                                    .coarse_pointer
                                    .then_some(ArmedControl { action_id, bounds });
                            }),
                        )
                        .child(button),
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
            .filter_map(|(revision, placement)| {
                let id = placement.id;
                let label = self
                    .runtime
                    .metadata()
                    .external_links
                    .iter()
                    .find(|link| link.id == id)?
                    .label
                    .clone();
                let link = browser_external_link_control(
                    id,
                    label,
                    cx.listener(move |this, _, _, _| {
                        if let Ok(request) = this.runtime.external_link_request(revision, id) {
                            activate_browser_external_link(&request);
                        }
                    }),
                );
                Some(
                    browser_control_layer(placement.bounds)
                        .id(("external-link", placement.id as usize))
                        .child(link),
                )
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
                    browser_control_layer(placement.bounds)
                        .id(("text-field", placement.id as usize))
                        .child(Input::new(&field.state).h_full()),
                )
            })
            .collect::<Vec<_>>();
        let canvas_layer = div()
            .id("aedicule-web-canvas")
            .absolute()
            .inset_0()
            .bg(rgba(frame.background))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                let x = f32::from(event.position.x);
                let y = f32::from(event.position.y);
                this.queue_event(Event::PointerMove {
                    x,
                    y,
                });
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                this.queue_pointer_scroll(event);
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.queue_pointer_down(PointerButton::Primary, event);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.rescue_armed_control(event);
                    this.queue_pointer_up(PointerButton::Primary, event);
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.rescue_armed_control(event);
                    this.queue_pointer_up(PointerButton::Primary, event);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.queue_pointer_down(PointerButton::Secondary, event);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Right,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.queue_pointer_up(PointerButton::Secondary, event);
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Right,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.queue_pointer_up(PointerButton::Secondary, event);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.queue_pointer_down(PointerButton::Middle, event);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.queue_pointer_up(PointerButton::Middle, event);
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Middle,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.queue_pointer_up(PointerButton::Middle, event);
                    cx.notify();
                }),
            )
            .child(
                canvas(
                    move |bounds, _, _| (bounds, frame),
                    move |_, (bounds, frame), window, cx| paint_frame(&frame, bounds, window, cx),
                )
                .size_full(),
            );
        report_browser_frame_completed(
            self.origin.elapsed().as_secs_f64() * 1000.0,
            frame_background,
        );
        div()
            .relative()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            .child(canvas_layer)
            .children(control_panels)
            .children(slider_layers)
            .children(button_layers)
            .children(external_link_layers)
            .children(text_field_layers)
            .when_some(fatal_error, |this, error| {
                this.child(
                    div()
                        .absolute()
                        .inset_4()
                        .p_4()
                        .rounded_lg()
                        .bg(rgba(0x6b1020ee))
                        .text_color(rgba(0xffffffff))
                        .child(format!("Aedicule guest stopped: {error}")),
                )
            })
    }
}

/// Reads the application document that the static JavaScript loader installed
/// before wasm-bindgen invokes this browser entrypoint.
#[cfg(target_family = "wasm")]
fn browser_wat() -> String {
    let source = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str(BROWSER_WAT_GLOBAL))
        .ok()
        .and_then(|value| value.as_string());
    required_browser_wat(source).expect("Aedicule browser bootstrap must load code.wat")
}

/// Copies the bootstrap's bounded typed arrays into the same ordered virtual
/// asset map consumed by native and headless application adapters.
#[cfg(target_family = "wasm")]
fn browser_assets() -> ApplicationAssets {
    let value = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str(BROWSER_ASSETS_GLOBAL))
        .expect("Aedicule browser bootstrap must expose its asset catalog");
    if value.is_null() || value.is_undefined() {
        return ApplicationAssets::new();
    }
    let object = js_sys::Object::from(value);
    let keys = js_sys::Object::keys(&object);
    let mut assets = ApplicationAssets::new();
    for index in 0..keys.length() {
        let name = keys
            .get(index)
            .as_string()
            .expect("Aedicule browser asset name must be UTF-8");
        let bytes = js_sys::Uint8Array::new(
            &js_sys::Reflect::get(&object, &JsValue::from_str(&name))
                .expect("Aedicule browser asset must exist"),
        )
        .to_vec();
        assets.insert(name, bytes);
    }
    assets
}

/// Reads a visitor-selected `.aed` package if the bootstrap supplied one. The
/// bytes cross the boundary unparsed; the shared Rust archive reader is the
/// only `.aed` implementation, so the browser cannot diverge from native.
#[cfg(target_family = "wasm")]
fn browser_package() -> Option<Vec<u8>> {
    let value = js_sys::Reflect::get(
        &js_sys::global(),
        &JsValue::from_str(aedicule::web::BROWSER_PACKAGE_GLOBAL),
    )
    .ok()?;
    if value.is_null() || value.is_undefined() {
        return None;
    }
    Some(js_sys::Uint8Array::new(&value).to_vec())
}

#[cfg(target_family = "wasm")]
fn main() {
    gpui_platform::web_init();
    let (wat, assets) = match browser_package() {
        Some(package) => aedicule::web::browser_application_from_package(package)
            .expect("Aedicule browser package expands to a guest and its assets"),
        None => (browser_wat(), browser_assets()),
    };
    let origin = Instant::now();
    let runtime = BrowserRuntime::new_with_assets(
        &wat,
        DEFAULT_PLUGIN_INIT,
        std::time::Duration::ZERO,
        assets,
    )
    .expect("Aedicule browser code.wat initializes and renders");

    let application = gpui_platform::single_threaded_web()
        .with_assets(gpui_component_assets::Assets::default())
        .run_embedded(move |cx: &mut App| {
            gpui_component::init(cx);
            cx.text_system()
                .add_fonts(vec![Cow::Borrowed(GEIST_MONO_REGULAR)])
                .expect("register bundled Geist Mono Regular");
            let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    Theme::change(ThemeMode::Dark, Some(window), cx);
                    let view = cx.new(|cx| WebFrontplane::new(runtime, origin, window, cx));
                    view.focus_handle(cx).focus(window, cx);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("open Aedicule browser window");
        });
    retain_browser_application(application);
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    panic!("aedicule-web must be compiled for wasm32-unknown-unknown");
}
