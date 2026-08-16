use crate::canonical::CanonicalDecodeLimits;
use crate::cognition::{AgentCognitionSnapshotV1, AgentMemorySnapshotV1};
use crate::ids::StateRoot;
use crate::physical_animation::{
    PHYSICAL_ANIMATION_SCHEMA_VERSION, PHYSICAL_ANIMATION_SNAPSHOT_OWNER_ID,
    PHYSICAL_ANIMATION_SNAPSHOT_SCHEMA_ID, PHYSICAL_ANIMATION_SNAPSHOT_SEGMENT_ID,
    PhysicalAnimationSnapshotV1,
};
use crate::snapshot::WorldCheckpointV4;
use crate::world::WorldStreamingSnapshotV1;
use crate::world_activity::WorldActivitySnapshotV1;
use crate::world_population::WorldPopulationSnapshotV1;
use crate::world_routine::WorldRoutineSnapshotV1;

use super::{
    AuthorityGrant, DecodedReplayTickV9, ManifestCodecError, ManifestValidationError,
    REPLAY_MANIFEST_V9_SCHEMA_VERSION, REPLAY_MANIFEST_V10_SCHEMA_VERSION, ReplayComparePointV9,
    ReplayManifestV9, ReplayOwnerSegmentV2, ReplayTickManifestV9, SaveCompatibility,
    SaveSegmentDescriptor,
};

pub type ReplayTickManifestV10 = ReplayTickManifestV9;
pub type ReplayComparePointV10 = ReplayComparePointV9;
pub type DecodedReplayTickV10 = DecodedReplayTickV9;

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
