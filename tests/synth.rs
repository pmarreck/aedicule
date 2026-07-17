use gpui_wasm::{Frontplane, Limits, SynthFilter, SynthVoice, SynthWaveform};

const DECLARED_SYNTH_WAT: &str = r#"
(module
	(import "host.v0" "frame_begin" (func $frame_begin (param f32 f32 f32 f32) (result i32)))
	(import "host.v0" "frame_end" (func $frame_end (result i32)))
	(import "host.v0" "synth_voice"
		(func $synth_voice
			(param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
			(result i32)))
	(memory (export "memory") 1 1)
	(func (export "fp_abi_major") (result i32) i32.const 0)
	(func (export "fp_abi_minor") (result i32) i32.const 0)
	(func (export "fp_configure") (result i32)
		i32.const 42  ;; program
		i32.const 2   ;; saw
		i32.const 150 ;; delay ms
		i32.const 400 ;; duration ms
		i32.const 523250 i32.const 588656 i32.const 1046500 ;; frequency millihertz
		i32.const 0 i32.const 300000 i32.const 1000       ;; gain ppm
		i32.const 1 i32.const 1569750 i32.const 1569750  ;; low-pass millihertz
		i32.const 55 ;; cooldown ms
		call $synth_voice drop
		i32.const 0)
	(func (export "fp_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "fp_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "fp_tick") (param i32) (result i32) i32.const 0)
	(func (export "fp_render") (result i32)
		f32.const 0 f32.const 0 f32.const 0 f32.const 1 call $frame_begin drop
		call $frame_end drop i32.const 0)
	(func (export "fp_state_ptr") (result i32) i32.const 0)
	(func (export "fp_state_len") (result i32) i32.const 0)
	(func (export "fp_state_schema") (result i32) i32.const 1))
"#;

#[test]
fn guest_declares_composable_synth_voices_without_application_semantics() {
    let mut frontplane =
        Frontplane::from_wat(DECLARED_SYNTH_WAT, Limits::default()).expect("fixture compiles");
    frontplane.configure().expect("valid voice is accepted");

    assert_eq!(
        frontplane.metadata().synth_voices,
        vec![SynthVoice {
            program_id: 42,
            waveform: SynthWaveform::Saw,
            delay_ms: 150,
            duration_ms: 400,
            frequency_start_millihz: 523_250,
            frequency_mid_millihz: 588_656,
            frequency_end_millihz: 1_046_500,
            gain_start_ppm: 0,
            gain_peak_ppm: 300_000,
            gain_end_ppm: 1_000,
            filter: SynthFilter::LowPass,
            filter_start_millihz: 1_569_750,
            filter_end_millihz: 1_569_750,
            cooldown_ms: 55,
        }]
    );
}

#[test]
fn invalid_synth_voice_fields_are_rejected_as_a_set() {
    let cases = [
        (
            "i32.const 2   ;; saw",
            "i32.const 9   ;; unsupported waveform",
        ),
        (
            "i32.const 400 ;; duration ms",
            "i32.const 0 ;; zero duration",
        ),
        (
            "i32.const 0 i32.const 300000 i32.const 1000       ;; gain ppm",
            "i32.const 0 i32.const 1000001 i32.const 1000       ;; excessive gain",
        ),
        (
            "i32.const 1 i32.const 1569750 i32.const 1569750  ;; low-pass millihertz",
            "i32.const 9 i32.const 1569750 i32.const 1569750  ;; unsupported filter",
        ),
    ];

    for (needle, replacement) in cases {
        let source = DECLARED_SYNTH_WAT.replace(needle, replacement);
        assert_ne!(source, DECLARED_SYNTH_WAT, "fixture mutation must apply");
        let mut frontplane =
            Frontplane::from_wat(&source, Limits::default()).expect("fixture still compiles");
        assert!(
            frontplane.configure().is_err(),
            "invalid classifier member was accepted: {replacement}"
        );
    }
}
