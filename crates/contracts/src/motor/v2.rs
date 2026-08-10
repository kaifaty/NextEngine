use super::*;

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
