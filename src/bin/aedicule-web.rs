//! Browser GPUI entrypoint for the portable, fuel-metered Aedicule runtime.

#[cfg(target_family = "wasm")]
use std::{borrow::Cow, cell::OnceCell};

#[cfg_attr(not(target_family = "wasm"), allow(unused_imports))]
use gpui::{
    App, AppContext as _, Bounds, Context, InteractiveElement as _, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Render, ScrollDelta,
    ScrollWheelEvent, Styled as _, Window, WindowBounds, WindowOptions, canvas, div,
    prelude::FluentBuilder as _, px, rgba, size,
};
#[cfg_attr(not(target_family = "wasm"), allow(unused_imports))]
use gpui_wasm::{
    Event, GEIST_MONO_REGULAR, PluginInit, PointerButton, PointerScrollUnit,
    gpui_canvas::paint_frame,
    web::{BROWSER_WAT_GLOBAL, BrowserRuntime, required_browser_wat},
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::JsValue;
use web_time::Instant;

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

/// Bridges GPUI Web animation/input callbacks to Aedicule's tested rational
/// scheduler, keeping guest execution outside the canvas paint callback.
#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
struct WebFrontplane {
    runtime: BrowserRuntime,
    origin: Instant,
    viewport_bits: Option<(u32, u32)>,
    fatal_error: Option<String>,
}

#[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
impl WebFrontplane {
    fn new(runtime: BrowserRuntime, origin: Instant) -> Self {
        Self {
            runtime,
            origin,
            viewport_bits: None,
            fatal_error: None,
        }
    }

    /// Samples the browser's monotonic animation timeline; a rendering error
    /// freezes the last complete frame instead of escaping into a paint call.
    fn advance(&mut self) {
        if self.fatal_error.is_some() {
            return;
        }
        if let Err(error) = self.runtime.advance_to(self.origin.elapsed()) {
            self.fatal_error = Some(error.to_string());
        }
    }

    /// Gives GPUI input a monotonic timestamp before the shared scheduler
    /// decides which fixed-step boundary must observe it.
    fn queue_event(&mut self, event: Event) {
        if self.fatal_error.is_none() {
            self.runtime.queue_event(self.origin.elapsed(), event);
        }
    }

    /// Gives browser button presses the same numeric ABI identity as native
    /// GPUI before the shared scheduler orders the edge before a tick.
    fn queue_pointer_down(&mut self, button: PointerButton, event: &MouseDownEvent) {
        self.queue_event(button.down(f32::from(event.position.x), f32::from(event.position.y)));
    }

    /// Delivers the matching browser release edge; GPUI Web itself prevents
    /// `contextmenu` on its event element, so right-click gameplay stays in
    /// this callback rather than opening the browser menu.
    fn queue_pointer_up(&mut self, button: PointerButton, event: &MouseUpEvent) {
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

impl Render for WebFrontplane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.observe_viewport(window);
        self.advance();
        if self.fatal_error.is_none() {
            window.request_animation_frame();
        }
        let frame = self.runtime.frame().clone();
        let fatal_error = self.fatal_error.clone();
        div()
            .id("aedicule-web-canvas")
            .size_full()
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

#[cfg(target_family = "wasm")]
fn main() {
    gpui_platform::web_init();
    let wat = browser_wat();
    let origin = Instant::now();
    let runtime = BrowserRuntime::new(&wat, DEFAULT_PLUGIN_INIT, std::time::Duration::ZERO)
        .expect("Aedicule browser code.wat initializes and renders");

    let application = gpui_platform::single_threaded_web().run_embedded(move |cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(GEIST_MONO_REGULAR)])
            .expect("register bundled Geist Mono Regular");
        let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| WebFrontplane::new(runtime, origin)),
        )
        .expect("open Aedicule browser window");
    });
    retain_browser_application(application);
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    panic!("aedicule-web must be compiled for wasm32-unknown-unknown");
}
