#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use next_contracts::input::TickRateProfileV1;
use next_contracts::physics::{
    AuthoritativeNumericProfileV1, PHYSICS_CONTACT_NORMAL_X_FIELD_ID,
    PHYSICS_CONTACT_NORMAL_Y_FIELD_ID, PHYSICS_CONTACT_NORMAL_Z_FIELD_ID,
    PHYSICS_SWEEP_DISTANCE_FIELD_ID, PhysicsQuantizationProfileV1, PhysicsShapeIdV1,
    PhysicsWorldCheckpointV1,
};
use next_physics_api::{
    GroundedCapsuleQuery, GroundedCapsuleStaticBox, GroundedCapsuleSweepRequest,
    GroundedCapsuleSweepResult, GroundedCapsuleWorld, PhysicsBackendError, PhysicsBackendFactory,
    PhysicsBackendKind, PhysicsQuantizationError, PhysicsQuantizer, PhysicsWorldBackend,
    ReferencePhysicsError, grounded_capsule_collision_filter, reference_grounded_capsule_sweep,
};
use next_physics_physx_ffi::{
    ArticulationCollisionExclusionV2, ArticulationJointInput, ArticulationLinkInput,
    ArticulationLinkInputV2, ArticulationShapeInputV2, CapsuleAxisSweepInput, ContactOutput,
    ContactOutputV2, JointState, LinkState, NativeWorld, PhysXFfiError, SceneProfileInput,
    StaticBoxInput,
};

pub type PhysXPhysicsWorld = GroundedCapsuleWorld<PhysXGroundedCapsuleQuery>;

pub const PHYSX_CPU_TIMESTEP_HZ: u32 = 240;
pub const PHYSX_CPU_POSITION_ITERATIONS: u32 = 8;
pub const PHYSX_CPU_VELOCITY_ITERATIONS: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysXSceneProfile {
    pub gravity_bits: [u32; 3],
    pub timestep_bits: u32,
    pub position_iterations: u32,
    pub velocity_iterations: u32,
    pub max_contacts: u32,
    pub max_actors: u32,
    pub max_joints: u32,
}

impl PhysXSceneProfile {
    #[must_use]
    pub fn deterministic_humanoid(max_contacts: u32, max_actors: u32, max_joints: u32) -> Self {
        Self {
            gravity_bits: [0.0_f32.to_bits(), (-9.81_f32).to_bits(), 0.0_f32.to_bits()],
            timestep_bits: (1.0_f32 / PHYSX_CPU_TIMESTEP_HZ as f32).to_bits(),
            position_iterations: PHYSX_CPU_POSITION_ITERATIONS,
            velocity_iterations: PHYSX_CPU_VELOCITY_ITERATIONS,
            max_contacts,
            max_actors,
            max_joints,
        }
    }

    fn ffi(self) -> SceneProfileInput {
        SceneProfileInput {
            gravity_bits: self.gravity_bits,
            timestep_bits: self.timestep_bits,
            position_iterations: self.position_iterations,
            velocity_iterations: self.velocity_iterations,
            max_contacts: self.max_contacts,
            max_actors: self.max_actors,
            max_joints: self.max_joints,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysXArticulationCatalog {
    pub static_boxes: Vec<StaticBoxInput>,
    pub links: Vec<ArticulationLinkInput>,
    pub joints: Vec<ArticulationJointInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysXArticulationCatalogV2 {
    pub static_boxes: Vec<StaticBoxInput>,
    pub links: Vec<ArticulationLinkInputV2>,
    pub shapes: Vec<ArticulationShapeInputV2>,
    pub joints: Vec<ArticulationJointInput>,
    pub collision_exclusions: Vec<ArticulationCollisionExclusionV2>,
}

impl PhysXArticulationCatalogV2 {
    fn validate(&self, profile: PhysXSceneProfile) -> Result<(), PhysXAdapterError> {
        if self.links.is_empty()
            || self.joints.len().checked_add(1) != Some(self.links.len())
            || self.links.len() + self.static_boxes.len() > profile.max_actors as usize
            || self.joints.len() > profile.max_joints as usize
            || self.static_boxes.len() > u32::MAX as usize
        {
            return Err(PhysXAdapterError::CapacityExceeded);
        }
        if self
            .static_boxes
            .windows(2)
            .any(|pair| pair[0].user_token >= pair[1].user_token)
            || self.links.iter().enumerate().any(|(index, link)| {
                link.reserved != 0
                    || (index == 0 && link.parent_link_index != u32::MAX)
                    || (index != 0 && link.parent_link_index as usize >= index)
                    || self.links[..index]
                        .iter()
                        .any(|other| other.user_token == link.user_token)
            })
            || self.joints.iter().enumerate().any(|(index, joint)| {
                joint.child_link_index as usize != index + 1 || joint.reserved != 0
            })
            || self
                .collision_exclusions
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(PhysXAdapterError::NonCanonicalConstructionOrder);
        }
        let mut next_shape = 0_usize;
        for (link_index, link) in self.links.iter().enumerate() {
            if link.first_shape_index as usize != next_shape {
                return Err(PhysXAdapterError::NonCanonicalConstructionOrder);
            }
            next_shape = next_shape
                .checked_add(link.shape_count as usize)
                .ok_or(PhysXAdapterError::CapacityExceeded)?;
            if next_shape > self.shapes.len()
                || self.shapes[link.first_shape_index as usize..next_shape]
                    .iter()
                    .any(|shape| shape.link_index as usize != link_index)
            {
                return Err(PhysXAdapterError::NonCanonicalConstructionOrder);
            }
        }
        if next_shape != self.shapes.len() {
            return Err(PhysXAdapterError::NonCanonicalConstructionOrder);
        }
        Ok(())
    }
}

impl PhysXArticulationCatalog {
    fn validate(&self, profile: PhysXSceneProfile) -> Result<(), PhysXAdapterError> {
        if self.links.is_empty()
            || self.joints.len().checked_add(1) != Some(self.links.len())
            || self.links.len() + self.static_boxes.len() > profile.max_actors as usize
            || self.joints.len() > profile.max_joints as usize
            || self.static_boxes.len() > u32::MAX as usize
        {
            return Err(PhysXAdapterError::CapacityExceeded);
        }
        if self
            .static_boxes
            .windows(2)
            .any(|pair| pair[0].user_token >= pair[1].user_token)
            || self.links.iter().enumerate().any(|(index, link)| {
                (index == 0 && link.parent_link_index != u32::MAX)
                    || (index != 0 && link.parent_link_index as usize >= index)
            })
            || self.links.iter().enumerate().any(|(index, link)| {
                self.links[..index]
                    .iter()
                    .any(|other| other.user_token == link.user_token)
            })
            || self.joints.iter().enumerate().any(|(index, joint)| {
                joint.child_link_index as usize != index + 1 || joint.reserved != 0
            })
        {
            return Err(PhysXAdapterError::NonCanonicalConstructionOrder);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysXRawArticulationSnapshot {
    pub links: Vec<LinkState>,
    pub joints: Vec<JointState>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalPhysXLinkState {
    pub user_token: u64,
    pub position_micrometres: [i64; 3],
    pub rotation_q1_30: [i64; 4],
    pub linear_velocity_micrometres_per_second: [i64; 3],
    pub angular_velocity_microradians_per_second: [i64; 3],
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalPhysXJointState {
    pub ordinal: u32,
    pub position_microradians: i64,
    pub velocity_microradians_per_second: i64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalPhysXContact {
    pub actor_a_token: u64,
    pub actor_b_token: u64,
    pub position_micrometres: [i64; 3],
    pub normal_q1_30: [i64; 3],
    pub impulse_micronewton_seconds: [i64; 3],
    pub separation_micrometres: i64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CanonicalPhysXContactV2 {
    pub actor_a_token: u64,
    pub actor_b_token: u64,
    pub shape_a_token: u64,
    pub shape_b_token: u64,
    pub position_micrometres: [i64; 3],
    pub normal_q1_30: [i64; 3],
    pub impulse_micronewton_seconds: [i64; 3],
    pub separation_micrometres: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalPhysXSnapshotV2 {
    pub links: Vec<CanonicalPhysXLinkState>,
    pub joints: Vec<CanonicalPhysXJointState>,
    pub contacts: Vec<CanonicalPhysXContactV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalPhysXSnapshot {
    pub links: Vec<CanonicalPhysXLinkState>,
    pub joints: Vec<CanonicalPhysXJointState>,
    pub contacts: Vec<CanonicalPhysXContact>,
}

pub struct PhysXArticulationWorld {
    native: NativeWorld,
    root_token: u64,
    last_raw_snapshot: PhysXRawArticulationSnapshot,
}

impl PhysXArticulationWorld {
    pub fn create(
        profile: PhysXSceneProfile,
        catalog: &PhysXArticulationCatalog,
    ) -> Result<Self, PhysXAdapterError> {
        catalog.validate(profile)?;
        let mut native = NativeWorld::create()?;
        native.configure_scene(profile.ffi())?;
        native.reserve_static_boxes(
            u32::try_from(catalog.static_boxes.len())
                .map_err(|_| PhysXAdapterError::CapacityExceeded)?,
        )?;
        for descriptor in &catalog.static_boxes {
            native.add_static_box(*descriptor)?;
        }
        native.add_articulation(
            &catalog.links,
            &catalog.joints,
            profile.position_iterations,
            profile.velocity_iterations,
        )?;
        let (links, joints) = native.export_articulation_state()?;
        Ok(Self {
            native,
            root_token: catalog.links[0].user_token,
            last_raw_snapshot: PhysXRawArticulationSnapshot { links, joints },
        })
    }

    pub fn apply_efforts_and_step(
        &mut self,
        efforts_micronewton_metres: &[i64],
    ) -> Result<CanonicalPhysXSnapshot, PhysXAdapterError> {
        let efforts = efforts_micronewton_metres
            .iter()
            .map(|effort| scaled_f32_bits(*effort, 1_000_000.0))
            .collect::<Result<Vec<_>, _>>()?;
        self.native.apply_articulation_efforts(&efforts)?;
        self.native.step()?;
        self.capture()
    }

    pub fn capture(&mut self) -> Result<CanonicalPhysXSnapshot, PhysXAdapterError> {
        let (links, joints) = self.native.export_articulation_state()?;
        let contacts = self.native.export_contacts()?;
        self.last_raw_snapshot = PhysXRawArticulationSnapshot {
            links: links.clone(),
            joints: joints.clone(),
        };
        canonicalize_native_output(&links, &joints, &contacts)
    }

    #[must_use]
    pub fn raw_checkpoint(&self) -> PhysXRawArticulationSnapshot {
        self.last_raw_snapshot.clone()
    }

    pub fn restore(
        &mut self,
        checkpoint: &PhysXRawArticulationSnapshot,
    ) -> Result<CanonicalPhysXSnapshot, PhysXAdapterError> {
        let Some(root) = checkpoint.links.first().copied() else {
            return Err(PhysXAdapterError::InvalidOutput);
        };
        if root.user_token != self.root_token {
            return Err(PhysXAdapterError::ProfileMismatch);
        }
        self.native
            .import_articulation_state(root, &checkpoint.joints)?;
        self.capture()
    }
}

impl Debug for PhysXArticulationWorld {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysXArticulationWorld")
            .field("native", &self.native)
            .field("root_token", &self.root_token)
            .field("link_count", &self.last_raw_snapshot.links.len())
            .field("joint_count", &self.last_raw_snapshot.joints.len())
            .finish()
    }
}

pub struct PhysXArticulationWorldV2 {
    native: NativeWorld,
    root_token: u64,
    last_raw_snapshot: PhysXRawArticulationSnapshot,
}

impl PhysXArticulationWorldV2 {
    pub fn create(
        profile: PhysXSceneProfile,
        catalog: &PhysXArticulationCatalogV2,
    ) -> Result<Self, PhysXAdapterError> {
        catalog.validate(profile)?;
        let mut native = NativeWorld::create()?;
        native.configure_scene(profile.ffi())?;
        native.reserve_static_boxes(
            u32::try_from(catalog.static_boxes.len())
                .map_err(|_| PhysXAdapterError::CapacityExceeded)?,
        )?;
        for descriptor in &catalog.static_boxes {
            native.add_static_box(*descriptor)?;
        }
        native.add_articulation_v2(
            &catalog.links,
            &catalog.shapes,
            &catalog.joints,
            &catalog.collision_exclusions,
            profile.position_iterations,
            profile.velocity_iterations,
        )?;
        let (links, joints) = native.export_articulation_state()?;
        Ok(Self {
            native,
            root_token: catalog.links[0].user_token,
            last_raw_snapshot: PhysXRawArticulationSnapshot { links, joints },
        })
    }

    pub fn apply_efforts_and_step(
        &mut self,
        efforts_micronewton_metres: &[i64],
    ) -> Result<CanonicalPhysXSnapshotV2, PhysXAdapterError> {
        let efforts = efforts_micronewton_metres
            .iter()
            .map(|effort| scaled_f32_bits(*effort, 1_000_000.0))
            .collect::<Result<Vec<_>, _>>()?;
        self.native.apply_articulation_efforts(&efforts)?;
        self.native.step()?;
        self.capture()
    }

    pub fn capture(&mut self) -> Result<CanonicalPhysXSnapshotV2, PhysXAdapterError> {
        let (links, joints) = self.native.export_articulation_state()?;
        let contacts = self.native.export_contacts_v2()?;
        self.last_raw_snapshot = PhysXRawArticulationSnapshot {
            links: links.clone(),
            joints: joints.clone(),
        };
        canonicalize_native_output_v2(&links, &joints, &contacts)
    }

    #[must_use]
    pub fn raw_checkpoint(&self) -> PhysXRawArticulationSnapshot {
        self.last_raw_snapshot.clone()
    }

    pub fn restore(
        &mut self,
        checkpoint: &PhysXRawArticulationSnapshot,
    ) -> Result<CanonicalPhysXSnapshotV2, PhysXAdapterError> {
        let Some(root) = checkpoint.links.first().copied() else {
            return Err(PhysXAdapterError::InvalidOutput);
        };
        if root.user_token != self.root_token {
            return Err(PhysXAdapterError::ProfileMismatch);
        }
        self.native
            .import_articulation_state(root, &checkpoint.joints)?;
        self.capture()
    }
}

impl Debug for PhysXArticulationWorldV2 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysXArticulationWorldV2")
            .field("native", &self.native)
            .field("root_token", &self.root_token)
            .field("link_count", &self.last_raw_snapshot.links.len())
            .field("joint_count", &self.last_raw_snapshot.joints.len())
            .finish()
    }
}

pub struct PhysXGroundedCapsuleQuery {
    native: NativeWorld,
    quantizer: PhysicsQuantizer,
    loaded_shape_ids: BTreeMap<u64, PhysicsShapeIdV1>,
    loaded_scene: Option<Vec<GroundedCapsuleStaticBox>>,
    registration_seed: u64,
}

impl PhysXGroundedCapsuleQuery {
    pub fn new(
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Self, PhysXAdapterError> {
        Self::with_registration_seed(numeric_profile, quantization_profile, 0)
    }

    pub fn with_registration_seed(
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
        registration_seed: u64,
    ) -> Result<Self, PhysXAdapterError> {
        let expected_quantization = PhysicsQuantizationProfileV1::grounded_capsule_v2()
            .expect("built-in PhysX quantization identifiers are valid");
        let expected_numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&expected_quantization)
                .expect("built-in PhysX numeric profile is canonical");
        if quantization_profile != expected_quantization || numeric_profile != expected_numeric {
            return Err(PhysXAdapterError::UnsupportedProfile);
        }
        let quantizer = PhysicsQuantizer::new(&quantization_profile, &numeric_profile)?;
        for field in [
            PHYSICS_SWEEP_DISTANCE_FIELD_ID,
            PHYSICS_CONTACT_NORMAL_X_FIELD_ID,
            PHYSICS_CONTACT_NORMAL_Y_FIELD_ID,
            PHYSICS_CONTACT_NORMAL_Z_FIELD_ID,
        ] {
            quantizer.quantize_f32_bits(field, 0.0_f32.to_bits())?;
        }
        Ok(Self {
            native: NativeWorld::create()?,
            quantizer,
            loaded_shape_ids: BTreeMap::new(),
            loaded_scene: None,
            registration_seed,
        })
    }

    fn ensure_loaded(
        &mut self,
        request: &GroundedCapsuleSweepRequest<'_>,
    ) -> Result<(), PhysXAdapterError> {
        let mut filtered = request
            .static_boxes
            .iter()
            .filter(|shape| {
                grounded_capsule_collision_filter(
                    request.capsule_collision_layer,
                    request.capsule_collision_mask,
                    shape,
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        filtered.sort_by_key(|shape| shape.shape_id);
        if let Some(loaded) = &self.loaded_scene {
            return if loaded == &filtered {
                Ok(())
            } else {
                Err(PhysXAdapterError::SceneChangedAfterActivation)
            };
        }
        let mut shapes = filtered
            .iter()
            .enumerate()
            .map(|(index, shape)| {
                let token = u64::try_from(index)
                    .ok()
                    .and_then(|index| index.checked_add(1))
                    .ok_or(PhysXAdapterError::CapacityExceeded)?;
                Ok((token, shape))
            })
            .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
        shapes.sort_by_key(|(token, _)| permutation_key(self.registration_seed, *token));
        self.native.reserve_static_boxes(
            u32::try_from(shapes.len()).map_err(|_| PhysXAdapterError::CapacityExceeded)?,
        )?;
        for (user_token, shape) in shapes {
            let centre = checked_midpoint(shape.minimum, shape.maximum)?;
            let half_extents = checked_half_extents(shape.minimum, shape.maximum)?;
            self.native.add_static_box(StaticBoxInput {
                centre_bits: metres_bits(centre)?,
                half_extents_bits: metres_bits(half_extents)?,
                user_token,
            })?;
            self.loaded_shape_ids.insert(user_token, shape.shape_id);
        }
        self.loaded_scene = Some(filtered);
        Ok(())
    }

    fn shape_for_token(&self, token: u64) -> Result<PhysicsShapeIdV1, PhysXAdapterError> {
        self.loaded_shape_ids
            .get(&token)
            .copied()
            .ok_or(PhysXAdapterError::InvalidOutput)
    }
}

impl Debug for PhysXGroundedCapsuleQuery {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PhysXGroundedCapsuleQuery")
            .field("native", &self.native)
            .field("loaded_shape_ids", &self.loaded_shape_ids)
            .field(
                "loaded_scene_shape_count",
                &self.loaded_scene.as_ref().map(Vec::len),
            )
            .field("registration_seed", &self.registration_seed)
            .finish_non_exhaustive()
    }
}

impl GroundedCapsuleQuery for PhysXGroundedCapsuleQuery {
    fn backend_kind(&self) -> PhysicsBackendKind {
        PhysicsBackendKind::PhysX
    }

    fn recreate(&self) -> Result<Self, ReferencePhysicsError> {
        Self::with_registration_seed(
            self.quantizer.numeric_profile().clone(),
            self.quantizer.profile().clone(),
            self.registration_seed,
        )
        .map_err(ReferencePhysicsError::from)
    }

    fn sweep_axis(
        &mut self,
        request: GroundedCapsuleSweepRequest<'_>,
    ) -> Result<GroundedCapsuleSweepResult, ReferencePhysicsError> {
        let canonical = reference_grounded_capsule_sweep(GroundedCapsuleSweepRequest {
            centre_micrometres: request.centre_micrometres,
            axis: request.axis,
            delta_micrometres: request.delta_micrometres,
            capsule_radius_micrometres: request.capsule_radius_micrometres,
            capsule_half_segment_micrometres: request.capsule_half_segment_micrometres,
            capsule_collision_layer: request.capsule_collision_layer,
            capsule_collision_mask: request.capsule_collision_mask,
            static_boxes: request.static_boxes,
        })?;
        self.ensure_loaded(&request)?;
        if request.delta_micrometres == 0 {
            return Ok(canonical);
        }
        if canonical.applied_delta_micrometres == 0 && canonical.hit.is_some() {
            return Ok(canonical);
        }
        let distance_micrometres = request.delta_micrometres.unsigned_abs();
        let distance_micrometres =
            i64::try_from(distance_micrometres).map_err(|_| PhysXAdapterError::NumericOverflow)?;
        let output = self
            .native
            .sweep_capsule_axis(CapsuleAxisSweepInput {
                centre_bits: metres_bits(request.centre_micrometres)?,
                radius_bits: metres(request.capsule_radius_micrometres)?.to_bits(),
                half_segment_bits: metres(request.capsule_half_segment_micrometres)?.to_bits(),
                axis: u32::from(request.axis),
                direction_sign: request.delta_micrometres.signum() as i32,
                distance_bits: metres(distance_micrometres)?.to_bits(),
            })
            .map_err(PhysXAdapterError::from)?;
        if !output.hit {
            if canonical.hit.is_some() {
                return Err(ReferencePhysicsError::BackendHitMismatch);
            }
            return Ok(canonical);
        }
        let quantizer = &self.quantizer;
        let distance = quantizer
            .quantize_f32_bits(PHYSICS_SWEEP_DISTANCE_FIELD_ID, output.distance_bits)
            .map_err(PhysXAdapterError::from)?;
        if canonical.hit.is_none() {
            if u64::try_from(distance).ok() == u64::try_from(distance_micrometres).ok() {
                return Ok(canonical);
            }
            return Err(ReferencePhysicsError::BackendHitMismatch);
        }
        let canonical_distance = canonical.applied_delta_micrometres.unsigned_abs();
        if distance < 0
            || distance > distance_micrometres
            || u64::try_from(distance).ok() != Some(canonical_distance)
        {
            return Err(ReferencePhysicsError::BackendDistanceMismatch);
        }
        let shape_id = self.shape_for_token(output.user_token)?;
        let hit = canonical.hit.ok_or(PhysXAdapterError::InvalidOutput)?;
        if hit.shape_id != shape_id {
            return Err(ReferencePhysicsError::BackendFeatureMismatch);
        }
        let quantized_normal = [
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_X_FIELD_ID, output.normal_bits[0])
                .map_err(PhysXAdapterError::from)?,
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_Y_FIELD_ID, output.normal_bits[1])
                .map_err(PhysXAdapterError::from)?,
            quantizer
                .quantize_f32_bits(PHYSICS_CONTACT_NORMAL_Z_FIELD_ID, output.normal_bits[2])
                .map_err(PhysXAdapterError::from)?,
        ];
        let quantized_normal = [
            i32::try_from(quantized_normal[0]).map_err(|_| PhysXAdapterError::InvalidOutput)?,
            i32::try_from(quantized_normal[1]).map_err(|_| PhysXAdapterError::InvalidOutput)?,
            i32::try_from(quantized_normal[2]).map_err(|_| PhysXAdapterError::InvalidOutput)?,
        ];
        if quantized_normal != hit.normal_box_to_capsule {
            return Err(ReferencePhysicsError::BackendNormalMismatch);
        }
        Ok(canonical)
    }
}

fn canonicalize_native_output(
    links: &[LinkState],
    joints: &[JointState],
    contacts: &[ContactOutput],
) -> Result<CanonicalPhysXSnapshot, PhysXAdapterError> {
    let mut canonical_links = links
        .iter()
        .map(|link| {
            Ok(CanonicalPhysXLinkState {
                user_token: link.user_token,
                position_micrometres: quantize_vector(link.position_bits, 1_000_000.0)?,
                rotation_q1_30: [
                    quantize_bits(link.rotation_bits[0], (1_u64 << 30) as f64)?,
                    quantize_bits(link.rotation_bits[1], (1_u64 << 30) as f64)?,
                    quantize_bits(link.rotation_bits[2], (1_u64 << 30) as f64)?,
                    quantize_bits(link.rotation_bits[3], (1_u64 << 30) as f64)?,
                ],
                linear_velocity_micrometres_per_second: quantize_vector(
                    link.linear_velocity_bits,
                    1_000_000.0,
                )?,
                angular_velocity_microradians_per_second: quantize_vector(
                    link.angular_velocity_bits,
                    1_000_000.0,
                )?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
    canonical_links.sort_unstable_by_key(|link| link.user_token);
    if canonical_links
        .windows(2)
        .any(|pair| pair[0].user_token == pair[1].user_token)
    {
        return Err(PhysXAdapterError::InvalidOutput);
    }

    let canonical_joints = joints
        .iter()
        .enumerate()
        .map(|(ordinal, joint)| {
            Ok(CanonicalPhysXJointState {
                ordinal: u32::try_from(ordinal).map_err(|_| PhysXAdapterError::CapacityExceeded)?,
                position_microradians: quantize_bits(joint.position_bits, 1_000_000.0)?,
                velocity_microradians_per_second: quantize_bits(joint.velocity_bits, 1_000_000.0)?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;

    let mut canonical_contacts = contacts
        .iter()
        .map(|contact| {
            let swapped = contact.actor_a_token > contact.actor_b_token;
            let (actor_a_token, actor_b_token) = if swapped {
                (contact.actor_b_token, contact.actor_a_token)
            } else {
                (contact.actor_a_token, contact.actor_b_token)
            };
            let mut normal = quantize_vector(contact.normal_bits, (1_u64 << 30) as f64)?;
            let mut impulse = quantize_vector(contact.impulse_bits, 1_000_000.0)?;
            if swapped {
                for value in &mut normal {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
                for value in &mut impulse {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
            }
            Ok(CanonicalPhysXContact {
                actor_a_token,
                actor_b_token,
                position_micrometres: quantize_vector(contact.position_bits, 1_000_000.0)?,
                normal_q1_30: normal,
                impulse_micronewton_seconds: impulse,
                separation_micrometres: quantize_bits(contact.separation_bits, 1_000_000.0)?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
    canonical_contacts.sort_unstable();
    Ok(CanonicalPhysXSnapshot {
        links: canonical_links,
        joints: canonical_joints,
        contacts: canonical_contacts,
    })
}

fn canonicalize_native_output_v2(
    links: &[LinkState],
    joints: &[JointState],
    contacts: &[ContactOutputV2],
) -> Result<CanonicalPhysXSnapshotV2, PhysXAdapterError> {
    let base = canonicalize_native_output(links, joints, &[])?;
    let mut canonical_contacts = contacts
        .iter()
        .map(|contact| {
            let swapped = (contact.actor_a_token, contact.shape_a_token)
                > (contact.actor_b_token, contact.shape_b_token);
            let (actor_a_token, actor_b_token, shape_a_token, shape_b_token) = if swapped {
                (
                    contact.actor_b_token,
                    contact.actor_a_token,
                    contact.shape_b_token,
                    contact.shape_a_token,
                )
            } else {
                (
                    contact.actor_a_token,
                    contact.actor_b_token,
                    contact.shape_a_token,
                    contact.shape_b_token,
                )
            };
            let mut normal = quantize_vector(contact.normal_bits, (1_u64 << 30) as f64)?;
            let mut impulse = quantize_vector(contact.impulse_bits, 1_000_000.0)?;
            if swapped {
                for value in &mut normal {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
                for value in &mut impulse {
                    *value = value
                        .checked_neg()
                        .ok_or(PhysXAdapterError::NumericOverflow)?;
                }
            }
            Ok(CanonicalPhysXContactV2 {
                actor_a_token,
                actor_b_token,
                shape_a_token,
                shape_b_token,
                position_micrometres: quantize_vector(contact.position_bits, 1_000_000.0)?,
                normal_q1_30: normal,
                impulse_micronewton_seconds: impulse,
                separation_micrometres: quantize_bits(contact.separation_bits, 1_000_000.0)?,
            })
        })
        .collect::<Result<Vec<_>, PhysXAdapterError>>()?;
    canonical_contacts.sort_unstable();
    Ok(CanonicalPhysXSnapshotV2 {
        links: base.links,
        joints: base.joints,
        contacts: canonical_contacts,
    })
}

fn quantize_vector(bits: [u32; 3], scale: f64) -> Result<[i64; 3], PhysXAdapterError> {
    Ok([
        quantize_bits(bits[0], scale)?,
        quantize_bits(bits[1], scale)?,
        quantize_bits(bits[2], scale)?,
    ])
}

fn quantize_bits(bits: u32, scale: f64) -> Result<i64, PhysXAdapterError> {
    let value = f64::from(f32::from_bits(bits));
    let scaled = value * scale;
    if !scaled.is_finite() || scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
        return Err(PhysXAdapterError::NumericOverflow);
    }
    Ok(scaled.round_ties_even() as i64)
}

fn scaled_f32_bits(value: i64, scale: f64) -> Result<u32, PhysXAdapterError> {
    let value = value as f64 / scale;
    let value = value as f32;
    if value.is_finite() {
        Ok(value.to_bits())
    } else {
        Err(PhysXAdapterError::NumericOverflow)
    }
}

fn permutation_key(seed: u64, token: u64) -> u64 {
    let mut value = seed ^ token.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhysXPhysicsFactory;

impl PhysicsBackendFactory for PhysXPhysicsFactory {
    fn backend_kind(&self) -> PhysicsBackendKind {
        PhysicsBackendKind::PhysX
    }

    fn create(
        &self,
        checkpoint: PhysicsWorldCheckpointV1,
        tick_rate_profile: TickRateProfileV1,
        numeric_profile: AuthoritativeNumericProfileV1,
        quantization_profile: PhysicsQuantizationProfileV1,
    ) -> Result<Box<dyn PhysicsWorldBackend>, PhysicsBackendError> {
        let query =
            PhysXGroundedCapsuleQuery::new(numeric_profile.clone(), quantization_profile.clone())
                .map_err(ReferencePhysicsError::from)?;
        let world = PhysXPhysicsWorld::with_query(
            checkpoint,
            tick_rate_profile,
            numeric_profile,
            quantization_profile,
            query,
        )?;
        Ok(Box::new(world))
    }
}

fn checked_midpoint(minimum: [i64; 3], maximum: [i64; 3]) -> Result<[i64; 3], PhysXAdapterError> {
    let mut output = [0; 3];
    for axis in 0..3 {
        output[axis] = minimum[axis]
            .checked_add(maximum[axis])
            .and_then(|value| value.checked_div(2))
            .ok_or(PhysXAdapterError::NumericOverflow)?;
    }
    Ok(output)
}

fn checked_half_extents(
    minimum: [i64; 3],
    maximum: [i64; 3],
) -> Result<[i64; 3], PhysXAdapterError> {
    let mut output = [0; 3];
    for axis in 0..3 {
        output[axis] = maximum[axis]
            .checked_sub(minimum[axis])
            .and_then(|value| value.checked_div(2))
            .filter(|value| *value > 0)
            .ok_or(PhysXAdapterError::NumericOverflow)?;
    }
    Ok(output)
}

fn metres_bits(values: [i64; 3]) -> Result<[u32; 3], PhysXAdapterError> {
    Ok([
        metres(values[0])?.to_bits(),
        metres(values[1])?.to_bits(),
        metres(values[2])?.to_bits(),
    ])
}

fn metres(micrometres: i64) -> Result<f32, PhysXAdapterError> {
    let value = micrometres as f32 / 1_000_000.0;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PhysXAdapterError::NumericOverflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysXAdapterError {
    Ffi(PhysXFfiError),
    Quantization(PhysicsQuantizationError),
    UnsupportedProfile,
    CapacityExceeded,
    NumericOverflow,
    InvalidOutput,
    SceneChangedAfterActivation,
    NonCanonicalConstructionOrder,
    ProfileMismatch,
}

impl PhysXAdapterError {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::Ffi(error) => error.stable_code(),
            Self::Quantization(error) => error.stable_code(),
            Self::UnsupportedProfile => "PHYSX_PROFILE_UNSUPPORTED",
            Self::CapacityExceeded => "PHYSX_CAPACITY_EXCEEDED",
            Self::NumericOverflow => "PHYSICS_NUMERIC_OVERFLOW",
            Self::InvalidOutput => "PHYSX_INVALID_OUTPUT",
            Self::SceneChangedAfterActivation => "PHYSX_SCENE_CHANGED_AFTER_ACTIVATION",
            Self::NonCanonicalConstructionOrder => "PHYSX_NON_CANONICAL_CONSTRUCTION_ORDER",
            Self::ProfileMismatch => "PHYSX_PROFILE_MISMATCH",
        }
    }
}

impl Display for PhysXAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for PhysXAdapterError {}

impl From<PhysXFfiError> for PhysXAdapterError {
    fn from(error: PhysXFfiError) -> Self {
        Self::Ffi(error)
    }
}

impl From<PhysicsQuantizationError> for PhysXAdapterError {
    fn from(error: PhysicsQuantizationError) -> Self {
        Self::Quantization(error)
    }
}

impl From<PhysXAdapterError> for ReferencePhysicsError {
    fn from(error: PhysXAdapterError) -> Self {
        match error {
            PhysXAdapterError::Ffi(PhysXFfiError::Unavailable) => Self::BackendUnavailable,
            PhysXAdapterError::Ffi(PhysXFfiError::VersionMismatch) => Self::BackendVersionMismatch,
            PhysXAdapterError::Ffi(PhysXFfiError::CapacityExceeded)
            | PhysXAdapterError::CapacityExceeded => Self::BackendCapacityExceeded,
            PhysXAdapterError::Quantization(
                PhysicsQuantizationError::NumericOverflow | PhysicsQuantizationError::OutOfBounds,
            )
            | PhysXAdapterError::NumericOverflow => Self::NumericOverflow,
            PhysXAdapterError::UnsupportedProfile => Self::UnsupportedProfile,
            PhysXAdapterError::Ffi(_)
            | PhysXAdapterError::Quantization(_)
            | PhysXAdapterError::InvalidOutput
            | PhysXAdapterError::SceneChangedAfterActivation
            | PhysXAdapterError::NonCanonicalConstructionOrder
            | PhysXAdapterError::ProfileMismatch => Self::BackendFailure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
    fn two_link_catalog() -> PhysXArticulationCatalog {
        let zero = 0.0_f32.to_bits();
        let identity = [zero, zero, zero, 1.0_f32.to_bits()];
        PhysXArticulationCatalog {
            static_boxes: vec![StaticBoxInput {
                centre_bits: [zero, (-0.5_f32).to_bits(), zero],
                half_extents_bits: [5.0_f32.to_bits(), 0.5_f32.to_bits(), 5.0_f32.to_bits()],
                user_token: 1,
            }],
            links: vec![
                ArticulationLinkInput {
                    user_token: 10,
                    parent_link_index: next_physics_physx_ffi::NO_PARENT_LINK,
                    shape_kind: next_physics_physx_ffi::SHAPE_CAPSULE,
                    position_bits: [zero, 2.0_f32.to_bits(), zero],
                    rotation_bits: identity,
                    shape_dimensions_bits: [0.2_f32.to_bits(), 0.25_f32.to_bits(), zero],
                    mass_bits: 5.0_f32.to_bits(),
                    inertia_bits: [0.2_f32.to_bits(); 3],
                    linear_damping_bits: 0.05_f32.to_bits(),
                    angular_damping_bits: 0.05_f32.to_bits(),
                },
                ArticulationLinkInput {
                    user_token: 11,
                    parent_link_index: 0,
                    shape_kind: next_physics_physx_ffi::SHAPE_CAPSULE,
                    position_bits: [zero, 1.5_f32.to_bits(), zero],
                    rotation_bits: identity,
                    shape_dimensions_bits: [0.15_f32.to_bits(), 0.2_f32.to_bits(), zero],
                    mass_bits: 2.0_f32.to_bits(),
                    inertia_bits: [0.1_f32.to_bits(); 3],
                    linear_damping_bits: 0.05_f32.to_bits(),
                    angular_damping_bits: 0.05_f32.to_bits(),
                },
            ],
            joints: vec![ArticulationJointInput {
                child_link_index: 1,
                reserved: 0,
                parent_position_bits: [zero, (-0.25_f32).to_bits(), zero],
                parent_rotation_bits: identity,
                child_position_bits: [zero, 0.2_f32.to_bits(), zero],
                child_rotation_bits: identity,
                lower_limit_bits: (-1.0_f32).to_bits(),
                upper_limit_bits: 1.0_f32.to_bits(),
                max_velocity_bits: 20.0_f32.to_bits(),
            }],
        }
    }

    #[cfg(feature = "physx-sdk")]
    fn three_link_collision_catalog() -> PhysXArticulationCatalogV2 {
        let zero = 0.0_f32.to_bits();
        let identity = [zero, zero, zero, 1.0_f32.to_bits()];
        let links = (0_u32..3)
            .map(|index| ArticulationLinkInputV2 {
                user_token: 100 + u64::from(index),
                parent_link_index: if index == 0 {
                    next_physics_physx_ffi::NO_PARENT_LINK
                } else {
                    index - 1
                },
                first_shape_index: index,
                shape_count: 1,
                reserved: 0,
                position_bits: [zero, 2.0_f32.to_bits(), zero],
                rotation_bits: identity,
                centre_of_mass_position_bits: [zero; 3],
                centre_of_mass_rotation_bits: identity,
                mass_bits: 1.0_f32.to_bits(),
                inertia_bits: [0.1_f32.to_bits(); 3],
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            })
            .collect();
        let shapes = (0_u32..3)
            .map(|index| ArticulationShapeInputV2 {
                user_token: 1_000 + u64::from(index),
                link_index: index,
                shape_kind: next_physics_physx_ffi::SHAPE_SPHERE,
                position_bits: [zero; 3],
                rotation_bits: identity,
                shape_dimensions_bits: [0.4_f32.to_bits(), zero, zero],
                collision_layer: 2,
                collision_mask_low: 1 << 2,
                collision_mask_high: 0,
            })
            .collect();
        let joints = (1_u32..3)
            .map(|child_link_index| ArticulationJointInput {
                child_link_index,
                reserved: 0,
                parent_position_bits: [zero; 3],
                parent_rotation_bits: identity,
                child_position_bits: [zero; 3],
                child_rotation_bits: identity,
                lower_limit_bits: (-1.0_f32).to_bits(),
                upper_limit_bits: 1.0_f32.to_bits(),
                max_velocity_bits: 20.0_f32.to_bits(),
            })
            .collect();
        PhysXArticulationCatalogV2 {
            static_boxes: Vec::new(),
            links,
            shapes,
            joints,
            collision_exclusions: Vec::new(),
        }
    }

    #[cfg(feature = "physx-sdk")]
    fn observed_contact_pair(
        catalog: &PhysXArticulationCatalogV2,
        expected_shape_tokens: (u64, u64),
    ) -> bool {
        let mut profile = PhysXSceneProfile::deterministic_humanoid(128, 8, 4);
        profile.gravity_bits = [0.0_f32.to_bits(); 3];
        let mut world = PhysXArticulationWorldV2::create(profile, catalog).expect("V2 world");
        (0..4).any(|_| {
            world
                .apply_efforts_and_step(&[0; 2])
                .expect("collision step")
                .contacts
                .iter()
                .any(|contact| {
                    (contact.shape_a_token, contact.shape_b_token) == expected_shape_tokens
                })
        })
    }

    #[test]
    #[cfg(not(any(feature = "physx-sdk", feature = "mock-abi")))]
    fn disabled_sdk_is_reported_before_world_activation() {
        let quantization =
            PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization).expect("numeric");
        let error =
            PhysXGroundedCapsuleQuery::new(numeric, quantization).expect_err("SDK unavailable");
        assert_eq!(error.stable_code(), "PHYSX_SDK_UNAVAILABLE");
    }

    #[test]
    fn legacy_reference_profile_is_not_accepted_by_physx() {
        let quantization =
            PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
        let numeric =
            AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
        let error = PhysXGroundedCapsuleQuery::new(numeric, quantization)
            .expect_err("legacy profile remains reference-only");
        assert_eq!(error.stable_code(), "PHYSX_PROFILE_UNSUPPORTED");
    }

    #[test]
    fn altered_v2_recipe_is_not_accepted_under_the_builtin_profile_id() {
        let mut quantization =
            PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("quantization");
        quantization
            .rules
            .get_mut(
                &next_contracts::ids::SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID)
                    .expect("distance field ID"),
            )
            .expect("distance rule")
            .scale_numerator += 1;
        let numeric =
            AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization).expect("numeric");
        let error = PhysXGroundedCapsuleQuery::new(numeric, quantization)
            .expect_err("altered built-in recipe");
        assert_eq!(error.stable_code(), "PHYSX_PROFILE_UNSUPPORTED");
    }

    #[test]
    #[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
    fn articulation_adapter_steps_canonicalizes_and_restores() {
        let profile = PhysXSceneProfile::deterministic_humanoid(128, 8, 4);
        let catalog = two_link_catalog();
        let mut world = PhysXArticulationWorld::create(profile, &catalog).expect("world");
        let checkpoint = world.raw_checkpoint();
        let stepped = world.apply_efforts_and_step(&[10_000_000]).expect("step");
        assert_eq!(stepped.links.len(), 2);
        assert_eq!(stepped.joints.len(), 1);
        assert_ne!(stepped.joints[0].velocity_microradians_per_second, 0);
        let restored = world.restore(&checkpoint).expect("restore");
        assert_eq!(restored.joints[0].position_microradians, 0);
        assert_eq!(restored.joints[0].velocity_microradians_per_second, 0);
    }

    #[test]
    #[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
    fn articulation_adapter_rejects_ambiguous_construction_order() {
        let profile = PhysXSceneProfile::deterministic_humanoid(128, 8, 4);
        let mut catalog = two_link_catalog();
        catalog.links[1].user_token = catalog.links[0].user_token;
        let error = PhysXArticulationWorld::create(profile, &catalog)
            .expect_err("duplicate semantic token");
        assert_eq!(
            error.stable_code(),
            "PHYSX_NON_CANONICAL_CONSTRUCTION_ORDER"
        );
    }

    #[test]
    #[cfg(feature = "physx-sdk")]
    fn v2_native_contacts_obey_masks_exclusions_and_preserve_shape_tokens() {
        let catalog = three_link_collision_catalog();
        assert!(
            observed_contact_pair(&catalog, (1_000, 1_002)),
            "the admitted non-adjacent self-collision must expose both shape tokens"
        );

        let mut mask_filtered = catalog.clone();
        mask_filtered.shapes[2].collision_mask_low = 1 << 3;
        assert!(
            !observed_contact_pair(&mask_filtered, (1_000, 1_002)),
            "a mutually incompatible collision mask must suppress the pair"
        );

        let mut explicitly_excluded = catalog;
        explicitly_excluded.collision_exclusions = vec![ArticulationCollisionExclusionV2 {
            first_link_index: 0,
            second_link_index: 2,
        }];
        assert!(
            !observed_contact_pair(&explicitly_excluded, (1_000, 1_002)),
            "an explicit self-collision exclusion must suppress the pair"
        );
    }
}
