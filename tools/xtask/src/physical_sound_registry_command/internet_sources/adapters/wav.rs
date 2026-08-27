use super::RecordingReport;

#[derive(Clone, Copy)]
pub(super) struct FloatWavExpectation {
    pub(super) source_label: &'static str,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) maximum_frames: u64,
}

#[derive(Clone, Copy)]
pub(super) struct Pcm16WavExpectation {
    pub(super) source_label: &'static str,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) maximum_frames: u64,
}

pub(super) fn validate_float_wav(
    recording_id: &str,
    bytes: &[u8],
    expectation: FloatWavExpectation,
) -> Result<RecordingReport, String> {
    let label = expectation.source_label;
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{label} {recording_id} is not RIFF/WAVE"));
    }
    let declared_size = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes")) as usize;
    if declared_size.checked_add(8) != Some(bytes.len()) {
        return Err(format!(
            "{label} {recording_id} RIFF length does not match the payload"
        ));
    }
    let mut offset = 12_usize;
    let mut format = None;
    let mut fact_frames = None;
    let mut data = None;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err(format!("{label} {recording_id} has a truncated chunk"));
        }
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_bytes = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("four bytes"),
        ) as usize;
        let start = offset + 8;
        let end = start
            .checked_add(chunk_bytes)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("{label} {recording_id} chunk exceeds the payload"))?;
        match chunk_id {
            b"fmt " => {
                if format.is_some() || chunk_bytes < 16 {
                    return Err(format!("{label} {recording_id} has an invalid fmt chunk"));
                }
                format = Some(parse_format(recording_id, &bytes[start..end], expectation)?);
            }
            b"fact" => {
                if fact_frames.is_some() || chunk_bytes != 4 {
                    return Err(format!("{label} {recording_id} has an invalid fact chunk"));
                }
                fact_frames = Some(u32::from_le_bytes(
                    bytes[start..end].try_into().expect("four bytes"),
                ) as u64);
            }
            b"data" => {
                if data.is_some() {
                    return Err(format!("{label} {recording_id} has duplicate audio data"));
                }
                data = Some(&bytes[start..end]);
            }
            _ => {}
        }
        offset = end
            .checked_add(chunk_bytes & 1)
            .filter(|offset| *offset <= bytes.len())
            .ok_or_else(|| format!("{label} {recording_id} has invalid chunk padding"))?;
    }
    let format = format.ok_or_else(|| format!("{label} {recording_id} has no fmt chunk"))?;
    let data = data.ok_or_else(|| format!("{label} {recording_id} has no data chunk"))?;
    if data.is_empty() || data.len() % usize::from(format.block_align) != 0 {
        return Err(format!("{label} {recording_id} has misaligned audio data"));
    }
    let sample_frames = (data.len() / usize::from(format.block_align)) as u64;
    if sample_frames > expectation.maximum_frames || fact_frames != Some(sample_frames) {
        return Err(format!(
            "{label} {recording_id} has an invalid bounded frame count"
        ));
    }
    let mut any_nonzero = false;
    for sample in data.chunks_exact(4) {
        let value = f32::from_le_bytes(sample.try_into().expect("four bytes"));
        if !value.is_finite() {
            return Err(format!(
                "{label} {recording_id} contains a non-finite sample"
            ));
        }
        any_nonzero |= value != 0.0;
    }
    if !any_nonzero {
        return Err(format!("{label} {recording_id} is silent"));
    }
    Ok(RecordingReport {
        recording_id: recording_id.to_owned(),
        sample_encoding: "ieee_float32_le",
        sample_rate_hz: format.sample_rate_hz,
        channel_count: format.channel_count,
        bits_per_sample: format.bits_per_sample,
        sample_frames,
    })
}

pub(super) fn validate_pcm16_wav(
    recording_id: &str,
    bytes: &[u8],
    expectation: Pcm16WavExpectation,
) -> Result<RecordingReport, String> {
    let label = expectation.source_label;
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{label} {recording_id} is not RIFF/WAVE"));
    }
    let declared_size = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes")) as usize;
    if declared_size.checked_add(8) != Some(bytes.len()) {
        return Err(format!(
            "{label} {recording_id} RIFF length does not match the payload"
        ));
    }
    let mut offset = 12_usize;
    let mut format = None;
    let mut data = None;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err(format!("{label} {recording_id} has a truncated chunk"));
        }
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_bytes = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("four bytes"),
        ) as usize;
        let start = offset + 8;
        let end = start
            .checked_add(chunk_bytes)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("{label} {recording_id} chunk exceeds the payload"))?;
        match chunk_id {
            b"fmt " => {
                if format.is_some() || chunk_bytes != 16 {
                    return Err(format!("{label} {recording_id} has an invalid fmt chunk"));
                }
                format = Some(parse_pcm16_format(
                    recording_id,
                    &bytes[start..end],
                    expectation,
                )?);
            }
            b"data" => {
                if data.is_some() {
                    return Err(format!("{label} {recording_id} has duplicate audio data"));
                }
                data = Some(&bytes[start..end]);
            }
            _ => {}
        }
        offset = end
            .checked_add(chunk_bytes & 1)
            .filter(|offset| *offset <= bytes.len())
            .ok_or_else(|| format!("{label} {recording_id} has invalid chunk padding"))?;
    }
    let format = format.ok_or_else(|| format!("{label} {recording_id} has no fmt chunk"))?;
    let data = data.ok_or_else(|| format!("{label} {recording_id} has no data chunk"))?;
    if data.is_empty() || data.len() % usize::from(format.block_align) != 0 {
        return Err(format!("{label} {recording_id} has misaligned audio data"));
    }
    let sample_frames = (data.len() / usize::from(format.block_align)) as u64;
    if sample_frames > expectation.maximum_frames {
        return Err(format!(
            "{label} {recording_id} has an invalid bounded frame count"
        ));
    }
    if !data
        .chunks_exact(2)
        .any(|sample| i16::from_le_bytes(sample.try_into().expect("two bytes")) != 0)
    {
        return Err(format!("{label} {recording_id} is silent"));
    }
    Ok(RecordingReport {
        recording_id: recording_id.to_owned(),
        sample_encoding: "pcm_s16_le",
        sample_rate_hz: format.sample_rate_hz,
        channel_count: format.channel_count,
        bits_per_sample: format.bits_per_sample,
        sample_frames,
    })
}

struct WavFormat {
    sample_rate_hz: u32,
    channel_count: u16,
    bits_per_sample: u16,
    block_align: u16,
}

fn parse_pcm16_format(
    recording_id: &str,
    bytes: &[u8],
    expectation: Pcm16WavExpectation,
) -> Result<WavFormat, String> {
    let audio_format = u16::from_le_bytes(bytes[0..2].try_into().expect("two bytes"));
    let channel_count = u16::from_le_bytes(bytes[2..4].try_into().expect("two bytes"));
    let sample_rate_hz = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes"));
    let byte_rate = u32::from_le_bytes(bytes[8..12].try_into().expect("four bytes"));
    let block_align = u16::from_le_bytes(bytes[12..14].try_into().expect("two bytes"));
    let bits_per_sample = u16::from_le_bytes(bytes[14..16].try_into().expect("two bytes"));
    let expected_block_align = expectation
        .channel_count
        .checked_mul(2)
        .ok_or_else(|| "PCM16 WAV block-align overflow".to_owned())?;
    let expected_byte_rate = expectation
        .sample_rate_hz
        .checked_mul(u32::from(expected_block_align))
        .ok_or_else(|| "PCM16 WAV byte-rate overflow".to_owned())?;
    if audio_format != 1
        || channel_count != expectation.channel_count
        || sample_rate_hz != expectation.sample_rate_hz
        || bits_per_sample != 16
        || block_align != expected_block_align
        || byte_rate != expected_byte_rate
    {
        return Err(format!(
            "{} {recording_id} does not match its PCM16 source format",
            expectation.source_label
        ));
    }
    Ok(WavFormat {
        sample_rate_hz,
        channel_count,
        bits_per_sample,
        block_align,
    })
}

fn parse_format(
    recording_id: &str,
    bytes: &[u8],
    expectation: FloatWavExpectation,
) -> Result<WavFormat, String> {
    let audio_format = u16::from_le_bytes(bytes[0..2].try_into().expect("two bytes"));
    let channel_count = u16::from_le_bytes(bytes[2..4].try_into().expect("two bytes"));
    let sample_rate_hz = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes"));
    let byte_rate = u32::from_le_bytes(bytes[8..12].try_into().expect("four bytes"));
    let block_align = u16::from_le_bytes(bytes[12..14].try_into().expect("two bytes"));
    let bits_per_sample = u16::from_le_bytes(bytes[14..16].try_into().expect("two bytes"));
    let expected_block_align = expectation
        .channel_count
        .checked_mul(4)
        .ok_or_else(|| "float WAV block-align overflow".to_owned())?;
    let expected_byte_rate = expectation
        .sample_rate_hz
        .checked_mul(u32::from(expected_block_align))
        .ok_or_else(|| "float WAV byte-rate overflow".to_owned())?;
    if audio_format != 3
        || channel_count != expectation.channel_count
        || sample_rate_hz != expectation.sample_rate_hz
        || bits_per_sample != 32
        || block_align != expected_block_align
        || byte_rate != expected_byte_rate
    {
        return Err(format!(
            "{} {recording_id} does not match its IEEE float32 source format",
            expectation.source_label
        ));
    }
    Ok(WavFormat {
        sample_rate_hz,
        channel_count,
        bits_per_sample,
        block_align,
    })
}

#[cfg(test)]
pub(super) fn test_float_wav(sample_rate_hz: u32, channel_count: u16, samples: &[f32]) -> Vec<u8> {
    assert_eq!(samples.len() % usize::from(channel_count), 0);
    let sample_frames = samples.len() / usize::from(channel_count);
    let data_bytes = samples.len() * 4;
    let riff_bytes = 4 + (8 + 18) + (8 + 4) + (8 + data_bytes);
    let block_align = channel_count * 4;
    let mut bytes = Vec::with_capacity(riff_bytes + 8);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(riff_bytes as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&18_u32.to_le_bytes());
    bytes.extend_from_slice(&3_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate_hz * u32::from(block_align)).to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&32_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.extend_from_slice(b"fact");
    bytes.extend_from_slice(&4_u32.to_le_bytes());
    bytes.extend_from_slice(&(sample_frames as u32).to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_bytes as u32).to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}
