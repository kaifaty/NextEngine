use next_contracts::physics::{
    PhysicsBodyIdV1, PhysicsBodyStateV2, PhysicsContactReportingV1, PhysicsShapeIdV1,
};

use super::error::ReferencePhysicsError;
use super::query::GroundedCapsuleStaticBox;
use super::world::{checked_add_vec3, checked_sub_vec3};

const Q1_30_ONE: i32 = 1 << 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GroundedCapsuleAttachedBox {
    pub shape_id: PhysicsShapeIdV1,
    pub local_centre_micrometres: [i64; 3],
    pub half_extents_micrometres: [i64; 3],
    pub contact_reporting: PhysicsContactReportingV1,
    pub collision_layer: u8,
    pub collision_mask: u64,
}

impl GroundedCapsuleAttachedBox {
    pub fn at_body_centre(
        &self,
        body_centre_micrometres: [i64; 3],
    ) -> Result<GroundedCapsuleStaticBox, ReferencePhysicsError> {
        let centre = checked_add_vec3(body_centre_micrometres, self.local_centre_micrometres)?;
        Ok(GroundedCapsuleStaticBox {
            shape_id: self.shape_id,
            minimum: checked_sub_vec3(centre, self.half_extents_micrometres)?,
            maximum: checked_add_vec3(centre, self.half_extents_micrometres)?,
            contact_reporting: self.contact_reporting,
            collision_layer: self.collision_layer,
            collision_mask: self.collision_mask,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GroundedCapsuleDynamicBox {
    pub body_id: PhysicsBodyIdV1,
    pub shape_id: PhysicsShapeIdV1,
    pub local_centre_micrometres: [i64; 3],
    pub half_extents_micrometres: [i64; 3],
    pub contact_reporting: PhysicsContactReportingV1,
    pub collision_layer: u8,
    pub collision_mask: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct BoxSweepHit {
    pub shape_id: PhysicsShapeIdV1,
    pub box_feature: u8,
    pub moving_box_feature: u8,
    pub normal_box_to_moving: [i32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BoxSweepResult {
    pub applied_delta_micrometres: i64,
    pub hit: Option<BoxSweepHit>,
}

impl GroundedCapsuleDynamicBox {
    pub fn at_state(
        &self,
        state: &PhysicsBodyStateV2,
    ) -> Result<GroundedCapsuleStaticBox, ReferencePhysicsError> {
        if state.body_id != self.body_id {
            return Err(ReferencePhysicsError::SnapshotMismatch);
        }
        let centre = checked_add_vec3(
            state.pose.translation_micrometres,
            self.local_centre_micrometres,
        )?;
        Ok(GroundedCapsuleStaticBox {
            shape_id: self.shape_id,
            minimum: checked_sub_vec3(centre, self.half_extents_micrometres)?,
            maximum: checked_add_vec3(centre, self.half_extents_micrometres)?,
            contact_reporting: self.contact_reporting,
            collision_layer: self.collision_layer,
            collision_mask: self.collision_mask,
        })
    }
}

pub(super) fn sweep_box_axis(
    moving: &GroundedCapsuleStaticBox,
    obstacles: &[GroundedCapsuleStaticBox],
    axis: usize,
    delta: i64,
) -> Result<i64, ReferencePhysicsError> {
    Ok(sweep_box_axis_with_hit(moving, obstacles, axis, delta)?.applied_delta_micrometres)
}

pub(super) fn sweep_box_axis_with_hit(
    moving: &GroundedCapsuleStaticBox,
    obstacles: &[GroundedCapsuleStaticBox],
    axis: usize,
    delta: i64,
) -> Result<BoxSweepResult, ReferencePhysicsError> {
    if axis >= 3 {
        return Err(ReferencePhysicsError::BackendFailure);
    }
    if delta == 0 {
        return Ok(BoxSweepResult {
            applied_delta_micrometres: 0,
            hit: None,
        });
    }
    let mut applied = delta;
    let mut selected = None;
    for obstacle in obstacles {
        if obstacle.shape_id == moving.shape_id
            || !(0..3).filter(|other| *other != axis).all(|other| {
                moving.minimum[other] < obstacle.maximum[other]
                    && moving.maximum[other] > obstacle.minimum[other]
            })
        {
            continue;
        }
        let candidate = if delta > 0 && moving.maximum[axis] <= obstacle.minimum[axis] {
            let end = moving.maximum[axis]
                .checked_add(applied)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            (end > obstacle.minimum[axis]).then_some(
                obstacle.minimum[axis]
                    .checked_sub(moving.maximum[axis])
                    .ok_or(ReferencePhysicsError::NumericOverflow)?,
            )
        } else if delta < 0 && moving.minimum[axis] >= obstacle.maximum[axis] {
            let end = moving.minimum[axis]
                .checked_add(applied)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            (end < obstacle.maximum[axis]).then_some(
                obstacle.maximum[axis]
                    .checked_sub(moving.minimum[axis])
                    .ok_or(ReferencePhysicsError::NumericOverflow)?,
            )
        } else {
            None
        };
        if let Some(candidate) = candidate {
            let hit = BoxSweepHit {
                shape_id: obstacle.shape_id,
                box_feature: if delta > 0 {
                    negative_face(axis)
                } else {
                    positive_face(axis)
                },
                moving_box_feature: if delta > 0 {
                    positive_face(axis)
                } else {
                    negative_face(axis)
                },
                normal_box_to_moving: if delta > 0 {
                    negative_axis_normal(axis)
                } else {
                    positive_axis_normal(axis)
                },
            };
            if candidate.unsigned_abs() < applied.unsigned_abs()
                || (candidate.unsigned_abs() == applied.unsigned_abs()
                    && selected.is_none_or(|existing| hit < existing))
            {
                applied = candidate;
                selected = Some(hit);
            }
        }
    }
    Ok(BoxSweepResult {
        applied_delta_micrometres: applied,
        hit: selected,
    })
}

pub(super) fn boxes_penetrate(
    left: &GroundedCapsuleStaticBox,
    right: &GroundedCapsuleStaticBox,
) -> bool {
    (0..3).all(|axis| {
        left.minimum[axis] < right.maximum[axis] && left.maximum[axis] > right.minimum[axis]
    })
}

pub(super) fn boxes_touch_or_overlap(
    left: &GroundedCapsuleStaticBox,
    right: &GroundedCapsuleStaticBox,
) -> bool {
    (0..3).all(|axis| {
        left.minimum[axis] <= right.maximum[axis] && left.maximum[axis] >= right.minimum[axis]
    })
}

pub(super) fn box_contact_normal_and_features(
    moving: &GroundedCapsuleStaticBox,
    obstacle: &GroundedCapsuleStaticBox,
) -> Result<([i32; 3], u8, u8), ReferencePhysicsError> {
    if !boxes_touch_or_overlap(moving, obstacle) {
        return Err(ReferencePhysicsError::SnapshotMismatch);
    }
    let moving_centre = box_centre(moving)?;
    let obstacle_centre = box_centre(obstacle)?;
    let mut selected = None;
    for axis in 0..3 {
        let (penetration, normal, moving_feature, obstacle_feature) =
            if moving_centre[axis] <= obstacle_centre[axis] {
                (
                    moving.maximum[axis]
                        .checked_sub(obstacle.minimum[axis])
                        .ok_or(ReferencePhysicsError::NumericOverflow)?,
                    negative_axis_normal(axis),
                    positive_face(axis),
                    negative_face(axis),
                )
            } else {
                (
                    obstacle.maximum[axis]
                        .checked_sub(moving.minimum[axis])
                        .ok_or(ReferencePhysicsError::NumericOverflow)?,
                    positive_axis_normal(axis),
                    negative_face(axis),
                    positive_face(axis),
                )
            };
        let key = (penetration, axis);
        if selected.is_none_or(|(selected_key, _, _, _)| key < selected_key) {
            selected = Some((key, normal, moving_feature, obstacle_feature));
        }
    }
    selected
        .map(|(_, normal, moving_feature, obstacle_feature)| {
            (normal, moving_feature, obstacle_feature)
        })
        .ok_or(ReferencePhysicsError::SnapshotMismatch)
}

fn box_centre(shape: &GroundedCapsuleStaticBox) -> Result<[i64; 3], ReferencePhysicsError> {
    let mut centre = [0; 3];
    for (axis, value) in centre.iter_mut().enumerate() {
        *value = shape.minimum[axis]
            .checked_add(shape.maximum[axis])
            .ok_or(ReferencePhysicsError::NumericOverflow)?
            .checked_div(2)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
    }
    Ok(centre)
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
