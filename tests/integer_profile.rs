use aedicule::{
    ButtonPlacement, ControlLabelPlacement, ControlPanel, ControlPhase, DEVICE_FLAG_COARSE_POINTER,
    DeclaredAction, DrawCommand, Event, Frontplane, Key, Limits, PathSegment, Point, Rect,
    SliderControl, SliderPlacement, UiSnapshot, sin_cos_turn_q30,
};

const INTEGER_LIFECYCLE_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $frame_begin_rgba (param i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
	(import "aedicule.v0" "AE_menu_item" (func $menu_item (param i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_action" (func $action (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_slider_i32" (func $slider_i32 (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_ui_begin" (func $ui_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_ui_end" (func $ui_end (result i32)))
	(import "aedicule.v0" "AE_control_panel_q16" (func $control_panel_q16 (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_slider_place_q16" (func $slider_place_q16 (param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_button_place_q16" (func $button_place_q16 (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_path_begin" (func $path_begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_path_move_q16" (func $path_move_q16 (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_path_line_q16" (func $path_line_q16 (param i32 i32) (result i32)))
	(import "aedicule.v0" "AE_path_end_q16" (func $path_end_q16 (param i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_sin_cos_turn" (func $sin_cos_turn (param i32) (result i32 i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "Iterations")
	(data (i32.const 16) "Play/Pause")
	(global $ui_revision (mut i32) (i32.const 1))
	(global $sent_ui_revision (mut i32) (i32.const 0))
	(global $invalid_ui (mut i32) (i32.const 0))
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 8)
	(func (export "AE_configure") (result i32)
		i32.const 7
		i32.const 0
		i32.const 10
		i32.const 0
		i32.const 3600
		i32.const 3
		i32.const 1050
		call $slider_i32
		drop
		i32.const 42
		i32.const 16
		i32.const 10
		i32.const 0
		call $action)
	(func (export "AE_init_i32") (param i32 i32) (param $width i32) (param $height i32) (result i32)
		i32.const 64 local.get $width i32.store
		i32.const 68 local.get $height i32.store
		i32.const 72 i32.const 1050 i32.store
		i32.const 0)
	(func (export "AE_event_i32") (param $kind i32) (param $code i32) (param $a i32) (param $b i32) (result i32)
		local.get $kind i32.const 6 i32.eq
		if
			i32.const 64 local.get $a i32.store
			i32.const 68 local.get $b i32.store
			i32.const 88 local.get $code i32.store
			global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
		end
		local.get $kind i32.const 1 i32.eq
		if
			i32.const 1 global.set $invalid_ui
			global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
		end
		i32.const 0)
	(func (export "AE_control_event") (param i32) (param $value i32) (param $phase i32) (result i32)
		i32.const 72 local.get $value i32.store
		i32.const 76 local.get $phase i32.store
		global.get $ui_revision i32.const 1 i32.add global.set $ui_revision
		i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		(local $sin i32)
		(local $cos i32)
		(local $ui_status i32)
		global.get $ui_revision global.get $sent_ui_revision i32.ne
		if
			global.get $ui_revision call $ui_begin local.set $ui_status
			i32.const 5
			i32.const 786432 i32.const 44564480
			i32.const 65536000 i32.const 4194304
			i32.const 0x080b12e8 i32.const 0
			call $control_panel_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 7 i32.const 5
			global.get $invalid_ui
			if (result i32)
				i32.const 2401
			else
				i32.const 72 i32.load
			end
			i32.const 1572864 i32.const 45350912
			i32.const 63963136 i32.const 2621440
			i32.const 1 i32.const 0
			call $slider_place_q16
			local.get $ui_status i32.or local.set $ui_status
			i32.const 9 i32.const 5
			global.get $invalid_ui
			if (result i32)
				i32.const 43
			else
				i32.const 42
			end
			i32.const 1572864 i32.const 41943040
			i32.const 10485760 i32.const 2621440
			i32.const 1
			call $button_place_q16
			local.get $ui_status i32.or local.set $ui_status
			call $ui_end
			local.get $ui_status i32.or local.tee $ui_status
			i32.eqz
			if
				global.get $ui_revision global.set $sent_ui_revision
			end
		end
		i32.const 0x123456ff call $frame_begin_rgba drop
		i32.const 99 call $path_begin drop
		i32.const 688128 i32.const 1327104 call $path_move_q16 drop
		i32.const -229376 i32.const 262144 call $path_line_q16 drop
		i32.const 81920 i32.const 0 i32.const 0xf6d8a8e8 i32.const 0 call $path_end_q16 drop
		i32.const 0x40000000 call $sin_cos_turn
		local.set $cos
		local.set $sin
		i32.const 80 local.get $sin i32.store
		i32.const 84 local.get $cos i32.store
		call $frame_end drop
		i32.const 0)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 28)
	(func (export "AE_state_schema") (result i32) i32.const 1)
)"#;

fn snapshot_viewport(frontplane: &mut Frontplane) -> (i32, i32) {
    let bytes = frontplane.snapshot().unwrap().bytes;
    (
        i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
        i32::from_le_bytes(bytes[4..8].try_into().unwrap()),
    )
}

fn snapshot_control(frontplane: &mut Frontplane) -> (i32, i32) {
    let bytes = frontplane.snapshot().unwrap().bytes;
    (
        i32::from_le_bytes(bytes[8..12].try_into().unwrap()),
        i32::from_le_bytes(bytes[12..16].try_into().unwrap()),
    )
}

fn snapshot_trig(frontplane: &mut Frontplane) -> (i32, i32) {
    let bytes = frontplane.snapshot().unwrap().bytes;
    (
        i32::from_le_bytes(bytes[16..20].try_into().unwrap()),
        i32::from_le_bytes(bytes[20..24].try_into().unwrap()),
    )
}

fn snapshot_device_flags(frontplane: &mut Frontplane) -> i32 {
    let bytes = frontplane.snapshot().unwrap().bytes;
    i32::from_le_bytes(bytes[24..28].try_into().unwrap())
}

#[test]
fn device_change_events_carry_device_flags_in_the_code_slot() {
    let mut frontplane = Frontplane::from_wat(INTEGER_LIFECYCLE_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 320.5, 240.25).unwrap();
    frontplane
        .event(Event::DeviceChange {
            width: 640.5,
            height: 480.25,
            flags: DEVICE_FLAG_COARSE_POINTER,
        })
        .unwrap();
    assert_eq!(
        snapshot_device_flags(&mut frontplane),
        DEVICE_FLAG_COARSE_POINTER as i32,
    );
    frontplane
        .event(Event::DeviceChange {
            width: 800.0,
            height: 600.0,
            flags: 0,
        })
        .unwrap();
    assert_eq!(snapshot_device_flags(&mut frontplane), 0);
}

#[test]
fn integer_lifecycle_omits_legacy_float_exports_and_receives_q16_viewports() {
    let mut frontplane = Frontplane::from_wat(INTEGER_LIFECYCLE_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    assert_eq!(
        frontplane.metadata().controls,
        vec![SliderControl {
            id: 7,
            label: "Iterations".to_owned(),
            min: 0,
            max: 3600,
            step: 3,
            initial: 1050,
        }]
    );
    assert_eq!(
        frontplane.metadata().actions,
        vec![DeclaredAction {
            id: 42,
            label: "Play/Pause".to_owned(),
        }]
    );
    assert!(
        frontplane.metadata().menu_items.is_empty(),
        "a standalone button action must not create an application-menu item"
    );
    frontplane.init(7, 320.5, 240.25).unwrap();
    assert_eq!(snapshot_viewport(&mut frontplane), (21_004_288, 15_745_024));

    frontplane
        .event(Event::DeviceChange {
            width: 640.5,
            height: 480.25,
            flags: 0,
        })
        .unwrap();
    assert_eq!(snapshot_viewport(&mut frontplane), (41_975_808, 31_473_664));
    frontplane
        .event(Event::Control {
            id: 7,
            value: 2400,
            phase: ControlPhase::Release,
        })
        .unwrap();
    assert_eq!(snapshot_control(&mut frontplane), (2400, 2));
    assert!(
        frontplane
            .event(Event::Control {
                id: 7,
                value: 2401,
                phase: ControlPhase::Change,
            })
            .is_err()
    );
    assert_eq!(snapshot_control(&mut frontplane), (2400, 2));
    let frame = frontplane.render().unwrap();
    assert_eq!(frame.background, 0x123456ff);
    assert_eq!(frame.commands.len(), 1);
    assert_eq!(
        frontplane.ui_snapshot(),
        Some(&UiSnapshot {
            revision: 3,
            control_panels: vec![ControlPanel {
                id: 5,
                bounds: Rect {
                    x: 12.0,
                    y: 680.0,
                    width: 1000.0,
                    height: 64.0,
                },
                rgba: 0x080b12e8,
            }],
            sliders: vec![SliderPlacement {
                id: 7,
                panel_id: 5,
                value: 2400,
                bounds: Rect {
                    x: 24.0,
                    y: 692.0,
                    width: 976.0,
                    height: 40.0,
                },
                label_placement: ControlLabelPlacement::Above,
            }],
            buttons: vec![ButtonPlacement {
                id: 9,
                panel_id: 5,
                action_id: 42,
                bounds: Rect {
                    x: 24.0,
                    y: 640.0,
                    width: 160.0,
                    height: 40.0,
                },
                selected: true,
            }],
            external_links: Vec::new(),
            text_fields: Vec::new(),
        })
    );
    assert_eq!(
        frame.commands,
        vec![DrawCommand::Path {
            id: 99,
            segments: vec![
                PathSegment::Move(Point { x: 10.5, y: 20.25 }),
                PathSegment::Line(Point { x: -3.5, y: 4.0 }),
            ],
            width: 1.25,
            fill_rgba: None,
            stroke_rgba: Some(0xf6d8a8e8),
        }]
    );
    assert_eq!(snapshot_trig(&mut frontplane), (1 << 30, 0));
}

#[test]
fn standalone_and_menu_actions_share_one_collision_checked_namespace() {
    let duplicate = INTEGER_LIFECYCLE_WAT.replace(
        "\t\tcall $action)",
        "\t\tcall $action\n\t\tdrop\n\t\ti32.const 42 i32.const 16 i32.const 10 i32.const 0 i32.const 0 call $menu_item)",
    );
    let mut frontplane = Frontplane::from_wat(&duplicate, Limits::default()).unwrap();

    assert!(frontplane.configure().is_err());
}

#[test]
fn legacy_menu_actions_still_supply_button_labels_without_duplication() {
    let legacy = INTEGER_LIFECYCLE_WAT.replace(
        "\t\ti32.const 0\n\t\tcall $action)",
        "\t\ti32.const 0\n\t\ti32.const 0\n\t\tcall $menu_item)",
    );
    let mut frontplane = Frontplane::from_wat(&legacy, Limits::default()).unwrap();
    frontplane.configure().unwrap();

    assert_eq!(frontplane.metadata().actions.len(), 1);
    assert_eq!(frontplane.metadata().actions[0].label, "Play/Pause");
    assert_eq!(frontplane.metadata().menu_items.len(), 1);
    assert_eq!(frontplane.metadata().menu_items[0].id, Some(42));
}

#[test]
fn legacy_menu_action_cannot_create_an_unlabeled_button_action() {
    let empty_legacy = INTEGER_LIFECYCLE_WAT.replace(
        "\t\ti32.const 42\n\t\ti32.const 16\n\t\ti32.const 10\n\t\ti32.const 0\n\t\tcall $action)",
        "\t\ti32.const 42\n\t\ti32.const 16\n\t\ti32.const 0\n\t\ti32.const 0\n\t\ti32.const 0\n\t\tcall $menu_item)",
    );
    let mut frontplane = Frontplane::from_wat(&empty_legacy, Limits::default()).unwrap();

    assert!(frontplane.configure().is_err());
}

#[test]
fn standalone_action_rejects_reserved_flags_and_empty_labels() {
    let invalid_flags = INTEGER_LIFECYCLE_WAT.replacen(
        "\t\ti32.const 0\n\t\tcall $action)",
        "\t\ti32.const 1\n\t\tcall $action)",
        1,
    );
    let empty_label = INTEGER_LIFECYCLE_WAT.replacen(
        "\t\ti32.const 16\n\t\ti32.const 10\n\t\ti32.const 0\n\t\tcall $action)",
        "\t\ti32.const 16\n\t\ti32.const 0\n\t\ti32.const 0\n\t\tcall $action)",
        1,
    );

    for invalid in [invalid_flags, empty_label] {
        let mut frontplane = Frontplane::from_wat(&invalid, Limits::default()).unwrap();
        assert!(frontplane.configure().is_err());
    }
}

#[test]
fn invalid_ui_snapshot_preserves_the_last_accepted_revision() {
    let mut frontplane = Frontplane::from_wat(INTEGER_LIFECYCLE_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();
    let frame = frontplane.render().unwrap();
    let accepted = frontplane.ui_snapshot().cloned().unwrap();

    frontplane.event(Event::KeyDown(Key::ArrowLeft)).unwrap();
    assert!(frontplane.render().is_err());

    assert_eq!(frontplane.ui_snapshot(), Some(&accepted));
    assert_eq!(frame.background, 0x123456ff);
}

#[test]
fn invalid_button_snapshot_preserves_the_last_accepted_revision() {
    let button_only_failure = INTEGER_LIFECYCLE_WAT.replace(
        "global.get $invalid_ui\n\t\t\tif (result i32)\n\t\t\t\ti32.const 2401\n\t\t\telse\n\t\t\t\ti32.const 72 i32.load\n\t\t\tend",
        "i32.const 72 i32.load",
    );
    let mut frontplane = Frontplane::from_wat(&button_only_failure, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();
    frontplane.render().unwrap();
    let accepted = frontplane.ui_snapshot().cloned().unwrap();

    frontplane.event(Event::KeyDown(Key::ArrowLeft)).unwrap();
    assert!(frontplane.render().is_err());

    assert_eq!(frontplane.ui_snapshot(), Some(&accepted));
}

#[test]
fn invalid_button_references_flags_and_keys_are_rejected() {
    let action_expression = "global.get $invalid_ui\n\t\t\tif (result i32)\n\t\t\t\ti32.const 43\n\t\t\telse\n\t\t\t\ti32.const 42\n\t\t\tend";
    let button_call = "i32.const 9 i32.const 5\n\t\t\tglobal.get $invalid_ui\n\t\t\tif (result i32)\n\t\t\t\ti32.const 43\n\t\t\telse\n\t\t\t\ti32.const 42\n\t\t\tend\n\t\t\ti32.const 1572864 i32.const 41943040\n\t\t\ti32.const 10485760 i32.const 2621440\n\t\t\ti32.const 1\n\t\t\tcall $button_place_q16\n\t\t\tlocal.get $ui_status i32.or local.set $ui_status";
    let mutants = [
        INTEGER_LIFECYCLE_WAT.replacen(action_expression, "i32.const 99", 1),
        INTEGER_LIFECYCLE_WAT.replacen(
            "i32.const 9 i32.const 5\n\t\t\tglobal.get $invalid_ui",
            "i32.const 9 i32.const 99\n\t\t\tglobal.get $invalid_ui",
            1,
        ),
        INTEGER_LIFECYCLE_WAT.replacen(
            "i32.const 1\n\t\t\tcall $button_place_q16",
            "i32.const 2\n\t\t\tcall $button_place_q16",
            1,
        ),
        INTEGER_LIFECYCLE_WAT.replacen(button_call, &format!("{button_call}\n{button_call}"), 1),
    ];

    for mutant in mutants {
        let mut frontplane = Frontplane::from_wat(&mutant, Limits::default()).unwrap();
        frontplane.configure().unwrap();
        frontplane.init(7, 1024.0, 768.0).unwrap();
        assert!(frontplane.render().is_err());
    }
}

#[test]
fn unchanged_ui_revision_is_not_resubmitted_with_each_canvas_frame() {
    let mut frontplane = Frontplane::from_wat(INTEGER_LIFECYCLE_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();

    frontplane.render().unwrap();
    let accepted = frontplane.ui_snapshot().cloned().unwrap();
    frontplane.render().unwrap();

    assert_eq!(frontplane.ui_snapshot(), Some(&accepted));
}

#[test]
fn resubmitting_an_accepted_ui_revision_is_rejected_without_replacement() {
    let duplicate_revision = INTEGER_LIFECYCLE_WAT.replace(
        "global.get $ui_revision global.set $sent_ui_revision",
        "nop",
    );
    let mut frontplane = Frontplane::from_wat(&duplicate_revision, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();

    frontplane.render().unwrap();
    let accepted = frontplane.ui_snapshot().cloned().unwrap();
    assert!(frontplane.render().is_err());

    assert_eq!(frontplane.ui_snapshot(), Some(&accepted));
}

#[test]
fn slider_lattice_maps_only_bounded_exact_step_indices() {
    let control = SliderControl {
        id: 7,
        label: "Iterations".to_owned(),
        min: 0,
        max: 3600,
        step: 3,
        initial: 1050,
    };
    assert_eq!(control.step_count(), 1200);
    assert_eq!(control.step_index(0), Some(0));
    assert_eq!(control.step_index(2400), Some(800));
    assert_eq!(control.step_index(3600), Some(1200));
    assert_eq!(control.step_index(2401), None);
    assert_eq!(control.value_at_step(0), Some(0));
    assert_eq!(control.value_at_step(800), Some(2400));
    assert_eq!(control.value_at_step(1200), Some(3600));
    assert_eq!(control.value_at_step(1201), None);
}

#[test]
fn integer_lifecycle_requires_the_complete_export_pair() {
    let incomplete = INTEGER_LIFECYCLE_WAT.replace(
        "(export \"AE_event_i32\")",
        "(export \"AE_event_i32_missing\")",
    );
    assert!(Frontplane::from_wat(&incomplete, Limits::default()).is_err());
}

#[test]
fn a_declared_slider_requires_its_control_event_export() {
    let incomplete = INTEGER_LIFECYCLE_WAT.replace(
        "(export \"AE_control_event\")",
        "(export \"AE_control_event_missing\")",
    );
    let mut frontplane = Frontplane::from_wat(&incomplete, Limits::default()).unwrap();
    assert!(frontplane.configure().is_err());
    assert!(frontplane.metadata().controls.is_empty());
}

#[test]
fn integer_lifecycle_fixture_has_no_float_types_or_opcodes_after_compilation() {
    let wasm = wat::parse_str(INTEGER_LIFECYCLE_WAT).unwrap();
    let canonical = wasmprinter::print_bytes(wasm).unwrap();
    assert!(!canonical.contains("f32"), "{canonical}");
    assert!(!canonical.contains("f64"), "{canonical}");
}

#[test]
fn integer_trig_has_exact_cardinals_and_a_bounded_unit_circle() {
    assert_eq!(sin_cos_turn_q30(0), (0, 1 << 30));
    assert_eq!(sin_cos_turn_q30(0x4000_0000), (1 << 30, 0));
    assert_eq!(sin_cos_turn_q30(i32::MIN), (0, -(1 << 30)));
    assert_eq!(sin_cos_turn_q30(0xc000_0000_u32 as i32), (-(1 << 30), 0));

    let (sin, cos) = sin_cos_turn_q30(0x2000_0000);
    assert!((sin - 759_250_125).abs() <= 8, "sin={sin}");
    assert!((cos - 759_250_125).abs() <= 8, "cos={cos}");
    let norm = i128::from(sin) * i128::from(sin) + i128::from(cos) * i128::from(cos);
    let unit = i128::from(1_i64 << 60);
    assert!((norm - unit).abs() < 20_000_000_000, "norm={norm}");
}
