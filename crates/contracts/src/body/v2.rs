use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::sha256;
use crate::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use crate::physics::{PhysicsContactReportingV1, PhysicsGeometryV1, PhysicsParticipationV1};

pub const BODY_SCHEMA_VERSION_V2: u16 = 2;
pub const MAX_BODY_SCHEMA_V2_BODIES: usize = 128;
pub const MAX_BODY_SCHEMA_V2_JOINTS: usize = 256;
pub const MAX_BODY_SCHEMA_V2_ACTUATORS: usize = 256;
pub const MAX_BODY_SCHEMA_V2_EFFECTORS: usize = 128;
pub const MAX_BODY_SCHEMA_V2_MAPPING_GROUPS: usize = 128;
pub const MAX_BODY_SCHEMA_V2_EXCLUSIONS: usize = 1024;
const Q30_ONE: i128 = 1_i128 << 30;
const Q60_ONE: i128 = 1_i128 << 60;
const MICRO_SCALE_SQUARED: i128 = 1_000_000_000_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BodySemanticRoleV2 {
    PhysicalRoot = 1,
    PhysicalTorsoHead = 2,
    PhysicalThigh = 3,
    PhysicalShankKnee = 4,
    PhysicalTalus = 5,
    PhysicalFoot = 6,
    PhysicalUpperArm = 7,
    PhysicalForearmHand = 8,
    NonCollidingCarrier = 9,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BodyContactRoleV2 {
    PelvisGround = 1,
    TorsoGround = 2,
    HeadGround = 3,
    ThighGround = 4,
    ShankGround = 5,
    KneeGround = 6,
    AnkleGround = 7,
    FootWithSoleFeature = 8,
    UpperArmGround = 9,
    ForearmGround = 10,
    HandGround = 11,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(i8)]
pub enum BodyMirrorValueRuleV2 {
    Negated = -1,
    Preserved = 1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyPoseV2 {
    pub translation_micrometres: [i64; 3],
    pub rotation_q1_30: [i32; 4],
}

impl Default for BodyPoseV2 {
    fn default() -> Self {
        Self {
            translation_micrometres: [0; 3],
            rotation_q1_30: [0, 0, 0, 1 << 30],
        }
    }
}

impl BodyPoseV2 {
    fn validate(self, norm_tolerance_q2_60: u64) -> Result<(), BodyV2ContractError> {
        validate_q30_unit(self.rotation_q1_30, norm_tolerance_q2_60, true)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyColliderDefinitionV2 {
    pub collider_id: SchemaId,
    pub local_pose: BodyPoseV2,
    pub geometry: PhysicsGeometryV1,
    pub material_id: SchemaId,
    pub collision_layer: u8,
    pub collision_mask: u64,
    pub participation: PhysicsParticipationV1,
    pub contact_reporting: PhysicsContactReportingV1,
    pub contact_role: BodyContactRoleV2,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyDefinitionV2 {
    pub body_id: SchemaId,
    pub parent_body_id: Option<SchemaId>,
    pub local_bind_pose: BodyPoseV2,
    pub semantic_role: BodySemanticRoleV2,
    pub mapping_group_id: SchemaId,
    pub mass_microkilograms: u64,
    pub center_of_mass_micrometres: [i64; 3],
    pub inertia_tensor_microkilogram_metre_squared: [i64; 6],
    pub solver_mass_microkilograms: u64,
    pub solver_center_of_mass_micrometres: [i64; 3],
    pub solver_principal_inertia_microkilogram_metre_squared: [u64; 3],
    pub solver_principal_frame: BodyPoseV2,
    pub solver_tensor_error_max_microkilogram_metre_squared: u64,
    pub colliders: Vec<BodyColliderDefinitionV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyMassProjectionMemberV2 {
    pub body_id: SchemaId,
    pub body_origin_in_group_micrometres: [i64; 3],
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyMassProjectionGroupV2 {
    pub mapping_group_id: SchemaId,
    pub source_mass_microkilograms: u64,
    pub source_center_of_mass_micrometres: [i64; 3],
    pub source_inertia_tensor_microkilogram_metre_squared: [i64; 6],
    pub maximum_first_moment_error_microkilogram_micrometres: u64,
    pub maximum_inertia_error_microkilogram_metre_squared: u64,
    pub members: Vec<BodyMassProjectionMemberV2>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyJointDefinitionV2 {
    pub joint_id: SchemaId,
    pub parent_body_id: SchemaId,
    pub child_body_id: SchemaId,
    pub anatomical_semantic_id: SchemaId,
    pub parent_frame: BodyPoseV2,
    pub child_frame: BodyPoseV2,
    pub axis_q1_30: [i32; 3],
    pub hard_minimum_microradians: i64,
    pub hard_maximum_microradians: i64,
    pub soft_minimum_microradians: i64,
    pub soft_maximum_microradians: i64,
    pub neutral_position_microradians: i64,
    pub maximum_velocity_microradians_per_second: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyActuatorDefinitionV2 {
    pub actuator_id: SchemaId,
    pub joint_id: SchemaId,
    pub stiffness_q16: u64,
    pub damping_q16: u64,
    pub minimum_effort_micronewton_metres: i64,
    pub maximum_effort_micronewton_metres: i64,
    pub maximum_effort_rate_micronewton_metres_per_second: u64,
    pub maximum_power_microwatts: u64,
    pub maximum_positive_work_microjoules_per_motor_tick: u64,
    pub residual_scale_microradians: u64,
    pub minimum_target_delta_microradians_per_motor_tick: i64,
    pub maximum_target_delta_microradians_per_motor_tick: i64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyEffectorDefinitionV2 {
    pub effector_id: SchemaId,
    pub body_id: SchemaId,
    pub local_pose: BodyPoseV2,
    pub semantic_role_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodySymmetryPairV2 {
    pub left_id: SchemaId,
    pub right_id: SchemaId,
    pub value_rule: BodyMirrorValueRuleV2,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyCollisionExclusionV2 {
    pub first_body_id: SchemaId,
    pub second_body_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodySchemaV2 {
    pub schema_version: u16,
    pub schema_id: SchemaId,
    pub schema_revision: u32,
    pub family_id: SchemaId,
    pub coordinate_profile_hash: ContentHash,
    pub source_provenance_hash: ContentHash,
    pub solver_projection_profile_hash: ContentHash,
    pub rotation_norm_tolerance_q2_60: u64,
    pub bodies: Vec<BodyDefinitionV2>,
    pub mass_projection_groups: Vec<BodyMassProjectionGroupV2>,
    pub joints: Vec<BodyJointDefinitionV2>,
    pub actuators: Vec<BodyActuatorDefinitionV2>,
    pub effectors: Vec<BodyEffectorDefinitionV2>,
    pub symmetry_pairs: Vec<BodySymmetryPairV2>,
    pub collision_exclusions: Vec<BodyCollisionExclusionV2>,
    pub capability_ids: Vec<SchemaId>,
}

impl BodySchemaV2 {
    #[must_use]
    pub fn canonicalize(mut self) -> Self {
        for body in &mut self.bodies {
            body.colliders
                .sort_by(|left, right| left.collider_id.cmp(&right.collider_id));
        }
        for group in &mut self.mass_projection_groups {
            group
                .members
                .sort_by(|left, right| left.body_id.cmp(&right.body_id));
        }
        for exclusion in &mut self.collision_exclusions {
            if exclusion.first_body_id > exclusion.second_body_id {
                std::mem::swap(&mut exclusion.first_body_id, &mut exclusion.second_body_id);
            }
        }
        self.bodies
            .sort_by(|left, right| left.body_id.cmp(&right.body_id));
        self.mass_projection_groups
            .sort_by(|left, right| left.mapping_group_id.cmp(&right.mapping_group_id));
        self.joints
            .sort_by(|left, right| left.joint_id.cmp(&right.joint_id));
        self.actuators
            .sort_by(|left, right| left.actuator_id.cmp(&right.actuator_id));
        self.effectors
            .sort_by(|left, right| left.effector_id.cmp(&right.effector_id));
        self.symmetry_pairs.sort();
        self.collision_exclusions.sort();
        self.capability_ids.sort();
        self
    }

    pub fn validate(&self) -> Result<(), BodyV2ContractError> {
        if self.schema_version != BODY_SCHEMA_VERSION_V2 {
            return Err(BodyV2ContractError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.schema_revision == 0
            || self.rotation_norm_tolerance_q2_60 == 0
            || self.rotation_norm_tolerance_q2_60 > (1_u64 << 42)
            || self.bodies.is_empty()
            || self.bodies.len() > MAX_BODY_SCHEMA_V2_BODIES
            || self.mass_projection_groups.is_empty()
            || self.mass_projection_groups.len() > MAX_BODY_SCHEMA_V2_MAPPING_GROUPS
            || self.joints.len() > MAX_BODY_SCHEMA_V2_JOINTS
            || self.actuators.len() > MAX_BODY_SCHEMA_V2_ACTUATORS
            || self.effectors.len() > MAX_BODY_SCHEMA_V2_EFFECTORS
            || self.collision_exclusions.len() > MAX_BODY_SCHEMA_V2_EXCLUSIONS
        {
            return Err(BodyV2ContractError::InvalidBounds);
        }
        require_strict_order(self.bodies.iter().map(|value| &value.body_id))?;
        require_strict_order(
            self.mass_projection_groups
                .iter()
                .map(|value| &value.mapping_group_id),
        )?;
        require_strict_order(self.joints.iter().map(|value| &value.joint_id))?;
        require_strict_order(self.actuators.iter().map(|value| &value.actuator_id))?;
        require_strict_order(self.effectors.iter().map(|value| &value.effector_id))?;
        require_strict_order(self.capability_ids.iter())?;
        require_strict_struct_order(&self.symmetry_pairs)?;
        require_strict_struct_order(&self.collision_exclusions)?;

        let body_by_id = self
            .bodies
            .iter()
            .map(|body| (body.body_id.clone(), body))
            .collect::<BTreeMap<_, _>>();
        let body_ids = body_by_id.keys().cloned().collect::<BTreeSet<_>>();
        let roots = self
            .bodies
            .iter()
            .filter(|body| body.parent_body_id.is_none())
            .count();
        if roots != 1 || self.joints.len() + 1 != self.bodies.len() {
            return Err(BodyV2ContractError::InvalidTopology);
        }
        let mut parents = BTreeMap::new();
        let mut collider_ids = BTreeSet::new();
        for body in &self.bodies {
            body.local_bind_pose
                .validate(self.rotation_norm_tolerance_q2_60)?;
            body.solver_principal_frame
                .validate(self.rotation_norm_tolerance_q2_60)?;
            if body.solver_principal_frame.translation_micrometres != [0; 3]
                || body.mass_microkilograms == 0
                || body.solver_mass_microkilograms == 0
                || body.mass_microkilograms != body.solver_mass_microkilograms
                || body.center_of_mass_micrometres != body.solver_center_of_mass_micrometres
                || body
                    .solver_principal_inertia_microkilogram_metre_squared
                    .contains(&0)
            {
                return Err(BodyV2ContractError::InvalidPhysicalParameter);
            }
            validate_inertia(body.inertia_tensor_microkilogram_metre_squared)?;
            validate_solver_tensor(body, self.rotation_norm_tolerance_q2_60)?;
            match body.semantic_role {
                BodySemanticRoleV2::NonCollidingCarrier if !body.colliders.is_empty() => {
                    return Err(BodyV2ContractError::InvalidCollider);
                }
                BodySemanticRoleV2::NonCollidingCarrier => {}
                _ if body.colliders.is_empty() => {
                    return Err(BodyV2ContractError::InvalidCollider);
                }
                _ => {}
            }
            if let Some(parent) = &body.parent_body_id {
                if parent == &body.body_id || !body_ids.contains(parent) {
                    return Err(BodyV2ContractError::InvalidTopology);
                }
                parents.insert(body.body_id.clone(), parent.clone());
            }
            require_strict_order(body.colliders.iter().map(|value| &value.collider_id))?;
            for collider in &body.colliders {
                if !collider_ids.insert(collider.collider_id.clone())
                    || collider.collision_layer >= 64
                    || collider.collision_mask == 0
                    || collider.participation == PhysicsParticipationV1::QueryOnly
                    || collider.contact_reporting == PhysicsContactReportingV1::Disabled
                {
                    return Err(BodyV2ContractError::InvalidCollider);
                }
                collider
                    .local_pose
                    .validate(self.rotation_norm_tolerance_q2_60)?;
                validate_geometry(&collider.geometry)?;
            }
        }
        for body_id in &body_ids {
            let mut cursor = body_id;
            let mut visited = BTreeSet::new();
            while let Some(parent) = parents.get(cursor) {
                if !visited.insert(cursor.clone()) {
                    return Err(BodyV2ContractError::InvalidTopology);
                }
                cursor = parent;
            }
        }

        validate_mapping_groups(self, &body_by_id)?;

        let mut child_ids = BTreeSet::new();
        let joint_ids = self
            .joints
            .iter()
            .map(|joint| joint.joint_id.clone())
            .collect::<BTreeSet<_>>();
        for joint in &self.joints {
            if !body_ids.contains(&joint.parent_body_id)
                || !body_ids.contains(&joint.child_body_id)
                || joint.parent_body_id == joint.child_body_id
                || !child_ids.insert(joint.child_body_id.clone())
                || parents.get(&joint.child_body_id) != Some(&joint.parent_body_id)
                || joint.hard_minimum_microradians >= joint.hard_maximum_microradians
                || joint.soft_minimum_microradians < joint.hard_minimum_microradians
                || joint.soft_maximum_microradians > joint.hard_maximum_microradians
                || joint.soft_minimum_microradians > joint.neutral_position_microradians
                || joint.neutral_position_microradians > joint.soft_maximum_microradians
                || joint.maximum_velocity_microradians_per_second == 0
            {
                return Err(BodyV2ContractError::InvalidJoint);
            }
            joint
                .parent_frame
                .validate(self.rotation_norm_tolerance_q2_60)?;
            joint
                .child_frame
                .validate(self.rotation_norm_tolerance_q2_60)?;
            validate_q30_unit(joint.axis_q1_30, self.rotation_norm_tolerance_q2_60, false)?;
        }
        let mut actuator_joints = BTreeSet::new();
        for actuator in &self.actuators {
            if !joint_ids.contains(&actuator.joint_id)
                || !actuator_joints.insert(actuator.joint_id.clone())
                || actuator.stiffness_q16 == 0
                || actuator.damping_q16 == 0
                || actuator.minimum_effort_micronewton_metres >= 0
                || actuator.maximum_effort_micronewton_metres <= 0
                || actuator.maximum_effort_rate_micronewton_metres_per_second == 0
                || actuator.maximum_power_microwatts == 0
                || actuator.maximum_positive_work_microjoules_per_motor_tick == 0
                || actuator.residual_scale_microradians == 0
                || actuator.minimum_target_delta_microradians_per_motor_tick >= 0
                || actuator.maximum_target_delta_microradians_per_motor_tick <= 0
            {
                return Err(BodyV2ContractError::InvalidActuator);
            }
        }
        if actuator_joints != joint_ids {
            return Err(BodyV2ContractError::InvalidActuator);
        }
        for effector in &self.effectors {
            if !body_ids.contains(&effector.body_id) {
                return Err(BodyV2ContractError::InvalidReference);
            }
            effector
                .local_pose
                .validate(self.rotation_norm_tolerance_q2_60)?;
        }
        let semantic_ids = body_ids
            .iter()
            .chain(joint_ids.iter())
            .chain(self.actuators.iter().map(|value| &value.actuator_id))
            .chain(self.effectors.iter().map(|value| &value.effector_id))
            .collect::<BTreeSet<_>>();
        let mut symmetry_members = BTreeSet::new();
        for pair in &self.symmetry_pairs {
            if pair.left_id == pair.right_id
                || !semantic_ids.contains(&pair.left_id)
                || !semantic_ids.contains(&pair.right_id)
                || !symmetry_members.insert(pair.left_id.clone())
                || !symmetry_members.insert(pair.right_id.clone())
            {
                return Err(BodyV2ContractError::InvalidSymmetry);
            }
        }
        for exclusion in &self.collision_exclusions {
            if exclusion.first_body_id >= exclusion.second_body_id
                || !body_ids.contains(&exclusion.first_body_id)
                || !body_ids.contains(&exclusion.second_body_id)
            {
                return Err(BodyV2ContractError::InvalidReference);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BodyV2ContractError> {
        self.validate()?;
        let mut out = Vec::new();
        push_bytes(&mut out, b"nextengine.body-schema.v2\0")?;
        push_u16(&mut out, self.schema_version);
        push_id(&mut out, &self.schema_id)?;
        push_u32(&mut out, self.schema_revision);
        push_id(&mut out, &self.family_id)?;
        for hash in [
            self.coordinate_profile_hash,
            self.source_provenance_hash,
            self.solver_projection_profile_hash,
        ] {
            out.extend_from_slice(hash.as_bytes());
        }
        push_u64(&mut out, self.rotation_norm_tolerance_q2_60);
        push_sequence(&mut out, &self.bodies, encode_body)?;
        push_sequence(&mut out, &self.mass_projection_groups, encode_mapping_group)?;
        push_sequence(&mut out, &self.joints, encode_joint)?;
        push_sequence(&mut out, &self.actuators, encode_actuator)?;
        push_sequence(&mut out, &self.effectors, encode_effector)?;
        push_sequence(&mut out, &self.symmetry_pairs, |value, bytes| {
            push_id(bytes, &value.left_id)?;
            push_id(bytes, &value.right_id)?;
            bytes.push(value.value_rule as i8 as u8);
            Ok(())
        })?;
        push_sequence(&mut out, &self.collision_exclusions, |value, bytes| {
            push_id(bytes, &value.first_body_id)?;
            push_id(bytes, &value.second_body_id)
        })?;
        push_sequence(&mut out, &self.capability_ids, |value, bytes| {
            push_id(bytes, value)
        })?;
        Ok(out)
    }

    pub fn schema_hash(&self) -> Result<ContentHash, BodyV2ContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyV2ContractError {
    UnsupportedSchemaVersion(u16),
    InvalidBounds,
    NonCanonicalOrder,
    InvalidTopology,
    InvalidPhysicalParameter,
    InvalidInertia,
    InvalidMapping,
    InvalidCollider,
    InvalidJoint,
    InvalidActuator,
    InvalidSymmetry,
    InvalidReference,
    UnsupportedGeometry,
    ArithmeticOverflow,
    LengthOverflow,
    Physics(crate::physics::PhysicsContractError),
}

impl BodyV2ContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion(_) => "UNSUPPORTED_BODY_SCHEMA_V2_VERSION",
            Self::InvalidBounds => "BODY_SCHEMA_V2_BOUNDS_INVALID",
            Self::NonCanonicalOrder => "BODY_SCHEMA_V2_ORDER_INVALID",
            Self::InvalidTopology => "BODY_SCHEMA_V2_TOPOLOGY_INVALID",
            Self::InvalidPhysicalParameter => "BODY_SCHEMA_V2_PHYSICAL_PARAMETER_INVALID",
            Self::InvalidInertia => "BODY_SCHEMA_V2_INERTIA_INVALID",
            Self::InvalidMapping => "BODY_SCHEMA_V2_MAPPING_INVALID",
            Self::InvalidCollider => "BODY_SCHEMA_V2_COLLIDER_INVALID",
            Self::InvalidJoint => "BODY_SCHEMA_V2_JOINT_INVALID",
            Self::InvalidActuator => "BODY_SCHEMA_V2_ACTUATOR_INVALID",
            Self::InvalidSymmetry => "BODY_SCHEMA_V2_SYMMETRY_INVALID",
            Self::InvalidReference => "BODY_SCHEMA_V2_REFERENCE_INVALID",
            Self::UnsupportedGeometry => "BODY_SCHEMA_V2_GEOMETRY_UNSUPPORTED",
            Self::ArithmeticOverflow => "BODY_SCHEMA_V2_ARITHMETIC_OVERFLOW",
            Self::LengthOverflow => "BODY_SCHEMA_V2_LENGTH_OVERFLOW",
            Self::Physics(_) => "BODY_SCHEMA_V2_PHYSICS_VALUE_INVALID",
        }
    }
}

impl Display for BodyV2ContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for BodyV2ContractError {}

impl From<crate::physics::PhysicsContractError> for BodyV2ContractError {
    fn from(value: crate::physics::PhysicsContractError) -> Self {
        Self::Physics(value)
    }
}

fn validate_mapping_groups(
    schema: &BodySchemaV2,
    body_by_id: &BTreeMap<SchemaId, &BodyDefinitionV2>,
) -> Result<(), BodyV2ContractError> {
    let mut mapped_bodies = BTreeSet::new();
    for group in &schema.mass_projection_groups {
        if group.source_mass_microkilograms == 0
            || group.maximum_first_moment_error_microkilogram_micrometres == 0
            || group.maximum_inertia_error_microkilogram_metre_squared == 0
            || group.members.is_empty()
        {
            return Err(BodyV2ContractError::InvalidMapping);
        }
        validate_inertia(group.source_inertia_tensor_microkilogram_metre_squared)?;
        require_strict_order(group.members.iter().map(|value| &value.body_id))?;
        let mut mass = 0_u64;
        let mut first_moment = [0_i128; 3];
        let mut inertia_numerator = [0_i128; 6];
        for member in &group.members {
            let body = body_by_id
                .get(&member.body_id)
                .copied()
                .ok_or(BodyV2ContractError::InvalidMapping)?;
            if body.mapping_group_id != group.mapping_group_id
                || !mapped_bodies.insert(body.body_id.clone())
            {
                return Err(BodyV2ContractError::InvalidMapping);
            }
            mass = mass
                .checked_add(body.mass_microkilograms)
                .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
            let position = checked_add_i64_3(
                member.body_origin_in_group_micrometres,
                body.center_of_mass_micrometres,
            )?;
            for index in 0..3 {
                first_moment[index] = first_moment[index]
                    .checked_add(i128::from(body.mass_microkilograms) * i128::from(position[index]))
                    .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
            }
            let delta = checked_sub_i64_3(position, group.source_center_of_mass_micrometres)?;
            let parallel = parallel_axis_numerator(body.mass_microkilograms, delta)?;
            for index in 0..6 {
                inertia_numerator[index] = inertia_numerator[index]
                    .checked_add(
                        i128::from(body.inertia_tensor_microkilogram_metre_squared[index])
                            .checked_mul(MICRO_SCALE_SQUARED)
                            .ok_or(BodyV2ContractError::ArithmeticOverflow)?,
                    )
                    .and_then(|value| value.checked_add(parallel[index]))
                    .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
            }
        }
        if mass != group.source_mass_microkilograms {
            return Err(BodyV2ContractError::InvalidMapping);
        }
        for (index, actual) in first_moment.iter().enumerate() {
            let expected = i128::from(group.source_mass_microkilograms)
                .checked_mul(i128::from(group.source_center_of_mass_micrometres[index]))
                .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
            if abs_diff_i128(*actual, expected)
                > u128::from(group.maximum_first_moment_error_microkilogram_micrometres)
            {
                return Err(BodyV2ContractError::InvalidMapping);
            }
        }
        for (index, numerator) in inertia_numerator.iter().enumerate() {
            let mapped = divide_ties_even(*numerator, MICRO_SCALE_SQUARED)?;
            if abs_diff_i128(
                mapped,
                i128::from(group.source_inertia_tensor_microkilogram_metre_squared[index]),
            ) > u128::from(group.maximum_inertia_error_microkilogram_metre_squared)
            {
                return Err(BodyV2ContractError::InvalidMapping);
            }
        }
    }
    if mapped_bodies.len() != schema.bodies.len() {
        return Err(BodyV2ContractError::InvalidMapping);
    }
    Ok(())
}

fn validate_solver_tensor(
    body: &BodyDefinitionV2,
    norm_tolerance_q2_60: u64,
) -> Result<(), BodyV2ContractError> {
    body.solver_principal_frame.validate(norm_tolerance_q2_60)?;
    let rotation = rotation_matrix_q1_30(body.solver_principal_frame.rotation_q1_30)?;
    let mut reconstructed = [0_i128; 6];
    for (slot, (row, column)) in [(0, 0), (0, 1), (0, 2), (1, 1), (1, 2), (2, 2)]
        .into_iter()
        .enumerate()
    {
        let mut numerator = 0_i128;
        for (axis, principal_inertia) in body
            .solver_principal_inertia_microkilogram_metre_squared
            .iter()
            .enumerate()
        {
            numerator = numerator
                .checked_add(
                    i128::from(*principal_inertia)
                        .checked_mul(i128::from(rotation[row][axis]))
                        .and_then(|value| value.checked_mul(i128::from(rotation[column][axis])))
                        .ok_or(BodyV2ContractError::ArithmeticOverflow)?,
                )
                .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
        }
        reconstructed[slot] = divide_ties_even(numerator, Q60_ONE)?;
    }
    for (actual, expected) in reconstructed
        .into_iter()
        .zip(body.inertia_tensor_microkilogram_metre_squared)
    {
        if abs_diff_i128(actual, i128::from(expected))
            > u128::from(body.solver_tensor_error_max_microkilogram_metre_squared)
        {
            return Err(BodyV2ContractError::InvalidInertia);
        }
    }
    Ok(())
}

fn validate_inertia(inertia: [i64; 6]) -> Result<(), BodyV2ContractError> {
    let [xx, xy, xz, yy, yz, zz] = inertia.map(i128::from);
    let leading_two = xx
        .checked_mul(yy)
        .and_then(|value| value.checked_sub(xy * xy))
        .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
    let determinant = xx
        .checked_mul(yy * zz - yz * yz)
        .and_then(|value| value.checked_sub(xy * (xy * zz - yz * xz)))
        .and_then(|value| value.checked_add(xz * (xy * yz - yy * xz)))
        .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
    if xx <= 0 || yy <= 0 || zz <= 0 || leading_two <= 0 || determinant <= 0 {
        return Err(BodyV2ContractError::InvalidInertia);
    }
    Ok(())
}

fn validate_q30_unit<const N: usize>(
    values: [i32; N],
    tolerance: u64,
    canonical_sign: bool,
) -> Result<(), BodyV2ContractError> {
    let norm = values.into_iter().try_fold(0_i128, |sum, value| {
        let value = i128::from(value);
        sum.checked_add(value * value)
    });
    if norm.is_none_or(|norm| abs_diff_i128(norm, Q60_ONE) > u128::from(tolerance)) {
        return Err(BodyV2ContractError::InvalidPhysicalParameter);
    }
    if canonical_sign
        && values
            .into_iter()
            .find(|value| *value != 0)
            .is_none_or(|value| value < 0)
    {
        return Err(BodyV2ContractError::InvalidPhysicalParameter);
    }
    Ok(())
}

fn rotation_matrix_q1_30(quaternion: [i32; 4]) -> Result<[[i64; 3]; 3], BodyV2ContractError> {
    let [x, y, z, w] = quaternion.map(i128::from);
    let two = 2_i128;
    let entries = [
        Q30_ONE - divide_ties_even(two * (y * y + z * z), Q30_ONE)?,
        divide_ties_even(two * (x * y - z * w), Q30_ONE)?,
        divide_ties_even(two * (x * z + y * w), Q30_ONE)?,
        divide_ties_even(two * (x * y + z * w), Q30_ONE)?,
        Q30_ONE - divide_ties_even(two * (x * x + z * z), Q30_ONE)?,
        divide_ties_even(two * (y * z - x * w), Q30_ONE)?,
        divide_ties_even(two * (x * z - y * w), Q30_ONE)?,
        divide_ties_even(two * (y * z + x * w), Q30_ONE)?,
        Q30_ONE - divide_ties_even(two * (x * x + y * y), Q30_ONE)?,
    ];
    let mut output = [[0_i64; 3]; 3];
    for (index, value) in entries.into_iter().enumerate() {
        output[index / 3][index % 3] =
            i64::try_from(value).map_err(|_| BodyV2ContractError::ArithmeticOverflow)?;
    }
    Ok(output)
}

fn parallel_axis_numerator(
    mass_microkilograms: u64,
    delta_micrometres: [i64; 3],
) -> Result<[i128; 6], BodyV2ContractError> {
    let [x, y, z] = delta_micrometres.map(i128::from);
    let mass = i128::from(mass_microkilograms);
    [
        y * y + z * z,
        -x * y,
        -x * z,
        x * x + z * z,
        -y * z,
        x * x + y * y,
    ]
    .map(|value| {
        mass.checked_mul(value)
            .ok_or(BodyV2ContractError::ArithmeticOverflow)
    })
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .try_into()
    .map_err(|_| BodyV2ContractError::ArithmeticOverflow)
}

fn divide_ties_even(numerator: i128, denominator: i128) -> Result<i128, BodyV2ContractError> {
    if denominator <= 0 {
        return Err(BodyV2ContractError::ArithmeticOverflow);
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let twice = remainder
        .unsigned_abs()
        .checked_mul(2)
        .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
    let denominator_unsigned = denominator as u128;
    let increment = twice > denominator_unsigned
        || (twice == denominator_unsigned && quotient.unsigned_abs() % 2 == 1);
    if !increment {
        return Ok(quotient);
    }
    quotient
        .checked_add(if numerator >= 0 { 1 } else { -1 })
        .ok_or(BodyV2ContractError::ArithmeticOverflow)
}

fn checked_add_i64_3(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], BodyV2ContractError> {
    let mut output = [0_i64; 3];
    for index in 0..3 {
        output[index] = left[index]
            .checked_add(right[index])
            .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
    }
    Ok(output)
}

fn checked_sub_i64_3(left: [i64; 3], right: [i64; 3]) -> Result<[i64; 3], BodyV2ContractError> {
    let mut output = [0_i64; 3];
    for index in 0..3 {
        output[index] = left[index]
            .checked_sub(right[index])
            .ok_or(BodyV2ContractError::ArithmeticOverflow)?;
    }
    Ok(output)
}

fn abs_diff_i128(left: i128, right: i128) -> u128 {
    left.abs_diff(right)
}

fn validate_geometry(value: &PhysicsGeometryV1) -> Result<(), BodyV2ContractError> {
    match value {
        PhysicsGeometryV1::Box { .. }
        | PhysicsGeometryV1::Sphere { .. }
        | PhysicsGeometryV1::Capsule { .. } => value.validate().map_err(Into::into),
        _ => Err(BodyV2ContractError::UnsupportedGeometry),
    }
}

fn require_strict_order<'a>(
    values: impl Iterator<Item = &'a SchemaId>,
) -> Result<(), BodyV2ContractError> {
    let mut previous: Option<&SchemaId> = None;
    for value in values {
        if previous.is_some_and(|previous| previous >= value) {
            return Err(BodyV2ContractError::NonCanonicalOrder);
        }
        previous = Some(value);
    }
    Ok(())
}

fn require_strict_struct_order<T: Ord>(values: &[T]) -> Result<(), BodyV2ContractError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(BodyV2ContractError::NonCanonicalOrder);
    }
    Ok(())
}

fn encode_body(value: &BodyDefinitionV2, out: &mut Vec<u8>) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.body_id)?;
    encode_optional_id(out, value.parent_body_id.as_ref())?;
    encode_pose(out, value.local_bind_pose);
    out.push(value.semantic_role as u8);
    push_id(out, &value.mapping_group_id)?;
    push_u64(out, value.mass_microkilograms);
    encode_i64_3(out, value.center_of_mass_micrometres);
    encode_i64_6(out, value.inertia_tensor_microkilogram_metre_squared);
    push_u64(out, value.solver_mass_microkilograms);
    encode_i64_3(out, value.solver_center_of_mass_micrometres);
    for item in value.solver_principal_inertia_microkilogram_metre_squared {
        push_u64(out, item);
    }
    encode_pose(out, value.solver_principal_frame);
    push_u64(
        out,
        value.solver_tensor_error_max_microkilogram_metre_squared,
    );
    push_sequence(out, &value.colliders, |collider, bytes| {
        push_id(bytes, &collider.collider_id)?;
        encode_pose(bytes, collider.local_pose);
        encode_geometry(bytes, &collider.geometry)?;
        push_id(bytes, &collider.material_id)?;
        bytes.push(collider.collision_layer);
        push_u64(bytes, collider.collision_mask);
        bytes.push(collider.participation as u8);
        bytes.push(collider.contact_reporting as u8);
        bytes.push(collider.contact_role as u8);
        Ok(())
    })
}

fn encode_mapping_group(
    value: &BodyMassProjectionGroupV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.mapping_group_id)?;
    push_u64(out, value.source_mass_microkilograms);
    encode_i64_3(out, value.source_center_of_mass_micrometres);
    encode_i64_6(out, value.source_inertia_tensor_microkilogram_metre_squared);
    push_u64(
        out,
        value.maximum_first_moment_error_microkilogram_micrometres,
    );
    push_u64(out, value.maximum_inertia_error_microkilogram_metre_squared);
    push_sequence(out, &value.members, |member, bytes| {
        push_id(bytes, &member.body_id)?;
        encode_i64_3(bytes, member.body_origin_in_group_micrometres);
        Ok(())
    })
}

fn encode_joint(
    value: &BodyJointDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.joint_id)?;
    push_id(out, &value.parent_body_id)?;
    push_id(out, &value.child_body_id)?;
    push_id(out, &value.anatomical_semantic_id)?;
    encode_pose(out, value.parent_frame);
    encode_pose(out, value.child_frame);
    for axis in value.axis_q1_30 {
        out.extend_from_slice(&axis.to_le_bytes());
    }
    for angle in [
        value.hard_minimum_microradians,
        value.hard_maximum_microradians,
        value.soft_minimum_microradians,
        value.soft_maximum_microradians,
        value.neutral_position_microradians,
    ] {
        push_i64(out, angle);
    }
    push_u64(out, value.maximum_velocity_microradians_per_second);
    Ok(())
}

fn encode_actuator(
    value: &BodyActuatorDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.actuator_id)?;
    push_id(out, &value.joint_id)?;
    for item in [value.stiffness_q16, value.damping_q16] {
        push_u64(out, item);
    }
    push_i64(out, value.minimum_effort_micronewton_metres);
    push_i64(out, value.maximum_effort_micronewton_metres);
    for item in [
        value.maximum_effort_rate_micronewton_metres_per_second,
        value.maximum_power_microwatts,
        value.maximum_positive_work_microjoules_per_motor_tick,
        value.residual_scale_microradians,
    ] {
        push_u64(out, item);
    }
    push_i64(out, value.minimum_target_delta_microradians_per_motor_tick);
    push_i64(out, value.maximum_target_delta_microradians_per_motor_tick);
    Ok(())
}

fn encode_effector(
    value: &BodyEffectorDefinitionV2,
    out: &mut Vec<u8>,
) -> Result<(), BodyV2ContractError> {
    push_id(out, &value.effector_id)?;
    push_id(out, &value.body_id)?;
    encode_pose(out, value.local_pose);
    push_id(out, &value.semantic_role_id)
}

fn encode_pose(out: &mut Vec<u8>, value: BodyPoseV2) {
    encode_i64_3(out, value.translation_micrometres);
    for item in value.rotation_q1_30 {
        out.extend_from_slice(&item.to_le_bytes());
    }
}

fn encode_geometry(
    out: &mut Vec<u8>,
    value: &PhysicsGeometryV1,
) -> Result<(), BodyV2ContractError> {
    match value {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => {
            out.push(1);
            encode_i64_3(out, *half_extents_micrometres);
        }
        PhysicsGeometryV1::Sphere { radius_micrometres } => {
            out.push(2);
            push_i64(out, *radius_micrometres);
        }
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => {
            out.push(3);
            push_i64(out, *radius_micrometres);
            push_i64(out, *half_segment_micrometres);
        }
        _ => return Err(BodyV2ContractError::UnsupportedGeometry),
    }
    Ok(())
}

fn encode_optional_id(
    out: &mut Vec<u8>,
    value: Option<&SchemaId>,
) -> Result<(), BodyV2ContractError> {
    if let Some(value) = value {
        out.push(1);
        push_id(out, value)
    } else {
        out.push(0);
        Ok(())
    }
}

fn encode_i64_3(out: &mut Vec<u8>, values: [i64; 3]) {
    for value in values {
        push_i64(out, value);
    }
}

fn encode_i64_6(out: &mut Vec<u8>, values: [i64; 6]) {
    for value in values {
        push_i64(out, value);
    }
}

fn push_sequence<T>(
    out: &mut Vec<u8>,
    values: &[T],
    mut encode: impl FnMut(&T, &mut Vec<u8>) -> Result<(), BodyV2ContractError>,
) -> Result<(), BodyV2ContractError> {
    push_u32(
        out,
        u32::try_from(values.len()).map_err(|_| BodyV2ContractError::LengthOverflow)?,
    );
    for value in values {
        encode(value, out)?;
    }
    Ok(())
}

fn push_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<(), BodyV2ContractError> {
    push_u32(
        out,
        u32::try_from(value.len()).map_err(|_| BodyV2ContractError::LengthOverflow)?,
    );
    out.extend_from_slice(value);
    Ok(())
}

fn push_id(out: &mut Vec<u8>, value: &SchemaId) -> Result<(), BodyV2ContractError> {
    push_bytes(out, value.as_str().as_bytes())
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn push_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::content_hash_from_bytes;

    fn id(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test ID")
    }

    fn body(name: &str, parent: Option<&str>, group: &str) -> BodyDefinitionV2 {
        BodyDefinitionV2 {
            body_id: id(name),
            parent_body_id: parent.map(id),
            local_bind_pose: BodyPoseV2::default(),
            semantic_role: if parent.is_none() {
                BodySemanticRoleV2::PhysicalRoot
            } else {
                BodySemanticRoleV2::PhysicalFoot
            },
            mapping_group_id: id(group),
            mass_microkilograms: 1_000_000,
            center_of_mass_micrometres: [0; 3],
            inertia_tensor_microkilogram_metre_squared: [1000, 0, 0, 1000, 0, 1000],
            solver_mass_microkilograms: 1_000_000,
            solver_center_of_mass_micrometres: [0; 3],
            solver_principal_inertia_microkilogram_metre_squared: [1000; 3],
            solver_principal_frame: BodyPoseV2::default(),
            solver_tensor_error_max_microkilogram_metre_squared: 1,
            colliders: vec![BodyColliderDefinitionV2 {
                collider_id: id(&format!("{name}.collider")),
                local_pose: BodyPoseV2::default(),
                geometry: PhysicsGeometryV1::Sphere {
                    radius_micrometres: 100_000,
                },
                material_id: id("material.test"),
                collision_layer: 1,
                collision_mask: 1,
                participation: PhysicsParticipationV1::Solid,
                contact_reporting: PhysicsContactReportingV1::BeginPersistEnd,
                contact_role: BodyContactRoleV2::FootWithSoleFeature,
            }],
        }
    }

    fn schema() -> BodySchemaV2 {
        let root = body("body.root", None, "map.root");
        let child = body("body.child", Some("body.root"), "map.child");
        BodySchemaV2 {
            schema_version: BODY_SCHEMA_VERSION_V2,
            schema_id: id("body-schema.test.v2"),
            schema_revision: 1,
            family_id: id("family.humanoid"),
            coordinate_profile_hash: content_hash_from_bytes([1; 32]),
            source_provenance_hash: content_hash_from_bytes([2; 32]),
            solver_projection_profile_hash: content_hash_from_bytes([3; 32]),
            rotation_norm_tolerance_q2_60: 1,
            bodies: vec![root, child],
            mass_projection_groups: vec![
                BodyMassProjectionGroupV2 {
                    mapping_group_id: id("map.root"),
                    source_mass_microkilograms: 1_000_000,
                    source_center_of_mass_micrometres: [0; 3],
                    source_inertia_tensor_microkilogram_metre_squared: [1000, 0, 0, 1000, 0, 1000],
                    maximum_first_moment_error_microkilogram_micrometres: 1,
                    maximum_inertia_error_microkilogram_metre_squared: 1,
                    members: vec![BodyMassProjectionMemberV2 {
                        body_id: id("body.root"),
                        body_origin_in_group_micrometres: [0; 3],
                    }],
                },
                BodyMassProjectionGroupV2 {
                    mapping_group_id: id("map.child"),
                    source_mass_microkilograms: 1_000_000,
                    source_center_of_mass_micrometres: [0; 3],
                    source_inertia_tensor_microkilogram_metre_squared: [1000, 0, 0, 1000, 0, 1000],
                    maximum_first_moment_error_microkilogram_micrometres: 1,
                    maximum_inertia_error_microkilogram_metre_squared: 1,
                    members: vec![BodyMassProjectionMemberV2 {
                        body_id: id("body.child"),
                        body_origin_in_group_micrometres: [0; 3],
                    }],
                },
            ],
            joints: vec![BodyJointDefinitionV2 {
                joint_id: id("joint.child"),
                parent_body_id: id("body.root"),
                child_body_id: id("body.child"),
                anatomical_semantic_id: id("joint-semantic.flexion"),
                parent_frame: BodyPoseV2::default(),
                child_frame: BodyPoseV2::default(),
                axis_q1_30: [1 << 30, 0, 0],
                hard_minimum_microradians: 0,
                hard_maximum_microradians: 2_000_000,
                soft_minimum_microradians: 0,
                soft_maximum_microradians: 1_900_000,
                neutral_position_microradians: 0,
                maximum_velocity_microradians_per_second: 5_000_000,
            }],
            actuators: vec![BodyActuatorDefinitionV2 {
                actuator_id: id("actuator.child"),
                joint_id: id("joint.child"),
                stiffness_q16: 65_536,
                damping_q16: 65_536,
                minimum_effort_micronewton_metres: -1_000_000,
                maximum_effort_micronewton_metres: 1_000_000,
                maximum_effort_rate_micronewton_metres_per_second: 2_000_000,
                maximum_power_microwatts: 3_000_000,
                maximum_positive_work_microjoules_per_motor_tick: 4_000,
                residual_scale_microradians: 100_000,
                minimum_target_delta_microradians_per_motor_tick: -50_000,
                maximum_target_delta_microradians_per_motor_tick: 50_000,
            }],
            effectors: vec![BodyEffectorDefinitionV2 {
                effector_id: id("effector.foot"),
                body_id: id("body.child"),
                local_pose: BodyPoseV2::default(),
                semantic_role_id: id("contact-role.foot"),
            }],
            symmetry_pairs: Vec::new(),
            collision_exclusions: vec![BodyCollisionExclusionV2 {
                first_body_id: id("body.child"),
                second_body_id: id("body.root"),
            }],
            capability_ids: vec![id("capability.stand")],
        }
        .canonicalize()
    }

    #[test]
    fn v2_canonicalization_erases_source_order() {
        let expected = schema().schema_hash().expect("valid schema");
        let mut permuted = schema();
        permuted.bodies.reverse();
        permuted.mass_projection_groups.reverse();
        assert_eq!(
            permuted.canonicalize().schema_hash().expect("canonical"),
            expected
        );
    }

    #[test]
    fn v2_rejects_non_positive_definite_inertia() {
        let mut invalid = schema();
        invalid.bodies[0].inertia_tensor_microkilogram_metre_squared = [1, 2, 0, 1, 0, 1];
        assert_eq!(invalid.validate(), Err(BodyV2ContractError::InvalidInertia));
    }

    #[test]
    fn v2_rejects_mass_projection_drift() {
        let mut invalid = schema();
        invalid.mass_projection_groups[0].source_mass_microkilograms += 1;
        assert_eq!(invalid.validate(), Err(BodyV2ContractError::InvalidMapping));
    }

    #[test]
    fn v2_rejects_hyperextension_side_for_flexion_joint() {
        let mut invalid = schema();
        invalid.joints[0].hard_minimum_microradians = 1;
        assert_eq!(invalid.validate(), Err(BodyV2ContractError::InvalidJoint));
    }

    #[test]
    fn v2_rejects_hidden_carrier_collider() {
        let mut invalid = schema();
        invalid.bodies[0].semantic_role = BodySemanticRoleV2::NonCollidingCarrier;
        assert_eq!(
            invalid.validate(),
            Err(BodyV2ContractError::InvalidCollider)
        );
    }

    #[test]
    fn v2_rejects_unnormalized_joint_axis_and_frame() {
        let mut invalid_axis = schema();
        invalid_axis.joints[0].axis_q1_30 = [1, 0, 0];
        assert_eq!(
            invalid_axis.validate(),
            Err(BodyV2ContractError::InvalidPhysicalParameter)
        );

        let mut invalid_frame = schema();
        invalid_frame.joints[0].parent_frame.rotation_q1_30 = [0; 4];
        assert_eq!(
            invalid_frame.validate(),
            Err(BodyV2ContractError::InvalidPhysicalParameter)
        );
    }

    #[test]
    fn v2_rejects_inverted_soft_limit_and_disabled_collider_mask() {
        let mut invalid_limit = schema();
        invalid_limit.joints[0].soft_minimum_microradians = 1_950_000;
        assert_eq!(
            invalid_limit.validate(),
            Err(BodyV2ContractError::InvalidJoint)
        );

        let mut invalid_collider = schema();
        invalid_collider.bodies[0].colliders[0].collision_mask = 0;
        assert_eq!(
            invalid_collider.validate(),
            Err(BodyV2ContractError::InvalidCollider)
        );
    }
}
