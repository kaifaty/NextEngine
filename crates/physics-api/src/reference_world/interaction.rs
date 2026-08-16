use next_contracts::physics::{
    PhysicsBodyIdV1, PhysicsBodyStateV2, PhysicsContactReportingV1, PhysicsShapeIdV1,
};

use super::error::ReferencePhysicsError;
use super::query::GroundedCapsuleStaticBox;
use super::world::{checked_add_vec3, checked_sub_vec3};

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
    if axis >= 3 {
        return Err(ReferencePhysicsError::BackendFailure);
    }
    if delta == 0 {
        return Ok(0);
    }
    let mut applied = delta;
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
        if let Some(candidate) = candidate
            && candidate.unsigned_abs() < applied.unsigned_abs()
        {
            applied = candidate;
        }
    }
    Ok(applied)
}

pub(super) fn boxes_penetrate(
    left: &GroundedCapsuleStaticBox,
    right: &GroundedCapsuleStaticBox,
) -> bool {
    (0..3).all(|axis| {
        left.minimum[axis] < right.maximum[axis] && left.maximum[axis] > right.minimum[axis]
    })
}
