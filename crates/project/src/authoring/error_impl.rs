use std::error::Error;
use std::fmt::{Display, Formatter};

use super::*;

impl ProjectAuthoringError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Io { .. } => "PROJECT_AUTHORING_IO",
            Self::Json(_) => "PROJECT_MANIFEST_INVALID",
            Self::UnsupportedFormat(_) => "UNSUPPORTED_PROJECT_AUTHORING_FORMAT",
            Self::Identifier(_) | Self::InvalidHex | Self::InvalidValue => "CONTENT_VALUE_INVALID",
            Self::Contract(_) => "CONTENT_SCHEMA_INVALID",
            Self::Neutral(_)
            | Self::Render(_)
            | Self::Localization(_)
            | Self::Audio(_)
            | Self::Animation(_) => "CONTENT_SCHEMA_INVALID",
            Self::UnsafePath(_) => "CONTENT_SOURCE_PATH_INVALID",
            Self::InvalidSourceSpan => "CONTENT_SOURCE_SPAN_INVALID",
            Self::InvalidProvenance | Self::HashMismatch(_) => "CONTENT_PROVENANCE_INVALID",
            Self::DuplicateIdentity => "CONTENT_ID_DUPLICATE",
            Self::MissingReference(_) => "CONTENT_REFERENCE_MISSING",
            Self::SourceLimitExceeded { .. } => "CONTENT_SOURCE_LIMIT_EXCEEDED",
        }
    }
}

impl Display for ProjectAuthoringError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot read authoring source {path}: {source}")
            }
            Self::Json(error) => write!(formatter, "authoring manifest JSON is invalid: {error}"),
            Self::Identifier(error) => {
                write!(formatter, "authoring identifier is invalid: {error}")
            }
            Self::Contract(error) => {
                write!(formatter, "authoring project contract is invalid: {error}")
            }
            Self::Neutral(error) => {
                write!(formatter, "neutral authoring record is invalid: {error}")
            }
            Self::Render(error) => write!(formatter, "render authoring record is invalid: {error}"),
            Self::Localization(error) => {
                write!(formatter, "text authoring record is invalid: {error}")
            }
            Self::Audio(error) => write!(formatter, "audio authoring record is invalid: {error}"),
            Self::Animation(error) => {
                write!(formatter, "animation authoring record is invalid: {error}")
            }
            Self::UnsupportedFormat(format) => {
                write!(formatter, "unsupported authoring format {format}")
            }
            Self::UnsafePath(path) => {
                write!(formatter, "authoring path is not project-relative: {path}")
            }
            Self::InvalidHex => formatter.write_str("authoring hex value is invalid"),
            Self::InvalidSourceSpan => formatter.write_str("authoring source span is invalid"),
            Self::InvalidProvenance => {
                formatter.write_str("authoring provenance closure is invalid")
            }
            Self::InvalidValue => formatter.write_str("authoring value is invalid"),
            Self::DuplicateIdentity => formatter.write_str("authoring identity is duplicated"),
            Self::MissingReference(reference) => {
                write!(formatter, "authoring reference is missing: {reference}")
            }
            Self::HashMismatch(subject) => write!(formatter, "authoring hash mismatch: {subject}"),
            Self::SourceLimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "authoring source size {actual} exceeds limit {limit}"
                )
            }
        }
    }
}

impl Error for ProjectAuthoringError {}

macro_rules! from_error {
    ($source:ty, $variant:ident) => {
        impl From<$source> for ProjectAuthoringError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from_error!(serde_json::Error, Json);
from_error!(IdentifierError, Identifier);
from_error!(ProjectContractError, Contract);
from_error!(NeutralRecordError, Neutral);
from_error!(RenderContentContractError, Render);
from_error!(TextCatalogErrorV1, Localization);
from_error!(NeutralAudioErrorV1, Audio);
from_error!(NeutralAnimationContentErrorV1, Animation);
