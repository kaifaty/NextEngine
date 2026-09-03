use std::collections::{BTreeMap, BTreeSet};

use next_contracts::physics::{
    CAPSULE_GROUND_SNAP_DISTANCE_MICROMETRES, CAPSULE_MAX_STEP_HEIGHT_MICROMETRES,
    ExternalImpulseV1, PhysicsBodyIdV1, PhysicsBodyStateV2, PhysicsCanonicalSnapshotV2,
    PhysicsContactReportingV1, PhysicsShapeIdV1, velocity_delta_micrometres_per_second,
};

use super::error::ReferencePhysicsError;
use super::interaction::{sweep_box_axis, sweep_box_axis_with_hit};
use super::query::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, GroundedCapsuleSweepHit,
    GroundedCapsuleSweepRequest, GroundedCapsuleSweepResult, grounded_capsule_collision_filter,
    reference_grounded_capsule_sweep,
};
use super::world::{GroundedCapsuleWorld, checked_add_vec3, checked_sub_vec3};

pub(super) struct CapsuleSubstep {
    pub body_after: PhysicsBodyStateV2,
    pub forced_hits: BTreeSet<AvatarSweepHit>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct AvatarSweepHit {
    pub avatar_shape_id: PhysicsShapeIdV1,
    pub avatar_feature: u8,
    pub other: GroundedCapsuleSweepHit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AvatarSweepResult {
    applied_delta_micrometres: i64,
    hit: Option<AvatarSweepHit>,
}

impl<Q: GroundedCapsuleQuery> GroundedCapsuleWorld<Q> {
    pub(super) fn integrate_capsule_substep(
        &mut self,
        staged: &mut PhysicsCanonicalSnapshotV2,
        body_before: &PhysicsBodyStateV2,
        direction: [i16; 2],
        impulses: &[ExternalImpulseV1],
    ) -> Result<CapsuleSubstep, ReferencePhysicsError> {
        let dynamic_before = self.prepare_dynamic_states(staged)?;
        // WR1: every dynamic box falls and finds support before the capsule
        // integrates against the boxes' new poses.
        self.integrate_dynamic_boxes(
            staged,
            Some(body_before.pose.translation_micrometres),
            impulses,
        )?;
        let mut body_after = body_before.clone();
        let mut forced_hits = BTreeSet::new();
        body_after.linear_velocity_micrometres_per_second[1] = body_after
            .linear_velocity_micrometres_per_second[1]
            .checked_add(self.gravity_velocity_delta)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let physics_hz = i64::from(self.tick_rate_profile.physics_hz());
        let vertical_delta = body_after.linear_velocity_micrometres_per_second[1]
            .checked_div(physics_hz)
            .ok_or(ReferencePhysicsError::NonIntegralProfile)?;
        let vertical = self.sweep_solids_axis(
            staged,
            body_after.pose.translation_micrometres,
            1,
            vertical_delta,
        )?;
        body_after.pose.translation_micrometres[1] = body_after.pose.translation_micrometres[1]
            .checked_add(vertical.applied_delta_micrometres)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let grounded = vertical_delta <= 0
            && vertical.hit.is_some_and(|hit| {
                Some(hit.avatar_shape_id) == self.capsule_shape_id
                    && hit.other.normal_box_to_capsule[1] > 0
            });
        if vertical.applied_delta_micrometres != vertical_delta {
            body_after.linear_velocity_micrometres_per_second[1] = 0;
        }
        forced_hits.extend(vertical.hit);

        for (axis, component) in [(0, direction[0]), (2, direction[1])] {
            let delta = self
                .locomotion_per_substep
                .checked_mul(i64::from(component.signum()))
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let movement = self.move_horizontal(
                staged,
                body_after.pose.translation_micrometres,
                axis,
                delta,
                grounded,
            )?;
            body_after.pose.translation_micrometres = movement.centre;
            forced_hits.extend(movement.forced_hits);
        }

        self.finish_dynamic_states(staged, dynamic_before)?;
        Ok(CapsuleSubstep {
            body_after,
            forced_hits,
        })
    }

    fn move_horizontal(
        &mut self,
        staged: &mut PhysicsCanonicalSnapshotV2,
        centre: [i64; 3],
        axis: usize,
        delta: i64,
        grounded: bool,
    ) -> Result<HorizontalMovement, ReferencePhysicsError> {
        if delta == 0 {
            return Ok(HorizontalMovement::stationary(centre));
        }
        let direct = self.sweep_solids_axis(staged, centre, axis, delta)?;
        let mut applied = direct.applied_delta_micrometres;
        let mut selected_hit = direct.hit;
        let hit_is_dynamic = selected_hit.is_some_and(|hit| {
            self.dynamic_boxes
                .iter()
                .any(|binding| binding.shape_id == hit.other.shape_id)
        });
        if hit_is_dynamic {
            let remaining = delta
                .checked_sub(applied)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let pushed = self.push_dynamic_box(
                staged,
                selected_hit
                    .expect("dynamic hit was checked")
                    .other
                    .shape_id,
                axis,
                remaining,
            )?;
            applied = applied
                .checked_add(pushed)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
        } else if grounded
            && applied != delta
            && let Some(blocking_hit) = selected_hit
            && Some(blocking_hit.avatar_shape_id) == self.capsule_shape_id
            && let Some(stepped) =
                self.try_step(staged, centre, axis, delta, applied, blocking_hit.other)?
        {
            return Ok(stepped);
        }

        let mut moved = centre;
        moved[axis] = moved[axis]
            .checked_add(applied)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let mut forced_hits = BTreeSet::new();
        forced_hits.extend(selected_hit.take());
        if grounded && applied != 0 {
            self.snap_to_ground(staged, &mut moved, &mut forced_hits)?;
        }
        Ok(HorizontalMovement {
            centre: moved,
            forced_hits,
        })
    }

    fn try_step(
        &mut self,
        staged: &PhysicsCanonicalSnapshotV2,
        centre: [i64; 3],
        axis: usize,
        delta: i64,
        direct_applied: i64,
        blocking_hit: GroundedCapsuleSweepHit,
    ) -> Result<Option<HorizontalMovement>, ReferencePhysicsError> {
        let Some(target) = self
            .static_boxes
            .iter()
            .find(|shape| shape.shape_id == blocking_hit.shape_id)
        else {
            return Ok(None);
        };
        let capsule_bottom = centre[1]
            .checked_sub(self.capsule_half_segment)
            .and_then(|value| value.checked_sub(self.capsule_radius))
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let support_height = self
            .static_boxes
            .iter()
            .filter(|shape| {
                shape.shape_id != target.shape_id
                    && (shape.minimum[0]..=shape.maximum[0]).contains(&centre[0])
                    && (shape.minimum[2]..=shape.maximum[2]).contains(&centre[2])
                    && shape.maximum[1] <= capsule_bottom
            })
            .map(|shape| shape.maximum[1])
            .max();
        let Some(support_height) = support_height else {
            return Ok(None);
        };
        let rise = target.maximum[1]
            .checked_sub(support_height)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        if rise <= 0 || rise > CAPSULE_MAX_STEP_HEIGHT_MICROMETRES {
            return Ok(None);
        }
        let up = self.sweep_solids_axis(staged, centre, 1, CAPSULE_MAX_STEP_HEIGHT_MICROMETRES)?;
        if up.applied_delta_micrometres != CAPSULE_MAX_STEP_HEIGHT_MICROMETRES {
            return Ok(None);
        }
        let mut raised = centre;
        raised[1] = raised[1]
            .checked_add(CAPSULE_MAX_STEP_HEIGHT_MICROMETRES)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let across = self.sweep_solids_axis(staged, raised, axis, delta)?;
        if across.applied_delta_micrometres.unsigned_abs() <= direct_applied.unsigned_abs()
            || across.hit.is_some_and(|hit| {
                self.dynamic_boxes
                    .iter()
                    .any(|binding| binding.shape_id == hit.other.shape_id)
            })
        {
            return Ok(None);
        }
        raised[axis] = raised[axis]
            .checked_add(across.applied_delta_micrometres)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        // The sweep contract reports a hit only after crossing a boundary. Probe one
        // canonical micrometre beyond the raised height so a surface exactly at the
        // maximum step distance is classified as support instead of a horizontal
        // blocker at its leading edge. The committed displacement remains the exact
        // quantized hit distance returned by the sweep.
        let down_probe = CAPSULE_MAX_STEP_HEIGHT_MICROMETRES
            .checked_add(1)
            .and_then(i64::checked_neg)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let down = self.sweep_solids_axis(staged, raised, 1, down_probe)?;
        if !down.hit.is_some_and(|hit| {
            Some(hit.avatar_shape_id) == self.capsule_shape_id
                && hit.other.normal_box_to_capsule[1] > 0
        }) {
            return Ok(None);
        }
        raised[1] = raised[1]
            .checked_add(down.applied_delta_micrometres)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        let forced_hits = [across.hit, down.hit].into_iter().flatten().collect();
        Ok(Some(HorizontalMovement {
            centre: raised,
            forced_hits,
        }))
    }

    fn snap_to_ground(
        &mut self,
        staged: &PhysicsCanonicalSnapshotV2,
        centre: &mut [i64; 3],
        forced_hits: &mut BTreeSet<AvatarSweepHit>,
    ) -> Result<(), ReferencePhysicsError> {
        let down = self.sweep_solids_axis(
            staged,
            *centre,
            1,
            -CAPSULE_GROUND_SNAP_DISTANCE_MICROMETRES,
        )?;
        if down.hit.is_some_and(|hit| {
            Some(hit.avatar_shape_id) == self.capsule_shape_id
                && hit.other.normal_box_to_capsule[1] > 0
        }) {
            centre[1] = centre[1]
                .checked_add(down.applied_delta_micrometres)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            forced_hits.extend(down.hit);
        }
        Ok(())
    }

    fn sweep_solids_axis(
        &mut self,
        staged: &PhysicsCanonicalSnapshotV2,
        centre: [i64; 3],
        axis: usize,
        delta: i64,
    ) -> Result<AvatarSweepResult, ReferencePhysicsError> {
        let axis = u8::try_from(axis).map_err(|_| ReferencePhysicsError::BackendFailure)?;
        let request = |static_boxes| GroundedCapsuleSweepRequest {
            centre_micrometres: centre,
            axis,
            delta_micrometres: delta,
            capsule_radius_micrometres: self.capsule_radius,
            capsule_half_segment_micrometres: self.capsule_half_segment,
            capsule_collision_layer: self.capsule_collision_layer,
            capsule_collision_mask: self.capsule_collision_mask,
            static_boxes,
        };
        let static_result = self.query.sweep_axis(request(&self.static_boxes))?;
        let dynamic_boxes = self.current_dynamic_boxes(staged)?;
        let dynamic_result = reference_grounded_capsule_sweep(request(&dynamic_boxes))?;
        let capsule_shape_id = self
            .capsule_shape_id
            .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
        let mut selected = select_nearest_sweep(
            AvatarSweepResult::from_capsule(capsule_shape_id, static_result),
            AvatarSweepResult::from_capsule(capsule_shape_id, dynamic_result),
        );
        let mut obstacles = self.static_boxes.to_vec();
        obstacles.extend(dynamic_boxes);
        obstacles.sort_by_key(|shape| shape.shape_id);
        for attached in self.attached_boxes.iter() {
            let moving = attached.at_body_centre(centre)?;
            let eligible = obstacles
                .iter()
                .filter(|shape| {
                    grounded_capsule_collision_filter(
                        moving.collision_layer,
                        moving.collision_mask,
                        shape,
                    )
                })
                .cloned()
                .collect::<Vec<_>>();
            let result = sweep_box_axis_with_hit(&moving, &eligible, usize::from(axis), delta)?;
            selected = select_nearest_sweep(
                selected,
                AvatarSweepResult {
                    applied_delta_micrometres: result.applied_delta_micrometres,
                    hit: result.hit.map(|hit| AvatarSweepHit {
                        avatar_shape_id: attached.shape_id,
                        avatar_feature: hit.moving_box_feature,
                        other: GroundedCapsuleSweepHit {
                            shape_id: hit.shape_id,
                            box_feature: hit.box_feature,
                            normal_box_to_capsule: hit.normal_box_to_moving,
                        },
                    }),
                },
            );
        }
        Ok(selected)
    }

    pub(super) fn current_dynamic_boxes(
        &self,
        staged: &PhysicsCanonicalSnapshotV2,
    ) -> Result<Vec<GroundedCapsuleStaticBox>, ReferencePhysicsError> {
        self.dynamic_boxes
            .iter()
            .map(|binding| {
                let state = staged
                    .sorted_body_states
                    .get(&binding.body_id)
                    .ok_or(ReferencePhysicsError::BodyMissing)?;
                binding.at_state(state)
            })
            .collect()
    }

    pub(super) fn contact_boxes(
        &self,
        staged: &PhysicsCanonicalSnapshotV2,
    ) -> Result<Vec<GroundedCapsuleStaticBox>, ReferencePhysicsError> {
        let mut boxes = self.static_boxes.to_vec();
        boxes.extend(self.current_dynamic_boxes(staged)?);
        boxes.extend(self.sensor_boxes.iter().cloned());
        boxes.sort_by_key(|shape| shape.shape_id);
        Ok(boxes)
    }

    fn push_dynamic_box(
        &self,
        staged: &mut PhysicsCanonicalSnapshotV2,
        shape_id: next_contracts::physics::PhysicsShapeIdV1,
        axis: usize,
        delta: i64,
    ) -> Result<i64, ReferencePhysicsError> {
        if delta == 0 {
            return Ok(0);
        }
        let binding = self
            .dynamic_boxes
            .iter()
            .find(|binding| binding.shape_id == shape_id)
            .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
        let state = staged
            .sorted_body_states
            .get(&binding.body_id)
            .cloned()
            .ok_or(ReferencePhysicsError::BodyMissing)?;
        let moving = binding.at_state(&state)?;
        // The pushed box stops at static solids and at the other dynamic
        // boxes (no chain push); the pusher itself is never an obstacle.
        let obstacles = self.box_obstacles(staged, &moving, None)?;
        let applied = sweep_box_axis(&moving, &obstacles, axis, delta)?;
        let state = staged
            .sorted_body_states
            .get_mut(&binding.body_id)
            .ok_or(ReferencePhysicsError::BodyMissing)?;
        state.pose.translation_micrometres[axis] = state.pose.translation_micrometres[axis]
            .checked_add(applied)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        state.linear_velocity_micrometres_per_second[axis] = applied
            .checked_mul(i64::from(self.tick_rate_profile.physics_hz()))
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        Ok(applied)
    }

    /// Obstacles for a moving dynamic box: the static solids, the other
    /// dynamic boxes at their current staged poses and, when given, the
    /// capsule's axis-aligned bounds plus its attached carried boxes; all
    /// filtered by the moving box's layer and mask.
    fn box_obstacles(
        &self,
        staged: &PhysicsCanonicalSnapshotV2,
        moving: &GroundedCapsuleStaticBox,
        capsule_centre: Option<[i64; 3]>,
    ) -> Result<Vec<GroundedCapsuleStaticBox>, ReferencePhysicsError> {
        let mut obstacles = self.static_boxes.to_vec();
        obstacles.extend(
            self.current_dynamic_boxes(staged)?
                .into_iter()
                .filter(|shape| shape.shape_id != moving.shape_id),
        );
        if let Some(centre) = capsule_centre
            && let Some(capsule_shape_id) = self.capsule_shape_id
        {
            let reach = [
                self.capsule_radius,
                self.capsule_half_segment
                    .checked_add(self.capsule_radius)
                    .ok_or(ReferencePhysicsError::NumericOverflow)?,
                self.capsule_radius,
            ];
            obstacles.push(GroundedCapsuleStaticBox {
                shape_id: capsule_shape_id,
                minimum: checked_sub_vec3(centre, reach)?,
                maximum: checked_add_vec3(centre, reach)?,
                contact_reporting: PhysicsContactReportingV1::Disabled,
                collision_layer: self.capsule_collision_layer,
                collision_mask: self.capsule_collision_mask,
            });
            for attached in self.attached_boxes.iter() {
                obstacles.push(attached.at_body_centre(centre)?);
            }
        }
        obstacles.retain(|shape| {
            grounded_capsule_collision_filter(moving.collision_layer, moving.collision_mask, shape)
        });
        obstacles.sort_by_key(|shape| shape.shape_id);
        Ok(obstacles)
    }

    /// WR1: one vertical free-body substep for every dynamic box in shape-id
    /// order: gravity into the vertical velocity, a swept move against the
    /// box obstacles, and inelastic support when the sweep is cut.
    pub(super) fn integrate_dynamic_boxes(
        &self,
        staged: &mut PhysicsCanonicalSnapshotV2,
        capsule_centre: Option<[i64; 3]>,
        impulses: &[ExternalImpulseV1],
    ) -> Result<(), ReferencePhysicsError> {
        let physics_hz = i64::from(self.tick_rate_profile.physics_hz());
        for binding in self.dynamic_boxes.iter() {
            let state = staged
                .sorted_body_states
                .get(&binding.body_id)
                .cloned()
                .ok_or(ReferencePhysicsError::BodyMissing)?;
            // ADR-105: an external impulse changes the vertical velocity by
            // `J_y / m` once, before gravity. Horizontal components have no
            // effect in the push-only profile.
            let impulse_delta = match impulses
                .iter()
                .find(|impulse| impulse.body_id == binding.body_id)
            {
                Some(impulse) => velocity_delta_micrometres_per_second(
                    impulse.impulse_micronewton_seconds[1],
                    binding.mass_microkilograms,
                )
                .map_err(|_| ReferencePhysicsError::NumericOverflow)?,
                None => 0,
            };
            let mut vertical_velocity = state.linear_velocity_micrometres_per_second[1]
                .checked_add(impulse_delta)
                .and_then(|velocity| velocity.checked_add(self.gravity_velocity_delta))
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let delta = vertical_velocity
                .checked_div(physics_hz)
                .ok_or(ReferencePhysicsError::NonIntegralProfile)?;
            let moving = binding.at_state(&state)?;
            let obstacles = self.box_obstacles(staged, &moving, capsule_centre)?;
            let applied = sweep_box_axis(&moving, &obstacles, 1, delta)?;
            if applied != delta {
                vertical_velocity = 0;
            }
            let state = staged
                .sorted_body_states
                .get_mut(&binding.body_id)
                .ok_or(ReferencePhysicsError::BodyMissing)?;
            state.pose.translation_micrometres[1] = state.pose.translation_micrometres[1]
                .checked_add(applied)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            state.linear_velocity_micrometres_per_second[1] = vertical_velocity;
        }
        Ok(())
    }

    /// Clears the push-derived horizontal velocity of every dynamic box for
    /// the substep; the vertical velocity is free-body state and persists.
    pub(super) fn prepare_dynamic_states(
        &self,
        staged: &mut PhysicsCanonicalSnapshotV2,
    ) -> Result<BTreeMap<PhysicsBodyIdV1, PhysicsBodyStateV2>, ReferencePhysicsError> {
        let mut before = BTreeMap::new();
        for binding in self.dynamic_boxes.iter() {
            let state = staged
                .sorted_body_states
                .get_mut(&binding.body_id)
                .ok_or(ReferencePhysicsError::BodyMissing)?;
            before.insert(binding.body_id, state.clone());
            state.linear_velocity_micrometres_per_second[0] = 0;
            state.linear_velocity_micrometres_per_second[2] = 0;
        }
        Ok(before)
    }

    pub(super) fn finish_dynamic_states(
        &self,
        staged: &mut PhysicsCanonicalSnapshotV2,
        before: BTreeMap<PhysicsBodyIdV1, PhysicsBodyStateV2>,
    ) -> Result<(), ReferencePhysicsError> {
        for (body_id, previous) in before {
            let state = staged
                .sorted_body_states
                .get_mut(&body_id)
                .ok_or(ReferencePhysicsError::BodyMissing)?;
            state.body_revision = if state.pose != previous.pose
                || state.linear_velocity_micrometres_per_second
                    != previous.linear_velocity_micrometres_per_second
            {
                previous
                    .body_revision
                    .checked_add(1)
                    .ok_or(ReferencePhysicsError::NumericOverflow)?
            } else {
                previous.body_revision
            };
        }
        Ok(())
    }
}

struct HorizontalMovement {
    centre: [i64; 3],
    forced_hits: BTreeSet<AvatarSweepHit>,
}

impl HorizontalMovement {
    fn stationary(centre: [i64; 3]) -> Self {
        Self {
            centre,
            forced_hits: BTreeSet::new(),
        }
    }
}

fn select_nearest_sweep(left: AvatarSweepResult, right: AvatarSweepResult) -> AvatarSweepResult {
    match left
        .applied_delta_micrometres
        .unsigned_abs()
        .cmp(&right.applied_delta_micrometres.unsigned_abs())
    {
        std::cmp::Ordering::Less => left,
        std::cmp::Ordering::Greater => right,
        std::cmp::Ordering::Equal => {
            if left.hit <= right.hit {
                left
            } else {
                right
            }
        }
    }
}

impl AvatarSweepResult {
    fn from_capsule(avatar_shape_id: PhysicsShapeIdV1, result: GroundedCapsuleSweepResult) -> Self {
        Self {
            applied_delta_micrometres: result.applied_delta_micrometres,
            hit: result.hit.map(|other| AvatarSweepHit {
                avatar_shape_id,
                avatar_feature: 1,
                other,
            }),
        }
    }
}
