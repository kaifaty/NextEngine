use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I64, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, sha256,
};
use crate::ids::{ContentHash, content_hash_from_bytes};

use super::error::PhysicsContractError;

pub(super) fn encode_i64_vec3(values: [i64; 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(24);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

pub(super) fn decode_i64_vec3(bytes: &[u8]) -> Result<[i64; 3], PhysicsContractError> {
    if bytes.len() != 24 {
        return Err(PhysicsContractError::FieldLength);
    }
    Ok([
        i64::from_le_bytes(exact(&bytes[0..8])?),
        i64::from_le_bytes(exact(&bytes[8..16])?),
        i64::from_le_bytes(exact(&bytes[16..24])?),
    ])
}

pub(super) fn encode_i32_vec3(values: [i32; 3]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

pub(super) fn decode_i32_vec3(bytes: &[u8]) -> Result<[i32; 3], PhysicsContractError> {
    if bytes.len() != 12 {
        return Err(PhysicsContractError::FieldLength);
    }
    Ok([
        i32::from_le_bytes(exact(&bytes[0..4])?),
        i32::from_le_bytes(exact(&bytes[4..8])?),
        i32::from_le_bytes(exact(&bytes[8..12])?),
    ])
}

pub(super) fn physics_contract_hash(
    domain: &[u8],
    bytes: &[u8],
) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub(super) fn profile_hash(bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.profile.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

pub(super) fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
    fields: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, PhysicsContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != segment_id
    {
        return Err(PhysicsContractError::WrongEnvelope);
    }
    require_fields(&segment.fields, fields)?;
    Ok(segment)
}

pub(super) fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), PhysicsContractError> {
    for field in actual {
        let Some((_, expected_tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(PhysicsContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *expected_tag {
            return Err(PhysicsContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(PhysicsContractError::MissingField(*id));
        }
    }
    Ok(())
}

pub(super) fn field(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<&[u8], PhysicsContractError> {
    Ok(&segment
        .field(id)
        .ok_or(PhysicsContractError::MissingField(id))?
        .payload)
}

pub(super) fn field_from(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&CanonicalField, PhysicsContractError> {
    fields
        .binary_search_by_key(&id, |field| field.field_id)
        .ok()
        .map(|index| &fields[index])
        .ok_or(PhysicsContractError::MissingField(id))
}

pub(super) fn read_u8(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u8, PhysicsContractError> {
    Ok(exact::<1>(field(segment, id)?)?[0])
}

pub(super) fn read_u16(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u16, PhysicsContractError> {
    Ok(u16::from_le_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_u64(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u64, PhysicsContractError> {
    Ok(u64::from_le_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_bool(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<bool, PhysicsContractError> {
    match read_u8(segment, id)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PhysicsContractError::FieldType),
    }
}

pub(super) fn read_hash(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<ContentHash, PhysicsContractError> {
    Ok(content_hash_from_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_utf8(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<&str, PhysicsContractError> {
    std::str::from_utf8(field(segment, id)?)
        .map_err(|_| PhysicsContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn read_u8_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<u8, PhysicsContractError> {
    Ok(exact::<1>(&field_from(fields, id)?.payload)?[0])
}

pub(super) fn read_u16_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<u16, PhysicsContractError> {
    Ok(u16::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_u32_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<u32, PhysicsContractError> {
    Ok(u32::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_u64_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<u64, PhysicsContractError> {
    Ok(u64::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_i64_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<i64, PhysicsContractError> {
    Ok(i64::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_bool_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<bool, PhysicsContractError> {
    match read_u8_fields(fields, id)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(PhysicsContractError::FieldType),
    }
}

pub(super) fn read_utf8_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&str, PhysicsContractError> {
    std::str::from_utf8(&field_from(fields, id)?.payload)
        .map_err(|_| PhysicsContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn read_hash_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<ContentHash, PhysicsContractError> {
    Ok(content_hash_from_bytes(exact(
        &field_from(fields, id)?.payload,
    )?))
}

pub(super) fn exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], PhysicsContractError> {
    bytes
        .try_into()
        .map_err(|_| PhysicsContractError::FieldLength)
}

pub(super) fn field_u8(id: u32, value: u8) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U8, vec![value])
}

pub(super) fn field_u16(id: u32, value: u16) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U16, value.to_le_bytes().to_vec())
}

pub(super) fn field_u32(id: u32, value: u32) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U32, value.to_le_bytes().to_vec())
}

pub(super) fn field_u64(id: u32, value: u64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U64, value.to_le_bytes().to_vec())
}

pub(super) fn field_i64(id: u32, value: i64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_I64, value.to_le_bytes().to_vec())
}

pub(super) fn field_bool(id: u32, value: bool) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_BOOL, vec![u8::from(value)])
}

pub(super) fn field_id<const N: usize>(id: u32, value: &[u8; N]) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.to_vec())
}

pub(super) fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

pub(super) fn nested(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = vec![type_tag];
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

pub(super) fn read_nested<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &'a [u8]), PhysicsContractError> {
    let tag = cursor.read_u8()?;
    let length =
        usize::try_from(cursor.read_u64()?).map_err(|_| PhysicsContractError::FieldLength)?;
    if length > limits.max_field_payload_bytes {
        return Err(PhysicsContractError::FieldLength);
    }
    Ok((tag, cursor.read_exact(length)?))
}

pub(super) fn encode_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if fields
        .windows(2)
        .any(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(0));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        bytes.extend_from_slice(&field.field_id.to_le_bytes());
        bytes.push(field.type_tag);
        bytes.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&field.payload);
    }
    Ok(bytes)
}

pub(super) fn decode_struct(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, PhysicsContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let id = cursor.read_u32()?;
        if previous.is_some_and(|previous| id <= previous) {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        previous = Some(id);
        let tag = cursor.read_u8()?;
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| PhysicsContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(PhysicsContractError::FieldLength);
        }
        fields.push(CanonicalField::new(
            id,
            tag,
            cursor.read_exact(length)?.to_vec(),
        ));
    }
    cursor.finish()?;
    Ok(fields)
}

pub(super) fn encode_sequence(items: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(items.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for item in items {
        bytes.extend_from_slice(
            &u64::try_from(item.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&item);
    }
    Ok(bytes)
}

pub(super) fn decode_sequence(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<u8>>, PhysicsContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| PhysicsContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(PhysicsContractError::FieldLength);
        }
        items.push(cursor.read_exact(length)?.to_vec());
    }
    cursor.finish()?;
    Ok(items)
}

pub(super) fn require_round_trip(
    original: &[u8],
    encoded: Vec<u8>,
) -> Result<(), PhysicsContractError> {
    if original != encoded {
        return Err(PhysicsContractError::NonCanonicalEncoding);
    }
    Ok(())
}
