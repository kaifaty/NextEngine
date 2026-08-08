use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeError, CanonicalError};
use crate::ids::IdentifierError;
use crate::persistence::ManifestCodecError;

mod close;
mod codec;
mod lifecycle;
mod types;

pub use crate::platform::PresentationTargetKindV1;
pub use close::*;
pub use lifecycle::*;
pub use types::*;

pub const APPLICATION_SESSION_SCHEMA_VERSION: u32 = 2;
pub const APPLICATION_SESSION_MANIFEST_FORMAT_V2: &str =
    "nextengine.application-session-manifest.v2";

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SessionContractError {
    Manifest(ManifestCodecError),
    Canonical(CanonicalError),
    CanonicalDecode(CanonicalDecodeError),
    Identifier(IdentifierError),
    UnsupportedVersion,
    HashMismatch,
    IdentityMismatch,
    InvalidTransition,
    InvalidRootTarget,
    InvalidStateFields,
    InvalidCloseState,
    DuplicateIdentity,
    NonCanonicalOrder,
    UnknownClosedValue,
}

impl SessionContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::InvalidRootTarget => "PLATFORM_FORBIDDEN_PRESENTATION_TARGET",
            Self::InvalidTransition | Self::InvalidStateFields | Self::InvalidCloseState => {
                "SESSION_TRANSITION_INVALID"
            }
            Self::IdentityMismatch => "SESSION_REQUEST_IDENTITY_COLLISION",
            Self::HashMismatch
            | Self::Manifest(_)
            | Self::Canonical(_)
            | Self::CanonicalDecode(_)
            | Self::Identifier(_)
            | Self::UnsupportedVersion
            | Self::DuplicateIdentity
            | Self::NonCanonicalOrder
            | Self::UnknownClosedValue => "SESSION_MANIFEST_INVALID",
        }
    }
}

impl Display for SessionContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Manifest(_) => "session manifest encoding is invalid",
            Self::Canonical(_) => "session canonical encoding is invalid",
            Self::CanonicalDecode(_) => "session canonical bytes are invalid",
            Self::Identifier(_) => "session identifier is invalid",
            Self::UnsupportedVersion => "session schema version is unsupported",
            Self::HashMismatch => "session canonical hash does not match",
            Self::IdentityMismatch => "session request identity collides",
            Self::InvalidTransition => "session transition is not allowed",
            Self::InvalidRootTarget => "presentation target is forbidden for composition root",
            Self::InvalidStateFields => "session state fields are inconsistent",
            Self::InvalidCloseState => "close request state is invalid",
            Self::DuplicateIdentity => "session identity is duplicated",
            Self::NonCanonicalOrder => "session collection is not in canonical order",
            Self::UnknownClosedValue => "unknown closed session value",
        })
    }
}

impl Error for SessionContractError {}

impl From<ManifestCodecError> for SessionContractError {
    fn from(value: ManifestCodecError) -> Self {
        Self::Manifest(value)
    }
}

impl From<CanonicalError> for SessionContractError {
    fn from(value: CanonicalError) -> Self {
        Self::Canonical(value)
    }
}

impl From<CanonicalDecodeError> for SessionContractError {
    fn from(value: CanonicalDecodeError) -> Self {
        Self::CanonicalDecode(value)
    }
}

impl From<IdentifierError> for SessionContractError {
    fn from(value: IdentifierError) -> Self {
        Self::Identifier(value)
    }
}

#[cfg(test)]
mod tests;
