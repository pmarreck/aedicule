use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use aedicule::{
    FALLBACK_WAT, FileRevision, Frontplane, LaunchAction, Limits, PluginSource, RevisionTracker,
    WatRejectionStage, file_url_to_path, format_wat_rejection_diagnostic,
    resolve_launch as resolve_launch_with_default,
};

fn resolve_launch(
    arguments: impl IntoIterator<Item = OsString>,
    working_directory: &Path,
) -> Result<LaunchAction, String> {
    resolve_launch_with_default(arguments, working_directory, None)
}

fn arguments<'a>(values: &'a [&'a str]) -> impl Iterator<Item = OsString> + 'a {
    values.iter().map(OsString::from)
}

fn temporary_directory(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("aedicule-launch-{}-{label}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn default_source_prefers_colocated_then_packaged_then_launcher() {
    let directory = temporary_directory("default");
    let packaged = directory.join("installed/application.wat");

    assert_eq!(
        resolve_launch(arguments(&[]), &directory).unwrap(),
        LaunchAction::Launcher { seed: None }
    );

    assert_eq!(
        resolve_launch_with_default(arguments(&[]), &directory, Some(&packaged)).unwrap(),
        LaunchAction::Run {
            source: PluginSource::File(packaged.clone()),
            watch: false,
            seed: None,
        }
    );

    fs::write(directory.join("code.wat"), "(module)").unwrap();
    assert_eq!(
        resolve_launch_with_default(arguments(&[]), &directory, Some(&packaged)).unwrap(),
        LaunchAction::Run {
            source: PluginSource::File(directory.join("code.wat")),
            watch: false,
            seed: None,
        }
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn embedded_fallback_is_a_stable_generic_conformance_application() {
    let mut frontplane = Frontplane::from_wat(FALLBACK_WAT, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(1, 320.0, 240.0).unwrap();

    assert_eq!(
        frontplane.metadata().title,
        "GPUI–WASM Conformance Fallback"
    );
    assert!(frontplane.render().is_ok());
}

#[test]
fn explicit_file_directory_watch_and_embedded_modes_resolve_predictably() {
    let directory = temporary_directory("explicit");
    let app_directory = directory.join("application with spaces");
    fs::create_dir_all(&app_directory).unwrap();
    let file = directory.join("custom game.wat");

    assert_eq!(
        resolve_launch(arguments(&["--watch", file.to_str().unwrap()]), &directory).unwrap(),
        LaunchAction::Run {
            source: PluginSource::File(file),
            watch: true,
            seed: None,
        }
    );
    assert_eq!(
        resolve_launch(arguments(&[app_directory.to_str().unwrap()]), &directory).unwrap(),
        LaunchAction::Run {
            source: PluginSource::Directory(app_directory),
            watch: false,
            seed: None,
        }
    );
    assert_eq!(
        resolve_launch(arguments(&["--watch", "--embedded"]), &directory).unwrap(),
        LaunchAction::Run {
            source: PluginSource::Embedded,
            watch: false,
            seed: None,
        }
    );
    assert_eq!(
        resolve_launch(arguments(&["--embedded", "--watch"]), &directory).unwrap(),
        LaunchAction::Run {
            source: PluginSource::File(directory.join("code.wat")),
            watch: true,
            seed: None,
        }
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn package_depackage_and_web_actions_derive_predictable_paths() {
    let directory = temporary_directory("package-actions");
    let app = directory.join("Vibesteroids source");
    let package = directory.join("Vibesteroids source.aed");

    assert_eq!(
        resolve_launch(arguments(&["--package", app.to_str().unwrap()]), &directory,).unwrap(),
        LaunchAction::Package {
            source: app.clone(),
            output: package.clone(),
        }
    );
    assert_eq!(
        resolve_launch(
            arguments(&["--depackage", package.to_str().unwrap()]),
            &directory,
        )
        .unwrap(),
        LaunchAction::Depackage {
            source: package.clone(),
            output: app.clone(),
        }
    );
    assert_eq!(
        resolve_launch(
            arguments(&[
                "--web",
                package.to_str().unwrap(),
                "--port",
                "0",
                "--bind",
                "127.0.0.1",
            ]),
            &directory,
        )
        .unwrap(),
        LaunchAction::Web {
            source: PluginSource::Archive(package),
            bind: "127.0.0.1".to_owned(),
            port: 0,
        }
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn test_action_accepts_bare_wat_directory_and_archive_sources() {
    let temporary = temporary_directory("test-action");
    let application = temporary.join("application");
    fs::create_dir_all(&application).unwrap();
    let wat = application.join("code.wat");
    fs::write(&wat, b"(module)").unwrap();
    let archive = temporary.join("application.aed");

    assert_eq!(
        resolve_launch(
            arguments(&["--test", "--seed", "1", wat.to_str().unwrap()]),
            &temporary,
        )
        .unwrap(),
        LaunchAction::Test {
            source: PluginSource::File(wat),
            seed: Some(1),
        }
    );
    assert_eq!(
        resolve_launch(
            arguments(&["--test", application.to_str().unwrap()]),
            &temporary
        )
        .unwrap(),
        LaunchAction::Test {
            source: PluginSource::Directory(application),
            seed: None,
        }
    );
    assert_eq!(
        resolve_launch(
            arguments(&["--test", archive.to_str().unwrap()]),
            &temporary
        )
        .unwrap(),
        LaunchAction::Test {
            source: PluginSource::Archive(archive),
            seed: None,
        }
    );

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn platform_document_urls_decode_to_the_same_wat_path_as_cli_launches() {
    assert_eq!(
        file_url_to_path("file:///Applications/Ulam%20Flower/code.wat"),
        Some(PathBuf::from("/Applications/Ulam Flower/code.wat"))
    );
    assert_eq!(
        file_url_to_path("file://localhost/tmp/100%25-fun.wat"),
        Some(PathBuf::from("/tmp/100%-fun.wat"))
    );
    assert_eq!(file_url_to_path("https://example.test/code.wat"), None);
    assert_eq!(file_url_to_path("file://remote-host/tmp/code.wat"), None);
    assert_eq!(file_url_to_path("file:///tmp/broken%2.wat"), None);
    assert_eq!(file_url_to_path("file:///tmp/not-utf8-%FF.wat"), None);
}

#[test]
fn deterministic_seed_is_validated_and_later_values_override_earlier_ones() {
    let directory = temporary_directory("seed");

    assert_eq!(
        resolve_launch(
            arguments(&["--seed", "42", "--seed", "99", "game.wat"]),
            &directory,
        )
        .unwrap(),
        LaunchAction::Run {
            source: PluginSource::File(directory.join("game.wat")),
            watch: false,
            seed: Some(99),
        }
    );
    assert_eq!(
        resolve_launch(arguments(&["--seed", "0x5eedcafe"]), &directory).unwrap(),
        LaunchAction::Launcher {
            seed: Some(0x5eed_cafe),
        }
    );
    assert!(resolve_launch(arguments(&["--seed"]), &directory).is_err());
    assert!(resolve_launch(arguments(&["--seed", "not-a-number"]), &directory).is_err());
    assert!(resolve_launch(arguments(&["--seed", "-1"]), &directory).is_err());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn help_about_unknown_options_and_extra_paths_are_classified() {
    let directory = temporary_directory("actions");

    assert_eq!(
        resolve_launch(arguments(&["--help"]), &directory).unwrap(),
        LaunchAction::Help
    );
    assert_eq!(
        resolve_launch(arguments(&["--about"]), &directory).unwrap(),
        LaunchAction::About
    );
    assert!(resolve_launch(arguments(&["--wat"]), &directory).is_err());
    assert!(resolve_launch(arguments(&["one.wat", "two.wat"]), &directory).is_err());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn revision_tracker_reports_each_distinct_editor_save_once() {
    let mut tracker = RevisionTracker::default();
    let revisions = [
        FileRevision::Missing,
        FileRevision::Missing,
        FileRevision::Content(b"first".to_vec()),
        FileRevision::Content(b"first".to_vec()),
        FileRevision::Content(b"second".to_vec()),
        FileRevision::Unreadable("permission denied".into()),
        FileRevision::Unreadable("permission denied".into()),
        FileRevision::Content(b"fixed".to_vec()),
    ];

    let changed: Vec<_> = revisions
        .into_iter()
        .filter(|revision| tracker.observe(revision))
        .collect();

    assert_eq!(
        changed,
        vec![
            FileRevision::Missing,
            FileRevision::Content(b"first".to_vec()),
            FileRevision::Content(b"second".to_vec()),
            FileRevision::Unreadable("permission denied".into()),
            FileRevision::Content(b"fixed".to_vec()),
        ]
    );
}

#[test]
fn wat_rejections_have_searchable_stage_path_error_and_survivor_diagnostics() {
    let path = Path::new("/applications/example/code.wat");
    let cases = [
        (
            WatRejectionStage::InitialLoad,
            "invalid frame lifecycle: unsupported synth filter",
            "AEDICULE_WAT_REJECTED: initial-load: /applications/example/code.wat: invalid frame lifecycle: unsupported synth filter: embedded fallback remains active",
        ),
        (
            WatRejectionStage::Reload,
            "invalid frame lifecycle: unsupported synth filter",
            "AEDICULE_WAT_REJECTED: reload: /applications/example/code.wat: invalid frame lifecycle: unsupported synth filter: previous plugin remains active",
        ),
    ];

    let actual: Vec<_> = cases
        .iter()
        .map(|(stage, error, _)| format_wat_rejection_diagnostic(path, *stage, error))
        .collect();
    let expected: Vec<_> = cases
        .iter()
        .map(|(_, _, expected)| expected.to_string())
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn reading_by_path_observes_editor_style_atomic_replacement() {
    let directory = temporary_directory("atomic-save");
    let path = directory.join("code.wat");
    let replacement = directory.join("code.wat.new");
    fs::write(&path, "first").unwrap();
    assert_eq!(
        FileRevision::read(&path),
        FileRevision::Content(b"first".to_vec())
    );

    fs::write(&replacement, "second").unwrap();
    fs::rename(&replacement, &path).unwrap();

    assert_eq!(
        FileRevision::read(&path),
        FileRevision::Content(b"second".to_vec())
    );
    fs::remove_dir_all(directory).unwrap();
}
