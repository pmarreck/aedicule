//! Capability-bounded runtime core for WAT-authored GPUI applications.
//!
//! This crate intentionally keeps WebAssembly execution and immutable frame
//! output independent of GPUI. Native and future web frontplanes are adapters
//! over the same lifecycle, validation, snapshot, and command-buffer logic.

use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::ffi::OsString;
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;

#[cfg(all(feature = "native-runtime", feature = "portable-runtime"))]
compile_error!("select exactly one Aedicule guest runtime");
#[cfg(not(any(feature = "native-runtime", feature = "portable-runtime")))]
compile_error!("select either the native-runtime or portable-runtime feature");

use runtime::{
    Caller, Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder, TypedFunc,
};
#[cfg(feature = "portable-runtime")]
use wasmi as runtime;
#[cfg(feature = "native-runtime")]
use wasmtime as runtime;

#[cfg(feature = "native-runtime")]
mod application_test;
mod flac;
mod package;
mod wat_abi;
#[cfg(feature = "native-runtime")]
mod web_server;

#[cfg(feature = "native-runtime")]
pub use application_test::{ApplicationTestReport, run_application_tests};
pub use flac::{FlacError, decode_flac};
pub use package::{
    AED_MIME_TYPE, depackage_application, package_application, read_application_assets,
    read_application_file,
};
#[cfg(feature = "native-runtime")]
pub use web_server::{WebServer, discover_web_runtime};

/// Deterministic fixed-point synthesis shared by native and browser adapters.
pub mod audio;
/// Bounded, device-independent decoding for packaged digitized-audio clips.
pub mod wav;

/// Shared GPUI adapter for validated command frames, used by both native and
/// browser frontplanes without allowing rendering code to execute guest WAT.
#[cfg(any(feature = "gui", feature = "web"))]
pub mod gpui_canvas;

/// Browser-delivery boundary for the WAT document supplied as a static asset.
#[cfg(feature = "portable-runtime")]
pub mod web;

pub use wat_abi::{
    LLM_GUIDE_VERSION, WAT_ABI_IMPORTS, WatAbiImport, guide_for_llms_markdown, wat_abi_markdown,
};

pub const ABI_MAJOR: i32 = wat_abi::ABI_MAJOR;
pub const ABI_MINOR: i32 = wat_abi::ABI_MINOR;
pub const DEFAULT_PLUGIN_FILE: &str = "code.wat";
pub const DEFAULT_PLUGIN_ENV: &str = "AEDICULE_DEFAULT_APPLICATION";
/// Overrides the host's nominal initial display timing with an exact rate.
pub const DISPLAY_REFRESH_RATE_ENV: &str = "AE_DISPLAY_REFRESH_RATE";
pub const FALLBACK_WAT: &str = include_str!("fallback.wat");
pub const DEFAULT_SIMULATION_HZ: u32 = 60;
pub const MAX_SIMULATION_HZ: u32 = 1_000;
pub const MAX_TICK_RATE_DENOMINATOR: u32 = 1_000_000;
/// Stable terminal prefix for rejected external WAT sources.
pub const WAT_REJECTION_DIAGNOSTIC_PREFIX: &str = "AEDICULE_WAT_REJECTED";
/// Exact OFL-1.1 Geist Mono Regular payload registered by every GUI adapter.
pub const GEIST_MONO_REGULAR: &[u8] = include_bytes!("../assets/fonts/GeistMono-Regular.ttf");
/// Stable family identity encoded in the bundled font's OpenType name table.
pub const GEIST_MONO_FAMILY: &str = "Geist Mono";

/// Immutable virtual-root files admitted by an application I/O adapter before
/// guest instantiation; only explicit capability imports can observe them.
pub type ApplicationAssets = BTreeMap<String, Vec<u8>>;

const NANOS_PER_SECOND: u128 = 1_000_000_000;
const Q16_SCALE: f64 = 65_536.0;
const WAT_IMPORT_MODULE: &str = wat_abi::IMPORT_MODULE;
const Q30_ONE: i64 = 1 << 30;
const MAX_PAUSE_TRIGGERS: usize = 16;
const CORDIC_INVERSE_GAIN_Q30: i64 = 652_032_874;
const CORDIC_ATAN_TURN: [i64; 31] = [
    0x2000_0000,
    0x12e4_051e,
    0x09fb_385b,
    0x0511_11d4,
    0x028b_0d43,
    0x0145_d7e1,
    0x00a2_f61e,
    0x0051_7c55,
    0x0028_be53,
    0x0014_5f2f,
    0x000a_2f98,
    0x0005_17cc,
    0x0002_8be6,
    0x0001_45f3,
    0x0000_a2fa,
    0x0000_517d,
    0x0000_28be,
    0x0000_145f,
    0x0000_0a30,
    0x0000_0518,
    0x0000_028c,
    0x0000_0146,
    0x0000_00a3,
    0x0000_0051,
    0x0000_0029,
    0x0000_0014,
    0x0000_000a,
    0x0000_0005,
    0x0000_0003,
    0x0000_0001,
    0x0000_0001,
];

/// Evaluates sine and cosine with a fixed integer CORDIC table, making WAT
/// animation bit-reproducible across CPUs without host `libm` or float opcodes.
pub fn sin_cos_turn_q30(angle: i32) -> (i32, i32) {
    let unsigned = angle as u32;
    match unsigned {
        0 => return (0, Q30_ONE as i32),
        0x4000_0000 => return (Q30_ONE as i32, 0),
        0x8000_0000 => return (0, -(Q30_ONE as i32)),
        0xc000_0000 => return (-(Q30_ONE as i32), 0),
        _ => {}
    }

    let mut residual = i64::from(angle);
    let mut negate = false;
    if residual > 0x4000_0000 {
        residual -= 0x8000_0000;
        negate = true;
    } else if residual < -0x4000_0000 {
        residual += 0x8000_0000;
        negate = true;
    }

    let mut cosine = CORDIC_INVERSE_GAIN_Q30;
    let mut sine = 0_i64;
    for (shift, arctangent) in CORDIC_ATAN_TURN.into_iter().enumerate() {
        let previous_cosine = cosine;
        if residual >= 0 {
            cosine -= sine >> shift;
            sine += previous_cosine >> shift;
            residual -= arctangent;
        } else {
            cosine += sine >> shift;
            sine -= previous_cosine >> shift;
            residual += arctangent;
        }
    }
    if negate {
        sine = -sine;
        cosine = -cosine;
    }
    (
        sine.clamp(-Q30_ONE, Q30_ONE) as i32,
        cosine.clamp(-Q30_ONE, Q30_ONE) as i32,
    )
}

/// Identifies whether a rejection retained the embedded fallback or the last
/// known-good external guest, so terminal diagnostics describe the survivor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatRejectionStage {
    InitialLoad,
    Reload,
}

impl WatRejectionStage {
    fn label(self) -> &'static str {
        match self {
            Self::InitialLoad => "initial-load",
            Self::Reload => "reload",
        }
    }

    fn survivor(self) -> &'static str {
        match self {
            Self::InitialLoad => "embedded fallback remains active",
            Self::Reload => "previous plugin remains active",
        }
    }
}

/// Renders a stable, grep-friendly terminal diagnostic without performing I/O,
/// so native watcher tests can prove the source, error, and retained guest.
pub fn format_wat_rejection_diagnostic(
    path: &Path,
    stage: WatRejectionStage,
    error: &str,
) -> String {
    format!(
        "{WAT_REJECTION_DIAGNOSTIC_PREFIX}: {}: {}: {error}: {}",
        stage.label(),
        path.display(),
        stage.survivor(),
    )
}

/// An exact fixed-step frequency expressed as whole ticks per rational second.
/// This keeps fractional display modes such as 60,000/1,001 Hz out of `f32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TickRate {
    pub numerator: u32,
    pub denominator: u32,
}

impl TickRate {
    /// Constructs a normalized supported rate. Invalid programmer-supplied
    /// values panic; untrusted WAT values use `validate_tick_rate` instead.
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        assert!(numerator > 0);
        assert!(denominator > 0);
        assert!(denominator <= MAX_TICK_RATE_DENOMINATOR);
        assert!(numerator >= denominator);
        assert!((numerator as u128) <= (MAX_SIMULATION_HZ as u128) * (denominator as u128));
        let divisor = greatest_common_divisor(numerator, denominator);
        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        }
    }

    /// Returns the ceil-rounded offset of the boundary after `ticks` exact
    /// fixed steps, preserving phase for non-integral periods.
    pub fn boundary_after(self, ticks: u32) -> Duration {
        let nanos = u128::from(ticks)
            .saturating_mul(NANOS_PER_SECOND)
            .saturating_mul(u128::from(self.denominator))
            .div_ceil(u128::from(self.numerator));
        duration_from_nanos(nanos)
    }
}

impl From<u32> for TickRate {
    fn from(hz: u32) -> Self {
        Self::new(hz, 1)
    }
}

impl fmt::Display for TickRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(formatter, "{}", self.numerator)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// Explains why a user-supplied display refresh override cannot become a
/// bounded exact simulation rate.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid AE_DISPLAY_REFRESH_RATE value {value:?}: {reason}")]
pub struct DisplayRefreshRateError {
    value: String,
    reason: &'static str,
}

impl DisplayRefreshRateError {
    fn invalid(value: &str, reason: &'static str) -> Self {
        Self {
            value: value.to_owned(),
            reason,
        }
    }
}

/// Parses the exact `AE_DISPLAY_REFRESH_RATE` grammar without floating point.
/// Canonical broadcast-rate spellings select their 1000/1001 rational forms;
/// all other decimals preserve precisely the digits the user supplied.
pub fn parse_display_refresh_rate(value: &str) -> Result<TickRate, DisplayRefreshRateError> {
    let value = value.trim();
    match value {
        "23.976" => Ok(TickRate::new(24_000, 1_001)),
        "29.97" => Ok(TickRate::new(30_000, 1_001)),
        "59.94" => Ok(TickRate::new(60_000, 1_001)),
        "119.88" => Ok(TickRate::new(120_000, 1_001)),
        _ => {
            let (numerator, denominator) =
                if let Some((numerator, denominator)) = value.split_once('/') {
                    (
                        parse_rate_component(value, numerator)?,
                        parse_rate_component(value, denominator)?,
                    )
                } else {
                    parse_decimal_rate(value)?
                };
            validate_host_tick_rate(value, numerator, denominator)
        }
    }
}

/// Reads one process-start display override. Its absence deliberately leaves
/// platform display discovery in control of future display-change events.
pub fn display_refresh_rate_from_environment() -> Result<Option<TickRate>, DisplayRefreshRateError>
{
    match std::env::var(DISPLAY_REFRESH_RATE_ENV) {
        Ok(value) => parse_display_refresh_rate(&value).map(Some),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(DisplayRefreshRateError::invalid(
            "<non-Unicode>",
            "the value must be UTF-8",
        )),
    }
}

/// Parses a decimal token as an integer fraction, so user configuration never
/// crosses an `f32` or `f64` rounding boundary before rate validation.
fn parse_decimal_rate(value: &str) -> Result<(u32, u32), DisplayRefreshRateError> {
    let Some((whole, fraction)) = value.split_once('.') else {
        return Ok((parse_rate_component(value, value)?, 1));
    };
    if whole.is_empty() || fraction.is_empty() || fraction.contains('.') {
        return Err(DisplayRefreshRateError::invalid(
            value,
            "expected a whole decimal such as 120 or 59.97",
        ));
    }
    let whole = parse_rate_component(value, whole)?;
    let fraction = parse_rate_component(value, fraction)?;
    let denominator = 10_u32
        .checked_pow(fraction_decimal_places(value)?)
        .ok_or_else(|| DisplayRefreshRateError::invalid(value, "too many decimal places"))?;
    let numerator = whole
        .checked_mul(denominator)
        .and_then(|whole| whole.checked_add(fraction))
        .ok_or_else(|| DisplayRefreshRateError::invalid(value, "rate overflows u32"))?;
    Ok((numerator, denominator))
}

fn fraction_decimal_places(value: &str) -> Result<u32, DisplayRefreshRateError> {
    let (_, fraction) = value
        .split_once('.')
        .expect("decimal rates have a fractional separator");
    u32::try_from(fraction.len())
        .map_err(|_| DisplayRefreshRateError::invalid(value, "too many decimal places"))
}

fn parse_rate_component(value: &str, component: &str) -> Result<u32, DisplayRefreshRateError> {
    if component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DisplayRefreshRateError::invalid(
            value,
            "expected positive base-10 numerator and denominator digits",
        ));
    }
    component
        .parse()
        .map_err(|_| DisplayRefreshRateError::invalid(value, "rate component overflows u32"))
}

fn validate_host_tick_rate(
    value: &str,
    numerator: u32,
    denominator: u32,
) -> Result<TickRate, DisplayRefreshRateError> {
    if numerator == 0
        || denominator == 0
        || denominator > MAX_TICK_RATE_DENOMINATOR
        || numerator < denominator
        || u128::from(numerator) > u128::from(MAX_SIMULATION_HZ) * u128::from(denominator)
    {
        return Err(DisplayRefreshRateError::invalid(
            value,
            "expected a rate in the inclusive 1..=1000 Hz range with denominator at most 1,000,000",
        ));
    }
    Ok(TickRate::new(numerator, denominator))
}

const fn greatest_common_divisor(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TickAdvance {
    pub ticks: u32,
    pub dropped_ticks: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FixedStepClock {
    fractional_tick_nanos: u128,
}

impl FixedStepClock {
    /// Converts arbitrary monotonic elapsed-time partitions into exact fixed
    /// ticks by retaining the rational nanoseconds-times-hertz remainder.
    pub fn advance(&mut self, elapsed: Duration, hz: u32, max_ticks: u32) -> TickAdvance {
        self.advance_rate(elapsed, TickRate::from(hz), max_ticks)
    }

    /// Converts elapsed monotonic time into exact ticks at a rational rate by
    /// carrying the numerator-times-nanoseconds remainder over its full period.
    pub fn advance_rate(
        &mut self,
        elapsed: Duration,
        rate: TickRate,
        max_ticks: u32,
    ) -> TickAdvance {
        let total = self.fractional_tick_nanos.saturating_add(
            elapsed
                .as_nanos()
                .saturating_mul(u128::from(rate.numerator)),
        );
        let tick_units = NANOS_PER_SECOND.saturating_mul(u128::from(rate.denominator));
        let due = total / tick_units;
        self.fractional_tick_nanos = total % tick_units;
        let ticks = due.min(u128::from(max_ticks)) as u32;
        let dropped = due.saturating_sub(u128::from(ticks));
        TickAdvance {
            ticks,
            dropped_ticks: dropped.min(u128::from(u64::MAX)) as u64,
        }
    }

    /// Returns the ceil-rounded monotonic delay until the next exact tick so
    /// timer adapters never wake late merely because 1e9/hz is fractional.
    pub fn time_until_next_tick(&self, hz: u32) -> Duration {
        self.time_until_next_tick_at(TickRate::from(hz))
    }

    /// Returns the ceil-rounded delay to the next exact rational boundary.
    pub fn time_until_next_tick_at(&self, rate: TickRate) -> Duration {
        let tick_units = NANOS_PER_SECOND.saturating_mul(u128::from(rate.denominator));
        let remaining = tick_units - self.fractional_tick_nanos;
        duration_from_nanos(remaining.div_ceil(u128::from(rate.numerator)))
    }
}

/// One ordered guest invocation planned for a monotonic scheduler wake.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationCall {
    Event(Event),
    Tick(u32),
}

/// Bounded work and overload telemetry produced without touching the guest.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationPump {
    pub calls: Vec<SimulationCall>,
    pub dropped_ticks: u64,
    pub render: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TimestampedEvent {
    timestamp: Duration,
    event: Event,
}

/// Maps advisory timer wakes and timestamped input onto exact fixed-step
/// boundaries while bounding the amount of guest catch-up work per pump.
#[derive(Debug, Clone)]
pub struct SimulationScheduler {
    tick_rate: TickRate,
    max_ticks_per_pump: u32,
    origin: Duration,
    last_sample: Duration,
    elapsed_boundaries: u128,
    total_dropped_ticks: u64,
    clock: FixedStepClock,
    events: VecDeque<TimestampedEvent>,
}

impl SimulationScheduler {
    /// Plans deterministic event/tick calls against absolute rational
    /// boundaries, making timer wakes advisory rather than authoritative.
    pub fn new(tick_rate: impl Into<TickRate>, max_ticks_per_pump: u32, origin: Duration) -> Self {
        assert!(max_ticks_per_pump > 0);
        Self {
            tick_rate: tick_rate.into(),
            max_ticks_per_pump,
            origin,
            last_sample: origin,
            elapsed_boundaries: 0,
            total_dropped_ticks: 0,
            clock: FixedStepClock::default(),
            events: VecDeque::new(),
        }
    }

    /// Preserves native arrival order while attaching the monotonic timestamp
    /// used to classify the event at an exact simulation boundary.
    pub fn queue_event(&mut self, timestamp: Duration, event: Event) {
        self.events.push_back(TimestampedEvent { timestamp, event });
    }

    /// Flushes raw input that arrived before a host-owned pause barrier so it
    /// cannot be stranded and replayed after a later resume.
    pub fn drain_events_through(&mut self, timestamp: Duration) -> Vec<Event> {
        let mut events = Vec::new();
        while self
            .events
            .front()
            .is_some_and(|event| event.timestamp <= timestamp)
        {
            events.push(self.events.pop_front().expect("front event exists").event);
        }
        events
    }

    /// Removes wall time spent suspended from the exact rational timeline while
    /// retaining its fractional tick phase and every pre-pause boundary.
    pub fn resume_after_pause(&mut self, paused_at: Duration, resumed_at: Duration) {
        assert!(paused_at >= self.last_sample);
        assert!(resumed_at >= paused_at);
        let suspension = resumed_at - paused_at;
        self.origin = self.origin.saturating_add(suspension);
        self.last_sample = self.last_sample.saturating_add(suspension);
        for event in &mut self.events {
            if event.timestamp >= paused_at {
                event.timestamp = event.timestamp.saturating_add(suspension);
            }
        }
    }

    /// Converts one arbitrary wake into the smallest ordered sequence of guest
    /// calls, batching only adjacent ticks with no intervening input boundary.
    pub fn pump(&mut self, now: Duration) -> SimulationPump {
        assert!(now >= self.last_sample);
        let elapsed = now - self.last_sample;
        let advance = self
            .clock
            .advance_rate(elapsed, self.tick_rate, self.max_ticks_per_pump);
        self.last_sample = now;

        let dropped = u128::from(advance.dropped_ticks);
        let first_surviving_boundary = self
            .elapsed_boundaries
            .saturating_add(dropped)
            .saturating_add(1);
        let mut calls = Vec::new();
        let mut pending_ticks = 0_u32;

        for tick_offset in 0..advance.ticks {
            let boundary =
                self.boundary_at(first_surviving_boundary.saturating_add(u128::from(tick_offset)));
            let has_due_event = self
                .events
                .front()
                .is_some_and(|event| event.timestamp <= boundary);
            if has_due_event {
                push_tick_run(&mut calls, &mut pending_ticks);
                while self
                    .events
                    .front()
                    .is_some_and(|event| event.timestamp <= boundary)
                {
                    let event = self.events.pop_front().expect("front event exists");
                    calls.push(SimulationCall::Event(event.event));
                }
            }
            pending_ticks += 1;
        }
        push_tick_run(&mut calls, &mut pending_ticks);

        self.elapsed_boundaries = self
            .elapsed_boundaries
            .saturating_add(dropped)
            .saturating_add(u128::from(advance.ticks));
        self.total_dropped_ticks = self
            .total_dropped_ticks
            .saturating_add(advance.dropped_ticks);

        SimulationPump {
            render: !calls.is_empty(),
            calls,
            dropped_ticks: advance.dropped_ticks,
        }
    }

    /// Returns the next absolute rational boundary in the scheduler clock domain.
    pub fn next_deadline(&self) -> Duration {
        self.boundary_at(self.elapsed_boundaries.saturating_add(1))
    }

    /// Derives a fresh one-shot timer delay without letting prior wake jitter drift.
    pub fn time_until_next_wake(&self, now: Duration) -> Duration {
        self.next_deadline().saturating_sub(now)
    }

    /// Exposes cumulative overload debt so the UI cannot hide simulation slowdown.
    pub fn total_dropped_ticks(&self) -> u64 {
        self.total_dropped_ticks
    }

    fn boundary_at(&self, ordinal: u128) -> Duration {
        let offset_nanos = ordinal
            .saturating_mul(NANOS_PER_SECOND)
            .saturating_mul(u128::from(self.tick_rate.denominator))
            .div_ceil(u128::from(self.tick_rate.numerator));
        duration_from_nanos(self.origin.as_nanos().saturating_add(offset_nanos))
    }
}

fn push_tick_run(calls: &mut Vec<SimulationCall>, pending_ticks: &mut u32) {
    if *pending_ticks > 0 {
        calls.push(SimulationCall::Tick(*pending_ticks));
        *pending_ticks = 0;
    }
}

fn duration_from_nanos(nanos: u128) -> Duration {
    let seconds = nanos / NANOS_PER_SECOND;
    if seconds > u128::from(u64::MAX) {
        return Duration::new(u64::MAX, 999_999_999);
    }
    Duration::new(seconds as u64, (nanos % NANOS_PER_SECOND) as u32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSource {
    Embedded,
    File(PathBuf),
    Directory(PathBuf),
    Archive(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchAction {
    Launcher {
        seed: Option<u64>,
    },
    Run {
        source: PluginSource,
        watch: bool,
        seed: Option<u64>,
    },
    Package {
        source: PathBuf,
        output: PathBuf,
    },
    Depackage {
        source: PathBuf,
        output: PathBuf,
    },
    Web {
        source: PluginSource,
        bind: String,
        port: u16,
    },
    Test {
        source: PluginSource,
        seed: Option<u64>,
    },
    Help,
    About,
}

/// Converts a platform `file:` open-document URL into the same local path
/// accepted by the CLI, rejecting remote authorities and malformed escapes.
pub fn file_url_to_path(url: &str) -> Option<PathBuf> {
    let remainder = url.strip_prefix("file://")?;
    let encoded_path = if remainder.starts_with('/') {
        remainder.to_owned()
    } else if let Some(path) = remainder.strip_prefix("localhost/") {
        format!("/{path}")
    } else {
        return None;
    };
    let bytes = encoded_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }
        let high = *bytes.get(index + 1)?;
        let low = *bytes.get(index + 2)?;
        decoded.push(hex_digit(high)? * 16 + hex_digit(low)?);
        index += 3;
    }
    let decoded = String::from_utf8(decoded).ok()?;
    #[cfg(windows)]
    let decoded = decoded
        .strip_prefix('/')
        .filter(|path| path.as_bytes().get(1) == Some(&b':'))
        .unwrap_or(&decoded);
    Some(PathBuf::from(decoded))
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Resolves the native runner's order-sensitive source switches while treating
/// an application directory as a container for the conventional `code.wat`.
pub fn resolve_launch(
    arguments: impl IntoIterator<Item = OsString>,
    working_directory: &Path,
    packaged_default: Option<&Path>,
) -> Result<LaunchAction, String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Run,
        Package,
        Depackage,
        Web,
        Test,
    }

    let mut source = None;
    let mut mode = Mode::Run;
    let mut watch = false;
    let mut seed = None;
    let mut bind = "127.0.0.1".to_owned();
    let mut port = 8080;
    let mut positionals = Vec::new();
    let mut positional_only = false;
    let mut arguments = arguments.into_iter();

    while let Some(argument) = arguments.next() {
        let recognized = argument.to_str();
        if !positional_only {
            match recognized {
                Some("-h" | "--help") => return Ok(LaunchAction::Help),
                Some("--about") => return Ok(LaunchAction::About),
                Some("--watch") => {
                    watch = true;
                    if source == Some(PluginSource::Embedded) {
                        source = None;
                    }
                    continue;
                }
                Some("--embedded") => {
                    mode = Mode::Run;
                    source = Some(PluginSource::Embedded);
                    watch = false;
                    continue;
                }
                Some("--package") => {
                    mode = Mode::Package;
                    source = None;
                    watch = false;
                    continue;
                }
                Some("--depackage") => {
                    mode = Mode::Depackage;
                    source = None;
                    watch = false;
                    continue;
                }
                Some("--web") => {
                    mode = Mode::Web;
                    source = None;
                    watch = false;
                    continue;
                }
                Some("--test") => {
                    mode = Mode::Test;
                    source = None;
                    watch = false;
                    continue;
                }
                Some("--bind") => {
                    bind = arguments
                        .next()
                        .ok_or_else(|| "--bind requires an address".to_owned())?
                        .into_string()
                        .map_err(|_| "--bind must be valid UTF-8".to_owned())?;
                    continue;
                }
                Some("--port") => {
                    let value = arguments.next().ok_or_else(|| {
                        "--port requires an integer from 0 through 65535".to_owned()
                    })?;
                    let text = value
                        .to_str()
                        .ok_or_else(|| "--port must be valid UTF-8 digits".to_owned())?;
                    port = text.parse::<u16>().map_err(|_| {
                        "--port requires an integer from 0 through 65535".to_owned()
                    })?;
                    continue;
                }
                Some("--seed") => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "--seed requires an unsigned integer".to_owned())?;
                    let text = value
                        .to_str()
                        .ok_or_else(|| "--seed must be valid UTF-8 digits".to_owned())?;
                    seed = Some(parse_seed(text)?);
                    continue;
                }
                Some("--") => {
                    positional_only = true;
                    continue;
                }
                Some(option) if option.starts_with('-') && option != "-" => {
                    return Err(format!("unknown option: {option}"));
                }
                _ => {}
            }
        }

        positionals.push(PathBuf::from(argument));
    }

    let absolute = |path: PathBuf| {
        if path.is_relative() {
            working_directory.join(path)
        } else {
            path
        }
    };
    let plugin_source = |path: PathBuf| {
        let path = absolute(path);
        if path.is_dir() {
            PluginSource::Directory(path)
        } else if path.extension().is_some_and(|extension| extension == "aed") {
            PluginSource::Archive(path)
        } else {
            PluginSource::File(path)
        }
    };

    match mode {
        Mode::Package => {
            if !(1..=2).contains(&positionals.len()) {
                return Err(
                    "--package requires a source directory and optional output .aed path".into(),
                );
            }
            let source = absolute(positionals.remove(0));
            let output = positionals
                .pop()
                .map(absolute)
                .unwrap_or_else(|| source.with_extension("aed"));
            return Ok(LaunchAction::Package { source, output });
        }
        Mode::Depackage => {
            if !(1..=2).contains(&positionals.len()) {
                return Err(
                    "--depackage requires a source .aed and optional output directory".into(),
                );
            }
            let source = absolute(positionals.remove(0));
            let output = positionals
                .pop()
                .map(absolute)
                .unwrap_or_else(|| source.with_extension(""));
            return Ok(LaunchAction::Depackage { source, output });
        }
        Mode::Web => {
            if positionals.len() != 1 {
                return Err(
                    "--web requires exactly one WAT, application directory, or .aed path".into(),
                );
            }
            return Ok(LaunchAction::Web {
                source: plugin_source(positionals.remove(0)),
                bind,
                port,
            });
        }
        Mode::Test => {
            if positionals.len() != 1 {
                return Err(
                    "--test requires exactly one WAT, application directory, or .aed path".into(),
                );
            }
            return Ok(LaunchAction::Test {
                source: plugin_source(positionals.remove(0)),
                seed,
            });
        }
        Mode::Run => {}
    }

    if positionals.len() > 1 {
        return Err(
            "only one WAT file, .aed package, or application directory may be specified".into(),
        );
    }
    if let Some(path) = positionals.pop() {
        source = Some(plugin_source(path));
    }

    let source = match source {
        Some(source) => source,
        None => {
            let default = working_directory.join(DEFAULT_PLUGIN_FILE);
            if watch || default.is_file() {
                PluginSource::File(default)
            } else if let Some(packaged_default) = packaged_default {
                PluginSource::File(packaged_default.to_owned())
            } else {
                return Ok(LaunchAction::Launcher { seed });
            }
        }
    };
    Ok(LaunchAction::Run {
        source,
        watch,
        seed,
    })
}

fn parse_seed(text: &str) -> Result<u64, String> {
    let parsed = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .map_or_else(
            || text.parse::<u64>(),
            |digits| u64::from_str_radix(digits, 16),
        );
    parsed.map_err(|_| "--seed requires an unsigned decimal or 0x-prefixed integer".to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRevision {
    Missing,
    Unreadable(String),
    Content(Vec<u8>),
}

impl FileRevision {
    pub fn read(path: &Path) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => Self::Content(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::Missing,
            Err(error) => Self::Unreadable(error.to_string()),
        }
    }
}

#[derive(Debug, Default)]
pub struct RevisionTracker {
    previous: Option<FileRevision>,
}

impl RevisionTracker {
    /// Classifies directory-safe file observations by content, suppressing
    /// duplicate polls and duplicate editor events without relying on time.
    pub fn observe(&mut self, revision: &FileRevision) -> bool {
        if self.previous.as_ref() == Some(revision) {
            false
        } else {
            self.previous = Some(revision.clone());
            true
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Limits {
    pub fuel_per_call: u64,
    pub max_memory_bytes: usize,
    pub max_table_elements: usize,
    pub max_coordinate_abs: f32,
    pub max_commands: usize,
    pub max_string_bytes: usize,
    pub max_menu_items: usize,
    pub max_controls: usize,
    pub max_control_steps: u32,
    pub max_synth_voices: usize,
    pub max_audio_events: usize,
    pub max_sample_assets: usize,
    pub max_sample_encoded_bytes: usize,
    pub max_total_sample_decoded_bytes: usize,
    pub max_effects: usize,
    pub max_image_bytes: usize,
    pub max_total_image_bytes: usize,
    pub max_path_segments: usize,
    pub max_snapshot_bytes: usize,
    pub max_ticks_per_call: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            fuel_per_call: 2_000_000,
            max_memory_bytes: 4 * 1024 * 1024,
            max_table_elements: 4_096,
            max_coordinate_abs: 1_000_000.0,
            max_commands: 4_096,
            max_string_bytes: 4_096,
            max_menu_items: 128,
            max_controls: 32,
            max_control_steps: 1_000_000,
            max_synth_voices: 128,
            max_audio_events: 256,
            max_sample_assets: 64,
            max_sample_encoded_bytes: 16 * 1024 * 1024,
            max_total_sample_decoded_bytes: 64 * 1024 * 1024,
            max_effects: 256,
            max_image_bytes: 4 * 1024 * 1024,
            max_total_image_bytes: 16 * 1024 * 1024,
            max_path_segments: 8_192,
            max_snapshot_bytes: 1024 * 1024,
            max_ticks_per_call: 240,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub title: String,
    pub menu_items: Vec<MenuItem>,
    pub pause_triggers: Vec<PauseTrigger>,
    pub controls: Vec<SliderControl>,
    pub synth_voices: Vec<SynthVoice>,
    pub sample_assets: Vec<SampleAsset>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: "WAT Application".into(),
            menu_items: Vec::new(),
            pause_triggers: Vec::new(),
            controls: Vec::new(),
            synth_voices: Vec::new(),
            sample_assets: Vec::new(),
        }
    }
}

/// One immutable decoded sample admitted from an application virtual root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleAsset {
    pub id: u32,
    pub path: String,
    pub clip: wav::WavClip,
}

/// Describes a host-rendered slider whose exact guest values stay on a
/// bounded integer lattice even when a GUI toolkit positions its thumb in f32.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliderControl {
    pub id: u32,
    pub label: String,
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub initial: i32,
}

impl SliderControl {
    /// Maps the guest's authoritative semantic value to the exact GPUI step
    /// index used only inside the native adapter.
    pub fn step_index(&self, value: i32) -> Option<u32> {
        self.contains_value(value)
            .then(|| ((i64::from(value) - i64::from(self.min)) / i64::from(self.step)) as u32)
    }

    /// Reconstructs an exact semantic value from a host UI step index without
    /// admitting floating-point rounding into the WAT control event.
    pub fn value_at_step(&self, index: u32) -> Option<i32> {
        let value = i64::from(self.min) + i64::from(index) * i64::from(self.step);
        (value <= i64::from(self.max)).then_some(value as i32)
    }

    pub fn step_count(&self) -> u32 {
        ((i64::from(self.max) - i64::from(self.min)) / i64::from(self.step)) as u32
    }

    /// Checks an untrusted event or frame value against the exact declared
    /// integer lattice without passing through a GUI toolkit's float scalar.
    pub fn contains_value(&self, value: i32) -> bool {
        let offset = i64::from(value) - i64::from(self.min);
        value >= self.min && value <= self.max && offset % i64::from(self.step) == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SynthWaveform {
    Sine = 1,
    Saw = 2,
    WhiteNoise = 3,
    BrownNoise = 4,
}

impl SynthWaveform {
    fn from_abi(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Sine),
            2 => Some(Self::Saw),
            3 => Some(Self::WhiteNoise),
            4 => Some(Self::BrownNoise),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SynthFilter {
    None = 0,
    LowPass = 1,
    BandPass = 2,
}

impl SynthFilter {
    fn from_abi(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::LowPass),
            2 => Some(Self::BandPass),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthVoice {
    pub program_id: u32,
    pub waveform: SynthWaveform,
    pub delay_ms: u32,
    pub duration_ms: u32,
    pub frequency_start_millihz: u32,
    pub frequency_mid_millihz: u32,
    pub frequency_end_millihz: u32,
    pub gain_start_ppm: u32,
    pub gain_peak_ppm: u32,
    pub gain_end_ppm: u32,
    pub filter: SynthFilter,
    pub filter_start_millihz: u32,
    pub filter_end_millihz: u32,
    pub cooldown_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub id: Option<u32>,
    pub label: String,
    pub shortcut: Option<String>,
    pub separator: bool,
}

impl MenuItem {
    pub fn action(id: u32, label: impl Into<String>, shortcut: Option<&str>) -> Self {
        Self {
            id: Some(id),
            label: label.into(),
            shortcut: shortcut.map(str::to_owned),
            separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: None,
            label: String::new(),
            shortcut: None,
            separator: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameOutput {
    pub background: u32,
    pub commands: Vec<DrawCommand>,
}

/// Represents one complete guest-authoritative native UI document. Adapters
/// reconcile stable IDs only after the whole revision validates successfully.
#[derive(Debug, Clone, PartialEq)]
pub struct UiSnapshot {
    pub revision: u32,
    pub control_panels: Vec<ControlPanel>,
    pub sliders: Vec<SliderPlacement>,
    pub buttons: Vec<ButtonPlacement>,
}

/// Places one host-native control surface in the guest-authored UI snapshot;
/// Aedicule supplies platform styling but never invents its geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlPanel {
    pub id: u32,
    pub bounds: Rect,
    pub rgba: u32,
}

/// Selects how the native label/value is composed inside a guest-provided
/// slider rectangle without transferring layout ownership to the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ControlLabelPlacement {
    Hidden = 0,
    Above = 1,
}

impl ControlLabelPlacement {
    fn from_abi(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Hidden),
            1 => Some(Self::Above),
            _ => None,
        }
    }
}

/// Carries the guest's authoritative integer value and exact Q16-derived
/// placement for one native slider in the current immutable UI snapshot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderPlacement {
    pub id: u32,
    pub panel_id: u32,
    pub value: i32,
    pub bounds: Rect,
    pub label_placement: ControlLabelPlacement,
}

/// Places a guest-keyed native button whose label and action identity come
/// from one configure-time action declaration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonPlacement {
    pub id: u32,
    pub panel_id: u32,
    pub action_id: u32,
    pub bounds: Rect,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub tx: f32,
    pub ty: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    Move(Point),
    Line(Point),
    Quadratic {
        control: Point,
        end: Point,
    },
    Cubic {
        control_1: Point,
        control_2: Point,
        end: Point,
    },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Encoded,
    RawRgba,
}

/// Portable WAT-facing text-face selectors; explicit variants prevent a
/// missing host font from silently changing numerical layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TextFont {
    PlatformDefault = 0,
    GeistMonoRegular = 1,
}

impl TryFrom<i32> for TextFont {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::PlatformDefault),
            1 => Ok(Self::GeistMonoRegular),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageResource {
    pub id: u32,
    pub format: ImageFormat,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    PushTransform(Affine),
    PopTransform,
    Line {
        id: u32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        rgba: u32,
    },
    Circle {
        id: u32,
        x: f32,
        y: f32,
        radius: f32,
        width: f32,
        rgba: u32,
        filled: bool,
    },
    Text {
        id: u32,
        text: String,
        x: f32,
        y: f32,
        size: f32,
        rgba: u32,
        centered: bool,
        font: TextFont,
    },
    Path {
        id: u32,
        segments: Vec<PathSegment>,
        width: f32,
        fill_rgba: Option<u32>,
        stroke_rgba: Option<u32>,
    },
    Sprite {
        id: u32,
        image_id: u32,
        source: Rect,
        destination: Rect,
        pivot: Point,
        tint_rgba: u32,
        flags: u32,
    },
}

/// Serializes an immutable scene-command frame as deterministic SVG so visual
/// output can be inspected in headless CI and by agents without a desktop.
pub fn render_svg(frame: &FrameOutput, logical_width: f32, logical_height: f32) -> String {
    let mut svg = String::new();
    let width = svg_number(logical_width);
    let height = svg_number(logical_height);
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    )
    .unwrap();
    let (background, background_opacity) = svg_color(frame.background);
    write!(
        svg,
        "<rect width=\"{width}\" height=\"{height}\" fill=\"{background}\""
    )
    .unwrap();
    write_opacity(&mut svg, "fill", background_opacity);
    svg.push_str("/>\n");

    for command in &frame.commands {
        match command {
            DrawCommand::PushTransform(matrix) => {
                writeln!(
                    svg,
                    "<g transform=\"matrix({} {} {} {} {} {})\">",
                    svg_number(matrix.m11),
                    svg_number(matrix.m12),
                    svg_number(matrix.m21),
                    svg_number(matrix.m22),
                    svg_number(matrix.tx),
                    svg_number(matrix.ty),
                )
                .unwrap();
            }
            DrawCommand::PopTransform => svg.push_str("</g>\n"),
            DrawCommand::Line {
                id,
                x1,
                y1,
                x2,
                y2,
                width,
                rgba,
            } => {
                let (color, opacity) = svg_color(*rgba);
                write!(
                    svg,
                    "<line data-command-id=\"{id}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{}\"",
                    svg_number(*x1),
                    svg_number(*y1),
                    svg_number(*x2),
                    svg_number(*y2),
                    svg_number(*width),
                )
                .unwrap();
                write_opacity(&mut svg, "stroke", opacity);
                svg.push_str(" stroke-linecap=\"round\"/>\n");
            }
            DrawCommand::Circle {
                id,
                x,
                y,
                radius,
                width,
                rgba,
                filled,
            } => {
                let (color, opacity) = svg_color(*rgba);
                write!(
                    svg,
                    "<circle data-command-id=\"{id}\" cx=\"{}\" cy=\"{}\" r=\"{}\"",
                    svg_number(*x),
                    svg_number(*y),
                    svg_number(*radius),
                )
                .unwrap();
                if *filled {
                    write!(svg, " fill=\"{color}\"").unwrap();
                    write_opacity(&mut svg, "fill", opacity);
                } else {
                    write!(
                        svg,
                        " fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\"",
                        svg_number(*width),
                    )
                    .unwrap();
                    write_opacity(&mut svg, "stroke", opacity);
                }
                svg.push_str("/>\n");
            }
            DrawCommand::Text {
                id,
                text,
                x,
                y,
                size,
                rgba,
                centered,
                font,
            } => {
                let (color, opacity) = svg_color(*rgba);
                write!(
                    svg,
                    "<text data-command-id=\"{id}\" x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"{color}\"",
                    svg_number(*x),
                    svg_number(*y),
                    match font {
                        TextFont::PlatformDefault => "sans-serif",
                        TextFont::GeistMonoRegular => GEIST_MONO_FAMILY,
                    },
                    svg_number(*size),
                )
                .unwrap();
                write_opacity(&mut svg, "fill", opacity);
                if *centered {
                    svg.push_str(" text-anchor=\"middle\"");
                }
                svg.push('>');
                write_xml_text(&mut svg, text);
                svg.push_str("</text>\n");
            }
            DrawCommand::Path {
                id,
                segments,
                width,
                fill_rgba,
                stroke_rgba,
            } => {
                write!(svg, "<path data-command-id=\"{id}\" d=\"").unwrap();
                for segment in segments {
                    match segment {
                        PathSegment::Move(point) => {
                            write!(svg, "M{} {}", svg_number(point.x), svg_number(point.y))
                                .unwrap();
                        }
                        PathSegment::Line(point) => {
                            write!(svg, "L{} {}", svg_number(point.x), svg_number(point.y))
                                .unwrap();
                        }
                        PathSegment::Quadratic { control, end } => {
                            write!(
                                svg,
                                "Q{} {} {} {}",
                                svg_number(control.x),
                                svg_number(control.y),
                                svg_number(end.x),
                                svg_number(end.y),
                            )
                            .unwrap();
                        }
                        PathSegment::Cubic {
                            control_1,
                            control_2,
                            end,
                        } => {
                            write!(
                                svg,
                                "C{} {} {} {} {} {}",
                                svg_number(control_1.x),
                                svg_number(control_1.y),
                                svg_number(control_2.x),
                                svg_number(control_2.y),
                                svg_number(end.x),
                                svg_number(end.y),
                            )
                            .unwrap();
                        }
                        PathSegment::Close => svg.push('Z'),
                    }
                }
                svg.push('"');
                if let Some(rgba) = fill_rgba {
                    let (color, opacity) = svg_color(*rgba);
                    write!(svg, " fill=\"{color}\"").unwrap();
                    write_opacity(&mut svg, "fill", opacity);
                } else {
                    svg.push_str(" fill=\"none\"");
                }
                if let Some(rgba) = stroke_rgba {
                    let (color, opacity) = svg_color(*rgba);
                    write!(
                        svg,
                        " stroke=\"{color}\" stroke-width=\"{}\"",
                        svg_number(*width),
                    )
                    .unwrap();
                    write_opacity(&mut svg, "stroke", opacity);
                }
                svg.push_str("/>\n");
            }
            DrawCommand::Sprite {
                id,
                image_id,
                source,
                destination,
                pivot,
                tint_rgba,
                flags,
            } => {
                let (color, opacity) = svg_color(*tint_rgba);
                write!(
                    svg,
                    "<rect data-command-id=\"{id}\" data-image-id=\"{image_id}\" data-source=\"{} {} {} {}\" data-pivot=\"{} {}\" data-flags=\"{flags}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{color}\"",
                    svg_number(source.x),
                    svg_number(source.y),
                    svg_number(source.width),
                    svg_number(source.height),
                    svg_number(pivot.x),
                    svg_number(pivot.y),
                    svg_number(destination.x),
                    svg_number(destination.y),
                    svg_number(destination.width),
                    svg_number(destination.height),
                )
                .unwrap();
                write_opacity(&mut svg, "stroke", opacity);
                svg.push_str(" stroke-dasharray=\"4 3\"/>\n");
            }
        }
    }
    svg.push_str("</svg>\n");
    svg
}

fn svg_number(number: f32) -> String {
    if number == 0.0 {
        "0".to_owned()
    } else {
        number.to_string()
    }
}

fn svg_color(rgba: u32) -> (String, u8) {
    (
        format!(
            "#{:02x}{:02x}{:02x}",
            rgba >> 24,
            (rgba >> 16) & 0xff,
            (rgba >> 8) & 0xff,
        ),
        (rgba & 0xff) as u8,
    )
}

fn write_opacity(svg: &mut String, attribute: &str, alpha: u8) {
    if alpha != u8::MAX {
        write!(
            svg,
            " {attribute}-opacity=\"{}\"",
            svg_number(f32::from(alpha) / 255.0),
        )
        .unwrap();
    }
}

fn write_xml_text(svg: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => svg.push_str("&amp;"),
            '<' => svg.push_str("&lt;"),
            '>' => svg.push_str("&gt;"),
            '"' => svg.push_str("&quot;"),
            '\'' => svg.push_str("&apos;"),
            _ => svg.push(character),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioEvent {
    pub id: u32,
    pub volume: f32,
    pub pitch: f32,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleAudioEvent {
    pub id: u32,
    pub volume: f32,
    pub pitch: f32,
    pub flags: u32,
}

impl AudioEvent {
    pub fn fire() -> Self {
        Self {
            id: 1,
            volume: 1.0,
            pitch: 1.0,
            flags: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostEffect {
    Redraw,
    Quit,
    SetCursor(i32),
    PersistSnapshot,
    OpenUrl,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    PointerMove {
        x: f32,
        y: f32,
    },
    PointerDown {
        button: u32,
        x: f32,
        y: f32,
    },
    PointerUp {
        button: u32,
        x: f32,
        y: f32,
    },
    PointerScroll {
        unit: PointerScrollUnit,
        delta_x: f32,
        delta_y: f32,
    },
    Viewport {
        width: f32,
        height: f32,
    },
    MenuAction(u32),
    Focus(bool),
    Control {
        id: u32,
        value: i32,
        phase: ControlPhase,
    },
    /// Host-reported nominal display mode. The numerator travels in the `code`
    /// slot and the bounded denominator in `a` for WAT ABI event kind 9.
    DisplayRefresh(TickRate),
    /// Host-owned scheduling lifecycle emitted instead of a declared trigger's
    /// raw physical edge. ABI event kind 15 reserves kinds 11-14 for touch.
    Pause(PausePhase),
}

/// Identifies the guest-visible stages of one host-owned suspension lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PausePhase {
    Paused = 1,
    Resumed = 2,
    Restored = 3,
}

/// Names the physical input domain of a declarative pause/wake selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PauseTriggerKind {
    Key = 1,
}

/// A bounded configure-time selector whose fresh physical edge is consumed by
/// Aedicule and converted into a semantic pause lifecycle transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseTrigger {
    pub kind: PauseTriggerKind,
    pub code: u32,
}

impl PauseTrigger {
    pub const fn key(key: Key) -> Self {
        Self {
            kind: PauseTriggerKind::Key,
            code: key as u32,
        }
    }

    fn matches_key(self, key: Key) -> bool {
        self.kind == PauseTriggerKind::Key && self.code == key as u32
    }
}

/// Separates continuous slider motion from its final committed release edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ControlPhase {
    Change = 1,
    Release = 2,
}

/// Identifies the portable pointer buttons that Aedicule adapters currently
/// expose to WAT. Keeping the numeric IDs here makes native and browser
/// delivery agree without leaking their distinct platform button types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PointerButton {
    Primary = 1,
    Secondary = 2,
    Middle = 3,
}

impl PointerButton {
    /// Creates the stable ABI edge event while retaining logical-pixel
    /// coordinates outside the WAT-specific numeric event conversion.
    pub fn down(self, x: f32, y: f32) -> Event {
        Event::PointerDown {
            button: self as u32,
            x,
            y,
        }
    }

    /// Creates the matching release edge so guests can clear button state
    /// even when native and web adapters represent mouse input differently.
    pub fn up(self, x: f32, y: f32) -> Event {
        Event::PointerUp {
            button: self as u32,
            x,
            y,
        }
    }
}

/// Preserves whether GPUI reported discrete wheel lines or precise logical
/// pixels, avoiding a platform-dependent conversion constant at the WAT ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PointerScrollUnit {
    Lines = 1,
    LogicalPixels = 2,
}

impl PointerScrollUnit {
    /// Creates a two-axis scroll edge while omitting phase-only events whose
    /// zero deltas contain no guest-observable wheel movement.
    pub fn event(self, delta_x: f32, delta_y: f32) -> Option<Event> {
        if delta_x == 0.0 && delta_y == 0.0 {
            return None;
        }
        Some(Event::PointerScroll {
            unit: self,
            delta_x,
            delta_y,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Key {
    ArrowLeft = 1,
    ArrowRight = 2,
    ArrowUp = 3,
    Space = 4,
    P = 5,
    R = 6,
    F = 7,
    K = 8,
    B = 9,
    H = 10,
    Escape = 11,
    F1 = 12,
    W = 13,
    A = 14,
    D = 15,
}

impl Key {
    /// Validates a guest-declared stable key ID without accepting arbitrary
    /// numeric values as future ABI meanings.
    pub fn from_abi(code: u32) -> Option<Self> {
        match code {
            1 => Some(Self::ArrowLeft),
            2 => Some(Self::ArrowRight),
            3 => Some(Self::ArrowUp),
            4 => Some(Self::Space),
            5 => Some(Self::P),
            6 => Some(Self::R),
            7 => Some(Self::F),
            8 => Some(Self::K),
            9 => Some(Self::B),
            10 => Some(Self::H),
            11 => Some(Self::Escape),
            12 => Some(Self::F1),
            13 => Some(Self::W),
            14 => Some(Self::A),
            15 => Some(Self::D),
            _ => None,
        }
    }

    /// Normalizes GPUI's cross-platform physical key names into Aedicule's
    /// stable guest IDs so native and browser adapters cannot drift.
    pub fn from_gpui_name(name: &str) -> Option<Self> {
        match name {
            "left" => Some(Self::ArrowLeft),
            "right" => Some(Self::ArrowRight),
            "up" => Some(Self::ArrowUp),
            "space" | " " => Some(Self::Space),
            "p" => Some(Self::P),
            "r" => Some(Self::R),
            "f" => Some(Self::F),
            "k" => Some(Self::K),
            "b" => Some(Self::B),
            "h" => Some(Self::H),
            "escape" => Some(Self::Escape),
            "f1" => Some(Self::F1),
            "w" => Some(Self::W),
            "a" => Some(Self::A),
            "d" => Some(Self::D),
            _ => None,
        }
    }
}

/// Tells an adapter whether to deliver, consume, or perform a host-owned
/// suspension transition for one raw input edge.
#[derive(Debug, Clone, PartialEq)]
pub enum SuspensionDisposition {
    Deliver(Event),
    Maintenance(Event),
    Consumed,
    Pause,
    Resume { reconciliation: Vec<Event> },
}

/// Tracks physical-versus-guest input ownership while Aedicule has stopped the
/// guest scheduler, preventing stuck controls and held-key self-resume.
#[derive(Debug, Clone)]
pub struct GuestSuspension {
    triggers: Vec<PauseTrigger>,
    suspended: bool,
    physical_keys: BTreeSet<Key>,
    consumed_trigger_keys: BTreeSet<Key>,
    guest_keys: BTreeSet<Key>,
    physical_pointer_buttons: BTreeSet<u32>,
    guest_pointer_buttons: BTreeMap<u32, (f32, f32)>,
    reconciliation: Vec<Event>,
    triggers_after_resume: Option<Vec<PauseTrigger>>,
}

impl GuestSuspension {
    pub fn new(triggers: impl IntoIterator<Item = PauseTrigger>) -> Self {
        Self {
            triggers: triggers.into_iter().collect(),
            suspended: false,
            physical_keys: BTreeSet::new(),
            consumed_trigger_keys: BTreeSet::new(),
            guest_keys: BTreeSet::new(),
            physical_pointer_buttons: BTreeSet::new(),
            guest_pointer_buttons: BTreeMap::new(),
            reconciliation: Vec::new(),
            triggers_after_resume: None,
        }
    }

    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    pub fn is_enabled(&self) -> bool {
        !self.triggers.is_empty() || self.triggers_after_resume.is_some()
    }

    fn is_trigger(&self, key: Key) -> bool {
        self.triggers.iter().any(|trigger| trigger.matches_key(key))
    }

    /// Adopts replacement metadata without stranding an already-paused guest
    /// that removed every wake selector during hot reload.
    pub fn replace_triggers(&mut self, triggers: impl IntoIterator<Item = PauseTrigger>) {
        let triggers = triggers.into_iter().collect::<Vec<_>>();
        if self.suspended {
            if !triggers.is_empty() {
                self.triggers.clone_from(&triggers);
            }
            self.triggers_after_resume = Some(triggers);
        } else {
            self.triggers = triggers;
        }
    }

    /// Classifies fresh edges and records releases that must reach the guest
    /// before its semantic resumed event.
    pub fn handle(&mut self, event: Event) -> SuspensionDisposition {
        if !self.is_enabled() {
            return SuspensionDisposition::Deliver(event);
        }
        match event {
            Event::KeyDown(key) => {
                let fresh = self.physical_keys.insert(key);
                if self.is_trigger(key) {
                    if !fresh {
                        return SuspensionDisposition::Consumed;
                    }
                    self.consumed_trigger_keys.insert(key);
                    if self.suspended {
                        self.suspended = false;
                        if let Some(triggers) = self.triggers_after_resume.take() {
                            self.triggers = triggers;
                        }
                        return SuspensionDisposition::Resume {
                            reconciliation: std::mem::take(&mut self.reconciliation),
                        };
                    }
                    self.suspended = true;
                    return SuspensionDisposition::Pause;
                }
                if self.suspended || !fresh {
                    return SuspensionDisposition::Consumed;
                }
                self.guest_keys.insert(key);
                SuspensionDisposition::Deliver(event)
            }
            Event::KeyUp(key) => {
                self.physical_keys.remove(&key);
                if self.consumed_trigger_keys.remove(&key) {
                    return SuspensionDisposition::Consumed;
                }
                if self.is_trigger(key) {
                    return SuspensionDisposition::Consumed;
                }
                if self.suspended {
                    if self.guest_keys.remove(&key) {
                        self.reconciliation.push(event);
                    }
                    SuspensionDisposition::Consumed
                } else {
                    self.guest_keys.remove(&key);
                    SuspensionDisposition::Deliver(event)
                }
            }
            Event::PointerDown { button, x, y } => {
                let fresh = self.physical_pointer_buttons.insert(button);
                if self.suspended || !fresh {
                    return SuspensionDisposition::Consumed;
                }
                self.guest_pointer_buttons.insert(button, (x, y));
                SuspensionDisposition::Deliver(event)
            }
            Event::PointerUp { button, x, y } => {
                self.physical_pointer_buttons.remove(&button);
                if self.suspended {
                    if self.guest_pointer_buttons.remove(&button).is_some() {
                        self.reconciliation.push(Event::PointerUp { button, x, y });
                    }
                    SuspensionDisposition::Consumed
                } else {
                    self.guest_pointer_buttons.remove(&button);
                    SuspensionDisposition::Deliver(event)
                }
            }
            Event::Focus(false) => {
                self.physical_keys.clear();
                self.physical_pointer_buttons.clear();
                if self.suspended {
                    self.reconciliation.extend(
                        std::mem::take(&mut self.guest_keys)
                            .into_iter()
                            .map(Event::KeyUp),
                    );
                    self.reconciliation.extend(
                        std::mem::take(&mut self.guest_pointer_buttons)
                            .into_iter()
                            .map(|(button, (x, y))| Event::PointerUp { button, x, y }),
                    );
                    self.reconciliation.push(Event::Focus(false));
                    SuspensionDisposition::Consumed
                } else {
                    self.guest_keys.clear();
                    self.guest_pointer_buttons.clear();
                    SuspensionDisposition::Deliver(event)
                }
            }
            Event::Focus(true) if self.suspended => {
                self.reconciliation.push(event);
                SuspensionDisposition::Consumed
            }
            Event::Viewport { .. } | Event::DisplayRefresh(_) if self.suspended => {
                SuspensionDisposition::Maintenance(event)
            }
            _ if self.suspended => SuspensionDisposition::Consumed,
            _ => SuspensionDisposition::Deliver(event),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub schema: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluginInit {
    pub seed: u64,
    pub viewport_width: f32,
    pub viewport_height: f32,
    /// The nominal display timing known at this lifecycle boundary. It is
    /// delivered once as an event before the guest chooses its simulation rate.
    pub display_refresh: TickRate,
}

impl PluginInit {
    pub const fn new(seed: u64, viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            seed,
            viewport_width,
            viewport_height,
            display_refresh: TickRate::new(DEFAULT_SIMULATION_HZ, 1),
        }
    }

    /// Replaces the deterministic default display mode for a host lifecycle
    /// boundary without giving the guest an ambient display-query capability.
    pub const fn with_display_refresh(mut self, display_refresh: TickRate) -> Self {
        self.display_refresh = display_refresh;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransfer {
    Preserved,
    Restarted,
}

/// A display-timing event can retain or replace the simulation rate after the
/// guest has processed the new display mode and selected its own policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationRateChange {
    pub previous: TickRate,
    pub current: TickRate,
}

#[derive(Debug)]
pub struct PreparedReload {
    pub frontplane: Frontplane,
    pub frame: FrameOutput,
    pub state_transfer: StateTransfer,
}

impl PreparedReload {
    /// Completes candidate admission in the suspended lifecycle before an
    /// adapter atomically replaces the current guest.
    pub fn restore_suspension(&mut self) -> Result<(), FrontplaneError> {
        self.frontplane.event(Event::Pause(PausePhase::Restored))?;
        self.frame = self.frontplane.render()?;
        self.frontplane.drain_audio();
        self.frontplane.drain_sample_audio();
        self.frontplane.drain_effects();
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum FrontplaneError {
    #[error("could not load application: {0}")]
    Application(String),
    #[error("could not parse WAT: {0}")]
    Wat(String),
    #[error("could not compile or instantiate WebAssembly: {0}")]
    Runtime(String),
    #[error("unsupported import {module}.{name}")]
    UnsupportedImport { module: String, name: String },
    #[error("required export {name} is missing or has the wrong type")]
    MissingExport { name: &'static str },
    #[error("ABI major mismatch: host {expected}, plugin {found}")]
    WrongAbi { expected: i32, found: i32 },
    #[error("ABI minor mismatch: host supports 0..={supported}, plugin requires {found}")]
    UnsupportedAbiMinor { supported: i32, found: i32 },
    #[error("plugin tick rate {found} Hz is outside the supported 1..={maximum} range")]
    InvalidSimulationHz { found: i32, maximum: u32 },
    #[error(
        "plugin rational tick rate {numerator}/{denominator} Hz is outside the supported 1..={maximum} range"
    )]
    InvalidSimulationRate {
        numerator: i32,
        denominator: i32,
        maximum: u32,
    },
    #[error("plugin exhausted its fuel during {operation}")]
    FuelExhausted { operation: &'static str },
    #[error("plugin trapped during {operation}: {message}")]
    Trap {
        operation: &'static str,
        message: String,
    },
    #[error("plugin returned status {status} from {operation}")]
    PluginStatus {
        operation: &'static str,
        status: i32,
    },
    #[error("invalid pointer or length in {operation}")]
    InvalidPointer { operation: &'static str },
    #[error("invalid UTF-8 in {operation}")]
    InvalidUtf8 { operation: &'static str },
    #[error("invalid or non-finite number in {operation}")]
    InvalidNumber { operation: &'static str },
    #[error("{kind} budget exhausted in {operation}")]
    BudgetExhausted {
        operation: &'static str,
        kind: &'static str,
    },
    #[error("invalid frame lifecycle: {0}")]
    InvalidFrame(&'static str),
    #[error("duplicate stable ID {id} in {operation}")]
    DuplicateId { operation: &'static str, id: u32 },
    #[error("unsupported or unavailable capability in {operation}")]
    UnsupportedCapability { operation: &'static str },
    #[error("snapshot schema or size does not match the plugin")]
    SnapshotMismatch,
}

#[derive(Debug, Clone)]
enum PendingError {
    InvalidPointer(&'static str),
    InvalidUtf8(&'static str),
    InvalidNumber(&'static str),
    Budget(&'static str, &'static str),
    InvalidFrame(&'static str),
    DuplicateId(&'static str, u32),
    Unsupported(&'static str),
}

impl PendingError {
    fn into_public(self) -> FrontplaneError {
        match self {
            Self::InvalidPointer(operation) => FrontplaneError::InvalidPointer { operation },
            Self::InvalidUtf8(operation) => FrontplaneError::InvalidUtf8 { operation },
            Self::InvalidNumber(operation) => FrontplaneError::InvalidNumber { operation },
            Self::Budget(operation, kind) => FrontplaneError::BudgetExhausted { operation, kind },
            Self::InvalidFrame(message) => FrontplaneError::InvalidFrame(message),
            Self::DuplicateId(operation, id) => FrontplaneError::DuplicateId { operation, id },
            Self::Unsupported(operation) => FrontplaneError::UnsupportedCapability { operation },
        }
    }
}

struct FrameBuilder {
    background: u32,
    commands: Vec<DrawCommand>,
    ids: HashSet<u32>,
}

struct UiBuilder {
    revision: u32,
    control_panels: Vec<ControlPanel>,
    control_panel_ids: HashSet<u32>,
    sliders: Vec<SliderPlacement>,
    slider_ids: HashSet<u32>,
    buttons: Vec<ButtonPlacement>,
    button_ids: HashSet<u32>,
}

struct PathBuilder {
    id: u32,
    segments: Vec<PathSegment>,
}

struct HostState {
    limits: Limits,
    store_limits: StoreLimits,
    metadata: Metadata,
    application_assets: ApplicationAssets,
    menu_ids: HashSet<u32>,
    control_ids: HashSet<u32>,
    sample_ids: HashSet<u32>,
    images: Vec<ImageResource>,
    image_ids: HashSet<u32>,
    frame: Option<FrameBuilder>,
    transform_depth: usize,
    current_path: Option<PathBuilder>,
    completed_frame: Option<FrameOutput>,
    ui: Option<UiBuilder>,
    pending_ui: Option<UiSnapshot>,
    accepted_ui: Option<UiSnapshot>,
    audio: Vec<AudioEvent>,
    sample_audio: Vec<SampleAudioEvent>,
    effects: Vec<HostEffect>,
    audio_checkpoint: usize,
    sample_audio_checkpoint: usize,
    effect_checkpoint: usize,
    pending_error: Option<PendingError>,
    active_operation: Option<&'static str>,
}

impl HostState {
    fn audio_event_budget_exhausted(&self) -> bool {
        self.audio.len().saturating_add(self.sample_audio.len()) >= self.limits.max_audio_events
    }

    fn reject(&mut self, error: PendingError, status: i32) -> i32 {
        if self.pending_error.is_none() {
            self.pending_error = Some(error);
        }
        status
    }
}

#[derive(Clone)]
enum InitExport {
    Legacy(TypedFunc<(i32, i32, f32, f32), i32>),
    Integer(TypedFunc<(i32, i32, i32, i32), i32>),
}

#[derive(Clone)]
enum EventExport {
    Legacy(TypedFunc<(i32, i32, f32, f32), i32>),
    Integer(TypedFunc<(i32, i32, i32, i32), i32>),
}

struct Exports {
    abi_major: TypedFunc<(), i32>,
    abi_minor: TypedFunc<(), i32>,
    configure: TypedFunc<(), i32>,
    init: InitExport,
    event: EventExport,
    control_event: Option<TypedFunc<(i32, i32, i32), i32>>,
    tick: TypedFunc<i32, i32>,
    render: TypedFunc<(), i32>,
    state_ptr: TypedFunc<(), i32>,
    state_len: TypedFunc<(), i32>,
    state_schema: TypedFunc<(), i32>,
    tick_rate: Option<TypedFunc<(i32, i32), (i32, i32)>>,
    after_restore: Option<TypedFunc<(), i32>>,
}

/// Owns one untrusted plugin instance and exposes only validated plain data.
pub struct Frontplane {
    store: Store<HostState>,
    _memory: Memory,
    _instance: Instance,
    exports: Exports,
    simulation_rate: TickRate,
    display_refresh: Option<TickRate>,
}

impl fmt::Debug for Frontplane {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Frontplane")
            .field("metadata", &self.store.data().metadata)
            .finish_non_exhaustive()
    }
}

impl Frontplane {
    /// Resolves code and immutable assets through one virtual-root adapter so a
    /// bare WAT, source directory, and `.aed` instantiate identically.
    pub fn from_application(
        source: &PluginSource,
        limits: Limits,
    ) -> Result<Self, FrontplaneError> {
        let code = read_application_file(source, DEFAULT_PLUGIN_FILE)
            .map_err(FrontplaneError::Application)?;
        let source_text = String::from_utf8(code)
            .map_err(|error| FrontplaneError::Application(error.to_string()))?;
        let assets = read_application_assets(source).map_err(FrontplaneError::Application)?;
        Self::from_wat_with_assets(&source_text, limits, assets)
    }

    /// Compiles WAT, rejects ambient capabilities, binds the bounded host ABI,
    /// and validates all lifecycle export signatures before returning.
    pub fn from_wat(source: &str, limits: Limits) -> Result<Self, FrontplaneError> {
        Self::from_wat_with_assets(source, limits, ApplicationAssets::new())
    }

    /// Instantiates a guest with an immutable, prevalidated virtual-root asset
    /// catalog; imports can bind only named entries and never ambient files.
    pub fn from_wat_with_assets(
        source: &str,
        limits: Limits,
        application_assets: ApplicationAssets,
    ) -> Result<Self, FrontplaneError> {
        let bytes =
            wat::parse_str(source).map_err(|error| FrontplaneError::Wat(error.to_string()))?;
        #[cfg(feature = "native-runtime")]
        let mut config = Config::new();
        #[cfg(feature = "portable-runtime")]
        let mut config = Config::default();
        config.consume_fuel(true);
        #[cfg(feature = "native-runtime")]
        let engine = Engine::new(&config).map_err(runtime_error)?;
        #[cfg(feature = "portable-runtime")]
        let engine = Engine::new(&config);
        let module = Module::new(&engine, bytes).map_err(runtime_error)?;

        for import in module.imports() {
            if import.module() != WAT_IMPORT_MODULE
                || !WAT_ABI_IMPORTS
                    .iter()
                    .any(|supported| supported.name == import.name())
            {
                return Err(FrontplaneError::UnsupportedImport {
                    module: import.module().to_owned(),
                    name: import.name().to_owned(),
                });
            }
        }

        let mut linker = Linker::new(&engine);
        bind_host_functions(&mut linker)?;
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes)
            .table_elements(limits.max_table_elements)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();
        let state = HostState {
            limits: limits.clone(),
            store_limits,
            metadata: Metadata::default(),
            application_assets,
            menu_ids: HashSet::new(),
            control_ids: HashSet::new(),
            sample_ids: HashSet::new(),
            images: Vec::new(),
            image_ids: HashSet::new(),
            frame: None,
            transform_depth: 0,
            current_path: None,
            completed_frame: None,
            ui: None,
            pending_ui: None,
            accepted_ui: None,
            audio: Vec::new(),
            sample_audio: Vec::new(),
            effects: Vec::new(),
            audio_checkpoint: 0,
            sample_audio_checkpoint: 0,
            effect_checkpoint: 0,
            pending_error: None,
            active_operation: None,
        };
        let mut store = Store::new(&engine, state);
        store.limiter(|state| &mut state.store_limits);
        store
            .set_fuel(limits.fuel_per_call)
            .map_err(runtime_error)?;
        #[cfg(feature = "native-runtime")]
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(runtime_error)?;
        #[cfg(feature = "portable-runtime")]
        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(runtime_error)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or(FrontplaneError::MissingExport { name: "memory" })?;
        let integer_lifecycle = instance.get_export(&mut store, "AE_init_i32").is_some()
            || instance.get_export(&mut store, "AE_event_i32").is_some();
        let (init, event) = if integer_lifecycle {
            (
                InitExport::Integer(required_func(&instance, &mut store, "AE_init_i32")?),
                EventExport::Integer(required_func(&instance, &mut store, "AE_event_i32")?),
            )
        } else {
            (
                InitExport::Legacy(required_func(&instance, &mut store, "AE_init")?),
                EventExport::Legacy(required_func(&instance, &mut store, "AE_event")?),
            )
        };
        let exports = Exports {
            abi_major: required_func(&instance, &mut store, "AE_abi_major")?,
            abi_minor: required_func(&instance, &mut store, "AE_abi_minor")?,
            configure: required_func(&instance, &mut store, "AE_configure")?,
            init,
            event,
            control_event: instance
                .get_typed_func::<(i32, i32, i32), i32>(&mut store, "AE_control_event")
                .ok(),
            tick: required_func(&instance, &mut store, "AE_tick")?,
            render: required_func(&instance, &mut store, "AE_render")?,
            state_ptr: required_func(&instance, &mut store, "AE_state_ptr")?,
            state_len: required_func(&instance, &mut store, "AE_state_len")?,
            state_schema: required_func(&instance, &mut store, "AE_state_schema")?,
            tick_rate: optional_func(&instance, &mut store, "AE_tick_rate")?,
            after_restore: instance
                .get_typed_func::<(), i32>(&mut store, "AE_after_restore")
                .ok(),
        };
        let mut frontplane = Self {
            store,
            _memory: memory,
            _instance: instance,
            exports,
            simulation_rate: TickRate::from(DEFAULT_SIMULATION_HZ),
            display_refresh: None,
        };
        let abi_major_function = frontplane.exports.abi_major.clone();
        let abi_major = frontplane.call_value("AE_abi_major", abi_major_function)?;
        if abi_major != ABI_MAJOR {
            return Err(FrontplaneError::WrongAbi {
                expected: ABI_MAJOR,
                found: abi_major,
            });
        }
        let abi_minor_function = frontplane.exports.abi_minor.clone();
        let abi_minor = frontplane.call_value("AE_abi_minor", abi_minor_function)?;
        if !(0..=ABI_MINOR).contains(&abi_minor) {
            return Err(FrontplaneError::UnsupportedAbiMinor {
                supported: ABI_MINOR,
                found: abi_minor,
            });
        }
        Ok(frontplane)
    }

    /// Runs the plugin's declarative metadata phase transactionally, discarding
    /// partial menus and resources if any host import or export fails.
    pub fn configure(&mut self) -> Result<(), FrontplaneError> {
        self.store.data_mut().metadata = Metadata::default();
        self.store.data_mut().menu_ids.clear();
        self.store.data_mut().control_ids.clear();
        self.store.data_mut().sample_ids.clear();
        self.store.data_mut().images.clear();
        self.store.data_mut().image_ids.clear();
        self.store.data_mut().sample_audio.clear();
        let function = self.exports.configure.clone();
        let result = self.call_status("AE_configure", function, ());
        if result.is_ok()
            && !self.store.data().metadata.controls.is_empty()
            && self.exports.control_event.is_none()
        {
            let state = self.store.data_mut();
            state.metadata = Metadata::default();
            state.menu_ids.clear();
            state.control_ids.clear();
            state.sample_ids.clear();
            state.images.clear();
            state.image_ids.clear();
            state.sample_audio.clear();
            return Err(FrontplaneError::MissingExport {
                name: "AE_control_event",
            });
        }
        if result.is_err() {
            let state = self.store.data_mut();
            state.metadata = Metadata::default();
            state.menu_ids.clear();
            state.control_ids.clear();
            state.sample_ids.clear();
            state.images.clear();
            state.image_ids.clear();
            state.sample_audio.clear();
        }
        result
    }

    /// Injects the only entropy and initial viewport available to the plugin;
    /// no ambient clock, RNG, filesystem, or process state is exposed.
    pub fn init(
        &mut self,
        seed: u64,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Result<(), FrontplaneError> {
        validate_bounded_numbers(
            "AE_init",
            &[viewport_width, viewport_height],
            self.store.data().limits.max_coordinate_abs,
        )?;
        if viewport_width <= 0.0 || viewport_height <= 0.0 {
            return Err(FrontplaneError::InvalidNumber {
                operation: "AE_init",
            });
        }
        match self.exports.init.clone() {
            InitExport::Legacy(function) => self.call_status(
                "AE_init",
                function,
                (
                    seed as i32,
                    (seed >> 32) as i32,
                    viewport_width,
                    viewport_height,
                ),
            ),
            InitExport::Integer(function) => {
                let width = logical_to_q16(
                    viewport_width,
                    self.store.data().limits.max_coordinate_abs,
                    "AE_init_i32",
                )?;
                let height = logical_to_q16(
                    viewport_height,
                    self.store.data().limits.max_coordinate_abs,
                    "AE_init_i32",
                )?;
                self.call_status(
                    "AE_init_i32",
                    function,
                    (seed as i32, (seed >> 32) as i32, width, height),
                )
            }
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.store.data().metadata
    }

    /// Returns the currently agreed exact simulation rate. Before a host
    /// supplies display timing, compatibility startup uses 60/1 Hz.
    pub fn simulation_rate(&self) -> TickRate {
        self.simulation_rate
    }

    /// Delivers a newly observed display mode exactly once, then asks the
    /// guest to choose a rational simulation rate using the prior agreement.
    /// A `(0, 0)` guest response follows the just-delivered display rate.
    pub fn observe_display_refresh(
        &mut self,
        refresh: TickRate,
    ) -> Result<Option<SimulationRateChange>, FrontplaneError> {
        if self.display_refresh == Some(refresh) {
            return Ok(None);
        }

        let previous = self.simulation_rate;
        if self.display_refresh.is_none() {
            self.simulation_rate = refresh;
        }
        self.display_refresh = Some(refresh);
        self.event(Event::DisplayRefresh(refresh))?;
        self.simulation_rate = self.select_simulation_rate()?;

        Ok(Some(SimulationRateChange {
            previous,
            current: self.simulation_rate,
        }))
    }

    pub fn images(&self) -> &[ImageResource] {
        &self.store.data().images
    }

    /// Exposes the last fully accepted declarative UI revision; omitted guest
    /// updates leave this snapshot intact for retained adapters to reconcile.
    pub fn ui_snapshot(&self) -> Option<&UiSnapshot> {
        self.store.data().accepted_ui.as_ref()
    }

    /// Converts portable typed input into the stable numeric ABI, keeping GPUI
    /// and platform key representations outside the guest boundary.
    pub fn event(&mut self, event: Event) -> Result<(), FrontplaneError> {
        if let Event::Control { id, value, phase } = event {
            let Some(control) = self
                .store
                .data()
                .metadata
                .controls
                .iter()
                .find(|control| control.id == id)
            else {
                return Err(FrontplaneError::UnsupportedCapability {
                    operation: "AE_control_event",
                });
            };
            if !control.contains_value(value) {
                return Err(FrontplaneError::InvalidNumber {
                    operation: "AE_control_event",
                });
            }
            let function =
                self.exports
                    .control_event
                    .clone()
                    .ok_or(FrontplaneError::MissingExport {
                        name: "AE_control_event",
                    })?;
            return self.call_status(
                "AE_control_event",
                function,
                (id as i32, value, phase as i32),
            );
        }
        match self.exports.event.clone() {
            EventExport::Legacy(function) => {
                let (kind, code, a, b) = legacy_event_parameters(event);
                validate_bounded_numbers(
                    "AE_event",
                    &[a, b],
                    self.store.data().limits.max_coordinate_abs,
                )?;
                self.call_status("AE_event", function, (kind, code, a, b))
            }
            EventExport::Integer(function) => {
                let parameters =
                    integer_event_parameters(event, self.store.data().limits.max_coordinate_abs)?;
                self.call_status("AE_event_i32", function, parameters)
            }
        }
    }

    /// Resolves the guest's stateful rate policy after it has received the
    /// display event, keeping display discovery outside the selector arguments.
    fn select_simulation_rate(&mut self) -> Result<TickRate, FrontplaneError> {
        if let Some(function) = self.exports.tick_rate.clone() {
            let current = self.simulation_rate;
            let (numerator, denominator) = self.call_tick_rate(function, current)?;
            if numerator == 0 && denominator == 0 {
                return Ok(self
                    .display_refresh
                    .expect("display refresh is set before selection"));
            }
            return validate_tick_rate(numerator, denominator);
        }
        Ok(self
            .display_refresh
            .expect("display refresh is set before selection"))
    }

    /// Advances deterministic simulation time by a bounded integer count.
    pub fn tick(&mut self, ticks: u32) -> Result<(), FrontplaneError> {
        if ticks > self.store.data().limits.max_ticks_per_call {
            return Err(FrontplaneError::BudgetExhausted {
                operation: "AE_tick",
                kind: "tick",
            });
        }
        let function = self.exports.tick.clone();
        self.call_status("AE_tick", function, ticks as i32)
    }

    /// Captures one all-or-nothing immutable command buffer, rejecting partial
    /// frames and balanced-stack violations before an adapter can paint them.
    pub fn render(&mut self) -> Result<FrameOutput, FrontplaneError> {
        {
            let state = self.store.data_mut();
            state.frame = None;
            state.transform_depth = 0;
            state.current_path = None;
            state.completed_frame = None;
            state.ui = None;
            state.pending_ui = None;
        }
        let function = self.exports.render.clone();
        self.call_status("AE_render", function, ())?;
        let state = self.store.data_mut();
        if state.ui.is_some() {
            state.ui = None;
            state.pending_ui = None;
            return Err(FrontplaneError::InvalidFrame(
                "AE_render left an incomplete UI snapshot",
            ));
        }
        let frame = state
            .completed_frame
            .take()
            .ok_or(FrontplaneError::InvalidFrame(
                "AE_render did not complete a frame",
            ))?;
        if let Some(snapshot) = state.pending_ui.take() {
            state.accepted_ui = Some(snapshot);
        }
        Ok(frame)
    }

    /// Copies the guest-declared opaque state region after independently
    /// validating its pointer, schema, and configured byte limit.
    pub fn snapshot(&mut self) -> Result<Snapshot, FrontplaneError> {
        let (pointer, length, schema) = self.state_region()?;
        let bytes = self
            ._memory
            .data(&self.store)
            .get(pointer..pointer + length)
            .ok_or(FrontplaneError::InvalidPointer {
                operation: "snapshot",
            })?
            .to_vec();
        Ok(Snapshot { schema, bytes })
    }

    /// Restores only an exact schema/length match so malformed or stale state
    /// cannot partially overwrite the current simulation.
    pub fn restore(&mut self, snapshot: &Snapshot) -> Result<(), FrontplaneError> {
        let (pointer, length, schema) = self.state_region()?;
        if schema != snapshot.schema || length != snapshot.bytes.len() {
            return Err(FrontplaneError::SnapshotMismatch);
        }
        self._memory
            .write(&mut self.store, pointer, &snapshot.bytes)
            .map_err(|_| FrontplaneError::InvalidPointer {
                operation: "restore",
            })?;
        if let Some(after_restore) = self.exports.after_restore.clone() {
            self.call_status("AE_after_restore", after_restore, ())?;
        }
        Ok(())
    }

    pub fn drain_audio(&mut self) -> Vec<AudioEvent> {
        std::mem::take(&mut self.store.data_mut().audio)
    }

    pub fn drain_sample_audio(&mut self) -> Vec<SampleAudioEvent> {
        std::mem::take(&mut self.store.data_mut().sample_audio)
    }

    pub fn drain_effects(&mut self) -> Vec<HostEffect> {
        std::mem::take(&mut self.store.data_mut().effects)
    }

    /// Negotiates the opaque snapshot slice while guarding signed conversions,
    /// arithmetic overflow, guest memory bounds, and host policy.
    fn state_region(&mut self) -> Result<(usize, usize, u32), FrontplaneError> {
        let state_ptr = self.exports.state_ptr.clone();
        let state_len = self.exports.state_len.clone();
        let state_schema = self.exports.state_schema.clone();
        let pointer = self.call_value("AE_state_ptr", state_ptr)?;
        let length = self.call_value("AE_state_len", state_len)?;
        let schema = self.call_value("AE_state_schema", state_schema)?;
        if pointer < 0
            || length < 0
            || length as usize > self.store.data().limits.max_snapshot_bytes
            || schema < 0
        {
            return Err(FrontplaneError::InvalidPointer {
                operation: "snapshot",
            });
        }
        let pointer = pointer as usize;
        let length = length as usize;
        if pointer
            .checked_add(length)
            .is_none_or(|end| end > self._memory.data_size(&self.store))
        {
            return Err(FrontplaneError::InvalidPointer {
                operation: "snapshot",
            });
        }
        Ok((pointer, length, schema as u32))
    }

    fn call_value(
        &mut self,
        operation: &'static str,
        function: TypedFunc<(), i32>,
    ) -> Result<i32, FrontplaneError> {
        self.prepare_call(operation)?;
        let result = function.call(&mut self.store, ());
        self.finish_call(operation, result)
    }

    fn call_tick_rate(
        &mut self,
        function: TypedFunc<(i32, i32), (i32, i32)>,
        current: TickRate,
    ) -> Result<(i32, i32), FrontplaneError> {
        self.prepare_call("AE_tick_rate")?;
        let result = function.call(
            &mut self.store,
            (current.numerator as i32, current.denominator as i32),
        );
        self.finish_call("AE_tick_rate", result)
    }

    fn call_status<P>(
        &mut self,
        operation: &'static str,
        function: TypedFunc<P, i32>,
        parameters: P,
    ) -> Result<(), FrontplaneError>
    where
        P: runtime::WasmParams,
    {
        self.prepare_call(operation)?;
        let result = function.call(&mut self.store, parameters);
        let status = self.finish_call(operation, result)?;
        if status == 0 {
            Ok(())
        } else {
            self.rollback_outputs();
            Err(FrontplaneError::PluginStatus { operation, status })
        }
    }

    /// Establishes fuel and output checkpoints that make each untrusted guest
    /// export an independently bounded transaction.
    fn prepare_call(&mut self, operation: &'static str) -> Result<(), FrontplaneError> {
        let state = self.store.data_mut();
        state.pending_error = None;
        state.audio_checkpoint = state.audio.len();
        state.sample_audio_checkpoint = state.sample_audio.len();
        state.effect_checkpoint = state.effects.len();
        self.store
            .set_fuel(self.store.data().limits.fuel_per_call)
            .map_err(runtime_error)?;
        self.store.data_mut().active_operation = Some(operation);
        Ok(())
    }

    /// Converts Wasmtime traps and host-import rejections into typed failures,
    /// rolling back semantic side effects from the failed export.
    fn finish_call<T>(
        &mut self,
        operation: &'static str,
        result: Result<T, runtime::Error>,
    ) -> Result<T, FrontplaneError> {
        let state = self.store.data_mut();
        let pending_error = state.pending_error.take();
        state.active_operation = None;
        let outcome = if let Some(error) = pending_error {
            Err(error.into_public())
        } else {
            match result {
                Ok(value) => Ok(value),
                Err(error) => {
                    if self.store.get_fuel().is_ok_and(|fuel| fuel == 0) {
                        Err(FrontplaneError::FuelExhausted { operation })
                    } else {
                        Err(FrontplaneError::Trap {
                            operation,
                            message: error.to_string(),
                        })
                    }
                }
            }
        };
        if outcome.is_err() {
            self.rollback_outputs();
        }
        outcome
    }

    /// Truncates only events emitted since the current call began, preserving
    /// prior successful output until the adapter drains it.
    fn rollback_outputs(&mut self) {
        let state = self.store.data_mut();
        state.audio.truncate(state.audio_checkpoint);
        state.sample_audio.truncate(state.sample_audio_checkpoint);
        state.effects.truncate(state.effect_checkpoint);
        state.ui = None;
        state.pending_ui = None;
    }
}

/// Applies the shared startup lifecycle so every frontplane adapter delivers
/// the first display mode before the optional guest rate selector runs.
pub fn initialize_frontplane(
    frontplane: &mut Frontplane,
    init: PluginInit,
) -> Result<(), FrontplaneError> {
    frontplane.configure()?;
    frontplane.init(init.seed, init.viewport_width, init.viewport_height)?;
    frontplane.observe_display_refresh(init.display_refresh)?;
    Ok(())
}

fn validate_tick_rate(numerator: i32, denominator: i32) -> Result<TickRate, FrontplaneError> {
    if numerator <= 0
        || denominator <= 0
        || (denominator as u32) > MAX_TICK_RATE_DENOMINATOR
        || (numerator as u32) < (denominator as u32)
        || u128::from(numerator as u32)
            > u128::from(MAX_SIMULATION_HZ).saturating_mul(u128::from(denominator as u32))
    {
        return Err(FrontplaneError::InvalidSimulationRate {
            numerator,
            denominator,
            maximum: MAX_SIMULATION_HZ,
        });
    }
    Ok(TickRate::new(numerator as u32, denominator as u32))
}

/// Builds and smoke-renders a replacement instance before exposing it to the
/// caller, preserving exact compatible snapshots while leaving `current`
/// untouched on every candidate failure.
pub fn prepare_reload(
    current: &mut Frontplane,
    source: &str,
    limits: Limits,
    init: PluginInit,
) -> Result<PreparedReload, FrontplaneError> {
    prepare_reload_with_assets(current, source, limits, init, ApplicationAssets::new())
}

/// Prepares a transactional replacement against the newly resolved immutable
/// asset catalog, so archive/directory reload never retains stale media bytes.
pub fn prepare_reload_with_assets(
    current: &mut Frontplane,
    source: &str,
    limits: Limits,
    init: PluginInit,
    application_assets: ApplicationAssets,
) -> Result<PreparedReload, FrontplaneError> {
    let snapshot = current.snapshot()?;
    let mut candidate = Frontplane::from_wat_with_assets(source, limits, application_assets)?;
    candidate.configure()?;
    candidate.init(init.seed, init.viewport_width, init.viewport_height)?;
    let state_transfer = match candidate.restore(&snapshot) {
        Ok(()) => StateTransfer::Preserved,
        Err(FrontplaneError::SnapshotMismatch) => StateTransfer::Restarted,
        Err(error) => return Err(error),
    };
    candidate.observe_display_refresh(current.display_refresh.unwrap_or(init.display_refresh))?;
    let frame = candidate.render()?;
    candidate.drain_audio();
    candidate.drain_sample_audio();
    candidate.drain_effects();
    Ok(PreparedReload {
        frontplane: candidate,
        frame,
        state_transfer,
    })
}

fn required_func<P, R>(
    instance: &Instance,
    store: &mut Store<HostState>,
    name: &'static str,
) -> Result<TypedFunc<P, R>, FrontplaneError>
where
    P: runtime::WasmParams,
    R: runtime::WasmResults,
{
    instance
        .get_typed_func::<P, R>(store, name)
        .map_err(|_| FrontplaneError::MissingExport { name })
}

fn optional_func<P, R>(
    instance: &Instance,
    store: &mut Store<HostState>,
    name: &'static str,
) -> Result<Option<TypedFunc<P, R>>, FrontplaneError>
where
    P: runtime::WasmParams,
    R: runtime::WasmResults,
{
    if instance.get_export(&mut *store, name).is_none() {
        return Ok(None);
    }
    instance
        .get_typed_func::<P, R>(store, name)
        .map(Some)
        .map_err(|_| FrontplaneError::MissingExport { name })
}

fn runtime_error(error: impl fmt::Display) -> FrontplaneError {
    FrontplaneError::Runtime(error.to_string())
}

fn validate_bounded_numbers(
    operation: &'static str,
    values: &[f32],
    max_abs: f32,
) -> Result<(), FrontplaneError> {
    if bounded_finite(values, max_abs) {
        Ok(())
    } else {
        Err(FrontplaneError::InvalidNumber { operation })
    }
}

fn legacy_event_parameters(event: Event) -> (i32, i32, f32, f32) {
    match event {
        Event::KeyDown(key) => (1, key as i32, 0.0, 0.0),
        Event::KeyUp(key) => (2, key as i32, 0.0, 0.0),
        Event::PointerMove { x, y } => (3, 0, x, y),
        Event::PointerDown { button, x, y } => (4, button as i32, x, y),
        Event::PointerUp { button, x, y } => (5, button as i32, x, y),
        Event::Viewport { width, height } => (6, 0, width, height),
        Event::MenuAction(id) => (7, id as i32, 0.0, 0.0),
        Event::Focus(focused) => (8, i32::from(focused), 0.0, 0.0),
        Event::Control { .. } => unreachable!("control events use AE_control_event"),
        Event::DisplayRefresh(rate) => (9, rate.numerator as i32, rate.denominator as f32, 0.0),
        Event::PointerScroll {
            unit,
            delta_x,
            delta_y,
        } => (10, unit as i32, delta_x, delta_y),
        Event::Pause(phase) => (15, phase as i32, 0.0, 0.0),
    }
}

/// Converts only logical-pixel event fields to Q16.16; integer event metadata
/// such as key IDs and display-rate denominators remains unscaled.
fn integer_event_parameters(
    event: Event,
    max_coordinate_abs: f32,
) -> Result<(i32, i32, i32, i32), FrontplaneError> {
    let coordinates = |a, b| {
        Ok((
            logical_to_q16(a, max_coordinate_abs, "AE_event_i32")?,
            logical_to_q16(b, max_coordinate_abs, "AE_event_i32")?,
        ))
    };
    let parameters = match event {
        Event::KeyDown(key) => (1, key as i32, 0, 0),
        Event::KeyUp(key) => (2, key as i32, 0, 0),
        Event::PointerMove { x, y } => {
            let (x, y) = coordinates(x, y)?;
            (3, 0, x, y)
        }
        Event::PointerDown { button, x, y } => {
            let (x, y) = coordinates(x, y)?;
            (4, button as i32, x, y)
        }
        Event::PointerUp { button, x, y } => {
            let (x, y) = coordinates(x, y)?;
            (5, button as i32, x, y)
        }
        Event::Viewport { width, height } => {
            let (width, height) = coordinates(width, height)?;
            (6, 0, width, height)
        }
        Event::MenuAction(id) => (7, id as i32, 0, 0),
        Event::Focus(focused) => (8, i32::from(focused), 0, 0),
        Event::Control { .. } => unreachable!("control events use AE_control_event"),
        Event::DisplayRefresh(rate) => (9, rate.numerator as i32, rate.denominator as i32, 0),
        Event::PointerScroll {
            unit,
            delta_x,
            delta_y,
        } => {
            let (delta_x, delta_y) = coordinates(delta_x, delta_y)?;
            (10, unit as i32, delta_x, delta_y)
        }
        Event::Pause(phase) => (15, phase as i32, 0, 0),
    };
    Ok(parameters)
}

/// Quantizes adapter-side logical pixels to signed Q16.16 while rejecting
/// non-finite, policy-exceeding, or representation-overflowing values.
fn logical_to_q16(
    value: f32,
    max_coordinate_abs: f32,
    operation: &'static str,
) -> Result<i32, FrontplaneError> {
    if !value.is_finite() || value.abs() > max_coordinate_abs {
        return Err(FrontplaneError::InvalidNumber { operation });
    }
    let scaled = (f64::from(value) * Q16_SCALE).round();
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&scaled) {
        return Err(FrontplaneError::InvalidNumber { operation });
    }
    Ok(scaled as i32)
}

/// Defines the complete capability allowlist and validates each guest value at
/// the import boundary before adding it to trusted host-owned buffers.
fn bind_host_functions(linker: &mut Linker<HostState>) -> Result<(), FrontplaneError> {
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_title",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| match read_string(
                &mut caller,
                ptr,
                len,
                "title",
            ) {
                Ok(title) => {
                    caller.data_mut().metadata.title = title;
                    0
                }
                Err(error) => caller.data_mut().reject(error, -3),
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_pause_trigger",
            |mut caller: Caller<'_, HostState>, kind: i32, code: i32, flags: i32| {
                if caller.data().active_operation != Some("AE_configure") {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("AE_pause_trigger is configure-only"),
                        -8,
                    );
                }
                if flags != 0 || kind != PauseTriggerKind::Key as i32 {
                    return caller
                        .data_mut()
                        .reject(PendingError::Unsupported("AE_pause_trigger"), -4);
                }
                let Some(key) = Key::from_abi(code as u32) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_pause_trigger"), -5);
                };
                if caller.data().metadata.pause_triggers.len() >= MAX_PAUSE_TRIGGERS {
                    return caller.data_mut().reject(
                        PendingError::Budget("AE_pause_trigger", "pause trigger"),
                        -2,
                    );
                }
                let trigger = PauseTrigger::key(key);
                if caller.data().metadata.pause_triggers.contains(&trigger) {
                    return caller.data_mut().reject(
                        PendingError::DuplicateId("AE_pause_trigger", code as u32),
                        -7,
                    );
                }
                caller.data_mut().metadata.pause_triggers.push(trigger);
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_frame_begin_rgba",
            |mut caller: Caller<'_, HostState>, rgba: i32| {
                begin_frame(caller.data_mut(), rgba as u32)
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_ui_begin",
            |mut caller: Caller<'_, HostState>, revision: i32| {
                begin_ui(caller.data_mut(), revision as u32)
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_ui_end",
            |mut caller: Caller<'_, HostState>| end_ui(caller.data_mut()),
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_control_panel_q16",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             x: i32,
             y: i32,
             width: i32,
             height: i32,
             rgba: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some(bounds) = q16_rect(x, y, width, height, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_control_panel_q16"), -5);
                };
                if flags != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_control_panel_q16"), -5);
                }
                push_control_panel(
                    caller.data_mut(),
                    ControlPanel {
                        id: id as u32,
                        bounds,
                        rgba: rgba as u32,
                    },
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_slider_place_q16",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             panel_id: i32,
             value: i32,
             x: i32,
             y: i32,
             width: i32,
             height: i32,
             label_placement: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some(bounds) = q16_rect(x, y, width, height, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_slider_place_q16"), -5);
                };
                let Some(label_placement) = ControlLabelPlacement::from_abi(label_placement) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_slider_place_q16"), -5);
                };
                if flags != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_slider_place_q16"), -5);
                }
                let id = id as u32;
                let Some(control) = caller
                    .data()
                    .metadata
                    .controls
                    .iter()
                    .find(|control| control.id == id)
                else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("slider placement uses undeclared control"),
                        -8,
                    );
                };
                if !control.contains_value(value) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_slider_place_q16"), -5);
                }
                push_slider_placement(
                    caller.data_mut(),
                    SliderPlacement {
                        id,
                        panel_id: panel_id as u32,
                        value,
                        bounds,
                        label_placement,
                    },
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_button_place_q16",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             panel_id: i32,
             action_id: i32,
             x: i32,
             y: i32,
             width: i32,
             height: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some(bounds) = q16_rect(x, y, width, height, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_button_place_q16"), -5);
                };
                if flags & !1 != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_button_place_q16"), -5);
                }
                let action_id = action_id as u32;
                if !caller.data().menu_ids.contains(&action_id) {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("button placement uses undeclared action"),
                        -8,
                    );
                }
                push_button_placement(
                    caller.data_mut(),
                    ButtonPlacement {
                        id: id as u32,
                        panel_id: panel_id as u32,
                        action_id,
                        bounds,
                        selected: flags & 1 != 0,
                    },
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_menu_item",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             ptr: i32,
             len: i32,
             shortcut: i32,
             flags: i32| {
                let separator = flags & 1 != 0;
                if flags & !1 != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("unsupported menu flags"), -8);
                }
                if caller.data().metadata.menu_items.len() >= caller.data().limits.max_menu_items {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("menu_item", "menu"), -2);
                }
                if separator {
                    caller
                        .data_mut()
                        .metadata
                        .menu_items
                        .push(MenuItem::separator());
                    return 0;
                }
                let id = id as u32;
                if !caller.data_mut().menu_ids.insert(id) {
                    return caller
                        .data_mut()
                        .reject(PendingError::DuplicateId("menu_item", id), -7);
                }
                let label = match read_string(&mut caller, ptr, len, "menu_item") {
                    Ok(label) => label,
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                let shortcut = match shortcut {
                    0 => None,
                    1 => Some("Ctrl+N"),
                    6 => Some("Ctrl+Q"),
                    7 => Some("F1"),
                    _ => None,
                };
                caller
                    .data_mut()
                    .metadata
                    .menu_items
                    .push(MenuItem::action(id, label, shortcut));
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_slider_i32",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             label_ptr: i32,
             label_len: i32,
             min: i32,
             max: i32,
             step: i32,
             initial: i32| {
                if caller.data().metadata.controls.len() >= caller.data().limits.max_controls {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("AE_slider_i32", "control"), -2);
                }
                let range = i64::from(max) - i64::from(min);
                let initial_offset = i64::from(initial) - i64::from(min);
                if range <= 0
                    || step <= 0
                    || range % i64::from(step) != 0
                    || initial < min
                    || initial > max
                    || initial_offset % i64::from(step) != 0
                    || range / i64::from(step) > i64::from(caller.data().limits.max_control_steps)
                {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("invalid integer slider lattice"),
                        -8,
                    );
                }
                let id = id as u32;
                if !caller.data_mut().control_ids.insert(id) {
                    return caller
                        .data_mut()
                        .reject(PendingError::DuplicateId("AE_slider_i32", id), -7);
                }
                let label = match read_string(&mut caller, label_ptr, label_len, "AE_slider_i32") {
                    Ok(label) if !label.is_empty() => label,
                    Ok(_) => {
                        return caller
                            .data_mut()
                            .reject(PendingError::InvalidFrame("empty integer slider label"), -8);
                    }
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                caller.data_mut().metadata.controls.push(SliderControl {
                    id,
                    label,
                    min,
                    max,
                    step,
                    initial,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(WAT_IMPORT_MODULE, "AE_sin_cos_turn", |angle: i32| {
            sin_cos_turn_q30(angle)
        })
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_synth_voice",
            |mut caller: Caller<'_, HostState>,
             program_id: i32,
             waveform: i32,
             delay_ms: i32,
             duration_ms: i32,
             frequency_start_millihz: i32,
             frequency_mid_millihz: i32,
             frequency_end_millihz: i32,
             gain_start_ppm: i32,
             gain_peak_ppm: i32,
             gain_end_ppm: i32,
             filter: i32,
             filter_start_millihz: i32,
             filter_end_millihz: i32,
             cooldown_ms: i32| {
                let Some(waveform) = SynthWaveform::from_abi(waveform) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("unsupported synth waveform"), -8);
                };
                let Some(filter) = SynthFilter::from_abi(filter) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("unsupported synth filter"), -8);
                };
                let scalars = [
                    delay_ms,
                    duration_ms,
                    frequency_start_millihz,
                    frequency_mid_millihz,
                    frequency_end_millihz,
                    gain_start_ppm,
                    gain_peak_ppm,
                    gain_end_ppm,
                    filter_start_millihz,
                    filter_end_millihz,
                    cooldown_ms,
                ];
                if scalars.iter().any(|value| *value < 0)
                    || duration_ms == 0
                    || delay_ms > 5_000
                    || duration_ms > 5_000
                    || delay_ms.saturating_add(duration_ms) > 6_000
                    || [
                        frequency_start_millihz,
                        frequency_mid_millihz,
                        frequency_end_millihz,
                        filter_start_millihz,
                        filter_end_millihz,
                    ]
                    .iter()
                    .any(|value| *value > 24_000_000)
                    || [gain_start_ppm, gain_peak_ppm, gain_end_ppm]
                        .iter()
                        .any(|value| *value > 1_000_000)
                    || cooldown_ms > 5_000
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("invalid synth voice bounds"), -8);
                }
                if caller.data().metadata.synth_voices.len()
                    >= caller.data().limits.max_synth_voices
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("synth_voice", "synth voice"), -2);
                }
                caller.data_mut().metadata.synth_voices.push(SynthVoice {
                    program_id: program_id as u32,
                    waveform,
                    delay_ms: delay_ms as u32,
                    duration_ms: duration_ms as u32,
                    frequency_start_millihz: frequency_start_millihz as u32,
                    frequency_mid_millihz: frequency_mid_millihz as u32,
                    frequency_end_millihz: frequency_end_millihz as u32,
                    gain_start_ppm: gain_start_ppm as u32,
                    gain_peak_ppm: gain_peak_ppm as u32,
                    gain_end_ppm: gain_end_ppm as u32,
                    filter,
                    filter_start_millihz: filter_start_millihz as u32,
                    filter_end_millihz: filter_end_millihz as u32,
                    cooldown_ms: cooldown_ms as u32,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_sample_asset",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             path_ptr: i32,
             path_len: i32,
             flags: i32| {
                if caller.data().active_operation != Some("AE_configure") {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("AE_sample_asset is configure-only"),
                        -8,
                    );
                }
                if flags != 0 {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("unsupported AE_sample_asset flags"),
                        -8,
                    );
                }
                if caller.data().metadata.sample_assets.len()
                    >= caller.data().limits.max_sample_assets
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("AE_sample_asset", "sample asset"), -2);
                }
                let id = id as u32;
                if caller.data().sample_ids.contains(&id) {
                    return caller
                        .data_mut()
                        .reject(PendingError::DuplicateId("AE_sample_asset", id), -7);
                }
                let path = match read_string(&mut caller, path_ptr, path_len, "AE_sample_asset") {
                    Ok(path) => path,
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                if !crate::package::is_valid_asset_name(&path) || !path.ends_with(".flac") {
                    return caller
                        .data_mut()
                        .reject(PendingError::Unsupported("AE_sample_asset"), -4);
                }
                let Some(bytes) = caller.data().application_assets.get(&path).cloned() else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("sample asset is unavailable"),
                        -8,
                    );
                };
                if bytes.len() > caller.data().limits.max_sample_encoded_bytes {
                    return caller.data_mut().reject(
                        PendingError::Budget("AE_sample_asset", "encoded sample byte"),
                        -2,
                    );
                }
                let current_decoded_bytes = caller
                    .data()
                    .metadata
                    .sample_assets
                    .iter()
                    .map(|sample| sample.clip.samples.len().saturating_mul(4))
                    .fold(0_usize, usize::saturating_add);
                let remaining_decoded_bytes = caller
                    .data()
                    .limits
                    .max_total_sample_decoded_bytes
                    .saturating_sub(current_decoded_bytes);
                let decode_limits = wav::WavLimits {
                    max_decoded_bytes: remaining_decoded_bytes.min(32 * 1024 * 1024),
                    ..wav::WavLimits::default()
                };
                let clip = match decode_flac(&bytes, &decode_limits) {
                    Ok(clip) => clip,
                    Err(_) => {
                        return caller.data_mut().reject(
                            PendingError::InvalidFrame(
                                "sample asset is not a supported bounded FLAC",
                            ),
                            -8,
                        );
                    }
                };
                caller.data_mut().sample_ids.insert(id);
                caller
                    .data_mut()
                    .metadata
                    .sample_assets
                    .push(SampleAsset { id, path, clip });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_image_define",
            |mut caller: Caller<'_, HostState>, id: i32, ptr: i32, len: i32, flags: i32| {
                if flags & !1 != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("unsupported image flags"), -8);
                }
                if len < 0 || len as usize > caller.data().limits.max_image_bytes {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("image_define", "image byte"), -2);
                }
                let current_bytes: usize = caller
                    .data()
                    .images
                    .iter()
                    .map(|image| image.bytes.len())
                    .sum();
                if current_bytes.saturating_add(len as usize)
                    > caller.data().limits.max_total_image_bytes
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("image_define", "total image byte"), -2);
                }
                let id = id as u32;
                if !caller.data_mut().image_ids.insert(id) {
                    return caller
                        .data_mut()
                        .reject(PendingError::DuplicateId("image_define", id), -7);
                }
                let max_bytes = caller.data().limits.max_image_bytes;
                let bytes = match read_bytes(&mut caller, ptr, len, max_bytes, "image_define") {
                    Ok(bytes) => bytes,
                    Err(error) => return caller.data_mut().reject(error, -3),
                };
                caller.data_mut().images.push(ImageResource {
                    id,
                    format: if flags & 1 == 0 {
                        ImageFormat::Encoded
                    } else {
                        ImageFormat::RawRgba
                    },
                    bytes,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_image_release",
            |mut caller: Caller<'_, HostState>, id: i32| {
                let id = id as u32;
                caller.data_mut().image_ids.remove(&id);
                caller.data_mut().images.retain(|image| image.id != id);
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_frame_begin",
            |mut caller: Caller<'_, HostState>, r: f32, g: f32, b: f32, a: f32| {
                if !normalized(&[r, g, b, a]) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("frame_begin"), -5);
                }
                begin_frame(caller.data_mut(), components_to_rgba(r, g, b, a))
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_transform_push",
            |mut caller: Caller<'_, HostState>,
             m11: f32,
             m12: f32,
             m21: f32,
             m22: f32,
             tx: f32,
             ty: f32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[m11, m12, m21, m22, tx, ty], max_abs) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("transform_push"), -5);
                }
                let command = DrawCommand::PushTransform(Affine {
                    m11,
                    m12,
                    m21,
                    m22,
                    tx,
                    ty,
                });
                let status = push_unkeyed(caller.data_mut(), command, "transform_push");
                if status == 0 {
                    caller.data_mut().transform_depth += 1;
                }
                status
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_transform_pop",
            |mut caller: Caller<'_, HostState>| {
                if caller.data().transform_depth == 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("transform stack underflow"), -6);
                }
                let status = push_unkeyed(
                    caller.data_mut(),
                    DrawCommand::PopTransform,
                    "transform_pop",
                );
                if status == 0 {
                    caller.data_mut().transform_depth -= 1;
                }
                status
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_begin",
            |mut caller: Caller<'_, HostState>, id: i32| {
                if caller.data().frame.is_none() || caller.data().current_path.is_some() {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("nested or out-of-frame path_begin"),
                        -6,
                    );
                }
                caller.data_mut().current_path = Some(PathBuilder {
                    id: id as u32,
                    segments: Vec::new(),
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_move",
            |mut caller: Caller<'_, HostState>, x: f32, y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Move(Point { x, y }),
                    &[x, y],
                    "path_move",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_move_q16",
            |mut caller: Caller<'_, HostState>, x: i32, y: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some((x, y)) = q16_point(x, y, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_path_move_q16"), -5);
                };
                push_path_segment(
                    &mut caller,
                    PathSegment::Move(Point { x, y }),
                    &[x, y],
                    "AE_path_move_q16",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_line",
            |mut caller: Caller<'_, HostState>, x: f32, y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Line(Point { x, y }),
                    &[x, y],
                    "path_line",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_line_q16",
            |mut caller: Caller<'_, HostState>, x: i32, y: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some((x, y)) = q16_point(x, y, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_path_line_q16"), -5);
                };
                push_path_segment(
                    &mut caller,
                    PathSegment::Line(Point { x, y }),
                    &[x, y],
                    "AE_path_line_q16",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_quad",
            |mut caller: Caller<'_, HostState>, cx: f32, cy: f32, x: f32, y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Quadratic {
                        control: Point { x: cx, y: cy },
                        end: Point { x, y },
                    },
                    &[cx, cy, x, y],
                    "path_quad",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_cubic",
            |mut caller: Caller<'_, HostState>,
             c1x: f32,
             c1y: f32,
             c2x: f32,
             c2y: f32,
             x: f32,
             y: f32| {
                push_path_segment(
                    &mut caller,
                    PathSegment::Cubic {
                        control_1: Point { x: c1x, y: c1y },
                        control_2: Point { x: c2x, y: c2y },
                        end: Point { x, y },
                    },
                    &[c1x, c1y, c2x, c2y, x, y],
                    "path_cubic",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_close",
            |mut caller: Caller<'_, HostState>| {
                push_path_segment(&mut caller, PathSegment::Close, &[], "path_close")
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_end",
            |mut caller: Caller<'_, HostState>, width: f32, fill: i32, stroke: i32, flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[width], max_abs) || width < 0.0 || flags != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("path_end"), -5);
                }
                let Some(path) = caller.data_mut().current_path.take() else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("path_end without path_begin"),
                        -6,
                    );
                };
                push_command(
                    caller.data_mut(),
                    path.id,
                    DrawCommand::Path {
                        id: path.id,
                        segments: path.segments,
                        width,
                        fill_rgba: Some(fill as u32),
                        stroke_rgba: Some(stroke as u32),
                    },
                    "path_end",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_path_end_q16",
            |mut caller: Caller<'_, HostState>, width: i32, fill: i32, stroke: i32, flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some(width) = q16_logical(width, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_path_end_q16"), -5);
                };
                if width < 0.0 || flags != 0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_path_end_q16"), -5);
                }
                let Some(path) = caller.data_mut().current_path.take() else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("path_end without path_begin"),
                        -6,
                    );
                };
                push_command(
                    caller.data_mut(),
                    path.id,
                    DrawCommand::Path {
                        id: path.id,
                        segments: path.segments,
                        width,
                        fill_rgba: (fill != 0).then_some(fill as u32),
                        stroke_rgba: (stroke != 0).then_some(stroke as u32),
                    },
                    "AE_path_end_q16",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_sprite",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             image_id: i32,
             src_x: f32,
             src_y: f32,
             src_w: f32,
             src_h: f32,
             dst_x: f32,
             dst_y: f32,
             dst_w: f32,
             dst_h: f32,
             pivot_x: f32,
             pivot_y: f32,
             tint: i32,
             flags: i32| {
                let values = [
                    src_x, src_y, src_w, src_h, dst_x, dst_y, dst_w, dst_h, pivot_x, pivot_y,
                ];
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&values, max_abs)
                    || [src_w, src_h, dst_w, dst_h]
                        .iter()
                        .any(|value| *value < 0.0)
                    || !(0.0..=1.0).contains(&pivot_x)
                    || !(0.0..=1.0).contains(&pivot_y)
                    || flags != 0
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("sprite"), -5);
                }
                if !caller.data().image_ids.contains(&(image_id as u32)) {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidFrame("sprite uses unknown image"), -8);
                }
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Sprite {
                        id: id as u32,
                        image_id: image_id as u32,
                        source: Rect {
                            x: src_x,
                            y: src_y,
                            width: src_w,
                            height: src_h,
                        },
                        destination: Rect {
                            x: dst_x,
                            y: dst_y,
                            width: dst_w,
                            height: dst_h,
                        },
                        pivot: Point {
                            x: pivot_x,
                            y: pivot_y,
                        },
                        tint_rgba: tint as u32,
                        flags: flags as u32,
                    },
                    "sprite",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_line",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             x1: f32,
             y1: f32,
             x2: f32,
             y2: f32,
             width: f32,
             rgba: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[x1, y1, x2, y2, width], max_abs) || width < 0.0 {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("line"), -5);
                }
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Line {
                        id: id as u32,
                        x1,
                        y1,
                        x2,
                        y2,
                        width,
                        rgba: rgba as u32,
                    },
                    "line",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_circle",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             x: f32,
             y: f32,
             radius: f32,
             width: f32,
             rgba: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                if !bounded_finite(&[x, y, radius, width], max_abs)
                    || radius < 0.0
                    || width < 0.0
                    || flags & !1 != 0
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("circle"), -5);
                }
                push_command(
                    caller.data_mut(),
                    id as u32,
                    DrawCommand::Circle {
                        id: id as u32,
                        x,
                        y,
                        radius,
                        width,
                        rgba: rgba as u32,
                        filled: flags & 1 != 0,
                    },
                    "circle",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_text",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             ptr: i32,
             len: i32,
             x: f32,
             y: f32,
             size: f32,
             rgba: i32,
             flags: i32| {
                push_text_command(
                    &mut caller,
                    id,
                    ptr,
                    len,
                    x,
                    y,
                    size,
                    rgba,
                    TextFont::PlatformDefault,
                    flags,
                    "text",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_text_font",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             ptr: i32,
             len: i32,
             x: f32,
             y: f32,
             size: f32,
             rgba: i32,
             font: i32,
             flags: i32| {
                let Ok(font) = TextFont::try_from(font) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("text font"), -5);
                };
                push_text_command(
                    &mut caller,
                    id,
                    ptr,
                    len,
                    x,
                    y,
                    size,
                    rgba,
                    font,
                    flags,
                    "text font",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_text_font_q16",
            |mut caller: Caller<'_, HostState>,
             id: i32,
             ptr: i32,
             len: i32,
             x: i32,
             y: i32,
             size: i32,
             rgba: i32,
             font: i32,
             flags: i32| {
                let max_abs = caller.data().limits.max_coordinate_abs;
                let Some((x, y)) = q16_point(x, y, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("text font"), -5);
                };
                let Some(size) = q16_logical(size, max_abs) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("text font"), -5);
                };
                let Ok(font) = TextFont::try_from(font) else {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("text font"), -5);
                };
                push_text_command(
                    &mut caller,
                    id,
                    ptr,
                    len,
                    x,
                    y,
                    size,
                    rgba,
                    font,
                    flags,
                    "text font",
                )
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_frame_end",
            |mut caller: Caller<'_, HostState>| {
                if caller.data().transform_depth != 0 || caller.data().current_path.is_some() {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("unbalanced composition stack"),
                        -6,
                    );
                }
                let Some(frame) = caller.data_mut().frame.take() else {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("frame_end without frame_begin"),
                        -6,
                    );
                };
                caller.data_mut().completed_frame = Some(FrameOutput {
                    background: frame.background,
                    commands: frame.commands,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_audio",
            |mut caller: Caller<'_, HostState>, id: i32, volume: f32, pitch: f32, flags: i32| {
                if !finite(&[volume, pitch])
                    || !(0.0..=1.0).contains(&volume)
                    || !(0.25..=4.0).contains(&pitch)
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("audio"), -5);
                }
                if caller.data().audio_event_budget_exhausted() {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("audio", "audio event"), -2);
                }
                caller.data_mut().audio.push(AudioEvent {
                    id: id as u32,
                    volume,
                    pitch,
                    flags: flags as u32,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_sample_play",
            |mut caller: Caller<'_, HostState>, id: i32, volume: f32, pitch: f32, flags: i32| {
                if caller.data().active_operation == Some("AE_configure") || flags != 0 {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("invalid AE_sample_play phase or flags"),
                        -8,
                    );
                }
                if !finite(&[volume, pitch])
                    || !(0.0..=1.0).contains(&volume)
                    || !(0.25..=4.0).contains(&pitch)
                {
                    return caller
                        .data_mut()
                        .reject(PendingError::InvalidNumber("AE_sample_play"), -5);
                }
                let id = id as u32;
                if !caller.data().sample_ids.contains(&id) {
                    return caller.data_mut().reject(
                        PendingError::InvalidFrame("AE_sample_play uses undeclared sample"),
                        -8,
                    );
                }
                if caller.data().audio_event_budget_exhausted() {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("AE_sample_play", "audio event"), -2);
                }
                caller.data_mut().sample_audio.push(SampleAudioEvent {
                    id,
                    volume,
                    pitch,
                    flags: flags as u32,
                });
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_effect",
            |mut caller: Caller<'_, HostState>, kind: i32, a: i32, _b: i32| {
                if caller.data().effects.len() >= caller.data().limits.max_effects {
                    return caller
                        .data_mut()
                        .reject(PendingError::Budget("effect", "effect"), -2);
                }
                let effect = match kind {
                    1 => HostEffect::Redraw,
                    2 => HostEffect::Quit,
                    3 => HostEffect::SetCursor(a),
                    4 => HostEffect::PersistSnapshot,
                    // open-url (5) requires a manifest capability that v0 does not yet expose.
                    _ => {
                        return caller
                            .data_mut()
                            .reject(PendingError::Unsupported("effect"), -8);
                    }
                };
                caller.data_mut().effects.push(effect);
                0
            },
        )
        .map_err(runtime_error)?;
    linker
        .func_wrap(
            WAT_IMPORT_MODULE,
            "AE_log",
            |mut caller: Caller<'_, HostState>, _level: i32, ptr: i32, len: i32| match read_string(
                &mut caller,
                ptr,
                len,
                "log",
            ) {
                Ok(_) => 0,
                Err(error) => caller.data_mut().reject(error, -3),
            },
        )
        .map_err(runtime_error)?;
    Ok(())
}

fn read_string(
    caller: &mut Caller<'_, HostState>,
    pointer: i32,
    length: i32,
    operation: &'static str,
) -> Result<String, PendingError> {
    let max_bytes = caller.data().limits.max_string_bytes;
    let bytes = read_bytes(caller, pointer, length, max_bytes, operation)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| PendingError::InvalidUtf8(operation))?;
    Ok(text.to_owned())
}

/// Converts one validated UTF-8 text import into the immutable scene command
/// shared by floating-point and exact Q16.16 guest profiles.
#[allow(clippy::too_many_arguments)]
fn push_text_command(
    caller: &mut Caller<'_, HostState>,
    id: i32,
    pointer: i32,
    length: i32,
    x: f32,
    y: f32,
    size: f32,
    rgba: i32,
    font: TextFont,
    flags: i32,
    operation: &'static str,
) -> i32 {
    let max_abs = caller.data().limits.max_coordinate_abs;
    if !bounded_finite(&[x, y, size], max_abs) || size < 0.0 || flags & !1 != 0 {
        return caller
            .data_mut()
            .reject(PendingError::InvalidNumber(operation), -5);
    }
    let text = match read_string(caller, pointer, length, operation) {
        Ok(text) => text,
        Err(error) => return caller.data_mut().reject(error, -3),
    };
    push_command(
        caller.data_mut(),
        id as u32,
        DrawCommand::Text {
            id: id as u32,
            text,
            x,
            y,
            size,
            rgba: rgba as u32,
            centered: flags & 1 != 0,
            font,
        },
        operation,
    )
}

/// Copies a bounded slice out of guest linear memory without exposing borrowed
/// Wasmtime memory beyond the import invocation.
fn read_bytes(
    caller: &mut Caller<'_, HostState>,
    pointer: i32,
    length: i32,
    max_bytes: usize,
    operation: &'static str,
) -> Result<Vec<u8>, PendingError> {
    if pointer < 0 || length < 0 || length as usize > max_bytes {
        return Err(PendingError::InvalidPointer(operation));
    }
    let memory = caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
        .ok_or(PendingError::InvalidPointer(operation))?;
    let pointer = pointer as usize;
    let length = length as usize;
    let bytes = memory
        .data(&*caller)
        .get(pointer..pointer.saturating_add(length))
        .ok_or(PendingError::InvalidPointer(operation))?;
    Ok(bytes.to_vec())
}

/// Adds a keyed scene primitive only inside a live frame, enforcing both the
/// global command budget and stable-ID uniqueness.
fn push_command(
    state: &mut HostState,
    id: u32,
    command: DrawCommand,
    operation: &'static str,
) -> i32 {
    let max_commands = state.limits.max_commands;
    let Some(frame) = state.frame.as_mut() else {
        return state.reject(PendingError::InvalidFrame("draw command outside frame"), -6);
    };
    if frame.commands.len() >= max_commands {
        return state.reject(PendingError::Budget(operation, "draw command"), -2);
    }
    if !frame.ids.insert(id) {
        return state.reject(PendingError::DuplicateId(operation, id), -7);
    }
    frame.commands.push(command);
    0
}

/// Adds a guest-positioned native surface to the live frame while keeping its
/// stable identity and capacity independent from canvas draw-command IDs.
fn push_control_panel(state: &mut HostState, panel: ControlPanel) -> i32 {
    let max_controls = state.limits.max_controls;
    let Some(ui) = state.ui.as_mut() else {
        return state.reject(
            PendingError::InvalidFrame("control panel outside UI snapshot"),
            -6,
        );
    };
    if ui.control_panels.len() >= max_controls {
        return state.reject(
            PendingError::Budget("AE_control_panel_q16", "control panel"),
            -2,
        );
    }
    if !ui.control_panel_ids.insert(panel.id) {
        return state.reject(
            PendingError::DuplicateId("AE_control_panel_q16", panel.id),
            -7,
        );
    }
    ui.control_panels.push(panel);
    0
}

/// Adds one declared slider to the current UI frame, validating panel
/// ownership and per-frame identity before any native adapter sees it.
fn push_slider_placement(state: &mut HostState, slider: SliderPlacement) -> i32 {
    let max_controls = state.limits.max_controls;
    let Some(ui) = state.ui.as_mut() else {
        return state.reject(
            PendingError::InvalidFrame("slider placement outside UI snapshot"),
            -6,
        );
    };
    if !ui.control_panel_ids.contains(&slider.panel_id) {
        return state.reject(
            PendingError::InvalidFrame("slider placement uses unknown control panel"),
            -8,
        );
    }
    if ui.sliders.len() >= max_controls {
        return state.reject(
            PendingError::Budget("AE_slider_place_q16", "slider placement"),
            -2,
        );
    }
    if !ui.slider_ids.insert(slider.id) {
        return state.reject(
            PendingError::DuplicateId("AE_slider_place_q16", slider.id),
            -7,
        );
    }
    ui.sliders.push(slider);
    0
}

/// Adds one declared action button to the current UI document while keeping
/// panel ownership and stable widget identity guest-authored and validated.
fn push_button_placement(state: &mut HostState, button: ButtonPlacement) -> i32 {
    let max_controls = state.limits.max_controls;
    let Some(ui) = state.ui.as_mut() else {
        return state.reject(
            PendingError::InvalidFrame("button placement outside UI snapshot"),
            -6,
        );
    };
    if !ui.control_panel_ids.contains(&button.panel_id) {
        return state.reject(
            PendingError::InvalidFrame("button placement uses unknown control panel"),
            -8,
        );
    }
    if ui.buttons.len() >= max_controls {
        return state.reject(
            PendingError::Budget("AE_button_place_q16", "button placement"),
            -2,
        );
    }
    if !ui.button_ids.insert(button.id) {
        return state.reject(
            PendingError::DuplicateId("AE_button_place_q16", button.id),
            -7,
        );
    }
    ui.buttons.push(button);
    0
}

fn push_unkeyed(state: &mut HostState, command: DrawCommand, operation: &'static str) -> i32 {
    let max_commands = state.limits.max_commands;
    let Some(frame) = state.frame.as_mut() else {
        return state.reject(
            PendingError::InvalidFrame("composition command outside frame"),
            -6,
        );
    };
    if frame.commands.len() >= max_commands {
        return state.reject(PendingError::Budget(operation, "draw command"), -2);
    }
    frame.commands.push(command);
    0
}

/// Extends the one active vector path under finite-number and segment-count
/// controls, preventing malformed paths from reaching GPUI.
fn push_path_segment(
    caller: &mut Caller<'_, HostState>,
    segment: PathSegment,
    values: &[f32],
    operation: &'static str,
) -> i32 {
    let max_abs = caller.data().limits.max_coordinate_abs;
    if !bounded_finite(values, max_abs) {
        return caller
            .data_mut()
            .reject(PendingError::InvalidNumber(operation), -5);
    }
    let limit = caller.data().limits.max_path_segments;
    let Some(path) = caller.data_mut().current_path.as_mut() else {
        return caller.data_mut().reject(
            PendingError::InvalidFrame("path segment without path_begin"),
            -6,
        );
    };
    if path.segments.len() >= limit {
        return caller
            .data_mut()
            .reject(PendingError::Budget(operation, "path segment"), -2);
    }
    path.segments.push(segment);
    0
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn bounded_finite(values: &[f32], max_abs: f32) -> bool {
    max_abs.is_finite()
        && max_abs >= 0.0
        && values
            .iter()
            .all(|value| value.is_finite() && value.abs() <= max_abs)
}

/// Converts the guest's exact signed Q16.16 geometry into the host renderer's
/// logical-pixel scalar only after enforcing the configured coordinate policy.
fn q16_logical(value: i32, max_abs: f32) -> Option<f32> {
    let logical = (f64::from(value) / Q16_SCALE) as f32;
    bounded_finite(&[logical], max_abs).then_some(logical)
}

fn q16_point(x: i32, y: i32, max_abs: f32) -> Option<(f32, f32)> {
    Some((q16_logical(x, max_abs)?, q16_logical(y, max_abs)?))
}

fn q16_rect(x: i32, y: i32, width: i32, height: i32, max_abs: f32) -> Option<Rect> {
    if width <= 0 || height <= 0 {
        return None;
    }
    Some(Rect {
        x: q16_logical(x, max_abs)?,
        y: q16_logical(y, max_abs)?,
        width: q16_logical(width, max_abs)?,
        height: q16_logical(height, max_abs)?,
    })
}

fn normalized(values: &[f32]) -> bool {
    finite(values) && values.iter().all(|value| (0.0..=1.0).contains(value))
}

fn components_to_rgba(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let byte = |value: f32| (value * 255.0).round() as u32;
    (byte(r) << 24) | (byte(g) << 16) | (byte(b) << 8) | byte(a)
}

fn begin_frame(state: &mut HostState, background: u32) -> i32 {
    if state.frame.is_some() {
        return state.reject(PendingError::InvalidFrame("nested frame_begin"), -6);
    }
    state.frame = Some(FrameBuilder {
        background,
        commands: Vec::new(),
        ids: HashSet::new(),
    });
    0
}

/// Starts a keyed declarative UI transaction whose opaque revision changes
/// only when the guest's desired native UI tree changes.
fn begin_ui(state: &mut HostState, revision: u32) -> i32 {
    if state.ui.is_some() {
        return state.reject(PendingError::InvalidFrame("nested ui_begin"), -6);
    }
    if state.pending_ui.is_some() {
        return state.reject(
            PendingError::InvalidFrame("multiple UI snapshots in one render"),
            -6,
        );
    }
    if state
        .accepted_ui
        .as_ref()
        .is_some_and(|accepted| accepted.revision == revision)
    {
        return state.reject(
            PendingError::InvalidFrame("UI revision was already accepted"),
            -6,
        );
    }
    state.ui = Some(UiBuilder {
        revision,
        control_panels: Vec::new(),
        control_panel_ids: HashSet::new(),
        sliders: Vec::new(),
        slider_ids: HashSet::new(),
        buttons: Vec::new(),
        button_ids: HashSet::new(),
    });
    0
}

/// Validates and stages one complete UI document; publication waits until the
/// surrounding `AE_render` transaction also returns a complete canvas frame.
fn end_ui(state: &mut HostState) -> i32 {
    if state.pending_error.is_some() {
        state.ui = None;
        return -6;
    }
    let Some(ui) = state.ui.take() else {
        return state.reject(PendingError::InvalidFrame("ui_end without ui_begin"), -6);
    };
    state.pending_ui = Some(UiSnapshot {
        revision: ui.revision,
        control_panels: ui.control_panels,
        sliders: ui.sliders,
        buttons: ui.buttons,
    });
    0
}
