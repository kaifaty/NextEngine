use crate::canonical::{
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, DecodedCanonicalSegment,
};
use crate::{ContentHash, IssuerPrincipal};

use super::error::IdentityContractError;

type EncodedMapEntries = Vec<(Vec<u8>, Vec<u8>)>;

pub(super) fn require_envelope(
    segment: &DecodedCanonicalSegment,
    owner: &str,
    schema: &str,
    id: &str,
) -> Result<(), IdentityContractError> {
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != id {
        return Err(IdentityContractError::WrongEnvelope);
    }
    Ok(())
}

pub(super) fn require_fields(
    segment: &DecodedCanonicalSegment,
    expected: &[(u32, u8)],
) -> Result<(), IdentityContractError> {
    for actual in &segment.fields {
        if !expected.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(IdentityContractError::UnknownField(actual.field_id));
        }
    }
    for (id, expected_type) in expected {
        let actual = segment
            .field(*id)
            .ok_or(IdentityContractError::MissingField(*id))?;
        if actual.type_tag != *expected_type {
            return Err(IdentityContractError::WrongFieldType {
                field_id: *id,
                expected: *expected_type,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

pub(super) fn field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<&CanonicalField, IdentityContractError> {
    segment
        .field(field_id)
        .ok_or(IdentityContractError::MissingField(field_id))
}

pub(super) fn read_array<const N: usize>(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<[u8; N], IdentityContractError> {
    field(segment, field_id)?
        .payload
        .as_slice()
        .try_into()
        .map_err(|_| IdentityContractError::InvalidFieldLength {
            field_id,
            expected: N,
            actual: field(segment, field_id)
                .map(|field| field.payload.len())
                .unwrap_or_default(),
        })
}

pub(super) fn read_u16(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u16, IdentityContractError> {
    Ok(u16::from_le_bytes(read_array(segment, field_id)?))
}

pub(super) fn read_u32(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<u32, IdentityContractError> {
    Ok(u32::from_le_bytes(read_array(segment, field_id)?))
}

pub(super) fn read_hash(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<ContentHash, IdentityContractError> {
    Ok(ContentHash::from_bytes(read_array(segment, field_id)?))
}

pub(super) fn read_text(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<&str, IdentityContractError> {
    std::str::from_utf8(&field(segment, field_id)?.payload)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn nested_value(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = vec![type_tag];
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

pub(super) fn nested_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    let mut payload = Vec::new();
    payload.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        payload.extend_from_slice(&field.field_id.to_le_bytes());
        payload.push(field.type_tag);
        payload.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        payload.extend_from_slice(&field.payload);
    }
    nested_value(CANONICAL_TYPE_STRUCT, &payload)
}

pub(super) fn encode_map(mut entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<Vec<u8>, CanonicalError> {
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(entries.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for (key, value) in entries {
        bytes.extend_from_slice(&key);
        bytes.extend_from_slice(&value);
    }
    Ok(bytes)
}

fn decode_nested_value<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &'a [u8]), IdentityContractError> {
    let tag = cursor.read_u8()?;
    let length = usize::try_from(cursor.read_u64()?)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
    if length > limits.max_field_payload_bytes {
        return Err(IdentityContractError::Decode(
            CanonicalDecodeError::FieldPayloadTooLarge {
                field_id: 0,
                actual: length,
                limit: limits.max_field_payload_bytes,
            },
        ));
    }
    Ok((tag, cursor.read_exact(length)?))
}

pub(super) fn decode_map(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<EncodedMapEntries, IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut result = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let (key_tag, key_payload) = decode_nested_value(&mut cursor, limits)?;
        let key = nested_value(key_tag, key_payload)?;
        if previous.as_ref().is_some_and(|prior| prior >= &key) {
            return Err(IdentityContractError::DuplicateKey);
        }
        let (value_tag, value_payload) = decode_nested_value(&mut cursor, limits)?;
        let value = nested_value(value_tag, value_payload)?;
        previous = Some(key.clone());
        result.push((key, value));
    }
    cursor.finish()?;
    Ok(result)
}

pub(super) fn decode_nested_struct(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, IdentityContractError> {
    let mut outer = CanonicalCursor::new(bytes);
    let (tag, payload) = decode_nested_value(&mut outer, limits)?;
    outer.finish()?;
    if tag != CANONICAL_TYPE_STRUCT {
        return Err(IdentityContractError::WrongFieldType {
            field_id: 0,
            expected: CANONICAL_TYPE_STRUCT,
            actual: tag,
        });
    }
    let mut cursor = CanonicalCursor::new(payload);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let field_id = cursor.read_u32()?;
        if previous.is_some_and(|id| id >= field_id) {
            return Err(IdentityContractError::DuplicateKey);
        }
        let type_tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::LengthOverflow))?;
        let payload = cursor.read_exact(length)?.to_vec();
        fields.push(CanonicalField::new(field_id, type_tag, payload));
        previous = Some(field_id);
    }
    cursor.finish()?;
    Ok(fields)
}

pub(super) fn require_nested_fields(
    fields: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), IdentityContractError> {
    for actual in fields {
        if !expected.iter().any(|(id, _)| *id == actual.field_id) {
            return Err(IdentityContractError::UnknownField(actual.field_id));
        }
    }
    for (id, tag) in expected {
        let actual = fields
            .iter()
            .find(|field| field.field_id == *id)
            .ok_or(IdentityContractError::MissingField(*id))?;
        if actual.type_tag != *tag {
            return Err(IdentityContractError::WrongFieldType {
                field_id: *id,
                expected: *tag,
                actual: actual.type_tag,
            });
        }
    }
    Ok(())
}

fn nested_field(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&CanonicalField, IdentityContractError> {
    fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(IdentityContractError::MissingField(id))
}

pub(super) fn nested_payload(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&[u8], IdentityContractError> {
    Ok(&nested_field(fields, id)?.payload)
}

pub(super) fn nested_array<const N: usize>(
    fields: &[CanonicalField],
    id: u32,
) -> Result<[u8; N], IdentityContractError> {
    nested_payload(fields, id)?
        .try_into()
        .map_err(|_| IdentityContractError::InvalidFieldLength {
            field_id: id,
            expected: N,
            actual: nested_payload(fields, id).map_or(0, <[u8]>::len),
        })
}

pub(super) fn nested_text(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&str, IdentityContractError> {
    std::str::from_utf8(nested_payload(fields, id)?)
        .map_err(|_| IdentityContractError::Decode(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn nested_u8(fields: &[CanonicalField], id: u32) -> Result<u8, IdentityContractError> {
    Ok(nested_array::<1>(fields, id)?[0])
}

pub(super) fn nested_u32(fields: &[CanonicalField], id: u32) -> Result<u32, IdentityContractError> {
    Ok(u32::from_le_bytes(nested_array(fields, id)?))
}

pub(super) fn decode_nested_fixed<const N: usize>(
    bytes: &[u8],
    expected_tag: u8,
) -> Result<[u8; N], IdentityContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let tag = cursor.read_u8()?;
    let length = cursor.read_u64()?;
    if tag != expected_tag {
        return Err(IdentityContractError::WrongFieldType {
            field_id: 0,
            expected: expected_tag,
            actual: tag,
        });
    }
    if length != N as u64 {
        return Err(IdentityContractError::InvalidFieldLength {
            field_id: 0,
            expected: N,
            actual: usize::try_from(length).unwrap_or(usize::MAX),
        });
    }
    let value = cursor.read_exact(N)?.try_into().map_err(|_| {
        IdentityContractError::InvalidFieldLength {
            field_id: 0,
            expected: N,
            actual: 0,
        }
    })?;
    cursor.finish()?;
    Ok(value)
}

pub(super) fn decode_nested_id(
    bytes: &[u8],
    expected_tag: u8,
) -> Result<[u8; 16], IdentityContractError> {
    decode_nested_fixed(bytes, expected_tag)
}

pub(super) fn principal_union_payload(
    principal: &IssuerPrincipal,
) -> Result<Vec<u8>, CanonicalError> {
    let full = principal.canonical_bytes()?;
    let mut cursor = CanonicalCursor::new(&full);
    let tag = cursor
        .read_u8()
        .map_err(|_| CanonicalError::LengthOverflow)?;
    if tag != CANONICAL_TYPE_TAGGED_UNION {
        return Err(CanonicalError::LengthOverflow);
    }
    let length = usize::try_from(
        cursor
            .read_u64()
            .map_err(|_| CanonicalError::LengthOverflow)?,
    )
    .map_err(|_| CanonicalError::LengthOverflow)?;
    let payload = cursor
        .read_exact(length)
        .map_err(|_| CanonicalError::LengthOverflow)?
        .to_vec();
    cursor
        .finish()
        .map_err(|_| CanonicalError::LengthOverflow)?;
    Ok(payload)
}

pub(super) fn principal_from_union_payload(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<IssuerPrincipal, IdentityContractError> {
    IssuerPrincipal::from_canonical_bytes(
        &nested_value(CANONICAL_TYPE_TAGGED_UNION, payload)?,
        limits,
    )
    .map_err(Into::into)
}
