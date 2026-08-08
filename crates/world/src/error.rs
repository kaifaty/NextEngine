use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WorldAssetLoadErrorCodeV1 {
    ContentSource = 1,
    Decode = 2,
    SemanticClass = 3,
    Schema = 4,
    AssetIdentity = 5,
    RecordRevision = 6,
    MissingDependency = 7,
    WorkerPanic = 8,
}

impl WorldAssetLoadErrorCodeV1 {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ContentSource => "WORLD_STREAM_ASSET_CONTENT_SOURCE",
            Self::Decode => "WORLD_STREAM_ASSET_DECODE",
            Self::SemanticClass => "WORLD_STREAM_ASSET_SEMANTIC_CLASS",
            Self::Schema => "WORLD_STREAM_ASSET_SCHEMA",
            Self::AssetIdentity => "WORLD_STREAM_ASSET_IDENTITY",
            Self::RecordRevision => "WORLD_STREAM_ASSET_REVISION",
            Self::MissingDependency => "WORLD_STREAM_ASSET_DEPENDENCY_MISSING",
            Self::WorkerPanic => "WORLD_STREAM_WORKER_PANIC",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldStreamingError {
    Contract(WorldStreamingContractError),
    Project(next_contracts::project::ProjectContractError),
    Neutral(next_contracts::content::NeutralRecordError),
    UnknownChunk,
    ProjectMismatch,
    TransitionAlreadyPending,
    NoPendingTransition,
    InvalidLifecycle,
    RequestCorrupt,
    ResultCorrupt,
    PublicationStale,
    CompletionTickInvalid,
    RequiredAssetUnavailable,
    ObjectIdCollision,
    ChunkDependencyMismatch,
    AssetCountLimit {
        actual: usize,
        limit: usize,
    },
    EncodedBytesLimit {
        actual: usize,
        limit: usize,
    },
    WorkerCountLimit {
        actual: usize,
        limit: usize,
    },
    AssetLoad {
        asset_id: AssetId,
        code: WorldAssetLoadErrorCodeV1,
    },
    Overflow,
}

impl Display for WorldStreamingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(error) => Display::fmt(error, formatter),
            Self::Project(error) => Display::fmt(error, formatter),
            Self::Neutral(error) => Display::fmt(error, formatter),
            Self::UnknownChunk => formatter.write_str("WORLD_STREAM_CHUNK_UNKNOWN"),
            Self::ProjectMismatch => formatter.write_str("WORLD_STREAM_PROJECT_MISMATCH"),
            Self::TransitionAlreadyPending => {
                formatter.write_str("WORLD_STREAM_TRANSITION_PENDING")
            }
            Self::NoPendingTransition => formatter.write_str("WORLD_STREAM_TRANSITION_MISSING"),
            Self::InvalidLifecycle => formatter.write_str("WORLD_STREAM_LIFECYCLE_INVALID"),
            Self::RequestCorrupt => formatter.write_str("WORLD_STREAM_REQUEST_CORRUPT"),
            Self::ResultCorrupt => formatter.write_str("WORLD_STREAM_RESULT_CORRUPT"),
            Self::PublicationStale => formatter.write_str("WORLD_STREAM_PUBLICATION_STALE"),
            Self::CompletionTickInvalid => {
                formatter.write_str("WORLD_STREAM_COMPLETION_TICK_INVALID")
            }
            Self::RequiredAssetUnavailable => {
                formatter.write_str("WORLD_STREAM_REQUIRED_UNAVAILABLE")
            }
            Self::ObjectIdCollision => formatter.write_str("WORLD_STREAM_OBJECT_ID_COLLISION"),
            Self::ChunkDependencyMismatch => {
                formatter.write_str("WORLD_STREAM_CHUNK_DEPENDENCIES_INVALID")
            }
            Self::AssetCountLimit { actual, limit } => {
                write!(formatter, "WORLD_STREAM_ASSET_COUNT_LIMIT:{actual}:{limit}")
            }
            Self::EncodedBytesLimit { actual, limit } => {
                write!(
                    formatter,
                    "WORLD_STREAM_ENCODED_BYTES_LIMIT:{actual}:{limit}"
                )
            }
            Self::WorkerCountLimit { actual, limit } => {
                write!(
                    formatter,
                    "WORLD_STREAM_WORKER_COUNT_LIMIT:{actual}:{limit}"
                )
            }
            Self::AssetLoad { asset_id, code } => {
                write!(formatter, "{}:{asset_id:?}", code.as_str())
            }
            Self::Overflow => formatter.write_str("WORLD_STREAM_OVERFLOW"),
        }
    }
}

impl Error for WorldStreamingError {}

impl From<WorldStreamingContractError> for WorldStreamingError {
    fn from(value: WorldStreamingContractError) -> Self {
        Self::Contract(value)
    }
}

impl From<next_contracts::project::ProjectContractError> for WorldStreamingError {
    fn from(value: next_contracts::project::ProjectContractError) -> Self {
        Self::Project(value)
    }
}

impl From<next_contracts::content::NeutralRecordError> for WorldStreamingError {
    fn from(value: next_contracts::content::NeutralRecordError) -> Self {
        Self::Neutral(value)
    }
}
