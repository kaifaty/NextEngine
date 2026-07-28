use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_STRUCT, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment,
};
use crate::ids::{ContentHash, PersistentId, content_hash_from_bytes};

use super::body::{
    COMMAND_BODY_OWNER_ID, COMMAND_BODY_SCHEMA_ID, COMMAND_BODY_SEGMENT_ID, CapabilityRefV1,
    CommandPreconditionV1,
};
use super::error::CommandDecodeError;

pub(super) fn validate_command_body_envelope(
    segment: &DecodedCanonicalSegment,
) -> Result<(), CommandDecodeError> {
    if segment.owner_id != COMMAND_BODY_OWNER_ID
        || segment.schema_id != COMMAND_BODY_SCHEMA_ID
        || segment.segment_id != COMMAND_BODY_SEGMENT_ID
    {
        return Err(CommandDecodeError::WrongEnvelope);
    }
    Ok(())
}

pub(super) fn field(
    fields: &[CanonicalField],
    field_id: u32,
) -> Result<&CanonicalField, CommandDecodeError> {
    fields
        .binary_search_by_key(&field_id, |field| field.field_id)
        .ok()
        .map(|index| &fields[index])
        .ok_or(CommandDecodeError::MissingField(field_id))
}

pub(super) fn validate_exact_fields(
    fields: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), CommandDecodeError> {
    for field in fields {
        let Some((_, expected_type)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(CommandDecodeError::UnknownField(field.field_id));
        };
        if field.type_tag != *expected_type {
            return Err(CommandDecodeError::FieldType {
                field_id: field.field_id,
                expected: *expected_type,
                actual: field.type_tag,
            });
        }
    }
    for (field_id, _) in expected {
        if fields.iter().all(|field| field.field_id != *field_id) {
            return Err(CommandDecodeError::MissingField(*field_id));
        }
    }
    Ok(())
}

pub(super) fn decode_exact<const LENGTH: usize>(
    bytes: &[u8],
) -> Result<[u8; LENGTH], CommandDecodeError> {
    bytes
        .try_into()
        .map_err(|_| CommandDecodeError::FieldLength {
            field_id: 0,
            expected: LENGTH,
            actual: bytes.len(),
        })
}

pub(super) fn decode_u8(bytes: &[u8]) -> Result<u8, CommandDecodeError> {
    Ok(decode_exact::<1>(bytes)?[0])
}

pub(super) fn decode_u16(bytes: &[u8]) -> Result<u16, CommandDecodeError> {
    Ok(u16::from_le_bytes(decode_exact(bytes)?))
}

pub(super) fn decode_u32(bytes: &[u8]) -> Result<u32, CommandDecodeError> {
    Ok(u32::from_le_bytes(decode_exact(bytes)?))
}

pub(super) fn decode_u64(bytes: &[u8]) -> Result<u64, CommandDecodeError> {
    Ok(u64::from_le_bytes(decode_exact(bytes)?))
}

pub(super) fn decode_utf8(bytes: &[u8]) -> Result<&str, CommandDecodeError> {
    std::str::from_utf8(bytes)
        .map_err(|_| CommandDecodeError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn encode_nested_value(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.push(type_tag);
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

pub(super) fn read_nested_value<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &'a [u8]), CanonicalDecodeError> {
    let type_tag = cursor.read_u8()?;
    let length =
        usize::try_from(cursor.read_u64()?).map_err(|_| CanonicalDecodeError::LengthOverflow)?;
    if length > limits.max_field_payload_bytes {
        return Err(CanonicalDecodeError::FieldPayloadTooLarge {
            field_id: 0,
            actual: length,
            limit: limits.max_field_payload_bytes,
        });
    }
    Ok((type_tag, cursor.read_exact(length)?))
}

pub(super) fn encode_struct_payload(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if let Some(pair) = fields
        .windows(2)
        .find(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(pair[0].field_id));
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

pub(super) fn decode_struct_payload(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, CommandDecodeError> {
    let mut cursor = CanonicalCursor::new(payload);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let field_id = cursor.read_u32()?;
        if let Some(previous) = previous {
            if field_id == previous {
                return Err(CommandDecodeError::Canonical(
                    CanonicalDecodeError::DuplicateField(field_id),
                ));
            }
            if field_id < previous {
                return Err(CommandDecodeError::Canonical(
                    CanonicalDecodeError::FieldsNotStrictlySorted {
                        previous,
                        actual: field_id,
                    },
                ));
            }
        }
        previous = Some(field_id);
        let type_tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| CanonicalDecodeError::LengthOverflow)?;
        if length > limits.max_field_payload_bytes {
            return Err(CommandDecodeError::Canonical(
                CanonicalDecodeError::FieldPayloadTooLarge {
                    field_id,
                    actual: length,
                    limit: limits.max_field_payload_bytes,
                },
            ));
        }
        fields.push(CanonicalField::new(
            field_id,
            type_tag,
            cursor.read_exact(length)?.to_vec(),
        ));
    }
    cursor.finish()?;
    Ok(fields)
}

pub(super) fn encode_optional_id(value: Option<&PersistentId>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut bytes = vec![1];
    bytes.extend_from_slice(&encode_nested_value(
        CANONICAL_TYPE_ID128,
        value.as_bytes(),
    )?);
    Ok(bytes)
}

pub(super) fn decode_optional_id(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<PersistentId>, CommandDecodeError> {
    let mut cursor = CanonicalCursor::new(payload);
    match cursor.read_u8()? {
        0 => {
            cursor.finish()?;
            Ok(None)
        }
        1 => {
            let (type_tag, bytes) = read_nested_value(&mut cursor, limits)?;
            cursor.finish()?;
            if type_tag != CANONICAL_TYPE_ID128 {
                return Err(CommandDecodeError::InvalidNestedType {
                    expected: CANONICAL_TYPE_ID128,
                    actual: type_tag,
                });
            }
            Ok(Some(PersistentId::from_bytes(decode_exact(bytes)?)))
        }
        _ => Err(CommandDecodeError::InvalidOptional),
    }
}

pub(super) fn encode_optional_hash(value: Option<&ContentHash>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut bytes = vec![1];
    bytes.extend_from_slice(&encode_nested_value(
        CANONICAL_TYPE_HASH256,
        value.as_bytes(),
    )?);
    Ok(bytes)
}

pub(super) fn decode_optional_hash(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<ContentHash>, CommandDecodeError> {
    let mut cursor = CanonicalCursor::new(payload);
    match cursor.read_u8()? {
        0 => {
            cursor.finish()?;
            Ok(None)
        }
        1 => {
            let (type_tag, bytes) = read_nested_value(&mut cursor, limits)?;
            cursor.finish()?;
            if type_tag != CANONICAL_TYPE_HASH256 {
                return Err(CommandDecodeError::InvalidNestedType {
                    expected: CANONICAL_TYPE_HASH256,
                    actual: type_tag,
                });
            }
            Ok(Some(content_hash_from_bytes(decode_exact(bytes)?)))
        }
        _ => Err(CommandDecodeError::InvalidOptional),
    }
}

pub(super) fn encode_canonical_set(mut records: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    records.sort();
    if records.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        bytes.extend_from_slice(&record);
    }
    Ok(bytes)
}

pub(super) fn encode_precondition_set(
    preconditions: &[CommandPreconditionV1],
) -> Result<Vec<u8>, CanonicalError> {
    let mut keyed = preconditions
        .iter()
        .map(|precondition| {
            Ok((
                precondition.canonical_key_without_constraint()?,
                precondition.canonical_record()?,
            ))
        })
        .collect::<Result<Vec<_>, CanonicalError>>()?;
    keyed.sort_by(|left, right| left.1.cmp(&right.1));
    if keyed.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    encode_canonical_set(keyed.into_iter().map(|(_, record)| record).collect())
}

struct DecodedSetRecord<'a> {
    type_tag: u8,
    payload: &'a [u8],
    encoded: Vec<u8>,
}

fn decode_set_records(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<DecodedSetRecord<'_>>, CommandDecodeError> {
    let mut cursor = CanonicalCursor::new(payload);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let (type_tag, nested_payload) = read_nested_value(&mut cursor, limits)?;
        let encoded = encode_nested_value(type_tag, nested_payload)?;
        records.push(DecodedSetRecord {
            type_tag,
            payload: nested_payload,
            encoded,
        });
    }
    cursor.finish()?;
    if records
        .windows(2)
        .any(|pair| pair[0].encoded.as_slice() >= pair[1].encoded.as_slice())
    {
        return Err(CommandDecodeError::SetNotStrictlySorted);
    }
    Ok(records)
}

pub(super) fn decode_capability_set(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CapabilityRefV1>, CommandDecodeError> {
    decode_set_records(payload, limits)?
        .into_iter()
        .map(|record| {
            if record.type_tag != CANONICAL_TYPE_STRUCT {
                return Err(CommandDecodeError::InvalidNestedType {
                    expected: CANONICAL_TYPE_STRUCT,
                    actual: record.type_tag,
                });
            }
            CapabilityRefV1::from_struct_payload(record.payload, limits)
        })
        .collect()
}

pub(super) fn decode_precondition_set(
    payload: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CommandPreconditionV1>, CommandDecodeError> {
    let values = decode_set_records(payload, limits)?
        .into_iter()
        .map(|record| {
            if record.type_tag != CANONICAL_TYPE_STRUCT {
                return Err(CommandDecodeError::InvalidNestedType {
                    expected: CANONICAL_TYPE_STRUCT,
                    actual: record.type_tag,
                });
            }
            CommandPreconditionV1::from_struct_payload(record.payload, limits)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let keys = values
        .iter()
        .map(CommandPreconditionV1::canonical_key_without_constraint)
        .collect::<Result<Vec<_>, _>>()?;
    if keys
        .iter()
        .enumerate()
        .any(|(index, key)| keys.iter().skip(index + 1).any(|other| other == key))
    {
        return Err(CommandDecodeError::ConflictingPreconditions);
    }
    Ok(values)
}
