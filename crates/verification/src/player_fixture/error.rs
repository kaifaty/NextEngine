use std::error::Error;
use std::fmt::{Display, Formatter};

use next_runtime::{RuntimeFatalError, SnapshotRestoreError};
use next_world::WorldStreamingError;

use crate::NeutralFixtureError;

#[derive(Debug)]
pub enum CanonicalFixtureError {
    Identifier(next_contracts::ids::IdentifierError),
    Canonical(next_contracts::canonical::CanonicalError),
}

impl Display for CanonicalFixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for CanonicalFixtureError {}

impl From<next_contracts::ids::IdentifierError> for CanonicalFixtureError {
    fn from(error: next_contracts::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<next_contracts::canonical::CanonicalError> for CanonicalFixtureError {
    fn from(error: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

#[derive(Debug)]
pub enum PlayCheckError {
    Fixture(NeutralFixtureError),
    ReferenceGame(next_reference_game::ReferenceGameError),
    ReferenceInput(next_reference_game::ReferenceInputError),
    CanonicalFixture(CanonicalFixtureError),
    Input(next_runtime::InputAdmissionError),
    Runtime(RuntimeFatalError),
    Restore(SnapshotRestoreError),
    Checkpoint(next_contracts::snapshot::WorldCheckpointError),
    Replay(crate::ReplayError),
    PersistenceReplay(crate::PersistenceReplayCheckError),
    Ledger(next_contracts::ledger::CommandLedgerError),
    Canonical(next_contracts::canonical::CanonicalError),
    CountOverflow,
    BodyMissing,
    InteractiveObjectMissing,
    CookedNpcMissing,
    CookedPlayerMissing,
    CookedDialogueMissing,
    CookedQuestMissing,
    AcceptanceMismatch(String),
    BackendParityMismatch,
    PresentationAssetMissing,
    WorldPartitionEmpty,
    WorldStreaming(WorldStreamingError),
    WorldStreamingContract(next_contracts::world::WorldStreamingContractError),
    WorldStreamingResumeMismatch,
    WorldStreamingMutatedRpg,
    Agent(next_agent::AgentPlannerError),
    AgentActionMissing,
    AgentCommandRejected,
    PresentationExtraction(next_presentation::PresentationExtractionError),
    Render(next_render::RenderDeviceError),
}

impl Display for PlayCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixture(error) => write!(formatter, "{error}"),
            Self::ReferenceGame(error) => write!(formatter, "{error}"),
            Self::ReferenceInput(error) => write!(formatter, "{error}"),
            Self::CanonicalFixture(error) => write!(formatter, "{error}"),
            Self::Input(error) => write!(formatter, "{error}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::Restore(error) => write!(formatter, "{error}"),
            Self::Checkpoint(error) => write!(formatter, "{error}"),
            Self::Replay(error) => write!(formatter, "{error}"),
            Self::PersistenceReplay(error) => write!(formatter, "{error}"),
            Self::Ledger(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::CountOverflow => formatter.write_str("play check count overflow"),
            Self::BodyMissing => formatter.write_str("play check capsule body is missing"),
            Self::InteractiveObjectMissing => {
                formatter.write_str("play check interactive object is missing")
            }
            Self::CookedNpcMissing => formatter.write_str("play check cooked NPC is missing"),
            Self::CookedPlayerMissing => formatter.write_str("play check player is missing"),
            Self::CookedDialogueMissing => {
                formatter.write_str("play check cooked dialogue is missing")
            }
            Self::CookedQuestMissing => formatter.write_str("play check cooked quest is missing"),
            Self::AcceptanceMismatch(details) => {
                write!(formatter, "play check result did not match: {details}")
            }
            Self::BackendParityMismatch => {
                formatter.write_str("reference and PhysX tick reports diverged")
            }
            Self::PresentationAssetMissing => {
                formatter.write_str("cooked presentation asset is missing")
            }
            Self::WorldPartitionEmpty => {
                formatter.write_str("cooked world partition requires two chunks")
            }
            Self::WorldStreaming(error) => write!(formatter, "{error}"),
            Self::WorldStreamingContract(error) => write!(formatter, "{error}"),
            Self::WorldStreamingResumeMismatch => {
                formatter.write_str("restaged world group changed after save/restore")
            }
            Self::WorldStreamingMutatedRpg => {
                formatter.write_str("world transition mutated durable RPG state")
            }
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::AgentActionMissing => formatter.write_str("deterministic NPC action is missing"),
            Self::AgentCommandRejected => {
                formatter.write_str("deterministic NPC command was rejected")
            }
            Self::PresentationExtraction(error) => write!(formatter, "{error}"),
            Self::Render(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for PlayCheckError {}

impl From<NeutralFixtureError> for PlayCheckError {
    fn from(error: NeutralFixtureError) -> Self {
        Self::Fixture(error)
    }
}

impl From<next_reference_game::ReferenceGameError> for PlayCheckError {
    fn from(error: next_reference_game::ReferenceGameError) -> Self {
        Self::ReferenceGame(error)
    }
}

impl From<next_reference_game::ReferenceInputError> for PlayCheckError {
    fn from(error: next_reference_game::ReferenceInputError) -> Self {
        Self::ReferenceInput(error)
    }
}

impl From<CanonicalFixtureError> for PlayCheckError {
    fn from(error: CanonicalFixtureError) -> Self {
        Self::CanonicalFixture(error)
    }
}

impl From<next_runtime::InputAdmissionError> for PlayCheckError {
    fn from(error: next_runtime::InputAdmissionError) -> Self {
        Self::Input(error)
    }
}

impl From<RuntimeFatalError> for PlayCheckError {
    fn from(error: RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl From<SnapshotRestoreError> for PlayCheckError {
    fn from(error: SnapshotRestoreError) -> Self {
        Self::Restore(error)
    }
}

impl From<next_contracts::snapshot::WorldCheckpointError> for PlayCheckError {
    fn from(error: next_contracts::snapshot::WorldCheckpointError) -> Self {
        Self::Checkpoint(error)
    }
}

impl From<crate::ReplayError> for PlayCheckError {
    fn from(error: crate::ReplayError) -> Self {
        Self::Replay(error)
    }
}

impl From<crate::PersistenceReplayCheckError> for PlayCheckError {
    fn from(error: crate::PersistenceReplayCheckError) -> Self {
        Self::PersistenceReplay(error)
    }
}

impl From<next_contracts::ledger::CommandLedgerError> for PlayCheckError {
    fn from(error: next_contracts::ledger::CommandLedgerError) -> Self {
        Self::Ledger(error)
    }
}

impl From<next_contracts::canonical::CanonicalError> for PlayCheckError {
    fn from(error: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<next_presentation::PresentationExtractionError> for PlayCheckError {
    fn from(error: next_presentation::PresentationExtractionError) -> Self {
        Self::PresentationExtraction(error)
    }
}

impl From<next_render::RenderDeviceError> for PlayCheckError {
    fn from(error: next_render::RenderDeviceError) -> Self {
        Self::Render(error)
    }
}

impl From<WorldStreamingError> for PlayCheckError {
    fn from(error: WorldStreamingError) -> Self {
        Self::WorldStreaming(error)
    }
}

impl From<next_contracts::world::WorldStreamingContractError> for PlayCheckError {
    fn from(error: next_contracts::world::WorldStreamingContractError) -> Self {
        Self::WorldStreamingContract(error)
    }
}

impl From<next_agent::AgentPlannerError> for PlayCheckError {
    fn from(error: next_agent::AgentPlannerError) -> Self {
        Self::Agent(error)
    }
}
