use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::*;
use crate::command::WorldCommand;
use crate::ids::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InputContractError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identifier(crate::IdentifierError),
    Principal(crate::PrincipalDecodeError),
    Command(crate::CommandDecodeError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType,
    FieldLength,
    InvalidOptional,
    InvalidValue,
    InvalidProfile,
    ResourceLimit,
    NonCanonicalOrder,
    DuplicateKey,
    RegistryCollision,
    HashMismatch,
    UnsupportedCompletionSignal,
    UnsupportedVersion {
        contract: &'static str,
        version: u32,
    },
    UnknownTag(u8),
    NonCanonicalEncoding,
}

impl InputContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion { .. } => "INPUT_CONTRACT_UNSUPPORTED_VERSION",
            Self::ResourceLimit => "INPUT_RESOURCE_LIMIT",
            Self::HashMismatch => "INGRESS_ASSIGNMENT_CORRUPT",
            Self::RegistryCollision => "PLAYER_CONTROLLER_REGISTRY_CORRUPT",
            _ => "INPUT_CONTRACT_INVALID",
        }
    }
}

impl Display for InputContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "canonical input is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "canonical input could not be encoded: {error}")
            }
            Self::Identifier(error) => write!(formatter, "input identifier is invalid: {error}"),
            Self::Principal(error) => write!(formatter, "input principal is invalid: {error}"),
            Self::Command(error) => write!(formatter, "input command is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("input contract envelope does not match"),
            Self::UnknownField(id) => write!(formatter, "unknown input field {id}"),
            Self::MissingField(id) => write!(formatter, "missing input field {id}"),
            Self::FieldType => formatter.write_str("input field has the wrong canonical type"),
            Self::FieldLength => formatter.write_str("input field has the wrong length"),
            Self::InvalidOptional => formatter.write_str("input optional value is invalid"),
            Self::InvalidValue => formatter.write_str("input value is invalid"),
            Self::InvalidProfile => formatter.write_str("input profile is invalid"),
            Self::ResourceLimit => formatter.write_str("input exceeds a bound profile limit"),
            Self::NonCanonicalOrder => {
                formatter.write_str("input collection order is not canonical")
            }
            Self::DuplicateKey => formatter.write_str("input map contains a duplicate key"),
            Self::RegistryCollision => {
                formatter.write_str("controller registry contains a collision")
            }
            Self::HashMismatch => formatter.write_str("input hash does not match its body"),
            Self::UnsupportedCompletionSignal => {
                formatter.write_str("completion signals are not active in this runtime slice")
            }
            Self::UnsupportedVersion { contract, version } => {
                write!(formatter, "unsupported {contract} version {version}")
            }
            Self::UnknownTag(tag) => write!(formatter, "unknown input tag {tag}"),
            Self::NonCanonicalEncoding => {
                formatter.write_str("input does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for InputContractError {}

impl From<CanonicalDecodeError> for InputContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for InputContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<crate::IdentifierError> for InputContractError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::PrincipalDecodeError> for InputContractError {
    fn from(error: crate::PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<crate::CommandDecodeError> for InputContractError {
    fn from(error: crate::CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

pub(super) fn hash_canonical_profile(bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
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

pub(super) fn closed_batch_hash(
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

pub(super) fn encode_command_envelope(command: &WorldCommand) -> Result<Vec<u8>, CanonicalError> {
    encode_struct([
        field_u16(1, command.envelope_schema_version),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_OPTIONAL,
            encode_optional_id(command.claimed_command_id.as_ref())?,
        ),
        CanonicalField::new(3, CANONICAL_TYPE_BYTES, command.canonical_bytes()?),
    ])
}

pub(super) fn decode_command_envelope(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<WorldCommand, InputContractError> {
    let fields = decode_struct(bytes, limits)?;
    require_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_OPTIONAL),
            (3, CANONICAL_TYPE_BYTES),
        ],
    )?;
    let mut command = WorldCommand::from_canonical_bytes(&field_from(&fields, 3)?.payload, limits)?;
    command.envelope_schema_version = read_u16_fields(&fields, 1)?;
    command.claimed_command_id = decode_optional_id(&field_from(&fields, 2)?.payload, limits)?;
    Ok(command)
}

pub(super) fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
    fields: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, InputContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != segment_id
    {
        return Err(InputContractError::WrongEnvelope);
    }
    require_fields(&segment.fields, fields)?;
    Ok(segment)
}

pub(super) fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), InputContractError> {
    for field in actual {
        let Some((_, expected_tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(InputContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *expected_tag {
            return Err(InputContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(InputContractError::MissingField(*id));
        }
    }
    Ok(())
}

pub(super) fn field(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<&[u8], InputContractError> {
    Ok(&segment
        .field(id)
        .ok_or(InputContractError::MissingField(id))?
        .payload)
}

pub(super) fn field_from(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&CanonicalField, InputContractError> {
    fields
        .binary_search_by_key(&id, |field| field.field_id)
        .ok()
        .map(|index| &fields[index])
        .ok_or(InputContractError::MissingField(id))
}

pub(super) fn read_u8(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u8, InputContractError> {
    Ok(exact::<1>(field(segment, id)?)?[0])
}

pub(super) fn read_u16(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u16, InputContractError> {
    Ok(u16::from_le_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_u32(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u32, InputContractError> {
    Ok(u32::from_le_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_u64(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<u64, InputContractError> {
    Ok(u64::from_le_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_hash(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<ContentHash, InputContractError> {
    Ok(content_hash_from_bytes(exact(field(segment, id)?)?))
}

pub(super) fn read_utf8(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<&str, InputContractError> {
    std::str::from_utf8(field(segment, id)?)
        .map_err(|_| InputContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn read_u8_fields(fields: &[CanonicalField], id: u32) -> Result<u8, InputContractError> {
    Ok(exact::<1>(&field_from(fields, id)?.payload)?[0])
}

fn read_u16_fields(fields: &[CanonicalField], id: u32) -> Result<u16, InputContractError> {
    Ok(u16::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_u32_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<u32, InputContractError> {
    Ok(u32::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_u64_fields(
    fields: &[CanonicalField],
    id: u32,
) -> Result<u64, InputContractError> {
    Ok(u64::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

pub(super) fn read_utf8_field(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&str, InputContractError> {
    std::str::from_utf8(&field_from(fields, id)?.payload)
        .map_err(|_| InputContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

pub(super) fn exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], InputContractError> {
    bytes
        .try_into()
        .map_err(|_| InputContractError::FieldLength)
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

pub(super) fn field_id<const N: usize>(id: u32, value: &[u8; N]) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.to_vec())
}

pub(super) fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

pub(super) fn encode_nested(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
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
) -> Result<(u8, &'a [u8]), InputContractError> {
    let type_tag = cursor.read_u8()?;
    let length =
        usize::try_from(cursor.read_u64()?).map_err(|_| InputContractError::FieldLength)?;
    if length > limits.max_field_payload_bytes {
        return Err(InputContractError::ResourceLimit);
    }
    Ok((type_tag, cursor.read_exact(length)?))
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
) -> Result<Vec<CanonicalField>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let field_id = cursor.read_u32()?;
        if previous.is_some_and(|previous| field_id <= previous) {
            return Err(InputContractError::NonCanonicalOrder);
        }
        previous = Some(field_id);
        let type_tag = cursor.read_u8()?;
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| InputContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(InputContractError::ResourceLimit);
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
) -> Result<Vec<Vec<u8>>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| InputContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(InputContractError::ResourceLimit);
        }
        items.push(cursor.read_exact(length)?.to_vec());
    }
    cursor.finish()?;
    Ok(items)
}

pub(super) fn encode_optional_hash(value: Option<&ContentHash>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut bytes = vec![1];
    bytes.extend_from_slice(&encode_nested(CANONICAL_TYPE_HASH256, value.as_bytes())?);
    Ok(bytes)
}

pub(super) fn decode_optional_hash(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<ContentHash>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    match cursor.read_u8()? {
        0 => {
            cursor.finish()?;
            Ok(None)
        }
        1 => {
            let (tag, payload) = read_nested(&mut cursor, limits)?;
            cursor.finish()?;
            if tag != CANONICAL_TYPE_HASH256 {
                return Err(InputContractError::FieldType);
            }
            Ok(Some(content_hash_from_bytes(exact(payload)?)))
        }
        _ => Err(InputContractError::InvalidOptional),
    }
}

pub(super) fn encode_optional_id(value: Option<&CommandId>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut bytes = vec![1];
    bytes.extend_from_slice(&encode_nested(CANONICAL_TYPE_ID128, value.as_bytes())?);
    Ok(bytes)
}

pub(super) fn decode_optional_id(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<CommandId>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    match cursor.read_u8()? {
        0 => {
            cursor.finish()?;
            Ok(None)
        }
        1 => {
            let (tag, payload) = read_nested(&mut cursor, limits)?;
            cursor.finish()?;
            if tag != CANONICAL_TYPE_ID128 {
                return Err(InputContractError::FieldType);
            }
            Ok(Some(CommandId::from_bytes(exact(payload)?)))
        }
        _ => Err(InputContractError::InvalidOptional),
    }
}

pub(super) fn require_round_trip(
    original: &[u8],
    encoded: Vec<u8>,
) -> Result<(), InputContractError> {
    if original != encoded {
        return Err(InputContractError::NonCanonicalEncoding);
    }
    Ok(())
}

pub(super) fn is_non_decreasing_by<T, K, E>(
    values: &[T],
    mut key: impl FnMut(&T) -> Result<K, E>,
) -> bool
where
    K: Ord,
{
    let mut previous = None;
    for value in values {
        let Ok(actual) = key(value) else {
            return false;
        };
        if previous.as_ref().is_some_and(|previous| previous > &actual) {
            return false;
        }
        previous = Some(actual);
    }
    true
}
