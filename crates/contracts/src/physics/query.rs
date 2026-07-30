use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128,
    CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE, CANONICAL_TYPE_STRUCT,
    CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalCursor, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    encode_canonical_segment,
};
use crate::ids::{CommandStreamId, ContentHash, PhysicsWorldId};

use super::codec::*;
use super::error::PhysicsContractError;
use super::primitives::{
    PhysicsBodyIdV1, PhysicsGeometryV1, PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeIdV1,
};
use super::{
    MAX_PHYSICS_QUERY_DISTANCE_MICROMETRES, MAX_PHYSICS_QUERY_EXCLUSIONS,
    MAX_PHYSICS_QUERY_PUBLISHED_HITS, MAX_PHYSICS_QUERY_REQUESTS_PER_BATCH, PHYSICS_OWNER_ID,
    PHYSICS_QUERY_BATCH_SCHEMA_VERSION, PHYSICS_QUERY_RESULT_SCHEMA_VERSION,
    PHYSICS_QUERY_SCHEMA_VERSION, SEGMENT_V1,
};

const QUERY_REQUEST_SCHEMA_ID: &str = "nextengine.physics-query-request";
const QUERY_FILTER_SCHEMA_ID: &str = "nextengine.physics-query-filter";
const QUERY_BATCH_SCHEMA_ID: &str = "nextengine.physics-query-batch";
const QUERY_RESULT_SCHEMA_ID: &str = "nextengine.physics-query-result";
const Q1_30_ONE: i32 = 1 << 30;
const MAX_POSITION_MICROMETRES: i64 = 8_388_608_000_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsQueryIdV1 {
    pub physics_tick: u64,
    pub query_slot: u32,
    pub issuer_stream_id: CommandStreamId,
}

impl PhysicsQueryIdV1 {
    fn canonical_record(self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u64(1, self.physics_tick),
            field_u32(2, self.query_slot),
            field_id(3, self.issuer_stream_id.as_bytes()),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_ID128),
            ],
        )?;
        Ok(Self {
            physics_tick: read_u64_fields(&fields, 1)?,
            query_slot: read_u32_fields(&fields, 2)?,
            issuer_stream_id: CommandStreamId::from_bytes(exact(&field_from(&fields, 3)?.payload)?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsSnapshotSelectorV1 {
    pub physics_tick: u64,
    pub completed_substep: u32,
    pub physics_snapshot_hash: ContentHash,
}

impl PhysicsSnapshotSelectorV1 {
    fn canonical_record(self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u64(1, self.physics_tick),
            field_u32(2, self.completed_substep),
            field_hash(3, self.physics_snapshot_hash),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_HASH256),
            ],
        )?;
        Ok(Self {
            physics_tick: read_u64_fields(&fields, 1)?,
            completed_substep: read_u32_fields(&fields, 2)?,
            physics_snapshot_hash: read_hash_fields(&fields, 3)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsQueryKindV1 {
    RayCast = 1,
    ShapeCast = 2,
    Overlap = 3,
    ClosestPoint = 4,
}

impl PhysicsQueryKindV1 {
    pub(crate) fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::RayCast),
            2 => Ok(Self::ShapeCast),
            3 => Ok(Self::Overlap),
            4 => Ok(Self::ClosestPoint),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsQueryShapeV1 {
    Primitive(PhysicsGeometryV1),
    CatalogShape {
        shape_id: PhysicsShapeIdV1,
        descriptor_revision: u32,
        descriptor_hash: ContentHash,
    },
}

impl PhysicsQueryShapeV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        match self {
            Self::Primitive(geometry) => {
                geometry.validate()?;
                if !matches!(
                    geometry,
                    PhysicsGeometryV1::Box { .. }
                        | PhysicsGeometryV1::Sphere { .. }
                        | PhysicsGeometryV1::Capsule { .. }
                ) {
                    return Err(PhysicsContractError::InvalidQuery);
                }
            }
            Self::CatalogShape {
                descriptor_revision,
                ..
            } if *descriptor_revision == 0 => return Err(PhysicsContractError::InvalidQuery),
            Self::CatalogShape { .. } => {}
        }
        Ok(())
    }

    fn canonical_tagged_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, nested_tag, payload) = match self {
            Self::Primitive(geometry) => (
                1,
                CANONICAL_TYPE_TAGGED_UNION,
                geometry.canonical_tagged_bytes()?,
            ),
            Self::CatalogShape {
                shape_id,
                descriptor_revision,
                descriptor_hash,
            } => (
                2,
                CANONICAL_TYPE_STRUCT,
                encode_struct([
                    CanonicalField::new(1, CANONICAL_TYPE_STRUCT, shape_id.canonical_record()?),
                    field_u32(2, *descriptor_revision),
                    field_hash(3, *descriptor_hash),
                ])?,
            ),
        };
        let mut bytes = vec![tag];
        bytes.extend_from_slice(&nested(nested_tag, &payload)?);
        Ok(bytes)
    }

    fn from_tagged_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let mut cursor = CanonicalCursor::new(bytes);
        let tag = cursor.read_u8()?;
        let (nested_tag, payload) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        let value = match tag {
            1 => {
                if nested_tag != CANONICAL_TYPE_TAGGED_UNION {
                    return Err(PhysicsContractError::FieldType);
                }
                Self::Primitive(PhysicsGeometryV1::from_tagged_bytes(payload, limits)?)
            }
            2 => {
                if nested_tag != CANONICAL_TYPE_STRUCT {
                    return Err(PhysicsContractError::FieldType);
                }
                let fields = decode_struct(payload, limits)?;
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_STRUCT),
                        (2, CANONICAL_TYPE_U32),
                        (3, CANONICAL_TYPE_HASH256),
                    ],
                )?;
                Self::CatalogShape {
                    shape_id: PhysicsShapeIdV1::from_record(
                        &field_from(&fields, 1)?.payload,
                        limits,
                    )?,
                    descriptor_revision: read_u32_fields(&fields, 2)?,
                    descriptor_hash: read_hash_fields(&fields, 3)?,
                }
            }
            other => return Err(PhysicsContractError::UnknownTag(other)),
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicsQueryGeometryV1 {
    RayCast {
        origin_micrometres: [i64; 3],
        unit_direction_q1_30: [i32; 3],
        maximum_distance_micrometres: i64,
    },
    ShapeCast {
        query_shape: PhysicsQueryShapeV1,
        origin_pose: PhysicsPoseV1,
        unit_direction_q1_30: [i32; 3],
        maximum_distance_micrometres: i64,
    },
    Overlap {
        query_shape: PhysicsQueryShapeV1,
        pose: PhysicsPoseV1,
    },
    ClosestPoint {
        point_micrometres: [i64; 3],
        maximum_distance_micrometres: i64,
    },
}

impl PhysicsQueryGeometryV1 {
    #[must_use]
    pub const fn kind(&self) -> PhysicsQueryKindV1 {
        match self {
            Self::RayCast { .. } => PhysicsQueryKindV1::RayCast,
            Self::ShapeCast { .. } => PhysicsQueryKindV1::ShapeCast,
            Self::Overlap { .. } => PhysicsQueryKindV1::Overlap,
            Self::ClosestPoint { .. } => PhysicsQueryKindV1::ClosestPoint,
        }
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        match self {
            Self::RayCast {
                origin_micrometres,
                unit_direction_q1_30,
                maximum_distance_micrometres,
            } => {
                validate_position(*origin_micrometres)?;
                validate_unit_direction(*unit_direction_q1_30)?;
                validate_query_distance(*maximum_distance_micrometres)?;
            }
            Self::ShapeCast {
                query_shape,
                origin_pose,
                unit_direction_q1_30,
                maximum_distance_micrometres,
            } => {
                query_shape.validate()?;
                origin_pose.validate()?;
                validate_position(origin_pose.translation_micrometres)?;
                validate_unit_direction(*unit_direction_q1_30)?;
                validate_query_distance(*maximum_distance_micrometres)?;
            }
            Self::Overlap { query_shape, pose } => {
                query_shape.validate()?;
                pose.validate()?;
                validate_position(pose.translation_micrometres)?;
            }
            Self::ClosestPoint {
                point_micrometres,
                maximum_distance_micrometres,
            } => {
                validate_position(*point_micrometres)?;
                validate_query_distance(*maximum_distance_micrometres)?;
            }
        }
        Ok(())
    }

    fn canonical_tagged_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, record) = match self {
            Self::RayCast {
                origin_micrometres,
                unit_direction_q1_30,
                maximum_distance_micrometres,
            } => (
                1,
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_BYTES,
                        encode_i64_vec3(*origin_micrometres),
                    ),
                    CanonicalField::new(
                        2,
                        CANONICAL_TYPE_BYTES,
                        encode_i32_vec3(*unit_direction_q1_30),
                    ),
                    field_i64(3, *maximum_distance_micrometres),
                ])?,
            ),
            Self::ShapeCast {
                query_shape,
                origin_pose,
                unit_direction_q1_30,
                maximum_distance_micrometres,
            } => (
                2,
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_TAGGED_UNION,
                        query_shape.canonical_tagged_bytes()?,
                    ),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, origin_pose.canonical_record()?),
                    CanonicalField::new(
                        3,
                        CANONICAL_TYPE_BYTES,
                        encode_i32_vec3(*unit_direction_q1_30),
                    ),
                    field_i64(4, *maximum_distance_micrometres),
                ])?,
            ),
            Self::Overlap { query_shape, pose } => (
                3,
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_TAGGED_UNION,
                        query_shape.canonical_tagged_bytes()?,
                    ),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, pose.canonical_record()?),
                ])?,
            ),
            Self::ClosestPoint {
                point_micrometres,
                maximum_distance_micrometres,
            } => (
                4,
                encode_struct([
                    CanonicalField::new(
                        1,
                        CANONICAL_TYPE_BYTES,
                        encode_i64_vec3(*point_micrometres),
                    ),
                    field_i64(2, *maximum_distance_micrometres),
                ])?,
            ),
        };
        let mut bytes = vec![tag];
        bytes.extend_from_slice(&nested(CANONICAL_TYPE_STRUCT, &record)?);
        Ok(bytes)
    }

    fn from_tagged_bytes(
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
        let value = match PhysicsQueryKindV1::from_tag(tag)? {
            PhysicsQueryKindV1::RayCast => {
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_BYTES),
                        (2, CANONICAL_TYPE_BYTES),
                        (3, crate::canonical::CANONICAL_TYPE_I64),
                    ],
                )?;
                Self::RayCast {
                    origin_micrometres: decode_i64_vec3(&field_from(&fields, 1)?.payload)?,
                    unit_direction_q1_30: decode_i32_vec3(&field_from(&fields, 2)?.payload)?,
                    maximum_distance_micrometres: read_i64_fields(&fields, 3)?,
                }
            }
            PhysicsQueryKindV1::ShapeCast => {
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_TAGGED_UNION),
                        (2, CANONICAL_TYPE_STRUCT),
                        (3, CANONICAL_TYPE_BYTES),
                        (4, crate::canonical::CANONICAL_TYPE_I64),
                    ],
                )?;
                Self::ShapeCast {
                    query_shape: PhysicsQueryShapeV1::from_tagged_bytes(
                        &field_from(&fields, 1)?.payload,
                        limits,
                    )?,
                    origin_pose: PhysicsPoseV1::from_record(
                        &field_from(&fields, 2)?.payload,
                        limits,
                    )?,
                    unit_direction_q1_30: decode_i32_vec3(&field_from(&fields, 3)?.payload)?,
                    maximum_distance_micrometres: read_i64_fields(&fields, 4)?,
                }
            }
            PhysicsQueryKindV1::Overlap => {
                require_fields(
                    &fields,
                    &[(1, CANONICAL_TYPE_TAGGED_UNION), (2, CANONICAL_TYPE_STRUCT)],
                )?;
                Self::Overlap {
                    query_shape: PhysicsQueryShapeV1::from_tagged_bytes(
                        &field_from(&fields, 1)?.payload,
                        limits,
                    )?,
                    pose: PhysicsPoseV1::from_record(&field_from(&fields, 2)?.payload, limits)?,
                }
            }
            PhysicsQueryKindV1::ClosestPoint => {
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_BYTES),
                        (2, crate::canonical::CANONICAL_TYPE_I64),
                    ],
                )?;
                Self::ClosestPoint {
                    point_micrometres: decode_i64_vec3(&field_from(&fields, 1)?.payload)?,
                    maximum_distance_micrometres: read_i64_fields(&fields, 2)?,
                }
            }
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsQueryCardinalityV1 {
    Any = 1,
    Closest = 2,
    All = 3,
}

impl PhysicsQueryCardinalityV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Any),
            2 => Ok(Self::Closest),
            3 => Ok(Self::All),
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsQueryFilterV1 {
    pub query_collision_layer: u8,
    pub query_collision_mask: u64,
    pub include_solid: bool,
    pub include_sensor: bool,
    pub include_query_only: bool,
    pub excluded_bodies: Vec<PhysicsBodyIdV1>,
    pub excluded_shapes: Vec<PhysicsShapeIdV1>,
}

impl PhysicsQueryFilterV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        let exclusion_count = self
            .excluded_bodies
            .len()
            .checked_add(self.excluded_shapes.len())
            .ok_or(PhysicsContractError::QueryCapacityExceeded)?;
        if self.query_collision_layer > 63 || exclusion_count > MAX_PHYSICS_QUERY_EXCLUSIONS {
            return Err(PhysicsContractError::InvalidQuery);
        }
        if self
            .excluded_bodies
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            || self
                .excluded_shapes
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    #[must_use]
    pub fn includes_participation(&self, participation: PhysicsParticipationV1) -> bool {
        match participation {
            PhysicsParticipationV1::Solid => self.include_solid,
            PhysicsParticipationV1::Sensor => self.include_sensor,
            PhysicsParticipationV1::QueryOnly => self.include_query_only,
        }
    }

    #[must_use]
    pub fn excludes(&self, shape_id: PhysicsShapeIdV1) -> bool {
        self.excluded_bodies
            .binary_search(&shape_id.body_id)
            .is_ok()
            || self.excluded_shapes.binary_search(&shape_id).is_ok()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            QUERY_FILTER_SCHEMA_ID,
            SEGMENT_V1,
            [CanonicalField::new(
                1,
                CANONICAL_TYPE_STRUCT,
                self.canonical_record()?,
            )],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            QUERY_FILTER_SCHEMA_ID,
            SEGMENT_V1,
            &[(1, CANONICAL_TYPE_STRUCT)],
        )?;
        let value = Self::from_record(field(&segment, 1)?, limits)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_u8(1, self.query_collision_layer),
            field_u64(2, self.query_collision_mask),
            field_bool(3, self.include_solid),
            field_bool(4, self.include_sensor),
            field_bool(5, self.include_query_only),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.excluded_bodies
                        .iter()
                        .copied()
                        .map(PhysicsBodyIdV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
            ),
            CanonicalField::new(
                7,
                CANONICAL_TYPE_SEQUENCE,
                encode_sequence(
                    self.excluded_shapes
                        .iter()
                        .copied()
                        .map(PhysicsShapeIdV1::canonical_record)
                        .collect::<Result<Vec<_>, _>>()?,
                )?,
            ),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_U8),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_BOOL),
                (4, CANONICAL_TYPE_BOOL),
                (5, CANONICAL_TYPE_BOOL),
                (6, CANONICAL_TYPE_SEQUENCE),
                (7, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            query_collision_layer: read_u8_fields(&fields, 1)?,
            query_collision_mask: read_u64_fields(&fields, 2)?,
            include_solid: read_bool_fields(&fields, 3)?,
            include_sensor: read_bool_fields(&fields, 4)?,
            include_query_only: read_bool_fields(&fields, 5)?,
            excluded_bodies: decode_sequence(&field_from(&fields, 6)?.payload, limits)?
                .into_iter()
                .map(|record| PhysicsBodyIdV1::from_record(&record, limits))
                .collect::<Result<_, _>>()?,
            excluded_shapes: decode_sequence(&field_from(&fields, 7)?.payload, limits)?
                .into_iter()
                .map(|record| PhysicsShapeIdV1::from_record(&record, limits))
                .collect::<Result<_, _>>()?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsQueryRequestV1 {
    pub schema_version: u16,
    pub query_id: PhysicsQueryIdV1,
    pub world_id: PhysicsWorldId,
    pub snapshot_selector: PhysicsSnapshotSelectorV1,
    pub geometry: PhysicsQueryGeometryV1,
    pub filter: PhysicsQueryFilterV1,
    pub cardinality: PhysicsQueryCardinalityV1,
    pub maximum_published_hits: u32,
}

impl PhysicsQueryRequestV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_QUERY_SCHEMA_VERSION
            || self.query_id.physics_tick != self.snapshot_selector.physics_tick
        {
            return Err(PhysicsContractError::InvalidQuery);
        }
        self.geometry.validate()?;
        self.filter.validate()?;
        match self.cardinality {
            PhysicsQueryCardinalityV1::Any if self.maximum_published_hits != 0 => {
                return Err(PhysicsContractError::InvalidQuery);
            }
            PhysicsQueryCardinalityV1::Closest if self.maximum_published_hits != 1 => {
                return Err(PhysicsContractError::InvalidQuery);
            }
            PhysicsQueryCardinalityV1::All
                if !(1..=MAX_PHYSICS_QUERY_PUBLISHED_HITS)
                    .contains(&self.maximum_published_hits) =>
            {
                return Err(PhysicsContractError::InvalidQuery);
            }
            _ => {}
        }
        if self.geometry.kind() == PhysicsQueryKindV1::Overlap
            && self.cardinality == PhysicsQueryCardinalityV1::Closest
        {
            return Err(PhysicsContractError::InvalidQuery);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            QUERY_REQUEST_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.query_id.canonical_record()?),
                field_id(3, self.world_id.as_bytes()),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_STRUCT,
                    self.snapshot_selector.canonical_record()?,
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_TAGGED_UNION,
                    self.geometry.canonical_tagged_bytes()?,
                ),
                CanonicalField::new(6, CANONICAL_TYPE_STRUCT, self.filter.canonical_record()?),
                field_u8(7, self.cardinality as u8),
                field_u32(8, self.maximum_published_hits),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            QUERY_REQUEST_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_STRUCT),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_STRUCT),
                (5, CANONICAL_TYPE_TAGGED_UNION),
                (6, CANONICAL_TYPE_STRUCT),
                (7, CANONICAL_TYPE_U8),
                (8, CANONICAL_TYPE_U32),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            query_id: PhysicsQueryIdV1::from_record(field(&segment, 2)?, limits)?,
            world_id: PhysicsWorldId::from_bytes(exact(field(&segment, 3)?)?),
            snapshot_selector: PhysicsSnapshotSelectorV1::from_record(field(&segment, 4)?, limits)?,
            geometry: PhysicsQueryGeometryV1::from_tagged_bytes(field(&segment, 5)?, limits)?,
            filter: PhysicsQueryFilterV1::from_record(field(&segment, 6)?, limits)?,
            cardinality: PhysicsQueryCardinalityV1::from_tag(read_u8(&segment, 7)?)?,
            maximum_published_hits: u32::from_le_bytes(exact(field(&segment, 8)?)?),
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn request_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-query-request.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsQueryBatchV1 {
    pub schema_version: u16,
    pub snapshot_selector: PhysicsSnapshotSelectorV1,
    pub requests: Vec<PhysicsQueryRequestV1>,
}

impl PhysicsQueryBatchV1 {
    pub fn new(
        snapshot_selector: PhysicsSnapshotSelectorV1,
        requests: Vec<PhysicsQueryRequestV1>,
    ) -> Result<Self, PhysicsContractError> {
        let mut keyed_requests = requests
            .into_iter()
            .map(|request| Ok((request.query_id, request.request_hash()?, request)))
            .collect::<Result<Vec<_>, PhysicsContractError>>()?;
        keyed_requests.sort_by_key(|(query_id, request_hash, _)| (*query_id, *request_hash));
        let requests = keyed_requests
            .into_iter()
            .map(|(_, _, request)| request)
            .collect();
        let value = Self {
            schema_version: PHYSICS_QUERY_BATCH_SCHEMA_VERSION,
            snapshot_selector,
            requests,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_QUERY_BATCH_SCHEMA_VERSION
            || self.requests.len() > MAX_PHYSICS_QUERY_REQUESTS_PER_BATCH
        {
            return Err(PhysicsContractError::QueryCapacityExceeded);
        }
        let mut previous: Option<(PhysicsQueryIdV1, ContentHash)> = None;
        let mut previous_id = None;
        for request in &self.requests {
            request.validate()?;
            if request.snapshot_selector != self.snapshot_selector {
                return Err(PhysicsContractError::SnapshotSelectorMismatch);
            }
            if previous_id.is_some_and(|id| id == request.query_id) {
                return Err(PhysicsContractError::DuplicateKey);
            }
            let key = (request.query_id, request.request_hash()?);
            if previous.is_some_and(|previous| previous >= key) {
                return Err(PhysicsContractError::NonCanonicalOrder);
            }
            previous = Some(key);
            previous_id = Some(request.query_id);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            QUERY_BATCH_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_STRUCT,
                    self.snapshot_selector.canonical_record()?,
                ),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.requests
                            .iter()
                            .map(PhysicsQueryRequestV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            QUERY_BATCH_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_STRUCT),
                (3, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            snapshot_selector: PhysicsSnapshotSelectorV1::from_record(field(&segment, 2)?, limits)?,
            requests: decode_sequence(field(&segment, 3)?, limits)?
                .into_iter()
                .map(|record| PhysicsQueryRequestV1::from_canonical_bytes(&record, limits))
                .collect::<Result<_, _>>()?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

mod result;
pub use result::{PhysicsQueryHitV1, PhysicsQueryResultPayloadV1, PhysicsQueryResultV1};

fn validate_position(position: [i64; 3]) -> Result<(), PhysicsContractError> {
    if position.into_iter().any(|component| {
        !(-MAX_POSITION_MICROMETRES..=MAX_POSITION_MICROMETRES).contains(&component)
    }) {
        return Err(PhysicsContractError::InvalidQuery);
    }
    Ok(())
}

fn validate_query_distance(distance: i64) -> Result<(), PhysicsContractError> {
    if !(0..=MAX_PHYSICS_QUERY_DISTANCE_MICROMETRES).contains(&distance) {
        return Err(PhysicsContractError::InvalidQuery);
    }
    Ok(())
}

fn validate_unit_direction(direction: [i32; 3]) -> Result<(), PhysicsContractError> {
    let squared_norm = direction.into_iter().try_fold(0_i128, |sum, component| {
        let component = i128::from(component);
        sum.checked_add(component * component)
    });
    if squared_norm != Some(1_i128 << 60) {
        return Err(PhysicsContractError::InvalidQuery);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
