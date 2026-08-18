//! Backend-free, content-addressed contracts for the minimal B0 render path.
//!
//! These records deliberately contain no graphics API types, native handles,
//! floating-point values, source paths, or runtime cache identity.

mod catalog;
mod codec;
mod error;
mod neutral;
mod profile;
mod skinning;

pub use catalog::{B0CookedMeshV1, B0MeshletV1, RenderContentCatalogV1};
pub use error::RenderContentContractError;
pub use neutral::{
    AabbI64V1, MaterialAlphaModeV1, MaterialColorSpaceV1, MaterialTextureSlotV1,
    MeshPrimitiveTopologyV1, NeutralMaterialTextureBindingV1, NeutralMaterialV1,
    NeutralMeshPrimitiveV1, NeutralMeshV1, NeutralRenderRecordV1, NeutralTangentV1,
    NeutralTexelEncodingV1, NeutralTextureAlphaSemanticsV1, NeutralTextureColorSpaceV1,
    NeutralTextureDimensionV1, NeutralTextureMipLevelV1, NeutralTextureV1, UvTransformV1,
};
pub use profile::{
    B0_SHADER_INTERFACE_MANIFEST_CANONICAL_BYTES, B0RenderContentProfileV1,
    b0_shader_interface_manifest_sha256,
};
pub use skinning::{
    BASE_SKINNING_MAX_INFLUENCES_PER_VERTEX_V1, BASE_SKINNING_MAX_RENDER_JOINTS_V1,
    BaseSkinningFallbackV1, BaseSkinningMethodV1, NeutralBaseSkinningProfileV1,
    NeutralPoseCorrectiveV1, NeutralPoseCorrectiveVertexDeltaV1, NeutralRenderJointV1,
    NeutralSkinInfluenceV1, NeutralSkinVertexV1, POSE_CORRECTIVE_MAX_RECORDS_V1,
    POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1, PoseCorrectiveDriverAxisV1, PoseCorrectiveLodClassV1,
};

pub const NEUTRAL_MESH_SCHEMA_ID: &str = "nextengine.content.mesh";
pub const NEUTRAL_MATERIAL_SCHEMA_ID: &str = "nextengine.content.material";
pub const NEUTRAL_TEXTURE_SCHEMA_ID: &str = "nextengine.content.texture";
pub const NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID: &str =
    "nextengine.content.base-skinning-profile";
pub const B0_RENDER_CONTENT_PROFILE_SCHEMA_ID: &str = "nextengine.content.render-profile-b0";
pub const RENDER_CONTENT_CATALOG_SCHEMA_ID: &str = "nextengine.render-content.catalog";

pub const RENDER_CONTENT_OWNER_ID: &str = "nextengine.assets";
pub const RENDER_CONTENT_SEGMENT_ID: &str = "v1";
pub const RENDER_CONTENT_SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
mod tests;
