use next_contracts::command::{DomainEvent, IssuerPrincipal, WorldCommand};
use std::collections::BTreeMap;

use next_contracts::ids::{CommandId, CommandStreamId, SystemId};
use next_contracts::mechanics::InteractionRoutineRevisionBindingV1;
use next_contracts::world_routine::{
    WORLD_ROUTINE_SYSTEM_ID, WorldRoutineActivityChangedV1, WorldRoutineActivityV1,
    WorldRoutineCatalogV1, WorldRoutineCommandV1, WorldRoutineSnapshotV1,
};

use super::error::RuntimeFatalError;
use super::state::RuntimeState;

/// Private staged World Services projection threaded only through the joint
/// Runtime/routine preparation entry point.
pub(super) struct WorldRoutineStageContextV1 {
    catalog_or_none: Option<WorldRoutineCatalogV1>,
    staged_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    route_or_none: Option<(SystemId, CommandStreamId)>,
    proposal_or_none: Option<WorldCommand>,
    proposal_applied: bool,
    interaction_bindings: BTreeMap<CommandId, InteractionRoutineRevisionBindingV1>,
}

impl WorldRoutineStageContextV1 {
    pub(super) fn capture(
        runtime: &RuntimeState,
        owner: &next_world::WorldRoutineOwnerV1,
    ) -> Result<Self, RuntimeFatalError> {
        owner
            .validate(runtime.next_tick)
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let catalog_or_none = owner.catalog_or_none().copied();
        let staged_snapshot_or_none = owner.snapshot_or_none().copied();
        let system_id = SystemId::new(WORLD_ROUTINE_SYSTEM_ID)
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let principal = IssuerPrincipal::InternalSystem(system_id.clone());
        let mut matching_streams = runtime
            .stream_registry
            .entries
            .iter()
            .filter(|(key, _)| {
                key.principal == principal && key.stream_slot == 0 && key.stream_epoch == 0
            })
            .map(|(_, stream_id)| *stream_id);
        let route_or_none = matching_streams
            .next()
            .map(|stream_id| (system_id, stream_id));
        if matching_streams.next().is_some()
            || catalog_or_none.is_some() != route_or_none.is_some()
            || catalog_or_none.is_some() != staged_snapshot_or_none.is_some()
        {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        Ok(Self {
            catalog_or_none,
            staged_snapshot_or_none,
            route_or_none,
            proposal_or_none: None,
            proposal_applied: false,
            interaction_bindings: BTreeMap::new(),
        })
    }

    pub(super) fn produce_stage_6(
        &mut self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<(), RuntimeFatalError> {
        if self.proposal_or_none.is_some() || self.proposal_applied {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        self.proposal_or_none = self.expected_proposal(simulation_tick, authoritative_revision)?;
        Ok(())
    }

    pub(super) fn proposal_for_stage_9(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        let expected = self.expected_proposal(simulation_tick, authoritative_revision)?;
        if expected != self.proposal_or_none {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        Ok(expected)
    }

    pub(super) fn apply_stage_9(
        &mut self,
        command: &WorldCommand,
        simulation_tick: u64,
        authoritative_revision: u64,
        command_id: next_contracts::ids::CommandId,
    ) -> Result<(DomainEvent, Vec<u8>), RuntimeFatalError> {
        if self.proposal_applied
            || self.proposal_or_none.as_ref() != Some(command)
            || self
                .expected_proposal(simulation_tick, authoritative_revision)?
                .as_ref()
                != Some(command)
        {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        let catalog = self
            .catalog_or_none
            .as_ref()
            .ok_or(RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let snapshot = self
            .staged_snapshot_or_none
            .as_mut()
            .ok_or(RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let payload = match &command.payload {
            next_contracts::command::CommandPayload::WorldRoutine(payload) => *payload,
            _ => return Err(RuntimeFatalError::WorldRoutineInternalInvariant),
        };
        let WorldRoutineCommandV1::CommitActivityBoundary {
            subject_id,
            expected_record_revision,
            catalog_asset_id,
            catalog_revision,
            boundary_world_tick,
            previous_activity,
            current_activity,
        } = payload;
        if subject_id != snapshot.record.subject_id
            || expected_record_revision != snapshot.record.record_revision
            || catalog_asset_id != snapshot.record.catalog_asset_id
            || catalog_revision != snapshot.record.catalog_revision
            || boundary_world_tick != catalog.routine.transition_world_tick
            || previous_activity != snapshot.record.current_activity
            || previous_activity != WorldRoutineActivityV1::Duty
            || current_activity != WorldRoutineActivityV1::Rest
        {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        let record_revision = expected_record_revision
            .checked_add(1)
            .ok_or(RuntimeFatalError::WorldRoutineInternalInvariant)?;
        snapshot.record.record_revision = record_revision;
        snapshot.record.current_activity = current_activity;
        snapshot
            .validate_against(
                catalog,
                simulation_tick
                    .checked_add(1)
                    .ok_or(RuntimeFatalError::TickExhausted)?,
            )
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let event_payload = WorldRoutineActivityChangedV1 {
            subject_id,
            previous_activity,
            current_activity,
            boundary_world_tick,
            record_revision,
        };
        let event = DomainEvent::world_routine(
            simulation_tick,
            next_contracts::command::CommandPhase::Outcome,
            command_id,
            0,
            event_payload,
        )?;
        let delta = payload
            .owner_delta_bytes()
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        self.proposal_applied = true;
        Ok((event, delta))
    }

    pub(super) fn register_interaction_binding(
        &mut self,
        command_id: CommandId,
        binding: InteractionRoutineRevisionBindingV1,
    ) -> Result<(), RuntimeFatalError> {
        let snapshot = self
            .staged_snapshot_or_none
            .as_ref()
            .filter(|snapshot| {
                snapshot.record.subject_id == binding.subject_id
                    && snapshot.record.record_revision == binding.routine_record_revision
            })
            .ok_or(RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let _ = snapshot;
        if self
            .interaction_bindings
            .insert(command_id, binding)
            .is_some()
        {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        Ok(())
    }

    pub(super) fn validate_interaction_command(
        &mut self,
        command_id: CommandId,
    ) -> Result<(), RuntimeFatalError> {
        let Some(binding) = self.interaction_bindings.remove(&command_id) else {
            return Ok(());
        };
        if self
            .staged_snapshot_or_none
            .as_ref()
            .is_none_or(|snapshot| {
                snapshot.record.subject_id != binding.subject_id
                    || snapshot.record.record_revision != binding.routine_record_revision
            })
        {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        Ok(())
    }

    pub(super) fn finish(
        &self,
        next_simulation_tick: u64,
    ) -> Result<Option<WorldRoutineSnapshotV1>, RuntimeFatalError> {
        if self.proposal_or_none.is_some() != self.proposal_applied
            || !self.interaction_bindings.is_empty()
        {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        match (&self.catalog_or_none, &self.staged_snapshot_or_none) {
            (Some(catalog), Some(snapshot)) => snapshot
                .validate_against(catalog, next_simulation_tick)
                .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?,
            (None, None) => {}
            _ => return Err(RuntimeFatalError::WorldRoutineInternalInvariant),
        }
        Ok(self.staged_snapshot_or_none)
    }

    #[must_use]
    pub(super) const fn committed_projection_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.staged_snapshot_or_none.as_ref()
    }

    fn expected_proposal(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        let (Some(catalog), Some(snapshot), Some((system_id, stream_id))) = (
            self.catalog_or_none.as_ref(),
            self.staged_snapshot_or_none.as_ref(),
            self.route_or_none.as_ref(),
        ) else {
            if self.catalog_or_none.is_none()
                && self.staged_snapshot_or_none.is_none()
                && self.route_or_none.is_none()
            {
                return Ok(None);
            }
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        };
        snapshot
            .validate_against(catalog, simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        if simulation_tick < catalog.profile.anchor_simulation_tick {
            return Err(RuntimeFatalError::WorldRoutineInternalInvariant);
        }
        if simulation_tick == catalog.profile.anchor_simulation_tick {
            return Ok(None);
        }
        let previous_tick = simulation_tick
            .checked_sub(1)
            .ok_or(RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let previous_world_tick = catalog
            .profile
            .world_tick(previous_tick)
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let current_world_tick = catalog
            .profile
            .world_tick(simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        let due = previous_world_tick < catalog.routine.transition_world_tick
            && current_world_tick >= catalog.routine.transition_world_tick;
        if !due {
            return Ok(None);
        }
        let payload = WorldRoutineCommandV1::commit_boundary(snapshot, catalog)
            .map_err(|_| RuntimeFatalError::WorldRoutineInternalInvariant)?;
        WorldCommand::world_routine(
            *stream_id,
            system_id.clone(),
            snapshot.record.record_revision,
            simulation_tick,
            authoritative_revision,
            payload,
        )
        .map(Some)
        .map_err(RuntimeFatalError::from)
    }
}
