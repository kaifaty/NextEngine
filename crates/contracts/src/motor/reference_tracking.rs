use crate::canonical::sha256;
use crate::ids::{ContentHash, SchemaId, content_hash_from_bytes};

use super::{
    MAX_REWARD_COMPONENTS, MotorContractError, MotorTrainingEnvironmentManifestV2, STAGE0_MOTOR_HZ,
    STAGE0_PHYSICS_HZ, header, push_id, push_len,
};

pub const MOTOR_REFERENCE_TRACKING_PROFILE_V1_SCHEMA_VERSION: u16 = 1;
pub const MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V3_SCHEMA_VERSION: u16 = 3;
pub const HUMANOID_REFERENCE_TRACKER_CHANNELS: u32 = 435;
pub const HUMANOID_REFERENCE_TRACKER_ACTIONS: u32 = 23;
pub const HUMANOID_REFERENCE_HORIZON_OFFSETS: [u32; 4] = [0, 4, 8, 16];

/// Immutable, path-free closure for the ADR-070 biomechanics reference task.
///
/// The repository JSON document owns the detailed channel/reward definitions;
/// `profile_document_sha256` binds those bytes while the duplicated essential
/// bounds below let every consumer reject a semantically incompatible profile
/// before resolving the external corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorReferenceTrackingProfileV1 {
    pub schema_version: u16,
    pub profile_id: SchemaId,
    pub body_schema_hash: ContentHash,
    pub compiled_descriptor_hash: ContentHash,
    pub corpus_profile_hash: ContentHash,
    pub corpus_manifest_hash: ContentHash,
    pub profile_document_sha256: ContentHash,
    pub observation_layout_hash: ContentHash,
    pub action_layout_hash: ContentHash,
    pub reset_profile_hash: ContentHash,
    pub reward_profile_hash: ContentHash,
    pub termination_profile_hash: ContentHash,
    pub rng_derivation_profile_hash: ContentHash,
    pub eligible_partition_id: SchemaId,
    pub clip_selection_stream_id: SchemaId,
    pub phase_selection_stream_id: SchemaId,
    pub reset_mode_stream_id: SchemaId,
    pub physics_hz: u32,
    pub motor_hz: u32,
    pub reference_hz: u32,
    pub horizon_offsets_motor_ticks: [u32; 4],
    pub reset_mode_weights_basis_points: [u16; 4],
    pub actor_channel_count: u32,
    pub critic_channel_count: u32,
    pub action_channel_count: u32,
    pub success_reason_id: SchemaId,
    pub failure_reason_ids: Vec<SchemaId>,
}

impl MotorReferenceTrackingProfileV1 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        let reset_weight = self
            .reset_mode_weights_basis_points
            .iter()
            .try_fold(0_u32, |total, weight| total.checked_add(u32::from(*weight)))
            .ok_or(MotorContractError::InvalidBounds)?;
        let required_hashes = [
            self.body_schema_hash,
            self.compiled_descriptor_hash,
            self.corpus_profile_hash,
            self.corpus_manifest_hash,
            self.profile_document_sha256,
            self.observation_layout_hash,
            self.action_layout_hash,
            self.reset_profile_hash,
            self.reward_profile_hash,
            self.termination_profile_hash,
            self.rng_derivation_profile_hash,
        ];
        if self.schema_version != MOTOR_REFERENCE_TRACKING_PROFILE_V1_SCHEMA_VERSION
            || required_hashes.contains(&ContentHash::default())
            || self.physics_hz != STAGE0_PHYSICS_HZ
            || self.motor_hz != STAGE0_MOTOR_HZ
            || self.reference_hz != STAGE0_MOTOR_HZ
            || self.horizon_offsets_motor_ticks != HUMANOID_REFERENCE_HORIZON_OFFSETS
            || reset_weight != 10_000
            || self.reset_mode_weights_basis_points[3] != 0
            || self.actor_channel_count != HUMANOID_REFERENCE_TRACKER_CHANNELS
            || self.critic_channel_count != HUMANOID_REFERENCE_TRACKER_CHANNELS
            || self.action_channel_count != HUMANOID_REFERENCE_TRACKER_ACTIONS
            || self.failure_reason_ids.is_empty()
            || self.failure_reason_ids.len() > MAX_REWARD_COMPONENTS
            || self
                .failure_reason_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(MotorContractError::InvalidManifest);
        }
        Ok(())
    }

    pub fn profile_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-reference-tracking-profile.v1",
            self.schema_version,
        )?;
        push_id(&mut bytes, &self.profile_id)?;
        for hash in [
            self.body_schema_hash,
            self.compiled_descriptor_hash,
            self.corpus_profile_hash,
            self.corpus_manifest_hash,
            self.profile_document_sha256,
            self.observation_layout_hash,
            self.action_layout_hash,
            self.reset_profile_hash,
            self.reward_profile_hash,
            self.termination_profile_hash,
            self.rng_derivation_profile_hash,
        ] {
            bytes.extend_from_slice(hash.as_bytes());
        }
        for id in [
            &self.eligible_partition_id,
            &self.clip_selection_stream_id,
            &self.phase_selection_stream_id,
            &self.reset_mode_stream_id,
        ] {
            push_id(&mut bytes, id)?;
        }
        for value in [self.physics_hz, self.motor_hz, self.reference_hz] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for offset in self.horizon_offsets_motor_ticks {
            bytes.extend_from_slice(&offset.to_le_bytes());
        }
        for weight in self.reset_mode_weights_basis_points {
            bytes.extend_from_slice(&weight.to_le_bytes());
        }
        for count in [
            self.actor_channel_count,
            self.critic_channel_count,
            self.action_channel_count,
        ] {
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        push_id(&mut bytes, &self.success_reason_id)?;
        push_len(&mut bytes, self.failure_reason_ids.len())?;
        for reason in &self.failure_reason_ids {
            push_id(&mut bytes, reason)?;
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

/// V3 preserves the executable V2 environment closure and adds immutable task
/// input/provenance roots. Existing V2 reset, step, trajectory and checkpoint
/// records bind this manifest through `environment_manifest_hash`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotorTrainingEnvironmentManifestV3 {
    pub schema_version: u16,
    pub base: MotorTrainingEnvironmentManifestV2,
    pub task_input_profile_hash: ContentHash,
    pub input_provenance_root: ContentHash,
}

impl MotorTrainingEnvironmentManifestV3 {
    pub fn validate(&self) -> Result<(), MotorContractError> {
        if self.schema_version != MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V3_SCHEMA_VERSION
            || self.task_input_profile_hash == ContentHash::default()
            || self.input_provenance_root == ContentHash::default()
        {
            return Err(MotorContractError::InvalidManifest);
        }
        self.base.validate()
    }

    pub fn validate_for_protocol_v2(&self) -> Result<(), MotorContractError> {
        self.validate()
    }

    pub fn manifest_hash(&self) -> Result<ContentHash, MotorContractError> {
        self.validate()?;
        let mut bytes = header(
            "nextengine.motor-training-environment.v3",
            self.schema_version,
        )?;
        bytes.extend_from_slice(self.base.manifest_hash()?.as_bytes());
        bytes.extend_from_slice(self.task_input_profile_hash.as_bytes());
        bytes.extend_from_slice(self.input_provenance_root.as_bytes());
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}
