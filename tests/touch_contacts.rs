use aedicule::{Event, Frontplane, Limits, TouchContactPhase, TouchContactTracker};

/// Integer-profile probe which opts into eight simultaneous contacts and logs
/// every ordinary event as four adjacent words starting at byte 68.
const TOUCH_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_touch_interest" (func $touch_interest (param i32 i32) (result i32)))
	(memory (export "memory") 1)
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 10)
	(func (export "AE_configure") (result i32)
		i32.const 8 i32.const 0 call $touch_interest)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_event_i32")
		(param $kind i32) (param $code i32) (param $a i32) (param $b i32) (result i32)
		(local $address i32)
		i32.const 68
		i32.const 64 i32.load i32.const 16 i32.mul
		i32.add local.set $address
		local.get $address local.get $kind i32.store
		local.get $address i32.const 4 i32.add local.get $code i32.store
		local.get $address i32.const 8 i32.add local.get $a i32.store
		local.get $address i32.const 12 i32.add local.get $b i32.store
		i32.const 64 i32.const 64 i32.load i32.const 1 i32.add i32.store
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32) i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 132)
	(func (export "AE_state_schema") (result i32) i32.const 1))"#;

fn ready_frontplane(wat: &str) -> Frontplane {
	let mut frontplane = Frontplane::from_wat(wat, Limits::default()).unwrap();
	frontplane.configure().unwrap();
	frontplane.init(7, 320.0, 240.0).unwrap();
	frontplane
}

fn state_words(frontplane: &mut Frontplane) -> Vec<i32> {
	frontplane
		.snapshot()
		.unwrap()
		.bytes
		.chunks_exact(4)
		.map(|word| i32::from_le_bytes(word.try_into().unwrap()))
		.collect()
}

#[test]
fn touch_interest_is_configure_only_bounded_and_transactional() {
	let mut frontplane = Frontplane::from_wat(TOUCH_WAT, Limits::default()).unwrap();
	frontplane.configure().unwrap();
	assert_eq!(frontplane.metadata().touch_max_contacts, Some(8));

	for (name, arguments) in [
		("zero contacts", "i32.const 0 i32.const 0"),
		("negative contacts", "i32.const -1 i32.const 0"),
		("too many contacts", "i32.const 17 i32.const 0"),
		("unknown flags", "i32.const 8 i32.const 1"),
	] {
		let invalid = TOUCH_WAT.replace("i32.const 8 i32.const 0", arguments);
		let mut frontplane = Frontplane::from_wat(&invalid, Limits::default()).unwrap();
		assert!(frontplane.configure().is_err(), "{name}");
		assert_eq!(frontplane.metadata().touch_max_contacts, None, "{name}");
	}
}

#[test]
fn touch_phases_and_opaque_identity_cross_the_integer_event_boundary() {
	let mut frontplane = ready_frontplane(TOUCH_WAT);
	for event in [
		Event::TouchStart { id: 41, x: 10.5, y: 20.25 },
		Event::TouchMove { id: 42, x: 300.0, y: 100.0 },
		Event::TouchEnd { id: 41, x: 11.0, y: 21.0 },
		Event::TouchCancel { id: 42, x: 300.0, y: 100.0 },
	] {
		frontplane.event(event).unwrap();
	}

	let words = state_words(&mut frontplane);
	assert_eq!(words[0], 4);
	assert_eq!(
		&words[1..17],
		&[
			11, 41, 10 * 65536 + 32768, 20 * 65536 + 16384,
			12, 42, 300 * 65536, 100 * 65536,
			13, 41, 11 * 65536, 21 * 65536,
			14, 42, 300 * 65536, 100 * 65536,
		],
	);
}

#[test]
fn legacy_guest_without_touch_interest_cannot_receive_raw_contacts() {
	let legacy = TOUCH_WAT
		.replace(
			"\t(import \"aedicule.v0\" \"AE_touch_interest\" (func $touch_interest (param i32 i32) (result i32)))\n",
			"",
		)
		.replace("\t\ti32.const 8 i32.const 0 call $touch_interest", "\t\ti32.const 0");
	let mut frontplane = ready_frontplane(&legacy);
	assert!(
		frontplane
			.event(Event::TouchStart { id: 1, x: 1.0, y: 2.0 })
			.is_err()
	);
}

#[test]
fn tracker_classifies_complete_contact_sets_without_crossing_identities() {
	let mut tracker = TouchContactTracker::new(2).unwrap();
	let start = |id, x| (TouchContactPhase::Start, id, x, 20.0);
	let move_to = |id, x| (TouchContactPhase::Move, id, x, 30.0);
	let end = |id, x| (TouchContactPhase::End, id, x, 99.0);

	let observe = |tracker: &mut TouchContactTracker, (phase, id, x, y)| {
		tracker.observe(phase, id, x, y)
	};
	assert_eq!(
		observe(&mut tracker, start(41, 10.0)),
		Some(Event::TouchStart { id: 41, x: 10.0, y: 20.0 })
	);
	assert_eq!(
		observe(&mut tracker, start(42, 200.0)),
		Some(Event::TouchStart { id: 42, x: 200.0, y: 20.0 })
	);
	assert_eq!(observe(&mut tracker, start(41, 50.0)), None, "duplicate start");
	assert_eq!(observe(&mut tracker, start(43, 50.0)), None, "over guest bound");
	assert_eq!(observe(&mut tracker, move_to(99, 1.0)), None, "unknown move");
	assert_eq!(observe(&mut tracker, end(99, 1.0)), None, "unknown end");
	assert_eq!(
		observe(&mut tracker, move_to(42, 210.0)),
		Some(Event::TouchMove { id: 42, x: 210.0, y: 30.0 })
	);
	assert_eq!(
		observe(&mut tracker, end(41, 999.0)),
		Some(Event::TouchEnd { id: 41, x: 10.0, y: 20.0 }),
		"terminal carries the last admitted coordinate"
	);
	assert_eq!(observe(&mut tracker, end(41, 999.0)), None, "duplicate terminal");
	assert_eq!(
		observe(&mut tracker, start(41, 30.0)),
		Some(Event::TouchStart { id: 41, x: 30.0, y: 20.0 }),
		"an ID may be reused only after its terminal edge"
	);
	assert_eq!(
		observe(&mut tracker, move_to(43, 80.0)),
		None,
		"a contact suppressed at capacity cannot appear through a move"
	);
}

#[test]
fn focus_cleanup_cancels_each_admitted_contact_in_start_order() {
	let mut tracker = TouchContactTracker::new(3).unwrap();
	for (id, x) in [(8, 10.0), (2, 20.0), (5, 30.0)] {
		tracker.observe(TouchContactPhase::Start, id, x, 40.0).unwrap();
	}
	tracker.observe(TouchContactPhase::Move, 2, 25.0, 45.0).unwrap();

	assert_eq!(
		tracker.cancel_all(),
		vec![
			Event::TouchCancel { id: 8, x: 10.0, y: 40.0 },
			Event::TouchCancel { id: 2, x: 25.0, y: 45.0 },
			Event::TouchCancel { id: 5, x: 30.0, y: 40.0 },
		]
	);
	assert!(tracker.cancel_all().is_empty());
}
