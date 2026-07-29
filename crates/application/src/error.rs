use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ApplicationError {
    Identifier(next_contracts::ids::IdentifierError),
    SessionContract(next_contracts::session::SessionContractError),
    SessionMachine(next_runtime::SessionMachineError),
    SessionStore(next_assets::SessionStoreError),
    ContentStore(next_assets::ContentStoreError),
    SaveStore(next_assets::SaveStoreError),
    SaveLoad(next_assets::SaveLoadError),
    ProjectCook(next_project::ProjectCookError),
    ProjectActivation(next_project::ProjectActivationError),
    ReferenceGame(next_reference_game::ReferenceGameError),
    Checkpoint(next_contracts::snapshot::WorldCheckpointError),
    WorldStreaming(next_contracts::world::WorldStreamingContractError),
    Presentation(next_presentation::PresentationExtractionError),
    Manifest(next_contracts::persistence::ManifestCodecError),
    Canonical(next_contracts::canonical::CanonicalError),
    CanonicalDecode(next_contracts::canonical::CanonicalDecodeError),
    Io(std::io::Error),
    ProjectLockMismatch,
    ProjectTargetForbidden,
    SessionAlreadyLive,
    NoRunOutcome,
    CloseIdentityCollision,
    CloseStateInvalid,
    CloseJournalInvalid,
    FinalSaveFailed,
    RecoveryIncompatible,
    TerminalReceiptMissing,
    DurableSnapshotInvalid,
    StateRootUnavailable,
}

impl ApplicationError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::ProjectLockMismatch => "PROJECT_LOCK_MISMATCH",
            Self::ProjectTargetForbidden => "PLATFORM_FORBIDDEN_PRESENTATION_TARGET",
            Self::SessionAlreadyLive => "SESSION_LIVE_REGISTRY_CONFLICT",
            Self::CloseIdentityCollision => "SESSION_FINAL_SAVE_IDENTITY_COLLISION",
            Self::CloseStateInvalid | Self::CloseJournalInvalid => "SESSION_TRANSITION_INVALID",
            Self::FinalSaveFailed => "SESSION_FINAL_SAVE_FAILED",
            Self::RecoveryIncompatible => "SESSION_RECOVERY_INCOMPATIBLE",
            Self::TerminalReceiptMissing => "SESSION_TERMINAL_RECEIPT_MISSING",
            Self::StateRootUnavailable => "SESSION_STORAGE_UNAVAILABLE",
            Self::SessionMachine(error) => error.diagnostic_code(),
            Self::SessionStore(error) => error.diagnostic_code(),
            Self::SessionContract(error) => error.diagnostic_code(),
            Self::ContentStore(_)
            | Self::SaveStore(_)
            | Self::SaveLoad(_)
            | Self::Io(_)
            | Self::DurableSnapshotInvalid => "SESSION_STORAGE_UNAVAILABLE",
            Self::Identifier(_)
            | Self::ProjectCook(_)
            | Self::ProjectActivation(_)
            | Self::ReferenceGame(_)
            | Self::Checkpoint(_)
            | Self::WorldStreaming(_)
            | Self::Presentation(_)
            | Self::Manifest(_)
            | Self::Canonical(_)
            | Self::CanonicalDecode(_)
            | Self::NoRunOutcome => "SESSION_RUNTIME_FAILED",
        }
    }
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::SessionContract(error) => write!(formatter, "{error}"),
            Self::SessionMachine(error) => write!(formatter, "{error}"),
            Self::SessionStore(error) => write!(formatter, "{error}"),
            Self::ContentStore(error) => write!(formatter, "{error}"),
            Self::SaveStore(error) => write!(formatter, "{error}"),
            Self::SaveLoad(error) => write!(formatter, "{error}"),
            Self::ProjectCook(error) => write!(formatter, "{error}"),
            Self::ProjectActivation(error) => write!(formatter, "{error}"),
            Self::ReferenceGame(error) => write!(formatter, "{error}"),
            Self::Checkpoint(error) => write!(formatter, "{error}"),
            Self::WorldStreaming(error) => write!(formatter, "{error}"),
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::Manifest(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::CanonicalDecode(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::ProjectLockMismatch => formatter.write_str("project lock does not match"),
            Self::ProjectTargetForbidden => {
                formatter.write_str("presentation target is not allowed by project")
            }
            Self::SessionAlreadyLive => formatter.write_str("an application session is live"),
            Self::NoRunOutcome => formatter.write_str("reference game has not run"),
            Self::CloseIdentityCollision => formatter.write_str("close request identity collides"),
            Self::CloseStateInvalid => formatter.write_str("close state is invalid"),
            Self::CloseJournalInvalid => formatter.write_str("close journal is invalid"),
            Self::FinalSaveFailed => formatter.write_str("final save failed"),
            Self::RecoveryIncompatible => formatter.write_str("session recovery is incompatible"),
            Self::TerminalReceiptMissing => formatter.write_str("terminal receipt is missing"),
            Self::DurableSnapshotInvalid => formatter.write_str("durable snapshot is invalid"),
            Self::StateRootUnavailable => formatter.write_str("user state root is unavailable"),
        }
    }
}

impl Error for ApplicationError {}

macro_rules! from_error {
    ($source:ty, $variant:ident) => {
        impl From<$source> for ApplicationError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from_error!(next_contracts::ids::IdentifierError, Identifier);
from_error!(
    next_contracts::session::SessionContractError,
    SessionContract
);
from_error!(next_runtime::SessionMachineError, SessionMachine);
from_error!(next_assets::SessionStoreError, SessionStore);
from_error!(next_assets::ContentStoreError, ContentStore);
from_error!(next_assets::SaveStoreError, SaveStore);
from_error!(next_assets::SaveLoadError, SaveLoad);
from_error!(next_project::ProjectCookError, ProjectCook);
from_error!(next_project::ProjectActivationError, ProjectActivation);
from_error!(next_reference_game::ReferenceGameError, ReferenceGame);
from_error!(next_contracts::snapshot::WorldCheckpointError, Checkpoint);
from_error!(
    next_contracts::world::WorldStreamingContractError,
    WorldStreaming
);
from_error!(next_presentation::PresentationExtractionError, Presentation);
from_error!(next_contracts::persistence::ManifestCodecError, Manifest);
from_error!(next_contracts::canonical::CanonicalError, Canonical);
from_error!(
    next_contracts::canonical::CanonicalDecodeError,
    CanonicalDecode
);
from_error!(std::io::Error, Io);
