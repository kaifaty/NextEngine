use crate::canonical::{
    CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_ID128, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U16, CANONICAL_TYPE_U32,
    CANONICAL_TYPE_U64, CanonicalCursor, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    encode_canonical_segment,
};
use crate::{CommandId, ContentHash, PersistentId, PhysicsContactId, PhysicsWorldId};

use super::codec::*;
use super::contact::ClosedPhysicsContactBatchV1;
use super::error::PhysicsContractError;
use super::primitives::{PhysicsBodyIdV1, PhysicsPoseV1};
use super::{
    PHYSICAL_COMMAND_SCHEMA_ID, PHYSICAL_EVENT_SCHEMA_ID, PHYSICS_OWNER_ID,
    PHYSICS_STEP_INPUT_SCHEMA_VERSION, SEGMENT_V1,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PhysicalCommandV1 {
    SetCapsuleLocomotionIntent { direction_q15: [i16; 2] },
}

impl PhysicalCommandV1 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        match self {
            Self::SetCapsuleLocomotionIntent { direction_q15 } => {
                let [x, z] = *direction_q15;
                if !matches!(
                    (x, z),
                    (0, 0) | (32_767, 0) | (-32_767, 0) | (0, 32_767) | (0, -32_767)
                ) {
                    return Err(PhysicsContractError::DirectionOutOfProfile);
                }
            }
        }
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut tagged = vec![1];
        let mut direction = Vec::with_capacity(4);
        let Self::SetCapsuleLocomotionIntent { direction_q15 } = self;
        direction.extend_from_slice(&direction_q15[0].to_le_bytes());
        direction.extend_from_slice(&direction_q15[1].to_le_bytes());
        tagged.extend_from_slice(&nested(CANONICAL_TYPE_BYTES, &direction)?);
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            PHYSICAL_COMMAND_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, 1),
                CanonicalField::new(2, CANONICAL_TYPE_TAGGED_UNION, tagged),
            ],
        )
    }

    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            PHYSICS_OWNER_ID,
            PHYSICAL_COMMAND_SCHEMA_ID,
            SEGMENT_V1,
            &[(1, CANONICAL_TYPE_U16), (2, CANONICAL_TYPE_TAGGED_UNION)],
        )?;
        if read_u16(&segment, 1)? != 1 {
            return Err(PhysicsContractError::UnsupportedVersion(1));
        }
        let mut cursor = CanonicalCursor::new(field(&segment, 2)?);
        let tag = cursor.read_u8()?;
        let (nested_tag, payload) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        if tag != 1 || nested_tag != CANONICAL_TYPE_BYTES || payload.len() != 4 {
            return Err(PhysicsContractError::UnknownTag(tag));
        }
        let value = Self::SetCapsuleLocomotionIntent {
            direction_q15: [
                i16::from_le_bytes(exact(&payload[..2])?),
                i16::from_le_bytes(exact(&payload[2..])?),
            ],
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_payload_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AcceptedLocomotionIntentV2 {
    pub causal_command_id: CommandId,
    pub controlled_target_id: PersistentId,
    pub body_id: PhysicsBodyIdV1,
    pub target_gameplay_tick: u64,
    pub direction_q15: [i16; 2],
}

impl AcceptedLocomotionIntentV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.controlled_target_id != self.body_id.subject_id {
            return Err(PhysicsContractError::ReferenceInvalid);
        }
        PhysicalCommandV1::SetCapsuleLocomotionIntent {
            direction_q15: self.direction_q15,
        }
        .validate()
    }

    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut direction = Vec::with_capacity(4);
        direction.extend_from_slice(&self.direction_q15[0].to_le_bytes());
        direction.extend_from_slice(&self.direction_q15[1].to_le_bytes());
        encode_struct([
            field_id(1, self.causal_command_id.as_bytes()),
            field_id(2, self.controlled_target_id.as_bytes()),
            CanonicalField::new(3, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
            field_u64(4, self.target_gameplay_tick),
            CanonicalField::new(5, CANONICAL_TYPE_BYTES, direction),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, PhysicsContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_ID128),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_STRUCT),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_BYTES),
            ],
        )?;
        let direction = &field_from(&fields, 5)?.payload;
        if direction.len() != 4 {
            return Err(PhysicsContractError::FieldLength);
        }
        let value = Self {
            causal_command_id: CommandId::from_bytes(exact(&field_from(&fields, 1)?.payload)?),
            controlled_target_id: PersistentId::from_bytes(exact(
                &field_from(&fields, 2)?.payload,
            )?),
            body_id: PhysicsBodyIdV1::from_record(&field_from(&fields, 3)?.payload, limits)?,
            target_gameplay_tick: read_u64_fields(&fields, 4)?,
            direction_q15: [
                i16::from_le_bytes(exact(&direction[..2])?),
                i16::from_le_bytes(exact(&direction[2..])?),
            ],
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsStepInputV2 {
    pub schema_version: u16,
    pub world_id: PhysicsWorldId,
    pub expected_world_revision: u64,
    pub expected_snapshot_hash: ContentHash,
    pub expected_catalog_hash: ContentHash,
    pub gameplay_tick: u64,
    pub first_physics_tick: u64,
    pub physics_substeps: u32,
    pub accepted_intents: Vec<AcceptedLocomotionIntentV2>,
}

impl PhysicsStepInputV2 {
    pub fn validate(&self) -> Result<(), PhysicsContractError> {
        if self.schema_version != PHYSICS_STEP_INPUT_SCHEMA_VERSION || self.physics_substeps == 0 {
            return Err(PhysicsContractError::InvalidProfile);
        }
        if self.accepted_intents.windows(2).any(|pair| {
            (pair[0].body_id, pair[0].causal_command_id)
                >= (pair[1].body_id, pair[1].causal_command_id)
        }) {
            return Err(PhysicsContractError::NonCanonicalOrder);
        }
        for intent in &self.accepted_intents {
            intent.validate()?;
            if intent.target_gameplay_tick != self.gameplay_tick {
                return Err(PhysicsContractError::InvalidProfile);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.physics-step-input",
            "v2",
            [
                field_u16(1, self.schema_version),
                field_id(2, self.world_id.as_bytes()),
                field_u64(3, self.expected_world_revision),
                field_hash(4, self.expected_snapshot_hash),
                field_hash(5, self.expected_catalog_hash),
                field_u64(6, self.gameplay_tick),
                field_u64(7, self.first_physics_tick),
                field_u32(8, self.physics_substeps),
                CanonicalField::new(
                    9,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.accepted_intents
                            .iter()
                            .map(AcceptedLocomotionIntentV2::canonical_record)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
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
            "nextengine.physics-step-input",
            "v2",
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_HASH256),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_U32),
                (9, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            world_id: PhysicsWorldId::from_bytes(exact(field(&segment, 2)?)?),
            expected_world_revision: read_u64(&segment, 3)?,
            expected_snapshot_hash: read_hash(&segment, 4)?,
            expected_catalog_hash: read_hash(&segment, 5)?,
            gameplay_tick: read_u64(&segment, 6)?,
            first_physics_tick: read_u64(&segment, 7)?,
            physics_substeps: u32::from_le_bytes(exact(field(&segment, 8)?)?),
            accepted_intents: decode_sequence(field(&segment, 9)?, limits)?
                .into_iter()
                .map(|record| AcceptedLocomotionIntentV2::from_record(&record, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn input_hash(&self) -> Result<ContentHash, CanonicalError> {
        physics_contract_hash(
            b"nextengine.physics-step-input.v2\0",
            &self.canonical_bytes()?,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedLocomotionResultV1 {
    pub causal_command_id: CommandId,
    pub body_id: PhysicsBodyIdV1,
    pub requested_direction_q15: [i16; 2],
    pub before_state_hash: ContentHash,
    pub after_state_hash: ContentHash,
    pub applied_displacement_micrometres: [i64; 3],
    pub related_contact_ids: Vec<PhysicsContactId>,
}

impl AppliedLocomotionResultV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let mut direction = Vec::with_capacity(4);
        direction.extend_from_slice(&self.requested_direction_q15[0].to_le_bytes());
        direction.extend_from_slice(&self.requested_direction_q15[1].to_le_bytes());
        encode_canonical_segment(
            PHYSICS_OWNER_ID,
            "nextengine.applied-locomotion-result",
            SEGMENT_V1,
            [
                field_id(1, self.causal_command_id.as_bytes()),
                CanonicalField::new(2, CANONICAL_TYPE_STRUCT, self.body_id.canonical_record()?),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, direction),
                field_hash(4, self.before_state_hash),
                field_hash(5, self.after_state_hash),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_BYTES,
                    encode_i64_vec3(self.applied_displacement_micrometres),
                ),
                CanonicalField::new(
                    7,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.related_contact_ids
                            .iter()
                            .map(|id| id.as_bytes().to_vec())
                            .collect(),
                    )?,
                ),
            ],
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicsStepResultV1 {
    pub step_input_hash: ContentHash,
    pub before_snapshot_hash: ContentHash,
    pub after_snapshot_hash: ContentHash,
    pub applied_locomotion: Vec<AppliedLocomotionResultV1>,
    pub contact_batch: ClosedPhysicsContactBatchV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicalEventV1 {
    CapsuleStepApplied {
        body_id: PersistentId,
        physics_tick: u64,
        before: PhysicsPoseV1,
        after: PhysicsPoseV1,
    },
}

impl PhysicalEventV1 {
    #[must_use]
    pub const fn schema_id(&self) -> &'static str {
        PHYSICAL_EVENT_SCHEMA_ID
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        match self {
            Self::CapsuleStepApplied {
                body_id,
                physics_tick,
                before,
                after,
            } => encode_canonical_segment(
                PHYSICS_OWNER_ID,
                PHYSICAL_EVENT_SCHEMA_ID,
                SEGMENT_V1,
                [
                    field_id(1, body_id.as_bytes()),
                    field_u64(2, *physics_tick),
                    CanonicalField::new(3, CANONICAL_TYPE_STRUCT, before.canonical_record()?),
                    CanonicalField::new(4, CANONICAL_TYPE_STRUCT, after.canonical_record()?),
                ],
            ),
        }
    }
}
