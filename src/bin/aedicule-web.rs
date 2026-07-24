//! Browser GPUI entrypoint for the portable, fuel-metered Aedicule runtime.

#[cfg(target_family = "wasm")]
use std::{borrow::Cow, cell::OnceCell};

#[cfg_attr(not(target_family = "wasm"), allow(unused_imports))]
use aedicule::{
    ApplicationAssets, ControlLabelPlacement, ControlPhase, Event, ExternalLinkRequest,
    GEIST_MONO_REGULAR, Key, PluginInit, PointerButton, PointerScrollUnit, Rect, SliderControl,
    UiSnapshot,
    gpui_canvas::paint_frame,
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
    ] {
        set_js_property(&diagnostic, name, &JsValue::from_f64(value as f64));
    }
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
    set_js_property(&diagnostic, "panels", &panels);
    set_js_property(&diagnostic, "sliders", &sliders);
    set_js_property(&diagnostic, "buttons", &buttons);
    set_js_property(&diagnostic, "externalLinks", &external_links);
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
    viewport_bits: Option<(u32, u32)>,
    fatal_error: Option<String>,
    pressed_pointer_buttons: [u16; 3],
    sliders: Vec<WebSlider>,
    _slider_subscriptions: Vec<Subscription>,
    _focus_subscriptions: Vec<Subscription>,
}

#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
#[derive(Clone)]
struct WebSlider {
    control: SliderControl,
    state: Entity<SliderState>,
    synced_ui_revision: Option<u32>,
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

    fn new(
        runtime: BrowserRuntime,
        origin: Instant,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let (sliders, slider_subscriptions) = Self::build_sliders(&runtime.metadata().controls, cx);
        let focus_handle = cx.focus_handle();
        let focus_subscriptions = vec![
            cx.on_focus_in(&focus_handle, window, |this, _, cx| {
                if this.runtime.pause_is_enabled() {
                    this.queue_event(Event::Focus(true));
                    cx.notify();
                }
            }),
            cx.on_focus_out(&focus_handle, window, |this, _, _, cx| {
                if this.runtime.pause_is_enabled() {
                    this.queue_event(Event::Focus(false));
                    cx.notify();
                }
            }),
        ];
        Self {
            runtime,
            origin,
            focus_handle,
            reported_delivered_events: 0,
            viewport_bits: None,
            fatal_error: None,
            pressed_pointer_buttons: [0; 3],
            sliders,
            _slider_subscriptions: slider_subscriptions,
            _focus_subscriptions: focus_subscriptions,
        }
    }

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
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
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
        let index = button as usize - 1;
        self.pressed_pointer_buttons[index] = self.pressed_pointer_buttons[index].saturating_add(1);
        self.queue_event(button.down(f32::from(event.position.x), f32::from(event.position.y)));
    }

    /// Delivers the matching browser release edge; GPUI Web itself prevents
    /// `contextmenu` on its event element, so right-click gameplay stays in
    /// this callback rather than opening the browser menu.
    fn queue_pointer_up(&mut self, button: PointerButton, event: &MouseUpEvent) {
        let index = button as usize - 1;
        if self.pressed_pointer_buttons[index] == 0 {
            return;
        }
        self.pressed_pointer_buttons[index] -= 1;
        self.queue_event(button.up(f32::from(event.position.x), f32::from(event.position.y)));
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

    /// Emits a viewport event only for an actual logical-size change, avoiding
    /// per-frame guest work while retaining resize semantics.
    fn observe_viewport(&mut self, window: &Window) {
        let viewport = window.viewport_size();
        let width = f32::from(viewport.width);
        let height = f32::from(viewport.height);
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return;
        }
        let bits = (width.to_bits(), height.to_bits());
        if self.viewport_bits == Some(bits) {
            return;
        }
        self.viewport_bits = Some(bits);
        self.queue_event(Event::Viewport { width, height });
    }
}

impl Focusable for WebFrontplane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
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
        self.observe_viewport(window);
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
                let label = self
                    .runtime
                    .metadata()
                    .menu_items
                    .iter()
                    .find(|item| item.id == Some(action_id))?
                    .label
                    .clone();
                let button = Button::new(("native-button-control", placement.id as usize))
                    .label(label)
                    .selected(placement.selected)
                    .w_full()
                    .h_full()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.queue_event(Event::MenuAction(action_id));
                        cx.notify();
                    }));
                Some(
                    browser_control_layer(placement.bounds)
                        .id(("native-button", placement.id as usize))
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
        let canvas_layer = div()
            .id("aedicule-web-canvas")
            .absolute()
            .inset_0()
            .bg(rgba(frame.background))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                this.queue_event(Event::PointerMove {
                    x: f32::from(event.position.x),
                    y: f32::from(event.position.y),
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
                    this.queue_pointer_up(PointerButton::Primary, event);
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
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

#[cfg(target_family = "wasm")]
fn main() {
    gpui_platform::web_init();
    let wat = browser_wat();
    let assets = browser_assets();
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
