use crate::canonical::{CanonicalDecodeLimits, decode_canonical_segment};
use crate::ids::{AssetId, ContentHash, SchemaId};
use crate::project::{AssetRevisionRefV1, ContentSemanticClassV1, SchemaRefV1};

use super::super::{
    B0_RENDER_CONTENT_PROFILE_SCHEMA_ID, B0RenderContentProfileV1,
    NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID, NEUTRAL_MATERIAL_SCHEMA_ID, NEUTRAL_MESH_SCHEMA_ID,
    NEUTRAL_TEXTURE_SCHEMA_ID, NeutralBaseSkinningProfileV1, RenderContentContractError,
};
use super::{NeutralMaterialV1, NeutralMeshV1, NeutralTextureV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeutralRenderRecordV1 {
    Mesh(NeutralMeshV1),
    Material(NeutralMaterialV1),
    Texture(NeutralTextureV1),
    Profile(B0RenderContentProfileV1),
    BaseSkinningProfile(NeutralBaseSkinningProfileV1),
}

impl NeutralRenderRecordV1 {
    #[must_use]
    pub fn supports_schema_id(schema_id: &SchemaId) -> bool {
        Self::supports_schema_id_str(schema_id.as_str())
    }

    #[must_use]
    pub fn supports_schema_id_str(schema_id: &str) -> bool {
        matches!(
            schema_id,
            NEUTRAL_MESH_SCHEMA_ID
                | NEUTRAL_MATERIAL_SCHEMA_ID
                | NEUTRAL_TEXTURE_SCHEMA_ID
                | B0_RENDER_CONTENT_PROFILE_SCHEMA_ID
                | NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID
        )
    }

    #[must_use]
    pub const fn schema_ref(&self) -> &SchemaRefV1 {
        match self {
            Self::Mesh(value) => value.schema_ref(),
            Self::Material(value) => value.schema_ref(),
            Self::Texture(value) => value.schema_ref(),
            Self::Profile(value) => value.schema_ref(),
            Self::BaseSkinningProfile(value) => value.schema_ref(),
        }
    }

    #[must_use]
    pub const fn asset_id(&self) -> AssetId {
        match self {
            Self::Mesh(value) => value.asset_id(),
            Self::Material(value) => value.asset_id(),
            Self::Texture(value) => value.asset_id(),
            Self::Profile(value) => value.asset_id(),
            Self::BaseSkinningProfile(value) => value.asset_id(),
        }
    }

    #[must_use]
    pub const fn record_revision(&self) -> u64 {
        match self {
            Self::Mesh(value) => value.record_revision(),
            Self::Material(value) => value.record_revision(),
            Self::Texture(value) => value.record_revision(),
            Self::Profile(value) => value.record_revision(),
            Self::BaseSkinningProfile(value) => value.record_revision(),
        }
    }

    #[must_use]
    pub const fn semantic_class(&self) -> ContentSemanticClassV1 {
        ContentSemanticClassV1::PresentationOnly
    }

    #[must_use]
    pub fn dependencies(&self) -> Vec<AssetRevisionRefV1> {
        let mut values = match self {
            Self::Material(value) => value
                .texture_bindings()
                .iter()
                .map(|binding| binding.texture())
                .collect(),
            Self::Profile(value) => value.dependencies(),
            Self::BaseSkinningProfile(value) => value.dependencies(),
            Self::Mesh(_) | Self::Texture(_) => Vec::new(),
        };
        values.sort();
        values.dedup();
        values
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        match self {
            Self::Mesh(value) => value.canonical_bytes(),
            Self::Material(value) => value.canonical_bytes(),
            Self::Texture(value) => value.canonical_bytes(),
            Self::Profile(value) => value.canonical_bytes(),
            Self::BaseSkinningProfile(value) => value.canonical_bytes(),
        }
    }

    pub fn record_sha256(&self) -> Result<ContentHash, RenderContentContractError> {
        match self {
            Self::Mesh(value) => value.record_sha256(),
            Self::Material(value) => value.record_sha256(),
            Self::Texture(value) => value.record_sha256(),
            Self::Profile(value) => value.record_sha256(),
            Self::BaseSkinningProfile(value) => value.record_sha256(),
        }
    }

    pub fn asset_revision(&self) -> Result<AssetRevisionRefV1, RenderContentContractError> {
        Ok(AssetRevisionRefV1 {
            asset_id: self.asset_id(),
            record_sha256: self.record_sha256()?,
        })
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        match segment.schema_id.as_str() {
            NEUTRAL_MESH_SCHEMA_ID => {
                NeutralMeshV1::from_canonical_bytes(bytes, limits).map(Self::Mesh)
            }
            NEUTRAL_MATERIAL_SCHEMA_ID => {
                NeutralMaterialV1::from_canonical_bytes(bytes, limits).map(Self::Material)
            }
            NEUTRAL_TEXTURE_SCHEMA_ID => {
                NeutralTextureV1::from_canonical_bytes(bytes, limits).map(Self::Texture)
            }
            B0_RENDER_CONTENT_PROFILE_SCHEMA_ID => {
                B0RenderContentProfileV1::from_canonical_bytes(bytes, limits).map(Self::Profile)
            }
            NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID => {
                NeutralBaseSkinningProfileV1::from_canonical_bytes(bytes, limits)
                    .map(Self::BaseSkinningProfile)
            }
            _ => Err(RenderContentContractError::UnknownRecordSchema),
        }
    }
}

impl From<NeutralMeshV1> for NeutralRenderRecordV1 {
    fn from(value: NeutralMeshV1) -> Self {
        Self::Mesh(value)
    }
}

impl From<NeutralMaterialV1> for NeutralRenderRecordV1 {
    fn from(value: NeutralMaterialV1) -> Self {
        Self::Material(value)
    }
}

impl From<NeutralTextureV1> for NeutralRenderRecordV1 {
    fn from(value: NeutralTextureV1) -> Self {
        Self::Texture(value)
    }
}

impl From<B0RenderContentProfileV1> for NeutralRenderRecordV1 {
    fn from(value: B0RenderContentProfileV1) -> Self {
        Self::Profile(value)
    }
}

impl From<NeutralBaseSkinningProfileV1> for NeutralRenderRecordV1 {
    fn from(value: NeutralBaseSkinningProfileV1) -> Self {
        Self::BaseSkinningProfile(value)
    }
}
