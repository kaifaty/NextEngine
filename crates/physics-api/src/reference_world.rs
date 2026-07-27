use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    AppliedLocomotionResultV1, AuthoritativeNumericProfileV1,
    CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND, ClosedPhysicsContactBatchV1, ContactEventV1,
    ContactPhaseV1, ContentHash, PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2,
    PhysicsContactContinuityStateV1, PhysicsContactReportingV1, PhysicsContractError,
    PhysicsGeometryV1, PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1,
    PhysicsQuantizationProfileV1, PhysicsShapeDescriptorV1, PhysicsShapeIdV1, PhysicsStepInputV2,
    PhysicsStepResultV1, PhysicsWorldCheckpointV1, TickRateProfileV1, derive_physics_contact_id,
};

const Q1_30_ONE: i32 = 1 << 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferencePhysicsWorld {
    checkpoint: PhysicsWorldCheckpointV1,
    tick_rate_profile: TickRateProfileV1,
    numeric_profile: AuthoritativeNumericProfileV1,
    quantization_profile: PhysicsQuantizationProfileV1,
    capsule_body_id: Option<PhysicsBodyIdV1>,
    capsule_shape_id: Option<PhysicsShapeIdV1>,
    capsule_radius: i64,
    capsule_half_segment: i64,
    capsule_collision_layer: u8,
    capsule_collision_mask: u64,
    static_boxes: Vec<StaticBox>,
    locomotion_per_substep: i64,
    gravity_velocity_delta: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StaticBox {
    shape_id: PhysicsShapeIdV1,
    minimum: [i64; 3],
    maximum: [i64; 3],
    contact_reporting: PhysicsContactReportingV1,
    collision_layer: u8,
    collision_mask: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ContactCandidate {
    shape_id: PhysicsShapeIdV1,
    box_feature: u8,
    point: [i64; 3],
    normal_box_to_capsule: [i32; 3],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SweepHit {
    shape_id: PhysicsShapeIdV1,
    box_feature: u8,
    normal_box_to_capsule: [i32; 3],
}

impl ReferencePhysicsWorld {
    pub fn new(
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Self, ReferencePhysicsError> {
        checkpoint.validate()?;
        tick_rate_profile
            .validate()
            .map_err(PhysicsContractError::from)?;
        numeric_profile.validate()?;
        quantization_profile.validate()?;
        checkpoint.snapshot.validate_profile_closure(
            &checkpoint.catalog,
            &tick_rate_profile,
            &numeric_profile,
            &quantization_profile,
        )?;

        let bindings = &checkpoint.catalog.avatar_bindings;
        if bindings.len() > 1 {
            return Err(ReferencePhysicsError::UnsupportedProfile);
        }
        let capsule_body_id = bindings.values().next().copied();
        let (
            capsule_shape_id,
            capsule_radius,
            capsule_half_segment,
            capsule_collision_layer,
            capsule_collision_mask,
        ) = if let Some(capsule_body_id) = capsule_body_id {
            let capsule_descriptor = checkpoint
                .catalog
                .bodies
                .get(&capsule_body_id)
                .ok_or(ReferencePhysicsError::UnsupportedProfile)?;
            if capsule_descriptor.motion_kind != PhysicsMotionKindV1::Kinematic
                || capsule_descriptor.shapes.len() != 1
                || capsule_descriptor.initial_pose.rotation_q1_30
                    != PhysicsPoseV1::default().rotation_q1_30
            {
                return Err(ReferencePhysicsError::UnsupportedProfile);
            }
            let capsule_shape = capsule_descriptor
                .shapes
                .values()
                .next()
                .ok_or(ReferencePhysicsError::UnsupportedProfile)?;
            validate_reference_shape(capsule_shape)?;
            let (capsule_radius, capsule_half_segment) = match capsule_shape.geometry {
                PhysicsGeometryV1::Capsule {
                    radius_micrometres,
                    half_segment_micrometres,
                } => (radius_micrometres, half_segment_micrometres),
                _ => return Err(ReferencePhysicsError::UnsupportedProfile),
            };
            (
                Some(capsule_shape.shape_id),
                capsule_radius,
                capsule_half_segment,
                capsule_shape.collision_layer,
                capsule_shape.collision_mask,
            )
        } else {
            if checkpoint
                .catalog
                .bodies
                .values()
                .any(|body| body.motion_kind != PhysicsMotionKindV1::Static)
            {
                return Err(ReferencePhysicsError::UnsupportedProfile);
            }
            (None, 0, 0, 0, 0)
        };

        let mut static_boxes = Vec::new();
        for (body_id, body) in &checkpoint.catalog.bodies {
            let state = checkpoint
                .snapshot
                .sorted_body_states
                .get(body_id)
                .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
            if Some(*body_id) == capsule_body_id {
                continue;
            }
            if body.motion_kind != PhysicsMotionKindV1::Static
                || !state.active
                || state.pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
            {
                return Err(ReferencePhysicsError::UnsupportedProfile);
            }
            for shape in body.shapes.values() {
                validate_reference_shape(shape)?;
                let PhysicsGeometryV1::Box {
                    half_extents_micrometres,
                } = shape.geometry
                else {
                    return Err(ReferencePhysicsError::UnsupportedProfile);
                };
                if shape.local_pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30 {
                    return Err(ReferencePhysicsError::UnsupportedProfile);
                }
                let centre = checked_add_vec3(
                    state.pose.translation_micrometres,
                    shape.local_pose.translation_micrometres,
                )?;
                static_boxes.push(StaticBox {
                    shape_id: shape.shape_id,
                    minimum: checked_sub_vec3(centre, half_extents_micrometres)?,
                    maximum: checked_add_vec3(centre, half_extents_micrometres)?,
                    contact_reporting: shape.contact_reporting,
                    collision_layer: shape.collision_layer,
                    collision_mask: shape.collision_mask,
                });
            }
        }
        static_boxes.sort_by_key(|shape| shape.shape_id);
        if checkpoint.catalog.materials.values().any(|material| {
            material.static_friction_q16 != 0
                || material.dynamic_friction_q16 != 0
                || material.restitution_q16 != 0
        }) {
            return Err(ReferencePhysicsError::UnsupportedProfile);
        }

        let physics_hz = i64::from(tick_rate_profile.physics_hz());
        let gravity = checkpoint
            .catalog
            .solver_profile
            .gravity_micrometres_per_second_squared[1];
        let physics_hz_squared = physics_hz
            .checked_mul(physics_hz)
            .ok_or(ReferencePhysicsError::NumericOverflow)?;
        if gravity.checked_rem(physics_hz_squared) != Some(0)
            || CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND.checked_rem(physics_hz) != Some(0)
        {
            return Err(ReferencePhysicsError::NonIntegralProfile);
        }
        let world = Self {
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
            capsule_body_id,
            capsule_shape_id,
            capsule_radius,
            capsule_half_segment,
            capsule_collision_layer,
            capsule_collision_mask,
            static_boxes,
            locomotion_per_substep: CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND / physics_hz,
            gravity_velocity_delta: gravity / physics_hz,
        };
        world.validate_activation_snapshot()?;
        Ok(world)
    }

    #[must_use]
    pub const fn checkpoint(&self) -> &PhysicsWorldCheckpointV1 {
        &self.checkpoint
    }

    #[must_use]
    pub const fn snapshot(&self) -> &PhysicsCanonicalSnapshotV2 {
        &self.checkpoint.snapshot
    }

    #[must_use]
    pub const fn tick_rate_profile(&self) -> &TickRateProfileV1 {
        &self.tick_rate_profile
    }

    #[must_use]
    pub const fn numeric_profile(&self) -> &AuthoritativeNumericProfileV1 {
        &self.numeric_profile
    }

    #[must_use]
    pub const fn quantization_profile(&self) -> &PhysicsQuantizationProfileV1 {
        &self.quantization_profile
    }

    pub fn snapshot_hash(&self) -> Result<ContentHash, next_contracts::CanonicalError> {
        self.checkpoint.snapshot.snapshot_hash()
    }

    pub fn checkpoint_hash(&self) -> Result<ContentHash, next_contracts::CanonicalError> {
        self.checkpoint.checkpoint_hash()
    }

    pub fn set_checkpoint_revision(&mut self, revision: u64) {
        self.checkpoint.snapshot.checkpoint_revision = revision;
    }

    pub fn step(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, ReferencePhysicsError> {
        input.validate()?;
        if input.world_id != self.checkpoint.snapshot.world_id
            || input.expected_world_revision != self.checkpoint.snapshot.world_revision
            || input.expected_snapshot_hash != self.checkpoint.snapshot.snapshot_hash()?
            || input.expected_catalog_hash != self.checkpoint.catalog.catalog_hash()?
            || self.checkpoint.snapshot.physics_tick.checked_add(1)
                != Some(input.first_physics_tick)
            || input.physics_substeps != self.tick_rate_profile.physics_substeps_per_gameplay_tick
        {
            return Err(ReferencePhysicsError::StepInputMismatch);
        }
        let mut seen = BTreeSet::new();
        for intent in &input.accepted_intents {
            if Some(intent.body_id) != self.capsule_body_id
                || !seen.insert(intent.body_id)
                || intent.target_gameplay_tick != input.gameplay_tick
            {
                return Err(ReferencePhysicsError::StepInputMismatch);
            }
        }
        let Some(capsule_body_id) = self.capsule_body_id else {
            if !input.accepted_intents.is_empty() {
                return Err(ReferencePhysicsError::StepInputMismatch);
            }
            return self.step_empty(input);
        };

        let before_snapshot_hash = self.checkpoint.snapshot.snapshot_hash()?;
        let before_states = input
            .accepted_intents
            .iter()
            .map(|intent| {
                self.checkpoint
                    .snapshot
                    .sorted_body_states
                    .get(&intent.body_id)
                    .cloned()
                    .ok_or(ReferencePhysicsError::BodyMissing)
                    .map(|state| (intent.causal_command_id, state))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        let mut staged = self.checkpoint.snapshot.clone();
        let mut all_events = Vec::new();
        for substep in 0..input.physics_substeps {
            let physics_tick = staged
                .physics_tick
                .checked_add(1)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let body_before = staged
                .sorted_body_states
                .get(&capsule_body_id)
                .cloned()
                .ok_or(ReferencePhysicsError::BodyMissing)?;
            let direction = input
                .accepted_intents
                .first()
                .map_or([0, 0], |intent| intent.direction_q15);
            let mut body_after = body_before.clone();
            body_after.linear_velocity_micrometres_per_second[1] = body_after
                .linear_velocity_micrometres_per_second[1]
                .checked_add(self.gravity_velocity_delta)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let physics_hz = i64::from(self.tick_rate_profile.physics_hz());
            let vertical_delta = body_after.linear_velocity_micrometres_per_second[1]
                .checked_div(physics_hz)
                .ok_or(ReferencePhysicsError::NonIntegralProfile)?;
            let vertical =
                self.sweep_axis(body_after.pose.translation_micrometres, 1, vertical_delta)?;
            body_after.pose.translation_micrometres[1] = body_after.pose.translation_micrometres[1]
                .checked_add(vertical.0)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            if vertical.0 != vertical_delta {
                body_after.linear_velocity_micrometres_per_second[1] = 0;
            }

            let x_delta = self
                .locomotion_per_substep
                .checked_mul(i64::from(direction[0].signum()))
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let x = self.sweep_axis(body_after.pose.translation_micrometres, 0, x_delta)?;
            body_after.pose.translation_micrometres[0] = body_after.pose.translation_micrometres[0]
                .checked_add(x.0)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;

            let z_delta = self
                .locomotion_per_substep
                .checked_mul(i64::from(direction[1].signum()))
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let z = self.sweep_axis(body_after.pose.translation_micrometres, 2, z_delta)?;
            body_after.pose.translation_micrometres[2] = body_after.pose.translation_micrometres[2]
                .checked_add(z.0)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;

            if body_after.pose != body_before.pose
                || body_after.linear_velocity_micrometres_per_second
                    != body_before.linear_velocity_micrometres_per_second
            {
                body_after.body_revision = body_after
                    .body_revision
                    .checked_add(1)
                    .ok_or(ReferencePhysicsError::NumericOverflow)?;
            }
            staged
                .sorted_body_states
                .insert(capsule_body_id, body_after.clone());

            let forced_hits = [vertical.1, x.1, z.1]
                .into_iter()
                .flatten()
                .collect::<BTreeSet<_>>();
            let candidates =
                self.contact_candidates(body_after.pose.translation_micrometres, &forced_hits)?;
            if candidates.len()
                > usize::try_from(
                    self.checkpoint
                        .catalog
                        .limits_profile
                        .maximum_contacts_per_substep,
                )
                .map_err(|_| ReferencePhysicsError::ContactCapacityExceeded)?
            {
                return Err(ReferencePhysicsError::ContactCapacityExceeded);
            }
            let previous = staged.sorted_contact_continuity_states.clone();
            let mut current = BTreeMap::new();
            for candidate in candidates {
                let (low, high, feature_low, feature_high, normal) =
                    self.canonicalize_contact(candidate);
                let contact_id = derive_physics_contact_id(low, high, feature_low, feature_high);
                current.insert(
                    contact_id,
                    PhysicsContactContinuityStateV1 {
                        contact_id,
                        participant_low: low,
                        participant_high: high,
                        feature_low,
                        feature_high,
                        point_micrometres: candidate.point,
                        normal_low_to_high_q1_30: normal,
                        last_seen_physics_tick: physics_tick,
                    },
                );
            }
            staged.sorted_contact_continuity_states = current.clone();
            staged.sorted_solver_continuation_states =
                current.keys().copied().map(|id| (id, [0; 3])).collect();
            staged.physics_tick = physics_tick;
            staged.world_revision = staged
                .world_revision
                .checked_add(1)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let substep_snapshot_hash = staged.snapshot_hash()?;
            for state in current.values() {
                let phase = if previous.contains_key(&state.contact_id) {
                    ContactPhaseV1::Persist
                } else {
                    ContactPhaseV1::Begin
                };
                if should_report_contact(self.reporting_for_contact(state), phase) {
                    all_events.push(contact_event(
                        input.gameplay_tick,
                        physics_tick,
                        substep,
                        phase,
                        state,
                        substep_snapshot_hash,
                    ));
                }
            }
            for (id, state) in previous {
                if !current.contains_key(&id)
                    && should_report_contact(
                        self.reporting_for_contact(&state),
                        ContactPhaseV1::End,
                    )
                {
                    all_events.push(contact_event(
                        input.gameplay_tick,
                        physics_tick,
                        substep,
                        ContactPhaseV1::End,
                        &state,
                        substep_snapshot_hash,
                    ));
                }
            }
        }
        all_events.sort();
        let after_snapshot_hash = staged.snapshot_hash()?;
        let contact_batch = ClosedPhysicsContactBatchV1::new(
            input.gameplay_tick,
            input.first_physics_tick,
            input.physics_substeps,
            all_events,
            after_snapshot_hash,
        )?;
        contact_batch.validate_against_catalog(&self.checkpoint.catalog)?;
        let mut related_ids = contact_batch
            .events
            .iter()
            .filter(|event| {
                event.participant_low.body_id == capsule_body_id
                    || event.participant_high.body_id == capsule_body_id
            })
            .map(|event| event.contact_id)
            .collect::<Vec<_>>();
        related_ids.sort();
        related_ids.dedup();
        let applied_locomotion = input
            .accepted_intents
            .iter()
            .map(|intent| {
                let before = before_states
                    .get(&intent.causal_command_id)
                    .ok_or(ReferencePhysicsError::BodyMissing)?;
                let after = staged
                    .sorted_body_states
                    .get(&intent.body_id)
                    .ok_or(ReferencePhysicsError::BodyMissing)?;
                Ok(AppliedLocomotionResultV1 {
                    causal_command_id: intent.causal_command_id,
                    body_id: intent.body_id,
                    requested_direction_q15: intent.direction_q15,
                    before_state_hash: before.state_hash()?,
                    after_state_hash: after.state_hash()?,
                    applied_displacement_micrometres: checked_sub_vec3(
                        after.pose.translation_micrometres,
                        before.pose.translation_micrometres,
                    )?,
                    related_contact_ids: related_ids.clone(),
                })
            })
            .collect::<Result<Vec<_>, ReferencePhysicsError>>()?;
        self.checkpoint.snapshot = staged;
        Ok(PhysicsStepResultV1 {
            step_input_hash: input.input_hash()?,
            before_snapshot_hash,
            after_snapshot_hash,
            applied_locomotion,
            contact_batch,
        })
    }

    fn step_empty(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, ReferencePhysicsError> {
        let before_snapshot_hash = self.checkpoint.snapshot.snapshot_hash()?;
        let mut staged = self.checkpoint.snapshot.clone();
        for _ in 0..input.physics_substeps {
            staged.physics_tick = staged
                .physics_tick
                .checked_add(1)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            staged.world_revision = staged
                .world_revision
                .checked_add(1)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
        }
        let after_snapshot_hash = staged.snapshot_hash()?;
        let contact_batch = ClosedPhysicsContactBatchV1::new(
            input.gameplay_tick,
            input.first_physics_tick,
            input.physics_substeps,
            Vec::new(),
            after_snapshot_hash,
        )?;
        self.checkpoint.snapshot = staged;
        Ok(PhysicsStepResultV1 {
            step_input_hash: input.input_hash()?,
            before_snapshot_hash,
            after_snapshot_hash,
            applied_locomotion: Vec::new(),
            contact_batch,
        })
    }

    fn sweep_axis(
        &self,
        centre: [i64; 3],
        axis: usize,
        delta: i64,
    ) -> Result<(i64, Option<SweepHit>), ReferencePhysicsError> {
        if delta == 0 {
            return Ok((0, None));
        }
        let mut applied = delta;
        let mut selected = None;
        for shape in &self.static_boxes {
            if !self.collides_with(shape) {
                continue;
            }
            let Some((lower, upper)) = expanded_axis_interval(
                centre,
                axis,
                self.capsule_radius,
                self.capsule_half_segment,
                shape,
            )?
            else {
                continue;
            };
            let start = centre[axis];
            let end = start
                .checked_add(applied)
                .ok_or(ReferencePhysicsError::NumericOverflow)?;
            let hit = if delta > 0 && start <= lower && end > lower {
                Some((
                    lower - start,
                    negative_axis_normal(axis),
                    negative_face(axis),
                ))
            } else if delta < 0 && start >= upper && end < upper {
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
                        && selected
                            .is_none_or(|existing: SweepHit| shape.shape_id < existing.shape_id)))
            {
                applied = candidate;
                selected = Some(SweepHit {
                    shape_id: shape.shape_id,
                    box_feature: feature,
                    normal_box_to_capsule: normal,
                });
            }
        }
        Ok((applied, selected))
    }

    fn contact_candidates(
        &self,
        centre: [i64; 3],
        forced_hits: &BTreeSet<SweepHit>,
    ) -> Result<Vec<ContactCandidate>, ReferencePhysicsError> {
        let radius_squared = square(self.capsule_radius)?;
        let mut candidates = Vec::new();
        for shape in &self.static_boxes {
            if !self.collides_with(shape) {
                continue;
            }
            let distance_squared =
                capsule_box_distance_squared(centre, self.capsule_half_segment, shape)?;
            let forced = forced_hits
                .iter()
                .find(|hit| hit.shape_id == shape.shape_id);
            if distance_squared > radius_squared && forced.is_none() {
                continue;
            }
            let (normal, feature) = if let Some(hit) = forced {
                (hit.normal_box_to_capsule, hit.box_feature)
            } else {
                contact_normal_and_feature(centre, self.capsule_half_segment, shape)?
            };
            let point = [
                centre[0].clamp(shape.minimum[0], shape.maximum[0]),
                centre[1].clamp(shape.minimum[1], shape.maximum[1]),
                centre[2].clamp(shape.minimum[2], shape.maximum[2]),
            ];
            candidates.push(ContactCandidate {
                shape_id: shape.shape_id,
                box_feature: feature,
                point,
                normal_box_to_capsule: normal,
            });
        }
        candidates.sort();
        candidates.dedup();
        Ok(candidates)
    }

    fn canonicalize_contact(
        &self,
        candidate: ContactCandidate,
    ) -> (PhysicsShapeIdV1, PhysicsShapeIdV1, u8, u8, [i32; 3]) {
        let capsule_shape_id = self
            .capsule_shape_id
            .expect("contact candidates only exist in a capsule world");
        if candidate.shape_id < capsule_shape_id {
            (
                candidate.shape_id,
                capsule_shape_id,
                candidate.box_feature,
                1,
                candidate.normal_box_to_capsule,
            )
        } else {
            (
                capsule_shape_id,
                candidate.shape_id,
                1,
                candidate.box_feature,
                [
                    -candidate.normal_box_to_capsule[0],
                    -candidate.normal_box_to_capsule[1],
                    -candidate.normal_box_to_capsule[2],
                ],
            )
        }
    }

    fn reporting_for_contact(
        &self,
        state: &PhysicsContactContinuityStateV1,
    ) -> PhysicsContactReportingV1 {
        self.static_boxes
            .iter()
            .find(|shape| {
                shape.shape_id == state.participant_low || shape.shape_id == state.participant_high
            })
            .map_or(PhysicsContactReportingV1::Disabled, |shape| {
                shape.contact_reporting
            })
    }

    fn collides_with(&self, shape: &StaticBox) -> bool {
        let capsule_to_box =
            self.capsule_collision_mask & (1_u64 << u32::from(shape.collision_layer)) != 0;
        let box_to_capsule =
            shape.collision_mask & (1_u64 << u32::from(self.capsule_collision_layer)) != 0;
        capsule_to_box && box_to_capsule
    }

    fn validate_activation_snapshot(&self) -> Result<(), ReferencePhysicsError> {
        let Some(capsule_body_id) = self.capsule_body_id else {
            if self
                .checkpoint
                .snapshot
                .sorted_contact_continuity_states
                .is_empty()
            {
                return Ok(());
            }
            return Err(ReferencePhysicsError::SnapshotMismatch);
        };
        let body = self
            .checkpoint
            .snapshot
            .sorted_body_states
            .get(&capsule_body_id)
            .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
        if !body.active
            || body.pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
            || body.angular_velocity_q16 != [0; 3]
        {
            return Err(ReferencePhysicsError::SnapshotMismatch);
        }
        let radius_squared = square(self.capsule_radius)?;
        for shape in &self.static_boxes {
            if self.collides_with(shape)
                && capsule_box_distance_squared(
                    body.pose.translation_micrometres,
                    self.capsule_half_segment,
                    shape,
                )? < radius_squared
            {
                return Err(ReferencePhysicsError::SnapshotPenetrating);
            }
        }

        let snapshot = &self.checkpoint.snapshot;
        if snapshot.physics_tick == 0
            && snapshot.world_revision == 0
            && snapshot.sorted_contact_continuity_states.is_empty()
        {
            return Ok(());
        }
        let candidates =
            self.contact_candidates(body.pose.translation_micrometres, &BTreeSet::new())?;
        let mut recomputed = BTreeMap::new();
        for candidate in candidates {
            let (low, high, feature_low, feature_high, normal) =
                self.canonicalize_contact(candidate);
            let contact_id = derive_physics_contact_id(low, high, feature_low, feature_high);
            recomputed.insert(
                contact_id,
                PhysicsContactContinuityStateV1 {
                    contact_id,
                    participant_low: low,
                    participant_high: high,
                    feature_low,
                    feature_high,
                    point_micrometres: candidate.point,
                    normal_low_to_high_q1_30: normal,
                    last_seen_physics_tick: snapshot.physics_tick,
                },
            );
        }
        if recomputed != snapshot.sorted_contact_continuity_states {
            return Err(ReferencePhysicsError::SnapshotMismatch);
        }
        Ok(())
    }
}

fn validate_reference_shape(shape: &PhysicsShapeDescriptorV1) -> Result<(), ReferencePhysicsError> {
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
    shape: &StaticBox,
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

fn capsule_box_distance_squared(
    centre: [i64; 3],
    half_segment: i64,
    shape: &StaticBox,
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

const fn should_report_contact(
    reporting: PhysicsContactReportingV1,
    phase: ContactPhaseV1,
) -> bool {
    match (reporting, phase) {
        (PhysicsContactReportingV1::Disabled, _) => false,
        (PhysicsContactReportingV1::BeginEnd, ContactPhaseV1::Persist) => false,
        (
            PhysicsContactReportingV1::BeginEnd | PhysicsContactReportingV1::BeginPersistEnd,
            ContactPhaseV1::Begin | ContactPhaseV1::End,
        )
        | (PhysicsContactReportingV1::BeginPersistEnd, ContactPhaseV1::Persist) => true,
    }
}

fn square(value: i64) -> Result<i128, ReferencePhysicsError> {
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

fn contact_normal_and_feature(
    centre: [i64; 3],
    half_segment: i64,
    shape: &StaticBox,
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

fn contact_event(
    gameplay_tick: u64,
    physics_tick: u64,
    substep: u32,
    phase: ContactPhaseV1,
    state: &PhysicsContactContinuityStateV1,
    source_snapshot_hash: ContentHash,
) -> ContactEventV1 {
    ContactEventV1 {
        gameplay_tick,
        physics_tick,
        substep,
        participant_low: state.participant_low,
        participant_high: state.participant_high,
        feature_low: state.feature_low,
        feature_high: state.feature_high,
        point_micrometres: state.point_micrometres,
        normal_low_to_high_q1_30: state.normal_low_to_high_q1_30,
        phase,
        contact_id: state.contact_id,
        source_snapshot_hash,
    }
}

fn checked_add_vec3(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], ReferencePhysicsError> {
    Ok([
        left[0]
            .checked_add(right[0])
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
        left[1]
            .checked_add(right[1])
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
        left[2]
            .checked_add(right[2])
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
    ])
}

fn checked_sub_vec3(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], ReferencePhysicsError> {
    Ok([
        left[0]
            .checked_sub(right[0])
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
        left[1]
            .checked_sub(right[1])
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
        left[2]
            .checked_sub(right[2])
            .ok_or(ReferencePhysicsError::NumericOverflow)?,
    ])
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReferencePhysicsError {
    Contract(PhysicsContractError),
    Canonical(next_contracts::CanonicalError),
    UnsupportedProfile,
    NonIntegralProfile,
    SnapshotMismatch,
    StepInputMismatch,
    BodyMissing,
    NumericOverflow,
    ContactCapacityExceeded,
    SnapshotPenetrating,
}

impl ReferencePhysicsError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::Canonical(_) => "PHYSICS_CANONICALIZATION_FAILED",
            Self::UnsupportedProfile => "PHYS_REFERENCE_PROFILE_UNSUPPORTED",
            Self::NonIntegralProfile => "PHYSICS_PROFILE_MISMATCH",
            Self::SnapshotMismatch => "PHYS_SNAPSHOT_INCOMPATIBLE",
            Self::StepInputMismatch => "PHYS_STEP_INPUT_MISMATCH",
            Self::BodyMissing => "PHYSICAL_TARGET_UNBOUND",
            Self::NumericOverflow => "PHYSICS_NUMERIC_OVERFLOW",
            Self::ContactCapacityExceeded => "PHYS_CONTACT_CAPACITY_EXCEEDED",
            Self::SnapshotPenetrating => "PHYS_SNAPSHOT_PENETRATING",
        }
    }
}

impl Display for ReferencePhysicsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for ReferencePhysicsError {}

impl From<PhysicsContractError> for ReferencePhysicsError {
    fn from(error: PhysicsContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<next_contracts::CanonicalError> for ReferencePhysicsError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::{
        AcceptedLocomotionIntentV2, CommandId, PHYSICS_STEP_INPUT_SCHEMA_VERSION, PersistentId,
        PhysicsBodyDescriptorV1, PhysicsBodyStateV2, PhysicsCoordinateProfileV1,
        PhysicsLimitsProfileV1, PhysicsMaterialDescriptorV1, PhysicsShapeIdV1,
        PhysicsSolverSemanticsProfileV1, PhysicsWorldCatalogProfilesV1, PhysicsWorldCatalogV1,
        PhysicsWorldId, SchemaId,
    };

    fn world(
        gameplay_hz: u32,
        physics_hz: u32,
        initial_centre: [i64; 3],
        wall_collision_mask: u64,
    ) -> ReferencePhysicsWorld {
        world_with_wall(
            gameplay_hz,
            physics_hz,
            initial_centre,
            wall_collision_mask,
            700_000,
            100_000,
        )
    }

    fn world_with_wall(
        gameplay_hz: u32,
        physics_hz: u32,
        initial_centre: [i64; 3],
        wall_collision_mask: u64,
        wall_centre_z: i64,
        wall_half_extent_z: i64,
    ) -> ReferencePhysicsWorld {
        world_with_wall_reporting(
            gameplay_hz,
            physics_hz,
            initial_centre,
            wall_collision_mask,
            wall_centre_z,
            wall_half_extent_z,
            PhysicsContactReportingV1::BeginPersistEnd,
        )
    }

    fn world_with_wall_reporting(
        gameplay_hz: u32,
        physics_hz: u32,
        initial_centre: [i64; 3],
        wall_collision_mask: u64,
        wall_centre_z: i64,
        wall_half_extent_z: i64,
        contact_reporting: PhysicsContactReportingV1,
    ) -> ReferencePhysicsWorld {
        let tick_rate = TickRateProfileV1 {
            schema_version: 1,
            gameplay_hz,
            physics_substeps_per_gameplay_tick: physics_hz / gameplay_hz,
            motor_period_physics_substeps: 1,
            first_gameplay_tick: 0,
        };
        let quantization =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
        let material_id = SchemaId::new("nextengine.physics.material.reference-zero")
            .expect("material identifier");
        let material = PhysicsMaterialDescriptorV1 {
            material_id: material_id.clone(),
            descriptor_revision: 1,
            static_friction_q16: 0,
            dynamic_friction_q16: 0,
            restitution_q16: 0,
            canonical_material_tags: Vec::new(),
        };
        let capsule_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([1; 16]),
            body_slot: 0,
        };
        let capsule_shape_id = PhysicsShapeIdV1 {
            body_id: capsule_body_id,
            shape_slot: 0,
        };
        let capsule = body(
            capsule_body_id,
            PhysicsMotionKindV1::Kinematic,
            initial_centre,
            shape(
                capsule_shape_id,
                PhysicsGeometryV1::Capsule {
                    radius_micrometres: 300_000,
                    half_segment_micrometres: 600_000,
                },
                &material_id,
                1,
                contact_reporting,
            ),
        );
        let floor_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([2; 16]),
            body_slot: 0,
        };
        let floor_shape_id = PhysicsShapeIdV1 {
            body_id: floor_body_id,
            shape_slot: 0,
        };
        let floor = body(
            floor_body_id,
            PhysicsMotionKindV1::Static,
            [0, -100_000, 0],
            shape(
                floor_shape_id,
                PhysicsGeometryV1::Box {
                    half_extents_micrometres: [10_000_000, 100_000, 10_000_000],
                },
                &material_id,
                1,
                contact_reporting,
            ),
        );
        let wall_body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([3; 16]),
            body_slot: 0,
        };
        let wall_shape_id = PhysicsShapeIdV1 {
            body_id: wall_body_id,
            shape_slot: 0,
        };
        let wall = body(
            wall_body_id,
            PhysicsMotionKindV1::Static,
            [0, 900_000, wall_centre_z],
            shape(
                wall_shape_id,
                PhysicsGeometryV1::Box {
                    half_extents_micrometres: [10_000_000, 10_000_000, wall_half_extent_z],
                },
                &material_id,
                wall_collision_mask,
                contact_reporting,
            ),
        );
        let catalog = PhysicsWorldCatalogV1::new(
            PhysicsWorldId::from_bytes([9; 16]),
            PhysicsWorldCatalogProfilesV1 {
                coordinate: PhysicsCoordinateProfileV1::reference_v1().expect("coordinates"),
                limits: PhysicsLimitsProfileV1::reference_v1().expect("limits"),
                solver: PhysicsSolverSemanticsProfileV1::grounded_capsule_v1().expect("solver"),
                tick_rate_hash: tick_rate.profile_hash().expect("tick hash"),
                authoritative_numeric_hash: numeric.profile_hash().expect("numeric hash"),
                quantization_hash: quantization.profile_hash().expect("quantization hash"),
            },
            BTreeMap::from([(material_id, material)]),
            BTreeMap::from([
                (capsule_body_id, capsule),
                (floor_body_id, floor),
                (wall_body_id, wall),
            ]),
            BTreeMap::from([(capsule_body_id.subject_id, capsule_body_id)]),
        )
        .expect("catalog");
        let snapshot =
            PhysicsCanonicalSnapshotV2::genesis(&catalog, &tick_rate, &numeric, &quantization)
                .expect("snapshot");
        let checkpoint = PhysicsWorldCheckpointV1::new(catalog, snapshot).expect("checkpoint");
        ReferencePhysicsWorld::new(checkpoint, tick_rate, numeric, quantization).expect("world")
    }

    fn body(
        body_id: PhysicsBodyIdV1,
        motion_kind: PhysicsMotionKindV1,
        translation_micrometres: [i64; 3],
        shape: PhysicsShapeDescriptorV1,
    ) -> PhysicsBodyDescriptorV1 {
        PhysicsBodyDescriptorV1 {
            body_id,
            descriptor_revision: 1,
            motion_kind,
            initial_pose: PhysicsPoseV1 {
                translation_micrometres,
                ..PhysicsPoseV1::default()
            },
            initial_linear_velocity_micrometres_per_second: [0; 3],
            initial_angular_velocity_q16: [0; 3],
            active: true,
            shapes: BTreeMap::from([(shape.shape_id, shape)]),
        }
    }

    fn shape(
        shape_id: PhysicsShapeIdV1,
        geometry: PhysicsGeometryV1,
        material_id: &SchemaId,
        collision_mask: u64,
        contact_reporting: PhysicsContactReportingV1,
    ) -> PhysicsShapeDescriptorV1 {
        PhysicsShapeDescriptorV1 {
            shape_id,
            descriptor_revision: 1,
            local_pose: PhysicsPoseV1::default(),
            geometry,
            material_id: material_id.clone(),
            collision_layer: 0,
            collision_mask,
            participation: PhysicsParticipationV1::Solid,
            contact_reporting,
        }
    }

    fn step(
        world: &mut ReferencePhysicsWorld,
        gameplay_tick: u64,
        direction: Option<[i16; 2]>,
    ) -> PhysicsStepResultV1 {
        let snapshot = world.snapshot();
        let accepted_intents = direction.map_or_else(Vec::new, |direction_q15| {
            let body_id = *world
                .checkpoint()
                .catalog
                .avatar_bindings
                .values()
                .next()
                .expect("capsule binding");
            vec![AcceptedLocomotionIntentV2 {
                causal_command_id: CommandId::from_bytes([gameplay_tick as u8; 16]),
                controlled_target_id: body_id.subject_id,
                body_id,
                target_gameplay_tick: gameplay_tick,
                direction_q15,
            }]
        });
        let input = PhysicsStepInputV2 {
            schema_version: PHYSICS_STEP_INPUT_SCHEMA_VERSION,
            world_id: snapshot.world_id,
            expected_world_revision: snapshot.world_revision,
            expected_snapshot_hash: snapshot.snapshot_hash().expect("snapshot hash"),
            expected_catalog_hash: world
                .checkpoint()
                .catalog
                .catalog_hash()
                .expect("catalog hash"),
            gameplay_tick,
            first_physics_tick: snapshot
                .physics_tick
                .checked_add(1)
                .expect("test physics tick remains bounded"),
            physics_substeps: world.tick_rate_profile().physics_substeps_per_gameplay_tick,
            accepted_intents,
        };
        world.step(&input).expect("physics step")
    }

    fn capsule_state(world: &ReferencePhysicsWorld) -> &PhysicsBodyStateV2 {
        let body_id = world
            .checkpoint()
            .catalog
            .avatar_bindings
            .values()
            .next()
            .expect("capsule binding");
        &world.snapshot().sorted_body_states[body_id]
    }

    fn reconstruct(
        checkpoint: PhysicsWorldCheckpointV1,
        source: &ReferencePhysicsWorld,
    ) -> Result<ReferencePhysicsWorld, ReferencePhysicsError> {
        ReferencePhysicsWorld::new(
            checkpoint,
            source.tick_rate_profile().to_owned(),
            source.numeric_profile().to_owned(),
            source.quantization_profile().to_owned(),
        )
    }

    #[test]
    fn activation_accepts_exact_touching_and_rejects_penetration() {
        let source = world(30, 60, [0, 900_000, 0], 1);
        let capsule_body_id = *source
            .checkpoint()
            .catalog
            .avatar_bindings
            .values()
            .next()
            .expect("capsule binding");

        let mut touching = source.checkpoint().clone();
        touching
            .snapshot
            .sorted_body_states
            .get_mut(&capsule_body_id)
            .expect("capsule state")
            .pose
            .translation_micrometres[2] = 300_000;
        reconstruct(touching, &source).expect("exact touching activates");

        let mut penetrating = source.checkpoint().clone();
        penetrating
            .snapshot
            .sorted_body_states
            .get_mut(&capsule_body_id)
            .expect("capsule state")
            .pose
            .translation_micrometres[2] = 300_001;
        assert_eq!(
            reconstruct(penetrating, &source),
            Err(ReferencePhysicsError::SnapshotPenetrating)
        );
    }

    #[test]
    fn checkpoint_closure_rejects_missing_body_static_drift_and_solver_mismatch() {
        let mut world = world(30, 60, [0, 900_000, 0], 1);
        let _ = step(&mut world, 0, None);
        let checkpoint = world.checkpoint().clone();
        let capsule_body_id = *checkpoint
            .catalog
            .avatar_bindings
            .values()
            .next()
            .expect("capsule binding");
        let static_body_id = checkpoint
            .catalog
            .bodies
            .keys()
            .copied()
            .find(|body_id| *body_id != capsule_body_id)
            .expect("static body");

        let mut missing_body = checkpoint.clone();
        missing_body
            .snapshot
            .sorted_body_states
            .remove(&static_body_id);
        assert_eq!(
            missing_body.validate(),
            Err(PhysicsContractError::ReferenceInvalid)
        );

        let mut static_drift = checkpoint.clone();
        static_drift
            .snapshot
            .sorted_body_states
            .get_mut(&static_body_id)
            .expect("static state")
            .linear_velocity_micrometres_per_second[0] = 1;
        assert_eq!(
            static_drift.validate(),
            Err(PhysicsContractError::ReferenceInvalid)
        );

        let mut missing_solver_state = checkpoint;
        missing_solver_state
            .snapshot
            .sorted_solver_continuation_states
            .clear();
        assert_eq!(
            missing_solver_state.validate(),
            Err(PhysicsContractError::ReferenceInvalid)
        );
    }

    #[test]
    fn checkpoint_and_contact_batch_reject_stale_ticks_and_invalid_features() {
        let mut world = world(30, 60, [0, 900_000, 0], 1);
        let step_result = step(&mut world, 0, None);
        let checkpoint = world.checkpoint().clone();

        let mut stale_tick = checkpoint.clone();
        stale_tick
            .snapshot
            .sorted_contact_continuity_states
            .values_mut()
            .next()
            .expect("floor contact")
            .last_seen_physics_tick -= 1;
        assert_eq!(
            stale_tick.validate(),
            Err(PhysicsContractError::ContactIdentityMismatch)
        );

        let mut invalid_feature = checkpoint.clone();
        let (_, mut contact) = invalid_feature
            .snapshot
            .sorted_contact_continuity_states
            .pop_first()
            .expect("floor contact");
        contact.feature_low = 0;
        contact.contact_id = derive_physics_contact_id(
            contact.participant_low,
            contact.participant_high,
            contact.feature_low,
            contact.feature_high,
        );
        invalid_feature
            .snapshot
            .sorted_contact_continuity_states
            .insert(contact.contact_id, contact.clone());
        invalid_feature
            .snapshot
            .sorted_solver_continuation_states
            .clear();
        invalid_feature
            .snapshot
            .sorted_solver_continuation_states
            .insert(contact.contact_id, [0; 3]);
        assert_eq!(
            invalid_feature.validate(),
            Err(PhysicsContractError::ReferenceInvalid)
        );

        let event = step_result
            .contact_batch
            .events
            .first()
            .expect("floor begin")
            .clone();
        let mut wrong_tick = event.clone();
        wrong_tick.physics_tick += 1;
        assert_eq!(
            ClosedPhysicsContactBatchV1::new(
                step_result.contact_batch.gameplay_tick,
                step_result.contact_batch.first_physics_tick,
                step_result.contact_batch.substep_count,
                vec![wrong_tick],
                step_result.contact_batch.source_snapshot_hash,
            ),
            Err(PhysicsContractError::NonCanonicalOrder)
        );

        let mut wrong_feature = event;
        wrong_feature.feature_low = 0;
        wrong_feature.contact_id = derive_physics_contact_id(
            wrong_feature.participant_low,
            wrong_feature.participant_high,
            wrong_feature.feature_low,
            wrong_feature.feature_high,
        );
        let batch = ClosedPhysicsContactBatchV1::new(
            step_result.contact_batch.gameplay_tick,
            step_result.contact_batch.first_physics_tick,
            step_result.contact_batch.substep_count,
            vec![wrong_feature],
            step_result.contact_batch.source_snapshot_hash,
        )
        .expect("identity-valid batch");
        assert_eq!(
            batch.validate_against_catalog(&checkpoint.catalog),
            Err(PhysicsContractError::ReferenceInvalid)
        );
    }

    #[test]
    fn contact_reporting_policy_does_not_change_continuity() {
        let mut disabled = world_with_wall_reporting(
            30,
            60,
            [0, 900_000, 0],
            1,
            700_000,
            100_000,
            PhysicsContactReportingV1::Disabled,
        );
        for tick in 0..4 {
            let result = step(&mut disabled, tick, Some([0, 32_767]));
            assert!(result.contact_batch.events.is_empty());
        }
        assert!(
            !disabled
                .snapshot()
                .sorted_contact_continuity_states
                .is_empty()
        );

        let mut begin_end = world_with_wall_reporting(
            30,
            60,
            [0, 900_000, 0],
            1,
            700_000,
            100_000,
            PhysicsContactReportingV1::BeginEnd,
        );
        let mut phases = Vec::new();
        for tick in 0..4 {
            phases.extend(
                step(&mut begin_end, tick, Some([0, 32_767]))
                    .contact_batch
                    .events
                    .into_iter()
                    .map(|event| event.phase),
            );
        }
        phases.extend(
            step(&mut begin_end, 4, Some([0, -32_767]))
                .contact_batch
                .events
                .into_iter()
                .map(|event| event.phase),
        );
        assert!(phases.contains(&ContactPhaseV1::Begin));
        assert!(phases.contains(&ContactPhaseV1::End));
        assert!(!phases.contains(&ContactPhaseV1::Persist));
    }

    #[test]
    fn landing_begins_then_persists_with_zero_vertical_velocity() {
        let mut world = world(30, 60, [0, 1_200_000, 0], 1);
        let mut phases = Vec::new();
        for tick in 0..12 {
            phases.extend(
                step(&mut world, tick, None)
                    .contact_batch
                    .events
                    .into_iter()
                    .map(|event| event.phase),
            );
        }
        assert_eq!(
            capsule_state(&world).pose.translation_micrometres[1],
            900_000
        );
        assert_eq!(
            capsule_state(&world).linear_velocity_micrometres_per_second[1],
            0
        );
        assert_eq!(
            phases
                .iter()
                .filter(|phase| **phase == ContactPhaseV1::Begin)
                .count(),
            1
        );
        assert!(phases.contains(&ContactPhaseV1::Persist));
    }

    #[test]
    fn wall_stops_capsule_and_departure_emits_one_end() {
        let mut world = world(30, 60, [0, 900_000, 0], 1);
        let mut wall_begin = 0;
        for tick in 0..5 {
            wall_begin += step(&mut world, tick, Some([0, 32_767]))
                .contact_batch
                .events
                .iter()
                .filter(|event| {
                    event.phase == ContactPhaseV1::Begin && event.normal_low_to_high_q1_30[2] != 0
                })
                .count();
        }
        assert_eq!(
            capsule_state(&world).pose.translation_micrometres[2],
            300_000
        );
        assert_eq!(wall_begin, 1);
        let departure = step(&mut world, 5, Some([0, -32_767]));
        assert_eq!(
            departure
                .contact_batch
                .events
                .iter()
                .filter(|event| {
                    event.phase == ContactPhaseV1::End && event.normal_low_to_high_q1_30[2] != 0
                })
                .count(),
            1
        );
    }

    #[test]
    fn axial_sweep_cannot_tunnel_through_one_micrometre_wall() {
        let mut world = world_with_wall(20, 60, [0, 900_000, 0], 1, 350_001, 1);
        let result = step(&mut world, 0, Some([0, 32_767]));
        assert_eq!(
            capsule_state(&world).pose.translation_micrometres[2],
            50_000
        );
        assert!(result.contact_batch.events.iter().any(|event| {
            event.phase == ContactPhaseV1::Begin && event.normal_low_to_high_q1_30[2] != 0
        }));
    }

    #[test]
    fn restored_contact_continuity_resumes_with_persist() {
        let mut original = world(30, 60, [0, 900_000, 0], 1);
        let first = step(&mut original, 0, None);
        assert!(
            first
                .contact_batch
                .events
                .iter()
                .any(|event| event.phase == ContactPhaseV1::Begin)
        );
        let checkpoint = original.checkpoint().clone();
        let mut restored = ReferencePhysicsWorld::new(
            checkpoint,
            original.tick_rate_profile().to_owned(),
            original.numeric_profile().to_owned(),
            original.quantization_profile().to_owned(),
        )
        .expect("restored world");
        let continued = step(&mut restored, 1, None);
        assert!(
            continued
                .contact_batch
                .events
                .iter()
                .all(|event| event.phase != ContactPhaseV1::Begin)
        );
        assert!(
            continued
                .contact_batch
                .events
                .iter()
                .any(|event| event.phase == ContactPhaseV1::Persist)
        );
    }

    #[test]
    fn filtered_wall_does_not_collide_or_report_contact() {
        let mut world = world(30, 60, [0, 900_000, 0], 0);
        let mut wall_events = 0;
        for tick in 0..6 {
            wall_events += step(&mut world, tick, Some([0, 32_767]))
                .contact_batch
                .events
                .iter()
                .filter(|event| event.normal_low_to_high_q1_30[2] != 0)
                .count();
        }
        assert_eq!(
            capsule_state(&world).pose.translation_micrometres[2],
            600_000
        );
        assert_eq!(wall_events, 0);
    }

    #[test]
    fn primitive_box_edge_and_vertex_features_have_canonical_ids() {
        let body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([0x44; 16]),
            body_slot: 0,
        };
        let shape = StaticBox {
            shape_id: PhysicsShapeIdV1 {
                body_id,
                shape_slot: 0,
            },
            minimum: [0, 0, 0],
            maximum: [10, 10, 10],
            contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
            collision_layer: 0,
            collision_mask: 1,
        };
        let (edge_normal, edge_feature) =
            contact_normal_and_feature([13, 5, 14], 1, &shape).expect("edge feature");
        assert_eq!(edge_feature, 14);
        assert!(edge_normal[0] > 0 && edge_normal[1] == 0 && edge_normal[2] > 0);

        let (vertex_normal, vertex_feature) =
            contact_normal_and_feature([13, 20, 14], 1, &shape).expect("vertex feature");
        assert_eq!(vertex_feature, 26);
        assert!(vertex_normal.into_iter().all(|component| component > 0));
    }

    #[test]
    fn supported_cadences_move_exactly_three_metres_per_second_without_wall() {
        for gameplay_hz in [20, 30, 60] {
            for physics_hz in [60, 120, 240] {
                let mut world = world(gameplay_hz, physics_hz, [0, 900_000, 0], 0);
                for tick in 0..u64::from(gameplay_hz) {
                    let _ = step(&mut world, tick, Some([32_767, 0]));
                }
                assert_eq!(
                    capsule_state(&world).pose.translation_micrometres[0],
                    3_000_000
                );
            }
        }
    }
}
