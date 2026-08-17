#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeLimits, sha256};
use crate::ids::{AssetId, ContentHash, SchemaId, content_hash_from_bytes};
use crate::physics::{PhysicsGeometryV1, PhysicsPoseV1};

mod reference;
mod v2;
pub use reference::*;
pub use v2::*;

pub const BODY_SCHEMA_VERSION_V1: u16 = 1;
pub const BODY_INSTANCE_PROJECTION_VERSION_V1: u16 = 1;
pub const BODY_SCHEMA_ASSET_VERSION_V1: u16 = 1;
pub const BODY_PROJECTION_ROOTS_VERSION_V1: u16 = 1;
pub const BODY_SCHEMA_ASSET_SCHEMA_ID: &str = "nextengine.body-schema-asset.v1";
pub const BODY_PROJECTION_COMPILER_PROFILE_ID_V1: &str =
    "nextengine.body-projection-compiler.stage0.v1";
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

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, BodyContractError> {
        if bytes.len() > limits.max_total_bytes {
            return Err(BodyContractError::InputTooLarge);
        }
        let mut cursor = BodyCursor::new(bytes, limits);
        if cursor.bytes()? != b"nextengine.body-schema.v1\0" {
            return Err(BodyContractError::MalformedEncoding);
        }
        let value = Self {
            schema_version: cursor.u16()?,
            schema_id: cursor.id()?,
            schema_revision: cursor.u32()?,
            family_id: cursor.id()?,
            bodies: cursor.sequence(MAX_BODY_SCHEMA_BODIES, decode_body)?,
            joints: cursor.sequence(MAX_BODY_SCHEMA_JOINTS, decode_joint)?,
            actuators: cursor.sequence(MAX_BODY_SCHEMA_ACTUATORS, decode_actuator)?,
            effectors: cursor.sequence(MAX_BODY_SCHEMA_EFFECTORS, decode_effector)?,
            symmetry_pairs: cursor.sequence(MAX_BODY_SCHEMA_BODIES, |cursor| {
                Ok(BodySymmetryPairV1 {
                    left_id: cursor.id()?,
                    right_id: cursor.id()?,
                })
            })?,
            capability_ids: cursor.sequence(MAX_BODY_SCHEMA_BODIES, BodyCursor::id)?,
        };
        cursor.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(BodyContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodySchemaAssetV1 {
    pub schema_version: u16,
    pub asset_id: AssetId,
    pub record_revision: u32,
    pub compiler_profile_id: SchemaId,
    pub body_schema: BodySchemaV1,
}

impl BodySchemaAssetV1 {
    pub fn validate(&self) -> Result<(), BodyContractError> {
        if self.schema_version != BODY_SCHEMA_ASSET_VERSION_V1
            || self.asset_id == AssetId::default()
            || self.record_revision == 0
            || self.compiler_profile_id.as_str() != BODY_PROJECTION_COMPILER_PROFILE_ID_V1
        {
            return Err(BodyContractError::InvalidBounds);
        }
        self.body_schema.validate()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BodyContractError> {
        self.validate()?;
        let mut output = Vec::new();
        push_bytes(&mut output, b"nextengine.body-schema-asset.v1\0")?;
        push_u16(&mut output, self.schema_version);
        output.extend_from_slice(self.asset_id.as_bytes());
        push_u32(&mut output, self.record_revision);
        push_id(&mut output, &self.compiler_profile_id)?;
        push_bytes(&mut output, &self.body_schema.canonical_bytes()?)?;
        Ok(output)
    }

    pub fn record_sha256(&self) -> Result<ContentHash, BodyContractError> {
        Ok(content_hash_from_bytes(sha256(&self.canonical_bytes()?)))
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, BodyContractError> {
        if bytes.len() > limits.max_total_bytes {
            return Err(BodyContractError::InputTooLarge);
        }
        let mut cursor = BodyCursor::new(bytes, limits);
        if cursor.bytes()? != b"nextengine.body-schema-asset.v1\0" {
            return Err(BodyContractError::MalformedEncoding);
        }
        let schema_version = cursor.u16()?;
        let asset_id = AssetId::from_bytes(cursor.fixed()?);
        let record_revision = cursor.u32()?;
        let compiler_profile_id = cursor.id()?;
        let body_schema_bytes = cursor.bytes()?;
        let body_schema = BodySchemaV1::from_canonical_bytes(body_schema_bytes, limits)?;
        let value = Self {
            schema_version,
            asset_id,
            record_revision,
            compiler_profile_id,
            body_schema,
        };
        cursor.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(BodyContractError::NonCanonicalEncoding);
        }
        Ok(value)
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
        if self.body_schema_revision == 0
            || self.topology_revision == 0
            || self.body_schema_hash == ContentHash::default()
            || [
                self.morphology_hash,
                self.equipment_hash,
                self.stats_hash,
                self.damage_hash,
                self.fatigue_hash,
                self.attachment_hash,
            ]
            .contains(&ContentHash::default())
        {
            return Err(BodyContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn validate_against(&self, schema: &BodySchemaV1) -> Result<(), BodyContractError> {
        self.validate()?;
        schema.validate()?;
        if self.body_schema_id != schema.schema_id
            || self.body_schema_revision != schema.schema_revision
            || self.body_schema_hash != schema.schema_hash()?
        {
            return Err(BodyContractError::ProjectionMismatch);
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BodyProjectionRootsV1 {
    pub schema_version: u16,
    pub body_schema_hash: ContentHash,
    pub body_instance_projection_hash: ContentHash,
    pub compiler_profile_hash: ContentHash,
    pub physics_descriptor_root: ContentHash,
    pub observation_layout_hash: ContentHash,
    pub action_layout_hash: ContentHash,
    pub actuator_safety_root: ContentHash,
}

impl BodyProjectionRootsV1 {
    pub fn validate(&self) -> Result<(), BodyContractError> {
        if self.schema_version != BODY_PROJECTION_ROOTS_VERSION_V1
            || [
                self.body_schema_hash,
                self.body_instance_projection_hash,
                self.compiler_profile_hash,
                self.physics_descriptor_root,
                self.observation_layout_hash,
                self.action_layout_hash,
                self.actuator_safety_root,
            ]
            .contains(&ContentHash::default())
        {
            return Err(BodyContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn projection_root(&self) -> Result<ContentHash, BodyContractError> {
        self.validate()?;
        let mut bytes = b"nextengine.body-projection-roots.v1\0".to_vec();
        push_u16(&mut bytes, self.schema_version);
        for hash in [
            self.body_schema_hash,
            self.body_instance_projection_hash,
            self.compiler_profile_hash,
            self.physics_descriptor_root,
            self.observation_layout_hash,
            self.action_layout_hash,
            self.actuator_safety_root,
        ] {
            bytes.extend_from_slice(hash.as_bytes());
        }
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
    InputTooLarge,
    MalformedEncoding,
    NonCanonicalEncoding,
    ProjectionMismatch,
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
            Self::InputTooLarge => "BODY_SCHEMA_INPUT_TOO_LARGE",
            Self::MalformedEncoding => "BODY_SCHEMA_ENCODING_MALFORMED",
            Self::NonCanonicalEncoding => "BODY_SCHEMA_ENCODING_NON_CANONICAL",
            Self::ProjectionMismatch => "BODY_INSTANCE_PROJECTION_MISMATCH",
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

struct BodyCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    limits: CanonicalDecodeLimits,
}

impl<'a> BodyCursor<'a> {
    const fn new(bytes: &'a [u8], limits: CanonicalDecodeLimits) -> Self {
        Self {
            bytes,
            offset: 0,
            limits,
        }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], BodyContractError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(BodyContractError::MalformedEncoding)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(BodyContractError::MalformedEncoding)?;
        self.offset = end;
        Ok(value)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], BodyContractError> {
        self.take(N)?
            .try_into()
            .map_err(|_| BodyContractError::MalformedEncoding)
    }

    fn u8(&mut self) -> Result<u8, BodyContractError> {
        Ok(self.fixed::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, BodyContractError> {
        Ok(u16::from_le_bytes(self.fixed()?))
    }

    fn u32(&mut self) -> Result<u32, BodyContractError> {
        Ok(u32::from_le_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, BodyContractError> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, BodyContractError> {
        Ok(i32::from_le_bytes(self.fixed()?))
    }

    fn i64(&mut self) -> Result<i64, BodyContractError> {
        Ok(i64::from_le_bytes(self.fixed()?))
    }

    fn bytes(&mut self) -> Result<&'a [u8], BodyContractError> {
        let length =
            usize::try_from(self.u32()?).map_err(|_| BodyContractError::MalformedEncoding)?;
        if length > self.limits.max_field_payload_bytes {
            return Err(BodyContractError::InputTooLarge);
        }
        self.take(length)
    }

    fn id(&mut self) -> Result<SchemaId, BodyContractError> {
        let bytes = self.bytes()?;
        if bytes.len() > self.limits.max_identifier_bytes {
            return Err(BodyContractError::InputTooLarge);
        }
        let value = std::str::from_utf8(bytes).map_err(|_| BodyContractError::MalformedEncoding)?;
        SchemaId::new(value).map_err(|_| BodyContractError::MalformedEncoding)
    }

    fn sequence<T>(
        &mut self,
        contract_limit: usize,
        mut decode: impl FnMut(&mut BodyCursor<'a>) -> Result<T, BodyContractError>,
    ) -> Result<Vec<T>, BodyContractError> {
        let length =
            usize::try_from(self.u32()?).map_err(|_| BodyContractError::MalformedEncoding)?;
        if length > contract_limit || length > self.limits.max_sequence_items {
            return Err(BodyContractError::InputTooLarge);
        }
        (0..length).map(|_| decode(self)).collect()
    }

    fn finish(&self) -> Result<(), BodyContractError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(BodyContractError::MalformedEncoding)
        }
    }
}

fn decode_body(cursor: &mut BodyCursor<'_>) -> Result<BodyDefinitionV1, BodyContractError> {
    let body_id = cursor.id()?;
    let parent_body_id = match cursor.u8()? {
        0 => None,
        1 => Some(cursor.id()?),
        _ => return Err(BodyContractError::MalformedEncoding),
    };
    Ok(BodyDefinitionV1 {
        body_id,
        parent_body_id,
        local_bind_pose: decode_pose(cursor)?,
        mass_microkilograms: cursor.u64()?,
        center_of_mass_micrometres: decode_i64_3(cursor)?,
        inertia_microkilogram_metre_squared: [cursor.u64()?, cursor.u64()?, cursor.u64()?],
        colliders: cursor.sequence(MAX_BODY_SCHEMA_JOINTS, |cursor| {
            Ok(BodyColliderDefinitionV1 {
                collider_id: cursor.id()?,
                local_pose: decode_pose(cursor)?,
                geometry: decode_geometry(cursor)?,
                material_id: cursor.id()?,
            })
        })?,
    })
}

fn decode_joint(cursor: &mut BodyCursor<'_>) -> Result<BodyJointDefinitionV1, BodyContractError> {
    Ok(BodyJointDefinitionV1 {
        joint_id: cursor.id()?,
        parent_body_id: cursor.id()?,
        child_body_id: cursor.id()?,
        parent_frame: decode_pose(cursor)?,
        child_frame: decode_pose(cursor)?,
        axis_q1_30: [cursor.i32()?, cursor.i32()?, cursor.i32()?],
        limit_min_microradians: cursor.i64()?,
        limit_max_microradians: cursor.i64()?,
        maximum_velocity_microradians_per_second: cursor.u64()?,
    })
}

fn decode_actuator(
    cursor: &mut BodyCursor<'_>,
) -> Result<BodyActuatorDefinitionV1, BodyContractError> {
    Ok(BodyActuatorDefinitionV1 {
        actuator_id: cursor.id()?,
        joint_id: cursor.id()?,
        neutral_position_microradians: cursor.i64()?,
        stiffness_q16: cursor.u64()?,
        damping_q16: cursor.u64()?,
        maximum_effort_micronewton_metres: cursor.u64()?,
        maximum_effort_rate_micronewton_metres_per_second: cursor.u64()?,
    })
}

fn decode_effector(
    cursor: &mut BodyCursor<'_>,
) -> Result<BodyEffectorDefinitionV1, BodyContractError> {
    Ok(BodyEffectorDefinitionV1 {
        effector_id: cursor.id()?,
        body_id: cursor.id()?,
        local_pose: decode_pose(cursor)?,
        semantic_role_id: cursor.id()?,
    })
}

fn decode_pose(cursor: &mut BodyCursor<'_>) -> Result<PhysicsPoseV1, BodyContractError> {
    Ok(PhysicsPoseV1 {
        translation_micrometres: decode_i64_3(cursor)?,
        rotation_q1_30: [cursor.i32()?, cursor.i32()?, cursor.i32()?, cursor.i32()?],
    })
}

fn decode_i64_3(cursor: &mut BodyCursor<'_>) -> Result<[i64; 3], BodyContractError> {
    Ok([cursor.i64()?, cursor.i64()?, cursor.i64()?])
}

fn decode_geometry(cursor: &mut BodyCursor<'_>) -> Result<PhysicsGeometryV1, BodyContractError> {
    match cursor.u8()? {
        1 => Ok(PhysicsGeometryV1::Box {
            half_extents_micrometres: decode_i64_3(cursor)?,
        }),
        2 => Ok(PhysicsGeometryV1::Sphere {
            radius_micrometres: cursor.i64()?,
        }),
        3 => Ok(PhysicsGeometryV1::Capsule {
            radius_micrometres: cursor.i64()?,
            half_segment_micrometres: cursor.i64()?,
        }),
        _ => Err(BodyContractError::UnsupportedGeometry),
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
mod tests;
