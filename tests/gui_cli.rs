#![cfg(feature = "gui")]

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Command, Stdio},
};

use gpui_wasm::{DISPLAY_REFRESH_RATE_ENV, package_application};

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
    assert!(help.contains("--package"));
    assert!(help.contains("--depackage"));
    assert!(help.contains("--test"));
    assert!(help.contains("--web"));
    assert!(help.contains("--port"));
    assert!(help.contains("./code.wat"));
    assert!(help.contains(DISPLAY_REFRESH_RATE_ENV));

    assert!(about.status.success(), "{:?}", about.stderr);
    assert!(about.stderr.is_empty(), "{:?}", about.stderr);
    assert!(
        String::from_utf8(about.stdout)
            .unwrap()
            .contains(env!("CARGO_PKG_VERSION"))
    );
}

fn temporary_directory(label: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("aedicule-gui-cli-{}-{label}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn native_cli_packages_and_depackages_paths_with_spaces_without_opening_gpui() {
    let temporary = temporary_directory("package");
    let application = temporary.join("application source");
    let archive = temporary.join("delivery package.aed");
    let unpacked = temporary.join("unpacked application");
    fs::create_dir_all(application.join("assets")).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    fs::write(application.join("assets/sample.flac"), b"fLaC").unwrap();

    let package = run(&[
        "--package",
        application.to_str().unwrap(),
        archive.to_str().unwrap(),
    ]);
    assert!(package.status.success(), "{:?}", package.stderr);
    assert!(package.stderr.is_empty(), "{:?}", package.stderr);
    assert_eq!(
        String::from_utf8(package.stdout).unwrap(),
        format!("{}\n", archive.display())
    );

    let depackage = run(&[
        "--depackage",
        archive.to_str().unwrap(),
        unpacked.to_str().unwrap(),
    ]);
    assert!(depackage.status.success(), "{:?}", depackage.stderr);
    assert!(depackage.stderr.is_empty(), "{:?}", depackage.stderr);
    assert_eq!(
        String::from_utf8(depackage.stdout).unwrap(),
        format!("{}\n", unpacked.display())
    );
    assert_eq!(fs::read(unpacked.join("code.wat")).unwrap(), b"(module)");
    assert_eq!(
        fs::read(unpacked.join("assets/sample.flac")).unwrap(),
        b"fLaC"
    );

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn native_cli_runs_application_wast_suites_with_an_emitted_replay_seed() {
    let temporary = temporary_directory("test");
    let application = temporary.join("tested application");
    fs::create_dir_all(application.join("tests/fragments")).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    let suite = br#"(module $probe (func (export "answer") (result i32) i32.const 42))
        (assert_return (invoke $probe "answer") (i32.const 42))"#;
    fs::write(application.join("tests/alpha.wast"), suite).unwrap();
    fs::write(application.join("tests/main.wast"), suite).unwrap();
    fs::write(
        application.join("tests/fragments/helper.wast"),
        b"(assert_return (invoke $not_standalone \"probe\"))",
    )
    .unwrap();

    let result = run(&["--test", "--seed", "1", application.to_str().unwrap()]);

    assert!(result.status.success(), "{:?}", result.stderr);
    assert!(result.stderr.is_empty(), "{:?}", result.stderr);
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "test seed: 0x0000000000000001\n\
PASS tests/alpha.wast\n\
PASS tests/main.wast\n\
2 passed\n"
    );

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn native_cli_serves_an_aed_through_the_bundled_web_runtime() {
    let temporary = temporary_directory("web");
    let application = temporary.join("web application");
    let runtime = temporary.join("bundled web runtime");
    fs::create_dir_all(&application).unwrap();
    fs::create_dir_all(&runtime).unwrap();
    fs::write(application.join("code.wat"), b"(module $served)").unwrap();
    for (name, bytes) in [
        (
            "index.html",
            b"<!doctype html><title>Aedicule test</title>".as_slice(),
        ),
        ("bootstrap.js", b"console.info('bootstrap')".as_slice()),
        ("coi-serviceworker.js", b"// service worker".as_slice()),
        ("manifest.webmanifest", b"{}".as_slice()),
        ("icon.png", b"PNG".as_slice()),
        (
            "aedicule_web.js",
            b"export default function() {}".as_slice(),
        ),
        ("aedicule_web_bg.wasm", b"\0asm".as_slice()),
    ] {
        fs::write(runtime.join(name), bytes).unwrap();
    }
    let archive = temporary.join("web application.aed");
    package_application(&application, &archive).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_gpui-wasm"))
        .env("MUTE_DEBUG_STATUS", "1")
        .env("AEDICULE_WEB_RUNTIME", &runtime)
        .args(["--web", "--port", "0", archive.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut url = String::new();
    stdout.read_line(&mut url).unwrap();
    let url = url.trim();
    assert!(url.starts_with("http://127.0.0.1:"), "{url:?}");
    let address = url.strip_prefix("http://").unwrap().trim_end_matches('/');

    let mut connection = TcpStream::connect(address).unwrap();
    connection
        .write_all(b"GET /code.wat HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    connection.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"), "{response}");
    assert!(response.contains("Cross-Origin-Opener-Policy: same-origin\r\n"));
    assert!(response.contains("Cross-Origin-Embedder-Policy: require-corp\r\n"));
    assert!(response.ends_with("(module $served)"), "{response}");

    child.kill().unwrap();
    let status = child.wait().unwrap();
    assert!(!status.success());
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(stderr.is_empty(), "{stderr}");
    fs::remove_dir_all(temporary).unwrap();
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
