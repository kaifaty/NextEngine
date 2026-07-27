use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL,
    CANONICAL_TYPE_SET, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC,
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::ids::{
    CapabilityId, CommandBodyHash, CommandId, CommandStreamId, ContentHash, EventId,
    MechanicPackageId, PersistentId, PlayerPrincipalId, PluginId, SchemaId, ScriptPrincipalId,
    SystemId, ToolPrincipalId, command_body_hash_from_bytes, content_hash_from_bytes,
};
use crate::rpg::{
    RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID, RpgCommand, RpgDecodeError, RpgEvent,
};
use crate::{
    PHYSICAL_COMMAND_CAPABILITY_ID, PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_COMMAND_SCHEMA_VERSION,
    PhysicalCommandV1, PhysicalEventV1, PhysicsContractError,
};

pub const COMMAND_BODY_SCHEMA_VERSION: u16 = 2;
pub const COMMAND_ENVELOPE_SCHEMA_VERSION: u16 = 2;
pub const COMMAND_SCHEMA_VERSION: u32 = 1;
pub const NOOP_COMMAND_SCHEMA_ID: &str = "nextengine.command.noop";
pub const NOOP_COMMAND_CAPABILITY_ID: &str = "runtime.command.noop";
pub const COMMAND_BODY_OWNER_ID: &str = "nextengine.runtime";
pub const COMMAND_BODY_SCHEMA_ID: &str = "nextengine.canonical-command-body";
pub const COMMAND_BODY_SEGMENT_ID: &str = "v2";
const EVENT_SCHEMA_ID: &str = "nextengine.event.command-committed";
const EVENT_OWNER_ID: &str = "nextengine.runtime";
const EVENT_SEGMENT_ID: &str = "v1";
const EVENT_ENVELOPE_SCHEMA_ID: &str = "nextengine.domain-event-envelope";
const EVENT_ENVELOPE_SEGMENT_ID: &str = "v2";
const REVISION_PRECONDITION_KIND: &str = "runtime.authoritative-revision";
const REVISION_PRECONDITION_OWNER: &str = "nextengine.runtime";
const REVISION_PRECONDITION_SCHEMA: &str = "nextengine.runtime.state";

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IssuerPrincipal {
    Player(PlayerPrincipalId),
    Agent(PersistentId),
    Package(MechanicPackageId),
    Script(ScriptPrincipalId),
    Plugin(PluginId),
    Tool(ToolPrincipalId),
    InternalSystem(SystemId),
}

impl IssuerPrincipal {
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::Player(_) => 0x01,
            Self::Agent(_) => 0x02,
            Self::Package(_) => 0x03,
            Self::Script(_) => 0x04,
            Self::Plugin(_) => 0x05,
            Self::Tool(_) => 0x06,
            Self::InternalSystem(_) => 0x07,
        }
    }

    #[must_use]
    pub fn identifier_bytes(&self) -> &[u8] {
        match self {
            Self::Player(id) => id.as_bytes(),
            Self::Agent(id) => id.as_bytes(),
            Self::Package(id) => id.as_str().as_bytes(),
            Self::Script(id) => id.as_str().as_bytes(),
            Self::Plugin(id) => id.as_str().as_bytes(),
            Self::Tool(id) => id.as_str().as_bytes(),
            Self::InternalSystem(id) => id.as_str().as_bytes(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_nested_value(CANONICAL_TYPE_TAGGED_UNION, &self.tagged_union_payload()?)
    }

    fn tagged_union_payload(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut payload = vec![self.tag()];
        let nested = match self {
            Self::Player(id) => encode_nested_value(CANONICAL_TYPE_ID128, id.as_bytes())?,
            Self::Agent(id) => encode_nested_value(CANONICAL_TYPE_ID128, id.as_bytes())?,
            Self::Package(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
            Self::Script(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
            Self::Plugin(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
            Self::Tool(id) => encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?,
            Self::InternalSystem(id) => {
                encode_nested_value(CANONICAL_TYPE_UTF8_NFC, id.as_str().as_bytes())?
            }
        };
        payload.extend_from_slice(&nested);
        Ok(payload)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PrincipalDecodeError> {
        if bytes.len() > limits.max_field_payload_bytes {
            return Err(PrincipalDecodeError::InputTooLarge {
                actual: bytes.len(),
                limit: limits.max_field_payload_bytes,
            });
        }
        let mut cursor = CanonicalCursor::new(bytes);
        let (type_tag, payload) = read_nested_value(&mut cursor, limits)?;
        cursor.finish()?;
        if type_tag != CANONICAL_TYPE_TAGGED_UNION {
            return Err(PrincipalDecodeError::WrongType(type_tag));
        }
        Self::from_tagged_union_payload(payload, limits)
    }

    fn from_tagged_union_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PrincipalDecodeError> {
        let mut cursor = CanonicalCursor::new(payload);
        let tag = cursor.read_u8()?;
        let (type_tag, nested) = read_nested_value(&mut cursor, limits)?;
        cursor.finish()?;
        let principal = match tag {
            0x01 => Self::Player(PlayerPrincipalId::from_bytes(decode_id128(
                type_tag, nested,
            )?)),
            0x02 => Self::Agent(PersistentId::from_bytes(decode_id128(type_tag, nested)?)),
            0x03 => Self::Package(MechanicPackageId::new(decode_text(type_tag, nested)?)?),
            0x04 => Self::Script(ScriptPrincipalId::new(decode_text(type_tag, nested)?)?),
            0x05 => Self::Plugin(PluginId::new(decode_text(type_tag, nested)?)?),
            0x06 => Self::Tool(ToolPrincipalId::new(decode_text(type_tag, nested)?)?),
            0x07 => Self::InternalSystem(SystemId::new(decode_text(type_tag, nested)?)?),
            _ => return Err(PrincipalDecodeError::UnknownTag(tag)),
        };
        Ok(principal)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PrincipalDecodeError {
    InputTooLarge { actual: usize, limit: usize },
    Canonical(CanonicalDecodeError),
    Identifier(crate::IdentifierError),
    WrongType(u8),
    InvalidNestedType { expected: u8, actual: u8 },
    InvalidNestedLength { expected: usize, actual: usize },
    UnknownTag(u8),
}

impl Display for PrincipalDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputTooLarge { actual, limit } => {
                write!(formatter, "principal has {actual} bytes; limit is {limit}")
            }
            Self::Canonical(error) => write!(formatter, "principal encoding is invalid: {error}"),
            Self::Identifier(error) => {
                write!(formatter, "principal identifier is invalid: {error}")
            }
            Self::WrongType(type_tag) => {
                write!(formatter, "principal has canonical type {type_tag:#04x}")
            }
            Self::InvalidNestedType { expected, actual } => write!(
                formatter,
                "principal nested value has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::InvalidNestedLength { expected, actual } => write!(
                formatter,
                "principal nested value has {actual} bytes; expected {expected}"
            ),
            Self::UnknownTag(tag) => write!(formatter, "unknown issuer principal tag {tag}"),
        }
    }
}

impl Error for PrincipalDecodeError {}

impl From<CanonicalDecodeError> for PrincipalDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<crate::IdentifierError> for PrincipalDecodeError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CommandPhase {
    Ingress = 0,
    Outcome = 1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CommandPayload {
    Noop,
    Rpg(RpgCommand),
    Physical(PhysicalCommandV1),
}

impl CommandPayload {
    fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        match self {
            Self::Noop => Ok(Vec::new()),
            Self::Rpg(command) => command.canonical_payload_bytes(),
            Self::Physical(command) => command.canonical_payload_bytes(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapabilityRefV1 {
    pub capability_id: CapabilityId,
    pub scope_hash: Option<ContentHash>,
}

impl CapabilityRefV1 {
    pub fn unscoped(capability_id: impl Into<String>) -> Result<Self, crate::IdentifierError> {
        Ok(Self {
            capability_id: CapabilityId::new(capability_id)?,
            scope_hash: None,
        })
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

    fn from_struct_payload(
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

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
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

    fn canonical_key_without_constraint(&self) -> Result<Vec<u8>, CanonicalError> {
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

    fn from_struct_payload(
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
        if payload_schema_version != COMMAND_SCHEMA_VERSION {
            return Err(CommandDecodeError::UnsupportedPayloadSchemaVersion(
                payload_schema_version,
            ));
        }
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
        let payload = match payload_schema_id.as_str() {
            NOOP_COMMAND_SCHEMA_ID if payload_bytes.is_empty() => CommandPayload::Noop,
            NOOP_COMMAND_SCHEMA_ID => return Err(CommandDecodeError::InvalidNoopPayload),
            RPG_COMMAND_SCHEMA_ID => CommandPayload::Rpg(RpgCommand::from_canonical_payload_bytes(
                payload_bytes,
                limits,
            )?),
            PHYSICAL_COMMAND_SCHEMA_ID => CommandPayload::Physical(
                PhysicalCommandV1::from_canonical_payload_bytes(payload_bytes, limits)?,
            ),
            schema => return Err(CommandDecodeError::UnknownPayloadSchema(schema.to_owned())),
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldCommandEnvelopeV2 {
    pub envelope_schema_version: u16,
    pub claimed_command_id: Option<CommandId>,
    pub body: CanonicalCommandBodyV2,
}

pub type WorldCommand = WorldCommandEnvelopeV2;
pub type IssuerPrincipalV2 = IssuerPrincipal;

impl Deref for WorldCommandEnvelopeV2 {
    type Target = CanonicalCommandBodyV2;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl DerefMut for WorldCommandEnvelopeV2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

impl WorldCommandEnvelopeV2 {
    pub fn noop(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        target_tick: u64,
    ) -> Result<Self, CanonicalError> {
        let mut command = Self {
            envelope_schema_version: COMMAND_ENVELOPE_SCHEMA_VERSION,
            claimed_command_id: None,
            body: CanonicalCommandBodyV2 {
                payload_schema_id: SchemaId::new(NOOP_COMMAND_SCHEMA_ID)?,
                payload_schema_version: COMMAND_SCHEMA_VERSION,
                issuer,
                stream_id,
                sequence,
                target_tick,
                phase: CommandPhase::Ingress,
                target: None,
                capability_claims: vec![CapabilityRefV1::unscoped(NOOP_COMMAND_CAPABILITY_ID)?],
                preconditions: Vec::new(),
                payload: CommandPayload::Noop,
            },
        };
        command.refresh_command_id()?;
        Ok(command)
    }

    pub fn rpg(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        target_tick: u64,
        payload: RpgCommand,
    ) -> Result<Self, CanonicalError> {
        let mut command = Self {
            envelope_schema_version: COMMAND_ENVELOPE_SCHEMA_VERSION,
            claimed_command_id: None,
            body: CanonicalCommandBodyV2 {
                payload_schema_id: SchemaId::new(RPG_COMMAND_SCHEMA_ID)?,
                payload_schema_version: COMMAND_SCHEMA_VERSION,
                issuer,
                stream_id,
                sequence,
                target_tick,
                phase: CommandPhase::Ingress,
                target: None,
                capability_claims: vec![CapabilityRefV1::unscoped(RPG_COMMAND_CAPABILITY_ID)?],
                preconditions: Vec::new(),
                payload: CommandPayload::Rpg(payload),
            },
        };
        command.refresh_command_id()?;
        Ok(command)
    }

    pub fn physical(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        target_tick: u64,
        target: PersistentId,
        payload: PhysicalCommandV1,
    ) -> Result<Self, CanonicalError> {
        let mut command = Self {
            envelope_schema_version: COMMAND_ENVELOPE_SCHEMA_VERSION,
            claimed_command_id: None,
            body: CanonicalCommandBodyV2 {
                payload_schema_id: SchemaId::new(PHYSICAL_COMMAND_SCHEMA_ID)?,
                payload_schema_version: PHYSICAL_COMMAND_SCHEMA_VERSION,
                issuer,
                stream_id,
                sequence,
                target_tick,
                phase: CommandPhase::Ingress,
                target: Some(target),
                capability_claims: vec![CapabilityRefV1::unscoped(PHYSICAL_COMMAND_CAPABILITY_ID)?],
                preconditions: Vec::new(),
                payload: CommandPayload::Physical(payload),
            },
        };
        command.refresh_command_id()?;
        Ok(command)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.body.canonical_bytes()
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandDecodeError> {
        let body = CanonicalCommandBodyV2::from_canonical_bytes(bytes, limits)?;
        let mut command = Self {
            envelope_schema_version: COMMAND_ENVELOPE_SCHEMA_VERSION,
            claimed_command_id: None,
            body,
        };
        command.refresh_command_id()?;
        Ok(command)
    }

    pub fn body_hash(&self) -> Result<CommandBodyHash, CanonicalError> {
        Ok(command_body_hash_from_bytes(sha256(
            &self.canonical_bytes()?,
        )))
    }

    pub fn compute_command_id(&self) -> Result<CommandId, CanonicalError> {
        compute_command_id_from_body_bytes(&self.canonical_bytes()?)
    }

    pub fn refresh_command_id(&mut self) -> Result<(), CanonicalError> {
        self.claimed_command_id = Some(self.compute_command_id()?);
        Ok(())
    }

    #[must_use]
    pub fn precondition_revision(&self) -> Option<u64> {
        match self.preconditions.as_slice() {
            [precondition] if precondition.is_authoritative_revision() => {
                Some(precondition.expected_revision)
            }
            _ => None,
        }
    }

    pub fn set_precondition_revision(
        &mut self,
        revision: Option<u64>,
    ) -> Result<(), CanonicalError> {
        self.preconditions = revision
            .map(CommandPreconditionV1::authoritative_revision)
            .transpose()?
            .into_iter()
            .collect();
        self.refresh_command_id()
    }
}

pub fn compute_command_id_from_body_bytes(body_bytes: &[u8]) -> Result<CommandId, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-id.v2\0");
    preimage.extend_from_slice(
        &u64::try_from(body_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(body_bytes);
    let digest = sha256(&preimage);
    let mut truncated = [0; 16];
    truncated.copy_from_slice(&digest[..16]);
    Ok(CommandId::from_bytes(truncated))
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandDecodeError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Principal(PrincipalDecodeError),
    Rpg(RpgDecodeError),
    Physics(PhysicsContractError),
    Identifier(crate::IdentifierError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    FieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedBodySchemaVersion(u16),
    UnsupportedPayloadSchemaVersion(u32),
    UnknownPayloadSchema(String),
    InvalidPhase(u8),
    InvalidNoopPayload,
    InvalidOptional,
    InvalidNestedType {
        expected: u8,
        actual: u8,
    },
    TooManySetItems {
        actual: usize,
        limit: usize,
    },
    SetNotStrictlySorted,
    ConflictingPreconditions,
    NonCanonicalEncoding,
}

impl Display for CommandDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "command encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "command canonicalization failed: {error}")
            }
            Self::Principal(error) => write!(formatter, "command principal is invalid: {error}"),
            Self::Rpg(error) => write!(formatter, "command RPG payload is invalid: {error}"),
            Self::Physics(error) => {
                write!(formatter, "command physical payload is invalid: {error}")
            }
            Self::Identifier(error) => write!(formatter, "command identifier is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("command body envelope does not match V2"),
            Self::UnknownField(field_id) => write!(formatter, "unknown command field {field_id}"),
            Self::MissingField(field_id) => write!(formatter, "missing command field {field_id}"),
            Self::FieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "command field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::FieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "command field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedBodySchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported command body schema version {version}"
                )
            }
            Self::UnsupportedPayloadSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported command payload schema version {version}"
                )
            }
            Self::UnknownPayloadSchema(schema) => {
                write!(formatter, "unknown command payload schema {schema}")
            }
            Self::InvalidPhase(phase) => write!(formatter, "invalid command phase {phase}"),
            Self::InvalidNoopPayload => formatter.write_str("noop command payload must be empty"),
            Self::InvalidOptional => formatter.write_str("invalid canonical optional value"),
            Self::InvalidNestedType { expected, actual } => write!(
                formatter,
                "nested value has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::TooManySetItems { actual, limit } => {
                write!(
                    formatter,
                    "canonical set has {actual} items; limit is {limit}"
                )
            }
            Self::SetNotStrictlySorted => {
                formatter.write_str("canonical set is not strictly sorted")
            }
            Self::ConflictingPreconditions => {
                formatter.write_str("preconditions have the same key with different constraints")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("decoded command does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for CommandDecodeError {}

impl From<CanonicalDecodeError> for CommandDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for CommandDecodeError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<PrincipalDecodeError> for CommandDecodeError {
    fn from(error: PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<RpgDecodeError> for CommandDecodeError {
    fn from(error: RpgDecodeError) -> Self {
        Self::Rpg(error)
    }
}

impl From<PhysicsContractError> for CommandDecodeError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<crate::IdentifierError> for CommandDecodeError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn validate_command_body_envelope(
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

fn field(fields: &[CanonicalField], field_id: u32) -> Result<&CanonicalField, CommandDecodeError> {
    fields
        .binary_search_by_key(&field_id, |field| field.field_id)
        .ok()
        .map(|index| &fields[index])
        .ok_or(CommandDecodeError::MissingField(field_id))
}

fn validate_exact_fields(
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

fn decode_exact<const LENGTH: usize>(bytes: &[u8]) -> Result<[u8; LENGTH], CommandDecodeError> {
    bytes
        .try_into()
        .map_err(|_| CommandDecodeError::FieldLength {
            field_id: 0,
            expected: LENGTH,
            actual: bytes.len(),
        })
}

fn decode_u8(bytes: &[u8]) -> Result<u8, CommandDecodeError> {
    Ok(decode_exact::<1>(bytes)?[0])
}

fn decode_u16(bytes: &[u8]) -> Result<u16, CommandDecodeError> {
    Ok(u16::from_le_bytes(decode_exact(bytes)?))
}

fn decode_u32(bytes: &[u8]) -> Result<u32, CommandDecodeError> {
    Ok(u32::from_le_bytes(decode_exact(bytes)?))
}

fn decode_u64(bytes: &[u8]) -> Result<u64, CommandDecodeError> {
    Ok(u64::from_le_bytes(decode_exact(bytes)?))
}

fn decode_utf8(bytes: &[u8]) -> Result<&str, CommandDecodeError> {
    std::str::from_utf8(bytes)
        .map_err(|_| CommandDecodeError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

fn encode_nested_value(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
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

fn read_nested_value<'a>(
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

fn encode_struct_payload(
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

fn decode_struct_payload(
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

fn encode_optional_id(value: Option<&PersistentId>) -> Result<Vec<u8>, CanonicalError> {
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

fn decode_optional_id(
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

fn encode_optional_hash(value: Option<&ContentHash>) -> Result<Vec<u8>, CanonicalError> {
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

fn decode_optional_hash(
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

fn encode_canonical_set(mut records: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
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

fn encode_precondition_set(
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

fn decode_capability_set(
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

fn decode_precondition_set(
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

fn decode_id128(type_tag: u8, payload: &[u8]) -> Result<[u8; 16], PrincipalDecodeError> {
    if type_tag != CANONICAL_TYPE_ID128 {
        return Err(PrincipalDecodeError::InvalidNestedType {
            expected: CANONICAL_TYPE_ID128,
            actual: type_tag,
        });
    }
    payload
        .try_into()
        .map_err(|_| PrincipalDecodeError::InvalidNestedLength {
            expected: 16,
            actual: payload.len(),
        })
}

fn decode_text(type_tag: u8, payload: &[u8]) -> Result<&str, PrincipalDecodeError> {
    if type_tag != CANONICAL_TYPE_UTF8_NFC {
        return Err(PrincipalDecodeError::InvalidNestedType {
            expected: CANONICAL_TYPE_UTF8_NFC,
            actual: type_tag,
        });
    }
    std::str::from_utf8(payload)
        .map_err(|_| PrincipalDecodeError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainEventEnvelopeV2 {
    pub schema_version: u16,
    pub event_id: EventId,
    pub tick: u64,
    pub phase: CommandPhase,
    pub causal_command_id: CommandId,
    pub event_slot: u32,
    pub schema_id: SchemaId,
    pub event_schema_version: u32,
    pub event_body_hash: ContentHash,
    pub payload: EventPayload,
}

pub type DomainEvent = DomainEventEnvelopeV2;

impl DomainEventEnvelopeV2 {
    pub fn command_committed(
        tick: u64,
        phase: CommandPhase,
        command_id: CommandId,
        command_sequence: u64,
    ) -> Result<Self, CanonicalError> {
        Self::build(
            tick,
            phase,
            command_id,
            0,
            SchemaId::new(EVENT_SCHEMA_ID)?,
            EventPayload::CommandCommitted { command_sequence },
        )
    }

    pub fn rpg(
        tick: u64,
        phase: CommandPhase,
        command_id: CommandId,
        event_slot: u32,
        payload: RpgEvent,
    ) -> Result<Self, CanonicalError> {
        let schema_id = SchemaId::new(payload.schema_id())?;
        Self::build(
            tick,
            phase,
            command_id,
            event_slot,
            schema_id,
            EventPayload::Rpg(payload),
        )
    }

    pub fn physical(
        tick: u64,
        phase: CommandPhase,
        command_id: CommandId,
        event_slot: u32,
        payload: PhysicalEventV1,
    ) -> Result<Self, CanonicalError> {
        let schema_id = SchemaId::new(payload.schema_id())?;
        Self::build(
            tick,
            phase,
            command_id,
            event_slot,
            schema_id,
            EventPayload::Physical(payload),
        )
    }

    fn build(
        tick: u64,
        phase: CommandPhase,
        command_id: CommandId,
        event_slot: u32,
        schema_id: SchemaId,
        payload: EventPayload,
    ) -> Result<Self, CanonicalError> {
        let event_body_bytes = payload.canonical_body_bytes(&schema_id)?;
        let event_body_hash = event_body_hash(&event_body_bytes)?;
        let event_id = derive_event_id(command_id, event_slot, &schema_id, event_body_hash)?;
        let event = Self {
            schema_version: 2,
            event_id,
            tick,
            phase,
            causal_command_id: command_id,
            event_slot,
            schema_id,
            event_schema_version: 1,
            event_body_hash,
            payload,
        };
        event.validate()?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<(), CanonicalError> {
        if self.schema_version != 2 || self.event_schema_version != 1 {
            return Err(CanonicalError::LengthOverflow);
        }
        let body_bytes = self.payload.canonical_body_bytes(&self.schema_id)?;
        let body_hash = event_body_hash(&body_bytes)?;
        if body_hash != self.event_body_hash
            || derive_event_id(
                self.causal_command_id,
                self.event_slot,
                &self.schema_id,
                body_hash,
            )? != self.event_id
        {
            return Err(CanonicalError::DuplicateSequenceValue);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        self.validate()?;
        encode_canonical_segment(
            EVENT_OWNER_ID,
            EVENT_ENVELOPE_SCHEMA_ID,
            EVENT_ENVELOPE_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U16,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_ID128, self.event_id.as_bytes().to_vec()),
                CanonicalField::new(3, CANONICAL_TYPE_U64, self.tick.to_le_bytes().to_vec()),
                CanonicalField::new(4, CANONICAL_TYPE_U8, vec![self.phase as u8]),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_ID128,
                    self.causal_command_id.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_U32,
                    self.event_slot.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.schema_id.as_str().as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_U32,
                    self.event_schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_HASH256,
                    self.event_body_hash.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    10,
                    CANONICAL_TYPE_BYTES,
                    self.payload.canonical_body_bytes(&self.schema_id)?,
                ),
            ],
        )
    }
}

fn event_body_hash(event_body_bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.event-body.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(event_body_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(event_body_bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn derive_event_id(
    command_id: CommandId,
    event_slot: u32,
    schema_id: &SchemaId,
    body_hash: ContentHash,
) -> Result<EventId, CanonicalError> {
    let schema_bytes = schema_id.as_str().as_bytes();
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.event-id.v1\0");
    preimage.extend_from_slice(command_id.as_bytes());
    preimage.extend_from_slice(&event_slot.to_le_bytes());
    preimage.extend_from_slice(
        &u64::try_from(schema_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(schema_bytes);
    preimage.extend_from_slice(body_hash.as_bytes());
    let digest = sha256(&preimage);
    let mut event_id = [0; 16];
    event_id.copy_from_slice(&digest[..16]);
    Ok(EventId::from_bytes(event_id))
}

impl EventPayload {
    fn canonical_body_bytes(&self, schema_id: &SchemaId) -> Result<Vec<u8>, CanonicalError> {
        let payload = match self {
            Self::CommandCommitted { command_sequence } => command_sequence.to_le_bytes().to_vec(),
            Self::Rpg(payload) => payload.canonical_payload_bytes()?,
            Self::Physical(payload) => payload.canonical_payload_bytes()?,
        };
        encode_canonical_segment(
            EVENT_OWNER_ID,
            schema_id.as_str(),
            EVENT_SEGMENT_ID,
            [CanonicalField::new(1, CANONICAL_TYPE_BYTES, payload)],
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventPayload {
    CommandCommitted { command_sequence: u64 },
    Rpg(RpgEvent),
    Physical(PhysicalEventV1),
}

#[cfg(test)]
mod tests {
    use crate::{
        CANONICAL_TYPE_BYTES, CanonicalDecodeLimits, CanonicalField, CapabilityRefV1, CommandId,
        CommandStreamId, PlayerPrincipalId, decode_canonical_segment, encode_canonical_segment,
    };

    use super::{
        COMMAND_BODY_OWNER_ID, COMMAND_BODY_SCHEMA_ID, COMMAND_BODY_SEGMENT_ID, CommandDecodeError,
        CommandPhase, DomainEventEnvelopeV2, IssuerPrincipal, WorldCommand,
    };

    fn command() -> WorldCommand {
        WorldCommand::noop(
            CommandStreamId::from_bytes([1; 16]),
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
            7,
            11,
        )
        .expect("fixed command is canonical")
    }

    #[test]
    fn command_id_v2_matches_golden_vector_and_is_body_sensitive() {
        let first = command();
        let mut second = first.clone();
        second.stream_id = CommandStreamId::from_bytes([3; 16]);
        second
            .refresh_command_id()
            .expect("command remains canonical");

        assert_eq!(
            first
                .claimed_command_id
                .expect("constructor computes the claim")
                .to_hex(),
            "c2ca2d2370d816fe888565a66bda6eeb"
        );
        assert_ne!(first.claimed_command_id, second.claimed_command_id);
    }

    #[test]
    fn envelope_claim_is_not_part_of_canonical_body_bytes() {
        let first = command();
        let mut second = first.clone();
        second.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
        assert_eq!(
            first.canonical_bytes().expect("canonical command"),
            second.canonical_bytes().expect("canonical command")
        );
        assert_ne!(
            second.claimed_command_id,
            Some(second.compute_command_id().expect("computed command ID"))
        );
    }

    #[test]
    fn domain_event_id_is_body_sensitive_and_canonical() {
        let command_id = command().compute_command_id().expect("command ID");
        let first =
            DomainEventEnvelopeV2::command_committed(11, CommandPhase::Ingress, command_id, 7)
                .expect("event");
        let second =
            DomainEventEnvelopeV2::command_committed(11, CommandPhase::Ingress, command_id, 8)
                .expect("event");

        assert_ne!(first.event_id, second.event_id);
        assert_ne!(first.event_body_hash, second.event_body_hash);
        assert_ne!(
            first.canonical_bytes().expect("first event"),
            second.canonical_bytes().expect("second event")
        );
        first.validate().expect("first event validates");
        let mut corrupt = first;
        corrupt.event_body_hash = crate::ContentHash::from_bytes([9; 32]);
        assert!(corrupt.validate().is_err());
    }

    #[test]
    fn capabilities_are_canonicalized_as_a_set() {
        let mut first = command();
        first.capability_claims = vec![
            CapabilityRefV1::unscoped("world.read").expect("valid capability"),
            CapabilityRefV1::unscoped("world.write").expect("valid capability"),
        ];
        let mut second = first.clone();
        second.capability_claims.reverse();

        assert_eq!(
            first.canonical_bytes().expect("canonical command"),
            second.canonical_bytes().expect("canonical command")
        );
    }

    #[test]
    fn command_round_trip_is_byte_exact() {
        let command = command();
        let bytes = command.canonical_bytes().expect("canonical command");
        let decoded = WorldCommand::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("command decodes");

        assert_eq!(decoded, command);
        assert_eq!(
            decoded.canonical_bytes().expect("command re-encodes"),
            bytes
        );
        let envelope =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("decodes");
        assert_eq!(envelope.owner_id, COMMAND_BODY_OWNER_ID);
        assert_eq!(envelope.schema_id, COMMAND_BODY_SCHEMA_ID);
        assert_eq!(envelope.segment_id, COMMAND_BODY_SEGMENT_ID);
    }

    #[test]
    fn rpg_command_round_trip_is_byte_exact() {
        let command = WorldCommand::rpg(
            CommandStreamId::from_bytes([4; 16]),
            IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([5; 16])),
            8,
            13,
            crate::RpgCommand::LearnSkill {
                character_id: crate::PersistentId::from_bytes([6; 16]),
                skill_id: crate::SchemaId::new("rpg.skill.survival").expect("skill id is valid"),
                delta: 25,
            },
        )
        .expect("RPG command is canonical");
        let bytes = command.canonical_bytes().expect("canonical RPG command");

        assert_eq!(
            WorldCommand::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("RPG command decodes"),
            command
        );
    }

    #[test]
    fn decoder_rejects_unknown_required_field_and_wrong_body_version() {
        let command = command();
        let bytes = command.canonical_bytes().expect("canonical command");
        let decoded =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("decodes");
        let mut fields = decoded.fields.clone();
        fields.push(CanonicalField::new(13, CANONICAL_TYPE_BYTES, vec![]));
        let unknown_field = encode_canonical_segment(
            &decoded.owner_id,
            &decoded.schema_id,
            &decoded.segment_id,
            fields,
        )
        .expect("generic envelope can encode future field");
        assert!(matches!(
            WorldCommand::from_canonical_bytes(&unknown_field, CanonicalDecodeLimits::default()),
            Err(CommandDecodeError::UnknownField(13))
        ));

        let mut fields = decoded.fields;
        fields
            .iter_mut()
            .find(|field| field.field_id == 1)
            .expect("body version field")
            .payload = 3_u16.to_le_bytes().to_vec();
        let future_bytes = encode_canonical_segment(
            COMMAND_BODY_OWNER_ID,
            COMMAND_BODY_SCHEMA_ID,
            COMMAND_BODY_SEGMENT_ID,
            fields,
        )
        .expect("future body encodes");
        assert!(matches!(
            WorldCommand::from_canonical_bytes(&future_bytes, CanonicalDecodeLimits::default()),
            Err(CommandDecodeError::UnsupportedBodySchemaVersion(3))
        ));
    }

    #[test]
    fn decoder_applies_total_input_bound_before_parsing() {
        let bytes = command().canonical_bytes().expect("canonical command");
        let limits = CanonicalDecodeLimits {
            max_total_bytes: bytes.len() - 1,
            ..CanonicalDecodeLimits::default()
        };
        assert!(matches!(
            WorldCommand::from_canonical_bytes(&bytes, limits),
            Err(CommandDecodeError::Canonical(
                crate::CanonicalDecodeError::InputTooLarge { .. }
            ))
        ));
    }
}
