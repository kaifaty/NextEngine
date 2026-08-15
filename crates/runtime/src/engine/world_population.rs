use next_contracts::command::{CommandPayload, DomainEvent, IssuerPrincipal, WorldCommand};
use next_contracts::ids::{CommandStreamId, SystemId};
use next_contracts::world_population::{
    WORLD_POPULATION_SYSTEM_ID, WorldPopulationCommandV1, WorldPopulationSnapshotV1,
};
use next_world::{PopulationNavigationServiceReportV1, WorldPopulationOwnerV1};

use super::error::RuntimeFatalError;
use super::state::RuntimeState;

/// Private population/navigation projection used only while preparing the
/// joint Runtime + World Services transaction.
pub(super) struct WorldPopulationStageContextV1 {
    owner: WorldPopulationOwnerV1,
    staged_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
    route_or_none: Option<(SystemId, CommandStreamId)>,
    proposal_or_none: Option<WorldCommand>,
    proposal_applied: bool,
    service_report_or_none: Option<PopulationNavigationServiceReportV1>,
}

impl WorldPopulationStageContextV1 {
    pub(super) fn capture(
        runtime: &RuntimeState,
        owner: &WorldPopulationOwnerV1,
    ) -> Result<Self, RuntimeFatalError> {
        owner
            .validate(runtime.next_tick)
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        let staged_snapshot_or_none = owner.snapshot_or_none().cloned();
        let system_id = SystemId::new(WORLD_POPULATION_SYSTEM_ID)
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
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
            || staged_snapshot_or_none.is_some() != route_or_none.is_some()
            || owner.population_catalog_or_none().is_some() != staged_snapshot_or_none.is_some()
            || owner.navigation_catalog_or_none().is_some() != staged_snapshot_or_none.is_some()
        {
            return Err(RuntimeFatalError::WorldPopulationInternalInvariant);
        }
        Ok(Self {
            owner: owner.clone(),
            staged_snapshot_or_none,
            route_or_none,
            proposal_or_none: None,
            proposal_applied: false,
            service_report_or_none: None,
        })
    }

    pub(super) fn produce_stage_6(
        &mut self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<(), RuntimeFatalError> {
        if self.proposal_or_none.is_some()
            || self.proposal_applied
            || self.service_report_or_none.is_some()
        {
            return Err(RuntimeFatalError::WorldPopulationInternalInvariant);
        }
        if let Some(snapshot) = self.staged_snapshot_or_none.as_ref() {
            self.service_report_or_none = Some(
                self.owner
                    .service_snapshot(snapshot, simulation_tick)
                    .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?,
            );
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
        if expected != self.proposal_or_none
            || self.staged_snapshot_or_none.is_some() != self.service_report_or_none.is_some()
        {
            return Err(RuntimeFatalError::WorldPopulationInternalInvariant);
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
            return Err(RuntimeFatalError::WorldPopulationInternalInvariant);
        }
        let payload = match &command.payload {
            CommandPayload::WorldPopulation(payload) => payload.clone(),
            _ => return Err(RuntimeFatalError::WorldPopulationInternalInvariant),
        };
        let snapshot = self
            .staged_snapshot_or_none
            .as_mut()
            .ok_or(RuntimeFatalError::WorldPopulationInternalInvariant)?;
        let event_payload = self
            .owner
            .apply_command_to_snapshot(snapshot, &payload, simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        let event = DomainEvent::world_population(
            simulation_tick,
            next_contracts::command::CommandPhase::Outcome,
            command_id,
            0,
            event_payload,
        )?;
        let delta = payload
            .owner_delta_bytes()
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        self.proposal_applied = true;
        Ok((event, delta))
    }

    pub(super) fn finish(
        &self,
        next_simulation_tick: u64,
    ) -> Result<
        (
            Option<WorldPopulationSnapshotV1>,
            Option<PopulationNavigationServiceReportV1>,
        ),
        RuntimeFatalError,
    > {
        if self.proposal_or_none.is_some() != self.proposal_applied
            || self.staged_snapshot_or_none.is_some() != self.service_report_or_none.is_some()
        {
            return Err(RuntimeFatalError::WorldPopulationInternalInvariant);
        }
        let mut staged_owner = self.owner.clone();
        let prepared = staged_owner
            .prepare_publication(
                self.staged_snapshot_or_none.clone(),
                self.service_report_or_none.clone(),
                next_simulation_tick,
            )
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        let validated = staged_owner
            .validate_prepared_publication(prepared)
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        staged_owner.commit_validated_publication(validated);
        staged_owner
            .validate(next_simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?;
        Ok((
            self.staged_snapshot_or_none.clone(),
            self.service_report_or_none.clone(),
        ))
    }

    fn expected_proposal(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        let (Some(snapshot), Some((system_id, stream_id))) = (
            self.staged_snapshot_or_none.as_ref(),
            self.route_or_none.as_ref(),
        ) else {
            if self.staged_snapshot_or_none.is_none() && self.route_or_none.is_none() {
                return Ok(None);
            }
            return Err(RuntimeFatalError::WorldPopulationInternalInvariant);
        };
        let Some(payload) = self
            .owner
            .expected_command_for_snapshot(snapshot, simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldPopulationInternalInvariant)?
        else {
            return Ok(None);
        };
        WorldCommand::world_population(
            *stream_id,
            system_id.clone(),
            population_sequence(&payload),
            simulation_tick,
            authoritative_revision,
            payload,
        )
        .map(Some)
        .map_err(RuntimeFatalError::from)
    }
}

const fn population_sequence(command: &WorldPopulationCommandV1) -> u64 {
    match command {
        WorldPopulationCommandV1::TransitionTier {
            expected_record_revision,
            ..
        }
        | WorldPopulationCommandV1::CommitAbstractTransfer {
            expected_record_revision,
            ..
        } => *expected_record_revision,
    }
}
