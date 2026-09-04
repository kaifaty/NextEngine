use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{
    ContentHash, PersistentId, SchemaId, StateRoot, content_hash_from_bytes,
};
use next_contracts::motor::{
    MOTOR_RESET_RECORD_V2_SCHEMA_VERSION, MOTOR_STEP_RECORD_V2_SCHEMA_VERSION, MotorContractError,
    MotorResetRecordV2, MotorStepRecordV2, MotorTerminalDispositionV1,
    MotorTrainingEnvironmentManifestV2,
};
use next_contracts::physics::AppliedActuatorEffortV1;
use next_physics_physx::{CanonicalPhysXSnapshotV2, PhysXAdapterError, PhysXArticulationWorldV3};

use crate::{
    BIOMECHANICS_FALL_HEIGHT_MICROMETRES, BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V2,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V3,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V4,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6,
    BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V7,
    BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS,
    BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5,
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS,
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V2,
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3,
    BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID, BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS,
    BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS, BiomechanicsContactClassV1,
    BiomechanicsContactClassifier, BiomechanicsForwardStartStopRewardFactsV1,
    BiomechanicsProceduralStandingControllerV1, BiomechanicsSafetyController,
    BiomechanicsSkillContactProfileV1, BiomechanicsStandingRewardFactsV1,
    BiomechanicsTerminalEvaluator, CompiledBodySchemaV2, CompiledBodySchemaV3,
    ContactClassificationError, JointControlStateV1, MotorCompileError, MotorObservationError,
    MotorSafetyError, ProceduralStandingError, TrainingEnvironmentError, VectorPolicyStepInput,
    VectorResetOutput, biomechanics_forward_start_stop_command_schedule_v2,
    biomechanics_forward_start_stop_environment_manifest_v1,
    biomechanics_forward_start_stop_environment_manifest_v2,
    biomechanics_forward_start_stop_environment_manifest_v3,
    biomechanics_forward_start_stop_environment_manifest_v4,
    biomechanics_forward_start_stop_environment_manifest_v5,
    biomechanics_forward_start_stop_reward_q16_v1, biomechanics_forward_start_stop_reward_q16_v2,
    biomechanics_forward_start_stop_reward_q16_v3, biomechanics_humanoid_body_schema_v3,
    biomechanics_humanoid_body_schema_v4, biomechanics_standing_environment_manifest_v2,
    biomechanics_standing_reward_q16_v1, curriculum_locomotion_command_schedule,
    derive_curriculum_locomotion_episode_seed_set, derive_episode_seed_set,
    rotate_world_to_root_local_q1_30,
};

const OBSERVATION_WIDTH: usize = 84;
const ACTION_WIDTH: usize = 23;
const PHYSICS_SUBSTEPS: usize = 4;

#[derive(Clone, Copy, Debug)]
struct WalkingSoleGeometry {
    actor: u64,
    offset_um: [i64; 3],
    half_um: [i64; 3],
}

fn sole_heights_um(
    snapshot: &CanonicalPhysXSnapshotV2,
    soles: &[WalkingSoleGeometry; 2],
) -> Result<[i64; 2], BiomechanicsStandingRunnerError> {
    let mut heights = [0; 2];
    for (height, sole) in heights.iter_mut().zip(soles) {
        let link = snapshot
            .links
            .iter()
            .find(|link| link.user_token == sole.actor)
            .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
        *height = crate::walking_box_minimum_y_um(
            link.position_micrometres,
            link.rotation_q1_30,
            sole.offset_um,
            sole.half_um,
        )?;
    }
    Ok(heights)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsStandingFrameResult {
    pub motor_tick: u64,
    pub applied_action_q1_30: Vec<i64>,
    pub applied_targets_microradians: Vec<i64>,
    pub observation_raw: Vec<i64>,
    pub snapshot: CanonicalPhysXSnapshotV2,
    pub contact_frames: Vec<crate::BiomechanicsContactFrameV1>,
    pub contact_flags: [i64; 2],
    /// Diagnostic projection only; does not alter the terminal or replay record.
    pub joint_safety_error: Option<MotorSafetyError>,
    /// Actual simulation steps, excluding repeated contact samples after a failure.
    pub completed_physics_substeps: usize,
    pub safety_checkpoint: Option<crate::BiomechanicsSafetyCheckpointV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomechanicsStandingVectorStepOutput {
    pub episode_ordinal: u64,
    pub vector_slot: u32,
    pub command_raw: [i64; 3],
    pub next_command_raw: [i64; 3],
    pub frame: BiomechanicsStandingFrameResult,
    pub reward_components_raw: Vec<(SchemaId, i64)>,
    pub reward_total_q16: i64,
    pub terminated: bool,
    pub truncated: bool,
    pub terminal_reason_id: Option<SchemaId>,
    pub step_record: MotorStepRecordV2,
}

#[derive(Debug)]
struct BiomechanicsStandingSlot {
    episode_ordinal: u64,
    world: PhysXArticulationWorldV3,
    snapshot: CanonicalPhysXSnapshotV2,
    safety: BiomechanicsSafetyController,
    classifier: BiomechanicsContactClassifier,
    terminal: BiomechanicsTerminalEvaluator,
    standing: BiomechanicsProceduralStandingControllerV1,
    envelopes: Vec<crate::JointTargetEnvelopeV1>,
    command_schedule: Vec<[i64; 3]>,
    motor_tick: u64,
    active: bool,
    terminal_disposition: MotorTerminalDispositionV1,
    current_observation_root: ContentHash,
    last_step_root: StateRoot,
}

#[derive(Debug)]
pub struct BiomechanicsStandingVectorRunner {
    run_root: ContentHash,
    compiled: CompiledBodySchemaV3,
    manifest: MotorTrainingEnvironmentManifestV2,
    manifest_hash: ContentHash,
    left_foot_actor: u64,
    right_foot_actor: u64,
    forward_start_stop: bool,
    forward_start_stop_v2: bool,
    forward_start_stop_v3: bool,
    forward_start_stop_v4: bool,
    periodic_gait: bool,
    lift_return_soles: Option<[WalkingSoleGeometry; 2]>,
    residual_scale_multiplier_q16: i64,
    slots: Vec<BiomechanicsStandingSlot>,
}

impl BiomechanicsStandingVectorRunner {
    pub fn create(
        slot_count: u32,
        run_root: ContentHash,
    ) -> Result<Self, BiomechanicsStandingRunnerError> {
        Self::create_profile(
            BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID,
            slot_count,
            run_root,
        )
    }

    pub fn create_profile(
        profile_id: &str,
        slot_count: u32,
        run_root: ContentHash,
    ) -> Result<Self, BiomechanicsStandingRunnerError> {
        if slot_count == 0 || slot_count > crate::training::MAX_CPU_VECTOR_SLOTS {
            return Err(BiomechanicsStandingRunnerError::SlotCount);
        }
        let (
            forward_start_stop,
            forward_start_stop_v2,
            forward_start_stop_v3,
            forward_start_stop_v4,
            forward_start_stop_v5,
        ) = match profile_id {
            BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID => (false, false, false, false, false),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID => {
                (true, false, false, false, false)
            }
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V2 => {
                (true, true, false, false, false)
            }
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V3 => {
                (true, true, true, false, false)
            }
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V4 => {
                (true, true, true, true, false)
            }
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5
            | BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6
            | BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V7 => {
                (true, true, true, true, true)
            }
            _ => return Err(BiomechanicsStandingRunnerError::ProfileMismatch),
        };
        let schema = if forward_start_stop_v5 {
            biomechanics_humanoid_body_schema_v4()
        } else {
            biomechanics_humanoid_body_schema_v3()
        };
        let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
        let lift_return = profile_id == BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V7;
        let periodic_gait =
            lift_return || profile_id == BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6;
        let manifest = if lift_return {
            crate::biomechanics_forward_start_stop_environment_manifest_v7()?
        } else if periodic_gait {
            crate::biomechanics_forward_start_stop_environment_manifest_v6()?
        } else if forward_start_stop_v5 {
            biomechanics_forward_start_stop_environment_manifest_v5()?
        } else if forward_start_stop_v4 {
            biomechanics_forward_start_stop_environment_manifest_v4()?
        } else if forward_start_stop_v3 {
            biomechanics_forward_start_stop_environment_manifest_v3()?
        } else if forward_start_stop_v2 {
            biomechanics_forward_start_stop_environment_manifest_v2()?
        } else if forward_start_stop {
            biomechanics_forward_start_stop_environment_manifest_v1()?
        } else {
            biomechanics_standing_environment_manifest_v2()?
        };
        let manifest_hash = manifest.manifest_hash()?;
        let residual_scale_multiplier_q16 = if forward_start_stop_v5 {
            BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5
        } else {
            65_536
        };
        let (left_foot_actor, right_foot_actor) = foot_actor_tokens(&compiled.base)?;
        let lift_return_soles = if lift_return {
            let mut soles = [WalkingSoleGeometry {
                actor: 0,
                offset_um: [0; 3],
                half_um: [0; 3],
            }; 2];
            for (sole, (name, actor)) in soles.iter_mut().zip([
                ("body.left-ankle-roll", left_foot_actor),
                ("body.right-ankle-roll", right_foot_actor),
            ]) {
                let body = schema
                    .bodies
                    .iter()
                    .find(|body| body.body_id.as_str() == name)
                    .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
                if body.colliders.len() != 1
                    || body.colliders[0].local_pose.rotation_q1_30 != [0, 0, 0, 1 << 30]
                {
                    return Err(BiomechanicsStandingRunnerError::ProfileMismatch);
                }
                let next_contracts::physics::PhysicsGeometryV1::Box {
                    half_extents_micrometres,
                } = body.colliders[0].geometry
                else {
                    return Err(BiomechanicsStandingRunnerError::ProfileMismatch);
                };
                *sole = WalkingSoleGeometry {
                    actor,
                    offset_um: body.colliders[0].local_pose.translation_micrometres,
                    half_um: half_extents_micrometres,
                };
            }
            Some(soles)
        } else {
            None
        };
        let mut slots = Vec::with_capacity(slot_count as usize);
        for _ in 0..slot_count {
            slots.push(fresh_slot(
                &compiled,
                0,
                false,
                forward_start_stop,
                forward_start_stop_v4,
                None,
            )?);
        }
        Ok(Self {
            run_root,
            compiled,
            manifest,
            manifest_hash,
            left_foot_actor,
            right_foot_actor,
            forward_start_stop,
            forward_start_stop_v2,
            forward_start_stop_v3,
            forward_start_stop_v4,
            periodic_gait,
            lift_return_soles,
            residual_scale_multiplier_q16,
            slots,
        })
    }

    #[must_use]
    pub fn slot_count(&self) -> u32 {
        self.slots.len() as u32
    }

    #[must_use]
    pub fn observation_width(&self) -> u32 {
        if self.lift_return_soles.is_some() {
            88
        } else if self.periodic_gait {
            86
        } else {
            OBSERVATION_WIDTH as u32
        }
    }

    #[must_use]
    pub fn manifest(&self) -> &MotorTrainingEnvironmentManifestV2 {
        &self.manifest
    }

    #[must_use]
    pub const fn manifest_hash(&self) -> ContentHash {
        self.manifest_hash
    }

    pub fn reset_slots(
        &mut self,
        vector_slots: &[u32],
    ) -> Result<Vec<VectorResetOutput>, BiomechanicsStandingRunnerError> {
        if vector_slots.is_empty() {
            return Err(BiomechanicsStandingRunnerError::SlotIdentity);
        }
        let mut seen = BTreeSet::new();
        let mut staged = Vec::with_capacity(vector_slots.len());
        for &vector_slot in vector_slots {
            let index = usize::try_from(vector_slot)
                .ok()
                .filter(|index| *index < self.slots.len())
                .ok_or(BiomechanicsStandingRunnerError::SlotIdentity)?;
            if !seen.insert(vector_slot) {
                return Err(BiomechanicsStandingRunnerError::SlotIdentity);
            }
            let episode_ordinal = self.slots[index]
                .episode_ordinal
                .checked_add(1)
                .ok_or(BiomechanicsStandingRunnerError::EpisodeOverflow)?;
            let seed_set = if self.forward_start_stop {
                derive_curriculum_locomotion_episode_seed_set(
                    self.run_root,
                    episode_ordinal,
                    vector_slot,
                )?
            } else {
                derive_episode_seed_set(self.run_root, episode_ordinal, vector_slot)?
            };
            let command_schedule = if self.forward_start_stop_v2 {
                Some(biomechanics_forward_start_stop_command_schedule_v2())
            } else if self.forward_start_stop {
                let command_seed = seed_set
                    .purpose_seeds
                    .iter()
                    .find(|(purpose, _)| purpose.as_str() == "randomization.command")
                    .map(|(_, seed)| *seed)
                    .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
                Some(curriculum_locomotion_command_schedule(command_seed, 0)?)
            } else {
                None
            };
            let slot = fresh_slot(
                &self.compiled,
                episode_ordinal,
                true,
                self.forward_start_stop,
                self.forward_start_stop_v4,
                command_schedule,
            )?;
            let mut observation_raw = slot_observation(
                &self.compiled.base,
                &slot.snapshot,
                &slot.safety.checkpoint().applied_targets_microradians,
                self.left_foot_actor,
                self.right_foot_actor,
                self.forward_start_stop,
                slot.command_schedule[0],
            )?;
            if self.periodic_gait {
                observation_raw.extend(crate::walking_clock_q1_30(0, slot.command_schedule[0]));
            }
            if let Some(soles) = &self.lift_return_soles {
                observation_raw.extend(sole_heights_um(&slot.snapshot, soles)?);
            }
            let current_observation_root = observation_root(&observation_raw);
            let physics_root = physics_witness_hash_v2(&slot.snapshot);
            let motor_root = motor_state_root(
                &slot,
                0,
                &[],
                self.forward_start_stop,
                slot.command_schedule[0],
            );
            let mut reset_record = MotorResetRecordV2 {
                schema_version: MOTOR_RESET_RECORD_V2_SCHEMA_VERSION,
                environment_manifest_hash: self.manifest_hash,
                environment_profile_id: schema_id(self.manifest.environment_id.as_str()),
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
            let mut slot = slot;
            slot.current_observation_root = current_observation_root;
            slot.last_step_root = reset_record.reset_root;
            staged.push((
                index,
                slot,
                VectorResetOutput {
                    episode_ordinal,
                    vector_slot,
                    seed_set,
                    observation_raw,
                    reset_record,
                },
            ));
        }
        let mut output = Vec::with_capacity(staged.len());
        for (index, slot, reset) in staged {
            self.slots[index] = slot;
            output.push(reset);
        }
        output.sort_by_key(|value| (value.episode_ordinal, value.vector_slot));
        Ok(output)
    }

    pub fn step_actions_lockstep(
        &mut self,
        inputs: Vec<VectorPolicyStepInput>,
    ) -> Result<Vec<BiomechanicsStandingVectorStepOutput>, BiomechanicsStandingRunnerError> {
        if inputs.len() != self.slots.len() {
            return Err(BiomechanicsStandingRunnerError::IncompleteBatch);
        }
        let mut staged = vec![None; self.slots.len()];
        for input in inputs {
            let index = usize::try_from(input.vector_slot)
                .ok()
                .filter(|index| *index < self.slots.len())
                .ok_or(BiomechanicsStandingRunnerError::SlotIdentity)?;
            let slot = &self.slots[index];
            if staged[index].is_some() {
                return Err(BiomechanicsStandingRunnerError::SlotIdentity);
            }
            if !slot.active {
                return Err(BiomechanicsStandingRunnerError::SlotNotReset);
            }
            if slot.terminal_disposition != MotorTerminalDispositionV1::Running {
                return Err(BiomechanicsStandingRunnerError::TerminalSlot);
            }
            if input.episode_ordinal != slot.episode_ordinal {
                return Err(BiomechanicsStandingRunnerError::StaleEpisode);
            }
            if input.action_microradians.len() != ACTION_WIDTH
                || input.action_microradians.iter().any(|value| {
                    !(-crate::NORMALIZED_RESIDUAL_ONE_Q1_30..=crate::NORMALIZED_RESIDUAL_ONE_Q1_30)
                        .contains(value)
                })
            {
                return Err(BiomechanicsStandingRunnerError::ActionLength);
            }
            staged[index] = Some(input.action_microradians);
        }
        if staged.iter().any(Option::is_none) {
            return Err(BiomechanicsStandingRunnerError::IncompleteBatch);
        }
        let mut output = Vec::with_capacity(self.slots.len());
        for (index, action) in staged.into_iter().enumerate() {
            output.push(step_slot(
                &self.compiled,
                index as u32,
                self.left_foot_actor,
                self.right_foot_actor,
                &mut self.slots[index],
                &action.expect("complete staging was checked"),
                self.forward_start_stop,
                self.forward_start_stop_v2,
                self.forward_start_stop_v3,
                self.residual_scale_multiplier_q16,
                self.periodic_gait,
                self.lift_return_soles.as_ref(),
            )?);
        }
        output.sort_by_key(|value| (value.episode_ordinal, value.vector_slot));
        Ok(output)
    }
}

fn fresh_slot(
    compiled: &CompiledBodySchemaV3,
    episode_ordinal: u64,
    active: bool,
    forward_start_stop: bool,
    translation_invariant_reference: bool,
    command_schedule: Option<Vec<[i64; 3]>>,
) -> Result<BiomechanicsStandingSlot, BiomechanicsStandingRunnerError> {
    let mut world = PhysXArticulationWorldV3::create(
        compiled.base.physx_scene_profile,
        &compiled.physx_catalog,
    )?;
    let snapshot = world.capture()?;
    let safety = BiomechanicsSafetyController::new(&compiled.base)?;
    let classifier = BiomechanicsContactClassifier::new(&compiled.base)?;
    let terminal = BiomechanicsTerminalEvaluator::new(
        &compiled.base,
        BiomechanicsSkillContactProfileV1::Locomotion,
        if forward_start_stop {
            BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS
        } else {
            BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS
        },
    )?;
    let standing = if translation_invariant_reference {
        BiomechanicsProceduralStandingControllerV1::new_walking_translation_invariant(
            &compiled.base,
            &snapshot,
        )?
    } else {
        BiomechanicsProceduralStandingControllerV1::new(&compiled.base, &snapshot)?
    };
    let envelopes = safety.default_skill_envelopes();
    Ok(BiomechanicsStandingSlot {
        episode_ordinal,
        world,
        snapshot,
        safety,
        classifier,
        terminal,
        standing,
        envelopes,
        command_schedule: command_schedule.unwrap_or_else(|| {
            vec![[0; 3]; BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS as usize + 1]
        }),
        motor_tick: 0,
        active,
        terminal_disposition: MotorTerminalDispositionV1::Running,
        current_observation_root: ContentHash::default(),
        last_step_root: StateRoot::default(),
    })
}

#[allow(clippy::too_many_arguments)]
fn step_slot(
    compiled: &CompiledBodySchemaV3,
    vector_slot: u32,
    left_foot_actor: u64,
    right_foot_actor: u64,
    slot: &mut BiomechanicsStandingSlot,
    action_q1_30: &[i64],
    forward_start_stop: bool,
    forward_start_stop_v2: bool,
    forward_start_stop_v3: bool,
    residual_scale_multiplier_q16: i64,
    periodic_gait: bool,
    lift_return_soles: Option<&[WalkingSoleGeometry; 2]>,
) -> Result<BiomechanicsStandingVectorStepOutput, BiomechanicsStandingRunnerError> {
    let next_tick = slot
        .motor_tick
        .checked_add(1)
        .ok_or(BiomechanicsStandingRunnerError::EpisodeOverflow)?;
    let tick_index = usize::try_from(slot.motor_tick)
        .map_err(|_| BiomechanicsStandingRunnerError::ArithmeticOverflow)?;
    let next_tick_index = usize::try_from(next_tick)
        .map_err(|_| BiomechanicsStandingRunnerError::ArithmeticOverflow)?;
    let command_raw = *slot
        .command_schedule
        .get(tick_index)
        .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
    let next_command_raw = *slot
        .command_schedule
        .get(next_tick_index)
        .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
    let reference_targets = slot.standing.reference_targets(&slot.snapshot)?;
    let previous_applied_targets = slot.safety.checkpoint().applied_targets_microradians;
    let mut joint_safety_error = None;
    let applied = match slot.safety.begin_motor_tick_with_residual_scale_multiplier(
        &reference_targets,
        action_q1_30,
        &slot.envelopes,
        residual_scale_multiplier_q16,
    ) {
        Ok(targets) => targets,
        Err(error) => {
            joint_safety_error = Some(error);
            Vec::new()
        }
    };
    let applied_targets = if applied.is_empty() {
        slot.safety.checkpoint().applied_targets_microradians
    } else {
        applied
            .iter()
            .map(|target| target.target_microradians)
            .collect()
    };
    let mut absolute_effort_sum = 0_u128;
    let mut contact_frames = Vec::with_capacity(PHYSICS_SUBSTEPS);
    let mut sole_vertical_impulses = [0_u128; 2];
    if joint_safety_error.is_none() {
        for _ in 0..PHYSICS_SUBSTEPS {
            let states = actuator_states(&compiled.base, &slot.snapshot)?;
            let efforts = match slot.safety.step_substep(&states) {
                Ok(efforts) => efforts,
                Err(error) => {
                    joint_safety_error = Some(error);
                    break;
                }
            };
            absolute_effort_sum = absolute_effort_sum
                .checked_add(absolute_effort(&efforts))
                .ok_or(BiomechanicsStandingRunnerError::ArithmeticOverflow)?;
            let mut dof_efforts = vec![0; compiled.base.physx_catalog.joints.len()];
            for (effort, dof) in efforts.iter().zip(&compiled.base.actuator_dof_ordinals) {
                dof_efforts[*dof as usize] = effort.effort_micronewton_metres;
            }
            slot.snapshot = slot.world.apply_efforts_and_step(&dof_efforts)?;
            if periodic_gait {
                for contact in &slot.snapshot.contacts {
                    for (side, foot) in [left_foot_actor, right_foot_actor].into_iter().enumerate()
                    {
                        if (contact.actor_a_token == foot
                            && contact.actor_b_token == crate::HUMANOID_GROUND_ACTOR_TOKEN)
                            || (contact.actor_b_token == foot
                                && contact.actor_a_token == crate::HUMANOID_GROUND_ACTOR_TOKEN)
                        {
                            sole_vertical_impulses[side] +=
                                u128::from(contact.impulse_micronewton_seconds[1].unsigned_abs());
                        }
                    }
                }
            }
            if let Err(error) = slot
                .safety
                .validate_observed_joint_states(&actuator_states(&compiled.base, &slot.snapshot)?)
            {
                joint_safety_error = Some(error);
            }
            contact_frames.push(slot.classifier.classify_substep(
                &slot.snapshot,
                BiomechanicsSkillContactProfileV1::Locomotion,
            )?);
            if joint_safety_error.is_some() {
                break;
            }
        }
    }
    let completed_physics_substeps = contact_frames.len();
    while contact_frames.len() < PHYSICS_SUBSTEPS {
        contact_frames.push(slot.classifier.classify_substep(
            &slot.snapshot,
            BiomechanicsSkillContactProfileV1::Locomotion,
        )?);
    }
    let decision = slot.terminal.evaluate_motor_tick(
        next_tick,
        &slot.snapshot,
        &contact_frames,
        false,
        joint_safety_error,
    )?;
    let root_token = compiled.base.body_tokens[&compiled.base.construction_order[0]];
    let root = slot
        .snapshot
        .links
        .iter()
        .find(|link| link.user_token == root_token)
        .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
    let joint_states = actuator_states(&compiled.base, &slot.snapshot)?;
    let joint_positions = joint_states
        .iter()
        .map(|state| state.position_microradians)
        .collect::<Vec<_>>();
    let contacting_soles = contact_frames
        .iter()
        .flat_map(|frame| &frame.contacts)
        .filter(|contact| contact.class == BiomechanicsContactClassV1::SoleSupport)
        .map(|contact| {
            if contact.pair.actor_a_token == crate::HUMANOID_GROUND_ACTOR_TOKEN {
                contact.pair.actor_b_token
            } else {
                contact.pair.actor_a_token
            }
        })
        .collect::<BTreeSet<_>>();
    let contacting_sole_slip_sum = contacting_soles
        .iter()
        .filter_map(|actor| {
            slot.snapshot
                .links
                .iter()
                .find(|link| link.user_token == *actor)
        })
        .map(|link| {
            u128::from(link.linear_velocity_micrometres_per_second[0].unsigned_abs())
                + u128::from(link.linear_velocity_micrometres_per_second[2].unsigned_abs())
        })
        .sum();
    let contacting_sole_count = u8::try_from(contacting_soles.len())
        .map_err(|_| BiomechanicsStandingRunnerError::ProfileMismatch)?;
    let (mut reward_components_raw, mut reward_total_q16) = if forward_start_stop {
        let local_linear = rotate_world_to_root_local_q1_30(
            root.rotation_q1_30,
            root.linear_velocity_micrometres_per_second,
        )?;
        let local_angular = rotate_world_to_root_local_q1_30(
            root.rotation_q1_30,
            root.angular_velocity_microradians_per_second,
        )?;
        let facts = BiomechanicsForwardStartStopRewardFactsV1 {
            root_rotation_q1_30: root.rotation_q1_30,
            root_height_micrometres: root.position_micrometres[1],
            root_vertical_velocity_micrometres_per_second: root
                .linear_velocity_micrometres_per_second[1],
            root_local_linear_velocity_micrometres_per_second: local_linear,
            root_local_angular_velocity_microradians_per_second: local_angular,
            command_raw,
            absolute_applied_effort_sum_micronewton_metres: absolute_effort_sum,
            applied_targets_microradians: &applied_targets,
            previous_applied_targets_microradians: &previous_applied_targets,
            contacting_sole_slip_sum_micrometres_per_second: contacting_sole_slip_sum,
            contacting_sole_count,
            fell: root.position_micrometres[1] <= BIOMECHANICS_FALL_HEIGHT_MICROMETRES,
        };
        if forward_start_stop_v3 {
            let (components, total) =
                biomechanics_forward_start_stop_reward_q16_v3(compiled, &facts)?;
            (
                BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3
                    .into_iter()
                    .zip(components)
                    .map(|(id, value)| (schema_id(id), value))
                    .collect::<Vec<_>>(),
                total,
            )
        } else if forward_start_stop_v2 {
            let (components, total) =
                biomechanics_forward_start_stop_reward_q16_v2(compiled, &facts)?;
            (
                BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V2
                    .into_iter()
                    .zip(components)
                    .map(|(id, value)| (schema_id(id), value))
                    .collect::<Vec<_>>(),
                total,
            )
        } else {
            let (components, total) =
                biomechanics_forward_start_stop_reward_q16_v1(compiled, &facts)?;
            (
                BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS
                    .into_iter()
                    .zip(components)
                    .map(|(id, value)| (schema_id(id), value))
                    .collect::<Vec<_>>(),
                total,
            )
        }
    } else {
        let facts = BiomechanicsStandingRewardFactsV1 {
            root_rotation_q1_30: root.rotation_q1_30,
            root_height_micrometres: root.position_micrometres[1],
            joint_positions_microradians: &joint_positions,
            reference_targets_microradians: &reference_targets,
            root_linear_velocity_micrometres_per_second: root
                .linear_velocity_micrometres_per_second,
            root_angular_velocity_microradians_per_second: root
                .angular_velocity_microradians_per_second,
            absolute_applied_effort_sum_micronewton_metres: absolute_effort_sum,
            applied_targets_microradians: &applied_targets,
            previous_applied_targets_microradians: &previous_applied_targets,
            contacting_sole_slip_sum_micrometres_per_second: contacting_sole_slip_sum,
            contacting_sole_count,
            fell: root.position_micrometres[1] <= BIOMECHANICS_FALL_HEIGHT_MICROMETRES,
        };
        let (components, total) = biomechanics_standing_reward_q16_v1(compiled, &facts)?;
        (
            BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS
                .into_iter()
                .zip(components)
                .map(|(id, value)| (schema_id(id), value))
                .collect::<Vec<_>>(),
            total,
        )
    };
    let contact_flags = contact_flags(&slot.snapshot, left_foot_actor, right_foot_actor);
    if periodic_gait {
        let mut speeds = [0_u128; 2];
        for (side, foot) in [left_foot_actor, right_foot_actor].into_iter().enumerate() {
            let link = slot
                .snapshot
                .links
                .iter()
                .find(|link| link.user_token == foot)
                .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
            speeds[side] =
                u128::from(link.linear_velocity_micrometres_per_second[0].unsigned_abs())
                    + u128::from(link.linear_velocity_micrometres_per_second[2].unsigned_abs());
        }
        let credit = crate::periodic_load_credit_q16(
            slot.motor_tick,
            command_raw,
            sole_vertical_impulses,
            speeds,
            contact_flags,
        );
        // V3 support is binary with coefficient 1/4, so subtraction is exact.
        reward_total_q16 = reward_total_q16 - reward_components_raw[9].1 / 4 + credit;
        reward_components_raw[9] = (schema_id(crate::WALKING_LOAD_REWARD_ID), credit);
    }
    let mut observation_raw = slot_observation(
        &compiled.base,
        &slot.snapshot,
        &applied_targets,
        left_foot_actor,
        right_foot_actor,
        forward_start_stop,
        next_command_raw,
    )?;
    if periodic_gait {
        observation_raw.extend(crate::walking_clock_q1_30(next_tick, next_command_raw));
    }
    if let Some(soles) = lift_return_soles {
        let heights = sole_heights_um(&slot.snapshot, soles)?;
        let costs = crate::walking_sole_height_costs_q16(slot.motor_tick, command_raw, heights);
        for (id, cost) in crate::WALKING_LIFT_RETURN_REWARD_IDS.into_iter().zip(costs) {
            reward_components_raw.push((schema_id(id), cost));
            reward_total_q16 -= cost;
        }
        observation_raw.extend(heights);
    }
    let next_observation_root = observation_root(&observation_raw);
    let physics_root = physics_witness_hash_v2(&slot.snapshot);
    let motor_root = motor_state_root(
        slot,
        next_tick,
        action_q1_30,
        forward_start_stop,
        next_command_raw,
    );
    let terminal_reason_id = decision.reason.map(|reason| schema_id(reason.stable_id()));
    let terminated = decision.disposition == MotorTerminalDispositionV1::Terminated;
    let truncated = decision.disposition == MotorTerminalDispositionV1::Truncated;
    let mut step_record = MotorStepRecordV2 {
        schema_version: MOTOR_STEP_RECORD_V2_SCHEMA_VERSION,
        episode_ordinal: slot.episode_ordinal,
        vector_slot,
        motor_tick: next_tick,
        prior_observation_root: slot.current_observation_root,
        next_observation_root,
        applied_command_raw: command_raw,
        next_command_raw,
        applied_action_raw: action_q1_30.to_vec(),
        reward_components_q16: reward_components_raw
            .iter()
            .map(|(_, value)| *value)
            .collect(),
        reward_total_q16,
        terminated,
        truncated,
        terminal_reason_id: terminal_reason_id.clone(),
        physics_root,
        motor_root,
        prior_step_root: slot.last_step_root,
        step_root: StateRoot::default(),
    };
    step_record.step_root = step_record.computed_step_root()?;
    step_record.validate()?;
    slot.motor_tick = next_tick;
    slot.current_observation_root = next_observation_root;
    slot.last_step_root = step_record.step_root;
    slot.terminal_disposition = decision.disposition;
    Ok(BiomechanicsStandingVectorStepOutput {
        episode_ordinal: slot.episode_ordinal,
        vector_slot,
        command_raw,
        next_command_raw,
        frame: BiomechanicsStandingFrameResult {
            motor_tick: next_tick,
            applied_action_q1_30: action_q1_30.to_vec(),
            applied_targets_microradians: applied_targets,
            observation_raw,
            snapshot: slot.snapshot.clone(),
            contact_frames,
            contact_flags,
            joint_safety_error,
            completed_physics_substeps,
            safety_checkpoint: joint_safety_error.map(|_| slot.safety.checkpoint()),
        },
        reward_components_raw,
        reward_total_q16,
        terminated,
        truncated,
        terminal_reason_id,
        step_record,
    })
}

fn actuator_states(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
) -> Result<Vec<JointControlStateV1>, BiomechanicsStandingRunnerError> {
    compiled
        .actuator_dof_ordinals
        .iter()
        .map(|ordinal| {
            let joint = snapshot
                .joints
                .get(*ordinal as usize)
                .filter(|joint| joint.ordinal == *ordinal)
                .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
            Ok(JointControlStateV1 {
                position_microradians: joint.position_microradians,
                velocity_microradians_per_second: joint.velocity_microradians_per_second,
            })
        })
        .collect()
}

fn slot_observation(
    compiled: &CompiledBodySchemaV2,
    snapshot: &CanonicalPhysXSnapshotV2,
    applied_targets: &[i64],
    left_foot_actor: u64,
    right_foot_actor: u64,
    root_local: bool,
    command_raw: [i64; 3],
) -> Result<Vec<i64>, BiomechanicsStandingRunnerError> {
    if applied_targets.len() != ACTION_WIDTH {
        return Err(BiomechanicsStandingRunnerError::ProfileMismatch);
    }
    let root_token = compiled.body_tokens[&compiled.construction_order[0]];
    let root = snapshot
        .links
        .iter()
        .find(|link| link.user_token == root_token)
        .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)?;
    let states = actuator_states(compiled, snapshot)?;
    let mut observation = Vec::with_capacity(OBSERVATION_WIDTH);
    observation.extend(root.rotation_q1_30);
    if root_local {
        observation.extend(rotate_world_to_root_local_q1_30(
            root.rotation_q1_30,
            root.linear_velocity_micrometres_per_second,
        )?);
        observation.extend(rotate_world_to_root_local_q1_30(
            root.rotation_q1_30,
            root.angular_velocity_microradians_per_second,
        )?);
    } else {
        observation.extend(root.linear_velocity_micrometres_per_second);
        observation.extend(root.angular_velocity_microradians_per_second);
    }
    observation.extend(states.iter().map(|state| state.position_microradians));
    observation.extend(
        states
            .iter()
            .map(|state| state.velocity_microradians_per_second),
    );
    observation.extend_from_slice(applied_targets);
    observation.extend(command_raw);
    observation.extend(contact_flags(snapshot, left_foot_actor, right_foot_actor));
    if observation.len() != OBSERVATION_WIDTH {
        return Err(BiomechanicsStandingRunnerError::ProfileMismatch);
    }
    Ok(observation)
}

fn contact_flags(
    snapshot: &CanonicalPhysXSnapshotV2,
    left_foot_actor: u64,
    right_foot_actor: u64,
) -> [i64; 2] {
    [left_foot_actor, right_foot_actor].map(|token| {
        i64::from(snapshot.contacts.iter().any(|contact| {
            (contact.actor_a_token == token
                && contact.actor_b_token == crate::HUMANOID_GROUND_ACTOR_TOKEN)
                || (contact.actor_b_token == token
                    && contact.actor_a_token == crate::HUMANOID_GROUND_ACTOR_TOKEN)
        }))
    })
}

fn foot_actor_tokens(
    compiled: &CompiledBodySchemaV2,
) -> Result<(u64, u64), BiomechanicsStandingRunnerError> {
    let find = |suffix: &str| {
        compiled
            .body_tokens
            .iter()
            .find(|(id, _)| id.as_str().ends_with(suffix))
            .map(|(_, token)| *token)
            .ok_or(BiomechanicsStandingRunnerError::ProfileMismatch)
    };
    Ok((find("left-ankle-roll")?, find("right-ankle-roll")?))
}

fn absolute_effort(efforts: &[AppliedActuatorEffortV1]) -> u128 {
    efforts
        .iter()
        .map(|effort| u128::from(effort.effort_micronewton_metres.unsigned_abs()))
        .sum()
}

fn observation_root(observation: &[i64]) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.motor-observation-root.v2\0");
    bytes.extend_from_slice(&(observation.len() as u64).to_le_bytes());
    for value in observation {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn physics_witness_hash_v2(snapshot: &CanonicalPhysXSnapshotV2) -> ContentHash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"nextengine.motor-physics-witness.biomechanics.v1\0");
    bytes.extend_from_slice(&(snapshot.links.len() as u64).to_le_bytes());
    for link in &snapshot.links {
        bytes.extend_from_slice(&link.user_token.to_le_bytes());
        for values in [
            &link.position_micrometres[..],
            &link.rotation_q1_30[..],
            &link.linear_velocity_micrometres_per_second[..],
            &link.angular_velocity_microradians_per_second[..],
        ] {
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    bytes.extend_from_slice(&(snapshot.joints.len() as u64).to_le_bytes());
    for joint in &snapshot.joints {
        bytes.extend_from_slice(&joint.ordinal.to_le_bytes());
        bytes.extend_from_slice(&joint.position_microradians.to_le_bytes());
        bytes.extend_from_slice(&joint.velocity_microradians_per_second.to_le_bytes());
    }
    bytes.extend_from_slice(&(snapshot.contacts.len() as u64).to_le_bytes());
    for contact in &snapshot.contacts {
        bytes.extend_from_slice(&contact.actor_a_token.to_le_bytes());
        bytes.extend_from_slice(&contact.shape_a_token.to_le_bytes());
        bytes.extend_from_slice(&contact.actor_b_token.to_le_bytes());
        bytes.extend_from_slice(&contact.shape_b_token.to_le_bytes());
        for values in [
            &contact.position_micrometres[..],
            &contact.normal_q1_30[..],
            &contact.impulse_micronewton_seconds[..],
        ] {
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&contact.separation_micrometres.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn motor_state_root(
    slot: &BiomechanicsStandingSlot,
    motor_tick: u64,
    action: &[i64],
    forward_start_stop: bool,
    command: [i64; 3],
) -> ContentHash {
    let mut bytes = Vec::new();
    if forward_start_stop {
        bytes.extend_from_slice(b"nextengine.motor-biomechanics-forward-start-stop-state.v1\0");
    } else {
        bytes.extend_from_slice(b"nextengine.motor-biomechanics-standing-state.v1\0");
    }
    bytes.extend_from_slice(&motor_tick.to_le_bytes());
    bytes.extend_from_slice(slot.safety.checkpoint_root().as_bytes());
    bytes.extend_from_slice(slot.classifier.continuity_root().as_bytes());
    bytes.extend_from_slice(slot.terminal.terminal_state_root().as_bytes());
    bytes.extend_from_slice(slot.standing.state_root().as_bytes());
    bytes.extend_from_slice(&(action.len() as u64).to_le_bytes());
    for value in action {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    if forward_start_stop {
        for value in command {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn schema_id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("engine-owned biomechanics training identifiers are valid")
}

#[derive(Debug)]
pub enum BiomechanicsStandingRunnerError {
    Compile(MotorCompileError),
    Physics(PhysXAdapterError),
    Safety(MotorSafetyError),
    Contact(ContactClassificationError),
    Terminal(crate::BiomechanicsTerminalError),
    Standing(ProceduralStandingError),
    Training(TrainingEnvironmentError),
    Observation(MotorObservationError),
    Contract(MotorContractError),
    SlotCount,
    SlotIdentity,
    SlotNotReset,
    IncompleteBatch,
    StaleEpisode,
    TerminalSlot,
    ActionLength,
    EpisodeOverflow,
    ProfileMismatch,
    ArithmeticOverflow,
}

impl BiomechanicsStandingRunnerError {
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Compile(error) => error.stable_code(),
            Self::Physics(error) => error.stable_code(),
            Self::Safety(error) => error.stable_code(),
            Self::Contact(error) => error.stable_code(),
            Self::Terminal(error) => error.stable_code(),
            Self::Standing(error) => error.stable_code(),
            Self::Training(error) => error.stable_code(),
            Self::Observation(error) => error.stable_code(),
            Self::Contract(error) => error.stable_code(),
            Self::SlotCount => "MOTOR_ENV_SLOT_COUNT_INVALID",
            Self::SlotIdentity => "MOTOR_ENV_SLOT_IDENTITY_INVALID",
            Self::SlotNotReset => "MOTOR_ENV_SLOT_NOT_RESET",
            Self::IncompleteBatch => "MOTOR_ENV_BATCH_INCOMPLETE",
            Self::StaleEpisode => "MOTOR_ENV_EPISODE_STALE",
            Self::TerminalSlot => "MOTOR_ENV_SLOT_TERMINAL",
            Self::ActionLength => "MOTOR_ENV_ACTION_LENGTH_INVALID",
            Self::EpisodeOverflow => "MOTOR_ENV_EPISODE_OVERFLOW",
            Self::ProfileMismatch => "MOTOR_ENV_BIOMECHANICS_PROFILE_MISMATCH",
            Self::ArithmeticOverflow => "MOTOR_ENV_ARITHMETIC_OVERFLOW",
        }
    }
}

impl Display for BiomechanicsStandingRunnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.stable_code())
    }
}

impl Error for BiomechanicsStandingRunnerError {}

macro_rules! error_from {
    ($source:ty, $variant:ident) => {
        impl From<$source> for BiomechanicsStandingRunnerError {
            fn from(value: $source) -> Self {
                Self::$variant(value)
            }
        }
    };
}

error_from!(MotorCompileError, Compile);
error_from!(PhysXAdapterError, Physics);
error_from!(MotorSafetyError, Safety);
error_from!(ContactClassificationError, Contact);
error_from!(crate::BiomechanicsTerminalError, Terminal);
error_from!(ProceduralStandingError, Standing);
error_from!(TrainingEnvironmentError, Training);
error_from!(MotorObservationError, Observation);
error_from!(MotorContractError, Contract);

#[cfg(all(test, feature = "physx-sdk"))]
mod tests {
    use super::*;
    use crate::NORMALIZED_RESIDUAL_ONE_Q1_30;

    #[test]
    fn policy_runner_exposes_current_biomechanics_observation_and_q1_30_action() {
        let mut runner =
            BiomechanicsStandingVectorRunner::create(1, ContentHash::from_bytes([7; 32]))
                .expect("runner");
        let reset = runner.reset_slots(&[0]).expect("reset");
        assert_eq!(reset[0].observation_raw.len(), OBSERVATION_WIDTH);
        let step = runner
            .step_actions_lockstep(vec![VectorPolicyStepInput {
                vector_slot: 0,
                episode_ordinal: 1,
                action_microradians: vec![NORMALIZED_RESIDUAL_ONE_Q1_30; ACTION_WIDTH],
            }])
            .expect("step");
        assert_eq!(step[0].frame.motor_tick, 1);
        assert_eq!(step[0].frame.applied_action_q1_30.len(), ACTION_WIDTH);
        assert_eq!(step[0].frame.observation_raw.len(), OBSERVATION_WIDTH);
        assert_eq!(step[0].reward_components_raw.len(), 8);
    }

    #[test]
    fn forward_start_stop_runner_uses_distinct_profile_and_foundation_timeout() {
        let mut runner = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID,
            1,
            ContentHash::from_bytes([9; 32]),
        )
        .expect("walking runner");
        assert_eq!(
            runner.manifest().environment_id.as_str(),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID
        );
        assert_eq!(runner.manifest().maximum_episode_steps, 1_200);
        let reset = runner.reset_slots(&[0]).expect("walking reset");
        assert_eq!(&reset[0].observation_raw[79..82], &[0, 0, 0]);
        let step = runner
            .step_actions_lockstep(vec![VectorPolicyStepInput {
                vector_slot: 0,
                episode_ordinal: 1,
                action_microradians: vec![0; ACTION_WIDTH],
            }])
            .expect("walking step");
        assert_eq!(step[0].reward_components_raw.len(), 11);
        assert_eq!(step[0].command_raw, [0; 3]);
    }

    #[test]
    fn forward_start_stop_v4_uses_translation_invariant_reference_identity() {
        let mut v3 = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V3,
            1,
            ContentHash::from_bytes([11; 32]),
        )
        .expect("v3 runner");
        let mut v4 = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V4,
            1,
            ContentHash::from_bytes([11; 32]),
        )
        .expect("v4 runner");
        let v3_reset = v3.reset_slots(&[0]).expect("v3 reset");
        let v4_reset = v4.reset_slots(&[0]).expect("v4 reset");
        assert_eq!(v3_reset[0].observation_raw, v4_reset[0].observation_raw);
        assert_ne!(
            v3_reset[0].reset_record.motor_root,
            v4_reset[0].reset_record.motor_root
        );
        assert_ne!(v3.manifest_hash(), v4.manifest_hash());
        assert_ne!(
            v3.manifest().action_layout_hash,
            v4.manifest().action_layout_hash
        );
    }

    #[test]
    fn zero_policy_survives_the_native_contact_preflight() {
        let mut runner =
            BiomechanicsStandingVectorRunner::create(1, ContentHash::from_bytes([7; 32]))
                .expect("runner");
        runner.reset_slots(&[0]).expect("reset");
        for expected_tick in 1..=32 {
            let step = runner
                .step_actions_lockstep(vec![VectorPolicyStepInput {
                    vector_slot: 0,
                    episode_ordinal: 1,
                    action_microradians: vec![0; ACTION_WIDTH],
                }])
                .expect("step");
            assert_eq!(step[0].frame.motor_tick, expected_tick);
            assert!(!step[0].terminated, "unexpected native termination");
            assert!(!step[0].truncated, "unexpected native truncation");
            assert_eq!(step[0].frame.joint_safety_error, None);
            assert_eq!(step[0].frame.completed_physics_substeps, PHYSICS_SUBSTEPS);
            assert!(
                step[0]
                    .frame
                    .contact_frames
                    .iter()
                    .flat_map(|frame| &frame.contacts)
                    .all(|contact| {
                        contact.class != BiomechanicsContactClassV1::SelfCollisionViolation
                    }),
                "native zero policy produced a self-collision"
            );
        }
    }

    #[test]
    fn periodic_reward_changes_neither_native_dynamics_nor_safety() {
        let mut old = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5,
            1,
            ContentHash::from_bytes([19; 32]),
        )
        .expect("V5");
        let mut new = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6,
            1,
            ContentHash::from_bytes([19; 32]),
        )
        .expect("V6");
        assert_eq!(old.manifest.body_schema_hash, new.manifest.body_schema_hash);
        assert_eq!(
            old.manifest.action_layout_hash,
            new.manifest.action_layout_hash
        );
        assert_eq!(
            old.manifest.termination_profile_hash,
            new.manifest.termination_profile_hash
        );
        assert_ne!(
            old.manifest.reward_profile_hash,
            new.manifest.reward_profile_hash
        );
        assert_ne!(
            old.manifest.observation_layout_hash,
            new.manifest.observation_layout_hash
        );
        let old_reset = old.reset_slots(&[0]).expect("reset V5");
        let new_reset = new.reset_slots(&[0]).expect("reset V6");
        assert_eq!(
            old_reset[0].observation_raw,
            new_reset[0].observation_raw[..84]
        );
        assert_eq!(&new_reset[0].observation_raw[84..], &[0, 0]);
        let mut moving_credit_seen = false;
        for _ in 0..300 {
            let action = vec![VectorPolicyStepInput {
                vector_slot: 0,
                episode_ordinal: 1,
                action_microradians: vec![0; 23],
            }];
            let a = old
                .step_actions_lockstep(action.clone())
                .expect("step V5")
                .remove(0);
            let b = new
                .step_actions_lockstep(action)
                .expect("step V6")
                .remove(0);
            assert_eq!(a.frame.snapshot, b.frame.snapshot);
            assert_eq!(
                a.frame.applied_targets_microradians,
                b.frame.applied_targets_microradians
            );
            assert_eq!(a.terminal_reason_id, b.terminal_reason_id);
            assert_eq!(a.frame.joint_safety_error, b.frame.joint_safety_error);
            assert_eq!(a.frame.observation_raw, b.frame.observation_raw[..84]);
            assert_eq!(
                &b.frame.observation_raw[84..],
                &crate::walking_clock_q1_30(b.frame.motor_tick, b.next_command_raw)
            );
            for index in (0..11).filter(|index| *index != 9) {
                assert_eq!(
                    a.reward_components_raw[index],
                    b.reward_components_raw[index]
                );
            }
            let credit = b.reward_components_raw[9].1;
            assert!((0..=65_536).contains(&credit));
            if b.command_raw != [0; 3] && credit > 0 {
                moving_credit_seen = true;
            }
            assert_eq!(
                b.reward_total_q16,
                a.reward_total_q16 - a.reward_components_raw[9].1 / 4 + credit
            );
            if a.terminated || a.truncated {
                break;
            }
        }
        assert!(
            moving_credit_seen,
            "native contact impulses must produce observable movement-phase credit"
        );
    }

    #[test]
    fn lift_return_changes_neither_native_dynamics_nor_safety() {
        let root = ContentHash::from_bytes([19; 32]);
        let mut old = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6,
            1,
            root,
        )
        .expect("V6");
        let mut new = BiomechanicsStandingVectorRunner::create_profile(
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V7,
            1,
            root,
        )
        .expect("V7");
        assert_eq!(new.observation_width(), 88);
        assert_eq!(old.manifest.body_schema_hash, new.manifest.body_schema_hash);
        assert_eq!(
            old.manifest.action_layout_hash,
            new.manifest.action_layout_hash
        );
        assert_eq!(
            old.manifest.termination_profile_hash,
            new.manifest.termination_profile_hash
        );
        assert_ne!(
            old.manifest.observation_layout_hash,
            new.manifest.observation_layout_hash
        );
        assert_ne!(
            old.manifest.reward_profile_hash,
            new.manifest.reward_profile_hash
        );
        let a = old.reset_slots(&[0]).expect("reset V6");
        let b = new.reset_slots(&[0]).expect("reset V7");
        assert_eq!(a[0].observation_raw, b[0].observation_raw[..86]);
        assert_eq!(b[0].observation_raw[86..], [0, 0]);
        let mut saw_cost = false;
        let mut saw_terminal = false;
        for _ in 0..1200 {
            let action = vec![VectorPolicyStepInput {
                vector_slot: 0,
                episode_ordinal: 1,
                action_microradians: vec![0; 23],
            }];
            let a = old
                .step_actions_lockstep(action.clone())
                .expect("step V6")
                .remove(0);
            let b = new
                .step_actions_lockstep(action)
                .expect("step V7")
                .remove(0);
            assert_eq!(a.frame.snapshot, b.frame.snapshot);
            assert_eq!(
                a.frame.applied_targets_microradians,
                b.frame.applied_targets_microradians
            );
            assert_eq!(a.frame.joint_safety_error, b.frame.joint_safety_error);
            assert_eq!(a.frame.safety_checkpoint, b.frame.safety_checkpoint);
            assert_eq!(
                a.frame.completed_physics_substeps,
                b.frame.completed_physics_substeps
            );
            assert_eq!(a.terminal_reason_id, b.terminal_reason_id);
            assert_eq!((a.terminated, a.truncated), (b.terminated, b.truncated));
            assert_eq!(a.frame.observation_raw, b.frame.observation_raw[..86]);
            assert_eq!(a.reward_components_raw, b.reward_components_raw[..11]);
            let heights: [i64; 2] = b.frame.observation_raw[86..].try_into().unwrap();
            let cost =
                crate::walking_sole_height_cost_q16(b.frame.motor_tick - 1, b.command_raw, heights);
            assert_eq!(
                b.reward_components_raw[11].1 + b.reward_components_raw[12].1,
                cost
            );
            assert_eq!(b.reward_total_q16, a.reward_total_q16 - cost);
            saw_cost |= cost > 0;
            if a.terminated || a.truncated {
                saw_terminal = true;
                break;
            }
        }
        assert!(
            saw_cost && saw_terminal,
            "control must exercise objective and actual terminal"
        );
    }

    #[test]
    fn saturated_policy_prioritizes_joint_safety_over_simultaneous_self_collision() {
        let action = vec![
            153_828_000,
            -385_996_096,
            991_093_376,
            -1_073_741_824,
            602_893_120,
            800_711_552,
            883_240_960,
            1_073_741_824,
            -1_073_741_824,
            -1_073_741_824,
            -137_148_928,
            -601_897_536,
            1_073_741_824,
            -1_073_741_824,
            -162_109_088,
            -34_855_964,
            213_967_200,
            -137_256_592,
            -1_040_632_000,
            -800_568_192,
            206_366_320,
            -575_262_592,
            21_493_304,
        ];
        let mut runner =
            BiomechanicsStandingVectorRunner::create(1, ContentHash::from_bytes([7; 32]))
                .expect("runner");
        runner.reset_slots(&[0]).expect("reset");
        let mut terminal = None;
        for _ in 0..60 {
            let step = runner
                .step_actions_lockstep(vec![VectorPolicyStepInput {
                    vector_slot: 0,
                    episode_ordinal: 1,
                    action_microradians: action.clone(),
                }])
                .expect("step");
            if step[0].terminated {
                terminal = step.into_iter().next();
                break;
            }
        }
        let step = terminal.expect("saturated policy must reach a safety terminal");
        let violations = step
            .frame
            .contact_frames
            .iter()
            .flat_map(|frame| &frame.contacts)
            .filter(|contact| contact.class == BiomechanicsContactClassV1::SelfCollisionViolation)
            .collect::<Vec<_>>();
        assert_eq!(
            step.terminal_reason_id.as_ref().map(SchemaId::as_str),
            Some("terminal.joint-safety")
        );
        assert!(!violations.is_empty());
        assert!(step.frame.joint_safety_error.is_some());
        assert!(step.frame.completed_physics_substeps <= PHYSICS_SUBSTEPS);
    }
}
