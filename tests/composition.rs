use gpui_wasm::{Affine, DrawCommand, Frontplane, ImageFormat, Limits, PathSegment, Point, Rect};

const COMPOSITION_WAT: &str = r#"(module
	(import "host.v0" "image_define" (func $image (param i32 i32 i32 i32) (result i32)))
	(import "host.v0" "frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
	(import "host.v0" "transform_push" (func $push (param f32 f32 f32 f32 f32 f32) (result i32)))
	(import "host.v0" "transform_pop" (func $pop (result i32)))
	(import "host.v0" "path_begin" (func $path_begin (param i32) (result i32)))
	(import "host.v0" "path_move" (func $move (param f32 f32) (result i32)))
	(import "host.v0" "path_line" (func $line (param f32 f32) (result i32)))
	(import "host.v0" "path_close" (func $close (result i32)))
	(import "host.v0" "path_end" (func $path_end (param f32 i32 i32 i32) (result i32)))
	(import "host.v0" "sprite" (func $sprite
		(param i32 i32 f32 f32 f32 f32 f32 f32 f32 f32 f32 f32 i32 i32)
		(result i32)))
	(import "host.v0" "frame_end" (func $end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "\89PNG")
	(func (export "fp_abi_major") (result i32) i32.const 0)
	(func (export "fp_abi_minor") (result i32) i32.const 0)
	(func (export "fp_configure") (result i32)
		i32.const 7 i32.const 0 i32.const 4 i32.const 0 call $image drop
		i32.const 0)
	(func (export "fp_init") (param i32 i32 f32 f32) (result i32)
		i32.const 64 i32.const 0 i32.store
		i32.const 0)
	(func (export "fp_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "fp_tick") (param $count i32) (result i32)
		i32.const 64 i32.const 64 i32.load local.get $count i32.add i32.store
		i32.const 0)
	(func (export "fp_render") (result i32)
		f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop
		;; 90-degree rotation followed by translation.
		f32.const 0 f32.const 1 f32.const -1 f32.const 0 f32.const 100 f32.const 50 call $push drop
		i32.const 40 call $path_begin drop
		f32.const 0 f32.const 0 call $move drop
		f32.const 16 f32.const 0 call $line drop
		f32.const 8 f32.const 12 call $line drop
		call $close drop
		f32.const 2 i32.const 0x44ccffff i32.const -1 i32.const 0 call $path_end drop
		i32.const 50 i32.const 7
		i32.const 64 i32.load i32.const 1 i32.and i32.const 16 i32.mul f32.convert_i32_u
		f32.const 0 f32.const 16 f32.const 16
		f32.const 10 f32.const 20 f32.const 32 f32.const 32
		f32.const 0.5 f32.const 0.5 i32.const -1 i32.const 0 call $sprite drop
		call $pop drop
		call $end drop
		i32.const 0)
	(func (export "fp_state_ptr") (result i32) i32.const 64)
	(func (export "fp_state_len") (result i32) i32.const 4)
	(func (export "fp_state_schema") (result i32) i32.const 1)
)"#;

fn fixture() -> Frontplane {
    let mut frontplane = Frontplane::from_wat(COMPOSITION_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(1, 320.0, 240.0).unwrap();
    frontplane
}

#[test]
fn resources_paths_and_rotatable_sprites_are_generic_commands() {
    let mut frontplane = fixture();
    assert_eq!(frontplane.images().len(), 1);
    assert_eq!(frontplane.images()[0].id, 7);
    assert_eq!(frontplane.images()[0].format, ImageFormat::Encoded);
    assert_eq!(frontplane.images()[0].bytes, b"\x89PNG");

    let frame = frontplane.render().unwrap();
    assert_eq!(
        frame.commands,
        vec![
            DrawCommand::PushTransform(Affine {
                m11: 0.0,
                m12: 1.0,
                m21: -1.0,
                m22: 0.0,
                tx: 100.0,
                ty: 50.0,
            }),
            DrawCommand::Path {
                id: 40,
                segments: vec![
                    PathSegment::Move(Point { x: 0.0, y: 0.0 }),
                    PathSegment::Line(Point { x: 16.0, y: 0.0 }),
                    PathSegment::Line(Point { x: 8.0, y: 12.0 }),
                    PathSegment::Close,
                ],
                width: 2.0,
                fill_rgba: Some(0x44ccffff),
                stroke_rgba: Some(0xffffffff),
            },
            DrawCommand::Sprite {
                id: 50,
                image_id: 7,
                source: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 16.0,
                    height: 16.0
                },
                destination: Rect {
                    x: 10.0,
                    y: 20.0,
                    width: 32.0,
                    height: 32.0
                },
                pivot: Point { x: 0.5, y: 0.5 },
                tint_rgba: 0xffffffff,
                flags: 0,
            },
            DrawCommand::PopTransform,
        ]
    );
}

#[test]
fn the_plugin_not_the_host_owns_sprite_animation_time() {
    let mut frontplane = fixture();
    let first = frontplane.render().unwrap();
    let snapshot = frontplane.snapshot().unwrap();
    frontplane.tick(1).unwrap();
    let second = frontplane.render().unwrap();
    assert_ne!(first, second, "the plugin selected the next atlas frame");

    frontplane.restore(&snapshot).unwrap();
    assert_eq!(frontplane.render().unwrap(), first);
}

#[test]
fn unbalanced_composition_stacks_reject_the_whole_frame() {
    let source = COMPOSITION_WAT.replace("call $pop drop", "nop");
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(1, 320.0, 240.0).unwrap();

    assert!(frontplane.render().is_err());
}

#[test]
fn image_and_text_payloads_have_independent_budgets() {
    let source = COMPOSITION_WAT
        .replace(
            "i32.const 4 i32.const 0 call $image",
            "i32.const 5000 i32.const 0 call $image",
        )
        .replace("(data (i32.const 0) \"\\89PNG\")", "");
    let limits = Limits {
        max_string_bytes: 32,
        max_image_bytes: 8_192,
        ..Limits::default()
    };
    let mut frontplane = Frontplane::from_wat(&source, limits).unwrap();

    frontplane
        .configure()
        .expect("image bytes use the image budget, not the text budget");
    assert_eq!(frontplane.images()[0].bytes.len(), 5_000);
}
