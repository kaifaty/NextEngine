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
    Physics(next_contracts::physics::PhysicsContractError),
    Authority(next_runtime::AuthorityRegistryError),
    InputContract(ReferenceInputError),
    InputAdmission(next_runtime::InputAdmissionError),
    Restore(next_runtime::SnapshotRestoreError),
    Runtime(next_runtime::RuntimeFatalError),
    WorldStreaming(next_world::WorldStreamingError),
    WorldStreamingContract(next_contracts::world::WorldStreamingContractError),
    Agent(next_agent::AgentPlannerError),
    DuplicatePrincipal,
    CountOverflow,
    BodyMissing,
    WorldPartitionEmpty,
    WorldStreamingResumeMismatch,
    WorldStreamingMutatedRpg,
    AgentActionMissing,
    AgentCommandRejected,
    PresentationAssetMissing,
}

impl Display for ReferenceGameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "{error}"),
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Identity(error) => write!(formatter, "{error}"),
            Self::Physics(error) => write!(formatter, "{error}"),
            Self::Authority(error) => write!(formatter, "{error}"),
            Self::InputContract(error) => write!(formatter, "{error}"),
            Self::InputAdmission(error) => write!(formatter, "{error}"),
            Self::Restore(error) => write!(formatter, "{error}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::WorldStreaming(error) => write!(formatter, "{error}"),
            Self::WorldStreamingContract(error) => write!(formatter, "{error}"),
            Self::Agent(error) => write!(formatter, "{error}"),
            Self::DuplicatePrincipal => formatter.write_str("reference principal is duplicated"),
            Self::CountOverflow => formatter.write_str("reference run count overflow"),
            Self::BodyMissing => formatter.write_str("reference player body is missing"),
            Self::WorldPartitionEmpty => formatter.write_str("reference world requires two chunks"),
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
            Self::PresentationAssetMissing => {
                formatter.write_str("reference presentation asset is missing")
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
from_error!(next_contracts::physics::PhysicsContractError, Physics);
from_error!(next_runtime::AuthorityRegistryError, Authority);
from_error!(ReferenceInputError, InputContract);
from_error!(next_runtime::InputAdmissionError, InputAdmission);
from_error!(next_runtime::SnapshotRestoreError, Restore);
from_error!(next_runtime::RuntimeFatalError, Runtime);
from_error!(next_world::WorldStreamingError, WorldStreaming);
from_error!(
    next_contracts::world::WorldStreamingContractError,
    WorldStreamingContract
);
from_error!(next_agent::AgentPlannerError, Agent);
