use next_contracts::physics::{
    PhysicsContactReportingV1, PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeDescriptorV1,
    PhysicsShapeIdV1,
};

use super::error::ReferencePhysicsError;

const Q1_30_ONE: i32 = 1 << 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedCapsuleStaticBox {
    pub shape_id: PhysicsShapeIdV1,
    pub minimum: [i64; 3],
    pub maximum: [i64; 3],
    pub contact_reporting: PhysicsContactReportingV1,
    pub collision_layer: u8,
    pub collision_mask: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GroundedCapsuleSweepHit {
    pub shape_id: PhysicsShapeIdV1,
    pub box_feature: u8,
    pub normal_box_to_capsule: [i32; 3],
}

pub struct GroundedCapsuleSweepRequest<'a> {
    pub centre_micrometres: [i64; 3],
    pub axis: u8,
    pub delta_micrometres: i64,
    pub capsule_radius_micrometres: i64,
    pub capsule_half_segment_micrometres: i64,
    pub capsule_collision_layer: u8,
    pub capsule_collision_mask: u64,
    pub static_boxes: &'a [GroundedCapsuleStaticBox],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroundedCapsuleSweepResult {
    pub applied_delta_micrometres: i64,
    pub hit: Option<GroundedCapsuleSweepHit>,
}

pub trait GroundedCapsuleQuery: std::fmt::Debug {
    fn backend_kind(&self) -> crate::PhysicsBackendKind;

    fn recreate(&self) -> Result<Self, ReferencePhysicsError>
    where
        Self: Sized;

    fn sweep_axis(
        &mut self,
        request: GroundedCapsuleSweepRequest<'_>,
    ) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReferenceGroundedCapsuleQuery;

impl GroundedCapsuleQuery for ReferenceGroundedCapsuleQuery {
    fn backend_kind(&self) -> crate::PhysicsBackendKind {
        crate::PhysicsBackendKind::Reference
    }

    fn recreate(&self) -> Result<Self, ReferencePhysicsError> {
        Ok(*self)
    }

    fn sweep_axis(
        &mut self,
        request: GroundedCapsuleSweepRequest<'_>,
    ) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError> {
        reference_grounded_capsule_sweep(request)
    }
}

pub fn reference_grounded_capsule_sweep(
    request: GroundedCapsuleSweepRequest<'_>,
) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError> {
    if request.delta_micrometres == 0 {
        return Ok(GroundedCapsuleSweepResult {
            applied_delta_micrometres: 0,
            hit: None,
        });
    }
    let axis = usize::from(request.axis);
    if axis >= 3 {
        return Err(ReferencePhysicsError::BackendFailure);
    }
    let mut applied = request.delta_micrometres;
    let mut selected = None;
    for shape in request.static_boxes {
        if !grounded_capsule_collision_filter(
            request.capsule_collision_layer,
            request.capsule_collision_mask,
            shape,
        ) {
            continue;
        }
        let Some((lower, upper)) = expanded_axis_interval(
            request.centre_micrometres,
            axis,
            request.capsule_radius_micrometres,
            request.capsule_half_segment_micrometres,
            shape,
        )?
        else {
            continue;
        };
        let start = request.centre_micrometres[axis];
        let end = start
            .checked_add(applied)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let hit = if request.delta_micrometres > 0 && start <= lower && end > lower {
            Some((
                lower - start,
                negative_axis_normal(axis),
                negative_face(axis),
            ))
        } else if request.delta_micrometres < 0 && start >= upper && end < upper {
            Some((
                upper - start,
                positive_axis_normal(axis),
                positive_face(axis),
            ))
        } else {
            None
        };
        if let Some((candidate, normal, feature)) = hit
            && (candidate.unsigned_abs() < applied.unsigned_abs()
                || (candidate.unsigned_abs() == applied.unsigned_abs()
                    && selected.is_none_or(|existing: GroundedCapsuleSweepHit| {
                        shape.shape_id < existing.shape_id
                    })))
        {
            applied = candidate;
            selected = Some(GroundedCapsuleSweepHit {
                shape_id: shape.shape_id,
                box_feature: feature,
                normal_box_to_capsule: normal,
            });
        }
    }
    Ok(GroundedCapsuleSweepResult {
        applied_delta_micrometres: applied,
        hit: selected,
    })
}

pub fn grounded_capsule_collision_filter(
    capsule_collision_layer: u8,
    capsule_collision_mask: u64,
    shape: &GroundedCapsuleStaticBox,
) -> bool {
    let capsule_to_box = capsule_collision_mask & (1_u64 << u32::from(shape.collision_layer)) != 0;
    let box_to_capsule = shape.collision_mask & (1_u64 << u32::from(capsule_collision_layer)) != 0;
    capsule_to_box && box_to_capsule
}

pub fn grounded_capsule_axis_hit(
    shape_id: PhysicsShapeIdV1,
    axis: u8,
    delta_micrometres: i64,
) -> Result<GroundedCapsuleSweepHit, ReferencePhysicsError> {
    let axis = usize::from(axis);
    if axis >= 3 || delta_micrometres == 0 {
        return Err(ReferencePhysicsError::BackendFailure);
    }
    Ok(if delta_micrometres > 0 {
        GroundedCapsuleSweepHit {
            shape_id,
            box_feature: negative_face(axis),
            normal_box_to_capsule: negative_axis_normal(axis),
        }
    } else {
        GroundedCapsuleSweepHit {
            shape_id,
            box_feature: positive_face(axis),
            normal_box_to_capsule: positive_axis_normal(axis),
        }
    })
}

pub(super) fn validate_reference_shape(
    shape: &PhysicsShapeDescriptorV1,
) -> Result<(), ReferencePhysicsError> {
    shape.validate()?;
    if shape.participation != PhysicsParticipationV1::Solid
        || shape.local_pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
    {
        return Err(ReferencePhysicsError::UnsupportedProfile);
    }
    Ok(())
}

fn expanded_axis_interval(
    centre: [i64; 3],
    axis: usize,
    radius: i64,
    half_segment: i64,
    shape: &GroundedCapsuleStaticBox,
) -> Result<Option<(i64, i64)>, ReferencePhysicsError> {
    let mut perpendicular_squared = 0_i128;
    for other in 0..3 {
        if other == axis {
            continue;
        }
        let distance = if other == 1 {
            let segment_min = centre[1]
                .checked_sub(half_segment)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let segment_max = centre[1]
                .checked_add(half_segment)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            interval_interval_distance(segment_min, segment_max, shape.minimum[1], shape.maximum[1])
        } else {
            interval_distance(centre[other], shape.minimum[other], shape.maximum[other])
        };
        perpendicular_squared = perpendicular_squared
            .checked_add(square(distance)?)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
    }
    let radius_squared = square(radius)?;
    if perpendicular_squared > radius_squared {
        return Ok(None);
    }
    let radial_reach = ceil_sqrt(
        radius_squared
            .checked_sub(perpendicular_squared)
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
    )?;
    let axis_reach = if axis == 1 {
        half_segment
            .checked_add(radial_reach)
            .ok_or(ReferencePhysicsError::NumericOverflow)?
    } else {
        radial_reach
    };
    Ok(Some((
        shape.minimum[axis]
            .checked_sub(axis_reach)
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
        shape.maximum[axis]
            .checked_add(axis_reach)
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
    )))
}

fn interval_distance(value: i64, minimum: i64, maximum: i64) -> i64 {
    if value < minimum {
        minimum - value
    } else if value > maximum {
        value - maximum
    } else {
        0
    }
}

fn interval_interval_distance(
    first_minimum: i64,
    first_maximum: i64,
    second_minimum: i64,
    second_maximum: i64,
) -> i64 {
    if first_maximum < second_minimum {
        second_minimum - first_maximum
    } else if first_minimum > second_maximum {
        first_minimum - second_maximum
    } else {
        0
    }
}

pub(super) fn capsule_box_distance_squared(
    centre: [i64; 3],
    half_segment: i64,
    shape: &GroundedCapsuleStaticBox,
) -> Result<i128, ReferencePhysicsError> {
    let dx = interval_distance(centre[0], shape.minimum[0], shape.maximum[0]);
    let dz = interval_distance(centre[2], shape.minimum[2], shape.maximum[2]);
    let segment_minimum = centre[1]
        .checked_sub(half_segment)
        .ok_or(ReferencePhysicsError::NumericOverflow)?;
    let segment_maximum = centre[1]
        .checked_add(half_segment)
        .ok_or(ReferencePhysicsError::NumericOverflow)?;
    let dy = interval_interval_distance(
        segment_minimum,
        segment_maximum,
        shape.minimum[1],
        shape.maximum[1],
    );
    square(dx)?
        .checked_add(square(dy)?)
        .and_then(|value| value.checked_add(square(dz).ok()?))
        .ok_or(ReferencePhysicsError::NumericOverflow)
}

pub(super) fn square(value: i64) -> Result<i128, ReferencePhysicsError> {
    let value = i128::from(value);
    value
        .checked_mul(value)
        .ok_or(ReferencePhysicsError::NumericOverflow)
}

fn ceil_sqrt(value: i128) -> Result<i64, ReferencePhysicsError> {
    if value < 0 {
        return Err(ReferencePhysicsError::NumericOverflow);
    }
    let value = u128::try_from(value).map_err(|_| ReferencePhysicsError::NumericOverflow)?;
    let mut low = 0_u128;
    let mut high = value.min(u128::from(u64::MAX));
    while low < high {
        let middle = low + (high - low) / 2;
        let square = middle
            .checked_mul(middle)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        if square >= value {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    i64::try_from(low).map_err(|_| ReferencePhysicsError::NumericOverflow)
}

fn negative_axis_normal(axis: usize) -> [i32; 3] {
    let mut normal = [0; 3];
    normal[axis] = -Q1_30_ONE;
    normal
}

fn positive_axis_normal(axis: usize) -> [i32; 3] {
    let mut normal = [0; 3];
    normal[axis] = Q1_30_ONE;
    normal
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

pub(super) fn contact_normal_and_feature(
    centre: [i64; 3],
    half_segment: i64,
    shape: &GroundedCapsuleStaticBox,
) -> Result<([i32; 3], u8), ReferencePhysicsError> {
    let segment_minimum = centre[1]
        .checked_sub(half_segment)
        .ok_or(ReferencePhysicsError::NumericOverflow)?;
    let segment_maximum = centre[1]
        .checked_add(half_segment)
        .ok_or(ReferencePhysicsError::NumericOverflow)?;
    let candidates = [
        (centre[0] < shape.minimum[0]).then(|| {
            (
                0,
                false,
                i64::try_from(centre[0].abs_diff(shape.minimum[0]))
                    .map_err(|_| ReferencePhysicsError::NumericOverflow),
            )
        }),
        (centre[0] > shape.maximum[0]).then(|| {
            (
                0,
                true,
                i64::try_from(centre[0].abs_diff(shape.maximum[0]))
                    .map_err(|_| ReferencePhysicsError::NumericOverflow),
            )
        }),
        (segment_maximum < shape.minimum[1]).then(|| {
            (
                1,
                false,
                i64::try_from(segment_maximum.abs_diff(shape.minimum[1]))
                    .map_err(|_| ReferencePhysicsError::NumericOverflow),
            )
        }),
        (segment_minimum > shape.maximum[1]).then(|| {
            (
                1,
                true,
                i64::try_from(segment_minimum.abs_diff(shape.maximum[1]))
                    .map_err(|_| ReferencePhysicsError::NumericOverflow),
            )
        }),
        (centre[2] < shape.minimum[2]).then(|| {
            (
                2,
                false,
                i64::try_from(centre[2].abs_diff(shape.minimum[2]))
                    .map_err(|_| ReferencePhysicsError::NumericOverflow),
            )
        }),
        (centre[2] > shape.maximum[2]).then(|| {
            (
                2,
                true,
                i64::try_from(centre[2].abs_diff(shape.maximum[2]))
                    .map_err(|_| ReferencePhysicsError::NumericOverflow),
            )
        }),
    ];
    let outside = candidates
        .into_iter()
        .flatten()
        .map(|(axis, positive, distance)| Ok((axis, positive, distance?)))
        .collect::<Result<Vec<_>, ReferencePhysicsError>>()?;
    match outside.as_slice() {
        [] => Ok((positive_axis_normal(1), positive_face(1))),
        [(axis, positive, _)] => Ok((
            if *positive {
                positive_axis_normal(*axis)
            } else {
                negative_axis_normal(*axis)
            },
            if *positive {
                positive_face(*axis)
            } else {
                negative_face(*axis)
            },
        )),
        _ => Ok((
            normalized_feature_normal(&outside)?,
            primitive_box_feature(&outside)?,
        )),
    }
}

fn primitive_box_feature(outside: &[(usize, bool, i64)]) -> Result<u8, ReferencePhysicsError> {
    match outside {
        [
            (first_axis, first_positive, _),
            (second_axis, second_positive, _),
        ] => {
            let pair_slot = match (*first_axis, *second_axis) {
                (0, 1) => 0,
                (0, 2) => 1,
                (1, 2) => 2,
                _ => return Err(ReferencePhysicsError::SnapshotMismatch),
            };
            let sign_slot = u8::from(*first_positive) * 2 + u8::from(*second_positive);
            Ok(7 + pair_slot * 4 + sign_slot)
        }
        [(0, x_positive, _), (1, y_positive, _), (2, z_positive, _)] => {
            Ok(19 + u8::from(*x_positive) * 4 + u8::from(*y_positive) * 2 + u8::from(*z_positive))
        }
        _ => Err(ReferencePhysicsError::SnapshotMismatch),
    }
}

fn normalized_feature_normal(
    outside: &[(usize, bool, i64)],
) -> Result<[i32; 3], ReferencePhysicsError> {
    let squared_length = outside.iter().try_fold(0_i128, |sum, (_, _, distance)| {
        sum.checked_add(square(*distance)?)
            .ok_or(ReferencePhysicsError::NumericOverflow)
    })?;
    let length = i128::from(ceil_sqrt(squared_length)?);
    if length == 0 {
        return Err(ReferencePhysicsError::NumericOverflow);
    }
    let mut normal = [0; 3];
    for (axis, positive, distance) in outside {
        let magnitude = i128::from(*distance)
            .checked_mul(i128::from(Q1_30_ONE))
            .ok_or(ReferencePhysicsError::NumericOverflow)?
            / length;
        let signed = if *positive { magnitude } else { -magnitude };
        normal[*axis] =
            i32::try_from(signed).map_err(|_| ReferencePhysicsError::NumericOverflow)?;
    }
    Ok(normal)
}
