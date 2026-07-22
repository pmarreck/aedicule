//! Browser-only delivery helpers that keep the application WAT outside the
//! compiled host while preserving the ordinary Aedicule initialization path.

use std::time::Duration;

use crate::{
    Event, FrameOutput, Frontplane, FrontplaneError, Limits, PluginInit, SimulationCall,
    SimulationScheduler, initialize_frontplane,
};

const MAX_TICKS_PER_BROWSER_FRAME: u32 = 8;

/// Name of the JavaScript global populated by the static delivery bootstrap
/// before wasm-bindgen enters the GPUI application.
pub const BROWSER_WAT_GLOBAL: &str = "__AEDICULE_WAT";

/// Requires a non-empty WAT document supplied by the delivery bootstrap.
///
/// A web bundle must never silently substitute the host's fallback guest:
/// doing so would make a missing application asset look like a valid launch.
pub fn required_browser_wat(source: Option<String>) -> Result<String, &'static str> {
    source
        .filter(|source| !source.trim().is_empty())
        .ok_or("missing browser code.wat")
}

/// Owns the browser adapter's deterministic guest timeline independently of
/// GPUI callbacks. Browser animation frames merely supply monotonic elapsed
/// time; this controller preserves native input-before-tick ordering and
/// bounded catch-up behavior.
pub struct BrowserRuntime {
    frontplane: Frontplane,
    scheduler: SimulationScheduler,
    frame: FrameOutput,
}

impl BrowserRuntime {
    /// Builds the browser guest through the shared lifecycle before any visual
    /// callback can see it, then anchors its rational scheduler at `origin`.
    pub fn new(wat: &str, init: PluginInit, origin: Duration) -> Result<Self, FrontplaneError> {
        let mut frontplane = Frontplane::from_wat(wat, Limits::default())?;
        initialize_frontplane(&mut frontplane, init)?;
        let frame = frontplane.render()?;
        frontplane.drain_audio();
        frontplane.drain_effects();
        let scheduler = SimulationScheduler::new(
            frontplane.simulation_rate(),
            MAX_TICKS_PER_BROWSER_FRAME,
            origin,
        );
        Ok(Self {
            frontplane,
            scheduler,
            frame,
        })
    }

    /// Stamps browser input in the same monotonic timeline that classifies the
    /// next simulation boundary; the event is delivered during `advance_to`.
    pub fn queue_event(&mut self, timestamp: Duration, event: Event) {
        self.scheduler.queue_event(timestamp, event);
    }

    /// Executes one animation-frame sample. Unsupported browser audio and
    /// effects are drained until their dedicated adapters exist, preserving the
    /// runtime's bounded output queues without inventing JavaScript behavior.
    pub fn advance_to(&mut self, now: Duration) -> Result<bool, FrontplaneError> {
        let pump = self.scheduler.pump(now);
        for call in pump.calls {
            match call {
                SimulationCall::Event(event) => self.frontplane.event(event)?,
                SimulationCall::Tick(ticks) => self.frontplane.tick(ticks)?,
            }
        }
        if pump.render {
            self.frame = self.frontplane.render()?;
        }
        self.frontplane.drain_audio();
        self.frontplane.drain_effects();
        Ok(pump.render || pump.dropped_ticks > 0)
    }

    /// Exposes only the last complete validated command frame to GPUI's paint
    /// callback, keeping guest execution outside paint replay.
    pub fn frame(&self) -> &FrameOutput {
        &self.frame
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{DrawCommand, Event, PluginInit};

    use super::{BrowserRuntime, required_browser_wat};

    const POINTER_WAT: &str = r#"
        (module
            (import "aedicule.v0" "AE_frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
            (import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
            (memory (export "memory") 1)
            (global $x (mut f32) (f32.const 10))
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) i32.const 0)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param $kind i32) (param i32) (param $x_event f32) (param f32) (result i32)
                local.get $kind i32.const 3 i32.eq
                if local.get $x_event global.set $x end
                i32.const 0)
            (func (export "AE_tick") (param i32) (result i32) i32.const 0)
            (func (export "AE_tick_rate") (param i32 i32) (result i32 i32) i32.const 60 i32.const 1)
            (func (export "AE_render") (result i32)
                f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $frame_begin drop
                i32.const 1 global.get $x f32.const 10 f32.const 4 f32.const 1 i32.const 0xffffffff i32.const 1 call $circle drop
                call $frame_end drop
                i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 0)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1))
    "#;

    const INPUT_AND_TICK_WAT: &str = r#"
        (module
            (import "aedicule.v0" "AE_frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
            (import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
            (memory (export "memory") 1)
            (global $x (mut f32) (f32.const 100))
            (global $y (mut f32) (f32.const 0))
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) i32.const 0)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param $kind i32) (param $code i32) (param f32 f32) (result i32)
                local.get $kind i32.const 1 i32.eq
                local.get $code i32.const 1 i32.eq
                i32.and
                if
                    global.get $x f32.const 50 f32.sub global.set $x
                end
                i32.const 0)
            (func (export "AE_tick") (param $ticks i32) (result i32)
                global.get $y
                local.get $ticks f32.convert_i32_u f32.const 10 f32.mul
                f32.add global.set $y
                i32.const 0)
            (func (export "AE_tick_rate") (param i32 i32) (result i32 i32) i32.const 60 i32.const 1)
            (func (export "AE_render") (result i32)
                f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $frame_begin drop
                i32.const 1 global.get $x global.get $y f32.const 4 f32.const 1 i32.const 0xffffffff i32.const 1 call $circle drop
                call $frame_end drop
                i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 0)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1))
    "#;

    #[test]
    fn accepts_the_external_wat_document_without_substitution() {
        let source = "(module (memory (export \"memory\") 1))".to_owned();

        assert_eq!(required_browser_wat(Some(source.clone())), Ok(source));
    }

    #[test]
    fn rejects_missing_or_blank_external_wat_documents() {
        assert_eq!(required_browser_wat(None), Err("missing browser code.wat"));
        assert_eq!(
            required_browser_wat(Some(" \n\t".to_owned())),
            Err("missing browser code.wat")
        );
    }

    #[test]
    fn delivers_pointer_input_at_the_next_exact_browser_tick_boundary() {
        let mut runtime = BrowserRuntime::new(
            POINTER_WAT,
            PluginInit::new(7, 1024.0, 768.0),
            Duration::ZERO,
        )
        .unwrap();
        runtime.queue_event(
            Duration::from_millis(5),
            Event::PointerMove { x: 500.0, y: 10.0 },
        );

        assert!(!runtime.advance_to(Duration::from_millis(16)).unwrap());
        assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());

        let Some(DrawCommand::Circle { x, .. }) = runtime.frame().commands.first() else {
            panic!("expected a pointer-controlled circle");
        };
        assert_eq!(*x, 500.0);
    }

    #[test]
    fn synthesizes_keyboard_edges_before_their_following_game_update() {
        let mut runtime = BrowserRuntime::new(
            INPUT_AND_TICK_WAT,
            PluginInit::new(7, 1024.0, 768.0),
            Duration::ZERO,
        )
        .unwrap();
        runtime.queue_event(
            Duration::from_millis(5),
            Event::KeyDown(crate::Key::ArrowLeft),
        );

        assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());

        let Some(DrawCommand::Circle { x, y, .. }) = runtime.frame().commands.first() else {
            panic!("expected a keyboard-and-tick-controlled circle");
        };
        assert_eq!((*x, *y), (50.0, 10.0));
    }
}
