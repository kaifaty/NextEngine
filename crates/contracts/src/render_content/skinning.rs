use std::collections::{BTreeMap, BTreeSet};

use crate::animation_content::{NeutralSkeletonV1, NeutralTransformV1};
use crate::body::BodySchemaAssetV1;
use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CanonicalCursor,
    CanonicalDecodeLimits, CanonicalField, decode_canonical_segment, encode_canonical_segment,
};
use crate::ids::{AssetId, ContentHash, SchemaId};
use crate::physics::PhysicsPoseV1;
use crate::project::{AssetRevisionRefV1, SchemaRefV1};

use super::codec::{
    asset_id_from_segment, decode_asset_revision, encode_asset_revision, ensure_limit,
    ensure_nonzero_hash, extend_count, field, neutral_record_hash, read_count, read_u32, read_u64,
    schema_ref_from_segment, validate_envelope, validate_schema_ref,
};
use super::{
    NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID, RENDER_CONTENT_OWNER_ID,
    RENDER_CONTENT_SCHEMA_VERSION, RENDER_CONTENT_SEGMENT_ID, RenderContentContractError,
};

pub const BASE_SKINNING_MAX_RENDER_JOINTS_V1: usize = 256;
pub const BASE_SKINNING_MAX_INFLUENCES_PER_VERTEX_V1: usize = 4;
const BASE_SKINNING_MAX_VERTICES_V1: usize = 16_777_216;
const BASE_SKINNING_MAX_INSTANCES_PER_FRAME_V1: u32 = 4_096;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BaseSkinningMethodV1 {
    LinearBlend = 1,
}

impl BaseSkinningMethodV1 {
    fn from_tag(value: u8) -> Result<Self, RenderContentContractError> {
        match value {
            1 => Ok(Self::LinearBlend),
            _ => Err(RenderContentContractError::InvalidSkinningProfile),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BaseSkinningFallbackV1 {
    BindPose = 1,
}

impl BaseSkinningFallbackV1 {
    fn from_tag(value: u8) -> Result<Self, RenderContentContractError> {
        match value {
            1 => Ok(Self::BindPose),
            _ => Err(RenderContentContractError::InvalidSkinningProfile),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralRenderJointV1 {
    pub render_joint_id: SchemaId,
    pub parent_render_joint_id: Option<SchemaId>,
    pub animation_joint_id: SchemaId,
    pub body_semantic_id: SchemaId,
    pub bind_transform: NeutralTransformV1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralSkinInfluenceV1 {
    pub render_joint_id: SchemaId,
    pub weight_unorm16: u16,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralSkinVertexV1 {
    influences: Vec<NeutralSkinInfluenceV1>,
}

impl NeutralSkinVertexV1 {
    pub fn new(
        mut influences: Vec<NeutralSkinInfluenceV1>,
    ) -> Result<Self, RenderContentContractError> {
        influences.sort_by(|left, right| left.render_joint_id.cmp(&right.render_joint_id));
        if influences.is_empty()
            || influences.len() > BASE_SKINNING_MAX_INFLUENCES_PER_VERTEX_V1
            || influences
                .windows(2)
                .any(|pair| pair[0].render_joint_id == pair[1].render_joint_id)
            || influences
                .iter()
                .any(|influence| influence.weight_unorm16 == 0)
            || influences
                .iter()
                .map(|influence| u32::from(influence.weight_unorm16))
                .sum::<u32>()
                != u32::from(u16::MAX)
        {
            return Err(RenderContentContractError::InvalidSkinWeights);
        }
        Ok(Self { influences })
    }

    #[must_use]
    pub fn influences(&self) -> &[NeutralSkinInfluenceV1] {
        &self.influences
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralBaseSkinningProfileV1 {
    schema_ref: SchemaRefV1,
    asset_id: AssetId,
    record_revision: u64,
    mesh_revision: AssetRevisionRefV1,
    skeleton_revision: AssetRevisionRefV1,
    body_schema_revision: AssetRevisionRefV1,
    mesh_origin_in_skeleton_micrometres: [i64; 3],
    method: BaseSkinningMethodV1,
    fallback: BaseSkinningFallbackV1,
    max_instances_per_frame: u32,
    render_joints: Vec<NeutralRenderJointV1>,
    vertices: Vec<NeutralSkinVertexV1>,
}

impl NeutralBaseSkinningProfileV1 {
    #[allow(
        clippy::too_many_arguments,
        reason = "the exact base-skinning content closure keeps every source revision explicit"
    )]
    pub fn new(
        schema_ref: SchemaRefV1,
        asset_id: AssetId,
        record_revision: u64,
        mesh_revision: AssetRevisionRefV1,
        skeleton_revision: AssetRevisionRefV1,
        body_schema_revision: AssetRevisionRefV1,
        mesh_origin_in_skeleton_micrometres: [i64; 3],
        method: BaseSkinningMethodV1,
        fallback: BaseSkinningFallbackV1,
        max_instances_per_frame: u32,
        render_joints: Vec<NeutralRenderJointV1>,
        vertices: Vec<NeutralSkinVertexV1>,
    ) -> Result<Self, RenderContentContractError> {
        validate_schema_ref(&schema_ref, NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID)?;
        if record_revision == 0 {
            return Err(RenderContentContractError::ZeroRevision);
        }
        for revision in [mesh_revision, skeleton_revision, body_schema_revision] {
            ensure_nonzero_hash(revision.record_sha256)?;
        }
        let render_joints = canonicalize_render_joints(render_joints)?;
        ensure_limit(vertices.len(), BASE_SKINNING_MAX_VERTICES_V1)?;
        if vertices.is_empty()
            || max_instances_per_frame == 0
            || max_instances_per_frame > BASE_SKINNING_MAX_INSTANCES_PER_FRAME_V1
        {
            return Err(RenderContentContractError::InvalidSkinningProfile);
        }
        let joint_ids = render_joints
            .iter()
            .map(|joint| &joint.render_joint_id)
            .collect::<BTreeSet<_>>();
        for vertex in &vertices {
            for influence in vertex.influences() {
                if !joint_ids.contains(&influence.render_joint_id) {
                    return Err(RenderContentContractError::InvalidSkinWeights);
                }
            }
        }
        Ok(Self {
            schema_ref,
            asset_id,
            record_revision,
            mesh_revision,
            skeleton_revision,
            body_schema_revision,
            mesh_origin_in_skeleton_micrometres,
            method,
            fallback,
            max_instances_per_frame,
            render_joints,
            vertices,
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
    pub const fn mesh_revision(&self) -> AssetRevisionRefV1 {
        self.mesh_revision
    }

    #[must_use]
    pub const fn skeleton_revision(&self) -> AssetRevisionRefV1 {
        self.skeleton_revision
    }

    #[must_use]
    pub const fn body_schema_revision(&self) -> AssetRevisionRefV1 {
        self.body_schema_revision
    }

    #[must_use]
    pub const fn mesh_origin_in_skeleton_micrometres(&self) -> [i64; 3] {
        self.mesh_origin_in_skeleton_micrometres
    }

    #[must_use]
    pub const fn method(&self) -> BaseSkinningMethodV1 {
        self.method
    }

    #[must_use]
    pub const fn fallback(&self) -> BaseSkinningFallbackV1 {
        self.fallback
    }

    #[must_use]
    pub const fn max_instances_per_frame(&self) -> u32 {
        self.max_instances_per_frame
    }

    #[must_use]
    pub fn render_joints(&self) -> &[NeutralRenderJointV1] {
        &self.render_joints
    }

    #[must_use]
    pub fn vertices(&self) -> &[NeutralSkinVertexV1] {
        &self.vertices
    }

    #[must_use]
    pub fn dependencies(&self) -> Vec<AssetRevisionRefV1> {
        let mut dependencies = vec![
            self.mesh_revision,
            self.skeleton_revision,
            self.body_schema_revision,
        ];
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    pub fn validate_against(
        &self,
        mesh: &super::NeutralMeshV1,
        skeleton: &NeutralSkeletonV1,
        body_schema: &BodySchemaAssetV1,
    ) -> Result<(), RenderContentContractError> {
        if mesh.asset_revision()? != self.mesh_revision
            || skeleton
                .asset_revision()
                .map_err(|_| RenderContentContractError::InvalidSkinningProfile)?
                != self.skeleton_revision
            || (AssetRevisionRefV1 {
                asset_id: body_schema.asset_id,
                record_sha256: body_schema
                    .record_sha256()
                    .map_err(|_| RenderContentContractError::InvalidSkinningProfile)?,
            }) != self.body_schema_revision
            || mesh.positions_micrometres().len() != self.vertices.len()
        {
            return Err(RenderContentContractError::InvalidSkinningProfile);
        }
        let animation_ids = skeleton
            .joints
            .iter()
            .map(|joint| &joint.joint_key)
            .collect::<BTreeSet<_>>();
        let body_ids = body_schema
            .body_schema
            .bodies
            .iter()
            .map(|body| &body.body_id)
            .collect::<BTreeSet<_>>();
        if self.render_joints.iter().any(|joint| {
            !animation_ids.contains(&joint.animation_joint_id)
                || !body_ids.contains(&joint.body_semantic_id)
        }) {
            return Err(RenderContentContractError::InvalidSkinningProfile);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RenderContentContractError> {
        Ok(encode_canonical_segment(
            RENDER_CONTENT_OWNER_ID,
            NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID,
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
                    CANONICAL_TYPE_STRUCT,
                    encode_asset_revision(self.mesh_revision).to_vec(),
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_STRUCT,
                    encode_asset_revision(self.skeleton_revision).to_vec(),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_STRUCT,
                    encode_asset_revision(self.body_schema_revision).to_vec(),
                ),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_3(self.mesh_origin_in_skeleton_micrometres),
                ),
                CanonicalField::new(9, CANONICAL_TYPE_BYTES, vec![self.method as u8]),
                CanonicalField::new(10, CANONICAL_TYPE_BYTES, vec![self.fallback as u8]),
                CanonicalField::new(
                    11,
                    CANONICAL_TYPE_U32,
                    self.max_instances_per_frame.to_le_bytes().to_vec(),
                ),
                CanonicalField::new(
                    12,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_render_joints(&self.render_joints)?,
                ),
                CanonicalField::new(
                    13,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_vertices(&self.vertices)?,
                ),
            ],
        )?)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, RenderContentContractError> {
        let segment = decode_canonical_segment(bytes, limits)?;
        validate_envelope(&segment, NEUTRAL_BASE_SKINNING_PROFILE_SCHEMA_ID, 13)?;
        let value = Self::new(
            schema_ref_from_segment(&segment, 4)?,
            asset_id_from_segment(&segment, 2)?,
            read_u64(field(&segment, 3, CANONICAL_TYPE_U64)?)?,
            decode_asset_revision(field(&segment, 5, CANONICAL_TYPE_STRUCT)?)?,
            decode_asset_revision(field(&segment, 6, CANONICAL_TYPE_STRUCT)?)?,
            decode_asset_revision(field(&segment, 7, CANONICAL_TYPE_STRUCT)?)?,
            decode_i64_3(field(&segment, 8, CANONICAL_TYPE_BYTES)?)?,
            BaseSkinningMethodV1::from_tag(single_byte(field(
                &segment,
                9,
                CANONICAL_TYPE_BYTES,
            )?)?)?,
            BaseSkinningFallbackV1::from_tag(single_byte(field(
                &segment,
                10,
                CANONICAL_TYPE_BYTES,
            )?)?)?,
            read_u32(field(&segment, 11, CANONICAL_TYPE_U32)?)?,
            decode_render_joints(field(&segment, 12, CANONICAL_TYPE_SEQUENCE)?, limits)?,
            decode_vertices(field(&segment, 13, CANONICAL_TYPE_SEQUENCE)?, limits)?,
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

fn canonicalize_render_joints(
    joints: Vec<NeutralRenderJointV1>,
) -> Result<Vec<NeutralRenderJointV1>, RenderContentContractError> {
    ensure_limit(joints.len(), BASE_SKINNING_MAX_RENDER_JOINTS_V1)?;
    if joints.is_empty() {
        return Err(RenderContentContractError::InvalidSkinningProfile);
    }
    let mut by_id = BTreeMap::new();
    for joint in joints {
        if joint.bind_transform.scale_q16_16 != [1 << 16; 3]
            || (PhysicsPoseV1 {
                translation_micrometres: joint.bind_transform.translation_micrometres,
                rotation_q1_30: joint.bind_transform.rotation_q1_30,
            })
            .validate()
            .is_err()
            || by_id.insert(joint.render_joint_id.clone(), joint).is_some()
        {
            return Err(RenderContentContractError::InvalidSkinningProfile);
        }
    }
    let roots = by_id
        .values()
        .filter(|joint| joint.parent_render_joint_id.is_none())
        .count();
    if roots != 1 {
        return Err(RenderContentContractError::InvalidSkinningProfile);
    }
    let mut depths = BTreeMap::new();
    for id in by_id.keys() {
        let depth = render_joint_depth(id, &by_id, &mut BTreeSet::new())?;
        depths.insert(id.clone(), depth);
    }
    let mut joints = by_id.into_values().collect::<Vec<_>>();
    joints.sort_by(|left, right| {
        depths[&left.render_joint_id]
            .cmp(&depths[&right.render_joint_id])
            .then_with(|| left.render_joint_id.cmp(&right.render_joint_id))
    });
    Ok(joints)
}

fn render_joint_depth(
    id: &SchemaId,
    joints: &BTreeMap<SchemaId, NeutralRenderJointV1>,
    visiting: &mut BTreeSet<SchemaId>,
) -> Result<usize, RenderContentContractError> {
    if !visiting.insert(id.clone()) {
        return Err(RenderContentContractError::InvalidSkinningProfile);
    }
    let joint = joints
        .get(id)
        .ok_or(RenderContentContractError::InvalidSkinningProfile)?;
    let depth = match &joint.parent_render_joint_id {
        Some(parent) => {
            if parent == id {
                return Err(RenderContentContractError::InvalidSkinningProfile);
            }
            render_joint_depth(parent, joints, visiting)?
                .checked_add(1)
                .ok_or(RenderContentContractError::IntegerOverflow)?
        }
        None => 0,
    };
    visiting.remove(id);
    Ok(depth)
}

fn encode_render_joints(
    joints: &[NeutralRenderJointV1],
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, joints.len())?;
    for joint in joints {
        encode_id(&mut bytes, &joint.render_joint_id)?;
        bytes.push(u8::from(joint.parent_render_joint_id.is_some()));
        if let Some(parent) = &joint.parent_render_joint_id {
            encode_id(&mut bytes, parent)?;
        }
        encode_id(&mut bytes, &joint.animation_joint_id)?;
        encode_id(&mut bytes, &joint.body_semantic_id)?;
        bytes.extend_from_slice(&encode_transform(joint.bind_transform));
    }
    Ok(bytes)
}

fn decode_render_joints(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralRenderJointV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, BASE_SKINNING_MAX_RENDER_JOINTS_V1)?;
    let mut joints = Vec::with_capacity(count);
    for _ in 0..count {
        let render_joint_id = decode_id(&mut cursor, limits)?;
        let parent_render_joint_id = match cursor.read_u8()? {
            0 => None,
            1 => Some(decode_id(&mut cursor, limits)?),
            _ => return Err(RenderContentContractError::InvalidPayload),
        };
        joints.push(NeutralRenderJointV1 {
            render_joint_id,
            parent_render_joint_id,
            animation_joint_id: decode_id(&mut cursor, limits)?,
            body_semantic_id: decode_id(&mut cursor, limits)?,
            bind_transform: decode_transform(&mut cursor)?,
        });
    }
    cursor.finish()?;
    Ok(joints)
}

fn encode_vertices(
    vertices: &[NeutralSkinVertexV1],
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, vertices.len())?;
    for vertex in vertices {
        bytes.push(
            u8::try_from(vertex.influences.len())
                .map_err(|_| RenderContentContractError::IntegerOverflow)?,
        );
        for influence in &vertex.influences {
            encode_id(&mut bytes, &influence.render_joint_id)?;
            bytes.extend_from_slice(&influence.weight_unorm16.to_le_bytes());
        }
    }
    Ok(bytes)
}

fn decode_vertices(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralSkinVertexV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, BASE_SKINNING_MAX_VERTICES_V1)?;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..count {
        let influence_count = usize::from(cursor.read_u8()?);
        if influence_count == 0 || influence_count > BASE_SKINNING_MAX_INFLUENCES_PER_VERTEX_V1 {
            return Err(RenderContentContractError::InvalidSkinWeights);
        }
        let mut influences = Vec::with_capacity(influence_count);
        for _ in 0..influence_count {
            influences.push(NeutralSkinInfluenceV1 {
                render_joint_id: decode_id(&mut cursor, limits)?,
                weight_unorm16: u16::from_le_bytes(
                    cursor
                        .read_exact(2)?
                        .try_into()
                        .map_err(|_| RenderContentContractError::InvalidPayload)?,
                ),
            });
        }
        vertices.push(NeutralSkinVertexV1::new(influences)?);
    }
    cursor.finish()?;
    Ok(vertices)
}

fn encode_id(bytes: &mut Vec<u8>, id: &SchemaId) -> Result<(), RenderContentContractError> {
    let value = id.as_str().as_bytes();
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| RenderContentContractError::IntegerOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}

fn decode_id(
    cursor: &mut CanonicalCursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, RenderContentContractError> {
    let bytes = cursor.read_u32_length_prefixed(limits.max_field_payload_bytes)?;
    Ok(SchemaId::new(std::str::from_utf8(bytes).map_err(
        |_| RenderContentContractError::InvalidPayload,
    )?)?)
}

fn encode_transform(transform: NeutralTransformV1) -> [u8; 40] {
    let mut bytes = [0_u8; 40];
    for (index, value) in transform.translation_micrometres.into_iter().enumerate() {
        bytes[index * 8..index * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    for (index, value) in transform.rotation_q1_30.into_iter().enumerate() {
        let start = 24 + index * 4;
        bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_transform(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<NeutralTransformV1, RenderContentContractError> {
    let bytes = cursor.read_exact(40)?;
    let mut translation_micrometres = [0_i64; 3];
    for (index, value) in translation_micrometres.iter_mut().enumerate() {
        *value = i64::from_le_bytes(
            bytes[index * 8..index * 8 + 8]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        );
    }
    let mut rotation_q1_30 = [0_i32; 4];
    for (index, value) in rotation_q1_30.iter_mut().enumerate() {
        let start = 24 + index * 4;
        *value = i32::from_le_bytes(
            bytes[start..start + 4]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        );
    }
    Ok(NeutralTransformV1 {
        translation_micrometres,
        rotation_q1_30,
        scale_q16_16: [1 << 16; 3],
    })
}

fn encode_i64_3(value: [i64; 3]) -> Vec<u8> {
    value
        .into_iter()
        .flat_map(i64::to_le_bytes)
        .collect::<Vec<_>>()
}

fn decode_i64_3(bytes: &[u8]) -> Result<[i64; 3], RenderContentContractError> {
    if bytes.len() != 24 {
        return Err(RenderContentContractError::InvalidPayload);
    }
    Ok([
        i64::from_le_bytes(
            bytes[0..8]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        ),
        i64::from_le_bytes(
            bytes[8..16]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        ),
        i64::from_le_bytes(
            bytes[16..24]
                .try_into()
                .map_err(|_| RenderContentContractError::InvalidPayload)?,
        ),
    ])
}

fn single_byte(bytes: &[u8]) -> Result<u8, RenderContentContractError> {
    bytes
        .first()
        .copied()
        .filter(|_| bytes.len() == 1)
        .ok_or(RenderContentContractError::InvalidPayload)
}
