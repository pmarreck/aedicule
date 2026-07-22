#![cfg(feature = "native-runtime")]

use std::{fs, path::PathBuf};

use aedicule::{PluginSource, package_application, run_application_tests};

fn temporary_directory(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "aedicule-application-tests-{}-{label}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    path
}

fn passing_wast() -> &'static [u8] {
    br#"(module $math
        (func (export "double") (param i32) (result i32)
            local.get 0 i32.const 2 i32.mul))
    (assert_return (invoke $math "double" (i32.const 21)) (i32.const 42))
"#
}

#[test]
fn canonical_main_wast_runs_equivalently_from_bare_wat_directory_and_archive() {
    let temporary = temporary_directory("equivalent");
    let application = temporary.join("application with spaces");
    fs::create_dir_all(application.join("tests/fragments")).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    fs::write(application.join("tests/alpha.wast"), passing_wast()).unwrap();
    fs::write(application.join("tests/main.wast"), passing_wast()).unwrap();
    fs::write(application.join("tests/zeta.wast"), passing_wast()).unwrap();
    fs::write(
        application.join("tests/fragments/not-standalone.wast"),
        b"(assert_return (invoke $registered_elsewhere \"probe\"))\n",
    )
    .unwrap();
    let archive = temporary.join("application.aed");
    package_application(&application, &archive).unwrap();

    for source in [
        PluginSource::File(application.join("code.wat")),
        PluginSource::Directory(application.clone()),
        PluginSource::Archive(archive),
    ] {
        let report = run_application_tests(&source, 1).unwrap();
        assert_eq!(report.seed, 1);
        // This exact PCG32/Fisher-Yates order was independently obtained from
        // Peter's `random --deterministic --seed 1 --shuffle` implementation.
        assert_eq!(
            report.passed,
            vec!["tests/main.wast", "tests/zeta.wast", "tests/alpha.wast",]
        );
    }

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn missing_or_failing_main_wast_is_never_reported_as_success() {
    let temporary = temporary_directory("failure");
    let application = temporary.join("application");
    fs::create_dir_all(application.join("tests")).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    fs::write(application.join("tests/behavior.wast"), passing_wast()).unwrap();

    let missing = run_application_tests(&PluginSource::Directory(application.clone()), 1)
        .unwrap_err()
        .to_string();
    assert!(missing.contains("tests/main.wast"), "{missing}");

    fs::write(
        application.join("tests/main.wast"),
        br#"(module $math (func (export "answer") (result i32) i32.const 41))
        (assert_return (invoke $math "answer") (i32.const 42))"#,
    )
    .unwrap();
    let failed = run_application_tests(&PluginSource::Directory(application), 1)
        .unwrap_err()
        .to_string();
    assert!(failed.contains("tests/main.wast"), "{failed}");
    assert!(failed.contains("expected"), "{failed}");

    fs::remove_dir_all(temporary).unwrap();
}
