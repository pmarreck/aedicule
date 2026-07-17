use std::{fs, path::PathBuf, process::Command};

fn animated_wat() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/animated.wat")
}

fn render(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gpui-wasm-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("GPUI_WASM_DEFAULT_PLUGIN", animated_wat())
        .args(arguments)
        .output()
        .expect("render CLI should execute")
}

#[test]
fn default_plugin_environment_selects_runtime_data_instead_of_compiled_application() {
    let directory =
        std::env::temp_dir().join(format!("gpui wasm runtime default {}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let plugin = directory.join("runtime default.wat");
    fs::write(
        &plugin,
        r#"(module
            (import "host.v0" "frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
            (import "host.v0" "frame_end" (func $end (result i32)))
            (memory (export "memory") 1)
            (func (export "fp_abi_major") (result i32) i32.const 0)
            (func (export "fp_abi_minor") (result i32) i32.const 0)
            (func (export "fp_configure") (result i32) i32.const 0)
            (func (export "fp_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "fp_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "fp_tick") (param i32) (result i32) i32.const 0)
            (func (export "fp_render") (result i32)
                f32.const 1 f32.const 0 f32.const 0 f32.const 1 call $begin drop
                call $end drop i32.const 0)
            (func (export "fp_state_ptr") (result i32) i32.const 0)
            (func (export "fp_state_len") (result i32) i32.const 0)
            (func (export "fp_state_schema") (result i32) i32.const 1))"#,
    )
    .unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_gpui-wasm-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("GPUI_WASM_DEFAULT_PLUGIN", &plugin)
        .output()
        .unwrap();

    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("fill=\"#ff0000\"")
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_renders_distinct_requested_frames_to_stdout() {
    let before = render(&["--ticks", "0", "--output", "-"]);
    let after = render(&["--ticks", "300", "--output", "@stdout"]);

    assert!(before.status.success(), "{:?}", before.stderr);
    assert!(after.status.success(), "{:?}", after.stderr);
    assert!(before.stderr.is_empty(), "{:?}", before.stderr);
    assert!(after.stderr.is_empty(), "{:?}", after.stderr);
    assert!(before.stdout.starts_with(b"<svg "));
    assert_ne!(before.stdout, after.stdout);
}

#[test]
fn cli_accepts_a_plugin_path_with_spaces_and_writes_the_requested_file() {
    let directory =
        std::env::temp_dir().join(format!("gpui wasm render cli {}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let plugin = directory.join("plugin with spaces.wat");
    let output = directory.join("frame with spaces.svg");
    fs::copy(animated_wat(), &plugin).unwrap();

    let result = render(&[
        plugin.to_str().unwrap(),
        "--ticks",
        "12",
        "--output",
        output.to_str().unwrap(),
    ]);

    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stdout.is_empty(), "{:?}", result.stdout);
    assert!(result.stderr.is_empty(), "{:?}", result.stderr);
    assert!(fs::read_to_string(&output).unwrap().contains("<circle"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_reports_help_about_and_invalid_arguments_cleanly() {
    let help = render(&["--help"]);
    let about = render(&["--about"]);
    let invalid = render(&["--ticks", "not-a-number"]);

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--output"));
    assert!(about.status.success());
    assert!(String::from_utf8_lossy(&about.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid --ticks"));
}
