//! Runs `.aed` compression and decompression inside a real wasm module.
//!
//! Nothing else executes this path: the CLI tests run native code, and the
//! browser currently unpacks archives in JavaScript, so a Zstandard decoder
//! that compiled for wasm but did not *work* there would ship undetected.
#![cfg(target_family = "wasm")]

use aedicule::{AED_MIME_TYPE, Frontplane, Limits, read_application_archive};
use async_zip::{Compression, DeflateOption, ZipEntryBuilder, base::write::ZipFileWriter};
use futures_lite::{future::block_on, io::Cursor};
use wasm_bindgen_test::wasm_bindgen_test;

fn compressed_archive(name: &str, contents: &[u8]) -> Vec<u8> {
    block_on(async {
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
                ZipEntryBuilder::new(name.into(), Compression::Zstd)
                    .deflate_option(DeflateOption::Other(19)),
                contents,
            )
            .await
            .unwrap();
        writer.close().await.unwrap().into_inner()
    })
}

#[wasm_bindgen_test]
fn a_zstandard_entry_round_trips_through_wasm() {
    // Redundant enough that a failure to decompress cannot coincidentally
    // produce the original bytes, and large enough to cross block boundaries.
    let contents = "(func $repeated (param i32) (result i32) local.get 0)\n"
        .repeat(2_048)
        .into_bytes();
    let archive = compressed_archive("code.wat", &contents);
    assert!(
        archive.len() < contents.len() / 4,
        "expected compression: {} bytes for {}",
        archive.len(),
        contents.len(),
    );

    let entries = read_application_archive(archive).expect("read compressed package");
    assert_eq!(
        entries.get("code.wat").map(Vec::as_slice),
        Some(contents.as_slice()),
    );
}

#[wasm_bindgen_test]
fn a_browser_package_expands_to_guest_and_assets_in_wasm() {
    // The exact call the browser adapter makes when a visitor picks a `.aed`,
    // so this proves the whole selected-package path executes under wasm, not
    // merely the decompressor beneath it.
    let contents = "(module (func (export \"AE_abi_major\") (result i32) i32.const 0))\n";
    let mut writer_input: Vec<(&str, Vec<u8>)> = Vec::new();
    writer_input.push(("code.wat", contents.as_bytes().to_vec()));
    let archive = block_on(async {
        let mut writer = ZipFileWriter::new(Cursor::new(Vec::new())).force_no_zip64();
        writer
            .write_entry_whole(
                ZipEntryBuilder::new("mimetype".into(), Compression::Stored),
                AED_MIME_TYPE.as_bytes(),
            )
            .await
            .unwrap();
        for (name, bytes) in &writer_input {
            writer
                .write_entry_whole(
                    ZipEntryBuilder::new((*name).into(), Compression::Zstd)
                        .deflate_option(DeflateOption::Other(19)),
                    bytes,
                )
                .await
                .unwrap();
        }
        writer.close().await.unwrap().into_inner()
    });

    let (wat, assets) =
        aedicule::web::browser_application_from_package(archive).expect("expand browser package");
    assert_eq!(wat, contents);
    assert!(assets.is_empty());
}

#[wasm_bindgen_test]
fn random_v1_executes_inside_the_real_browser_wasm_runtime() {
    let wat = r#"(module
        (import "aedicule.v0" "AE_random_v1_seed_u64"
            (func $seed (param i32 i32 i32) (result i32)))
        (import "aedicule.v0" "AE_random_v1_fill"
            (func $fill (param i32 i32 i32) (result i32)))
        (memory (export "memory") 1)
        (func (export "AE_abi_major") (result i32) i32.const 0)
        (func (export "AE_abi_minor") (result i32) i32.const 9)
        (func (export "AE_configure") (result i32) i32.const 0)
        (func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32)
            i32.const 0 i32.const 42 i32.const 0 call $seed drop
            i32.const 0 i32.const 48 i32.const 16 call $fill)
        (func (export "AE_event_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
        (func (export "AE_tick") (param i32) (result i32) i32.const 0)
        (func (export "AE_render") (result i32) i32.const 0)
        (func (export "AE_state_ptr") (result i32) i32.const 0)
        (func (export "AE_state_len") (result i32) i32.const 64)
        (func (export "AE_state_schema") (result i32) i32.const 1))"#;
    let mut frontplane = Frontplane::from_wat(wat, Limits::default()).unwrap();
    frontplane.configure().unwrap();
    frontplane.init(42, 320.0, 240.0).unwrap();
    assert_eq!(
        &frontplane.snapshot().unwrap().bytes[48..64],
        &[
            0x69, 0xdf, 0xe2, 0xe9, 0xb5, 0x79, 0xcf, 0x6d, 0xfe, 0x3d, 0x71, 0xb1, 0x10, 0x24,
            0xdb, 0x6e,
        ]
    );
}
