use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{
    CANONICAL_TYPE_BOOL, CANONICAL_TYPE_BYTES, CANONICAL_TYPE_HASH256, CANONICAL_TYPE_I16,
    CANONICAL_TYPE_ID128, CANONICAL_TYPE_MAP, CANONICAL_TYPE_OPTIONAL, CANONICAL_TYPE_SEQUENCE,
    CANONICAL_TYPE_STRUCT, CANONICAL_TYPE_TAGGED_UNION, CANONICAL_TYPE_U8, CANONICAL_TYPE_U16,
    CANONICAL_TYPE_U32, CANONICAL_TYPE_U64, CANONICAL_TYPE_UTF8_NFC, CanonicalCursor,
    CanonicalDecodeError, CanonicalDecodeLimits, CanonicalError, CanonicalField,
    DecodedCanonicalSegment, decode_canonical_segment, encode_canonical_segment, sha256,
};
use crate::{
    CommandId, CommandPhase, CommandStreamId, ContentHash, InputSourceId, IssuerPrincipal,
    PersistentId, SchemaId, WorldCommand, WorldNamespaceId, content_hash_from_bytes,
};

pub const RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION: u16 = 1;
pub const TICK_RATE_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION: u16 = 1;
pub const PLAYER_ACTION_FRAME_SCHEMA_VERSION: u16 = 1;
pub const INPUT_SAMPLE_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_ASSIGNMENT_SCHEMA_VERSION: u16 = 1;
pub const CLOSED_INGRESS_BATCH_SCHEMA_VERSION: u16 = 1;
pub const CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION: u16 = 2;
pub const PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const INGRESS_CHECKPOINT_SCHEMA_VERSION: u16 = 1;

pub const PLAYER_ACTION_FRAME_SCHEMA_ID: &str = "nextengine.player-action-frame";
pub const PLAYER_ACTION_SOURCE_CLASS: &str = "nextengine.input.player-action";
pub const CORE_MOVE_ACTION_ID: &str = "nextengine.action.move";
pub const CORE_INTERACT_ACTION_ID: &str = "nextengine.action.interact";
pub const PLAYER_INTERACTION_SYSTEM_ID: &str = "nextengine.system.player-interaction";
pub const MAX_PLAYER_ACTIONS_PER_FRAME: usize = 64;

#[must_use]
pub fn core_player_action_map_v1_hash() -> ContentHash {
    let mut preimage = b"nextengine.core-player-action-map.v1\0".to_vec();
    for action_id in [CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID] {
        preimage.extend_from_slice(
            &u32::try_from(action_id.len())
                .expect("built-in action identifier length fits u32")
                .to_le_bytes(),
        );
        preimage.extend_from_slice(action_id.as_bytes());
    }
    ContentHash::from_bytes(sha256(&preimage))
}

const RUNTIME_OWNER_ID: &str = "nextengine.runtime";
const INPUT_OWNER_ID: &str = "nextengine.input";
const RUNTIME_ADMISSION_LIMITS_SCHEMA_ID: &str = "nextengine.runtime-admission-limits";
const TICK_RATE_PROFILE_SCHEMA_ID: &str = "nextengine.tick-rate-profile";
const INGRESS_ASSIGNMENT_PROFILE_SCHEMA_ID: &str = "nextengine.ingress-assignment-profile";
const INPUT_SAMPLE_SCHEMA_ID: &str = "nextengine.input-sample";
const INGRESS_ASSIGNMENT_SCHEMA_ID: &str = "nextengine.ingress-assignment";
const INGRESS_RECEIPT_SCHEMA_ID: &str = "nextengine.ingress-equivalence-receipt";
const CLOSED_INGRESS_BODY_SCHEMA_ID: &str = "nextengine.closed-ingress-batch-body";
const CLOSED_INGRESS_SCHEMA_ID: &str = "nextengine.closed-ingress-batch";
const CLOSED_COMMAND_BODY_SCHEMA_ID: &str = "nextengine.closed-command-admission-batch-body";
const CLOSED_COMMAND_SCHEMA_ID: &str = "nextengine.closed-command-admission-batch";
const PLAYER_CONTROLLER_REGISTRY_SCHEMA_ID: &str = "nextengine.player-controller-registry";
const INPUT_MAPPING_RECEIPT_SCHEMA_ID: &str = "nextengine.input-mapping-receipt";
const INGRESS_CHECKPOINT_SCHEMA_ID: &str = "nextengine.ingress-checkpoint";
const SEGMENT_V1: &str = "v1";
const SEGMENT_V2: &str = "v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAdmissionLimitsV1 {
    pub schema_version: u16,
    pub max_namespaced_id_bytes: u32,
    pub max_identity_string_bytes: u32,
    pub max_command_body_bytes: u32,
    pub max_capability_claims: u32,
    pub max_preconditions: u32,
    pub max_commands_per_closed_batch: u32,
    pub max_unique_collision_candidates: u32,
    pub max_input_payload_bytes: u32,
    pub max_task_result_bytes: u32,
    pub max_pending_per_stream: u32,
    pub receipt_window_size: u32,
    pub max_identity_occurrences: u64,
}

impl Default for RuntimeAdmissionLimitsV1 {
    fn default() -> Self {
        Self {
            schema_version: RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION,
            max_namespaced_id_bytes: 255,
            max_identity_string_bytes: 4_096,
            max_command_body_bytes: 262_144,
            max_capability_claims: 64,
            max_preconditions: 128,
            max_commands_per_closed_batch: 4_096,
            max_unique_collision_candidates: 64,
            max_input_payload_bytes: 65_536,
            max_task_result_bytes: 4_194_304,
            max_pending_per_stream: 256,
            receipt_window_size: 4_096,
            max_identity_occurrences: 67_108_864,
        }
    }
}

impl RuntimeAdmissionLimitsV1 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != RUNTIME_ADMISSION_LIMITS_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "runtime admission limits",
                version: u32::from(self.schema_version),
            });
        }
        let defaults = Self::default();
        let within = self.max_namespaced_id_bytes <= defaults.max_namespaced_id_bytes
            && self.max_identity_string_bytes <= defaults.max_identity_string_bytes
            && self.max_command_body_bytes <= defaults.max_command_body_bytes
            && self.max_capability_claims <= defaults.max_capability_claims
            && self.max_preconditions <= defaults.max_preconditions
            && self.max_commands_per_closed_batch <= defaults.max_commands_per_closed_batch
            && self.max_unique_collision_candidates <= defaults.max_unique_collision_candidates
            && self.max_input_payload_bytes <= defaults.max_input_payload_bytes
            && self.max_task_result_bytes <= defaults.max_task_result_bytes
            && self.max_identity_occurrences <= defaults.max_identity_occurrences;
        if !within
            || self.max_namespaced_id_bytes == 0
            || self.max_identity_string_bytes == 0
            || self.max_command_body_bytes == 0
            || self.max_commands_per_closed_batch == 0
            || self.max_unique_collision_candidates == 0
            || self.max_input_payload_bytes == 0
            || self.max_task_result_bytes == 0
            || self.max_identity_occurrences == 0
            || self.max_pending_per_stream != 256
            || self.receipt_window_size != 4_096
        {
            return Err(InputContractError::InvalidProfile);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            RUNTIME_ADMISSION_LIMITS_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u32(2, self.max_namespaced_id_bytes),
                field_u32(3, self.max_identity_string_bytes),
                field_u32(4, self.max_command_body_bytes),
                field_u32(5, self.max_capability_claims),
                field_u32(6, self.max_preconditions),
                field_u32(7, self.max_commands_per_closed_batch),
                field_u32(8, self.max_unique_collision_candidates),
                field_u32(9, self.max_input_payload_bytes),
                field_u32(10, self.max_task_result_bytes),
                field_u32(11, self.max_pending_per_stream),
                field_u32(12, self.receipt_window_size),
                field_u64(13, self.max_identity_occurrences),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            RUNTIME_ADMISSION_LIMITS_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_U32),
                (6, CANONICAL_TYPE_U32),
                (7, CANONICAL_TYPE_U32),
                (8, CANONICAL_TYPE_U32),
                (9, CANONICAL_TYPE_U32),
                (10, CANONICAL_TYPE_U32),
                (11, CANONICAL_TYPE_U32),
                (12, CANONICAL_TYPE_U32),
                (13, CANONICAL_TYPE_U64),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            max_namespaced_id_bytes: read_u32(&segment, 2)?,
            max_identity_string_bytes: read_u32(&segment, 3)?,
            max_command_body_bytes: read_u32(&segment, 4)?,
            max_capability_claims: read_u32(&segment, 5)?,
            max_preconditions: read_u32(&segment, 6)?,
            max_commands_per_closed_batch: read_u32(&segment, 7)?,
            max_unique_collision_candidates: read_u32(&segment, 8)?,
            max_input_payload_bytes: read_u32(&segment, 9)?,
            max_task_result_bytes: read_u32(&segment, 10)?,
            max_pending_per_stream: read_u32(&segment, 11)?,
            receipt_window_size: read_u32(&segment, 12)?,
            max_identity_occurrences: read_u64(&segment, 13)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        hash_canonical_profile(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickRateProfileV1 {
    pub schema_version: u16,
    pub gameplay_hz: u32,
    pub physics_substeps_per_gameplay_tick: u32,
    pub motor_period_physics_substeps: u32,
    pub first_gameplay_tick: u64,
}

impl TickRateProfileV1 {
    #[must_use]
    pub const fn at_30_hz() -> Self {
        Self {
            schema_version: TICK_RATE_PROFILE_SCHEMA_VERSION,
            gameplay_hz: 30,
            physics_substeps_per_gameplay_tick: 2,
            motor_period_physics_substeps: 1,
            first_gameplay_tick: 0,
        }
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        let physics_hz = self
            .gameplay_hz
            .checked_mul(self.physics_substeps_per_gameplay_tick)
            .ok_or(InputContractError::InvalidProfile)?;
        if self.schema_version != TICK_RATE_PROFILE_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "tick rate profile",
                version: u32::from(self.schema_version),
            });
        }
        if !matches!(self.gameplay_hz, 20 | 30 | 60)
            || !matches!(physics_hz, 60 | 120 | 240)
            || self.physics_substeps_per_gameplay_tick == 0
            || self.motor_period_physics_substeps == 0
            || !self
                .physics_substeps_per_gameplay_tick
                .is_multiple_of(self.motor_period_physics_substeps)
        {
            return Err(InputContractError::InvalidProfile);
        }
        Ok(())
    }

    #[must_use]
    pub fn physics_hz(&self) -> u32 {
        self.gameplay_hz * self.physics_substeps_per_gameplay_tick
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            TICK_RATE_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u32(2, self.gameplay_hz),
                field_u32(3, self.physics_substeps_per_gameplay_tick),
                field_u32(4, self.motor_period_physics_substeps),
                field_u64(5, self.first_gameplay_tick),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            TICK_RATE_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U32),
                (3, CANONICAL_TYPE_U32),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_U64),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            gameplay_hz: read_u32(&segment, 2)?,
            physics_substeps_per_gameplay_tick: read_u32(&segment, 3)?,
            motor_period_physics_substeps: read_u32(&segment, 4)?,
            first_gameplay_tick: read_u64(&segment, 5)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        hash_canonical_profile(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IngressAssignmentProfileV1 {
    pub schema_version: u16,
    pub cutoff_policy: u8,
    pub input_sort: u8,
    pub completion_sort: u8,
    pub collision_policy: u8,
    pub admission_limits_hash: ContentHash,
}

impl IngressAssignmentProfileV1 {
    pub fn core_v1(limits: &RuntimeAdmissionLimitsV1) -> Result<Self, CanonicalError> {
        Ok(Self {
            schema_version: INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION,
            cutoff_policy: 1,
            input_sort: 1,
            completion_sort: 1,
            collision_policy: 1,
            admission_limits_hash: limits.profile_hash()?,
        })
    }

    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != INGRESS_ASSIGNMENT_PROFILE_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "ingress assignment profile",
                version: u32::from(self.schema_version),
            });
        }
        if self.cutoff_policy != 1
            || self.input_sort != 1
            || self.completion_sort != 1
            || self.collision_policy != 1
        {
            return Err(InputContractError::InvalidProfile);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_ASSIGNMENT_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u8(2, self.cutoff_policy),
                field_u8(3, self.input_sort),
                field_u8(4, self.completion_sort),
                field_u8(5, self.collision_policy),
                field_hash(6, self.admission_limits_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_ASSIGNMENT_PROFILE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_U8),
                (4, CANONICAL_TYPE_U8),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            cutoff_policy: read_u8(&segment, 2)?,
            input_sort: read_u8(&segment, 3)?,
            completion_sort: read_u8(&segment, 4)?,
            collision_policy: read_u8(&segment, 5)?,
            admission_limits_hash: read_hash(&segment, 6)?,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }

    pub fn profile_hash(&self) -> Result<ContentHash, CanonicalError> {
        hash_canonical_profile(&self.canonical_bytes()?)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PlayerActionPhaseV1 {
    Started = 1,
    Performed = 2,
    Completed = 3,
    Cancelled = 4,
}

impl PlayerActionPhaseV1 {
    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            1 => Ok(Self::Started),
            2 => Ok(Self::Performed),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Cancelled),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlayerActionValueV1 {
    Digital(bool),
    ScalarQ15(i16),
    Vector2Q15([i16; 2]),
}

impl PlayerActionValueV1 {
    fn canonical_tagged_payload(self) -> Result<Vec<u8>, CanonicalError> {
        let (tag, nested_type, payload) = match self {
            Self::Digital(value) => (1, CANONICAL_TYPE_BOOL, vec![u8::from(value)]),
            Self::ScalarQ15(value) => (2, CANONICAL_TYPE_I16, value.to_le_bytes().to_vec()),
            Self::Vector2Q15(value) => {
                let mut payload = Vec::with_capacity(4);
                payload.extend_from_slice(&value[0].to_le_bytes());
                payload.extend_from_slice(&value[1].to_le_bytes());
                (3, CANONICAL_TYPE_BYTES, payload)
            }
        };
        let mut bytes = vec![tag];
        bytes.extend_from_slice(&encode_nested(nested_type, &payload)?);
        Ok(bytes)
    }

    fn from_tagged_payload(
        payload: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let mut cursor = CanonicalCursor::new(payload);
        let tag = cursor.read_u8()?;
        let (nested_type, nested) = read_nested(&mut cursor, limits)?;
        cursor.finish()?;
        match tag {
            1 if nested_type == CANONICAL_TYPE_BOOL && nested.len() == 1 => match nested[0] {
                0 => Ok(Self::Digital(false)),
                1 => Ok(Self::Digital(true)),
                _ => Err(InputContractError::InvalidValue),
            },
            2 if nested_type == CANONICAL_TYPE_I16 => {
                Ok(Self::ScalarQ15(i16::from_le_bytes(exact(nested)?)))
            }
            3 if nested_type == CANONICAL_TYPE_BYTES && nested.len() == 4 => {
                Ok(Self::Vector2Q15([
                    i16::from_le_bytes(exact(&nested[..2])?),
                    i16::from_le_bytes(exact(&nested[2..])?),
                ]))
            }
            1..=3 => Err(InputContractError::FieldType),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlayerActionV1 {
    pub action_id: SchemaId,
    pub phase: PlayerActionPhaseV1,
    pub value: PlayerActionValueV1,
    pub semantic_occurrence_ordinal: u32,
}

impl PlayerActionV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(
                1,
                CANONICAL_TYPE_UTF8_NFC,
                self.action_id.as_str().as_bytes().to_vec(),
            ),
            field_u8(2, self.phase as u8),
            CanonicalField::new(
                3,
                CANONICAL_TYPE_TAGGED_UNION,
                self.value.canonical_tagged_payload()?,
            ),
            field_u32(4, self.semantic_occurrence_ordinal),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_UTF8_NFC),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_TAGGED_UNION),
                (4, CANONICAL_TYPE_U32),
            ],
        )?;
        Ok(Self {
            action_id: SchemaId::new(read_utf8_field(&fields, 1)?)?,
            phase: PlayerActionPhaseV1::from_tag(read_u8_fields(&fields, 2)?)?,
            value: PlayerActionValueV1::from_tagged_payload(
                field_from(&fields, 3)?.payload.as_slice(),
                limits,
            )?,
            semantic_occurrence_ordinal: read_u32_fields(&fields, 4)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerActionFrameV1 {
    pub schema_version: u16,
    pub controller_id: PersistentId,
    pub logical_frame_sequence: u64,
    pub action_map_hash: ContentHash,
    pub action_map_revision: u64,
    pub context_stack_hash: ContentHash,
    pub context_stack_revision: u64,
    pub actions: Vec<PlayerActionV1>,
}

impl PlayerActionFrameV1 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != PLAYER_ACTION_FRAME_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "player action frame",
                version: u32::from(self.schema_version),
            });
        }
        if self.actions.len() > MAX_PLAYER_ACTIONS_PER_FRAME {
            return Err(InputContractError::ResourceLimit);
        }
        let order_is_invalid = self.actions.windows(2).any(|pair| {
            (&pair[0].action_id, pair[0].semantic_occurrence_ordinal)
                >= (&pair[1].action_id, pair[1].semantic_occurrence_ordinal)
        });
        let mut ordinals = self
            .actions
            .iter()
            .map(|action| action.semantic_occurrence_ordinal)
            .collect::<Vec<_>>();
        ordinals.sort_unstable();
        let ordinals_are_invalid = ordinals
            .iter()
            .enumerate()
            .any(|(index, ordinal)| usize::try_from(*ordinal).ok() != Some(index));
        if order_is_invalid || ordinals_are_invalid {
            return Err(InputContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let actions = encode_sequence(
            self.actions
                .iter()
                .map(PlayerActionV1::canonical_record)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        encode_canonical_segment(
            INPUT_OWNER_ID,
            PLAYER_ACTION_FRAME_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.controller_id.as_bytes()),
                field_u64(3, self.logical_frame_sequence),
                field_hash(4, self.action_map_hash),
                field_u64(5, self.action_map_revision),
                field_hash(6, self.context_stack_hash),
                field_u64(7, self.context_stack_revision),
                CanonicalField::new(8, CANONICAL_TYPE_SEQUENCE, actions),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            INPUT_OWNER_ID,
            PLAYER_ACTION_FRAME_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_U64),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let actions = decode_sequence(field(&segment, 8)?, limits)?
            .into_iter()
            .map(|bytes| PlayerActionV1::from_record(&bytes, limits))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            controller_id: PersistentId::from_bytes(exact(field(&segment, 2)?)?),
            logical_frame_sequence: read_u64(&segment, 3)?,
            action_map_hash: read_hash(&segment, 4)?,
            action_map_revision: read_u64(&segment, 5)?,
            context_stack_hash: read_hash(&segment, 6)?,
            context_stack_revision: read_u64(&segment, 7)?,
            actions,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputSampleV1 {
    pub schema_version: u16,
    pub source_class: SchemaId,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_schema_id: SchemaId,
    pub payload_schema_version: u32,
    pub payload: Vec<u8>,
    pub sampled_wall_time: Option<i64>,
}

impl InputSampleV1 {
    pub fn validate(&self, limits: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != INPUT_SAMPLE_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "input sample",
                version: u32::from(self.schema_version),
            });
        }
        if self.source_class.as_str().len()
            > usize::try_from(limits.max_namespaced_id_bytes)
                .map_err(|_| InputContractError::ResourceLimit)?
            || self.payload_schema_id.as_str().len()
                > usize::try_from(limits.max_namespaced_id_bytes)
                    .map_err(|_| InputContractError::ResourceLimit)?
            || self.payload.len()
                > usize::try_from(limits.max_input_payload_bytes)
                    .map_err(|_| InputContractError::ResourceLimit)?
        {
            return Err(InputContractError::ResourceLimit);
        }
        Ok(())
    }

    pub fn payload_hash(&self) -> Result<ContentHash, CanonicalError> {
        let schema = self.payload_schema_id.as_str().as_bytes();
        let mut preimage = Vec::new();
        preimage.extend_from_slice(b"nextengine.input-payload.v1\0");
        preimage.extend_from_slice(
            &u32::try_from(schema.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(schema);
        preimage.extend_from_slice(&self.payload_schema_version.to_le_bytes());
        preimage.extend_from_slice(
            &u64::try_from(self.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        preimage.extend_from_slice(&self.payload);
        Ok(content_hash_from_bytes(sha256(&preimage)))
    }

    pub fn sort_key(&self) -> Result<(String, InputSourceId, u64, ContentHash), CanonicalError> {
        Ok((
            self.source_class.as_str().to_owned(),
            self.source_id,
            self.source_sequence,
            self.payload_hash()?,
        ))
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INPUT_SAMPLE_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(
                    2,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.source_class.as_str().as_bytes().to_vec(),
                ),
                field_id(3, self.source_id.as_bytes()),
                field_u64(4, self.source_sequence),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.payload_schema_id.as_str().as_bytes().to_vec(),
                ),
                field_u32(6, self.payload_schema_version),
                CanonicalField::new(7, CANONICAL_TYPE_BYTES, self.payload.clone()),
                CanonicalField::new(8, CANONICAL_TYPE_OPTIONAL, vec![0]),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INPUT_SAMPLE_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_UTF8_NFC),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_U64),
                (5, CANONICAL_TYPE_UTF8_NFC),
                (6, CANONICAL_TYPE_U32),
                (7, CANONICAL_TYPE_BYTES),
                (8, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        if field(&segment, 8)? != [0] {
            return Err(InputContractError::InvalidOptional);
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            source_class: SchemaId::new(read_utf8(&segment, 2)?)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 3)?)?),
            source_sequence: read_u64(&segment, 4)?,
            payload_schema_id: SchemaId::new(read_utf8(&segment, 5)?)?,
            payload_schema_version: read_u32(&segment, 6)?,
            payload: field(&segment, 7)?.to_vec(),
            sampled_wall_time: None,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IngressAssignmentV1 {
    pub schema_version: u16,
    pub queue_generation: u64,
    pub assigned_tick: u64,
    pub source_class: SchemaId,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_hash: ContentHash,
}

impl IngressAssignmentV1 {
    pub fn from_sample(
        queue_generation: u64,
        assigned_tick: u64,
        sample: &InputSampleV1,
    ) -> Result<Self, CanonicalError> {
        Ok(Self {
            schema_version: INGRESS_ASSIGNMENT_SCHEMA_VERSION,
            queue_generation,
            assigned_tick,
            source_class: sample.source_class.clone(),
            source_id: sample.source_id,
            source_sequence: sample.source_sequence,
            payload_hash: sample.payload_hash()?,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_ASSIGNMENT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.queue_generation),
                field_u64(3, self.assigned_tick),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_UTF8_NFC,
                    self.source_class.as_str().as_bytes().to_vec(),
                ),
                field_id(5, self.source_id.as_bytes()),
                field_u64(6, self.source_sequence),
                field_hash(7, self.payload_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_ASSIGNMENT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_UTF8_NFC),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_U64),
                (7, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            queue_generation: read_u64(&segment, 2)?,
            assigned_tick: read_u64(&segment, 3)?,
            source_class: SchemaId::new(read_utf8(&segment, 4)?)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 5)?)?),
            source_sequence: read_u64(&segment, 6)?,
            payload_hash: read_hash(&segment, 7)?,
        };
        if value.schema_version != INGRESS_ASSIGNMENT_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "ingress assignment",
                version: u32::from(value.schema_version),
            });
        }
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum IngressSubjectKindV1 {
    Input = 1,
    Completion = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum IngressResultCodeV1 {
    InputSequenceCollision = 1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IngressEquivalenceReceiptV1 {
    pub assigned_tick: u64,
    pub subject_kind: IngressSubjectKindV1,
    pub subject_id: Vec<u8>,
    pub candidate_hashes: BTreeSet<ContentHash>,
    pub result_code: IngressResultCodeV1,
}

impl IngressEquivalenceReceiptV1 {
    pub fn for_input_collision(
        assigned_tick: u64,
        source_class: &SchemaId,
        source_id: InputSourceId,
        source_sequence: u64,
        candidate_hashes: BTreeSet<ContentHash>,
    ) -> Result<Self, CanonicalError> {
        let source_class = source_class.as_str().as_bytes();
        let mut subject_id = Vec::new();
        subject_id.extend_from_slice(
            &u32::try_from(source_class.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        subject_id.extend_from_slice(source_class);
        subject_id.extend_from_slice(source_id.as_bytes());
        subject_id.extend_from_slice(&source_sequence.to_le_bytes());
        Ok(Self {
            assigned_tick,
            subject_kind: IngressSubjectKindV1::Input,
            subject_id,
            candidate_hashes,
            result_code: IngressResultCodeV1::InputSequenceCollision,
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let hashes = encode_sequence(
            self.candidate_hashes
                .iter()
                .map(|hash| hash.as_bytes().to_vec())
                .collect(),
        )?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u64(1, self.assigned_tick),
                field_u8(2, self.subject_kind as u8),
                CanonicalField::new(3, CANONICAL_TYPE_BYTES, self.subject_id.clone()),
                CanonicalField::new(4, CANONICAL_TYPE_SEQUENCE, hashes),
                field_u8(5, self.result_code as u8),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_U8),
                (3, CANONICAL_TYPE_BYTES),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_U8),
            ],
        )?;
        let candidate_hashes = decode_sequence(field(&segment, 4)?, limits)?
            .into_iter()
            .map(|bytes| Ok(content_hash_from_bytes(exact(&bytes)?)))
            .collect::<Result<BTreeSet<_>, InputContractError>>()?;
        if candidate_hashes.len() < 2 {
            return Err(InputContractError::InvalidValue);
        }
        let value = Self {
            assigned_tick: read_u64(&segment, 1)?,
            subject_kind: match read_u8(&segment, 2)? {
                1 => IngressSubjectKindV1::Input,
                2 => IngressSubjectKindV1::Completion,
                tag => return Err(InputContractError::UnknownTag(tag)),
            },
            subject_id: field(&segment, 3)?.to_vec(),
            candidate_hashes,
            result_code: match read_u8(&segment, 5)? {
                1 => IngressResultCodeV1::InputSequenceCollision,
                tag => return Err(InputContractError::UnknownTag(tag)),
            },
        };
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedIngressBatchBodyV1 {
    pub schema_version: u16,
    pub queue_generation: u64,
    pub assigned_tick: u64,
    pub input_samples: Vec<InputSampleV1>,
    pub completion_signals: Vec<Vec<u8>>,
    pub input_assignments: Vec<IngressAssignmentV1>,
    pub completion_assignments: Vec<Vec<u8>>,
    pub equivalence_receipts: Vec<IngressEquivalenceReceiptV1>,
}

impl ClosedIngressBatchBodyV1 {
    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_INGRESS_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed ingress batch body",
                version: u32::from(self.schema_version),
            });
        }
        if !self.completion_signals.is_empty() || !self.completion_assignments.is_empty() {
            return Err(InputContractError::UnsupportedCompletionSignal);
        }
        if self.input_samples.len()
            > usize::try_from(admission.max_commands_per_closed_batch)
                .map_err(|_| InputContractError::ResourceLimit)?
        {
            return Err(InputContractError::ResourceLimit);
        }
        for sample in &self.input_samples {
            sample.validate(admission)?;
        }
        let mut candidates =
            BTreeMap::<(SchemaId, InputSourceId, u64), BTreeSet<ContentHash>>::new();
        let mut previous = None;
        for sample in &self.input_samples {
            let sort_key = sample.sort_key()?;
            if previous
                .as_ref()
                .is_some_and(|previous| previous >= &sort_key)
            {
                return Err(InputContractError::NonCanonicalOrder);
            }
            previous = Some(sort_key);
            candidates
                .entry((
                    sample.source_class.clone(),
                    sample.source_id,
                    sample.source_sequence,
                ))
                .or_default()
                .insert(sample.payload_hash()?);
        }
        let mut expected_assignments = Vec::new();
        let mut expected_receipts = Vec::new();
        for ((source_class, source_id, source_sequence), hashes) in candidates {
            if hashes.len() == 1 {
                expected_assignments.push(IngressAssignmentV1 {
                    schema_version: INGRESS_ASSIGNMENT_SCHEMA_VERSION,
                    queue_generation: self.queue_generation,
                    assigned_tick: self.assigned_tick,
                    source_class,
                    source_id,
                    source_sequence,
                    payload_hash: *hashes
                        .first()
                        .expect("a candidate group always contains a payload hash"),
                });
            } else {
                expected_receipts.push(IngressEquivalenceReceiptV1::for_input_collision(
                    self.assigned_tick,
                    &source_class,
                    source_id,
                    source_sequence,
                    hashes,
                )?);
            }
        }
        if self.input_assignments != expected_assignments
            || self.equivalence_receipts != expected_receipts
        {
            return Err(InputContractError::InvalidValue);
        }
        if self
            .input_assignments
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            || self
                .equivalence_receipts
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(InputContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_INGRESS_BODY_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.queue_generation),
                field_u64(3, self.assigned_tick),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.input_samples
                            .iter()
                            .map(InputSampleV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(5, CANONICAL_TYPE_SEQUENCE, encode_sequence(Vec::new())?),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.input_assignments
                            .iter()
                            .map(IngressAssignmentV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(7, CANONICAL_TYPE_SEQUENCE, encode_sequence(Vec::new())?),
                CanonicalField::new(
                    8,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.equivalence_receipts
                            .iter()
                            .map(IngressEquivalenceReceiptV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            CLOSED_INGRESS_BODY_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_SEQUENCE),
                (7, CANONICAL_TYPE_SEQUENCE),
                (8, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            queue_generation: read_u64(&segment, 2)?,
            assigned_tick: read_u64(&segment, 3)?,
            input_samples: decode_sequence(field(&segment, 4)?, limits)?
                .into_iter()
                .map(|bytes| InputSampleV1::from_canonical_bytes(&bytes, limits, admission))
                .collect::<Result<Vec<_>, _>>()?,
            completion_signals: decode_sequence(field(&segment, 5)?, limits)?,
            input_assignments: decode_sequence(field(&segment, 6)?, limits)?
                .into_iter()
                .map(|bytes| IngressAssignmentV1::from_canonical_bytes(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
            completion_assignments: decode_sequence(field(&segment, 7)?, limits)?,
            equivalence_receipts: decode_sequence(field(&segment, 8)?, limits)?
                .into_iter()
                .map(|bytes| IngressEquivalenceReceiptV1::from_canonical_bytes(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedIngressBatchV1 {
    pub schema_version: u16,
    pub body: ClosedIngressBatchBodyV1,
    pub batch_hash: ContentHash,
}

impl ClosedIngressBatchV1 {
    pub fn from_body(body: ClosedIngressBatchBodyV1) -> Result<Self, CanonicalError> {
        let batch_hash = closed_batch_hash(
            b"nextengine.closed-ingress-batch.v1\0",
            &body.canonical_bytes()?,
        )?;
        Ok(Self {
            schema_version: CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
            body,
            batch_hash,
        })
    }

    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_INGRESS_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed ingress batch",
                version: u32::from(self.schema_version),
            });
        }
        self.body.validate(admission)?;
        let expected = closed_batch_hash(
            b"nextengine.closed-ingress-batch.v1\0",
            &self.body.canonical_bytes()?,
        )?;
        if expected != self.batch_hash {
            return Err(InputContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_INGRESS_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.body.canonical_bytes()?),
                field_hash(3, self.batch_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            CLOSED_INGRESS_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            body: ClosedIngressBatchBodyV1::from_canonical_bytes(
                field(&segment, 2)?,
                limits,
                admission,
            )?,
            batch_hash: read_hash(&segment, 3)?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedCommandAdmissionBatchBodyV2 {
    pub schema_version: u16,
    pub simulation_tick: u64,
    pub phase: CommandPhase,
    pub batch_ordinal: u32,
    pub envelopes: Vec<WorldCommand>,
}

impl ClosedCommandAdmissionBatchBodyV2 {
    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed command batch body",
                version: u32::from(self.schema_version),
            });
        }
        if self.envelopes.len()
            > usize::try_from(admission.max_commands_per_closed_batch)
                .map_err(|_| InputContractError::ResourceLimit)?
        {
            return Err(InputContractError::ResourceLimit);
        }
        let mut previous = None;
        for command in &self.envelopes {
            let key = (
                command.stream_id,
                command.sequence,
                command.body_hash()?,
                command.claimed_command_id,
            );
            if previous.as_ref().is_some_and(|previous| previous > &key) {
                return Err(InputContractError::NonCanonicalOrder);
            }
            previous = Some(key);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let envelopes = self
            .envelopes
            .iter()
            .map(encode_command_envelope)
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_COMMAND_BODY_SCHEMA_ID,
            SEGMENT_V2,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.simulation_tick),
                field_u8(3, self.phase as u8),
                field_u32(4, self.batch_ordinal),
                CanonicalField::new(5, CANONICAL_TYPE_SEQUENCE, encode_sequence(envelopes)?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            CLOSED_COMMAND_BODY_SCHEMA_ID,
            SEGMENT_V2,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U8),
                (4, CANONICAL_TYPE_U32),
                (5, CANONICAL_TYPE_SEQUENCE),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            simulation_tick: read_u64(&segment, 2)?,
            phase: match read_u8(&segment, 3)? {
                0 => CommandPhase::Ingress,
                1 => CommandPhase::Outcome,
                tag => return Err(InputContractError::UnknownTag(tag)),
            },
            batch_ordinal: read_u32(&segment, 4)?,
            envelopes: decode_sequence(field(&segment, 5)?, limits)?
                .into_iter()
                .map(|bytes| decode_command_envelope(&bytes, limits))
                .collect::<Result<Vec<_>, _>>()?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosedCommandAdmissionBatchV2 {
    pub schema_version: u16,
    pub body: ClosedCommandAdmissionBatchBodyV2,
    pub batch_hash: ContentHash,
}

impl ClosedCommandAdmissionBatchV2 {
    pub fn from_body(body: ClosedCommandAdmissionBatchBodyV2) -> Result<Self, CanonicalError> {
        Ok(Self {
            schema_version: CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION,
            batch_hash: closed_batch_hash(
                b"nextengine.closed-command-admission-batch.v2\0",
                &body.canonical_bytes()?,
            )?,
            body,
        })
    }

    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != CLOSED_COMMAND_ADMISSION_BATCH_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "closed command batch",
                version: u32::from(self.schema_version),
            });
        }
        self.body.validate(admission)?;
        if self.batch_hash
            != closed_batch_hash(
                b"nextengine.closed-command-admission-batch.v2\0",
                &self.body.canonical_bytes()?,
            )?
        {
            return Err(InputContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            CLOSED_COMMAND_SCHEMA_ID,
            SEGMENT_V2,
            [
                field_u16(1, self.schema_version),
                CanonicalField::new(2, CANONICAL_TYPE_BYTES, self.body.canonical_bytes()?),
                field_hash(3, self.batch_hash),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            CLOSED_COMMAND_SCHEMA_ID,
            SEGMENT_V2,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_BYTES),
                (3, CANONICAL_TYPE_HASH256),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            body: ClosedCommandAdmissionBatchBodyV2::from_canonical_bytes(
                field(&segment, 2)?,
                limits,
                admission,
            )?,
            batch_hash: read_hash(&segment, 3)?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PlayerControllerBindingV1 {
    pub principal: IssuerPrincipal,
    pub source_id: InputSourceId,
    pub controller_id: PersistentId,
    pub controlled_body_id: PersistentId,
    pub command_stream_id: CommandStreamId,
    pub action_map_hash: ContentHash,
    pub action_map_revision: u64,
    pub context_stack_hash: ContentHash,
    pub context_stack_revision: u64,
}

impl PlayerControllerBindingV1 {
    fn canonical_record(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_struct([
            CanonicalField::new(1, CANONICAL_TYPE_BYTES, self.principal.canonical_bytes()?),
            field_id(2, self.source_id.as_bytes()),
            field_id(3, self.controller_id.as_bytes()),
            field_id(4, self.controlled_body_id.as_bytes()),
            field_id(5, self.command_stream_id.as_bytes()),
            field_hash(6, self.action_map_hash),
            field_u64(7, self.action_map_revision),
            field_hash(8, self.context_stack_hash),
            field_u64(9, self.context_stack_revision),
        ])
    }

    fn from_record(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let fields = decode_struct(bytes, limits)?;
        require_fields(
            &fields,
            &[
                (1, CANONICAL_TYPE_BYTES),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_ID128),
                (4, CANONICAL_TYPE_ID128),
                (5, CANONICAL_TYPE_ID128),
                (6, CANONICAL_TYPE_HASH256),
                (7, CANONICAL_TYPE_U64),
                (8, CANONICAL_TYPE_HASH256),
                (9, CANONICAL_TYPE_U64),
            ],
        )?;
        Ok(Self {
            principal: IssuerPrincipal::from_canonical_bytes(
                &field_from(&fields, 1)?.payload,
                limits,
            )?,
            source_id: InputSourceId::from_bytes(exact(&field_from(&fields, 2)?.payload)?),
            controller_id: PersistentId::from_bytes(exact(&field_from(&fields, 3)?.payload)?),
            controlled_body_id: PersistentId::from_bytes(exact(&field_from(&fields, 4)?.payload)?),
            command_stream_id: CommandStreamId::from_bytes(exact(
                &field_from(&fields, 5)?.payload,
            )?),
            action_map_hash: content_hash_from_bytes(exact(&field_from(&fields, 6)?.payload)?),
            action_map_revision: read_u64_fields(&fields, 7)?,
            context_stack_hash: content_hash_from_bytes(exact(&field_from(&fields, 8)?.payload)?),
            context_stack_revision: read_u64_fields(&fields, 9)?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerControllerRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub bindings: BTreeMap<InputSourceId, PlayerControllerBindingV1>,
}

impl PlayerControllerRegistryV1 {
    pub fn validate(&self) -> Result<(), InputContractError> {
        if self.schema_version != PLAYER_CONTROLLER_REGISTRY_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "player controller registry",
                version: u32::from(self.schema_version),
            });
        }
        let mut controllers = BTreeSet::new();
        let mut bodies = BTreeSet::new();
        let mut streams = BTreeSet::new();
        for (source, binding) in &self.bindings {
            if source != &binding.source_id
                || !controllers.insert(binding.controller_id)
                || !bodies.insert(binding.controlled_body_id)
                || !streams.insert(binding.command_stream_id)
                || !matches!(binding.principal, IssuerPrincipal::Player(_))
            {
                return Err(InputContractError::RegistryCollision);
            }
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        let entries = self
            .bindings
            .iter()
            .map(|(source, binding)| {
                encode_struct([
                    field_id(1, source.as_bytes()),
                    CanonicalField::new(2, CANONICAL_TYPE_STRUCT, binding.canonical_record()?),
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            PLAYER_CONTROLLER_REGISTRY_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_id(2, self.world_namespace.as_bytes()),
                CanonicalField::new(3, CANONICAL_TYPE_MAP, encode_sequence(entries)?),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            PLAYER_CONTROLLER_REGISTRY_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_MAP),
            ],
        )?;
        let mut bindings = BTreeMap::new();
        for bytes in decode_sequence(field(&segment, 3)?, limits)? {
            let fields = decode_struct(&bytes, limits)?;
            require_fields(
                &fields,
                &[(1, CANONICAL_TYPE_ID128), (2, CANONICAL_TYPE_STRUCT)],
            )?;
            let source = InputSourceId::from_bytes(exact(&field_from(&fields, 1)?.payload)?);
            let binding =
                PlayerControllerBindingV1::from_record(&field_from(&fields, 2)?.payload, limits)?;
            if bindings.insert(source, binding).is_some() {
                return Err(InputContractError::DuplicateKey);
            }
        }
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            world_namespace: WorldNamespaceId::from_bytes(exact(field(&segment, 2)?)?),
            bindings,
        };
        value.validate()?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum InputMappingCodeV1 {
    Accepted = 0,
    PrincipalUnbound = 1,
    PayloadSchemaUnsupported = 2,
    ActionMapStale = 3,
    ContextStackStale = 4,
    ActionUnmapped = 5,
    ValueOutOfProfile = 6,
    FrameInvalid = 7,
}

impl InputMappingCodeV1 {
    #[must_use]
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::Accepted => "INPUT_ACCEPTED",
            Self::PrincipalUnbound => "INPUT_PRINCIPAL_UNBOUND",
            Self::PayloadSchemaUnsupported => "INPUT_SCHEMA_UNSUPPORTED",
            Self::ActionMapStale => "INPUT_ACTION_MAP_STALE",
            Self::ContextStackStale => "INPUT_CONTEXT_STALE",
            Self::ActionUnmapped => "INPUT_ACTION_UNMAPPED",
            Self::ValueOutOfProfile => "INPUT_VALUE_OUT_OF_PROFILE",
            Self::FrameInvalid => "INPUT_FRAME_INVALID",
        }
    }

    fn from_tag(tag: u8) -> Result<Self, InputContractError> {
        match tag {
            0 => Ok(Self::Accepted),
            1 => Ok(Self::PrincipalUnbound),
            2 => Ok(Self::PayloadSchemaUnsupported),
            3 => Ok(Self::ActionMapStale),
            4 => Ok(Self::ContextStackStale),
            5 => Ok(Self::ActionUnmapped),
            6 => Ok(Self::ValueOutOfProfile),
            7 => Ok(Self::FrameInvalid),
            _ => Err(InputContractError::UnknownTag(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputMappingReceiptV1 {
    pub assigned_tick: u64,
    pub source_id: InputSourceId,
    pub source_sequence: u64,
    pub payload_hash: ContentHash,
    pub code: InputMappingCodeV1,
    pub derived_command_id: Option<CommandId>,
}

impl InputMappingReceiptV1 {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INPUT_MAPPING_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u64(1, self.assigned_tick),
                field_id(2, self.source_id.as_bytes()),
                field_u64(3, self.source_sequence),
                field_hash(4, self.payload_hash),
                field_u8(5, self.code as u8),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_optional_id(self.derived_command_id.as_ref())?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INPUT_MAPPING_RECEIPT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U64),
                (2, CANONICAL_TYPE_ID128),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_HASH256),
                (5, CANONICAL_TYPE_U8),
                (6, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        let value = Self {
            assigned_tick: read_u64(&segment, 1)?,
            source_id: InputSourceId::from_bytes(exact(field(&segment, 2)?)?),
            source_sequence: read_u64(&segment, 3)?,
            payload_hash: read_hash(&segment, 4)?,
            code: InputMappingCodeV1::from_tag(read_u8(&segment, 5)?)?,
            derived_command_id: decode_optional_id(field(&segment, 6)?, limits)?,
        };
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngressCheckpointV1 {
    pub schema_version: u16,
    pub current_tick: u64,
    pub current_generation: u64,
    pub current_samples: Vec<InputSampleV1>,
    pub next_samples: Vec<InputSampleV1>,
    pub last_closed_batch_hash: Option<ContentHash>,
}

impl IngressCheckpointV1 {
    pub fn validate(&self, admission: &RuntimeAdmissionLimitsV1) -> Result<(), InputContractError> {
        if self.schema_version != INGRESS_CHECKPOINT_SCHEMA_VERSION {
            return Err(InputContractError::UnsupportedVersion {
                contract: "ingress checkpoint",
                version: u32::from(self.schema_version),
            });
        }
        let max = usize::try_from(admission.max_commands_per_closed_batch)
            .map_err(|_| InputContractError::ResourceLimit)?;
        if self.current_samples.len() > max || self.next_samples.len() > max {
            return Err(InputContractError::ResourceLimit);
        }
        for sample in self.current_samples.iter().chain(&self.next_samples) {
            sample.validate(admission)?;
        }
        if !is_non_decreasing_by(&self.current_samples, InputSampleV1::sort_key)
            || !is_non_decreasing_by(&self.next_samples, InputSampleV1::sort_key)
        {
            return Err(InputContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        encode_canonical_segment(
            RUNTIME_OWNER_ID,
            INGRESS_CHECKPOINT_SCHEMA_ID,
            SEGMENT_V1,
            [
                field_u16(1, self.schema_version),
                field_u64(2, self.current_tick),
                field_u64(3, self.current_generation),
                CanonicalField::new(
                    4,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.current_samples
                            .iter()
                            .map(InputSampleV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(
                    5,
                    CANONICAL_TYPE_SEQUENCE,
                    encode_sequence(
                        self.next_samples
                            .iter()
                            .map(InputSampleV1::canonical_bytes)
                            .collect::<Result<Vec<_>, _>>()?,
                    )?,
                ),
                CanonicalField::new(
                    6,
                    CANONICAL_TYPE_OPTIONAL,
                    encode_optional_hash(self.last_closed_batch_hash.as_ref())?,
                ),
            ],
        )
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
        admission: &RuntimeAdmissionLimitsV1,
    ) -> Result<Self, InputContractError> {
        let segment = decode_contract(
            bytes,
            limits,
            RUNTIME_OWNER_ID,
            INGRESS_CHECKPOINT_SCHEMA_ID,
            SEGMENT_V1,
            &[
                (1, CANONICAL_TYPE_U16),
                (2, CANONICAL_TYPE_U64),
                (3, CANONICAL_TYPE_U64),
                (4, CANONICAL_TYPE_SEQUENCE),
                (5, CANONICAL_TYPE_SEQUENCE),
                (6, CANONICAL_TYPE_OPTIONAL),
            ],
        )?;
        let value = Self {
            schema_version: read_u16(&segment, 1)?,
            current_tick: read_u64(&segment, 2)?,
            current_generation: read_u64(&segment, 3)?,
            current_samples: decode_sequence(field(&segment, 4)?, limits)?
                .into_iter()
                .map(|bytes| InputSampleV1::from_canonical_bytes(&bytes, limits, admission))
                .collect::<Result<Vec<_>, _>>()?,
            next_samples: decode_sequence(field(&segment, 5)?, limits)?
                .into_iter()
                .map(|bytes| InputSampleV1::from_canonical_bytes(&bytes, limits, admission))
                .collect::<Result<Vec<_>, _>>()?,
            last_closed_batch_hash: decode_optional_hash(field(&segment, 6)?, limits)?,
        };
        value.validate(admission)?;
        require_round_trip(bytes, value.canonical_bytes()?)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InputContractError {
    Canonical(CanonicalDecodeError),
    Canonicalization(CanonicalError),
    Identifier(crate::IdentifierError),
    Principal(crate::PrincipalDecodeError),
    Command(crate::CommandDecodeError),
    WrongEnvelope,
    UnknownField(u32),
    MissingField(u32),
    FieldType,
    FieldLength,
    InvalidOptional,
    InvalidValue,
    InvalidProfile,
    ResourceLimit,
    NonCanonicalOrder,
    DuplicateKey,
    RegistryCollision,
    HashMismatch,
    UnsupportedCompletionSignal,
    UnsupportedVersion {
        contract: &'static str,
        version: u32,
    },
    UnknownTag(u8),
    NonCanonicalEncoding,
}

impl InputContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion { .. } => "INPUT_CONTRACT_UNSUPPORTED_VERSION",
            Self::ResourceLimit => "INPUT_RESOURCE_LIMIT",
            Self::HashMismatch => "INGRESS_ASSIGNMENT_CORRUPT",
            Self::RegistryCollision => "PLAYER_CONTROLLER_REGISTRY_CORRUPT",
            _ => "INPUT_CONTRACT_INVALID",
        }
    }
}

impl Display for InputContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "canonical input is invalid: {error}"),
            Self::Canonicalization(error) => {
                write!(formatter, "canonical input could not be encoded: {error}")
            }
            Self::Identifier(error) => write!(formatter, "input identifier is invalid: {error}"),
            Self::Principal(error) => write!(formatter, "input principal is invalid: {error}"),
            Self::Command(error) => write!(formatter, "input command is invalid: {error}"),
            Self::WrongEnvelope => formatter.write_str("input contract envelope does not match"),
            Self::UnknownField(id) => write!(formatter, "unknown input field {id}"),
            Self::MissingField(id) => write!(formatter, "missing input field {id}"),
            Self::FieldType => formatter.write_str("input field has the wrong canonical type"),
            Self::FieldLength => formatter.write_str("input field has the wrong length"),
            Self::InvalidOptional => formatter.write_str("input optional value is invalid"),
            Self::InvalidValue => formatter.write_str("input value is invalid"),
            Self::InvalidProfile => formatter.write_str("input profile is invalid"),
            Self::ResourceLimit => formatter.write_str("input exceeds a bound profile limit"),
            Self::NonCanonicalOrder => {
                formatter.write_str("input collection order is not canonical")
            }
            Self::DuplicateKey => formatter.write_str("input map contains a duplicate key"),
            Self::RegistryCollision => {
                formatter.write_str("controller registry contains a collision")
            }
            Self::HashMismatch => formatter.write_str("input hash does not match its body"),
            Self::UnsupportedCompletionSignal => {
                formatter.write_str("completion signals are not active in this runtime slice")
            }
            Self::UnsupportedVersion { contract, version } => {
                write!(formatter, "unsupported {contract} version {version}")
            }
            Self::UnknownTag(tag) => write!(formatter, "unknown input tag {tag}"),
            Self::NonCanonicalEncoding => {
                formatter.write_str("input does not re-encode byte-exactly")
            }
        }
    }
}

impl Error for InputContractError {}

impl From<CanonicalDecodeError> for InputContractError {
    fn from(error: CanonicalDecodeError) -> Self {
        Self::Canonical(error)
    }
}

impl From<CanonicalError> for InputContractError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonicalization(error)
    }
}

impl From<crate::IdentifierError> for InputContractError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::PrincipalDecodeError> for InputContractError {
    fn from(error: crate::PrincipalDecodeError) -> Self {
        Self::Principal(error)
    }
}

impl From<crate::CommandDecodeError> for InputContractError {
    fn from(error: crate::CommandDecodeError) -> Self {
        Self::Command(error)
    }
}

fn hash_canonical_profile(bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.profile.v1\0");
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn closed_batch_hash(domain: &[u8], bytes: &[u8]) -> Result<ContentHash, CanonicalError> {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn encode_command_envelope(command: &WorldCommand) -> Result<Vec<u8>, CanonicalError> {
    encode_struct([
        field_u16(1, command.envelope_schema_version),
        CanonicalField::new(
            2,
            CANONICAL_TYPE_OPTIONAL,
            encode_optional_id(command.claimed_command_id.as_ref())?,
        ),
        CanonicalField::new(3, CANONICAL_TYPE_BYTES, command.canonical_bytes()?),
    ])
}

fn decode_command_envelope(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<WorldCommand, InputContractError> {
    let fields = decode_struct(bytes, limits)?;
    require_fields(
        &fields,
        &[
            (1, CANONICAL_TYPE_U16),
            (2, CANONICAL_TYPE_OPTIONAL),
            (3, CANONICAL_TYPE_BYTES),
        ],
    )?;
    let mut command = WorldCommand::from_canonical_bytes(&field_from(&fields, 3)?.payload, limits)?;
    command.envelope_schema_version = read_u16_fields(&fields, 1)?;
    command.claimed_command_id = decode_optional_id(&field_from(&fields, 2)?.payload, limits)?;
    Ok(command)
}

fn decode_contract(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
    owner: &str,
    schema: &str,
    segment_id: &str,
    fields: &[(u32, u8)],
) -> Result<DecodedCanonicalSegment, InputContractError> {
    let segment = decode_canonical_segment(bytes, limits)?;
    if segment.owner_id != owner || segment.schema_id != schema || segment.segment_id != segment_id
    {
        return Err(InputContractError::WrongEnvelope);
    }
    require_fields(&segment.fields, fields)?;
    Ok(segment)
}

fn require_fields(
    actual: &[CanonicalField],
    expected: &[(u32, u8)],
) -> Result<(), InputContractError> {
    for field in actual {
        let Some((_, expected_tag)) = expected.iter().find(|(id, _)| *id == field.field_id) else {
            return Err(InputContractError::UnknownField(field.field_id));
        };
        if field.type_tag != *expected_tag {
            return Err(InputContractError::FieldType);
        }
    }
    for (id, _) in expected {
        if !actual.iter().any(|field| field.field_id == *id) {
            return Err(InputContractError::MissingField(*id));
        }
    }
    Ok(())
}

fn field(segment: &DecodedCanonicalSegment, id: u32) -> Result<&[u8], InputContractError> {
    Ok(&segment
        .field(id)
        .ok_or(InputContractError::MissingField(id))?
        .payload)
}

fn field_from(fields: &[CanonicalField], id: u32) -> Result<&CanonicalField, InputContractError> {
    fields
        .binary_search_by_key(&id, |field| field.field_id)
        .ok()
        .map(|index| &fields[index])
        .ok_or(InputContractError::MissingField(id))
}

fn read_u8(segment: &DecodedCanonicalSegment, id: u32) -> Result<u8, InputContractError> {
    Ok(exact::<1>(field(segment, id)?)?[0])
}

fn read_u16(segment: &DecodedCanonicalSegment, id: u32) -> Result<u16, InputContractError> {
    Ok(u16::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_u32(segment: &DecodedCanonicalSegment, id: u32) -> Result<u32, InputContractError> {
    Ok(u32::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_u64(segment: &DecodedCanonicalSegment, id: u32) -> Result<u64, InputContractError> {
    Ok(u64::from_le_bytes(exact(field(segment, id)?)?))
}

fn read_hash(
    segment: &DecodedCanonicalSegment,
    id: u32,
) -> Result<ContentHash, InputContractError> {
    Ok(content_hash_from_bytes(exact(field(segment, id)?)?))
}

fn read_utf8(segment: &DecodedCanonicalSegment, id: u32) -> Result<&str, InputContractError> {
    std::str::from_utf8(field(segment, id)?)
        .map_err(|_| InputContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

fn read_u8_fields(fields: &[CanonicalField], id: u32) -> Result<u8, InputContractError> {
    Ok(exact::<1>(&field_from(fields, id)?.payload)?[0])
}

fn read_u16_fields(fields: &[CanonicalField], id: u32) -> Result<u16, InputContractError> {
    Ok(u16::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_u32_fields(fields: &[CanonicalField], id: u32) -> Result<u32, InputContractError> {
    Ok(u32::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_u64_fields(fields: &[CanonicalField], id: u32) -> Result<u64, InputContractError> {
    Ok(u64::from_le_bytes(exact(&field_from(fields, id)?.payload)?))
}

fn read_utf8_field(fields: &[CanonicalField], id: u32) -> Result<&str, InputContractError> {
    std::str::from_utf8(&field_from(fields, id)?.payload)
        .map_err(|_| InputContractError::Canonical(CanonicalDecodeError::InvalidUtf8))
}

fn exact<const N: usize>(bytes: &[u8]) -> Result<[u8; N], InputContractError> {
    bytes
        .try_into()
        .map_err(|_| InputContractError::FieldLength)
}

fn field_u8(id: u32, value: u8) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U8, vec![value])
}

fn field_u16(id: u32, value: u16) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U16, value.to_le_bytes().to_vec())
}

fn field_u32(id: u32, value: u32) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U32, value.to_le_bytes().to_vec())
}

fn field_u64(id: u32, value: u64) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_U64, value.to_le_bytes().to_vec())
}

fn field_id<const N: usize>(id: u32, value: &[u8; N]) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_ID128, value.to_vec())
}

fn field_hash(id: u32, value: ContentHash) -> CanonicalField {
    CanonicalField::new(id, CANONICAL_TYPE_HASH256, value.as_bytes().to_vec())
}

fn encode_nested(type_tag: u8, payload: &[u8]) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = vec![type_tag];
    bytes.extend_from_slice(
        &u64::try_from(payload.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

fn read_nested<'a>(
    cursor: &mut CanonicalCursor<'a>,
    limits: CanonicalDecodeLimits,
) -> Result<(u8, &'a [u8]), InputContractError> {
    let type_tag = cursor.read_u8()?;
    let length =
        usize::try_from(cursor.read_u64()?).map_err(|_| InputContractError::FieldLength)?;
    if length > limits.max_field_payload_bytes {
        return Err(InputContractError::ResourceLimit);
    }
    Ok((type_tag, cursor.read_exact(length)?))
}

fn encode_struct(
    fields: impl IntoIterator<Item = CanonicalField>,
) -> Result<Vec<u8>, CanonicalError> {
    let mut fields: Vec<_> = fields.into_iter().collect();
    fields.sort_by_key(|field| field.field_id);
    if fields
        .windows(2)
        .any(|pair| pair[0].field_id == pair[1].field_id)
    {
        return Err(CanonicalError::DuplicateField(0));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(fields.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for field in fields {
        bytes.extend_from_slice(&field.field_id.to_le_bytes());
        bytes.push(field.type_tag);
        bytes.extend_from_slice(
            &u64::try_from(field.payload.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&field.payload);
    }
    Ok(bytes)
}

fn decode_struct(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<CanonicalField>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_fields, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut fields = Vec::with_capacity(count);
    let mut previous = None;
    for _ in 0..count {
        let field_id = cursor.read_u32()?;
        if previous.is_some_and(|previous| field_id <= previous) {
            return Err(InputContractError::NonCanonicalOrder);
        }
        previous = Some(field_id);
        let type_tag = cursor.read_u8()?;
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| InputContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(InputContractError::ResourceLimit);
        }
        fields.push(CanonicalField::new(
            field_id,
            type_tag,
            cursor.read_exact(length)?.to_vec(),
        ));
    }
    cursor.finish()?;
    Ok(fields)
}

fn encode_sequence(items: Vec<Vec<u8>>) -> Result<Vec<u8>, CanonicalError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(
        &u32::try_from(items.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    for item in items {
        bytes.extend_from_slice(
            &u64::try_from(item.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&item);
    }
    Ok(bytes)
}

fn decode_sequence(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Vec<Vec<u8>>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    let count = cursor.read_count(limits.max_sequence_items, |actual, limit| {
        CanonicalDecodeError::TooManyFields { actual, limit }
    })?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let length =
            usize::try_from(cursor.read_u64()?).map_err(|_| InputContractError::FieldLength)?;
        if length > limits.max_field_payload_bytes {
            return Err(InputContractError::ResourceLimit);
        }
        items.push(cursor.read_exact(length)?.to_vec());
    }
    cursor.finish()?;
    Ok(items)
}

fn encode_optional_hash(value: Option<&ContentHash>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut bytes = vec![1];
    bytes.extend_from_slice(&encode_nested(CANONICAL_TYPE_HASH256, value.as_bytes())?);
    Ok(bytes)
}

fn decode_optional_hash(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<ContentHash>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    match cursor.read_u8()? {
        0 => {
            cursor.finish()?;
            Ok(None)
        }
        1 => {
            let (tag, payload) = read_nested(&mut cursor, limits)?;
            cursor.finish()?;
            if tag != CANONICAL_TYPE_HASH256 {
                return Err(InputContractError::FieldType);
            }
            Ok(Some(content_hash_from_bytes(exact(payload)?)))
        }
        _ => Err(InputContractError::InvalidOptional),
    }
}

fn encode_optional_id(value: Option<&CommandId>) -> Result<Vec<u8>, CanonicalError> {
    let Some(value) = value else {
        return Ok(vec![0]);
    };
    let mut bytes = vec![1];
    bytes.extend_from_slice(&encode_nested(CANONICAL_TYPE_ID128, value.as_bytes())?);
    Ok(bytes)
}

fn decode_optional_id(
    bytes: &[u8],
    limits: CanonicalDecodeLimits,
) -> Result<Option<CommandId>, InputContractError> {
    let mut cursor = CanonicalCursor::new(bytes);
    match cursor.read_u8()? {
        0 => {
            cursor.finish()?;
            Ok(None)
        }
        1 => {
            let (tag, payload) = read_nested(&mut cursor, limits)?;
            cursor.finish()?;
            if tag != CANONICAL_TYPE_ID128 {
                return Err(InputContractError::FieldType);
            }
            Ok(Some(CommandId::from_bytes(exact(payload)?)))
        }
        _ => Err(InputContractError::InvalidOptional),
    }
}

fn require_round_trip(original: &[u8], encoded: Vec<u8>) -> Result<(), InputContractError> {
    if original != encoded {
        return Err(InputContractError::NonCanonicalEncoding);
    }
    Ok(())
}

fn is_non_decreasing_by<T, K, E>(values: &[T], mut key: impl FnMut(&T) -> Result<K, E>) -> bool
where
    K: Ord,
{
    let mut previous = None;
    for value in values {
        let Ok(actual) = key(value) else {
            return false;
        };
        if previous.as_ref().is_some_and(|previous| previous > &actual) {
            return false;
        }
        previous = Some(actual);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(byte: u8) -> ContentHash {
        content_hash_from_bytes([byte; 32])
    }

    #[test]
    fn profiles_round_trip_and_bind_hashes() {
        let limits = RuntimeAdmissionLimitsV1::default();
        let bytes = limits.canonical_bytes().expect("limits encode");
        assert_eq!(
            RuntimeAdmissionLimitsV1::from_canonical_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("limits decode"),
            limits
        );

        let tick = TickRateProfileV1::at_30_hz();
        tick.validate().expect("tick profile valid");
        assert_eq!(tick.physics_hz(), 60);

        let ingress = IngressAssignmentProfileV1::core_v1(&limits).expect("profile");
        ingress.validate().expect("ingress profile valid");
        assert_eq!(
            ingress.admission_limits_hash,
            limits.profile_hash().expect("hash")
        );
    }

    #[test]
    fn core_player_action_map_hash_binds_move_and_interact() {
        assert_eq!(
            core_player_action_map_v1_hash().to_hex(),
            "78bcf0dea6dfdf4cfae9420f934d20eb4f04f1cc4d280cba852802dc38cd177c"
        );
    }

    #[test]
    fn action_frame_round_trips_and_wall_time_is_not_authoritative() {
        let frame = PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: PersistentId::from_bytes([1; 16]),
            logical_frame_sequence: 7,
            action_map_hash: hash(2),
            action_map_revision: 3,
            context_stack_hash: hash(4),
            context_stack_revision: 5,
            actions: vec![PlayerActionV1 {
                action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action id"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
                semantic_occurrence_ordinal: 0,
            }],
        };
        frame.validate().expect("frame valid");
        let bytes = frame.canonical_bytes().expect("frame encode");
        assert_eq!(
            PlayerActionFrameV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("frame decode"),
            frame
        );

        let mut sample = InputSampleV1 {
            schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
            source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source"),
            source_id: InputSourceId::from_bytes([8; 16]),
            source_sequence: 7,
            payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("schema"),
            payload_schema_version: 1,
            payload: bytes,
            sampled_wall_time: Some(123),
        };
        let first = sample.canonical_bytes().expect("sample encode");
        sample.sampled_wall_time = Some(-999);
        assert_eq!(sample.canonical_bytes().expect("sample encode"), first);
    }

    #[test]
    fn closed_ingress_batch_rejects_hash_corruption() {
        let admission = RuntimeAdmissionLimitsV1::default();
        let body = ClosedIngressBatchBodyV1 {
            schema_version: 1,
            queue_generation: 0,
            assigned_tick: 0,
            input_samples: Vec::new(),
            completion_signals: Vec::new(),
            input_assignments: Vec::new(),
            completion_assignments: Vec::new(),
            equivalence_receipts: Vec::new(),
        };
        let mut batch = ClosedIngressBatchV1::from_body(body).expect("batch");
        batch.validate(&admission).expect("valid batch");
        batch.batch_hash = hash(9);
        assert_eq!(
            batch.validate(&admission),
            Err(InputContractError::HashMismatch)
        );
    }

    #[test]
    fn closed_ingress_batch_requires_exact_assignment_and_dedup_closure() {
        let admission = RuntimeAdmissionLimitsV1::default();
        let sample = InputSampleV1 {
            schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
            source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
            source_id: InputSourceId::from_bytes([8; 16]),
            source_sequence: 4,
            payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)
                .expect("payload schema"),
            payload_schema_version: 1,
            payload: vec![],
            sampled_wall_time: None,
        };
        let mut body = ClosedIngressBatchBodyV1 {
            schema_version: CLOSED_INGRESS_BATCH_SCHEMA_VERSION,
            queue_generation: 3,
            assigned_tick: 7,
            input_samples: vec![sample.clone()],
            completion_signals: vec![],
            input_assignments: vec![],
            completion_assignments: vec![],
            equivalence_receipts: vec![],
        };
        assert_eq!(
            body.validate(&admission),
            Err(InputContractError::InvalidValue)
        );
        body.input_assignments.push(
            IngressAssignmentV1::from_sample(3, 7, &sample).expect("assignment is canonical"),
        );
        body.validate(&admission)
            .expect("exact assignment closure is valid");

        body.input_samples.push(sample);
        assert_eq!(
            body.validate(&admission),
            Err(InputContractError::NonCanonicalOrder)
        );
    }

    #[test]
    fn mapping_receipt_round_trips_with_derived_command_identity() {
        let receipt = InputMappingReceiptV1 {
            assigned_tick: 9,
            source_id: InputSourceId::from_bytes([7; 16]),
            source_sequence: 4,
            payload_hash: hash(6),
            code: InputMappingCodeV1::Accepted,
            derived_command_id: Some(CommandId::from_bytes([5; 16])),
        };
        let bytes = receipt.canonical_bytes().expect("receipt");
        assert_eq!(
            InputMappingReceiptV1::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("receipt decode"),
            receipt
        );
    }

    #[test]
    fn action_and_payload_limits_accept_boundary_and_reject_overflow() {
        let action = |ordinal| PlayerActionV1 {
            action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("action id"),
            phase: PlayerActionPhaseV1::Performed,
            value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
            semantic_occurrence_ordinal: ordinal,
        };
        for count in [
            MAX_PLAYER_ACTIONS_PER_FRAME - 1,
            MAX_PLAYER_ACTIONS_PER_FRAME,
        ] {
            let frame = PlayerActionFrameV1 {
                schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
                controller_id: PersistentId::from_bytes([1; 16]),
                logical_frame_sequence: 0,
                action_map_hash: hash(2),
                action_map_revision: 0,
                context_stack_hash: hash(3),
                context_stack_revision: 0,
                actions: (0..u32::try_from(count).expect("count fits"))
                    .map(action)
                    .collect(),
            };
            frame.validate().expect("boundary action count is valid");
        }
        let overflow_frame = PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: PersistentId::from_bytes([1; 16]),
            logical_frame_sequence: 0,
            action_map_hash: hash(2),
            action_map_revision: 0,
            context_stack_hash: hash(3),
            context_stack_revision: 0,
            actions: (0..=u32::try_from(MAX_PLAYER_ACTIONS_PER_FRAME).expect("count fits"))
                .map(action)
                .collect(),
        };
        assert_eq!(
            overflow_frame.validate(),
            Err(InputContractError::ResourceLimit)
        );

        let limits = RuntimeAdmissionLimitsV1::default();
        let sample = |payload_size| InputSampleV1 {
            schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
            source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
            source_id: InputSourceId::from_bytes([4; 16]),
            source_sequence: 0,
            payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID)
                .expect("payload schema"),
            payload_schema_version: 1,
            payload: vec![0; payload_size],
            sampled_wall_time: None,
        };
        let maximum = usize::try_from(limits.max_input_payload_bytes).expect("limit fits");
        sample(maximum - 1)
            .validate(&limits)
            .expect("N-1 payload is valid");
        sample(maximum)
            .validate(&limits)
            .expect("N payload is valid");
        assert_eq!(
            sample(maximum + 1).validate(&limits),
            Err(InputContractError::ResourceLimit)
        );
    }

    #[test]
    fn occurrence_ordinals_are_global_while_action_storage_is_key_sorted() {
        let frame = PlayerActionFrameV1 {
            schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
            controller_id: PersistentId::from_bytes([1; 16]),
            logical_frame_sequence: 0,
            action_map_hash: hash(2),
            action_map_revision: 0,
            context_stack_hash: hash(3),
            context_stack_revision: 0,
            actions: vec![
                PlayerActionV1 {
                    action_id: SchemaId::new("nextengine.action.alpha").expect("action id"),
                    phase: PlayerActionPhaseV1::Performed,
                    value: PlayerActionValueV1::Digital(true),
                    semantic_occurrence_ordinal: 1,
                },
                PlayerActionV1 {
                    action_id: SchemaId::new("nextengine.action.beta").expect("action id"),
                    phase: PlayerActionPhaseV1::Performed,
                    value: PlayerActionValueV1::Digital(true),
                    semantic_occurrence_ordinal: 0,
                },
            ],
        };
        frame
            .validate()
            .expect("key order and global continuous ordinals are independent");
    }
}
