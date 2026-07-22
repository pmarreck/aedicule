use aedicule::wav::{AUDIO_FIXED_SCALE, WavError, WavLimits, decode_wav_pcm};

const MONO_PCM16: &[u8] = &[
    b'R', b'I', b'F', b'F', 42, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0, 0,
    0, 1, 0, 1, 0, 0x44, 0xac, 0, 0, 0x88, 0x58, 1, 0, 2, 0, 16, 0, b'd', b'a', b't', b'a', 6, 0,
    0, 0, 0, 0x80, 0, 0, 0xff, 0x7f,
];

const STEREO_PCM8: &[u8] = &[
    b'R', b'I', b'F', b'F', 40, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0, 0,
    0, 1, 0, 2, 0, 0x40, 0x1f, 0, 0, 0x80, 0x3e, 0, 0, 2, 0, 8, 0, b'd', b'a', b't', b'a', 4, 0, 0,
    0, 0, 255, 128, 128,
];

const MONO_PCM24: &[u8] = &[
    b'R', b'I', b'F', b'F', 42, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0, 0,
    0, 1, 0, 1, 0, 1, 0, 0, 0, 3, 0, 0, 0, 3, 0, 24, 0, b'd', b'a', b't', b'a', 6, 0, 0, 0, 0, 0,
    0x80, 0xff, 0xff, 0x7f,
];

const MONO_PCM32: &[u8] = &[
    b'R', b'I', b'F', b'F', 44, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0, 0,
    0, 1, 0, 1, 0, 1, 0, 0, 0, 4, 0, 0, 0, 4, 0, 32, 0, b'd', b'a', b't', b'a', 8, 0, 0, 0, 0, 0,
    0, 0x80, 0xff, 0xff, 0xff, 0x7f,
];

#[test]
fn decodes_pcm_into_bounded_canonical_fixed_point_samples() {
    let clip = decode_wav_pcm(MONO_PCM16, &WavLimits::default()).expect("valid PCM WAVE");

    assert_eq!(clip.channels, 1);
    assert_eq!(clip.sample_rate, 44_100);
    assert_eq!(clip.frames, 3);
    assert_eq!(clip.samples, vec![-AUDIO_FIXED_SCALE, 0, AUDIO_FIXED_SCALE]);
}

#[test]
fn preserves_interleaved_stereo_samples_without_a_device_adapter() {
    let clip = decode_wav_pcm(STEREO_PCM8, &WavLimits::default()).expect("valid PCM WAVE");

    assert_eq!(clip.channels, 2);
    assert_eq!(clip.sample_rate, 8_000);
    assert_eq!(clip.frames, 2);
    assert_eq!(
        clip.samples,
        vec![-AUDIO_FIXED_SCALE, AUDIO_FIXED_SCALE, 0, 0,]
    );
}

#[test]
fn preserves_full_scale_signed_extrema_at_every_wide_pcm_depth() {
    for fixture in [MONO_PCM24, MONO_PCM32] {
        let clip = decode_wav_pcm(fixture, &WavLimits::default()).expect("valid PCM WAVE");
        assert_eq!(clip.samples, vec![-AUDIO_FIXED_SCALE, AUDIO_FIXED_SCALE]);
    }
}

#[test]
fn rejects_malformed_unsupported_and_over_budget_waves_as_distinct_classes() {
    let mut wrong_container = MONO_PCM16.to_vec();
    wrong_container[0] = b'X';

    let mut unsupported_encoding = MONO_PCM16.to_vec();
    unsupported_encoding[20] = 3;

    let mut unaligned_data = MONO_PCM16.to_vec();
    unaligned_data[4] = 40;
    unaligned_data[40] = 3;
    unaligned_data.truncate(48);

    let restrictive = WavLimits {
        max_decoded_bytes: 8,
        ..WavLimits::default()
    };

    let observed = [
        decode_wav_pcm(&wrong_container, &WavLimits::default()),
        decode_wav_pcm(&unsupported_encoding, &WavLimits::default()),
        decode_wav_pcm(&unaligned_data, &WavLimits::default()),
        decode_wav_pcm(MONO_PCM16, &restrictive),
    ];

    assert!(matches!(observed[0], Err(WavError::InvalidHeader)));
    assert!(matches!(observed[1], Err(WavError::UnsupportedEncoding(3))));
    assert!(matches!(observed[2], Err(WavError::InvalidDataAlignment)));
    assert!(matches!(observed[3], Err(WavError::DecodedLimit)));
}

#[test]
fn rejects_a_pcm_header_whose_declared_byte_rate_disagrees_with_its_frame_shape() {
    let mut contradictory = MONO_PCM16.to_vec();
    contradictory[28] = 0;

    assert!(matches!(
        decode_wav_pcm(&contradictory, &WavLimits::default()),
        Err(WavError::InvalidFormat)
    ));
}
