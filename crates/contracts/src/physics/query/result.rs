use super::*;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsQueryHitV1 {
    pub distance_micrometres: i64,
    pub shape_id: PhysicsShapeIdV1,
    pub feature_id: u32,
    pub point_micrometres: [i64; 3],
    pub outward_normal_q1_30: [i32; 3],
    pub fraction_q0_32: Option<u32>,
}

impl PhysicsQueryHitV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.distance_micrometres < 0
            || self.distance_micrometres > MAX_PHYSICS_QUERY_DISTANCE_MICROMETRES
            || self.feature_id == 0
            || self
                .outward_normal_q1_30
                .iter()
                .all(|component| *component == 0)
            || self
                .outward_normal_q1_30
                .iter()
                .any(|component| !(-Q1_30_ONE..=Q1_30_ONE).contains(component))
        {
            return Err(PhysicsContractError::InvalidQuery);
        }
        validate_position(self.point_micrometres)
    }

    /// Compares two normalized hits using the complete canonical key owned by
    /// the query kind. Callers must validate both hits against the request
    /// before relying on this order.
    #[must_use]
    pub fn cmp_canonical_for_kind(&self, other: &Self, kind: PhysicsQueryKindV1) -> Ordering {
        match kind {
            PhysicsQueryKindV1::RayCast | PhysicsQueryKindV1::ShapeCast => self
                .fraction_q0_32
                .cmp(&other.fraction_q0_32)
                .then_with(|| self.distance_micrometres.cmp(&other.distance_micrometres))
                .then_with(|| self.shape_id.cmp(&other.shape_id))
                .then_with(|| self.feature_id.cmp(&other.feature_id))
                .then_with(|| self.point_micrometres.cmp(&other.point_micrometres))
                .then_with(|| self.outward_normal_q1_30.cmp(&other.outward_normal_q1_30)),
            PhysicsQueryKindV1::ClosestPoint => self
                .distance_micrometres
                .cmp(&other.distance_micrometres)
                .then_with(|| self.shape_id.cmp(&other.shape_id))
                .then_with(|| self.feature_id.cmp(&other.feature_id))
                .then_with(|| self.point_micrometres.cmp(&other.point_micrometres))
                .then_with(|| self.outward_normal_q1_30.cmp(&other.outward_normal_q1_30)),
            PhysicsQueryKindV1::Overlap => self
                .shape_id
                .cmp(&other.shape_id)
                .then_with(|| self.feature_id.cmp(&other.feature_id)),
        }
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            field_i64(1, self.distance_micrometres),
            CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.shape_id.canonical_record()?),
            field_u32(3, self.feature_id),
            CanonicalField::new(
                4,
                CANONICAL_TYPE_BYTES,
                encode_i64_vec3(self.point_micrometres),
            ),
            CanonicalField::new(
                5,
                CANONICAL_TYPE_BYTES,
                encode_i32_vec3(self.outward_normal_q1_30),
            ),
            CanonicalField::new(
                6,
                CANONICAL_TYPE_OPTIONAL,
                encode_optional_u32(self.fraction_q0_32),
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
                (1, crate::canonical::CANONICAL_TYPE_I64),
                (2, CANONICAL_TYPE_STRUCT),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_BYTES),
                (5, CANONICAL_TYPE_BYTES),
                (6, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        let value = Self {
            distance_micrometres: read_i64_fields(&fields, 1)?,
            shape_id: PhysicsShapeIdV1::from_record(&field_from(&fields, 2)?.payload, limits)?,
            feature_id: read_u32_fields(&fields, 3)?,
            point_micrometres: decode_i64_vec3(&field_from(&fields, 4)?.payload)?,
            outward_normal_q1_30: decode_i32_vec3(&field_from(&fields, 5)?.payload)?,
            fraction_q0_32: decode_optional_u32(&field_from(&fields, 6)?.payload)?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicsQueryResultPayloadV1 {
    Any {
        eligible_hit: bool,
    },
    Closest {
        hit: Option<PhysicsQueryHitV1>,
    },
    All {
        eligible_hit_count: u32,
        truncated: bool,
        hits: Vec<PhysicsQueryHitV1>,
    },
}

impl PhysicsQueryResultPayloadV1 {
    fn canonical_tagged_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, record) = match self {
            Self::Any { eligible_hit } => (1, encode_struct([field_bool(1, *eligible_hit)])?),
            Self::Closest { hit } => (
                2,
                encode_struct([CanonicalField::new(
                    1,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_optional_hit(hit.as_ref())?,
                )])?,
            ),
            Self::All {
                eligible_hit_count,
                truncated,
                hits,
            } => (
                3,
                encode_struct([
                    field_u32(1, *eligible_hit_count),
                    field_bool(2, *truncated),
                    CanonicalField::new(
                        3,
                        CANONICAL_TYPE_SEQUENCE,
                        encode_sequence(
                            hits.iter()
                                .map(PhysicsQueryHitV1::canonical_record)
                                .collect::<Result<Vec<_>, _>>()?,
                        )?,
                    ),
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
        match tag {
            1 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_BOOL)])?;
                Ok(Self::Any {
                    eligible_hit: read_bool_fields(&fields, 1)?,
                })
            }
            2 => {
                require_fields(&fields, &[(1, CANONICAL_TYPE_OPTIONAL)])?;
                Ok(Self::Closest {
                    hit: decode_optional_hit(&field_from(&fields, 1)?.payload, limits)?,
                })
            }
            3 => {
                require_fields(
                    &fields,
                    &[
                        (1, CANONICAL_TYPE_U32),
                        (2, CANONICAL_TYPE_BOOL),
                        (3, CANONICAL_TYPE_SEQUENCE),
                    ],
                )?;
                Ok(Self::All {
                    eligible_hit_count: read_u32_fields(&fields, 1)?,
                    truncated: read_bool_fields(&fields, 2)?,
                    hits: decode_sequence(&field_from(&fields, 3)?.payload, limits)?
                        .into_iter()
                        .map(|record| PhysicsQueryHitV1::from_record(&record, limits))
                        .collect::<Result<_, _>>()?,
                })
            }
            other => Err(PhysicsContractError::UnknownTag(other)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsQueryResultV1 {
    pub schema_version: u16,
    pub query_id: PhysicsQueryIdV1,
    pub snapshot_selector: PhysicsSnapshotSelectorV1,
    pub request_hash: ContentHash,
    pub payload: PhysicsQueryResultPayloadV1,
}

impl PhysicsQueryResultV1 {
    pub fn validate_against_request(
        &self,
        request: &PhysicsQueryRequestV1,
    ) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_QUERY_RESULT_SCHEMA_VERSION
            || self.query_id != request.query_id
            || self.snapshot_selector != request.snapshot_selector
            || self.request_hash != request.request_hash()?
        {
            return Err(PhysicsContractError::InvalidQuery);
        }
        match (&self.payload, request.cardinality) {
            (PhysicsQueryResultPayloadV1::Any { .. }, PhysicsQueryCardinalityV1::Any) => {}
            (PhysicsQueryResultPayloadV1::Closest { hit }, PhysicsQueryCardinalityV1::Closest) => {
                if let Some(hit) = hit {
                    hit.validate()?;
                    validate_hit_kind(hit, request.geometry.kind())?;
                }
            }
            (
                PhysicsQueryResultPayloadV1::All {
                    eligible_hit_count,
                    truncated,
                    hits,
                },
                PhysicsQueryCardinalityV1::All,
            ) => {
                let published_limit = usize::try_from(request.maximum_published_hits)
                    .map_err(|_| PhysicsContractError::QueryCapacityExceeded)?;
                let eligible_count = usize::try_from(*eligible_hit_count)
                    .map_err(|_| PhysicsContractError::QueryCapacityExceeded)?;
                let expected_published = eligible_count.min(published_limit);
                if hits.len() != expected_published
                    || *truncated != (eligible_count > published_limit)
                    || hits.windows(2).any(|pair| {
                        pair[0].cmp_canonical_for_kind(&pair[1], request.geometry.kind())
                            != Ordering::Less
                    })
                {
                    return Err(PhysicsContractError::InvalidQuery);
                }
                for hit in hits {
                    hit.validate()?;
                    validate_hit_kind(hit, request.geometry.kind())?;
                }
            }
            _ => return Err(PhysicsContractError::InvalidQuery),
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            QUERY_RESULT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.query_id.canonical_record()?),
                CanonicalField::new(
                    3,
                    CANONICAL_TYPE_STRUCT,
                    self.snapshot_selector.canonical_record()?,
                ),
                field_hash(4, self.request_hash),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_TAGGED_UNION,
                    self.payload.canonical_tagged_bytes()?,
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
            QUERY_RESULT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_STRUCT),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_TAGGED_UNION),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            query_id: PhysicsQueryIdV1::from_record(field(&segment, 2)?, limits)?,
            snapshot_selector: PhysicsSnapshotSelectorV1::from_record(field(&segment, 3)?, limits)?,
            request_hash: read_hash(&segment, 4)?,
            payload: PhysicsQueryResultPayloadV1::from_tagged_bytes(field(&segment, 5)?, limits)?,
        };
        if value.schema_version != PHYSICS_QUERY_RESULT_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                value.schema_version,
            )));
        }
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn result_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-query-result.v1\0",
            &self.canonical_bytes()?,
        )
    }
}

fn validate_hit_kind(
    hit: &PhysicsQueryHitV1,
    kind: PhysicsQueryKindV1,
) -> Result<(), PhysicsContractError> {
    match kind {
        PhysicsQueryKindV1::RayCast | PhysicsQueryKindV1::ShapeCast
            if hit.fraction_q0_32.is_none() =>
        {
            Err(PhysicsContractError::InvalidQuery)
        }
        PhysicsQueryKindV1::Overlap | PhysicsQueryKindV1::ClosestPoint
            if hit.fraction_q0_32.is_some() =>
        {
            Err(PhysicsContractError::InvalidQuery)
        }
        _ => Ok(()),
    }
}

fn encode_optional_u32(value: Option<u32>) -> Vec<u8> {
    match value {
        None => vec![0],
        Some(value) => {
            let mut bytes = Vec::with_capacity(5);
            bytes.push(1);
            bytes.extend_from_slice(&value.to_le_bytes());
            bytes
        }
    }
}

fn decode_optional_u32(bytes: &[u8]) -> Result<Option<u32>, PhysicsContractError> {
    match bytes {
        [0] => Ok(None),
        [1, rest @ ..] if rest.len() == 4 => Ok(Some(u32::from_le_bytes(exact(rest)?))),
        _ => Err(PhysicsContractError::FieldLength),
    }
}

fn encode_optional_hit(hit: Option<&PhysicsQueryHitV1>) -> Result<Vec<u8>, CanonicalError> {
    match hit {
        None => Ok(vec![0]),
        Some(hit) => {
            let record = hit.canonical_record()?;
            let mut bytes = Vec::with_capacity(record.len() + 9);
            bytes.push(1);
            bytes.extend_from_slice(
                &u64::try_from(record.len())
                    .map_err(|_| CanonicalError::LengthOverflow)?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&record);
            Ok(bytes)
        }
    }
}

fn decode_optional_hit(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<PhysicsQueryHitV1>, PhysicsContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let present = cursor.read_u8()?;
    let value = match present {
        0 => None,
        1 => {
            let length = usize::try_from(cursor.read_u64()?)
                .map_err(|_| PhysicsContractError::FieldLength)?;
            if length > limits.max_field_payload_bytes {
                return Err(PhysicsContractError::FieldLength);
            }
            Some(PhysicsQueryHitV1::from_record(
                cursor.read_exact(length)?,
                limits,
            )?)
        }
        _ => return Err(PhysicsContractError::FieldType),
    };
    cursor.finish()?;
    Ok(value)
}
