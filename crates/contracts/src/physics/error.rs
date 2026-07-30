use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeError, CanonicalError};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicsContractError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identifier(crate::ids::IdentifierError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType,
    FieldLength,
    UnknownTag(u8),
    UnsupportedVersion(u32),
    InvalidProfile,
    InvalidRotation,
    DirectionOutOfProfile,
    NonCanonicalOrder,
    DuplicateKey,
    ProfileMismatch,
    UnsupportedPhysicalState,
    NonCanonicalEncoding,
    InvalidDescriptor,
    ReferenceInvalid,
    LimitExceeded,
    ContactIdentityMismatch,
    ReferenceProfileUnsupported,
    InvalidQuery,
    QueryCapacityExceeded,
    QueryUnsupported,
    SnapshotSelectorMismatch,
}

impl PhysicsContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion(_) => "UNSUPPORTED_PHYSICS_VERSION",
            Self::ProfileMismatch => "PHYSICS_PROFILE_MISMATCH",
            Self::DirectionOutOfProfile => "INPUT_VALUE_OUT_OF_PROFILE",
            Self::InvalidDescriptor => "PHYS_DESCRIPTOR_INVALID",
            Self::ReferenceInvalid => "PHYS_REFERENCE_INVALID",
            Self::LimitExceeded => "PHYS_LIMIT_EXCEEDED",
            Self::ContactIdentityMismatch => "PHYS_CONTACT_IDENTITY_INVALID",
            Self::ReferenceProfileUnsupported => "PHYS_REFERENCE_PROFILE_UNSUPPORTED",
            Self::InvalidQuery => "PHYS_QUERY_INVALID",
            Self::QueryCapacityExceeded => "PHYS_QUERY_CAPACITY_EXCEEDED",
            Self::QueryUnsupported => "PHYS_QUERY_UNSUPPORTED",
            Self::SnapshotSelectorMismatch => "PHYS_QUERY_SNAPSHOT_MISMATCH",
            _ => "PHYSICS_CONTRACT_INVALID",
        }
    }
}

impl Display for PhysicsContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "physics encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "physics canonicalization failed: {error}")
            }
            Self::Identifier(error) => write!(formatter, "physics identifier is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("physics envelope does not match"),
            Self::UnknownField(id) => write!(formatter, "unknown physics field {id}"),
            Self::MissingField(id) => write!(formatter, "missing physics field {id}"),
            Self::FieldType => formatter.write_str("physics field has the wrong type"),
            Self::FieldLength => formatter.write_str("physics field has the wrong length"),
            Self::UnknownTag(tag) => write!(formatter, "unknown physics tag {tag}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported physics version {version}")
            }
            Self::InvalidProfile => formatter.write_str("physics profile is invalid"),
            Self::InvalidRotation => {
                formatter.write_str("physics rotation is not exact Q1.30 unit")
            }
            Self::DirectionOutOfProfile => {
                formatter.write_str("locomotion direction is outside the axial profile")
            }
            Self::NonCanonicalOrder => formatter.write_str("physics collection order is invalid"),
            Self::DuplicateKey => formatter.write_str("physics map contains a duplicate key"),
            Self::ProfileMismatch => formatter.write_str("physics profile closure does not match"),
            Self::UnsupportedPhysicalState => {
                formatter.write_str("physical state is outside the capsule reference slice")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("physics value does not re-encode byte-exactly")
            }
            Self::InvalidDescriptor => formatter.write_str("physics descriptor is invalid"),
            Self::ReferenceInvalid => formatter.write_str("physics reference is invalid"),
            Self::LimitExceeded => formatter.write_str("physics limit is exceeded"),
            Self::ContactIdentityMismatch => {
                formatter.write_str("physics contact identity does not match its participants")
            }
            Self::ReferenceProfileUnsupported => {
                formatter.write_str("physics descriptor is outside the reference profile")
            }
            Self::InvalidQuery => formatter.write_str("physics query is invalid"),
            Self::QueryCapacityExceeded => {
                formatter.write_str("physics query capacity is exceeded")
            }
            Self::QueryUnsupported => {
                formatter.write_str("physics query is unsupported by this implementation")
            }
            Self::SnapshotSelectorMismatch => {
                formatter.write_str("physics query snapshot selector does not match")
            }
        }
    }
}

impl Error for PhysicsContractError {}

impl From<CanonicalDecodeError> for PhysicsContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for PhysicsContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<crate::ids::IdentifierError> for PhysicsContractError {
    fn from(error: crate::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::input::InputContractError> for PhysicsContractError {
    fn from(error: crate::input::InputContractError) -> Self {
        match error {
            crate::input::InputContractError::Canonical(error) => Self::Canonical(error),
            crate::input::InputContractError::Canonicalization(error) => {
                Self::Canonicalization(error)
            }
            _ => Self::InvalidProfile,
        }
    }
}
