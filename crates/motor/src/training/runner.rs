use super::*;

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
        if !self.profile.is_standing() {
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
                MotorEnvironmentProfile::BoundedStandingV2 => bounded_standing_reward_components(
                    &frame,
                    &previous_action,
                    &slot.foot_tokens,
                    slot.maximum_effort_per_frame,
                    terminal.terminated,
                )?,
                MotorEnvironmentProfile::HumanoidFlatCommandV1
                | MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
                    locomotion_reward_components(
                        self.profile,
                        &frame,
                        command_raw,
                        &previous_action,
                        &slot.foot_tokens,
                        slot.maximum_effort_per_frame,
                        terminal.terminated,
                    )?
                }
            };
            slot.previous_action_microradians = match self.profile {
                MotorEnvironmentProfile::StandingV1 => self.staged_actions[slot_index].clone(),
                MotorEnvironmentProfile::BoundedStandingV2
                | MotorEnvironmentProfile::HumanoidFlatCommandV1
                | MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
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
                MotorEnvironmentProfile::BoundedStandingV2
                | MotorEnvironmentProfile::HumanoidFlatCommandV1
                | MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => reward_components_raw
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
        if self.profile.is_locomotion()
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
