//! Bounded lossless FLAC admission for packaged sampled-audio assets.
//!
//! Claxon performs format decoding; this adapter proves channel/rate/frame and
//! allocation limits before exposing canonical fixed-point PCM to a host.

use std::io::Cursor;

use thiserror::Error;

use crate::wav::{WavClip, WavLimits, normalize_integer_sample};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FlacError {
    #[error("not a FLAC container")]
    InvalidHeader,
    #[error("malformed FLAC stream: {0}")]
    Malformed(String),
    #[error("unsupported FLAC bit width {0}")]
    UnsupportedBitDepth(u32),
    #[error("channel count exceeds policy")]
    ChannelLimit,
    #[error("sample rate exceeds policy")]
    SampleRateLimit,
    #[error("frame count exceeds policy")]
    FrameLimit,
    #[error("decoded PCM bytes exceed policy")]
    DecodedLimit,
    #[error("decoded sample count is not an exact number of frames")]
    InvalidSampleCount,
}

/// Decodes one complete FLAC stream into the same deterministic fixed-point
/// PCM shape used by integer WAV admission and the native sample adapter.
pub fn decode_flac(bytes: &[u8], limits: &WavLimits) -> Result<WavClip, FlacError> {
    if !bytes.starts_with(b"fLaC") {
        return Err(FlacError::InvalidHeader);
    }
    let mut reader = claxon::FlacReader::new(Cursor::new(bytes))
        .map_err(|error| FlacError::Malformed(error.to_string()))?;
    let info = reader.streaminfo();
    let channels = u16::try_from(info.channels).map_err(|_| FlacError::ChannelLimit)?;
    if channels == 0 || channels > limits.max_channels {
        return Err(FlacError::ChannelLimit);
    }
    if info.sample_rate == 0 || info.sample_rate > limits.max_sample_rate {
        return Err(FlacError::SampleRateLimit);
    }
    if !matches!(info.bits_per_sample, 8 | 16 | 24 | 32) {
        return Err(FlacError::UnsupportedBitDepth(info.bits_per_sample));
    }

    if let Some(frames) = info.samples {
        let frames = usize::try_from(frames).map_err(|_| FlacError::FrameLimit)?;
        enforce_decoded_limits(frames, channels, limits)?;
    }

    let extent = 1_i64 << (info.bits_per_sample - 1);
    let mut samples = Vec::new();
    for sample in reader.samples() {
        let next_sample_count = samples
            .len()
            .checked_add(1)
            .ok_or(FlacError::DecodedLimit)?;
        if next_sample_count.div_ceil(usize::from(channels)) > limits.max_frames {
            return Err(FlacError::FrameLimit);
        }
        if next_sample_count.saturating_mul(4) > limits.max_decoded_bytes {
            return Err(FlacError::DecodedLimit);
        }
        let sample = sample.map_err(|error| FlacError::Malformed(error.to_string()))?;
        samples.push(normalize_integer_sample(
            i64::from(sample),
            extent,
            extent - 1,
        ));
    }
    if samples.len() % usize::from(channels) != 0 {
        return Err(FlacError::InvalidSampleCount);
    }
    let frames = samples.len() / usize::from(channels);
    enforce_decoded_limits(frames, channels, limits)?;
    Ok(WavClip {
        channels,
        sample_rate: info.sample_rate,
        frames,
        samples,
    })
}

fn enforce_decoded_limits(
    frames: usize,
    channels: u16,
    limits: &WavLimits,
) -> Result<(), FlacError> {
    if frames > limits.max_frames {
        return Err(FlacError::FrameLimit);
    }
    let bytes = frames
        .checked_mul(usize::from(channels))
        .and_then(|samples| samples.checked_mul(4))
        .ok_or(FlacError::DecodedLimit)?;
    if bytes > limits.max_decoded_bytes {
        return Err(FlacError::DecodedLimit);
    }
    Ok(())
}
