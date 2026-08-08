use crate::canonical::sha256;
use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalDecodeLimits, CanonicalField,
    decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash, content_hash_from_bytes};
use crate::project::{AssetRevisionRefV1, SchemaRefV1};

use super::codec::{
    asset_id_from_segment, decode_asset_revision, encode_asset_revision, ensure_nonzero_hash,
    field, neutral_record_hash, read_array, read_u8, read_u32, read_u64, schema_ref_from_segment,
    validate_envelope, validate_schema_ref,
};
use super::{
    B0_RENDER_CONTENT_PROFILE_SCHEMA_ID, MaterialAlphaModeV1, MeshPrimitiveTopologyV1,
    NeutralTexelEncodingV1, NeutralTextureColorSpaceV1, RENDER_CONTENT_OWNER_ID,
    RENDER_CONTENT_SCHEMA_VERSION, RENDER_CONTENT_SEGMENT_ID, RenderContentContractError,
};

pub const B0_MAX_MESHLET_VERTICES: u32 = 64;
pub const B0_MAX_MESHLET_TRIANGLES: u32 = 126;
pub const B0_SHADER_INTERFACE_MANIFEST_CANONICAL_BYTES: &[u8] =
    b"nextengine.shader-interface.b0.v2;vertex-inputs:location0-f32x3-position,location1-f32x2-uv0,location2-r16g16b16a16-snorm-normal;descriptors:set0-binding0-uniform-buffer-vertex-fragment-min208,set1-binding0-combined-image-sampler-fragment-count1,set2-binding0-combined-depth-compare-sampler-fragment-count1;push-constants:offset0-size80-vertex-fragment-model-mat4-base-color-factor-f32x4;fragment-output:location0-f32x4-rgba";

#[must_use]
pub fn b0_shader_interface_manifest_sha256() -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.shader-interface-manifest.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(B0_SHADER_INTERFACE_MANIFEST_CANONICAL_BYTES.len())
            .expect("static shader-interface manifest length fits u64")
            .to_le_bytes(),
    );
    preimage.extend_from_slice(B0_SHADER_INTERFACE_MANIFEST_CANONICAL_BYTES);
    content_hash_from_bytes(sha256(&preimage))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct B0RenderContentProfileV1 {
    schema_ref: SchemaRefV1,
    asset_id: AssetId,
    record_revision: u64,
    shader_interface_manifest_sha256: ContentHash,
    fallback_material: AssetRevisionRefV1,
    fallback_texture: AssetRevisionRefV1,
}

impl B0RenderContentProfileV1 {
    pub fn new(
        schema_ref: SchemaRefV1,
        asset_id: AssetId,
        record_revision: u64,
        shader_interface_manifest_sha256: ContentHash,
        fallback_material: AssetRevisionRefV1,
        fallback_texture: AssetRevisionRefV1,
    ) -> Result<Self, RenderContentContractError> {
        validate_schema_ref(&schema_ref, B0_RENDER_CONTENT_PROFILE_SCHEMA_ID)?;
        if record_revision == 0 {
            return Err(RenderContentContractError::ZeroRevision);
        }
        ensure_nonzero_hash(shader_interface_manifest_sha256)?;
        ensure_nonzero_hash(fallback_material.record_sha256)?;
        ensure_nonzero_hash(fallback_texture.record_sha256)?;
        Ok(Self {
            schema_ref,
            asset_id,
            record_revision,
            shader_interface_manifest_sha256,
            fallback_material,
            fallback_texture,
        })
    }

    #[must_use]
    pub const fn schema_ref(&self) -> &SchemaRefV1 {
        &self.schema_ref
    }

    #[must_use]
    pub const fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    #[must_use]
    pub const fn record_revision(&self) -> u64 {
        self.record_revision
    }

    #[must_use]
    pub const fn shader_interface_manifest_sha256(&self) -> ContentHash {
        self.shader_interface_manifest_sha256
    }

    #[must_use]
    pub const fn fallback_material(&self) -> AssetRevisionRefV1 {
        self.fallback_material
    }

    #[must_use]
    pub const fn fallback_texture(&self) -> AssetRevisionRefV1 {
        self.fallback_texture
    }

    #[must_use]
    pub const fn topology(&self) -> MeshPrimitiveTopologyV1 {
        MeshPrimitiveTopologyV1::Triangles
    }

    #[must_use]
    pub const fn texel_encoding(&self) -> NeutralTexelEncodingV1 {
        NeutralTexelEncodingV1::Rgba8Unorm
    }

    #[must_use]
    pub const fn color_space(&self) -> NeutralTextureColorSpaceV1 {
        NeutralTextureColorSpaceV1::Srgb
    }

    #[must_use]
    pub const fn alpha_mode(&self) -> MaterialAlphaModeV1 {
        MaterialAlphaModeV1::Opaque
    }

    #[must_use]
    pub const fn max_meshlet_vertices(&self) -> u32 {
        B0_MAX_MESHLET_VERTICES
    }

    #[must_use]
    pub const fn max_meshlet_triangles(&self) -> u32 {
        B0_MAX_MESHLET_TRIANGLES
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            B0_RENDER_CONTENT_PROFILE_SCHEMA_ID,
            RENDER_CONTENT_SEGMENT_ID,
            [
                CanonicalField::new(
                    1,
                    CANONICAL_TYPE_U32,
                    RENDER_CONTENT_SCHEMA_VERSION.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(2, CANONICAL_TYPE_ID128, self.asset_id.as_bytes().to_vec()),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_U64,
                    self.record_revision.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_HASH256,
                    self.schema_ref.descriptor_sha256.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_HASH256,
                    self.shader_interface_manifest_sha256.as_bytes().to_vec(),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_STRUCT,
                    encode_asset_revision(self.fallback_material).to_vec(),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_STRUCT,
                    encode_asset_revision(self.fallback_texture).to_vec(),
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_U8,
                    vec![MeshPrimitiveTopologyV1::Triangles as u8],
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_U8,
                    vec![NeutralTexelEncodingV1::Rgba8Unorm as u8],
                ),
                CanonicalField::new(
                    10,
                    CANONICAL_TYPE_U8,
                    vec![NeutralTextureColorSpaceV1::Srgb as u8],
                ),
                CanonicalField::new(
                    11,
                    CANONICAL_TYPE_U8,
                    vec![MaterialAlphaModeV1::Opaque as u8],
                ),
                CanonicalField::new(
                    12,
                    CANONICAL_TYPE_U32,
                    B0_MAX_MESHLET_VERTICES.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    13,
                    CANONICAL_TYPE_U32,
                    B0_MAX_MESHLET_TRIANGLES.to_le_bytes().to_vec(),
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, B0_RENDER_CONTENT_PROFILE_SCHEMA_ID, 13)?;
        if read_u8(field(&segment, 8, CANONICAL_TYPE_U8)?)?
            != MeshPrimitiveTopologyV1::Triangles as u8
            || read_u8(field(&segment, 9, CANONICAL_TYPE_U8)?)?
                != NeutralTexelEncodingV1::Rgba8Unorm as u8
            || read_u8(field(&segment, 10, CANONICAL_TYPE_U8)?)?
                != NeutralTextureColorSpaceV1::Srgb as u8
            || read_u8(field(&segment, 11, CANONICAL_TYPE_U8)?)?
                != MaterialAlphaModeV1::Opaque as u8
            || read_u32(field(&segment, 12, CANONICAL_TYPE_U32)?)? != B0_MAX_MESHLET_VERTICES
            || read_u32(field(&segment, 13, CANONICAL_TYPE_U32)?)? != B0_MAX_MESHLET_TRIANGLES
        {
            return Err(RenderContentContractError::UnsupportedB0Feature);
        }
        let value = Self::new(
            schema_ref_from_segment(&segment, 4)?,
            asset_id_from_segment(&segment, 2)?,
            read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?,
            ContentHash::from_bytes(read_array(field(&segment, 5, CANONICAL_TYPE_HASH256)?)?),
            decode_asset_revision(field(&segment, 6, CANONICAL_TYPE_STRUCT)?)?,
            decode_asset_revision(field(&segment, 7, CANONICAL_TYPE_STRUCT)?)?,
        )?;
        if value.canonical_bytes()? != bytes {
            return Err(RenderContentContractError::NonCanonical);
        }
        Ok(value)
    }

    pub fn record_sha256(&self) -> Result<ContentHash, RenderContentContractError> {
        neutral_record_hash(&self.schema_ref, &self.canonical_bytes()?)
    }

    pub fn asset_revision(&self) -> Result<AssetRevisionRefV1, RenderContentContractError> {
        Ok(AssetRevisionRefV1 {
            asset_id: self.asset_id,
            record_sha256: self.record_sha256()?,
        })
    }

    #[must_use]
    pub fn dependencies(&self) -> Vec<AssetRevisionRefV1> {
        let mut values = vec![self.fallback_material, self.fallback_texture];
        values.sort();
        values.dedup();
        values
    }
}
