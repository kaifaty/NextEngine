//! Bounded live time advancement over the ordinary production evaluator.

use std::sync::Arc;

use next_contracts::ids::{SchemaId, StateRoot};

use super::*;

/// Hard admission limit for one bulk-time request. The consumer may stop
/// earlier at the first observable boundary.
pub const REFERENCE_BULK_TIME_MAX_TICKS_V1: u64 = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ReferenceBulkTimeStopReasonV1 {
    ObservableBoundary = 1,
    TickBudgetExhausted = 2,
}

/// Exact evidence for one bounded run. Bulk time does not skip simulation:
/// every admitted tick passes through the same prepare/validate/commit path as
/// [`ReferenceGameDriverV2::advance`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceBulkTimeAdvanceV1 {
    pub start_tick: u64,
    pub end_tick: u64,
    pub advanced_ticks: u64,
    pub stop_reason: ReferenceBulkTimeStopReasonV1,
    pub committed_event_count: u64,
    pub rpg_event_count: u64,
    pub world_activity_revision: u64,
    pub agent_cognition_revision: u64,
    pub checkpoint_state_root: StateRoot,
    pub application_state_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReferenceBulkObservableCursorV1 {
    committed_event_count: u64,
    rpg_snapshot: next_contracts::rpg::RpgSnapshotV2,
    physics_observable: PhysicsCanonicalSnapshotV2,
    world_streaming_snapshot: WorldStreamingSnapshotV1,
    world_routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    world_population_snapshot: WorldPopulationSnapshotV1,
    world_activity_snapshot: WorldActivitySnapshotV1,
    agent_cognition_snapshot: AgentCognitionSnapshotV1,
    agent_memory_snapshot: AgentMemorySnapshotV1,
    physical_animation_snapshot: PhysicalAnimationSnapshotV1,
    camera_yaw_millidegrees: i32,
    camera_pitch_millidegrees: i32,
    ui_screen: ReferenceUiScreenV1,
    dialogue: ReferenceDialogueUiV1,
    audio_subtitle_or_none: Option<SchemaId>,
    audio_pcm: Arc<[i16]>,
}

impl ReferenceBulkObservableCursorV1 {
    fn capture(driver: &ReferenceGameDriverV2) -> Result<Self, ReferenceGameError> {
        let mut physics_observable = driver.runtime.physics_snapshot().clone();
        // Simulation clocks and bookkeeping revisions advance on an empty
        // physics step. They are not an observable boundary by themselves.
        physics_observable.world_revision = 0;
        physics_observable.checkpoint_revision = 0;
        physics_observable.physics_tick = 0;
        for body in physics_observable.sorted_body_states.values_mut() {
            body.body_revision = 0;
            body.sleep_counter = 0;
        }
        for contact in physics_observable
            .sorted_contact_continuity_states
            .values_mut()
        {
            contact.last_seen_physics_tick = 0;
        }
        Ok(Self {
            committed_event_count: driver.events,
            rpg_snapshot: driver.runtime.rpg_snapshot().clone(),
            physics_observable,
            world_streaming_snapshot: driver.world_streamer.snapshot().clone(),
            world_routine_snapshot_or_none: driver.world_routine.snapshot_or_none().copied(),
            world_population_snapshot: driver
                .world_population
                .snapshot_or_none()
                .cloned()
                .ok_or(ReferenceGameError::RecoveryInvalid)?,
            world_activity_snapshot: driver.world_activity.snapshot().clone(),
            agent_cognition_snapshot: driver.cognition.agent_snapshot().clone(),
            agent_memory_snapshot: driver.cognition.memory_snapshot().clone(),
            physical_animation_snapshot: driver.physical_animation.snapshot().clone(),
            camera_yaw_millidegrees: driver.camera_yaw_millidegrees,
            camera_pitch_millidegrees: driver.camera_pitch_millidegrees,
            ui_screen: driver.ui_screen,
            dialogue: driver.dialogue,
            audio_subtitle_or_none: driver.current_audio_subtitle(driver.next_tick()),
            audio_pcm: driver.audio_pcm.clone(),
        })
    }
}

impl ReferenceGameDriverV2 {
    /// Advances by at most `max_ticks`, stopping immediately after the first
    /// tick that changes an authoritative or presentation-observable value.
    /// Invalid budgets are rejected before any driver mutation.
    pub fn advance_bulk_time(
        &mut self,
        max_ticks: u64,
    ) -> Result<ReferenceBulkTimeAdvanceV1, ReferenceGameError> {
        if max_ticks == 0 || max_ticks > REFERENCE_BULK_TIME_MAX_TICKS_V1 {
            return Err(ReferenceGameError::BulkTimeTickBudgetInvalid {
                requested: max_ticks,
                maximum: REFERENCE_BULK_TIME_MAX_TICKS_V1,
            });
        }
        let start_tick = self.next_tick();
        start_tick
            .checked_add(max_ticks)
            .ok_or(ReferenceGameError::CountOverflow)?;
        let mut cursor = ReferenceBulkObservableCursorV1::capture(self)?;
        let mut advanced_ticks = 0_u64;
        let stop_reason = loop {
            self.advance(&[])?;
            advanced_ticks = advanced_ticks
                .checked_add(1)
                .ok_or(ReferenceGameError::CountOverflow)?;
            let next_cursor = ReferenceBulkObservableCursorV1::capture(self)?;
            if next_cursor != cursor {
                break ReferenceBulkTimeStopReasonV1::ObservableBoundary;
            }
            if advanced_ticks == max_ticks {
                break ReferenceBulkTimeStopReasonV1::TickBudgetExhausted;
            }
            cursor = next_cursor;
        };
        let end_tick = self.next_tick();
        if end_tick.checked_sub(start_tick) != Some(advanced_ticks) {
            return Err(ReferenceGameError::RecoveryInvalid);
        }
        let (checkpoint_state_root, application_state_root) = self.bulk_time_state_roots()?;
        Ok(ReferenceBulkTimeAdvanceV1 {
            start_tick,
            end_tick,
            advanced_ticks,
            stop_reason,
            committed_event_count: self.events,
            rpg_event_count: self.rpg_events,
            world_activity_revision: self.world_activity.snapshot().record_revision,
            agent_cognition_revision: self.cognition.agent_snapshot().revision,
            checkpoint_state_root,
            application_state_root,
        })
    }

    fn bulk_time_state_roots(&self) -> Result<(StateRoot, StateRoot), ReferenceGameError> {
        let (checkpoint, components) = self.runtime.world_checkpoint_with_canonical_components()?;
        let population = self
            .world_population
            .snapshot_or_none()
            .ok_or(ReferenceGameError::RecoveryInvalid)?;
        let application = next_contracts::snapshot::
            world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components(
                &components,
                self.world_streamer.snapshot(),
                self.world_routine.snapshot_or_none(),
                population,
                self.world_activity.snapshot(),
                self.cognition.agent_snapshot(),
                self.cognition.memory_snapshot(),
                self.physical_animation.snapshot(),
            )?;
        Ok((checkpoint.state_root, application))
    }
}
