use next_agent::cognition::{
    PreparedStrategicAgentPublicationV1, StrategicAgentOwnersV1,
    ValidatedStrategicAgentPublicationV1,
};
use next_contracts::cognition::{
    AGENT_MEMORY_SNAPSHOT_OWNER_ID, AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
    AGENT_MEMORY_SNAPSHOT_SEGMENT_ID, AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
    AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID, AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID, AgentCognitionSnapshotV1,
    AgentMemorySnapshotV1, COGNITION_SCHEMA_VERSION, DecisionTraceV1,
};
use next_contracts::ids::{InputSourceId, SchemaId, StateRoot};
use next_contracts::input::{ActionMapManifestV1, InputContextStackV1};
use next_contracts::persistence::SaveSegmentDescriptor;
use next_contracts::physics::{
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
};
use next_contracts::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, WorldCheckpointCanonicalComponentsV1,
};
use next_contracts::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION, WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
    WorldStreamingSnapshotV1,
};
use next_contracts::world_activity::{
    WORLD_ACTIVITY_SCHEMA_VERSION, WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
    WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID, WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID, WorldActivitySnapshotV1,
};
use next_contracts::world_population::{
    WORLD_POPULATION_SCHEMA_VERSION, WORLD_POPULATION_SNAPSHOT_OWNER_ID,
    WORLD_POPULATION_SNAPSHOT_SCHEMA_ID, WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
    WorldPopulationSnapshotV1,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_SCHEMA_VERSION, WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
    WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID, WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID, WorldRoutineSnapshotV1,
};

mod application_closure;

use super::*;
use application_closure::application_closure;

/// Opaque workspace-only candidate for the specialized Runtime + World
/// Services transaction. It is not a durable contracts schema.
pub struct PreparedRuntimeWorldServicesTickV1 {
    runtime: PreparedRuntimeTick,
    routine: next_world::PreparedWorldRoutinePublicationV1,
    population: next_world::PreparedWorldPopulationPublicationV1,
    activity: Option<next_world::PreparedWorldActivityPublicationV1>,
    cognition: Option<PreparedStrategicAgentPublicationV1>,
    activity_snapshot_or_none: Option<WorldActivitySnapshotV1>,
    agent_snapshot_or_none: Option<AgentCognitionSnapshotV1>,
    memory_snapshot_or_none: Option<AgentMemorySnapshotV1>,
    decision_trace_or_none: Option<DecisionTraceV1>,
    base_world_state_hash: next_contracts::ids::ContentHash,
    staged_world_snapshot: WorldStreamingSnapshotV1,
    streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
}

/// Opaque jointly validated generation retaining every base for final
/// preflight. Application evidence is an optional composition-root concern,
/// not part of the atomic publication precondition.
struct ValidatedWorldServicesGenerationV1 {
    runtime: ValidatedRuntimeTick,
    routine: next_world::ValidatedWorldRoutinePublicationV1,
    population: next_world::ValidatedWorldPopulationPublicationV1,
    activity: Option<next_world::ValidatedWorldActivityPublicationV1>,
    cognition: Option<ValidatedStrategicAgentPublicationV1>,
    activity_snapshot_or_none: Option<WorldActivitySnapshotV1>,
    agent_snapshot_or_none: Option<AgentCognitionSnapshotV1>,
    memory_snapshot_or_none: Option<AgentMemorySnapshotV1>,
    decision_trace_or_none: Option<DecisionTraceV1>,
    base_world_state_hash: next_contracts::ids::ContentHash,
    staged_world_snapshot: WorldStreamingSnapshotV1,
    streaming: Option<next_world::ValidatedWorldStreamingPublicationV1>,
}

/// Opaque jointly validated candidate carrying the full application closure.
pub struct ValidatedRuntimeWorldServicesTickV1 {
    generation: ValidatedWorldServicesGenerationV1,
    application_owner_segments: Vec<SaveSegmentDescriptor>,
    application_state_root: StateRoot,
}

/// Opaque jointly validated candidate for a live advance that does not need
/// to materialize the full application closure on every tick.
pub struct ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
    generation: ValidatedWorldServicesGenerationV1,
}

struct CommittedWorldServicesGenerationV1 {
    runtime_report: TickReport,
    world_streaming_snapshot: WorldStreamingSnapshotV1,
    routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    population_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
    activity_snapshot_or_none: Option<WorldActivitySnapshotV1>,
    agent_snapshot_or_none: Option<AgentCognitionSnapshotV1>,
    memory_snapshot_or_none: Option<AgentMemorySnapshotV1>,
    decision_trace_or_none: Option<DecisionTraceV1>,
    population_service_report_or_none: Option<next_world::PopulationNavigationServiceReportV1>,
    streaming_transition_or_none: Option<next_world::WorldTransitionCommitV1>,
}

/// Immutable result of one committed Runtime + World Services generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldServicesTickCommitV1 {
    pub runtime_report: TickReport,
    pub world_streaming_snapshot: WorldStreamingSnapshotV1,
    pub routine_snapshot_or_none: Option<WorldRoutineSnapshotV1>,
    pub population_snapshot_or_none: Option<WorldPopulationSnapshotV1>,
    pub activity_snapshot_or_none: Option<WorldActivitySnapshotV1>,
    pub agent_snapshot_or_none: Option<AgentCognitionSnapshotV1>,
    pub memory_snapshot_or_none: Option<AgentMemorySnapshotV1>,
    pub decision_trace_or_none: Option<DecisionTraceV1>,
    pub population_service_report_or_none: Option<next_world::PopulationNavigationServiceReportV1>,
    pub streaming_transition_or_none: Option<next_world::WorldTransitionCommitV1>,
    pub application_owner_segments: Vec<SaveSegmentDescriptor>,
    pub application_state_root: StateRoot,
}

impl RuntimeTickPreparation<'_> {
    pub fn prepare_with_world_services(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        world: &next_world::WorldStreamerV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(
            commands, routine, population, None, None, world, None, None,
        )
    }

    pub fn prepare_with_world_services_and_cognition(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(
            commands,
            routine,
            population,
            None,
            Some(cognition),
            world,
            None,
            None,
        )
    }

    pub fn prepare_with_world_services_and_streaming(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        world: &next_world::WorldStreamerV1,
        publication: next_world::PreparedWorldStreamingPublicationV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(
            commands,
            routine,
            population,
            None,
            None,
            world,
            Some(publication),
            None,
        )
    }

    pub fn prepare_with_world_services_cognition_and_streaming(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        publication: next_world::PreparedWorldStreamingPublicationV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(
            commands,
            routine,
            population,
            None,
            Some(cognition),
            world,
            Some(publication),
            None,
        )
    }

    pub fn prepare_with_world_services_cognition_and_activity(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity: &next_world::WorldActivityOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(
            commands,
            routine,
            population,
            Some(activity),
            Some(cognition),
            world,
            None,
            None,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the R4d joint preparation boundary names every independently owned projection"
    )]
    pub fn prepare_with_world_services_cognition_activity_and_streaming(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity: &next_world::WorldActivityOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        publication: next_world::PreparedWorldStreamingPublicationV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.prepare_world_services_internal(
            commands,
            routine,
            population,
            Some(activity),
            Some(cognition),
            world,
            Some(publication),
            None,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the joint R4c preparation boundary names all independently owned world-service inputs"
    )]
    fn prepare_world_services_internal(
        self,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity: Option<&next_world::WorldActivityOwnerV1>,
        cognition: Option<&StrategicAgentOwnersV1>,
        world: &next_world::WorldStreamerV1,
        streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
        replay_ingress: Option<next_contracts::input::ClosedIngressBatchV1>,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        let base_world_state_hash = world.snapshot().state_hash()?;
        let staged_world_snapshot = streaming.as_ref().map_or_else(
            || world.snapshot().clone(),
            |value| value.next_snapshot().clone(),
        );
        let mut routine_stage = WorldRoutineStageContextV1::capture(self.runtime, routine)?;
        let mut population_stage =
            WorldPopulationStageContextV1::capture(self.runtime, population)?;
        let mut activity_stage = activity
            .map(|owner| {
                super::super::world_activity::WorldActivityStageContextV1::capture(
                    self.runtime,
                    owner,
                    population,
                )
            })
            .transpose()?;
        let mut cognition_stage = cognition
            .map(|owners| {
                super::super::agent_cognition::AgentCognitionStageContextV1::capture(
                    self.runtime,
                    owners,
                    population,
                )
            })
            .transpose()?;
        let runtime = self.runtime.prepare_tick_internal(
            self.base_generation,
            self.ingress_checkpoint,
            commands,
            &mut NoOutcomes,
            replay_ingress,
            streaming
                .as_ref()
                .map(|publication| WorldStreamingStageContext { world, publication }),
            Some(&mut routine_stage),
            Some(&mut population_stage),
            activity_stage.as_mut(),
            cognition_stage.as_mut(),
        )?;
        let routine_snapshot_or_none = routine_stage.finish(runtime.next_tick())?;
        let routine = routine
            .prepare_publication(routine_snapshot_or_none, runtime.next_tick())
            .map_err(map_routine_owner_error)?;
        let (population_snapshot_or_none, population_service_report_or_none) =
            population_stage.finish(runtime.next_tick())?;
        let population = population
            .prepare_publication(
                population_snapshot_or_none,
                population_service_report_or_none,
                runtime.next_tick(),
            )
            .map_err(map_population_owner_error)?;
        let (activity_publication, activity_snapshot_or_none) =
            match (activity, activity_stage.as_ref()) {
                (Some(owner), Some(stage)) => {
                    let snapshot = stage.finish(runtime.next_tick())?;
                    let publication = owner
                        .prepare_publication(snapshot.clone(), runtime.next_tick())
                        .map_err(map_activity_owner_error)?;
                    (Some(publication), Some(snapshot))
                }
                (None, None) => (None, None),
                _ => return Err(RuntimeFatalError::WorldActivityInternalInvariant),
            };
        let (
            cognition_publication,
            agent_snapshot_or_none,
            memory_snapshot_or_none,
            decision_trace_or_none,
        ) = match (cognition, cognition_stage.as_ref()) {
            (Some(owners), Some(stage)) => {
                let (agent_snapshot, memory_snapshot, trace) = stage.finish()?;
                let publication = owners
                    .prepare_publication(agent_snapshot.clone(), memory_snapshot.clone())
                    .map_err(map_cognition_owner_error)?;
                (
                    Some(publication),
                    Some(agent_snapshot),
                    Some(memory_snapshot),
                    trace,
                )
            }
            (None, None) => (None, None, None, None),
            _ => return Err(RuntimeFatalError::AgentCognitionInternalInvariant),
        };
        Ok(PreparedRuntimeWorldServicesTickV1 {
            runtime,
            routine,
            population,
            activity: activity_publication,
            cognition: cognition_publication,
            activity_snapshot_or_none,
            agent_snapshot_or_none,
            memory_snapshot_or_none,
            decision_trace_or_none,
            base_world_state_hash,
            staged_world_snapshot,
            streaming,
        })
    }
}

impl PreparedRuntimeWorldServicesTickV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.runtime.events()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.runtime.physics_snapshot()
    }

    #[must_use]
    pub fn rpg_snapshot(&self) -> next_contracts::rpg::RpgSnapshotV2 {
        self.runtime.rpg_snapshot()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn population_snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.population.snapshot_or_none()
    }

    #[must_use]
    pub const fn activity_snapshot_or_none(&self) -> Option<&WorldActivitySnapshotV1> {
        self.activity_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn agent_snapshot_or_none(&self) -> Option<&AgentCognitionSnapshotV1> {
        self.agent_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn memory_snapshot_or_none(&self) -> Option<&AgentMemorySnapshotV1> {
        self.memory_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn decision_trace_or_none(&self) -> Option<&DecisionTraceV1> {
        self.decision_trace_or_none.as_ref()
    }

    #[must_use]
    pub const fn population_service_report_or_none(
        &self,
    ) -> Option<&next_world::PopulationNavigationServiceReportV1> {
        self.population.service_report_or_none()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.runtime.world_checkpoint_with_canonical_components()
    }
}

impl ValidatedRuntimeWorldServicesTickV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.generation.runtime.next_tick()
    }

    #[must_use]
    pub fn events(&self) -> &[next_contracts::command::DomainEvent] {
        self.generation.runtime.events()
    }

    #[must_use]
    pub fn report(&self) -> &TickReport {
        self.generation.runtime.report()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.generation.runtime.physics_snapshot()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.generation.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.generation.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn population_snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.generation.population.snapshot_or_none()
    }

    #[must_use]
    pub const fn activity_snapshot_or_none(&self) -> Option<&WorldActivitySnapshotV1> {
        self.generation.activity_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn agent_snapshot_or_none(&self) -> Option<&AgentCognitionSnapshotV1> {
        self.generation.agent_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn memory_snapshot_or_none(&self) -> Option<&AgentMemorySnapshotV1> {
        self.generation.memory_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn decision_trace_or_none(&self) -> Option<&DecisionTraceV1> {
        self.generation.decision_trace_or_none.as_ref()
    }

    #[must_use]
    pub const fn population_service_report_or_none(
        &self,
    ) -> Option<&next_world::PopulationNavigationServiceReportV1> {
        self.generation.population.service_report_or_none()
    }

    #[must_use]
    pub const fn application_state_root(&self) -> StateRoot {
        self.application_state_root
    }

    #[must_use]
    pub fn application_owner_segments(&self) -> &[SaveSegmentDescriptor] {
        &self.application_owner_segments
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.generation
            .runtime
            .world_checkpoint_with_canonical_components()
    }
}

impl ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.generation.runtime.next_tick()
    }

    #[must_use]
    pub const fn world_streaming_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.generation.staged_world_snapshot
    }

    #[must_use]
    pub const fn routine_snapshot_or_none(&self) -> Option<&WorldRoutineSnapshotV1> {
        self.generation.routine.snapshot_or_none()
    }

    #[must_use]
    pub const fn population_snapshot_or_none(&self) -> Option<&WorldPopulationSnapshotV1> {
        self.generation.population.snapshot_or_none()
    }

    #[must_use]
    pub const fn activity_snapshot_or_none(&self) -> Option<&WorldActivitySnapshotV1> {
        self.generation.activity_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn agent_snapshot_or_none(&self) -> Option<&AgentCognitionSnapshotV1> {
        self.generation.agent_snapshot_or_none.as_ref()
    }

    #[must_use]
    pub const fn memory_snapshot_or_none(&self) -> Option<&AgentMemorySnapshotV1> {
        self.generation.memory_snapshot_or_none.as_ref()
    }

    pub fn world_checkpoint_with_canonical_components(
        &self,
    ) -> Result<
        (
            next_contracts::snapshot::WorldCheckpointV4,
            WorldCheckpointCanonicalComponentsV1,
        ),
        next_contracts::snapshot::WorldCheckpointError,
    > {
        self.generation
            .runtime
            .world_checkpoint_with_canonical_components()
    }
}

impl RuntimeState {
    #[allow(
        clippy::too_many_arguments,
        reason = "replay must restore the same independently owned inputs as live R4c preparation"
    )]
    pub(in crate::engine) fn prepare_replay_world_services_tick_with_cognition(
        &self,
        closed_ingress_batch: next_contracts::input::ClosedIngressBatchV1,
        commands: impl IntoIterator<Item = WorldCommand>,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        self.tick_preparation().prepare_world_services_internal(
            commands,
            routine,
            population,
            None,
            Some(cognition),
            world,
            streaming,
            Some(closed_ingress_batch),
        )
    }

    /// Stages a controller configuration revision inside an already prepared
    /// Runtime + World Services generation without exposing its Runtime-only
    /// candidate to the composition root.
    pub fn stage_player_input_configuration_activation_world_services(
        &self,
        mut prepared: PreparedRuntimeWorldServicesTickV1,
        source_id: InputSourceId,
        action_map: ActionMapManifestV1,
        context_stack: InputContextStackV1,
    ) -> Result<PreparedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        prepared.runtime = self.stage_player_input_configuration_activation(
            prepared.runtime,
            source_id,
            action_map,
            context_stack,
        )?;
        Ok(prepared)
    }

    pub fn validate_prepared_world_services_tick(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        let generation = self.validate_prepared_world_services_generation(
            routine, population, None, None, world, prepared,
        )?;
        let (_, components) = generation
            .runtime
            .world_checkpoint_with_canonical_components()?;
        let routine_snapshot_or_none = generation.routine.snapshot_or_none().copied();
        let population_snapshot_or_none = generation.population.snapshot_or_none().cloned();
        let (application_owner_segments, application_state_root) = application_closure(
            &components,
            &generation.staged_world_snapshot,
            routine_snapshot_or_none.as_ref(),
            population_snapshot_or_none.as_ref(),
            generation.activity_snapshot_or_none.as_ref(),
            generation.agent_snapshot_or_none.as_ref(),
            generation.memory_snapshot_or_none.as_ref(),
        )?;
        Ok(ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        })
    }

    pub fn validate_prepared_world_services_tick_with_cognition(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        let generation = self.validate_prepared_world_services_generation(
            routine,
            population,
            None,
            Some(cognition),
            world,
            prepared,
        )?;
        let (_, components) = generation
            .runtime
            .world_checkpoint_with_canonical_components()?;
        let routine_snapshot_or_none = generation.routine.snapshot_or_none().copied();
        let population_snapshot_or_none = generation.population.snapshot_or_none().cloned();
        let (application_owner_segments, application_state_root) = application_closure(
            &components,
            &generation.staged_world_snapshot,
            routine_snapshot_or_none.as_ref(),
            population_snapshot_or_none.as_ref(),
            generation.activity_snapshot_or_none.as_ref(),
            generation.agent_snapshot_or_none.as_ref(),
            generation.memory_snapshot_or_none.as_ref(),
        )?;
        Ok(ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the R4d validation boundary names every independently owned projection"
    )]
    pub fn validate_prepared_world_services_tick_with_cognition_and_activity(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity: &next_world::WorldActivityOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickV1, RuntimeFatalError> {
        let generation = self.validate_prepared_world_services_generation(
            routine,
            population,
            Some(activity),
            Some(cognition),
            world,
            prepared,
        )?;
        let (_, components) = generation
            .runtime
            .world_checkpoint_with_canonical_components()?;
        let routine_snapshot_or_none = generation.routine.snapshot_or_none().copied();
        let population_snapshot_or_none = generation.population.snapshot_or_none().cloned();
        let (application_owner_segments, application_state_root) = application_closure(
            &components,
            &generation.staged_world_snapshot,
            routine_snapshot_or_none.as_ref(),
            population_snapshot_or_none.as_ref(),
            generation.activity_snapshot_or_none.as_ref(),
            generation.agent_snapshot_or_none.as_ref(),
            generation.memory_snapshot_or_none.as_ref(),
        )?;
        Ok(ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        })
    }

    /// Validates the same joint Runtime + World Services generation used by
    /// the evidence-bearing path, but defers full application canonicalization
    /// until a checkpoint, replay proof, or save boundary actually requests it.
    pub fn validate_prepared_world_services_tick_without_application_evidence(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1, RuntimeFatalError>
    {
        Ok(
            ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
                generation: self.validate_prepared_world_services_generation(
                    routine, population, None, None, world, prepared,
                )?,
            },
        )
    }

    pub fn validate_prepared_world_services_tick_with_cognition_without_application_evidence(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1, RuntimeFatalError>
    {
        Ok(
            ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
                generation: self.validate_prepared_world_services_generation(
                    routine,
                    population,
                    None,
                    Some(cognition),
                    world,
                    prepared,
                )?,
            },
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the R4d validation boundary names every independently owned projection"
    )]
    pub fn validate_prepared_world_services_tick_with_cognition_and_activity_without_application_evidence(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity: &next_world::WorldActivityOwnerV1,
        cognition: &StrategicAgentOwnersV1,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1, RuntimeFatalError>
    {
        Ok(
            ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1 {
                generation: self.validate_prepared_world_services_generation(
                    routine,
                    population,
                    Some(activity),
                    Some(cognition),
                    world,
                    prepared,
                )?,
            },
        )
    }

    fn validate_prepared_world_services_generation(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
        population: &next_world::WorldPopulationOwnerV1,
        activity: Option<&next_world::WorldActivityOwnerV1>,
        cognition: Option<&StrategicAgentOwnersV1>,
        world: &next_world::WorldStreamerV1,
        prepared: PreparedRuntimeWorldServicesTickV1,
    ) -> Result<ValidatedWorldServicesGenerationV1, RuntimeFatalError> {
        if world.snapshot().state_hash()? != prepared.base_world_state_hash {
            return Err(RuntimeFatalError::PreparedWorldServicesGenerationStale);
        }
        let runtime = self
            .validate_prepared_tick(prepared.runtime)
            .map_err(|error| {
                if error == RuntimeFatalError::PreparedGenerationStale {
                    RuntimeFatalError::PreparedWorldServicesGenerationStale
                } else {
                    error
                }
            })?;
        let routine = routine
            .validate_prepared_publication(prepared.routine)
            .map_err(map_routine_owner_error)?;
        let population = population
            .validate_prepared_publication(prepared.population)
            .map_err(map_population_owner_error)?;
        let activity = match (activity, prepared.activity) {
            (Some(owner), Some(publication)) => Some(
                owner
                    .validate_prepared_publication(publication)
                    .map_err(map_activity_owner_error)?,
            ),
            (None, None) => None,
            _ => return Err(RuntimeFatalError::WorldActivityInternalInvariant),
        };
        let cognition = match (cognition, prepared.cognition) {
            (Some(owner), Some(publication)) => Some(
                owner
                    .validate_prepared_publication(publication)
                    .map_err(map_cognition_owner_error)?,
            ),
            (None, None) => None,
            _ => return Err(RuntimeFatalError::AgentCognitionInternalInvariant),
        };
        let streaming = prepared
            .streaming
            .map(|publication| {
                world.validate_prepared_publication(publication, runtime.report().tick)
            })
            .transpose()?;
        Ok(ValidatedWorldServicesGenerationV1 {
            runtime,
            routine,
            population,
            activity,
            cognition,
            activity_snapshot_or_none: prepared.activity_snapshot_or_none,
            agent_snapshot_or_none: prepared.agent_snapshot_or_none,
            memory_snapshot_or_none: prepared.memory_snapshot_or_none,
            decision_trace_or_none: prepared.decision_trace_or_none,
            base_world_state_hash: prepared.base_world_state_hash,
            staged_world_snapshot: prepared.staged_world_snapshot,
            streaming,
        })
    }

    /// Performs one final read-only preflight, then publishes every live owner
    /// in fixed order with no fallible operation after the first write.
    pub fn commit_validated_world_services_tick(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickV1,
    ) -> Result<WorldServicesTickCommitV1, RuntimeFatalError> {
        let ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        } = validated;
        let committed = self.commit_validated_world_services_generation(
            routine, population, None, None, world, generation,
        )?;
        Ok(WorldServicesTickCommitV1 {
            runtime_report: committed.runtime_report,
            world_streaming_snapshot: committed.world_streaming_snapshot,
            routine_snapshot_or_none: committed.routine_snapshot_or_none,
            population_snapshot_or_none: committed.population_snapshot_or_none,
            activity_snapshot_or_none: committed.activity_snapshot_or_none,
            agent_snapshot_or_none: committed.agent_snapshot_or_none,
            memory_snapshot_or_none: committed.memory_snapshot_or_none,
            decision_trace_or_none: committed.decision_trace_or_none,
            population_service_report_or_none: committed.population_service_report_or_none,
            streaming_transition_or_none: committed.streaming_transition_or_none,
            application_owner_segments,
            application_state_root,
        })
    }

    pub fn commit_validated_world_services_tick_with_cognition(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        cognition: &mut StrategicAgentOwnersV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickV1,
    ) -> Result<WorldServicesTickCommitV1, RuntimeFatalError> {
        let ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        } = validated;
        let committed = self.commit_validated_world_services_generation(
            routine,
            population,
            None,
            Some(cognition),
            world,
            generation,
        )?;
        Ok(WorldServicesTickCommitV1 {
            runtime_report: committed.runtime_report,
            world_streaming_snapshot: committed.world_streaming_snapshot,
            routine_snapshot_or_none: committed.routine_snapshot_or_none,
            population_snapshot_or_none: committed.population_snapshot_or_none,
            activity_snapshot_or_none: committed.activity_snapshot_or_none,
            agent_snapshot_or_none: committed.agent_snapshot_or_none,
            memory_snapshot_or_none: committed.memory_snapshot_or_none,
            decision_trace_or_none: committed.decision_trace_or_none,
            population_service_report_or_none: committed.population_service_report_or_none,
            streaming_transition_or_none: committed.streaming_transition_or_none,
            application_owner_segments,
            application_state_root,
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the R4d commit boundary names every independently owned projection"
    )]
    pub fn commit_validated_world_services_tick_with_cognition_and_activity(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        activity: &mut next_world::WorldActivityOwnerV1,
        cognition: &mut StrategicAgentOwnersV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickV1,
    ) -> Result<WorldServicesTickCommitV1, RuntimeFatalError> {
        let ValidatedRuntimeWorldServicesTickV1 {
            generation,
            application_owner_segments,
            application_state_root,
        } = validated;
        let committed = self.commit_validated_world_services_generation(
            routine,
            population,
            Some(activity),
            Some(cognition),
            world,
            generation,
        )?;
        Ok(WorldServicesTickCommitV1 {
            runtime_report: committed.runtime_report,
            world_streaming_snapshot: committed.world_streaming_snapshot,
            routine_snapshot_or_none: committed.routine_snapshot_or_none,
            population_snapshot_or_none: committed.population_snapshot_or_none,
            activity_snapshot_or_none: committed.activity_snapshot_or_none,
            agent_snapshot_or_none: committed.agent_snapshot_or_none,
            memory_snapshot_or_none: committed.memory_snapshot_or_none,
            decision_trace_or_none: committed.decision_trace_or_none,
            population_service_report_or_none: committed.population_service_report_or_none,
            streaming_transition_or_none: committed.streaming_transition_or_none,
            application_owner_segments,
            application_state_root,
        })
    }

    /// Commits a jointly validated live generation without constructing a
    /// full application root that the caller will discard.
    pub fn commit_validated_world_services_tick_without_application_evidence(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1,
    ) -> Result<TickReport, RuntimeFatalError> {
        Ok(self
            .commit_validated_world_services_generation(
                routine,
                population,
                None,
                None,
                world,
                validated.generation,
            )?
            .runtime_report)
    }

    pub fn commit_validated_world_services_tick_with_cognition_without_application_evidence(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        cognition: &mut StrategicAgentOwnersV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1,
    ) -> Result<TickReport, RuntimeFatalError> {
        Ok(self
            .commit_validated_world_services_generation(
                routine,
                population,
                None,
                Some(cognition),
                world,
                validated.generation,
            )?
            .runtime_report)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the R4d commit boundary names every independently owned projection"
    )]
    pub fn commit_validated_world_services_tick_with_cognition_and_activity_without_application_evidence(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        activity: &mut next_world::WorldActivityOwnerV1,
        cognition: &mut StrategicAgentOwnersV1,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedRuntimeWorldServicesTickWithoutApplicationEvidenceV1,
    ) -> Result<TickReport, RuntimeFatalError> {
        Ok(self
            .commit_validated_world_services_generation(
                routine,
                population,
                Some(activity),
                Some(cognition),
                world,
                validated.generation,
            )?
            .runtime_report)
    }

    fn commit_validated_world_services_generation(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        activity: Option<&mut next_world::WorldActivityOwnerV1>,
        cognition: Option<&mut StrategicAgentOwnersV1>,
        world: &mut next_world::WorldStreamerV1,
        validated: ValidatedWorldServicesGenerationV1,
    ) -> Result<CommittedWorldServicesGenerationV1, RuntimeFatalError> {
        if !validated.runtime.0.base_generation.matches(self)
            || world.snapshot().state_hash()? != validated.base_world_state_hash
            || routine
                .preflight_validated_publication(&validated.routine)
                .is_err()
            || population
                .preflight_validated_publication(&validated.population)
                .is_err()
            || match (activity.as_deref(), validated.activity.as_ref()) {
                (Some(owner), Some(publication)) => {
                    owner.preflight_validated_publication(publication).is_err()
                }
                (None, None) => false,
                _ => true,
            }
            || match (cognition.as_deref(), validated.cognition.as_ref()) {
                (Some(owner), Some(publication)) => {
                    owner.preflight_validated_publication(publication).is_err()
                }
                (None, None) => false,
                _ => true,
            }
            || validated.streaming.as_ref().is_some_and(|streaming| {
                world
                    .preflight_validated_publication(streaming, validated.runtime.report().tick)
                    .is_err()
            })
        {
            return Err(RuntimeFatalError::PreparedWorldServicesGenerationStale);
        }

        let ValidatedWorldServicesGenerationV1 {
            runtime: validated_runtime,
            routine: validated_routine,
            population: validated_population,
            activity: validated_activity,
            cognition: validated_cognition,
            activity_snapshot_or_none,
            agent_snapshot_or_none,
            memory_snapshot_or_none,
            decision_trace_or_none,
            staged_world_snapshot,
            streaming,
            ..
        } = validated;
        let runtime_report = self.commit_validated_tick(validated_runtime);
        routine.commit_validated_publication(validated_routine);
        let population_service_report_or_none =
            validated_population.service_report_or_none().cloned();
        population.commit_validated_publication(validated_population);
        if let (Some(owner), Some(publication)) = (activity, validated_activity) {
            owner.commit_validated_publication(publication);
        }
        if let (Some(owner), Some(publication)) = (cognition, validated_cognition) {
            owner.commit_validated_publication(publication);
        }
        let streaming_transition_or_none =
            streaming.and_then(|streaming| world.commit_validated_publication(streaming));
        Ok(CommittedWorldServicesGenerationV1 {
            runtime_report,
            world_streaming_snapshot: staged_world_snapshot,
            routine_snapshot_or_none: routine.snapshot_or_none().copied(),
            population_snapshot_or_none: population.snapshot_or_none().cloned(),
            activity_snapshot_or_none,
            agent_snapshot_or_none,
            memory_snapshot_or_none,
            decision_trace_or_none,
            population_service_report_or_none,
            streaming_transition_or_none,
        })
    }
}

fn map_routine_owner_error(error: next_world::WorldRoutineOwnerError) -> RuntimeFatalError {
    if matches!(error, next_world::WorldRoutineOwnerError::PublicationStale) {
        RuntimeFatalError::PreparedWorldServicesGenerationStale
    } else {
        RuntimeFatalError::WorldRoutineInternalInvariant
    }
}

fn map_population_owner_error(error: next_world::WorldPopulationOwnerError) -> RuntimeFatalError {
    if matches!(
        error,
        next_world::WorldPopulationOwnerError::PublicationStale
    ) {
        RuntimeFatalError::PreparedWorldServicesGenerationStale
    } else {
        RuntimeFatalError::WorldPopulationInternalInvariant
    }
}

fn map_activity_owner_error(error: next_world::WorldActivityOwnerError) -> RuntimeFatalError {
    if matches!(error, next_world::WorldActivityOwnerError::PublicationStale) {
        RuntimeFatalError::PreparedWorldServicesGenerationStale
    } else {
        RuntimeFatalError::WorldActivityInternalInvariant
    }
}

fn map_cognition_owner_error(
    error: next_agent::cognition::StrategicAgentError,
) -> RuntimeFatalError {
    if matches!(
        error,
        next_agent::cognition::StrategicAgentError::PreparedPublicationStale
    ) {
        RuntimeFatalError::PreparedWorldServicesGenerationStale
    } else {
        RuntimeFatalError::AgentCognitionInternalInvariant
    }
}
