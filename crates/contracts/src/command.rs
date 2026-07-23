use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor, CanonicalDecodeError,
    CanonicalDecodeLimits, CanonicalError, CanonicalField, DecodedCanonicalSegment,
    decode_canonical_segment, encode_canonical_segment, extend_u32_length_prefixed, sha256,
};
use crate::ids::{
    CapabilityId, CommandId, CommandStreamId, EventId, MechanicPackageId, PersistentId,
    PlayerPrincipalId, PluginId, SchemaId, SystemId,
};
use crate::rpg::{
    RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID, RpgCommand, RpgDecodeError, RpgEvent,
};

pub const COMMAND_SCHEMA_VERSION: u32 = 1;
pub const NOOP_COMMAND_SCHEMA_ID: &str = "nextengine.command.noop";
pub const NOOP_COMMAND_CAPABILITY_ID: &str = "runtime.command.noop";
const COMMAND_OWNER_ID: &str = "runtime";
const COMMAND_SEGMENT_ID: &str = "world-command";
const EVENT_SCHEMA_ID: &str = "nextengine.event.command-committed";

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IssuerPrincipal {
    Player(PlayerPrincipalId),
    Agent(PersistentId),
    Package(MechanicPackageId),
    Plugin(PluginId),
    InternalSystem(SystemId),
}

impl IssuerPrincipal {
    #[must_use]
    pub const fn tag(&self) -> u8 {
        match self {
            Self::Player(_) => 0x01,
            Self::Agent(_) => 0x02,
            Self::Package(_) => 0x03,
            Self::Plugin(_) => 0x04,
            Self::InternalSystem(_) => 0x05,
        }
    }

    #[must_use]
    pub fn identifier_bytes(&self) -> &[u8] {
        match self {
            Self::Player(id) => id.as_bytes(),
            Self::Agent(id) => id.as_bytes(),
            Self::Package(id) => id.as_str().as_bytes(),
            Self::Plugin(id) => id.as_str().as_bytes(),
            Self::InternalSystem(id) => id.as_str().as_bytes(),
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut bytes = vec![self.tag()];
        match self {
            Self::Player(id) => bytes.extend_from_slice(id.as_bytes()),
            Self::Agent(id) => bytes.extend_from_slice(id.as_bytes()),
            Self::Package(id) => {
                extend_u32_length_prefixed(&mut bytes, id.as_str().as_bytes())?;
            }
            Self::Plugin(id) => {
                extend_u32_length_prefixed(&mut bytes, id.as_str().as_bytes())?;
            }
            Self::InternalSystem(id) => {
                extend_u32_length_prefixed(&mut bytes, id.as_str().as_bytes())?;
            }
        }
        Ok(bytes)
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
        let tag = cursor.read_u8()?;
        let principal = match tag {
            0x01 => {
                let value: [u8; 16] = cursor
                    .read_exact(16)?
                    .try_into()
                    .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
                Self::Player(PlayerPrincipalId::from_bytes(value))
            }
            0x02 => {
                let value: [u8; 16] = cursor
                    .read_exact(16)?
                    .try_into()
                    .map_err(|_| CanonicalDecodeError::UnexpectedEnd)?;
                Self::Agent(PersistentId::from_bytes(value))
            }
            0x03 => Self::Package(MechanicPackageId::new(read_text_principal(
                &mut cursor,
                limits.max_identifier_bytes,
            )?)?),
            0x04 => Self::Plugin(PluginId::new(read_text_principal(
                &mut cursor,
                limits.max_identifier_bytes,
            )?)?),
            0x05 => Self::InternalSystem(SystemId::new(read_text_principal(
                &mut cursor,
                limits.max_identifier_bytes,
            )?)?),
            _ => return Err(PrincipalDecodeError::UnknownTag(tag)),
        };
        cursor.finish()?;
        Ok(principal)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PrincipalDecodeError {
    InputTooLarge { actual: usize, limit: usize },
    Canonical(CanonicalDecodeError),
    Identifier(crate::IdentifierError),
    UnknownTag(u8),
}

impl std::fmt::Display for PrincipalDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputTooLarge { actual, limit } => {
                write!(formatter, "principal has {actual} bytes; limit is {limit}")
            }
            Self::Canonical(error) => write!(formatter, "principal encoding is invalid: {error}"),
            Self::Identifier(error) => {
                write!(formatter, "principal identifier is invalid: {error}")
            }
            Self::UnknownTag(tag) => write!(formatter, "unknown issuer principal tag {tag}"),
        }
    }
}

impl std::error::Error for PrincipalDecodeError {}

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
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldCommand {
    pub schema_id: SchemaId,
    pub schema_version: u32,
    pub command_id: CommandId,
    pub issuer: IssuerPrincipal,
    pub stream_id: CommandStreamId,
    pub sequence: u64,
    pub target_tick: u64,
    pub phase: CommandPhase,
    pub target: Option<PersistentId>,
    pub declared_capabilities: Vec<CapabilityId>,
    pub precondition_revision: Option<u64>,
    pub payload: CommandPayload,
}

impl WorldCommand {
    pub fn noop(
        stream_id: CommandStreamId,
        issuer: IssuerPrincipal,
        sequence: u64,
        target_tick: u64,
    ) -> Result<Self, CanonicalError> {
        let mut command = Self {
            schema_id: SchemaId::new(NOOP_COMMAND_SCHEMA_ID)?,
            schema_version: COMMAND_SCHEMA_VERSION,
            command_id: CommandId::default(),
            issuer,
            stream_id,
            sequence,
            target_tick,
            phase: CommandPhase::Ingress,
            target: None,
            declared_capabilities: vec![
                CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID)
                    .map_err(CanonicalError::InvalidIdentifier)?,
            ],
            precondition_revision: None,
            payload: CommandPayload::Noop,
        };
        command.command_id = command.compute_command_id()?;
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
            schema_id: SchemaId::new(RPG_COMMAND_SCHEMA_ID)?,
            schema_version: COMMAND_SCHEMA_VERSION,
            command_id: CommandId::default(),
            issuer,
            stream_id,
            sequence,
            target_tick,
            phase: CommandPhase::Ingress,
            target: None,
            declared_capabilities: vec![CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)?],
            precondition_revision: None,
            payload: CommandPayload::Rpg(payload),
        };
        command.command_id = command.compute_command_id()?;
        Ok(command)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let issuer = self.issuer.canonical_bytes()?;
        let mut target = vec![u8::from(self.target.is_some())];
        if let Some(target_id) = self.target {
            target.extend_from_slice(target_id.as_bytes());
        }

        let mut capability_values: Vec<_> = self
            .declared_capabilities
            .iter()
            .map(CapabilityId::as_str)
            .collect();
        capability_values.sort_unstable();
        if capability_values.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(CanonicalError::DuplicateSequenceValue);
        }
        let mut capabilities = Vec::new();
        capabilities.extend_from_slice(
            &u32::try_from(capability_values.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        for capability in capability_values {
            extend_u32_length_prefixed(&mut capabilities, capability.as_bytes())?;
        }

        let mut precondition = vec![u8::from(self.precondition_revision.is_some())];
        if let Some(revision) = self.precondition_revision {
            precondition.extend_from_slice(&revision.to_le_bytes());
        }

        let payload = match &self.payload {
            CommandPayload::Noop => vec![0],
            CommandPayload::Rpg(command) => command.canonical_payload_bytes()?,
        };

        encode_canonical_segment(
            COMMAND_OWNER_ID,
            self.schema_id.as_str(),
            COMMAND_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    self.schema_version.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.stream_id.as_bytes().to_vec()),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, issuer),
                CanonicalField::new(4, CANONICAL_TYPE_U64, self.sequence.to_le_bytes().to_vec()),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_U64,
                    self.target_tick.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(6, CANONICAL_TYPE_U8, vec![self.phase as u8]),
                CanonicalField::new(7, CANONICAL_TYPE_OPTIONAL, target),
                CanonicalField::new(8, CANONICAL_TYPE_SEQUENCE, capabilities),
                CanonicalField::new(9, CANONICAL_TYPE_OPTIONAL, precondition),
                CanonicalField::new(10, CANONICAL_TYPE_BYTES, payload),
            ],
        )
    }

    pub fn compute_command_id(&self) -> Result<CommandId, CanonicalError> {
        let command_bytes = self.canonical_bytes()?;
        compute_command_id_from_canonical(
            self.stream_id,
            &self.issuer,
            self.sequence,
            &command_bytes,
        )
    }

    pub fn refresh_command_id(&mut self) -> Result<(), CanonicalError> {
        self.command_id = self.compute_command_id()?;
        Ok(())
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, CommandDecodeError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_command_envelope(&segment)?;
        validate_exact_command_fields(&segment)?;

        let schema_version = decode_u32_field(&segment, 1, CANONICAL_TYPE_U32)?;
        if schema_version != COMMAND_SCHEMA_VERSION {
            return Err(CommandDecodeError::UnsupportedSchemaVersion(schema_version));
        }
        let schema_id = SchemaId::new(segment.schema_id.clone())?;
        let known_schema = matches!(
            schema_id.as_str(),
            NOOP_COMMAND_SCHEMA_ID | RPG_COMMAND_SCHEMA_ID
        );
        if !known_schema {
            return Err(CommandDecodeError::UnknownSchema(segment.schema_id));
        }
        let stream_id = CommandStreamId::from_bytes(decode_fixed_field::<16>(
            &segment,
            2,
            CANONICAL_TYPE_BYTES,
        )?);
        let issuer_field = field(&segment, 3, CANONICAL_TYPE_BYTES)?;
        let issuer = IssuerPrincipal::from_canonical_bytes(&issuer_field.payload, limits)?;
        let sequence = decode_u64_field(&segment, 4, CANONICAL_TYPE_U64)?;
        let target_tick = decode_u64_field(&segment, 5, CANONICAL_TYPE_U64)?;
        let phase = match decode_u8_field(&segment, 6, CANONICAL_TYPE_U8)? {
            0 => CommandPhase::Ingress,
            1 => CommandPhase::Outcome,
            value => return Err(CommandDecodeError::InvalidPhase(value)),
        };
        let target = decode_optional_id(&segment, 7)?;
        let declared_capabilities = decode_capabilities(&segment, limits)?;
        let precondition_revision = decode_optional_u64(&segment, 9)?;
        let payload_bytes = field(&segment, 10, CANONICAL_TYPE_BYTES)?
            .payload
            .as_slice();
        let payload = match schema_id.as_str() {
            NOOP_COMMAND_SCHEMA_ID => match payload_bytes {
                [0] => CommandPayload::Noop,
                payload => return Err(CommandDecodeError::UnknownPayload(payload.to_vec())),
            },
            RPG_COMMAND_SCHEMA_ID => CommandPayload::Rpg(RpgCommand::from_canonical_payload_bytes(
                payload_bytes,
                limits,
            )?),
            _ => return Err(CommandDecodeError::UnknownSchema(segment.schema_id)),
        };

        let mut command = Self {
            schema_id,
            schema_version,
            command_id: CommandId::default(),
            issuer,
            stream_id,
            sequence,
            target_tick,
            phase,
            target,
            declared_capabilities,
            precondition_revision,
            payload,
        };
        command.command_id = command.compute_command_id()?;
        if command.canonical_bytes()? != bytes {
            return Err(CommandDecodeError::NonCanonicalEncoding);
        }
        Ok(command)
    }
}

pub fn compute_command_id_from_canonical(
    stream_id: CommandStreamId,
    issuer: &IssuerPrincipal,
    sequence: u64,
    command_bytes: &[u8],
) -> Result<CommandId, CanonicalError> {
    let principal_bytes = issuer.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-id.v1\0");
    preimage.extend_from_slice(stream_id.as_bytes());
    extend_u32_length_prefixed(&mut preimage, &principal_bytes)?;
    preimage.extend_from_slice(&sequence.to_le_bytes());
    extend_u32_length_prefixed(&mut preimage, command_bytes)?;
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
    UnsupportedSchemaVersion(u32),
    UnknownSchema(String),
    InvalidPhase(u8),
    InvalidOptional {
        field_id: u32,
    },
    TooManyCapabilities {
        actual: usize,
        limit: usize,
    },
    CapabilitiesNotStrictlySorted,
    UnknownPayload(Vec<u8>),
    NonCanonicalEncoding,
}

impl std::fmt::Display for CommandDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "canonical command is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "command canonicalization failed: {error}")
            }
            Self::Principal(error) => write!(formatter, "command principal is invalid: {error}"),
            Self::Rpg(error) => write!(formatter, "RPG command payload is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "command identifier is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("canonical command envelope does not match"),
            Self::UnknownField(field_id) => write!(formatter, "unknown command field {field_id}"),
            Self::MissingField(field_id) => write!(formatter, "missing command field {field_id}"),
            Self::FieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "command field {field_id} has type {actual}; expected {expected}"
            ),
            Self::FieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "command field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported command schema version {version}")
            }
            Self::UnknownSchema(schema) => write!(formatter, "unknown command schema {schema}"),
            Self::InvalidPhase(phase) => write!(formatter, "invalid command phase {phase}"),
            Self::InvalidOptional { field_id } => {
                write!(formatter, "command optional field {field_id} is malformed")
            }
            Self::TooManyCapabilities { actual, limit } => write!(
                formatter,
                "command declares {actual} capabilities; limit is {limit}"
            ),
            Self::CapabilitiesNotStrictlySorted => {
                formatter.write_str("command capabilities are not strictly sorted")
            }
            Self::UnknownPayload(payload) => {
                write!(formatter, "unknown command payload bytes {payload:?}")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("decoded command does not re-encode byte-exactly")
            }
        }
    }
}

impl std::error::Error for CommandDecodeError {}

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

impl From<crate::IdentifierError> for CommandDecodeError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn read_text_principal(
    cursor: &mut CanonicalCursor<'_>,
    max_identifier_bytes: usize,
) -> Result<String, PrincipalDecodeError> {
    let bytes = cursor.read_u32_length_prefixed(max_identifier_bytes)?;
    let value = std::str::from_utf8(bytes)
        .map_err(|_| PrincipalDecodeError::Canonical(CanonicalDecodeError::InvalidUtf8))?;
    Ok(value.to_owned())
}

fn validate_command_envelope(segment: &DecodedCanonicalSegment) -> Result<(), CommandDecodeError> {
    if segment.owner_id != COMMAND_OWNER_ID || segment.segment_id != COMMAND_SEGMENT_ID {
        return Err(CommandDecodeError::WrongEnvelope);
    }
    Ok(())
}

fn validate_exact_command_fields(
    segment: &DecodedCanonicalSegment,
) -> Result<(), CommandDecodeError> {
    const EXPECTED: [(u32, u8); 10] = [
        (1, CANONICAL_TYPE_U32),
        (2, CANONICAL_TYPE_BYTES),
        (3, CANONICAL_TYPE_BYTES),
        (4, CANONICAL_TYPE_U64),
        (5, CANONICAL_TYPE_U64),
        (6, CANONICAL_TYPE_U8),
        (7, CANONICAL_TYPE_OPTIONAL),
        (8, CANONICAL_TYPE_SEQUENCE),
        (9, CANONICAL_TYPE_OPTIONAL),
        (10, CANONICAL_TYPE_BYTES),
    ];
    for field in &segment.fields {
        if !EXPECTED
            .iter()
            .any(|(field_id, _)| *field_id == field.field_id)
        {
            return Err(CommandDecodeError::UnknownField(field.field_id));
        }
    }
    for (field_id, type_tag) in EXPECTED {
        let _ = field(segment, field_id, type_tag)?;
    }
    Ok(())
}

fn field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<&CanonicalField, CommandDecodeError> {
    let field = segment
        .field(field_id)
        .ok_or(CommandDecodeError::MissingField(field_id))?;
    if field.type_tag != expected_type {
        return Err(CommandDecodeError::FieldType {
            field_id,
            expected: expected_type,
            actual: field.type_tag,
        });
    }
    Ok(field)
}

fn decode_fixed_field<const LENGTH: usize>(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<[u8; LENGTH], CommandDecodeError> {
    let payload = &field(segment, field_id, expected_type)?.payload;
    payload
        .as_slice()
        .try_into()
        .map_err(|_| CommandDecodeError::FieldLength {
            field_id,
            expected: LENGTH,
            actual: payload.len(),
        })
}

fn decode_u8_field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<u8, CommandDecodeError> {
    Ok(decode_fixed_field::<1>(segment, field_id, expected_type)?[0])
}

fn decode_u32_field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<u32, CommandDecodeError> {
    Ok(u32::from_le_bytes(decode_fixed_field::<4>(
        segment,
        field_id,
        expected_type,
    )?))
}

fn decode_u64_field(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
    expected_type: u8,
) -> Result<u64, CommandDecodeError> {
    Ok(u64::from_le_bytes(decode_fixed_field::<8>(
        segment,
        field_id,
        expected_type,
    )?))
}

fn decode_optional_id(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<Option<PersistentId>, CommandDecodeError> {
    let payload = &field(segment, field_id, CANONICAL_TYPE_OPTIONAL)?.payload;
    match payload.as_slice() {
        [0] => Ok(None),
        [1, bytes @ ..] if bytes.len() == 16 => {
            let id: [u8; 16] = bytes
                .try_into()
                .map_err(|_| CommandDecodeError::InvalidOptional { field_id })?;
            Ok(Some(PersistentId::from_bytes(id)))
        }
        _ => Err(CommandDecodeError::InvalidOptional { field_id }),
    }
}

fn decode_optional_u64(
    segment: &DecodedCanonicalSegment,
    field_id: u32,
) -> Result<Option<u64>, CommandDecodeError> {
    let payload = &field(segment, field_id, CANONICAL_TYPE_OPTIONAL)?.payload;
    match payload.as_slice() {
        [0] => Ok(None),
        [1, bytes @ ..] if bytes.len() == 8 => {
            let value: [u8; 8] = bytes
                .try_into()
                .map_err(|_| CommandDecodeError::InvalidOptional { field_id })?;
            Ok(Some(u64::from_le_bytes(value)))
        }
        _ => Err(CommandDecodeError::InvalidOptional { field_id }),
    }
}

fn decode_capabilities(
    segment: &DecodedCanonicalSegment,
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CapabilityId>, CommandDecodeError> {
    let payload = &field(segment, 8, CANONICAL_TYPE_SEQUENCE)?.payload;
    let mut cursor = CanonicalCursor::new(payload);
    let count = usize::try_from(cursor.read_u32()?)
        .map_err(|_| CommandDecodeError::Canonical(CanonicalDecodeError::LengthOverflow))?;
    if count > limits.max_sequence_items {
        return Err(CommandDecodeError::TooManyCapabilities {
            actual: count,
            limit: limits.max_sequence_items,
        });
    }
    let mut capabilities = Vec::with_capacity(count);
    for _ in 0..count {
        let bytes = cursor.read_u32_length_prefixed(limits.max_identifier_bytes)?;
        let value = std::str::from_utf8(bytes).map_err(|_| CanonicalDecodeError::InvalidUtf8)?;
        capabilities.push(CapabilityId::new(value)?);
    }
    cursor.finish()?;
    if capabilities
        .windows(2)
        .any(|pair| pair[0].as_str() >= pair[1].as_str())
    {
        return Err(CommandDecodeError::CapabilitiesNotStrictlySorted);
    }
    Ok(capabilities)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainEvent {
    pub event_id: EventId,
    pub tick: u64,
    pub causal_command_id: CommandId,
    pub schema_id: SchemaId,
    pub schema_version: u32,
    pub payload: EventPayload,
}

impl DomainEvent {
    pub fn command_committed(
        tick: u64,
        command_id: CommandId,
        command_sequence: u64,
    ) -> Result<Self, CanonicalError> {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.event-id.v1\0");
        preimage.extend_from_slice(command_id.as_bytes());
        preimage.extend_from_slice(&0_u32.to_le_bytes());
        let digest = sha256(&preimage);
        let mut event_id = [0; 16];
        event_id.copy_from_slice(&digest[..16]);
        Ok(Self {
            event_id: EventId::from_bytes(event_id),
            tick,
            causal_command_id: command_id,
            schema_id: SchemaId::new(EVENT_SCHEMA_ID)?,
            schema_version: 1,
            payload: EventPayload::CommandCommitted { command_sequence },
        })
    }

    pub fn rpg(
        tick: u64,
        command_id: CommandId,
        event_slot: u32,
        payload: RpgEvent,
    ) -> Result<Self, CanonicalError> {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.event-id.v1\0");
        preimage.extend_from_slice(command_id.as_bytes());
        preimage.extend_from_slice(&event_slot.to_le_bytes());
        let digest = sha256(&preimage);
        let mut event_id = [0; 16];
        event_id.copy_from_slice(&digest[..16]);
        let schema_id = SchemaId::new(payload.schema_id())?;
        Ok(Self {
            event_id: EventId::from_bytes(event_id),
            tick,
            causal_command_id: command_id,
            schema_id,
            schema_version: 1,
            payload: EventPayload::Rpg(payload),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventPayload {
    CommandCommitted { command_sequence: u64 },
    Rpg(RpgEvent),
}

#[cfg(test)]
mod tests {
    use crate::{
        CANONICAL_TYPE_BYTES, CanonicalDecodeLimits, CanonicalField, CommandStreamId,
        PlayerPrincipalId, decode_canonical_segment, encode_canonical_segment,
    };

    use super::{CommandDecodeError, IssuerPrincipal, WorldCommand};

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
    fn command_id_is_stable_and_sensitive_to_stream() {
        let first = command();
        let mut second = first.clone();
        second.stream_id = CommandStreamId::from_bytes([3; 16]);
        second
            .refresh_command_id()
            .expect("command remains canonical");

        assert_eq!(
            first.command_id.to_hex(),
            "5d357b6846697921100488c74a7d2e65"
        );
        assert_ne!(first.command_id, second.command_id);
    }

    #[test]
    fn claimed_command_id_is_not_part_of_canonical_command_bytes() {
        let first = command();
        let mut second = first.clone();
        second.command_id = crate::CommandId::from_bytes([9; 16]);
        assert_eq!(
            first.canonical_bytes().expect("canonical command"),
            second.canonical_bytes().expect("canonical command")
        );
    }

    #[test]
    fn capabilities_are_canonicalized_as_a_set() {
        let mut first = command();
        first.declared_capabilities = vec![
            crate::CapabilityId::new("world.read").expect("valid capability"),
            crate::CapabilityId::new("world.write").expect("valid capability"),
        ];
        let mut second = first.clone();
        second.declared_capabilities.reverse();

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
    fn decoder_rejects_unknown_required_field_and_wrong_schema_version() {
        let command = command();
        let bytes = command.canonical_bytes().expect("canonical command");
        let decoded =
            decode_canonical_segment(&bytes, CanonicalDecodeLimits::default()).expect("decodes");
        let mut fields = decoded.fields.clone();
        fields.push(CanonicalField::new(11, CANONICAL_TYPE_BYTES, vec![]));
        let unknown_field = encode_canonical_segment(
            &decoded.owner_id,
            &decoded.schema_id,
            &decoded.segment_id,
            fields,
        )
        .expect("generic envelope can encode future field");
        assert!(matches!(
            WorldCommand::from_canonical_bytes(&unknown_field, CanonicalDecodeLimits::default()),
            Err(CommandDecodeError::UnknownField(11))
        ));

        let mut future = command;
        future.schema_version = 2;
        let future_bytes = future.canonical_bytes().expect("future command encodes");
        assert!(matches!(
            WorldCommand::from_canonical_bytes(&future_bytes, CanonicalDecodeLimits::default()),
            Err(CommandDecodeError::UnsupportedSchemaVersion(2))
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
