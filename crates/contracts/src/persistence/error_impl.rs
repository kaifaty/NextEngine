use std::error::Error;
use std::fmt::{Display, Formatter};

use super::*;

impl Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "manifest identifier is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "manifest canonicalization failed: {error}")
            }
            Self::Snapshot(error) => write!(formatter, "manifest snapshot is invalid: {error}"),
            Self::Command(error) => write!(formatter, "manifest command is invalid: {error}"),
            Self::InvalidTickSettings => formatter.write_str("manifest tick settings are invalid"),
            Self::HashBindingsNotStrictlySorted => {
                formatter.write_str("manifest hash bindings are not strictly sorted")
            }
            Self::SchemaBindingsNotStrictlySorted => {
                formatter.write_str("manifest schema bindings are not strictly sorted")
            }
            Self::SegmentsNotStrictlySorted => {
                formatter.write_str("manifest segments are not strictly sorted")
            }
            Self::CapabilitiesNotStrictlySorted => {
                formatter.write_str("manifest capabilities are not strictly sorted")
            }
            Self::AuthorityNotStrictlySorted => {
                formatter.write_str("manifest authority grants are not strictly sorted")
            }
            Self::MissingRequiredSegment => formatter.write_str("manifest has no state segment"),
            Self::CompatibilityMismatch => {
                formatter.write_str("manifest physical/training compatibility does not match")
            }
            Self::UnsupportedSaveVersion(version) => {
                write!(formatter, "unsupported save manifest version {version}")
            }
            Self::UnsupportedReplayVersion(version) => {
                write!(formatter, "unsupported replay manifest version {version}")
            }
            Self::ReplayCommandIdMismatch => {
                formatter.write_str("replay command id does not match canonical command bytes")
            }
            Self::ComparePointCountMismatch => {
                formatter.write_str("replay tick and compare-point counts differ")
            }
            Self::ReplayTickSequenceMismatch => {
                formatter.write_str("replay ticks or compare points are not contiguous")
            }
            Self::ReplayTickExhausted => formatter.write_str("replay tick sequence overflowed"),
            Self::ReplayInitialSegmentsInvalid => {
                formatter.write_str("replay initial owner segments are incomplete or invalid")
            }
            Self::ReplayOwnerSegmentsInvalid => {
                formatter.write_str("replay compare-point owner segments are invalid")
            }
            Self::ReplayWorldStreamingInputInvalid => {
                formatter.write_str("replay world-streaming input is invalid")
            }
            Self::ReplayBatchMismatch => {
                formatter.write_str("replay closed batch closure does not match")
            }
            Self::ReplayQueryFactsInvalid => {
                formatter.write_str("replay targeting/query facts do not form an exact closure")
            }
            Self::WorldCheckpoint(error) => {
                write!(formatter, "replay checkpoint is invalid: {error}")
            }
            Self::RpgV2(error) => write!(formatter, "replay RPG segment is invalid: {error}"),
            Self::Physics(error) => write!(formatter, "replay physics segment is invalid: {error}"),
            Self::Input(error) => write!(formatter, "replay ingress batch is invalid: {error}"),
            Self::Targeting(error) => {
                write!(formatter, "replay targeting fact is invalid: {error}")
            }
            Self::WorldStreaming(error) => {
                write!(
                    formatter,
                    "replay world-streaming segment is invalid: {error}"
                )
            }
            Self::WorldRoutine(error) => {
                write!(formatter, "replay world-routine value is invalid: {error}")
            }
            Self::WorldPopulation(error) => {
                write!(
                    formatter,
                    "replay world-population value is invalid: {error}"
                )
            }
        }
    }
}

impl Error for ManifestValidationError {}

impl From<crate::ids::IdentifierError> for ManifestValidationError {
    fn from(error: crate::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<CanonicalError> for ManifestValidationError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<crate::snapshot::SnapshotDecodeError> for ManifestValidationError {
    fn from(error: crate::snapshot::SnapshotDecodeError) -> Self {
        Self::Snapshot(error)
    }
}

impl From<CommandDecodeError> for ManifestValidationError {
    fn from(error: CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

impl From<crate::snapshot::WorldCheckpointError> for ManifestValidationError {
    fn from(error: crate::snapshot::WorldCheckpointError) -> Self {
        Self::WorldCheckpoint(error)
    }
}

impl From<crate::rpg::RpgContractErrorV1> for ManifestValidationError {
    fn from(error: crate::rpg::RpgContractErrorV1) -> Self {
        Self::RpgV2(error)
    }
}

impl From<crate::physics::PhysicsContractError> for ManifestValidationError {
    fn from(error: crate::physics::PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<crate::input::InputContractError> for ManifestValidationError {
    fn from(error: crate::input::InputContractError) -> Self {
        Self::Input(error)
    }
}

impl From<TargetingContractError> for ManifestValidationError {
    fn from(error: TargetingContractError) -> Self {
        Self::Targeting(error)
    }
}

impl From<crate::world::WorldStreamingContractError> for ManifestValidationError {
    fn from(error: crate::world::WorldStreamingContractError) -> Self {
        Self::WorldStreaming(error)
    }
}

impl From<crate::world_routine::WorldRoutineContractError> for ManifestValidationError {
    fn from(error: crate::world_routine::WorldRoutineContractError) -> Self {
        Self::WorldRoutine(error)
    }
}

impl From<crate::world_population::WorldPopulationContractError> for ManifestValidationError {
    fn from(error: crate::world_population::WorldPopulationContractError) -> Self {
        Self::WorldPopulation(error)
    }
}
