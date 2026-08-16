use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::CanonicalError;
use crate::command::PrincipalDecodeError;
use crate::persistence::ManifestValidationError;

mod jcs;
mod replay;
mod replay_event;
mod save;

#[cfg(test)]
mod tests;

pub(crate) use jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
pub(crate) use replay::{decode_replay_manifest_v9, encode_replay_manifest_v9};
pub(crate) use save::{decode_save_manifest, encode_save_manifest};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ManifestCodecError {
    Validation(ManifestValidationError),
    Canonicalization(CanonicalError),
    Principal(PrincipalDecodeError),
    Identifier(crate::ids::IdentifierError),
    InputTooLarge {
        actual: usize,
        limit: usize,
    },
    UnexpectedEnd,
    InvalidUtf8,
    InvalidSyntax,
    InvalidEscape,
    InvalidUnicodeEscape,
    DuplicateObjectKey(String),
    TooManyItems {
        limit: usize,
    },
    NestingTooDeep,
    MissingField(String),
    UnknownField(String),
    WrongType {
        field: String,
        expected: &'static str,
    },
    InvalidInteger(String),
    InvalidHex(String),
    NonCanonicalJcs,
}

impl Display for ManifestCodecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(error) => write!(formatter, "manifest validation failed: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "manifest canonicalization failed: {error}")
            }
            Self::Principal(error) => write!(formatter, "manifest principal is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "manifest identifier is invalid: {error}"),
            Self::InputTooLarge { actual, limit } => {
                write!(formatter, "manifest has {actual} bytes; limit is {limit}")
            }
            Self::UnexpectedEnd => formatter.write_str("manifest ended unexpectedly"),
            Self::InvalidUtf8 => formatter.write_str("manifest contains invalid UTF-8"),
            Self::InvalidSyntax => formatter.write_str("manifest JSON syntax is invalid"),
            Self::InvalidEscape => formatter.write_str("manifest JSON escape is invalid"),
            Self::InvalidUnicodeEscape => {
                formatter.write_str("manifest JSON Unicode escape is invalid")
            }
            Self::DuplicateObjectKey(key) => write!(formatter, "duplicate manifest key {key}"),
            Self::TooManyItems { limit } => {
                write!(formatter, "manifest exceeds item limit {limit}")
            }
            Self::NestingTooDeep => formatter.write_str("manifest nesting is too deep"),
            Self::MissingField(field) => write!(formatter, "manifest field {field} is missing"),
            Self::UnknownField(field) => write!(formatter, "manifest field {field} is unknown"),
            Self::WrongType { field, expected } => {
                write!(formatter, "manifest field {field} must be {expected}")
            }
            Self::InvalidInteger(field) => {
                write!(formatter, "manifest field {field} is not a valid integer")
            }
            Self::InvalidHex(field) => {
                write!(
                    formatter,
                    "manifest field {field} is not canonical lowercase hex"
                )
            }
            Self::NonCanonicalJcs => {
                formatter.write_str("manifest bytes are valid JSON but not canonical JCS")
            }
        }
    }
}

impl Error for ManifestCodecError {}

impl From<ManifestValidationError> for ManifestCodecError {
    fn from(error: ManifestValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<CanonicalError> for ManifestCodecError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<PrincipalDecodeError> for ManifestCodecError {
    fn from(error: PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<crate::ids::IdentifierError> for ManifestCodecError {
    fn from(error: crate::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}
