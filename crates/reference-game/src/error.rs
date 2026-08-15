use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ReferenceInputError {
    Identifier(next_contracts::ids::IdentifierError),
    Canonical(next_contracts::canonical::CanonicalError),
}

impl Display for ReferenceInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ReferenceInputError {}

impl From<next_contracts::ids::IdentifierError> for ReferenceInputError {
    fn from(value: next_contracts::ids::IdentifierError) -> Self {
        Self::Identifier(value)
    }
}

impl From<next_contracts::canonical::CanonicalError> for ReferenceInputError {
    fn from(value: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(value)
    }
}

#[derive(Debug)]
pub enum ReferenceGameError {
    Canonical(next_contracts::canonical::CanonicalError),
    Identifier(next_contracts::ids::IdentifierError),
    Identity(next_contracts::identity::IdentityContractError),
    Input(next_contracts::input::InputContractError),
    PlayerInput(next_player::PlayerInputError),
    Platform(next_contracts::platform::PlatformContractError),
    Physics(next_contracts::physics::PhysicsContractError),
    Authority(next_runtime::AuthorityRegistryError),
    InputContract(ReferenceInputError),
    InputAdmission(next_runtime::InputAdmissionError),
    Restore(next_runtime::SnapshotRestoreError),
    Runtime(next_runtime::RuntimeFatalError),
    Checkpoint(next_contracts::snapshot::WorldCheckpointError),
    PresentationContract(next_contracts::presentation::PresentationContractError),
    Presentation(next_presentation::PresentationExtractionError),
    AudioSceneContract(next_contracts::presentation::audio_scene::AudioSceneContractErrorV1),
    AudioExtraction(next_presentation::audio_scene::AudioSceneExtractionErrorV1),
    AudioMix(next_presentation::audio_mix::AudioMixErrorV1),
    WorldStreaming(next_world::WorldStreamingError),
    WorldRoutine(next_world::WorldRoutineOwnerError),
    WorldStreamingContract(next_contracts::world::WorldStreamingContractError),
    Agent(next_agent::AgentPlannerError),
    DuplicatePrincipal,
    CountOverflow,
    BodyMissing,
    WorldPartitionEmpty,
    WorldChunkRecordMissing,
    WorldChunkRecordKindMismatch,
    WorldChunkRoleMissing(&'static str),
    WorldChunkRoleDuplicate(&'static str),
    WorldRoutineContentInvalid,
    WorldStreamingResumeMismatch,
    WorldStreamingMutatedRpg,
    AgentActionMissing,
    AgentCommandRejected,
    InputFrameMissing,
    PresentationAssetMissing,
    PresentationSnapshotMissing,
    AudioAssetMissing,
    RecoveryInvalid,
}

impl Display for ReferenceGameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Identity(error) => write!(formatter, "{error}"),
            Self::Input(error) => write!(formatter, "{error}"),
            Self::PlayerInput(error) => write!(formatter, "{error}"),
            Self::Platform(error) => write!(formatter, "{error}"),
            Self::Physics(error) => write!(formatter, "{error}"),
            Self::Authority(error) => write!(formatter, "{error}"),
            Self::InputContract(error) => write!(formatter, "{error}"),
            Self::InputAdmission(error) => write!(formatter, "{error}"),
            Self::Restore(error) => write!(formatter, "{error}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::Checkpoint(error) => write!(formatter, "{error}"),
            Self::PresentationContract(error) => write!(formatter, "{error}"),
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::AudioSceneContract(error) => write!(formatter, "{error}"),
            Self::AudioExtraction(error) => write!(formatter, "{error}"),
            Self::AudioMix(error) => write!(formatter, "{error}"),
            Self::WorldStreaming(error) => write!(formatter, "{error}"),
            Self::WorldRoutine(error) => write!(formatter, "{error}"),
            Self::WorldStreamingContract(error) => write!(formatter, "{error}"),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::DuplicatePrincipal => formatter.write_str("reference principal is duplicated"),
            Self::CountOverflow => formatter.write_str("reference run count overflow"),
            Self::BodyMissing => formatter.write_str("reference player body is missing"),
            Self::WorldPartitionEmpty => formatter.write_str("reference world partition is empty"),
            Self::WorldChunkRecordMissing => {
                formatter.write_str("reference world chunk record is missing")
            }
            Self::WorldChunkRecordKindMismatch => {
                formatter.write_str("reference world chunk binding has the wrong semantic class")
            }
            Self::WorldChunkRoleMissing(role) => {
                write!(formatter, "reference world {role} chunk role is missing")
            }
            Self::WorldChunkRoleDuplicate(role) => {
                write!(formatter, "reference world {role} chunk role is duplicated")
            }
            Self::WorldRoutineContentInvalid => {
                formatter.write_str("WORLD_ROUTINE_CONTENT_INVALID")
            }
            Self::WorldStreamingResumeMismatch => {
                formatter.write_str("restaged world group changed after restore")
            }
            Self::WorldStreamingMutatedRpg => {
                formatter.write_str("world transition mutated durable RPG state")
            }
            Self::AgentActionMissing => formatter.write_str("reference agent action is missing"),
            Self::AgentCommandRejected => {
                formatter.write_str("reference agent command was rejected")
            }
            Self::InputFrameMissing => {
                formatter.write_str("reference semantic control produced no input frame")
            }
            Self::PresentationAssetMissing => {
                formatter.write_str("reference presentation asset is missing")
            }
            Self::PresentationSnapshotMissing => {
                formatter.write_str("reference presentation snapshot is missing")
            }
            Self::AudioAssetMissing => formatter.write_str("reference audio clip asset is missing"),
            Self::RecoveryInvalid => {
                formatter.write_str("reference live recovery state is invalid")
            }
        }
    }
}

impl Error for ReferenceGameError {}

macro_rules! from_error {
    ($source:ty, $variant:ident) => {
        impl From<$source> for ReferenceGameError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from_error!(next_contracts::canonical::CanonicalError, Canonical);
from_error!(next_contracts::ids::IdentifierError, Identifier);
from_error!(next_contracts::identity::IdentityContractError, Identity);
from_error!(next_contracts::input::InputContractError, Input);
from_error!(next_player::PlayerInputError, PlayerInput);
from_error!(next_contracts::platform::PlatformContractError, Platform);
from_error!(next_contracts::physics::PhysicsContractError, Physics);
from_error!(next_runtime::AuthorityRegistryError, Authority);
from_error!(ReferenceInputError, InputContract);
from_error!(next_runtime::InputAdmissionError, InputAdmission);
from_error!(next_runtime::SnapshotRestoreError, Restore);
from_error!(next_runtime::RuntimeFatalError, Runtime);
from_error!(next_contracts::snapshot::WorldCheckpointError, Checkpoint);
from_error!(
    next_contracts::presentation::PresentationContractError,
    PresentationContract
);
from_error!(next_presentation::PresentationExtractionError, Presentation);
from_error!(
    next_contracts::presentation::audio_scene::AudioSceneContractErrorV1,
    AudioSceneContract
);
from_error!(
    next_presentation::audio_scene::AudioSceneExtractionErrorV1,
    AudioExtraction
);
from_error!(next_presentation::audio_mix::AudioMixErrorV1, AudioMix);
from_error!(next_world::WorldStreamingError, WorldStreaming);
from_error!(next_world::WorldRoutineOwnerError, WorldRoutine);
from_error!(
    next_contracts::world::WorldStreamingContractError,
    WorldStreamingContract
);
from_error!(next_agent::AgentPlannerError, Agent);
