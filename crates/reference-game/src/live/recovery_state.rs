use next_contracts::snapshot::{WorldCheckpointCanonicalComponentsV1, WorldCheckpointV4};

use super::*;

impl ReferenceGameDriverV2 {
    pub fn state(&self) -> Result<ReferenceLiveStateV2, ReferenceGameError> {
        let presentation_input_count = u64::try_from(self.presentation_bindings.len())
            .map_err(|_| ReferenceGameError::CountOverflow)?
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let (checkpoint, checkpoint_canonical_components) =
            self.runtime.world_checkpoint_with_canonical_components()?;
        Ok(ReferenceLiveStateV2 {
            checkpoint,
            checkpoint_canonical_components,
            world_streaming_snapshot: self.world_streamer.snapshot().clone(),
            world_routine_snapshot_or_none: self.world_routine.snapshot_or_none().copied(),
            world_population_snapshot: self
                .world_population
                .snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            world_activity_snapshot: self.world_activity.snapshot().clone(),
            agent_cognition_snapshot: self.cognition.agent_snapshot().clone(),
            agent_memory_snapshot: self.cognition.memory_snapshot().clone(),
            physical_animation_snapshot: self.physical_animation.snapshot().clone(),
            ticks: self.runtime.next_tick(),
            events: self.events,
            rpg_events: self.rpg_events,
            project_composition_lock_hash: self
                .fixture
                .activated_project
                .project_lock
                .project_lock_sha256,
            content_manifest_hash: self
                .fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            presentation_input_count,
            presentation_snapshot: self
                .presentation_extractor
                .accepted_snapshot()
                .cloned()
                .ok_or(ReferenceGameError::PresentationSnapshotMissing)?,
            driver_recovery: ReferenceLiveDriverRecoveryV1 {
                next_logical_frame_sequence: self.next_logical_frame_sequence,
                events: self.events,
                rpg_events: self.rpg_events,
                camera_yaw_millidegrees: self.camera_yaw_millidegrees,
                camera_pitch_millidegrees: self.camera_pitch_millidegrees,
                camera_cut: self.camera_cut,
                ui_screen: self.ui_screen,
                dialogue: self.dialogue,
                input_session_bytes: self.input.recovery_bytes()?,
                presentation_snapshot_bytes: self.presentation_extractor.recovery_bytes()?,
            },
        })
    }

    pub fn prepared_state(
        &self,
        prepared: &PreparedReferenceGameAdvance,
    ) -> Result<ReferenceLiveStateV2, ReferenceGameError> {
        let checkpoint = prepared
            .runtime
            .world_checkpoint_with_canonical_components()?;
        self.state_from_prepared_parts(
            checkpoint,
            prepared.runtime.world_streaming_snapshot().clone(),
            prepared.runtime.routine_snapshot_or_none().copied(),
            prepared
                .runtime
                .population_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            prepared
                .runtime
                .activity_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            prepared
                .runtime
                .agent_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            prepared
                .runtime
                .memory_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            prepared.next_tick(),
            &prepared.state,
        )
    }

    pub fn validated_state(
        &self,
        validated: &ValidatedReferenceGameAdvance,
    ) -> Result<ReferenceLiveStateV2, ReferenceGameError> {
        let checkpoint = validated
            .runtime
            .world_checkpoint_with_canonical_components()?;
        self.state_from_prepared_parts(
            checkpoint,
            validated.runtime.world_streaming_snapshot().clone(),
            validated.runtime.routine_snapshot_or_none().copied(),
            validated
                .runtime
                .population_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            validated
                .runtime
                .activity_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            validated
                .runtime
                .agent_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            validated
                .runtime
                .memory_snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            validated.next_tick(),
            &validated.state,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "recovery materializes the independently hashed R4c checkpoint owner snapshots"
    )]
    fn state_from_prepared_parts(
        &self,
        (checkpoint, checkpoint_canonical_components): (
            WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        world_streaming_snapshot: WorldStreamingSnapshotV1,
        world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
        world_population_snapshot: WorldPopulationSnapshotV1,
        world_activity_snapshot: next_contracts::world_activity::WorldActivitySnapshotV1,
        agent_cognition_snapshot: next_contracts::cognition::AgentCognitionSnapshotV1,
        agent_memory_snapshot: next_contracts::cognition::AgentMemorySnapshotV1,
        ticks: u64,
        state: &PreparedReferenceGameState,
    ) -> Result<ReferenceLiveStateV2, ReferenceGameError> {
        let presentation_input_count = u64::try_from(self.presentation_bindings.len())
            .map_err(|_| ReferenceGameError::CountOverflow)?
            .checked_add(1)
            .ok_or(ReferenceGameError::CountOverflow)?;
        Ok(ReferenceLiveStateV2 {
            checkpoint,
            checkpoint_canonical_components,
            world_streaming_snapshot,
            world_routine_snapshot_or_none,
            world_population_snapshot,
            world_activity_snapshot,
            agent_cognition_snapshot,
            agent_memory_snapshot,
            physical_animation_snapshot: state.physical_animation.snapshot().clone(),
            ticks,
            events: state.events,
            rpg_events: state.rpg_events,
            project_composition_lock_hash: self
                .fixture
                .activated_project
                .project_lock
                .project_lock_sha256,
            content_manifest_hash: self
                .fixture
                .activated_project
                .content_manifest
                .content_manifest_sha256,
            presentation_input_count,
            presentation_snapshot: state
                .presentation_extractor
                .accepted_snapshot()
                .cloned()
                .ok_or(ReferenceGameError::PresentationSnapshotMissing)?,
            driver_recovery: ReferenceLiveDriverRecoveryV1 {
                next_logical_frame_sequence: state.next_logical_frame_sequence,
                events: state.events,
                rpg_events: state.rpg_events,
                camera_yaw_millidegrees: state.camera_yaw_millidegrees,
                camera_pitch_millidegrees: state.camera_pitch_millidegrees,
                camera_cut: state.camera_cut,
                ui_screen: state.ui_screen,
                dialogue: state.dialogue,
                input_session_bytes: state.input.recovery_bytes()?,
                presentation_snapshot_bytes: state.presentation_extractor.recovery_bytes()?,
            },
        })
    }
}
