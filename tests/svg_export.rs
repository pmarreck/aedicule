use gpui_wasm::{Affine, DrawCommand, FrameOutput, Point, Rect, TextFont, render_svg};

#[test]
fn svg_export_is_a_complete_deterministic_visual_artifact() {
    let frame = FrameOutput {
        background: 0x080b12ff,
        commands: vec![
            DrawCommand::PushTransform(Affine {
                m11: 1.0,
                m12: 0.0,
                m21: 0.0,
                m22: 1.0,
                tx: 10.0,
                ty: 20.0,
            }),
            DrawCommand::Line {
                id: 1,
                x1: 1.0,
                y1: 2.0,
                x2: 3.0,
                y2: 4.0,
                width: 2.0,
                rgba: 0xff8000ff,
            },
            DrawCommand::Circle {
                id: 2,
                x: 30.0,
                y: 40.0,
                radius: 8.0,
                width: 1.5,
                rgba: 0x40c0ffff,
                filled: false,
            },
            DrawCommand::Text {
                id: 3,
                text: "WAT < GPUI & \"visible\"".into(),
                x: 50.0,
                y: 60.0,
                size: 18.0,
                rgba: 0xffffffff,
                centered: true,
                font: TextFont::PlatformDefault,
            },
            DrawCommand::Sprite {
                id: 4,
                image_id: 99,
                source: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 16.0,
                    height: 16.0,
                },
                destination: Rect {
                    x: 70.0,
                    y: 80.0,
                    width: 32.0,
                    height: 24.0,
                },
                pivot: Point { x: 16.0, y: 12.0 },
                tint_rgba: 0xffffffff,
                flags: 0,
            },
            DrawCommand::PopTransform,
        ],
    };

    let first = render_svg(&frame, 320.0, 240.0);
    let second = render_svg(&frame, 320.0, 240.0);

    assert_eq!(first, second);
    assert!(first.starts_with(
		"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"320\" height=\"240\" viewBox=\"0 0 320 240\">"
	));
    assert!(first.contains("<rect width=\"320\" height=\"240\" fill=\"#080b12\"/>"));
    assert!(first.contains("transform=\"matrix(1 0 0 1 10 20)\""));
    assert!(first.contains("stroke=\"#ff8000\" stroke-width=\"2\""));
    assert!(first.contains("fill=\"none\" stroke=\"#40c0ff\" stroke-width=\"1.5\""));
    assert!(first.contains("text-anchor=\"middle\""));
    assert!(first.contains("WAT &lt; GPUI &amp; &quot;visible&quot;"));
    assert!(first.contains("data-image-id=\"99\""));
    assert!(first.ends_with("</svg>\n"));
}

#[test]
fn exported_guest_frame_changes_after_simulation_ticks() {
    const ANIMATED_WAT: &str = include_str!("fixtures/animated.wat");
    let mut frontplane =
        gpui_wasm::Frontplane::from_wat(ANIMATED_WAT, gpui_wasm::Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(0x5eed_cafe, 1024.0, 768.0).unwrap();

    let before = render_svg(&frontplane.render().unwrap(), 1024.0, 768.0);
    frontplane.tick(30).unwrap();
    let after = render_svg(&frontplane.render().unwrap(), 1024.0, 768.0);

    assert_ne!(before, after);
    assert!(before.contains("<circle"));
    assert!(after.contains("<circle"));
}
