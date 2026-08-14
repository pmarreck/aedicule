use aedicule::{Frontplane, FrontplaneError, Limits, Metadata};

fn wat_module(body: &str) -> String {
    format!(
        r#"(module
			(memory (export "memory") 1)
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 0)
			(func (export "AE_configure") (result i32) i32.const 0)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
			{body}
			(func (export "AE_state_ptr") (result i32) i32.const 0)
			(func (export "AE_state_len") (result i32) i32.const 4)
			(func (export "AE_state_schema") (result i32) i32.const 1)
		)"#
    )
}

fn host_module(
    imports: &str,
    declarations: &str,
    configure: &str,
    tick: &str,
    render: &str,
) -> String {
    format!(
        r#"(module
            {imports}
            (memory (export "memory") 1 64)
            {declarations}
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) {configure})
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_tick") (param i32) (result i32) {tick})
            (func (export "AE_render") (result i32) {render})
            (func (export "AE_state_ptr") (result i32) i32.const 32)
            (func (export "AE_state_len") (result i32) i32.const 4)
            (func (export "AE_state_schema") (result i32) i32.const 1)
        )"#
    )
}

#[test]
fn unknown_host_capabilities_are_rejected() {
    let source = r#"(module
		(import "wasi_snapshot_preview1" "fd_write" (func))
		(memory (export "memory") 1)
	)"#;

    let error = Frontplane::from_wat(source, Limits::default()).unwrap_err();
    assert!(matches!(error, FrontplaneError::UnsupportedImport { .. }));
}

#[test]
fn fuel_stops_a_nonterminating_plugin_call() {
    let source = wat_module(
        r#"(func (export "AE_tick") (param i32) (result i32)
			(loop $forever br $forever)
			i32.const 0)
		(func (export "AE_render") (result i32) i32.const 0)"#,
    );
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();

    let error = frontplane.tick(1).unwrap_err();
    assert!(matches!(error, FrontplaneError::FuelExhausted { .. }));
}

#[test]
fn non_finite_drawing_values_fail_closed() {
    let source = r#"(module
			(import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
			(import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
			(import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
			(memory (export "memory") 1)
			(func (export "AE_abi_major") (result i32) i32.const 0)
			(func (export "AE_abi_minor") (result i32) i32.const 0)
			(func (export "AE_configure") (result i32) i32.const 0)
			(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
			(func (export "AE_tick") (param i32) (result i32) i32.const 0)
			(func (export "AE_render") (result i32)
				(f32.const 0) (f32.const 0) (f32.const 0) (f32.const 1) call $begin drop
				(i32.const 1) (f32.const nan) (f32.const 0) (f32.const 1)
				(f32.const 1) (i32.const -1) (i32.const 1) call $circle drop
				call $end drop
				i32.const 0)
			(func (export "AE_state_ptr") (result i32) i32.const 0)
			(func (export "AE_state_len") (result i32) i32.const 4)
			(func (export "AE_state_schema") (result i32) i32.const 1)
        )"#;
    let mut frontplane = Frontplane::from_wat(source, Limits::default()).unwrap();

    let error = frontplane.render().unwrap_err();
    assert!(matches!(error, FrontplaneError::InvalidNumber { .. }));
}

#[test]
fn circle_flags_fail_closed_with_a_specific_diagnostic() {
    let imports = r#"
        (import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
        (import "aedicule.v0" "AE_circle" (func $circle (param i32 f32 f32 f32 f32 i32 i32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
    "#;
    let render = r#"
        f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop
        i32.const 915 f32.const 320 f32.const 240 f32.const 3 f32.const 1
        i32.const 0xffcf5cff i32.const 0x5ee7ffff call $circle drop
        call $end drop
        i32.const 0
    "#;
    let source = host_module(imports, "", "i32.const 0", "i32.const 0", render);
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();

    assert!(matches!(
        frontplane.render(),
        Err(FrontplaneError::InvalidFrame("unsupported circle flags"))
    ));
}

#[test]
fn finite_but_pathological_coordinates_fail_closed() {
    let imports = r#"
        (import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
        (import "aedicule.v0" "AE_line" (func $line (param i32 f32 f32 f32 f32 f32 i32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
    "#;
    let render = r#"
        f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop
        i32.const 1 f32.const 1e30 f32.const 0 f32.const 1 f32.const 1
        f32.const 1 i32.const -1 call $line drop
        call $end drop
        i32.const 0
    "#;
    let source = host_module(imports, "", "i32.const 0", "i32.const 0", render);
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();

    assert!(matches!(
        frontplane.render(),
        Err(FrontplaneError::InvalidNumber { .. })
    ));
}

#[test]
fn incomplete_frames_are_rejected() {
    let body = r#"(func (export "AE_tick") (param i32) (result i32) i32.const 0)
		(func (export "AE_render") (result i32) i32.const 0)"#;
    let limits = Limits {
        max_commands: 2,
        ..Limits::default()
    };
    let mut frontplane = Frontplane::from_wat(&wat_module(body), limits).unwrap();

    assert!(
        frontplane.render().is_err(),
        "a missing complete frame is rejected"
    );
}

#[test]
fn pointer_and_utf8_failures_are_typed_and_fail_closed() {
    let title_import =
        r#"(import "aedicule.v0" "AE_title" (func $title (param i32 i32) (result i32)))"#;
    let sources = [
        host_module(
            title_import,
            "",
            "i32.const 65535 i32.const 2 call $title drop i32.const 0",
            "i32.const 0",
            "i32.const 0",
        ),
        host_module(
            title_import,
            r#"(data (i32.const 0) "\ff")"#,
            "i32.const 0 i32.const 1 call $title drop i32.const 0",
            "i32.const 0",
            "i32.const 0",
        ),
    ];
    let errors: Vec<_> = sources
        .iter()
        .map(|source| {
            let mut frontplane = Frontplane::from_wat(source, Limits::default()).unwrap();
            frontplane.configure().unwrap_err()
        })
        .collect();

    assert!(matches!(errors[0], FrontplaneError::InvalidPointer { .. }));
    assert!(matches!(errors[1], FrontplaneError::InvalidUtf8 { .. }));
}

#[test]
fn command_budget_classifies_complete_frames_as_a_set() {
    let imports = r#"
        (import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
        (import "aedicule.v0" "AE_line" (func $line (param i32 f32 f32 f32 f32 f32 i32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
    "#;
    let outcomes: Vec<_> = (0..=4)
        .map(|count| {
            let calls: String = (0..count)
                .map(|id| {
                    format!(
                        "i32.const {id} f32.const 0 f32.const 0 f32.const 1 f32.const 1 f32.const 1 i32.const -1 call $line drop\n"
                    )
                })
                .collect();
            let render = format!(
                "f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop {calls} call $end drop i32.const 0"
            );
            let limits = Limits {
                max_commands: 2,
                ..Limits::default()
            };
            let source = host_module(imports, "", "i32.const 0", "i32.const 0", &render);
            let mut frontplane = Frontplane::from_wat(&source, limits).unwrap();
            frontplane.render().is_ok()
        })
        .collect();

    assert_eq!(outcomes, vec![true, true, true, false, false]);
}

#[test]
fn memory_and_audio_budgets_are_independent_classifiers() {
    let audio_import =
        r#"(import "aedicule.v0" "AE_audio" (func $audio (param i32 f32 f32 i32) (result i32)))"#;
    let audio_calls = r#"
        i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $audio drop
        i32.const 2 f32.const 1 f32.const 1 i32.const 0 call $audio drop
        i32.const 3 f32.const 1 f32.const 1 i32.const 0 call $audio drop
        i32.const 0
    "#;
    let fixtures = [
        (
            host_module(
                "",
                "",
                "i32.const 0",
                "i32.const 1 memory.grow",
                "i32.const 0",
            ),
            Limits {
                max_memory_bytes: 65_536,
                ..Limits::default()
            },
        ),
        (
            host_module(audio_import, "", "i32.const 0", audio_calls, "i32.const 0"),
            Limits {
                max_audio_events: 2,
                ..Limits::default()
            },
        ),
    ];
    let rejected: Vec<_> = fixtures
        .into_iter()
        .map(|(source, limits)| {
            let mut frontplane = Frontplane::from_wat(&source, limits).unwrap();
            frontplane.tick(1).is_err()
        })
        .collect();

    assert_eq!(rejected, vec![true, true]);
}

#[test]
fn a_partial_trapped_frame_is_discarded_and_the_host_recovers() {
    let imports = r#"
        (import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
    "#;
    let render = r#"
        global.get $first
        if
            i32.const 0 global.set $first
            f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop
            unreachable
        end
        f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $begin drop
        call $end drop
        i32.const 0
    "#;
    let source = host_module(
        imports,
        "(global $first (mut i32) (i32.const 1))",
        "i32.const 0",
        "i32.const 0",
        render,
    );
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();

    assert!(matches!(
        frontplane.render(),
        Err(FrontplaneError::Trap { .. })
    ));
    assert!(frontplane.render().is_ok());
}

#[test]
fn a_failed_call_rolls_back_partial_audio_and_effect_output() {
    let imports = r#"
        (import "aedicule.v0" "AE_audio" (func $audio (param i32 f32 f32 i32) (result i32)))
        (import "aedicule.v0" "AE_effect" (func $effect (param i32 i32 i32) (result i32)))
    "#;
    let tick = r#"
        i32.const 1 f32.const 1 f32.const 1 i32.const 0 call $audio drop
        i32.const 1 i32.const 0 i32.const 0 call $effect drop
        i32.const 99 i32.const 0 i32.const 0 call $effect drop
        i32.const 0
    "#;
    let source = host_module(imports, "", "i32.const 0", tick, "i32.const 0");
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();

    assert!(frontplane.tick(1).is_err());
    assert!(frontplane.drain_audio().is_empty());
    assert!(frontplane.drain_effects().is_empty());
}

#[test]
fn failed_configuration_discards_partial_metadata() {
    let title_import =
        r#"(import "aedicule.v0" "AE_title" (func $title (param i32 i32) (result i32)))"#;
    let configure = r#"
        i32.const 0 i32.const 2 call $title drop
        i32.const 65535 i32.const 2 call $title drop
        i32.const 0
    "#;
    let source = host_module(
        title_import,
        r#"(data (i32.const 0) "ok")"#,
        configure,
        "i32.const 0",
        "i32.const 0",
    );
    let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();

    assert!(frontplane.configure().is_err());
    assert_eq!(frontplane.metadata(), &Metadata::default());
}

#[test]
fn tick_budget_classifies_counts_as_a_set() {
    let source = host_module("", "", "i32.const 0", "i32.const 0", "i32.const 0");
    let outcomes: Vec<_> = [0, 1, 3, 4, 100]
        .into_iter()
        .map(|ticks| {
            let limits = Limits {
                max_ticks_per_call: 3,
                ..Limits::default()
            };
            let mut frontplane = Frontplane::from_wat(&source, limits).unwrap();
            frontplane.tick(ticks).is_ok()
        })
        .collect();

    assert_eq!(outcomes, vec![true, true, true, false, false]);
}

#[test]
fn table_growth_is_bounded_even_when_the_guest_ignores_failure() {
    let source = r#"(module
        (memory (export "memory") 1)
        (table 1 10000 funcref)
        (func (export "AE_abi_major") (result i32) i32.const 0)
        (func (export "AE_abi_minor") (result i32) i32.const 0)
        (func (export "AE_configure") (result i32) i32.const 0)
        (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_tick") (param i32) (result i32)
            ref.null func i32.const 100 table.grow drop
            i32.const 0)
        (func (export "AE_render") (result i32) i32.const 0)
        (func (export "AE_state_ptr") (result i32) i32.const 0)
        (func (export "AE_state_len") (result i32) i32.const 4)
        (func (export "AE_state_schema") (result i32) table.size)
    )"#;
    let limits = Limits {
        max_table_elements: 16,
        ..Limits::default()
    };
    let mut frontplane = Frontplane::from_wat(source, limits).unwrap();

    frontplane.tick(1).unwrap();
    assert_eq!(frontplane.snapshot().unwrap().schema, 1);
}

#[test]
fn unavailable_and_unknown_effects_are_rejected_as_a_set() {
    let template = r#"(module
        (import "aedicule.v0" "AE_effect" (func $effect (param i32 i32 i32) (result i32)))
        (memory (export "memory") 1)
        (func (export "AE_abi_major") (result i32) i32.const 0)
        (func (export "AE_abi_minor") (result i32) i32.const 0)
        (func (export "AE_configure") (result i32) i32.const 0)
        (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_tick") (param i32) (result i32)
            i32.const $KIND i32.const 0 i32.const 0 call $effect drop
            i32.const 0)
        (func (export "AE_render") (result i32) i32.const 0)
        (func (export "AE_state_ptr") (result i32) i32.const 0)
        (func (export "AE_state_len") (result i32) i32.const 4)
        (func (export "AE_state_schema") (result i32) i32.const 1)
    )"#;
    let outcomes: Vec<_> = [5, 99]
        .into_iter()
        .map(|kind| {
            let source = template.replace("$KIND", &kind.to_string());
            let mut frontplane = Frontplane::from_wat(&source, Limits::default()).unwrap();
            frontplane.tick(1)
        })
        .collect();

    assert!(
        outcomes.iter().all(Result::is_err),
        "outcomes: {outcomes:?}"
    );
}
