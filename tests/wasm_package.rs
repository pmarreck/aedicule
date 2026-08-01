//! Runs `.aed` compression and decompression inside a real wasm module.
//!
//! Nothing else executes this path: the CLI tests run native code, and the
//! browser currently unpacks archives in JavaScript, so a Zstandard decoder
//! that compiled for wasm but did not *work* there would ship undetected.
#![cfg(target_family = "wasm")]

use aedicule::{AED_MIME_TYPE, read_application_archive};
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
