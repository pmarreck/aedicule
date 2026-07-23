//! Browser-only delivery helpers that keep the application WAT outside the
//! compiled host while preserving the ordinary Aedicule initialization path.

use std::time::Duration;

use crate::{
    ApplicationAssets, Event, FrameOutput, Frontplane, FrontplaneError, Limits, Metadata,
    PluginInit, SimulationCall, SimulationScheduler, UiSnapshot, initialize_frontplane,
    wav::AUDIO_FIXED_SCALE,
};

const MAX_TICKS_PER_BROWSER_FRAME: u32 = 8;

/// Name of the JavaScript global populated by the static delivery bootstrap
/// before wasm-bindgen enters the GPUI application.
pub const BROWSER_WAT_GLOBAL: &str = "__AEDICULE_WAT";
/// Name of the JavaScript object populated with validated virtual asset bytes.
pub const BROWSER_ASSETS_GLOBAL: &str = "__AEDICULE_ASSETS";

/// One fully decoded, bounded sample request ready for the browser's Web Audio
/// device adapter; deterministic guest execution observes no playback state.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserSamplePlayback {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub volume: f32,
    pub pitch: f32,
}

/// Separates semantic AVP delivery from raw canvas input so browser tests can
/// prove controls neither vanish nor leak their gestures through the canvas.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BrowserDeliveredEventCounts {
    pub total: u64,
    pub controls: u64,
    pub menu_actions: u64,
    pub pointer: u64,
}

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
    delivered_events: BrowserDeliveredEventCounts,
}

impl BrowserRuntime {
    /// Builds the browser guest through the shared lifecycle before any visual
    /// callback can see it, then anchors its rational scheduler at `origin`.
    pub fn new(wat: &str, init: PluginInit, origin: Duration) -> Result<Self, FrontplaneError> {
        Self::new_with_assets(wat, init, origin, ApplicationAssets::new())
    }

    /// Builds the browser guest with the same immutable virtual asset catalog
    /// used by native and headless adapters before configuration begins.
    pub fn new_with_assets(
        wat: &str,
        init: PluginInit,
        origin: Duration,
        assets: ApplicationAssets,
    ) -> Result<Self, FrontplaneError> {
        let mut frontplane = Frontplane::from_wat_with_assets(wat, Limits::default(), assets)?;
        initialize_frontplane(&mut frontplane, init)?;
        let frame = frontplane.render()?;
        frontplane.drain_audio();
        frontplane.drain_sample_audio();
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
            delivered_events: BrowserDeliveredEventCounts::default(),
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
                SimulationCall::Event(event) => {
                    match &event {
                        Event::Control { .. } => {
                            self.delivered_events.controls =
                                self.delivered_events.controls.saturating_add(1);
                        }
                        Event::MenuAction(_) => {
                            self.delivered_events.menu_actions =
                                self.delivered_events.menu_actions.saturating_add(1);
                        }
                        Event::PointerMove { .. }
                        | Event::PointerDown { .. }
                        | Event::PointerUp { .. }
                        | Event::PointerScroll { .. } => {
                            self.delivered_events.pointer =
                                self.delivered_events.pointer.saturating_add(1);
                        }
                        _ => {}
                    }
                    self.frontplane.event(event)?;
                    self.delivered_events.total = self.delivered_events.total.saturating_add(1);
                }
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

    /// Exposes immutable configure-time declarations to the browser adapter so
    /// it can render the same guest-authored controls as native.
    pub fn metadata(&self) -> &Metadata {
        self.frontplane.metadata()
    }

    /// Exposes the last transactionally accepted UI document; browser layout
    /// must never render a partially submitted guest revision.
    pub fn ui_snapshot(&self) -> Option<&UiSnapshot> {
        self.frontplane.ui_snapshot()
    }

    /// Reports guest-accepted browser events for opt-in adapter diagnostics;
    /// DOM dispatch alone does not prove an edge crossed the WAT boundary.
    pub fn delivered_event_count(&self) -> u64 {
        self.delivered_events.total
    }

    /// Exposes semantic-versus-pointer delivery counters for browser adapter
    /// diagnostics without revealing or mutating guest state.
    pub fn delivered_event_counts(&self) -> BrowserDeliveredEventCounts {
        self.delivered_events
    }

    /// Resolves transactional sample events against immutable admitted metadata
    /// and converts fixed PCM only at the browser-device boundary.
    pub fn drain_sample_audio(&mut self) -> Vec<BrowserSamplePlayback> {
        let events = self.frontplane.drain_sample_audio();
        events
            .into_iter()
            .map(|event| {
                let sample = self
                    .frontplane
                    .metadata()
                    .sample_assets
                    .iter()
                    .find(|sample| sample.id == event.id)
                    .expect("accepted sample event must reference an admitted asset");
                BrowserSamplePlayback {
                    sample_rate: sample.clip.sample_rate,
                    channels: sample.clip.channels,
                    samples: sample
                        .clip
                        .samples
                        .iter()
                        .map(|sample| *sample as f32 / AUDIO_FIXED_SCALE as f32)
                        .collect(),
                    volume: event.volume,
                    pitch: event.pitch,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use crate::{ApplicationAssets, DrawCommand, Event, PluginInit};

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

    const SILENT_FLAC: &[u8] = &[
        102, 76, 97, 67, 0, 0, 0, 34, 16, 0, 16, 0, 0, 0, 15, 0, 0, 15, 1, 244, 2, 240, 0, 0, 0, 8,
        112, 188, 143, 75, 114, 168, 105, 33, 70, 139, 248, 232, 68, 29, 206, 81, 132, 0, 0, 40,
        32, 0, 0, 0, 114, 101, 102, 101, 114, 101, 110, 99, 101, 32, 108, 105, 98, 70, 76, 65, 67,
        32, 49, 46, 53, 46, 48, 32, 50, 48, 50, 53, 48, 50, 49, 49, 0, 0, 0, 0, 255, 248, 100, 24,
        0, 7, 84, 0, 0, 0, 0, 0, 0, 140, 21,
    ];

    const SAMPLE_WAT: &str = r#"
		(module
			(import "aedicule.v0" "AE_sample_asset"
				(func $sample_asset (param i32 i32 i32 i32) (result i32)))
			(import "aedicule.v0" "AE_sample_play"
				(func $sample_play (param i32 f32 f32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_begin_rgba"
				(func $frame_begin (param i32) (result i32)))
			(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
			(memory (export "memory") 1)
			(data (i32.const 0) "assets/audio/sample.flac")
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 1)
			(func (export "AE_configure") (result i32)
				i32.const 77 i32.const 0 i32.const 24 i32.const 0 call $sample_asset)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_tick") (param i32) (result i32)
				i32.const 77 f32.const 0.5 f32.const 1.25 i32.const 0 call $sample_play drop
				i32.const 0)
			(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
				i32.const 60 i32.const 1)
			(func (export "AE_render") (result i32)
				i32.const 255 call $frame_begin drop call $frame_end drop i32.const 0)
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
        assert_eq!(runtime.delivered_event_count(), 1);
        assert_eq!(runtime.delivered_event_counts().pointer, 1);

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
        assert_eq!(runtime.delivered_event_counts().pointer, 0);

        let Some(DrawCommand::Circle { x, y, .. }) = runtime.frame().commands.first() else {
            panic!("expected a keyboard-and-tick-controlled circle");
        };
        assert_eq!((*x, *y), (50.0, 10.0));
    }

    #[test]
    fn admitted_flac_assets_become_browser_pcm_playback_requests() {
        let assets: ApplicationAssets =
            BTreeMap::from([("assets/audio/sample.flac".to_owned(), SILENT_FLAC.to_vec())]);
        let mut runtime = BrowserRuntime::new_with_assets(
            SAMPLE_WAT,
            PluginInit::new(7, 1024.0, 768.0),
            Duration::ZERO,
            assets,
        )
        .unwrap();

        assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());
        let playback = runtime.drain_sample_audio();
        assert_eq!(playback.len(), 1);
        assert_eq!(playback[0].sample_rate, 8_000);
        assert_eq!(playback[0].channels, 2);
        assert_eq!(playback[0].samples, vec![0.0; 16]);
        assert_eq!(playback[0].volume, 0.5);
        assert_eq!(playback[0].pitch, 1.25);
    }
}
