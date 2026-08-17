use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeError, CanonicalError};
use crate::cognition::CognitionContractError;
use crate::physical_animation::PhysicalAnimationContractErrorV1;
use crate::physics::PhysicsContractError;
use crate::rpg::RpgContractErrorV1;
use crate::world_activity::WorldActivityContractError;
use crate::world_population::WorldPopulationContractError;
use crate::world_routine::WorldRoutineContractError;

use super::principal::PrincipalDecodeError;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandDecodeError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Principal(PrincipalDecodeError),
    Rpg(RpgContractErrorV1),
    Physics(PhysicsContractError),
    PhysicalAnimation(PhysicalAnimationContractErrorV1),
    WorldRoutine(WorldRoutineContractError),
    WorldPopulation(WorldPopulationContractError),
    WorldActivity(WorldActivityContractError),
    AgentCognition(CognitionContractError),
    Identifier(crate::ids::IdentifierError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    FieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedBodySchemaVersion(u16),
    UnsupportedPayloadSchemaVersion(u32),
    UnknownPayloadSchema(String),
    InvalidPhase(u8),
    InvalidNoopPayload,
    InvalidOptional,
    InvalidNestedType {
        expected: u8,
        actual: u8,
    },
    TooManySetItems {
        actual: usize,
        limit: usize,
    },
    SetNotStrictlySorted,
    ConflictingPreconditions,
    NonCanonicalEncoding,
}

impl Display for CommandDecodeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "command encoding is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "command canonicalization failed: {error}")
            }
            Self::Principal(error) => write!(formatter, "command principal is invalid: {error}"),
            Self::Rpg(error) => write!(formatter, "command RPG payload is invalid: {error}"),
            Self::Physics(error) => {
                write!(formatter, "command physical payload is invalid: {error}")
            }
            Self::PhysicalAnimation(error) => {
                write!(formatter, "command root-motion payload is invalid: {error}")
            }
            Self::WorldRoutine(error) => {
                write!(
                    formatter,
                    "command world routine payload is invalid: {error}"
                )
            }
            Self::WorldPopulation(error) => {
                write!(
                    formatter,
                    "command world population payload is invalid: {error}"
                )
            }
            Self::WorldActivity(error) => {
                write!(
                    formatter,
                    "command world activity payload is invalid: {error}"
                )
            }
            Self::AgentCognition(error) => {
                write!(
                    formatter,
                    "command agent cognition payload is invalid: {error}"
                )
            }
            Self::Identifier(error) => write!(formatter, "command identifier is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("command body envelope does not match V2"),
            Self::UnknownField(field_id) => write!(formatter, "unknown command field {field_id}"),
            Self::MissingField(field_id) => write!(formatter, "missing command field {field_id}"),
            Self::FieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "command field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::FieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "command field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedBodySchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported command body schema version {version}"
                )
            }
            Self::UnsupportedPayloadSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported command payload schema version {version}"
                )
            }
            Self::UnknownPayloadSchema(schema) => {
                write!(formatter, "unknown command payload schema {schema}")
            }
            Self::InvalidPhase(phase) => write!(formatter, "invalid command phase {phase}"),
            Self::InvalidNoopPayload => formatter.write_str("noop command payload must be empty"),
            Self::InvalidOptional => formatter.write_str("invalid canonical optional value"),
            Self::InvalidNestedType { expected, actual } => write!(
                formatter,
                "nested value has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::TooManySetItems { actual, limit } => {
                write!(
                    formatter,
                    "canonical set has {actual} items; limit is {limit}"
                )
            }
            Self::SetNotStrictlySorted => {
                formatter.write_str("canonical set is not strictly sorted")
            }
            Self::ConflictingPreconditions => {
                formatter.write_str("preconditions have the same key with different constraints")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("decoded command does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for CommandDecodeError {}

impl From<CanonicalDecodeError> for CommandDecodeError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for CommandDecodeError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<PrincipalDecodeError> for CommandDecodeError {
    fn from(error: PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<RpgContractErrorV1> for CommandDecodeError {
    fn from(error: RpgContractErrorV1) -> Self {
        Self::Rpg(error)
    }
}

impl From<PhysicsContractError> for CommandDecodeError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<PhysicalAnimationContractErrorV1> for CommandDecodeError {
    fn from(error: PhysicalAnimationContractErrorV1) -> Self {
        Self::PhysicalAnimation(error)
    }
}

impl From<WorldRoutineContractError> for CommandDecodeError {
    fn from(error: WorldRoutineContractError) -> Self {
        Self::WorldRoutine(error)
    }
}

impl From<WorldPopulationContractError> for CommandDecodeError {
    fn from(error: WorldPopulationContractError) -> Self {
        Self::WorldPopulation(error)
    }
}

impl From<WorldActivityContractError> for CommandDecodeError {
    fn from(error: WorldActivityContractError) -> Self {
        Self::WorldActivity(error)
    }
}

impl From<CognitionContractError> for CommandDecodeError {
    fn from(error: CognitionContractError) -> Self {
        Self::AgentCognition(error)
    }
}

impl From<crate::ids::IdentifierError> for CommandDecodeError {
    fn from(error: crate::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}
