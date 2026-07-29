use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I64, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U32, CanonicalCursor, CanonicalDecodeLimits,
    CanonicalError, CanonicalField,
};
use crate::{AssetId, ContentHash, PersistentId};

use super::codec::*;
use super::error::PhysicsContractError;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsPoseV1 {
    pub translation_micrometres: [i64; 3],
    pub rotation_q1_30: [i32; 4],
}

impl Default for PhysicsPoseV1 {
    fn default() -> Self {
        Self {
            translation_micrometres: [0; 3],
            rotation_q1_30: [0, 0, 0, 1 << 30],
        }
    }
}

impl PhysicsPoseV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        let norm = self.rotation_q1_30.iter().try_fold(0_i128, |sum, value| {
            let value = i128::from(*value);
            sum.checked_add(value * value)
        });
        if norm != Some(1_i128 << 60) {
            return Err(PhysicsContractError::InvalidRotation);
        }
        Ok(())
    }

    pub(super) fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut translation = Vec::with_capacity(24);
        for value in self.translation_micrometres {
            translation.extend_from_slice(&value.to_le_bytes());
        }
        let mut rotation = Vec::with_capacity(16);
        for value in self.rotation_q1_30 {
            rotation.extend_from_slice(&value.to_le_bytes());
        }
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_BYTES, translation),
            CanonicalField::new(2, CANONICAL_TYPE_BYTES, rotation),
        ])
    }

    pub(super) fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[(1, CANONICAL_TYPE_BYTES), (2, CANONICAL_TYPE_BYTES)],
        )?;
        let translation = &field_from(&fields, 1)?.payload;
        let rotation = &field_from(&fields, 2)?.payload;
        if translation.len() != 24 || rotation.len() != 16 {
            return Err(PhysicsContractError::FieldLength);
        }
        let value = Self {
            translation_micrometres: [
                i64::from_le_bytes(exact(&translation[0..8])?),
                i64::from_le_bytes(exact(&translation[8..16])?),
                i64::from_le_bytes(exact(&translation[16..24])?),
            ],
            rotation_q1_30: [
                i32::from_le_bytes(exact(&rotation[0..4])?),
                i32::from_le_bytes(exact(&rotation[4..8])?),
                i32::from_le_bytes(exact(&rotation[8..12])?),
                i32::from_le_bytes(exact(&rotation[12..16])?),
            ],
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsBodyIdV1 {
    pub subject_id: PersistentId,
    pub body_slot: u32,
}

impl PhysicsBodyIdV1 {
    #[must_use]
    pub fn canonical_key_bytes(self) -> [u8; 20] {
        let mut bytes = [0; 20];
        bytes[..16].copy_from_slice(self.subject_id.as_bytes());
        bytes[16..].copy_from_slice(&self.body_slot.to_le_bytes());
        bytes
    }

    pub(super) fn canonical_record(self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_id(1, self.subject_id.as_bytes()),
            field_u32(2, self.body_slot),
        ])
    }

    pub(super) fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_U32)],
        )?;
        Ok(Self {
            subject_id: PersistentId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            body_slot: u32::from_le_bytes(exact(&field_from(&fields, 2)?.payload)?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsShapeIdV1 {
    pub body_id: PhysicsBodyIdV1,
    pub shape_slot: u32,
}

impl PhysicsShapeIdV1 {
    #[must_use]
    pub fn canonical_key_bytes(self) -> [u8; 24] {
        let mut bytes = [0; 24];
        bytes[..20].copy_from_slice(&self.body_id.canonical_key_bytes());
        bytes[20..].copy_from_slice(&self.shape_slot.to_le_bytes());
        bytes
    }

    pub(super) fn canonical_record(self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u32(2, self.shape_slot),
        ])
    }

    pub(super) fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[(1, CANONICAL_TYPE_STRUCT), (2, CANONICAL_TYPE_U32)],
        )?;
        Ok(Self {
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 1)?.payload, limits)?,
            shape_slot: u32::from_le_bytes(exact(&field_from(&fields, 2)?.payload)?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsMotionKindV1 {
    Static = 1,
    Kinematic = 2,
    Dynamic = 3,
}

impl PhysicsMotionKindV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Static),
            2 => Ok(Self::Kinematic),
            3 => Ok(Self::Dynamic),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsParticipationV1 {
    Solid = 1,
    Sensor = 2,
    QueryOnly = 3,
}

impl PhysicsParticipationV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Solid),
            2 => Ok(Self::Sensor),
            3 => Ok(Self::QueryOnly),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsContactReportingV1 {
    Disabled = 1,
    BeginEnd = 2,
    BeginPersistEnd = 3,
}

impl PhysicsContactReportingV1 {
    pub(super) fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Disabled),
            2 => Ok(Self::BeginEnd),
            3 => Ok(Self::BeginPersistEnd),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsGeometryV1 {
    Box {
        half_extents_micrometres: [i64; 3],
    },
    Sphere {
        radius_micrometres: i64,
    },
    Capsule {
        radius_micrometres: i64,
        half_segment_micrometres: i64,
    },
    ConvexHull {
        collision_asset_id: AssetId,
        collision_asset_revision: u32,
        shape_index: u32,
        feature_table_hash: ContentHash,
    },
    TriangleMesh {
        collision_asset_id: AssetId,
        collision_asset_revision: u32,
        shape_index: u32,
        feature_table_hash: ContentHash,
    },
    HeightField {
        collision_asset_id: AssetId,
        collision_asset_revision: u32,
        shape_index: u32,
        feature_table_hash: ContentHash,
    },
}

impl PhysicsGeometryV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        match self {
            Self::Box {
                half_extents_micrometres,
            } => {
                validate_positive_vec3(*half_extents_micrometres)?;
            }
            Self::Sphere { radius_micrometres } => validate_positive(*radius_micrometres)?,
            Self::Capsule {
                radius_micrometres,
                half_segment_micrometres,
            } => {
                validate_positive(*radius_micrometres)?;
                if *half_segment_micrometres < 0 {
                    return Err(PhysicsContractError::InvalidDescriptor);
                }
            }
            Self::ConvexHull {
                collision_asset_revision,
                ..
            }
            | Self::TriangleMesh {
                collision_asset_revision,
                ..
            }
            | Self::HeightField {
                collision_asset_revision,
                ..
            } if *collision_asset_revision == 0 => {
                return Err(PhysicsContractError::InvalidDescriptor);
            }
            Self::ConvexHull { .. } | Self::TriangleMesh { .. } | Self::HeightField { .. } => {}
        }
        Ok(())
    }

    pub(super) fn canonical_tagged_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, record) = match self {
            Self::Box {
                half_extents_micrometres,
            } => (
                1,
                encode_struct([CanonicalField::new(
                    1,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_vec3(*half_extents_micrometres),
                )])?,
            ),
            Self::Sphere { radius_micrometres } => {
                (2, encode_struct([field_i64(1, *radius_micrometres)])?)
            }
            Self::Capsule {
                radius_micrometres,
                half_segment_micrometres,
            } => (
                3,
                encode_struct([
                    field_i64(1, *radius_micrometres),
                    field_i64(2, *half_segment_micrometres),
                ])?,
            ),
            Self::ConvexHull {
                collision_asset_id,
                collision_asset_revision,
                shape_index,
                feature_table_hash,
            } => (
                4,
                encode_asset_geometry(
                    *collision_asset_id,
                    *collision_asset_revision,
                    *shape_index,
                    *feature_table_hash,
                )?,
            ),
            Self::TriangleMesh {
                collision_asset_id,
                collision_asset_revision,
                shape_index,
                feature_table_hash,
            } => (
                5,
                encode_asset_geometry(
                    *collision_asset_id,
                    *collision_asset_revision,
                    *shape_index,
                    *feature_table_hash,
                )?,
            ),
            Self::HeightField {
                collision_asset_id,
                collision_asset_revision,
                shape_index,
                feature_table_hash,
            } => (
                6,
                encode_asset_geometry(
                    *collision_asset_id,
                    *collision_asset_revision,
                    *shape_index,
                    *feature_table_hash,
                )?,
            ),
        };
        let mut tagged = vec![tag];
        tagged.extend_from_slice(&nested(CANONICAL_TYPE_STRUCT, &record)?);
        Ok(tagged)
    }

    pub(super) fn from_tagged_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let tag = cursor.read_u8()?;
        let (nested_tag, record) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        if nested_tag != CANONICAL_TYPE_STRUCT {
            return Err(PhysicsContractError::FieldType);
        }
        let fields = decode_struct(record, limits)?;
        let geometry = match tag {
            1 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_BYTES)])?;
                Self::Box {
                    half_extents_micrometres: decode_i64_vec3(&field_from(&fields, 1)?.payload)?,
                }
            }
            2 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_I64)])?;
                Self::Sphere {
                    radius_micrometres: read_i64_fields(&fields, 1)?,
                }
            }
            3 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_I64), (2, CANONICAL_TYPE_I64)])?;
                Self::Capsule {
                    radius_micrometres: read_i64_fields(&fields, 1)?,
                    half_segment_micrometres: read_i64_fields(&fields, 2)?,
                }
            }
            4..=6 => {
                let (asset, revision, shape_index, feature_hash) = decode_asset_geometry(&fields)?;
                match tag {
                    4 => Self::ConvexHull {
                        collision_asset_id: asset,
                        collision_asset_revision: revision,
                        shape_index,
                        feature_table_hash: feature_hash,
                    },
                    5 => Self::TriangleMesh {
                        collision_asset_id: asset,
                        collision_asset_revision: revision,
                        shape_index,
                        feature_table_hash: feature_hash,
                    },
                    _ => Self::HeightField {
                        collision_asset_id: asset,
                        collision_asset_revision: revision,
                        shape_index,
                        feature_table_hash: feature_hash,
                    },
                }
            }
            other => return Err(PhysicsContractError::UnknownTag(other)),
        };
        geometry.validate()?;
        Ok(geometry)
    }
}

pub(super) fn primitive_feature_is_valid(geometry: &PhysicsGeometryV1, feature: u8) -> bool {
    match geometry {
        PhysicsGeometryV1::Box { .. } => (1..=26).contains(&feature),
        PhysicsGeometryV1::Sphere { .. } | PhysicsGeometryV1::Capsule { .. } => feature == 1,
        PhysicsGeometryV1::ConvexHull { .. }
        | PhysicsGeometryV1::TriangleMesh { .. }
        | PhysicsGeometryV1::HeightField { .. } => feature != 0,
    }
}

fn validate_positive(value: i64) -> Result<(), PhysicsContractError> {
    if !(1..=8_388_608_000_000).contains(&value) {
        return Err(PhysicsContractError::InvalidDescriptor);
    }
    Ok(())
}

fn validate_positive_vec3(values: [i64; 3]) -> Result<(), PhysicsContractError> {
    for value in values {
        validate_positive(value)?;
    }
    Ok(())
}

fn encode_asset_geometry(
    asset: AssetId,
    revision: u32,
    shape_index: u32,
    feature_hash: ContentHash,
) -> Result<Vec<u8>, CanonicalError> {
    encode_struct([
        field_id(1, asset.as_bytes()),
        field_u32(2, revision),
        field_u32(3, shape_index),
        field_hash(4, feature_hash),
    ])
}

fn decode_asset_geometry(
    fields: &[CanonicalField],
) -> Result<(AssetId, u32, u32, ContentHash), PhysicsContractError> {
    require_fields(
        fields,
        &[
            (1, CANONICAL_TYPE_ID128),
            (2, CANONICAL_TYPE_U32),
            (3, CANONICAL_TYPE_U32),
            (4, CANONICAL_TYPE_HASH256),
        ],
    )?;
    Ok((
        AssetId::from_bytes(exact(&field_from(fields, 1)?.payload)?),
        read_u32_fields(fields, 2)?,
        read_u32_fields(fields, 3)?,
        read_hash_fields(fields, 4)?,
    ))
}
