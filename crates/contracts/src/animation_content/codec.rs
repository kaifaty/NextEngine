use crate::canonical::DecodedCanonicalSegment;
use crate::ids::SchemaId;

use super::{
    MAX_ANIMATION_KEYS_V1, MAX_ANIMATION_MARKERS_V1, NEUTRAL_ANIMATION_OWNER_ID,
    NeutralAnimationContentErrorV1, NeutralAnimationKeyV1, NeutralAnimationMarkerV1,
    NeutralAnimationValueV1,
};

pub(super) fn encode_keys(
    values: &[NeutralAnimationKeyV1],
) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
    let mut bytes = Vec::new();
    append_len(&mut bytes, values.len())?;
    for key in values {
        bytes.extend_from_slice(&key.time_microseconds.to_le_bytes());
        match key.value {
            NeutralAnimationValueV1::Translation(value) => {
                bytes.push(1);
                for item in value {
                    bytes.extend_from_slice(&item.to_le_bytes());
                }
            }
            NeutralAnimationValueV1::Rotation(value) => {
                bytes.push(2);
                for item in value {
                    bytes.extend_from_slice(&item.to_le_bytes());
                }
            }
            NeutralAnimationValueV1::Scale(value) => {
                bytes.push(3);
                for item in value {
                    bytes.extend_from_slice(&item.to_le_bytes());
                }
            }
            NeutralAnimationValueV1::MorphWeight(value) => {
                bytes.push(4);
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    Ok(bytes)
}

pub(super) fn decode_keys(
    bytes: &[u8],
) -> Result<Vec<NeutralAnimationKeyV1>, NeutralAnimationContentErrorV1> {
    let mut reader = Reader::new(bytes);
    let count = reader.len()?;
    if count > MAX_ANIMATION_KEYS_V1 {
        return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let time_microseconds = reader.u64()?;
        let value = match reader.u8()? {
            1 => {
                NeutralAnimationValueV1::Translation([reader.i64()?, reader.i64()?, reader.i64()?])
            }
            2 => NeutralAnimationValueV1::Rotation([
                reader.i32()?,
                reader.i32()?,
                reader.i32()?,
                reader.i32()?,
            ]),
            3 => NeutralAnimationValueV1::Scale([reader.u32()?, reader.u32()?, reader.u32()?]),
            4 => NeutralAnimationValueV1::MorphWeight(reader.u16()?),
            _ => return Err(NeutralAnimationContentErrorV1::UnknownTag),
        };
        values.push(NeutralAnimationKeyV1 {
            time_microseconds,
            value,
        });
    }
    reader.finish()?;
    Ok(values)
}

pub(super) fn encode_markers(
    values: &[NeutralAnimationMarkerV1],
) -> Result<Vec<u8>, NeutralAnimationContentErrorV1> {
    let mut bytes = Vec::new();
    append_len(&mut bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(&value.time_microseconds.to_le_bytes());
        append_string(&mut bytes, value.marker_id.as_str())?;
    }
    Ok(bytes)
}

pub(super) fn decode_markers(
    bytes: &[u8],
) -> Result<Vec<NeutralAnimationMarkerV1>, NeutralAnimationContentErrorV1> {
    let mut reader = Reader::new(bytes);
    let count = reader.len()?;
    if count > MAX_ANIMATION_MARKERS_V1 {
        return Err(NeutralAnimationContentErrorV1::LimitOrRevision);
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(NeutralAnimationMarkerV1 {
            time_microseconds: reader.u64()?,
            marker_id: SchemaId::new(reader.string()?)?,
        });
    }
    reader.finish()?;
    Ok(values)
}

pub(super) fn append_len(
    bytes: &mut Vec<u8>,
    length: usize,
) -> Result<(), NeutralAnimationContentErrorV1> {
    bytes.extend_from_slice(
        &u32::try_from(length)
            .map_err(|_| NeutralAnimationContentErrorV1::LimitOrRevision)?
            .to_le_bytes(),
    );
    Ok(())
}

pub(super) fn append_string(
    bytes: &mut Vec<u8>,
    value: &str,
) -> Result<(), NeutralAnimationContentErrorV1> {
    append_len(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}
impl<'a> Reader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
    pub(super) fn bytes(
        &mut self,
        length: usize,
    ) -> Result<&'a [u8], NeutralAnimationContentErrorV1> {
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(NeutralAnimationContentErrorV1::Decode)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or(NeutralAnimationContentErrorV1::Decode)?;
        self.cursor = end;
        Ok(value)
    }
    pub(super) fn u8(&mut self) -> Result<u8, NeutralAnimationContentErrorV1> {
        Ok(self.bytes(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, NeutralAnimationContentErrorV1> {
        Ok(u16::from_le_bytes(
            self.bytes(2)?
                .try_into()
                .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
        ))
    }
    pub(super) fn u32(&mut self) -> Result<u32, NeutralAnimationContentErrorV1> {
        Ok(u32::from_le_bytes(
            self.bytes(4)?
                .try_into()
                .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
        ))
    }
    pub(super) fn i32(&mut self) -> Result<i32, NeutralAnimationContentErrorV1> {
        Ok(i32::from_le_bytes(
            self.bytes(4)?
                .try_into()
                .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
        ))
    }
    pub(super) fn u64(&mut self) -> Result<u64, NeutralAnimationContentErrorV1> {
        Ok(u64::from_le_bytes(
            self.bytes(8)?
                .try_into()
                .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
        ))
    }
    pub(super) fn i64(&mut self) -> Result<i64, NeutralAnimationContentErrorV1> {
        Ok(i64::from_le_bytes(
            self.bytes(8)?
                .try_into()
                .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
        ))
    }
    pub(super) fn len(&mut self) -> Result<usize, NeutralAnimationContentErrorV1> {
        usize::try_from(self.u32()?).map_err(|_| NeutralAnimationContentErrorV1::Decode)
    }
    pub(super) fn string(&mut self) -> Result<&'a str, NeutralAnimationContentErrorV1> {
        let length = self.len()?;
        std::str::from_utf8(self.bytes(length)?).map_err(|_| NeutralAnimationContentErrorV1::Decode)
    }
    pub(super) fn finish(self) -> Result<(), NeutralAnimationContentErrorV1> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(NeutralAnimationContentErrorV1::Decode)
        }
    }
}

pub(super) fn field(
    segment: &DecodedCanonicalSegment,
    id: u32,
    tag: u8,
) -> Result<&[u8], NeutralAnimationContentErrorV1> {
    segment
        .fields
        .iter()
        .find(|field| field.field_id == id && field.type_tag == tag)
        .map(|field| field.payload.as_slice())
        .ok_or(NeutralAnimationContentErrorV1::EnvelopeMismatch)
}
pub(super) fn read_u32(bytes: &[u8]) -> Result<u32, NeutralAnimationContentErrorV1> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
    ))
}
pub(super) fn read_u64(bytes: &[u8]) -> Result<u64, NeutralAnimationContentErrorV1> {
    Ok(u64::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| NeutralAnimationContentErrorV1::Decode)?,
    ))
}
pub(super) fn read_fixed<const N: usize>(
    bytes: &[u8],
) -> Result<[u8; N], NeutralAnimationContentErrorV1> {
    bytes
        .try_into()
        .map_err(|_| NeutralAnimationContentErrorV1::Decode)
}
pub(super) fn read_string(bytes: &[u8]) -> Result<&str, NeutralAnimationContentErrorV1> {
    std::str::from_utf8(bytes).map_err(|_| NeutralAnimationContentErrorV1::Decode)
}
pub(super) fn require_envelope(
    segment: &DecodedCanonicalSegment,
    schema: &str,
    id: &str,
    fields: usize,
) -> Result<(), NeutralAnimationContentErrorV1> {
    if segment.owner_id == NEUTRAL_ANIMATION_OWNER_ID
        && segment.schema_id == schema
        && segment.segment_id == id
        && segment.fields.len() == fields
    {
        Ok(())
    } else {
        Err(NeutralAnimationContentErrorV1::EnvelopeMismatch)
    }
}
