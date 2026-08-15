use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeError, CanonicalError};
use crate::ids::IdentifierError;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IdentityContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    Principal(crate::command::PrincipalDecodeError),
    WrongEnvelope,
    MissingField(u32),
    UnknownField(u32),
    WrongFieldType {
        field_id: u32,
        expected: u8,
        actual: u8,
    },
    InvalidFieldLength {
        field_id: u32,
        expected: usize,
        actual: usize,
    },
    UnsupportedVersion {
        contract: &'static str,
        version: u16,
    },
    IdentityEpochUnsupported(u32),
    WorldNamespaceMismatch,
    PrincipalCollision,
    CommandStreamMismatch,
    CommandStreamCollision,
    NextStreamSlotMismatch,
    StreamSlotExhausted,
    FutureHorizonExceeded(u32),
    UnknownTag(u8),
    DuplicateKey,
    RegistryClosureInvalid,
    ScheduleClosureInvalid,
    NonCanonicalEncoding,
}

impl Display for IdentityContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => {
                write!(formatter, "identity canonicalization failed: {error}")
            }
            Self::Decode(error) => write!(formatter, "identity encoding is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "identity identifier is invalid: {error}"),
            Self::Principal(error) => write!(formatter, "identity principal is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("identity contract envelope is invalid"),
            Self::MissingField(id) => write!(formatter, "identity contract field {id} is missing"),
            Self::UnknownField(id) => write!(formatter, "identity contract field {id} is unknown"),
            Self::WrongFieldType {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "identity field {field_id} has type {actual:#04x}; expected {expected:#04x}"
            ),
            Self::InvalidFieldLength {
                field_id,
                expected,
                actual,
            } => write!(
                formatter,
                "identity field {field_id} has {actual} bytes; expected {expected}"
            ),
            Self::UnsupportedVersion { contract, version } => {
                write!(formatter, "unsupported {contract} schema version {version}")
            }
            Self::IdentityEpochUnsupported(epoch) => {
                write!(formatter, "identity epoch {epoch} is unsupported")
            }
            Self::WorldNamespaceMismatch => {
                formatter.write_str("world namespace does not match project and creation nonce")
            }
            Self::PrincipalCollision => formatter.write_str("principal has conflicting provenance"),
            Self::CommandStreamMismatch => {
                formatter.write_str("command stream ID does not match its canonical provenance")
            }
            Self::CommandStreamCollision => {
                formatter.write_str("command stream ID has conflicting provenance")
            }
            Self::NextStreamSlotMismatch => {
                formatter.write_str("next command stream slot does not follow allocated slots")
            }
            Self::StreamSlotExhausted => formatter.write_str("command stream slot exhausted"),
            Self::FutureHorizonExceeded(value) => write!(
                formatter,
                "maximum future command horizon {value} exceeds the hard maximum"
            ),
            Self::UnknownTag(tag) => write!(formatter, "identity contract tag {tag} is unknown"),
            Self::DuplicateKey => formatter.write_str("identity contract contains a duplicate key"),
            Self::RegistryClosureInvalid => {
                formatter.write_str("command kind registry closure is invalid")
            }
            Self::ScheduleClosureInvalid => {
                formatter.write_str("schedule manifest closure is invalid")
            }
            Self::NonCanonicalEncoding => {
                formatter.write_str("identity contract does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for IdentityContractError {}

impl From<CanonicalError> for IdentityContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for IdentityContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for IdentityContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::command::PrincipalDecodeError> for IdentityContractError {
    fn from(error: crate::command::PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}
