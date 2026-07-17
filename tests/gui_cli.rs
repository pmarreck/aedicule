#![cfg(feature = "gui")]

use std::process::Command;

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gpui-wasm"))
        .env("MUTE_DEBUG_STATUS", "1")
        .args(arguments)
        .output()
        .expect("native frontplane CLI should execute")
}

#[test]
fn native_frontplane_reports_live_source_options_without_opening_a_window() {
    let help = run(&["--help"]);
    let about = run(&["--about"]);

    assert!(help.status.success(), "{:?}", help.stderr);
    assert!(help.stderr.is_empty(), "{:?}", help.stderr);
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("--watch"));
    assert!(help.contains("--embedded"));
    assert!(help.contains("--seed"));
    assert!(help.contains("./code.wat"));

    assert!(about.status.success(), "{:?}", about.stderr);
    assert!(about.stderr.is_empty(), "{:?}", about.stderr);
    assert!(
        String::from_utf8(about.stdout)
            .unwrap()
            .contains(env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_native_frontplane_options_fail_cleanly() {
    let output = run(&["--wat"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unknown option: --wat")
    );
}
