use std::time::Duration;

use aedicule::{
    Event, Frontplane, GuestSuspension, Key, Limits, PausePhase, PauseTrigger, PauseTriggerKind,
    PluginInit, SimulationCall, SimulationScheduler, SuspensionDisposition, initialize_frontplane,
    prepare_reload,
};

const PAUSE_GUEST: &str = r#"(module
	(import "aedicule.v0" "AE_pause_trigger"
		(func $pause_trigger (param i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_begin_rgba"
		(func $frame_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 3)
	(func (export "AE_configure") (result i32)
		i32.const 1 i32.const 5 i32.const 0 call $pause_trigger)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32)
		i32.const 0)
	(func (export "AE_event_i32")
		(param $kind i32) (param $code i32) (param i32) (param i32) (result i32)
		i32.const 64 local.get $kind i32.store
		i32.const 68 local.get $code i32.store
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		i32.const 0x101820ff call $frame_begin drop
		call $frame_end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 8)
	(func (export "AE_state_schema") (result i32) i32.const 1))
"#;

fn pause_guest_with_declaration(declaration: &str) -> String {
    PAUSE_GUEST.replace(
        "i32.const 1 i32.const 5 i32.const 0 call $pause_trigger",
        declaration,
    )
}

#[test]
fn pause_trigger_is_declared_and_semantic_phases_cross_the_integer_abi() {
    let mut frontplane = Frontplane::from_wat(PAUSE_GUEST, Limits::default()).unwrap();
    initialize_frontplane(&mut frontplane, PluginInit::new(7, 800.0, 600.0)).unwrap();

    assert_eq!(
        frontplane.metadata().pause_triggers,
        vec![PauseTrigger::key(Key::P)]
    );

    for (phase, expected_code) in [
        (PausePhase::Paused, 1_u8),
        (PausePhase::Resumed, 2),
        (PausePhase::Restored, 3),
    ] {
        frontplane.event(Event::Pause(phase)).unwrap();
        let bytes = frontplane.snapshot().unwrap().bytes;
        assert_eq!(i32::from_le_bytes(bytes[0..4].try_into().unwrap()), 15);
        assert_eq!(bytes[4], expected_code);
    }
}

#[test]
fn invalid_or_duplicate_pause_selectors_reject_the_complete_configuration() {
    for (declaration, expected) in [
        (
            "i32.const 2 i32.const 5 i32.const 0 call $pause_trigger",
            "unsupported or unavailable capability in AE_pause_trigger",
        ),
        (
            "i32.const 1 i32.const 99 i32.const 0 call $pause_trigger",
            "invalid or non-finite number in AE_pause_trigger",
        ),
        (
            "i32.const 1 i32.const 5 i32.const 1 call $pause_trigger",
            "unsupported or unavailable capability in AE_pause_trigger",
        ),
        (
            "i32.const 1 i32.const 5 i32.const 0 call $pause_trigger drop
			 i32.const 1 i32.const 5 i32.const 0 call $pause_trigger",
            "duplicate stable ID 5 in AE_pause_trigger",
        ),
    ] {
        let mut frontplane = Frontplane::from_wat(
            &pause_guest_with_declaration(declaration),
            Limits::default(),
        )
        .unwrap();
        let error = frontplane.configure().unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
        assert!(frontplane.metadata().pause_triggers.is_empty());
    }
}

#[test]
fn a_guest_without_pause_triggers_keeps_every_raw_input_edge() {
    let mut suspension = GuestSuspension::new([]);
    let events = [
        Event::KeyDown(Key::P),
        Event::KeyDown(Key::P),
        Event::KeyUp(Key::P),
        Event::PointerDown {
            button: 1,
            x: 10.0,
            y: 20.0,
        },
        Event::PointerDown {
            button: 1,
            x: 11.0,
            y: 21.0,
        },
        Event::PointerUp {
            button: 1,
            x: 12.0,
            y: 22.0,
        },
        Event::Focus(false),
        Event::Focus(true),
    ];

    for event in events {
        assert_eq!(
            suspension.handle(event),
            SuspensionDisposition::Deliver(event)
        );
    }
    assert!(!suspension.is_suspended());
}

#[test]
fn trigger_edges_are_consumed_and_held_input_is_reconciled_before_resume() {
    let mut suspension = GuestSuspension::new([PauseTrigger::key(Key::P)]);

    assert_eq!(
        suspension.handle(Event::KeyDown(Key::ArrowLeft)),
        SuspensionDisposition::Deliver(Event::KeyDown(Key::ArrowLeft))
    );
    assert_eq!(
        suspension.handle(Event::KeyDown(Key::P)),
        SuspensionDisposition::Pause
    );
    assert!(suspension.is_suspended());

    // A repeated or still-held trigger cannot immediately undo the pause.
    assert_eq!(
        suspension.handle(Event::KeyDown(Key::P)),
        SuspensionDisposition::Consumed
    );
    assert_eq!(
        suspension.handle(Event::KeyUp(Key::P)),
        SuspensionDisposition::Consumed
    );

    // New gameplay input is observed physically but is not armed for resume.
    assert_eq!(
        suspension.handle(Event::KeyDown(Key::Space)),
        SuspensionDisposition::Consumed
    );
    assert_eq!(
        suspension.handle(Event::KeyUp(Key::ArrowLeft)),
        SuspensionDisposition::Consumed
    );
    assert_eq!(
        suspension.handle(Event::KeyUp(Key::Space)),
        SuspensionDisposition::Consumed
    );

    assert_eq!(
        suspension.handle(Event::KeyDown(Key::P)),
        SuspensionDisposition::Resume {
            reconciliation: vec![Event::KeyUp(Key::ArrowLeft)],
        }
    );
    assert!(!suspension.is_suspended());
    assert_eq!(
        suspension.handle(Event::KeyUp(Key::P)),
        SuspensionDisposition::Consumed
    );
}

#[test]
fn a_paused_scheduler_preserves_fractional_phase_without_catch_up() {
    let rate = aedicule::TickRate::new(3, 1);
    let mut scheduler = SimulationScheduler::new(rate, 8, Duration::ZERO);

    assert!(scheduler.pump(Duration::from_millis(250)).calls.is_empty());
    scheduler.resume_after_pause(Duration::from_millis(250), Duration::from_secs(10));

    let next = scheduler.next_deadline();
    assert_eq!(next, Duration::from_nanos(10_083_333_334));
    assert!(
        scheduler
            .pump(next - Duration::from_nanos(1))
            .calls
            .is_empty()
    );
    assert_eq!(scheduler.pump(next).calls, vec![SimulationCall::Tick(1)]);
}

#[test]
fn a_pause_barrier_drains_prior_input_before_the_semantic_pause_event() {
    let mut scheduler = SimulationScheduler::new(60, 8, Duration::ZERO);
    let key = Event::KeyDown(Key::ArrowLeft);
    scheduler.queue_event(Duration::from_millis(5), key);

    assert!(scheduler.pump(Duration::from_millis(5)).calls.is_empty());
    assert_eq!(
        scheduler.drain_events_through(Duration::from_millis(5)),
        vec![key]
    );
    assert_eq!(
        scheduler.pump(Duration::from_millis(17)).calls,
        vec![SimulationCall::Tick(1)]
    );
}

#[test]
fn pause_trigger_selector_is_typed_for_future_non_keyboard_sources() {
    assert_eq!(PauseTrigger::key(Key::P).kind, PauseTriggerKind::Key);
    assert_eq!(PauseTrigger::key(Key::P).code, Key::P as u32);
}

#[test]
fn paused_focus_loss_reconciles_held_inputs_and_resize_is_maintenance_only() {
    let mut suspension = GuestSuspension::new([PauseTrigger::key(Key::P)]);
    let pointer_down = Event::PointerDown {
        button: 1,
        x: 25.0,
        y: 50.0,
    };
    assert_eq!(
        suspension.handle(Event::KeyDown(Key::ArrowLeft)),
        SuspensionDisposition::Deliver(Event::KeyDown(Key::ArrowLeft))
    );
    assert_eq!(
        suspension.handle(pointer_down),
        SuspensionDisposition::Deliver(pointer_down)
    );
    assert_eq!(
        suspension.handle(Event::KeyDown(Key::P)),
        SuspensionDisposition::Pause
    );

    let viewport = Event::Viewport {
        width: 1200.0,
        height: 700.0,
    };
    assert_eq!(
        suspension.handle(viewport),
        SuspensionDisposition::Maintenance(viewport)
    );
    assert_eq!(
        suspension.handle(Event::Focus(false)),
        SuspensionDisposition::Consumed
    );
    assert_eq!(
        suspension.handle(Event::KeyDown(Key::P)),
        SuspensionDisposition::Resume {
            reconciliation: vec![
                Event::KeyUp(Key::ArrowLeft),
                Event::PointerUp {
                    button: 1,
                    x: 25.0,
                    y: 50.0,
                },
                Event::Focus(false),
            ],
        }
    );
}

#[test]
fn a_reload_candidate_can_be_admitted_directly_into_the_paused_lifecycle() {
    let init = PluginInit::new(7, 800.0, 600.0);
    let mut current = Frontplane::from_wat(PAUSE_GUEST, Limits::default()).unwrap();
    initialize_frontplane(&mut current, init).unwrap();

    let mut prepared = prepare_reload(&mut current, PAUSE_GUEST, Limits::default(), init).unwrap();
    prepared.restore_suspension().unwrap();

    let bytes = prepared.frontplane.snapshot().unwrap().bytes;
    assert_eq!(i32::from_le_bytes(bytes[0..4].try_into().unwrap()), 15);
    assert_eq!(i32::from_le_bytes(bytes[4..8].try_into().unwrap()), 3);
}
