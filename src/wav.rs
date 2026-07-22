//! Bounded RIFF/WAVE PCM decoding for portable Aedicule asset adapters.
//!
//! The decoder only accepts integer PCM and normalizes it to the same decimal
//! fixed-point domain used by the synthesized-audio path. Device adapters turn
//! those samples into their preferred representation only after admission.

use thiserror::Error;

/// Canonical amplitude scale shared by future sampled-audio and synth mixers.
pub const AUDIO_FIXED_SCALE: i32 = 1_000_000;

/// Admission limits applied before allocating decoded samples from an
/// untrusted package. The limits describe decoded work, not a guest request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WavLimits {
    pub max_channels: u16,
    pub max_sample_rate: u32,
    pub max_frames: usize,
    pub max_decoded_bytes: usize,
}

impl Default for WavLimits {
    fn default() -> Self {
        Self {
            max_channels: 2,
            max_sample_rate: 192_000,
            max_frames: 4_000_000,
            max_decoded_bytes: 32 * 1024 * 1024,
        }
    }
}

/// A decoded interleaved PCM clip with no link to a particular audio device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WavClip {
    pub channels: u16,
    pub sample_rate: u32,
    pub frames: usize,
    pub samples: Vec<i32>,
}

/// Stable failure classes let package admission reject malformed media without
/// confusing corruption, unsupported formats, and resource policy failures.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WavError {
    #[error("not a RIFF/WAVE container")]
    InvalidHeader,
    #[error("RIFF length does not exactly cover the input")]
    InvalidRiffLength,
    #[error("chunk extends beyond the declared RIFF length")]
    TruncatedChunk,
    #[error("missing fmt chunk")]
    MissingFormat,
    #[error("missing data chunk")]
    MissingData,
    #[error("duplicate fmt or data chunk")]
    DuplicateChunk,
    #[error("invalid PCM format fields")]
    InvalidFormat,
    #[error("unsupported WAVE encoding {0}")]
    UnsupportedEncoding(u16),
    #[error("unsupported PCM bit width {0}")]
    UnsupportedBitDepth(u16),
    #[error("channel count exceeds policy")]
    ChannelLimit,
    #[error("sample rate exceeds policy")]
    SampleRateLimit,
    #[error("data length is not an exact number of PCM frames")]
    InvalidDataAlignment,
    #[error("frame count exceeds policy")]
    FrameLimit,
    #[error("decoded PCM bytes exceed policy")]
    DecodedLimit,
}

#[derive(Clone, Copy)]
struct PcmFormat {
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

/// Decodes a standard RIFF/WAVE integer-PCM clip after proving every chunk,
/// frame, and allocation boundary. This keeps hostile media out of device
/// adapters and produces deterministic fixed-point samples for later mixing.
pub fn decode_wav_pcm(bytes: &[u8], limits: &WavLimits) -> Result<WavClip, WavError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(WavError::InvalidHeader);
    }

    let riff_size = read_u32(&bytes[4..8]) as usize;
    let riff_end = 8usize
        .checked_add(riff_size)
        .ok_or(WavError::InvalidRiffLength)?;
    if riff_end != bytes.len() {
        return Err(WavError::InvalidRiffLength);
    }

    let mut cursor = 12usize;
    let mut format = None;
    let mut data = None;
    while cursor < riff_end {
        let header_end = cursor.checked_add(8).ok_or(WavError::TruncatedChunk)?;
        if header_end > riff_end {
            return Err(WavError::TruncatedChunk);
        }
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_len = read_u32(&bytes[cursor + 4..header_end]) as usize;
        let content_end = header_end
            .checked_add(chunk_len)
            .ok_or(WavError::TruncatedChunk)?;
        let padded_end = content_end
            .checked_add(chunk_len & 1)
            .ok_or(WavError::TruncatedChunk)?;
        if padded_end > riff_end {
            return Err(WavError::TruncatedChunk);
        }
        let content = &bytes[header_end..content_end];

        if chunk_id == b"fmt " {
            if format.is_some() {
                return Err(WavError::DuplicateChunk);
            }
            format = Some(parse_format(content)?);
        } else if chunk_id == b"data" {
            if data.is_some() {
                return Err(WavError::DuplicateChunk);
            }
            data = Some(content);
        }
        cursor = padded_end;
    }

    let format = format.ok_or(WavError::MissingFormat)?;
    let data = data.ok_or(WavError::MissingData)?;
    validate_format(format, limits)?;

    let bytes_per_sample = usize::from(format.bits_per_sample / 8);
    let block_align = usize::from(format.block_align);
    if data.len() % block_align != 0 {
        return Err(WavError::InvalidDataAlignment);
    }
    let frames = data.len() / block_align;
    if frames > limits.max_frames {
        return Err(WavError::FrameLimit);
    }
    let sample_count = frames
        .checked_mul(usize::from(format.channels))
        .ok_or(WavError::DecodedLimit)?;
    let decoded_bytes = sample_count
        .checked_mul(std::mem::size_of::<i32>())
        .ok_or(WavError::DecodedLimit)?;
    if decoded_bytes > limits.max_decoded_bytes {
        return Err(WavError::DecodedLimit);
    }

    let mut samples = Vec::with_capacity(sample_count);
    for frame in data.chunks_exact(block_align) {
        for channel in 0..usize::from(format.channels) {
            let offset = channel * bytes_per_sample;
            samples.push(decode_sample(
                &frame[offset..offset + bytes_per_sample],
                format.bits_per_sample,
            ));
        }
    }
    Ok(WavClip {
        channels: format.channels,
        sample_rate: format.sample_rate,
        frames,
        samples,
    })
}

/// Parses the fixed portion of a WAVE fmt chunk and ignores recognized future
/// extension bytes only after the standard PCM contract has been validated.
fn parse_format(content: &[u8]) -> Result<PcmFormat, WavError> {
    if content.len() < 16 {
        return Err(WavError::InvalidFormat);
    }
    let encoding = read_u16(&content[0..2]);
    if encoding != 1 {
        return Err(WavError::UnsupportedEncoding(encoding));
    }
    Ok(PcmFormat {
        channels: read_u16(&content[2..4]),
        sample_rate: read_u32(&content[4..8]),
        byte_rate: read_u32(&content[8..12]),
        block_align: read_u16(&content[12..14]),
        bits_per_sample: read_u16(&content[14..16]),
    })
}

/// Rejects contradictory WAVE fields before payload iteration, preventing
/// malicious headers from changing the parser's frame stride or allocation.
fn validate_format(format: PcmFormat, limits: &WavLimits) -> Result<(), WavError> {
    if format.channels == 0 || format.channels > limits.max_channels {
        return Err(WavError::ChannelLimit);
    }
    if format.sample_rate == 0 || format.sample_rate > limits.max_sample_rate {
        return Err(WavError::SampleRateLimit);
    }
    if !matches!(format.bits_per_sample, 8 | 16 | 24 | 32) {
        return Err(WavError::UnsupportedBitDepth(format.bits_per_sample));
    }
    let bytes_per_sample = u32::from(format.bits_per_sample / 8);
    let expected_block_align = u32::from(format.channels)
        .checked_mul(bytes_per_sample)
        .ok_or(WavError::InvalidFormat)?;
    if u32::from(format.block_align) != expected_block_align {
        return Err(WavError::InvalidFormat);
    }
    let expected_byte_rate = format
        .sample_rate
        .checked_mul(expected_block_align)
        .ok_or(WavError::InvalidFormat)?;
    if format.byte_rate != expected_byte_rate {
        return Err(WavError::InvalidFormat);
    }
    Ok(())
}

/// Converts one signed integer PCM value into the symmetric decimal amplitude
/// range, preserving both representable extrema exactly without float math.
fn normalize_integer_sample(value: i64, negative_extent: i64, positive_extent: i64) -> i32 {
    let scale = i64::from(AUDIO_FIXED_SCALE);
    let normalized = if value < 0 {
        value * scale / negative_extent
    } else if value > 0 {
        value * scale / positive_extent
    } else {
        0
    };
    normalized as i32
}

/// Decodes little-endian PCM widths without alignment-dependent casts or any
/// floating-point conversion, so fixture results are adapter-independent.
fn decode_sample(bytes: &[u8], bits_per_sample: u16) -> i32 {
    match bits_per_sample {
        8 => normalize_integer_sample(i64::from(bytes[0]) - 128, 128, 127),
        16 => normalize_integer_sample(
            i64::from(i16::from_le_bytes([bytes[0], bytes[1]])),
            32_768,
            32_767,
        ),
        24 => {
            let raw =
                i32::from(bytes[0]) | (i32::from(bytes[1]) << 8) | (i32::from(bytes[2]) << 16);
            let signed = if raw & 0x0080_0000 == 0 {
                raw
            } else {
                raw | !0x00ff_ffff
            };
            normalize_integer_sample(i64::from(signed), 8_388_608, 8_388_607)
        }
        32 => normalize_integer_sample(
            i64::from(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            2_147_483_648,
            2_147_483_647,
        ),
        _ => unreachable!("validate_format accepts only supported PCM widths"),
    }
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
