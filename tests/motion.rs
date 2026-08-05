use aedicule::{
    Event, Frontplane, Limits, MotionInterest, MotionInterestKind, SHAKE_COOLDOWN_MS,
    SHAKE_THRESHOLD_MPS2, ShakeDetector,
};

/// Integer-profile guest that registers both motion interests and records
/// what it receives: the kind-16 gesture tuple at 64..72, the six-axis
/// sample at 72..96.
const MOTION_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $frame_begin_rgba (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(import "aedicule.v0" "AE_motion_interest" (func $motion_interest (param i32 i32 i32) (result i32)))
	(memory (export "memory") 1)
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 7)
	(func (export "AE_configure") (result i32)
		i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop
		i32.const 2 i32.const 0 i32.const 0 call $motion_interest drop
		i32.const 0)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_event_i32") (param $kind i32) (param $code i32) (param $a i32) (param $b i32) (result i32)
		local.get $kind i32.const 16 i32.eq
		if
			i32.const 64 local.get $code i32.store
			i32.const 68 local.get $a i32.store
		end
		i32.const 0)
	(func (export "AE_motion_event")
		(param $ax i32) (param $ay i32) (param $az i32)
		(param $rx i32) (param $ry i32) (param $rz i32) (result i32)
		i32.const 72 local.get $ax i32.store
		i32.const 76 local.get $ay i32.store
		i32.const 80 local.get $az i32.store
		i32.const 84 local.get $rx i32.store
		i32.const 88 local.get $ry i32.store
		i32.const 92 local.get $rz i32.store
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		i32.const 0x123456ff call $frame_begin_rgba drop
		call $frame_end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 32)
	(func (export "AE_state_schema") (result i32) i32.const 1)
)"#;

fn state_words(frontplane: &mut Frontplane) -> Vec<i32> {
    frontplane
        .snapshot()
        .unwrap()
        .bytes
        .chunks_exact(4)
        .map(|word| i32::from_le_bytes(word.try_into().unwrap()))
        .collect()
}

fn ready_frontplane(wat: &str) -> Frontplane {
    let mut frontplane = Frontplane::from_wat(wat, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 320.0, 240.0).unwrap();
    frontplane
}

#[test]
fn both_motion_interests_register_during_configure() {
    let mut frontplane = Frontplane::from_wat(MOTION_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    assert_eq!(
        frontplane.metadata().motion_interests,
        vec![
            MotionInterest {
                kind: MotionInterestKind::ShakeGesture,
                rate_hz: 0,
            },
            MotionInterest {
                kind: MotionInterestKind::SixAxisSample,
                rate_hz: 60,
            },
        ],
    );
}

#[test]
fn motion_interest_rejections_cover_the_argument_domain() {
    // Classify the whole (kind, rate, flags) domain, not single examples:
    // every invalid registration must reject the configure transaction.
    for (name, call) in [
        ("unknown kind 0", "i32.const 0 i32.const 0 i32.const 0"),
        ("unknown kind 3", "i32.const 3 i32.const 0 i32.const 0"),
        ("shake with a rate", "i32.const 1 i32.const 60 i32.const 0"),
        ("negative rate", "i32.const 2 i32.const -1 i32.const 0"),
        ("rate beyond 120", "i32.const 2 i32.const 121 i32.const 0"),
        ("nonzero flags", "i32.const 1 i32.const 0 i32.const 1"),
        (
            "duplicate kind",
            "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop \
             i32.const 1 i32.const 0 i32.const 0",
        ),
    ] {
        let invalid = MOTION_WAT.replace(
            "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop\n\t\ti32.const 2 i32.const 0 i32.const 0 call $motion_interest drop",
            &format!("{call} call $motion_interest drop"),
        );
        let mut frontplane = Frontplane::from_wat(&invalid, Limits::default()).unwrap();
        assert!(
            frontplane.configure().is_err(),
            "{name} must reject the configure transaction",
        );
        assert!(
            frontplane.metadata().motion_interests.is_empty(),
            "{name} must roll back every registered interest",
        );
    }
}

#[test]
fn a_bounded_sample_rate_registers_exactly() {
    let explicit = MOTION_WAT.replace(
        "i32.const 2 i32.const 0 i32.const 0",
        "i32.const 2 i32.const 120 i32.const 0",
    );
    let mut frontplane = Frontplane::from_wat(&explicit, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    assert_eq!(frontplane.metadata().motion_interests[1].rate_hz, 120);
}

#[test]
fn a_declared_sample_interest_requires_its_motion_event_export() {
    let incomplete = MOTION_WAT.replace(
        "(export \"AE_motion_event\")",
        "(export \"AE_motion_event_missing\")",
    );
    let mut frontplane = Frontplane::from_wat(&incomplete, Limits::default()).unwrap();
    assert!(frontplane.configure().is_err());
    assert!(frontplane.metadata().motion_interests.is_empty());
}

#[test]
fn a_shake_only_guest_needs_no_motion_event_export() {
    let shake_only = MOTION_WAT
        .replace(
            "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop\n\t\ti32.const 2 i32.const 0 i32.const 0 call $motion_interest drop",
            "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop",
        )
        .replace("(export \"AE_motion_event\")", "(export \"AE_motion_event_unused\")");
    let mut frontplane = Frontplane::from_wat(&shake_only, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    assert_eq!(frontplane.metadata().motion_interests.len(), 1);
}

#[test]
fn a_motion_gesture_arrives_as_event_kind_sixteen_in_q16() {
    let mut frontplane = ready_frontplane(MOTION_WAT);
    frontplane
        .event(Event::MotionGesture { magnitude: 18.5 })
        .unwrap();
    let words = state_words(&mut frontplane);
    assert_eq!(words[0], 1, "gesture code 1 is the shake gesture");
    assert_eq!(words[1], (f64::from(18.5f32) * 65536.0).round() as i32);
}

#[test]
fn a_six_axis_sample_arrives_through_the_motion_event_export_in_q16() {
    let mut frontplane = ready_frontplane(MOTION_WAT);
    frontplane
        .event(Event::MotionSample {
            acceleration: [0.5, -9.8, 0.25],
            rotation: [90.0, -45.5, 0.0],
        })
        .unwrap();
    let words = state_words(&mut frontplane);
    assert_eq!(
        &words[2..8],
        &[
            (f64::from(0.5f32) * 65536.0).round() as i32,
            (f64::from(-9.8f32) * 65536.0).round() as i32,
            (f64::from(0.25f32) * 65536.0).round() as i32,
            (f64::from(90.0f32) * 65536.0).round() as i32,
            (f64::from(-45.5f32) * 65536.0).round() as i32,
            0,
        ],
    );
}

#[test]
fn undeclared_motion_events_are_rejected_not_delivered() {
    let shake_only = MOTION_WAT.replace(
        "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop\n\t\ti32.const 2 i32.const 0 i32.const 0 call $motion_interest drop",
        "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop",
    );
    let mut frontplane = ready_frontplane(&shake_only);
    assert!(
        frontplane
            .event(Event::MotionSample {
                acceleration: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
            })
            .is_err(),
        "samples must not reach a guest that only asked for shakes",
    );

    let sample_only = MOTION_WAT.replace(
        "i32.const 1 i32.const 0 i32.const 0 call $motion_interest drop\n\t\ti32.const 2 i32.const 0 i32.const 0 call $motion_interest drop",
        "i32.const 2 i32.const 0 i32.const 0 call $motion_interest drop",
    );
    let mut frontplane = ready_frontplane(&sample_only);
    assert!(
        frontplane
            .event(Event::MotionGesture { magnitude: 20.0 })
            .is_err(),
        "gestures must not reach a guest that only asked for samples",
    );
}

#[test]
fn non_finite_or_unbounded_motion_values_are_rejected() {
    let mut frontplane = ready_frontplane(MOTION_WAT);
    for acceleration in [
        [f32::NAN, 0.0, 0.0],
        [f32::INFINITY, 0.0, 0.0],
        [20000.0, 0.0, 0.0],
    ] {
        assert!(
            frontplane
                .event(Event::MotionSample {
                    acceleration,
                    rotation: [0.0, 0.0, 0.0],
                })
                .is_err(),
        );
    }
    assert!(
        frontplane
            .event(Event::MotionGesture {
                magnitude: f32::NAN,
            })
            .is_err(),
    );
}

#[test]
fn shake_detection_fires_on_threshold_once_per_cooldown() {
    let mut detector = ShakeDetector::default();

    // Ordinary handling noise stays below the threshold and never fires.
    for step in 0..50u64 {
        assert_eq!(detector.observe(step * 16, SHAKE_THRESHOLD_MPS2 - 0.1), None);
    }

    // A spike fires exactly once with its magnitude...
    assert_eq!(
        detector.observe(1000, SHAKE_THRESHOLD_MPS2 + 5.0),
        Some(SHAKE_THRESHOLD_MPS2 + 5.0),
    );
    // ...and the cooldown swallows the rest of the same shake.
    assert_eq!(detector.observe(1016, SHAKE_THRESHOLD_MPS2 + 10.0), None);
    assert_eq!(
        detector.observe(1000 + SHAKE_COOLDOWN_MS - 1, SHAKE_THRESHOLD_MPS2 + 1.0),
        None,
    );

    // After the cooldown a fresh shake fires again.
    assert_eq!(
        detector.observe(1000 + SHAKE_COOLDOWN_MS, SHAKE_THRESHOLD_MPS2 + 2.0),
        Some(SHAKE_THRESHOLD_MPS2 + 2.0),
    );
}
