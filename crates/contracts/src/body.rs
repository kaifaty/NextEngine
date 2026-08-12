#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::sha256;
use crate::ids::{ContentHash, SchemaId, content_hash_from_bytes};
use crate::physics::{PhysicsGeometryV1, PhysicsPoseV1};

mod v2;
pub use v2::*;

pub const BODY_SCHEMA_VERSION_V1: u16 = 1;
pub const BODY_INSTANCE_PROJECTION_VERSION_V1: u16 = 1;
pub const MAX_BODY_SCHEMA_BODIES: usize = 128;
pub const MAX_BODY_SCHEMA_JOINTS: usize = 256;
pub const MAX_BODY_SCHEMA_ACTUATORS: usize = 256;
pub const MAX_BODY_SCHEMA_EFFECTORS: usize = 64;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyColliderDefinitionV1 {
    pub collider_id: SchemaId,
    pub local_pose: PhysicsPoseV1,
    pub geometry: PhysicsGeometryV1,
    pub material_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyDefinitionV1 {
    pub body_id: SchemaId,
    pub parent_body_id: Option<SchemaId>,
    pub local_bind_pose: PhysicsPoseV1,
    pub mass_microkilograms: u64,
    pub center_of_mass_micrometres: [i64; 3],
    pub inertia_microkilogram_metre_squared: [u64; 3],
    pub colliders: Vec<BodyColliderDefinitionV1>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyJointDefinitionV1 {
    pub joint_id: SchemaId,
    pub parent_body_id: SchemaId,
    pub child_body_id: SchemaId,
    pub parent_frame: PhysicsPoseV1,
    pub child_frame: PhysicsPoseV1,
    pub axis_q1_30: [i32; 3],
    pub limit_min_microradians: i64,
    pub limit_max_microradians: i64,
    pub maximum_velocity_microradians_per_second: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyActuatorDefinitionV1 {
    pub actuator_id: SchemaId,
    pub joint_id: SchemaId,
    pub neutral_position_microradians: i64,
    pub stiffness_q16: u64,
    pub damping_q16: u64,
    pub maximum_effort_micronewton_metres: u64,
    pub maximum_effort_rate_micronewton_metres_per_second: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyEffectorDefinitionV1 {
    pub effector_id: SchemaId,
    pub body_id: SchemaId,
    pub local_pose: PhysicsPoseV1,
    pub semantic_role_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodySymmetryPairV1 {
    pub left_id: SchemaId,
    pub right_id: SchemaId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodySchemaV1 {
    pub schema_version: u16,
    pub schema_id: SchemaId,
    pub schema_revision: u32,
    pub family_id: SchemaId,
    pub bodies: Vec<BodyDefinitionV1>,
    pub joints: Vec<BodyJointDefinitionV1>,
    pub actuators: Vec<BodyActuatorDefinitionV1>,
    pub effectors: Vec<BodyEffectorDefinitionV1>,
    pub symmetry_pairs: Vec<BodySymmetryPairV1>,
    pub capability_ids: Vec<SchemaId>,
}

impl BodySchemaV1 {
    #[must_use]
    pub fn canonicalize(mut self) -> Self {
        for body in &mut self.bodies {
            body.colliders
                .sort_by(|left, right| left.collider_id.cmp(&right.collider_id));
        }
        self.bodies
            .sort_by(|left, right| left.body_id.cmp(&right.body_id));
        self.joints
            .sort_by(|left, right| left.joint_id.cmp(&right.joint_id));
        self.actuators
            .sort_by(|left, right| left.actuator_id.cmp(&right.actuator_id));
        self.effectors
            .sort_by(|left, right| left.effector_id.cmp(&right.effector_id));
        self.symmetry_pairs.sort();
        self.capability_ids.sort();
        self
    }

    pub fn validate(&self) -> Result<(), BodyContractError> {
        if self.schema_version != BODY_SCHEMA_VERSION_V1 {
            return Err(BodyContractError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.schema_revision == 0
            || self.bodies.is_empty()
            || self.bodies.len() > MAX_BODY_SCHEMA_BODIES
            || self.joints.len() > MAX_BODY_SCHEMA_JOINTS
            || self.actuators.len() > MAX_BODY_SCHEMA_ACTUATORS
            || self.effectors.len() > MAX_BODY_SCHEMA_EFFECTORS
        {
            return Err(BodyContractError::InvalidBounds);
        }
        require_strict_order(self.bodies.iter().map(|value| &value.body_id))?;
        require_strict_order(self.joints.iter().map(|value| &value.joint_id))?;
        require_strict_order(self.actuators.iter().map(|value| &value.actuator_id))?;
        require_strict_order(self.effectors.iter().map(|value| &value.effector_id))?;
        require_strict_order(self.capability_ids.iter())?;
        if self
            .symmetry_pairs
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(BodyContractError::NonCanonicalOrder);
        }

        let body_ids = self
            .bodies
            .iter()
            .map(|body| body.body_id.clone())
            .collect::<BTreeSet<_>>();
        let roots = self
            .bodies
            .iter()
            .filter(|body| body.parent_body_id.is_none())
            .count();
        if roots != 1 {
            return Err(BodyContractError::InvalidTopology);
        }
        let mut parents = BTreeMap::new();
        for body in &self.bodies {
            if body.mass_microkilograms == 0
                || body.inertia_microkilogram_metre_squared.contains(&0)
                || body.colliders.is_empty()
            {
                return Err(BodyContractError::InvalidPhysicalParameter);
            }
            body.local_bind_pose.validate()?;
            if let Some(parent) = &body.parent_body_id {
                if !body_ids.contains(parent) || parent == &body.body_id {
                    return Err(BodyContractError::InvalidTopology);
                }
                parents.insert(body.body_id.clone(), parent.clone());
            }
            require_strict_order(body.colliders.iter().map(|value| &value.collider_id))?;
            for collider in &body.colliders {
                collider.local_pose.validate()?;
                match collider.geometry {
                    PhysicsGeometryV1::Box { .. }
                    | PhysicsGeometryV1::Sphere { .. }
                    | PhysicsGeometryV1::Capsule { .. } => collider.geometry.validate()?,
                    _ => return Err(BodyContractError::UnsupportedGeometry),
                }
            }
        }
        for body_id in &body_ids {
            let mut cursor = body_id;
            let mut visited = BTreeSet::new();
            while let Some(parent) = parents.get(cursor) {
                if !visited.insert(cursor.clone()) {
                    return Err(BodyContractError::InvalidTopology);
                }
                cursor = parent;
            }
        }

        let joint_ids = self
            .joints
            .iter()
            .map(|joint| joint.joint_id.clone())
            .collect::<BTreeSet<_>>();
        let mut joint_children = BTreeSet::new();
        for joint in &self.joints {
            if !body_ids.contains(&joint.parent_body_id)
                || !body_ids.contains(&joint.child_body_id)
                || joint.parent_body_id == joint.child_body_id
                || !joint_children.insert(joint.child_body_id.clone())
                || joint.limit_min_microradians >= joint.limit_max_microradians
                || joint.maximum_velocity_microradians_per_second == 0
                || axis_norm(joint.axis_q1_30) != Some(1_i128 << 60)
            {
                return Err(BodyContractError::InvalidJoint);
            }
            joint.parent_frame.validate()?;
            joint.child_frame.validate()?;
            if parents.get(&joint.child_body_id) != Some(&joint.parent_body_id) {
                return Err(BodyContractError::InvalidTopology);
            }
        }
        if self.joints.len() + 1 != self.bodies.len() {
            return Err(BodyContractError::InvalidTopology);
        }
        for actuator in &self.actuators {
            if !joint_ids.contains(&actuator.joint_id)
                || actuator.stiffness_q16 == 0
                || actuator.damping_q16 == 0
                || actuator.maximum_effort_micronewton_metres == 0
                || actuator.maximum_effort_rate_micronewton_metres_per_second == 0
            {
                return Err(BodyContractError::InvalidActuator);
            }
        }
        for effector in &self.effectors {
            if !body_ids.contains(&effector.body_id) {
                return Err(BodyContractError::InvalidReference);
            }
            effector.local_pose.validate()?;
        }
        let semantic_ids = body_ids
            .iter()
            .chain(joint_ids.iter())
            .chain(self.actuators.iter().map(|value| &value.actuator_id))
            .chain(self.effectors.iter().map(|value| &value.effector_id))
            .collect::<BTreeSet<_>>();
        for pair in &self.symmetry_pairs {
            if pair.left_id == pair.right_id
                || !semantic_ids.contains(&pair.left_id)
                || !semantic_ids.contains(&pair.right_id)
            {
                return Err(BodyContractError::InvalidReference);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BodyContractError> {
        self.validate()?;
        let mut output = Vec::new();
        push_bytes(&mut output, b"nextengine.body-schema.v1\0")?;
        push_u16(&mut output, self.schema_version);
        push_id(&mut output, &self.schema_id)?;
        push_u32(&mut output, self.schema_revision);
        push_id(&mut output, &self.family_id)?;
        push_sequence(&mut output, &self.bodies, encode_body)?;
        push_sequence(&mut output, &self.joints, encode_joint)?;
        push_sequence(&mut output, &self.actuators, encode_actuator)?;
        push_sequence(&mut output, &self.effectors, encode_effector)?;
        push_sequence(&mut output, &self.symmetry_pairs, |pair, bytes| {
            push_id(bytes, &pair.left_id)?;
            push_id(bytes, &pair.right_id)
        })?;
        push_sequence(&mut output, &self.capability_ids, |id, bytes| {
            push_id(bytes, id)
        })?;
        Ok(output)
    }

    pub fn schema_hash(&self) -> Result<ContentHash, BodyContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodyInstanceProjectionV1 {
    pub schema_version: u16,
    pub subject_id: crate::ids::PersistentId,
    pub body_schema_id: SchemaId,
    pub body_schema_revision: u32,
    pub body_schema_hash: ContentHash,
    pub morphology_revision: u64,
    pub morphology_hash: ContentHash,
    pub equipment_revision: u64,
    pub equipment_hash: ContentHash,
    pub stats_revision: u64,
    pub stats_hash: ContentHash,
    pub damage_revision: u64,
    pub damage_hash: ContentHash,
    pub fatigue_revision: u64,
    pub fatigue_hash: ContentHash,
    pub attachment_revision: u64,
    pub attachment_hash: ContentHash,
    pub topology_revision: u64,
}

impl BodyInstanceProjectionV1 {
    pub fn validate(&self) -> Result<(), BodyContractError> {
        if self.schema_version != BODY_INSTANCE_PROJECTION_VERSION_V1 {
            return Err(BodyContractError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.body_schema_revision == 0 || self.topology_revision == 0 {
            return Err(BodyContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn projection_hash(&self) -> Result<ContentHash, BodyContractError> {
        self.validate()?;
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, b"nextengine.body-instance-projection.v1\0")?;
        push_u16(&mut bytes, self.schema_version);
        bytes.extend_from_slice(self.subject_id.as_bytes());
        push_id(&mut bytes, &self.body_schema_id)?;
        push_u32(&mut bytes, self.body_schema_revision);
        bytes.extend_from_slice(self.body_schema_hash.as_bytes());
        for (revision, hash) in [
            (self.morphology_revision, self.morphology_hash),
            (self.equipment_revision, self.equipment_hash),
            (self.stats_revision, self.stats_hash),
            (self.damage_revision, self.damage_hash),
            (self.fatigue_revision, self.fatigue_hash),
            (self.attachment_revision, self.attachment_hash),
        ] {
            push_u64(&mut bytes, revision);
            bytes.extend_from_slice(hash.as_bytes());
        }
        push_u64(&mut bytes, self.topology_revision);
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyContractError {
    UnsupportedSchemaVersion(u16),
    InvalidBounds,
    NonCanonicalOrder,
    InvalidTopology,
    InvalidPhysicalParameter,
    InvalidJoint,
    InvalidActuator,
    InvalidReference,
    UnsupportedGeometry,
    LengthOverflow,
    Physics(crate::physics::PhysicsContractError),
}

impl BodyContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion(_) => "UNSUPPORTED_BODY_SCHEMA_VERSION",
            Self::InvalidBounds => "BODY_SCHEMA_BOUNDS_INVALID",
            Self::NonCanonicalOrder => "BODY_SCHEMA_ORDER_INVALID",
            Self::InvalidTopology => "BODY_SCHEMA_TOPOLOGY_INVALID",
            Self::InvalidPhysicalParameter => "BODY_SCHEMA_PHYSICAL_PARAMETER_INVALID",
            Self::InvalidJoint => "BODY_SCHEMA_JOINT_INVALID",
            Self::InvalidActuator => "BODY_SCHEMA_ACTUATOR_INVALID",
            Self::InvalidReference => "BODY_SCHEMA_REFERENCE_INVALID",
            Self::UnsupportedGeometry => "BODY_SCHEMA_GEOMETRY_UNSUPPORTED",
            Self::LengthOverflow => "BODY_SCHEMA_LENGTH_OVERFLOW",
            Self::Physics(_) => "BODY_SCHEMA_PHYSICS_VALUE_INVALID",
        }
    }
}

impl Display for BodyContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for BodyContractError {}

impl From<crate::physics::PhysicsContractError> for BodyContractError {
    fn from(value: crate::physics::PhysicsContractError) -> Self {
        Self::Physics(value)
    }
}

fn require_strict_order<'a>(
    values: impl Iterator<Item = &'a SchemaId>,
) -> Result<(), BodyContractError> {
    let mut previous: Option<&SchemaId> = None;
    for value in values {
        if previous.is_some_and(|previous| previous >= value) {
            return Err(BodyContractError::NonCanonicalOrder);
        }
        previous = Some(value);
    }
    Ok(())
}

fn axis_norm(axis: [i32; 3]) -> Option<i128> {
    axis.into_iter().try_fold(0_i128, |sum, value| {
        let value = i128::from(value);
        sum.checked_add(value * value)
    })
}

fn encode_body(value: &BodyDefinitionV1, output: &mut Vec<u8>) -> Result<(), BodyContractError> {
    push_id(output, &value.body_id)?;
    match &value.parent_body_id {
        Some(id) => {
            output.push(1);
            push_id(output, id)?;
        }
        None => output.push(0),
    }
    encode_pose(value.local_bind_pose, output);
    push_u64(output, value.mass_microkilograms);
    encode_i64_3(value.center_of_mass_micrometres, output);
    for value in value.inertia_microkilogram_metre_squared {
        push_u64(output, value);
    }
    push_sequence(output, &value.colliders, |collider, bytes| {
        push_id(bytes, &collider.collider_id)?;
        encode_pose(collider.local_pose, bytes);
        encode_geometry(&collider.geometry, bytes)?;
        push_id(bytes, &collider.material_id)
    })
}

fn encode_joint(
    value: &BodyJointDefinitionV1,
    output: &mut Vec<u8>,
) -> Result<(), BodyContractError> {
    push_id(output, &value.joint_id)?;
    push_id(output, &value.parent_body_id)?;
    push_id(output, &value.child_body_id)?;
    encode_pose(value.parent_frame, output);
    encode_pose(value.child_frame, output);
    for value in value.axis_q1_30 {
        output.extend_from_slice(&value.to_le_bytes());
    }
    push_i64(output, value.limit_min_microradians);
    push_i64(output, value.limit_max_microradians);
    push_u64(output, value.maximum_velocity_microradians_per_second);
    Ok(())
}

fn encode_actuator(
    value: &BodyActuatorDefinitionV1,
    output: &mut Vec<u8>,
) -> Result<(), BodyContractError> {
    push_id(output, &value.actuator_id)?;
    push_id(output, &value.joint_id)?;
    push_i64(output, value.neutral_position_microradians);
    push_u64(output, value.stiffness_q16);
    push_u64(output, value.damping_q16);
    push_u64(output, value.maximum_effort_micronewton_metres);
    push_u64(
        output,
        value.maximum_effort_rate_micronewton_metres_per_second,
    );
    Ok(())
}

fn encode_effector(
    value: &BodyEffectorDefinitionV1,
    output: &mut Vec<u8>,
) -> Result<(), BodyContractError> {
    push_id(output, &value.effector_id)?;
    push_id(output, &value.body_id)?;
    encode_pose(value.local_pose, output);
    push_id(output, &value.semantic_role_id)
}

fn encode_pose(value: PhysicsPoseV1, output: &mut Vec<u8>) {
    encode_i64_3(value.translation_micrometres, output);
    for value in value.rotation_q1_30 {
        output.extend_from_slice(&value.to_le_bytes());
    }
}

fn encode_geometry(
    value: &PhysicsGeometryV1,
    output: &mut Vec<u8>,
) -> Result<(), BodyContractError> {
    match value {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => {
            output.push(1);
            encode_i64_3(*half_extents_micrometres, output);
        }
        PhysicsGeometryV1::Sphere { radius_micrometres } => {
            output.push(2);
            push_i64(output, *radius_micrometres);
        }
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => {
            output.push(3);
            push_i64(output, *radius_micrometres);
            push_i64(output, *half_segment_micrometres);
        }
        _ => return Err(BodyContractError::UnsupportedGeometry),
    }
    Ok(())
}

fn encode_i64_3(values: [i64; 3], output: &mut Vec<u8>) {
    for value in values {
        push_i64(output, value);
    }
}

fn push_sequence<T>(
    output: &mut Vec<u8>,
    values: &[T],
    mut encode: impl FnMut(&T, &mut Vec<u8>) -> Result<(), BodyContractError>,
) -> Result<(), BodyContractError> {
    push_u32(
        output,
        u32::try_from(values.len()).map_err(|_| BodyContractError::LengthOverflow)?,
    );
    for value in values {
        encode(value, output)?;
    }
    Ok(())
}

fn push_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), BodyContractError> {
    push_u32(
        output,
        u32::try_from(bytes.len()).map_err(|_| BodyContractError::LengthOverflow)?,
    );
    output.extend_from_slice(bytes);
    Ok(())
}

fn push_id(output: &mut Vec<u8>, id: &SchemaId) -> Result<(), BodyContractError> {
    push_bytes(output, id.as_str().as_bytes())
}

fn push_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_i64(output: &mut Vec<u8>, value: i64) {
    output.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test identifier")
    }

    fn body(body_id: &str, parent: Option<&str>) -> BodyDefinitionV1 {
        BodyDefinitionV1 {
            body_id: id(body_id),
            parent_body_id: parent.map(id),
            local_bind_pose: PhysicsPoseV1::default(),
            mass_microkilograms: 1_000_000,
            center_of_mass_micrometres: [0; 3],
            inertia_microkilogram_metre_squared: [1; 3],
            colliders: vec![BodyColliderDefinitionV1 {
                collider_id: id(&format!("{body_id}.collider")),
                local_pose: PhysicsPoseV1::default(),
                geometry: PhysicsGeometryV1::Sphere {
                    radius_micrometres: 100_000,
                },
                material_id: id("nextengine.material.training"),
            }],
        }
    }

    fn schema() -> BodySchemaV1 {
        BodySchemaV1 {
            schema_version: BODY_SCHEMA_VERSION_V1,
            schema_id: id("nextengine.body.test"),
            schema_revision: 1,
            family_id: id("nextengine.family.humanoid"),
            bodies: vec![
                body("nextengine.body.root", None),
                body("nextengine.body.child", Some("nextengine.body.root")),
            ],
            joints: vec![BodyJointDefinitionV1 {
                joint_id: id("nextengine.joint.test"),
                parent_body_id: id("nextengine.body.root"),
                child_body_id: id("nextengine.body.child"),
                parent_frame: PhysicsPoseV1::default(),
                child_frame: PhysicsPoseV1::default(),
                axis_q1_30: [1 << 30, 0, 0],
                limit_min_microradians: -1_000_000,
                limit_max_microradians: 1_000_000,
                maximum_velocity_microradians_per_second: 5_000_000,
            }],
            actuators: vec![BodyActuatorDefinitionV1 {
                actuator_id: id("nextengine.actuator.test"),
                joint_id: id("nextengine.joint.test"),
                neutral_position_microradians: 0,
                stiffness_q16: 65_536,
                damping_q16: 65_536,
                maximum_effort_micronewton_metres: 10_000_000,
                maximum_effort_rate_micronewton_metres_per_second: 20_000_000,
            }],
            effectors: vec![BodyEffectorDefinitionV1 {
                effector_id: id("nextengine.effector.test"),
                body_id: id("nextengine.body.child"),
                local_pose: PhysicsPoseV1::default(),
                semantic_role_id: id("nextengine.role.foot"),
            }],
            symmetry_pairs: Vec::new(),
            capability_ids: vec![id("nextengine.capability.stand")],
        }
        .canonicalize()
    }

    #[test]
    fn canonicalization_erases_source_record_order() {
        let expected = schema().schema_hash().expect("valid schema");
        let mut permuted = schema();
        permuted.bodies.reverse();
        assert!(permuted.validate().is_err());
        assert_eq!(
            permuted.canonicalize().schema_hash().expect("canonical"),
            expected
        );
    }

    #[test]
    fn topology_cycle_fails_closed() {
        let mut invalid = schema();
        invalid.bodies[1].parent_body_id = Some(invalid.bodies[0].body_id.clone());
        invalid.bodies[0].parent_body_id = Some(invalid.bodies[1].body_id.clone());
        assert_eq!(invalid.validate(), Err(BodyContractError::InvalidTopology));
    }
}
