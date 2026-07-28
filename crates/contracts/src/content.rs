use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::CanonicalCursor;
use crate::{
    AssetId, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP,
    CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8, CANONICAL_TYPE_U32, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, ContentHash, PersistentId,
    ProjectContractError, SchemaEncodingV1, SchemaId, SchemaRefV1, SchemaRoleV1,
    decode_canonical_segment, domain_hash, encode_canonical_segment,
};

pub const NEUTRAL_RECORD_OWNER_ID: &str = "nextengine.assets";
pub const NEUTRAL_RECORD_SEGMENT_ID: &str = "nextengine.neutral-record.v1";
pub const NEUTRAL_RECORD_SCHEMA_VERSION: u32 = 1;
pub const NEUTRAL_RECORD_MAX_REFERENCES: usize = 1_024;
pub const NEUTRAL_RECORD_MAX_PROPERTIES: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NeutralRecordKindV1 {
    Scene = 1,
    Collider = 2,
    CharacterDefinition = 3,
    ItemDefinition = 4,
    InventoryDefinition = 5,
    EquipmentDefinition = 6,
    DialogueDefinition = 7,
    QuestDefinition = 8,
    RelationshipDefinition = 9,
    InteractionDefinition = 10,
    AbilityDefinition = 11,
}

impl NeutralRecordKindV1 {
    #[must_use]
    pub const fn schema_id(self) -> &'static str {
        match self {
            Self::Scene => "nextengine.content.scene.v1",
            Self::Collider => "nextengine.content.collider.v1",
            Self::CharacterDefinition => "nextengine.content.character-definition.v1",
            Self::ItemDefinition => "nextengine.content.item-definition.v1",
            Self::InventoryDefinition => "nextengine.content.inventory-definition.v1",
            Self::EquipmentDefinition => "nextengine.content.equipment-definition.v1",
            Self::DialogueDefinition => "nextengine.content.dialogue-definition.v1",
            Self::QuestDefinition => "nextengine.content.quest-definition.v1",
            Self::RelationshipDefinition => "nextengine.content.relationship-definition.v1",
            Self::InteractionDefinition => "nextengine.content.interaction-definition.v1",
            Self::AbilityDefinition => "nextengine.content.ability-definition.v1",
        }
    }

    fn from_tag(value: u8) -> Result<Self, NeutralRecordError> {
        match value {
            1 => Ok(Self::Scene),
            2 => Ok(Self::Collider),
            3 => Ok(Self::CharacterDefinition),
            4 => Ok(Self::ItemDefinition),
            5 => Ok(Self::InventoryDefinition),
            6 => Ok(Self::EquipmentDefinition),
            7 => Ok(Self::DialogueDefinition),
            8 => Ok(Self::QuestDefinition),
            9 => Ok(Self::RelationshipDefinition),
            10 => Ok(Self::InteractionDefinition),
            11 => Ok(Self::AbilityDefinition),
            _ => Err(NeutralRecordError::UnknownKind),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralPropertyV1 {
    pub property_id: SchemaId,
    pub value_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralRecordV1 {
    pub schema_ref: SchemaRefV1,
    pub asset_id: AssetId,
    pub kind: NeutralRecordKindV1,
    pub record_id: PersistentId,
    pub persistent_references: Vec<PersistentId>,
    pub asset_dependencies: Vec<AssetId>,
    pub properties: Vec<NeutralPropertyV1>,
}

impl NeutralRecordV1 {
    pub fn new(
        schema_ref: SchemaRefV1,
        asset_id: AssetId,
        kind: NeutralRecordKindV1,
        record_id: PersistentId,
        mut persistent_references: Vec<PersistentId>,
        mut asset_dependencies: Vec<AssetId>,
        mut properties: Vec<NeutralPropertyV1>,
    ) -> Result<Self, NeutralRecordError> {
        schema_ref.validate()?;
        if schema_ref.schema_id.as_str() != kind.schema_id()
            || schema_ref.role != SchemaRoleV1::Definition
            || schema_ref.encoding != SchemaEncodingV1::CanonicalBinaryV1
        {
            return Err(NeutralRecordError::SchemaMismatch);
        }
        persistent_references.sort();
        asset_dependencies.sort();
        properties.sort();
        ensure_unique(&persistent_references)?;
        ensure_unique(&asset_dependencies)?;
        ensure_unique_by(&properties, |property| property.property_id.as_str())?;
        enforce_limit(persistent_references.len(), NEUTRAL_RECORD_MAX_REFERENCES)?;
        enforce_limit(asset_dependencies.len(), NEUTRAL_RECORD_MAX_REFERENCES)?;
        enforce_limit(properties.len(), NEUTRAL_RECORD_MAX_PROPERTIES)?;
        Ok(Self {
            schema_ref,
            asset_id,
            kind,
            record_id,
            persistent_references,
            asset_dependencies,
            properties,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, NeutralRecordError> {
        let canonical = Self::new(
            self.schema_ref.clone(),
            self.asset_id,
            self.kind,
            self.record_id,
            self.persistent_references.clone(),
            self.asset_dependencies.clone(),
            self.properties.clone(),
        )?;
        Ok(encode_canonical_segment(
            NEUTRAL_RECORD_OWNER_ID,
            canonical.schema_ref.schema_id.as_str(),
            NEUTRAL_RECORD_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    NEUTRAL_RECORD_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_ID128,
                    canonical.asset_id.as_bytes().to_vec(),
                ),
                CanonicalField::new(3, CANONICAL_TYPE_U8, vec![canonical.kind as u8]),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_ID128,
                    canonical.record_id.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_fixed_ids(
                        canonical
                            .persistent_references
                            .iter()
                            .map(PersistentId::as_bytes),
                    )?,
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_fixed_ids(canonical.asset_dependencies.iter().map(AssetId::as_bytes))?,
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_MAP,
                    encode_properties(&canonical.properties)?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_HASH256,
                    canonical.schema_ref.descriptor_sha256.as_bytes().to_vec(),
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, NeutralRecordError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        if segment.owner_id != NEUTRAL_RECORD_OWNER_ID
            || segment.segment_id != NEUTRAL_RECORD_SEGMENT_ID
            || segment.fields.len() != 8
        {
            return Err(NeutralRecordError::EnvelopeMismatch);
        }
        let version = read_u32(field(&segment, 1, CANONICAL_TYPE_U32)?)?;
        if version != NEUTRAL_RECORD_SCHEMA_VERSION {
            return Err(NeutralRecordError::UnsupportedVersion(version));
        }
        let asset_id = AssetId::from_bytes(read_fixed(field(&segment, 2, CANONICAL_TYPE_ID128)?)?);
        let kind = NeutralRecordKindV1::from_tag(read_u8(field(&segment, 3, CANONICAL_TYPE_U8)?)?)?;
        if segment.schema_id != kind.schema_id() {
            return Err(NeutralRecordError::SchemaMismatch);
        }
        let record_id =
            PersistentId::from_bytes(read_fixed(field(&segment, 4, CANONICAL_TYPE_ID128)?)?);
        let persistent_references = decode_fixed_ids(
            field(&segment, 5, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            PersistentId::from_bytes,
        )?;
        let asset_dependencies = decode_fixed_ids(
            field(&segment, 6, CANONICAL_TYPE_SEQUENCE)?,
            limits,
            AssetId::from_bytes,
        )?;
        let properties = decode_properties(field(&segment, 7, CANONICAL_TYPE_MAP)?, limits)?;
        let descriptor_sha256 =
            ContentHash::from_bytes(read_fixed(field(&segment, 8, CANONICAL_TYPE_HASH256)?)?);
        let record = Self::new(
            SchemaRefV1 {
                schema_id: SchemaId::new(segment.schema_id)?,
                schema_version: version,
                descriptor_sha256,
                role: SchemaRoleV1::Definition,
                encoding: SchemaEncodingV1::CanonicalBinaryV1,
            },
            asset_id,
            kind,
            record_id,
            persistent_references,
            asset_dependencies,
            properties,
        )?;
        if record.canonical_bytes()? != bytes {
            return Err(NeutralRecordError::NonCanonical);
        }
        Ok(record)
    }

    pub fn record_sha256(&self) -> Result<ContentHash, NeutralRecordError> {
        Ok(domain_hash(
            "nextengine.neutral-record.v1",
            &self.canonical_bytes()?,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NeutralRecordError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(crate::IdentifierError),
    Project(ProjectContractError),
    SchemaMismatch,
    EnvelopeMismatch,
    UnsupportedVersion(u32),
    UnknownKind,
    DuplicateIdentity,
    LimitExceeded { actual: usize, limit: usize },
    WrongFieldType(u32),
    InvalidPayload,
    NonCanonical,
}

impl Display for NeutralRecordError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "neutral record encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "neutral record decode failed: {error}"),
            Self::Identifier(error) => write!(formatter, "neutral record ID is invalid: {error}"),
            Self::Project(error) => write!(formatter, "neutral record schema is invalid: {error}"),
            Self::SchemaMismatch => formatter.write_str("neutral record schema/kind mismatch"),
            Self::EnvelopeMismatch => formatter.write_str("neutral record envelope mismatch"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported neutral record version {version}")
            }
            Self::UnknownKind => formatter.write_str("unknown neutral record kind"),
            Self::DuplicateIdentity => formatter.write_str("duplicate neutral record identity"),
            Self::LimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "neutral record count {actual} exceeds limit {limit}"
                )
            }
            Self::WrongFieldType(field_id) => {
                write!(formatter, "neutral record field {field_id} has wrong type")
            }
            Self::InvalidPayload => formatter.write_str("neutral record payload is invalid"),
            Self::NonCanonical => formatter.write_str("neutral record bytes are not canonical"),
        }
    }
}

impl Error for NeutralRecordError {}

impl From<CanonicalError> for NeutralRecordError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for NeutralRecordError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<crate::IdentifierError> for NeutralRecordError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<ProjectContractError> for NeutralRecordError {
    fn from(error: ProjectContractError) -> Self {
        Self::Project(error)
    }
}

fn field(
    segment: &crate::DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&[u8], NeutralRecordError> {
    let field = segment
        .field(field_id)
        .ok_or(NeutralRecordError::InvalidPayload)?;
    if field.type_tag != expected_type {
        return Err(NeutralRecordError::WrongFieldType(field_id));
    }
    Ok(&field.payload)
}

fn read_u8(bytes: &[u8]) -> Result<u8, NeutralRecordError> {
    bytes
        .first()
        .copied()
        .filter(|_| bytes.len() == 1)
        .ok_or(NeutralRecordError::InvalidPayload)
}

fn read_u32(bytes: &[u8]) -> Result<u32, NeutralRecordError> {
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| NeutralRecordError::InvalidPayload)?,
    ))
}

fn read_fixed<const LENGTH: usize>(bytes: &[u8]) -> Result<[u8; LENGTH], NeutralRecordError> {
    bytes
        .try_into()
        .map_err(|_| NeutralRecordError::InvalidPayload)
}

fn encode_fixed_ids<'a, const LENGTH: usize>(
    values: impl IntoIterator<Item = &'a [u8; LENGTH]>,
) -> Result<Vec<u8>, NeutralRecordError> {
    let values: Vec<_> = values.into_iter().collect();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(values.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for value in values {
        bytes.extend_from_slice(value);
    }
    Ok(bytes)
}

fn decode_fixed_ids<T, const LENGTH: usize>(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    construct: impl Fn([u8; LENGTH]) -> T,
) -> Result<Vec<T>, NeutralRecordError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count =
        usize::try_from(cursor.read_u32()?).map_err(|_| NeutralRecordError::InvalidPayload)?;
    if count > limits.max_sequence_items {
        return Err(NeutralRecordError::LimitExceeded {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(construct(read_fixed(cursor.read_exact(LENGTH)?)?));
    }
    cursor.finish()?;
    Ok(values)
}

fn encode_properties(properties: &[NeutralPropertyV1]) -> Result<Vec<u8>, NeutralRecordError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(properties.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for property in properties {
        extend_text(&mut bytes, property.property_id.as_str())?;
        extend_text(&mut bytes, property.value_id.as_str())?;
    }
    Ok(bytes)
}

fn decode_properties(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralPropertyV1>, NeutralRecordError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count =
        usize::try_from(cursor.read_u32()?).map_err(|_| NeutralRecordError::InvalidPayload)?;
    if count > limits.max_sequence_items {
        return Err(NeutralRecordError::LimitExceeded {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    let mut properties = Vec::with_capacity(count);
    for _ in 0..count {
        properties.push(NeutralPropertyV1 {
            property_id: read_text(&mut cursor, limits)?,
            value_id: read_text(&mut cursor, limits)?,
        });
    }
    cursor.finish()?;
    Ok(properties)
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), NeutralRecordError> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn read_text(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, NeutralRecordError> {
    let length =
        usize::try_from(cursor.read_u32()?).map_err(|_| NeutralRecordError::InvalidPayload)?;
    if length > limits.max_identifier_bytes {
        return Err(NeutralRecordError::LimitExceeded {
            actual: length,
            limit: limits.max_identifier_bytes,
        });
    }
    let bytes = cursor.read_exact(length)?;
    let value = std::str::from_utf8(bytes).map_err(|_| NeutralRecordError::InvalidPayload)?;
    Ok(SchemaId::new(value)?)
}

fn ensure_unique<T: Ord>(values: &[T]) -> Result<(), NeutralRecordError> {
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(NeutralRecordError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn ensure_unique_by<T>(values: &[T], key: impl Fn(&T) -> &str) -> Result<(), NeutralRecordError> {
    if values.windows(2).any(|pair| key(&pair[0]) == key(&pair[1])) {
        Err(NeutralRecordError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn enforce_limit(actual: usize, limit: usize) -> Result<(), NeutralRecordError> {
    if actual > limit {
        Err(NeutralRecordError::LimitExceeded { actual, limit })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{NeutralPropertyV1, NeutralRecordKindV1, NeutralRecordV1};
    use crate::{
        AssetId, CanonicalDecodeLimits, PersistentId, SchemaEncodingV1, SchemaId, SchemaRefV1,
        SchemaRoleV1, domain_hash,
    };

    #[test]
    fn every_neutral_kind_round_trips_canonically() {
        let kinds = [
            NeutralRecordKindV1::Scene,
            NeutralRecordKindV1::Collider,
            NeutralRecordKindV1::CharacterDefinition,
            NeutralRecordKindV1::ItemDefinition,
            NeutralRecordKindV1::InventoryDefinition,
            NeutralRecordKindV1::EquipmentDefinition,
            NeutralRecordKindV1::DialogueDefinition,
            NeutralRecordKindV1::QuestDefinition,
            NeutralRecordKindV1::RelationshipDefinition,
            NeutralRecordKindV1::InteractionDefinition,
            NeutralRecordKindV1::AbilityDefinition,
        ];
        for (index, kind) in kinds.into_iter().enumerate() {
            let record = NeutralRecordV1::new(
                SchemaRefV1 {
                    schema_id: SchemaId::new(kind.schema_id()).expect("schema"),
                    schema_version: 1,
                    descriptor_sha256: domain_hash(
                        "nextengine.schema-descriptor.v1",
                        kind.schema_id().as_bytes(),
                    ),
                    role: SchemaRoleV1::Definition,
                    encoding: SchemaEncodingV1::CanonicalBinaryV1,
                },
                AssetId::from_bytes([u8::try_from(index + 1).expect("small"); 16]),
                kind,
                PersistentId::from_bytes([u8::try_from(index + 21).expect("small"); 16]),
                Vec::new(),
                Vec::new(),
                vec![NeutralPropertyV1 {
                    property_id: SchemaId::new("nextengine.test.key").expect("key"),
                    value_id: SchemaId::new("nextengine.test.value").expect("value"),
                }],
            )
            .expect("record");
            let bytes = record.canonical_bytes().expect("encode");
            assert_eq!(
                NeutralRecordV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                    .expect("decode"),
                record
            );
        }
    }
}
