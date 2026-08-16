use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_U16,
    CanonicalCursor, CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::cognition::{
    COGNITION_Q16_ONE, SpeechActKindV1, StructuredSpeechActV1, StructuredSpeechExchangeV1,
};
use crate::ids::{
    AssetId, ContentHash, IdentifierError, PersistentId, SchemaId, content_hash_from_bytes,
};

mod systemic_work;

use systemic_work::{read_systemic_work, write_systemic_work};

pub const WORLD_ACTIVITY_SCHEMA_VERSION: u16 = 1;
pub const WORLD_ACTIVITY_COMMAND_SCHEMA_VERSION: u32 = 1;
pub const WORLD_ACTIVITY_EVENT_SCHEMA_VERSION: u32 = 1;
pub const WORLD_ACTIVITY_CATALOG_OWNER_ID: &str = "nextengine.assets";
pub const WORLD_ACTIVITY_CATALOG_SCHEMA_ID: &str = "nextengine.content.world-activity-catalog";
pub const WORLD_ACTIVITY_CATALOG_SEGMENT_ID: &str = "nextengine.world-activity-catalog.v1";
pub const WORLD_ACTIVITY_SNAPSHOT_OWNER_ID: &str = "nextengine.world-services";
pub const WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID: &str = "nextengine.world-activity-snapshot";
pub const WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID: &str = "world-activity";
pub const WORLD_ACTIVITY_COMMAND_SCHEMA_ID: &str = "nextengine.command.world-activity";
pub const WORLD_ACTIVITY_COMMAND_KIND_ID: &str = "nextengine.command-kind.world-activity";
pub const WORLD_ACTIVITY_EVENT_SCHEMA_ID: &str = "nextengine.event.world-activity-changed";
pub const WORLD_ACTIVITY_CAPABILITY_ID: &str = "nextengine.capability.world-activity-commit";
pub const WORLD_ACTIVITY_SYSTEM_ID: &str = "nextengine.system.world-activity-boundary";
pub const WORLD_ACTIVITY_CAPABILITY_SUBJECT_ID: &str =
    "nextengine.capability-subject.world-activity-boundary";
pub const WORLD_ACTIVITY_SHARD_PLAN_ID: &str = "nextengine.shard-plan.world-activity-single";
pub const WORLD_ACTIVITY_PRIORITY_CLASS: u16 = 280;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WorldActivityStateV1 {
    Unassigned = 1,
    Assigned = 2,
    Working = 3,
    Completed = 4,
}

impl WorldActivityStateV1 {
    fn from_tag(tag: u8) -> Result<Self, WorldActivityContractError> {
        match tag {
            1 => Ok(Self::Unassigned),
            2 => Ok(Self::Assigned),
            3 => Ok(Self::Working),
            4 => Ok(Self::Completed),
            value => Err(WorldActivityContractError::UnknownState(value)),
        }
    }

    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Unassigned => Some(Self::Assigned),
            Self::Assigned => Some(Self::Working),
            Self::Working => Some(Self::Completed),
            Self::Completed => None,
        }
    }

    const fn expected_revision(self) -> u64 {
        match self {
            Self::Unassigned => 0,
            Self::Assigned => 1,
            Self::Working => 2,
            Self::Completed => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldActivityCatalogV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub worker_subject_id: PersistentId,
    pub commitment_id: PersistentId,
    pub work_id: SchemaId,
    pub workplace_node_id: SchemaId,
    pub work_duration_ticks: u64,
    pub systemic_work: SystemicWorkProfileV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemicWorkProfileV1 {
    pub employer_character_id: PersistentId,
    pub seller_character_id: PersistentId,
    pub worker_inventory_id: PersistentId,
    pub seller_inventory_id: PersistentId,
    pub food_item_id: PersistentId,
    pub currency_resource_id: SchemaId,
    pub hunger_resource_id: SchemaId,
    pub satiety_resource_id: SchemaId,
    pub wage_amount: i32,
    pub food_price: i32,
    pub hunger_restore_amount: i32,
    pub satiety_gain_amount: i32,
    pub listener_trust_q16: u32,
    pub social_action_id: SchemaId,
    pub await_activity_action_id: SchemaId,
    pub settlement_action_id: SchemaId,
    pub social_ready_fact_id: SchemaId,
    pub activity_ready_fact_id: SchemaId,
    pub settlement_ready_fact_id: SchemaId,
    pub work_exchange: StructuredSpeechExchangeV1,
    pub threat_act: StructuredSpeechActV1,
}

impl SystemicWorkProfileV1 {
    pub fn validate(
        &self,
        worker_subject_id: PersistentId,
    ) -> Result<(), WorldActivityContractError> {
        self.work_exchange
            .validate_work_exchange()
            .map_err(|_| WorldActivityContractError::ContentInvalid)?;
        self.threat_act
            .validate()
            .map_err(|_| WorldActivityContractError::ContentInvalid)?;
        let acts = &self.work_exchange.acts;
        let participants_valid = acts[0].speaker_id == worker_subject_id
            && acts[0].listener_id == self.employer_character_id
            && acts[1].speaker_id == self.employer_character_id
            && acts[1].listener_id == worker_subject_id
            && acts[2].speaker_id == self.employer_character_id
            && acts[2].listener_id == worker_subject_id
            && acts[3].speaker_id == worker_subject_id
            && acts[3].listener_id == self.employer_character_id
            && self.threat_act.kind == SpeechActKindV1::Threaten
            && self.threat_act.speaker_id == self.seller_character_id
            && self.threat_act.listener_id == worker_subject_id
            && self.threat_act.creation_tick > acts[3].creation_tick;
        let identities = [
            worker_subject_id,
            self.employer_character_id,
            self.seller_character_id,
            self.worker_inventory_id,
            self.seller_inventory_id,
            self.food_item_id,
        ];
        let identities_unique = identities.iter().enumerate().all(|(index, value)| {
            *value != PersistentId::default() && !identities[index + 1..].contains(value)
        });
        let resources = [
            &self.currency_resource_id,
            &self.hunger_resource_id,
            &self.satiety_resource_id,
        ];
        let resources_unique = resources
            .iter()
            .enumerate()
            .all(|(index, value)| !resources[index + 1..].contains(value));
        let actions = [
            &self.social_action_id,
            &self.await_activity_action_id,
            &self.settlement_action_id,
        ];
        let actions_unique = actions
            .iter()
            .enumerate()
            .all(|(index, value)| !actions[index + 1..].contains(value));
        let facts = [
            &self.social_ready_fact_id,
            &self.activity_ready_fact_id,
            &self.settlement_ready_fact_id,
        ];
        let facts_unique = facts
            .iter()
            .enumerate()
            .all(|(index, value)| !facts[index + 1..].contains(value));
        if !participants_valid
            || !identities_unique
            || !resources_unique
            || !actions_unique
            || !facts_unique
            || self.wage_amount <= 0
            || self.food_price <= 0
            || self.wage_amount < self.food_price
            || self.hunger_restore_amount <= 0
            || self.satiety_gain_amount <= 0
            || self.listener_trust_q16 > COGNITION_Q16_ONE.unsigned_abs()
        {
            return Err(WorldActivityContractError::ContentInvalid);
        }
        Ok(())
    }
}

impl WorldActivityCatalogV1 {
    pub fn validate(&self) -> Result<(), WorldActivityContractError> {
        if self.schema_version != WORLD_ACTIVITY_SCHEMA_VERSION || self.work_duration_ticks == 0 {
            return Err(WorldActivityContractError::ContentInvalid);
        }
        self.systemic_work.validate(self.worker_subject_id)?;
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, WorldActivityContractError> {
        self.validate()?;
        let mut payload = Writer::default();
        payload.id(self.worker_subject_id);
        payload.id(self.commitment_id);
        payload.schema_id(&self.work_id)?;
        payload.schema_id(&self.workplace_node_id)?;
        payload.u64(self.work_duration_ticks);
        write_systemic_work(&mut payload, &self.systemic_work)?;
        Ok(encode_canonical_segment(
            WORLD_ACTIVITY_CATALOG_OWNER_ID,
            WORLD_ACTIVITY_CATALOG_SCHEMA_ID,
            WORLD_ACTIVITY_CATALOG_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.catalog_asset_id.as_bytes()),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, payload.finish()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldActivityContractError> {
        let fields = decode_contract(
            bytes,
            limits,
            WORLD_ACTIVITY_CATALOG_OWNER_ID,
            WORLD_ACTIVITY_CATALOG_SCHEMA_ID,
            WORLD_ACTIVITY_CATALOG_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let mut payload = Reader::new(field(&fields, 3)?, limits);
        let value = Self {
            schema_version: read_u16(field(&fields, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&fields, 2)?)?),
            worker_subject_id: payload.id()?,
            commitment_id: payload.id()?,
            work_id: payload.schema_id()?,
            workplace_node_id: payload.schema_id()?,
            work_duration_ticks: payload.u64()?,
            systemic_work: read_systemic_work(&mut payload)?,
        };
        payload.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(WorldActivityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn revision(&self) -> Result<ContentHash, WorldActivityContractError> {
        let bytes = self.canonical_bytes()?;
        Ok(domain_hash(WORLD_ACTIVITY_CATALOG_SEGMENT_ID, &bytes))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldActivitySnapshotV1 {
    pub schema_version: u16,
    pub catalog_asset_id: AssetId,
    pub catalog_revision: ContentHash,
    pub worker_subject_id: PersistentId,
    pub record_revision: u64,
    pub state: WorldActivityStateV1,
    pub assigned_tick_or_none: Option<u64>,
    pub work_started_tick_or_none: Option<u64>,
    pub completed_tick_or_none: Option<u64>,
}

impl WorldActivitySnapshotV1 {
    pub fn initial(catalog: &WorldActivityCatalogV1) -> Result<Self, WorldActivityContractError> {
        catalog.validate()?;
        Ok(Self {
            schema_version: WORLD_ACTIVITY_SCHEMA_VERSION,
            catalog_asset_id: catalog.catalog_asset_id,
            catalog_revision: catalog.revision()?,
            worker_subject_id: catalog.worker_subject_id,
            record_revision: 0,
            state: WorldActivityStateV1::Unassigned,
            assigned_tick_or_none: None,
            work_started_tick_or_none: None,
            completed_tick_or_none: None,
        })
    }

    pub fn validate_against(
        &self,
        catalog: &WorldActivityCatalogV1,
        next_simulation_tick: u64,
    ) -> Result<(), WorldActivityContractError> {
        catalog.validate()?;
        let timestamps_valid = match self.state {
            WorldActivityStateV1::Unassigned => {
                self.assigned_tick_or_none.is_none()
                    && self.work_started_tick_or_none.is_none()
                    && self.completed_tick_or_none.is_none()
            }
            WorldActivityStateV1::Assigned => {
                self.assigned_tick_or_none
                    .is_some_and(|assigned| assigned < next_simulation_tick)
                    && self.work_started_tick_or_none.is_none()
                    && self.completed_tick_or_none.is_none()
            }
            WorldActivityStateV1::Working => self
                .assigned_tick_or_none
                .zip(self.work_started_tick_or_none)
                .is_some_and(|(assigned, started)| {
                    assigned < started
                        && started < next_simulation_tick
                        && self.completed_tick_or_none.is_none()
                }),
            WorldActivityStateV1::Completed => self
                .assigned_tick_or_none
                .zip(self.work_started_tick_or_none)
                .zip(self.completed_tick_or_none)
                .is_some_and(|((assigned, started), completed)| {
                    assigned < started
                        && started < completed
                        && completed < next_simulation_tick
                        && completed.saturating_sub(started) >= catalog.work_duration_ticks
                }),
        };
        if self.schema_version != WORLD_ACTIVITY_SCHEMA_VERSION
            || self.catalog_asset_id != catalog.catalog_asset_id
            || self.catalog_revision != catalog.revision()?
            || self.worker_subject_id != catalog.worker_subject_id
            || self.record_revision != self.state.expected_revision()
            || !timestamps_valid
        {
            return Err(WorldActivityContractError::SnapshotClosureInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, WorldActivityContractError> {
        let mut payload = Writer::default();
        payload.id(self.worker_subject_id);
        payload.u64(self.record_revision);
        payload.u8(self.state as u8);
        payload.optional_u64(self.assigned_tick_or_none);
        payload.optional_u64(self.work_started_tick_or_none);
        payload.optional_u64(self.completed_tick_or_none);
        Ok(encode_canonical_segment(
            WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
            WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
            WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.catalog_asset_id.as_bytes()),
                field_hash(3, self.catalog_revision),
                CanonicalField::new(4, CANONICAL_TYPE_BYTES, payload.finish()),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldActivityContractError> {
        let fields = decode_contract(
            bytes,
            limits,
            WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
            WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
            WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_HASH256),
                (4, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let mut payload = Reader::new(field(&fields, 4)?, limits);
        let value = Self {
            schema_version: read_u16(field(&fields, 1)?)?,
            catalog_asset_id: AssetId::from_bytes(read_exact(field(&fields, 2)?)?),
            catalog_revision: ContentHash::from_bytes(read_exact(field(&fields, 3)?)?),
            worker_subject_id: payload.id()?,
            record_revision: payload.u64()?,
            state: WorldActivityStateV1::from_tag(payload.u8()?)?,
            assigned_tick_or_none: payload.optional_u64()?,
            work_started_tick_or_none: payload.optional_u64()?,
            completed_tick_or_none: payload.optional_u64()?,
        };
        payload.finish()?;
        if value.schema_version != WORLD_ACTIVITY_SCHEMA_VERSION {
            return Err(WorldActivityContractError::UnsupportedVersion(
                value.schema_version,
            ));
        }
        if value.canonical_bytes()? != bytes {
            return Err(WorldActivityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorldActivityEvidenceV1 {
    AcceptedCommitment {
        commitment_id: PersistentId,
        commitment_revision: u64,
    },
    WorkplacePresence {
        population_record_revision: u64,
        population_snapshot_hash: ContentHash,
    },
    ElapsedWork {
        work_started_tick: u64,
    },
}

impl WorldActivityEvidenceV1 {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Self::AcceptedCommitment {
                commitment_id,
                commitment_revision,
            } => {
                bytes.push(1);
                bytes.extend_from_slice(commitment_id.as_bytes());
                bytes.extend_from_slice(&commitment_revision.to_le_bytes());
            }
            Self::WorkplacePresence {
                population_record_revision,
                population_snapshot_hash,
            } => {
                bytes.push(2);
                bytes.extend_from_slice(&population_record_revision.to_le_bytes());
                bytes.extend_from_slice(population_snapshot_hash.as_bytes());
            }
            Self::ElapsedWork { work_started_tick } => {
                bytes.push(3);
                bytes.extend_from_slice(&work_started_tick.to_le_bytes());
            }
        }
        bytes
    }

    pub fn canonical_hash(&self) -> ContentHash {
        domain_hash(
            "nextengine.world-activity-evidence.v1",
            &self.canonical_bytes(),
        )
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, WorldActivityContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let value = match cursor.read_u8()? {
            1 => Self::AcceptedCommitment {
                commitment_id: read_id(&mut cursor)?,
                commitment_revision: cursor.read_u64()?,
            },
            2 => Self::WorkplacePresence {
                population_record_revision: cursor.read_u64()?,
                population_snapshot_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
            },
            3 => Self::ElapsedWork {
                work_started_tick: cursor.read_u64()?,
            },
            value => return Err(WorldActivityContractError::UnknownEvidence(value)),
        };
        cursor.finish()?;
        if value.canonical_bytes() != bytes {
            return Err(WorldActivityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorldActivityCommandV1 {
    Transition {
        subject_id: PersistentId,
        expected_record_revision: u64,
        catalog_asset_id: AssetId,
        catalog_revision: ContentHash,
        previous_state: WorldActivityStateV1,
        current_state: WorldActivityStateV1,
        boundary_tick: u64,
        evidence: WorldActivityEvidenceV1,
    },
}

impl WorldActivityCommandV1 {
    #[must_use]
    pub const fn subject_id(&self) -> PersistentId {
        match self {
            Self::Transition { subject_id, .. } => *subject_id,
        }
    }

    pub fn validate_shape(&self) -> Result<(), WorldActivityContractError> {
        let Self::Transition {
            expected_record_revision,
            previous_state,
            current_state,
            evidence,
            ..
        } = self;
        let evidence_matches = matches!(
            (previous_state, current_state, evidence),
            (
                WorldActivityStateV1::Unassigned,
                WorldActivityStateV1::Assigned,
                WorldActivityEvidenceV1::AcceptedCommitment { .. }
            ) | (
                WorldActivityStateV1::Assigned,
                WorldActivityStateV1::Working,
                WorldActivityEvidenceV1::WorkplacePresence { .. }
            ) | (
                WorldActivityStateV1::Working,
                WorldActivityStateV1::Completed,
                WorldActivityEvidenceV1::ElapsedWork { .. }
            )
        );
        if previous_state.next() != Some(*current_state)
            || *expected_record_revision != previous_state.expected_revision()
            || !evidence_matches
        {
            return Err(WorldActivityContractError::CommandInvalid);
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, WorldActivityContractError> {
        self.validate_shape()?;
        let Self::Transition {
            subject_id,
            expected_record_revision,
            catalog_asset_id,
            catalog_revision,
            previous_state,
            current_state,
            boundary_tick,
            evidence,
        } = self;
        let evidence = evidence.canonical_bytes();
        let mut bytes = vec![1];
        bytes.extend_from_slice(subject_id.as_bytes());
        bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
        bytes.extend_from_slice(catalog_asset_id.as_bytes());
        bytes.extend_from_slice(catalog_revision.as_bytes());
        bytes.push(*previous_state as u8);
        bytes.push(*current_state as u8);
        bytes.extend_from_slice(&boundary_tick.to_le_bytes());
        extend_bytes(&mut bytes, &evidence)?;
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldActivityContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        if cursor.read_u8()? != 1 {
            return Err(WorldActivityContractError::CommandInvalid);
        }
        let value = Self::Transition {
            subject_id: read_id(&mut cursor)?,
            expected_record_revision: cursor.read_u64()?,
            catalog_asset_id: AssetId::from_bytes(read_array(&mut cursor)?),
            catalog_revision: ContentHash::from_bytes(read_array(&mut cursor)?),
            previous_state: WorldActivityStateV1::from_tag(cursor.read_u8()?)?,
            current_state: WorldActivityStateV1::from_tag(cursor.read_u8()?)?,
            boundary_tick: cursor.read_u64()?,
            evidence: WorldActivityEvidenceV1::from_canonical_bytes(
                cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?,
            )?,
        };
        cursor.finish()?;
        value.validate_shape()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(WorldActivityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }

    pub fn owner_delta_bytes(&self) -> Result<Vec<u8>, WorldActivityContractError> {
        self.validate_shape()?;
        let Self::Transition {
            subject_id,
            expected_record_revision,
            catalog_asset_id,
            catalog_revision,
            previous_state,
            current_state,
            boundary_tick,
            evidence,
        } = self;
        let after = expected_record_revision
            .checked_add(1)
            .ok_or(WorldActivityContractError::RevisionExhausted)?;
        let mut bytes = b"nextengine.world-activity-owner-write-set.v1\0".to_vec();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(subject_id.as_bytes());
        bytes.extend_from_slice(&expected_record_revision.to_le_bytes());
        bytes.extend_from_slice(&after.to_le_bytes());
        bytes.extend_from_slice(catalog_asset_id.as_bytes());
        bytes.extend_from_slice(catalog_revision.as_bytes());
        bytes.push(*previous_state as u8);
        bytes.push(*current_state as u8);
        bytes.extend_from_slice(&boundary_tick.to_le_bytes());
        bytes.extend_from_slice(evidence.canonical_hash().as_bytes());
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldActivityChangedV1 {
    pub subject_id: PersistentId,
    pub previous_state: WorldActivityStateV1,
    pub current_state: WorldActivityStateV1,
    pub boundary_tick: u64,
    pub record_revision: u64,
    pub evidence_hash: ContentHash,
}

impl WorldActivityChangedV1 {
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, WorldActivityContractError> {
        if self.previous_state.next() != Some(self.current_state)
            || self.record_revision != self.current_state.expected_revision()
            || self.evidence_hash == ContentHash::default()
        {
            return Err(WorldActivityContractError::EventInvalid);
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.subject_id.as_bytes());
        bytes.push(self.previous_state as u8);
        bytes.push(self.current_state as u8);
        bytes.extend_from_slice(&self.boundary_tick.to_le_bytes());
        bytes.extend_from_slice(&self.record_revision.to_le_bytes());
        bytes.extend_from_slice(self.evidence_hash.as_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_payload_bytes(bytes: &[u8]) -> Result<Self, WorldActivityContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let value = Self {
            subject_id: read_id(&mut cursor)?,
            previous_state: WorldActivityStateV1::from_tag(cursor.read_u8()?)?,
            current_state: WorldActivityStateV1::from_tag(cursor.read_u8()?)?,
            boundary_tick: cursor.read_u64()?,
            record_revision: cursor.read_u64()?,
            evidence_hash: ContentHash::from_bytes(read_array(&mut cursor)?),
        };
        cursor.finish()?;
        if value.canonical_payload_bytes()? != bytes {
            return Err(WorldActivityContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldActivityContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    UnsupportedVersion(u16),
    UnknownState(u8),
    UnknownEvidence(u8),
    ContentInvalid,
    SnapshotClosureInvalid,
    CommandInvalid,
    EventInvalid,
    RevisionExhausted,
    WrongEnvelope,
    FieldSetInvalid,
    FieldLength,
    NonCanonicalEncoding,
}

impl WorldActivityContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion(_) => "WORLD_ACTIVITY_SCHEMA_UNSUPPORTED",
            Self::ContentInvalid => "WORLD_ACTIVITY_CONTENT_INVALID",
            Self::SnapshotClosureInvalid => "WORLD_ACTIVITY_SNAPSHOT_CLOSURE_INVALID",
            Self::CommandInvalid => "WORLD_ACTIVITY_COMMAND_INVALID",
            Self::EventInvalid => "WORLD_ACTIVITY_EVENT_INVALID",
            Self::RevisionExhausted => "WORLD_ACTIVITY_REVISION_EXHAUSTED",
            _ => "WORLD_ACTIVITY_CONTRACT_INVALID",
        }
    }
}

impl Display for WorldActivityContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.diagnostic_code())
    }
}

impl Error for WorldActivityContractError {}

impl From<CanonicalError> for WorldActivityContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for WorldActivityContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for WorldActivityContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn id(&mut self, value: PersistentId) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn schema_id(&mut self, value: &SchemaId) -> Result<(), WorldActivityContractError> {
        extend_bytes(&mut self.bytes, value.as_str().as_bytes())?;
        Ok(())
    }

    fn bytes(&mut self, value: &[u8]) -> Result<(), WorldActivityContractError> {
        extend_bytes(&mut self.bytes, value)?;
        Ok(())
    }

    fn optional_u64(&mut self, value: Option<u64>) {
        match value {
            None => self.u8(0),
            Some(value) => {
                self.u8(1);
                self.u64(value);
            }
        }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct Reader<'a> {
    cursor: CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            cursor: CanonicalCursor::new(bytes),
            limits,
        }
    }

    fn u8(&mut self) -> Result<u8, WorldActivityContractError> {
        Ok(self.cursor.read_u8()?)
    }

    fn u64(&mut self) -> Result<u64, WorldActivityContractError> {
        Ok(self.cursor.read_u64()?)
    }

    fn u32(&mut self) -> Result<u32, WorldActivityContractError> {
        Ok(self.cursor.read_u32()?)
    }

    fn i32(&mut self) -> Result<i32, WorldActivityContractError> {
        Ok(i32::from_le_bytes(read_array(&mut self.cursor)?))
    }

    fn id(&mut self) -> Result<PersistentId, WorldActivityContractError> {
        Ok(PersistentId::from_bytes(read_array(&mut self.cursor)?))
    }

    fn schema_id(&mut self) -> Result<SchemaId, WorldActivityContractError> {
        let bytes = self
            .cursor
            .read_u32_length_prefixed(self.limits.max_field_payload_bytes)?;
        Ok(SchemaId::new(std::str::from_utf8(bytes).map_err(
            |_| WorldActivityContractError::ContentInvalid,
        )?)?)
    }

    fn bytes(&mut self) -> Result<&'a [u8], WorldActivityContractError> {
        Ok(self
            .cursor
            .read_u32_length_prefixed(self.limits.max_field_payload_bytes)?)
    }

    fn optional_u64(&mut self) -> Result<Option<u64>, WorldActivityContractError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u64()?)),
            _ => Err(WorldActivityContractError::SnapshotClosureInvalid),
        }
    }

    fn finish(self) -> Result<(), WorldActivityContractError> {
        Ok(self.cursor.finish()?)
    }
}

fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment: &str,
    expected: &[(u32, u8)],
) -> Result<Vec<CanonicalField>, WorldActivityContractError> {
    let decoded = decode_canonical_segment(bytes, limits)?;
    if decoded.owner_id != owner || decoded.schema_id != schema || decoded.segment_id != segment {
        return Err(WorldActivityContractError::WrongEnvelope);
    }
    if decoded.fields.len() != expected.len()
        || decoded
            .fields
            .iter()
            .zip(expected)
            .any(|(field, expected)| (field.field_id, field.type_tag) != *expected)
    {
        return Err(WorldActivityContractError::FieldSetInvalid);
    }
    Ok(decoded.fields)
}

fn field(fields: &[CanonicalField], id: u32) -> Result<&[u8], WorldActivityContractError> {
    Ok(&fields
        .iter()
        .find(|field| field.field_id == id)
        .ok_or(WorldActivityContractError::FieldSetInvalid)?
        .payload)
}

fn field_u16(id: u32, value: u16) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U16, value.to_le_bytes().to_vec())
}

fn field_id(id: u32, value: &[u8; 16]) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.to_vec())
}

fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

fn read_u16(bytes: &[u8]) -> Result<u16, WorldActivityContractError> {
    Ok(u16::from_le_bytes(read_exact(bytes)?))
}

fn read_exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], WorldActivityContractError> {
    bytes
        .try_into()
        .map_err(|_| WorldActivityContractError::FieldLength)
}

fn read_id(cursor: &mut CanonicalCursor<'_>) -> Result<PersistentId, WorldActivityContractError> {
    Ok(PersistentId::from_bytes(read_array(cursor)?))
}

fn read_array<const N: usize>(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<[u8; N], WorldActivityContractError> {
    cursor
        .read_exact(N)?
        .try_into()
        .map_err(|_| WorldActivityContractError::FieldLength)
}

fn extend_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), CanonicalError> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}

fn domain_hash(domain: &str, bytes: &[u8]) -> ContentHash {
    let mut preimage = domain.as_bytes().to_vec();
    preimage.push(0);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .expect("world activity payload fits u64")
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    content_hash_from_bytes(sha256(&preimage))
}

#[cfg(test)]
mod tests;
