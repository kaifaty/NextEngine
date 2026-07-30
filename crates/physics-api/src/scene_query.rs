use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::physics::{
    MAX_PHYSICS_QUERY_CANDIDATE_HITS, PHYSICS_QUERY_RESULT_SCHEMA_VERSION, PhysicsContractError,
    PhysicsGeometryV1, PhysicsPoseV1, PhysicsQueryCardinalityV1, PhysicsQueryGeometryV1,
    PhysicsQueryHitV1, PhysicsQueryKindV1, PhysicsQueryRequestV1, PhysicsQueryResultPayloadV1,
    PhysicsQueryResultV1, PhysicsShapeDescriptorV1, PhysicsWorldCheckpointV1,
};

const Q1_30_ONE: i32 = 1 << 30;

pub fn execute_scene_query(
    checkpoint: &PhysicsWorldCheckpointV1,
    request: &PhysicsQueryRequestV1,
) -> Result<PhysicsQueryResultV1, PhysicsSceneQueryError> {
    checkpoint.validate()?;
    request.validate()?;
    validate_snapshot_selector(checkpoint, request)?;

    let PhysicsQueryGeometryV1::ClosestPoint {
        point_micrometres,
        maximum_distance_micrometres,
    } = request.geometry
    else {
        return Err(PhysicsSceneQueryError::UnsupportedQueryKind(
            request.geometry.kind(),
        ));
    };

    let maximum_distance_squared = square(maximum_distance_micrometres)?;
    let mut candidate_count = 0_usize;
    let mut hits = Vec::new();
    for (body_id, body_descriptor) in &checkpoint.catalog.bodies {
        let state = checkpoint
            .snapshot
            .sorted_body_states
            .get(body_id)
            .ok_or(PhysicsSceneQueryError::SnapshotMismatch)?;
        if !state.active {
            continue;
        }
        for shape in body_descriptor.shapes.values() {
            if !filter_accepts_shape(request, shape) {
                continue;
            }
            candidate_count = candidate_count
                .checked_add(1)
                .ok_or(PhysicsSceneQueryError::CapacityExceeded)?;
            if candidate_count > MAX_PHYSICS_QUERY_CANDIDATE_HITS {
                return Err(PhysicsSceneQueryError::CapacityExceeded);
            }
            let candidate = closest_point_on_shape(
                point_micrometres,
                state.pose,
                shape,
                maximum_distance_squared,
            )?;
            if let Some(hit) = candidate {
                hits.push(hit);
            }
        }
    }
    hits.sort_by(|left, right| left.cmp_canonical_for_kind(right, request.geometry.kind()));
    hits.dedup();
    let eligible_hit_count =
        u32::try_from(hits.len()).map_err(|_| PhysicsSceneQueryError::CapacityExceeded)?;

    let payload = match request.cardinality {
        PhysicsQueryCardinalityV1::Any => PhysicsQueryResultPayloadV1::Any {
            eligible_hit: eligible_hit_count != 0,
        },
        PhysicsQueryCardinalityV1::Closest => PhysicsQueryResultPayloadV1::Closest {
            hit: hits.first().cloned(),
        },
        PhysicsQueryCardinalityV1::All => {
            let publish_count = usize::try_from(request.maximum_published_hits)
                .map_err(|_| PhysicsSceneQueryError::CapacityExceeded)?
                .min(hits.len());
            let published = hits.into_iter().take(publish_count).collect();
            PhysicsQueryResultPayloadV1::All {
                eligible_hit_count,
                truncated: eligible_hit_count
                    > u32::try_from(publish_count)
                        .map_err(|_| PhysicsSceneQueryError::CapacityExceeded)?,
                hits: published,
            }
        }
    };
    let result = PhysicsQueryResultV1 {
        schema_version: PHYSICS_QUERY_RESULT_SCHEMA_VERSION,
        query_id: request.query_id,
        snapshot_selector: request.snapshot_selector,
        request_hash: request.request_hash()?,
        payload,
    };
    result.validate_against_request(request)?;
    Ok(result)
}

fn validate_snapshot_selector(
    checkpoint: &PhysicsWorldCheckpointV1,
    request: &PhysicsQueryRequestV1,
) -> Result<(), PhysicsSceneQueryError> {
    let snapshot = &checkpoint.snapshot;
    // PhysicsCanonicalSnapshotV2 increments `physics_tick` for every completed
    // substep and therefore has one closed boundary per tick. Until a later
    // snapshot schema carries a separate ordinal, its only representable
    // completed-substep slot is zero.
    if request.world_id != snapshot.world_id
        || request.snapshot_selector.physics_tick != snapshot.physics_tick
        || request.snapshot_selector.completed_substep != 0
        || request.snapshot_selector.physics_snapshot_hash != snapshot.snapshot_hash()?
    {
        return Err(PhysicsSceneQueryError::SnapshotMismatch);
    }
    Ok(())
}

fn filter_accepts_shape(request: &PhysicsQueryRequestV1, shape: &PhysicsShapeDescriptorV1) -> bool {
    if request.filter.excludes(shape.shape_id)
        || !request.filter.includes_participation(shape.participation)
    {
        return false;
    }
    let request_accepts_shape =
        (request.filter.query_collision_mask >> shape.collision_layer) & 1 == 1;
    let shape_accepts_request =
        (shape.collision_mask >> request.filter.query_collision_layer) & 1 == 1;
    request_accepts_shape && shape_accepts_request
}

fn closest_point_on_shape(
    query_point: [i64; 3],
    body_pose: PhysicsPoseV1,
    shape: &PhysicsShapeDescriptorV1,
    maximum_distance_squared: i128,
) -> Result<Option<PhysicsQueryHitV1>, PhysicsSceneQueryError> {
    if body_pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
        || shape.local_pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
    {
        return Err(PhysicsSceneQueryError::UnsupportedShapePose);
    }
    let centre = checked_add_vec3(
        body_pose.translation_micrometres,
        shape.local_pose.translation_micrometres,
    )?;
    match shape.geometry {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => closest_point_on_box(
            query_point,
            centre,
            half_extents_micrometres,
            shape,
            maximum_distance_squared,
        ),
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => closest_point_on_capsule(
            query_point,
            centre,
            radius_micrometres,
            half_segment_micrometres,
            shape,
            maximum_distance_squared,
        ),
        PhysicsGeometryV1::Sphere { .. }
        | PhysicsGeometryV1::ConvexHull { .. }
        | PhysicsGeometryV1::TriangleMesh { .. }
        | PhysicsGeometryV1::HeightField { .. } => {
            Err(PhysicsSceneQueryError::UnsupportedShapeGeometry)
        }
    }
}

fn closest_point_on_box(
    query_point: [i64; 3],
    centre: [i64; 3],
    half_extents: [i64; 3],
    shape: &PhysicsShapeDescriptorV1,
    maximum_distance_squared: i128,
) -> Result<Option<PhysicsQueryHitV1>, PhysicsSceneQueryError> {
    let minimum = checked_sub_vec3(centre, half_extents)?;
    let maximum = checked_add_vec3(centre, half_extents)?;
    let closest = [
        query_point[0].clamp(minimum[0], maximum[0]),
        query_point[1].clamp(minimum[1], maximum[1]),
        query_point[2].clamp(minimum[2], maximum[2]),
    ];
    let delta = checked_sub_vec3(query_point, closest)?;
    let squared_distance = squared_length(delta)?;
    if squared_distance > maximum_distance_squared {
        return Ok(None);
    }

    let (feature_id, outward_normal) = if squared_distance == 0 {
        inside_box_feature(query_point, minimum, maximum)?
    } else {
        let outside = box_outside_components(query_point, minimum, maximum)?;
        (
            u32::from(primitive_box_feature(&outside)?),
            normalized_outward(delta, squared_distance)?,
        )
    };
    Ok(Some(PhysicsQueryHitV1 {
        distance_micrometres: ceil_sqrt(squared_distance)?,
        shape_id: shape.shape_id,
        feature_id,
        point_micrometres: closest,
        outward_normal_q1_30: outward_normal,
        fraction_q0_32: None,
    }))
}

fn closest_point_on_capsule(
    query_point: [i64; 3],
    centre: [i64; 3],
    radius: i64,
    half_segment: i64,
    shape: &PhysicsShapeDescriptorV1,
    maximum_distance_squared: i128,
) -> Result<Option<PhysicsQueryHitV1>, PhysicsSceneQueryError> {
    let segment_minimum = centre[1]
        .checked_sub(half_segment)
        .ok_or(PhysicsSceneQueryError::NumericOverflow)?;
    let segment_maximum = centre[1]
        .checked_add(half_segment)
        .ok_or(PhysicsSceneQueryError::NumericOverflow)?;
    let axis_point = [
        centre[0],
        query_point[1].clamp(segment_minimum, segment_maximum),
        centre[2],
    ];
    let radial = checked_sub_vec3(query_point, axis_point)?;
    let radial_squared = squared_length(radial)?;
    let maximum_radial_distance = radius
        .checked_add(ceil_sqrt(maximum_distance_squared)?)
        .ok_or(PhysicsSceneQueryError::NumericOverflow)?;
    if radial_squared > square(maximum_radial_distance)? {
        return Ok(None);
    }

    let radial_length = ceil_sqrt(radial_squared)?;
    let (distance, closest, normal) = if radial_length <= radius {
        let normal = if radial_squared == 0 {
            [Q1_30_ONE, 0, 0]
        } else {
            normalized_outward(radial, radial_squared)?
        };
        (0, query_point, normal)
    } else {
        let distance = radial_length
            .checked_sub(radius)
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?;
        let normal = normalized_outward(radial, radial_squared)?;
        let closest = [
            checked_scale_add(axis_point[0], radial[0], radius, radial_length)?,
            checked_scale_add(axis_point[1], radial[1], radius, radial_length)?,
            checked_scale_add(axis_point[2], radial[2], radius, radial_length)?,
        ];
        (distance, closest, normal)
    };
    Ok(Some(PhysicsQueryHitV1 {
        distance_micrometres: distance,
        shape_id: shape.shape_id,
        feature_id: 1,
        point_micrometres: closest,
        outward_normal_q1_30: normal,
        fraction_q0_32: None,
    }))
}

fn inside_box_feature(
    point: [i64; 3],
    minimum: [i64; 3],
    maximum: [i64; 3],
) -> Result<(u32, [i32; 3]), PhysicsSceneQueryError> {
    let candidates = [
        (point[0].abs_diff(minimum[0]), 1_u32, [-Q1_30_ONE, 0, 0]),
        (point[0].abs_diff(maximum[0]), 2_u32, [Q1_30_ONE, 0, 0]),
        (point[1].abs_diff(minimum[1]), 3_u32, [0, -Q1_30_ONE, 0]),
        (point[1].abs_diff(maximum[1]), 4_u32, [0, Q1_30_ONE, 0]),
        (point[2].abs_diff(minimum[2]), 5_u32, [0, 0, -Q1_30_ONE]),
        (point[2].abs_diff(maximum[2]), 6_u32, [0, 0, Q1_30_ONE]),
    ];
    let (_, feature, normal) = candidates
        .into_iter()
        .min_by_key(|(distance, feature, _)| (*distance, *feature))
        .ok_or(PhysicsSceneQueryError::NumericOverflow)?;
    Ok((feature, normal))
}

fn box_outside_components(
    point: [i64; 3],
    minimum: [i64; 3],
    maximum: [i64; 3],
) -> Result<Vec<(usize, bool, i64)>, PhysicsSceneQueryError> {
    let mut outside = Vec::with_capacity(3);
    for axis in 0..3 {
        if point[axis] < minimum[axis] {
            outside.push((
                axis,
                false,
                i64::try_from(point[axis].abs_diff(minimum[axis]))
                    .map_err(|_| PhysicsSceneQueryError::NumericOverflow)?,
            ));
        } else if point[axis] > maximum[axis] {
            outside.push((
                axis,
                true,
                i64::try_from(point[axis].abs_diff(maximum[axis]))
                    .map_err(|_| PhysicsSceneQueryError::NumericOverflow)?,
            ));
        }
    }
    if outside.is_empty() {
        return Err(PhysicsSceneQueryError::NumericOverflow);
    }
    Ok(outside)
}

fn primitive_box_feature(outside: &[(usize, bool, i64)]) -> Result<u8, PhysicsSceneQueryError> {
    match outside {
        [(axis, positive, _)] => Ok(if *positive {
            positive_face(*axis)
        } else {
            negative_face(*axis)
        }),
        [
            (first_axis, first_positive, _),
            (second_axis, second_positive, _),
        ] => {
            let pair_slot = match (*first_axis, *second_axis) {
                (0, 1) => 0,
                (0, 2) => 1,
                (1, 2) => 2,
                _ => return Err(PhysicsSceneQueryError::NumericOverflow),
            };
            let sign_slot = u8::from(*first_positive) * 2 + u8::from(*second_positive);
            Ok(7 + pair_slot * 4 + sign_slot)
        }
        [(0, x_positive, _), (1, y_positive, _), (2, z_positive, _)] => {
            Ok(19 + u8::from(*x_positive) * 4 + u8::from(*y_positive) * 2 + u8::from(*z_positive))
        }
        _ => Err(PhysicsSceneQueryError::NumericOverflow),
    }
}

const fn negative_face(axis: usize) -> u8 {
    match axis {
        0 => 1,
        1 => 3,
        _ => 5,
    }
}

const fn positive_face(axis: usize) -> u8 {
    match axis {
        0 => 2,
        1 => 4,
        _ => 6,
    }
}

fn normalized_outward(
    delta: [i64; 3],
    squared_length: i128,
) -> Result<[i32; 3], PhysicsSceneQueryError> {
    let length = i128::from(ceil_sqrt(squared_length)?);
    if length == 0 {
        return Err(PhysicsSceneQueryError::NumericOverflow);
    }
    let mut normal = [0; 3];
    for (output, component) in normal.iter_mut().zip(delta) {
        let scaled = i128::from(component)
            .checked_mul(i128::from(Q1_30_ONE))
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?
            / length;
        *output = i32::try_from(scaled).map_err(|_| PhysicsSceneQueryError::NumericOverflow)?;
    }
    Ok(normal)
}

fn checked_scale_add(
    origin: i64,
    delta: i64,
    numerator: i64,
    denominator: i64,
) -> Result<i64, PhysicsSceneQueryError> {
    let offset = i128::from(delta)
        .checked_mul(i128::from(numerator))
        .ok_or(PhysicsSceneQueryError::NumericOverflow)?
        / i128::from(denominator);
    let offset = i64::try_from(offset).map_err(|_| PhysicsSceneQueryError::NumericOverflow)?;
    origin
        .checked_add(offset)
        .ok_or(PhysicsSceneQueryError::NumericOverflow)
}

fn squared_length(value: [i64; 3]) -> Result<i128, PhysicsSceneQueryError> {
    value.into_iter().try_fold(0_i128, |sum, component| {
        sum.checked_add(square(component)?)
            .ok_or(PhysicsSceneQueryError::NumericOverflow)
    })
}

fn square(value: i64) -> Result<i128, PhysicsSceneQueryError> {
    let value = i128::from(value);
    value
        .checked_mul(value)
        .ok_or(PhysicsSceneQueryError::NumericOverflow)
}

fn ceil_sqrt(value: i128) -> Result<i64, PhysicsSceneQueryError> {
    if value < 0 {
        return Err(PhysicsSceneQueryError::NumericOverflow);
    }
    let value = u128::try_from(value).map_err(|_| PhysicsSceneQueryError::NumericOverflow)?;
    let mut low = 0_u128;
    let mut high = value.min(u128::from(u64::MAX));
    while low < high {
        let middle = low + (high - low) / 2;
        let middle_squared = middle
            .checked_mul(middle)
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?;
        if middle_squared >= value {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    i64::try_from(low).map_err(|_| PhysicsSceneQueryError::NumericOverflow)
}

fn checked_add_vec3(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], PhysicsSceneQueryError> {
    Ok([
        left[0]
            .checked_add(right[0])
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?,
        left[1]
            .checked_add(right[1])
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?,
        left[2]
            .checked_add(right[2])
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?,
    ])
}

fn checked_sub_vec3(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], PhysicsSceneQueryError> {
    Ok([
        left[0]
            .checked_sub(right[0])
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?,
        left[1]
            .checked_sub(right[1])
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?,
        left[2]
            .checked_sub(right[2])
            .ok_or(PhysicsSceneQueryError::NumericOverflow)?,
    ])
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicsSceneQueryError {
    Contract(PhysicsContractError),
    SnapshotMismatch,
    UnsupportedQueryKind(PhysicsQueryKindV1),
    UnsupportedShapeGeometry,
    UnsupportedShapePose,
    CapacityExceeded,
    NumericOverflow,
}

impl PhysicsSceneQueryError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::SnapshotMismatch => "PHYS_QUERY_SNAPSHOT_MISMATCH",
            Self::UnsupportedQueryKind(_)
            | Self::UnsupportedShapeGeometry
            | Self::UnsupportedShapePose => "PHYS_QUERY_UNSUPPORTED",
            Self::CapacityExceeded => "PHYS_QUERY_CAPACITY_EXCEEDED",
            Self::NumericOverflow => "PHYS_QUERY_INVALID",
        }
    }
}

impl Display for PhysicsSceneQueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for PhysicsSceneQueryError {}

impl From<PhysicsContractError> for PhysicsSceneQueryError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<next_contracts::canonical::CanonicalError> for PhysicsSceneQueryError {
    fn from(error: next_contracts::canonical::CanonicalError) -> Self {
        Self::Contract(PhysicsContractError::Canonicalization(error))
    }
}
