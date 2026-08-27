use super::RecordingReport;

const MPEG1_LAYER3_SAMPLES_PER_FRAME: u64 = 1_152;
const BITRATES_KBPS: [u32; 16] = [
    0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0,
];
const SAMPLE_RATES_HZ: [u32; 4] = [44_100, 48_000, 32_000, 0];

pub(super) struct Expectation<'a> {
    pub(super) source_label: &'a str,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) maximum_sample_frames: u64,
}

pub(super) fn validate_mpeg1_layer3(
    recording_id: &str,
    bytes: &[u8],
    expected: Expectation<'_>,
) -> Result<RecordingReport, String> {
    if bytes.len() < 4 || bytes.starts_with(b"ID3") {
        return Err(format!(
            "{} {} must begin with an MPEG audio frame",
            expected.source_label, recording_id
        ));
    }
    let first = parse_header(bytes, 0, expected.source_label)?;
    if first.sample_rate_hz != expected.sample_rate_hz
        || first.channel_count != expected.channel_count
    {
        return Err(format!(
            "{} {} has the wrong sample rate or channel count",
            expected.source_label, recording_id
        ));
    }
    let xing_offset = 4_usize
        .checked_add(first.side_information_bytes)
        .ok_or_else(|| "MPEG Xing offset overflow".to_owned())?;
    let xing = bytes.get(xing_offset..).ok_or_else(|| {
        format!(
            "{} {} has no Xing header",
            expected.source_label, recording_id
        )
    })?;
    if !xing.starts_with(b"Xing") && !xing.starts_with(b"Info") {
        return Err(format!(
            "{} {} has no Xing/Info gapless header",
            expected.source_label, recording_id
        ));
    }
    let flags = be_u32(xing, 4)?;
    if flags != 0x0f {
        return Err(format!(
            "{} {} has unsupported Xing flags",
            expected.source_label, recording_id
        ));
    }
    let declared_frames = u64::from(be_u32(xing, 8)?);
    let declared_bytes = u64::from(be_u32(xing, 12)?);
    if declared_bytes != bytes.len() as u64 || declared_frames == 0 {
        return Err(format!(
            "{} {} Xing byte/frame count changed",
            expected.source_label, recording_id
        ));
    }
    let lame_offset = 4 + 4 + 4 + 4 + 100 + 4;
    let lame = xing
        .get(lame_offset..)
        .ok_or_else(|| format!("{} {} has no LAME tag", expected.source_label, recording_id))?;
    if !lame.starts_with(b"LAME") || lame.len() < 24 {
        return Err(format!(
            "{} {} has no supported LAME gapless tag",
            expected.source_label, recording_id
        ));
    }
    let encoder_delay = (u64::from(lame[21]) << 4) | u64::from(lame[22] >> 4);
    let encoder_padding = (u64::from(lame[22] & 0x0f) << 8) | u64::from(lame[23]);

    let mut offset = 0_usize;
    let mut parsed_frames = 0_u64;
    while offset < bytes.len() {
        let header = parse_header(bytes, offset, expected.source_label)?;
        if header.sample_rate_hz != expected.sample_rate_hz
            || header.channel_count != expected.channel_count
        {
            return Err(format!(
                "{} {} changes format between MPEG frames",
                expected.source_label, recording_id
            ));
        }
        offset = offset
            .checked_add(header.frame_bytes)
            .ok_or_else(|| "MPEG frame offset overflow".to_owned())?;
        if offset > bytes.len() {
            return Err(format!(
                "{} {} ends inside an MPEG frame",
                expected.source_label, recording_id
            ));
        }
        parsed_frames += 1;
    }
    let expected_physical_frames = declared_frames
        .checked_add(1)
        .ok_or_else(|| "MPEG Xing frame count overflow".to_owned())?;
    if parsed_frames != expected_physical_frames {
        return Err(format!(
            "{} {} MPEG audio frame count plus its Xing metadata frame does not match the stream",
            expected.source_label, recording_id
        ));
    }
    let sample_frames = declared_frames
        .checked_mul(MPEG1_LAYER3_SAMPLES_PER_FRAME)
        .and_then(|value| value.checked_sub(encoder_delay))
        .and_then(|value| value.checked_sub(encoder_padding))
        .ok_or_else(|| "MPEG gapless sample count underflow".to_owned())?;
    if sample_frames == 0 || sample_frames > expected.maximum_sample_frames {
        return Err(format!(
            "{} {} exceeds its decoded sample-frame bound",
            expected.source_label, recording_id
        ));
    }
    Ok(RecordingReport {
        recording_id: recording_id.to_owned(),
        sample_encoding: "mpeg1_layer3_gapless",
        sample_rate_hz: expected.sample_rate_hz,
        channel_count: expected.channel_count,
        bits_per_sample: 0,
        sample_frames,
    })
}

struct FrameHeader {
    sample_rate_hz: u32,
    channel_count: u16,
    side_information_bytes: usize,
    frame_bytes: usize,
}

fn parse_header(bytes: &[u8], offset: usize, source_label: &str) -> Result<FrameHeader, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("{source_label} has a truncated MPEG header"))?;
    let header = u32::from_be_bytes(raw.try_into().expect("four-byte MPEG header"));
    let sync = (header >> 21) & 0x7ff;
    let version = (header >> 19) & 0x3;
    let layer = (header >> 17) & 0x3;
    let bitrate_index = usize::try_from((header >> 12) & 0x0f)
        .map_err(|_| "MPEG bitrate index overflow".to_owned())?;
    let sample_rate_index = usize::try_from((header >> 10) & 0x03)
        .map_err(|_| "MPEG sample-rate index overflow".to_owned())?;
    let padding = usize::try_from((header >> 9) & 0x01)
        .map_err(|_| "MPEG padding bit overflow".to_owned())?;
    let channel_mode = (header >> 6) & 0x03;
    let bitrate_kbps = BITRATES_KBPS[bitrate_index];
    let sample_rate_hz = SAMPLE_RATES_HZ[sample_rate_index];
    if sync != 0x7ff || version != 0x03 || layer != 0x01 || bitrate_kbps == 0 || sample_rate_hz == 0
    {
        return Err(format!("{source_label} has an unsupported MPEG frame"));
    }
    let channel_count = if channel_mode == 0x03 { 1 } else { 2 };
    let side_information_bytes = if channel_count == 1 { 17 } else { 32 };
    let frame_bytes_u32 = 144_u32
        .checked_mul(bitrate_kbps)
        .and_then(|value| value.checked_mul(1_000))
        .map(|value| value / sample_rate_hz)
        .and_then(|value| value.checked_add(padding as u32))
        .ok_or_else(|| "MPEG frame byte count overflow".to_owned())?;
    let frame_bytes = usize::try_from(frame_bytes_u32)
        .map_err(|_| "MPEG frame size does not fit usize".to_owned())?;
    Ok(FrameHeader {
        sample_rate_hz,
        channel_count,
        side_information_bytes,
        frame_bytes,
    })
}

fn be_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .map(|value| u32::from_be_bytes(value.try_into().expect("four bytes")))
        .ok_or_else(|| "truncated MPEG metadata integer".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_gapless_xing_lame_stream() {
        let bytes = test_mp3();
        let report = validate_mpeg1_layer3(
            "001",
            &bytes,
            Expectation {
                source_label: "test preview",
                sample_rate_hz: 44_100,
                channel_count: 2,
                maximum_sample_frames: 2_000,
            },
        )
        .expect("valid MPEG stream");
        assert_eq!(report.sample_frames, 1_152);
        assert_eq!(report.bits_per_sample, 0);
    }

    #[test]
    fn rejects_xing_count_or_frame_mutation() {
        let mut bytes = test_mp3();
        bytes[47] = 3;
        assert!(
            validate_mpeg1_layer3(
                "001",
                &bytes,
                Expectation {
                    source_label: "test preview",
                    sample_rate_hz: 44_100,
                    channel_count: 2,
                    maximum_sample_frames: 4_000,
                }
            )
            .is_err()
        );
    }

    fn test_mp3() -> Vec<u8> {
        let frame_bytes = 417_usize;
        let mut bytes = vec![0_u8; frame_bytes * 3];
        for offset in [0, frame_bytes, frame_bytes * 2] {
            bytes[offset..offset + 4].copy_from_slice(&[0xff, 0xfb, 0x90, 0x64]);
        }
        let xing = 36_usize;
        bytes[xing..xing + 4].copy_from_slice(b"Xing");
        bytes[xing + 4..xing + 8].copy_from_slice(&0x0f_u32.to_be_bytes());
        bytes[xing + 8..xing + 12].copy_from_slice(&2_u32.to_be_bytes());
        let stream_bytes = u32::try_from(bytes.len()).expect("test size").to_be_bytes();
        bytes[xing + 12..xing + 16].copy_from_slice(&stream_bytes);
        let lame = xing + 120;
        bytes[lame..lame + 4].copy_from_slice(b"LAME");
        bytes[lame + 21..lame + 24].copy_from_slice(&[0x24, 0x02, 0x40]);
        bytes
    }
}
