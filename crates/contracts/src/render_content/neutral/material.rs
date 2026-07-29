use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I32,
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_SET, CANONICAL_TYPE_U8,
    CANONICAL_TYPE_U16, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor,
    CanonicalDecodeLimits, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash, SchemaId};
use crate::project::{AssetRevisionRefV1, SchemaRefV1};

use super::super::codec::{
    asset_id_from_segment, decode_asset_revision, encode_asset_revision, ensure_limit,
    ensure_nonzero_hash, ensure_unique, extend_count, field, neutral_record_hash, read_count,
    read_i32, read_u8, read_u16, read_u32, read_u64, schema_ref_from_segment, validate_envelope,
    validate_schema_ref,
};
use super::super::{
    NEUTRAL_MATERIAL_SCHEMA_ID, RENDER_CONTENT_OWNER_ID, RENDER_CONTENT_SCHEMA_VERSION,
    RENDER_CONTENT_SEGMENT_ID, RenderContentContractError,
};
use super::common::UvTransformV1;

const MAX_TEXTURE_SLOTS: usize = 64;
const MAX_FEATURE_TAGS: usize = 32;
const MAX_UV_SETS: u8 = 8;
const MAX_EMISSIVE_INTENSITY_Q16_16: u32 = 65_535_u32 << 16;
const MAX_NORMAL_SCALE_Q16_16: i32 = 4 * 65_536;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MaterialAlphaModeV1 {
    Opaque = 1,
    Mask = 2,
    Blend = 3,
}

impl MaterialAlphaModeV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::Opaque),
            2 => Ok(Self::Mask),
            3 => Ok(Self::Blend),
            _ => Err(RenderContentContractError::InvalidMaterial),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MaterialColorSpaceV1 {
    Srgb = 1,
    Linear = 2,
}

impl MaterialColorSpaceV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::Srgb),
            2 => Ok(Self::Linear),
            _ => Err(RenderContentContractError::InvalidMaterial),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MaterialTextureSlotV1 {
    BaseColor = 1,
    MetallicRoughness = 2,
    Normal = 3,
    Occlusion = 4,
    Emissive = 5,
}

impl MaterialTextureSlotV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::BaseColor),
            2 => Ok(Self::MetallicRoughness),
            3 => Ok(Self::Normal),
            4 => Ok(Self::Occlusion),
            5 => Ok(Self::Emissive),
            _ => Err(RenderContentContractError::InvalidMaterial),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralMaterialTextureBindingV1 {
    slot: MaterialTextureSlotV1,
    texture: AssetRevisionRefV1,
    uv_set: u8,
    uv_transform: UvTransformV1,
}

impl NeutralMaterialTextureBindingV1 {
    pub fn new(
        slot: MaterialTextureSlotV1,
        texture: AssetRevisionRefV1,
        uv_set: u8,
        uv_transform: UvTransformV1,
    ) -> Result<Self, RenderContentContractError> {
        ensure_nonzero_hash(texture.record_sha256)?;
        if uv_set >= MAX_UV_SETS {
            return Err(RenderContentContractError::InvalidMaterial);
        }
        Ok(Self {
            slot,
            texture,
            uv_set,
            uv_transform,
        })
    }

    #[must_use]
    pub const fn slot(&self) -> MaterialTextureSlotV1 {
        self.slot
    }

    #[must_use]
    pub const fn texture(&self) -> AssetRevisionRefV1 {
        self.texture
    }

    #[must_use]
    pub const fn uv_set(&self) -> u8 {
        self.uv_set
    }

    #[must_use]
    pub const fn uv_transform(&self) -> UvTransformV1 {
        self.uv_transform
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralMaterialV1 {
    schema_ref: SchemaRefV1,
    asset_id: AssetId,
    record_revision: u64,
    base_color_rgba_unorm16: [u16; 4],
    base_color_space: MaterialColorSpaceV1,
    metallic_unorm16: u16,
    roughness_unorm16: u16,
    emissive_rgb_unorm16: [u16; 3],
    emissive_color_space: MaterialColorSpaceV1,
    emissive_intensity_q16_16: u32,
    normal_scale_q16_16: i32,
    occlusion_strength_unorm16: u16,
    alpha_mode: MaterialAlphaModeV1,
    alpha_cutoff_unorm16: u16,
    double_sided: bool,
    texture_bindings: Vec<NeutralMaterialTextureBindingV1>,
    feature_tags: Vec<SchemaId>,
}

impl NeutralMaterialV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_ref: SchemaRefV1,
        asset_id: AssetId,
        record_revision: u64,
        base_color_rgba_unorm16: [u16; 4],
        base_color_space: MaterialColorSpaceV1,
        metallic_unorm16: u16,
        roughness_unorm16: u16,
        emissive_rgb_unorm16: [u16; 3],
        emissive_color_space: MaterialColorSpaceV1,
        emissive_intensity_q16_16: u32,
        normal_scale_q16_16: i32,
        occlusion_strength_unorm16: u16,
        alpha_mode: MaterialAlphaModeV1,
        alpha_cutoff_unorm16: u16,
        double_sided: bool,
        mut texture_bindings: Vec<NeutralMaterialTextureBindingV1>,
        mut feature_tags: Vec<SchemaId>,
    ) -> Result<Self, RenderContentContractError> {
        validate_schema_ref(&schema_ref, NEUTRAL_MATERIAL_SCHEMA_ID)?;
        if record_revision == 0 {
            return Err(RenderContentContractError::ZeroRevision);
        }
        texture_bindings.sort_by_key(|binding| binding.slot);
        feature_tags.sort();
        ensure_unique_by_slot(&texture_bindings)?;
        ensure_unique(&feature_tags)?;
        ensure_limit(texture_bindings.len(), MAX_TEXTURE_SLOTS)?;
        ensure_limit(feature_tags.len(), MAX_FEATURE_TAGS)?;
        if emissive_intensity_q16_16 > MAX_EMISSIVE_INTENSITY_Q16_16
            || !(0..=MAX_NORMAL_SCALE_Q16_16).contains(&normal_scale_q16_16)
            || (alpha_mode != MaterialAlphaModeV1::Mask && alpha_cutoff_unorm16 != 0)
            || (base_color_rgba_unorm16[3] == 0 && base_color_rgba_unorm16[..3] != [0; 3])
            || (alpha_mode == MaterialAlphaModeV1::Opaque && base_color_rgba_unorm16[3] != u16::MAX)
        {
            return Err(RenderContentContractError::InvalidMaterial);
        }
        Ok(Self {
            schema_ref,
            asset_id,
            record_revision,
            base_color_rgba_unorm16,
            base_color_space,
            metallic_unorm16,
            roughness_unorm16,
            emissive_rgb_unorm16,
            emissive_color_space,
            emissive_intensity_q16_16,
            normal_scale_q16_16,
            occlusion_strength_unorm16,
            alpha_mode,
            alpha_cutoff_unorm16,
            double_sided,
            texture_bindings,
            feature_tags,
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
    pub const fn base_color_rgba_unorm16(&self) -> [u16; 4] {
        self.base_color_rgba_unorm16
    }

    #[must_use]
    pub const fn base_color_space(&self) -> MaterialColorSpaceV1 {
        self.base_color_space
    }

    #[must_use]
    pub const fn metallic_unorm16(&self) -> u16 {
        self.metallic_unorm16
    }

    #[must_use]
    pub const fn roughness_unorm16(&self) -> u16 {
        self.roughness_unorm16
    }

    #[must_use]
    pub const fn emissive_rgb_unorm16(&self) -> [u16; 3] {
        self.emissive_rgb_unorm16
    }

    #[must_use]
    pub const fn emissive_color_space(&self) -> MaterialColorSpaceV1 {
        self.emissive_color_space
    }

    #[must_use]
    pub const fn emissive_intensity_q16_16(&self) -> u32 {
        self.emissive_intensity_q16_16
    }

    #[must_use]
    pub const fn normal_scale_q16_16(&self) -> i32 {
        self.normal_scale_q16_16
    }

    #[must_use]
    pub const fn occlusion_strength_unorm16(&self) -> u16 {
        self.occlusion_strength_unorm16
    }

    #[must_use]
    pub const fn alpha_mode(&self) -> MaterialAlphaModeV1 {
        self.alpha_mode
    }

    #[must_use]
    pub const fn alpha_cutoff_unorm16(&self) -> u16 {
        self.alpha_cutoff_unorm16
    }

    #[must_use]
    pub const fn double_sided(&self) -> bool {
        self.double_sided
    }

    #[must_use]
    pub fn texture_bindings(&self) -> &[NeutralMaterialTextureBindingV1] {
        &self.texture_bindings
    }

    #[must_use]
    pub fn feature_tags(&self) -> &[SchemaId] {
        &self.feature_tags
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            NEUTRAL_MATERIAL_SCHEMA_ID,
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
                    CANONICAL_TYPE_BYTES,
                    encode_u16_array(self.base_color_rgba_unorm16),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_U16,
                    self.metallic_unorm16.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_U16,
                    self.roughness_unorm16.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_BYTES,
                    encode_u16_array(self.emissive_rgb_unorm16),
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_U32,
                    self.emissive_intensity_q16_16.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    10,
                    CANONICAL_TYPE_I32,
                    self.normal_scale_q16_16.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    11,
                    CANONICAL_TYPE_U16,
                    self.occlusion_strength_unorm16.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(12, CANONICAL_TYPE_U8, vec![self.alpha_mode as u8]),
                CanonicalField::new(
                    13,
                    CANONICAL_TYPE_U16,
                    self.alpha_cutoff_unorm16.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(14, CANONICAL_TYPE_BOOL, vec![u8::from(self.double_sided)]),
                CanonicalField::new(
                    15,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_bindings(&self.texture_bindings)?,
                ),
                CanonicalField::new(
                    16,
                    CANONICAL_TYPE_SET,
                    encode_feature_tags(&self.feature_tags)?,
                ),
                CanonicalField::new(17, CANONICAL_TYPE_U8, vec![self.base_color_space as u8]),
                CanonicalField::new(18, CANONICAL_TYPE_U8, vec![self.emissive_color_space as u8]),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, NEUTRAL_MATERIAL_SCHEMA_ID, 18)?;
        let value = Self::new(
            schema_ref_from_segment(&segment, 4)?,
            asset_id_from_segment(&segment, 2)?,
            read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?,
            decode_u16_array(field(&segment, 5, CANONICAL_TYPE_BYTES)?)?,
            MaterialColorSpaceV1::from_tag(read_u8(field(&segment, 17, CANONICAL_TYPE_U8)?)?)?,
            read_u16(field(&segment, 6, CANONICAL_TYPE_U16)?)?,
            read_u16(field(&segment, 7, CANONICAL_TYPE_U16)?)?,
            decode_u16_array(field(&segment, 8, CANONICAL_TYPE_BYTES)?)?,
            MaterialColorSpaceV1::from_tag(read_u8(field(&segment, 18, CANONICAL_TYPE_U8)?)?)?,
            read_u32(field(&segment, 9, CANONICAL_TYPE_U32)?)?,
            read_i32(field(&segment, 10, CANONICAL_TYPE_I32)?)?,
            read_u16(field(&segment, 11, CANONICAL_TYPE_U16)?)?,
            MaterialAlphaModeV1::from_tag(read_u8(field(&segment, 12, CANONICAL_TYPE_U8)?)?)?,
            read_u16(field(&segment, 13, CANONICAL_TYPE_U16)?)?,
            decode_bool(field(&segment, 14, CANONICAL_TYPE_BOOL)?)?,
            decode_bindings(field(&segment, 15, CANONICAL_TYPE_SEQUENCE)?, limits)?,
            decode_feature_tags(field(&segment, 16, CANONICAL_TYPE_SET)?, limits)?,
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
}

fn ensure_unique_by_slot(
    bindings: &[NeutralMaterialTextureBindingV1],
) -> Result<(), RenderContentContractError> {
    if bindings.windows(2).any(|pair| pair[0].slot == pair[1].slot) {
        Err(RenderContentContractError::DuplicateIdentity)
    } else {
        Ok(())
    }
}

fn encode_u16_array<const LENGTH: usize>(values: [u16; LENGTH]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(LENGTH * 2);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_u16_array<const LENGTH: usize>(
    bytes: &[u8],
) -> Result<[u16; LENGTH], RenderContentContractError> {
    if bytes.len() != LENGTH * 2 {
        return Err(RenderContentContractError::InvalidPayload);
    }
    let mut output = [0_u16; LENGTH];
    for (index, value) in output.iter_mut().enumerate() {
        let offset = index * 2;
        *value = u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        );
    }
    Ok(output)
}

fn encode_bindings(
    bindings: &[NeutralMaterialTextureBindingV1],
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, bindings.len())?;
    for binding in bindings {
        bytes.push(binding.slot as u8);
        bytes.extend_from_slice(&encode_asset_revision(binding.texture));
        bytes.push(binding.uv_set);
        bytes.extend_from_slice(&binding.uv_transform.encode());
    }
    Ok(bytes)
}

fn decode_bindings(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralMaterialTextureBindingV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_TEXTURE_SLOTS)?;
    let mut bindings = Vec::with_capacity(count);
    for _ in 0..count {
        bindings.push(NeutralMaterialTextureBindingV1::new(
            MaterialTextureSlotV1::from_tag(cursor.read_u8()?)?,
            decode_asset_revision(cursor.read_exact(48)?)?,
            cursor.read_u8()?,
            UvTransformV1::decode(cursor.read_exact(24)?)?,
        )?);
    }
    cursor.finish()?;
    Ok(bindings)
}

fn encode_feature_tags(tags: &[SchemaId]) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, tags.len())?;
    for tag in tags {
        extend_count(&mut bytes, tag.as_str().len())?;
        bytes.extend_from_slice(tag.as_str().as_bytes());
    }
    Ok(bytes)
}

fn decode_feature_tags(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<SchemaId>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_FEATURE_TAGS)?;
    let mut tags = Vec::with_capacity(count);
    for _ in 0..count {
        let length = read_count(&mut cursor, limits, limits.max_identifier_bytes)?;
        let text = std::str::from_utf8(cursor.read_exact(length)?)
            .map_err(|_| RenderContentContractError::InvalidPayload)?;
        tags.push(SchemaId::new(text)?);
    }
    cursor.finish()?;
    Ok(tags)
}

fn decode_bool(bytes: &[u8]) -> Result<bool, RenderContentContractError> {
    match read_u8(bytes)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(RenderContentContractError::InvalidPayload),
    }
}
