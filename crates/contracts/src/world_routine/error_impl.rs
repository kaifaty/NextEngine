use std::error::Error;
use std::fmt::{Display, Formatter};

use super::*;

impl Display for WorldRoutineContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => {
                write!(formatter, "world routine canonicalization failed: {error}")
            }
            Self::Decode(error) => write!(formatter, "world routine encoding is invalid: {error}"),
            Self::Identifier(error) => {
                write!(formatter, "world routine identifier is invalid: {error}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported world routine version {version}")
            }
            Self::UnknownActivity(tag) => write!(formatter, "unknown world routine activity {tag}"),
            Self::UnknownCommand(tag) => write!(formatter, "unknown world routine command {tag}"),
            Self::CalendarProfileInvalid => formatter.write_str("WORLD_CALENDAR_PROFILE_INVALID"),
            Self::ContentInvalid => formatter.write_str("WORLD_ROUTINE_CONTENT_INVALID"),
            Self::BindingInvalid => {
                formatter.write_str("world routine interaction binding is invalid")
            }
            Self::SnapshotClosureInvalid => {
                formatter.write_str("world routine snapshot closure is invalid")
            }
            Self::AvailabilityInvalid => {
                formatter.write_str("world routine interaction availability is invalid")
            }
            Self::RevisionExhausted => {
                formatter.write_str("world routine record revision exhausted")
            }
            Self::WrongEnvelope => formatter.write_str("world routine envelope is invalid"),
            Self::MissingField(id) => write!(formatter, "world routine field {id} is missing"),
            Self::UnknownField(id) => write!(formatter, "world routine field {id} is unknown"),
            Self::FieldType => formatter.write_str("world routine field type is invalid"),
            Self::FieldLength => formatter.write_str("world routine field length is invalid"),
            Self::NonCanonicalEncoding => {
                formatter.write_str("world routine value is not canonical")
            }
        }
    }
}

impl Error for WorldRoutineContractError {}

impl From<CanonicalError> for WorldRoutineContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for WorldRoutineContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for WorldRoutineContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}
