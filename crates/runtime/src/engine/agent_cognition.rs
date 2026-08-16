use next_agent::cognition::{
    StrategicAgentOwnersV1, StrategicEvaluationV1, StrategicObservationV1,
    SystemicStrategicObservationV1, evaluate_strategic_decision_v1,
    evaluate_systemic_strategic_decision_v1,
};
use next_contracts::cognition::{
    AGENT_COGNITION_SYSTEM_ID, AgentCognitionCommandV1, AgentCognitionSnapshotV1,
    AgentDecisionCommittedV1, AgentMemorySnapshotV1, COGNITION_SCHEMA_VERSION, DecisionTraceV1,
    SystemicExecutionFailureV1,
};
use next_contracts::command::{
    CommandPayload, CommandPhase, DomainEvent, IssuerPrincipal, WorldCommand,
};
use next_contracts::ids::{CommandStreamId, SchemaId, SystemId};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1};
use next_contracts::world_population::{
    NavigationCapabilityV1, NavigationQueryV1, PopulationTierV1, WORLD_POPULATION_SCHEMA_VERSION,
};
use next_rpg::RpgState;

use super::error::RuntimeFatalError;
use super::state::RuntimeState;

pub(super) struct AgentCognitionStageContextV1 {
    owners: StrategicAgentOwnersV1,
    population: next_world::WorldPopulationOwnerV1,
    activity_or_none: Option<next_world::WorldActivityOwnerV1>,
    staged_agent_snapshot: AgentCognitionSnapshotV1,
    staged_memory_snapshot: AgentMemorySnapshotV1,
    route: (SystemId, CommandStreamId),
    systemic_rpg_route_or_none: Option<(SystemId, CommandStreamId)>,
    proposal_or_none: Option<WorldCommand>,
    systemic_rpg_proposal_or_none: Option<WorldCommand>,
    systemic_failure_or_none: Option<SystemicExecutionFailureV1>,
    decision_trace_or_none: Option<DecisionTraceV1>,
    proposal_applied: bool,
}

impl AgentCognitionStageContextV1 {
    pub(super) fn capture(
        runtime: &RuntimeState,
        owners: &StrategicAgentOwnersV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity_or_none: Option<&next_world::WorldActivityOwnerV1>,
    ) -> Result<Self, RuntimeFatalError> {
        owners
            .validate()
            .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        population
            .validate(runtime.next_tick)
            .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let subject_id = owners.catalog().subject_id;
        if population
            .population_catalog_or_none()
            .and_then(|catalog| catalog.definition(subject_id))
            .is_none()
            || population
                .snapshot_or_none()
                .and_then(|snapshot| {
                    snapshot
                        .records
                        .binary_search_by_key(&subject_id, |record| record.subject_id)
                        .ok()
                })
                .is_none()
        {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        if let Some(activity) = activity_or_none {
            activity
                .validate(runtime.next_tick)
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
            if activity.catalog().worker_subject_id != subject_id {
                return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
            }
        }
        let system_id = SystemId::new(AGENT_COGNITION_SYSTEM_ID)
            .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
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
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        if matching_streams.next().is_some() {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        let systemic_rpg_route_or_none = if activity_or_none.is_some() {
            let mut matching_streams = runtime
                .stream_registry
                .entries
                .iter()
                .filter(|(key, _)| {
                    key.principal == principal && key.stream_slot == 1 && key.stream_epoch == 0
                })
                .map(|(_, stream_id)| *stream_id);
            let stream_id = matching_streams
                .next()
                .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
            if matching_streams.next().is_some() {
                return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
            }
            Some((system_id.clone(), stream_id))
        } else {
            None
        };
        Ok(Self {
            owners: owners.clone(),
            population: population.clone(),
            activity_or_none: activity_or_none.cloned(),
            staged_agent_snapshot: owners.agent_snapshot().clone(),
            staged_memory_snapshot: owners.memory_snapshot().clone(),
            route: (system_id, stream_id),
            systemic_rpg_route_or_none,
            proposal_or_none: None,
            systemic_rpg_proposal_or_none: None,
            systemic_failure_or_none: None,
            decision_trace_or_none: None,
            proposal_applied: false,
        })
    }

    pub(super) fn produce_stage_7(
        &mut self,
        simulation_tick: u64,
        authoritative_revision: u64,
        rpg: &RpgState,
    ) -> Result<(), RuntimeFatalError> {
        if self.proposal_or_none.is_some()
            || self.systemic_rpg_proposal_or_none.is_some()
            || self.systemic_failure_or_none.is_some()
            || self.decision_trace_or_none.is_some()
            || self.proposal_applied
        {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        if let (Some(activity), Some((system_id, stream_id))) =
            (&self.activity_or_none, &self.systemic_rpg_route_or_none)
        {
            let decision = super::systemic_agent::build_systemic_rpg_decision_v1(
                self.staged_agent_snapshot.pending_intent_or_none.as_ref(),
                activity.catalog(),
                activity.snapshot(),
                rpg,
                simulation_tick,
            );
            self.systemic_failure_or_none = decision.failure_or_none;
            if let Some((sequence, payload)) = decision.command_or_none {
                let mut command = WorldCommand::rpg(
                    *stream_id,
                    IssuerPrincipal::InternalSystem(system_id.clone()),
                    sequence,
                    simulation_tick,
                    payload,
                )?;
                command.phase = CommandPhase::Outcome;
                command.set_precondition_revision(Some(authoritative_revision))?;
                self.systemic_rpg_proposal_or_none = Some(command);
            }
        }
        if !self.owners.catalog().is_due(simulation_tick) {
            return Ok(());
        }
        let Some(evaluation) = self.evaluate(simulation_tick, rpg)? else {
            return Ok(());
        };
        self.decision_trace_or_none = Some(evaluation.decision_trace.clone());
        let Some(proposal) = evaluation.proposal_or_none else {
            return Ok(());
        };
        let payload = AgentCognitionCommandV1::CommitDecision {
            expected_agent_revision: self.staged_agent_snapshot.revision,
            expected_memory_revision: self.staged_memory_snapshot.revision,
            cognition_catalog_revision: self
                .owners
                .catalog()
                .revision()
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?,
            epistemic_view_hash: evaluation.epistemic_view.view_hash,
            next_agent_snapshot: proposal.next_agent_snapshot,
            next_memory_snapshot: proposal.next_memory_snapshot,
        };
        self.proposal_or_none = Some(WorldCommand::agent_cognition(
            self.route.1,
            self.route.0.clone(),
            self.staged_agent_snapshot.revision,
            simulation_tick,
            authoritative_revision,
            payload,
        )?);
        Ok(())
    }

    pub(super) fn proposal_for_stage_9(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        if self.proposal_or_none.as_ref().is_some_and(|proposal| {
            proposal.target_tick != simulation_tick
                || proposal.precondition_revision() != Some(authoritative_revision)
        }) {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        Ok(self.proposal_or_none.clone())
    }

    pub(super) fn systemic_rpg_proposal_for_stage_9(
        &self,
        simulation_tick: u64,
        authoritative_revision: u64,
    ) -> Result<Option<WorldCommand>, RuntimeFatalError> {
        if self
            .systemic_rpg_proposal_or_none
            .as_ref()
            .is_some_and(|proposal| {
                proposal.target_tick != simulation_tick
                    || proposal.precondition_revision() != Some(authoritative_revision)
            })
        {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        Ok(self.systemic_rpg_proposal_or_none.clone())
    }

    pub(super) const fn systemic_failure_or_none(&self) -> Option<SystemicExecutionFailureV1> {
        self.systemic_failure_or_none
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
            || command.target_tick != simulation_tick
            || command.precondition_revision() != Some(authoritative_revision)
        {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        let payload = match &command.payload {
            CommandPayload::AgentCognition(payload) => payload.clone(),
            _ => return Err(RuntimeFatalError::AgentCognitionInternalInvariant),
        };
        payload
            .validate()
            .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let AgentCognitionCommandV1::CommitDecision {
            expected_agent_revision,
            expected_memory_revision,
            next_agent_snapshot,
            next_memory_snapshot,
            ..
        } = &payload;
        if *expected_agent_revision != self.staged_agent_snapshot.revision
            || *expected_memory_revision != self.staged_memory_snapshot.revision
        {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        let active_goal = next_agent_snapshot
            .active_goal_or_none
            .as_ref()
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let intent = next_agent_snapshot
            .pending_intent_or_none
            .as_ref()
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let event = DomainEvent::agent_cognition(
            simulation_tick,
            next_contracts::command::CommandPhase::Outcome,
            command_id,
            0,
            AgentDecisionCommittedV1 {
                schema_version: COGNITION_SCHEMA_VERSION,
                subject_id: next_agent_snapshot.subject_id,
                agent_revision: next_agent_snapshot.revision,
                memory_revision: next_memory_snapshot.revision,
                active_goal_id: active_goal.goal_id.clone(),
                intent_id: intent.intent_id,
            },
        )?;
        let delta = payload.canonical_payload_bytes()?;
        self.staged_agent_snapshot = next_agent_snapshot.clone();
        self.staged_memory_snapshot = next_memory_snapshot.clone();
        self.proposal_applied = true;
        Ok((event, delta))
    }

    pub(super) fn finish(
        &self,
    ) -> Result<
        (
            AgentCognitionSnapshotV1,
            AgentMemorySnapshotV1,
            Option<DecisionTraceV1>,
        ),
        RuntimeFatalError,
    > {
        if self.proposal_or_none.is_some() != self.proposal_applied {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        let prepared = self
            .owners
            .prepare_publication(
                self.staged_agent_snapshot.clone(),
                self.staged_memory_snapshot.clone(),
            )
            .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        if self.proposal_or_none.is_some() {
            self.owners
                .validate_prepared_publication(prepared)
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        } else if self.staged_agent_snapshot != *self.owners.agent_snapshot()
            || self.staged_memory_snapshot != *self.owners.memory_snapshot()
        {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        }
        Ok((
            self.staged_agent_snapshot.clone(),
            self.staged_memory_snapshot.clone(),
            self.decision_trace_or_none.clone(),
        ))
    }

    fn evaluate(
        &self,
        simulation_tick: u64,
        rpg: &RpgState,
    ) -> Result<Option<StrategicEvaluationV1>, RuntimeFatalError> {
        let catalog = self.owners.catalog();
        let population_catalog = self
            .population
            .population_catalog_or_none()
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let navigation_catalog = self
            .population
            .navigation_catalog_or_none()
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let population_snapshot = self
            .population
            .snapshot_or_none()
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let definition = population_catalog
            .definition(catalog.subject_id)
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let record = population_snapshot
            .records
            .binary_search_by_key(&catalog.subject_id, |record| record.subject_id)
            .ok()
            .and_then(|index| population_snapshot.records.get(index))
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let systemic_or_none = self
            .activity_or_none
            .as_ref()
            .map(|activity| {
                let aggregate = rpg
                    .aggregate(
                        RpgAggregateKindV1::Commitment,
                        activity.catalog().commitment_id,
                    )
                    .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
                let commitment = rpg
                    .commitment(activity.catalog().commitment_id)
                    .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
                Ok::<_, RuntimeFatalError>(SystemicStrategicObservationV1 {
                    activity_catalog: activity.catalog(),
                    activity_snapshot: activity.snapshot(),
                    commitment_revision: aggregate.revision,
                    commitment_state: commitment.state,
                })
            })
            .transpose()?;
        if let Some(systemic) = systemic_or_none {
            let explicit_wake = systemic
                .activity_catalog
                .systemic_work
                .work_exchange
                .acts
                .first()
                .is_some_and(|act| act.creation_tick == simulation_tick);
            if !explicit_wake
                && !matches!(
                    record.tier,
                    PopulationTierV1::Simulated | PopulationTierV1::Active
                )
            {
                return Ok(None);
            }
        }
        let rpg_snapshot = rpg.snapshot();
        let aggregate = rpg_snapshot
            .aggregates
            .iter()
            .find(|aggregate| {
                aggregate.aggregate_kind == RpgAggregateKindV1::Character
                    && aggregate.persistent_id == catalog.subject_id
            })
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let RpgAggregatePayloadV1::Character(character) = &aggregate.payload else {
            return Err(RuntimeFatalError::AgentCognitionInternalInvariant);
        };
        let health_id = SchemaId::new(CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let health = character
            .resources
            .binary_search_by(|resource| resource.resource_id.cmp(&health_id))
            .ok()
            .and_then(|index| character.resources.get(index))
            .ok_or(RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let query = NavigationQueryV1 {
            schema_version: WORLD_POPULATION_SCHEMA_VERSION,
            catalog_asset_id: navigation_catalog.catalog_asset_id,
            graph_revision: navigation_catalog
                .revision()
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?,
            start_node_id: record.current_node_id.clone(),
            goal_node_id: definition.navigation_goal_node_id.clone(),
            capability: NavigationCapabilityV1::AbstractTransfer,
        };
        let route = self.population.route(&query).ok();
        let epistemic = next_agent::cognition::build_epistemic_view_v1(&StrategicObservationV1 {
            gameplay_tick: simulation_tick,
            cognition_catalog_revision: catalog
                .revision()
                .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?,
            catalog,
            memory_snapshot: &self.staged_memory_snapshot,
            rpg_owner_revision: aggregate.revision,
            population_record_revision: record.record_revision,
            current_node_id: record.current_node_id.clone(),
            navigation_goal_node_id: definition.navigation_goal_node_id.clone(),
            health_current: health.current_value,
            health_maximum: health.maximum_value,
            route_plan_or_none: route.as_ref(),
            systemic_or_none,
        })
        .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        let evaluation = match systemic_or_none {
            Some(systemic) => evaluate_systemic_strategic_decision_v1(
                catalog,
                &self.staged_agent_snapshot,
                &self.staged_memory_snapshot,
                epistemic,
                systemic,
            ),
            None => evaluate_strategic_decision_v1(
                catalog,
                &self.staged_agent_snapshot,
                &self.staged_memory_snapshot,
                epistemic,
            ),
        }
        .map_err(|_| RuntimeFatalError::AgentCognitionInternalInvariant)?;
        Ok(Some(evaluation))
    }
}
