use next_contracts::ids::ContentHash;
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND,
    PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2, PhysicsContractError, PhysicsGeometryV1,
    PhysicsMotionKindV1, PhysicsPoseV1, PhysicsQuantizationProfileV1, PhysicsShapeIdV1,
    PhysicsWorldCheckpointV1,
};

use super::error::ReferencePhysicsError;
use super::query::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, ReferenceGroundedCapsuleQuery,
    validate_reference_shape,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedCapsuleWorld<Q> {
    pub(super) checkpoint: PhysicsWorldCheckpointV1,
    pub(super) tick_rate_profile: TickRateProfileV1,
    numeric_profile: AuthoritativeNumericProfileV1,
    quantization_profile: PhysicsQuantizationProfileV1,
    pub(super) capsule_body_id: Option<PhysicsBodyIdV1>,
    pub(super) capsule_shape_id: Option<PhysicsShapeIdV1>,
    pub(super) capsule_radius: i64,
    pub(super) capsule_half_segment: i64,
    pub(super) capsule_collision_layer: u8,
    pub(super) capsule_collision_mask: u64,
    pub(super) static_boxes: Vec<GroundedCapsuleStaticBox>,
    pub(super) locomotion_per_substep: i64,
    pub(super) gravity_velocity_delta: i64,
    pub(super) query: Q,
}

pub type ReferencePhysicsWorld = GroundedCapsuleWorld<ReferenceGroundedCapsuleQuery>;

impl GroundedCapsuleWorld<ReferenceGroundedCapsuleQuery> {
    pub fn new(
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Self, ReferencePhysicsError> {
        Self::with_query(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
            ReferenceGroundedCapsuleQuery,
        )
    }
}

impl<Q: GroundedCapsuleQuery> GroundedCapsuleWorld<Q> {
    pub fn with_query(
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
        query: Q,
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
                static_boxes.push(GroundedCapsuleStaticBox {
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
            query,
        };
        world.validate_activation_snapshot()?;
        Ok(world)
    }

    #[must_use]
    pub fn backend_kind(&self) -> crate::PhysicsBackendKind {
        self.query.backend_kind()
    }

    pub fn recreate_from_checkpoint(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
    ) -> Result<Self, ReferencePhysicsError> {
        Self::with_query(
            checkpoint,
            self.tick_rate_profile,
            self.numeric_profile.clone(),
            self.quantization_profile.clone(),
            self.query.recreate()?,
        )
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

    pub fn snapshot_hash(&self) -> Result<ContentHash, next_contracts::canonical::CanonicalError> {
        self.checkpoint.snapshot.snapshot_hash()
    }

    pub fn checkpoint_hash(
        &self,
    ) -> Result<ContentHash, next_contracts::canonical::CanonicalError> {
        self.checkpoint.checkpoint_hash()
    }

    pub fn set_checkpoint_revision(&mut self, revision: u64) {
        self.checkpoint.snapshot.checkpoint_revision = revision;
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

pub(super) fn checked_sub_vec3(
    left: [i64; 3],
    right: [i64; 3],
) -> Result<[i64; 3], ReferencePhysicsError> {
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
