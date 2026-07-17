use gpui_wasm::{Frontplane, Limits};

const STATEFUL_FIXTURE: &str = r#"(module
	(import "host.v0" "frame_begin"
		(func $frame_begin (param f32 f32 f32 f32) (result i32)))
	(import "host.v0" "circle"
		(func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
	(import "host.v0" "frame_end" (func $frame_end (result i32)))
	(memory (export "memory") 1)
	(func (export "fp_abi_major") (result i32) i32.const 0)
	(func (export "fp_abi_minor") (result i32) i32.const 0)
	(func (export "fp_configure") (result i32) i32.const 0)
	(func (export "fp_init") (param i32 i32 f32 f32) (result i32)
		i32.const 32 i32.const 0 i32.store
		i32.const 0)
	(func (export "fp_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "fp_tick") (param $count i32) (result i32)
		i32.const 32 i32.const 32 i32.load local.get $count i32.add i32.store
		i32.const 0)
	(func (export "fp_render") (result i32)
		f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $frame_begin drop
		i32.const 1 i32.const 32 i32.load f32.convert_i32_s f32.const 0
		f32.const 2 f32.const 0 i32.const 0xffffffff i32.const 1 call $circle drop
		call $frame_end drop
		i32.const 0)
	(func (export "fp_state_ptr") (result i32) i32.const 32)
	(func (export "fp_state_len") (result i32) i32.const 4)
	(func (export "fp_state_schema") (result i32) i32.const 1)
)"#;

fn fixture() -> Frontplane {
    Frontplane::from_wat(STATEFUL_FIXTURE, Limits::default())
        .and_then(|mut frontplane| {
            frontplane.configure()?;
            frontplane.init(7, 1024.0, 768.0)?;
            Ok(frontplane)
        })
        .expect("generic stateful fixture should load")
}

#[test]
fn rendering_is_pure_and_repeatable() {
    let mut frontplane = fixture();
    frontplane.tick(1).unwrap();
    let before = frontplane.snapshot().unwrap();
    let first = frontplane.render().unwrap();
    let second = frontplane.render().unwrap();

    assert_eq!(first, second);
    assert_eq!(frontplane.snapshot().unwrap(), before);
}

#[test]
fn snapshots_restore_the_exact_rendered_state() {
    let mut frontplane = fixture();
    frontplane.tick(12).unwrap();
    let snapshot = frontplane.snapshot().unwrap();
    let expected_frame = frontplane.render().unwrap();

    frontplane.tick(40).unwrap();
    assert_ne!(frontplane.render().unwrap(), expected_frame);
    frontplane.restore(&snapshot).unwrap();

    assert_eq!(frontplane.render().unwrap(), expected_frame);
}

#[test]
fn mismatched_snapshots_are_rejected_without_mutating_state() {
    let mut frontplane = fixture();
    frontplane.tick(7).unwrap();
    let current = frontplane.snapshot().unwrap();
    let mut wrong_schema = current.clone();
    wrong_schema.schema += 1;
    let mut wrong_length = current.clone();
    wrong_length.bytes.pop();

    for invalid in [&wrong_schema, &wrong_length] {
        assert!(frontplane.restore(invalid).is_err());
        assert_eq!(frontplane.snapshot().unwrap(), current);
    }
}
