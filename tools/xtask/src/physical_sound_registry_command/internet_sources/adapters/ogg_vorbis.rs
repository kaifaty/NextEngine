use super::RecordingReport;

const OGG_CAPTURE_PATTERN: &[u8; 4] = b"OggS";
const VORBIS_SIGNATURE: &[u8; 6] = b"vorbis";

pub(super) struct Expectation<'a> {
    pub(super) source_label: &'a str,
    pub(super) sample_rate_hz: u32,
    pub(super) channel_count: u16,
    pub(super) maximum_sample_frames: u64,
}

pub(super) fn validate_vorbis_in_ogg(
    recording_id: &str,
    bytes: &[u8],
    expectation: Expectation<'_>,
) -> Result<RecordingReport, String> {
    let mut offset = 0_usize;
    let mut serial = None;
    let mut expected_sequence = 0_u32;
    let mut packet = Vec::new();
    let mut packet_count = 0_u64;
    let mut page_count = 0_u64;
    let mut last_granule = None;
    let mut saw_end = false;

    while offset < bytes.len() {
        if saw_end {
            return Err(format!(
                "{} {} contains data after its Ogg end page",
                expectation.source_label, recording_id
            ));
        }
        let header = bytes.get(offset..offset + 27).ok_or_else(|| {
            format!(
                "{} {} has a truncated Ogg page header",
                expectation.source_label, recording_id
            )
        })?;
        if &header[0..4] != OGG_CAPTURE_PATTERN || header[4] != 0 || header[5] & !0x07 != 0 {
            return Err(format!(
                "{} {} has an unsupported Ogg page header",
                expectation.source_label, recording_id
            ));
        }
        let header_type = header[5];
        let granule = le_u64(header, 6)?;
        let page_serial = le_u32(header, 14)?;
        let sequence = le_u32(header, 18)?;
        let stored_checksum = le_u32(header, 22)?;
        let segment_count = usize::from(header[26]);
        let segment_table = bytes
            .get(offset + 27..offset + 27 + segment_count)
            .ok_or_else(|| {
                format!(
                    "{} {} has a truncated Ogg segment table",
                    expectation.source_label, recording_id
                )
            })?;
        let payload_bytes = segment_table.iter().try_fold(0_usize, |total, value| {
            total
                .checked_add(usize::from(*value))
                .ok_or_else(|| "Ogg page payload size overflow".to_owned())
        })?;
        let page_bytes = 27_usize
            .checked_add(segment_count)
            .and_then(|value| value.checked_add(payload_bytes))
            .ok_or_else(|| "Ogg page size overflow".to_owned())?;
        let page = bytes.get(offset..offset + page_bytes).ok_or_else(|| {
            format!(
                "{} {} has a truncated Ogg page payload",
                expectation.source_label, recording_id
            )
        })?;
        if ogg_checksum(page) != stored_checksum {
            return Err(format!(
                "{} {} has an invalid Ogg page checksum",
                expectation.source_label, recording_id
            ));
        }

        if page_count == 0 {
            if header_type & 0x02 == 0 || header_type & 0x01 != 0 || sequence != 0 {
                return Err(format!(
                    "{} {} does not start with one Ogg logical stream",
                    expectation.source_label, recording_id
                ));
            }
            serial = Some(page_serial);
        } else if serial != Some(page_serial)
            || header_type & 0x02 != 0
            || sequence != expected_sequence
            || (header_type & 0x01 != 0) != !packet.is_empty()
        {
            return Err(format!(
                "{} {} changes or discontinuously continues its Ogg stream",
                expectation.source_label, recording_id
            ));
        }
        expected_sequence = sequence
            .checked_add(1)
            .ok_or_else(|| "Ogg page sequence overflow".to_owned())?;

        if granule != u64::MAX {
            if last_granule.is_some_and(|previous| granule < previous) {
                return Err(format!(
                    "{} {} has a decreasing Ogg granule position",
                    expectation.source_label, recording_id
                ));
            }
            last_granule = Some(granule);
        }

        let mut payload_offset = offset + 27 + segment_count;
        for segment_bytes in segment_table {
            let segment_bytes = usize::from(*segment_bytes);
            let segment = bytes
                .get(payload_offset..payload_offset + segment_bytes)
                .ok_or_else(|| "truncated Ogg packet segment".to_owned())?;
            packet.extend_from_slice(segment);
            payload_offset += segment_bytes;
            if segment_bytes < 255 {
                validate_header_packet(packet_count, &packet, &expectation)?;
                packet_count = packet_count
                    .checked_add(1)
                    .ok_or_else(|| "Ogg packet count overflow".to_owned())?;
                packet.clear();
            }
        }

        saw_end = header_type & 0x04 != 0;
        if saw_end && !packet.is_empty() {
            return Err(format!(
                "{} {} ends inside an Ogg packet",
                expectation.source_label, recording_id
            ));
        }
        page_count = page_count
            .checked_add(1)
            .ok_or_else(|| "Ogg page count overflow".to_owned())?;
        offset += page_bytes;
    }

    let sample_frames = last_granule.ok_or_else(|| {
        format!(
            "{} {} has no Ogg granule position",
            expectation.source_label, recording_id
        )
    })?;
    if !saw_end || page_count == 0 || packet_count < 4 || !packet.is_empty() {
        return Err(format!(
            "{} {} is not a complete Ogg/Vorbis stream",
            expectation.source_label, recording_id
        ));
    }
    if sample_frames == 0 || sample_frames > expectation.maximum_sample_frames {
        return Err(format!(
            "{} {} exceeds its decoded sample-frame bound",
            expectation.source_label, recording_id
        ));
    }

    Ok(RecordingReport {
        recording_id: recording_id.to_owned(),
        sample_encoding: "vorbis_in_ogg",
        sample_rate_hz: expectation.sample_rate_hz,
        channel_count: expectation.channel_count,
        bits_per_sample: 0,
        sample_frames,
    })
}

fn validate_header_packet(
    packet_index: u64,
    packet: &[u8],
    expectation: &Expectation<'_>,
) -> Result<(), String> {
    match packet_index {
        0 => {
            if packet.len() < 30
                || packet[0] != 0x01
                || &packet[1..7] != VORBIS_SIGNATURE
                || le_u32(packet, 7)? != 0
                || u16::from(packet[11]) != expectation.channel_count
                || le_u32(packet, 12)? != expectation.sample_rate_hz
                || packet[28] & 0x0f > packet[28] >> 4
                || packet[28] >> 4 > 13
                || packet[29] != 1
            {
                return Err(format!(
                    "{} has an unexpected Vorbis identification header",
                    expectation.source_label
                ));
            }
        }
        1 if packet.len() < 7 || packet[0] != 0x03 || &packet[1..7] != VORBIS_SIGNATURE => {
            return Err(format!(
                "{} has an unexpected Vorbis comment header",
                expectation.source_label
            ));
        }
        2 if packet.len() < 7 || packet[0] != 0x05 || &packet[1..7] != VORBIS_SIGNATURE => {
            return Err(format!(
                "{} has an unexpected Vorbis setup header",
                expectation.source_label
            ));
        }
        _ => {}
    }
    Ok(())
}

fn le_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .map(|value| u32::from_le_bytes(value.try_into().expect("four bytes")))
        .ok_or_else(|| "truncated Ogg/Vorbis integer".to_owned())
}

fn le_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    bytes
        .get(offset..offset + 8)
        .map(|value| u64::from_le_bytes(value.try_into().expect("eight bytes")))
        .ok_or_else(|| "truncated Ogg/Vorbis integer".to_owned())
}

fn ogg_checksum(page: &[u8]) -> u32 {
    let mut checksum = 0_u32;
    for (index, byte) in page.iter().copied().enumerate() {
        let byte = if (22..26).contains(&index) { 0 } else { byte };
        checksum ^= u32::from(byte) << 24;
        for _ in 0..8 {
            checksum = if checksum & 0x8000_0000 != 0 {
                (checksum << 1) ^ 0x04c1_1db7
            } else {
                checksum << 1
            };
        }
    }
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_one_bounded_vorbis_logical_stream() {
        let bytes = test_stream();
        let report = validate_vorbis_in_ogg(
            "clip-001",
            &bytes,
            Expectation {
                source_label: "test source",
                sample_rate_hz: 44_100,
                channel_count: 2,
                maximum_sample_frames: 2_000,
            },
        )
        .expect("valid test stream");
        assert_eq!(report.sample_encoding, "vorbis_in_ogg");
        assert_eq!(report.sample_frames, 1_024);
        assert_eq!(report.bits_per_sample, 0);
    }

    #[test]
    fn rejects_checksum_and_format_mutations() {
        let mut bytes = test_stream();
        bytes[30] ^= 1;
        assert!(
            validate_vorbis_in_ogg(
                "clip-001",
                &bytes,
                Expectation {
                    source_label: "test source",
                    sample_rate_hz: 44_100,
                    channel_count: 2,
                    maximum_sample_frames: 2_000,
                }
            )
            .is_err()
        );

        let mut bytes = test_stream();
        let identification_payload = 28_usize;
        bytes[identification_payload + 11] = 1;
        rewrite_checksum(&mut bytes, 0);
        assert!(
            validate_vorbis_in_ogg(
                "clip-001",
                &bytes,
                Expectation {
                    source_label: "test source",
                    sample_rate_hz: 44_100,
                    channel_count: 2,
                    maximum_sample_frames: 2_000,
                }
            )
            .is_err()
        );
    }

    fn test_stream() -> Vec<u8> {
        let mut identification = vec![0_u8; 30];
        identification[0] = 0x01;
        identification[1..7].copy_from_slice(VORBIS_SIGNATURE);
        identification[11] = 2;
        identification[12..16].copy_from_slice(&44_100_u32.to_le_bytes());
        identification[28] = 0xb8;
        identification[29] = 1;
        let comment = [b"\x03vorbis".as_slice(), b"comment"].concat();
        let setup = [b"\x05vorbis".as_slice(), b"setup"].concat();

        let mut bytes = page(0x02, 7, 0, 0, &[&identification]);
        bytes.extend(page(0, 7, 1, 0, &[&comment, &setup]));
        bytes.extend(page(0x04, 7, 2, 1_024, &[&[0x01, 0x02]]));
        bytes
    }

    fn page(
        header_type: u8,
        serial: u32,
        sequence: u32,
        granule: u64,
        packets: &[&[u8]],
    ) -> Vec<u8> {
        assert!(packets.iter().all(|packet| packet.len() < 255));
        let mut page = Vec::new();
        page.extend_from_slice(OGG_CAPTURE_PATTERN);
        page.push(0);
        page.push(header_type);
        page.extend_from_slice(&granule.to_le_bytes());
        page.extend_from_slice(&serial.to_le_bytes());
        page.extend_from_slice(&sequence.to_le_bytes());
        page.extend_from_slice(&0_u32.to_le_bytes());
        page.push(u8::try_from(packets.len()).expect("segment count"));
        page.extend(
            packets
                .iter()
                .map(|packet| u8::try_from(packet.len()).expect("packet size")),
        );
        for packet in packets {
            page.extend_from_slice(packet);
        }
        let checksum = ogg_checksum(&page);
        page[22..26].copy_from_slice(&checksum.to_le_bytes());
        page
    }

    fn rewrite_checksum(bytes: &mut [u8], page_offset: usize) {
        let segment_count = usize::from(bytes[page_offset + 26]);
        let payload_bytes = bytes[page_offset + 27..page_offset + 27 + segment_count]
            .iter()
            .map(|value| usize::from(*value))
            .sum::<usize>();
        let page_bytes = 27 + segment_count + payload_bytes;
        let checksum = ogg_checksum(&bytes[page_offset..page_offset + page_bytes]);
        bytes[page_offset + 22..page_offset + 26].copy_from_slice(&checksum.to_le_bytes());
    }
}
