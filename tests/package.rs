use std::{fs, path::PathBuf};

use aedicule::{
    AED_MIME_TYPE, PluginSource, depackage_application, package_application,
    read_application_assets, read_application_file,
};
use async_zip::{
    Compression, ZipEntryBuilder, base::read::mem::ZipFileReader, base::write::ZipFileWriter,
};
use futures_lite::{future::block_on, io::Cursor};

fn temporary_directory(label: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("aedicule-package-{}-{label}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn directory_packages_are_deterministic_round_trippable_and_source_equivalent() {
    let temporary = temporary_directory("round-trip");
    let application = temporary.join("Vibesteroids source");
    fs::create_dir_all(application.join("assets")).unwrap();
    fs::create_dir_all(application.join("lib")).unwrap();
    fs::create_dir_all(application.join("tests")).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    fs::write(
        application.join("assets/greta.flac"),
        [0x66, 0x4c, 0x61, 0x43],
    )
    .unwrap();
    fs::write(application.join("lib/helper.wat"), b"(module)").unwrap();
    fs::write(application.join("tests/behavior.wast"), b"(module)\n").unwrap();
    fs::write(application.join("README.md"), b"# Vibesteroids\n").unwrap();
    fs::write(application.join("LICENSE"), b"MIT\n").unwrap();
    fs::write(application.join("notes with spaces.txt"), b"arbitrary\n").unwrap();

    let first = temporary.join("first.aed");
    let second = temporary.join("second.aed");
    package_application(&application, &first).unwrap();
    package_application(&application, &second).unwrap();

    let first_bytes = fs::read(&first).unwrap();
    assert!(first_bytes.starts_with(b"PK\x03\x04"));
    assert_eq!(first_bytes, fs::read(&second).unwrap());
    block_on(async {
        let reader = ZipFileReader::new(first_bytes.clone()).await.unwrap();
        let first_entry = &reader.file().entries()[0];
        assert_eq!(first_entry.filename().as_str().unwrap(), "mimetype");
        assert_eq!(first_entry.compression(), Compression::Stored);
        let mut bytes = Vec::new();
        reader
            .reader_with_entry(0)
            .await
            .unwrap()
            .read_to_end_checked(&mut bytes)
            .await
            .unwrap();
        assert_eq!(bytes, AED_MIME_TYPE.as_bytes());
    });

    let file_source = PluginSource::File(application.join("code.wat"));
    let directory_source = PluginSource::Directory(application.clone());
    let archive_source = PluginSource::Archive(first.clone());
    assert_eq!(
        read_application_file(&file_source, "code.wat").unwrap(),
        read_application_file(&directory_source, "code.wat").unwrap(),
    );
    assert_eq!(
        read_application_file(&directory_source, "code.wat").unwrap(),
        read_application_file(&archive_source, "code.wat").unwrap(),
    );
    assert_eq!(
        read_application_file(&archive_source, "assets/greta.flac").unwrap(),
        [0x66, 0x4c, 0x61, 0x43],
    );
    let file_assets = read_application_assets(&file_source).unwrap();
    let directory_assets = read_application_assets(&directory_source).unwrap();
    let archive_assets = read_application_assets(&archive_source).unwrap();
    assert_eq!(file_assets, directory_assets);
    assert_eq!(directory_assets, archive_assets);
    assert_eq!(
        archive_assets.into_iter().collect::<Vec<_>>(),
        vec![("assets/greta.flac".to_owned(), vec![0x66, 0x4c, 0x61, 0x43],)],
    );

    let unpacked = temporary.join("unpacked");
    depackage_application(&first, &unpacked).unwrap();
    for relative in [
        "code.wat",
        "assets/greta.flac",
        "lib/helper.wat",
        "tests/behavior.wast",
        "README.md",
        "LICENSE",
        "notes with spaces.txt",
    ] {
        assert_eq!(
            fs::read(application.join(relative)).unwrap(),
            fs::read(unpacked.join(relative)).unwrap(),
            "round-trip mismatch for {relative}",
        );
    }

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn archive_requires_epub_style_aedicule_mimetype_sentinel() {
    let temporary = temporary_directory("mimetype");
    let archive = temporary.join("ordinary.zip.aed");
    let bytes = block_on(async {
        let mut writer = ZipFileWriter::new(Cursor::new(Vec::new())).force_no_zip64();
        writer
            .write_entry_whole(
                ZipEntryBuilder::new("code.wat".into(), Compression::Stored),
                b"(module)",
            )
            .await
            .unwrap();
        writer.close().await.unwrap().into_inner()
    });
    fs::write(&archive, bytes).unwrap();

    let error = read_application_file(&PluginSource::Archive(archive), "code.wat").unwrap_err();
    assert!(error.contains("mimetype"), "{error}");

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn bundled_vibesteroids_package_is_a_runnable_audio_application() {
    let package = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("demos/vibesteroids.aed");
    let source = PluginSource::Archive(package);
    let wat = String::from_utf8(read_application_file(&source, "code.wat").unwrap()).unwrap();
    assert!(wat.contains("AE_sample_asset"));
    assert!(wat.contains("AE_sample_play"));
    assert!(wat.contains("(func (export \"AE_state_schema\") (result i32) i32.const 11)"));
    assert_eq!(
        read_application_file(&source, "assets/audio/satellite-destroyed.flac")
            .unwrap()
            .len(),
        56_711,
    );
    assert!(
        read_application_file(&source, "tests/main.wast")
            .unwrap()
            .len()
            > 1_000_000,
    );
}

#[test]
fn package_paths_cannot_become_windows_drive_relative_extraction_paths() {
    let temporary = temporary_directory("windows-drive-path");
    let application = temporary.join("application");
    fs::create_dir(&application).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    fs::write(application.join("C:escape.txt"), b"must remain contained").unwrap();

    let error = package_application(&application, &temporary.join("application.aed")).unwrap_err();
    assert!(error.contains("invalid package path"), "{error}");

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn package_entries_are_compressed_rather_than_merely_stored() {
    let temporary = temporary_directory("compressed");
    let application = temporary.join("application");
    fs::create_dir_all(application.join("tests")).unwrap();
    fs::write(application.join("code.wat"), b"(module)").unwrap();
    // Highly redundant, like the WAT and WAST text that dominates a real
    // package, so a stored archive and a compressed one differ by a wide margin
    // rather than by a few bytes of framing.
    let redundant = "(func $repeated (param i32) (result i32) local.get 0)\n".repeat(4_096);
    fs::write(application.join("tests/main.wast"), redundant.as_bytes()).unwrap();

    let archive = temporary.join("application.aed");
    package_application(&application, &archive).unwrap();
    let bytes = fs::read(&archive).unwrap();

    block_on(async {
        let reader = ZipFileReader::new(bytes.clone()).await.unwrap();
        for (index, entry) in reader.file().entries().iter().enumerate() {
            let name = entry.filename().as_str().unwrap().to_owned();
            let expected = if index == 0 {
                // The sentinel stays stored so the package is identifiable by
                // its leading bytes the way EPUB and ODF are.
                Compression::Stored
            } else {
                Compression::Zstd
            };
            assert_eq!(entry.compression(), expected, "compression for {name}");
        }
    });

    assert!(
        (bytes.len() as u64) < redundant.len() as u64 / 8,
        "package of {} bytes did not compress {} bytes of redundant source",
        bytes.len(),
        redundant.len(),
    );

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn a_compressed_entry_larger_than_the_cap_is_refused_despite_a_small_package() {
    let temporary = temporary_directory("oversize-entry");
    let archive = temporary.join("oversize.aed");
    // Compression makes an entry's packaged size say nothing about the memory it
    // demands on the way out, which stored entries never allowed. Seventeen
    // megabytes of zeros occupy a trivial archive and must still be refused.
    let oversize = vec![0_u8; 17 * 1024 * 1024];
    let bytes = block_on(async {
        let mut writer = ZipFileWriter::new(Cursor::new(Vec::new())).force_no_zip64();
        writer
            .write_entry_whole(
                ZipEntryBuilder::new("mimetype".into(), Compression::Stored),
                AED_MIME_TYPE.as_bytes(),
            )
            .await
            .unwrap();
        writer
            .write_entry_whole(
                ZipEntryBuilder::new("code.wat".into(), Compression::Zstd),
                &oversize,
            )
            .await
            .unwrap();
        writer.close().await.unwrap().into_inner()
    });
    assert!(
        bytes.len() < 1024 * 1024,
        "expected a small archive, got {} bytes",
        bytes.len(),
    );
    fs::write(&archive, bytes).unwrap();

    let error = read_application_file(&PluginSource::Archive(archive), "code.wat").unwrap_err();
    assert!(error.contains("exceeds"), "{error}");

    fs::remove_dir_all(temporary).unwrap();
}
