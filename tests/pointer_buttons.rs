use aedicule::{DrawCommand, Event, Frontplane, Key, Limits, PointerButton, PointerScrollUnit};

const SECONDARY_BUTTON_WAT: &str = r#"
    (module
        (import "aedicule.v0" "AE_frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
        (import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
        (memory (export "memory") 1)
        (global $down (mut i32) (i32.const 0))
        (global $up (mut i32) (i32.const 0))
        (global $scroll_x (mut f32) (f32.const 0))
        (global $scroll_y (mut f32) (f32.const 0))
        (global $scroll_unit (mut i32) (i32.const 0))
        (func (export "AE_abi_major") (result i32) i32.const 0)
        (func (export "AE_abi_minor") (result i32) i32.const 0)
        (func (export "AE_configure") (result i32) i32.const 0)
        (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_event") (param $kind i32) (param $code i32) (param $x f32) (param $y f32) (result i32)
            local.get $kind i32.const 4 i32.eq
            local.get $code i32.const 2 i32.eq
            i32.and
            local.get $x f32.const 42.5 f32.eq
            i32.and
            local.get $y f32.const 99.25 f32.eq
            i32.and
            if i32.const 1 global.set $down end
            local.get $kind i32.const 5 i32.eq
            local.get $code i32.const 2 i32.eq
            i32.and
            local.get $x f32.const 42.5 f32.eq
            i32.and
            local.get $y f32.const 99.25 f32.eq
            i32.and
            if i32.const 1 global.set $up end
            local.get $kind i32.const 10 i32.eq
            if
                local.get $code global.set $scroll_unit
                local.get $x global.set $scroll_x
                local.get $y global.set $scroll_y
            end
            i32.const 0)
        (func (export "AE_tick") (param i32) (result i32) i32.const 0)
        (func (export "AE_render") (result i32)
            f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $frame_begin drop
            i32.const 1
            global.get $down f32.convert_i32_u
            global.get $up f32.convert_i32_u
            f32.const 4 f32.const 1 i32.const 0xffffffff i32.const 1 call $circle drop
            i32.const 2
            global.get $scroll_x
            global.get $scroll_y
            global.get $scroll_unit f32.convert_i32_u
            f32.const 1 i32.const 0xffffffff i32.const 1 call $circle drop
            call $frame_end)
        (func (export "AE_state_ptr") (result i32) i32.const 0)
        (func (export "AE_state_len") (result i32) i32.const 0)
        (func (export "AE_state_schema") (result i32) i32.const 1))
"#;

#[test]
fn gpui_key_names_classify_the_complete_native_and_browser_key_set() {
    let cases = [
        ("left", Some(Key::ArrowLeft)),
        ("right", Some(Key::ArrowRight)),
        ("up", Some(Key::ArrowUp)),
        ("space", Some(Key::Space)),
        (" ", Some(Key::Space)),
        ("p", Some(Key::P)),
        ("r", Some(Key::R)),
        ("f", Some(Key::F)),
        ("k", Some(Key::K)),
        ("b", Some(Key::B)),
        ("h", Some(Key::H)),
        ("escape", Some(Key::Escape)),
        ("f1", Some(Key::F1)),
        ("w", Some(Key::W)),
        ("a", Some(Key::A)),
        ("d", Some(Key::D)),
        ("down", None),
        ("P", None),
        ("", None),
    ];

    for (name, expected) in cases {
        assert_eq!(
            Key::from_gpui_name(name),
            expected,
            "classification mismatch for {name:?}"
        );
    }
}

#[test]
fn stable_key_ids_round_trip_as_one_append_only_set() {
    let cases = [
        (1, Some(Key::ArrowLeft)),
        (2, Some(Key::ArrowRight)),
        (3, Some(Key::ArrowUp)),
        (4, Some(Key::Space)),
        (5, Some(Key::P)),
        (6, Some(Key::R)),
        (7, Some(Key::F)),
        (8, Some(Key::K)),
        (9, Some(Key::B)),
        (10, Some(Key::H)),
        (11, Some(Key::Escape)),
        (12, Some(Key::F1)),
        (13, Some(Key::W)),
        (14, Some(Key::A)),
        (15, Some(Key::D)),
        (0, None),
        (16, None),
        (u32::MAX, None),
    ];

    for (code, expected) in cases {
        assert_eq!(Key::from_abi(code), expected, "stable key ID {code}");
        if let Some(key) = expected {
            assert_eq!(key as u32, code, "key discriminant drifted for ID {code}");
        }
    }
}

#[test]
fn pointer_buttons_and_two_axis_scroll_reach_ae_event_with_stable_codes() {
    let mut frontplane = Frontplane::from_wat(SECONDARY_BUTTON_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();

    let down = PointerButton::Secondary.down(42.5, 99.25);
    let up = PointerButton::Secondary.up(42.5, 99.25);
    assert_eq!(
        PointerButton::Primary.down(12.0, 24.0),
        Event::PointerDown {
            button: 1,
            x: 12.0,
            y: 24.0,
        }
    );
    assert_eq!(
        PointerButton::Primary.up(12.0, 24.0),
        Event::PointerUp {
            button: 1,
            x: 12.0,
            y: 24.0,
        }
    );
    assert_eq!(
        PointerButton::Middle.down(12.0, 24.0),
        Event::PointerDown {
            button: 3,
            x: 12.0,
            y: 24.0,
        }
    );
    assert_eq!(
        PointerButton::Middle.up(12.0, 24.0),
        Event::PointerUp {
            button: 3,
            x: 12.0,
            y: 24.0,
        }
    );
    assert_eq!(
        down,
        Event::PointerDown {
            button: 2,
            x: 42.5,
            y: 99.25,
        }
    );
    assert_eq!(
        up,
        Event::PointerUp {
            button: 2,
            x: 42.5,
            y: 99.25,
        }
    );

    frontplane.event(down).unwrap();
    frontplane.event(up).unwrap();
    let scroll = PointerScrollUnit::Lines
        .event(-3.5, 2.25)
        .expect("non-zero two-axis scroll is observable");
    assert_eq!(
        scroll,
        Event::PointerScroll {
            unit: PointerScrollUnit::Lines,
            delta_x: -3.5,
            delta_y: 2.25,
        }
    );
    assert_eq!(PointerScrollUnit::Lines.event(0.0, 0.0), None);
    frontplane.event(scroll).unwrap();
    let frame = frontplane.render().unwrap();

    assert_eq!(
        frame.commands,
        vec![
            DrawCommand::Circle {
                id: 1,
                x: 1.0,
                y: 1.0,
                radius: 4.0,
                width: 1.0,
                rgba: 0xffffffff,
                filled: true,
            },
            DrawCommand::Circle {
                id: 2,
                x: -3.5,
                y: 2.25,
                radius: 1.0,
                width: 1.0,
                rgba: 0xffffffff,
                filled: true,
            },
        ]
    );

    let precise_scroll = PointerScrollUnit::LogicalPixels
        .event(1.25, -4.5)
        .expect("one non-zero precise axis is observable");
    frontplane.event(precise_scroll).unwrap();
    let precise_frame = frontplane.render().unwrap();
    assert_eq!(
        precise_frame.commands[1],
        DrawCommand::Circle {
            id: 2,
            x: 1.25,
            y: -4.5,
            radius: 2.0,
            width: 1.0,
            rgba: 0xffffffff,
            filled: true,
        }
    );
}
