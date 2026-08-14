use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_UTF8_NFC, CanonicalDecodeLimits, CanonicalError,
    CanonicalField, encode_canonical_segment,
};
use crate::ids::{ContentHash, SchemaId};

use super::codec::{
    decode_contract, decode_i64_vec3, encode_i64_vec3, field, field_u8, field_u16, field_u32,
    physics_contract_hash, profile_hash as canonical_profile_hash, read_u8, read_u16, read_u32,
    read_utf8, require_round_trip,
};
use super::{PHYSICS_OWNER_ID, PhysicsContractError, PhysicsMaterialDescriptorV1};

pub const PHYSICS_MATERIAL_DESCRIPTOR_V2_SCHEMA_VERSION: u16 = 2;
pub const PHYSICS_MATERIAL_COMBINE_PROFILE_V1_SCHEMA_VERSION: u16 = 1;

const MATERIAL_DESCRIPTOR_SCHEMA_ID: &str = "nextengine.physics-material-descriptor";
const MATERIAL_COMBINE_PROFILE_SCHEMA_ID: &str = "nextengine.physics-material-combine-profile";
const SEGMENT_V1: &str = "v1";
const SEGMENT_V2: &str = "v2";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsMaterialDescriptorV2 {
    pub schema_version: u16,
    pub base: PhysicsMaterialDescriptorV1,
    pub rolling_friction_q16: u32,
    pub spinning_friction_q16: u32,
    pub surface_velocity_micrometres_per_second: [i64; 3],
}

impl PhysicsMaterialDescriptorV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_MATERIAL_DESCRIPTOR_V2_SCHEMA_VERSION {
            return Err(PhysicsContractError::InvalidDescriptor);
        }
        self.base.validate()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            MATERIAL_DESCRIPTOR_SCHEMA_ID,
            SEGMENT_V2,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.base.canonical_record()?),
                field_u32(3, self.rolling_friction_q16),
                field_u32(4, self.spinning_friction_q16),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_vec3(self.surface_velocity_micrometres_per_second),
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            MATERIAL_DESCRIPTOR_SCHEMA_ID,
            SEGMENT_V2,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_STRUCT),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            base: PhysicsMaterialDescriptorV1::from_record(field(&segment, 2)?, limits)?,
            rolling_friction_q16: read_u32(&segment, 3)?,
            spinning_friction_q16: read_u32(&segment, 4)?,
            surface_velocity_micrometres_per_second: decode_i64_vec3(field(&segment, 5)?)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn descriptor_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-material-descriptor.v2\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsMaterialCombineRuleV1 {
    Minimum = 1,
    Maximum = 2,
    ArithmeticMeanTiesToEven = 3,
    ProductTiesToEvenClamped = 4,
}

impl PhysicsMaterialCombineRuleV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::Minimum),
            2 => Ok(Self::Maximum),
            3 => Ok(Self::ArithmeticMeanTiesToEven),
            4 => Ok(Self::ProductTiesToEvenClamped),
            _ => Err(PhysicsContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PhysicsSurfaceVelocityCombineRuleV1 {
    CanonicalParticipantOrder = 1,
}

impl PhysicsSurfaceVelocityCombineRuleV1 {
    fn from_tag(tag: u8) -> Result<Self, PhysicsContractError> {
        match tag {
            1 => Ok(Self::CanonicalParticipantOrder),
            _ => Err(PhysicsContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsMaterialCombineProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub profile_revision: u32,
    pub static_friction: PhysicsMaterialCombineRuleV1,
    pub dynamic_friction: PhysicsMaterialCombineRuleV1,
    pub restitution: PhysicsMaterialCombineRuleV1,
    pub rolling_friction: PhysicsMaterialCombineRuleV1,
    pub spinning_friction: PhysicsMaterialCombineRuleV1,
    pub surface_velocity: PhysicsSurfaceVelocityCombineRuleV1,
}

impl PhysicsMaterialCombineProfileV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_MATERIAL_COMBINE_PROFILE_V1_SCHEMA_VERSION
            || self.profile_revision == 0
        {
            return Err(PhysicsContractError::InvalidProfile);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            MATERIAL_COMBINE_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.profile_id.as_str().as_bytes().to_vec(),
                ),
                field_u32(3, self.profile_revision),
                field_u8(4, self.static_friction as u8),
                field_u8(5, self.dynamic_friction as u8),
                field_u8(6, self.restitution as u8),
                field_u8(7, self.rolling_friction as u8),
                field_u8(8, self.spinning_friction as u8),
                field_u8(9, self.surface_velocity as u8),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            MATERIAL_COMBINE_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_U8),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_U8),
                (7, CANONICAL_TYPE_U8),
                (8, CANONICAL_TYPE_U8),
                (9, CANONICAL_TYPE_U8),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            profile_id: SchemaId::new(read_utf8(&segment, 2)?)?,
            profile_revision: read_u32(&segment, 3)?,
            static_friction: PhysicsMaterialCombineRuleV1::from_tag(read_u8(&segment, 4)?)?,
            dynamic_friction: PhysicsMaterialCombineRuleV1::from_tag(read_u8(&segment, 5)?)?,
            restitution: PhysicsMaterialCombineRuleV1::from_tag(read_u8(&segment, 6)?)?,
            rolling_friction: PhysicsMaterialCombineRuleV1::from_tag(read_u8(&segment, 7)?)?,
            spinning_friction: PhysicsMaterialCombineRuleV1::from_tag(read_u8(&segment, 8)?)?,
            surface_velocity: PhysicsSurfaceVelocityCombineRuleV1::from_tag(read_u8(&segment, 9)?)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        canonical_profile_hash(&self.canonical_bytes()?)
    }
}
