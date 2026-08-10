use std::collections::{BTreeMap, BTreeSet};

use crate::canonical::{CanonicalError, sha256};
use crate::ids::{ContentHash, SchemaId, content_hash_from_bytes};

use super::{
    ClosedPhysicsContactBatchV1, PhysicsBodyDescriptorV1, PhysicsBodyIdV1,
    PhysicsCanonicalSnapshotV2, PhysicsContractError, PhysicsStepInputV2, PhysicsStepResultV1,
    PhysicsWorldCatalogV1,
};

pub const PHYSICS_BODY_DESCRIPTOR_V2_SCHEMA_VERSION: u16 = 2;
pub const PHYSICS_WORLD_CATALOG_V2_SCHEMA_VERSION: u16 = 2;
pub const PHYSICS_JOINT_DESCRIPTOR_V1_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_ACTUATOR_DESCRIPTOR_V1_SCHEMA_VERSION: u16 = 1;
pub const PHYSICS_STEP_INPUT_V3_SCHEMA_VERSION: u16 = 3;
pub const PHYSICS_STEP_RESULT_V2_SCHEMA_VERSION: u16 = 2;
pub const PHYSICS_CANONICAL_SNAPSHOT_V3_SCHEMA_VERSION: u16 = 3;
pub const PHYSICS_WORLD_CHECKPOINT_V2_SCHEMA_VERSION: u16 = 2;
pub const MAX_ARTICULATION_JOINTS: usize = 256;
pub const MAX_ARTICULATION_ACTUATORS: usize = 256;
pub const MAX_ACTUATOR_EFFORTS_PER_SUBSTEP: usize = 256;
pub const MAX_MOTOR_SUBSTEPS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsJointKindV1 {
    Fixed = 1,
    Revolute = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsBodyDescriptorV2 {
    pub schema_version: u16,
    pub semantic_body_id: SchemaId,
    pub base: PhysicsBodyDescriptorV1,
    pub mass_microkilograms: u64,
    pub center_of_mass_micrometres: [i64; 3],
    pub inertia_microkilogram_metre_squared: [u64; 3],
    pub articulation_id: Option<SchemaId>,
}

impl PhysicsBodyDescriptorV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_BODY_DESCRIPTOR_V2_SCHEMA_VERSION
            || self.mass_microkilograms == 0
            || self.inertia_microkilogram_metre_squared.contains(&0)
        {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        self.base.validate()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsJointDescriptorV1 {
    pub schema_version: u16,
    pub joint_id: SchemaId,
    pub articulation_id: SchemaId,
    pub parent_body_id: PhysicsBodyIdV1,
    pub child_body_id: PhysicsBodyIdV1,
    pub joint_kind: PhysicsJointKindV1,
    pub axis_q1_30: [i32; 3],
    pub limit_min_microradians: i64,
    pub limit_max_microradians: i64,
    pub maximum_velocity_microradians_per_second: u64,
}

impl PhysicsJointDescriptorV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        let norm = self.axis_q1_30.into_iter().try_fold(0_i128, |sum, value| {
            let value = i128::from(value);
            sum.checked_add(value * value)
        });
        if self.schema_version != PHYSICS_JOINT_DESCRIPTOR_V1_SCHEMA_VERSION
            || self.parent_body_id == self.child_body_id
            || self.limit_min_microradians >= self.limit_max_microradians
            || self.maximum_velocity_microradians_per_second == 0
            || (self.joint_kind == PhysicsJointKindV1::Revolute && norm != Some(1_i128 << 60))
        {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsActuatorDescriptorV1 {
    pub schema_version: u16,
    pub actuator_id: SchemaId,
    pub joint_id: SchemaId,
    pub neutral_position_microradians: i64,
    pub limit_min_microradians: i64,
    pub limit_max_microradians: i64,
    pub maximum_effort_micronewton_metres: u64,
    pub maximum_effort_rate_micronewton_metres_per_second: u64,
}

impl PhysicsActuatorDescriptorV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_ACTUATOR_DESCRIPTOR_V1_SCHEMA_VERSION
            || self.limit_min_microradians >= self.limit_max_microradians
            || !(self.limit_min_microradians..=self.limit_max_microradians)
                .contains(&self.neutral_position_microradians)
            || self.maximum_effort_micronewton_metres == 0
            || self.maximum_effort_rate_micronewton_metres_per_second == 0
        {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsWorldCatalogV2 {
    pub schema_version: u16,
    pub base: PhysicsWorldCatalogV1,
    pub body_schema_hash: ContentHash,
    pub body_instance_projection_hash: ContentHash,
    pub physx_build_profile_hash: ContentHash,
    pub scene_profile_hash: ContentHash,
    pub bridge_abi_hash: ContentHash,
    pub bodies: BTreeMap<PhysicsBodyIdV1, PhysicsBodyDescriptorV2>,
    pub joints: Vec<PhysicsJointDescriptorV1>,
    pub actuators: Vec<PhysicsActuatorDescriptorV1>,
}

impl PhysicsWorldCatalogV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_WORLD_CATALOG_V2_SCHEMA_VERSION
            || self.joints.len() > MAX_ARTICULATION_JOINTS
            || self.actuators.len() > MAX_ARTICULATION_ACTUATORS
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        self.base.validate()?;
        for (id, body) in &self.bodies {
            if id != &body.base.body_id || !self.base.bodies.contains_key(id) {
                return Err(PhysicsContractError::ReferenceInvalid);
            }
            body.validate()?;
        }
        if self
            .joints
            .windows(2)
            .any(|pair| pair[0].joint_id >= pair[1].joint_id)
            || self
                .actuators
                .windows(2)
                .any(|pair| pair[0].actuator_id >= pair[1].actuator_id)
        {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        let mut child_bodies = BTreeSet::new();
        let joint_ids = self
            .joints
            .iter()
            .map(|joint| joint.joint_id.clone())
            .collect::<BTreeSet<_>>();
        for joint in &self.joints {
            joint.validate()?;
            if !self.bodies.contains_key(&joint.parent_body_id)
                || !self.bodies.contains_key(&joint.child_body_id)
                || !child_bodies.insert(joint.child_body_id)
            {
                return Err(PhysicsContractError::ReferenceInvalid);
            }
        }
        for actuator in &self.actuators {
            actuator.validate()?;
            if !joint_ids.contains(&actuator.joint_id) {
                return Err(PhysicsContractError::ReferenceInvalid);
            }
        }
        Ok(())
    }

    pub fn catalog_hash_v2(&self) -> Result<ContentHash, PhysicsContractError> {
        self.validate()?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.physics-world-catalog.v2\0");
        bytes.extend_from_slice(self.base.catalog_hash()?.as_bytes());
        for hash in [
            self.body_schema_hash,
            self.body_instance_projection_hash,
            self.physx_build_profile_hash,
            self.scene_profile_hash,
            self.bridge_abi_hash,
        ] {
            bytes.extend_from_slice(hash.as_bytes());
        }
        for (id, body) in &self.bodies {
            bytes.extend_from_slice(&id.canonical_key_bytes());
            push_id(&mut bytes, &body.semantic_body_id)?;
            bytes.extend_from_slice(&body.mass_microkilograms.to_le_bytes());
            for value in body.center_of_mass_micrometres {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            for value in body.inertia_microkilogram_metre_squared {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        for joint in &self.joints {
            push_id(&mut bytes, &joint.joint_id)?;
            bytes.extend_from_slice(&joint.parent_body_id.canonical_key_bytes());
            bytes.extend_from_slice(&joint.child_body_id.canonical_key_bytes());
            bytes.push(joint.joint_kind as u8);
            for value in joint.axis_q1_30 {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes.extend_from_slice(&joint.limit_min_microradians.to_le_bytes());
            bytes.extend_from_slice(&joint.limit_max_microradians.to_le_bytes());
        }
        for actuator in &self.actuators {
            push_id(&mut bytes, &actuator.actuator_id)?;
            push_id(&mut bytes, &actuator.joint_id)?;
            bytes.extend_from_slice(&actuator.neutral_position_microradians.to_le_bytes());
            bytes.extend_from_slice(&actuator.maximum_effort_micronewton_metres.to_le_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AppliedActuatorEffortV1 {
    pub actuator_id: SchemaId,
    pub effort_micronewton_metres: i64,
    pub clamp_flags: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsSubstepActuationV1 {
    pub physics_tick: u64,
    pub substep_ordinal: u16,
    pub efforts: Vec<AppliedActuatorEffortV1>,
}

impl PhysicsSubstepActuationV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.efforts.len() > MAX_ACTUATOR_EFFORTS_PER_SUBSTEP
            || self
                .efforts
                .windows(2)
                .any(|pair| pair[0].actuator_id >= pair[1].actuator_id)
        {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsStepInputV3 {
    pub schema_version: u16,
    pub base: PhysicsStepInputV2,
    pub body_schema_hash: ContentHash,
    pub motor_action_layout_hash: ContentHash,
    pub actuation_substeps: Vec<PhysicsSubstepActuationV1>,
}

impl PhysicsStepInputV3 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_STEP_INPUT_V3_SCHEMA_VERSION
            || self.actuation_substeps.is_empty()
            || self.actuation_substeps.len() > MAX_MOTOR_SUBSTEPS
            || usize::try_from(self.base.physics_substeps).ok()
                != Some(self.actuation_substeps.len())
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        self.base.validate()?;
        for (expected, substep) in self.actuation_substeps.iter().enumerate() {
            substep.validate()?;
            if usize::from(substep.substep_ordinal) != expected
                || substep.physics_tick
                    != self.base.first_physics_tick
                        + u64::try_from(expected)
                            .map_err(|_| PhysicsContractError::InvalidProfile)?
            {
                return Err(PhysicsContractError::InvalidProfile);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicsArticulationJointStateV1 {
    pub joint_id: SchemaId,
    pub position_microradians: i64,
    pub velocity_microradians_per_second: i64,
    pub applied_effort_micronewton_metres: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsCanonicalSnapshotV3 {
    pub schema_version: u16,
    pub base: PhysicsCanonicalSnapshotV2,
    pub catalog_v2_hash: ContentHash,
    pub physx_build_profile_hash: ContentHash,
    pub scene_profile_hash: ContentHash,
    pub bridge_abi_hash: ContentHash,
    pub sorted_joint_states: Vec<PhysicsArticulationJointStateV1>,
}

impl PhysicsCanonicalSnapshotV3 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_CANONICAL_SNAPSHOT_V3_SCHEMA_VERSION
            || self
                .sorted_joint_states
                .windows(2)
                .any(|pair| pair[0].joint_id >= pair[1].joint_id)
        {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        self.base.validate()
    }

    pub fn snapshot_hash_v3(&self) -> Result<ContentHash, PhysicsContractError> {
        self.validate()?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.physics-snapshot.v3\0");
        bytes.extend_from_slice(self.base.snapshot_hash()?.as_bytes());
        for hash in [
            self.catalog_v2_hash,
            self.physx_build_profile_hash,
            self.scene_profile_hash,
            self.bridge_abi_hash,
        ] {
            bytes.extend_from_slice(hash.as_bytes());
        }
        for joint in &self.sorted_joint_states {
            push_id(&mut bytes, &joint.joint_id)?;
            bytes.extend_from_slice(&joint.position_microradians.to_le_bytes());
            bytes.extend_from_slice(&joint.velocity_microradians_per_second.to_le_bytes());
            bytes.extend_from_slice(&joint.applied_effort_micronewton_metres.to_le_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsStepResultV2 {
    pub schema_version: u16,
    pub base: PhysicsStepResultV1,
    pub after_snapshot: PhysicsCanonicalSnapshotV3,
    pub applied_actuation_hash: ContentHash,
    pub contacts: ClosedPhysicsContactBatchV1,
}

impl PhysicsStepResultV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_STEP_RESULT_V2_SCHEMA_VERSION
            || self.base.after_snapshot_hash != self.after_snapshot.base.snapshot_hash()?
        {
            return Err(PhysicsContractError::ProfileMismatch);
        }
        self.after_snapshot.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsWorldCheckpointV2 {
    pub schema_version: u16,
    pub catalog: PhysicsWorldCatalogV2,
    pub snapshot: PhysicsCanonicalSnapshotV3,
}

impl PhysicsWorldCheckpointV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_WORLD_CHECKPOINT_V2_SCHEMA_VERSION {
            return Err(PhysicsContractError::UnsupportedVersion(u32::from(
                self.schema_version,
            )));
        }
        self.catalog.validate()?;
        self.snapshot.validate()?;
        if self.catalog.catalog_hash_v2()? != self.snapshot.catalog_v2_hash
            || self.catalog.physx_build_profile_hash != self.snapshot.physx_build_profile_hash
            || self.catalog.scene_profile_hash != self.snapshot.scene_profile_hash
            || self.catalog.bridge_abi_hash != self.snapshot.bridge_abi_hash
        {
            return Err(PhysicsContractError::ProfileMismatch);
        }
        Ok(())
    }

    pub fn checkpoint_hash_v2(&self) -> Result<ContentHash, PhysicsContractError> {
        self.validate()?;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"nextengine.physics-world-checkpoint.v2\0");
        bytes.extend_from_slice(self.catalog.catalog_hash_v2()?.as_bytes());
        bytes.extend_from_slice(self.snapshot.snapshot_hash_v3()?.as_bytes());
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

fn push_id(bytes: &mut Vec<u8>, id: &SchemaId) -> Result<(), PhysicsContractError> {
    let value = id.as_str().as_bytes();
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| PhysicsContractError::Canonicalization(CanonicalError::LengthOverflow))?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}
