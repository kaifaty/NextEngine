use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::command::{CommandPhase, WorldCommand};
use next_contracts::input::{ClosedCommandAdmissionBatchV2, ClosedIngressBatchV1};
use next_contracts::mechanics::RpgDefinitionRegistryV1;
use next_contracts::physics::{ClosedPhysicsContactBatchV1, PhysicsStepInputV2};
use next_contracts::snapshot::{WorldCheckpointError, WorldCheckpointV4};
use next_physics_api::{PhysicsBackendError, ReferencePhysicsError};

use crate::authority::AuthorityRegistry;

use super::error::{RuntimeFatalError, SnapshotRestoreError};
use super::result::TickReport;
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
            RpgDefinitionRegistryV1::empty().expect("empty RPG definition registry is canonical"),
            physics_options,
        )
    }

    pub fn new_with_definitions(
        checkpoint: WorldCheckpointV4,
        authority: AuthorityRegistry,
        rpg_definitions: RpgDefinitionRegistryV1,
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
        rpg_definitions: RpgDefinitionRegistryV1,
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
        let mut staged_runtime = self.runtime.fork_from_checkpoint()?;
        let report =
            staged_runtime.replay_closed_ingress_tick(closed_ingress_batch, direct_commands)?;
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
        self.runtime = staged_runtime;
        Ok(report)
    }

    pub fn world_checkpoint(&self) -> Result<WorldCheckpointV4, WorldCheckpointError> {
        self.runtime.world_checkpoint()
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
            | Self::ContactBatchMismatch { .. } => None,
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
