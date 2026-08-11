use next_contracts::canonical::sha256;
use next_contracts::ids::{
    ContentHash, PersistentId, SchemaId, StateRoot, content_hash_from_bytes,
};
use next_contracts::motor::{
    MOTOR_ENVIRONMENT_CHECKPOINT_ENVELOPE_V1_SCHEMA_VERSION,
    MOTOR_EPISODE_SEED_SET_V1_SCHEMA_VERSION, MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION,
    MOTOR_RESET_RECORD_V2_SCHEMA_VERSION, MOTOR_STEP_RECORD_V2_SCHEMA_VERSION,
    MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION, MotorEnvironmentCheckpointEnvelopeV1,
    MotorEpisodeSeedSetV1, MotorLocomotionCommandModeV1, MotorLocomotionCommandProfileV1,
    MotorResetRecordV2, MotorRewardComponentV1, MotorStepRecordV2, MotorTerminalDispositionV1,
    MotorTrainingEnvironmentManifestV2, STAGE0_MOTOR_HZ, STAGE0_PHYSICS_HZ, STAGE0_SUBSTEPS,
};
use std::collections::BTreeSet;

use crate::runtime::physics_witness_hash;
use crate::{
    CompiledBodySchemaV1, DeterministicHumanoidMotor, HumanoidMotorCheckpoint, MotorFrameResult,
    REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES, reference_humanoid_body_schema_v1,
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

mod runner;

pub use runner::MotorVectorRunner;

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
        .saturating_sub(REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES)
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
    let root_height_tracking = root.map_or(
        -REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES,
        |root| {
            -unsigned_sum([root.position_micrometres[1]
                .saturating_sub(REFERENCE_HUMANOID_STANDING_ROOT_HEIGHT_MICROMETRES)])
        },
    );
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

mod error;

pub use error::TrainingEnvironmentError;

#[cfg(all(test, any(feature = "physx-sdk", feature = "mock-abi")))]
mod tests;
