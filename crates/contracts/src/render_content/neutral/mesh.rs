use crate::canonical::{
    CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor,
    CanonicalDecodeLimits, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash};
use crate::project::{AssetRevisionRefV1, SchemaRefV1};

use super::super::codec::{
    asset_id_from_segment, ensure_limit, field, neutral_record_hash, read_count, read_u64,
    schema_ref_from_segment, validate_envelope, validate_schema_ref,
};
use super::super::{
    NEUTRAL_MESH_SCHEMA_ID, RENDER_CONTENT_OWNER_ID, RENDER_CONTENT_SCHEMA_VERSION,
    RENDER_CONTENT_SEGMENT_ID, RenderContentContractError,
};
use super::common::{AabbI64V1, POSITION_LIMIT_MICROMETRES};

const MAX_VERTICES: usize = 16_777_216;
const MAX_INDICES: usize = 50_331_648;
const MAX_PRIMITIVES: usize = 65_535;
const MAX_UV_SETS: usize = 8;
const UNIT_SNORM16_SQUARED: i64 = 32_767_i64 * 32_767_i64;
const UNIT_SNORM16_TOLERANCE: i64 = 65_535;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MeshPrimitiveTopologyV1 {
    Triangles = 1,
    Lines = 2,
    Points = 3,
}

impl MeshPrimitiveTopologyV1 {
    fn from_tag(tag: u8) -> Result<Self, RenderContentContractError> {
        match tag {
            1 => Ok(Self::Triangles),
            2 => Ok(Self::Lines),
            3 => Ok(Self::Points),
            _ => Err(RenderContentContractError::InvalidPrimitive),
        }
    }

    const fn divisor(self) -> u32 {
        match self {
            Self::Triangles => 3,
            Self::Lines => 2,
            Self::Points => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralTangentV1 {
    components_snorm16: [i16; 3],
    handedness: i8,
}

impl NeutralTangentV1 {
    pub fn new(
        components_snorm16: [i16; 3],
        handedness: i8,
    ) -> Result<Self, RenderContentContractError> {
        validate_unit_snorm16(components_snorm16)
            .map_err(|_| RenderContentContractError::InvalidTangent)?;
        if !matches!(handedness, -1 | 1) {
            return Err(RenderContentContractError::InvalidTangent);
        }
        Ok(Self {
            components_snorm16,
            handedness,
        })
    }

    #[must_use]
    pub const fn components_snorm16(&self) -> [i16; 3] {
        self.components_snorm16
    }

    #[must_use]
    pub const fn handedness(&self) -> i8 {
        self.handedness
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralMeshPrimitiveV1 {
    topology: MeshPrimitiveTopologyV1,
    first_index: u32,
    index_count: u32,
    material_slot: u32,
}

impl NeutralMeshPrimitiveV1 {
    pub fn new(
        topology: MeshPrimitiveTopologyV1,
        first_index: u32,
        index_count: u32,
        material_slot: u32,
    ) -> Result<Self, RenderContentContractError> {
        if index_count == 0 || !index_count.is_multiple_of(topology.divisor()) {
            return Err(RenderContentContractError::InvalidPrimitive);
        }
        first_index
            .checked_add(index_count)
            .ok_or(RenderContentContractError::IntegerOverflow)?;
        Ok(Self {
            topology,
            first_index,
            index_count,
            material_slot,
        })
    }

    #[must_use]
    pub const fn topology(&self) -> MeshPrimitiveTopologyV1 {
        self.topology
    }

    #[must_use]
    pub const fn first_index(&self) -> u32 {
        self.first_index
    }

    #[must_use]
    pub const fn index_count(&self) -> u32 {
        self.index_count
    }

    #[must_use]
    pub const fn material_slot(&self) -> u32 {
        self.material_slot
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralMeshV1 {
    schema_ref: SchemaRefV1,
    asset_id: AssetId,
    record_revision: u64,
    bounds: AabbI64V1,
    positions_micrometres: Vec<[i64; 3]>,
    normals_snorm16: Option<Vec<[i16; 3]>>,
    tangents_snorm16: Option<Vec<NeutralTangentV1>>,
    texcoords_q16_16: Vec<Vec<[i32; 2]>>,
    indices: Vec<u32>,
    primitives: Vec<NeutralMeshPrimitiveV1>,
}

impl NeutralMeshV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        schema_ref: SchemaRefV1,
        asset_id: AssetId,
        record_revision: u64,
        bounds: AabbI64V1,
        positions_micrometres: Vec<[i64; 3]>,
        normals_snorm16: Option<Vec<[i16; 3]>>,
        tangents_snorm16: Option<Vec<NeutralTangentV1>>,
        texcoords_q16_16: Vec<Vec<[i32; 2]>>,
        indices: Vec<u32>,
        mut primitives: Vec<NeutralMeshPrimitiveV1>,
    ) -> Result<Self, RenderContentContractError> {
        validate_schema_ref(&schema_ref, NEUTRAL_MESH_SCHEMA_ID)?;
        if record_revision == 0 {
            return Err(RenderContentContractError::ZeroRevision);
        }
        primitives.sort_by_key(|primitive| primitive.first_index);
        let value = Self {
            schema_ref,
            asset_id,
            record_revision,
            bounds,
            positions_micrometres,
            normals_snorm16,
            tangents_snorm16,
            texcoords_q16_16,
            indices,
            primitives,
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
    pub const fn bounds(&self) -> AabbI64V1 {
        self.bounds
    }

    #[must_use]
    pub fn positions_micrometres(&self) -> &[[i64; 3]] {
        &self.positions_micrometres
    }

    #[must_use]
    pub fn normals_snorm16(&self) -> Option<&[[i16; 3]]> {
        self.normals_snorm16.as_deref()
    }

    #[must_use]
    pub fn tangents_snorm16(&self) -> Option<&[NeutralTangentV1]> {
        self.tangents_snorm16.as_deref()
    }

    #[must_use]
    pub fn texcoords_q16_16(&self) -> &[Vec<[i32; 2]>] {
        &self.texcoords_q16_16
    }

    #[must_use]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    #[must_use]
    pub fn primitives(&self) -> &[NeutralMeshPrimitiveV1] {
        &self.primitives
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        self.validate()?;
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            NEUTRAL_MESH_SCHEMA_ID,
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
                CanonicalField::new(5, CANONICAL_TYPE_STRUCT, self.bounds.encode()),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_positions(&self.positions_micrometres)?,
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_normals(self.normals_snorm16.as_deref())?,
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_tangents(self.tangents_snorm16.as_deref())?,
                ),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_texcoords(&self.texcoords_q16_16)?,
                ),
                CanonicalField::new(10, CANONICAL_TYPE_SEQUENCE, encode_indices(&self.indices)?),
                CanonicalField::new(
                    11,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_primitives(&self.primitives)?,
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, NEUTRAL_MESH_SCHEMA_ID, 11)?;
        let value = Self::new(
            schema_ref_from_segment(&segment, 4)?,
            asset_id_from_segment(&segment, 2)?,
            read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?,
            AabbI64V1::decode(field(&segment, 5, CANONICAL_TYPE_STRUCT)?)?,
            decode_positions(field(&segment, 6, CANONICAL_TYPE_SEQUENCE)?, limits)?,
            decode_normals(field(&segment, 7, CANONICAL_TYPE_OPTIONAL)?, limits)?,
            decode_tangents(field(&segment, 8, CANONICAL_TYPE_OPTIONAL)?, limits)?,
            decode_texcoords(field(&segment, 9, CANONICAL_TYPE_SEQUENCE)?, limits)?,
            decode_indices(field(&segment, 10, CANONICAL_TYPE_SEQUENCE)?, limits)?,
            decode_primitives(field(&segment, 11, CANONICAL_TYPE_SEQUENCE)?, limits)?,
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
        validate_schema_ref(&self.schema_ref, NEUTRAL_MESH_SCHEMA_ID)?;
        ensure_limit(self.positions_micrometres.len(), MAX_VERTICES)?;
        ensure_limit(self.indices.len(), MAX_INDICES)?;
        ensure_limit(self.primitives.len(), MAX_PRIMITIVES)?;
        ensure_limit(self.texcoords_q16_16.len(), MAX_UV_SETS)?;
        if self.positions_micrometres.is_empty()
            || self.indices.is_empty()
            || self.primitives.is_empty()
        {
            return Err(RenderContentContractError::EmptyMesh);
        }
        for position in &self.positions_micrometres {
            if position.iter().any(|value| {
                !(-POSITION_LIMIT_MICROMETRES..=POSITION_LIMIT_MICROMETRES).contains(value)
            }) || !self.bounds.contains(*position)
            {
                return Err(RenderContentContractError::PositionOutsideBounds);
            }
        }
        if let Some(normals) = &self.normals_snorm16 {
            validate_stream_length(normals.len(), self.positions_micrometres.len())?;
            for normal in normals {
                validate_unit_snorm16(*normal)?;
            }
        }
        if let Some(tangents) = &self.tangents_snorm16 {
            validate_stream_length(tangents.len(), self.positions_micrometres.len())?;
        }
        for set in &self.texcoords_q16_16 {
            validate_stream_length(set.len(), self.positions_micrometres.len())?;
        }
        let vertex_count = u32::try_from(self.positions_micrometres.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?;
        if self.indices.iter().any(|index| *index >= vertex_count) {
            return Err(RenderContentContractError::InvalidIndex);
        }
        let mut expected_first = 0_u32;
        for primitive in &self.primitives {
            if primitive.first_index != expected_first
                || !primitive
                    .index_count
                    .is_multiple_of(primitive.topology.divisor())
            {
                return Err(RenderContentContractError::InvalidPrimitive);
            }
            expected_first = expected_first
                .checked_add(primitive.index_count)
                .ok_or(RenderContentContractError::IntegerOverflow)?;
        }
        if usize::try_from(expected_first)
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            != self.indices.len()
        {
            return Err(RenderContentContractError::InvalidPrimitive);
        }
        Ok(())
    }
}

fn validate_stream_length(
    actual: usize,
    expected: usize,
) -> Result<(), RenderContentContractError> {
    if actual == expected {
        Ok(())
    } else {
        Err(RenderContentContractError::AttributeLengthMismatch)
    }
}

fn validate_unit_snorm16(value: [i16; 3]) -> Result<(), RenderContentContractError> {
    if value.contains(&i16::MIN) {
        return Err(RenderContentContractError::InvalidNormal);
    }
    let squared: i64 = value
        .into_iter()
        .map(|component| i64::from(component) * i64::from(component))
        .sum();
    if (UNIT_SNORM16_SQUARED - UNIT_SNORM16_TOLERANCE
        ..=UNIT_SNORM16_SQUARED + UNIT_SNORM16_TOLERANCE)
        .contains(&squared)
    {
        Ok(())
    } else {
        Err(RenderContentContractError::InvalidNormal)
    }
}

fn encode_positions(values: &[[i64; 3]]) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    super::super::codec::extend_count(&mut bytes, values.len())?;
    for value in values {
        for component in value {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    Ok(bytes)
}

fn decode_positions(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<[i64; 3]>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_VERTICES)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut value = [0_i64; 3];
        for component in &mut value {
            *component = i64::from_le_bytes(
                cursor
                    .read_exact(8)?
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
        }
        values.push(value);
    }
    cursor.finish()?;
    Ok(values)
}

fn encode_normals(values: Option<&[[i16; 3]]>) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = vec![u8::from(values.is_some())];
    if let Some(values) = values {
        super::super::codec::extend_count(&mut bytes, values.len())?;
        for value in values {
            for component in value {
                bytes.extend_from_slice(&component.to_le_bytes());
            }
        }
    }
    Ok(bytes)
}

fn decode_normals(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<Vec<[i16; 3]>>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let present = cursor.read_u8()?;
    if present == 0 {
        cursor.finish()?;
        return Ok(None);
    }
    if present != 1 {
        return Err(RenderContentContractError::InvalidPayload);
    }
    let count = read_count(&mut cursor, limits, MAX_VERTICES)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut value = [0_i16; 3];
        for component in &mut value {
            *component = i16::from_le_bytes(
                cursor
                    .read_exact(2)?
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
        }
        values.push(value);
    }
    cursor.finish()?;
    Ok(Some(values))
}

fn encode_tangents(
    values: Option<&[NeutralTangentV1]>,
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = vec![u8::from(values.is_some())];
    if let Some(values) = values {
        super::super::codec::extend_count(&mut bytes, values.len())?;
        for value in values {
            for component in value.components_snorm16 {
                bytes.extend_from_slice(&component.to_le_bytes());
            }
            bytes.extend_from_slice(&value.handedness.to_le_bytes());
        }
    }
    Ok(bytes)
}

fn decode_tangents(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<Vec<NeutralTangentV1>>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let present = cursor.read_u8()?;
    if present == 0 {
        cursor.finish()?;
        return Ok(None);
    }
    if present != 1 {
        return Err(RenderContentContractError::InvalidPayload);
    }
    let count = read_count(&mut cursor, limits, MAX_VERTICES)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut components = [0_i16; 3];
        for component in &mut components {
            *component = i16::from_le_bytes(
                cursor
                    .read_exact(2)?
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
        }
        values.push(NeutralTangentV1::new(
            components,
            i8::from_le_bytes([cursor.read_u8()?]),
        )?);
    }
    cursor.finish()?;
    Ok(Some(values))
}

fn encode_texcoords(sets: &[Vec<[i32; 2]>]) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    super::super::codec::extend_count(&mut bytes, sets.len())?;
    for set in sets {
        super::super::codec::extend_count(&mut bytes, set.len())?;
        for value in set {
            for component in value {
                bytes.extend_from_slice(&component.to_le_bytes());
            }
        }
    }
    Ok(bytes)
}

fn decode_texcoords(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<[i32; 2]>>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let set_count = read_count(&mut cursor, limits, MAX_UV_SETS)?;
    let mut sets = Vec::with_capacity(set_count);
    for _ in 0..set_count {
        let count = read_count(&mut cursor, limits, MAX_VERTICES)?;
        let mut set = Vec::with_capacity(count);
        for _ in 0..count {
            let u = i32::from_le_bytes(
                cursor
                    .read_exact(4)?
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
            let v = i32::from_le_bytes(
                cursor
                    .read_exact(4)?
                    .try_into()
                    .map_err(|_| RenderContentContractError::InvalidPayload)?,
            );
            set.push([u, v]);
        }
        sets.push(set);
    }
    cursor.finish()?;
    Ok(sets)
}

fn encode_indices(values: &[u32]) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    super::super::codec::extend_count(&mut bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

fn decode_indices(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<u32>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_INDICES)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(cursor.read_u32()?);
    }
    cursor.finish()?;
    Ok(values)
}

fn encode_primitives(
    values: &[NeutralMeshPrimitiveV1],
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    super::super::codec::extend_count(&mut bytes, values.len())?;
    for value in values {
        bytes.push(value.topology as u8);
        bytes.extend_from_slice(&value.first_index.to_le_bytes());
        bytes.extend_from_slice(&value.index_count.to_le_bytes());
        bytes.extend_from_slice(&value.material_slot.to_le_bytes());
    }
    Ok(bytes)
}

fn decode_primitives(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralMeshPrimitiveV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, MAX_PRIMITIVES)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(NeutralMeshPrimitiveV1::new(
            MeshPrimitiveTopologyV1::from_tag(cursor.read_u8()?)?,
            cursor.read_u32()?,
            cursor.read_u32()?,
            cursor.read_u32()?,
        )?);
    }
    cursor.finish()?;
    Ok(values)
}
