#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::sha256;
use crate::ids::{ContentHash, PersistentId, SchemaId, StateRoot, content_hash_from_bytes};
use crate::physics::{AppliedActuatorEffortV1, PhysicsWorldCheckpointV2};

pub const MOTOR_OBSERVATION_LAYOUT_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_ACTION_LAYOUT_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_WORLD_CHECKPOINT_V1_SCHEMA_VERSION: u16 = 1;
pub const POLICY_STATE_RECORD_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_RESET_RECORD_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_STEP_RECORD_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_TRAJECTORY_MANIFEST_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION: u16 = 2;
pub const MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_RESET_RECORD_V2_SCHEMA_VERSION: u16 = 2;
pub const MOTOR_STEP_RECORD_V2_SCHEMA_VERSION: u16 = 2;
pub const MOTOR_TRAJECTORY_MANIFEST_V2_SCHEMA_VERSION: u16 = 2;
pub const MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION: u16 = 1;
pub const MAX_MOTOR_CHANNELS: usize = 4_096;
pub const MAX_POLICY_STATE_VALUES: usize = 16_384;
pub const MAX_REWARD_COMPONENTS: usize = 128;
pub const MAX_MOTOR_ENVIRONMENT_CHECKPOINT_BYTES: usize = 4 * 1024 * 1024;
pub const STAGE0_PHYSICS_HZ: u32 = 240;
pub const STAGE0_MOTOR_HZ: u32 = 60;
pub const STAGE0_SUBSTEPS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MotorObservationSemanticV1 {
    RootOrientation = 1,
    RootLinearVelocity = 2,
    RootAngularVelocity = 3,
    JointPosition = 4,
    JointVelocity = 5,
    PreviousAction = 6,
    Command = 7,
    Contact = 8,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MotorActionSemanticV1 {
    ResidualJointPosition = 1,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MotorObservationChannelV1 {
    pub channel_id: SchemaId,
    pub semantic: MotorObservationSemanticV1,
    pub source_id: SchemaId,
    pub minimum_raw: i64,
    pub maximum_raw: i64,
    pub scale_numerator: i64,
    pub scale_denominator: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MotorActionChannelV1 {
    pub channel_id: SchemaId,
    pub semantic: MotorActionSemanticV1,
    pub actuator_id: SchemaId,
    pub minimum_raw: i64,
    pub maximum_raw: i64,
    pub scale_numerator: i64,
    pub scale_denominator: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorObservationLayoutV1 {
    pub schema_version: u16,
    pub layout_id: SchemaId,
    pub body_schema_hash: ContentHash,
    pub channels: Vec<MotorObservationChannelV1>,
}

impl MotorObservationLayoutV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_OBSERVATION_LAYOUT_V1_SCHEMA_VERSION {
            return Err(MotorContractError::UnsupportedVersion(self.schema_version));
        }
        validate_channels(
            self.channels.iter().map(|channel| {
                (
                    &channel.channel_id,
                    channel.minimum_raw,
                    channel.maximum_raw,
                    channel.scale_denominator,
                )
            }),
            self.channels.len(),
        )
    }

    pub fn layout_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-observation-layout.v1",
            self.schema_version,
        )?;
        push_id(&mut bytes, &self.layout_id)?;
        bytes.extend_from_slice(self.body_schema_hash.as_bytes());
        push_len(&mut bytes, self.channels.len())?;
        for channel in &self.channels {
            push_id(&mut bytes, &channel.channel_id)?;
            bytes.push(channel.semantic as u8);
            push_id(&mut bytes, &channel.source_id)?;
            encode_bounds(
                &mut bytes,
                channel.minimum_raw,
                channel.maximum_raw,
                channel.scale_numerator,
                channel.scale_denominator,
            );
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorActionLayoutV1 {
    pub schema_version: u16,
    pub layout_id: SchemaId,
    pub body_schema_hash: ContentHash,
    pub actuator_profile_hash: ContentHash,
    pub channels: Vec<MotorActionChannelV1>,
}

impl MotorActionLayoutV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_ACTION_LAYOUT_V1_SCHEMA_VERSION {
            return Err(MotorContractError::UnsupportedVersion(self.schema_version));
        }
        validate_channels(
            self.channels.iter().map(|channel| {
                (
                    &channel.channel_id,
                    channel.minimum_raw,
                    channel.maximum_raw,
                    channel.scale_denominator,
                )
            }),
            self.channels.len(),
        )?;
        if self
            .channels
            .windows(2)
            .any(|pair| pair[0].actuator_id >= pair[1].actuator_id)
        {
            return Err(MotorContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn layout_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header("nextengine.motor-action-layout.v1", self.schema_version)?;
        push_id(&mut bytes, &self.layout_id)?;
        bytes.extend_from_slice(self.body_schema_hash.as_bytes());
        bytes.extend_from_slice(self.actuator_profile_hash.as_bytes());
        push_len(&mut bytes, self.channels.len())?;
        for channel in &self.channels {
            push_id(&mut bytes, &channel.channel_id)?;
            bytes.push(channel.semantic as u8);
            push_id(&mut bytes, &channel.actuator_id)?;
            encode_bounds(
                &mut bytes,
                channel.minimum_raw,
                channel.maximum_raw,
                channel.scale_numerator,
                channel.scale_denominator,
            );
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyStateRecordV1 {
    pub schema_version: u16,
    pub policy_route_id: SchemaId,
    pub policy_bundle_hash: ContentHash,
    pub adaptation_state_raw: Vec<i64>,
    pub motion_generator_state_raw: Vec<i64>,
    pub expert_router_state_raw: Vec<i64>,
}

impl PolicyStateRecordV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != POLICY_STATE_RECORD_V1_SCHEMA_VERSION
            || self.adaptation_state_raw.len() > MAX_POLICY_STATE_VALUES
            || self.motion_generator_state_raw.len() > MAX_POLICY_STATE_VALUES
            || self.expert_router_state_raw.len() > MAX_POLICY_STATE_VALUES
        {
            return Err(MotorContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn state_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header("nextengine.policy-state.v1", self.schema_version)?;
        push_id(&mut bytes, &self.policy_route_id)?;
        bytes.extend_from_slice(self.policy_bundle_hash.as_bytes());
        for segment in [
            &self.adaptation_state_raw,
            &self.motion_generator_state_raw,
            &self.expert_router_state_raw,
        ] {
            push_len(&mut bytes, segment.len())?;
            for value in segment {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorWorldCheckpointV1 {
    pub schema_version: u16,
    pub subject_id: PersistentId,
    pub motor_tick: u64,
    pub cadence_substep: u16,
    pub observation_layout_hash: ContentHash,
    pub action_layout_hash: ContentHash,
    pub current_command_raw: Vec<i64>,
    pub current_action_raw: Vec<i64>,
    pub substep_efforts: Vec<Vec<AppliedActuatorEffortV1>>,
    pub policy_state: PolicyStateRecordV1,
    pub rng_stream_states: Vec<(SchemaId, [u8; 32])>,
}

impl MotorWorldCheckpointV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_WORLD_CHECKPOINT_V1_SCHEMA_VERSION
            || usize::from(self.cadence_substep) >= STAGE0_SUBSTEPS
            || self.current_command_raw.len() > MAX_MOTOR_CHANNELS
            || self.current_action_raw.len() > MAX_MOTOR_CHANNELS
            || self.substep_efforts.len() != STAGE0_SUBSTEPS
            || self
                .rng_stream_states
                .windows(2)
                .any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(MotorContractError::InvalidBounds);
        }
        self.policy_state.validate()?;
        for efforts in &self.substep_efforts {
            if efforts.len() > MAX_MOTOR_CHANNELS
                || efforts
                    .windows(2)
                    .any(|pair| pair[0].actuator_id >= pair[1].actuator_id)
            {
                return Err(MotorContractError::NonCanonicalOrder);
            }
        }
        Ok(())
    }

    pub fn checkpoint_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header("nextengine.motor-world-checkpoint.v1", self.schema_version)?;
        bytes.extend_from_slice(self.subject_id.as_bytes());
        bytes.extend_from_slice(&self.motor_tick.to_le_bytes());
        bytes.extend_from_slice(&self.cadence_substep.to_le_bytes());
        bytes.extend_from_slice(self.observation_layout_hash.as_bytes());
        bytes.extend_from_slice(self.action_layout_hash.as_bytes());
        encode_i64_values(&mut bytes, &self.current_command_raw)?;
        encode_i64_values(&mut bytes, &self.current_action_raw)?;
        push_len(&mut bytes, self.substep_efforts.len())?;
        for efforts in &self.substep_efforts {
            push_len(&mut bytes, efforts.len())?;
            for effort in efforts {
                push_id(&mut bytes, &effort.actuator_id)?;
                bytes.extend_from_slice(&effort.effort_micronewton_metres.to_le_bytes());
                bytes.extend_from_slice(&effort.clamp_flags.to_le_bytes());
            }
        }
        bytes.extend_from_slice(self.policy_state.state_hash()?.as_bytes());
        push_len(&mut bytes, self.rng_stream_states.len())?;
        for (purpose, state) in &self.rng_stream_states {
            push_id(&mut bytes, purpose)?;
            bytes.extend_from_slice(state);
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MotorRewardComponentV1 {
    pub component_id: SchemaId,
    pub coefficient_q16: i64,
    pub minimum_raw: i64,
    pub maximum_raw: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorTrainingEnvironmentManifestV1 {
    pub schema_version: u16,
    pub environment_id: SchemaId,
    pub body_schema_hash: ContentHash,
    pub body_instance_projection_hash: ContentHash,
    pub physics_catalog_hash: ContentHash,
    pub observation_layout_hash: ContentHash,
    pub action_layout_hash: ContentHash,
    pub physics_build_profile_hash: ContentHash,
    pub scene_profile_hash: ContentHash,
    pub bridge_abi_hash: ContentHash,
    pub quantization_profile_hash: ContentHash,
    pub translator_version_hash: ContentHash,
    pub physics_hz: u32,
    pub motor_hz: u32,
    pub maximum_vector_slots: u32,
    pub maximum_episode_steps: u64,
    pub reward_components: Vec<MotorRewardComponentV1>,
}

impl MotorTrainingEnvironmentManifestV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION
            || self.physics_hz != STAGE0_PHYSICS_HZ
            || self.motor_hz != STAGE0_MOTOR_HZ
            || self.maximum_vector_slots == 0
            || self.maximum_episode_steps == 0
            || self.reward_components.is_empty()
            || self.reward_components.len() > MAX_REWARD_COMPONENTS
            || self
                .reward_components
                .iter()
                .map(|component| &component.component_id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.reward_components.len()
            || self
                .reward_components
                .iter()
                .any(|component| component.minimum_raw > component.maximum_raw)
        {
            return Err(MotorContractError::InvalidManifest);
        }
        Ok(())
    }

    pub fn manifest_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-training-environment.v1",
            self.schema_version,
        )?;
        push_id(&mut bytes, &self.environment_id)?;
        for hash in [
            self.body_schema_hash,
            self.body_instance_projection_hash,
            self.physics_catalog_hash,
            self.observation_layout_hash,
            self.action_layout_hash,
            self.physics_build_profile_hash,
            self.scene_profile_hash,
            self.bridge_abi_hash,
            self.quantization_profile_hash,
            self.translator_version_hash,
        ] {
            bytes.extend_from_slice(hash.as_bytes());
        }
        bytes.extend_from_slice(&self.physics_hz.to_le_bytes());
        bytes.extend_from_slice(&self.motor_hz.to_le_bytes());
        bytes.extend_from_slice(&self.maximum_vector_slots.to_le_bytes());
        bytes.extend_from_slice(&self.maximum_episode_steps.to_le_bytes());
        push_len(&mut bytes, self.reward_components.len())?;
        for component in &self.reward_components {
            push_id(&mut bytes, &component.component_id)?;
            bytes.extend_from_slice(&component.coefficient_q16.to_le_bytes());
            bytes.extend_from_slice(&component.minimum_raw.to_le_bytes());
            bytes.extend_from_slice(&component.maximum_raw.to_le_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }

    pub fn validate_for_protocol_v2(&self) -> Result<(), MotorContractError> {
        Err(MotorContractError::UnsupportedEnvironmentManifestVersion(
            self.schema_version,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MotorLocomotionCommandModeV1 {
    Stop = 0,
    Translation = 1,
    Turn = 2,
    Combined = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorLocomotionCommandProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub randomization_stream_id: SchemaId,
    pub warmup_ticks: u32,
    pub segment_ticks: u32,
    pub episode_ticks: u32,
    pub mode_weights_basis_points: [u16; 4],
    pub right_velocity_min_micrometres_per_second: i64,
    pub right_velocity_max_micrometres_per_second: i64,
    pub forward_velocity_min_micrometres_per_second: i64,
    pub forward_velocity_max_micrometres_per_second: i64,
    pub yaw_rate_min_microradians_per_second: i64,
    pub yaw_rate_max_microradians_per_second: i64,
    pub linear_rate_limit_micrometres_per_second_squared: u64,
    pub yaw_rate_limit_microradians_per_second_squared: u64,
}

impl MotorLocomotionCommandProfileV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        let total_weight = self
            .mode_weights_basis_points
            .iter()
            .try_fold(0_u32, |total, weight| total.checked_add(u32::from(*weight)))
            .ok_or(MotorContractError::InvalidBounds)?;
        if self.schema_version != MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION
            || self.warmup_ticks == 0
            || self.segment_ticks == 0
            || self.episode_ticks <= self.warmup_ticks
            || total_weight != 10_000
            || self.right_velocity_min_micrometres_per_second
                > self.right_velocity_max_micrometres_per_second
            || self.forward_velocity_min_micrometres_per_second
                > self.forward_velocity_max_micrometres_per_second
            || self.yaw_rate_min_microradians_per_second > self.yaw_rate_max_microradians_per_second
            || self.linear_rate_limit_micrometres_per_second_squared == 0
            || self.yaw_rate_limit_microradians_per_second_squared == 0
        {
            return Err(MotorContractError::InvalidManifest);
        }
        Ok(())
    }

    pub fn profile_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-locomotion-command-profile.v1",
            self.schema_version,
        )?;
        push_id(&mut bytes, &self.profile_id)?;
        push_id(&mut bytes, &self.randomization_stream_id)?;
        bytes.extend_from_slice(&self.warmup_ticks.to_le_bytes());
        bytes.extend_from_slice(&self.segment_ticks.to_le_bytes());
        bytes.extend_from_slice(&self.episode_ticks.to_le_bytes());
        for weight in self.mode_weights_basis_points {
            bytes.extend_from_slice(&weight.to_le_bytes());
        }
        for value in [
            self.right_velocity_min_micrometres_per_second,
            self.right_velocity_max_micrometres_per_second,
            self.forward_velocity_min_micrometres_per_second,
            self.forward_velocity_max_micrometres_per_second,
            self.yaw_rate_min_microradians_per_second,
            self.yaw_rate_max_microradians_per_second,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(
            &self
                .linear_rate_limit_micrometres_per_second_squared
                .to_le_bytes(),
        );
        bytes.extend_from_slice(
            &self
                .yaw_rate_limit_microradians_per_second_squared
                .to_le_bytes(),
        );
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorTrainingEnvironmentManifestV2 {
    pub schema_version: u16,
    pub environment_id: SchemaId,
    pub body_schema_hash: ContentHash,
    pub body_instance_projection_hash: ContentHash,
    pub physics_catalog_hash: ContentHash,
    pub observation_layout_hash: ContentHash,
    pub action_layout_hash: ContentHash,
    pub physics_build_profile_hash: ContentHash,
    pub scene_profile_hash: ContentHash,
    pub bridge_abi_hash: ContentHash,
    pub quantization_profile_hash: ContentHash,
    pub translator_version_hash: ContentHash,
    pub command_schedule_profile_hash: ContentHash,
    pub reward_profile_hash: ContentHash,
    pub termination_profile_hash: ContentHash,
    pub rng_derivation_profile_hash: ContentHash,
    pub correspondence_profile_hash: ContentHash,
    pub physics_hz: u32,
    pub motor_hz: u32,
    pub maximum_vector_slots: u32,
    pub maximum_episode_steps: u64,
    pub reward_components: Vec<MotorRewardComponentV1>,
}

impl MotorTrainingEnvironmentManifestV2 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION
            || self.physics_hz != STAGE0_PHYSICS_HZ
            || self.motor_hz != STAGE0_MOTOR_HZ
            || self.maximum_vector_slots == 0
            || self.maximum_episode_steps == 0
            || self.reward_components.is_empty()
            || self.reward_components.len() > MAX_REWARD_COMPONENTS
            || self
                .reward_components
                .iter()
                .map(|component| &component.component_id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.reward_components.len()
            || self
                .reward_components
                .iter()
                .any(|component| component.minimum_raw > component.maximum_raw)
        {
            return Err(MotorContractError::InvalidManifest);
        }
        Ok(())
    }

    pub fn validate_for_protocol_v2(&self) -> Result<(), MotorContractError> {
        self.validate()
    }

    pub fn manifest_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-training-environment.v2",
            self.schema_version,
        )?;
        push_id(&mut bytes, &self.environment_id)?;
        for hash in [
            self.body_schema_hash,
            self.body_instance_projection_hash,
            self.physics_catalog_hash,
            self.observation_layout_hash,
            self.action_layout_hash,
            self.physics_build_profile_hash,
            self.scene_profile_hash,
            self.bridge_abi_hash,
            self.quantization_profile_hash,
            self.translator_version_hash,
            self.command_schedule_profile_hash,
            self.reward_profile_hash,
            self.termination_profile_hash,
            self.rng_derivation_profile_hash,
            self.correspondence_profile_hash,
        ] {
            bytes.extend_from_slice(hash.as_bytes());
        }
        bytes.extend_from_slice(&self.physics_hz.to_le_bytes());
        bytes.extend_from_slice(&self.motor_hz.to_le_bytes());
        bytes.extend_from_slice(&self.maximum_vector_slots.to_le_bytes());
        bytes.extend_from_slice(&self.maximum_episode_steps.to_le_bytes());
        push_len(&mut bytes, self.reward_components.len())?;
        for component in &self.reward_components {
            push_id(&mut bytes, &component.component_id)?;
            bytes.extend_from_slice(&component.coefficient_q16.to_le_bytes());
            bytes.extend_from_slice(&component.minimum_raw.to_le_bytes());
            bytes.extend_from_slice(&component.maximum_raw.to_le_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorEpisodeSeedSetV1 {
    pub schema_version: u16,
    pub run_root: ContentHash,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub purpose_seeds: Vec<(SchemaId, [u8; 32])>,
}

impl MotorEpisodeSeedSetV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION
            || self.purpose_seeds.is_empty()
            || self
                .purpose_seeds
                .windows(2)
                .any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(MotorContractError::NonCanonicalOrder);
        }
        Ok(())
    }

    pub fn seed_set_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header("nextengine.motor-episode-seeds.v1", self.schema_version)?;
        bytes.extend_from_slice(self.run_root.as_bytes());
        bytes.extend_from_slice(&self.episode_ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.vector_slot.to_le_bytes());
        push_len(&mut bytes, self.purpose_seeds.len())?;
        for (purpose, seed) in &self.purpose_seeds {
            push_id(&mut bytes, purpose)?;
            bytes.extend_from_slice(seed);
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorResetRecordV1 {
    pub schema_version: u16,
    pub environment_manifest_hash: ContentHash,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub seed_set_hash: ContentHash,
    pub physics_checkpoint_hash: ContentHash,
    pub motor_checkpoint_hash: ContentHash,
    pub reset_root: StateRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorStepRecordV1 {
    pub schema_version: u16,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub motor_tick: u64,
    pub command_raw: Vec<i64>,
    pub action_raw: Vec<i64>,
    pub substep_efforts: Vec<Vec<AppliedActuatorEffortV1>>,
    pub observation_hash: ContentHash,
    pub physics_snapshot_hash: ContentHash,
    pub motor_checkpoint_hash: ContentHash,
    pub reward_components_raw: Vec<i64>,
    pub terminal_reason_id: Option<SchemaId>,
    pub step_root: StateRoot,
}

impl MotorStepRecordV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_STEP_RECORD_V1_SCHEMA_VERSION
            || self.command_raw.len() > MAX_MOTOR_CHANNELS
            || self.action_raw.len() > MAX_MOTOR_CHANNELS
            || self.substep_efforts.len() != STAGE0_SUBSTEPS
            || self.reward_components_raw.len() > MAX_REWARD_COMPONENTS
        {
            return Err(MotorContractError::InvalidBounds);
        }
        for efforts in &self.substep_efforts {
            if efforts
                .windows(2)
                .any(|pair| pair[0].actuator_id >= pair[1].actuator_id)
            {
                return Err(MotorContractError::NonCanonicalOrder);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorTrajectoryManifestV1 {
    pub schema_version: u16,
    pub environment_manifest_hash: ContentHash,
    pub reset_record_hash: ContentHash,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub step_count: u64,
    pub ordered_step_root: ContentHash,
    pub final_physics_checkpoint_hash: ContentHash,
    pub final_motor_checkpoint_hash: ContentHash,
    pub terminal_reason_id: SchemaId,
}

impl MotorTrajectoryManifestV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_TRAJECTORY_MANIFEST_V1_SCHEMA_VERSION
            || self.step_count == 0
        {
            return Err(MotorContractError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum MotorTerminalDispositionV1 {
    Running = 0,
    Terminated = 1,
    Truncated = 2,
}

impl MotorTerminalDispositionV1 {
    fn from_u8(value: u8) -> Result<Self, MotorContractError> {
        match value {
            0 => Ok(Self::Running),
            1 => Ok(Self::Terminated),
            2 => Ok(Self::Truncated),
            _ => Err(MotorContractError::InvalidBounds),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorResetRecordV2 {
    pub schema_version: u16,
    pub environment_manifest_hash: ContentHash,
    pub environment_profile_id: SchemaId,
    pub run_root: ContentHash,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub seed_set_hash: ContentHash,
    pub initial_observation_root: ContentHash,
    pub physics_root: ContentHash,
    pub motor_root: ContentHash,
    pub reset_root: StateRoot,
}

impl MotorResetRecordV2 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_RESET_RECORD_V2_SCHEMA_VERSION {
            return Err(MotorContractError::UnsupportedVersion(self.schema_version));
        }
        if self.reset_root != self.computed_reset_root()? {
            return Err(MotorContractError::RootMismatch);
        }
        Ok(())
    }

    pub fn computed_reset_root(&self) -> Result<StateRoot, MotorContractError> {
        let mut bytes = header("nextengine.motor-reset-record.v2", self.schema_version)?;
        bytes.extend_from_slice(self.environment_manifest_hash.as_bytes());
        push_id(&mut bytes, &self.environment_profile_id)?;
        bytes.extend_from_slice(self.run_root.as_bytes());
        bytes.extend_from_slice(&self.episode_ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.vector_slot.to_le_bytes());
        bytes.extend_from_slice(self.seed_set_hash.as_bytes());
        bytes.extend_from_slice(self.initial_observation_root.as_bytes());
        bytes.extend_from_slice(self.physics_root.as_bytes());
        bytes.extend_from_slice(self.motor_root.as_bytes());
        Ok(StateRoot::from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorStepRecordV2 {
    pub schema_version: u16,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub motor_tick: u64,
    pub prior_observation_root: ContentHash,
    pub next_observation_root: ContentHash,
    pub applied_command_raw: [i64; 3],
    pub next_command_raw: [i64; 3],
    pub applied_action_raw: Vec<i64>,
    pub reward_components_q16: Vec<i64>,
    pub reward_total_q16: i64,
    pub terminated: bool,
    pub truncated: bool,
    pub terminal_reason_id: Option<SchemaId>,
    pub physics_root: ContentHash,
    pub motor_root: ContentHash,
    pub prior_step_root: StateRoot,
    pub step_root: StateRoot,
}

impl MotorStepRecordV2 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_STEP_RECORD_V2_SCHEMA_VERSION {
            return Err(MotorContractError::UnsupportedVersion(self.schema_version));
        }
        if self.applied_action_raw.len() > MAX_MOTOR_CHANNELS
            || self.reward_components_q16.is_empty()
            || self.reward_components_q16.len() > MAX_REWARD_COMPONENTS
            || self
                .reward_components_q16
                .iter()
                .any(|value| !(0..=65_536).contains(value))
            || (self.terminated && self.truncated)
            || ((self.terminated || self.truncated) != self.terminal_reason_id.is_some())
            || self.step_root != self.computed_step_root()?
        {
            return Err(MotorContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn computed_step_root(&self) -> Result<StateRoot, MotorContractError> {
        let mut bytes = header("nextengine.motor-step-record.v2", self.schema_version)?;
        bytes.extend_from_slice(&self.episode_ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.vector_slot.to_le_bytes());
        bytes.extend_from_slice(&self.motor_tick.to_le_bytes());
        bytes.extend_from_slice(self.prior_observation_root.as_bytes());
        bytes.extend_from_slice(self.next_observation_root.as_bytes());
        for value in self.applied_command_raw {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self.next_command_raw {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        encode_i64_values(&mut bytes, &self.applied_action_raw)?;
        encode_i64_values(&mut bytes, &self.reward_components_q16)?;
        bytes.extend_from_slice(&self.reward_total_q16.to_le_bytes());
        bytes.push(u8::from(self.terminated));
        bytes.push(u8::from(self.truncated));
        match &self.terminal_reason_id {
            Some(reason) => {
                bytes.push(1);
                push_id(&mut bytes, reason)?;
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(self.physics_root.as_bytes());
        bytes.extend_from_slice(self.motor_root.as_bytes());
        bytes.extend_from_slice(self.prior_step_root.as_bytes());
        Ok(StateRoot::from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorTrajectoryManifestV2 {
    pub schema_version: u16,
    pub environment_manifest_hash: ContentHash,
    pub reset_root: StateRoot,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub step_count: u64,
    pub final_step_root: StateRoot,
    pub final_observation_root: ContentHash,
    pub final_physics_root: ContentHash,
    pub final_motor_root: ContentHash,
    pub terminal_disposition: MotorTerminalDispositionV1,
    pub terminal_reason_id: SchemaId,
}

impl MotorTrajectoryManifestV2 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_TRAJECTORY_MANIFEST_V2_SCHEMA_VERSION
            || self.step_count == 0
            || self.terminal_disposition == MotorTerminalDispositionV1::Running
        {
            return Err(MotorContractError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorEnvironmentCheckpointEnvelopeV1 {
    pub schema_version: u16,
    pub environment_profile_id: SchemaId,
    pub environment_manifest_hash: ContentHash,
    pub run_root: ContentHash,
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub motor_tick: u64,
    pub terminal_disposition: MotorTerminalDispositionV1,
    pub terminal_reason_id: Option<SchemaId>,
    pub current_observation_root: ContentHash,
    pub last_step_root: StateRoot,
    pub motor_runtime_checkpoint_bytes: Vec<u8>,
    pub motor_runtime_checkpoint_hash: ContentHash,
}

impl MotorEnvironmentCheckpointEnvelopeV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION {
            return Err(MotorContractError::UnsupportedVersion(self.schema_version));
        }
        if self.motor_runtime_checkpoint_bytes.is_empty()
            || self.motor_runtime_checkpoint_bytes.len() > MAX_MOTOR_ENVIRONMENT_CHECKPOINT_BYTES
            || content_hash_from_bytes(sha256(&self.motor_runtime_checkpoint_bytes))
                != self.motor_runtime_checkpoint_hash
            || ((self.terminal_disposition == MotorTerminalDispositionV1::Running)
                != self.terminal_reason_id.is_none())
        {
            return Err(MotorContractError::InvalidBounds);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-environment-checkpoint-envelope.v1",
            self.schema_version,
        )?;
        push_id(&mut bytes, &self.environment_profile_id)?;
        bytes.extend_from_slice(self.environment_manifest_hash.as_bytes());
        bytes.extend_from_slice(self.run_root.as_bytes());
        bytes.extend_from_slice(&self.episode_ordinal.to_le_bytes());
        bytes.extend_from_slice(&self.vector_slot.to_le_bytes());
        bytes.extend_from_slice(&self.motor_tick.to_le_bytes());
        bytes.push(self.terminal_disposition as u8);
        match &self.terminal_reason_id {
            Some(reason) => {
                bytes.push(1);
                push_id(&mut bytes, reason)?;
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(self.current_observation_root.as_bytes());
        bytes.extend_from_slice(self.last_step_root.as_bytes());
        push_len(&mut bytes, self.motor_runtime_checkpoint_bytes.len())?;
        bytes.extend_from_slice(&self.motor_runtime_checkpoint_bytes);
        bytes.extend_from_slice(self.motor_runtime_checkpoint_hash.as_bytes());
        Ok(bytes)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, MotorContractError> {
        let mut cursor = MotorByteCursor::new(bytes);
        cursor.expect_header(
            "nextengine.motor-environment-checkpoint-envelope.v1",
            MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
        )?;
        let environment_profile_id = cursor.schema_id()?;
        let environment_manifest_hash = ContentHash::from_bytes(cursor.array()?);
        let run_root = ContentHash::from_bytes(cursor.array()?);
        let episode_ordinal = cursor.u64()?;
        let vector_slot = cursor.u32()?;
        let motor_tick = cursor.u64()?;
        let terminal_disposition = MotorTerminalDispositionV1::from_u8(cursor.u8()?)?;
        let terminal_reason_id = match cursor.u8()? {
            0 => None,
            1 => Some(cursor.schema_id()?),
            _ => return Err(MotorContractError::InvalidBounds),
        };
        let current_observation_root = ContentHash::from_bytes(cursor.array()?);
        let last_step_root = StateRoot::from_bytes(cursor.array()?);
        let checkpoint_length = cursor.u32()? as usize;
        if checkpoint_length > MAX_MOTOR_ENVIRONMENT_CHECKPOINT_BYTES {
            return Err(MotorContractError::InvalidBounds);
        }
        let motor_runtime_checkpoint_bytes = cursor.bytes(checkpoint_length)?.to_vec();
        let motor_runtime_checkpoint_hash = ContentHash::from_bytes(cursor.array()?);
        if !cursor.is_empty() {
            return Err(MotorContractError::InvalidBounds);
        }
        let value = Self {
            schema_version: MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
            environment_profile_id,
            environment_manifest_hash,
            run_root,
            episode_ordinal,
            vector_slot,
            motor_tick,
            terminal_disposition,
            terminal_reason_id,
            current_observation_root,
            last_step_root,
            motor_runtime_checkpoint_bytes,
            motor_runtime_checkpoint_hash,
        };
        value.validate()?;
        if value.canonical_bytes()? != bytes {
            return Err(MotorContractError::NonCanonicalOrder);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MotorContractError {
    UnsupportedVersion(u16),
    UnsupportedEnvironmentManifestVersion(u16),
    InvalidBounds,
    InvalidManifest,
    NonCanonicalOrder,
    LengthOverflow,
    RootMismatch,
    Physics(crate::physics::PhysicsContractError),
}

impl MotorContractError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion(_) => "UNSUPPORTED_MOTOR_SCHEMA_VERSION",
            Self::UnsupportedEnvironmentManifestVersion(_) => {
                "UNSUPPORTED_MOTOR_ENVIRONMENT_MANIFEST_VERSION"
            }
            Self::InvalidBounds => "MOTOR_BOUNDS_INVALID",
            Self::InvalidManifest => "MOTOR_ENVIRONMENT_MANIFEST_INVALID",
            Self::NonCanonicalOrder => "MOTOR_ORDER_INVALID",
            Self::LengthOverflow => "MOTOR_LENGTH_OVERFLOW",
            Self::RootMismatch => "MOTOR_ROOT_MISMATCH",
            Self::Physics(_) => "MOTOR_PHYSICS_CLOSURE_INVALID",
        }
    }
}

impl Display for MotorContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for MotorContractError {}

impl From<crate::physics::PhysicsContractError> for MotorContractError {
    fn from(value: crate::physics::PhysicsContractError) -> Self {
        Self::Physics(value)
    }
}

pub fn validate_checkpoint_closure(
    physics: &PhysicsWorldCheckpointV2,
    motor: &MotorWorldCheckpointV1,
) -> Result<(), MotorContractError> {
    physics.validate()?;
    motor.validate()?;
    if physics.catalog.body_schema_hash != motor.action_layout_hash
        && physics.catalog.body_schema_hash != motor.observation_layout_hash
    {
        // Layout hashes are not schema hashes; this guard merely rejects the
        // degenerate all-default closure. Exact relationships live in the
        // save/replay compatibility record.
        if motor.action_layout_hash == ContentHash::default()
            || motor.observation_layout_hash == ContentHash::default()
        {
            return Err(MotorContractError::InvalidManifest);
        }
    }
    Ok(())
}

fn validate_channels<'a>(
    channels: impl Iterator<Item = (&'a SchemaId, i64, i64, u64)>,
    count: usize,
) -> Result<(), MotorContractError> {
    if count == 0 || count > MAX_MOTOR_CHANNELS {
        return Err(MotorContractError::InvalidBounds);
    }
    let mut previous: Option<&SchemaId> = None;
    for (id, minimum, maximum, denominator) in channels {
        if previous.is_some_and(|previous| previous >= id) || minimum > maximum || denominator == 0
        {
            return Err(MotorContractError::NonCanonicalOrder);
        }
        previous = Some(id);
    }
    Ok(())
}

fn header(domain: &str, version: u16) -> Result<Vec<u8>, MotorContractError> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, domain.as_bytes())?;
    bytes.extend_from_slice(&version.to_le_bytes());
    Ok(bytes)
}

fn push_id(bytes: &mut Vec<u8>, id: &SchemaId) -> Result<(), MotorContractError> {
    push_bytes(bytes, id.as_str().as_bytes())
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), MotorContractError> {
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| MotorContractError::LengthOverflow)?
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_len(bytes: &mut Vec<u8>, value: usize) -> Result<(), MotorContractError> {
    bytes.extend_from_slice(
        &u32::try_from(value)
            .map_err(|_| MotorContractError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

fn encode_bounds(
    bytes: &mut Vec<u8>,
    minimum: i64,
    maximum: i64,
    numerator: i64,
    denominator: u64,
) {
    bytes.extend_from_slice(&minimum.to_le_bytes());
    bytes.extend_from_slice(&maximum.to_le_bytes());
    bytes.extend_from_slice(&numerator.to_le_bytes());
    bytes.extend_from_slice(&denominator.to_le_bytes());
}

fn encode_i64_values(bytes: &mut Vec<u8>, values: &[i64]) -> Result<(), MotorContractError> {
    push_len(bytes, values.len())?;
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

struct MotorByteCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> MotorByteCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn expect_header(&mut self, domain: &str, version: u16) -> Result<(), MotorContractError> {
        let encoded_domain = self.length_prefixed_bytes()?;
        if encoded_domain != domain.as_bytes() {
            return Err(MotorContractError::InvalidBounds);
        }
        let encoded_version = self.u16()?;
        if encoded_version != version {
            return Err(MotorContractError::UnsupportedVersion(encoded_version));
        }
        Ok(())
    }

    fn schema_id(&mut self) -> Result<SchemaId, MotorContractError> {
        let value = std::str::from_utf8(self.length_prefixed_bytes()?)
            .map_err(|_| MotorContractError::InvalidBounds)?;
        SchemaId::new(value).map_err(|_| MotorContractError::InvalidBounds)
    }

    fn length_prefixed_bytes(&mut self) -> Result<&'a [u8], MotorContractError> {
        let length = self.u32()? as usize;
        self.bytes(length)
    }

    fn bytes(&mut self, length: usize) -> Result<&'a [u8], MotorContractError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(MotorContractError::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(MotorContractError::InvalidBounds)?;
        self.offset = end;
        Ok(value)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], MotorContractError> {
        self.bytes(N)?
            .try_into()
            .map_err(|_| MotorContractError::InvalidBounds)
    }

    fn u8(&mut self) -> Result<u8, MotorContractError> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, MotorContractError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, MotorContractError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, MotorContractError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test identifier")
    }

    #[test]
    fn action_layout_rejects_implicit_or_reordered_actuators() {
        let mut layout = MotorActionLayoutV1 {
            schema_version: MOTOR_ACTION_LAYOUT_V1_SCHEMA_VERSION,
            layout_id: id("nextengine.motor.action.test"),
            body_schema_hash: content_hash_from_bytes([1; 32]),
            actuator_profile_hash: content_hash_from_bytes([2; 32]),
            channels: vec![
                MotorActionChannelV1 {
                    channel_id: id("nextengine.channel.a"),
                    semantic: MotorActionSemanticV1::ResidualJointPosition,
                    actuator_id: id("nextengine.actuator.a"),
                    minimum_raw: -1,
                    maximum_raw: 1,
                    scale_numerator: 1,
                    scale_denominator: 1,
                },
                MotorActionChannelV1 {
                    channel_id: id("nextengine.channel.b"),
                    semantic: MotorActionSemanticV1::ResidualJointPosition,
                    actuator_id: id("nextengine.actuator.b"),
                    minimum_raw: -1,
                    maximum_raw: 1,
                    scale_numerator: 1,
                    scale_denominator: 1,
                },
            ],
        };
        assert!(layout.validate().is_ok());
        layout.channels.swap(0, 1);
        assert_eq!(
            layout.validate(),
            Err(MotorContractError::NonCanonicalOrder)
        );
    }

    #[test]
    fn stage0_manifest_requires_240_over_60_profile() {
        let mut manifest = MotorTrainingEnvironmentManifestV1 {
            schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION,
            environment_id: id("nextengine.motor.env.test"),
            body_schema_hash: content_hash_from_bytes([1; 32]),
            body_instance_projection_hash: content_hash_from_bytes([2; 32]),
            physics_catalog_hash: content_hash_from_bytes([3; 32]),
            observation_layout_hash: content_hash_from_bytes([4; 32]),
            action_layout_hash: content_hash_from_bytes([5; 32]),
            physics_build_profile_hash: content_hash_from_bytes([6; 32]),
            scene_profile_hash: content_hash_from_bytes([7; 32]),
            bridge_abi_hash: content_hash_from_bytes([8; 32]),
            quantization_profile_hash: content_hash_from_bytes([9; 32]),
            translator_version_hash: content_hash_from_bytes([10; 32]),
            physics_hz: STAGE0_PHYSICS_HZ,
            motor_hz: STAGE0_MOTOR_HZ,
            maximum_vector_slots: 128,
            maximum_episode_steps: 14_400,
            reward_components: vec![MotorRewardComponentV1 {
                component_id: id("nextengine.reward.upright"),
                coefficient_q16: 65_536,
                minimum_raw: 0,
                maximum_raw: 65_536,
            }],
        };
        assert!(manifest.validate().is_ok());
        manifest.physics_hz = 120;
        assert_eq!(
            manifest.validate(),
            Err(MotorContractError::InvalidManifest)
        );
    }

    #[test]
    fn reward_component_order_is_semantic_but_duplicate_ids_are_rejected() {
        let mut manifest = MotorTrainingEnvironmentManifestV1 {
            schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION,
            environment_id: id("nextengine.motor.env.reward-order"),
            body_schema_hash: content_hash_from_bytes([1; 32]),
            body_instance_projection_hash: content_hash_from_bytes([2; 32]),
            physics_catalog_hash: content_hash_from_bytes([3; 32]),
            observation_layout_hash: content_hash_from_bytes([4; 32]),
            action_layout_hash: content_hash_from_bytes([5; 32]),
            physics_build_profile_hash: content_hash_from_bytes([6; 32]),
            scene_profile_hash: content_hash_from_bytes([7; 32]),
            bridge_abi_hash: content_hash_from_bytes([8; 32]),
            quantization_profile_hash: content_hash_from_bytes([9; 32]),
            translator_version_hash: content_hash_from_bytes([10; 32]),
            physics_hz: STAGE0_PHYSICS_HZ,
            motor_hz: STAGE0_MOTOR_HZ,
            maximum_vector_slots: 4_096,
            maximum_episode_steps: 3_600,
            reward_components: ["reward.upright", "reward.action-rate-penalty"]
                .map(|component_id| MotorRewardComponentV1 {
                    component_id: id(component_id),
                    coefficient_q16: 65_536,
                    minimum_raw: i64::MIN,
                    maximum_raw: i64::MAX,
                })
                .into(),
        };
        assert!(manifest.validate().is_ok());
        manifest.reward_components[1].component_id = id("reward.upright");
        assert_eq!(
            manifest.validate(),
            Err(MotorContractError::InvalidManifest)
        );
    }

    #[test]
    fn protocol_v2_rejects_v1_manifest_with_typed_version_error() {
        let manifest = MotorTrainingEnvironmentManifestV1 {
            schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V1_SCHEMA_VERSION,
            environment_id: id("nextengine.motor.env.legacy"),
            body_schema_hash: content_hash_from_bytes([1; 32]),
            body_instance_projection_hash: content_hash_from_bytes([2; 32]),
            physics_catalog_hash: content_hash_from_bytes([3; 32]),
            observation_layout_hash: content_hash_from_bytes([4; 32]),
            action_layout_hash: content_hash_from_bytes([5; 32]),
            physics_build_profile_hash: content_hash_from_bytes([6; 32]),
            scene_profile_hash: content_hash_from_bytes([7; 32]),
            bridge_abi_hash: content_hash_from_bytes([8; 32]),
            quantization_profile_hash: content_hash_from_bytes([9; 32]),
            translator_version_hash: content_hash_from_bytes([10; 32]),
            physics_hz: STAGE0_PHYSICS_HZ,
            motor_hz: STAGE0_MOTOR_HZ,
            maximum_vector_slots: 1,
            maximum_episode_steps: 1,
            reward_components: vec![MotorRewardComponentV1 {
                component_id: id("reward.test"),
                coefficient_q16: 65_536,
                minimum_raw: 0,
                maximum_raw: 65_536,
            }],
        };
        let error = manifest
            .validate_for_protocol_v2()
            .expect_err("v1 is not a protocol-v2 manifest");
        assert_eq!(
            error.stable_code(),
            "UNSUPPORTED_MOTOR_ENVIRONMENT_MANIFEST_VERSION"
        );
    }

    #[test]
    fn checkpoint_envelope_round_trips_and_rejects_payload_tamper() {
        let payload = vec![1, 2, 3, 4];
        let envelope = MotorEnvironmentCheckpointEnvelopeV1 {
            schema_version: MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
            environment_profile_id: id("nextengine.motor.env.test"),
            environment_manifest_hash: content_hash_from_bytes([1; 32]),
            run_root: content_hash_from_bytes([2; 32]),
            episode_ordinal: 7,
            vector_slot: 3,
            motor_tick: 11,
            terminal_disposition: MotorTerminalDispositionV1::Running,
            terminal_reason_id: None,
            current_observation_root: content_hash_from_bytes([3; 32]),
            last_step_root: StateRoot::from_bytes([4; 32]),
            motor_runtime_checkpoint_hash: content_hash_from_bytes(sha256(&payload)),
            motor_runtime_checkpoint_bytes: payload,
        };
        let bytes = envelope.canonical_bytes().expect("encode");
        assert_eq!(
            MotorEnvironmentCheckpointEnvelopeV1::from_canonical_bytes(&bytes).expect("decode"),
            envelope
        );

        let mut tampered = envelope;
        tampered.motor_runtime_checkpoint_bytes[0] ^= 1;
        assert_eq!(tampered.validate(), Err(MotorContractError::InvalidBounds));
    }
}
