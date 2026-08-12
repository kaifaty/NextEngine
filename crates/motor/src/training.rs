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
pub const CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID: &str =
    "nextengine.motor.env.humanoid-flat-command-curriculum.v2";
pub const ISAAC_TRANSLATOR_VERSION: &str = "nextengine.isaac-usda-translator.v3";

// Standing and flat-command V1 manifests are immutable compatibility records.
// Their historical identity predates the exact executable translator version
// string used by curriculum V2 and must remain byte-for-byte stable.
const LEGACY_ISAAC_TRANSLATOR_PROFILE_ID: &str = "nextengine.isaac-translator.v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum CurriculumSupportCommandModeV2 {
    ExactZero = 1,
}

const CURRICULUM_SUPPORT_COMMAND_MODE_V2: CurriculumSupportCommandModeV2 =
    CurriculumSupportCommandModeV2::ExactZero;

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

pub const CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS: [&str; 11] = [
    "reward.planar-command-tracking",
    "reward.yaw-rate-tracking",
    "reward.upright-yaw-invariant",
    "reward.root-height-tracking",
    "reward.vertical-velocity-cost",
    "reward.roll-pitch-rate-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-action-rate-cost",
    "reward.contacting-foot-tangential-slip-cost",
    "reward.command-conditioned-support",
    "reward.fall-component",
];

pub const CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16: [i64; 11] = [
    131_072, 32_768, 65_536, 32_768, -6_554, -6_554, -655, -1_311, -13_107, 16_384, -655_360,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MotorEnvironmentProfile {
    StandingV1,
    HumanoidFlatCommandV1,
    HumanoidFlatCommandCurriculumV2,
}

impl MotorEnvironmentProfile {
    pub fn parse_exact(profile_id: &str) -> Result<Self, TrainingEnvironmentError> {
        match profile_id {
            STANDING_ENVIRONMENT_PROFILE_ID => Ok(Self::StandingV1),
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID => Ok(Self::HumanoidFlatCommandV1),
            CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID => {
                Ok(Self::HumanoidFlatCommandCurriculumV2)
            }
            _ => Err(TrainingEnvironmentError::UnsupportedProfile),
        }
    }

    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        match self {
            Self::StandingV1 => STANDING_ENVIRONMENT_PROFILE_ID,
            Self::HumanoidFlatCommandV1 => FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
            Self::HumanoidFlatCommandCurriculumV2 => CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID,
        }
    }

    #[must_use]
    const fn is_locomotion(self) -> bool {
        matches!(
            self,
            Self::HumanoidFlatCommandV1 | Self::HumanoidFlatCommandCurriculumV2
        )
    }

    const fn maximum_episode_steps(self) -> u64 {
        match self {
            Self::StandingV1 => DEFAULT_MAX_EPISODE_MOTOR_STEPS,
            Self::HumanoidFlatCommandV1 | Self::HumanoidFlatCommandCurriculumV2 => {
                FLAT_LOCOMOTION_MAX_EPISODE_MOTOR_STEPS
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocomotionCurriculumStageV2 {
    pub first_episode_ordinal: u64,
    pub command_profile: MotorLocomotionCommandProfileV1,
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

#[must_use]
pub fn curriculum_locomotion_stages_v2() -> Vec<LocomotionCurriculumStageV2> {
    vec![
        LocomotionCurriculumStageV2 {
            first_episode_ordinal: 0,
            command_profile: MotorLocomotionCommandProfileV1 {
                schema_version: MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION,
                profile_id: schema_id(
                    "nextengine.motor.command.humanoid-flat-curriculum.foundation.v2",
                ),
                randomization_stream_id: schema_id("randomization.command"),
                warmup_ticks: 120,
                segment_ticks: 240,
                episode_ticks: 1_200,
                mode_weights_basis_points: [4_000, 6_000, 0, 0],
                right_velocity_min_micrometres_per_second: 0,
                right_velocity_max_micrometres_per_second: 0,
                forward_velocity_min_micrometres_per_second: 0,
                forward_velocity_max_micrometres_per_second: 750_000,
                yaw_rate_min_microradians_per_second: 0,
                yaw_rate_max_microradians_per_second: 0,
                linear_rate_limit_micrometres_per_second_squared: 1_000_000,
                yaw_rate_limit_microradians_per_second_squared: 500_000,
            },
        },
        LocomotionCurriculumStageV2 {
            first_episode_ordinal: 32,
            command_profile: MotorLocomotionCommandProfileV1 {
                schema_version: MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION,
                profile_id: schema_id(
                    "nextengine.motor.command.humanoid-flat-curriculum.steering.v2",
                ),
                randomization_stream_id: schema_id("randomization.command"),
                warmup_ticks: 90,
                segment_ticks: 180,
                episode_ticks: 1_200,
                mode_weights_basis_points: [2_500, 5_500, 1_500, 500],
                right_velocity_min_micrometres_per_second: -350_000,
                right_velocity_max_micrometres_per_second: 350_000,
                forward_velocity_min_micrometres_per_second: 0,
                forward_velocity_max_micrometres_per_second: 1_250_000,
                yaw_rate_min_microradians_per_second: -600_000,
                yaw_rate_max_microradians_per_second: 600_000,
                linear_rate_limit_micrometres_per_second_squared: 1_500_000,
                yaw_rate_limit_microradians_per_second_squared: 750_000,
            },
        },
        LocomotionCurriculumStageV2 {
            first_episode_ordinal: 96,
            command_profile: MotorLocomotionCommandProfileV1 {
                schema_version: MOTOR_LOCOMOTION_COMMAND_PROFILE_V1_SCHEMA_VERSION,
                profile_id: schema_id("nextengine.motor.command.humanoid-flat-curriculum.full.v2"),
                randomization_stream_id: schema_id("randomization.command"),
                warmup_ticks: 60,
                segment_ticks: 120,
                episode_ticks: 1_200,
                mode_weights_basis_points: [1_500, 4_500, 1_500, 2_500],
                right_velocity_min_micrometres_per_second: -1_000_000,
                right_velocity_max_micrometres_per_second: 1_000_000,
                forward_velocity_min_micrometres_per_second: -500_000,
                forward_velocity_max_micrometres_per_second: 2_000_000,
                yaw_rate_min_microradians_per_second: -1_000_000,
                yaw_rate_max_microradians_per_second: 1_000_000,
                linear_rate_limit_micrometres_per_second_squared: 2_000_000,
                yaw_rate_limit_microradians_per_second_squared: 1_000_000,
            },
        },
    ]
}

pub fn flat_locomotion_command_schedule(
    command_seed: [u8; 32],
) -> Result<Vec<[i64; 3]>, TrainingEnvironmentError> {
    let profile = flat_locomotion_command_profile_v1();
    command_schedule_from_profile(&profile, command_seed)
}

pub fn curriculum_locomotion_command_schedule(
    command_seed: [u8; 32],
    episode_ordinal: u64,
) -> Result<Vec<[i64; 3]>, TrainingEnvironmentError> {
    let stages = curriculum_locomotion_stages_v2();
    let stage = stages
        .iter()
        .rev()
        .find(|stage| episode_ordinal >= stage.first_episode_ordinal)
        .ok_or(TrainingEnvironmentError::ScheduleBounds)?;
    command_schedule_from_profile(&stage.command_profile, command_seed)
}

fn command_schedule_from_profile(
    profile: &MotorLocomotionCommandProfileV1,
    command_seed: [u8; 32],
) -> Result<Vec<[i64; 3]>, TrainingEnvironmentError> {
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
            target = command_target(profile, command_seed, segment_index)?;
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

pub fn derive_curriculum_locomotion_episode_seed_set(
    run_root: ContentHash,
    episode_ordinal: u64,
    vector_slot: u32,
) -> Result<MotorEpisodeSeedSetV1, TrainingEnvironmentError> {
    derive_episode_seed_set_for_profile(
        MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2,
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
        MotorEnvironmentProfile::HumanoidFlatCommandV1
        | MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
            derive_seed_set_from_purposes(
                run_root,
                episode_ordinal,
                vector_slot,
                &LOCOMOTION_RANDOMIZATION_PURPOSES,
            )
        }
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
        MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
            let command_id = schema_id("randomization.command");
            let command_seed = seed_set
                .purpose_seeds
                .iter()
                .find_map(|(purpose, seed)| (purpose == &command_id).then_some(*seed))
                .ok_or(TrainingEnvironmentError::SeedProfile)?;
            curriculum_locomotion_command_schedule(command_seed, seed_set.episode_ordinal)
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
    if profile.is_locomotion() {
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
        MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
            CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS
                .into_iter()
                .zip(CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16)
                .map(|(component_id, coefficient_q16)| MotorRewardComponentV1 {
                    component_id: schema_id(component_id),
                    coefficient_q16,
                    minimum_raw: 0,
                    maximum_raw: 65_536,
                })
                .collect()
        }
    };
    let command_schedule_profile_hash = match profile {
        MotorEnvironmentProfile::StandingV1 => profile_constant_hash(
            "nextengine.motor.command.external-standing.v1",
            compiled.body_schema_hash,
        ),
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
            flat_locomotion_command_profile_v1().profile_hash()?
        }
        MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
            curriculum_locomotion_profile_hash_v2()?
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
                MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
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
            match profile {
                MotorEnvironmentProfile::StandingV1
                | MotorEnvironmentProfile::HumanoidFlatCommandV1 => {
                    LEGACY_ISAAC_TRANSLATOR_PROFILE_ID
                }
                MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
                    ISAAC_TRANSLATOR_VERSION
                }
            },
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
                MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
                    "nextengine.motor.termination.flat-command-curriculum.v2"
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

pub fn curriculum_locomotion_profile_hash_v2() -> Result<ContentHash, TrainingEnvironmentError> {
    let stages = curriculum_locomotion_stages_v2();
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor-command-curriculum.v2\0");
    preimage.extend_from_slice(&(stages.len() as u32).to_le_bytes());
    for stage in stages {
        stage.command_profile.validate()?;
        preimage.extend_from_slice(&stage.first_episode_ordinal.to_le_bytes());
        preimage.extend_from_slice(stage.command_profile.profile_hash()?.as_bytes());
    }
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn reward_profile_hash(
    profile: MotorEnvironmentProfile,
    body_schema_hash: ContentHash,
) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor.reward-profile.v2\0");
    preimage.extend_from_slice(profile.profile_id().as_bytes());
    preimage.extend_from_slice(body_schema_hash.as_bytes());
    let (components, normalizations) = match profile {
        MotorEnvironmentProfile::StandingV1 => return content_hash_from_bytes(sha256(&preimage)),
        MotorEnvironmentProfile::HumanoidFlatCommandV1 => (
            LOCOMOTION_REWARD_COMPONENT_IDS
                .into_iter()
                .zip(LOCOMOTION_REWARD_COEFFICIENTS_Q16)
                .collect::<Vec<_>>(),
            [
                6_500_000_i64,
                3_000_000,
                600_000,
                3_000_000,
                6_000_000,
                4_000_000,
            ],
        ),
        MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
            let shaping_id = "squared-tracking-command-support-exact-zero-v2";
            preimage.extend_from_slice(&(shaping_id.len() as u32).to_le_bytes());
            preimage.extend_from_slice(shaping_id.as_bytes());
            preimage.push(CURRICULUM_SUPPORT_COMMAND_MODE_V2 as u8);
            (
                CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS
                    .into_iter()
                    .zip(CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16)
                    .collect::<Vec<_>>(),
                [
                    2_500_000_i64,
                    1_500_000,
                    400_000,
                    2_000_000,
                    4_000_000,
                    2_000_000,
                ],
            )
        }
    };
    for (component, coefficient) in &components {
        preimage.extend_from_slice(&(component.len() as u32).to_le_bytes());
        preimage.extend_from_slice(component.as_bytes());
        preimage.extend_from_slice(&coefficient.to_le_bytes());
    }
    for normalization in normalizations {
        preimage.extend_from_slice(&normalization.to_le_bytes());
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

mod reward;
use reward::{locomotion_reward_components, standing_reward_components};

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
        MotorEnvironmentProfile::HumanoidFlatCommandV1
        | MotorEnvironmentProfile::HumanoidFlatCommandCurriculumV2 => {
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
