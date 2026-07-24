//! Deterministic, device-independent synthesis shared by native and browser
//! audio adapters.

use crate::{AudioEvent, SynthFilter, SynthVoice, SynthWaveform};

pub const DECIMAL_SCALE: i64 = 1_000_000;

/// Converts the frontplane's float audio ABI into the decimal representation;
/// this is an ingress boundary, never part of synthesis or game state.
pub fn scalar_from_host(value: f32) -> i64 {
    if value.is_finite() {
        (value * DECIMAL_SCALE as f32).round() as i64
    } else {
        0
    }
}

/// Converts completed decimal PCM samples to a device adapter's required host
/// representation only after oscillators, envelopes, and filters finish.
pub fn samples_to_host(samples: Vec<i32>) -> Vec<f32> {
    samples
        .into_iter()
        .map(|sample| sample as f32 / DECIMAL_SCALE as f32)
        .collect()
}

/// Renders one admitted guest synth event into portable mono PCM so native and
/// browser adapters cannot silently implement different instruments.
pub fn render_synth_program(
    voices: &[SynthVoice],
    event: AudioEvent,
    sample_rate: u32,
) -> Vec<f32> {
    samples_to_host(render_synth_program_fixed(
        voices,
        event.id,
        scalar_from_host(event.volume),
        scalar_from_host(event.pitch),
        sample_rate,
    ))
}

// FIXED_AUDIO_BEGIN
fn fixed_mul(left: i64, right: i64) -> i64 {
    ((left as i128 * right as i128) / DECIMAL_SCALE as i128) as i64
}

fn interpolate_integer(start: i64, end: i64, index: usize, count: usize) -> i64 {
    if count == 0 {
        return start;
    }
    (start as i128 + (end as i128 - start as i128) * index as i128 / count as i128) as i64
}

fn three_point_integer(start: i64, middle: i64, end: i64, index: usize, count: usize) -> i64 {
    if index.saturating_mul(2) < count {
        interpolate_integer(start, middle, index.saturating_mul(2), count)
    } else {
        interpolate_integer(middle, end, index.saturating_mul(2) - count, count)
    }
}

/// Approximates a sinusoid from a decimal microturn phase using a corrected
/// parabola, preserving exact zeroes and extrema without a float lookup table.
pub fn fixed_sine(phase: i64) -> i64 {
    let phase = phase.rem_euclid(DECIMAL_SCALE);
    let (half_phase, sign) = if phase < DECIMAL_SCALE / 2 {
        (phase * 2, 1)
    } else {
        ((phase - DECIMAL_SCALE / 2) * 2, -1)
    };
    let parabola = (4_i128 * half_phase as i128 * (DECIMAL_SCALE - half_phase) as i128
        / DECIMAL_SCALE as i128) as i64;
    let corrected = parabola + fixed_mul(225_000, fixed_mul(parabola, parabola) - parabola);
    corrected * sign
}

fn noise_sample(random: u32) -> i64 {
    ((random as i128 - 2_147_483_648_i128) * DECIMAL_SCALE as i128 / 2_147_483_648_i128) as i64
}

fn frequency_phase_step(frequency_millihz: i64, sample_rate: u32) -> i64 {
    (frequency_millihz as i128 * 1_000 / sample_rate.max(1) as i128) as i64
}

fn cutoff_omega(cutoff_millihz: i64, sample_rate: u32) -> i64 {
    let maximum = sample_rate as i64 * 450;
    let cutoff = cutoff_millihz.clamp(1, maximum);
    (6_283_185_i128 * cutoff as i128 / (sample_rate.max(1) as i128 * 1_000)) as i64
}

fn low_pass_coefficient(cutoff_millihz: i64, sample_rate: u32) -> i64 {
    let omega = cutoff_omega(cutoff_millihz, sample_rate);
    (omega as i128 * DECIMAL_SCALE as i128 / (DECIMAL_SCALE + omega) as i128) as i64
}

/// Renders all guest-declared synthesis with signed decimal millionths,
/// keeping the portable core independent of any native or browser device.
pub fn render_synth_program_fixed(
    voices: &[SynthVoice],
    event_id: u32,
    event_volume: i64,
    event_pitch: i64,
    sample_rate: u32,
) -> Vec<i32> {
    let total_ms = voices
        .iter()
        .map(|voice| voice.delay_ms + voice.duration_ms)
        .max()
        .unwrap_or_default();
    let mut output = vec![0_i64; total_ms as usize * sample_rate as usize / 1_000];

    for (voice_index, voice) in voices.iter().enumerate() {
        let start_sample = voice.delay_ms as usize * sample_rate as usize / 1_000;
        let sample_count = voice.duration_ms as usize * sample_rate as usize / 1_000;
        let mut phase = 0_i64;
        let mut random = event_id ^ (voice_index as u32 + 1).wrapping_mul(0x9e37_79b9);
        let mut brown = 0_i64;
        let mut low = 0_i64;
        let mut band = 0_i64;

        for local_index in 0..sample_count {
            let frequency = fixed_mul(
                three_point_integer(
                    voice.frequency_start_millihz.into(),
                    voice.frequency_mid_millihz.into(),
                    voice.frequency_end_millihz.into(),
                    local_index,
                    sample_count,
                ),
                event_pitch,
            );
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            let white = noise_sample(random);
            brown = fixed_mul(brown + fixed_mul(white, 20_000), 980_392)
                .clamp(-DECIMAL_SCALE, DECIMAL_SCALE);
            let raw = match voice.waveform {
                SynthWaveform::Sine => fixed_sine(phase),
                SynthWaveform::Saw => phase * 2 - DECIMAL_SCALE,
                SynthWaveform::WhiteNoise => white,
                SynthWaveform::BrownNoise => fixed_mul(brown, 3_500_000),
            };
            phase =
                (phase + frequency_phase_step(frequency, sample_rate)).rem_euclid(DECIMAL_SCALE);

            let cutoff = interpolate_integer(
                voice.filter_start_millihz.max(1).into(),
                voice.filter_end_millihz.max(1).into(),
                local_index,
                sample_count,
            );
            let filtered = match voice.filter {
                SynthFilter::None => raw,
                SynthFilter::LowPass => {
                    let alpha = low_pass_coefficient(cutoff, sample_rate);
                    low += fixed_mul(alpha, raw - low);
                    low
                }
                SynthFilter::BandPass => {
                    let coefficient = cutoff_omega(cutoff, sample_rate).clamp(0, 990_000);
                    let high = raw - low - fixed_mul(800_000, band);
                    band += fixed_mul(coefficient, high);
                    low += fixed_mul(coefficient, band);
                    band
                }
            };
            let attack_count = (sample_count / 20).max(1);
            let gain = if local_index < attack_count {
                interpolate_integer(
                    voice.gain_start_ppm.into(),
                    voice.gain_peak_ppm.into(),
                    local_index,
                    attack_count,
                )
            } else {
                interpolate_integer(
                    voice.gain_peak_ppm.into(),
                    voice.gain_end_ppm.into(),
                    local_index - attack_count,
                    sample_count.saturating_sub(attack_count),
                )
            };
            output[start_sample + local_index] +=
                fixed_mul(fixed_mul(filtered, gain), event_volume);
        }
    }

    output
        .into_iter()
        .map(|sample| sample.clamp(-DECIMAL_SCALE, DECIMAL_SCALE) as i32)
        .collect()
}
// FIXED_AUDIO_END
