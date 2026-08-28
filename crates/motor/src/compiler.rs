use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::body::{
    BODY_PROJECTION_COMPILER_PROFILE_ID_V1, BODY_PROJECTION_ROOTS_VERSION_V1,
    BodyActuatorDefinitionV1, BodyContractError, BodyInstanceProjectionV1, BodyProjectionRootsV1,
    BodySchemaV1, BodyV2ContractError,
};
use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::motor::{
    MOTOR_ACTION_LAYOUT_V1_SCHEMA_VERSION, MOTOR_OBSERVATION_LAYOUT_V1_SCHEMA_VERSION,
    MotorActionChannelV1, MotorActionLayoutV1, MotorActionSemanticV1, MotorContractError,
    MotorObservationChannelV1, MotorObservationLayoutV1, MotorObservationSemanticV1,
};
use next_contracts::physics::{
    PHYSICS_ACTUATOR_DESCRIPTOR_V1_SCHEMA_VERSION, PHYSICS_BODY_DESCRIPTOR_V2_SCHEMA_VERSION,
    PHYSICS_JOINT_DESCRIPTOR_V1_SCHEMA_VERSION, PhysicsActuatorDescriptorV1,
    PhysicsBodyDescriptorV1, PhysicsBodyDescriptorV2, PhysicsBodyIdV1, PhysicsContactReportingV1,
    PhysicsContractError, PhysicsGeometryV1, PhysicsJointDescriptorV1, PhysicsJointKindV1,
    PhysicsMotionKindV1, PhysicsParticipationV1, PhysicsPoseV1, PhysicsShapeDescriptorV1,
    PhysicsShapeIdV1,
};
use next_physics_physx::{PhysXArticulationCatalog, PhysXSceneProfile};
use next_physics_physx_ffi::{
    ArticulationJointInput, ArticulationLinkInput, NO_PARENT_LINK, SHAPE_BOX, SHAPE_CAPSULE,
    SHAPE_SPHERE, StaticBoxInput,
};

use crate::neutral_body_instance_projection_v1;

const ARTICULATION_ID: &str = "articulation.humanoid-stage0";
const HUMANOID_TOKEN_BASE: u64 = 1_000;
const FLAT_LOCOMOTION_OBSERVATION_LAYOUT_ID: &str =
    "motor-observation-layout.humanoid-flat-command.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledPhysicsDescriptorsV1 {
    pub bodies: BTreeMap<PhysicsBodyIdV1, PhysicsBodyDescriptorV2>,
    pub joints: Vec<PhysicsJointDescriptorV1>,
    pub actuators: Vec<PhysicsActuatorDescriptorV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBodySchemaV1 {
    pub subject_id: PersistentId,
    pub body_schema_hash: ContentHash,
    pub body_instance_projection_hash: ContentHash,
    pub projection_roots: BodyProjectionRootsV1,
    pub construction_order: Vec<SchemaId>,
    pub body_tokens: BTreeMap<SchemaId, u64>,
    pub effector_tokens: BTreeMap<SchemaId, u64>,
    pub joint_dof_ordinals: BTreeMap<SchemaId, u32>,
    pub actuator_dof_ordinals: Vec<u32>,
    pub physics_descriptors: CompiledPhysicsDescriptorsV1,
    pub physx_scene_profile: PhysXSceneProfile,
    pub physx_catalog: PhysXArticulationCatalog,
    pub observation_layout: MotorObservationLayoutV1,
    pub action_layout: MotorActionLayoutV1,
    pub actuator_definitions: Vec<BodyActuatorDefinitionV1>,
}

impl CompiledBodySchemaV1 {
    pub fn compile(
        schema: &BodySchemaV1,
        subject_id: PersistentId,
    ) -> Result<Self, MotorCompileError> {
        let projection = neutral_body_instance_projection_v1(schema, subject_id)?;
        Self::compile_projection(schema, &projection)
    }

    pub fn compile_projection(
        schema: &BodySchemaV1,
        projection: &BodyInstanceProjectionV1,
    ) -> Result<Self, MotorCompileError> {
        schema.validate()?;
        projection.validate_against(schema)?;
        let body_schema_hash = schema.schema_hash()?;
        let body_instance_projection_hash = projection.projection_hash()?;
        let subject_id = projection.subject_id;
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
        let mut body_tokens = BTreeMap::new();
        let mut body_ids = BTreeMap::new();
        let mut global_poses: BTreeMap<SchemaId, PhysicsPoseV1> = BTreeMap::new();
        let mut ffi_links = Vec::with_capacity(construction_order.len());
        let mut ffi_joints = Vec::with_capacity(schema.joints.len());
        let mut joint_dof_ordinals = BTreeMap::new();
        let mut bodies = BTreeMap::new();
        let articulation_id = schema_id(ARTICULATION_ID);

        for (ordinal, semantic_id) in construction_order.iter().enumerate() {
            let body = body_by_id
                .get(semantic_id)
                .copied()
                .ok_or(MotorCompileError::InvalidReference)?;
            require_identity_rotation(body.local_bind_pose)?;
            let parent_index = if let Some(parent) = &body.parent_body_id {
                construction_order[..ordinal]
                    .iter()
                    .position(|candidate| candidate == parent)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or(MotorCompileError::NonCanonicalTopology)?
            } else {
                NO_PARENT_LINK
            };
            let parent_translation = body.parent_body_id.as_ref().map_or([0; 3], |parent| {
                global_poses[parent].translation_micrometres
            });
            let global_translation = checked_add_vector(
                parent_translation,
                body.local_bind_pose.translation_micrometres,
            )?;
            let global_pose = PhysicsPoseV1 {
                translation_micrometres: global_translation,
                rotation_q1_30: body.local_bind_pose.rotation_q1_30,
            };
            global_poses.insert(semantic_id.clone(), global_pose);
            let token = HUMANOID_TOKEN_BASE
                .checked_add(u64::try_from(ordinal).map_err(|_| MotorCompileError::Capacity)?)
                .ok_or(MotorCompileError::Capacity)?;
            body_tokens.insert(semantic_id.clone(), token);
            let body_id = PhysicsBodyIdV1 {
                subject_id,
                body_slot: u32::try_from(ordinal).map_err(|_| MotorCompileError::Capacity)?,
            };
            body_ids.insert(semantic_id.clone(), body_id);
            let collider = body
                .colliders
                .first()
                .ok_or(MotorCompileError::UnsupportedBodyProfile)?;
            require_identity_rotation(collider.local_pose)?;
            if collider.local_pose.translation_micrometres != [0; 3] || body.colliders.len() != 1 {
                return Err(MotorCompileError::UnsupportedBodyProfile);
            }
            let (shape_kind, shape_dimensions_bits) = geometry_to_ffi(&collider.geometry)?;
            ffi_links.push(ArticulationLinkInput {
                user_token: token,
                parent_link_index: parent_index,
                shape_kind,
                position_bits: metres_bits(global_translation)?,
                rotation_bits: quaternion_bits(global_pose.rotation_q1_30),
                shape_dimensions_bits,
                mass_bits: scaled_u64_bits(body.mass_microkilograms, 1_000_000.0)?,
                inertia_bits: [
                    scaled_u64_bits(body.inertia_microkilogram_metre_squared[0], 1_000_000.0)?,
                    scaled_u64_bits(body.inertia_microkilogram_metre_squared[1], 1_000_000.0)?,
                    scaled_u64_bits(body.inertia_microkilogram_metre_squared[2], 1_000_000.0)?,
                ],
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            });

            let shape_id = PhysicsShapeIdV1 {
                body_id,
                shape_slot: 0,
            };
            let shape = PhysicsShapeDescriptorV1 {
                shape_id,
                descriptor_revision: 1,
                local_pose: collider.local_pose,
                geometry: collider.geometry.clone(),
                material_id: collider.material_id.clone(),
                collision_layer: 1,
                collision_mask: u64::MAX,
                participation: PhysicsParticipationV1::Solid,
                contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
            };
            let base = PhysicsBodyDescriptorV1 {
                body_id,
                descriptor_revision: 1,
                motion_kind: PhysicsMotionKindV1::Dynamic,
                initial_pose: global_pose,
                initial_linear_velocity_micrometres_per_second: [0; 3],
                initial_angular_velocity_q16: [0; 3],
                active: true,
                shapes: BTreeMap::from([(shape_id, shape)]),
            };
            bodies.insert(
                body_id,
                PhysicsBodyDescriptorV2 {
                    schema_version: PHYSICS_BODY_DESCRIPTOR_V2_SCHEMA_VERSION,
                    semantic_body_id: semantic_id.clone(),
                    base,
                    mass_microkilograms: body.mass_microkilograms,
                    center_of_mass_micrometres: body.center_of_mass_micrometres,
                    inertia_microkilogram_metre_squared: body.inertia_microkilogram_metre_squared,
                    articulation_id: Some(articulation_id.clone()),
                },
            );

            if ordinal != 0 {
                let joint = joint_by_child
                    .get(semantic_id)
                    .copied()
                    .ok_or(MotorCompileError::InvalidReference)?;
                if joint.axis_q1_30 != [1 << 30, 0, 0] {
                    return Err(MotorCompileError::UnsupportedJointAxis);
                }
                require_identity_rotation(joint.parent_frame)?;
                require_identity_rotation(joint.child_frame)?;
                joint_dof_ordinals.insert(
                    joint.joint_id.clone(),
                    u32::try_from(ffi_joints.len()).map_err(|_| MotorCompileError::Capacity)?,
                );
                ffi_joints.push(ArticulationJointInput {
                    child_link_index: u32::try_from(ordinal)
                        .map_err(|_| MotorCompileError::Capacity)?,
                    reserved: 0,
                    parent_position_bits: metres_bits(joint.parent_frame.translation_micrometres)?,
                    parent_rotation_bits: quaternion_bits(joint.parent_frame.rotation_q1_30),
                    child_position_bits: metres_bits(joint.child_frame.translation_micrometres)?,
                    child_rotation_bits: quaternion_bits(joint.child_frame.rotation_q1_30),
                    lower_limit_bits: microradians_bits(joint.limit_min_microradians)?,
                    upper_limit_bits: microradians_bits(joint.limit_max_microradians)?,
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
                Ok(PhysicsJointDescriptorV1 {
                    schema_version: PHYSICS_JOINT_DESCRIPTOR_V1_SCHEMA_VERSION,
                    joint_id: joint.joint_id.clone(),
                    articulation_id: articulation_id.clone(),
                    parent_body_id: body_ids[&joint.parent_body_id],
                    child_body_id: body_ids[&joint.child_body_id],
                    joint_kind: PhysicsJointKindV1::Revolute,
                    axis_q1_30: joint.axis_q1_30,
                    limit_min_microradians: joint.limit_min_microradians,
                    limit_max_microradians: joint.limit_max_microradians,
                    maximum_velocity_microradians_per_second: joint
                        .maximum_velocity_microradians_per_second,
                })
            })
            .collect::<Result<Vec<_>, MotorCompileError>>()?;
        joints.sort();
        let joint_by_id = joints
            .iter()
            .map(|joint| (joint.joint_id.clone(), joint))
            .collect::<BTreeMap<_, _>>();
        let mut actuator_definitions = schema.actuators.clone();
        actuator_definitions.sort_by(|left, right| left.actuator_id.cmp(&right.actuator_id));
        let actuators = actuator_definitions
            .iter()
            .map(|actuator| {
                let joint = joint_by_id
                    .get(&actuator.joint_id)
                    .ok_or(MotorCompileError::InvalidReference)?;
                Ok(PhysicsActuatorDescriptorV1 {
                    schema_version: PHYSICS_ACTUATOR_DESCRIPTOR_V1_SCHEMA_VERSION,
                    actuator_id: actuator.actuator_id.clone(),
                    joint_id: actuator.joint_id.clone(),
                    neutral_position_microradians: actuator.neutral_position_microradians,
                    limit_min_microradians: joint.limit_min_microradians,
                    limit_max_microradians: joint.limit_max_microradians,
                    maximum_effort_micronewton_metres: actuator.maximum_effort_micronewton_metres,
                    maximum_effort_rate_micronewton_metres_per_second: actuator
                        .maximum_effort_rate_micronewton_metres_per_second,
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
        let (observation_layout, action_layout) =
            compile_layouts(schema, body_schema_hash, &actuator_definitions)?;
        let effector_tokens = schema
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
        let physx_scene_profile = PhysXSceneProfile::deterministic_humanoid(
            2_048,
            u32::try_from(schema.bodies.len() + 1).map_err(|_| MotorCompileError::Capacity)?,
            u32::try_from(schema.joints.len()).map_err(|_| MotorCompileError::Capacity)?,
        );
        let physx_catalog = PhysXArticulationCatalog {
            static_boxes: vec![StaticBoxInput {
                centre_bits: [0.0_f32.to_bits(), (-0.5_f32).to_bits(), 0.0_f32.to_bits()],
                half_extents_bits: [50.0_f32.to_bits(), 0.5_f32.to_bits(), 50.0_f32.to_bits()],
                user_token: 1,
            }],
            links: ffi_links,
            joints: ffi_joints,
        };
        let physics_descriptors = CompiledPhysicsDescriptorsV1 {
            bodies,
            joints,
            actuators,
        };
        let projection_roots = compile_projection_roots(
            body_schema_hash,
            body_instance_projection_hash,
            &physics_descriptors,
            &observation_layout,
            &action_layout,
            &actuator_definitions,
        )?;
        let result = Self {
            subject_id,
            body_schema_hash,
            body_instance_projection_hash,
            projection_roots,
            construction_order,
            body_tokens,
            effector_tokens,
            joint_dof_ordinals,
            actuator_dof_ordinals,
            physics_descriptors,
            physx_scene_profile,
            physx_catalog,
            observation_layout,
            action_layout,
            actuator_definitions,
        };
        result.observation_layout.validate()?;
        result.action_layout.validate()?;
        for body in result.physics_descriptors.bodies.values() {
            body.validate()?;
        }
        for joint in &result.physics_descriptors.joints {
            joint.validate()?;
        }
        for actuator in &result.physics_descriptors.actuators {
            actuator.validate()?;
        }
        result.projection_roots.validate()?;
        Ok(result)
    }

    pub fn apply_flat_locomotion_profile(&mut self) -> Result<(), MotorCompileError> {
        let ground = self
            .physx_catalog
            .static_boxes
            .first_mut()
            .ok_or(MotorCompileError::UnsupportedBodyProfile)?;
        ground.half_extents_bits = [100.0_f32.to_bits(), 0.5_f32.to_bits(), 100.0_f32.to_bits()];
        self.observation_layout.layout_id = schema_id(FLAT_LOCOMOTION_OBSERVATION_LAYOUT_ID);
        let mut commands = self
            .observation_layout
            .channels
            .iter_mut()
            .filter(|channel| channel.semantic == MotorObservationSemanticV1::Command);
        let right = commands
            .next()
            .ok_or(MotorCompileError::UnsupportedBodyProfile)?;
        right.source_id = schema_id("command.local-right-velocity");
        right.minimum_raw = -2_000_000;
        right.maximum_raw = 2_000_000;
        let forward = commands
            .next()
            .ok_or(MotorCompileError::UnsupportedBodyProfile)?;
        forward.source_id = schema_id("command.local-forward-velocity");
        forward.minimum_raw = -1_500_000;
        forward.maximum_raw = 3_000_000;
        let yaw = commands
            .next()
            .ok_or(MotorCompileError::UnsupportedBodyProfile)?;
        yaw.source_id = schema_id("command.yaw-rate");
        yaw.minimum_raw = -1_500_000;
        yaw.maximum_raw = 1_500_000;
        if commands.next().is_some() {
            return Err(MotorCompileError::UnsupportedBodyProfile);
        }
        self.observation_layout.validate()?;
        self.projection_roots.observation_layout_hash = self.observation_layout.layout_hash()?;
        self.projection_roots.validate()?;
        Ok(())
    }
}

fn topological_construction_order(
    schema: &BodySchemaV1,
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

fn compile_layouts(
    schema: &BodySchemaV1,
    body_schema_hash: ContentHash,
    actuators: &[BodyActuatorDefinitionV1],
) -> Result<(MotorObservationLayoutV1, MotorActionLayoutV1), MotorCompileError> {
    let root = schema
        .bodies
        .iter()
        .find(|body| body.parent_body_id.is_none())
        .ok_or(MotorCompileError::NonCanonicalTopology)?;
    let mut observation_channels = Vec::new();
    let mut push_observation = |semantic: MotorObservationSemanticV1,
                                source_id: SchemaId,
                                minimum_raw: i64,
                                maximum_raw: i64| {
        let ordinal = observation_channels.len();
        observation_channels.push(MotorObservationChannelV1 {
            channel_id: schema_id(&format!("motor-observation.{ordinal:04}")),
            semantic,
            source_id,
            minimum_raw,
            maximum_raw,
            scale_numerator: 1,
            scale_denominator: 1,
        });
    };
    for axis in 0..4 {
        push_observation(
            MotorObservationSemanticV1::RootOrientation,
            schema_id(&format!("{}.rotation-{axis}", root.body_id.as_str())),
            -(1 << 30),
            1 << 30,
        );
    }
    for axis in 0..3 {
        push_observation(
            MotorObservationSemanticV1::RootLinearVelocity,
            schema_id(&format!("{}.linear-velocity-{axis}", root.body_id.as_str())),
            -20_000_000,
            20_000_000,
        );
    }
    for axis in 0..3 {
        push_observation(
            MotorObservationSemanticV1::RootAngularVelocity,
            schema_id(&format!(
                "{}.angular-velocity-{axis}",
                root.body_id.as_str()
            )),
            -50_000_000,
            50_000_000,
        );
    }
    for joint in &schema.joints {
        push_observation(
            MotorObservationSemanticV1::JointPosition,
            joint.joint_id.clone(),
            joint.limit_min_microradians,
            joint.limit_max_microradians,
        );
    }
    for joint in &schema.joints {
        push_observation(
            MotorObservationSemanticV1::JointVelocity,
            joint.joint_id.clone(),
            -(joint.maximum_velocity_microradians_per_second as i64),
            joint.maximum_velocity_microradians_per_second as i64,
        );
    }
    for actuator in actuators {
        push_observation(
            MotorObservationSemanticV1::PreviousAction,
            actuator.actuator_id.clone(),
            -1_500_000,
            1_500_000,
        );
    }
    for source in [
        "command.velocity-x",
        "command.velocity-z",
        "command.heading",
    ] {
        push_observation(
            MotorObservationSemanticV1::Command,
            schema_id(source),
            -10_000_000,
            10_000_000,
        );
    }
    for effector in &schema.effectors {
        push_observation(
            MotorObservationSemanticV1::Contact,
            effector.effector_id.clone(),
            0,
            1,
        );
    }
    let observation_layout = MotorObservationLayoutV1 {
        schema_version: MOTOR_OBSERVATION_LAYOUT_V1_SCHEMA_VERSION,
        layout_id: schema_id("motor-observation-layout.humanoid-stage0.v1"),
        body_schema_hash,
        channels: observation_channels,
    };
    let action_layout = MotorActionLayoutV1 {
        schema_version: MOTOR_ACTION_LAYOUT_V1_SCHEMA_VERSION,
        layout_id: schema_id("motor-action-layout.humanoid-stage0.v1"),
        body_schema_hash,
        actuator_profile_hash: body_schema_hash,
        channels: actuators
            .iter()
            .enumerate()
            .map(|(ordinal, actuator)| MotorActionChannelV1 {
                channel_id: schema_id(&format!("motor-action.{ordinal:04}")),
                semantic: MotorActionSemanticV1::ResidualJointPosition,
                actuator_id: actuator.actuator_id.clone(),
                minimum_raw: -1_000_000,
                maximum_raw: 1_000_000,
                scale_numerator: 1,
                scale_denominator: 1,
            })
            .collect(),
    };
    Ok((observation_layout, action_layout))
}

#[must_use]
pub fn body_projection_compiler_profile_hash_v1() -> ContentHash {
    let mut bytes = b"nextengine.body-projection-compiler-profile.v1\0".to_vec();
    bytes.extend_from_slice(BODY_PROJECTION_COMPILER_PROFILE_ID_V1.as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn compile_projection_roots(
    body_schema_hash: ContentHash,
    body_instance_projection_hash: ContentHash,
    physics: &CompiledPhysicsDescriptorsV1,
    observation_layout: &MotorObservationLayoutV1,
    action_layout: &MotorActionLayoutV1,
    actuator_definitions: &[BodyActuatorDefinitionV1],
) -> Result<BodyProjectionRootsV1, MotorCompileError> {
    let value = BodyProjectionRootsV1 {
        schema_version: BODY_PROJECTION_ROOTS_VERSION_V1,
        body_schema_hash,
        body_instance_projection_hash,
        compiler_profile_hash: body_projection_compiler_profile_hash_v1(),
        physics_descriptor_root: physics_descriptor_root(physics)?,
        observation_layout_hash: observation_layout.layout_hash()?,
        action_layout_hash: action_layout.layout_hash()?,
        actuator_safety_root: actuator_safety_root(actuator_definitions, action_layout)?,
    };
    value.validate()?;
    Ok(value)
}

fn physics_descriptor_root(
    descriptors: &CompiledPhysicsDescriptorsV1,
) -> Result<ContentHash, MotorCompileError> {
    let mut bytes = b"nextengine.compiled-physics-descriptors.v1\0".to_vec();
    push_len(&mut bytes, descriptors.bodies.len())?;
    for (body_id, body) in &descriptors.bodies {
        bytes.extend_from_slice(&body_id.canonical_key_bytes());
        bytes.extend_from_slice(&body.schema_version.to_le_bytes());
        push_schema_id(&mut bytes, &body.semantic_body_id)?;
        bytes.extend_from_slice(&body.base.descriptor_revision.to_le_bytes());
        bytes.push(body.base.motion_kind as u8);
        push_pose(&mut bytes, body.base.initial_pose);
        push_i64_3(
            &mut bytes,
            body.base.initial_linear_velocity_micrometres_per_second,
        );
        push_i64_3(&mut bytes, body.base.initial_angular_velocity_q16);
        bytes.push(u8::from(body.base.active));
        push_len(&mut bytes, body.base.shapes.len())?;
        for (shape_id, shape) in &body.base.shapes {
            bytes.extend_from_slice(&shape_id.canonical_key_bytes());
            bytes.extend_from_slice(&shape.descriptor_revision.to_le_bytes());
            push_pose(&mut bytes, shape.local_pose);
            push_geometry(&mut bytes, &shape.geometry)?;
            push_schema_id(&mut bytes, &shape.material_id)?;
            bytes.push(shape.collision_layer);
            bytes.extend_from_slice(&shape.collision_mask.to_le_bytes());
            bytes.push(shape.participation as u8);
            bytes.push(shape.contact_reporting as u8);
        }
        bytes.extend_from_slice(&body.mass_microkilograms.to_le_bytes());
        push_i64_3(&mut bytes, body.center_of_mass_micrometres);
        for value in body.inertia_microkilogram_metre_squared {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        match &body.articulation_id {
            Some(id) => {
                bytes.push(1);
                push_schema_id(&mut bytes, id)?;
            }
            None => bytes.push(0),
        }
    }
    push_len(&mut bytes, descriptors.joints.len())?;
    for joint in &descriptors.joints {
        bytes.extend_from_slice(&joint.schema_version.to_le_bytes());
        push_schema_id(&mut bytes, &joint.joint_id)?;
        push_schema_id(&mut bytes, &joint.articulation_id)?;
        bytes.extend_from_slice(&joint.parent_body_id.canonical_key_bytes());
        bytes.extend_from_slice(&joint.child_body_id.canonical_key_bytes());
        bytes.push(joint.joint_kind as u8);
        for value in joint.axis_q1_30 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&joint.limit_min_microradians.to_le_bytes());
        bytes.extend_from_slice(&joint.limit_max_microradians.to_le_bytes());
        bytes.extend_from_slice(&joint.maximum_velocity_microradians_per_second.to_le_bytes());
    }
    push_len(&mut bytes, descriptors.actuators.len())?;
    for actuator in &descriptors.actuators {
        bytes.extend_from_slice(&actuator.schema_version.to_le_bytes());
        push_schema_id(&mut bytes, &actuator.actuator_id)?;
        push_schema_id(&mut bytes, &actuator.joint_id)?;
        bytes.extend_from_slice(&actuator.neutral_position_microradians.to_le_bytes());
        bytes.extend_from_slice(&actuator.limit_min_microradians.to_le_bytes());
        bytes.extend_from_slice(&actuator.limit_max_microradians.to_le_bytes());
        bytes.extend_from_slice(&actuator.maximum_effort_micronewton_metres.to_le_bytes());
        bytes.extend_from_slice(
            &actuator
                .maximum_effort_rate_micronewton_metres_per_second
                .to_le_bytes(),
        );
    }
    Ok(content_hash_from_bytes(sha256(&bytes)))
}

fn actuator_safety_root(
    actuators: &[BodyActuatorDefinitionV1],
    action_layout: &MotorActionLayoutV1,
) -> Result<ContentHash, MotorCompileError> {
    let mut bytes = b"nextengine.compiled-actuator-safety.v1\0".to_vec();
    bytes.extend_from_slice(action_layout.layout_hash()?.as_bytes());
    push_len(&mut bytes, actuators.len())?;
    for actuator in actuators {
        push_schema_id(&mut bytes, &actuator.actuator_id)?;
        push_schema_id(&mut bytes, &actuator.joint_id)?;
        bytes.extend_from_slice(&actuator.neutral_position_microradians.to_le_bytes());
        bytes.extend_from_slice(&actuator.stiffness_q16.to_le_bytes());
        bytes.extend_from_slice(&actuator.damping_q16.to_le_bytes());
        bytes.extend_from_slice(&actuator.maximum_effort_micronewton_metres.to_le_bytes());
        bytes.extend_from_slice(
            &actuator
                .maximum_effort_rate_micronewton_metres_per_second
                .to_le_bytes(),
        );
    }
    Ok(content_hash_from_bytes(sha256(&bytes)))
}

fn push_len(bytes: &mut Vec<u8>, length: usize) -> Result<(), MotorCompileError> {
    bytes.extend_from_slice(
        &u32::try_from(length)
            .map_err(|_| MotorCompileError::Capacity)?
            .to_le_bytes(),
    );
    Ok(())
}

fn push_schema_id(bytes: &mut Vec<u8>, id: &SchemaId) -> Result<(), MotorCompileError> {
    push_len(bytes, id.as_str().len())?;
    bytes.extend_from_slice(id.as_str().as_bytes());
    Ok(())
}

fn push_pose(bytes: &mut Vec<u8>, pose: PhysicsPoseV1) {
    push_i64_3(bytes, pose.translation_micrometres);
    for value in pose.rotation_q1_30 {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_i64_3(bytes: &mut Vec<u8>, values: [i64; 3]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_geometry(
    bytes: &mut Vec<u8>,
    geometry: &PhysicsGeometryV1,
) -> Result<(), MotorCompileError> {
    match geometry {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => {
            bytes.push(1);
            push_i64_3(bytes, *half_extents_micrometres);
        }
        PhysicsGeometryV1::Sphere { radius_micrometres } => {
            bytes.push(2);
            bytes.extend_from_slice(&radius_micrometres.to_le_bytes());
        }
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => {
            bytes.push(3);
            bytes.extend_from_slice(&radius_micrometres.to_le_bytes());
            bytes.extend_from_slice(&half_segment_micrometres.to_le_bytes());
        }
        _ => return Err(MotorCompileError::UnsupportedBodyProfile),
    }
    Ok(())
}

fn require_identity_rotation(pose: PhysicsPoseV1) -> Result<(), MotorCompileError> {
    if pose.rotation_q1_30 == [0, 0, 0, 1 << 30] {
        Ok(())
    } else {
        Err(MotorCompileError::UnsupportedBodyProfile)
    }
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
    [
        (value[0] as f32 / (1_u32 << 30) as f32).to_bits(),
        (value[1] as f32 / (1_u32 << 30) as f32).to_bits(),
        (value[2] as f32 / (1_u32 << 30) as f32).to_bits(),
        (value[3] as f32 / (1_u32 << 30) as f32).to_bits(),
    ]
}

fn schema_id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned generated identifiers are valid")
}

#[derive(Debug)]
pub enum MotorCompileError {
    Body(BodyContractError),
    BodyV2(BodyV2ContractError),
    Physics(PhysicsContractError),
    Motor(MotorContractError),
    InvalidReference,
    NonCanonicalTopology,
    UnsupportedBodyProfile,
    UnsupportedMaterialProfile,
    UnsupportedJointAxis,
    NumericOverflow,
    Capacity,
}

impl MotorCompileError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Body(_) => "MOTOR_BODY_SCHEMA_INVALID",
            Self::BodyV2(_) => "MOTOR_BODY_SCHEMA_V2_INVALID",
            Self::Physics(_) => "MOTOR_PHYSICS_DESCRIPTOR_INVALID",
            Self::Motor(_) => "MOTOR_LAYOUT_INVALID",
            Self::InvalidReference => "MOTOR_BODY_REFERENCE_INVALID",
            Self::NonCanonicalTopology => "MOTOR_BODY_TOPOLOGY_NON_CANONICAL",
            Self::UnsupportedBodyProfile => "MOTOR_BODY_PROFILE_UNSUPPORTED",
            Self::UnsupportedMaterialProfile => "MOTOR_MATERIAL_PROFILE_UNSUPPORTED",
            Self::UnsupportedJointAxis => "MOTOR_JOINT_AXIS_UNSUPPORTED",
            Self::NumericOverflow => "MOTOR_NUMERIC_OVERFLOW",
            Self::Capacity => "MOTOR_CAPACITY_EXCEEDED",
        }
    }
}

impl Display for MotorCompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorCompileError {}

impl From<BodyContractError> for MotorCompileError {
    fn from(value: BodyContractError) -> Self {
        Self::Body(value)
    }
}

impl From<BodyV2ContractError> for MotorCompileError {
    fn from(value: BodyV2ContractError) -> Self {
        Self::BodyV2(value)
    }
}

impl From<PhysicsContractError> for MotorCompileError {
    fn from(value: PhysicsContractError) -> Self {
        Self::Physics(value)
    }
}

impl From<MotorContractError> for MotorCompileError {
    fn from(value: MotorContractError) -> Self {
        Self::Motor(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        REFERENCE_HUMANOID_DOF, neutral_body_instance_projection_v1,
        reference_humanoid_body_schema_v1,
    };

    #[test]
    fn source_record_permutation_has_identical_projection_hashes_and_order() {
        let source = reference_humanoid_body_schema_v1();
        let mut permuted = source.clone();
        permuted.bodies.reverse();
        permuted.joints.reverse();
        permuted.actuators.reverse();
        permuted.effectors.reverse();
        permuted.symmetry_pairs.reverse();
        permuted.capability_ids.reverse();
        let permuted = permuted.canonicalize();
        let subject = PersistentId::from_bytes([7; 16]);
        let first = CompiledBodySchemaV1::compile(&source, subject).expect("first compile");
        let second = CompiledBodySchemaV1::compile(&permuted, subject).expect("second compile");
        assert_eq!(first.body_schema_hash, second.body_schema_hash);
        assert_eq!(first.projection_roots, second.projection_roots);
        assert_eq!(first.construction_order, second.construction_order);
        assert_eq!(first.physx_catalog, second.physx_catalog);
        assert_eq!(
            first
                .observation_layout
                .layout_hash()
                .expect("observation hash"),
            second
                .observation_layout
                .layout_hash()
                .expect("observation hash")
        );
        assert_eq!(
            first.action_layout.layout_hash().expect("action hash"),
            second.action_layout.layout_hash().expect("action hash")
        );
        assert_eq!(first.physx_catalog.joints.len(), REFERENCE_HUMANOID_DOF);
        assert_eq!(first.action_layout.channels.len(), REFERENCE_HUMANOID_DOF);
    }

    #[test]
    fn two_subjects_share_tensor_and_safety_roots_but_keep_distinct_instance_physics() {
        let schema = reference_humanoid_body_schema_v1();
        let player = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0x54; 16]))
            .expect("player projection");
        let npc = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0x59; 16]))
            .expect("npc projection");
        assert_eq!(player.body_schema_hash, npc.body_schema_hash);
        assert_eq!(
            player.projection_roots.observation_layout_hash,
            npc.projection_roots.observation_layout_hash
        );
        assert_eq!(
            player.projection_roots.action_layout_hash,
            npc.projection_roots.action_layout_hash
        );
        assert_eq!(
            player.projection_roots.actuator_safety_root,
            npc.projection_roots.actuator_safety_root
        );
        assert_ne!(
            player.body_instance_projection_hash,
            npc.body_instance_projection_hash
        );
        assert_ne!(
            player.projection_roots.physics_descriptor_root,
            npc.projection_roots.physics_descriptor_root
        );
        assert_ne!(
            player
                .projection_roots
                .projection_root()
                .expect("player root"),
            npc.projection_roots.projection_root().expect("npc root")
        );
    }

    #[test]
    fn mismatched_projection_rejects_before_any_compiled_value_is_published() {
        let schema = reference_humanoid_body_schema_v1();
        let mut projection =
            neutral_body_instance_projection_v1(&schema, PersistentId::from_bytes([7; 16]))
                .expect("projection");
        projection.body_schema_revision += 1;
        assert!(matches!(
            CompiledBodySchemaV1::compile_projection(&schema, &projection),
            Err(MotorCompileError::Body(
                BodyContractError::ProjectionMismatch
            ))
        ));
    }
}
