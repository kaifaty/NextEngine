use std::collections::BTreeSet;

use crate::canonical::{CanonicalCursor, CanonicalDecodeLimits};
use crate::ids::SchemaId;

use super::super::RenderContentContractError;
use super::super::codec::{ensure_limit, extend_count, read_count};
use super::{decode_id, encode_id};

pub const POSE_CORRECTIVE_MAX_RECORDS_V1: usize = 64;
pub const POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1: usize = 1_048_576;
const POSE_CORRECTIVE_MAX_DRIVER_DELTA_MICROMETRES_V1: i64 = 2_000_000;
const POSE_CORRECTIVE_MAX_VERTEX_DELTA_MICROMETRES_V1: i64 = 500_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PoseCorrectiveDriverAxisV1 {
    X = 1,
    Y = 2,
    Z = 3,
}

impl PoseCorrectiveDriverAxisV1 {
    pub(super) fn from_tag(value: u8) -> Result<Self, RenderContentContractError> {
        match value {
            1 => Ok(Self::X),
            2 => Ok(Self::Y),
            3 => Ok(Self::Z),
            _ => Err(RenderContentContractError::InvalidPoseCorrective),
        }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PoseCorrectiveLodClassV1 {
    Essential = 1,
    Detail = 2,
}

impl PoseCorrectiveLodClassV1 {
    pub(super) fn from_tag(value: u8) -> Result<Self, RenderContentContractError> {
        match value {
            1 => Ok(Self::Essential),
            2 => Ok(Self::Detail),
            _ => Err(RenderContentContractError::InvalidPoseCorrective),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralPoseCorrectiveVertexDeltaV1 {
    pub vertex_index: u32,
    pub delta_micrometres: [i64; 3],
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NeutralPoseCorrectiveV1 {
    corrective_id: SchemaId,
    driver_render_joint_id: SchemaId,
    driver_axis: PoseCorrectiveDriverAxisV1,
    activation_start_delta_micrometres: i64,
    activation_full_delta_micrometres: i64,
    lod_class: PoseCorrectiveLodClassV1,
    vertex_deltas: Vec<NeutralPoseCorrectiveVertexDeltaV1>,
}

impl NeutralPoseCorrectiveV1 {
    pub fn new(
        corrective_id: SchemaId,
        driver_render_joint_id: SchemaId,
        driver_axis: PoseCorrectiveDriverAxisV1,
        activation_start_delta_micrometres: i64,
        activation_full_delta_micrometres: i64,
        lod_class: PoseCorrectiveLodClassV1,
        mut vertex_deltas: Vec<NeutralPoseCorrectiveVertexDeltaV1>,
    ) -> Result<Self, RenderContentContractError> {
        vertex_deltas.sort_by_key(|delta| delta.vertex_index);
        if activation_start_delta_micrometres == activation_full_delta_micrometres
            || activation_start_delta_micrometres.unsigned_abs()
                > POSE_CORRECTIVE_MAX_DRIVER_DELTA_MICROMETRES_V1 as u64
            || activation_full_delta_micrometres.unsigned_abs()
                > POSE_CORRECTIVE_MAX_DRIVER_DELTA_MICROMETRES_V1 as u64
            || vertex_deltas.is_empty()
            || vertex_deltas.len() > POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1
            || vertex_deltas
                .windows(2)
                .any(|pair| pair[0].vertex_index == pair[1].vertex_index)
            || vertex_deltas.iter().any(|delta| {
                delta.delta_micrometres == [0; 3]
                    || delta.delta_micrometres.iter().any(|value| {
                        value.unsigned_abs()
                            > POSE_CORRECTIVE_MAX_VERTEX_DELTA_MICROMETRES_V1 as u64
                    })
            })
        {
            return Err(RenderContentContractError::InvalidPoseCorrective);
        }
        Ok(Self {
            corrective_id,
            driver_render_joint_id,
            driver_axis,
            activation_start_delta_micrometres,
            activation_full_delta_micrometres,
            lod_class,
            vertex_deltas,
        })
    }

    #[must_use]
    pub const fn corrective_id(&self) -> &SchemaId {
        &self.corrective_id
    }

    #[must_use]
    pub const fn driver_render_joint_id(&self) -> &SchemaId {
        &self.driver_render_joint_id
    }

    #[must_use]
    pub const fn driver_axis(&self) -> PoseCorrectiveDriverAxisV1 {
        self.driver_axis
    }

    #[must_use]
    pub const fn activation_start_delta_micrometres(&self) -> i64 {
        self.activation_start_delta_micrometres
    }

    #[must_use]
    pub const fn activation_full_delta_micrometres(&self) -> i64 {
        self.activation_full_delta_micrometres
    }

    #[must_use]
    pub const fn lod_class(&self) -> PoseCorrectiveLodClassV1 {
        self.lod_class
    }

    #[must_use]
    pub fn vertex_deltas(&self) -> &[NeutralPoseCorrectiveVertexDeltaV1] {
        &self.vertex_deltas
    }
}

pub(super) fn canonicalize_pose_correctives(
    mut correctives: Vec<NeutralPoseCorrectiveV1>,
    render_joint_ids: &BTreeSet<&SchemaId>,
    vertex_count: usize,
) -> Result<Vec<NeutralPoseCorrectiveV1>, RenderContentContractError> {
    ensure_limit(correctives.len(), POSE_CORRECTIVE_MAX_RECORDS_V1)?;
    if correctives.is_empty() {
        return Err(RenderContentContractError::InvalidPoseCorrective);
    }
    correctives.sort_by(|left, right| left.corrective_id.cmp(&right.corrective_id));
    if correctives
        .windows(2)
        .any(|pair| pair[0].corrective_id == pair[1].corrective_id)
        || correctives.iter().any(|corrective| {
            !render_joint_ids.contains(&corrective.driver_render_joint_id)
                || corrective.vertex_deltas.iter().any(|delta| {
                    usize::try_from(delta.vertex_index).map_or(true, |index| index >= vertex_count)
                })
        })
        || !correctives
            .iter()
            .any(|corrective| corrective.lod_class == PoseCorrectiveLodClassV1::Essential)
    {
        return Err(RenderContentContractError::InvalidPoseCorrective);
    }
    let total_deltas = correctives.iter().try_fold(0_usize, |total, corrective| {
        total.checked_add(corrective.vertex_deltas.len())
    });
    if total_deltas.is_none_or(|count| count > POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1) {
        return Err(RenderContentContractError::InvalidPoseCorrective);
    }
    Ok(correctives)
}

pub(super) fn encode_pose_correctives(
    correctives: &[NeutralPoseCorrectiveV1],
) -> Result<Vec<u8>, RenderContentContractError> {
    let mut bytes = Vec::new();
    extend_count(&mut bytes, correctives.len())?;
    for corrective in correctives {
        encode_id(&mut bytes, &corrective.corrective_id)?;
        encode_id(&mut bytes, &corrective.driver_render_joint_id)?;
        bytes.push(corrective.driver_axis as u8);
        bytes.push(corrective.lod_class as u8);
        bytes.extend_from_slice(&corrective.activation_start_delta_micrometres.to_le_bytes());
        bytes.extend_from_slice(&corrective.activation_full_delta_micrometres.to_le_bytes());
        extend_count(&mut bytes, corrective.vertex_deltas.len())?;
        for delta in &corrective.vertex_deltas {
            bytes.extend_from_slice(&delta.vertex_index.to_le_bytes());
            for value in delta.delta_micrometres {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    Ok(bytes)
}

pub(super) fn decode_pose_correctives(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<NeutralPoseCorrectiveV1>, RenderContentContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = read_count(&mut cursor, limits, POSE_CORRECTIVE_MAX_RECORDS_V1)?;
    let mut correctives = Vec::with_capacity(count);
    let mut total_delta_count = 0_usize;
    for _ in 0..count {
        let corrective_id = decode_id(&mut cursor, limits)?;
        let driver_render_joint_id = decode_id(&mut cursor, limits)?;
        let driver_axis = PoseCorrectiveDriverAxisV1::from_tag(cursor.read_u8()?)?;
        let lod_class = PoseCorrectiveLodClassV1::from_tag(cursor.read_u8()?)?;
        let activation_start_delta_micrometres = read_i64(&mut cursor)?;
        let activation_full_delta_micrometres = read_i64(&mut cursor)?;
        let delta_count = read_count(&mut cursor, limits, POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1)?;
        total_delta_count = total_delta_count
            .checked_add(delta_count)
            .filter(|count| *count <= POSE_CORRECTIVE_MAX_VERTEX_DELTAS_V1)
            .ok_or(RenderContentContractError::InvalidPoseCorrective)?;
        let mut vertex_deltas = Vec::with_capacity(delta_count);
        for _ in 0..delta_count {
            vertex_deltas.push(NeutralPoseCorrectiveVertexDeltaV1 {
                vertex_index: u32::from_le_bytes(
                    cursor
                        .read_exact(4)?
                        .try_into()
                        .map_err(|_| RenderContentContractError::InvalidPayload)?,
                ),
                delta_micrometres: [
                    read_i64(&mut cursor)?,
                    read_i64(&mut cursor)?,
                    read_i64(&mut cursor)?,
                ],
            });
        }
        correctives.push(NeutralPoseCorrectiveV1::new(
            corrective_id,
            driver_render_joint_id,
            driver_axis,
            activation_start_delta_micrometres,
            activation_full_delta_micrometres,
            lod_class,
            vertex_deltas,
        )?);
    }
    cursor.finish()?;
    Ok(correctives)
}

fn read_i64(cursor: &mut CanonicalCursor<'_>) -> Result<i64, RenderContentContractError> {
    Ok(i64::from_le_bytes(
        cursor
            .read_exact(8)?
            .try_into()
            .map_err(|_| RenderContentContractError::InvalidPayload)?,
    ))
}
