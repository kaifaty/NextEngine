use std::sync::{Arc, OnceLock};

use next_contracts::canonical::CanonicalError;
use next_contracts::ids::ContentHash;
use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND,
    PhysicsBodyIdV1, PhysicsCanonicalSnapshotV2, PhysicsContractError, PhysicsGeometryV1,
    PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1, PhysicsQuantizationProfileV1,
    PhysicsShapeIdV1, PhysicsWorldCheckpointV1,
};

use super::error::ReferencePhysicsError;
use super::interaction::{GroundedCapsuleAttachedBox, GroundedCapsuleDynamicBox};
use super::query::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, ReferenceGroundedCapsuleQuery,
    validate_reference_shape,
};

#[derive(Clone, Debug)]
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
    pub(super) attached_boxes: Arc<[GroundedCapsuleAttachedBox]>,
    pub(super) static_boxes: Arc<[GroundedCapsuleStaticBox]>,
    pub(super) dynamic_boxes: Arc<[GroundedCapsuleDynamicBox]>,
    pub(super) sensor_boxes: Arc<[GroundedCapsuleStaticBox]>,
    pub(super) locomotion_per_substep: i64,
    pub(super) gravity_velocity_delta: i64,
    pub(super) query: Q,
    memoized_snapshot_hash: OnceLock<Result<ContentHash, CanonicalError>>,
    memoized_catalog_hash: OnceLock<Result<ContentHash, CanonicalError>>,
}

// The memoized hashes are derived caches of exact canonical bytes, never
// parallel authority: equality compares authoritative fields only.
impl<Q: PartialEq> PartialEq for GroundedCapsuleWorld<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.checkpoint == other.checkpoint
            && self.tick_rate_profile == other.tick_rate_profile
            && self.numeric_profile == other.numeric_profile
            && self.quantization_profile == other.quantization_profile
            && self.capsule_body_id == other.capsule_body_id
            && self.capsule_shape_id == other.capsule_shape_id
            && self.capsule_radius == other.capsule_radius
            && self.capsule_half_segment == other.capsule_half_segment
            && self.capsule_collision_layer == other.capsule_collision_layer
            && self.capsule_collision_mask == other.capsule_collision_mask
            && self.attached_boxes == other.attached_boxes
            && self.static_boxes == other.static_boxes
            && self.dynamic_boxes == other.dynamic_boxes
            && self.sensor_boxes == other.sensor_boxes
            && self.locomotion_per_substep == other.locomotion_per_substep
            && self.gravity_velocity_delta == other.gravity_velocity_delta
            && self.query == other.query
    }
}

impl<Q: Eq> Eq for GroundedCapsuleWorld<Q> {}

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
        // ADR-103 / SPEC-26 2.6: the flow network integrates once per
        // gameplay tick, so its authored tick rate must equal the world's;
        // a mismatch rejects before activation or restore.
        if !checkpoint.water_flow.is_empty()
            && checkpoint.water_flow.ticks_per_second != tick_rate_profile.gameplay_hz
        {
            return Err(PhysicsContractError::WaterFlowInvalid.into());
        }

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
            attached_boxes,
        ) = if let Some(capsule_body_id) = capsule_body_id {
            let capsule_descriptor = checkpoint
                .catalog
                .bodies
                .get(&capsule_body_id)
                .ok_or(ReferencePhysicsError::UnsupportedProfile)?;
            if capsule_descriptor.motion_kind != PhysicsMotionKindV1::Kinematic
                || capsule_descriptor.initial_pose.rotation_q1_30
                    != PhysicsPoseV1::default().rotation_q1_30
            {
                return Err(ReferencePhysicsError::UnsupportedProfile);
            }
            let mut capsule_shape = None;
            let mut attached_boxes = Vec::new();
            for shape in capsule_descriptor.shapes.values() {
                validate_reference_shape(shape)?;
                if shape.participation != PhysicsParticipationV1::Solid {
                    return Err(ReferencePhysicsError::UnsupportedProfile);
                }
                match shape.geometry {
                    PhysicsGeometryV1::Capsule {
                        radius_micrometres,
                        half_segment_micrometres,
                    } => {
                        if capsule_shape.is_some()
                            || shape.local_pose.translation_micrometres != [0; 3]
                        {
                            return Err(ReferencePhysicsError::UnsupportedProfile);
                        }
                        capsule_shape = Some((shape, radius_micrometres, half_segment_micrometres));
                    }
                    PhysicsGeometryV1::Box {
                        half_extents_micrometres,
                    } => {
                        if attached_boxes.len() == 1 {
                            return Err(ReferencePhysicsError::UnsupportedProfile);
                        }
                        attached_boxes.push(GroundedCapsuleAttachedBox {
                            shape_id: shape.shape_id,
                            local_centre_micrometres: shape.local_pose.translation_micrometres,
                            half_extents_micrometres,
                            contact_reporting: shape.contact_reporting,
                            collision_layer: shape.collision_layer,
                            collision_mask: shape.collision_mask,
                        });
                    }
                    _ => return Err(ReferencePhysicsError::UnsupportedProfile),
                }
            }
            let (capsule_shape, capsule_radius, capsule_half_segment) =
                capsule_shape.ok_or(ReferencePhysicsError::UnsupportedProfile)?;
            (
                Some(capsule_shape.shape_id),
                capsule_radius,
                capsule_half_segment,
                capsule_shape.collision_layer,
                capsule_shape.collision_mask,
                attached_boxes,
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
            (None, 0, 0, 0, 0, Vec::new())
        };

        let mut static_boxes = Vec::new();
        let mut dynamic_boxes = Vec::new();
        let mut sensor_boxes = Vec::new();
        for (body_id, body) in &checkpoint.catalog.bodies {
            let state = checkpoint
                .snapshot
                .sorted_body_states
                .get(body_id)
                .ok_or(ReferencePhysicsError::SnapshotMismatch)?;
            if Some(*body_id) == capsule_body_id {
                continue;
            }
            if !state.active || state.pose.rotation_q1_30 != PhysicsPoseV1::default().rotation_q1_30
            {
                return Err(ReferencePhysicsError::UnsupportedProfile);
            }
            if body.motion_kind == PhysicsMotionKindV1::Kinematic {
                return Err(ReferencePhysicsError::UnsupportedProfile);
            }
            if body.motion_kind == PhysicsMotionKindV1::Dynamic
                && (body.shapes.len() != 1 || state.angular_velocity_q16 != [0; 3])
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
                if body.motion_kind == PhysicsMotionKindV1::Dynamic {
                    if shape.participation != PhysicsParticipationV1::Solid {
                        return Err(ReferencePhysicsError::UnsupportedProfile);
                    }
                    dynamic_boxes.push(GroundedCapsuleDynamicBox {
                        body_id: *body_id,
                        shape_id: shape.shape_id,
                        local_centre_micrometres: shape.local_pose.translation_micrometres,
                        half_extents_micrometres,
                        contact_reporting: shape.contact_reporting,
                        collision_layer: shape.collision_layer,
                        collision_mask: shape.collision_mask,
                    });
                    continue;
                }
                let centre = checked_add_vec3(
                    state.pose.translation_micrometres,
                    shape.local_pose.translation_micrometres,
                )?;
                let world_box = GroundedCapsuleStaticBox {
                    shape_id: shape.shape_id,
                    minimum: checked_sub_vec3(centre, half_extents_micrometres)?,
                    maximum: checked_add_vec3(centre, half_extents_micrometres)?,
                    contact_reporting: shape.contact_reporting,
                    collision_layer: shape.collision_layer,
                    collision_mask: shape.collision_mask,
                };
                match shape.participation {
                    PhysicsParticipationV1::Solid => static_boxes.push(world_box),
                    PhysicsParticipationV1::Sensor => sensor_boxes.push(world_box),
                    PhysicsParticipationV1::QueryOnly => {}
                }
            }
        }
        static_boxes.sort_by_key(|shape| shape.shape_id);
        dynamic_boxes.sort_by_key(|shape| shape.shape_id);
        sensor_boxes.sort_by_key(|shape| shape.shape_id);
        if dynamic_boxes.len() > 1 {
            return Err(ReferencePhysicsError::UnsupportedProfile);
        }
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
            attached_boxes: Arc::from(attached_boxes),
            static_boxes: Arc::from(static_boxes),
            dynamic_boxes: Arc::from(dynamic_boxes),
            sensor_boxes: Arc::from(sensor_boxes),
            locomotion_per_substep: CAPSULE_LOCOMOTION_SPEED_MICROMETRES_PER_SECOND / physics_hz,
            gravity_velocity_delta: gravity / physics_hz,
            query,
            memoized_snapshot_hash: OnceLock::new(),
            memoized_catalog_hash: OnceLock::new(),
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

    /// Creates an independent live-tick staging world without revalidating or
    /// reconstructing the already validated canonical world closure.
    ///
    /// `None` means the private query/backend state cannot be copied safely and
    /// the caller must use canonical checkpoint reconstruction instead.
    pub fn try_fork_for_staging(&self) -> Result<Option<Self>, ReferencePhysicsError> {
        let Some(query) = self.query.try_fork_for_staging()? else {
            return Ok(None);
        };
        Ok(Some(Self {
            checkpoint: self.checkpoint.clone(),
            tick_rate_profile: self.tick_rate_profile,
            numeric_profile: self.numeric_profile.clone(),
            quantization_profile: self.quantization_profile.clone(),
            capsule_body_id: self.capsule_body_id,
            capsule_shape_id: self.capsule_shape_id,
            capsule_radius: self.capsule_radius,
            capsule_half_segment: self.capsule_half_segment,
            capsule_collision_layer: self.capsule_collision_layer,
            capsule_collision_mask: self.capsule_collision_mask,
            attached_boxes: self.attached_boxes.clone(),
            static_boxes: self.static_boxes.clone(),
            dynamic_boxes: self.dynamic_boxes.clone(),
            sensor_boxes: self.sensor_boxes.clone(),
            locomotion_per_substep: self.locomotion_per_substep,
            gravity_velocity_delta: self.gravity_velocity_delta,
            query,
            memoized_snapshot_hash: self.memoized_snapshot_hash.clone(),
            memoized_catalog_hash: self.memoized_catalog_hash.clone(),
        }))
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

    pub fn snapshot_hash(&self) -> Result<ContentHash, CanonicalError> {
        *self
            .memoized_snapshot_hash
            .get_or_init(|| self.checkpoint.snapshot.snapshot_hash())
    }

    pub fn catalog_hash(&self) -> Result<ContentHash, CanonicalError> {
        *self
            .memoized_catalog_hash
            .get_or_init(|| self.checkpoint.catalog.catalog_hash())
    }

    pub(super) fn reseed_snapshot_hash_memo(&mut self, hash: ContentHash) {
        self.memoized_snapshot_hash = OnceLock::from(Ok(hash));
    }

    pub fn checkpoint_hash(
        &self,
    ) -> Result<ContentHash, next_contracts::canonical::CanonicalError> {
        self.checkpoint.checkpoint_hash()
    }

    #[must_use]
    pub const fn water_volumes(&self) -> &next_contracts::physics::WaterVolumeSetV1 {
        &self.checkpoint.water_volumes
    }

    /// The water table is checkpoint field 4, outside the snapshot and the
    /// catalog: neither derived hash memo is affected.
    pub fn set_water_volumes(&mut self, water_volumes: next_contracts::physics::WaterVolumeSetV1) {
        self.checkpoint.water_volumes = water_volumes;
    }

    #[must_use]
    pub const fn water_flow(&self) -> &next_contracts::physics::WaterFlowNetworkV1 {
        &self.checkpoint.water_flow
    }

    /// The flow network is checkpoint field 5, outside the snapshot and the
    /// catalog: neither derived hash memo is affected.
    pub fn set_water_flow(&mut self, water_flow: next_contracts::physics::WaterFlowNetworkV1) {
        self.checkpoint.water_flow = water_flow;
    }

    pub fn set_checkpoint_revision(&mut self, revision: u64) {
        self.checkpoint.snapshot.checkpoint_revision = revision;
        // checkpoint_revision is canonical snapshot field 4: the mutation
        // invalidates the derived snapshot-hash memo.
        let _ = self.memoized_snapshot_hash.take();
    }
}

pub(super) fn checked_add_vec3(
    left: [i64; 3],
    right: [i64; 3],
) -> Result<[i64; 3], ReferencePhysicsError> {
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
