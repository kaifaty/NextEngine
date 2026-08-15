use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SET,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits,
    CanonicalError, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{CapabilityId, CommandStreamId, ContentHash, PersistentId, SchemaId};
use crate::physics::{
    PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_COMMAND_SCHEMA_VERSION, PhysicalCommandV1,
};
use crate::rpg::RPG_COMMAND_SCHEMA_ID;
use crate::rpg::{RPG_TRANSACTION_COMMAND_SCHEMA_VERSION, RpgCommandV1};
use crate::world_routine::{
    WORLD_ROUTINE_COMMAND_SCHEMA_ID, WORLD_ROUTINE_COMMAND_SCHEMA_VERSION, WorldRoutineCommandV1,
};

use super::codec::{
    decode_capability_set, decode_exact, decode_optional_hash, decode_optional_id,
    decode_precondition_set, decode_struct_payload, decode_u8, decode_u16, decode_u32, decode_u64,
    decode_utf8, encode_canonical_set, encode_nested_value, encode_optional_hash,
    encode_optional_id, encode_precondition_set, encode_struct_payload, field,
    validate_command_body_envelope, validate_exact_fields,
};
use super::error::CommandDecodeError;
use super::principal::IssuerPrincipal;

pub const COMMAND_BODY_SCHEMA_VERSION: u16 = 2;
pub const COMMAND_SCHEMA_VERSION: u32 = 1;
pub const NOOP_COMMAND_SCHEMA_ID: &str = "nextengine.command.noop";
pub const NOOP_COMMAND_CAPABILITY_ID: &str = "runtime.command.noop";
pub const COMMAND_BODY_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_BODY_SCHEMA_ID: &str = "nextengine.canonical-command-body";
pub const COMMAND_BODY_SEGMENT_ID: &str = "v2";
const REVISION_PRECONDITION_KIND: &str = "runtime.authoritative-revision";
const REVISION_PRECONDITION_OWNER: &str = "nextengine.runtime";
const REVISION_PRECONDITION_SCHEMA: &str = "nextengine.runtime.state";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CommandPhase {
    Ingress = 0,
    Outcome = 1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CommandPayload {
    Noop,
    Rpg(RpgCommandV1),
    Physical(PhysicalCommandV1),
    WorldRoutine(WorldRoutineCommandV1),
}

impl CommandPayload {
    fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        match self {
            Self::Noop => Ok(Vec::new()),
            Self::Rpg(command) => command.canonical_payload_bytes(),
            Self::Physical(command) => command.canonical_payload_bytes(),
            Self::WorldRoutine(command) => command.canonical_payload_bytes(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapabilityRefV1 {
    pub capability_id: CapabilityId,
    pub scope_hash: Option<ContentHash>,
}

impl CapabilityRefV1 {
    pub fn unscoped(capability_id: impl Into<String>) -> Result<Self, crate::ids::IdentifierError> {
        Ok(Self {
            capability_id: CapabilityId::new(capability_id)?,
            scope_hash: None,
        })
    }

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let scope = encode_optional_hash(self.scope_hash.as_ref())?;
        let payload = encode_struct_payload([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.capability_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(2, CANONICAL_TYPE_OPTIONAL, scope),
        ])?;
        encode_nested_value(CANONICAL_TYPE_STRUCT, &payload)
    }

    pub(super) fn from_struct_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandDecodeError> {
        let fields = decode_struct_payload(payload, limits)?;
        validate_exact_fields(
            &fields,
            &[(1, CANONICAL_TYPE_UTF8_NFC), (2, CANONICAL_TYPE_OPTIONAL)],
        )?;
        Ok(Self {
            capability_id: CapabilityId::new(decode_utf8(field(&fields, 1)?.payload.as_slice())?)?,
            scope_hash: decode_optional_hash(field(&fields, 2)?.payload.as_slice(), limits)?,
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandPreconditionV1 {
    pub precondition_kind: SchemaId,
    pub owner_id: SchemaId,
    pub target_schema_id: SchemaId,
    pub target_key: Vec<u8>,
    pub expected_revision: u64,
    pub constraint_hash: Option<ContentHash>,
}

impl CommandPreconditionV1 {
    pub fn authoritative_revision(expected_revision: u64) -> Result<Self, CanonicalError> {
        Ok(Self {
            precondition_kind: SchemaId::new(REVISION_PRECONDITION_KIND)?,
            owner_id: SchemaId::new(REVISION_PRECONDITION_OWNER)?,
            target_schema_id: SchemaId::new(REVISION_PRECONDITION_SCHEMA)?,
            target_key: Vec::new(),
            expected_revision,
            constraint_hash: None,
        })
    }

    #[must_use]
    pub fn is_authoritative_revision(&self) -> bool {
        self.precondition_kind.as_str() == REVISION_PRECONDITION_KIND
            && self.owner_id.as_str() == REVISION_PRECONDITION_OWNER
            && self.target_schema_id.as_str() == REVISION_PRECONDITION_SCHEMA
            && self.target_key.is_empty()
            && self.constraint_hash.is_none()
    }

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let constraint = encode_optional_hash(self.constraint_hash.as_ref())?;
        let payload = encode_struct_payload([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.precondition_kind.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_UTF8_NFC,
                self.owner_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_UTF8_NFC,
                self.target_schema_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(4, CANONICAL_TYPE_BYTES, self.target_key.clone()),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U64,
                self.expected_revision.to_le_bytes().to_vec(),
            ),
            CanonicalField::new(6, CANONICAL_TYPE_OPTIONAL, constraint),
        ])?;
        encode_nested_value(CANONICAL_TYPE_STRUCT, &payload)
    }

    pub(super) fn canonical_key_without_constraint(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct_payload([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.precondition_kind.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                2,
                CANONICAL_TYPE_UTF8_NFC,
                self.owner_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_UTF8_NFC,
                self.target_schema_id.as_str().as_bytes().to_vec(),
            ),
            CanonicalField::new(4, CANONICAL_TYPE_BYTES, self.target_key.clone()),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_U64,
                self.expected_revision.to_le_bytes().to_vec(),
            ),
        ])
    }

    pub(super) fn from_struct_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandDecodeError> {
        let fields = decode_struct_payload(payload, limits)?;
        validate_exact_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_UTF8_NFC),
                (4, CANONICAL_TYPE_BYTES),
                (5, CANONICAL_TYPE_U64),
                (6, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        Ok(Self {
            precondition_kind: SchemaId::new(decode_utf8(&field(&fields, 1)?.payload)?)?,
            owner_id: SchemaId::new(decode_utf8(&field(&fields, 2)?.payload)?)?,
            target_schema_id: SchemaId::new(decode_utf8(&field(&fields, 3)?.payload)?)?,
            target_key: field(&fields, 4)?.payload.clone(),
            expected_revision: decode_u64(&field(&fields, 5)?.payload)?,
            constraint_hash: decode_optional_hash(&field(&fields, 6)?.payload, limits)?,
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalCommandBodyV2 {
    pub payload_schema_id: SchemaId,
    pub payload_schema_version: u32,
    pub issuer: IssuerPrincipal,
    pub stream_id: CommandStreamId,
    pub sequence: u64,
    pub target_tick: u64,
    pub phase: CommandPhase,
    pub target: Option<PersistentId>,
    pub capability_claims: Vec<CapabilityRefV1>,
    pub preconditions: Vec<CommandPreconditionV1>,
    pub payload: CommandPayload,
}

impl CanonicalCommandBodyV2 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let target = encode_optional_id(self.target.as_ref())?;
        let capabilities = encode_canonical_set(
            self.capability_claims
                .iter()
                .map(CapabilityRefV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let preconditions = encode_precondition_set(&self.preconditions)?;
        encode_canonical_segment(
            COMMAND_BODY_OWNER_ID,
            COMMAND_BODY_SCHEMA_ID,
            COMMAND_BODY_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    COMMAND_BODY_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.payload_schema_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U32,
                    self.payload_schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_TAGGED_UNION,
                    self.issuer.tagged_union_payload()?,
                ),
                CanonicalField::new(5, CANONICAL_TYPE_ID128, self.stream_id.as_bytes().to_vec()),
                CanonicalField::new(6, CANONICAL_TYPE_U64, self.sequence.to_le_bytes().to_vec()),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_U64,
                    self.target_tick.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(8, CANONICAL_TYPE_U8, vec![self.phase as u8]),
                CanonicalField::new(9, CANONICAL_TYPE_OPTIONAL, target),
                CanonicalField::new(10, CANONICAL_TYPE_SET, capabilities),
                CanonicalField::new(11, CANONICAL_TYPE_SET, preconditions),
                CanonicalField::new(12, CANONICAL_TYPE_BYTES, self.payload.canonical_bytes()?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_command_body_envelope(&segment)?;
        validate_exact_fields(
            &segment.fields,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_TAGGED_UNION),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_U8),
                (9, CANONICAL_TYPE_OPTIONAL),
                (10, CANONICAL_TYPE_SET),
                (11, CANONICAL_TYPE_SET),
                (12, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let body_schema_version = decode_u16(&field(&segment.fields, 1)?.payload)?;
        if body_schema_version != COMMAND_BODY_SCHEMA_VERSION {
            return Err(CommandDecodeError::UnsupportedBodySchemaVersion(
                body_schema_version,
            ));
        }
        let payload_schema_id = SchemaId::new(decode_utf8(&field(&segment.fields, 2)?.payload)?)?;
        let payload_schema_version = decode_u32(&field(&segment.fields, 3)?.payload)?;
        let issuer = IssuerPrincipal::from_tagged_union_payload(
            &field(&segment.fields, 4)?.payload,
            limits,
        )?;
        let stream_id =
            CommandStreamId::from_bytes(decode_exact::<16>(&field(&segment.fields, 5)?.payload)?);
        let sequence = decode_u64(&field(&segment.fields, 6)?.payload)?;
        let target_tick = decode_u64(&field(&segment.fields, 7)?.payload)?;
        let phase = match decode_u8(&field(&segment.fields, 8)?.payload)? {
            0 => CommandPhase::Ingress,
            1 => CommandPhase::Outcome,
            value => return Err(CommandDecodeError::InvalidPhase(value)),
        };
        let target = decode_optional_id(&field(&segment.fields, 9)?.payload, limits)?;
        let capability_claims =
            decode_capability_set(&field(&segment.fields, 10)?.payload, limits)?;
        let preconditions = decode_precondition_set(&field(&segment.fields, 11)?.payload, limits)?;
        let payload_bytes = &field(&segment.fields, 12)?.payload;
        let payload = match (payload_schema_id.as_str(), payload_schema_version) {
            (NOOP_COMMAND_SCHEMA_ID, COMMAND_SCHEMA_VERSION) if payload_bytes.is_empty() => {
                CommandPayload::Noop
            }
            (NOOP_COMMAND_SCHEMA_ID, COMMAND_SCHEMA_VERSION) => {
                return Err(CommandDecodeError::InvalidNoopPayload);
            }
            (RPG_COMMAND_SCHEMA_ID, RPG_TRANSACTION_COMMAND_SCHEMA_VERSION) => CommandPayload::Rpg(
                RpgCommandV1::from_canonical_payload_bytes(payload_bytes, limits)?,
            ),
            (PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_COMMAND_SCHEMA_VERSION) => {
                CommandPayload::Physical(PhysicalCommandV1::from_canonical_payload_bytes(
                    payload_bytes,
                    limits,
                )?)
            }
            (WORLD_ROUTINE_COMMAND_SCHEMA_ID, WORLD_ROUTINE_COMMAND_SCHEMA_VERSION) => {
                CommandPayload::WorldRoutine(WorldRoutineCommandV1::from_canonical_payload_bytes(
                    payload_bytes,
                    limits,
                )?)
            }
            (
                NOOP_COMMAND_SCHEMA_ID
                | RPG_COMMAND_SCHEMA_ID
                | PHYSICAL_COMMAND_SCHEMA_ID
                | WORLD_ROUTINE_COMMAND_SCHEMA_ID,
                version,
            ) => {
                return Err(CommandDecodeError::UnsupportedPayloadSchemaVersion(version));
            }
            (schema, _) => return Err(CommandDecodeError::UnknownPayloadSchema(schema.to_owned())),
        };
        let body = Self {
            payload_schema_id,
            payload_schema_version,
            issuer,
            stream_id,
            sequence,
            target_tick,
            phase,
            target,
            capability_claims,
            preconditions,
            payload,
        };
        if body.canonical_bytes()? != bytes {
            return Err(CommandDecodeError::NonCanonicalEncoding);
        }
        Ok(body)
    }
}
