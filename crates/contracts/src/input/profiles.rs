use crate::canonical::*;
use crate::ids::*;

use super::codec::*;
use super::constants::*;

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
