//! Browser-only delivery helpers that keep the application WAT outside the
//! compiled host while preserving the ordinary Aedicule initialization path.

use std::time::Duration;

use crate::{
    ApplicationAssets, AudioEvent, Event, FrameOutput, Frontplane, FrontplaneError,
    GuestSuspension, Limits, Metadata, PausePhase, PluginInit, SampleAudioEvent, SimulationCall,
    SimulationScheduler, SuspensionDisposition, UiSnapshot, audio::render_synth_program,
    initialize_frontplane, wav::AUDIO_FIXED_SCALE,
};

const MAX_TICKS_PER_BROWSER_FRAME: u32 = 8;
const BROWSER_SYNTH_SAMPLE_RATE: u32 = 48_000;

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
	pub last_menu_action_id: Option<u32>,
	pub pointer: u64,
	pub touch: u64,
	pub motion_gestures: u64,
}

/// Reports whether one browser edge entered the fixed-step queue, was consumed,
/// or completed a host-owned suspension transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserInputOutcome {
    Queued,
    Rendered,
    Consumed,
    Paused,
    Resumed,
}

/// Name of the JavaScript global holding a visitor-selected `.aed` package as
/// raw bytes. The bootstrap performs no archive parsing of its own: handing
/// the whole file across this boundary is what lets the single Rust `.aed`
/// implementation serve the browser, replacing the JavaScript zip reader whose
/// stored-entries-only support silently diverged once packages compressed.
pub const BROWSER_PACKAGE_GLOBAL: &str = "__AEDICULE_AED";

/// Expands a visitor-selected `.aed` into the guest WAT and its bounded asset
/// catalog using the same validated archive reader as every native adapter.
/// Non-asset entries (tests, documentation, license) ride in the package for
/// humans but never become runtime-visible virtual files.
pub fn browser_application_from_package(
    bytes: Vec<u8>,
) -> Result<(String, ApplicationAssets), String> {
    let mut entries = crate::read_application_archive(bytes)?;
    let wat = entries
        .remove(crate::DEFAULT_PLUGIN_FILE)
        .ok_or_else(|| format!("package lacks {}", crate::DEFAULT_PLUGIN_FILE))?;
    let wat = String::from_utf8(wat)
        .map_err(|_| format!("package {} is not UTF-8", crate::DEFAULT_PLUGIN_FILE))?;
    entries.retain(|name, _| crate::package::is_valid_asset_name(name));
    Ok((wat, entries))
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
    suspension: GuestSuspension,
    paused_at: Option<Duration>,
    pending_audio: Vec<AudioEvent>,
    pending_sample_audio: Vec<SampleAudioEvent>,
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
        let suspension = GuestSuspension::new(frontplane.metadata().pause_triggers.iter().copied());
        Ok(Self {
            frontplane,
            scheduler,
            frame,
            delivered_events: BrowserDeliveredEventCounts::default(),
            suspension,
            paused_at: None,
            pending_audio: Vec::new(),
            pending_sample_audio: Vec::new(),
        })
    }

    /// Lets scheduler-ordering tests enqueue ordinary input without exposing a
    /// production bypass around host-owned suspension classification.
    #[cfg(test)]
    fn queue_event(&mut self, timestamp: Duration, event: Event) {
        self.scheduler.queue_event(timestamp, event);
    }

    /// Converts configured raw triggers into semantic pause phases while
    /// preserving ordinary input scheduling for guests that declare none.
    pub fn handle_input(
        &mut self,
        timestamp: Duration,
        event: Event,
    ) -> Result<BrowserInputOutcome, FrontplaneError> {
        match self.suspension.handle(event) {
            SuspensionDisposition::Deliver(event) => {
                self.scheduler.queue_event(timestamp, event);
                Ok(BrowserInputOutcome::Queued)
            }
            SuspensionDisposition::Maintenance(event) => {
                self.deliver_event(event)?;
                self.frame = self.frontplane.render()?;
                self.discard_transition_outputs();
                Ok(BrowserInputOutcome::Rendered)
            }
            SuspensionDisposition::Consumed => Ok(BrowserInputOutcome::Consumed),
            SuspensionDisposition::Pause => {
                self.advance_scheduler_to(timestamp)?;
                for event in self.scheduler.drain_events_through(timestamp) {
                    self.deliver_event(event)?;
                }
                self.collect_runtime_outputs();
                self.deliver_event(Event::Pause(PausePhase::Paused))?;
                self.frame = self.frontplane.render()?;
                self.discard_transition_outputs();
                self.paused_at = Some(timestamp);
                Ok(BrowserInputOutcome::Paused)
            }
            SuspensionDisposition::Resume { reconciliation } => {
                let paused_at = self
                    .paused_at
                    .take()
                    .expect("resume disposition follows a completed pause");
                self.scheduler.resume_after_pause(paused_at, timestamp);
                for event in reconciliation {
                    self.deliver_event(event)?;
                }
                self.deliver_event(Event::Pause(PausePhase::Resumed))?;
                self.frame = self.frontplane.render()?;
                self.collect_runtime_outputs();
                Ok(BrowserInputOutcome::Resumed)
            }
        }
    }

    /// Executes one animation-frame sample, retaining committed audio for the
    /// browser adapter while draining effects that have no browser device yet.
    pub fn advance_to(&mut self, now: Duration) -> Result<bool, FrontplaneError> {
        if self.suspension.is_suspended() {
            return Ok(false);
        }
        self.advance_scheduler_to(now)
    }

    fn advance_scheduler_to(&mut self, now: Duration) -> Result<bool, FrontplaneError> {
        let pump = self.scheduler.pump(now);
        for call in pump.calls {
            match call {
                SimulationCall::Event(event) => self.deliver_event(event)?,
                SimulationCall::Tick(ticks) => self.frontplane.tick(ticks)?,
            }
        }
        if pump.render {
            self.frame = self.frontplane.render()?;
        }
        self.collect_runtime_outputs();
        Ok(pump.render || pump.dropped_ticks > 0)
    }

    fn deliver_event(&mut self, event: Event) -> Result<(), FrontplaneError> {
        match &event {
            Event::Control { .. } => {
                self.delivered_events.controls = self.delivered_events.controls.saturating_add(1);
            }
            Event::MenuAction(action_id) => {
                self.delivered_events.menu_actions =
                    self.delivered_events.menu_actions.saturating_add(1);
                self.delivered_events.last_menu_action_id = Some(*action_id);
            }
			Event::PointerMove { .. }
			| Event::PointerDown { .. }
			| Event::PointerUp { .. }
			| Event::PointerScroll { .. } => {
				self.delivered_events.pointer = self.delivered_events.pointer.saturating_add(1);
			}
			Event::TouchStart { .. }
			| Event::TouchMove { .. }
			| Event::TouchEnd { .. }
			| Event::TouchCancel { .. } => {
				self.delivered_events.touch = self.delivered_events.touch.saturating_add(1);
			}
			Event::MotionGesture { .. } => {
				self.delivered_events.motion_gestures =
					self.delivered_events.motion_gestures.saturating_add(1);
			}
            _ => {}
        }
        self.frontplane.event(event)?;
        self.delivered_events.total = self.delivered_events.total.saturating_add(1);
        Ok(())
    }

    fn collect_runtime_outputs(&mut self) {
        self.pending_audio.extend(self.frontplane.drain_audio());
        self.pending_sample_audio
            .extend(self.frontplane.drain_sample_audio());
        self.frontplane.drain_effects();
    }

    fn discard_transition_outputs(&mut self) {
        self.frontplane.drain_audio();
        self.frontplane.drain_sample_audio();
        self.frontplane.drain_effects();
    }

    /// Exposes only the last complete validated command frame to GPUI's paint
    /// callback, keeping guest execution outside paint replay.
    pub fn frame(&self) -> &FrameOutput {
        &self.frame
    }

    pub fn is_suspended(&self) -> bool {
        self.suspension.is_suspended()
    }

    pub fn pause_is_enabled(&self) -> bool {
        self.suspension.is_enabled()
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

    /// Resolves browser navigation only from an exact accepted retained-view
    /// revision; the adapter still decides whether a trusted click may open it.
    pub fn external_link_request(
        &self,
        revision: u32,
        id: u32,
    ) -> Result<crate::ExternalLinkRequest, crate::FrontplaneError> {
        self.frontplane.external_link_request(revision, id)
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

    /// Renders committed synth events through the same deterministic fixed
    /// point core as native, handing only bounded mono PCM to Web Audio.
    pub fn drain_audio(&mut self) -> Vec<BrowserSamplePlayback> {
        std::mem::take(&mut self.pending_audio)
            .into_iter()
            .filter_map(|event| {
                let voices = self
                    .frontplane
                    .metadata()
                    .synth_voices
                    .iter()
                    .filter(|voice| voice.program_id == event.id)
                    .cloned()
                    .collect::<Vec<_>>();
                let samples = render_synth_program(&voices, event, BROWSER_SYNTH_SAMPLE_RATE);
                (!samples.is_empty()).then_some(BrowserSamplePlayback {
                    sample_rate: BROWSER_SYNTH_SAMPLE_RATE,
                    channels: 1,
                    samples,
                    volume: 1.0,
                    pitch: 1.0,
                })
            })
            .collect()
    }

    /// Resolves transactional sample events against immutable admitted metadata
    /// and converts fixed PCM only at the browser-device boundary.
    pub fn drain_sample_audio(&mut self) -> Vec<BrowserSamplePlayback> {
        let events = std::mem::take(&mut self.pending_sample_audio);
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

    use super::{BrowserInputOutcome, BrowserRuntime, required_browser_wat};

    const PHYSICAL_KEY_WAT: &str = include_str!("../tests/fixtures/browser_synth.wat");

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

	const TOUCH_WAT: &str = r#"
		(module
			(import "aedicule.v0" "AE_touch_interest"
				(func $touch_interest (param i32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_begin_rgba"
				(func $frame_begin (param i32) (result i32)))
			(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
			(memory (export "memory") 1)
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 10)
			(func (export "AE_configure") (result i32)
				i32.const 2 i32.const 0 call $touch_interest)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_tick") (param i32) (result i32) i32.const 0)
			(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
				i32.const 60 i32.const 1)
			(func (export "AE_render") (result i32)
				i32.const 255 call $frame_begin drop call $frame_end drop i32.const 0)
			(func (export "AE_state_ptr") (result i32) i32.const 0)
			(func (export "AE_state_len") (result i32) i32.const 0)
			(func (export "AE_state_schema") (result i32) i32.const 1))
	"#;

	const MOTION_WAT: &str = r#"
		(module
			(import "aedicule.v0" "AE_motion_interest"
				(func $motion_interest (param i32 i32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_begin_rgba"
				(func $frame_begin (param i32) (result i32)))
			(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
			(memory (export "memory") 1)
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 7)
			(func (export "AE_configure") (result i32)
				i32.const 1 i32.const 0 i32.const 0 call $motion_interest)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_tick") (param i32) (result i32) i32.const 0)
			(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
				i32.const 60 i32.const 1)
			(func (export "AE_render") (result i32)
				i32.const 255 call $frame_begin drop call $frame_end drop i32.const 0)
			(func (export "AE_state_ptr") (result i32) i32.const 0)
			(func (export "AE_state_len") (result i32) i32.const 0)
			(func (export "AE_state_schema") (result i32) i32.const 1))
	"#;

    const PAUSE_WAT: &str = r#"
		(module
			(import "aedicule.v0" "AE_pause_trigger"
				(func $pause_trigger (param i32 i32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_begin_rgba"
				(func $frame_begin (param i32) (result i32)))
			(import "aedicule.v0" "AE_circle"
				(func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
			(memory (export "memory") 1)
			(global $ticks (mut i32) (i32.const 0))
			(global $phase (mut i32) (i32.const 0))
			(global $left_balance (mut i32) (i32.const 0))
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 3)
			(func (export "AE_configure") (result i32)
				i32.const 1 i32.const 5 i32.const 0 call $pause_trigger)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event")
				(param $kind i32) (param $code i32) (param f32 f32) (result i32)
				local.get $kind i32.const 15 i32.eq
				if local.get $code global.set $phase end
				local.get $kind i32.const 1 i32.eq
				local.get $code i32.const 1 i32.eq i32.and
				if
					global.get $left_balance i32.const 1 i32.add global.set $left_balance
				end
				local.get $kind i32.const 2 i32.eq
				local.get $code i32.const 1 i32.eq i32.and
				if
					global.get $left_balance i32.const 1 i32.sub global.set $left_balance
				end
				i32.const 0)
			(func (export "AE_tick") (param $count i32) (result i32)
				global.get $ticks local.get $count i32.add global.set $ticks
				i32.const 0)
			(func (export "AE_tick_rate") (param i32 i32) (result i32 i32)
				i32.const 10 i32.const 1)
			(func (export "AE_render") (result i32)
				i32.const 0x101820ff call $frame_begin drop
				i32.const 1
				global.get $ticks f32.convert_i32_u
				global.get $phase f32.convert_i32_u
				global.get $left_balance i32.const 1 i32.add f32.convert_i32_u
				f32.const 1 i32.const -1 i32.const 1
				call $circle drop
				call $frame_end drop
				i32.const 0)
			(func (export "AE_state_ptr") (result i32) i32.const 0)
			(func (export "AE_state_len") (result i32) i32.const 0)
			(func (export "AE_state_schema") (result i32) i32.const 1))
	"#;

    const SYNTH_WAT: &str = r#"
		(module
			(import "aedicule.v0" "AE_synth_voice"
				(func $synth_voice
					(param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
					(result i32)))
			(import "aedicule.v0" "AE_audio"
				(func $audio (param i32 f32 f32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_begin_rgba"
				(func $frame_begin (param i32) (result i32)))
			(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
			(memory (export "memory") 1)
			(global $played (mut i32) (i32.const 0))
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 0)
			(func (export "AE_configure") (result i32)
				i32.const 7
				i32.const 1
				i32.const 0
				i32.const 20
				i32.const 440000
				i32.const 440000
				i32.const 440000
				i32.const 0
				i32.const 1000000
				i32.const 0
				i32.const 0
				i32.const 0
				i32.const 0
				i32.const 0
				call $synth_voice)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event")
				(param $kind i32) (param i32) (param f32 f32) (result i32)
				local.get $kind i32.const 4 i32.eq
				global.get $played i32.eqz
				i32.and
				if
					i32.const 7 f32.const 1 f32.const 1 i32.const 0 call $audio drop
					i32.const 1 global.set $played
				end
				i32.const 0)
			(func (export "AE_tick") (param i32) (result i32) i32.const 0)
			(func (export "AE_render") (result i32)
				i32.const 0x081020ff call $frame_begin drop
				call $frame_end)
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
	fn counts_raw_touch_separately_from_compatibility_pointer_input() {
		let mut runtime = BrowserRuntime::new(
			TOUCH_WAT,
			PluginInit::new(7, 1024.0, 768.0),
			Duration::ZERO,
		)
		.unwrap();
		runtime.queue_event(
			Duration::from_millis(5),
			Event::TouchStart {
				id: 41,
				x: 250.0,
				y: 384.0,
			},
		);

		assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());
		assert_eq!(runtime.delivered_event_counts().touch, 1);
		assert_eq!(runtime.delivered_event_counts().pointer, 0);
	}

	#[test]
	fn counts_a_host_derived_shake_only_after_guest_delivery() {
		let mut runtime = BrowserRuntime::new(
			MOTION_WAT,
			PluginInit::new(7, 1024.0, 768.0),
			Duration::ZERO,
		)
		.unwrap();
		runtime
			.handle_input(
				Duration::from_millis(5),
				Event::MotionGesture { magnitude: 18.0 },
			)
			.unwrap();

		assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());
		assert_eq!(runtime.delivered_event_counts().motion_gestures, 1);
	}

    #[test]
    fn records_the_exact_standalone_action_identity_delivered_by_the_browser() {
        let mut runtime = BrowserRuntime::new(
            include_str!("../tests/fixtures/standalone_action.wat"),
            PluginInit::new(7, 1024.0, 768.0),
            Duration::ZERO,
        )
        .unwrap();
        runtime.queue_event(Duration::from_millis(5), Event::MenuAction(42));

        assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());
        assert_eq!(runtime.delivered_event_counts().menu_actions, 1);
        assert_eq!(
            runtime.delivered_event_counts().last_menu_action_id,
            Some(42)
        );
        assert_eq!(runtime.frame().background, 0x00ff00ff);
        assert!(runtime.metadata().menu_items.is_empty());
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
    fn wad_down_up_edges_cross_browser_wat_boundary_as_stable_ids() {
        let mut runtime = BrowserRuntime::new(
            PHYSICAL_KEY_WAT,
            PluginInit::new(7, 1024.0, 768.0),
            Duration::ZERO,
        )
        .unwrap();
        let before = runtime.delivered_event_count();
        for (millis, event) in [
            (5, Event::KeyDown(crate::Key::W)),
            (6, Event::KeyUp(crate::Key::W)),
            (7, Event::KeyDown(crate::Key::A)),
            (8, Event::KeyUp(crate::Key::A)),
            (9, Event::KeyDown(crate::Key::D)),
            (10, Event::KeyUp(crate::Key::D)),
        ] {
            assert_eq!(
                runtime
                    .handle_input(Duration::from_millis(millis), event)
                    .unwrap(),
                BrowserInputOutcome::Queued
            );
        }

        assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());
        assert_eq!(runtime.delivered_event_count(), before + 6);
        assert_eq!(runtime.frame().background, 0xd94b64ff);
    }

    #[test]
    fn pause_stops_browser_ticks_renders_once_and_resumes_without_time_debt() {
        let mut runtime =
            BrowserRuntime::new(PAUSE_WAT, PluginInit::new(7, 1024.0, 768.0), Duration::ZERO)
                .unwrap();

        assert_eq!(
            runtime
                .handle_input(Duration::from_millis(150), Event::KeyDown(crate::Key::P))
                .unwrap(),
            BrowserInputOutcome::Paused
        );
        let Some(DrawCommand::Circle { x, y, .. }) = runtime.frame().commands.first() else {
            panic!("expected pause-state probe");
        };
        assert_eq!((*x, *y), (1.0, 1.0));

        assert!(!runtime.advance_to(Duration::from_secs(20)).unwrap());
        let Some(DrawCommand::Circle { x, y, .. }) = runtime.frame().commands.first() else {
            panic!("expected cached paused frame");
        };
        assert_eq!((*x, *y), (1.0, 1.0));

        assert_eq!(
            runtime
                .handle_input(Duration::from_secs(20), Event::KeyUp(crate::Key::P))
                .unwrap(),
            BrowserInputOutcome::Consumed
        );
        assert_eq!(
            runtime
                .handle_input(Duration::from_millis(20_001), Event::KeyDown(crate::Key::P),)
                .unwrap(),
            BrowserInputOutcome::Resumed
        );
        let Some(DrawCommand::Circle { x, y, .. }) = runtime.frame().commands.first() else {
            panic!("expected resumed-state probe");
        };
        assert_eq!((*x, *y), (1.0, 2.0));

        assert!(!runtime.advance_to(Duration::from_millis(20_050)).unwrap());
        assert!(runtime.advance_to(Duration::from_millis(20_051)).unwrap());
        let Some(DrawCommand::Circle { x, y, .. }) = runtime.frame().commands.first() else {
            panic!("expected post-resume tick");
        };
        assert_eq!((*x, *y), (2.0, 2.0));
    }

    #[test]
    fn pause_barrier_delivers_prior_queued_input_and_reconciles_its_release() {
        let mut runtime =
            BrowserRuntime::new(PAUSE_WAT, PluginInit::new(7, 1024.0, 768.0), Duration::ZERO)
                .unwrap();
        assert_eq!(
            runtime
                .handle_input(
                    Duration::from_millis(40),
                    Event::KeyDown(crate::Key::ArrowLeft),
                )
                .unwrap(),
            BrowserInputOutcome::Queued
        );
        assert_eq!(
            runtime
                .handle_input(Duration::from_millis(50), Event::KeyDown(crate::Key::P))
                .unwrap(),
            BrowserInputOutcome::Paused
        );
        let Some(DrawCommand::Circle { radius, .. }) = runtime.frame().commands.first() else {
            panic!("expected pause input-state probe");
        };
        assert_eq!(*radius, 2.0);

        runtime
            .handle_input(
                Duration::from_millis(60),
                Event::KeyUp(crate::Key::ArrowLeft),
            )
            .unwrap();
        runtime
            .handle_input(Duration::from_millis(70), Event::KeyUp(crate::Key::P))
            .unwrap();
        runtime
            .handle_input(Duration::from_millis(80), Event::KeyDown(crate::Key::P))
            .unwrap();
        let Some(DrawCommand::Circle { radius, .. }) = runtime.frame().commands.first() else {
            panic!("expected resumed input-state probe");
        };
        assert_eq!(*radius, 1.0);
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

    #[test]
    fn committed_synth_events_become_bounded_nonzero_browser_pcm() {
        let mut runtime =
            BrowserRuntime::new(SYNTH_WAT, PluginInit::new(7, 1024.0, 768.0), Duration::ZERO)
                .unwrap();
        assert_eq!(
            runtime
                .handle_input(
                    Duration::from_millis(5),
                    Event::PointerDown {
                        button: 1,
                        x: 10.0,
                        y: 20.0,
                    },
                )
                .unwrap(),
            BrowserInputOutcome::Queued
        );
        assert!(runtime.advance_to(Duration::from_millis(17)).unwrap());

        let playback = runtime.drain_audio();
        assert_eq!(playback.len(), 1);
        assert_eq!(playback[0].sample_rate, 48_000);
        assert_eq!(playback[0].channels, 1);
        assert_eq!(playback[0].samples.len(), 960);
        assert!(playback[0].samples.iter().any(|sample| *sample != 0.0));
        assert!(
            playback[0]
                .samples
                .iter()
                .all(|sample| (-1.0..=1.0).contains(sample))
        );
        assert_eq!(playback[0].volume, 1.0);
        assert_eq!(playback[0].pitch, 1.0);
    }
}

#[cfg(test)]
mod package_expansion_tests {
    use crate::web::browser_application_from_package;
    use async_zip::{Compression, DeflateOption, ZipEntryBuilder, base::write::ZipFileWriter};
    use futures_lite::{future::block_on, io::Cursor};

    fn archive(entries: &[(&str, &[u8], Compression)]) -> Vec<u8> {
        block_on(async {
            let mut writer = ZipFileWriter::new(Cursor::new(Vec::new())).force_no_zip64();
            writer
                .write_entry_whole(
                    ZipEntryBuilder::new("mimetype".into(), Compression::Stored),
                    crate::AED_MIME_TYPE.as_bytes(),
                )
                .await
                .unwrap();
            for (name, bytes, compression) in entries {
                let mut descriptor = ZipEntryBuilder::new((*name).into(), *compression);
                if *compression == Compression::Zstd {
                    descriptor = descriptor.deflate_option(DeflateOption::Other(19));
                }
                writer.write_entry_whole(descriptor, bytes).await.unwrap();
            }
            writer.close().await.unwrap().into_inner()
        })
    }

    #[test]
    fn a_compressed_package_expands_to_wat_and_assets_only() {
        let bytes = archive(&[
            ("code.wat", b"(module)".as_slice(), Compression::Zstd),
            (
                "assets/audio/boom.flac",
                b"fLaC----".as_slice(),
                Compression::Zstd,
            ),
            (
                "tests/main.wast",
                b"(module)\n".as_slice(),
                Compression::Zstd,
            ),
            ("README.md", b"# hi\n".as_slice(), Compression::Zstd),
        ]);
        let (wat, assets) = browser_application_from_package(bytes).unwrap();
        assert_eq!(wat, "(module)");
        assert_eq!(
            assets.keys().collect::<Vec<_>>(),
            vec!["assets/audio/boom.flac"],
            "tests and documentation are package entries, not runtime assets",
        );
    }

    #[test]
    fn a_package_without_a_guest_is_refused() {
        let bytes = archive(&[("README.md", b"# hi\n".as_slice(), Compression::Zstd)]);
        let error = browser_application_from_package(bytes).unwrap_err();
        assert!(error.contains("code.wat"), "{error}");
    }

    #[test]
    fn a_guest_that_is_not_utf8_is_refused() {
        let bytes = archive(&[("code.wat", [0xff, 0xfe, 0x00].as_slice(), Compression::Zstd)]);
        let error = browser_application_from_package(bytes).unwrap_err();
        assert!(error.contains("UTF-8"), "{error}");
    }
}
