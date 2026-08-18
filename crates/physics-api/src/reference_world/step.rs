use std::collections::{BTreeMap, BTreeSet};

use next_contracts::ids::ContentHash;
use next_contracts::physics::{
    AppliedLocomotionResultV1, ClosedPhysicsContactBatchV1, ContactEventV1, ContactPhaseV1,
    PhysicsContactContinuityStateV1, PhysicsContactReportingV1, PhysicsPoseV1, PhysicsShapeIdV1,
    PhysicsStepInputV2, PhysicsStepResultV1, derive_physics_contact_id,
};

use super::error::ReferencePhysicsError;
use super::interaction::{
    box_contact_normal_and_features, boxes_penetrate, boxes_touch_or_overlap,
};
use super::locomotion::AvatarSweepHit;
use super::query::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, capsule_box_distance_squared,
    contact_normal_and_feature, grounded_capsule_collision_filter, square,
};
use super::world::{GroundedCapsuleWorld, checked_add_vec3, checked_sub_vec3};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ContactCandidate {
    avatar_shape_id: PhysicsShapeIdV1,
    avatar_feature: u8,
    shape_id: PhysicsShapeIdV1,
    box_feature: u8,
    point: [i64; 3],
    normal_box_to_capsule: [i32; 3],
}

impl<Q: GroundedCapsuleQuery> GroundedCapsuleWorld<Q> {
    pub fn step(
        &mut self,
        input: &PhysicsStepInputV2,
    ) -> Result<PhysicsStepResultV1, ReferencePhysicsError> {
        input.validate()?;
        if input.world_id != self.checkpoint.snapshot.world_id
            || input.expected_world_revision != self.checkpoint.snapshot.world_revision
            || input.expected_snapshot_hash != self.snapshot_hash()?
            || input.expected_catalog_hash != self.catalog_hash()?
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

        // Memo hit: the checkpoint snapshot is unchanged since validation.
        let before_snapshot_hash = self.snapshot_hash()?;
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
        let mut last_staged_hash = None;
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
            let integrated =
                self.integrate_capsule_substep(&mut staged, &body_before, direction)?;
            let mut body_after = integrated.body_after;

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

            let candidates = self.contact_candidates(
                &staged,
                body_after.pose.translation_micrometres,
                &integrated.forced_hits,
            )?;
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
            last_staged_hash = Some(substep_snapshot_hash);
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
        // The loop's final substep hash already covers the unchanged staged
        // snapshot; only a zero-substep input needs a fresh computation.
        let after_snapshot_hash = match last_staged_hash {
            Some(hash) => hash,
            None => staged.snapshot_hash()?,
        };
        let contact_batch = ClosedPhysicsContactBatchV1::new(
            input.gameplay_tick,
            input.first_physics_tick,
            input.physics_substeps,
            all_events,
            after_snapshot_hash,
        )?;
        // The batch is a fresh constructor result (already validated); only
        // the catalog-relative invariants need checking on this path.
        contact_batch.validate_events_against_catalog(&self.checkpoint.catalog)?;
        debug_assert!(contact_batch.validate().is_ok());
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
        self.reseed_snapshot_hash_memo(after_snapshot_hash);
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
        let before_snapshot_hash = self.snapshot_hash()?;
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
        self.reseed_snapshot_hash_memo(after_snapshot_hash);
        Ok(PhysicsStepResultV1 {
            step_input_hash: input.input_hash()?,
            before_snapshot_hash,
            after_snapshot_hash,
            applied_locomotion: Vec::new(),
            contact_batch,
        })
    }

    fn contact_candidates(
        &self,
        staged: &next_contracts::physics::PhysicsCanonicalSnapshotV2,
        centre: [i64; 3],
        forced_hits: &BTreeSet<AvatarSweepHit>,
    ) -> Result<Vec<ContactCandidate>, ReferencePhysicsError> {
        let radius_squared = square(self.capsule_radius)?;
        let mut candidates = Vec::new();
        let contact_boxes = self.contact_boxes(staged)?;
        let capsule_shape_id = self
            .capsule_shape_id
            .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
        for shape in &contact_boxes {
            if !self.collides_with(shape) {
                continue;
            }
            let distance_squared =
                capsule_box_distance_squared(centre, self.capsule_half_segment, shape)?;
            let forced = forced_hits.iter().find(|hit| {
                hit.avatar_shape_id == capsule_shape_id && hit.other.shape_id == shape.shape_id
            });
            if distance_squared > radius_squared && forced.is_none() {
                continue;
            }
            let (normal, feature) = if let Some(hit) = forced {
                (hit.other.normal_box_to_capsule, hit.other.box_feature)
            } else {
                contact_normal_and_feature(centre, self.capsule_half_segment, shape)?
            };
            let point = [
                centre[0].clamp(shape.minimum[0], shape.maximum[0]),
                centre[1].clamp(shape.minimum[1], shape.maximum[1]),
                centre[2].clamp(shape.minimum[2], shape.maximum[2]),
            ];
            candidates.push(ContactCandidate {
                avatar_shape_id: capsule_shape_id,
                avatar_feature: 1,
                shape_id: shape.shape_id,
                box_feature: feature,
                point,
                normal_box_to_capsule: normal,
            });
        }
        for attached in self.attached_boxes.iter() {
            let moving = attached.at_body_centre(centre)?;
            for shape in &contact_boxes {
                if !grounded_capsule_collision_filter(
                    moving.collision_layer,
                    moving.collision_mask,
                    shape,
                ) {
                    continue;
                }
                let forced = forced_hits.iter().find(|hit| {
                    hit.avatar_shape_id == attached.shape_id && hit.other.shape_id == shape.shape_id
                });
                if forced.is_none() && !boxes_touch_or_overlap(&moving, shape) {
                    continue;
                }
                let (normal, avatar_feature, feature) = if let Some(hit) = forced {
                    (
                        hit.other.normal_box_to_capsule,
                        hit.avatar_feature,
                        hit.other.box_feature,
                    )
                } else {
                    box_contact_normal_and_features(&moving, shape)?
                };
                let moving_centre = checked_add_vec3(centre, attached.local_centre_micrometres)?;
                let point = [
                    moving_centre[0].clamp(shape.minimum[0], shape.maximum[0]),
                    moving_centre[1].clamp(shape.minimum[1], shape.maximum[1]),
                    moving_centre[2].clamp(shape.minimum[2], shape.maximum[2]),
                ];
                candidates.push(ContactCandidate {
                    avatar_shape_id: attached.shape_id,
                    avatar_feature,
                    shape_id: shape.shape_id,
                    box_feature: feature,
                    point,
                    normal_box_to_capsule: normal,
                });
            }
        }
        candidates.sort();
        candidates.dedup();
        Ok(candidates)
    }

    fn canonicalize_contact(
        &self,
        candidate: ContactCandidate,
    ) -> (PhysicsShapeIdV1, PhysicsShapeIdV1, u8, u8, [i32; 3]) {
        if candidate.shape_id < candidate.avatar_shape_id {
            (
                candidate.shape_id,
                candidate.avatar_shape_id,
                candidate.box_feature,
                candidate.avatar_feature,
                candidate.normal_box_to_capsule,
            )
        } else {
            (
                candidate.avatar_shape_id,
                candidate.shape_id,
                candidate.avatar_feature,
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
        let Some(capsule_body_id) = self.capsule_body_id else {
            return PhysicsContactReportingV1::Disabled;
        };
        let other = if state.participant_low.body_id == capsule_body_id
            && state.participant_high.body_id != capsule_body_id
        {
            state.participant_high
        } else if state.participant_high.body_id == capsule_body_id
            && state.participant_low.body_id != capsule_body_id
        {
            state.participant_low
        } else {
            return PhysicsContactReportingV1::Disabled;
        };
        self.checkpoint
            .catalog
            .bodies
            .get(&other.body_id)
            .and_then(|body| body.shapes.get(&other))
            .map_or(PhysicsContactReportingV1::Disabled, |shape| {
                shape.contact_reporting
            })
    }

    fn collides_with(&self, shape: &GroundedCapsuleStaticBox) -> bool {
        grounded_capsule_collision_filter(
            self.capsule_collision_layer,
            self.capsule_collision_mask,
            shape,
        )
    }

    fn avatar_shape_collides_with(
        &self,
        avatar_shape_id: PhysicsShapeIdV1,
        shape: &GroundedCapsuleStaticBox,
    ) -> bool {
        if Some(avatar_shape_id) == self.capsule_shape_id {
            return self.collides_with(shape);
        }
        self.attached_boxes
            .iter()
            .find(|attached| attached.shape_id == avatar_shape_id)
            .is_some_and(|attached| {
                grounded_capsule_collision_filter(
                    attached.collision_layer,
                    attached.collision_mask,
                    shape,
                )
            })
    }

    pub(super) fn validate_activation_snapshot(&self) -> Result<(), ReferencePhysicsError> {
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
        let dynamic_boxes = self.current_dynamic_boxes(&self.checkpoint.snapshot)?;
        for shape in self.static_boxes.iter().chain(dynamic_boxes.iter()) {
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
        for attached in self.attached_boxes.iter() {
            let moving = attached.at_body_centre(body.pose.translation_micrometres)?;
            for shape in self.static_boxes.iter().chain(dynamic_boxes.iter()) {
                if grounded_capsule_collision_filter(
                    moving.collision_layer,
                    moving.collision_mask,
                    shape,
                ) && boxes_penetrate(&moving, shape)
                {
                    return Err(ReferencePhysicsError::SnapshotPenetrating);
                }
            }
        }
        for dynamic in &dynamic_boxes {
            for fixed in self.static_boxes.iter().filter(|fixed| {
                grounded_capsule_collision_filter(
                    dynamic.collision_layer,
                    dynamic.collision_mask,
                    fixed,
                )
            }) {
                if boxes_penetrate(dynamic, fixed) {
                    return Err(ReferencePhysicsError::SnapshotPenetrating);
                }
            }
        }

        for contact in self
            .checkpoint
            .snapshot
            .sorted_contact_continuity_states
            .values()
        {
            let (avatar_shape_id, other_shape_id) = if contact.participant_low.body_id
                == capsule_body_id
                && contact.participant_high.body_id != capsule_body_id
            {
                (contact.participant_low, contact.participant_high)
            } else if contact.participant_high.body_id == capsule_body_id
                && contact.participant_low.body_id != capsule_body_id
            {
                (contact.participant_high, contact.participant_low)
            } else {
                return Err(ReferencePhysicsError::SnapshotMismatch);
            };
            let other_shape = self
                .contact_boxes(&self.checkpoint.snapshot)?
                .into_iter()
                .find(|shape| shape.shape_id == other_shape_id)
                .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
            if !self.avatar_shape_collides_with(avatar_shape_id, &other_shape) {
                return Err(ReferencePhysicsError::SnapshotMismatch);
            }
        }

        // Contact continuity is imported canonical continuation state. Re-deriving
        // it from the quantized pose loses the swept face at box edges where the
        // closest representable non-penetrating point is just outside the analytic
        // radius. The generic checkpoint validator above already validates contact
        // identities, features, ticks and solver references; this adapter only
        // needs to prove that every contact belongs to its supported capsule/box
        // collision graph.
        Ok(())
    }
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
