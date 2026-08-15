use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC,
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, sha256,
};
use crate::ids::{
    AssetId, ContentHash, IdentifierError, PersistentId, SchemaId, content_hash_from_bytes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldPopulationContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    UnsupportedVersion(u16),
    UnknownTier(u8),
    UnknownCadence(u8),
    UnknownNavigationCapability(u8),
    UnknownCommand(u8),
    PopulationContentInvalid,
    NavigationContentInvalid,
    SnapshotClosureInvalid,
    RoutePlanInvalid,
    RouteStale,
    NodeUnavailable,
    RouteUnavailable,
    PhysicalTraversalRequired,
    CommandInvalid,
    RevisionExhausted,
    WrongEnvelope,
    MissingField(u32),
    UnknownField(u32),
    FieldType,
    FieldLength,
    NonCanonicalEncoding,
}

impl WorldPopulationContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::RouteStale => "NAVIGATION_ROUTE_STALE",
            Self::NodeUnavailable => "NAVIGATION_NODE_UNAVAILABLE",
            Self::RouteUnavailable => "NAVIGATION_ROUTE_UNAVAILABLE",
            Self::PhysicalTraversalRequired => "PHYSICAL_TRAVERSAL_REQUIRED",
            Self::RevisionExhausted => "WORLD_POPULATION_REVISION_EXHAUSTED",
            _ => "WORLD_POPULATION_CONTRACT_INVALID",
        }
    }
}

impl Display for WorldPopulationContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for WorldPopulationContractError {}

impl From<CanonicalError> for WorldPopulationContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for WorldPopulationContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for WorldPopulationContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

pub(super) fn domain_hash(domain: &str, bytes: &[u8]) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .expect("in-memory population bytes fit u64")
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    content_hash_from_bytes(sha256(&preimage))
}

pub(super) fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    !values.windows(2).any(|pair| pair[0] >= pair[1])
}

pub(super) fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
    fields: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, WorldPopulationContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != segment_id
    {
        return Err(WorldPopulationContractError::WrongEnvelope);
    }
    require_fields(&segment.fields, fields)?;
    Ok(segment)
}

pub(super) fn field(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<&[u8], WorldPopulationContractError> {
    Ok(&segment
        .field(id)
        .ok_or(WorldPopulationContractError::MissingField(id))?
        .payload)
}

pub(super) fn nested_field(
    fields: &[CanonicalField],
    id: u32,
) -> Result<&[u8], WorldPopulationContractError> {
    Ok(&fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(WorldPopulationContractError::MissingField(id))?
        .payload)
}

pub(super) fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), WorldPopulationContractError> {
    for field in actual {
        let Some((_, tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(WorldPopulationContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *tag {
            return Err(WorldPopulationContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(WorldPopulationContractError::MissingField(*id));
        }
    }
    Ok(())
}

pub(super) fn encode_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields = fields.into_iter().collect::<Vec<_>>();
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
) -> Result<Vec<CanonicalField>, WorldPopulationContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let id = cursor.read_u32()?;
        if previous.is_some_and(|previous| id <= previous) {
            return Err(WorldPopulationContractError::NonCanonicalEncoding);
        }
        previous = Some(id);
        let tag = cursor.read_u8()?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| WorldPopulationContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(WorldPopulationContractError::FieldLength);
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

pub(super) fn encode_sequence(records: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        bytes.push(CANONICAL_TYPE_STRUCT);
        bytes.extend_from_slice(
            &u64::try_from(record.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&record);
    }
    Ok(bytes)
}

pub(super) fn decode_sequence(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<u8>>, WorldPopulationContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        if cursor.read_u8()? != CANONICAL_TYPE_STRUCT {
            return Err(WorldPopulationContractError::FieldType);
        }
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| WorldPopulationContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(WorldPopulationContractError::FieldLength);
        }
        records.push(cursor.read_exact(length)?.to_vec());
    }
    cursor.finish()?;
    Ok(records)
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

pub(super) fn field_id(id: u32, value: AssetId) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.as_bytes().to_vec())
}

pub(super) fn field_persistent_id(id: u32, value: PersistentId) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.as_bytes().to_vec())
}

pub(super) fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

pub(super) fn field_text(id: u32, value: &SchemaId) -> CanonicalField {
    CanonicalField::new(
        id,
        CANONICAL_TYPE_UTF8_NFC,
        value.as_str().as_bytes().to_vec(),
    )
}

pub(super) fn read_exact<const N: usize>(
    bytes: &[u8],
) -> Result<[u8; N], WorldPopulationContractError> {
    bytes
        .try_into()
        .map_err(|_| WorldPopulationContractError::FieldLength)
}

pub(super) fn read_cursor_exact<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], WorldPopulationContractError> {
    read_exact(cursor.read_exact(N)?)
}

pub(super) fn read_u8(bytes: &[u8]) -> Result<u8, WorldPopulationContractError> {
    Ok(read_exact::<1>(bytes)?[0])
}

pub(super) fn read_u16(bytes: &[u8]) -> Result<u16, WorldPopulationContractError> {
    Ok(u16::from_le_bytes(read_exact(bytes)?))
}

pub(super) fn read_u32(bytes: &[u8]) -> Result<u32, WorldPopulationContractError> {
    Ok(u32::from_le_bytes(read_exact(bytes)?))
}

pub(super) fn read_u64(bytes: &[u8]) -> Result<u64, WorldPopulationContractError> {
    Ok(u64::from_le_bytes(read_exact(bytes)?))
}

pub(super) fn read_hash(bytes: &[u8]) -> Result<ContentHash, WorldPopulationContractError> {
    Ok(ContentHash::from_bytes(read_exact(bytes)?))
}

pub(super) fn read_schema_id(bytes: &[u8]) -> Result<SchemaId, WorldPopulationContractError> {
    Ok(SchemaId::new(std::str::from_utf8(bytes).map_err(
        |_| WorldPopulationContractError::PopulationContentInvalid,
    )?)?)
}

pub(super) fn read_cursor_schema_id(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, WorldPopulationContractError> {
    let length = usize::try_from(cursor.read_u32()?)
        .map_err(|_| WorldPopulationContractError::FieldLength)?;
    if length > limits.max_field_payload_bytes {
        return Err(WorldPopulationContractError::FieldLength);
    }
    read_schema_id(cursor.read_exact(length)?)
}

pub(super) fn append_text(
    bytes: &mut Vec<u8>,
    value: &str,
) -> Result<(), WorldPopulationContractError> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| WorldPopulationContractError::FieldLength)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}
