use crate::canonical::{CanonicalDecodeLimits, CanonicalError, sha256};
use crate::cognition::{
    AGENT_MEMORY_SNAPSHOT_OWNER_ID, AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
    AGENT_MEMORY_SNAPSHOT_SEGMENT_ID, AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
    AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID, AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID, AgentCognitionSnapshotV1,
    AgentMemorySnapshotV1,
};
use crate::command::{CommandDecodeError, DomainEvent, IssuerPrincipal, WorldCommand};
use crate::ids::{
    CapabilityId, CommandId, CommandLedgerHash, ContentHash, SchemaId, StateRoot, WorldNamespaceId,
    content_hash_from_bytes,
};
use crate::input::{ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1, InputMappingReceiptV2};
use crate::physical_animation::{
    PHYSICAL_ANIMATION_SCHEMA_VERSION, PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID,
    PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID, PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID,
    PhysicalAnimationSnapshotV1,
};
use crate::physics::{
    ClosedPhysicsContactBatchV1, PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID, PhysicsQueryBatchV1, PhysicsQueryResultV1,
    PhysicsStepInputV2, PhysicsWorldCheckpointV1,
};
use crate::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID, RpgSnapshotV2,
};
use crate::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SEGMENT_ID,
    RuntimeSnapshotV3, WorldCheckpointV4,
};
use crate::targeting::{
    AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION, AuthoritativeTargetingQueryV1,
    TargetingContractError, TargetingIntentV1,
};
use crate::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SEGMENT_ID, WorldStreamingSnapshotV1,
};
use crate::world_activity::{
    WORLD_ACTIVITY_SNAPSHOT_OWNER_ID, WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
    WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID, WorldActivitySnapshotV1,
};
use crate::world_population::{
    WORLD_POPULATION_SNAPSHOT_OWNER_ID, WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
    WORLD_POPULATION_SNAPSHOT_SEGMENT_ID, WorldPopulationSnapshotV1,
};
use crate::world_routine::{
    InteractionAvailabilityV1, WORLD_ROUTINE_SNAPSHOT_OWNER_ID, WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
    WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID, WorldRoutineSnapshotV1,
};

mod error_impl;

pub const SAVE_MANIFEST_SCHEMA_VERSION: u32 = 2;
pub const RETIRED_REPLAY_MANIFEST_V5_SCHEMA_VERSION: u32 = 5;
pub const RETIRED_REPLAY_MANIFEST_V6_SCHEMA_VERSION: u32 = 6;
pub const RETIRED_REPLAY_MANIFEST_V7_SCHEMA_VERSION: u32 = 7;
pub const RETIRED_REPLAY_MANIFEST_V8_SCHEMA_VERSION: u32 = 8;
pub const SAVE_MANIFEST_V3_SCHEMA_VERSION: u32 = 3;
pub const REPLAY_MANIFEST_V9_SCHEMA_VERSION: u32 = 9;
pub const REPLAY_MANIFEST_V10_SCHEMA_VERSION: u32 = 10;
pub const PHYSICAL_TRAINING_REPLAY_MANIFEST_SCHEMA_VERSION: u32 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickSettings {
    pub gameplay_hz: u32,
    pub physics_hz: u32,
    pub motor_hz: u32,
}

impl TickSettings {
    pub fn validate(self) -> Result<(), ManifestValidationError> {
        if self.gameplay_hz == 0 || self.physics_hz == 0 || self.motor_hz == 0 {
            return Err(ManifestValidationError::InvalidTickSettings);
        }
        if !self.physics_hz.is_multiple_of(self.gameplay_hz)
            || !self.physics_hz.is_multiple_of(self.motor_hz)
        {
            return Err(ManifestValidationError::InvalidTickSettings);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HashBinding {
    pub binding_id: SchemaId,
    pub content_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaBinding {
    pub schema_id: SchemaId,
    pub schema_version: u32,
    pub content_hash: ContentHash,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CommandLedgerDescriptorV2 {
    pub world_namespace: WorldNamespaceId,
    pub stream_count: u64,
    pub archive_root: ContentHash,
    pub identity_index_root: ContentHash,
    pub runtime_snapshot_segment_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SaveSegmentDescriptor {
    pub owner_id: SchemaId,
    pub schema_id: SchemaId,
    pub segment_id: SchemaId,
    pub schema_version: u32,
    pub byte_length: u64,
    pub content_hash: ContentHash,
}

impl SaveSegmentDescriptor {
    pub fn for_bytes(
        owner_id: SchemaId,
        schema_id: SchemaId,
        segment_id: SchemaId,
        schema_version: u32,
        bytes: &[u8],
    ) -> Result<Self, CanonicalError> {
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.state-segment.v1\0");
        preimage.extend_from_slice(
            &u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(bytes);
        Ok(Self {
            owner_id,
            schema_id,
            segment_id,
            schema_version,
            byte_length: u64::try_from(bytes.len()).map_err(|_| CanonicalError::LengthOverflow)?,
            content_hash: content_hash_from_bytes(sha256(&preimage)),
        })
    }

    #[must_use]
    pub fn matches_bytes(&self, bytes: &[u8]) -> bool {
        let Ok(actual) = Self::for_bytes(
            self.owner_id.clone(),
            self.schema_id.clone(),
            self.segment_id.clone(),
            self.schema_version,
            bytes,
        ) else {
            return false;
        };
        actual.byte_length == self.byte_length && actual.content_hash == self.content_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveCompatibility {
    pub engine_build_hash: ContentHash,
    pub game_build_hash: ContentHash,
    pub project_id: SchemaId,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub mechanics_lock_hash: ContentHash,
    pub tick_settings: TickSettings,
    pub loaded_chunk_revisions: Vec<HashBinding>,
    pub rng_stream_states: Vec<HashBinding>,
    pub physical_bindings: Vec<HashBinding>,
    pub policy_state_schemas: Vec<SchemaBinding>,
    pub plugin_script_bindings: Vec<HashBinding>,
}

impl SaveCompatibility {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        self.tick_settings.validate()?;
        validate_sorted_unique_hash_bindings(&self.loaded_chunk_revisions)?;
        validate_sorted_unique_hash_bindings(&self.rng_stream_states)?;
        validate_sorted_unique_hash_bindings(&self.physical_bindings)?;
        validate_sorted_unique_schema_bindings(&self.policy_state_schemas)?;
        validate_sorted_unique_hash_bindings(&self.plugin_script_bindings)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveManifestV2 {
    pub schema_version: u32,
    pub generation: u64,
    pub world_revision: u64,
    pub compatibility: SaveCompatibility,
    pub command_ledger: CommandLedgerDescriptorV2,
    pub segments: Vec<SaveSegmentDescriptor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalTrainingCompatibilityV1 {
    pub body_schema_hash: ContentHash,
    pub body_instance_projection_hash: ContentHash,
    pub physics_catalog_hash: ContentHash,
    pub observation_layout_hash: ContentHash,
    pub action_layout_hash: ContentHash,
    pub physics_build_profile_hash: ContentHash,
    pub scene_profile_hash: ContentHash,
    pub bridge_abi_hash: ContentHash,
    pub quantization_profile_hash: ContentHash,
}

impl PhysicalTrainingCompatibilityV1 {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if [
            self.body_schema_hash,
            self.body_instance_projection_hash,
            self.physics_catalog_hash,
            self.observation_layout_hash,
            self.action_layout_hash,
            self.physics_build_profile_hash,
            self.scene_profile_hash,
            self.bridge_abi_hash,
            self.quantization_profile_hash,
        ]
        .contains(&ContentHash::default())
        {
            return Err(ManifestValidationError::CompatibilityMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveManifestV3 {
    pub schema_version: u32,
    pub generation: u64,
    pub world_revision: u64,
    pub compatibility: SaveCompatibility,
    pub physical_training: PhysicalTrainingCompatibilityV1,
    pub command_ledger: CommandLedgerDescriptorV2,
    pub segments: Vec<SaveSegmentDescriptor>,
    pub world_checkpoint_v5_hash: ContentHash,
}

impl SaveManifestV3 {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != SAVE_MANIFEST_V3_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedSaveVersion(
                self.schema_version,
            ));
        }
        self.compatibility.validate()?;
        self.physical_training.validate()?;
        if self.segments.is_empty()
            || self.world_checkpoint_v5_hash == ContentHash::default()
            || self.segments.windows(2).any(|pair| {
                (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
                    >= (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
            })
        {
            return Err(ManifestValidationError::SegmentsNotStrictlySorted);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalTrainingReplayManifestV6 {
    pub schema_version: u32,
    pub replay_id: SchemaId,
    pub physical_training: PhysicalTrainingCompatibilityV1,
    pub initial_world_checkpoint_v5_hash: ContentHash,
    pub motor_trajectory_manifest_hash: ContentHash,
    pub canonical_applied_action_stream_hash: ContentHash,
    pub tick_count: u64,
    pub final_state_root: StateRoot,
    pub final_ledger_root: CommandLedgerHash,
}

impl PhysicalTrainingReplayManifestV6 {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != PHYSICAL_TRAINING_REPLAY_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedReplayVersion(
                self.schema_version,
            ));
        }
        self.physical_training.validate()?;
        if self.tick_count == 0
            || self.initial_world_checkpoint_v5_hash == ContentHash::default()
            || self.motor_trajectory_manifest_hash == ContentHash::default()
            || self.canonical_applied_action_stream_hash == ContentHash::default()
        {
            return Err(ManifestValidationError::CompatibilityMismatch);
        }
        Ok(())
    }
}

impl SaveManifestV2 {
    pub fn for_runtime_snapshot(
        generation: u64,
        compatibility: SaveCompatibility,
        snapshot: &RuntimeSnapshotV3,
        snapshot_bytes: &[u8],
    ) -> Result<Self, ManifestValidationError> {
        let segment = SaveSegmentDescriptor::for_bytes(
            SchemaId::new(crate::snapshot::RUNTIME_SNAPSHOT_OWNER_ID)?,
            SchemaId::new(crate::snapshot::RUNTIME_SNAPSHOT_SCHEMA_ID)?,
            SchemaId::new(crate::snapshot::RUNTIME_SNAPSHOT_SEGMENT_ID)?,
            crate::snapshot::RUNTIME_SNAPSHOT_SCHEMA_VERSION,
            snapshot_bytes,
        )?;
        let command_ledger = CommandLedgerDescriptorV2 {
            world_namespace: snapshot.world_identity.world_namespace,
            stream_count: u64::try_from(snapshot.command_ledger.streams.len())
                .map_err(|_| CanonicalError::LengthOverflow)?,
            archive_root: snapshot.command_ledger.body_archive.archive_root,
            identity_index_root: snapshot.command_ledger.identity_index.index_root,
            runtime_snapshot_segment_hash: segment.content_hash,
        };
        let manifest = Self {
            schema_version: SAVE_MANIFEST_SCHEMA_VERSION,
            generation,
            world_revision: snapshot.authoritative_revision,
            compatibility,
            command_ledger,
            segments: vec![segment],
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != SAVE_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedSaveVersion(
                self.schema_version,
            ));
        }
        self.compatibility.validate()?;
        if self.segments.is_empty() {
            return Err(ManifestValidationError::MissingRequiredSegment);
        }
        if self.segments.windows(2).any(|pair| {
            (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
                >= (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
        }) {
            return Err(ManifestValidationError::SegmentsNotStrictlySorted);
        }
        Ok(())
    }

    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ManifestCodecError> {
        crate::manifest_jcs::encode_save_manifest(self)
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ManifestCodecError> {
        crate::manifest_jcs::decode_save_manifest(bytes, limits)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityGrant {
    pub principal: IssuerPrincipal,
    pub capabilities: Vec<CapabilityId>,
}

impl AuthorityGrant {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.capabilities.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ManifestValidationError::CapabilitiesNotStrictlySorted);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCommandRecord {
    pub envelope_schema_version: u16,
    pub claimed_command_id: Option<CommandId>,
    pub command_id: CommandId,
    pub canonical_command_bytes: Vec<u8>,
}

impl ReplayCommandRecord {
    pub fn from_command(command: &WorldCommand) -> Result<Self, CanonicalError> {
        Ok(Self {
            envelope_schema_version: command.envelope_schema_version,
            claimed_command_id: command.claimed_command_id,
            command_id: command.compute_command_id()?,
            canonical_command_bytes: command.canonical_bytes()?,
        })
    }

    pub fn decode_command(
        &self,
        limits: CanonicalDecodeLimits,
    ) -> Result<WorldCommand, ManifestValidationError> {
        let mut command =
            WorldCommand::from_canonical_bytes(&self.canonical_command_bytes, limits)?;
        if command.compute_command_id()? != self.command_id {
            return Err(ManifestValidationError::ReplayCommandIdMismatch);
        }
        command.envelope_schema_version = self.envelope_schema_version;
        command.claimed_command_id = self.claimed_command_id;
        Ok(command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCommandResultV2 {
    pub command_id: CommandId,
    pub sequence: u64,
    pub disposition_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayOwnerSegmentV2 {
    pub descriptor: SaveSegmentDescriptor,
    pub canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldStreamingReplayInputV1 {
    None,
    BeginTransition {
        target_chunk_id: SchemaId,
        expected_base_world_state_hash: ContentHash,
        expected_next_world_state_hash: ContentHash,
    },
    CompletePendingTransition {
        target_chunk_id: SchemaId,
        expected_base_world_state_hash: ContentHash,
        expected_loaded_result_hash: ContentHash,
        expected_next_world_state_hash: ContentHash,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTickManifestV9 {
    pub tick: u64,
    pub world_streaming_input: WorldStreamingReplayInputV1,
    pub closed_ingress_batch: ClosedIngressBatchV1,
    pub direct_external_commands: Vec<ReplayCommandRecord>,
    pub expected_ingress_command_batch: ClosedCommandAdmissionBatchV2,
    pub expected_physics_step_input: PhysicsStepInputV2,
    pub expected_contact_batch: ClosedPhysicsContactBatchV1,
    pub expected_targeting_intents: Vec<TargetingIntentV1>,
    pub expected_authoritative_targeting_queries: Vec<AuthoritativeTargetingQueryV1>,
    pub expected_physics_query_batch: PhysicsQueryBatchV1,
    pub expected_physics_query_results: Vec<PhysicsQueryResultV1>,
    pub expected_outcome_command_batch: ClosedCommandAdmissionBatchV2,
    pub expected_mapping_receipts: Vec<InputMappingReceiptV2>,
    pub expected_interaction_availability: Vec<InteractionAvailabilityV1>,
    pub expected_command_results: Vec<ReplayCommandResultV2>,
    pub expected_events: Vec<DomainEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayComparePointV9 {
    pub tick: u64,
    pub state_root: StateRoot,
    pub command_ledger_hash: CommandLedgerHash,
    pub owner_segments: Vec<SaveSegmentDescriptor>,
    pub closed_ingress_batch_hash: ContentHash,
    pub ingress_command_batch_hash: ContentHash,
    pub physics_step_input_hash: ContentHash,
    pub contact_batch_hash: ContentHash,
    pub physics_query_batch_hash: ContentHash,
    pub physics_query_results_hash: ContentHash,
    pub targeting_query_trace_hash: ContentHash,
    pub outcome_command_batch_hash: ContentHash,
    pub interaction_availability_hash: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayManifestV9 {
    pub schema_version: u32,
    pub compatibility: SaveCompatibility,
    pub initial_owner_segments: Vec<ReplayOwnerSegmentV2>,
    pub initial_state_root: StateRoot,
    pub authority: Vec<AuthorityGrant>,
    pub ticks: Vec<ReplayTickManifestV9>,
    pub compare_points: Vec<ReplayComparePointV9>,
}

pub type ReplayTickManifestV10 = ReplayTickManifestV9;
pub type ReplayComparePointV10 = ReplayComparePointV9;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayManifestV10 {
    pub schema_version: u32,
    pub compatibility: SaveCompatibility,
    pub initial_owner_segments: Vec<ReplayOwnerSegmentV2>,
    pub initial_state_root: StateRoot,
    pub authority: Vec<AuthorityGrant>,
    pub ticks: Vec<ReplayTickManifestV10>,
    pub compare_points: Vec<ReplayComparePointV10>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedReplayInitialStateV9 {
    pub checkpoint: WorldCheckpointV4,
    pub world_streaming_snapshot: WorldStreamingSnapshotV1,
    pub world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub world_population_snapshot: WorldPopulationSnapshotV1,
    pub world_activity_snapshot: WorldActivitySnapshotV1,
    pub agent_cognition_snapshot: AgentCognitionSnapshotV1,
    pub agent_memory_snapshot: AgentMemorySnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedReplayInitialStateV10 {
    pub checkpoint: WorldCheckpointV4,
    pub world_streaming_snapshot: WorldStreamingSnapshotV1,
    pub world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub world_population_snapshot: WorldPopulationSnapshotV1,
    pub world_activity_snapshot: WorldActivitySnapshotV1,
    pub agent_cognition_snapshot: AgentCognitionSnapshotV1,
    pub agent_memory_snapshot: AgentMemorySnapshotV1,
    pub physical_animation_snapshot: PhysicalAnimationSnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedReplayTickV9 {
    pub tick: u64,
    pub world_streaming_input: WorldStreamingReplayInputV1,
    pub closed_ingress_batch: ClosedIngressBatchV1,
    pub direct_external_commands: Vec<WorldCommand>,
    pub expected_ingress_command_batch: ClosedCommandAdmissionBatchV2,
    pub expected_physics_step_input: PhysicsStepInputV2,
    pub expected_contact_batch: ClosedPhysicsContactBatchV1,
    pub expected_targeting_intents: Vec<TargetingIntentV1>,
    pub expected_authoritative_targeting_queries: Vec<AuthoritativeTargetingQueryV1>,
    pub expected_physics_query_batch: PhysicsQueryBatchV1,
    pub expected_physics_query_results: Vec<PhysicsQueryResultV1>,
    pub expected_outcome_command_batch: ClosedCommandAdmissionBatchV2,
    pub expected_mapping_receipts: Vec<InputMappingReceiptV2>,
    pub expected_interaction_availability: Vec<InteractionAvailabilityV1>,
    pub expected_command_results: Vec<ReplayCommandResultV2>,
    pub expected_events: Vec<DomainEvent>,
}

pub type DecodedReplayTickV10 = DecodedReplayTickV9;

pub fn replay_physics_query_batch_hash(
    batch: &PhysicsQueryBatchV1,
) -> Result<ContentHash, CanonicalError> {
    replay_framed_hash(
        b"nextengine.replay-physics-query-batch.v1\0",
        [batch.canonical_bytes()?],
    )
}

pub fn replay_physics_query_results_hash(
    results: &[PhysicsQueryResultV1],
) -> Result<ContentHash, CanonicalError> {
    replay_framed_hash(
        b"nextengine.replay-physics-query-results.v1\0",
        results
            .iter()
            .map(PhysicsQueryResultV1::canonical_bytes)
            .collect::<Result<Vec<_>, _>>()?,
    )
}

pub fn replay_targeting_query_trace_hash(
    intents: &[TargetingIntentV1],
    queries: &[AuthoritativeTargetingQueryV1],
) -> Result<ContentHash, CanonicalError> {
    let records = intents
        .iter()
        .zip(queries)
        .map(|(intent, query)| {
            let intent = intent.canonical_bytes()?;
            let query = query.canonical_bytes()?;
            let mut record = Vec::new();
            record.extend_from_slice(
                &u64::try_from(intent.len())
                    .map_err(|_| CanonicalError::LengthOverflow)?
                    .to_le_bytes(),
            );
            record.extend_from_slice(&intent);
            record.extend_from_slice(
                &u64::try_from(query.len())
                    .map_err(|_| CanonicalError::LengthOverflow)?
                    .to_le_bytes(),
            );
            record.extend_from_slice(&query);
            Ok(record)
        })
        .collect::<Result<Vec<_>, CanonicalError>>()?;
    replay_framed_hash(b"nextengine.replay-targeting-query-trace.v1\0", records)
}

fn replay_framed_hash(
    domain: &[u8],
    records: impl IntoIterator<Item = Vec<u8>>,
) -> Result<ContentHash, CanonicalError> {
    let records = records.into_iter().collect::<Vec<_>>();
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(
        &u64::try_from(records.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for record in records {
        preimage.extend_from_slice(
            &u64::try_from(record.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&record);
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

impl ReplayManifestV9 {
    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ManifestCodecError> {
        crate::manifest_jcs::encode_replay_manifest_v9(self)
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ManifestCodecError> {
        crate::manifest_jcs::decode_replay_manifest_v9(bytes, limits)
    }

    pub fn validate_and_decode(
        &self,
        limits: CanonicalDecodeLimits,
    ) -> Result<(DecodedReplayInitialStateV9, Vec<DecodedReplayTickV9>), ManifestValidationError>
    {
        if self.schema_version != REPLAY_MANIFEST_V9_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedReplayVersion(
                self.schema_version,
            ));
        }
        self.compatibility.validate()?;
        if !matches!(self.initial_owner_segments.len(), 8 | 9)
            || self.initial_owner_segments.windows(2).any(|pair| {
                (
                    &pair[0].descriptor.owner_id,
                    &pair[0].descriptor.schema_id,
                    &pair[0].descriptor.segment_id,
                ) >= (
                    &pair[1].descriptor.owner_id,
                    &pair[1].descriptor.schema_id,
                    &pair[1].descriptor.segment_id,
                )
            })
            || self
                .initial_owner_segments
                .iter()
                .any(|segment| !segment.descriptor.matches_bytes(&segment.canonical_bytes))
        {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        let segment = |owner: &str, schema: &str, id: &str| {
            self.initial_owner_segments.iter().find(|segment| {
                segment.descriptor.owner_id.as_str() == owner
                    && segment.descriptor.schema_id.as_str() == schema
                    && segment.descriptor.segment_id.as_str() == id
            })
        };
        let runtime = segment(
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let rpg = segment(
            RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let physics = segment(
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let streaming = segment(
            WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let routine_or_none = segment(
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
        );
        let population = segment(
            WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let activity = segment(
            WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
            WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
            WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let agent = segment(
            AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
            AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
            AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let memory = segment(
            AGENT_MEMORY_SNAPSHOT_OWNER_ID,
            AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
            AGENT_MEMORY_SNAPSHOT_SEGMENT_ID,
        )
        .ok_or(ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        if self.initial_owner_segments.len() != 8 + usize::from(routine_or_none.is_some()) {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        let runtime_snapshot =
            RuntimeSnapshotV3::from_canonical_bytes(&runtime.canonical_bytes, limits)?;
        let rpg_snapshot = RpgSnapshotV2::from_canonical_bytes(&rpg.canonical_bytes, limits)?;
        let physics_checkpoint =
            PhysicsWorldCheckpointV1::from_canonical_bytes(&physics.canonical_bytes, limits)?;
        let world_streaming_snapshot =
            WorldStreamingSnapshotV1::from_canonical_bytes(&streaming.canonical_bytes, limits)?;
        let world_routine_snapshot_or_none = routine_or_none
            .map(|routine| {
                WorldRoutineSnapshotV1::from_canonical_bytes(&routine.canonical_bytes, limits)
            })
            .transpose()?;
        let world_population_snapshot =
            WorldPopulationSnapshotV1::from_canonical_bytes(&population.canonical_bytes, limits)?;
        let world_activity_snapshot =
            WorldActivitySnapshotV1::from_canonical_bytes(&activity.canonical_bytes, limits)
                .map_err(|_| ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let agent_cognition_snapshot =
            AgentCognitionSnapshotV1::from_canonical_bytes(&agent.canonical_bytes, limits)
                .map_err(|_| ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let agent_memory_snapshot =
            AgentMemorySnapshotV1::from_canonical_bytes(&memory.canonical_bytes, limits)
                .map_err(|_| ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        let checkpoint =
            WorldCheckpointV4::new(runtime_snapshot, rpg_snapshot, physics_checkpoint)?;
        let actual_initial_root =
            crate::snapshot::world_checkpoint_with_systemic_cognition_v1_state_root(
                &checkpoint.runtime_snapshot,
                &checkpoint.rpg_snapshot,
                &checkpoint.physics_checkpoint,
                &world_streaming_snapshot,
                world_routine_snapshot_or_none.as_ref(),
                &world_population_snapshot,
                &world_activity_snapshot,
                &agent_cognition_snapshot,
                &agent_memory_snapshot,
            )?;
        if actual_initial_root != self.initial_state_root {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        if self
            .authority
            .windows(2)
            .any(|pair| pair[0].principal >= pair[1].principal)
        {
            return Err(ManifestValidationError::AuthorityNotStrictlySorted);
        }
        for grant in &self.authority {
            grant.validate()?;
        }
        if self.ticks.len() != self.compare_points.len() {
            return Err(ManifestValidationError::ComparePointCountMismatch);
        }

        let mut expected_tick = checkpoint.runtime_snapshot.next_tick;
        let mut decoded = Vec::with_capacity(self.ticks.len());
        for (tick, point) in self.ticks.iter().zip(&self.compare_points) {
            if tick.tick != expected_tick || point.tick != expected_tick {
                return Err(ManifestValidationError::ReplayTickSequenceMismatch);
            }
            tick.closed_ingress_batch
                .validate(&checkpoint.runtime_snapshot.admission_limits)?;
            tick.expected_ingress_command_batch
                .validate(&checkpoint.runtime_snapshot.admission_limits)?;
            tick.expected_physics_step_input.validate()?;
            tick.expected_contact_batch.validate()?;
            tick.expected_outcome_command_batch
                .validate(&checkpoint.runtime_snapshot.admission_limits)?;
            validate_replay_query_facts(tick)?;
            tick.world_streaming_input.validate()?;
            if point.owner_segments.len()
                != 8 + usize::from(world_routine_snapshot_or_none.is_some())
                || point.owner_segments.windows(2).any(|pair| {
                    (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
                        >= (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
                })
            {
                return Err(ManifestValidationError::ReplayOwnerSegmentsInvalid);
            }
            if tick.closed_ingress_batch.body.assigned_tick != expected_tick
                || tick.expected_ingress_command_batch.body.simulation_tick != expected_tick
                || tick.expected_ingress_command_batch.body.phase
                    != crate::command::CommandPhase::Ingress
                || tick.expected_physics_step_input.gameplay_tick != expected_tick
                || tick.expected_contact_batch.gameplay_tick != expected_tick
                || tick.expected_outcome_command_batch.body.simulation_tick != expected_tick
                || tick.expected_outcome_command_batch.body.phase
                    != crate::command::CommandPhase::Outcome
                || point.closed_ingress_batch_hash != tick.closed_ingress_batch.batch_hash
                || point.ingress_command_batch_hash
                    != tick.expected_ingress_command_batch.batch_hash
                || point.physics_step_input_hash != tick.expected_physics_step_input.input_hash()?
                || point.contact_batch_hash != tick.expected_contact_batch.batch_hash
                || point.outcome_command_batch_hash
                    != tick.expected_outcome_command_batch.batch_hash
                || point.interaction_availability_hash
                    != crate::world_routine::interaction_availability_batch_hash(
                        &tick.expected_interaction_availability,
                    )?
            {
                return Err(ManifestValidationError::ReplayBatchMismatch);
            }
            if point.physics_query_batch_hash
                != replay_physics_query_batch_hash(&tick.expected_physics_query_batch)?
                || point.physics_query_results_hash
                    != replay_physics_query_results_hash(&tick.expected_physics_query_results)?
                || point.targeting_query_trace_hash
                    != replay_targeting_query_trace_hash(
                        &tick.expected_targeting_intents,
                        &tick.expected_authoritative_targeting_queries,
                    )?
            {
                return Err(ManifestValidationError::ReplayQueryFactsInvalid);
            }
            let mut commands = Vec::with_capacity(tick.direct_external_commands.len());
            for record in &tick.direct_external_commands {
                commands.push(record.decode_command(limits)?);
            }
            decoded.push(DecodedReplayTickV9 {
                tick: tick.tick,
                world_streaming_input: tick.world_streaming_input.clone(),
                closed_ingress_batch: tick.closed_ingress_batch.clone(),
                direct_external_commands: commands,
                expected_ingress_command_batch: tick.expected_ingress_command_batch.clone(),
                expected_physics_step_input: tick.expected_physics_step_input.clone(),
                expected_contact_batch: tick.expected_contact_batch.clone(),
                expected_targeting_intents: tick.expected_targeting_intents.clone(),
                expected_authoritative_targeting_queries: tick
                    .expected_authoritative_targeting_queries
                    .clone(),
                expected_physics_query_batch: tick.expected_physics_query_batch.clone(),
                expected_physics_query_results: tick.expected_physics_query_results.clone(),
                expected_outcome_command_batch: tick.expected_outcome_command_batch.clone(),
                expected_mapping_receipts: tick.expected_mapping_receipts.clone(),
                expected_interaction_availability: tick.expected_interaction_availability.clone(),
                expected_command_results: tick.expected_command_results.clone(),
                expected_events: tick.expected_events.clone(),
            });
            expected_tick = expected_tick
                .checked_add(1)
                .ok_or(ManifestValidationError::ReplayTickExhausted)?;
        }
        Ok((
            DecodedReplayInitialStateV9 {
                checkpoint,
                world_streaming_snapshot,
                world_routine_snapshot_or_none,
                world_population_snapshot,
                world_activity_snapshot,
                agent_cognition_snapshot,
                agent_memory_snapshot,
            },
            decoded,
        ))
    }
}

impl ReplayManifestV10 {
    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ManifestCodecError> {
        crate::manifest_jcs::encode_replay_manifest_v10(self)
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ManifestCodecError> {
        crate::manifest_jcs::decode_replay_manifest_v10(bytes, limits)
    }

    pub fn validate_and_decode(
        &self,
        limits: CanonicalDecodeLimits,
    ) -> Result<(DecodedReplayInitialStateV10, Vec<DecodedReplayTickV10>), ManifestValidationError>
    {
        if self.schema_version != REPLAY_MANIFEST_V10_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedReplayVersion(
                self.schema_version,
            ));
        }
        if !matches!(self.initial_owner_segments.len(), 9 | 10)
            || self.initial_owner_segments.windows(2).any(|pair| {
                (
                    &pair[0].descriptor.owner_id,
                    &pair[0].descriptor.schema_id,
                    &pair[0].descriptor.segment_id,
                ) >= (
                    &pair[1].descriptor.owner_id,
                    &pair[1].descriptor.schema_id,
                    &pair[1].descriptor.segment_id,
                )
            })
            || self
                .initial_owner_segments
                .iter()
                .any(|segment| !segment.descriptor.matches_bytes(&segment.canonical_bytes))
        {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        let physical_segments = self
            .initial_owner_segments
            .iter()
            .filter(|segment| physical_animation_owner(&segment.descriptor))
            .collect::<Vec<_>>();
        let [physical_segment] = physical_segments.as_slice() else {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        };
        if !physical_animation_descriptor(&physical_segment.descriptor) {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        for point in &self.compare_points {
            if !matches!(point.owner_segments.len(), 9 | 10)
                || point.owner_segments.windows(2).any(|pair| {
                    (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
                        >= (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
                })
                || point
                    .owner_segments
                    .iter()
                    .filter(|descriptor| physical_animation_owner(descriptor))
                    .count()
                    != 1
                || point
                    .owner_segments
                    .iter()
                    .find(|descriptor| physical_animation_owner(descriptor))
                    .is_none_or(|descriptor| !physical_animation_descriptor(descriptor))
            {
                return Err(ManifestValidationError::ReplayOwnerSegmentsInvalid);
            }
        }

        let mut legacy_initial_segments = self.initial_owner_segments.clone();
        legacy_initial_segments.retain(|segment| !physical_animation_owner(&segment.descriptor));
        let legacy_initial_descriptors = legacy_initial_segments
            .iter()
            .map(|segment| segment.descriptor.clone())
            .collect::<Vec<_>>();
        let legacy_initial_root =
            crate::snapshot::state_root_from_save_segment_descriptors(&legacy_initial_descriptors)?;
        let legacy_compare_points = self
            .compare_points
            .iter()
            .cloned()
            .map(|mut point| {
                point
                    .owner_segments
                    .retain(|descriptor| !physical_animation_owner(descriptor));
                point
            })
            .collect();
        let legacy = ReplayManifestV9 {
            schema_version: REPLAY_MANIFEST_V9_SCHEMA_VERSION,
            compatibility: self.compatibility.clone(),
            initial_owner_segments: legacy_initial_segments,
            initial_state_root: legacy_initial_root,
            authority: self.authority.clone(),
            ticks: self.ticks.clone(),
            compare_points: legacy_compare_points,
        };
        let (legacy_initial, decoded_ticks) = legacy.validate_and_decode(limits)?;
        let physical_animation_snapshot = PhysicalAnimationSnapshotV1::from_canonical_bytes(
            &physical_segment.canonical_bytes,
            limits,
        )
        .map_err(|_| ManifestValidationError::ReplayInitialSegmentsInvalid)?;
        if physical_animation_snapshot.next_simulation_tick
            != legacy_initial.checkpoint.runtime_snapshot.next_tick
            || physical_animation_snapshot.records.iter().any(|record| {
                !legacy_initial
                    .checkpoint
                    .physics_checkpoint
                    .snapshot
                    .sorted_body_states
                    .contains_key(&record.body_id)
            })
        {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        let initial_descriptors = self
            .initial_owner_segments
            .iter()
            .map(|segment| segment.descriptor.clone())
            .collect::<Vec<_>>();
        if crate::snapshot::state_root_from_save_segment_descriptors(&initial_descriptors)?
            != self.initial_state_root
        {
            return Err(ManifestValidationError::ReplayInitialSegmentsInvalid);
        }
        Ok((
            DecodedReplayInitialStateV10 {
                checkpoint: legacy_initial.checkpoint,
                world_streaming_snapshot: legacy_initial.world_streaming_snapshot,
                world_routine_snapshot_or_none: legacy_initial.world_routine_snapshot_or_none,
                world_population_snapshot: legacy_initial.world_population_snapshot,
                world_activity_snapshot: legacy_initial.world_activity_snapshot,
                agent_cognition_snapshot: legacy_initial.agent_cognition_snapshot,
                agent_memory_snapshot: legacy_initial.agent_memory_snapshot,
                physical_animation_snapshot,
            },
            decoded_ticks,
        ))
    }
}

fn physical_animation_owner(descriptor: &SaveSegmentDescriptor) -> bool {
    descriptor.owner_id.as_str() == PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID
}

fn physical_animation_descriptor(descriptor: &SaveSegmentDescriptor) -> bool {
    physical_animation_owner(descriptor)
        && descriptor.schema_id.as_str() == PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID
        && descriptor.segment_id.as_str() == PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID
        && descriptor.schema_version == u32::from(PHYSICAL_ANIMATION_SCHEMA_VERSION)
}

impl WorldStreamingReplayInputV1 {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        match self {
            Self::None => Ok(()),
            Self::BeginTransition {
                expected_base_world_state_hash,
                expected_next_world_state_hash,
                ..
            } if *expected_base_world_state_hash != ContentHash::default()
                && *expected_next_world_state_hash != ContentHash::default()
                && expected_base_world_state_hash != expected_next_world_state_hash =>
            {
                Ok(())
            }
            Self::CompletePendingTransition {
                expected_base_world_state_hash,
                expected_loaded_result_hash,
                expected_next_world_state_hash,
                ..
            } if *expected_base_world_state_hash != ContentHash::default()
                && *expected_loaded_result_hash != ContentHash::default()
                && *expected_next_world_state_hash != ContentHash::default()
                && expected_base_world_state_hash != expected_next_world_state_hash =>
            {
                Ok(())
            }
            _ => Err(ManifestValidationError::ReplayWorldStreamingInputInvalid),
        }
    }
}

fn validate_replay_query_facts(tick: &ReplayTickManifestV9) -> Result<(), ManifestValidationError> {
    tick.expected_physics_query_batch.validate()?;
    for receipt in &tick.expected_mapping_receipts {
        receipt.validate()?;
        if receipt.assigned_tick != tick.tick {
            return Err(ManifestValidationError::ReplayBatchMismatch);
        }
    }
    let request_count = tick.expected_physics_query_batch.requests.len();
    if tick.expected_targeting_intents.len() != request_count
        || tick.expected_authoritative_targeting_queries.len() != request_count
        || tick.expected_physics_query_results.len() != request_count
    {
        return Err(ManifestValidationError::ReplayQueryFactsInvalid);
    }
    for (((intent, targeting_query), request), result) in tick
        .expected_targeting_intents
        .iter()
        .zip(&tick.expected_authoritative_targeting_queries)
        .zip(&tick.expected_physics_query_batch.requests)
        .zip(&tick.expected_physics_query_results)
    {
        intent.validate()?;
        if intent.assignment.assigned_tick != tick.tick
            || targeting_query.schema_version != AUTHORITATIVE_TARGETING_QUERY_SCHEMA_VERSION
            || targeting_query.intent_hash != intent.intent_hash()?
            || targeting_query.assignment != intent.assignment
            || targeting_query.actor_id != intent.player_id
            || targeting_query.actor_id != targeting_query.actor_body_id.subject_id
            || targeting_query.targeting_profile_hash != intent.targeting_profile_hash
            || targeting_query.physics_query != *request
            || intent.query_kind != request.geometry.kind()
        {
            return Err(ManifestValidationError::ReplayQueryFactsInvalid);
        }
        result.validate_against_request(request)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ManifestValidationError {
    Identifier(crate::ids::IdentifierError),
    Canonicalization(CanonicalError),
    Snapshot(crate::snapshot::SnapshotDecodeError),
    Command(CommandDecodeError),
    InvalidTickSettings,
    HashBindingsNotStrictlySorted,
    SchemaBindingsNotStrictlySorted,
    SegmentsNotStrictlySorted,
    CapabilitiesNotStrictlySorted,
    AuthorityNotStrictlySorted,
    MissingRequiredSegment,
    CompatibilityMismatch,
    UnsupportedSaveVersion(u32),
    UnsupportedReplayVersion(u32),
    ReplayCommandIdMismatch,
    ComparePointCountMismatch,
    ReplayTickSequenceMismatch,
    ReplayTickExhausted,
    ReplayInitialSegmentsInvalid,
    ReplayOwnerSegmentsInvalid,
    ReplayWorldStreamingInputInvalid,
    ReplayBatchMismatch,
    ReplayQueryFactsInvalid,
    WorldCheckpoint(crate::snapshot::WorldCheckpointError),
    RpgV2(crate::rpg::RpgContractErrorV1),
    Physics(crate::physics::PhysicsContractError),
    Input(crate::input::InputContractError),
    Targeting(TargetingContractError),
    WorldStreaming(crate::world::WorldStreamingContractError),
    WorldRoutine(crate::world_routine::WorldRoutineContractError),
    WorldPopulation(crate::world_population::WorldPopulationContractError),
}

fn validate_sorted_unique_hash_bindings(
    bindings: &[HashBinding],
) -> Result<(), ManifestValidationError> {
    if bindings
        .windows(2)
        .any(|pair| pair[0].binding_id >= pair[1].binding_id)
    {
        return Err(ManifestValidationError::HashBindingsNotStrictlySorted);
    }
    Ok(())
}

fn validate_sorted_unique_schema_bindings(
    bindings: &[SchemaBinding],
) -> Result<(), ManifestValidationError> {
    if bindings.windows(2).any(|pair| {
        (&pair[0].schema_id, pair[0].schema_version) >= (&pair[1].schema_id, pair[1].schema_version)
    }) {
        return Err(ManifestValidationError::SchemaBindingsNotStrictlySorted);
    }
    Ok(())
}

pub use crate::manifest_jcs::ManifestCodecError;

#[cfg(test)]
mod tests;
