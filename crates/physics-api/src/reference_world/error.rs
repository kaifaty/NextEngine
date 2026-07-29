use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::PhysicsContractError;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferencePhysicsError {
    Contract(PhysicsContractError),
    Canonical(next_contracts::CanonicalError),
    UnsupportedProfile,
    NonIntegralProfile,
    SnapshotMismatch,
    StepInputMismatch,
    BodyMissing,
    NumericOverflow,
    ContactCapacityExceeded,
    SnapshotPenetrating,
    BackendUnavailable,
    BackendVersionMismatch,
    BackendCapacityExceeded,
    BackendFailure,
    BackendHitMismatch,
    BackendDistanceMismatch,
    BackendFeatureMismatch,
    BackendNormalMismatch,
}

impl ReferencePhysicsError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::Canonical(_) => "PHYSICS_CANONICALIZATION_FAILED",
            Self::UnsupportedProfile => "PHYS_REFERENCE_PROFILE_UNSUPPORTED",
            Self::NonIntegralProfile => "PHYSICS_PROFILE_MISMATCH",
            Self::SnapshotMismatch => "PHYS_SNAPSHOT_INCOMPATIBLE",
            Self::StepInputMismatch => "PHYS_STEP_INPUT_MISMATCH",
            Self::BodyMissing => "PHYSICAL_TARGET_UNBOUND",
            Self::NumericOverflow => "PHYSICS_NUMERIC_OVERFLOW",
            Self::ContactCapacityExceeded => "PHYS_CONTACT_CAPACITY_EXCEEDED",
            Self::SnapshotPenetrating => "PHYS_SNAPSHOT_PENETRATING",
            Self::BackendUnavailable => "PHYS_BACKEND_UNAVAILABLE",
            Self::BackendVersionMismatch => "PHYS_BACKEND_VERSION_MISMATCH",
            Self::BackendCapacityExceeded => "PHYS_BACKEND_CAPACITY_EXCEEDED",
            Self::BackendFailure => "PHYS_BACKEND_FAILURE",
            Self::BackendHitMismatch => "PHYS_BACKEND_HIT_MISMATCH",
            Self::BackendDistanceMismatch => "PHYS_BACKEND_DISTANCE_MISMATCH",
            Self::BackendFeatureMismatch => "PHYS_BACKEND_FEATURE_MISMATCH",
            Self::BackendNormalMismatch => "PHYS_BACKEND_NORMAL_MISMATCH",
        }
    }
}

impl Display for ReferencePhysicsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for ReferencePhysicsError {}

impl From<PhysicsContractError> for ReferencePhysicsError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<next_contracts::CanonicalError> for ReferencePhysicsError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}
