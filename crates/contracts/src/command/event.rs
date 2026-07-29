use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC,
    CanonicalError, CanonicalField, encode_canonical_segment, sha256,
};
use crate::ids::{CommandId, ContentHash, EventId, SchemaId, content_hash_from_bytes};
use crate::physics::PhysicalEventV1;
use crate::rpg::RpgEventV1;

use super::body::CommandPhase;

const EVENT_SCHEMA_ID: &str = "nextengine.event.command-committed";
const EVENT_OWNER_ID: &str = "nextengine.runtime";
const EVENT_SEGMENT_ID: &str = "v1";
const EVENT_ENVELOPE_SCHEMA_ID: &str = "nextengine.domain-event-envelope";
const EVENT_ENVELOPE_SEGMENT_ID: &str = "v2";

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
        payload: RpgEventV1,
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
    Rpg(RpgEventV1),
    Physical(PhysicalEventV1),
}
