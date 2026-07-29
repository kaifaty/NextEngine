use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_U8, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor,
    CanonicalDecodeLimits, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash};
use crate::project::{AssetRevisionRefV1, SchemaRefV1};

use super::super::codec::{
    asset_id_from_segment, ensure_limit, extend_count, field, neutral_record_hash, read_count,
    read_u8, read_u32, read_u64, schema_ref_from_segment, validate_envelope, validate_schema_ref,
};
use super::super::{
    NEUTRAL_TEXTURE_SCHEMA_ID, RENDER_CONTENT_OWNER_ID, RENDER_CONTENT_SCHEMA_VERSION,
    RENDER_CONTENT_SEGMENT_ID, RenderContentContractError,
};

const MAX_2D_EXTENT: u32 = 16_384;
const MAX_3D_EXTENT: u32 = 2_048;
const MAX_ARRAY_LAYERS: u32 = 2_048;
const MAX_MIP_LEVELS: usize = 15;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NeutralTextureDimensionV1 {
    D1 = 1,
    D2 = 2,
    D3 = 3,
    Cube = 4,
}

impl NeutralTextureDimensionV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::D1),
            2 => Ok(Self::D2),
            3 => Ok(Self::D3),
            4 => Ok(Self::Cube),
            _ => Err(RenderContentContractError::InvalidTextureExtent),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NeutralTextureColorSpaceV1 {
    Srgb = 1,
    Linear = 2,
    Data = 3,
}

impl NeutralTextureColorSpaceV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::Srgb),
            2 => Ok(Self::Linear),
            3 => Ok(Self::Data),
            _ => Err(RenderContentContractError::InvalidTextureData),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NeutralTextureAlphaSemanticsV1 {
    Opaque = 1,
    Straight = 2,
}

impl NeutralTextureAlphaSemanticsV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::Opaque),
            2 => Ok(Self::Straight),
            _ => Err(RenderContentContractError::InvalidTextureData),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum NeutralTexelEncodingV1 {
    R8Unorm = 1,
    Rg8Unorm = 2,
    Rgba8Unorm = 3,
    Rgba16Unorm = 4,
    Rgba16FloatCanonical = 5,
}

impl NeutralTexelEncodingV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::R8Unorm),
            2 => Ok(Self::Rg8Unorm),
            3 => Ok(Self::Rgba8Unorm),
            4 => Ok(Self::Rgba16Unorm),
            5 => Ok(Self::Rgba16FloatCanonical),
            _ => Err(RenderContentContractError::InvalidTextureData),
        }
    }

    const fn bytes_per_texel(self) -> usize {
        match self {
            Self::R8Unorm => 1,
            Self::Rg8Unorm => 2,
            Self::Rgba8Unorm => 4,
            Self::Rgba16Unorm | Self::Rgba16FloatCanonical => 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralTextureMipLevelV1 {
    extent: [u32; 3],
    texels: Vec<u8>,
}

impl NeutralTextureMipLevelV1 {
    #[must_use]
    pub fn new(extent: [u32; 3], texels: Vec<u8>) -> Self {
        Self { extent, texels }
    }

    #[must_use]
    pub const fn extent(&self) -> [u32; 3] {
        self.extent
    }

    #[must_use]
    pub fn texels(&self) -> &[u8] {
        &self.texels
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralTextureV1 {
    schema_ref: SchemaRefV1,
    asset_id: AssetId,
    record_revision: u64,
    dimension: NeutralTextureDimensionV1,
    extent: [u32; 3],
    array_layers: u32,
    color_space: NeutralTextureColorSpaceV1,
    alpha_semantics: NeutralTextureAlphaSemanticsV1,
    texel_encoding: NeutralTexelEncodingV1,
    mip_levels: Vec<NeutralTextureMipLevelV1>,
}

impl NeutralTextureV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_ref: SchemaRefV1,
        asset_id: AssetId,
        record_revision: u64,
        dimension: NeutralTextureDimensionV1,
        extent: [u32; 3],
        array_layers: u32,
        color_space: NeutralTextureColorSpaceV1,
        alpha_semantics: NeutralTextureAlphaSemanticsV1,
        texel_encoding: NeutralTexelEncodingV1,
        mip_levels: Vec<NeutralTextureMipLevelV1>,
    ) -> Result<Self, RenderContentContractError> {
        validate_schema_ref(&schema_ref, NEUTRAL_TEXTURE_SCHEMA_ID)?;
        if record_revision == 0 {
            return Err(RenderContentContractError::ZeroRevision);
        }
        let value = Self {
            schema_ref,
            asset_id,
            record_revision,
            dimension,
            extent,
            array_layers,
            color_space,
            alpha_semantics,
            texel_encoding,
            mip_levels,
        };
        value.validate()?;
        Ok(value)
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
    pub const fn dimension(&self) -> NeutralTextureDimensionV1 {
        self.dimension
    }

    #[must_use]
    pub const fn extent(&self) -> [u32; 3] {
        self.extent
    }

    #[must_use]
    pub const fn array_layers(&self) -> u32 {
        self.array_layers
    }

    #[must_use]
    pub const fn color_space(&self) -> NeutralTextureColorSpaceV1 {
        self.color_space
    }

    #[must_use]
    pub const fn alpha_semantics(&self) -> NeutralTextureAlphaSemanticsV1 {
        self.alpha_semantics
    }

    #[must_use]
    pub const fn texel_encoding(&self) -> NeutralTexelEncodingV1 {
        self.texel_encoding
    }

    #[must_use]
    pub fn mip_levels(&self) -> &[NeutralTextureMipLevelV1] {
        &self.mip_levels
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        self.validate()?;
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            NEUTRAL_TEXTURE_SCHEMA_ID,
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
                CanonicalField::new(5, CANONICAL_TYPE_U8, vec![self.dimension as u8]),
                CanonicalField::new(6, CANONICAL_TYPE_BYTES, encode_extent(self.extent).to_vec()),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_U32,
                    self.array_layers.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(8, CANONICAL_TYPE_U8, vec![self.color_space as u8]),
                CanonicalField::new(9, CANONICAL_TYPE_U8, vec![self.alpha_semantics as u8]),
                CanonicalField::new(10, CANONICAL_TYPE_U8, vec![self.texel_encoding as u8]),
                CanonicalField::new(11, CANONICAL_TYPE_SEQUENCE, encode_mips(&self.mip_levels)?),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, NEUTRAL_TEXTURE_SCHEMA_ID, 11)?;
        let value = Self::new(
            schema_ref_from_segment(&segment, 4)?,
            asset_id_from_segment(&segment, 2)?,
            read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?,
            NeutralTextureDimensionV1::from_tag(read_u8(field(&segment, 5, CANONICAL_TYPE_U8)?)?)?,
            decode_extent(field(&segment, 6, CANONICAL_TYPE_BYTES)?)?,
            read_u32(field(&segment, 7, CANONICAL_TYPE_U32)?)?,
            NeutralTextureColorSpaceV1::from_tag(read_u8(field(&segment, 8, CANONICAL_TYPE_U8)?)?)?,
            NeutralTextureAlphaSemanticsV1::from_tag(read_u8(field(
                &segment,
                9,
                CANONICAL_TYPE_U8,
            )?)?)?,
            NeutralTexelEncodingV1::from_tag(read_u8(field(&segment, 10, CANONICAL_TYPE_U8)?)?)?,
            decode_mips(field(&segment, 11, CANONICAL_TYPE_SEQUENCE)?, limits)?,
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

    fn validate(&self) -> Result<(), RenderContentContractError> {
        validate_schema_ref(&self.schema_ref, NEUTRAL_TEXTURE_SCHEMA_ID)?;
        if self.array_layers == 0 || self.array_layers > MAX_ARRAY_LAYERS {
            return Err(RenderContentContractError::InvalidTextureExtent);
        }
        validate_base_extent(self.dimension, self.extent, self.array_layers)?;
        if self.mip_levels.is_empty() {
            return Err(RenderContentContractError::InvalidTextureData);
        }
        ensure_limit(self.mip_levels.len(), MAX_MIP_LEVELS)?;
        for (level, mip) in self.mip_levels.iter().enumerate() {
            let expected_extent = mip_extent(self.extent, level);
            if mip.extent != expected_extent {
                return Err(RenderContentContractError::InvalidTextureExtent);
            }
            let expected_length = texel_byte_length(
                expected_extent,
                self.array_layers,
                self.dimension,
                self.texel_encoding,
            )?;
            if mip.texels.len() != expected_length {
                return Err(RenderContentContractError::InvalidTextureData);
            }
            if self.texel_encoding == NeutralTexelEncodingV1::Rgba16FloatCanonical {
                validate_binary16_bytes(&mip.texels)?;
            }
            validate_alpha_semantics(self.texel_encoding, self.alpha_semantics, &mip.texels)?;
        }
        Ok(())
    }
}

fn validate_alpha_semantics(
    encoding: NeutralTexelEncodingV1,
    semantics: NeutralTextureAlphaSemanticsV1,
    texels: &[u8],
) -> Result<(), RenderContentContractError> {
    match encoding {
        NeutralTexelEncodingV1::R8Unorm | NeutralTexelEncodingV1::Rg8Unorm => {
            if semantics != NeutralTextureAlphaSemanticsV1::Opaque {
                return Err(RenderContentContractError::InvalidTextureData);
            }
        }
        NeutralTexelEncodingV1::Rgba8Unorm => {
            for texel in texels.chunks_exact(4) {
                validate_alpha_sample(
                    [
                        u16::from(texel[0]),
                        u16::from(texel[1]),
                        u16::from(texel[2]),
                    ],
                    u16::from(texel[3]),
                    u16::from(u8::MAX),
                    semantics,
                )?;
            }
        }
        NeutralTexelEncodingV1::Rgba16Unorm => {
            for texel in texels.chunks_exact(8) {
                validate_alpha_sample(
                    [
                        u16::from_le_bytes([texel[0], texel[1]]),
                        u16::from_le_bytes([texel[2], texel[3]]),
                        u16::from_le_bytes([texel[4], texel[5]]),
                    ],
                    u16::from_le_bytes([texel[6], texel[7]]),
                    u16::MAX,
                    semantics,
                )?;
            }
        }
        NeutralTexelEncodingV1::Rgba16FloatCanonical => {
            for texel in texels.chunks_exact(8) {
                validate_alpha_sample(
                    [
                        u16::from_le_bytes([texel[0], texel[1]]),
                        u16::from_le_bytes([texel[2], texel[3]]),
                        u16::from_le_bytes([texel[4], texel[5]]),
                    ],
                    u16::from_le_bytes([texel[6], texel[7]]),
                    0x3c00,
                    semantics,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_alpha_sample(
    rgb: [u16; 3],
    alpha: u16,
    opaque_alpha: u16,
    semantics: NeutralTextureAlphaSemanticsV1,
) -> Result<(), RenderContentContractError> {
    let valid = match semantics {
        NeutralTextureAlphaSemanticsV1::Opaque => alpha == opaque_alpha,
        NeutralTextureAlphaSemanticsV1::Straight => alpha != 0 || rgb == [0; 3],
    };
    if valid {
        Ok(())
    } else {
        Err(RenderContentContractError::InvalidTextureData)
    }
}

fn validate_base_extent(
    dimension: NeutralTextureDimensionV1,
    extent: [u32; 3],
    array_layers: u32,
) -> Result<(), RenderContentContractError> {
    if extent.contains(&0) {
        return Err(RenderContentContractError::InvalidTextureExtent);
    }
    let valid = match dimension {
        NeutralTextureDimensionV1::D1 => {
            extent[0] <= MAX_2D_EXTENT && extent[1] == 1 && extent[2] == 1
        }
        NeutralTextureDimensionV1::D2 => {
            extent[0] <= MAX_2D_EXTENT && extent[1] <= MAX_2D_EXTENT && extent[2] == 1
        }
        NeutralTextureDimensionV1::D3 => {
            extent.into_iter().all(|value| value <= MAX_3D_EXTENT) && array_layers == 1
        }
        NeutralTextureDimensionV1::Cube => {
            extent[0] <= MAX_2D_EXTENT && extent[0] == extent[1] && extent[2] == 1
        }
    };
    if valid {
        Ok(())
    } else {
        Err(RenderContentContractError::InvalidTextureExtent)
    }
}

fn mip_extent(base: [u32; 3], level: usize) -> [u32; 3] {
    let shift = u32::try_from(level).unwrap_or(u32::MAX);
    base.map(|value| value.checked_shr(shift).unwrap_or(0).max(1))
}

fn texel_byte_length(
    extent: [u32; 3],
    array_layers: u32,
    dimension: NeutralTextureDimensionV1,
    encoding: NeutralTexelEncodingV1,
) -> Result<usize, RenderContentContractError> {
    let face_count = if dimension == NeutralTextureDimensionV1::Cube {
        6_u64
    } else {
        1_u64
    };
    let mut length = u64::from(extent[0]);
    for value in [extent[1], extent[2], array_layers] {
        length = length
            .checked_mul(u64::from(value))
            .ok_or(RenderContentContractError::IntegerOverflow)?;
    }
    length = length
        .checked_mul(face_count)
        .and_then(|value| value.checked_mul(encoding.bytes_per_texel() as u64))
        .ok_or(RenderContentContractError::IntegerOverflow)?;
    usize::try_from(length).map_err(|_| RenderContentContractError::IntegerOverflow)
}

fn validate_binary16_bytes(bytes: &[u8]) -> Result<(), RenderContentContractError> {
    for chunk in bytes.chunks_exact(2) {
        let bits = u16::from_le_bytes(
            chunk
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidTextureData)?,
        );
        if bits == 0x8000 || bits & 0x7c00 == 0x7c00 {
            return Err(RenderContentContractError::InvalidHalfFloat);
        }
    }
    Ok(())
}

fn encode_extent(extent: [u32; 3]) -> [u8; 12] {
    let mut bytes = [0_u8; 12];
    for (index, value) in extent.into_iter().enumerate() {
        let offset = index * 4;
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_extent(bytes: &[u8]) -> Result<[u32; 3], RenderContentContractError> {
    if bytes.len() != 12 {
        return Err(RenderContentContractError::InvalidPayload);
    }
    let mut extent = [0_u32; 3];
    for (index, value) in extent.iter_mut().enumerate() {
        let offset = index * 4;
        *value = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        );
    }
    Ok(extent)
}

fn encode_mips(mips: &[NeutralTextureMipLevelV1]) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, mips.len())?;
    for mip in mips {
        bytes.extend_from_slice(&encode_extent(mip.extent));
        bytes.extend_from_slice(
            &u64::try_from(mip.texels.len())
                .map_err(|_| RenderContentContractError::IntegerOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&mip.texels);
    }
    Ok(bytes)
}

fn decode_mips(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralTextureMipLevelV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_MIP_LEVELS)?;
    let mut mips = Vec::with_capacity(count);
    for _ in 0..count {
        let extent = decode_extent(cursor.read_exact(12)?)?;
        let length = usize::try_from(cursor.read_u64()?)
            .map_err(|_| RenderContentContractError::IntegerOverflow)?;
        ensure_limit(length, limits.max_field_payload_bytes)?;
        mips.push(NeutralTextureMipLevelV1::new(
            extent,
            cursor.read_exact(length)?.to_vec(),
        ));
    }
    cursor.finish()?;
    Ok(mips)
}
