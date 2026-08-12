use std::collections::{BTreeMap, BTreeSet};

use next_contracts::body::{BodyActuatorDefinitionV2, BodyContactRoleV2, BodyPoseV2, BodySchemaV2};
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::physics::{
    PHYSICS_ACTUATOR_DESCRIPTOR_V1_SCHEMA_VERSION, PHYSICS_ACTUATOR_DESCRIPTOR_V2_SCHEMA_VERSION,
    PHYSICS_BODY_DESCRIPTOR_V2_SCHEMA_VERSION, PHYSICS_BODY_DESCRIPTOR_V3_SCHEMA_VERSION,
    PHYSICS_JOINT_DESCRIPTOR_V1_SCHEMA_VERSION, PHYSICS_JOINT_DESCRIPTOR_V2_SCHEMA_VERSION,
    PhysicsActuatorDescriptorV1, PhysicsActuatorDescriptorV2, PhysicsBodyDescriptorV1,
    PhysicsBodyDescriptorV2, PhysicsBodyDescriptorV3, PhysicsBodyIdV1, PhysicsGeometryV1,
    PhysicsJointDescriptorV1, PhysicsJointDescriptorV2, PhysicsJointKindV1, PhysicsMotionKindV1,
    PhysicsPoseV1, PhysicsShapeDescriptorV1, PhysicsShapeIdV1,
};
use next_physics_physx::{PhysXArticulationCatalogV2, PhysXSceneProfile};
use next_physics_physx_ffi::{
    ArticulationCollisionExclusionV2, ArticulationJointInput, ArticulationLinkInputV2,
    ArticulationShapeInputV2, NO_PARENT_LINK, SHAPE_BOX, SHAPE_CAPSULE, SHAPE_SPHERE,
    StaticBoxInput,
};

use crate::MotorCompileError;

const ARTICULATION_ID: &str = "articulation.humanoid-biomechanics.v2";
const BODY_TOKEN_BASE: u64 = 1_000;
const SHAPE_TOKEN_BASE: u64 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledPhysicsDescriptorsV2 {
    pub bodies: BTreeMap<PhysicsBodyIdV1, PhysicsBodyDescriptorV3>,
    pub joints: Vec<PhysicsJointDescriptorV2>,
    pub actuators: Vec<PhysicsActuatorDescriptorV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBodySchemaV2 {
    pub body_schema_hash: ContentHash,
    pub compiled_descriptor_hash: ContentHash,
    pub construction_order: Vec<SchemaId>,
    pub body_tokens: BTreeMap<SchemaId, u64>,
    pub collider_tokens: BTreeMap<SchemaId, u64>,
    pub collider_contact_roles: BTreeMap<u64, BodyContactRoleV2>,
    pub effector_body_tokens: BTreeMap<SchemaId, u64>,
    pub joint_dof_ordinals: BTreeMap<SchemaId, u32>,
    pub actuator_dof_ordinals: Vec<u32>,
    pub physics_descriptors: CompiledPhysicsDescriptorsV2,
    pub physx_scene_profile: PhysXSceneProfile,
    pub physx_catalog: PhysXArticulationCatalogV2,
    pub actuator_definitions: Vec<BodyActuatorDefinitionV2>,
}

impl CompiledBodySchemaV2 {
    pub fn compile(
        schema: &BodySchemaV2,
        subject_id: PersistentId,
    ) -> Result<Self, MotorCompileError> {
        schema.validate()?;
        let body_schema_hash = schema.schema_hash()?;
        let construction_order = topological_construction_order(schema)?;
        let body_by_id = schema
            .bodies
            .iter()
            .map(|body| (body.body_id.clone(), body))
            .collect::<BTreeMap<_, _>>();
        let joint_by_child = schema
            .joints
            .iter()
            .map(|joint| (joint.child_body_id.clone(), joint))
            .collect::<BTreeMap<_, _>>();
        let articulation_id = schema_id(ARTICULATION_ID);
        let mut body_tokens = BTreeMap::new();
        let mut collider_tokens = BTreeMap::new();
        let mut collider_contact_roles = BTreeMap::new();
        let mut body_ids = BTreeMap::new();
        let mut global_poses = BTreeMap::new();
        let mut ffi_links = Vec::with_capacity(schema.bodies.len());
        let mut ffi_shapes = Vec::new();
        let mut ffi_joints = Vec::with_capacity(schema.joints.len());
        let mut joint_dof_ordinals = BTreeMap::new();
        let mut bodies = BTreeMap::new();

        for (ordinal, semantic_id) in construction_order.iter().enumerate() {
            let body = body_by_id
                .get(semantic_id)
                .copied()
                .ok_or(MotorCompileError::InvalidReference)?;
            require_identity_rotation(body.local_bind_pose)?;
            let parent_index =
                body.parent_body_id
                    .as_ref()
                    .map_or(Ok(NO_PARENT_LINK), |parent| {
                        construction_order[..ordinal]
                            .iter()
                            .position(|candidate| candidate == parent)
                            .and_then(|value| u32::try_from(value).ok())
                            .ok_or(MotorCompileError::NonCanonicalTopology)
                    })?;
            let parent_translation = body
                .parent_body_id
                .as_ref()
                .map_or([0; 3], |parent| global_poses[parent]);
            let global_translation = checked_add_vector(
                parent_translation,
                body.local_bind_pose.translation_micrometres,
            )?;
            global_poses.insert(semantic_id.clone(), global_translation);
            let token = BODY_TOKEN_BASE
                .checked_add(u64::try_from(ordinal).map_err(|_| MotorCompileError::Capacity)?)
                .ok_or(MotorCompileError::Capacity)?;
            body_tokens.insert(semantic_id.clone(), token);
            let body_id = PhysicsBodyIdV1 {
                subject_id,
                body_slot: u32::try_from(ordinal).map_err(|_| MotorCompileError::Capacity)?,
            };
            body_ids.insert(semantic_id.clone(), body_id);

            let first_shape_index =
                u32::try_from(ffi_shapes.len()).map_err(|_| MotorCompileError::Capacity)?;
            let mut descriptor_shapes = BTreeMap::new();
            for (shape_slot, collider) in body.colliders.iter().enumerate() {
                let shape_index = ffi_shapes.len();
                let shape_token = SHAPE_TOKEN_BASE
                    .checked_add(
                        u64::try_from(shape_index).map_err(|_| MotorCompileError::Capacity)?,
                    )
                    .ok_or(MotorCompileError::Capacity)?;
                if collider_tokens
                    .insert(collider.collider_id.clone(), shape_token)
                    .is_some()
                    || collider_contact_roles
                        .insert(shape_token, collider.contact_role)
                        .is_some()
                {
                    return Err(MotorCompileError::InvalidReference);
                }
                let (shape_kind, shape_dimensions_bits) = geometry_to_ffi(&collider.geometry)?;
                ffi_shapes.push(ArticulationShapeInputV2 {
                    user_token: shape_token,
                    link_index: u32::try_from(ordinal).map_err(|_| MotorCompileError::Capacity)?,
                    shape_kind,
                    position_bits: metres_bits(collider.local_pose.translation_micrometres)?,
                    rotation_bits: quaternion_bits(collider.local_pose.rotation_q1_30),
                    shape_dimensions_bits,
                    collision_layer: u32::from(collider.collision_layer),
                    collision_mask_low: collider.collision_mask as u32,
                    collision_mask_high: (collider.collision_mask >> 32) as u32,
                });
                let shape_id = PhysicsShapeIdV1 {
                    body_id,
                    shape_slot: u32::try_from(shape_slot)
                        .map_err(|_| MotorCompileError::Capacity)?,
                };
                descriptor_shapes.insert(
                    shape_id,
                    PhysicsShapeDescriptorV1 {
                        shape_id,
                        descriptor_revision: 1,
                        local_pose: physics_pose(collider.local_pose)?,
                        geometry: collider.geometry.clone(),
                        material_id: collider.material_id.clone(),
                        collision_layer: collider.collision_layer,
                        collision_mask: collider.collision_mask,
                        participation: collider.participation,
                        contact_reporting: collider.contact_reporting,
                    },
                );
            }
            ffi_links.push(ArticulationLinkInputV2 {
                user_token: token,
                parent_link_index: parent_index,
                first_shape_index,
                shape_count: u32::try_from(body.colliders.len())
                    .map_err(|_| MotorCompileError::Capacity)?,
                reserved: 0,
                position_bits: metres_bits(global_translation)?,
                rotation_bits: quaternion_bits(body.local_bind_pose.rotation_q1_30),
                centre_of_mass_position_bits: metres_bits(body.solver_center_of_mass_micrometres)?,
                centre_of_mass_rotation_bits: quaternion_bits(
                    body.solver_principal_frame.rotation_q1_30,
                ),
                mass_bits: scaled_u64_bits(body.solver_mass_microkilograms, 1_000_000.0)?,
                inertia_bits: body
                    .solver_principal_inertia_microkilogram_metre_squared
                    .map(|value| scaled_u64_bits(value, 1_000_000.0))
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| MotorCompileError::Capacity)?,
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            });
            let base = PhysicsBodyDescriptorV1 {
                body_id,
                descriptor_revision: 2,
                motion_kind: PhysicsMotionKindV1::Dynamic,
                initial_pose: PhysicsPoseV1 {
                    translation_micrometres: global_translation,
                    rotation_q1_30: body.local_bind_pose.rotation_q1_30,
                },
                initial_linear_velocity_micrometres_per_second: [0; 3],
                initial_angular_velocity_q16: [0; 3],
                active: true,
                shapes: descriptor_shapes,
            };
            bodies.insert(
                body_id,
                PhysicsBodyDescriptorV3 {
                    schema_version: PHYSICS_BODY_DESCRIPTOR_V3_SCHEMA_VERSION,
                    base: PhysicsBodyDescriptorV2 {
                        schema_version: PHYSICS_BODY_DESCRIPTOR_V2_SCHEMA_VERSION,
                        semantic_body_id: semantic_id.clone(),
                        base,
                        mass_microkilograms: body.solver_mass_microkilograms,
                        center_of_mass_micrometres: body.solver_center_of_mass_micrometres,
                        inertia_microkilogram_metre_squared: body
                            .solver_principal_inertia_microkilogram_metre_squared,
                        articulation_id: Some(articulation_id.clone()),
                    },
                    non_colliding_carrier: body.colliders.is_empty(),
                    authoritative_inertia_tensor_microkilogram_metre_squared: body
                        .inertia_tensor_microkilogram_metre_squared,
                    solver_principal_frame: physics_pose(body.solver_principal_frame)?,
                    solver_tensor_error_max_microkilogram_metre_squared: body
                        .solver_tensor_error_max_microkilogram_metre_squared,
                },
            );

            if ordinal != 0 {
                let joint = joint_by_child
                    .get(semantic_id)
                    .copied()
                    .ok_or(MotorCompileError::InvalidReference)?;
                let dof =
                    u32::try_from(ffi_joints.len()).map_err(|_| MotorCompileError::Capacity)?;
                joint_dof_ordinals.insert(joint.joint_id.clone(), dof);
                let alignment = axis_alignment_quaternion_bits(joint.axis_q1_30)?;
                ffi_joints.push(ArticulationJointInput {
                    child_link_index: u32::try_from(ordinal)
                        .map_err(|_| MotorCompileError::Capacity)?,
                    reserved: 0,
                    parent_position_bits: metres_bits(joint.parent_frame.translation_micrometres)?,
                    parent_rotation_bits: compose_quaternion_bits(
                        joint.parent_frame.rotation_q1_30,
                        alignment,
                    )?,
                    child_position_bits: metres_bits(joint.child_frame.translation_micrometres)?,
                    child_rotation_bits: compose_quaternion_bits(
                        joint.child_frame.rotation_q1_30,
                        alignment,
                    )?,
                    lower_limit_bits: microradians_bits(joint.hard_minimum_microradians)?,
                    upper_limit_bits: microradians_bits(joint.hard_maximum_microradians)?,
                    max_velocity_bits: scaled_u64_bits(
                        joint.maximum_velocity_microradians_per_second,
                        1_000_000.0,
                    )?,
                });
            }
        }

        let mut joints = schema
            .joints
            .iter()
            .map(|joint| {
                Ok(PhysicsJointDescriptorV2 {
                    schema_version: PHYSICS_JOINT_DESCRIPTOR_V2_SCHEMA_VERSION,
                    base: PhysicsJointDescriptorV1 {
                        schema_version: PHYSICS_JOINT_DESCRIPTOR_V1_SCHEMA_VERSION,
                        joint_id: joint.joint_id.clone(),
                        articulation_id: articulation_id.clone(),
                        parent_body_id: body_ids[&joint.parent_body_id],
                        child_body_id: body_ids[&joint.child_body_id],
                        joint_kind: PhysicsJointKindV1::Revolute,
                        axis_q1_30: joint.axis_q1_30,
                        limit_min_microradians: joint.hard_minimum_microradians,
                        limit_max_microradians: joint.hard_maximum_microradians,
                        maximum_velocity_microradians_per_second: joint
                            .maximum_velocity_microradians_per_second,
                    },
                    anatomical_semantic_id: joint.anatomical_semantic_id.clone(),
                    parent_frame: physics_pose(joint.parent_frame)?,
                    child_frame: physics_pose(joint.child_frame)?,
                    soft_limit_min_microradians: joint.soft_minimum_microradians,
                    soft_limit_max_microradians: joint.soft_maximum_microradians,
                    neutral_position_microradians: joint.neutral_position_microradians,
                })
            })
            .collect::<Result<Vec<_>, MotorCompileError>>()?;
        joints.sort_by(|left, right| left.base.joint_id.cmp(&right.base.joint_id));
        let joint_by_id = joints
            .iter()
            .map(|joint| (joint.base.joint_id.clone(), joint))
            .collect::<BTreeMap<_, _>>();
        let mut actuator_definitions = schema.actuators.clone();
        actuator_definitions.sort_by(|left, right| left.actuator_id.cmp(&right.actuator_id));
        let actuators = actuator_definitions
            .iter()
            .map(|actuator| {
                let joint = joint_by_id
                    .get(&actuator.joint_id)
                    .ok_or(MotorCompileError::InvalidReference)?;
                Ok(PhysicsActuatorDescriptorV2 {
                    schema_version: PHYSICS_ACTUATOR_DESCRIPTOR_V2_SCHEMA_VERSION,
                    base: PhysicsActuatorDescriptorV1 {
                        schema_version: PHYSICS_ACTUATOR_DESCRIPTOR_V1_SCHEMA_VERSION,
                        actuator_id: actuator.actuator_id.clone(),
                        joint_id: actuator.joint_id.clone(),
                        neutral_position_microradians: joint.neutral_position_microradians,
                        limit_min_microradians: joint.base.limit_min_microradians,
                        limit_max_microradians: joint.base.limit_max_microradians,
                        maximum_effort_micronewton_metres: actuator
                            .maximum_effort_micronewton_metres
                            as u64,
                        maximum_effort_rate_micronewton_metres_per_second: actuator
                            .maximum_effort_rate_micronewton_metres_per_second,
                    },
                    stiffness_q16: actuator.stiffness_q16,
                    damping_q16: actuator.damping_q16,
                    minimum_effort_micronewton_metres: actuator.minimum_effort_micronewton_metres,
                    maximum_effort_micronewton_metres: actuator.maximum_effort_micronewton_metres,
                    maximum_power_microwatts: actuator.maximum_power_microwatts,
                    maximum_positive_work_microjoules_per_motor_tick: actuator
                        .maximum_positive_work_microjoules_per_motor_tick,
                    residual_scale_microradians: actuator.residual_scale_microradians,
                    minimum_target_delta_microradians_per_motor_tick: actuator
                        .minimum_target_delta_microradians_per_motor_tick,
                    maximum_target_delta_microradians_per_motor_tick: actuator
                        .maximum_target_delta_microradians_per_motor_tick,
                })
            })
            .collect::<Result<Vec<_>, MotorCompileError>>()?;
        let actuator_dof_ordinals = actuator_definitions
            .iter()
            .map(|actuator| {
                joint_dof_ordinals
                    .get(&actuator.joint_id)
                    .copied()
                    .ok_or(MotorCompileError::InvalidReference)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let effector_body_tokens = schema
            .effectors
            .iter()
            .map(|effector| {
                Ok((
                    effector.effector_id.clone(),
                    *body_tokens
                        .get(&effector.body_id)
                        .ok_or(MotorCompileError::InvalidReference)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, MotorCompileError>>()?;
        let mut collision_exclusions = schema
            .collision_exclusions
            .iter()
            .map(|exclusion| {
                let first = construction_order
                    .iter()
                    .position(|value| value == &exclusion.first_body_id)
                    .ok_or(MotorCompileError::InvalidReference)?;
                let second = construction_order
                    .iter()
                    .position(|value| value == &exclusion.second_body_id)
                    .ok_or(MotorCompileError::InvalidReference)?;
                let (first, second) = if first < second {
                    (first, second)
                } else {
                    (second, first)
                };
                Ok(ArticulationCollisionExclusionV2 {
                    first_link_index: u32::try_from(first)
                        .map_err(|_| MotorCompileError::Capacity)?,
                    second_link_index: u32::try_from(second)
                        .map_err(|_| MotorCompileError::Capacity)?,
                })
            })
            .collect::<Result<Vec<_>, MotorCompileError>>()?;
        collision_exclusions.sort_unstable();
        collision_exclusions.dedup();
        let mut physx_scene_profile = PhysXSceneProfile::deterministic_humanoid(
            4_096,
            u32::try_from(schema.bodies.len() + 1).map_err(|_| MotorCompileError::Capacity)?,
            u32::try_from(schema.joints.len()).map_err(|_| MotorCompileError::Capacity)?,
        );
        physx_scene_profile.position_iterations = 16;
        physx_scene_profile.velocity_iterations = 4;
        let physx_catalog = PhysXArticulationCatalogV2 {
            static_boxes: vec![StaticBoxInput {
                centre_bits: [0.0_f32.to_bits(), (-0.5_f32).to_bits(), 0.0_f32.to_bits()],
                half_extents_bits: [50.0_f32.to_bits(), 0.5_f32.to_bits(), 50.0_f32.to_bits()],
                user_token: 1,
            }],
            links: ffi_links,
            shapes: ffi_shapes,
            joints: ffi_joints,
            collision_exclusions,
        };
        let compiled_descriptor_hash = descriptor_hash(
            body_schema_hash,
            &construction_order,
            &body_tokens,
            &collider_tokens,
            physx_scene_profile,
            &physx_catalog,
        );
        let result = Self {
            body_schema_hash,
            compiled_descriptor_hash,
            construction_order,
            body_tokens,
            collider_tokens,
            collider_contact_roles,
            effector_body_tokens,
            joint_dof_ordinals,
            actuator_dof_ordinals,
            physics_descriptors: CompiledPhysicsDescriptorsV2 {
                bodies,
                joints,
                actuators,
            },
            physx_scene_profile,
            physx_catalog,
            actuator_definitions,
        };
        for body in result.physics_descriptors.bodies.values() {
            body.validate()?;
        }
        for joint in &result.physics_descriptors.joints {
            joint.validate()?;
        }
        for actuator in &result.physics_descriptors.actuators {
            actuator.validate()?;
        }
        Ok(result)
    }
}

fn topological_construction_order(
    schema: &BodySchemaV2,
) -> Result<Vec<SchemaId>, MotorCompileError> {
    let parents = schema
        .bodies
        .iter()
        .map(|body| (body.body_id.clone(), body.parent_body_id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut remaining = parents.keys().cloned().collect::<BTreeSet<_>>();
    let mut emitted = BTreeSet::new();
    let mut output = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next = remaining
            .iter()
            .find(|candidate| {
                parents[*candidate]
                    .as_ref()
                    .is_none_or(|parent| emitted.contains(parent))
            })
            .cloned()
            .ok_or(MotorCompileError::NonCanonicalTopology)?;
        remaining.remove(&next);
        emitted.insert(next.clone());
        output.push(next);
    }
    Ok(output)
}

fn descriptor_hash(
    body_schema_hash: ContentHash,
    construction_order: &[SchemaId],
    body_tokens: &BTreeMap<SchemaId, u64>,
    collider_tokens: &BTreeMap<SchemaId, u64>,
    scene_profile: PhysXSceneProfile,
    catalog: &PhysXArticulationCatalogV2,
) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.compiled-body-schema.v2\0");
    bytes.extend_from_slice(body_schema_hash.as_bytes());
    for value in scene_profile
        .gravity_bits
        .into_iter()
        .chain([scene_profile.timestep_bits])
        .chain([
            scene_profile.position_iterations,
            scene_profile.velocity_iterations,
            scene_profile.max_contacts,
            scene_profile.max_actors,
            scene_profile.max_joints,
        ])
    {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for id in construction_order {
        push_id_bytes(&mut bytes, id);
        bytes.extend_from_slice(&body_tokens[id].to_le_bytes());
    }
    for (id, token) in collider_tokens {
        push_id_bytes(&mut bytes, id);
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    for link in &catalog.links {
        bytes.extend_from_slice(&link.user_token.to_le_bytes());
        for value in [
            link.parent_link_index,
            link.first_shape_index,
            link.shape_count,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in link
            .position_bits
            .into_iter()
            .chain(link.rotation_bits)
            .chain(link.centre_of_mass_position_bits)
            .chain(link.centre_of_mass_rotation_bits)
            .chain([link.mass_bits])
            .chain(link.inertia_bits)
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    for shape in &catalog.shapes {
        bytes.extend_from_slice(&shape.user_token.to_le_bytes());
        for value in [shape.link_index, shape.shape_kind]
            .into_iter()
            .chain(shape.position_bits)
            .chain(shape.rotation_bits)
            .chain(shape.shape_dimensions_bits)
            .chain([
                shape.collision_layer,
                shape.collision_mask_low,
                shape.collision_mask_high,
            ])
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    for joint in &catalog.joints {
        for value in [joint.child_link_index]
            .into_iter()
            .chain(joint.parent_position_bits)
            .chain(joint.parent_rotation_bits)
            .chain(joint.child_position_bits)
            .chain(joint.child_rotation_bits)
            .chain([
                joint.lower_limit_bits,
                joint.upper_limit_bits,
                joint.max_velocity_bits,
            ])
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    for pair in &catalog.collision_exclusions {
        bytes.extend_from_slice(&pair.first_link_index.to_le_bytes());
        bytes.extend_from_slice(&pair.second_link_index.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn push_id_bytes(bytes: &mut Vec<u8>, id: &SchemaId) {
    let value = id.as_str().as_bytes();
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value);
}

fn physics_pose(value: BodyPoseV2) -> Result<PhysicsPoseV1, MotorCompileError> {
    let pose = PhysicsPoseV1 {
        translation_micrometres: value.translation_micrometres,
        rotation_q1_30: value.rotation_q1_30,
    };
    pose.validate()?;
    Ok(pose)
}

fn require_identity_rotation(pose: BodyPoseV2) -> Result<(), MotorCompileError> {
    if pose.rotation_q1_30 == [0, 0, 0, 1 << 30] {
        Ok(())
    } else {
        Err(MotorCompileError::UnsupportedBodyProfile)
    }
}

fn axis_alignment_quaternion_bits(axis: [i32; 3]) -> Result<[u32; 4], MotorCompileError> {
    let scale = (1_u64 << 30) as f64;
    let x = f64::from(axis[0]) / scale;
    let y = f64::from(axis[1]) / scale;
    let z = f64::from(axis[2]) / scale;
    let quaternion = if x < -0.999_999_999 {
        [0.0, 1.0, 0.0, 0.0]
    } else {
        let raw = [0.0, -z, y, 1.0 + x];
        let norm = raw
            .into_iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if !norm.is_finite() || norm <= 0.0 {
            return Err(MotorCompileError::UnsupportedJointAxis);
        }
        raw.map(|value| value / norm)
    };
    Ok(quaternion.map(|value| (value as f32).to_bits()))
}

fn compose_quaternion_bits(
    authored_q1_30: [i32; 4],
    alignment_bits: [u32; 4],
) -> Result<[u32; 4], MotorCompileError> {
    let scale = (1_u64 << 30) as f64;
    let [ax, ay, az, aw] = authored_q1_30.map(|value| f64::from(value) / scale);
    let [bx, by, bz, bw] = alignment_bits.map(|value| f64::from(f32::from_bits(value)));
    let mut output = [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ];
    let norm = output.iter().map(|value| value * value).sum::<f64>().sqrt();
    if !norm.is_finite() || norm <= 0.0 {
        return Err(MotorCompileError::UnsupportedJointAxis);
    }
    for value in &mut output {
        *value /= norm;
    }
    Ok(output.map(|value| (value as f32).to_bits()))
}

fn geometry_to_ffi(geometry: &PhysicsGeometryV1) -> Result<(u32, [u32; 3]), MotorCompileError> {
    match geometry {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => Ok((SHAPE_BOX, metres_bits(*half_extents_micrometres)?)),
        PhysicsGeometryV1::Sphere { radius_micrometres } => Ok((
            SHAPE_SPHERE,
            [
                micrometres_bits(*radius_micrometres)?,
                0.0_f32.to_bits(),
                0.0_f32.to_bits(),
            ],
        )),
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => Ok((
            SHAPE_CAPSULE,
            [
                micrometres_bits(*radius_micrometres)?,
                micrometres_bits(*half_segment_micrometres)?,
                0.0_f32.to_bits(),
            ],
        )),
        _ => Err(MotorCompileError::UnsupportedBodyProfile),
    }
}

fn checked_add_vector(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], MotorCompileError> {
    Ok([
        left[0]
            .checked_add(right[0])
            .ok_or(MotorCompileError::NumericOverflow)?,
        left[1]
            .checked_add(right[1])
            .ok_or(MotorCompileError::NumericOverflow)?,
        left[2]
            .checked_add(right[2])
            .ok_or(MotorCompileError::NumericOverflow)?,
    ])
}

fn metres_bits(values: [i64; 3]) -> Result<[u32; 3], MotorCompileError> {
    Ok([
        micrometres_bits(values[0])?,
        micrometres_bits(values[1])?,
        micrometres_bits(values[2])?,
    ])
}

fn micrometres_bits(value: i64) -> Result<u32, MotorCompileError> {
    scaled_i64_bits(value, 1_000_000.0)
}

fn microradians_bits(value: i64) -> Result<u32, MotorCompileError> {
    scaled_i64_bits(value, 1_000_000.0)
}

fn scaled_i64_bits(value: i64, divisor: f64) -> Result<u32, MotorCompileError> {
    finite_bits(value as f64 / divisor)
}

fn scaled_u64_bits(value: u64, divisor: f64) -> Result<u32, MotorCompileError> {
    finite_bits(value as f64 / divisor)
}

fn finite_bits(value: f64) -> Result<u32, MotorCompileError> {
    let value = value as f32;
    if value.is_finite() {
        Ok(value.to_bits())
    } else {
        Err(MotorCompileError::NumericOverflow)
    }
}

fn quaternion_bits(value: [i32; 4]) -> [u32; 4] {
    value.map(|value| (value as f32 / (1_u32 << 30) as f32).to_bits())
}

fn schema_id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned generated identifiers are valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BIOMECHANICS_HUMANOID_BODY_COUNT, BIOMECHANICS_HUMANOID_COLLIDER_COUNT,
        BIOMECHANICS_HUMANOID_DOF, biomechanics_humanoid_body_schema_v2,
    };

    #[test]
    fn frozen_biomechanics_profile_compiles_without_defaulted_rows() {
        let schema = biomechanics_humanoid_body_schema_v2();
        let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([9; 16]))
            .expect("compile frozen biomechanics profile");
        assert_eq!(
            compiled.construction_order.len(),
            BIOMECHANICS_HUMANOID_BODY_COUNT
        );
        assert_eq!(
            compiled.physx_catalog.shapes.len(),
            BIOMECHANICS_HUMANOID_COLLIDER_COUNT
        );
        assert_eq!(
            compiled.physx_catalog.joints.len(),
            BIOMECHANICS_HUMANOID_DOF
        );
        assert_eq!(
            compiled.actuator_definitions.len(),
            BIOMECHANICS_HUMANOID_DOF
        );
        assert_eq!(
            compiled.collider_tokens.len(),
            BIOMECHANICS_HUMANOID_COLLIDER_COUNT
        );
        for joint in &schema.joints {
            let dof = compiled.joint_dof_ordinals[&joint.joint_id] as usize;
            let ffi = compiled.physx_catalog.joints[dof];
            assert_eq!(
                ffi.child_link_index as usize,
                compiled
                    .construction_order
                    .iter()
                    .position(|body| body == &joint.child_body_id)
                    .expect("compiled child")
            );
            assert_eq!(
                ffi.parent_position_bits,
                metres_bits(joint.parent_frame.translation_micrometres).expect("frame")
            );
            assert_eq!(
                ffi.child_position_bits,
                metres_bits(joint.child_frame.translation_micrometres).expect("frame")
            );
            assert_eq!(
                ffi.lower_limit_bits,
                microradians_bits(joint.hard_minimum_microradians).expect("lower")
            );
            assert_eq!(
                ffi.upper_limit_bits,
                microradians_bits(joint.hard_maximum_microradians).expect("upper")
            );
            assert_axis_alignment(ffi.parent_rotation_bits, joint.axis_q1_30);
            assert_axis_alignment(ffi.child_rotation_bits, joint.axis_q1_30);
        }
        for body in &schema.bodies {
            let slot = compiled
                .construction_order
                .iter()
                .position(|candidate| candidate == &body.body_id)
                .expect("body slot");
            let descriptor = compiled
                .physics_descriptors
                .bodies
                .values()
                .find(|descriptor| descriptor.base.semantic_body_id == body.body_id)
                .expect("body descriptor");
            let ffi = compiled.physx_catalog.links[slot];
            assert_eq!(
                descriptor.base.mass_microkilograms,
                body.solver_mass_microkilograms
            );
            assert_eq!(
                descriptor.base.center_of_mass_micrometres,
                body.solver_center_of_mass_micrometres
            );
            assert_eq!(
                descriptor.base.inertia_microkilogram_metre_squared,
                body.solver_principal_inertia_microkilogram_metre_squared
            );
            assert_eq!(
                descriptor.authoritative_inertia_tensor_microkilogram_metre_squared,
                body.inertia_tensor_microkilogram_metre_squared
            );
            assert_eq!(descriptor.base.base.shapes.len(), body.colliders.len());
            assert_eq!(
                ffi.centre_of_mass_position_bits,
                metres_bits(body.solver_center_of_mass_micrometres).expect("CoM")
            );
            assert_eq!(
                ffi.mass_bits,
                scaled_u64_bits(body.solver_mass_microkilograms, 1_000_000.0).expect("mass")
            );
            assert_eq!(
                ffi.inertia_bits,
                body.solver_principal_inertia_microkilogram_metre_squared
                    .map(|value| scaled_u64_bits(value, 1_000_000.0).expect("inertia"))
            );
            let ffi_shapes = &compiled.physx_catalog.shapes[ffi.first_shape_index as usize
                ..ffi.first_shape_index as usize + ffi.shape_count as usize];
            for (shape_slot, (collider, ffi_shape)) in
                body.colliders.iter().zip(ffi_shapes).enumerate()
            {
                let descriptor_shape = descriptor
                    .base
                    .base
                    .shapes
                    .values()
                    .find(|shape| shape.shape_id.shape_slot == shape_slot as u32)
                    .expect("shape descriptor");
                assert_eq!(
                    descriptor_shape.local_pose,
                    physics_pose(collider.local_pose).unwrap()
                );
                assert_eq!(descriptor_shape.geometry, collider.geometry);
                assert_eq!(descriptor_shape.material_id, collider.material_id);
                assert_eq!(descriptor_shape.collision_layer, collider.collision_layer);
                assert_eq!(descriptor_shape.collision_mask, collider.collision_mask);
                assert_eq!(descriptor_shape.participation, collider.participation);
                assert_eq!(
                    descriptor_shape.contact_reporting,
                    collider.contact_reporting
                );
                let (expected_kind, expected_dimensions) =
                    geometry_to_ffi(&collider.geometry).expect("shape geometry");
                assert_eq!(ffi_shape.link_index, slot as u32);
                assert_eq!(ffi_shape.shape_kind, expected_kind);
                assert_eq!(ffi_shape.shape_dimensions_bits, expected_dimensions);
                assert_eq!(
                    ffi_shape.position_bits,
                    metres_bits(collider.local_pose.translation_micrometres).expect("shape pose")
                );
                assert_eq!(
                    ffi_shape.rotation_bits,
                    quaternion_bits(collider.local_pose.rotation_q1_30)
                );
                assert_eq!(
                    ffi_shape.collision_layer,
                    u32::from(collider.collision_layer)
                );
                assert_eq!(
                    u64::from(ffi_shape.collision_mask_low)
                        | (u64::from(ffi_shape.collision_mask_high) << 32),
                    collider.collision_mask
                );
                assert_eq!(
                    compiled.collider_contact_roles[&ffi_shape.user_token],
                    collider.contact_role
                );
            }
        }
    }

    #[test]
    fn declaration_permutation_preserves_descriptor_and_isaac_authority() {
        let source = biomechanics_humanoid_body_schema_v2();
        let mut permuted = source.clone();
        permuted.bodies.reverse();
        for body in &mut permuted.bodies {
            body.colliders.reverse();
        }
        permuted.mass_projection_groups.reverse();
        for group in &mut permuted.mass_projection_groups {
            group.members.reverse();
        }
        permuted.joints.reverse();
        permuted.actuators.reverse();
        permuted.effectors.reverse();
        permuted.symmetry_pairs.reverse();
        permuted.collision_exclusions.reverse();
        permuted.capability_ids.reverse();
        let permuted = permuted.canonicalize();
        let subject = PersistentId::from_bytes([13; 16]);
        let expected = CompiledBodySchemaV2::compile(&source, subject).expect("source");
        let actual = CompiledBodySchemaV2::compile(&permuted, subject).expect("permuted");
        assert_eq!(expected.body_schema_hash, actual.body_schema_hash);
        assert_eq!(
            expected.compiled_descriptor_hash,
            actual.compiled_descriptor_hash
        );
        assert_eq!(expected.construction_order, actual.construction_order);
        assert_eq!(expected.physx_catalog, actual.physx_catalog);
        assert_eq!(expected.physics_descriptors, actual.physics_descriptors);
    }

    #[test]
    #[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
    fn frozen_biomechanics_profile_builds_a_fresh_native_articulation() {
        use next_physics_physx::PhysXArticulationWorldV2;

        let compiled = CompiledBodySchemaV2::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([11; 16]),
        )
        .expect("compile frozen biomechanics profile");
        let mut world =
            PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
                .expect("create native biomechanics articulation");
        let snapshot = world.capture().expect("capture fresh articulation");
        assert_eq!(snapshot.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
        assert_eq!(snapshot.joints.len(), BIOMECHANICS_HUMANOID_DOF);
        let stepped = world
            .apply_efforts_and_step(&[0; BIOMECHANICS_HUMANOID_DOF])
            .expect("step neutral articulation");
        assert!(
            !stepped.contacts.is_empty(),
            "neutral soles must contact ground"
        );
        let mut sole_contact_count = 0;
        for contact in stepped.contacts {
            let shape_token = if contact.actor_a_token == 1 {
                Some(contact.shape_b_token)
            } else if contact.actor_b_token == 1 {
                Some(contact.shape_a_token)
            } else {
                None
            };
            let Some(shape_token) = shape_token else {
                assert!(
                    contact.separation_micrometres > 0
                        && contact.impulse_micronewton_seconds == [0; 3],
                    "neutral self pair must remain separated and non-supporting: separation={} impulse={:?}",
                    contact.separation_micrometres,
                    contact.impulse_micronewton_seconds,
                );
                continue;
            };
            let role = compiled.collider_contact_roles.get(&shape_token);
            if role == Some(&BodyContactRoleV2::FootWithSoleFeature) {
                sole_contact_count += 1;
            } else {
                assert!(
                    contact.separation_micrometres > 0
                        && contact.impulse_micronewton_seconds == [0; 3],
                    "non-sole neutral contact must remain a non-supporting speculative contact: {role:?} separation={} impulse={:?}",
                    contact.separation_micrometres,
                    contact.impulse_micronewton_seconds,
                );
            }
        }
        assert!(
            sole_contact_count > 0,
            "both feet define the support polygon"
        );
    }

    #[test]
    #[cfg(feature = "physx-sdk")]
    fn every_native_joint_limit_resists_outward_effort() {
        use next_physics_physx::PhysXArticulationWorldV2;

        let schema = biomechanics_humanoid_body_schema_v2();
        let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([17; 16]))
            .expect("compile");
        let mut sweep_profile = compiled.physx_scene_profile;
        sweep_profile.gravity_bits = [0.0_f32.to_bits(); 3];
        let mut sweep_catalog = compiled.physx_catalog.clone();
        sweep_catalog.static_boxes.clear();
        sweep_catalog.shapes.clear();
        for link in &mut sweep_catalog.links {
            link.first_shape_index = 0;
            link.shape_count = 0;
        }
        let mut target_by_dof = [0_i64; BIOMECHANICS_HUMANOID_DOF];
        let actuator_by_dof = compiled
            .actuator_definitions
            .iter()
            .zip(&compiled.actuator_dof_ordinals)
            .map(|(actuator, dof)| (*dof as usize, actuator))
            .collect::<BTreeMap<_, _>>();
        for joint in &schema.joints {
            let dof = compiled.joint_dof_ordinals[&joint.joint_id] as usize;
            for limit in [
                joint.hard_minimum_microradians,
                joint.hard_maximum_microradians,
            ] {
                target_by_dof.fill(0);
                let outward_direction = if limit == joint.hard_minimum_microradians {
                    -1
                } else {
                    1
                };
                target_by_dof[dof] = limit + outward_direction * 500_000;
                let mut world = PhysXArticulationWorldV2::create(sweep_profile, &sweep_catalog)
                    .expect("fresh sweep world");
                let mut snapshot = world.capture().expect("initial sweep state");
                let mut observed_minimum = snapshot.joints[dof].position_microradians;
                let mut observed_maximum = observed_minimum;
                for _ in 0..1_440 {
                    let efforts = snapshot
                        .joints
                        .iter()
                        .map(|state| {
                            let actuator = actuator_by_dof[&(state.ordinal as usize)];
                            let error =
                                target_by_dof[state.ordinal as usize] - state.position_microradians;
                            let numerator = i128::from(actuator.stiffness_q16) * i128::from(error)
                                - i128::from(actuator.damping_q16)
                                    * i128::from(state.velocity_microradians_per_second);
                            let effort = (numerator / 65_536) as i64;
                            effort.clamp(
                                actuator.minimum_effort_micronewton_metres,
                                actuator.maximum_effort_micronewton_metres,
                            )
                        })
                        .collect::<Vec<_>>();
                    snapshot = world
                        .apply_efforts_and_step(&efforts)
                        .expect("PD sweep step");
                    let position = snapshot.joints[dof].position_microradians;
                    observed_minimum = observed_minimum.min(position);
                    observed_maximum = observed_maximum.max(position);
                }
                let position = snapshot.joints[dof].position_microradians;
                assert!(
                    observed_minimum >= joint.hard_minimum_microradians - 10_000
                        && observed_maximum <= joint.hard_maximum_microradians + 10_000,
                    "{} crossed [{}, {}] during sweep: observed [{}, {}]",
                    joint.joint_id.as_str(),
                    joint.hard_minimum_microradians,
                    joint.hard_maximum_microradians,
                    observed_minimum,
                    observed_maximum,
                );
                assert!(
                    (position - limit).abs() <= 75_000,
                    "{} did not reach hard limit {}: final {}",
                    joint.joint_id.as_str(),
                    limit,
                    position,
                );
            }
        }
    }

    #[test]
    #[cfg(feature = "physx-sdk")]
    fn ten_thousand_native_reset_settle_cycles_are_exact_and_complete() {
        use next_physics_physx::PhysXArticulationWorldV2;

        let compiled = CompiledBodySchemaV2::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([19; 16]),
        )
        .expect("compile");
        let mut world =
            PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
                .expect("fresh reset world");
        let checkpoint = world.raw_checkpoint();
        let zero_efforts = [0; BIOMECHANICS_HUMANOID_DOF];
        let mut expected = world.restore(&checkpoint).expect("reference reset");
        for _ in 0..4 {
            expected = world
                .apply_efforts_and_step(&zero_efforts)
                .expect("reference settle step");
        }
        assert_eq!(expected.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
        assert_eq!(expected.joints.len(), BIOMECHANICS_HUMANOID_DOF);

        for cycle in 0..10_000 {
            let reset = world.restore(&checkpoint).expect("reset");
            assert_eq!(reset.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
            assert_eq!(reset.joints.len(), BIOMECHANICS_HUMANOID_DOF);
            let mut settled = reset;
            for _ in 0..4 {
                settled = world
                    .apply_efforts_and_step(&zero_efforts)
                    .expect("settle step");
            }
            assert_eq!(settled, expected, "reset/settle mismatch at cycle {cycle}");
        }
    }

    #[test]
    #[cfg(feature = "physx-sdk")]
    fn neutral_action_remains_bounded_for_sixty_simulated_seconds() {
        use next_physics_physx::PhysXArticulationWorldV2;

        let schema = biomechanics_humanoid_body_schema_v2();
        let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([23; 16]))
            .expect("compile");
        let hard_limits_by_dof = schema
            .joints
            .iter()
            .map(|joint| {
                (
                    compiled.joint_dof_ordinals[&joint.joint_id] as usize,
                    (
                        joint.hard_minimum_microradians,
                        joint.hard_maximum_microradians,
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let actuator_by_dof = compiled
            .actuator_definitions
            .iter()
            .zip(&compiled.actuator_dof_ordinals)
            .map(|(actuator, dof)| (*dof as usize, actuator))
            .collect::<BTreeMap<_, _>>();
        let mut world =
            PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
                .expect("fresh passive world");
        let mut snapshot = world.capture().expect("initial passive state");
        let mut previous_efforts = [0_i64; BIOMECHANICS_HUMANOID_DOF];
        let mut maximum_linear_speed_component = 0_i64;
        let mut maximum_angular_speed_component = 0_i64;
        let mut maximum_joint_speed = 0_i64;
        for step in 0..14_400 {
            let efforts = snapshot
                .joints
                .iter()
                .map(|state| {
                    let dof = state.ordinal as usize;
                    let actuator = actuator_by_dof[&dof];
                    let requested = (-(i128::from(actuator.stiffness_q16)
                        * i128::from(state.position_microradians)
                        + i128::from(actuator.damping_q16)
                            * i128::from(state.velocity_microradians_per_second)))
                        / 65_536;
                    let effort_limited = requested.clamp(
                        i128::from(actuator.minimum_effort_micronewton_metres),
                        i128::from(actuator.maximum_effort_micronewton_metres),
                    );
                    let maximum_delta =
                        actuator.maximum_effort_rate_micronewton_metres_per_second as i128 / 240;
                    let previous = i128::from(previous_efforts[dof]);
                    let rate_limited =
                        effort_limited.clamp(previous - maximum_delta, previous + maximum_delta);
                    previous_efforts[dof] = rate_limited as i64;
                    previous_efforts[dof]
                })
                .collect::<Vec<_>>();
            snapshot = world
                .apply_efforts_and_step(&efforts)
                .expect("passive step");
            assert_eq!(snapshot.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
            assert_eq!(snapshot.joints.len(), BIOMECHANICS_HUMANOID_DOF);
            for link in &snapshot.links {
                maximum_linear_speed_component = maximum_linear_speed_component.max(
                    link.linear_velocity_micrometres_per_second
                        .iter()
                        .map(|value| value.abs())
                        .max()
                        .unwrap_or(0),
                );
                maximum_angular_speed_component = maximum_angular_speed_component.max(
                    link.angular_velocity_microradians_per_second
                        .iter()
                        .map(|value| value.abs())
                        .max()
                        .unwrap_or(0),
                );
            }
            for joint in &snapshot.joints {
                let (minimum, maximum) = hard_limits_by_dof[&(joint.ordinal as usize)];
                assert!(
                    joint.position_microradians >= minimum - 10_000
                        && joint.position_microradians <= maximum + 10_000,
                    "joint {} crossed its hard ROM at step {step}: {} outside [{minimum}, {maximum}]",
                    joint.ordinal,
                    joint.position_microradians,
                );
                maximum_joint_speed =
                    maximum_joint_speed.max(joint.velocity_microradians_per_second.abs());
            }
        }
        assert!(
            maximum_linear_speed_component <= 20_000_000,
            "passive root/link speed was not bounded: {maximum_linear_speed_component} um/s"
        );
        assert!(
            maximum_angular_speed_component <= 50_000_000,
            "passive angular speed was not bounded: {maximum_angular_speed_component} urad/s"
        );
        assert!(
            maximum_joint_speed <= 50_000_000,
            "passive joint speed was not bounded: {maximum_joint_speed} urad/s"
        );
    }

    fn assert_axis_alignment(rotation_bits: [u32; 4], expected_q1_30: [i32; 3]) {
        let [x, y, z, w] = rotation_bits.map(|bits| f64::from(f32::from_bits(bits)));
        let rotated_x = [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
        ];
        let scale = (1_u64 << 30) as f64;
        for (actual, expected) in rotated_x
            .into_iter()
            .zip(expected_q1_30.map(|value| f64::from(value) / scale))
        {
            assert!((actual - expected).abs() <= 2.0e-7);
        }
    }
}
