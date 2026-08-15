use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::animation_content::NeutralAnimationContentErrorV1;
use next_contracts::audio::NeutralAudioErrorV1;
use next_contracts::content::NeutralRecordError;
use next_contracts::localization::TextCatalogErrorV1;
use next_contracts::mechanics::MechanicsContractError;
use next_contracts::project::ProjectContractError;
use next_contracts::render_content::RenderContentContractError;

#[derive(Debug)]
#[non_exhaustive]
pub enum ProjectCookError {
    Authoring(crate::ProjectAuthoringError),
    Contract(ProjectContractError),
    Neutral(NeutralRecordError),
    Render(RenderContentContractError),
    Localization(TextCatalogErrorV1),
    Audio(NeutralAudioErrorV1),
    Animation(NeutralAnimationContentErrorV1),
    Store(next_assets::ContentStoreError),
    Identifier(next_contracts::ids::IdentifierError),
    MissingReference,
    DuplicateIdentity,
    InvalidRevision,
    HashCollision,
    LocalizationClosureInvalid,
    Mechanics(MechanicsContractError),
    InvalidValue,
}

impl ProjectCookError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Authoring(error) => error.diagnostic_code(),
            Self::Contract(_) | Self::Neutral(_) => "CONTENT_SCHEMA_INVALID",
            Self::Render(error) => error.diagnostic_code(),
            Self::Localization(_) => "CONTENT_SCHEMA_INVALID",
            Self::Audio(_) => "CONTENT_SCHEMA_INVALID",
            Self::Animation(_) => "CONTENT_SCHEMA_INVALID",
            Self::Store(_) => "CONTENT_PUBLICATION_FAILED",
            Self::Identifier(_) => "CONTENT_IDENTIFIER_INVALID",
            Self::MissingReference => "CONTENT_REFERENCE_MISSING",
            Self::DuplicateIdentity => "CONTENT_ID_DUPLICATE",
            Self::InvalidRevision => "CONTENT_REVISION_INVALID",
            Self::HashCollision => "CONTENT_HASH_COLLISION",
            Self::LocalizationClosureInvalid => "LOCALIZATION_CLOSURE_INVALID",
            Self::Mechanics(_) => "MECHANICS_MANIFEST_INVALID",
            Self::InvalidValue => "CONTENT_VALUE_INVALID",
        }
    }
}

impl Display for ProjectCookError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authoring(error) => write!(formatter, "project authoring failed: {error}"),
            Self::Contract(error) => write!(formatter, "content contract invalid: {error}"),
            Self::Neutral(error) => write!(formatter, "neutral record invalid: {error}"),
            Self::Render(error) => write!(formatter, "render content invalid: {error}"),
            Self::Localization(error) => write!(formatter, "text catalog invalid: {error}"),
            Self::Audio(error) => write!(formatter, "neutral audio clip invalid: {error}"),
            Self::Animation(error) => {
                write!(formatter, "neutral animation content invalid: {error}")
            }
            Self::Store(error) => write!(formatter, "content publication failed: {error}"),
            Self::Identifier(error) => write!(formatter, "content identifier invalid: {error}"),
            Self::MissingReference => formatter.write_str("content reference is missing"),
            Self::DuplicateIdentity => formatter.write_str("content identity is duplicated"),
            Self::InvalidRevision => formatter.write_str("content revision must be positive"),
            Self::HashCollision => formatter.write_str("content hash collision"),
            Self::LocalizationClosureInvalid => {
                formatter.write_str("localization fallback closure is invalid")
            }
            Self::Mechanics(error) => write!(formatter, "mechanics contract invalid: {error}"),
            Self::InvalidValue => formatter.write_str("content property value is invalid"),
        }
    }
}

impl Error for ProjectCookError {}

impl From<crate::ProjectAuthoringError> for ProjectCookError {
    fn from(error: crate::ProjectAuthoringError) -> Self {
        Self::Authoring(error)
    }
}

impl From<ProjectContractError> for ProjectCookError {
    fn from(error: ProjectContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<NeutralRecordError> for ProjectCookError {
    fn from(error: NeutralRecordError) -> Self {
        Self::Neutral(error)
    }
}

impl From<RenderContentContractError> for ProjectCookError {
    fn from(error: RenderContentContractError) -> Self {
        Self::Render(error)
    }
}

impl From<TextCatalogErrorV1> for ProjectCookError {
    fn from(error: TextCatalogErrorV1) -> Self {
        Self::Localization(error)
    }
}

impl From<NeutralAudioErrorV1> for ProjectCookError {
    fn from(error: NeutralAudioErrorV1) -> Self {
        Self::Audio(error)
    }
}

impl From<NeutralAnimationContentErrorV1> for ProjectCookError {
    fn from(error: NeutralAnimationContentErrorV1) -> Self {
        Self::Animation(error)
    }
}

impl From<next_assets::ContentStoreError> for ProjectCookError {
    fn from(error: next_assets::ContentStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<next_contracts::ids::IdentifierError> for ProjectCookError {
    fn from(error: next_contracts::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<MechanicsContractError> for ProjectCookError {
    fn from(error: MechanicsContractError) -> Self {
        Self::Mechanics(error)
    }
}
