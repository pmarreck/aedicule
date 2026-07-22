//! Reproducible WAST execution for packaged Aedicule applications.
//!
//! Direct `tests/*.wast` entries are independent suites. Their order uses the
//! same PCG32/Fisher-Yates contract as Peter's LuaJIT `random` tool, while an
//! emitted seed makes every failure exactly replayable.

use crate::{PluginSource, package::application_test_entries, read_application_file};
use wasmtime::{Config, Engine};
use wasmtime_wast::{Async, WastContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationTestReport {
    pub seed: u64,
    pub passed: Vec<String>,
}

/// Executes runnable WAST suites in a reproducibly shuffled order, preserving
/// one WAST context so order-dependent registrations and state can be exposed.
pub fn run_application_tests(
    source: &PluginSource,
    seed: u64,
) -> Result<ApplicationTestReport, String> {
    let mut entries = application_test_entries(source)?;
    if !entries.iter().any(|entry| entry == "tests/main.wast") {
        return Err("application lacks required tests/main.wast".to_owned());
    }
    Pcg32::seeded(seed).shuffle(&mut entries);

    let mut config = Config::new();
    // Aedicule's v0 guest profile is core Wasm; disabling proposal-level GC
    // also avoids requiring a collector solely for the WAST harness.
    config.wasm_gc(false);
    let engine = Engine::new(&config).map_err(|error| format!("create WAST engine: {error}"))?;
    let mut context = WastContext::new(&engine, Async::No, |_| {});
    let mut passed = Vec::with_capacity(entries.len());
    for name in entries {
        let bytes = read_application_file(source, &name)?;
        context
            .run_wast(&name, &bytes)
            .map_err(|error| format!("{name}: {error:#}"))?;
        passed.push(name);
    }
    Ok(ApplicationTestReport { seed, passed })
}

struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    const INCREMENT: u64 = 1_442_695_040_888_963_407;
    const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

    /// Matches `random`'s two-step PCG32 initialization exactly, including
    /// unsigned wrapping behavior at the 64-bit state boundary.
    fn seeded(seed: u64) -> Self {
        let mut generator = Self { state: 0 };
        generator.next();
        generator.state = generator.state.wrapping_add(seed);
        generator.next();
        generator
    }

    fn next(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }

    /// Uses rejection sampling rather than modulo bias, matching `random`'s
    /// inclusive integer range selection for Fisher-Yates indices.
    fn index(&mut self, inclusive_end: usize) -> usize {
        let range = inclusive_end as u64 + 1;
        let bound = (1_u64 << 32) - ((1_u64 << 32) % range);
        loop {
            let value = u64::from(self.next());
            if value < bound {
                return (value % range) as usize;
            }
        }
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let replacement = self.index(index);
            values.swap(index, replacement);
        }
    }
}
