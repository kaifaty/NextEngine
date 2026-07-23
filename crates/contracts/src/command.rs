use crate::canonical::{
    CanonicalError, CanonicalField, encode_canonical_segment, extend_u32_length_prefixed, sha256,
};
use crate::ids::{
    CapabilityId, CommandId, CommandStreamId, EventId, MechanicPackageId, PersistentId,
    PlayerPrincipalId, PluginId, SchemaId, SystemId,
};

pub const COMMAND_SCHEMA_VERSION: u32 = 1;
pub const NOOP_COMMAND_SCHEMA_ID: &str = "nextengine.command.noop";
pub const NOOP_COMMAND_CAPABILITY_ID: &str = "runtime.command.noop";
const COMMAND_OWNER_ID: &str = "runtime";
const COMMAND_SEGMENT_ID: &str = "world-command";
const EVENT_SCHEMA_ID: &str = "nextengine.event.command-committed";

const TYPE_BYTES: u8 = 1;
const TYPE_U64: u8 = 2;
const TYPE_U32: u8 = 3;
const TYPE_U8: u8 = 4;
const TYPE_SEQUENCE: u8 = 5;
const TYPE_OPTIONAL: u8 = 6;

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

        let payload = match self.payload {
            CommandPayload::Noop => vec![0],
        };

        encode_canonical_segment(
            COMMAND_OWNER_ID,
            self.schema_id.as_str(),
            COMMAND_SEGMENT_ID,
            [
                CanonicalField::new(1, TYPE_U32, self.schema_version.to_le_bytes().to_vec()),
                CanonicalField::new(2, TYPE_BYTES, self.stream_id.as_bytes().to_vec()),
                CanonicalField::new(3, TYPE_BYTES, issuer),
                CanonicalField::new(4, TYPE_U64, self.sequence.to_le_bytes().to_vec()),
                CanonicalField::new(5, TYPE_U64, self.target_tick.to_le_bytes().to_vec()),
                CanonicalField::new(6, TYPE_U8, vec![self.phase as u8]),
                CanonicalField::new(7, TYPE_OPTIONAL, target),
                CanonicalField::new(8, TYPE_SEQUENCE, capabilities),
                CanonicalField::new(9, TYPE_OPTIONAL, precondition),
                CanonicalField::new(10, TYPE_BYTES, payload),
            ],
        )
    }

    pub fn compute_command_id(&self) -> Result<CommandId, CanonicalError> {
        let command_bytes = self.canonical_bytes()?;
        let principal_bytes = self.issuer.canonical_bytes()?;
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.command-id.v1\0");
        preimage.extend_from_slice(self.stream_id.as_bytes());
        extend_u32_length_prefixed(&mut preimage, &principal_bytes)?;
        preimage.extend_from_slice(&self.sequence.to_le_bytes());
        extend_u32_length_prefixed(&mut preimage, &command_bytes)?;
        let digest = sha256(&preimage);
        let mut truncated = [0; 16];
        truncated.copy_from_slice(&digest[..16]);
        Ok(CommandId::from_bytes(truncated))
    }

    pub fn refresh_command_id(&mut self) -> Result<(), CanonicalError> {
        self.command_id = self.compute_command_id()?;
        Ok(())
    }
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventPayload {
    CommandCommitted { command_sequence: u64 },
}

#[cfg(test)]
mod tests {
    use crate::{CommandStreamId, PlayerPrincipalId};

    use super::{IssuerPrincipal, WorldCommand};

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
}
