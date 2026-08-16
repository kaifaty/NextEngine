use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::command::{CommandPhase, WorldCommand};
use next_contracts::input::{ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1};
use next_contracts::mechanics::RpgDefinitionRegistryV2;
use next_contracts::physics::{
    ClosedPhysicsContactBatchV1, PhysicsQueryBatchV1, PhysicsQueryResultV1, PhysicsStepInputV2,
};
use next_contracts::snapshot::{WorldCheckpointError, WorldCheckpointV4};
use next_contracts::targeting::{AuthoritativeTargetingQueryV1, TargetingIntentV1};
use next_contracts::world_routine::InteractionAvailabilityV1;
use next_physics_api::{PhysicsBackendError, ReferencePhysicsError};

use crate::authority::AuthorityRegistry;

use super::error::{RuntimeFatalError, SnapshotRestoreError};
use super::result::TickReport;
use super::tick::{ValidatedRuntimeTick, WorldServicesTickCommitV1};
use super::{PhysicsLaunchOptions, RuntimeState};

#[derive(Debug)]
pub struct RuntimeReplayDriver {
    runtime: RuntimeState,
}

impl RuntimeReplayDriver {
    pub fn new(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::new_with_physics_options(checkpoint, authority, PhysicsLaunchOptions::default())
    }

    pub fn new_with_physics_options(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::new_with_definitions_and_physics_options(
            checkpoint,
            authority,
            RpgDefinitionRegistryV2::empty().expect("empty RPG definition registry is canonical"),
            physics_options,
        )
    }

    pub fn new_with_definitions(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        rpg_definitions: RpgDefinitionRegistryV2,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::new_with_definitions_and_physics_options(
            checkpoint,
            authority,
            rpg_definitions,
            PhysicsLaunchOptions::default(),
        )
    }

    pub fn new_with_definitions_and_physics_options(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        rpg_definitions: RpgDefinitionRegistryV2,
        physics_options: PhysicsLaunchOptions,
    ) -> Result<Self, SnapshotRestoreError> {
        Ok(Self {
            runtime: RuntimeState::restore_world_checkpoint_with_definitions_and_physics_options(
                checkpoint,
                authority,
                rpg_definitions,
                physics_options,
            )?,
        })
    }

    pub fn replay_tick(
        &mut self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: Vec<WorldCommand>,
        expected_ingress_batch: &ClosedCommandAdmissionBatchV2,
        expected_physics_step_input: &PhysicsStepInputV2,
        expected_contact_batch: &ClosedPhysicsContactBatchV1,
        expected_outcome_batch: &ClosedCommandAdmissionBatchV2,
    ) -> Result<TickReport, RuntimeReplayError> {
        let validated = self.prepare_replay_tick(
            closed_ingress_batch,
            direct_commands,
            expected_ingress_batch,
            expected_physics_step_input,
            expected_contact_batch,
            expected_outcome_batch,
        )?;
        Ok(self.runtime.commit_validated_tick(validated))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn replay_tick_with_query_facts(
        &mut self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: Vec<WorldCommand>,
        expected_ingress_batch: &ClosedCommandAdmissionBatchV2,
        expected_physics_step_input: &PhysicsStepInputV2,
        expected_contact_batch: &ClosedPhysicsContactBatchV1,
        expected_targeting_intents: &[TargetingIntentV1],
        expected_authoritative_targeting_queries: &[AuthoritativeTargetingQueryV1],
        expected_physics_query_batch: &PhysicsQueryBatchV1,
        expected_physics_query_results: &[PhysicsQueryResultV1],
        expected_outcome_batch: &ClosedCommandAdmissionBatchV2,
    ) -> Result<TickReport, RuntimeReplayError> {
        let validated = self.prepare_replay_tick(
            closed_ingress_batch,
            direct_commands,
            expected_ingress_batch,
            expected_physics_step_input,
            expected_contact_batch,
            expected_outcome_batch,
        )?;
        let report = validated.report();
        if report.targeting_intents != expected_targeting_intents {
            return Err(RuntimeReplayError::TargetingIntentMismatch { tick: report.tick });
        }
        if report.authoritative_targeting_queries != expected_authoritative_targeting_queries {
            return Err(RuntimeReplayError::TargetingQueryMismatch { tick: report.tick });
        }
        if &report.physics_query_batch != expected_physics_query_batch {
            return Err(RuntimeReplayError::PhysicsQueryBatchMismatch { tick: report.tick });
        }
        if report.physics_query_results != expected_physics_query_results {
            return Err(RuntimeReplayError::PhysicsQueryResultMismatch { tick: report.tick });
        }
        Ok(self.runtime.commit_validated_tick(validated))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn replay_world_services_tick_v9(
        &mut self,
        routine: &mut next_world::WorldRoutineOwnerV1,
        population: &mut next_world::WorldPopulationOwnerV1,
        activity: &mut next_world::WorldActivityOwnerV1,
        cognition: &mut next_agent::cognition::StrategicAgentOwnersV1,
        world: &mut next_world::WorldStreamerV1,
        streaming: Option<next_world::PreparedWorldStreamingPublicationV1>,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: Vec<WorldCommand>,
        expected_ingress_batch: &ClosedCommandAdmissionBatchV2,
        expected_physics_step_input: &PhysicsStepInputV2,
        expected_contact_batch: &ClosedPhysicsContactBatchV1,
        expected_targeting_intents: &[TargetingIntentV1],
        expected_authoritative_targeting_queries: &[AuthoritativeTargetingQueryV1],
        expected_physics_query_batch: &PhysicsQueryBatchV1,
        expected_physics_query_results: &[PhysicsQueryResultV1],
        expected_outcome_batch: &ClosedCommandAdmissionBatchV2,
        expected_interaction_availability: &[InteractionAvailabilityV1],
    ) -> Result<WorldServicesTickCommitV1, RuntimeReplayError> {
        let ingress_batch = self
            .runtime
            .preview_replay_ingress_batch(closed_ingress_batch.clone(), &direct_commands)?;
        if &ingress_batch != expected_ingress_batch {
            return Err(RuntimeReplayError::CommandBatchMismatch {
                tick: self.runtime.next_tick,
                phase: CommandPhase::Ingress,
            });
        }
        expected_contact_batch
            .validate_against_catalog(&self.runtime.physics.checkpoint().catalog)
            .map_err(ReferencePhysicsError::from)
            .map_err(PhysicsBackendError::from)
            .map_err(RuntimeFatalError::from)?;
        let prepared = self
            .runtime
            .prepare_replay_world_services_tick_with_cognition(
                closed_ingress_batch,
                direct_commands,
                routine,
                population,
                activity,
                cognition,
                world,
                streaming,
            )?;
        let validated = self
            .runtime
            .validate_prepared_world_services_tick_with_cognition_and_activity(
                routine, population, activity, cognition, world, prepared,
            )?;
        let report = validated.report();
        if &report.physics_step_input != expected_physics_step_input {
            return Err(RuntimeReplayError::PhysicsStepInputMismatch { tick: report.tick });
        }
        if &report.contact_batch != expected_contact_batch {
            return Err(RuntimeReplayError::ContactBatchMismatch { tick: report.tick });
        }
        if report.command_batches.get(1) != Some(expected_outcome_batch) {
            return Err(RuntimeReplayError::CommandBatchMismatch {
                tick: report.tick,
                phase: CommandPhase::Outcome,
            });
        }
        if report.targeting_intents != expected_targeting_intents {
            return Err(RuntimeReplayError::TargetingIntentMismatch { tick: report.tick });
        }
        if report.authoritative_targeting_queries != expected_authoritative_targeting_queries {
            return Err(RuntimeReplayError::TargetingQueryMismatch { tick: report.tick });
        }
        if &report.physics_query_batch != expected_physics_query_batch {
            return Err(RuntimeReplayError::PhysicsQueryBatchMismatch { tick: report.tick });
        }
        if report.physics_query_results != expected_physics_query_results {
            return Err(RuntimeReplayError::PhysicsQueryResultMismatch { tick: report.tick });
        }
        if report.interaction_availability != expected_interaction_availability {
            return Err(RuntimeReplayError::InteractionAvailabilityMismatch { tick: report.tick });
        }
        Ok(self
            .runtime
            .commit_validated_world_services_tick_with_cognition_and_activity(
                routine, population, activity, cognition, world, validated,
            )?)
    }

    /// Prepares one recorded tick against the live runtime generation without
    /// committing it. The live runtime stays untouched until every recorded
    /// expectation matched; no per-tick runtime fork is required.
    fn prepare_replay_tick(
        &self,
        closed_ingress_batch: ClosedIngressBatchV1,
        direct_commands: Vec<WorldCommand>,
        expected_ingress_batch: &ClosedCommandAdmissionBatchV2,
        expected_physics_step_input: &PhysicsStepInputV2,
        expected_contact_batch: &ClosedPhysicsContactBatchV1,
        expected_outcome_batch: &ClosedCommandAdmissionBatchV2,
    ) -> Result<ValidatedRuntimeTick, RuntimeReplayError> {
        let ingress_batch = self
            .runtime
            .preview_replay_ingress_batch(closed_ingress_batch.clone(), &direct_commands)?;
        if &ingress_batch != expected_ingress_batch {
            return Err(RuntimeReplayError::CommandBatchMismatch {
                tick: self.runtime.next_tick,
                phase: CommandPhase::Ingress,
            });
        }
        expected_contact_batch
            .validate_against_catalog(&self.runtime.physics.checkpoint().catalog)
            .map_err(ReferencePhysicsError::from)
            .map_err(PhysicsBackendError::from)
            .map_err(RuntimeFatalError::from)?;
        let validated = self
            .runtime
            .prepare_replay_ingress_tick(closed_ingress_batch, direct_commands)?;
        let report = validated.report();
        if &report.physics_step_input != expected_physics_step_input {
            return Err(RuntimeReplayError::PhysicsStepInputMismatch { tick: report.tick });
        }
        if &report.contact_batch != expected_contact_batch {
            return Err(RuntimeReplayError::ContactBatchMismatch { tick: report.tick });
        }
        if report.command_batches.get(1) != Some(expected_outcome_batch) {
            return Err(RuntimeReplayError::CommandBatchMismatch {
                tick: report.tick,
                phase: CommandPhase::Outcome,
            });
        }
        Ok(validated)
    }

    pub fn world_checkpoint(&self) -> Result<WorldCheckpointV4, WorldCheckpointError> {
        self.runtime.world_checkpoint()
    }

    #[must_use]
    pub fn physics_snapshot(&self) -> &next_contracts::physics::PhysicsCanonicalSnapshotV2 {
        self.runtime.physics_snapshot()
    }

    #[must_use]
    pub const fn next_tick(&self) -> u64 {
        self.runtime.next_tick()
    }

    pub fn validate_world_routine_ledger_closure(
        &self,
        routine: &next_world::WorldRoutineOwnerV1,
    ) -> Result<(), SnapshotRestoreError> {
        self.runtime.validate_world_routine_ledger_closure(routine)
    }

    pub fn validate_world_population_ledger_closure(
        &self,
        population: &next_world::WorldPopulationOwnerV1,
    ) -> Result<(), SnapshotRestoreError> {
        self.runtime
            .validate_world_population_ledger_closure(population)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RuntimeReplayError {
    Runtime(RuntimeFatalError),
    Restore(SnapshotRestoreError),
    CommandBatchMismatch { tick: u64, phase: CommandPhase },
    PhysicsStepInputMismatch { tick: u64 },
    ContactBatchMismatch { tick: u64 },
    TargetingIntentMismatch { tick: u64 },
    TargetingQueryMismatch { tick: u64 },
    PhysicsQueryBatchMismatch { tick: u64 },
    PhysicsQueryResultMismatch { tick: u64 },
    InteractionAvailabilityMismatch { tick: u64 },
}

impl RuntimeReplayError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Runtime(error) => error.stable_code(),
            Self::Restore(_) => "REPLAY_STAGING_RESTORE_FAILED",
            Self::CommandBatchMismatch { .. } => "REPLAY_COMMAND_BATCH_MISMATCH",
            Self::PhysicsStepInputMismatch { .. } => "REPLAY_PHYSICS_STEP_INPUT_MISMATCH",
            Self::ContactBatchMismatch { .. } => "REPLAY_CONTACT_BATCH_MISMATCH",
            Self::TargetingIntentMismatch { .. } => "REPLAY_TARGETING_INTENT_MISMATCH",
            Self::TargetingQueryMismatch { .. } => "REPLAY_TARGETING_QUERY_MISMATCH",
            Self::PhysicsQueryBatchMismatch { .. } => "REPLAY_PHYSICS_QUERY_BATCH_MISMATCH",
            Self::PhysicsQueryResultMismatch { .. } => "REPLAY_PHYSICS_QUERY_RESULT_MISMATCH",
            Self::InteractionAvailabilityMismatch { .. } => {
                "REPLAY_INTERACTION_AVAILABILITY_MISMATCH"
            }
        }
    }
}

impl Display for RuntimeReplayError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "replay runtime failed: {error}"),
            Self::Restore(error) => write!(formatter, "replay staging restore failed: {error}"),
            Self::CommandBatchMismatch { tick, phase } => {
                write!(
                    formatter,
                    "recorded {phase:?} command batch diverged at tick {tick}"
                )
            }
            Self::PhysicsStepInputMismatch { tick } => {
                write!(
                    formatter,
                    "recorded physics step input diverged at tick {tick}"
                )
            }
            Self::ContactBatchMismatch { tick } => {
                write!(formatter, "recorded contact batch diverged at tick {tick}")
            }
            Self::TargetingIntentMismatch { tick } => {
                write!(
                    formatter,
                    "recorded targeting intents diverged at tick {tick}"
                )
            }
            Self::TargetingQueryMismatch { tick } => {
                write!(
                    formatter,
                    "recorded targeting queries diverged at tick {tick}"
                )
            }
            Self::PhysicsQueryBatchMismatch { tick } => {
                write!(
                    formatter,
                    "recorded physics query batch diverged at tick {tick}"
                )
            }
            Self::PhysicsQueryResultMismatch { tick } => {
                write!(
                    formatter,
                    "recorded physics query results diverged at tick {tick}"
                )
            }
            Self::InteractionAvailabilityMismatch { tick } => {
                write!(
                    formatter,
                    "recorded interaction availability diverged at tick {tick}"
                )
            }
        }
    }
}

impl Error for RuntimeReplayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Restore(error) => Some(error),
            Self::CommandBatchMismatch { .. }
            | Self::PhysicsStepInputMismatch { .. }
            | Self::ContactBatchMismatch { .. }
            | Self::TargetingIntentMismatch { .. }
            | Self::TargetingQueryMismatch { .. }
            | Self::PhysicsQueryBatchMismatch { .. }
            | Self::PhysicsQueryResultMismatch { .. }
            | Self::InteractionAvailabilityMismatch { .. } => None,
        }
    }
}

impl From<RuntimeFatalError> for RuntimeReplayError {
    fn from(error: RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl From<SnapshotRestoreError> for RuntimeReplayError {
    fn from(error: SnapshotRestoreError) -> Self {
        Self::Restore(error)
    }
}
