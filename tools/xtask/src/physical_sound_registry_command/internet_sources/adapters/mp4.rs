use super::RecordingReport;

pub(super) struct Expectation<'a> {
    pub(super) source_label: &'a str,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) expected_sample_frames: u64,
    pub(super) maximum_sample_frames: u64,
}

#[derive(Clone, Copy)]
struct BoxRef<'a> {
    kind: [u8; 4],
    payload: &'a [u8],
}

#[derive(Clone, Copy)]
struct MediaHeader {
    timescale: u32,
    duration: u64,
}

#[derive(Clone, Copy)]
struct Edit {
    segment_duration: u64,
    media_time: i64,
}

struct AudioSampleTable {
    sample_rate_hz: u32,
    channel_count: u16,
    bits_per_sample: u16,
    compressed_samples: u64,
    decoded_sample_frames: u64,
    compressed_bytes: u64,
}

pub(super) fn validate_mpeg_layer3_isobmff(
    recording_id: &str,
    bytes: &[u8],
    expectation: Expectation<'_>,
) -> Result<RecordingReport, String> {
    let top = boxes(bytes, expectation.source_label)?;
    let ftyp = exactly_one(&top, *b"ftyp", expectation.source_label)?;
    validate_ftyp(ftyp.payload, expectation.source_label)?;
    let moov = exactly_one(&top, *b"moov", expectation.source_label)?;
    let mdat = exactly_one(&top, *b"mdat", expectation.source_label)?;
    if mdat.payload.is_empty() {
        return Err(format!(
            "{} has an empty mdat box",
            expectation.source_label
        ));
    }

    let moov_children = boxes(moov.payload, "ISO BMFF moov")?;
    let movie_timescale = parse_movie_timescale(
        exactly_one(&moov_children, *b"mvhd", expectation.source_label)?.payload,
        expectation.source_label,
    )?;
    let mut audio: Option<(MediaHeader, Edit, AudioSampleTable)> = None;
    let mut video_tracks = 0usize;
    for trak in moov_children.iter().filter(|child| child.kind == *b"trak") {
        let trak_children = boxes(trak.payload, "ISO BMFF trak")?;
        let mdia = exactly_one(&trak_children, *b"mdia", expectation.source_label)?;
        let mdia_children = boxes(mdia.payload, "ISO BMFF mdia")?;
        let handler = parse_handler(
            exactly_one(&mdia_children, *b"hdlr", expectation.source_label)?.payload,
            expectation.source_label,
        )?;
        if handler == *b"vide" {
            video_tracks += 1;
            continue;
        }
        if handler != *b"soun" {
            continue;
        }
        if audio.is_some() {
            return Err(format!(
                "{} contains more than one audio track",
                expectation.source_label
            ));
        }
        let media_header = parse_media_header(
            exactly_one(&mdia_children, *b"mdhd", expectation.source_label)?.payload,
            expectation.source_label,
        )?;
        let edts = exactly_one(&trak_children, *b"edts", expectation.source_label)?;
        let edit_children = boxes(edts.payload, "ISO BMFF edts")?;
        let edit = parse_edit(
            exactly_one(&edit_children, *b"elst", expectation.source_label)?.payload,
            expectation.source_label,
        )?;
        let minf = exactly_one(&mdia_children, *b"minf", expectation.source_label)?;
        let minf_children = boxes(minf.payload, "ISO BMFF minf")?;
        let stbl = exactly_one(&minf_children, *b"stbl", expectation.source_label)?;
        let sample_table = parse_audio_sample_table(stbl.payload, expectation.source_label)?;
        audio = Some((media_header, edit, sample_table));
    }
    if video_tracks == 0 {
        return Err(format!(
            "{} does not contain the accompanying video track",
            expectation.source_label
        ));
    }
    let Some((media_header, edit, sample_table)) = audio else {
        return Err(format!(
            "{} does not contain an audio track",
            expectation.source_label
        ));
    };
    if media_header.timescale != sample_table.sample_rate_hz
        || media_header.duration != sample_table.decoded_sample_frames
        || sample_table.sample_rate_hz != expectation.sample_rate_hz
        || sample_table.channel_count != expectation.channel_count
        || sample_table.compressed_samples == 0
        || sample_table.compressed_bytes == 0
        || sample_table.compressed_bytes > mdat.payload.len() as u64
    {
        return Err(format!(
            "{} audio sample table does not match the expected MP3 track",
            expectation.source_label
        ));
    }
    if edit.media_time < 0 {
        return Err(format!(
            "{} audio edit starts before its media",
            expectation.source_label
        ));
    }
    let numerator = edit
        .segment_duration
        .checked_mul(u64::from(media_header.timescale))
        .ok_or_else(|| format!("{} edit duration overflow", expectation.source_label))?;
    let movie_timescale = u64::from(movie_timescale);
    let effective_sample_frames = numerator
        .checked_add(movie_timescale / 2)
        .ok_or_else(|| format!("{} rounded duration overflow", expectation.source_label))?
        / movie_timescale;
    let media_time = u64::try_from(edit.media_time)
        .map_err(|_| format!("{} negative media time", expectation.source_label))?;
    let available_frames = sample_table
        .decoded_sample_frames
        .checked_sub(media_time)
        .ok_or_else(|| {
            format!(
                "{} edit starts after the audio track",
                expectation.source_label
            )
        })?;
    if effective_sample_frames == 0
        || effective_sample_frames > expectation.maximum_sample_frames
        || effective_sample_frames > available_frames
        || available_frames - effective_sample_frames >= 1_152
        || effective_sample_frames != expectation.expected_sample_frames
    {
        return Err(format!(
            "{} playable audio duration changed",
            expectation.source_label
        ));
    }

    Ok(RecordingReport {
        recording_id: recording_id.to_owned(),
        sample_encoding: "mpeg1_layer3_in_isobmff",
        sample_rate_hz: sample_table.sample_rate_hz,
        channel_count: sample_table.channel_count,
        bits_per_sample: sample_table.bits_per_sample,
        sample_frames: effective_sample_frames,
    })
}

fn boxes<'a>(bytes: &'a [u8], label: &str) -> Result<Vec<BoxRef<'a>>, String> {
    let mut result = Vec::new();
    let mut offset = 0usize;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err(format!("{label} has a truncated box header"));
        }
        let size32 = read_u32(bytes, offset, label)?;
        let kind: [u8; 4] = bytes[offset + 4..offset + 8]
            .try_into()
            .map_err(|_| format!("{label} box type is truncated"))?;
        let (header_bytes, size) = match size32 {
            0 => (8usize, bytes.len() - offset),
            1 => {
                if bytes.len() - offset < 16 {
                    return Err(format!("{label} has a truncated extended box header"));
                }
                let size = usize::try_from(read_u64(bytes, offset + 8, label)?)
                    .map_err(|_| format!("{label} box size exceeds this host"))?;
                (16usize, size)
            }
            value => (
                8usize,
                usize::try_from(value).map_err(|_| format!("{label} box size overflow"))?,
            ),
        };
        if size < header_bytes || size > bytes.len() - offset {
            return Err(format!("{label} has an invalid box size"));
        }
        result.push(BoxRef {
            kind,
            payload: &bytes[offset + header_bytes..offset + size],
        });
        offset += size;
        if size32 == 0 && offset != bytes.len() {
            return Err(format!("{label} has data after a zero-sized box"));
        }
    }
    Ok(result)
}

fn exactly_one<'a>(
    boxes: &'a [BoxRef<'a>],
    kind: [u8; 4],
    label: &str,
) -> Result<BoxRef<'a>, String> {
    let mut matches = boxes.iter().copied().filter(|entry| entry.kind == kind);
    let Some(value) = matches.next() else {
        return Err(format!(
            "{label} is missing box {}",
            String::from_utf8_lossy(&kind)
        ));
    };
    if matches.next().is_some() {
        return Err(format!(
            "{label} repeats box {}",
            String::from_utf8_lossy(&kind)
        ));
    }
    Ok(value)
}

fn validate_ftyp(payload: &[u8], label: &str) -> Result<(), String> {
    if payload.len() < 16
        || payload[0..4] != *b"isom"
        || !payload[8..].chunks_exact(4).any(|brand| brand == b"isom")
        || !payload[8..].chunks_exact(4).any(|brand| brand == b"mp41")
    {
        return Err(format!("{label} has an unexpected ISO BMFF brand set"));
    }
    Ok(())
}

fn parse_movie_timescale(payload: &[u8], label: &str) -> Result<u32, String> {
    let version = *payload
        .first()
        .ok_or_else(|| format!("{label} has a truncated mvhd box"))?;
    let offset = match version {
        0 => 12,
        1 => 20,
        _ => return Err(format!("{label} has an unsupported mvhd version")),
    };
    let timescale = read_u32(payload, offset, label)?;
    if timescale == 0 {
        return Err(format!("{label} has a zero movie timescale"));
    }
    Ok(timescale)
}

fn parse_handler(payload: &[u8], label: &str) -> Result<[u8; 4], String> {
    payload
        .get(8..12)
        .ok_or_else(|| format!("{label} has a truncated hdlr box"))?
        .try_into()
        .map_err(|_| format!("{label} has a truncated handler type"))
}

fn parse_media_header(payload: &[u8], label: &str) -> Result<MediaHeader, String> {
    let version = *payload
        .first()
        .ok_or_else(|| format!("{label} has a truncated mdhd box"))?;
    let (timescale_offset, duration_offset, duration64) = match version {
        0 => (12usize, 16usize, false),
        1 => (20usize, 24usize, true),
        _ => return Err(format!("{label} has an unsupported mdhd version")),
    };
    let timescale = read_u32(payload, timescale_offset, label)?;
    let duration = if duration64 {
        read_u64(payload, duration_offset, label)?
    } else {
        u64::from(read_u32(payload, duration_offset, label)?)
    };
    if timescale == 0 || duration == 0 {
        return Err(format!("{label} has an empty media duration"));
    }
    Ok(MediaHeader {
        timescale,
        duration,
    })
}

fn parse_edit(payload: &[u8], label: &str) -> Result<Edit, String> {
    let version = *payload
        .first()
        .ok_or_else(|| format!("{label} has a truncated elst box"))?;
    if read_u32(payload, 4, label)? != 1 {
        return Err(format!("{label} must contain exactly one edit"));
    }
    let (segment_duration, media_time, rate_offset) = match version {
        0 => (
            u64::from(read_u32(payload, 8, label)?),
            i64::from(read_i32(payload, 12, label)?),
            16usize,
        ),
        1 => (
            read_u64(payload, 8, label)?,
            read_i64(payload, 16, label)?,
            24usize,
        ),
        _ => return Err(format!("{label} has an unsupported elst version")),
    };
    if segment_duration == 0
        || read_i16(payload, rate_offset, label)? != 1
        || read_i16(payload, rate_offset + 2, label)? != 0
    {
        return Err(format!("{label} has an unsupported audio edit rate"));
    }
    Ok(Edit {
        segment_duration,
        media_time,
    })
}

fn parse_audio_sample_table(payload: &[u8], label: &str) -> Result<AudioSampleTable, String> {
    let children = boxes(payload, "ISO BMFF stbl")?;
    let (sample_rate_hz, channel_count, bits_per_sample) =
        parse_sample_description(exactly_one(&children, *b"stsd", label)?.payload, label)?;
    let (compressed_samples, decoded_sample_frames) =
        parse_time_to_sample(exactly_one(&children, *b"stts", label)?.payload, label)?;
    let (size_samples, compressed_bytes) =
        parse_sample_sizes(exactly_one(&children, *b"stsz", label)?.payload, label)?;
    if compressed_samples != size_samples
        || !children.iter().any(|entry| entry.kind == *b"stsc")
        || !children
            .iter()
            .any(|entry| entry.kind == *b"stco" || entry.kind == *b"co64")
    {
        return Err(format!("{label} has an incomplete audio sample table"));
    }
    Ok(AudioSampleTable {
        sample_rate_hz,
        channel_count,
        bits_per_sample,
        compressed_samples,
        decoded_sample_frames,
        compressed_bytes,
    })
}

fn parse_sample_description(payload: &[u8], label: &str) -> Result<(u32, u16, u16), String> {
    if read_u32(payload, 4, label)? != 1 {
        return Err(format!("{label} must contain one audio sample entry"));
    }
    let entries = boxes(
        payload
            .get(8..)
            .ok_or_else(|| format!("{label} has a truncated stsd box"))?,
        "ISO BMFF stsd entries",
    )?;
    let entry = exactly_one(&entries, *b"mp4a", label)?;
    if entry.payload.len() < 28
        || read_u16(entry.payload, 6, label)? != 1
        || read_u16(entry.payload, 8, label)? != 0
        || read_i16(entry.payload, 20, label)? != 0
        || read_u16(entry.payload, 22, label)? != 0
    {
        return Err(format!("{label} has an unsupported mp4a sample entry"));
    }
    let channel_count = read_u16(entry.payload, 16, label)?;
    let bits_per_sample = read_u16(entry.payload, 18, label)?;
    let fixed_rate = read_u32(entry.payload, 24, label)?;
    if fixed_rate & 0xffff != 0 {
        return Err(format!("{label} has a fractional audio sample rate"));
    }
    let sample_rate_hz = fixed_rate >> 16;
    let child_boxes = boxes(&entry.payload[28..], "ISO BMFF mp4a")?;
    validate_esds(exactly_one(&child_boxes, *b"esds", label)?.payload, label)?;
    Ok((sample_rate_hz, channel_count, bits_per_sample))
}

fn validate_esds(payload: &[u8], label: &str) -> Result<(), String> {
    let root = payload
        .get(4..)
        .ok_or_else(|| format!("{label} has a truncated esds full box"))?;
    let (tag, es, _) = descriptor(root, label)?;
    if tag != 0x03 || es.len() < 3 || es[2] != 0 {
        return Err(format!("{label} has an unsupported ES descriptor"));
    }
    let (tag, decoder, _) = descriptor(&es[3..], label)?;
    if tag != 0x04 || decoder.len() < 2 || decoder[0] != 0x6b || decoder[1] >> 2 != 5 {
        return Err(format!("{label} audio track is not MPEG-1/2 Layer III"));
    }
    Ok(())
}

fn descriptor<'a>(bytes: &'a [u8], label: &str) -> Result<(u8, &'a [u8], &'a [u8]), String> {
    let tag = *bytes
        .first()
        .ok_or_else(|| format!("{label} has a truncated descriptor tag"))?;
    let mut length = 0usize;
    let mut cursor = 1usize;
    let mut terminated = false;
    for _ in 0..4 {
        let value = *bytes
            .get(cursor)
            .ok_or_else(|| format!("{label} has a truncated descriptor length"))?;
        cursor += 1;
        length = length
            .checked_mul(128)
            .and_then(|current| current.checked_add(usize::from(value & 0x7f)))
            .ok_or_else(|| format!("{label} descriptor length overflow"))?;
        if value & 0x80 == 0 {
            terminated = true;
            break;
        }
    }
    if !terminated || length > bytes.len().saturating_sub(cursor) {
        return Err(format!("{label} has an invalid descriptor length"));
    }
    Ok((
        tag,
        &bytes[cursor..cursor + length],
        &bytes[cursor + length..],
    ))
}

fn parse_time_to_sample(payload: &[u8], label: &str) -> Result<(u64, u64), String> {
    let count = usize::try_from(read_u32(payload, 4, label)?)
        .map_err(|_| format!("{label} stts entry count overflow"))?;
    if count == 0 || count > 4_096 || payload.len() != 8 + count * 8 {
        return Err(format!("{label} has an invalid stts table"));
    }
    let mut samples = 0u64;
    let mut frames = 0u64;
    for index in 0..count {
        let offset = 8 + index * 8;
        let entry_samples = u64::from(read_u32(payload, offset, label)?);
        let sample_delta = u64::from(read_u32(payload, offset + 4, label)?);
        if entry_samples == 0 || sample_delta == 0 || sample_delta > 4_608 {
            return Err(format!("{label} has an invalid MP3 stts entry"));
        }
        samples = samples
            .checked_add(entry_samples)
            .ok_or_else(|| format!("{label} MP3 sample count overflow"))?;
        frames = frames
            .checked_add(
                entry_samples
                    .checked_mul(sample_delta)
                    .ok_or_else(|| format!("{label} decoded duration overflow"))?,
            )
            .ok_or_else(|| format!("{label} decoded duration overflow"))?;
    }
    Ok((samples, frames))
}

fn parse_sample_sizes(payload: &[u8], label: &str) -> Result<(u64, u64), String> {
    let default_size = u64::from(read_u32(payload, 4, label)?);
    let count = usize::try_from(read_u32(payload, 8, label)?)
        .map_err(|_| format!("{label} stsz sample count overflow"))?;
    if count == 0 || count > 1_000_000 {
        return Err(format!("{label} has an invalid stsz sample count"));
    }
    let bytes = if default_size != 0 {
        if payload.len() != 12 {
            return Err(format!("{label} has trailing default stsz data"));
        }
        default_size
            .checked_mul(count as u64)
            .ok_or_else(|| format!("{label} compressed byte count overflow"))?
    } else {
        if payload.len() != 12 + count * 4 {
            return Err(format!("{label} has a truncated stsz table"));
        }
        let mut total = 0u64;
        for index in 0..count {
            let size = u64::from(read_u32(payload, 12 + index * 4, label)?);
            if size == 0 || size > 1_048_576 {
                return Err(format!("{label} has an invalid compressed sample size"));
            }
            total = total
                .checked_add(size)
                .ok_or_else(|| format!("{label} compressed byte count overflow"))?;
        }
        total
    };
    Ok((count as u64, bytes))
}

fn read_u16(bytes: &[u8], offset: usize, label: &str) -> Result<u16, String> {
    Ok(u16::from_be_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or_else(|| format!("{label} has a truncated 16-bit field"))?
            .try_into()
            .map_err(|_| format!("{label} has a malformed 16-bit field"))?,
    ))
}

fn read_i16(bytes: &[u8], offset: usize, label: &str) -> Result<i16, String> {
    Ok(i16::from_be_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or_else(|| format!("{label} has a truncated signed 16-bit field"))?
            .try_into()
            .map_err(|_| format!("{label} has a malformed signed 16-bit field"))?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    Ok(u32::from_be_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| format!("{label} has a truncated 32-bit field"))?
            .try_into()
            .map_err(|_| format!("{label} has a malformed 32-bit field"))?,
    ))
}

fn read_i32(bytes: &[u8], offset: usize, label: &str) -> Result<i32, String> {
    Ok(i32::from_be_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| format!("{label} has a truncated signed 32-bit field"))?
            .try_into()
            .map_err(|_| format!("{label} has a malformed signed 32-bit field"))?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize, label: &str) -> Result<u64, String> {
    Ok(u64::from_be_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| format!("{label} has a truncated 64-bit field"))?
            .try_into()
            .map_err(|_| format!("{label} has a malformed 64-bit field"))?,
    ))
}

fn read_i64(bytes: &[u8], offset: usize, label: &str) -> Result<i64, String> {
    Ok(i64::from_be_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| format!("{label} has a truncated signed 64-bit field"))?
            .try_into()
            .map_err(|_| format!("{label} has a malformed signed 64-bit field"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{Expectation, validate_mpeg_layer3_isobmff};

    #[test]
    fn validates_trimmed_mp3_audio_in_video_container() {
        let bytes = fixture();
        let report = validate_mpeg_layer3_isobmff(
            "001",
            &bytes,
            Expectation {
                source_label: "fixture",
                sample_rate_hz: 44_100,
                channel_count: 2,
                expected_sample_frames: 2_293,
                maximum_sample_frames: 10_000,
            },
        )
        .expect("fixture must validate");
        assert_eq!(report.sample_encoding, "mpeg1_layer3_in_isobmff");
        assert_eq!(report.sample_frames, 2_293);
    }

    #[test]
    fn rejects_codec_or_container_mutation() {
        let mut bytes = fixture();
        let object_type = bytes
            .windows(4)
            .position(|window| window == [0x0d, 0x6b, 0x15, 0x00])
            .expect("decoder descriptor")
            + 1;
        bytes[object_type] = 0x40;
        assert!(validate(&bytes).is_err());

        let mut truncated = fixture();
        truncated.pop();
        assert!(validate(&truncated).is_err());
    }

    fn validate(bytes: &[u8]) -> Result<super::RecordingReport, String> {
        validate_mpeg_layer3_isobmff(
            "001",
            bytes,
            Expectation {
                source_label: "fixture",
                sample_rate_hz: 44_100,
                channel_count: 2,
                expected_sample_frames: 2_293,
                maximum_sample_frames: 10_000,
            },
        )
    }

    fn fixture() -> Vec<u8> {
        let mut ftyp = b"isom".to_vec();
        ftyp.extend_from_slice(&0x200_u32.to_be_bytes());
        ftyp.extend_from_slice(b"isomiso2avc1mp41");

        let mut mvhd = vec![0; 12];
        mvhd.extend_from_slice(&1_000_u32.to_be_bytes());
        mvhd.extend_from_slice(&52_u32.to_be_bytes());

        let mut video_handler = vec![0; 8];
        video_handler.extend_from_slice(b"vide");
        let video_mdia = atom(b"mdia", &atom(b"hdlr", &video_handler));
        let video_trak = atom(b"trak", &video_mdia);

        let mut mdhd = vec![0; 12];
        mdhd.extend_from_slice(&44_100_u32.to_be_bytes());
        mdhd.extend_from_slice(&2_304_u32.to_be_bytes());
        let mut audio_handler = vec![0; 8];
        audio_handler.extend_from_slice(b"soun");

        let esds = atom(
            b"esds",
            &[
                0, 0, 0, 0, 0x03, 0x80, 0x80, 0x80, 0x1b, 0, 2, 0, 0x04, 0x80, 0x80, 0x80, 0x0d,
                0x6b, 0x15, 0, 0, 0, 0, 1, 0xf4, 0, 0, 1, 0xf3, 0xff, 0x06, 0x80, 0x80, 0x80, 1, 2,
            ],
        );
        let mut mp4a = vec![0; 6];
        mp4a.extend_from_slice(&1_u16.to_be_bytes());
        mp4a.extend_from_slice(&0_u16.to_be_bytes());
        mp4a.extend_from_slice(&0_u16.to_be_bytes());
        mp4a.extend_from_slice(&0_u32.to_be_bytes());
        mp4a.extend_from_slice(&2_u16.to_be_bytes());
        mp4a.extend_from_slice(&16_u16.to_be_bytes());
        mp4a.extend_from_slice(&0_i16.to_be_bytes());
        mp4a.extend_from_slice(&0_u16.to_be_bytes());
        mp4a.extend_from_slice(&(44_100_u32 << 16).to_be_bytes());
        mp4a.extend_from_slice(&esds);
        let sample_entry = atom(b"mp4a", &mp4a);
        let mut stsd = vec![0; 4];
        stsd.extend_from_slice(&1_u32.to_be_bytes());
        stsd.extend_from_slice(&sample_entry);
        let mut stts = vec![0; 4];
        stts.extend_from_slice(&1_u32.to_be_bytes());
        stts.extend_from_slice(&2_u32.to_be_bytes());
        stts.extend_from_slice(&1_152_u32.to_be_bytes());
        let mut stsz = vec![0; 4];
        stsz.extend_from_slice(&0_u32.to_be_bytes());
        stsz.extend_from_slice(&2_u32.to_be_bytes());
        stsz.extend_from_slice(&100_u32.to_be_bytes());
        stsz.extend_from_slice(&100_u32.to_be_bytes());
        let mut stsc = vec![0; 4];
        stsc.extend_from_slice(&1_u32.to_be_bytes());
        stsc.extend_from_slice(&1_u32.to_be_bytes());
        stsc.extend_from_slice(&2_u32.to_be_bytes());
        stsc.extend_from_slice(&1_u32.to_be_bytes());
        let mut stco = vec![0; 4];
        stco.extend_from_slice(&1_u32.to_be_bytes());
        stco.extend_from_slice(&0_u32.to_be_bytes());
        let mut stbl = atom(b"stsd", &stsd);
        stbl.extend_from_slice(&atom(b"stts", &stts));
        stbl.extend_from_slice(&atom(b"stsc", &stsc));
        stbl.extend_from_slice(&atom(b"stsz", &stsz));
        stbl.extend_from_slice(&atom(b"stco", &stco));
        let minf = atom(b"minf", &atom(b"stbl", &stbl));
        let mut mdia = atom(b"mdhd", &mdhd);
        mdia.extend_from_slice(&atom(b"hdlr", &audio_handler));
        mdia.extend_from_slice(&minf);

        let mut elst = vec![0; 4];
        elst.extend_from_slice(&1_u32.to_be_bytes());
        elst.extend_from_slice(&52_u32.to_be_bytes());
        elst.extend_from_slice(&0_i32.to_be_bytes());
        elst.extend_from_slice(&1_i16.to_be_bytes());
        elst.extend_from_slice(&0_i16.to_be_bytes());
        let edts = atom(b"edts", &atom(b"elst", &elst));
        let mut audio_trak_payload = edts;
        audio_trak_payload.extend_from_slice(&atom(b"mdia", &mdia));
        let audio_trak = atom(b"trak", &audio_trak_payload);

        let mut moov = atom(b"mvhd", &mvhd);
        moov.extend_from_slice(&video_trak);
        moov.extend_from_slice(&audio_trak);

        let mut result = atom(b"ftyp", &ftyp);
        result.extend_from_slice(&atom(b"mdat", &[0; 300]));
        result.extend_from_slice(&atom(b"moov", &moov));
        result
    }

    fn atom(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(payload.len() + 8);
        result.extend_from_slice(&u32::try_from(payload.len() + 8).unwrap().to_be_bytes());
        result.extend_from_slice(kind);
        result.extend_from_slice(payload);
        result
    }
}
