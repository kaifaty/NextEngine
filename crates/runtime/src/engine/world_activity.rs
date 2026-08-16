use next_contracts::command::{CommandPayload, DomainEvent, IssuerPrincipal, WorldCommand};
use next_contracts::ids::{CommandStreamId, SystemId};
use next_contracts::rpg::{CommitmentPayloadV1, RpgAggregateKindV1};
use next_contracts::world_activity::{
    WORLD_ACTIVITY_SYSTEM_ID, WorldActivityCommandV1, WorldActivitySnapshotV1,
};
use next_rpg::RpgState;
use next_world::{WorldActivityObservationV1, WorldActivityOwnerV1, WorldPopulationOwnerV1};

use super::error::RuntimeFatalError;
use super::state::RuntimeState;

/// Private activity projection used while preparing the joint Runtime + World
/// Services generation. Observations are read from owner projections; this
/// context never writes RPG or population state.
pub(super) struct WorldActivityStageContextV1 {
    owner: WorldActivityOwnerV1,
    population: WorldPopulationOwnerV1,
    staged_snapshot: WorldActivitySnapshotV1,
    route: (SystemId, CommandStreamId),
    proposal_or_none: Option<WorldCommand>,
    proposal_applied: bool,
}

impl WorldActivityStageContextV1 {
    pub(super) fn capture(
        runtime: &RuntimeState,
        owner: &WorldActivityOwnerV1,
        population: &WorldPopulationOwnerV1,
    ) -> Result<Self, RuntimeFatalError> {
        owner
            .validate(runtime.next_tick)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        population
            .validate(runtime.next_tick)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        validate_owner_observation_closure(owner, population, &runtime.rpg)?;

        let system_id = SystemId::new(WORLD_ACTIVITY_SYSTEM_ID)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        let principal = IssuerPrincipal::InternalSystem(system_id.clone());
        let mut matching_streams = runtime
            .stream_registry
            .entries
            .iter()
            .filter(|(key, _)| {
                key.principal == principal && key.stream_slot == 0 && key.stream_epoch == 0
            })
            .map(|(_, stream_id)| *stream_id);
        let stream_id = matching_streams
            .next()
            .ok_or(RuntimeFatalError::WorldActivityInternalInvariant)?;
        if matching_streams.next().is_some() {
            return Err(RuntimeFatalError::WorldActivityInternalInvariant);
        }
        Ok(Self {
            owner: owner.clone(),
            population: population.clone(),
            staged_snapshot: owner.snapshot().clone(),
            route: (system_id, stream_id),
            proposal_or_none: None,
            proposal_applied: false,
        })
    }

    pub(super) fn produce_stage_7(
        &mut self,
        simulation_tick: u64,
        authoritative_revision: u64,
        rpg: &RpgState,
    ) -> Result<(), RuntimeFatalError> {
        if self.proposal_or_none.is_some() || self.proposal_applied {
            return Err(RuntimeFatalError::WorldActivityInternalInvariant);
        }
        self.proposal_or_none =
            self.expected_proposal(simulation_tick, authoritative_revision, rpg)?;
        Ok(())
    }

    pub(super) fn proposal_for_stage_9(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
        rpg: &RpgState,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        let expected = self.expected_proposal(simulation_tick, authoritative_revision, rpg)?;
        if expected != self.proposal_or_none {
            return Err(RuntimeFatalError::WorldActivityInternalInvariant);
        }
        Ok(expected)
    }

    pub(super) fn apply_stage_9(
        &mut self,
        command: &WorldCommand,
        simulation_tick: u64,
        authoritative_revision: u64,
        command_id: next_contracts::ids::CommandId,
        rpg: &RpgState,
    ) -> Result<(DomainEvent, Vec<u8>), RuntimeFatalError> {
        if self.proposal_applied
            || self.proposal_or_none.as_ref() != Some(command)
            || self
                .expected_proposal(simulation_tick, authoritative_revision, rpg)?
                .as_ref()
                != Some(command)
        {
            return Err(RuntimeFatalError::WorldActivityInternalInvariant);
        }
        let payload = match &command.payload {
            CommandPayload::WorldActivity(payload) => payload.clone(),
            _ => return Err(RuntimeFatalError::WorldActivityInternalInvariant),
        };
        let observation = observation(&self.owner, &self.population, rpg)?;
        let event_payload = self
            .owner
            .apply_command_to_snapshot(
                &mut self.staged_snapshot,
                &payload,
                simulation_tick,
                observation,
            )
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        let event = DomainEvent::world_activity(
            simulation_tick,
            next_contracts::command::CommandPhase::Outcome,
            command_id,
            0,
            event_payload,
        )?;
        let delta = payload
            .owner_delta_bytes()
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        self.proposal_applied = true;
        Ok((event, delta))
    }

    pub(super) fn finish(
        &self,
        next_simulation_tick: u64,
    ) -> Result<WorldActivitySnapshotV1, RuntimeFatalError> {
        if self.proposal_or_none.is_some() != self.proposal_applied {
            return Err(RuntimeFatalError::WorldActivityInternalInvariant);
        }
        let mut staged_owner = self.owner.clone();
        let prepared = staged_owner
            .prepare_publication(self.staged_snapshot.clone(), next_simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        let validated = staged_owner
            .validate_prepared_publication(prepared)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        staged_owner.commit_validated_publication(validated);
        staged_owner
            .validate(next_simulation_tick)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?;
        Ok(self.staged_snapshot.clone())
    }

    fn expected_proposal(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
        rpg: &RpgState,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        let observation = observation(&self.owner, &self.population, rpg)?;
        let Some(payload) = self
            .owner
            .expected_command_for_snapshot(&self.staged_snapshot, simulation_tick, observation)
            .map_err(|_| RuntimeFatalError::WorldActivityInternalInvariant)?
        else {
            return Ok(None);
        };
        WorldCommand::world_activity(
            self.route.1,
            self.route.0.clone(),
            activity_sequence(&payload),
            simulation_tick,
            authoritative_revision,
            payload,
        )
        .map(Some)
        .map_err(RuntimeFatalError::from)
    }
}

fn validate_owner_observation_closure(
    owner: &WorldActivityOwnerV1,
    population: &WorldPopulationOwnerV1,
    rpg: &RpgState,
) -> Result<(), RuntimeFatalError> {
    let catalog = owner.catalog();
    let commitment = commitment(catalog.commitment_id, rpg)?;
    if commitment.recipient_character_id != catalog.worker_subject_id
        || commitment.work_id != catalog.work_id
        || commitment.workplace_node_id != catalog.workplace_node_id
        || population
            .snapshot_or_none()
            .and_then(|snapshot| snapshot.record(catalog.worker_subject_id))
            .is_none()
    {
        return Err(RuntimeFatalError::WorldActivityInternalInvariant);
    }
    Ok(())
}

fn observation<'a>(
    owner: &WorldActivityOwnerV1,
    population: &'a WorldPopulationOwnerV1,
    rpg: &RpgState,
) -> Result<WorldActivityObservationV1<'a>, RuntimeFatalError> {
    validate_owner_observation_closure(owner, population, rpg)?;
    let catalog = owner.catalog();
    let aggregate = rpg
        .aggregate(RpgAggregateKindV1::Commitment, catalog.commitment_id)
        .ok_or(RuntimeFatalError::WorldActivityInternalInvariant)?;
    let commitment = commitment(catalog.commitment_id, rpg)?;
    let population_snapshot = population
        .snapshot_or_none()
        .ok_or(RuntimeFatalError::WorldActivityInternalInvariant)?;
    let population_record = population_snapshot
        .record(catalog.worker_subject_id)
        .ok_or(RuntimeFatalError::WorldActivityInternalInvariant)?;
    Ok(WorldActivityObservationV1 {
        commitment_id: catalog.commitment_id,
        commitment_revision: aggregate.revision,
        commitment_state: commitment.state,
        population_record,
        population_snapshot,
    })
}

fn commitment(
    commitment_id: next_contracts::ids::PersistentId,
    rpg: &RpgState,
) -> Result<&CommitmentPayloadV1, RuntimeFatalError> {
    rpg.commitment(commitment_id)
        .ok_or(RuntimeFatalError::WorldActivityInternalInvariant)
}

const fn activity_sequence(command: &WorldActivityCommandV1) -> u64 {
    match command {
        WorldActivityCommandV1::Transition {
            expected_record_revision,
            ..
        } => *expected_record_revision,
    }
}
