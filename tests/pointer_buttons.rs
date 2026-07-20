use gpui_wasm::{DrawCommand, Event, Frontplane, Limits, PointerButton, PointerScrollUnit};

const POINTER_WAT: &str = r#"
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
fn pointer_buttons_and_two_axis_scroll_reach_ae_event_with_stable_codes() {
    let mut frontplane = Frontplane::from_wat(POINTER_WAT, Limits::default()).unwrap();
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

#[test]
fn native_adapter_registers_the_complete_pointer_edge_and_scroll_set() {
    let source = include_str!("../src/main.rs");
    let expected_counts = [
        (".on_mouse_down(", 3),
        (".on_mouse_up(", 3),
        (".on_mouse_up_out(", 3),
        ("PointerButton::Primary", 3),
        ("PointerButton::Secondary", 3),
        ("PointerButton::Middle", 3),
        (".on_scroll_wheel(", 1),
    ];

    let mismatches: Vec<_> = expected_counts
        .into_iter()
        .filter_map(|(fragment, expected)| {
            let actual = source.matches(fragment).count();
            (actual != expected).then_some((fragment, expected, actual))
        })
        .collect();

    assert_eq!(mismatches, []);
}
