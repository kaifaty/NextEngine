use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeError, CanonicalError};
use crate::ids::IdentifierError;
use crate::project::ProjectContractError;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderContentContractError {
    Canonical(CanonicalError),
    Decode(CanonicalDecodeError),
    Identifier(IdentifierError),
    Project(ProjectContractError),
    EnvelopeMismatch,
    SchemaMismatch,
    UnsupportedVersion(u32),
    UnknownRecordSchema,
    WrongFieldType(u32),
    InvalidPayload,
    InvalidBounds,
    PositionOutsideBounds,
    EmptyMesh,
    AttributeLengthMismatch,
    InvalidNormal,
    InvalidTangent,
    InvalidIndex,
    InvalidPrimitive,
    InvalidTextureExtent,
    InvalidTextureData,
    InvalidHalfFloat,
    InvalidMaterial,
    InvalidSkinningProfile,
    InvalidSkinWeights,
    InvalidPoseCorrective,
    ZeroHash,
    ZeroRevision,
    DuplicateIdentity,
    MissingReference,
    UnsupportedB0Feature,
    LimitExceeded { actual: usize, limit: usize },
    IntegerOverflow,
    HashMismatch,
    NonCanonical,
}

impl RenderContentContractError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::SchemaMismatch | Self::UnsupportedVersion(_) | Self::UnknownRecordSchema => {
                "CONTENT_SCHEMA_MISMATCH"
            }
            Self::InvalidBounds | Self::PositionOutsideBounds => "CONTENT_BOUNDS_INVALID",
            Self::MissingReference => "CONTENT_DEPENDENCY_MISSING",
            Self::HashMismatch | Self::ZeroHash => "CONTENT_MANIFEST_INVALID",
            Self::InvalidMaterial | Self::UnsupportedB0Feature => "MATERIAL_SCHEMA_INVALID",
            Self::InvalidSkinningProfile | Self::InvalidSkinWeights => "CONTENT_SKINNING_INVALID",
            Self::InvalidPoseCorrective => "CONTENT_POSE_CORRECTIVE_INVALID",
            _ => "CONTENT_MANIFEST_INVALID",
        }
    }
}

impl Display for RenderContentContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "render content encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "render content decode failed: {error}"),
            Self::Identifier(error) => write!(formatter, "render content ID is invalid: {error}"),
            Self::Project(error) => write!(formatter, "render content schema is invalid: {error}"),
            Self::EnvelopeMismatch => formatter.write_str("render content envelope mismatch"),
            Self::SchemaMismatch => formatter.write_str("render content schema mismatch"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported render content version {version}")
            }
            Self::UnknownRecordSchema => formatter.write_str("unknown render content schema"),
            Self::WrongFieldType(field_id) => {
                write!(
                    formatter,
                    "render content field {field_id} has the wrong type"
                )
            }
            Self::InvalidPayload => formatter.write_str("render content payload is invalid"),
            Self::InvalidBounds => formatter.write_str("render content bounds are invalid"),
            Self::PositionOutsideBounds => {
                formatter.write_str("mesh position lies outside declared bounds")
            }
            Self::EmptyMesh => formatter.write_str("mesh must contain geometry"),
            Self::AttributeLengthMismatch => {
                formatter.write_str("mesh attribute stream length does not match vertex count")
            }
            Self::InvalidNormal => formatter.write_str("mesh normal is not canonical unit snorm16"),
            Self::InvalidTangent => formatter.write_str("mesh tangent or handedness is invalid"),
            Self::InvalidIndex => formatter.write_str("mesh index is outside the vertex range"),
            Self::InvalidPrimitive => formatter.write_str("mesh primitive range is invalid"),
            Self::InvalidTextureExtent => formatter.write_str("texture extent is invalid"),
            Self::InvalidTextureData => {
                formatter.write_str("texture mip bytes do not match the declared encoding")
            }
            Self::InvalidHalfFloat => {
                formatter.write_str("texture contains noncanonical binary16 data")
            }
            Self::InvalidMaterial => formatter.write_str("material definition is invalid"),
            Self::InvalidSkinningProfile => formatter.write_str("base-skinning profile is invalid"),
            Self::InvalidSkinWeights => formatter.write_str("base-skinning weights are invalid"),
            Self::InvalidPoseCorrective => formatter.write_str("pose corrective is invalid"),
            Self::ZeroHash => formatter.write_str("render content hash must not be zero"),
            Self::ZeroRevision => formatter.write_str("render content revision must be positive"),
            Self::DuplicateIdentity => formatter.write_str("render content identity is duplicated"),
            Self::MissingReference => formatter.write_str("render content reference is absent"),
            Self::UnsupportedB0Feature => {
                formatter.write_str("render content is outside the minimal B0 profile")
            }
            Self::LimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "render content count {actual} exceeds limit {limit}"
                )
            }
            Self::IntegerOverflow => formatter.write_str("render content size arithmetic overflow"),
            Self::HashMismatch => formatter.write_str("render content hash mismatch"),
            Self::NonCanonical => formatter.write_str("render content bytes are not canonical"),
        }
    }
}

impl Error for RenderContentContractError {}

impl From<CanonicalError> for RenderContentContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalDecodeError> for RenderContentContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Decode(error)
    }
}

impl From<IdentifierError> for RenderContentContractError {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<ProjectContractError> for RenderContentContractError {
    fn from(error: ProjectContractError) -> Self {
        Self::Project(error)
    }
}
