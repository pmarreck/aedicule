use std::{fs, process::Command};

const ASTEROIDS_WAT: &str = include_str!("../plugins/vibesteroids.wat");

fn render(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gpui-wasm-render"))
        .env("MUTE_DEBUG_STATUS", "1")
        .args(arguments)
        .output()
        .expect("render CLI should execute")
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
    fs::write(&plugin, ASTEROIDS_WAT).unwrap();

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
    assert!(
        fs::read_to_string(&output)
            .unwrap()
            .contains("VIBESTEROIDS")
    );
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
