use std::ops::{Deref, DerefMut};

use crate::canonical::{CanonicalDecodeLimits, CanonicalError, sha256};
use crate::ids::{
    CommandBodyHash, CommandId, CommandStreamId, PersistentId, SchemaId,
    command_body_hash_from_bytes,
};
use crate::physics::{
    PHYSICAL_COMMAND_CAPABILITY_ID, PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_COMMAND_SCHEMA_VERSION,
    PhysicalCommandV1,
};
use crate::rpg::{RPG_COMMAND_CAPABILITY_ID, RPG_COMMAND_SCHEMA_ID};
use crate::rpg::{RPG_TRANSACTION_COMMAND_SCHEMA_VERSION, RpgCommandV1};

use super::body::{
    COMMAND_SCHEMA_VERSION, CanonicalCommandBodyV2, CapabilityRefV1, CommandPayload, CommandPhase,
    CommandPreconditionV1, NOOP_COMMAND_CAPABILITY_ID, NOOP_COMMAND_SCHEMA_ID,
};
use super::error::CommandDecodeError;
use super::principal::IssuerPrincipal;

pub const COMMAND_ENVELOPE_SCHEMA_VERSION: u16 = 2;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldCommandEnvelopeV2 {
    pub envelope_schema_version: u16,
    pub claimed_command_id: Option<CommandId>,
    pub body: CanonicalCommandBodyV2,
}

pub type WorldCommand = WorldCommandEnvelopeV2;

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
        payload: RpgCommandV1,
    ) -> Result<Self, CanonicalError> {
        let mut command = Self {
            envelope_schema_version: COMMAND_ENVELOPE_SCHEMA_VERSION,
            claimed_command_id: None,
            body: CanonicalCommandBodyV2 {
                payload_schema_id: SchemaId::new(RPG_COMMAND_SCHEMA_ID)?,
                payload_schema_version: RPG_TRANSACTION_COMMAND_SCHEMA_VERSION,
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
    use sha2::{Digest as _, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.command-id.v2\0");
    hasher.update(
        u64::try_from(body_bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    hasher.update(body_bytes);
    let digest: [u8; 32] = hasher.finalize().into();
    let mut truncated = [0; 16];
    truncated.copy_from_slice(&digest[..16]);
    Ok(CommandId::from_bytes(truncated))
}
