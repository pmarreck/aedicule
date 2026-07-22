use std::time::Duration;

use aedicule::{
    Event, FixedStepClock, Frontplane, Key, Limits, MAX_SIMULATION_HZ, PluginInit, SimulationCall,
    SimulationScheduler, TickRate, initialize_frontplane, parse_display_refresh_rate,
    prepare_reload,
};

fn legacy_tick_rate_module(rate_export: &str) -> String {
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

fn tick_rate_module(rate_export: &str) -> String {
    format!(
        r#"(module
            (memory (export "memory") 1)
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) i32.const 0)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_tick") (param i32) (result i32) i32.const 0)
            (func (export "AE_render") (result i32) i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 0)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1)
            {rate_export})"#
    )
}

fn display_timing_module() -> &'static str {
    r#"(module
        (import "aedicule.v0" "AE_frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
        (memory (export "memory") 1)
        (global $display_num (mut i32) i32.const 0)
        (global $current_num (mut i32) i32.const 0)
        (global $current_den (mut i32) i32.const 0)
        (func (export "AE_abi_major") (result i32) i32.const 0)
        (func (export "AE_abi_minor") (result i32) i32.const 0)
        (func (export "AE_configure") (result i32) i32.const 0)
        (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_event") (param $kind i32) (param $code i32) (param f32 f32) (result i32)
            local.get $kind i32.const 9 i32.eq
            if
                local.get $code global.set $display_num
            end
            i32.const 0)
        (func (export "AE_tick_rate") (param $current_numerator i32) (param $current_denominator i32) (result i32 i32)
            local.get $current_numerator global.set $current_num
            local.get $current_denominator global.set $current_den
            global.get $display_num i32.const 60000 i32.eq
            if (result i32 i32)
                i32.const 120000 i32.const 1001
            else
                global.get $current_num i32.const 120000 i32.eq
                global.get $current_den i32.const 1001 i32.eq
                i32.and
                if (result i32 i32)
                    i32.const 0 i32.const 0
                else
                    i32.const 1 i32.const 1
                end
            end)
        (func (export "AE_tick") (param i32) (result i32) i32.const 0)
        (func (export "AE_render") (result i32)
            f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $frame_begin drop
            call $frame_end)
        (func (export "AE_state_ptr") (result i32) i32.const 0)
        (func (export "AE_state_len") (result i32) i32.const 0)
        (func (export "AE_state_schema") (result i32) i32.const 1))"#
}

#[test]
fn plugins_choose_rational_rates_after_display_timing_arrives() {
    let legacy = Frontplane::from_wat(&tick_rate_module(""), Limits::default()).unwrap();
    let declared = Frontplane::from_wat(
        &tick_rate_module(
            r#"(func (export "AE_tick_rate") (param i32 i32) (result i32 i32) i32.const 120 i32.const 1)"#,
        ),
        Limits::default(),
    )
    .unwrap();

    let display_rate = TickRate::new(60_000, 1_001);
    let mut legacy = legacy;
    let mut declared = declared;
    legacy.configure().unwrap();
    legacy.init(7, 1024.0, 768.0).unwrap();
    declared.configure().unwrap();
    declared.init(7, 1024.0, 768.0).unwrap();
    legacy.observe_display_refresh(display_rate).unwrap();
    declared.observe_display_refresh(display_rate).unwrap();

    assert_eq!(legacy.simulation_rate(), display_rate);
    assert_eq!(declared.simulation_rate(), TickRate::new(120, 1));
}

#[test]
fn rational_tick_rate_bounds_classify_the_complete_boundary_set() {
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
                "(func (export \"AE_tick_rate\") (param i32 i32) (result i32 i32) i32.const {rate} i32.const 1)"
            ));
            Frontplane::from_wat(&source, Limits::default())
                .and_then(|mut frontplane| {
                    frontplane.configure()?;
                    frontplane.init(7, 1024.0, 768.0)?;
                    frontplane.observe_display_refresh(TickRate::new(60, 1))?;
                    Ok(frontplane)
                })
                .is_ok()
        })
        .collect();
    let expected: Vec<_> = cases.iter().map(|(_, accepted)| *accepted).collect();

    assert_eq!(actual, expected);
}

#[test]
fn display_refresh_environment_rates_preserve_literal_precision_and_known_nominal_aliases() {
    let accepted = [
        ("120", TickRate::new(120, 1)),
        ("120/1", TickRate::new(120, 1)),
        ("5994/100", TickRate::new(2_997, 50)),
        ("59.94", TickRate::new(60_000, 1_001)),
        ("29.97", TickRate::new(30_000, 1_001)),
        ("23.976", TickRate::new(24_000, 1_001)),
        ("119.88", TickRate::new(120_000, 1_001)),
        ("59.97", TickRate::new(5_997, 100)),
    ];

    for (value, expected) in accepted {
        assert_eq!(
            parse_display_refresh_rate(value).unwrap(),
            expected,
            "{value}"
        );
    }

    let rejected = ["", "0", "0/1", "60/0", "59.", ".94", "120/1/1", "1000.001"];
    for value in rejected {
        assert!(parse_display_refresh_rate(value).is_err(), "{value}");
    }
}

#[test]
fn a_present_rational_tick_rate_export_with_the_wrong_signature_is_rejected() {
    let source =
        tick_rate_module(r#"(func (export "AE_tick_rate") (param i32) (result i32) local.get 0)"#);

    assert!(Frontplane::from_wat(&source, Limits::default()).is_err());
}

#[test]
fn legacy_wat_abi_names_are_rejected_after_the_hard_cutover() {
    assert!(Frontplane::from_wat(&legacy_tick_rate_module(""), Limits::default()).is_err());
}

#[test]
fn rational_ntsc_rates_preserve_exact_long_run_phase() {
    for rate in [TickRate::new(60_000, 1_001), TickRate::new(120_000, 1_001)] {
        let mut whole = FixedStepClock::default();
        let whole_advance = whole.advance_rate(Duration::from_secs(1_001), rate, rate.numerator);

        let mut fragmented = FixedStepClock::default();
        let fragmented_ticks: u32 = (0..1_001)
            .map(|_| {
                fragmented
                    .advance_rate(Duration::from_secs(1), rate, rate.numerator)
                    .ticks
            })
            .sum();

        let mut scheduler = SimulationScheduler::new(rate, rate.numerator, Duration::ZERO);
        let scheduler_pump = scheduler.pump(Duration::from_secs(1_001));

        assert_eq!(whole_advance.ticks, rate.numerator);
        assert_eq!(fragmented_ticks, rate.numerator);
        assert_eq!(tick_count(&scheduler_pump.calls), rate.numerator);
        assert_eq!(
            scheduler.next_deadline(),
            rate.boundary_after(rate.numerator + 1)
        );
    }
}

#[test]
fn display_refresh_is_an_event_then_reselects_a_rational_simulation_rate_once() {
    let initial_display = TickRate::new(60_000, 1_001);
    let moved_display = TickRate::new(144_000, 1_001);
    let mut frontplane = Frontplane::from_wat(display_timing_module(), Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();

    let initial_change = frontplane
        .observe_display_refresh(initial_display)
        .unwrap()
        .expect("initial display mode is delivered");
    assert_eq!(initial_change.current, TickRate::new(120_000, 1_001));
    assert_eq!(frontplane.simulation_rate(), TickRate::new(120_000, 1_001));
    assert!(
        frontplane
            .observe_display_refresh(initial_display)
            .unwrap()
            .is_none()
    );

    let moved_change = frontplane
        .observe_display_refresh(moved_display)
        .unwrap()
        .expect("changed display mode is delivered");
    assert_eq!(moved_change.previous, TickRate::new(120_000, 1_001));
    assert_eq!(moved_change.current, moved_display);
}

#[test]
fn plugin_initialization_delivers_the_initial_display_event_before_rate_selection() {
    let init = PluginInit::new(7, 1024.0, 768.0).with_display_refresh(TickRate::new(60_000, 1_001));
    let mut frontplane = Frontplane::from_wat(display_timing_module(), Limits::default()).unwrap();

    initialize_frontplane(&mut frontplane, init).unwrap();

    assert_eq!(frontplane.simulation_rate(), TickRate::new(120_000, 1_001));
}

#[test]
fn reload_redelivers_the_current_display_mode_instead_of_a_stale_startup_default() {
    let current_init =
        PluginInit::new(7, 1024.0, 768.0).with_display_refresh(TickRate::new(60_000, 1_001));
    let mut current = Frontplane::from_wat(display_timing_module(), Limits::default()).unwrap();
    initialize_frontplane(&mut current, current_init).unwrap();

    let stale_startup_default =
        PluginInit::new(7, 1024.0, 768.0).with_display_refresh(TickRate::new(144_000, 1_001));
    let prepared = prepare_reload(
        &mut current,
        display_timing_module(),
        Limits::default(),
        stale_startup_default,
    )
    .unwrap();

    assert_eq!(
        prepared.frontplane.simulation_rate(),
        TickRate::new(120_000, 1_001)
    );
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

fn tick_count(calls: &[SimulationCall]) -> u32 {
    calls
        .iter()
        .map(|call| match call {
            SimulationCall::Tick(ticks) => *ticks,
            SimulationCall::Event(_) => 0,
        })
        .sum()
}

#[test]
fn scheduler_partitions_elapsed_time_at_every_supported_hertz() {
    for rate in 1..=MAX_SIMULATION_HZ {
        let mut whole = SimulationScheduler::new(rate, MAX_SIMULATION_HZ, Duration::ZERO);
        let whole_pump = whole.pump(Duration::from_secs(1));

        let mut fragmented = SimulationScheduler::new(rate, MAX_SIMULATION_HZ, Duration::ZERO);
        let fragmented_ticks: u32 = (1..=1_000)
            .map(|millisecond| {
                tick_count(&fragmented.pump(Duration::from_millis(millisecond)).calls)
            })
            .sum();

        assert_eq!(tick_count(&whole_pump.calls), rate);
        assert_eq!(fragmented_ticks, rate);
        assert_eq!(whole_pump.dropped_ticks, 0);
        assert_eq!(whole.next_deadline(), fragmented.next_deadline());
    }
}

#[test]
fn early_late_and_spurious_wakes_never_shift_absolute_boundaries() {
    let mut scheduler = SimulationScheduler::new(120, 8, Duration::ZERO);

    let early = scheduler.pump(Duration::from_nanos(8_333_333));
    assert!(early.calls.is_empty());
    assert!(!early.render);
    assert_eq!(scheduler.next_deadline(), Duration::from_nanos(8_333_334));
    assert_eq!(
        scheduler.time_until_next_wake(Duration::from_nanos(8_333_333)),
        Duration::from_nanos(1)
    );

    let spurious = scheduler.pump(Duration::from_nanos(8_333_333));
    assert!(spurious.calls.is_empty());
    assert_eq!(scheduler.next_deadline(), Duration::from_nanos(8_333_334));

    let late = scheduler.pump(Duration::from_millis(25));
    assert_eq!(late.calls, vec![SimulationCall::Tick(3)]);
    assert_eq!(scheduler.next_deadline(), Duration::from_nanos(33_333_334));
}

#[test]
fn events_are_interleaved_before_their_exact_or_next_tick_boundary() {
    let mut scheduler = SimulationScheduler::new(10, 10, Duration::ZERO);
    let before = Event::KeyDown(Key::ArrowLeft);
    let exactly = Event::KeyUp(Key::ArrowLeft);
    let after = Event::KeyDown(Key::ArrowRight);
    scheduler.queue_event(Duration::from_nanos(99_999_999), before);
    scheduler.queue_event(Duration::from_millis(100), exactly);
    scheduler.queue_event(Duration::from_nanos(100_000_001), after);

    let pump = scheduler.pump(Duration::from_millis(300));

    assert_eq!(
        pump.calls,
        vec![
            SimulationCall::Event(before),
            SimulationCall::Event(exactly),
            SimulationCall::Tick(1),
            SimulationCall::Event(after),
            SimulationCall::Tick(2),
        ]
    );
    assert!(pump.render);
}

#[test]
fn late_wakes_batch_only_event_free_tick_runs() {
    let mut scheduler = SimulationScheduler::new(10, 10, Duration::ZERO);
    let first = Event::KeyDown(Key::Space);
    let second = Event::KeyUp(Key::Space);
    scheduler.queue_event(Duration::from_millis(250), first);
    scheduler.queue_event(Duration::from_millis(450), second);

    let pump = scheduler.pump(Duration::from_millis(600));

    assert_eq!(
        pump.calls,
        vec![
            SimulationCall::Tick(2),
            SimulationCall::Event(first),
            SimulationCall::Tick(2),
            SimulationCall::Event(second),
            SimulationCall::Tick(2),
        ]
    );
    assert_eq!(tick_count(&pump.calls), 6);
}

#[test]
fn bounded_overload_drops_oldest_ticks_but_preserves_every_event_edge() {
    let mut scheduler = SimulationScheduler::new(10, 2, Duration::ZERO);
    let dropped_interval = Event::KeyDown(Key::ArrowUp);
    let before_first_survivor = Event::KeyUp(Key::ArrowUp);
    let between_survivors = Event::KeyDown(Key::Space);
    let after_overload = Event::KeyUp(Key::Space);
    scheduler.queue_event(Duration::from_millis(150), dropped_interval);
    scheduler.queue_event(Duration::from_millis(850), before_first_survivor);
    scheduler.queue_event(Duration::from_millis(950), between_survivors);
    scheduler.queue_event(Duration::from_nanos(1_000_000_001), after_overload);

    let overloaded = scheduler.pump(Duration::from_secs(1));

    assert_eq!(overloaded.dropped_ticks, 8);
    assert_eq!(
        overloaded.calls,
        vec![
            SimulationCall::Event(dropped_interval),
            SimulationCall::Event(before_first_survivor),
            SimulationCall::Tick(1),
            SimulationCall::Event(between_survivors),
            SimulationCall::Tick(1),
        ]
    );
    assert!(overloaded.render);
    assert_eq!(scheduler.total_dropped_ticks(), 8);
    assert_eq!(scheduler.next_deadline(), Duration::from_millis(1_100));

    assert!(
        scheduler
            .pump(Duration::from_millis(1_099))
            .calls
            .is_empty()
    );
    assert_eq!(
        scheduler.pump(Duration::from_millis(1_100)).calls,
        vec![
            SimulationCall::Event(after_overload),
            SimulationCall::Tick(1),
        ]
    );
}

#[test]
fn fake_wake_driver_arms_computed_deadlines_without_sleeping() {
    let mut scheduler = SimulationScheduler::new(60, 8, Duration::from_secs(5));
    let mut armed_delays = Vec::new();

    for now in [
        Duration::from_secs(5),
        Duration::from_nanos(5_008_000_000),
        Duration::from_nanos(5_016_666_667),
    ] {
        scheduler.pump(now);
        armed_delays.push(scheduler.time_until_next_wake(now));
    }

    assert_eq!(
        armed_delays,
        vec![
            Duration::from_nanos(16_666_667),
            Duration::from_nanos(8_666_667),
            Duration::from_nanos(16_666_667),
        ]
    );
}

#[test]
fn live_adapter_uses_computed_scheduler_wakes_instead_of_fixed_sleeps() {
    let source = include_str!("../src/main.rs");
    let forbidden_fixed_loop_fragments = ["FRAME_INTERVAL", "tick(1)"];
    let forbidden_found: Vec<_> = forbidden_fixed_loop_fragments
        .into_iter()
        .filter(|fragment| source.contains(fragment))
        .collect();
    let required_scheduler_fragments = [
        "SimulationScheduler",
        "time_until_next_wake",
        "scheduler.pump",
    ];
    let required_missing: Vec<_> = required_scheduler_fragments
        .into_iter()
        .filter(|fragment| !source.contains(fragment))
        .collect();

    assert!(
        forbidden_found.is_empty(),
        "fixed-loop fragments remain: {forbidden_found:?}"
    );
    assert!(
        required_missing.is_empty(),
        "live scheduler fragments missing: {required_missing:?}"
    );
}
