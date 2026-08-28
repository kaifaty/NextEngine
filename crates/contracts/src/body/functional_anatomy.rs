use super::{
    BodyContractError, BodyCursor, BodySchemaV1, push_bytes, push_id, push_sequence, push_u16,
    push_u32, push_u64, require_strict_order,
};
use crate::canonical::{CanonicalDecodeLimits, sha256};
use crate::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};

pub const FUNCTIONAL_ANATOMY_PROFILE_VERSION_V1: u16 = 1;
pub const BODY_CAPABILITY_ENVELOPE_VERSION_V1: u16 = 1;
pub const FUNCTIONAL_CAPACITY_FULL_Q16: u16 = u16::MAX;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FunctionalActuatorDirectionV1 {
    Negative = 1,
    Positive = 2,
}

impl FunctionalActuatorDirectionV1 {
    fn from_tag(tag: u8) -> Result<Self, BodyContractError> {
        match tag {
            1 => Ok(Self::Negative),
            2 => Ok(Self::Positive),
            _ => Err(BodyContractError::MalformedEncoding),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionalAnatomyProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub profile_revision: u32,
    pub body_schema_hash: ContentHash,
    pub region_id: SchemaId,
    pub functional_group_id: SchemaId,
    pub actuator_id: SchemaId,
    pub affected_direction: FunctionalActuatorDirectionV1,
    pub partial_capacity_q16: u16,
    pub post_repair_capacity_q16: u16,
}

impl FunctionalAnatomyProfileV1 {
    pub fn validate(&self) -> Result<(), BodyContractError> {
        if self.schema_version != FUNCTIONAL_ANATOMY_PROFILE_VERSION_V1 {
            return Err(BodyContractError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.profile_revision == 0
            || self.body_schema_hash == ContentHash::default()
            || self.partial_capacity_q16 == 0
            || self.partial_capacity_q16 == FUNCTIONAL_CAPACITY_FULL_Q16
            || self.post_repair_capacity_q16 < self.partial_capacity_q16
            || self.post_repair_capacity_q16 == FUNCTIONAL_CAPACITY_FULL_Q16
        {
            return Err(BodyContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn validate_against(&self, schema: &BodySchemaV1) -> Result<(), BodyContractError> {
        self.validate()?;
        schema.validate()?;
        if self.body_schema_hash != schema.schema_hash()?
            || schema
                .actuators
                .binary_search_by(|actuator| actuator.actuator_id.cmp(&self.actuator_id))
                .is_err()
        {
            return Err(BodyContractError::InvalidReference);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BodyContractError> {
        self.validate()?;
        let mut output = Vec::new();
        push_bytes(&mut output, b"nextengine.functional-anatomy-profile.v1\0")?;
        push_u16(&mut output, self.schema_version);
        push_id(&mut output, &self.profile_id)?;
        push_u32(&mut output, self.profile_revision);
        output.extend_from_slice(self.body_schema_hash.as_bytes());
        push_id(&mut output, &self.region_id)?;
        push_id(&mut output, &self.functional_group_id)?;
        push_id(&mut output, &self.actuator_id)?;
        output.push(self.affected_direction as u8);
        push_u16(&mut output, self.partial_capacity_q16);
        push_u16(&mut output, self.post_repair_capacity_q16);
        Ok(output)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, BodyContractError> {
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
        if cursor.bytes()? != b"nextengine.functional-anatomy-profile.v1\0" {
            return Err(BodyContractError::MalformedEncoding);
        }
        let value = Self {
            schema_version: cursor.u16()?,
            profile_id: cursor.id()?,
            profile_revision: cursor.u32()?,
            body_schema_hash: ContentHash::from_bytes(cursor.fixed()?),
            region_id: cursor.id()?,
            functional_group_id: cursor.id()?,
            actuator_id: cursor.id()?,
            affected_direction: FunctionalActuatorDirectionV1::from_tag(cursor.u8()?)?,
            partial_capacity_q16: cursor.u16()?,
            post_repair_capacity_q16: cursor.u16()?,
        };
        cursor.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(BodyContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BodyActuatorCapabilityV1 {
    pub actuator_id: SchemaId,
    pub negative_capacity_q16: u16,
    pub positive_capacity_q16: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BodyCapabilityEnvelopeV1 {
    pub schema_version: u16,
    pub subject_id: PersistentId,
    pub body_schema_hash: ContentHash,
    pub anatomy_profile_hash: ContentHash,
    pub condition_state_hash: ContentHash,
    pub condition_revision: u64,
    pub actuator_capabilities: Vec<BodyActuatorCapabilityV1>,
}

impl BodyCapabilityEnvelopeV1 {
    pub fn validate(&self) -> Result<(), BodyContractError> {
        if self.schema_version != BODY_CAPABILITY_ENVELOPE_VERSION_V1 {
            return Err(BodyContractError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.subject_id == PersistentId::default()
            || self.body_schema_hash == ContentHash::default()
            || self.anatomy_profile_hash == ContentHash::default()
            || self.condition_state_hash == ContentHash::default()
            || self.actuator_capabilities.is_empty()
        {
            return Err(BodyContractError::InvalidBounds);
        }
        require_strict_order(
            self.actuator_capabilities
                .iter()
                .map(|value| &value.actuator_id),
        )?;
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, BodyContractError> {
        self.validate()?;
        let mut output = Vec::new();
        push_bytes(&mut output, b"nextengine.body-capability-envelope.v1\0")?;
        push_u16(&mut output, self.schema_version);
        output.extend_from_slice(self.subject_id.as_bytes());
        output.extend_from_slice(self.body_schema_hash.as_bytes());
        output.extend_from_slice(self.anatomy_profile_hash.as_bytes());
        output.extend_from_slice(self.condition_state_hash.as_bytes());
        push_u64(&mut output, self.condition_revision);
        push_sequence(&mut output, &self.actuator_capabilities, |value, bytes| {
            push_id(bytes, &value.actuator_id)?;
            push_u16(bytes, value.negative_capacity_q16);
            push_u16(bytes, value.positive_capacity_q16);
            Ok(())
        })?;
        Ok(output)
    }

    pub fn envelope_hash(&self) -> Result<ContentHash, BodyContractError> {
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
        if cursor.bytes()? != b"nextengine.body-capability-envelope.v1\0" {
            return Err(BodyContractError::MalformedEncoding);
        }
        let value = Self {
            schema_version: cursor.u16()?,
            subject_id: PersistentId::from_bytes(cursor.fixed()?),
            body_schema_hash: ContentHash::from_bytes(cursor.fixed()?),
            anatomy_profile_hash: ContentHash::from_bytes(cursor.fixed()?),
            condition_state_hash: ContentHash::from_bytes(cursor.fixed()?),
            condition_revision: cursor.u64()?,
            actuator_capabilities: cursor.sequence(256, |cursor| {
                Ok(BodyActuatorCapabilityV1 {
                    actuator_id: cursor.id()?,
                    negative_capacity_q16: cursor.u16()?,
                    positive_capacity_q16: cursor.u16()?,
                })
            })?,
        };
        cursor.finish()?;
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(BodyContractError::NonCanonicalEncoding);
        }
        Ok(value)
    }
}
