use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use next_contracts::canonical::CanonicalError;
use next_contracts::persistence::{ManifestCodecError, ManifestValidationError};
use next_contracts::physics::PhysicsContractError;
use next_contracts::rpg::RpgContractErrorV1;
use next_contracts::snapshot::{SnapshotDecodeError, WorldCheckpointError};
use next_contracts::world::WorldStreamingContractError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreservedFile {
    pub relative_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectedGeneration {
    pub slot: u8,
    pub stable_code: &'static str,
    pub original_files: Vec<PreservedFile>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum SaveStoreError {
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    Canonicalization(CanonicalError),
    ManifestValidation(ManifestValidationError),
    ManifestCodec(ManifestCodecError),
    SnapshotDecode(SnapshotDecodeError),
    RpgSnapshotDecode(RpgContractErrorV1),
    PhysicsSnapshotDecode(PhysicsContractError),
    WorldStreamingSnapshotDecode(WorldStreamingContractError),
    WorldCheckpoint(WorldCheckpointError),
    InvalidImage(&'static str),
    InvalidStaging(&'static str),
    GenerationExhausted,
    InjectedFault,
}

impl SaveStoreError {
    pub(super) fn io(
        operation: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io {
            operation,
            path: path.into(),
            source,
        }
    }

    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "SAVE_IO_FAILED",
            Self::Canonicalization(_) => "SAVE_CANONICALIZATION_FAILED",
            Self::ManifestValidation(_) => "SAVE_MANIFEST_INVALID",
            Self::ManifestCodec(_) => "SAVE_MANIFEST_CODEC_FAILED",
            Self::SnapshotDecode(_) => "SAVE_SNAPSHOT_INVALID",
            Self::RpgSnapshotDecode(error) => error.stable_code(),
            Self::PhysicsSnapshotDecode(_) => "SAVE_PHYSICS_SNAPSHOT_INVALID",
            Self::WorldStreamingSnapshotDecode(error) => match error {
                WorldStreamingContractError::UnsupportedVersion(_) => {
                    "WORLD_STREAM_SCHEMA_UNSUPPORTED"
                }
                _ => "SAVE_WORLD_STREAMING_SNAPSHOT_INVALID",
            },
            Self::WorldCheckpoint(error) => error.stable_code(),
            Self::InvalidImage(code) | Self::InvalidStaging(code) => code,
            Self::GenerationExhausted => "SAVE_GENERATION_EXHAUSTED",
            Self::InjectedFault => "SAVE_FAULT_INJECTED",
        }
    }
}

impl Display for SaveStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(formatter, "{operation} at {}: {source}", path.display()),
            Self::Canonicalization(error) => {
                write!(formatter, "save canonicalization failed: {error}")
            }
            Self::ManifestValidation(error) => {
                write!(formatter, "save manifest is invalid: {error}")
            }
            Self::ManifestCodec(error) => write!(formatter, "save manifest codec failed: {error}"),
            Self::SnapshotDecode(error) => write!(formatter, "save snapshot is invalid: {error}"),
            Self::RpgSnapshotDecode(error) => {
                write!(formatter, "save RPG snapshot is invalid: {error}")
            }
            Self::PhysicsSnapshotDecode(error) => {
                write!(formatter, "save physics snapshot is invalid: {error}")
            }
            Self::WorldStreamingSnapshotDecode(error) => {
                write!(
                    formatter,
                    "save world streaming snapshot is invalid: {error}"
                )
            }
            Self::WorldCheckpoint(error) => {
                write!(formatter, "save world checkpoint is invalid: {error}")
            }
            Self::InvalidImage(code) | Self::InvalidStaging(code) => formatter.write_str(code),
            Self::GenerationExhausted => formatter.write_str("SAVE_GENERATION_EXHAUSTED"),
            Self::InjectedFault => formatter.write_str("SAVE_FAULT_INJECTED"),
        }
    }
}

impl Error for SaveStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Canonicalization(error) => Some(error),
            Self::ManifestValidation(error) => Some(error),
            Self::ManifestCodec(error) => Some(error),
            Self::SnapshotDecode(error) => Some(error),
            Self::RpgSnapshotDecode(error) => Some(error),
            Self::PhysicsSnapshotDecode(error) => Some(error),
            Self::WorldStreamingSnapshotDecode(error) => Some(error),
            Self::WorldCheckpoint(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CanonicalError> for SaveStoreError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<ManifestValidationError> for SaveStoreError {
    fn from(error: ManifestValidationError) -> Self {
        Self::ManifestValidation(error)
    }
}

impl From<ManifestCodecError> for SaveStoreError {
    fn from(error: ManifestCodecError) -> Self {
        Self::ManifestCodec(error)
    }
}

impl From<SnapshotDecodeError> for SaveStoreError {
    fn from(error: SnapshotDecodeError) -> Self {
        Self::SnapshotDecode(error)
    }
}

impl From<RpgContractErrorV1> for SaveStoreError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::RpgSnapshotDecode(error)
    }
}

impl From<PhysicsContractError> for SaveStoreError {
    fn from(error: PhysicsContractError) -> Self {
        Self::PhysicsSnapshotDecode(error)
    }
}

impl From<WorldStreamingContractError> for SaveStoreError {
    fn from(error: WorldStreamingContractError) -> Self {
        Self::WorldStreamingSnapshotDecode(error)
    }
}

impl From<WorldCheckpointError> for SaveStoreError {
    fn from(error: WorldCheckpointError) -> Self {
        Self::WorldCheckpoint(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SaveLoadError {
    NoValidGeneration { rejected: Vec<RejectedGeneration> },
}

impl Display for SaveLoadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoValidGeneration { rejected } => {
                write!(
                    formatter,
                    "no valid save generation; {} rejected",
                    rejected.len()
                )
            }
        }
    }
}

impl Error for SaveLoadError {}
