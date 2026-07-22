use std::{fs, path::PathBuf, process::Command};

use aedicule::{DISPLAY_REFRESH_RATE_ENV, package_application};

const SILENT_FLAC: &[u8] = &[
    102, 76, 97, 67, 0, 0, 0, 34, 16, 0, 16, 0, 0, 0, 15, 0, 0, 15, 1, 244, 2, 240, 0, 0, 0, 8,
    112, 188, 143, 75, 114, 168, 105, 33, 70, 139, 248, 232, 68, 29, 206, 81, 132, 0, 0, 40, 32, 0,
    0, 0, 114, 101, 102, 101, 114, 101, 110, 99, 101, 32, 108, 105, 98, 70, 76, 65, 67, 32, 49, 46,
    53, 46, 48, 32, 50, 48, 50, 53, 48, 50, 49, 49, 0, 0, 0, 0, 255, 248, 100, 24, 0, 7, 84, 0, 0,
    0, 0, 0, 0, 140, 21,
];

fn animated_wat() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/animated.wat")
}

fn render(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aedicule-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("AEDICULE_DEFAULT_APPLICATION", animated_wat())
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
            (import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
            (memory (export "memory") 1)
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) i32.const 0)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_tick") (param i32) (result i32) i32.const 0)
            (func (export "AE_render") (result i32)
                f32.const 1 f32.const 0 f32.const 0 f32.const 1 call $begin drop
                call $end drop i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 0)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1))"#,
    )
    .unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_aedicule-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("AEDICULE_DEFAULT_APPLICATION", &plugin)
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
fn cli_loads_assets_from_bare_wat_directory_and_aed_application_sources() {
    let temporary = std::env::temp_dir().join(format!(
        "aedicule render applications {}",
        std::process::id()
    ));
    let application = temporary.join("application with spaces");
    fs::create_dir_all(application.join("assets/audio")).unwrap();
    fs::write(
        application.join("code.wat"),
        r#"(module
            (import "aedicule.v0" "AE_sample_asset"
                (func $sample (param i32 i32 i32 i32) (result i32)))
            (import "aedicule.v0" "AE_frame_begin_rgba"
                (func $begin (param i32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "assets/audio/satellite-destroyed.flac")
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 1)
            (func (export "AE_configure") (result i32)
                i32.const 1 i32.const 0 i32.const 37 i32.const 0 call $sample)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_tick") (param i32) (result i32) i32.const 0)
            (func (export "AE_render") (result i32)
                i32.const 0x336699ff call $begin drop call $end drop i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 64)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1))"#,
    )
    .unwrap();
    fs::write(
        application.join("assets/audio/satellite-destroyed.flac"),
        SILENT_FLAC,
    )
    .unwrap();
    let archive = temporary.join("application with spaces.aed");
    package_application(&application, &archive).unwrap();

    for source in [application.join("code.wat"), application.clone(), archive] {
        let result = render(&[source.to_str().unwrap()]);
        assert!(
            result.status.success(),
            "{}: {:?}",
            source.display(),
            result.stderr
        );
        assert!(
            result.stderr.is_empty(),
            "{}: {:?}",
            source.display(),
            result.stderr
        );
        assert!(
            String::from_utf8(result.stdout)
                .unwrap()
                .contains("fill=\"#336699\""),
            "{}",
            source.display()
        );
    }
    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn cli_reports_help_about_and_invalid_arguments_cleanly() {
    let help = render(&["--help"]);
    let about = render(&["--about"]);
    let invalid = render(&["--ticks", "not-a-number"]);

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--output"));
    assert!(String::from_utf8_lossy(&help.stdout).contains("--control"));
    assert!(
        String::from_utf8_lossy(&help.stdout).contains(DISPLAY_REFRESH_RATE_ENV),
        "headless help documents the display-refresh override"
    );
    assert!(about.status.success());
    assert!(String::from_utf8_lossy(&about.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid --ticks"));
}

#[test]
fn invalid_display_refresh_environment_override_fails_before_guest_execution() {
    let result = Command::new(env!("CARGO_BIN_EXE_aedicule-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("AEDICULE_DEFAULT_APPLICATION", animated_wat())
        .env(DISPLAY_REFRESH_RATE_ENV, "59.")
        .output()
        .expect("render CLI should execute");

    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty(), "{:?}", result.stdout);
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains(DISPLAY_REFRESH_RATE_ENV)
    );
}

#[test]
fn display_refresh_environment_override_reaches_the_guest_lifecycle_event() {
    let directory = std::env::temp_dir().join(format!(
        "aedicule display refresh override {}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    let plugin = directory.join("display-event.wat");
    fs::write(
        &plugin,
        r#"(module
            (import "aedicule.v0" "AE_frame_begin" (func $begin (param f32 f32 f32 f32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
            (memory (export "memory") 1)
            (global $display_code (mut i32) i32.const 0)
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 0)
            (func (export "AE_configure") (result i32) i32.const 0)
            (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
            (func (export "AE_event") (param $kind i32) (param $code i32) (param f32 f32) (result i32)
                local.get $kind i32.const 9 i32.eq
                if local.get $code global.set $display_code end
                i32.const 0)
            (func (export "AE_tick") (param i32) (result i32) i32.const 0)
            (func (export "AE_render") (result i32)
                global.get $display_code i32.const 60000 i32.eq
                if
                    f32.const 0 f32.const 1 f32.const 0 f32.const 1
                    call $begin drop
                else
                    f32.const 1 f32.const 0 f32.const 0 f32.const 1
                    call $begin drop
                end
                call $end drop
                i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 0)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1))"#,
    )
    .unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_aedicule-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("AEDICULE_DEFAULT_APPLICATION", &plugin)
        .env(DISPLAY_REFRESH_RATE_ENV, "59.94")
        .output()
        .expect("render CLI should execute");

    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("fill=\"#00ff00\"")
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn headless_controls_are_ordered_after_init_and_before_ticks() {
    let directory =
        std::env::temp_dir().join(format!("aedicule headless controls {}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let plugin = directory.join("integer-control.wat");
    fs::write(
        &plugin,
        r#"(module
            (import "aedicule.v0" "AE_slider_i32" (func $slider (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
            (import "aedicule.v0" "AE_frame_begin_rgba" (func $begin (param i32) (result i32)))
            (import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "N")
            (global $value (mut i32) i32.const 0)
            (func (export "AE_abi_major") (result i32) i32.const 0)
            (func (export "AE_abi_minor") (result i32) i32.const 1)
            (func (export "AE_configure") (result i32)
                i32.const 7 i32.const 0 i32.const 1
                i32.const 0 i32.const 3600 i32.const 3 i32.const 1050
                call $slider)
            (func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32)
                i32.const 1050 global.set $value i32.const 0)
            (func (export "AE_event_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
            (func (export "AE_control_event") (param i32) (param $value i32) (param i32) (result i32)
                local.get $value global.set $value i32.const 0)
            (func (export "AE_tick") (param $ticks i32) (result i32)
                global.get $value local.get $ticks i32.add global.set $value i32.const 0)
            (func (export "AE_render") (result i32)
                global.get $value i32.const 2401 i32.eq
                if (result i32) i32.const 0x00ff00ff else i32.const 0xff0000ff end
                call $begin drop call $end drop i32.const 0)
            (func (export "AE_state_ptr") (result i32) i32.const 0)
            (func (export "AE_state_len") (result i32) i32.const 0)
            (func (export "AE_state_schema") (result i32) i32.const 1))"#,
    )
    .unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_aedicule-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .args([
            plugin.to_str().unwrap(),
            "--control",
            "7=1050",
            "--control",
            "7=2400",
            "--ticks",
            "1",
        ])
        .output()
        .unwrap();

    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stderr.is_empty(), "{:?}", result.stderr);
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("fill=\"#00ff00\"")
    );

    for invalid in ["9=2400", "7=3603", "7=2401"] {
        let result = Command::new(env!("CARGO_BIN_EXE_aedicule-render"))
            .env("MUTE_DEBUG_STATUS", "1")
            .args([plugin.to_str().unwrap(), "--control", invalid])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1), "control {invalid}");
        assert!(result.stdout.is_empty(), "control {invalid}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("AE_control_event"),
            "control {invalid}: {:?}",
            result.stderr
        );
    }
    fs::remove_dir_all(directory).unwrap();
}
