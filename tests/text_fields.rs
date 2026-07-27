//! Guest-declared text-entry controls.
//!
//! Text entry is the one control that must focus an editable DOM element, which
//! is what raises a software keyboard. The complementary half — that ordinary
//! content interaction leaves nothing editable focused — is proven by the
//! browser gate's `editableFocusedAtStartup` oracle. Together they form the
//! classifier: editable focus exactly while text entry is active, never
//! otherwise.
//!
//! Delivery carries no bytes. One value becomes an ordered run of integer
//! scalar events terminated by an exact count, so the host never writes into
//! guest memory and the guest never decodes UTF-8.

use aedicule::{Event, Frontplane, Limits, TextField, TextPhase, text_value_events};

const TEXT_FIELD_WAT: &str = include_str!("fixtures/text_field.wat");

fn configured_text_field(wat: &str) -> Result<Frontplane, String> {
	let mut frontplane =
		Frontplane::from_wat(wat, Limits::default()).map_err(|error| error.to_string())?;
	frontplane.configure().map_err(|error| error.to_string())?;
	frontplane
		.init(0x5eed, 640.0, 480.0)
		.map_err(|error| error.to_string())?;
	Ok(frontplane)
}

/// Drives one complete value through the adapter-shared expansion so the tests
/// exercise exactly the sequence every adapter emits.
fn deliver(frontplane: &mut Frontplane, id: u32, value: &str, phase: TextPhase) -> Result<(), String> {
	for event in text_value_events(id, value, phase) {
		frontplane.event(event).map_err(|error| error.to_string())?;
	}
	Ok(())
}

struct Observed {
	count: u32,
	phase: u32,
	calls: u32,
	scalars: Vec<u32>,
}

/// Decodes the fixture's observable state region, which mirrors exactly what
/// crossed the ABI boundary rather than what the host believes it sent.
fn observe(frontplane: &mut Frontplane) -> Observed {
	let bytes = frontplane.snapshot().unwrap().bytes;
	let word = |offset: usize| {
		u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("aligned word"))
	};
	let count = word(0);
	Observed {
		count,
		phase: word(4),
		calls: word(8),
		scalars: (0..count as usize).map(|index| word(48 + index * 4)).collect(),
	}
}

#[test]
fn configure_time_declaration_becomes_one_bounded_text_field() {
	let frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	assert_eq!(
		frontplane.metadata().text_fields,
		vec![TextField {
			id: 9,
			label: "Name".to_owned(),
			max_scalars: 64,
		}]
	);
}

#[test]
fn a_declared_text_field_requires_its_text_event_export() {
	let incomplete = TEXT_FIELD_WAT.replace(
		"(export \"AE_text_event\")",
		"(export \"AE_text_event_missing\")",
	);

	let mut frontplane = Frontplane::from_wat(&incomplete, Limits::default()).unwrap();

	assert!(frontplane.configure().is_err());
	assert!(frontplane.metadata().text_fields.is_empty());
}

#[test]
fn a_change_delivers_every_scalar_in_order_then_a_terminating_count() {
	let mut frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	deliver(&mut frontplane, 9, "héllo", TextPhase::Change).unwrap();

	let observed = observe(&mut frontplane);
	assert_eq!(observed.scalars, vec![0x68, 0xE9, 0x6C, 0x6C, 0x6F]);
	assert_eq!(observed.count, 5);
	assert_eq!(observed.phase, TextPhase::Change as u32);
	assert_eq!(observed.calls, 6, "five scalars plus one terminator");
}

/// The bound is a *scalar* bound, not a byte bound. "héllo" is six UTF-8 bytes
/// and five scalars, so a byte-counting host would wrongly reject it at five.
#[test]
fn capacity_counts_unicode_scalars_rather_than_utf8_bytes() {
	let narrow = TEXT_FIELD_WAT.replace("i32.const 64\n\t\ti32.const 0\n\t\tcall $text_field", "i32.const 5\n\t\ti32.const 0\n\t\tcall $text_field");
	assert_ne!(narrow, TEXT_FIELD_WAT, "capacity literal must be rewritten");
	let mut frontplane = configured_text_field(&narrow).unwrap();
	assert_eq!(frontplane.metadata().text_fields[0].max_scalars, 5);

	deliver(&mut frontplane, 9, "héllo", TextPhase::Change).unwrap();

	assert_eq!(observe(&mut frontplane).count, 5);
}

#[test]
fn a_value_longer_than_the_declared_capacity_is_refused() {
	let narrow = TEXT_FIELD_WAT.replace("i32.const 64\n\t\ti32.const 0\n\t\tcall $text_field", "i32.const 5\n\t\ti32.const 0\n\t\tcall $text_field");
	let mut frontplane = configured_text_field(&narrow).unwrap();

	let refusals = text_value_events(9, "héllos", TextPhase::Change)
		.into_iter()
		.filter(|event| frontplane.event(*event).is_err())
		.count();

	assert!(refusals > 0, "an over-long value must not be delivered whole");
}

#[test]
fn an_empty_value_delivers_only_a_terminating_zero_count() {
	let mut frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	deliver(&mut frontplane, 9, "typed", TextPhase::Change).unwrap();
	deliver(&mut frontplane, 9, "", TextPhase::Change).unwrap();

	let observed = observe(&mut frontplane);
	assert_eq!(observed.count, 0, "clearing the field must be expressible");
	assert_eq!(observed.calls, 7, "six for the first value, one terminator");
}

#[test]
fn commit_is_distinguishable_from_a_continuous_change() {
	let mut frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	deliver(&mut frontplane, 9, "ok", TextPhase::Commit).unwrap();

	assert_eq!(observe(&mut frontplane).phase, TextPhase::Commit as u32);
	assert_ne!(TextPhase::Commit as u32, TextPhase::Change as u32);
}

#[test]
fn text_events_for_an_undeclared_field_are_refused() {
	let mut frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	for event in text_value_events(10, "x", TextPhase::Change) {
		assert!(frontplane.event(event).is_err(), "{event:?}");
	}
}

/// Astral-plane scalars are single scalars, not surrogate pairs. A host that
/// counted UTF-16 code units would report two here.
#[test]
fn astral_plane_scalars_travel_as_one_scalar_each() {
	let mut frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	deliver(&mut frontplane, 9, "\u{1F600}", TextPhase::Change).unwrap();

	let observed = observe(&mut frontplane);
	assert_eq!(observed.scalars, vec![0x1F600]);
	assert_eq!(observed.count, 1);
}

/// Lone surrogates are not Unicode scalar values. They cannot arise from a
/// Rust `&str`, so the guard has to be tested at the event boundary directly.
#[test]
fn lone_surrogates_and_out_of_range_code_points_are_refused() {
	let mut frontplane = configured_text_field(TEXT_FIELD_WAT).unwrap();

	for scalar in [0xD800_u32, 0xDFFF, 0x11_0000, u32::MAX] {
		let event = Event::TextScalar {
			id: 9,
			index: 0,
			scalar,
		};
		assert!(
			frontplane.event(event).is_err(),
			"{scalar:#X} is not a Unicode scalar value"
		);
	}
}

/// Expansion is a pure function shared by every adapter, so native, browser,
/// and headless cannot drift into three different wire sequences.
#[test]
fn expansion_is_a_pure_ordered_sequence_shared_by_every_adapter() {
	assert_eq!(
		text_value_events(9, "hé", TextPhase::Commit),
		vec![
			Event::TextScalar {
				id: 9,
				index: 0,
				scalar: 0x68
			},
			Event::TextScalar {
				id: 9,
				index: 1,
				scalar: 0xE9
			},
			Event::TextValue {
				id: 9,
				scalars: 2,
				phase: TextPhase::Commit
			},
		]
	);
}
