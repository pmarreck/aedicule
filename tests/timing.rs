use std::time::Duration;

use gpui_wasm::{FixedStepClock, Frontplane, Limits};

fn tick_rate_module(rate_export: &str) -> String {
    format!(
        r#"(module
            (memory (export "memory") 1)
            (func (export "fp_abi_major") (result i32) i32.const 0)
            (func (export "fp_abi_minor") (result i32) i32.const 0)
            (func (export "fp_configure") (result i32) i32.const 0)
            (func (export "fp_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "fp_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "fp_tick") (param i32) (result i32) i32.const 0)
            (func (export "fp_render") (result i32) i32.const 0)
            (func (export "fp_state_ptr") (result i32) i32.const 0)
            (func (export "fp_state_len") (result i32) i32.const 0)
            (func (export "fp_state_schema") (result i32) i32.const 1)
            {rate_export})"#
    )
}

#[test]
fn plugins_declare_a_tick_rate_without_breaking_legacy_sixty_hertz_modules() {
    let legacy = Frontplane::from_wat(&tick_rate_module(""), Limits::default()).unwrap();
    let declared = Frontplane::from_wat(
        &tick_rate_module(r#"(func (export "fp_tick_hz") (result i32) i32.const 120)"#),
        Limits::default(),
    )
    .unwrap();

    assert_eq!(legacy.simulation_hz(), 60);
    assert_eq!(declared.simulation_hz(), 120);
}

#[test]
fn tick_rate_bounds_classify_the_complete_boundary_set() {
    let cases = [
        (-1, false),
        (0, false),
        (1, true),
        (60, true),
        (120, true),
        (1_000, true),
        (1_001, false),
    ];

    let actual: Vec<_> = cases
        .iter()
        .map(|(rate, _)| {
            let source = tick_rate_module(&format!(
                "(func (export \"fp_tick_hz\") (result i32) i32.const {rate})"
            ));
            Frontplane::from_wat(&source, Limits::default()).is_ok()
        })
        .collect();
    let expected: Vec<_> = cases.iter().map(|(_, accepted)| *accepted).collect();

    assert_eq!(actual, expected);
}

#[test]
fn a_present_tick_rate_export_with_the_wrong_signature_is_not_legacy() {
    let source =
        tick_rate_module(r#"(func (export "fp_tick_hz") (param i32) (result i32) local.get 0)"#);

    assert!(Frontplane::from_wat(&source, Limits::default()).is_err());
}

#[test]
fn rational_accumulation_has_no_sixty_or_one_twenty_hertz_drift() {
    for rate in [60, 120] {
        let mut single = FixedStepClock::default();
        let whole = single.advance(Duration::from_secs(1), rate, 240);

        let mut fragmented = FixedStepClock::default();
        let fragmented_ticks: u32 = (0..1_000)
            .map(|_| {
                fragmented
                    .advance(Duration::from_millis(1), rate, 240)
                    .ticks
            })
            .sum();

        assert_eq!(whole.ticks, rate);
        assert_eq!(whole.dropped_ticks, 0);
        assert_eq!(fragmented_ticks, rate);
        assert_eq!(
            fragmented.time_until_next_tick(rate),
            Duration::from_nanos(
                1_000_000_000 / rate as u64 + u64::from(1_000_000_000 % rate as u64 != 0)
            )
        );
    }
}

#[test]
fn fractional_tick_phase_and_bounded_catchup_are_explicit() {
    let mut clock = FixedStepClock::default();

    let early = clock.advance(Duration::from_nanos(8_333_333), 120, 8);
    assert_eq!(early.ticks, 0);
    assert_eq!(clock.time_until_next_tick(120), Duration::from_nanos(1));

    let boundary = clock.advance(Duration::from_nanos(1), 120, 8);
    assert_eq!(boundary.ticks, 1);
    assert_eq!(boundary.dropped_ticks, 0);

    let overloaded = clock.advance(Duration::from_secs(10), 120, 8);
    assert_eq!(overloaded.ticks, 8);
    assert_eq!(overloaded.dropped_ticks, 1_192);
    assert_eq!(
        clock.time_until_next_tick(120),
        Duration::from_nanos(8_333_333)
    );
}
