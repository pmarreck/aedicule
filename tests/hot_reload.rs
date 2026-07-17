use gpui_wasm::{Frontplane, Limits, PluginInit, StateTransfer, prepare_reload};

fn stateful_wat(delta: i32, schema: u32, state_length: u32, render_status: i32) -> String {
    format!(
        r#"(module
            (import "aedicule.v0" "AE_frame_begin"
                (func $frame_begin (param f32 f32 f32 f32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
            (memory (export "memory") 1)
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) i32.const 0)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32)
                i32.const 32 i32.const 0 i32.store
                i32.const 0)
            (func (export "AE_event") (param i32 i32 f32 f32) (result i32)
                i32.const 0)
            (func (export "AE_tick") (param $count i32) (result i32)
                i32.const 32
                i32.const 32 i32.load
                local.get $count i32.const {delta} i32.mul
                i32.add
                i32.store
                i32.const 0)
            (func (export "AE_render") (result i32)
                f32.const 0 f32.const 0 f32.const 0 f32.const 1
                call $frame_begin drop
                call $frame_end drop
                i32.const {render_status})
            (func (export "AE_state_ptr") (result i32) i32.const 32)
            (func (export "AE_state_len") (result i32) i32.const {state_length})
            (func (export "AE_state_schema") (result i32) i32.const {schema})
        )"#
    )
}

fn load(source: &str) -> Frontplane {
    let mut frontplane = Frontplane::from_wat(source, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(7, 1024.0, 768.0).unwrap();
    frontplane
}

fn state_value(frontplane: &mut Frontplane) -> i32 {
    i32::from_le_bytes(
        frontplane.snapshot().unwrap().bytes[0..4]
            .try_into()
            .unwrap(),
    )
}

#[test]
fn calculation_only_reload_preserves_state_and_uses_new_code() {
    let mut current = load(&stateful_wat(1, 1, 4, 0));
    current.tick(3).unwrap();

    let mut prepared = prepare_reload(
        &mut current,
        &stateful_wat(10, 1, 4, 0),
        Limits::default(),
        PluginInit::new(7, 1024.0, 768.0),
    )
    .unwrap();

    assert_eq!(prepared.state_transfer, StateTransfer::Preserved);
    assert_eq!(state_value(&mut prepared.frontplane), 3);
    prepared.frontplane.tick(1).unwrap();
    assert_eq!(state_value(&mut prepared.frontplane), 13);
}

#[test]
fn changed_state_contract_restarts_the_candidate() {
    let mut current = load(&stateful_wat(1, 1, 4, 0));
    current.tick(3).unwrap();

    let mut prepared = prepare_reload(
        &mut current,
        &stateful_wat(10, 2, 8, 0),
        Limits::default(),
        PluginInit::new(7, 1024.0, 768.0),
    )
    .unwrap();

    assert_eq!(prepared.state_transfer, StateTransfer::Restarted);
    assert_eq!(state_value(&mut prepared.frontplane), 0);
    prepared.frontplane.tick(1).unwrap();
    assert_eq!(state_value(&mut prepared.frontplane), 10);
}

#[test]
fn failed_candidate_never_mutates_or_replaces_the_working_plugin() {
    let mut current = load(&stateful_wat(1, 1, 4, 0));
    current.tick(3).unwrap();

    let error = prepare_reload(
        &mut current,
        &stateful_wat(10, 1, 4, 9),
        Limits::default(),
        PluginInit::new(7, 1024.0, 768.0),
    )
    .unwrap_err();

    assert!(error.to_string().contains("AE_render"));
    assert_eq!(state_value(&mut current), 3);
    current.tick(1).unwrap();
    assert_eq!(state_value(&mut current), 4);
}
