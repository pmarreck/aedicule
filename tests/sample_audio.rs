use std::{collections::BTreeMap, fs};

use gpui_wasm::{
    ApplicationAssets, FlacError, Frontplane, Limits, PluginInit, PluginSource, decode_flac,
    initialize_frontplane, package_application, prepare_reload_with_assets, wav::WavLimits,
};

const SILENT_FLAC: &[u8] = &[
    102, 76, 97, 67, 0, 0, 0, 34, 16, 0, 16, 0, 0, 0, 15, 0, 0, 15, 1, 244, 2, 240, 0, 0, 0, 8,
    112, 188, 143, 75, 114, 168, 105, 33, 70, 139, 248, 232, 68, 29, 206, 81, 132, 0, 0, 40, 32, 0,
    0, 0, 114, 101, 102, 101, 114, 101, 110, 99, 101, 32, 108, 105, 98, 70, 76, 65, 67, 32, 49, 46,
    53, 46, 48, 32, 50, 48, 50, 53, 48, 50, 49, 49, 0, 0, 0, 0, 255, 248, 100, 24, 0, 7, 84, 0, 0,
    0, 0, 0, 0, 140, 21,
];

const SAMPLE_WAT: &str = r#"
    (module
        (import "aedicule.v0" "AE_sample_asset"
            (func $sample_asset (param i32 i32 i32 i32) (result i32)))
        (import "aedicule.v0" "AE_sample_play"
            (func $sample_play (param i32 f32 f32 i32) (result i32)))
        (import "aedicule.v0" "AE_audio"
            (func $synth_play (param i32 f32 f32 i32) (result i32)))
        (import "aedicule.v0" "AE_frame_begin_rgba"
            (func $frame_begin (param i32) (result i32)))
        (import "aedicule.v0" "AE_frame_end" (func $frame_end (result i32)))
        (memory (export "memory") 1)
        (data (i32.const 0) "assets/audio/satellite-destroyed.flac")
        (func (export "AE_abi_major") (result i32) i32.const 0)
        (func (export "AE_abi_minor") (result i32) i32.const 1)
        (func (export "AE_configure") (result i32)
            i32.const 77 i32.const 0 i32.const 37 i32.const 0 call $sample_asset)
        (func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
        (func (export "AE_tick") (param $ticks i32) (result i32)
            i32.const 77 f32.const 0.5 f32.const 1.25 i32.const 0 call $sample_play drop
            local.get $ticks i32.const 99 i32.eq)
        (func (export "AE_render") (result i32)
            i32.const 0xff call $frame_begin drop
            call $frame_end drop
            i32.const 0)
        (func (export "AE_state_ptr") (result i32) i32.const 64)
        (func (export "AE_state_len") (result i32) i32.const 0)
        (func (export "AE_state_schema") (result i32) i32.const 1))
"#;

fn sample_assets() -> ApplicationAssets {
    BTreeMap::from([(
        "assets/audio/satellite-destroyed.flac".to_owned(),
        SILENT_FLAC.to_vec(),
    )])
}

#[test]
fn flac_assets_are_declared_once_and_played_as_transactional_one_way_events() {
    let mut frontplane =
        Frontplane::from_wat_with_assets(SAMPLE_WAT, Limits::default(), sample_assets()).unwrap();
    initialize_frontplane(&mut frontplane, PluginInit::new(7, 800.0, 600.0)).unwrap();

    let sample = &frontplane.metadata().sample_assets[0];
    assert_eq!(sample.id, 77);
    assert_eq!(sample.path, "assets/audio/satellite-destroyed.flac");
    assert_eq!((sample.clip.channels, sample.clip.sample_rate), (2, 8_000));
    assert_eq!(sample.clip.frames, 8);
    assert_eq!(sample.clip.samples, vec![0; 16]);

    frontplane.tick(1).unwrap();
    assert_eq!(
        frontplane.drain_sample_audio(),
        vec![gpui_wasm::SampleAudioEvent {
            id: 77,
            volume: 0.5,
            pitch: 1.25,
            flags: 0,
        }]
    );

    assert!(frontplane.tick(99).is_err());
    assert!(frontplane.drain_sample_audio().is_empty());
}

#[test]
fn sampled_audio_declarations_fail_closed_without_an_admitted_asset() {
    let mut frontplane =
        Frontplane::from_wat_with_assets(SAMPLE_WAT, Limits::default(), ApplicationAssets::new())
            .unwrap();
    let error = frontplane.configure().unwrap_err().to_string();
    assert!(error.contains("sample asset"), "{error}");
    assert!(frontplane.metadata().sample_assets.is_empty());
}

#[test]
fn bare_wat_directory_and_aed_sources_supply_the_same_declared_sample() {
    let temporary = std::env::temp_dir().join(format!(
        "aedicule-sample-application-{}",
        std::process::id()
    ));
    if temporary.exists() {
        fs::remove_dir_all(&temporary).unwrap();
    }
    let application = temporary.join("application");
    fs::create_dir_all(application.join("assets/audio")).unwrap();
    fs::write(application.join("code.wat"), SAMPLE_WAT).unwrap();
    fs::write(
        application.join("assets/audio/satellite-destroyed.flac"),
        SILENT_FLAC,
    )
    .unwrap();
    let archive = temporary.join("application.aed");
    package_application(&application, &archive).unwrap();

    for source in [
        PluginSource::File(application.join("code.wat")),
        PluginSource::Directory(application.clone()),
        PluginSource::Archive(archive),
    ] {
        let mut frontplane = Frontplane::from_application(&source, Limits::default()).unwrap();
        frontplane.configure().unwrap();
        assert_eq!(frontplane.metadata().sample_assets[0].clip.frames, 8);
    }

    fs::remove_dir_all(temporary).unwrap();
}

#[test]
fn hot_reload_candidate_receives_the_reloaded_application_asset_catalog() {
    let init = PluginInit::new(7, 800.0, 600.0);
    let mut current =
        Frontplane::from_wat_with_assets(SAMPLE_WAT, Limits::default(), sample_assets()).unwrap();
    initialize_frontplane(&mut current, init).unwrap();

    let prepared = prepare_reload_with_assets(
        &mut current,
        SAMPLE_WAT,
        Limits::default(),
        init,
        sample_assets(),
    )
    .unwrap();
    assert_eq!(prepared.frontplane.metadata().sample_assets.len(), 1);
    assert_eq!(prepared.frontplane.metadata().sample_assets[0].id, 77);
}

#[test]
fn flac_admission_classifies_policy_failures_over_a_set() {
    let cases = [
        (
            WavLimits {
                max_channels: 1,
                ..WavLimits::default()
            },
            FlacError::ChannelLimit,
        ),
        (
            WavLimits {
                max_sample_rate: 7_999,
                ..WavLimits::default()
            },
            FlacError::SampleRateLimit,
        ),
        (
            WavLimits {
                max_frames: 7,
                ..WavLimits::default()
            },
            FlacError::FrameLimit,
        ),
        (
            WavLimits {
                max_decoded_bytes: 63,
                ..WavLimits::default()
            },
            FlacError::DecodedLimit,
        ),
    ];

    assert_eq!(
        cases
            .iter()
            .map(|(limits, _)| decode_flac(SILENT_FLAC, limits).unwrap_err())
            .collect::<Vec<_>>(),
        cases
            .iter()
            .map(|(_, expected)| expected.clone())
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        decode_flac(b"not a FLAC", &WavLimits::default()),
        Err(FlacError::InvalidHeader),
    );
}

#[test]
fn synth_and_sample_events_share_one_order_independent_output_budget() {
    let wat = SAMPLE_WAT.replace(
        "local.get $ticks i32.const 99 i32.eq",
        "i32.const 123 f32.const 1 f32.const 1 i32.const 0 call $synth_play drop\n\
         i32.const 0",
    );
    let mut frontplane = Frontplane::from_wat_with_assets(
        &wat,
        Limits {
            max_audio_events: 1,
            ..Limits::default()
        },
        sample_assets(),
    )
    .unwrap();
    initialize_frontplane(&mut frontplane, PluginInit::new(7, 800.0, 600.0)).unwrap();

    let error = frontplane.tick(1).unwrap_err().to_string();
    assert!(error.contains("audio event budget"), "{error}");
    assert!(frontplane.drain_audio().is_empty());
    assert!(frontplane.drain_sample_audio().is_empty());
}
