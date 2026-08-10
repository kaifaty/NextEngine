use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{
    ContentHash, PersistentId, SchemaId, StateRoot, content_hash_from_bytes,
};
use next_contracts::motor::{
    MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
    MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION, MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION,
    MOTOR_RESET_RECORD_V2_SCHEMA_VERSION, MOTOR_STEP_RECORD_V2_SCHEMA_VERSION,
    MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION, MotorContractError,
    MotorEnvironmentCheckpointEnvelopeV1, MotorEpisodeSeedSetV1, MotorLocomotionCommandModeV1,
    MotorLocomotionCommandProfileV1, MotorResetRecordV2, MotorRewardComponentV1, MotorStepRecordV2,
    MotorTerminalDispositionV1, MotorTrainingEnvironmentManifestV2, STAGE0_MOTOR_HZ,
    STAGE0_PHYSICS_HZ, STAGE0_SUBSTEPS,
};

use crate::runtime::physics_witness_hash;
use crate::{
    CompiledBodySchemaV1, DeterministicHumanoidMotor, HumanoidMotorCheckpoint, MotorFrameResult,
    MotorReplayCodecError, MotorRuntimeError, reference_humanoid_body_schema_v1,
};

pub const MAX_CPU_VECTOR_SLOTS: u32 = 256;
pub const DEFAULT_MAX_EPISODE_MOTOR_STEPS: u64 = 60 * 60;
pub const FLAT_LOCOMOTION_MAX_EPISODE_MOTOR_STEPS: u64 = 1_200;
pub const STANDING_ENVIRONMENT_PROFILE_ID: &str = "nextengine.motor.env.humanoid-standing.v1";
pub const FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID: &str =
    "nextengine.motor.env.humanoid-flat-command.v1";

const RANDOMIZATION_PURPOSES: [&str; 4] = [
    "randomization.action-noise",
    "randomization.friction",
    "randomization.initial-pose",
    "randomization.terrain",
];
const LOCOMOTION_RANDOMIZATION_PURPOSES: [&str; 5] = [
    "randomization.action-noise",
    "randomization.command",
    "randomization.friction",
    "randomization.initial-pose",
    "randomization.terrain",
];

pub const STANDING_REWARD_COMPONENT_IDS: [&str; 8] = [
    "reward.upright",
    "reward.root-height-tracking",
    "reward.standing-pose-tracking",
    "reward.velocity-penalty",
    "reward.effort-penalty",
    "reward.action-rate-penalty",
    "reward.foot-slip-penalty",
    "reward.fall-terminal",
];

pub const LOCOMOTION_REWARD_COMPONENT_IDS: [&str; 10] = [
    "reward.planar-command-tracking",
    "reward.yaw-rate-tracking",
    "reward.upright-yaw-invariant",
    "reward.root-height-tracking",
    "reward.vertical-velocity-cost",
    "reward.roll-pitch-rate-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-action-rate-cost",
    "reward.contacting-foot-tangential-slip-cost",
    "reward.fall-component",
];

pub const LOCOMOTION_REWARD_COEFFICIENTS_Q16: [i64; 10] = [
    98_304, 32_768, 32_768, 16_384, -3_277, -3_277, -1_311, -3_277, -6_554, -131_072,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorEnvironmentProfile {
    StandingV1,
    HumanoidFlatCommandV1,
}

impl MotorEnvironmentProfile {
    pub fn parse_exact(profile_id: &str) -> Result<Self, TrainingEnvironmentError> {
        match profile_id {
            STANDING_ENVIRONMENT_PROFILE_ID => Ok(Self::StandingV1),
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID => Ok(Self::HumanoidFlatCommandV1),
            _ => Err(TrainingEnvironmentError::UnsupportedProfile),
        }
    }

    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        match self {
            Self::StandingV1 => STANDING_ENVIRONMENT_PROFILE_ID,
            Self::HumanoidFlatCommandV1 => FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
        }
    }

    const fn maximum_episode_steps(self) -> u64 {
        match self {
            Self::StandingV1 => DEFAULT_MAX_EPISODE_MOTOR_STEPS,
            Self::HumanoidFlatCommandV1 => FLAT_LOCOMOTION_MAX_EPISODE_MOTOR_STEPS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorStepInput {
    pub vector_slot: u32,
    pub action_microradians: Vec<i64>,
    pub command_raw: [i64; 3],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorPolicyStepInput {
    pub vector_slot: u32,
    pub episode_ordinal: u64,
    pub action_microradians: Vec<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorResetOutput {
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub seed_set: MotorEpisodeSeedSetV1,
    pub observation_raw: Vec<i64>,
    pub reset_record: MotorResetRecordV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorStepOutput {
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub command_raw: [i64; 3],
    pub next_command_raw: [i64; 3],
    pub frame: MotorFrameResult,
    pub reward_components_raw: Vec<(SchemaId, i64)>,
    pub reward_total_q16: i64,
    pub terminated: bool,
    pub truncated: bool,
    pub terminal_reason_id: Option<SchemaId>,
    pub step_record: MotorStepRecordV2,
}

#[derive(Debug)]
struct VectorSlot {
    episode_ordinal: u64,
    runtime: DeterministicHumanoidMotor,
    previous_action_microradians: Vec<i64>,
    command_schedule: Vec<[i64; 3]>,
    foot_tokens: Vec<u64>,
    maximum_effort_per_frame: u128,
    active: bool,
    terminal_disposition: MotorTerminalDispositionV1,
    terminal_reason_id: Option<SchemaId>,
    current_observation_root: ContentHash,
    last_step_root: StateRoot,
}

#[derive(Debug)]
pub struct MotorVectorRunner {
    profile: MotorEnvironmentProfile,
    run_root: ContentHash,
    manifest: MotorTrainingEnvironmentManifestV2,
    manifest_hash: ContentHash,
    slots: Vec<VectorSlot>,
    staged_seen: Vec<bool>,
    staged_episode_ordinals: Vec<u64>,
    staged_actions: Vec<Vec<i64>>,
    staged_commands: Vec<[i64; 3]>,
}

impl MotorVectorRunner {
    pub fn create(
        slot_count: u32,
        run_root: ContentHash,
    ) -> Result<Self, TrainingEnvironmentError> {
        Self::create_typed(MotorEnvironmentProfile::StandingV1, slot_count, run_root)
    }

    pub fn create_profile(
        profile_id: &str,
        slot_count: u32,
        run_root: ContentHash,
    ) -> Result<Self, TrainingEnvironmentError> {
        Self::create_typed(
            MotorEnvironmentProfile::parse_exact(profile_id)?,
            slot_count,
            run_root,
        )
    }

    fn create_typed(
        profile: MotorEnvironmentProfile,
        slot_count: u32,
        run_root: ContentHash,
    ) -> Result<Self, TrainingEnvironmentError> {
        if slot_count == 0 || slot_count > MAX_CPU_VECTOR_SLOTS {
            return Err(TrainingEnvironmentError::SlotCount);
        }
        let mut slots = Vec::with_capacity(slot_count as usize);
        let mut manifest = None;
        for vector_slot in 0..slot_count {
            let compiled = compile_for_slot(profile, run_root, vector_slot)?;
            if manifest.is_none() {
                manifest = Some(environment_manifest(profile, &compiled)?);
            }
            let foot_tokens = compiled.effector_tokens.values().copied().collect();
            let maximum_effort_per_frame = compiled
                .actuator_definitions
                .iter()
                .map(|actuator| u128::from(actuator.maximum_effort_micronewton_metres))
                .sum::<u128>()
                .checked_mul(STAGE0_SUBSTEPS as u128)
                .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
            slots.push(VectorSlot {
                episode_ordinal: 0,
                runtime: DeterministicHumanoidMotor::create(compiled)?,
                previous_action_microradians: vec![0; crate::REFERENCE_HUMANOID_DOF],
                command_schedule: Vec::new(),
                foot_tokens,
                maximum_effort_per_frame,
                active: false,
                terminal_disposition: MotorTerminalDispositionV1::Running,
                terminal_reason_id: None,
                current_observation_root: ContentHash::default(),
                last_step_root: StateRoot::default(),
            });
        }
        let manifest = manifest.ok_or(TrainingEnvironmentError::SlotCount)?;
        let manifest_hash = manifest.manifest_hash()?;
        Ok(Self {
            profile,
            run_root,
            manifest,
            manifest_hash,
            slots,
            staged_seen: vec![false; slot_count as usize],
            staged_episode_ordinals: vec![0; slot_count as usize],
            staged_actions: vec![Vec::new(); slot_count as usize],
            staged_commands: vec![[0; 3]; slot_count as usize],
        })
    }

    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.slots.len() as u32
    }

    #[must_use]
    pub const fn profile(&self) -> MotorEnvironmentProfile {
        self.profile
    }

    #[must_use]
    pub fn manifest(&self) -> &MotorTrainingEnvironmentManifestV2 {
        &self.manifest
    }

    #[must_use]
    pub const fn manifest_hash(&self) -> ContentHash {
        self.manifest_hash
    }

    pub fn reset_all(
        &mut self,
        episode_ordinal: u64,
    ) -> Result<Vec<VectorResetOutput>, TrainingEnvironmentError> {
        let reset_plan = (0..self.slot_count())
            .map(|vector_slot| (vector_slot, episode_ordinal))
            .collect::<Vec<_>>();
        self.apply_reset_plan(&reset_plan)
    }

    pub fn reset_slots(
        &mut self,
        vector_slots: &[u32],
    ) -> Result<Vec<VectorResetOutput>, TrainingEnvironmentError> {
        if vector_slots.is_empty() {
            return Err(TrainingEnvironmentError::SlotIdentity);
        }
        self.clear_staging();
        let mut reset_plan = Vec::with_capacity(vector_slots.len());
        for vector_slot in vector_slots {
            let index = usize::try_from(*vector_slot)
                .ok()
                .filter(|index| *index < self.slots.len())
                .ok_or(TrainingEnvironmentError::SlotIdentity)?;
            if self.staged_seen[index] {
                return Err(TrainingEnvironmentError::SlotIdentity);
            }
            self.staged_seen[index] = true;
            let next_episode = self.slots[index]
                .episode_ordinal
                .checked_add(1)
                .ok_or(TrainingEnvironmentError::EpisodeOverflow)?;
            reset_plan.push((*vector_slot, next_episode));
        }
        reset_plan.sort_unstable_by_key(|(vector_slot, _)| *vector_slot);
        self.apply_reset_plan(&reset_plan)
    }

    fn apply_reset_plan(
        &mut self,
        reset_plan: &[(u32, u64)],
    ) -> Result<Vec<VectorResetOutput>, TrainingEnvironmentError> {
        let mut staged = Vec::with_capacity(reset_plan.len());
        for (vector_slot, episode_ordinal) in reset_plan {
            staged.push(self.fresh_slot(*vector_slot, *episode_ordinal)?);
        }
        let mut output = Vec::with_capacity(staged.len());
        for (vector_slot, slot, reset_output) in staged {
            self.slots[vector_slot as usize] = slot;
            output.push(reset_output);
        }
        output.sort_by_key(|value| (value.episode_ordinal, value.vector_slot));
        Ok(output)
    }

    fn fresh_slot(
        &self,
        vector_slot: u32,
        episode_ordinal: u64,
    ) -> Result<(u32, VectorSlot, VectorResetOutput), TrainingEnvironmentError> {
        let compiled = compile_for_slot(self.profile, self.run_root, vector_slot)?;
        let foot_tokens = compiled
            .effector_tokens
            .values()
            .copied()
            .collect::<Vec<_>>();
        let maximum_effort_per_frame = compiled
            .actuator_definitions
            .iter()
            .map(|actuator| u128::from(actuator.maximum_effort_micronewton_metres))
            .sum::<u128>()
            .checked_mul(STAGE0_SUBSTEPS as u128)
            .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
        let mut runtime = DeterministicHumanoidMotor::create(compiled)?;
        let observation_raw = runtime.reset()?;
        let seed_set = derive_episode_seed_set_for_profile(
            self.profile,
            self.run_root,
            episode_ordinal,
            vector_slot,
        )?;
        let command_schedule = command_schedule_for_profile(self.profile, &seed_set)?;
        let current_observation_root =
            hash_i64_values("nextengine.motor-observation-root.v2", &observation_raw);
        let physics_root = physics_witness_hash(runtime.current_snapshot());
        let motor_root = runtime.checkpoint().checkpoint_hash()?;
        let mut reset_record = MotorResetRecordV2 {
            schema_version: MOTOR_RESET_RECORD_V2_SCHEMA_VERSION,
            environment_manifest_hash: self.manifest_hash,
            environment_profile_id: schema_id(self.profile.profile_id()),
            run_root: self.run_root,
            episode_ordinal,
            vector_slot,
            seed_set_hash: seed_set.seed_set_hash()?,
            initial_observation_root: current_observation_root,
            physics_root,
            motor_root,
            reset_root: StateRoot::default(),
        };
        reset_record.reset_root = reset_record.computed_reset_root()?;
        reset_record.validate()?;
        let slot = VectorSlot {
            episode_ordinal,
            runtime,
            previous_action_microradians: vec![0; crate::REFERENCE_HUMANOID_DOF],
            command_schedule,
            foot_tokens,
            maximum_effort_per_frame,
            active: true,
            terminal_disposition: MotorTerminalDispositionV1::Running,
            terminal_reason_id: None,
            current_observation_root,
            last_step_root: reset_record.reset_root,
        };
        Ok((
            vector_slot,
            slot,
            VectorResetOutput {
                episode_ordinal,
                vector_slot,
                seed_set,
                observation_raw,
                reset_record,
            },
        ))
    }

    pub fn step_lockstep(
        &mut self,
        inputs: Vec<VectorStepInput>,
    ) -> Result<Vec<VectorStepOutput>, TrainingEnvironmentError> {
        if self.profile != MotorEnvironmentProfile::StandingV1 {
            return Err(TrainingEnvironmentError::ExternalCommandForbidden);
        }
        if inputs.len() != self.slots.len() {
            return Err(TrainingEnvironmentError::IncompleteBatch);
        }
        self.clear_staging();
        for input in inputs {
            let index = self.validate_stage_slot(input.vector_slot)?;
            validate_action_length(&input.action_microradians)?;
            self.staged_seen[index] = true;
            self.staged_episode_ordinals[index] = self.slots[index].episode_ordinal;
            self.staged_actions[index].clone_from(&input.action_microradians);
            self.staged_commands[index] = input.command_raw;
        }
        self.ensure_complete_staging()?;
        self.execute_staged(true)
    }

    pub fn step_actions_lockstep(
        &mut self,
        inputs: Vec<VectorPolicyStepInput>,
    ) -> Result<Vec<VectorStepOutput>, TrainingEnvironmentError> {
        if inputs.len() != self.slots.len() {
            return Err(TrainingEnvironmentError::IncompleteBatch);
        }
        self.clear_staging();
        for input in inputs {
            let index = self.validate_stage_slot(input.vector_slot)?;
            if input.episode_ordinal != self.slots[index].episode_ordinal {
                return Err(TrainingEnvironmentError::StaleEpisode);
            }
            validate_action_length(&input.action_microradians)?;
            self.staged_seen[index] = true;
            self.staged_episode_ordinals[index] = input.episode_ordinal;
            self.staged_actions[index].clone_from(&input.action_microradians);
        }
        self.ensure_complete_staging()?;
        self.execute_staged(false)
    }

    fn clear_staging(&mut self) {
        self.staged_seen.fill(false);
    }

    fn validate_stage_slot(&self, vector_slot: u32) -> Result<usize, TrainingEnvironmentError> {
        let index = usize::try_from(vector_slot)
            .ok()
            .filter(|index| *index < self.slots.len())
            .ok_or(TrainingEnvironmentError::SlotIdentity)?;
        if self.staged_seen[index] {
            return Err(TrainingEnvironmentError::SlotIdentity);
        }
        let slot = &self.slots[index];
        if !slot.active {
            return Err(TrainingEnvironmentError::SlotNotReset);
        }
        if slot.terminal_disposition != MotorTerminalDispositionV1::Running {
            return Err(TrainingEnvironmentError::TerminalSlot);
        }
        Ok(index)
    }

    fn ensure_complete_staging(&self) -> Result<(), TrainingEnvironmentError> {
        if self.staged_seen.iter().all(|seen| *seen) {
            Ok(())
        } else {
            Err(TrainingEnvironmentError::IncompleteBatch)
        }
    }

    fn execute_staged(
        &mut self,
        external_standing_commands: bool,
    ) -> Result<Vec<VectorStepOutput>, TrainingEnvironmentError> {
        let mut output = Vec::with_capacity(self.slots.len());
        for slot_index in 0..self.slots.len() {
            let vector_slot = slot_index as u32;
            let slot = &mut self.slots[slot_index];
            let tick = usize::try_from(slot.runtime.checkpoint().motor_tick)
                .map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
            let (command_raw, next_command_raw) = if external_standing_commands {
                let command = self.staged_commands[slot_index];
                (command, command)
            } else {
                let current = *slot
                    .command_schedule
                    .get(tick)
                    .ok_or(TrainingEnvironmentError::ScheduleBounds)?;
                let next = *slot
                    .command_schedule
                    .get(tick + 1)
                    .ok_or(TrainingEnvironmentError::ScheduleBounds)?;
                (current, next)
            };
            let prior_observation_root = slot.current_observation_root;
            let prior_step_root = slot.last_step_root;
            let previous_action = slot.previous_action_microradians.clone();
            let frame = slot
                .runtime
                .step_motor_frame(&self.staged_actions[slot_index], next_command_raw)?;
            let terminal = terminal_facts(self.profile, &frame);
            let (reward_components_raw, reward_total_q16) = match self.profile {
                MotorEnvironmentProfile::StandingV1 => {
                    let components = standing_reward_components(
                        &frame,
                        &self.staged_actions[slot_index],
                        &previous_action,
                    );
                    let bounded = components
                        .iter()
                        .map(|(_, value)| (*value).clamp(0, 65_536))
                        .collect::<Vec<_>>();
                    let total = bounded.iter().copied().fold(0_i64, i64::saturating_add);
                    (components, total)
                }
                MotorEnvironmentProfile::HumanoidFlatCommandV1 => locomotion_reward_components(
                    &frame,
                    command_raw,
                    &previous_action,
                    &slot.foot_tokens,
                    slot.maximum_effort_per_frame,
                    terminal.terminated,
                )?,
            };
            slot.previous_action_microradians = match self.profile {
                MotorEnvironmentProfile::StandingV1 => self.staged_actions[slot_index].clone(),
                MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
                    frame.applied_action_microradians.clone()
                }
            };
            let next_observation_root = hash_i64_values(
                "nextengine.motor-observation-root.v2",
                &frame.observation_raw,
            );
            let physics_root = physics_witness_hash(&frame.snapshot);
            let motor_root = motor_frame_root(&frame, next_command_raw);
            let record_components = match self.profile {
                MotorEnvironmentProfile::StandingV1 => reward_components_raw
                    .iter()
                    .map(|(_, value)| (*value).clamp(0, 65_536))
                    .collect(),
                MotorEnvironmentProfile::HumanoidFlatCommandV1 => reward_components_raw
                    .iter()
                    .map(|(_, value)| *value)
                    .collect(),
            };
            let mut step_record = MotorStepRecordV2 {
                schema_version: MOTOR_STEP_RECORD_V2_SCHEMA_VERSION,
                episode_ordinal: slot.episode_ordinal,
                vector_slot,
                motor_tick: frame.motor_tick,
                prior_observation_root,
                next_observation_root,
                applied_command_raw: command_raw,
                next_command_raw,
                applied_action_raw: frame.applied_action_microradians.clone(),
                reward_components_q16: record_components,
                reward_total_q16,
                terminated: terminal.terminated,
                truncated: terminal.truncated,
                terminal_reason_id: terminal.reason_id.clone(),
                physics_root,
                motor_root,
                prior_step_root,
                step_root: StateRoot::default(),
            };
            step_record.step_root = step_record.computed_step_root()?;
            step_record.validate()?;
            slot.current_observation_root = next_observation_root;
            slot.last_step_root = step_record.step_root;
            slot.terminal_disposition = terminal.disposition();
            slot.terminal_reason_id = terminal.reason_id.clone();
            output.push(VectorStepOutput {
                episode_ordinal: slot.episode_ordinal,
                vector_slot,
                command_raw,
                next_command_raw,
                frame,
                reward_components_raw,
                reward_total_q16,
                terminated: terminal.terminated,
                truncated: terminal.truncated,
                terminal_reason_id: terminal.reason_id,
                step_record,
            });
        }
        output.sort_by_key(|value| (value.episode_ordinal, value.vector_slot));
        Ok(output)
    }

    pub fn checkpoint_slot(
        &self,
        vector_slot: u32,
        episode_ordinal: u64,
    ) -> Result<MotorEnvironmentCheckpointEnvelopeV1, TrainingEnvironmentError> {
        let slot = self
            .slots
            .get(vector_slot as usize)
            .ok_or(TrainingEnvironmentError::SlotIdentity)?;
        if !slot.active {
            return Err(TrainingEnvironmentError::SlotNotReset);
        }
        if slot.episode_ordinal != episode_ordinal {
            return Err(TrainingEnvironmentError::StaleEpisode);
        }
        let checkpoint = slot.runtime.checkpoint();
        let motor_runtime_checkpoint_bytes = checkpoint.canonical_bytes()?;
        let motor_runtime_checkpoint_hash =
            content_hash_from_bytes(sha256(&motor_runtime_checkpoint_bytes));
        let value = MotorEnvironmentCheckpointEnvelopeV1 {
            schema_version: MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
            environment_profile_id: schema_id(self.profile.profile_id()),
            environment_manifest_hash: self.manifest_hash,
            run_root: self.run_root,
            episode_ordinal,
            vector_slot,
            motor_tick: checkpoint.motor_tick,
            terminal_disposition: slot.terminal_disposition,
            terminal_reason_id: slot.terminal_reason_id.clone(),
            current_observation_root: slot.current_observation_root,
            last_step_root: slot.last_step_root,
            motor_runtime_checkpoint_bytes,
            motor_runtime_checkpoint_hash,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn restore_slot(
        &mut self,
        envelope: &MotorEnvironmentCheckpointEnvelopeV1,
    ) -> Result<Vec<i64>, TrainingEnvironmentError> {
        envelope.validate()?;
        let vector_slot = envelope.vector_slot;
        if envelope.environment_profile_id.as_str() != self.profile.profile_id()
            || envelope.environment_manifest_hash != self.manifest_hash
            || envelope.run_root != self.run_root
            || vector_slot >= self.slot_count()
            || envelope.motor_tick > self.profile.maximum_episode_steps()
        {
            return Err(TrainingEnvironmentError::CheckpointIdentity);
        }
        let checkpoint = HumanoidMotorCheckpoint::from_canonical_bytes(
            &envelope.motor_runtime_checkpoint_bytes,
        )?;
        if checkpoint.motor_tick != envelope.motor_tick {
            return Err(TrainingEnvironmentError::CheckpointIdentity);
        }
        let seed_set = derive_episode_seed_set_for_profile(
            self.profile,
            self.run_root,
            envelope.episode_ordinal,
            vector_slot,
        )?;
        let command_schedule = command_schedule_for_profile(self.profile, &seed_set)?;
        if self.profile == MotorEnvironmentProfile::HumanoidFlatCommandV1
            && checkpoint.command_raw
                != *command_schedule
                    .get(checkpoint.motor_tick as usize)
                    .ok_or(TrainingEnvironmentError::ScheduleBounds)?
        {
            return Err(TrainingEnvironmentError::CheckpointIdentity);
        }
        let compiled = compile_for_slot(self.profile, self.run_root, vector_slot)?;
        let foot_tokens = compiled
            .effector_tokens
            .values()
            .copied()
            .collect::<Vec<_>>();
        let maximum_effort_per_frame = compiled
            .actuator_definitions
            .iter()
            .map(|actuator| u128::from(actuator.maximum_effort_micronewton_metres))
            .sum::<u128>()
            .checked_mul(STAGE0_SUBSTEPS as u128)
            .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
        let mut runtime = DeterministicHumanoidMotor::create(compiled)?;
        let observation = runtime.restore_fresh(&checkpoint)?;
        if hash_i64_values("nextengine.motor-observation-root.v2", &observation)
            != envelope.current_observation_root
        {
            return Err(TrainingEnvironmentError::CheckpointIdentity);
        }
        let facts = terminal_facts_from_snapshot(
            self.profile,
            checkpoint.motor_tick,
            runtime.current_snapshot(),
        );
        if facts.disposition() != envelope.terminal_disposition
            || facts.reason_id != envelope.terminal_reason_id
        {
            return Err(TrainingEnvironmentError::CheckpointIdentity);
        }
        let replacement = VectorSlot {
            episode_ordinal: envelope.episode_ordinal,
            runtime,
            previous_action_microradians: checkpoint.applied_action_microradians,
            command_schedule,
            foot_tokens,
            maximum_effort_per_frame,
            active: true,
            terminal_disposition: envelope.terminal_disposition,
            terminal_reason_id: envelope.terminal_reason_id.clone(),
            current_observation_root: envelope.current_observation_root,
            last_step_root: envelope.last_step_root,
        };
        self.slots[vector_slot as usize] = replacement;
        Ok(observation)
    }
}

pub fn flat_locomotion_command_profile_v1() -> MotorLocomotionCommandProfileV1 {
    MotorLocomotionCommandProfileV1 {
        schema_version: MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION,
        profile_id: schema_id("nextengine.motor.command.humanoid-flat.v1"),
        randomization_stream_id: schema_id("randomization.command"),
        warmup_ticks: 60,
        segment_ticks: 120,
        episode_ticks: 1_200,
        mode_weights_basis_points: [2_500, 3_500, 2_000, 2_000],
        right_velocity_min_micrometres_per_second: -2_000_000,
        right_velocity_max_micrometres_per_second: 2_000_000,
        forward_velocity_min_micrometres_per_second: -1_500_000,
        forward_velocity_max_micrometres_per_second: 3_000_000,
        yaw_rate_min_microradians_per_second: -1_500_000,
        yaw_rate_max_microradians_per_second: 1_500_000,
        linear_rate_limit_micrometres_per_second_squared: 3_000_000,
        yaw_rate_limit_microradians_per_second_squared: 1_500_000,
    }
}

pub fn flat_locomotion_command_schedule(
    command_seed: [u8; 32],
) -> Result<Vec<[i64; 3]>, TrainingEnvironmentError> {
    let profile = flat_locomotion_command_profile_v1();
    profile.validate()?;
    let mut schedule = Vec::with_capacity(profile.episode_ticks as usize + 1);
    schedule.push([0; 3]);
    let linear_delta = i64::try_from(
        profile.linear_rate_limit_micrometres_per_second_squared / u64::from(STAGE0_MOTOR_HZ),
    )
    .map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
    let yaw_delta = i64::try_from(
        profile.yaw_rate_limit_microradians_per_second_squared / u64::from(STAGE0_MOTOR_HZ),
    )
    .map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
    let mut target = [0; 3];
    for tick in 1..=profile.episode_ticks {
        if tick < profile.warmup_ticks {
            target = [0; 3];
        } else if tick == profile.warmup_ticks
            || (tick - profile.warmup_ticks).is_multiple_of(profile.segment_ticks)
        {
            let segment_index = u64::from((tick - profile.warmup_ticks) / profile.segment_ticks);
            target = command_target(&profile, command_seed, segment_index)?;
        }
        let previous = *schedule
            .last()
            .ok_or(TrainingEnvironmentError::ScheduleBounds)?;
        schedule.push([
            move_towards(previous[0], target[0], linear_delta),
            move_towards(previous[1], target[1], linear_delta),
            move_towards(previous[2], target[2], yaw_delta),
        ]);
    }
    Ok(schedule)
}

pub fn derive_episode_seed_set(
    run_root: ContentHash,
    episode_ordinal: u64,
    vector_slot: u32,
) -> Result<MotorEpisodeSeedSetV1, TrainingEnvironmentError> {
    derive_seed_set_from_purposes(
        run_root,
        episode_ordinal,
        vector_slot,
        &RANDOMIZATION_PURPOSES,
    )
}

pub fn derive_locomotion_episode_seed_set(
    run_root: ContentHash,
    episode_ordinal: u64,
    vector_slot: u32,
) -> Result<MotorEpisodeSeedSetV1, TrainingEnvironmentError> {
    derive_episode_seed_set_for_profile(
        MotorEnvironmentProfile::HumanoidFlatCommandV1,
        run_root,
        episode_ordinal,
        vector_slot,
    )
}

pub fn canonical_environment_manifest_v2(
    profile_id: &str,
) -> Result<MotorTrainingEnvironmentManifestV2, TrainingEnvironmentError> {
    let profile = MotorEnvironmentProfile::parse_exact(profile_id)?;
    let compiled = compile_for_slot(profile, ContentHash::default(), 0)?;
    environment_manifest(profile, &compiled)
}

fn derive_episode_seed_set_for_profile(
    profile: MotorEnvironmentProfile,
    run_root: ContentHash,
    episode_ordinal: u64,
    vector_slot: u32,
) -> Result<MotorEpisodeSeedSetV1, TrainingEnvironmentError> {
    match profile {
        MotorEnvironmentProfile::StandingV1 => {
            derive_episode_seed_set(run_root, episode_ordinal, vector_slot)
        }
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => derive_seed_set_from_purposes(
            run_root,
            episode_ordinal,
            vector_slot,
            &LOCOMOTION_RANDOMIZATION_PURPOSES,
        ),
    }
}

fn derive_seed_set_from_purposes(
    run_root: ContentHash,
    episode_ordinal: u64,
    vector_slot: u32,
    purposes: &[&str],
) -> Result<MotorEpisodeSeedSetV1, TrainingEnvironmentError> {
    let mut purpose_seeds = purposes
        .iter()
        .map(|purpose| {
            let purpose_id = schema_id(purpose);
            let purpose_bytes = purpose.as_bytes();
            let mut preimage = Vec::new();
            preimage.extend_from_slice(b"nextengine.motor-episode-seed.v1\0");
            preimage.extend_from_slice(run_root.as_bytes());
            preimage.extend_from_slice(&episode_ordinal.to_le_bytes());
            preimage.extend_from_slice(&vector_slot.to_le_bytes());
            preimage.extend_from_slice(&(purpose_bytes.len() as u32).to_le_bytes());
            preimage.extend_from_slice(purpose_bytes);
            (purpose_id, sha256(&preimage))
        })
        .collect::<Vec<_>>();
    purpose_seeds.sort_by(|left, right| left.0.cmp(&right.0));
    if purpose_seeds
        .iter()
        .map(|(_, seed)| seed)
        .collect::<BTreeSet<_>>()
        .len()
        != purpose_seeds.len()
    {
        return Err(TrainingEnvironmentError::SeedCollision);
    }
    let value = MotorEpisodeSeedSetV1 {
        schema_version: MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION,
        run_root,
        episode_ordinal,
        vector_slot,
        purpose_seeds,
    };
    value
        .validate()
        .map_err(|_| TrainingEnvironmentError::SeedProfile)?;
    Ok(value)
}

fn command_schedule_for_profile(
    profile: MotorEnvironmentProfile,
    seed_set: &MotorEpisodeSeedSetV1,
) -> Result<Vec<[i64; 3]>, TrainingEnvironmentError> {
    match profile {
        MotorEnvironmentProfile::StandingV1 => {
            Ok(vec![[0; 3]; DEFAULT_MAX_EPISODE_MOTOR_STEPS as usize + 1])
        }
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
            let command_id = schema_id("randomization.command");
            let command_seed = seed_set
                .purpose_seeds
                .iter()
                .find_map(|(purpose, seed)| (purpose == &command_id).then_some(*seed))
                .ok_or(TrainingEnvironmentError::SeedProfile)?;
            flat_locomotion_command_schedule(command_seed)
        }
    }
}

fn command_target(
    profile: &MotorLocomotionCommandProfileV1,
    command_seed: [u8; 32],
    segment_index: u64,
) -> Result<[i64; 3], TrainingEnvironmentError> {
    let selector = counter_u64(command_seed, segment_index, 0) % 10_000;
    let thresholds = [
        u64::from(profile.mode_weights_basis_points[0]),
        u64::from(profile.mode_weights_basis_points[0] + profile.mode_weights_basis_points[1]),
        u64::from(
            profile.mode_weights_basis_points[0]
                + profile.mode_weights_basis_points[1]
                + profile.mode_weights_basis_points[2],
        ),
    ];
    let mode = if selector < thresholds[0] {
        MotorLocomotionCommandModeV1::Stop
    } else if selector < thresholds[1] {
        MotorLocomotionCommandModeV1::Translation
    } else if selector < thresholds[2] {
        MotorLocomotionCommandModeV1::Turn
    } else {
        MotorLocomotionCommandModeV1::Combined
    };
    let right = bounded_counter_i64(
        command_seed,
        segment_index,
        1,
        profile.right_velocity_min_micrometres_per_second,
        profile.right_velocity_max_micrometres_per_second,
    )?;
    let forward = bounded_counter_i64(
        command_seed,
        segment_index,
        2,
        profile.forward_velocity_min_micrometres_per_second,
        profile.forward_velocity_max_micrometres_per_second,
    )?;
    let yaw = bounded_counter_i64(
        command_seed,
        segment_index,
        3,
        profile.yaw_rate_min_microradians_per_second,
        profile.yaw_rate_max_microradians_per_second,
    )?;
    Ok(match mode {
        MotorLocomotionCommandModeV1::Stop => [0; 3],
        MotorLocomotionCommandModeV1::Translation => [right, forward, 0],
        MotorLocomotionCommandModeV1::Turn => [0, 0, yaw],
        MotorLocomotionCommandModeV1::Combined => [right, forward, yaw],
    })
}

fn counter_u64(command_seed: [u8; 32], segment_index: u64, lane: u32) -> u64 {
    let mut preimage = Vec::with_capacity(84);
    preimage.extend_from_slice(b"nextengine.motor-command-counter.v1\0");
    preimage.extend_from_slice(&command_seed);
    preimage.extend_from_slice(&segment_index.to_le_bytes());
    preimage.extend_from_slice(&lane.to_le_bytes());
    let digest = sha256(&preimage);
    u64::from_le_bytes(
        digest[..8]
            .try_into()
            .expect("SHA-256 prefix has eight bytes"),
    )
}

fn bounded_counter_i64(
    seed: [u8; 32],
    segment_index: u64,
    lane: u32,
    minimum: i64,
    maximum: i64,
) -> Result<i64, TrainingEnvironmentError> {
    let span = i128::from(maximum)
        .checked_sub(i128::from(minimum))
        .and_then(|value| value.checked_add(1))
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let span = u64::try_from(span).map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
    let offset = counter_u64(seed, segment_index, lane) % span;
    minimum
        .checked_add(
            i64::try_from(offset).map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?,
        )
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)
}

fn move_towards(current: i64, target: i64, maximum_delta: i64) -> i64 {
    if current < target {
        current.saturating_add(maximum_delta).min(target)
    } else {
        current.saturating_sub(maximum_delta).max(target)
    }
}

fn compile_for_slot(
    profile: MotorEnvironmentProfile,
    run_root: ContentHash,
    vector_slot: u32,
) -> Result<CompiledBodySchemaV1, TrainingEnvironmentError> {
    let schema = reference_humanoid_body_schema_v1();
    let mut compiled = CompiledBodySchemaV1::compile(&schema, subject_id(run_root, vector_slot))
        .map_err(|_| TrainingEnvironmentError::Compile)?;
    if profile == MotorEnvironmentProfile::HumanoidFlatCommandV1 {
        compiled
            .apply_flat_locomotion_profile()
            .map_err(|_| TrainingEnvironmentError::Compile)?;
    }
    Ok(compiled)
}

fn environment_manifest(
    profile: MotorEnvironmentProfile,
    compiled: &CompiledBodySchemaV1,
) -> Result<MotorTrainingEnvironmentManifestV2, TrainingEnvironmentError> {
    let reward_components = match profile {
        MotorEnvironmentProfile::StandingV1 => STANDING_REWARD_COMPONENT_IDS
            .into_iter()
            .map(|component_id| MotorRewardComponentV1 {
                component_id: schema_id(component_id),
                coefficient_q16: 65_536,
                minimum_raw: i64::MIN,
                maximum_raw: i64::MAX,
            })
            .collect(),
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => LOCOMOTION_REWARD_COMPONENT_IDS
            .into_iter()
            .zip(LOCOMOTION_REWARD_COEFFICIENTS_Q16)
            .map(|(component_id, coefficient_q16)| MotorRewardComponentV1 {
                component_id: schema_id(component_id),
                coefficient_q16,
                minimum_raw: 0,
                maximum_raw: 65_536,
            })
            .collect(),
    };
    let command_schedule_profile_hash = match profile {
        MotorEnvironmentProfile::StandingV1 => profile_constant_hash(
            "nextengine.motor.command.external-standing.v1",
            compiled.body_schema_hash,
        ),
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
            flat_locomotion_command_profile_v1().profile_hash()?
        }
    };
    let value = MotorTrainingEnvironmentManifestV2 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
        environment_id: schema_id(profile.profile_id()),
        body_schema_hash: compiled.body_schema_hash,
        body_instance_projection_hash: profile_constant_hash(
            "nextengine.body-instance.neutral-fixed.v1",
            compiled.body_schema_hash,
        ),
        physics_catalog_hash: profile_constant_hash(
            match profile {
                MotorEnvironmentProfile::StandingV1 => {
                    "nextengine.physics.catalog.humanoid-standing-50m.v1"
                }
                MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
                    "nextengine.physics.catalog.humanoid-flat-100m.v1"
                }
            },
            compiled.body_schema_hash,
        ),
        observation_layout_hash: compiled.observation_layout.layout_hash()?,
        action_layout_hash: compiled.action_layout.layout_hash()?,
        physics_build_profile_hash: profile_constant_hash(
            "nextengine.physx.build-profile.locked.v1",
            compiled.body_schema_hash,
        ),
        scene_profile_hash: profile_constant_hash(
            "nextengine.physx.scene.deterministic-humanoid.v1",
            compiled.body_schema_hash,
        ),
        bridge_abi_hash: profile_constant_hash(
            "nextengine.physx.bridge-abi.v1",
            compiled.body_schema_hash,
        ),
        quantization_profile_hash: profile_constant_hash(
            "nextengine.physics.quantization.humanoid.v1",
            compiled.body_schema_hash,
        ),
        translator_version_hash: profile_constant_hash(
            "nextengine.isaac-translator.v2",
            compiled.body_schema_hash,
        ),
        command_schedule_profile_hash,
        reward_profile_hash: reward_profile_hash(profile, compiled.body_schema_hash),
        termination_profile_hash: profile_constant_hash(
            match profile {
                MotorEnvironmentProfile::StandingV1 => "nextengine.motor.termination.standing.v1",
                MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
                    "nextengine.motor.termination.flat-command.v1"
                }
            },
            compiled.body_schema_hash,
        ),
        rng_derivation_profile_hash: profile_constant_hash(
            "nextengine.motor.episode-seed-derivation.v1",
            compiled.body_schema_hash,
        ),
        correspondence_profile_hash: profile_constant_hash(
            "nextengine.motor.correspondence.v2",
            compiled.body_schema_hash,
        ),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: MAX_CPU_VECTOR_SLOTS,
        maximum_episode_steps: profile.maximum_episode_steps(),
        reward_components,
    };
    value.validate()?;
    Ok(value)
}

fn profile_constant_hash(domain: &str, body_schema_hash: ContentHash) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(body_schema_hash.as_bytes());
    content_hash_from_bytes(sha256(&preimage))
}

fn reward_profile_hash(
    profile: MotorEnvironmentProfile,
    body_schema_hash: ContentHash,
) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor.reward-profile.v2\0");
    preimage.extend_from_slice(profile.profile_id().as_bytes());
    preimage.extend_from_slice(body_schema_hash.as_bytes());
    if profile == MotorEnvironmentProfile::HumanoidFlatCommandV1 {
        for (component, coefficient) in LOCOMOTION_REWARD_COMPONENT_IDS
            .into_iter()
            .zip(LOCOMOTION_REWARD_COEFFICIENTS_Q16)
        {
            preimage.extend_from_slice(&(component.len() as u32).to_le_bytes());
            preimage.extend_from_slice(component.as_bytes());
            preimage.extend_from_slice(&coefficient.to_le_bytes());
        }
        for normalization in [
            6_500_000_i64,
            3_000_000,
            600_000,
            3_000_000,
            6_000_000,
            4_000_000,
        ] {
            preimage.extend_from_slice(&normalization.to_le_bytes());
        }
    }
    content_hash_from_bytes(sha256(&preimage))
}

fn subject_id(run_root: ContentHash, vector_slot: u32) -> PersistentId {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor-vector-subject.v1\0");
    preimage.extend_from_slice(run_root.as_bytes());
    preimage.extend_from_slice(&vector_slot.to_le_bytes());
    let digest = sha256(&preimage);
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    PersistentId::from_bytes(bytes)
}

fn locomotion_reward_components(
    frame: &MotorFrameResult,
    command_raw: [i64; 3],
    previous_applied_action: &[i64],
    foot_tokens: &[u64],
    maximum_effort_per_frame: u128,
    fell: bool,
) -> Result<(Vec<(SchemaId, i64)>, i64), TrainingEnvironmentError> {
    let root = frame
        .snapshot
        .links
        .first()
        .ok_or(TrainingEnvironmentError::RewardFacts)?;
    if frame.observation_raw.len() != 84
        || previous_applied_action.len() != frame.applied_action_microradians.len()
    {
        return Err(TrainingEnvironmentError::RewardFacts);
    }
    let local_right_velocity = frame.observation_raw[4];
    let local_forward_velocity = frame.observation_raw[6];
    let local_yaw_rate = frame.observation_raw[8];
    let planar_error = abs_sum([
        local_right_velocity.saturating_sub(command_raw[0]),
        local_forward_velocity.saturating_sub(command_raw[1]),
    ]);
    let planar_tracking = one_minus_normalized_q16(planar_error, 6_500_000);
    let yaw_tracking = one_minus_normalized_q16(
        local_yaw_rate.saturating_sub(command_raw[2]).unsigned_abs() as u128,
        3_000_000,
    );
    let upright = upright_reward_q16(root.rotation_q1_30)?;
    let height_error = root.position_micrometres[1]
        .saturating_sub(1_050_000)
        .unsigned_abs() as u128;
    let height_tracking = one_minus_normalized_q16(height_error, 600_000);
    let vertical_velocity_cost = ratio_q16(
        root.linear_velocity_micrometres_per_second[1].unsigned_abs() as u128,
        3_000_000,
    )?;
    let roll_pitch_rate_cost = ratio_q16(
        abs_sum([frame.observation_raw[7], frame.observation_raw[9]]),
        6_000_000,
    )?;
    let effort_sum = frame
        .substep_efforts
        .iter()
        .flatten()
        .map(|effort| u128::from(effort.effort_micronewton_metres.unsigned_abs()))
        .sum::<u128>();
    let effort_cost = ratio_q16(effort_sum, maximum_effort_per_frame)?;
    let action_rate_sum = frame
        .applied_action_microradians
        .iter()
        .zip(previous_applied_action)
        .map(|(current, previous)| current.saturating_sub(*previous).unsigned_abs() as u128)
        .sum::<u128>();
    let action_rate_denominator = (frame.applied_action_microradians.len() as u128)
        .checked_mul(2_000_000)
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let action_rate_cost = ratio_q16(action_rate_sum, action_rate_denominator)?;
    let contacting_foot_tokens =
        foot_tokens
            .iter()
            .copied()
            .filter(|token| {
                frame.snapshot.contacts.iter().any(|contact| {
                    contact.actor_a_token == *token || contact.actor_b_token == *token
                })
            })
            .collect::<BTreeSet<_>>();
    let slip_sum = frame
        .snapshot
        .links
        .iter()
        .filter(|link| contacting_foot_tokens.contains(&link.user_token))
        .map(|link| {
            abs_sum([
                link.linear_velocity_micrometres_per_second[0],
                link.linear_velocity_micrometres_per_second[2],
            ])
        })
        .sum::<u128>();
    let slip_denominator = (contacting_foot_tokens.len() as u128)
        .checked_mul(4_000_000)
        .unwrap_or(0);
    let slip_cost = if slip_denominator == 0 {
        0
    } else {
        ratio_q16(slip_sum, slip_denominator)?
    };
    let values = [
        planar_tracking,
        yaw_tracking,
        upright,
        height_tracking,
        vertical_velocity_cost,
        roll_pitch_rate_cost,
        effort_cost,
        action_rate_cost,
        slip_cost,
        i64::from(fell) * 65_536,
    ];
    let reward_total_q16 = values
        .iter()
        .zip(LOCOMOTION_REWARD_COEFFICIENTS_Q16)
        .try_fold(0_i64, |total, (component, coefficient)| {
            let weighted = round_shift_ties_even_i128(
                i128::from(*component)
                    .checked_mul(i128::from(coefficient))
                    .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?,
                16,
            )?;
            total
                .checked_add(weighted)
                .ok_or(TrainingEnvironmentError::ArithmeticOverflow)
        })?;
    Ok((
        LOCOMOTION_REWARD_COMPONENT_IDS
            .into_iter()
            .zip(values)
            .map(|(component_id, value)| (schema_id(component_id), value))
            .collect(),
        reward_total_q16,
    ))
}

fn upright_reward_q16(rotation_q1_30: [i64; 4]) -> Result<i64, TrainingEnvironmentError> {
    let [x, _, z, _] = rotation_q1_30;
    let tilt_reduction_q30 = i128::from(x)
        .checked_mul(i128::from(x))
        .and_then(|value| {
            i128::from(z)
                .checked_mul(i128::from(z))
                .and_then(|other| value.checked_add(other))
        })
        .and_then(|value| value.checked_mul(2))
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let tilt_reduction_q30 = round_shift_ties_even_i128(tilt_reduction_q30, 30)?;
    let upright_q30 = (1_i64 << 30)
        .saturating_sub(tilt_reduction_q30)
        .clamp(0, 1_i64 << 30);
    ratio_q16(upright_q30 as u128, 1_u128 << 30)
}

fn standing_reward_components(
    frame: &MotorFrameResult,
    action_microradians: &[i64],
    previous_action_microradians: &[i64],
) -> Vec<(SchemaId, i64)> {
    let root = frame.snapshot.links.first();
    let upright = root.map_or(0, |root| root.rotation_q1_30[3].unsigned_abs() as i64);
    let root_height_tracking = root.map_or(-1_050_000, |root| {
        -unsigned_sum([root.position_micrometres[1].saturating_sub(1_050_000)])
    });
    let standing_pose_tracking = -unsigned_sum(
        frame
            .snapshot
            .joints
            .iter()
            .map(|joint| joint.position_microradians),
    );
    let velocity_penalty = root.map_or(-1, |root| {
        -unsigned_sum(
            root.linear_velocity_micrometres_per_second
                .into_iter()
                .chain(root.angular_velocity_microradians_per_second),
        )
    });
    let effort_penalty = -(frame
        .substep_efforts
        .iter()
        .flatten()
        .map(|effort| effort.effort_micronewton_metres.unsigned_abs() / 1_000_000)
        .sum::<u64>()
        .min(i64::MAX as u64) as i64);
    let action_rate_penalty = -unsigned_sum(
        action_microradians
            .iter()
            .zip(previous_action_microradians)
            .map(|(current, previous)| current.saturating_sub(*previous)),
    );
    let contacting_tokens = frame
        .snapshot
        .contacts
        .iter()
        .flat_map(|contact| [contact.actor_a_token, contact.actor_b_token])
        .filter(|token| *token != 1)
        .collect::<BTreeSet<_>>();
    let foot_slip_penalty = -unsigned_sum(
        frame
            .snapshot
            .links
            .iter()
            .filter(|link| contacting_tokens.contains(&link.user_token))
            .flat_map(|link| {
                [
                    link.linear_velocity_micrometres_per_second[0],
                    link.linear_velocity_micrometres_per_second[2],
                ]
            }),
    );
    let fall_terminal = -i64::from(root.is_none_or(|root| root.position_micrometres[1] <= 250_000));
    [
        upright,
        root_height_tracking,
        standing_pose_tracking,
        velocity_penalty,
        effort_penalty,
        action_rate_penalty,
        foot_slip_penalty,
        fall_terminal,
    ]
    .into_iter()
    .zip(STANDING_REWARD_COMPONENT_IDS.map(schema_id))
    .map(|(value, id)| (id, value))
    .collect()
}

#[derive(Clone, Debug)]
struct TerminalFacts {
    terminated: bool,
    truncated: bool,
    reason_id: Option<SchemaId>,
}

impl TerminalFacts {
    fn disposition(&self) -> MotorTerminalDispositionV1 {
        if self.terminated {
            MotorTerminalDispositionV1::Terminated
        } else if self.truncated {
            MotorTerminalDispositionV1::Truncated
        } else {
            MotorTerminalDispositionV1::Running
        }
    }
}

fn terminal_facts(profile: MotorEnvironmentProfile, frame: &MotorFrameResult) -> TerminalFacts {
    terminal_facts_from_snapshot(profile, frame.motor_tick, &frame.snapshot)
}

fn terminal_facts_from_snapshot(
    profile: MotorEnvironmentProfile,
    motor_tick: u64,
    snapshot: &next_physics_physx::CanonicalPhysXSnapshot,
) -> TerminalFacts {
    let root = snapshot.links.first();
    let terminated_reason = match profile {
        MotorEnvironmentProfile::StandingV1 => root
            .is_none_or(|root| root.position_micrometres[1] <= 250_000)
            .then(|| schema_id("terminal.fall")),
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
            if root.is_none_or(|root| root.position_micrometres[1] <= 450_000) {
                Some(schema_id("terminal.fall"))
            } else if root.is_some_and(|root| {
                root.position_micrometres[0].unsigned_abs() >= 90_000_000
                    || root.position_micrometres[2].unsigned_abs() >= 90_000_000
            }) {
                Some(schema_id("terminal.world-bounds"))
            } else {
                None
            }
        }
    };
    if let Some(reason_id) = terminated_reason {
        TerminalFacts {
            terminated: true,
            truncated: false,
            reason_id: Some(reason_id),
        }
    } else if motor_tick >= profile.maximum_episode_steps() {
        TerminalFacts {
            terminated: false,
            truncated: true,
            reason_id: Some(schema_id("terminal.timeout")),
        }
    } else {
        TerminalFacts {
            terminated: false,
            truncated: false,
            reason_id: None,
        }
    }
}

fn validate_action_length(action: &[i64]) -> Result<(), TrainingEnvironmentError> {
    if action.len() == crate::REFERENCE_HUMANOID_DOF {
        Ok(())
    } else {
        Err(TrainingEnvironmentError::ActionLength)
    }
}

fn unsigned_sum(values: impl IntoIterator<Item = i64>) -> i64 {
    values
        .into_iter()
        .map(i64::unsigned_abs)
        .fold(0_u64, u64::saturating_add)
        .min(i64::MAX as u64) as i64
}

fn abs_sum(values: impl IntoIterator<Item = i64>) -> u128 {
    values
        .into_iter()
        .map(|value| value.unsigned_abs() as u128)
        .sum()
}

fn one_minus_normalized_q16(value: u128, maximum: u128) -> i64 {
    65_536_i64.saturating_sub(ratio_q16(value, maximum).unwrap_or(65_536))
}

fn ratio_q16(value: u128, maximum: u128) -> Result<i64, TrainingEnvironmentError> {
    if maximum == 0 {
        return Err(TrainingEnvironmentError::ArithmeticOverflow);
    }
    let bounded = value.min(maximum);
    let numerator = bounded
        .checked_mul(65_536)
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let quotient = numerator / maximum;
    let remainder = numerator % maximum;
    let twice_remainder = remainder
        .checked_mul(2)
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let rounded = quotient
        + u128::from(
            twice_remainder > maximum || (twice_remainder == maximum && quotient % 2 == 1),
        );
    i64::try_from(rounded).map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)
}

fn round_shift_ties_even_i128(value: i128, shift: u32) -> Result<i64, TrainingEnvironmentError> {
    let denominator = 1_i128
        .checked_shl(shift)
        .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?;
    let quotient = value / denominator;
    let remainder = (value % denominator).unsigned_abs();
    let half = (denominator / 2) as u128;
    let adjust = remainder > half || (remainder == half && quotient.unsigned_abs() % 2 == 1);
    let rounded = if adjust {
        quotient
            .checked_add(if value.is_negative() { -1 } else { 1 })
            .ok_or(TrainingEnvironmentError::ArithmeticOverflow)?
    } else {
        quotient
    };
    i64::try_from(rounded).map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)
}

fn hash_i64_values(domain: &str, values: &[i64]) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(&(values.len() as u64).to_le_bytes());
    for value in values {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&preimage))
}

fn motor_frame_root(frame: &MotorFrameResult, command_raw: [i64; 3]) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor-frame-root.v2\0");
    preimage.extend_from_slice(&frame.motor_tick.to_le_bytes());
    for value in command_raw {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    preimage.extend_from_slice(&(frame.applied_action_microradians.len() as u64).to_le_bytes());
    for value in &frame.applied_action_microradians {
        preimage.extend_from_slice(&value.to_le_bytes());
    }
    for effort in frame.substep_efforts.iter().flatten() {
        preimage.extend_from_slice(effort.actuator_id.as_str().as_bytes());
        preimage.push(0);
        preimage.extend_from_slice(&effort.effort_micronewton_metres.to_le_bytes());
        preimage.extend_from_slice(&effort.clamp_flags.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&preimage))
}

fn schema_id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned training identifiers are valid")
}

#[derive(Debug)]
pub enum TrainingEnvironmentError {
    Runtime(MotorRuntimeError),
    Replay(MotorReplayCodecError),
    Contract(MotorContractError),
    Compile,
    UnsupportedProfile,
    SlotCount,
    SlotIdentity,
    SlotNotReset,
    IncompleteBatch,
    StaleEpisode,
    TerminalSlot,
    ExternalCommandForbidden,
    ActionLength,
    EpisodeOverflow,
    SeedProfile,
    SeedCollision,
    ScheduleBounds,
    RewardFacts,
    CheckpointIdentity,
    ArithmeticOverflow,
}

impl TrainingEnvironmentError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Runtime(error) => error.stable_code(),
            Self::Replay(error) => (*error).stable_code(),
            Self::Contract(error) => error.stable_code(),
            Self::Compile => "MOTOR_ENV_COMPILE_FAILED",
            Self::UnsupportedProfile => "UNSUPPORTED_MOTOR_ENVIRONMENT_PROFILE",
            Self::SlotCount => "MOTOR_ENV_SLOT_COUNT_INVALID",
            Self::SlotIdentity => "MOTOR_ENV_SLOT_IDENTITY_INVALID",
            Self::SlotNotReset => "MOTOR_ENV_SLOT_NOT_RESET",
            Self::IncompleteBatch => "MOTOR_ENV_BATCH_INCOMPLETE",
            Self::StaleEpisode => "MOTOR_ENV_EPISODE_STALE",
            Self::TerminalSlot => "MOTOR_ENV_SLOT_TERMINAL",
            Self::ExternalCommandForbidden => "MOTOR_ENV_EXTERNAL_COMMAND_FORBIDDEN",
            Self::ActionLength => "MOTOR_ENV_ACTION_LENGTH_INVALID",
            Self::EpisodeOverflow => "MOTOR_ENV_EPISODE_OVERFLOW",
            Self::SeedProfile => "MOTOR_ENV_SEED_PROFILE_INVALID",
            Self::SeedCollision => "MOTOR_ENV_SEED_COLLISION",
            Self::ScheduleBounds => "MOTOR_ENV_COMMAND_SCHEDULE_BOUNDS",
            Self::RewardFacts => "MOTOR_ENV_REWARD_FACTS_INVALID",
            Self::CheckpointIdentity => "MOTOR_ENV_CHECKPOINT_IDENTITY_MISMATCH",
            Self::ArithmeticOverflow => "MOTOR_ENV_ARITHMETIC_OVERFLOW",
        }
    }
}

impl Display for TrainingEnvironmentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for TrainingEnvironmentError {}

impl From<MotorRuntimeError> for TrainingEnvironmentError {
    fn from(value: MotorRuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<MotorReplayCodecError> for TrainingEnvironmentError {
    fn from(value: MotorReplayCodecError) -> Self {
        Self::Replay(value)
    }
}

impl From<MotorContractError> for TrainingEnvironmentError {
    fn from(value: MotorContractError) -> Self {
        Self::Contract(value)
    }
}

#[cfg(all(test, any(feature = "physx-sdk", feature = "mock-abi")))]
mod tests {
    use super::*;

    fn standing_input(slot: u32, action: i64) -> VectorStepInput {
        VectorStepInput {
            vector_slot: slot,
            action_microradians: vec![action; 23],
            command_raw: [100_000, 0, 0],
        }
    }

    fn locomotion_input(slot: u32, episode: u64, action: i64) -> VectorPolicyStepInput {
        VectorPolicyStepInput {
            vector_slot: slot,
            episode_ordinal: episode,
            action_microradians: vec![action; 23],
        }
    }

    #[test]
    fn standing_publication_order_and_behavior_ignore_input_permutation() {
        let run_root = ContentHash::from_bytes([4; 32]);
        let mut first = MotorVectorRunner::create(4, run_root).expect("first runner");
        let mut second = MotorVectorRunner::create(4, run_root).expect("second runner");
        assert_eq!(
            first.reset_all(17).expect("first reset"),
            second.reset_all(17).expect("second reset")
        );
        let ascending = vec![
            standing_input(0, 1),
            standing_input(1, 2),
            standing_input(2, 3),
            standing_input(3, 4),
        ];
        let descending = vec![
            standing_input(3, 4),
            standing_input(2, 3),
            standing_input(1, 2),
            standing_input(0, 1),
        ];
        let left = first.step_lockstep(ascending).expect("ascending");
        let right = second.step_lockstep(descending).expect("descending");
        assert_eq!(left, right);
    }

    #[test]
    fn command_schedule_has_warmup_bounds_modes_and_rate_limits() {
        let schedule = flat_locomotion_command_schedule([9; 32]).expect("schedule");
        assert_eq!(schedule.len(), 1_201);
        assert!(schedule[..60].iter().all(|command| *command == [0; 3]));
        for pair in schedule.windows(2) {
            assert!(pair[1][0].abs_diff(pair[0][0]) <= 50_000);
            assert!(pair[1][1].abs_diff(pair[0][1]) <= 50_000);
            assert!(pair[1][2].abs_diff(pair[0][2]) <= 25_000);
            assert!((-2_000_000..=2_000_000).contains(&pair[1][0]));
            assert!((-1_500_000..=3_000_000).contains(&pair[1][1]));
            assert!((-1_500_000..=1_500_000).contains(&pair[1][2]));
        }
        assert_eq!(
            schedule,
            flat_locomotion_command_schedule([9; 32]).expect("repeat")
        );
        assert_eq!(
            [schedule[60], schedule[61], schedule[180], schedule[1_200]],
            [
                [50_000, -50_000, 0],
                [100_000, -100_000, 0],
                [621_567, -592_654, 0],
                [-1_191_367, 2_721_543, -1_298_134],
            ]
        );
    }

    #[test]
    fn one_million_aggregate_command_steps_remain_bounded() {
        let mut aggregate_steps = 0_u64;
        for ordinal in 0_u64..834 {
            let mut preimage = Vec::new();
            preimage.extend_from_slice(b"motor-locomotion-million-step-test\0");
            preimage.extend_from_slice(&ordinal.to_le_bytes());
            let schedule = flat_locomotion_command_schedule(sha256(&preimage)).expect("schedule");
            aggregate_steps += schedule.len() as u64;
            assert!(schedule.iter().all(|command| {
                (-2_000_000..=2_000_000).contains(&command[0])
                    && (-1_500_000..=3_000_000).contains(&command[1])
                    && (-1_500_000..=1_500_000).contains(&command[2])
            }));
        }
        assert!(aggregate_steps >= 1_000_000);
    }

    #[test]
    fn reward_tracking_is_monotonic_and_upright_ignores_yaw() {
        assert!(
            one_minus_normalized_q16(100_000, 3_000_000)
                > one_minus_normalized_q16(500_000, 3_000_000)
        );
        let yaw_half_sqrt_q30 = 759_250_125;
        assert_eq!(
            upright_reward_q16([0, 0, 0, 1 << 30]).expect("identity"),
            upright_reward_q16([0, yaw_half_sqrt_q30, 0, yaw_half_sqrt_q30]).expect("yaw")
        );
        assert!(
            upright_reward_q16([yaw_half_sqrt_q30, 0, 0, yaw_half_sqrt_q30]).expect("roll")
                < 65_536
        );
        assert!(!LOCOMOTION_REWARD_COMPONENT_IDS.contains(&"reward.standing-pose-tracking"));
        assert!(
            LOCOMOTION_REWARD_COEFFICIENTS_Q16[4..]
                .iter()
                .all(|value| *value < 0)
        );
    }

    #[test]
    fn action_rate_reward_uses_applied_not_requested_action() {
        let mut runner = MotorVectorRunner::create_profile(
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
            1,
            ContentHash::from_bytes([13; 32]),
        )
        .expect("runner");
        runner.reset_slots(&[0]).expect("reset");
        let output = runner
            .step_actions_lockstep(vec![locomotion_input(0, 1, 2_000_000)])
            .expect("step");
        assert!(
            output[0]
                .frame
                .applied_action_microradians
                .iter()
                .all(|value| *value == 1_000_000)
        );
        assert_eq!(output[0].reward_components_raw[7].1, 32_768);
    }

    #[test]
    fn terminal_slots_reject_steps_until_partial_reset() {
        let mut runner = MotorVectorRunner::create_profile(
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
            1,
            ContentHash::from_bytes([15; 32]),
        )
        .expect("runner");
        runner.reset_slots(&[0]).expect("reset");
        let mut terminal = None;
        for _ in 0..FLAT_LOCOMOTION_MAX_EPISODE_MOTOR_STEPS {
            let output = runner
                .step_actions_lockstep(vec![locomotion_input(0, 1, 0)])
                .expect("step");
            if output[0].terminated || output[0].truncated {
                terminal = Some((output[0].terminated, output[0].truncated));
                break;
            }
        }
        assert!(terminal.is_some());
        let error = runner
            .step_actions_lockstep(vec![locomotion_input(0, 1, 0)])
            .expect_err("post-terminal step");
        assert_eq!(error.stable_code(), "MOTOR_ENV_SLOT_TERMINAL");
        assert_eq!(
            runner.reset_slots(&[0]).expect("reset again")[0].episode_ordinal,
            2
        );
    }

    #[test]
    fn partial_resets_increment_only_selected_episode_ordinals() {
        let mut runner = MotorVectorRunner::create_profile(
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
            3,
            ContentHash::from_bytes([7; 32]),
        )
        .expect("runner");
        let first = runner.reset_slots(&[2, 0, 1]).expect("first reset");
        assert!(first.iter().all(|value| value.episode_ordinal == 1));
        let second = runner.reset_slots(&[1]).expect("partial reset");
        assert_eq!(second[0].episode_ordinal, 2);
        let error = runner
            .step_actions_lockstep(vec![
                locomotion_input(0, 1, 0),
                locomotion_input(1, 1, 0),
                locomotion_input(2, 1, 0),
            ])
            .expect_err("stale episode");
        assert_eq!(error.stable_code(), "MOTOR_ENV_EPISODE_STALE");
    }

    #[test]
    fn locomotion_input_permutations_match_and_invalid_batches_are_atomic() {
        let run_root = ContentHash::from_bytes([10; 32]);
        let mut first =
            MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 3, run_root)
                .expect("first");
        let mut second =
            MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 3, run_root)
                .expect("second");
        assert_eq!(
            first.reset_slots(&[0, 1, 2]).expect("first reset"),
            second.reset_slots(&[2, 1, 0]).expect("second reset")
        );
        let before = first
            .checkpoint_slot(0, 1)
            .expect("before")
            .canonical_bytes()
            .expect("encode");
        assert_eq!(
            first
                .step_actions_lockstep(vec![
                    locomotion_input(0, 1, 1),
                    locomotion_input(0, 1, 2),
                    locomotion_input(2, 1, 3),
                ])
                .expect_err("duplicate")
                .stable_code(),
            "MOTOR_ENV_SLOT_IDENTITY_INVALID"
        );
        assert_eq!(
            first
                .checkpoint_slot(0, 1)
                .expect("after")
                .canonical_bytes()
                .expect("encode"),
            before
        );
        let left = first
            .step_actions_lockstep(vec![
                locomotion_input(2, 1, 3),
                locomotion_input(0, 1, 1),
                locomotion_input(1, 1, 2),
            ])
            .expect("permuted");
        let right = second
            .step_actions_lockstep(vec![
                locomotion_input(0, 1, 1),
                locomotion_input(1, 1, 2),
                locomotion_input(2, 1, 3),
            ])
            .expect("ordered");
        assert_eq!(left, right);
    }

    #[test]
    fn checkpoint_restore_continues_byte_exact_and_is_idempotent() {
        let run_root = ContentHash::from_bytes([11; 32]);
        let mut source =
            MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 1, run_root)
                .expect("source");
        source.reset_slots(&[0]).expect("reset");
        for action in [100, -200, 300] {
            source
                .step_actions_lockstep(vec![locomotion_input(0, 1, action)])
                .expect("prefix");
        }
        let checkpoint = source.checkpoint_slot(0, 1).expect("checkpoint");
        let expected = source
            .step_actions_lockstep(vec![locomotion_input(0, 1, 400)])
            .expect("source continuation");

        let mut restored =
            MotorVectorRunner::create_profile(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, 1, run_root)
                .expect("restored");
        restored.restore_slot(&checkpoint).expect("restore");
        restored
            .restore_slot(&checkpoint)
            .expect("idempotent restore");
        let actual = restored
            .step_actions_lockstep(vec![locomotion_input(0, 1, 400)])
            .expect("restored continuation");
        assert_eq!(actual, expected);
    }

    #[test]
    fn purpose_slot_and_episode_seeds_are_domain_separated() {
        let run_root = ContentHash::from_bytes([6; 32]);
        let first = derive_episode_seed_set(run_root, 1, 0).expect("first");
        let second = derive_episode_seed_set(run_root, 1, 1).expect("slot");
        let third = derive_episode_seed_set(run_root, 2, 0).expect("episode");
        assert_ne!(first.purpose_seeds, second.purpose_seeds);
        assert_ne!(first.purpose_seeds, third.purpose_seeds);
        assert_eq!(
            first
                .purpose_seeds
                .iter()
                .map(|(_, seed)| seed)
                .collect::<BTreeSet<_>>()
                .len(),
            RANDOMIZATION_PURPOSES.len()
        );
    }
}
