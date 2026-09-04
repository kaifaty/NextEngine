use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::motor::{
    MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION, MotorRewardComponentV1,
    MotorTrainingEnvironmentManifestV2, STAGE0_MOTOR_HZ, STAGE0_PHYSICS_HZ,
};
use serde_json::{Value, json};

use crate::{
    BIOMECHANICS_FALL_HEIGHT_MICROMETRES, BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
    BIOMECHANICS_WORLD_BOUND_MICROMETRES, CompiledBodySchemaV3, MotorCompileError,
    PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS, PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS,
    PROCEDURAL_WALKING_REFERENCE_PROFILE_ID_V1, biomechanics_humanoid_body_schema_v2,
    biomechanics_humanoid_body_schema_v3, biomechanics_humanoid_body_schema_v4,
    biomechanics_isaac_mirror_descriptor_json_v2, biomechanics_isaac_mirror_descriptor_json_v3,
    biomechanics_isaac_mirror_descriptor_json_v4,
};

pub const BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID_V1: &str =
    "nextengine.motor.env.humanoid-biomechanics-standing.v1";
pub const BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID: &str =
    "nextengine.motor.env.humanoid-biomechanics-standing.v2";
pub const BIOMECHANICS_STANDING_OBSERVATION_LAYOUT_ID: &str =
    "nextengine.motor.observation.humanoid-biomechanics-standing.v2";
pub const BIOMECHANICS_STANDING_ACTION_LAYOUT_ID: &str =
    "nextengine.motor.action.humanoid-biomechanics-standing-residual.v2";
pub const BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS: u64 = 3_600;
pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v1";
pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V2: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v2";
pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V3: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v3";
pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V4: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v4";
pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v5";
pub const BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6: &str =
    "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v6";
pub const BIOMECHANICS_FORWARD_START_STOP_OBSERVATION_LAYOUT_ID: &str =
    "nextengine.motor.observation.humanoid-biomechanics-forward-start-stop.v1";
pub const BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID: &str =
    "nextengine.motor.action.humanoid-biomechanics-forward-start-stop-residual.v1";
pub const BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V2: &str =
    "nextengine.motor.action.humanoid-biomechanics-forward-start-stop-residual.v2";
pub const BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V3: &str =
    "nextengine.motor.action.humanoid-biomechanics-forward-start-stop-residual.v3";
pub const BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5: i64 = 262_144;
pub const BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS: u64 = 1_200;
pub const BIOMECHANICS_FORWARD_START_STOP_TARGET_MICROMETRES_PER_SECOND_V2: i64 = 500_000;
pub const BIOMECHANICS_FORWARD_START_STOP_RAMP_DOWN_TICK_V2: u64 = 991;
pub const BIOMECHANICS_FORWARD_START_STOP_FINAL_ZERO_TICKS_V2: u64 = 180;

pub const BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS: [&str; 8] = [
    "reward.upright-yaw-invariant",
    "reward.root-height-tracking",
    "reward.procedural-standing-pose-tracking-normalized",
    "reward.root-motion-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-target-rate-cost",
    "reward.contacting-sole-tangential-slip-cost",
    "reward.fall-component",
];
pub const BIOMECHANICS_STANDING_REWARD_COEFFICIENTS_Q16: [i64; 8] = [
    65_536, 32_768, 16_384, -6_554, -1_311, -3_277, -6_554, -131_072,
];

pub const BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS: [&str; 11] =
    crate::CURRICULUM_LOCOMOTION_REWARD_COMPONENT_IDS;
pub const BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16: [i64; 11] =
    crate::CURRICULUM_LOCOMOTION_REWARD_COEFFICIENTS_Q16;
pub const BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V2: [&str; 11] = [
    "reward.planar-command-tracking-compact",
    "reward.yaw-rate-tracking-compact",
    "reward.root-tilt-cost",
    "reward.root-height-error-cost",
    "reward.vertical-velocity-cost",
    "reward.roll-pitch-rate-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-target-rate-cost",
    "reward.contacting-sole-tangential-slip-cost",
    "reward.command-conditioned-support",
    "reward.fall-component",
];
pub const BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2: [i64; 11] = [
    131_072, 16_384, -32_768, -16_384, -6_554, -6_554, -1_311, -3_277, -6_554, 16_384, -655_360,
];
pub const BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3: [&str; 11] = [
    "reward.planar-command-tracking-dense",
    "reward.yaw-rate-tracking-dense",
    "reward.root-tilt-cost",
    "reward.root-height-error-cost",
    "reward.vertical-velocity-cost",
    "reward.roll-pitch-rate-cost",
    "reward.normalized-applied-effort-cost",
    "reward.applied-target-rate-cost",
    "reward.contacting-sole-tangential-slip-cost",
    "reward.command-conditioned-support",
    "reward.fall-component",
];
pub const BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3: [i64; 11] =
    BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2;

#[derive(Clone, Debug)]
pub struct BiomechanicsStandingRewardFactsV1<'a> {
    pub root_rotation_q1_30: [i64; 4],
    pub root_height_micrometres: i64,
    pub joint_positions_microradians: &'a [i64],
    pub reference_targets_microradians: &'a [i64],
    pub root_linear_velocity_micrometres_per_second: [i64; 3],
    pub root_angular_velocity_microradians_per_second: [i64; 3],
    pub absolute_applied_effort_sum_micronewton_metres: u128,
    pub applied_targets_microradians: &'a [i64],
    pub previous_applied_targets_microradians: &'a [i64],
    pub contacting_sole_slip_sum_micrometres_per_second: u128,
    pub contacting_sole_count: u8,
    pub fell: bool,
}

#[derive(Clone, Debug)]
pub struct BiomechanicsForwardStartStopRewardFactsV1<'a> {
    pub root_rotation_q1_30: [i64; 4],
    pub root_height_micrometres: i64,
    pub root_vertical_velocity_micrometres_per_second: i64,
    pub root_local_linear_velocity_micrometres_per_second: [i64; 3],
    pub root_local_angular_velocity_microradians_per_second: [i64; 3],
    pub command_raw: [i64; 3],
    pub absolute_applied_effort_sum_micronewton_metres: u128,
    pub applied_targets_microradians: &'a [i64],
    pub previous_applied_targets_microradians: &'a [i64],
    pub contacting_sole_slip_sum_micrometres_per_second: u128,
    pub contacting_sole_count: u8,
    pub fell: bool,
}

pub fn biomechanics_standing_reward_q16_v1(
    compiled: &CompiledBodySchemaV3,
    facts: &BiomechanicsStandingRewardFactsV1<'_>,
) -> Result<([i64; 8], i64), crate::TrainingEnvironmentError> {
    let width = compiled.base.actuator_definitions.len();
    if width != 23
        || facts.joint_positions_microradians.len() != width
        || facts.reference_targets_microradians.len() != width
        || facts.applied_targets_microradians.len() != width
        || facts.previous_applied_targets_microradians.len() != width
        || facts.contacting_sole_count > 2
    {
        return Err(crate::TrainingEnvironmentError::RewardFacts);
    }
    let (pose_normalization, effort_normalization, target_rate_normalization) =
        standing_normalizations(compiled);
    let [x, _, z, _] = facts.root_rotation_q1_30;
    let tilt = i128::from(x)
        .checked_mul(i128::from(x))
        .and_then(|value| {
            i128::from(z)
                .checked_mul(i128::from(z))
                .and_then(|other| value.checked_add(other))
        })
        .and_then(|value| value.checked_mul(2))
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let tilt_q30 = round_shift_ties_even(tilt, 30)?;
    let upright_q30 = (1_i64 << 30).saturating_sub(tilt_q30).clamp(0, 1_i64 << 30);
    let upright = ratio_q16(upright_q30 as u128, 1_u128 << 30)?;
    let height = one_minus_q16(
        facts
            .root_height_micrometres
            .saturating_sub(BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES)
            .unsigned_abs() as u128,
        600_000,
    )?;
    let pose_error = facts
        .joint_positions_microradians
        .iter()
        .zip(facts.reference_targets_microradians)
        .map(|(position, reference)| position.saturating_sub(*reference).unsigned_abs() as u128)
        .sum::<u128>();
    let pose = one_minus_q16(pose_error, pose_normalization as u128)?;
    let linear = ratio_q16(
        facts
            .root_linear_velocity_micrometres_per_second
            .into_iter()
            .map(|value| value.unsigned_abs() as u128)
            .sum(),
        9_000_000,
    )?;
    let angular = ratio_q16(
        facts
            .root_angular_velocity_microradians_per_second
            .into_iter()
            .map(|value| value.unsigned_abs() as u128)
            .sum(),
        18_000_000,
    )?;
    let effort = ratio_q16(
        facts.absolute_applied_effort_sum_micronewton_metres,
        effort_normalization as u128,
    )?;
    let target_rate = ratio_q16(
        facts
            .applied_targets_microradians
            .iter()
            .zip(facts.previous_applied_targets_microradians)
            .map(|(current, previous)| current.saturating_sub(*previous).unsigned_abs() as u128)
            .sum(),
        target_rate_normalization as u128,
    )?;
    let slip_denominator = u128::from(facts.contacting_sole_count) * 4_000_000;
    let slip = if slip_denominator == 0 {
        0
    } else {
        ratio_q16(
            facts.contacting_sole_slip_sum_micrometres_per_second,
            slip_denominator,
        )?
    };
    let components = [
        upright,
        height,
        pose,
        linear.max(angular),
        effort,
        target_rate,
        slip,
        i64::from(facts.fell) * 65_536,
    ];
    let total = components
        .iter()
        .zip(BIOMECHANICS_STANDING_REWARD_COEFFICIENTS_Q16)
        .try_fold(0_i64, |total, (component, coefficient)| {
            total
                .checked_add(round_shift_ties_even(
                    i128::from(*component)
                        .checked_mul(i128::from(coefficient))
                        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?,
                    16,
                )?)
                .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)
        })?;
    Ok((components, total))
}

pub fn biomechanics_forward_start_stop_reward_q16_v1(
    compiled: &CompiledBodySchemaV3,
    facts: &BiomechanicsForwardStartStopRewardFactsV1<'_>,
) -> Result<([i64; 11], i64), crate::TrainingEnvironmentError> {
    let width = compiled.base.actuator_definitions.len();
    if width != 23
        || facts.applied_targets_microradians.len() != width
        || facts.previous_applied_targets_microradians.len() != width
        || facts.contacting_sole_count > 2
    {
        return Err(crate::TrainingEnvironmentError::RewardFacts);
    }
    let (_, effort_normalization, target_rate_normalization) = standing_normalizations(compiled);
    let planar_error = facts.root_local_linear_velocity_micrometres_per_second[0]
        .saturating_sub(facts.command_raw[0])
        .unsigned_abs() as u128
        + facts.root_local_linear_velocity_micrometres_per_second[2]
            .saturating_sub(facts.command_raw[1])
            .unsigned_abs() as u128;
    let planar = square_q16(one_minus_q16(planar_error, 2_500_000)?)?;
    let yaw = square_q16(one_minus_q16(
        facts.root_local_angular_velocity_microradians_per_second[1]
            .saturating_sub(facts.command_raw[2])
            .unsigned_abs() as u128,
        1_500_000,
    )?)?;
    let [x, _, z, _] = facts.root_rotation_q1_30;
    let tilt = i128::from(x)
        .checked_mul(i128::from(x))
        .and_then(|value| {
            i128::from(z)
                .checked_mul(i128::from(z))
                .and_then(|other| value.checked_add(other))
        })
        .and_then(|value| value.checked_mul(2))
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let tilt_q30 = round_shift_ties_even(tilt, 30)?;
    let upright_q30 = (1_i64 << 30).saturating_sub(tilt_q30).clamp(0, 1_i64 << 30);
    let upright = ratio_q16(upright_q30 as u128, 1_u128 << 30)?;
    let height = one_minus_q16(
        facts
            .root_height_micrometres
            .saturating_sub(BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES)
            .unsigned_abs() as u128,
        400_000,
    )?;
    let vertical_velocity = ratio_q16(
        facts
            .root_vertical_velocity_micrometres_per_second
            .unsigned_abs() as u128,
        2_000_000,
    )?;
    let roll_pitch_rate = ratio_q16(
        facts.root_local_angular_velocity_microradians_per_second[0].unsigned_abs() as u128
            + facts.root_local_angular_velocity_microradians_per_second[2].unsigned_abs() as u128,
        4_000_000,
    )?;
    let effort = ratio_q16(
        facts.absolute_applied_effort_sum_micronewton_metres,
        effort_normalization as u128,
    )?;
    let target_rate = ratio_q16(
        facts
            .applied_targets_microradians
            .iter()
            .zip(facts.previous_applied_targets_microradians)
            .map(|(current, previous)| current.saturating_sub(*previous).unsigned_abs() as u128)
            .sum(),
        target_rate_normalization as u128,
    )?;
    let slip_denominator = u128::from(facts.contacting_sole_count) * 2_000_000;
    let slip = if slip_denominator == 0 {
        0
    } else {
        ratio_q16(
            facts.contacting_sole_slip_sum_micrometres_per_second,
            slip_denominator,
        )?
    };
    let moving = facts.command_raw != [0; 3];
    let support = i64::from(
        (moving && facts.contacting_sole_count == 1)
            || (!moving && facts.contacting_sole_count == 2),
    ) * 65_536;
    let components = [
        planar,
        yaw,
        upright,
        height,
        vertical_velocity,
        roll_pitch_rate,
        effort,
        target_rate,
        slip,
        support,
        i64::from(facts.fell) * 65_536,
    ];
    let total = components
        .iter()
        .zip(BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16)
        .try_fold(0_i64, |total, (component, coefficient)| {
            total
                .checked_add(round_shift_ties_even(
                    i128::from(*component)
                        .checked_mul(i128::from(coefficient))
                        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?,
                    16,
                )?)
                .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)
        })?;
    Ok((components, total))
}

pub fn biomechanics_forward_start_stop_reward_q16_v2(
    compiled: &CompiledBodySchemaV3,
    facts: &BiomechanicsForwardStartStopRewardFactsV1<'_>,
) -> Result<([i64; 11], i64), crate::TrainingEnvironmentError> {
    biomechanics_forward_start_stop_reward_q16_v2_or_v3(compiled, facts, false)
}

pub fn biomechanics_forward_start_stop_reward_q16_v3(
    compiled: &CompiledBodySchemaV3,
    facts: &BiomechanicsForwardStartStopRewardFactsV1<'_>,
) -> Result<([i64; 11], i64), crate::TrainingEnvironmentError> {
    biomechanics_forward_start_stop_reward_q16_v2_or_v3(compiled, facts, true)
}

fn biomechanics_forward_start_stop_reward_q16_v2_or_v3(
    compiled: &CompiledBodySchemaV3,
    facts: &BiomechanicsForwardStartStopRewardFactsV1<'_>,
    dense_tracking: bool,
) -> Result<([i64; 11], i64), crate::TrainingEnvironmentError> {
    let width = compiled.base.actuator_definitions.len();
    if width != 23
        || facts.applied_targets_microradians.len() != width
        || facts.previous_applied_targets_microradians.len() != width
        || facts.contacting_sole_count > 2
    {
        return Err(crate::TrainingEnvironmentError::RewardFacts);
    }
    let (_, effort_normalization, target_rate_normalization) = standing_normalizations(compiled);
    let planar_error = facts.root_local_linear_velocity_micrometres_per_second[0]
        .saturating_sub(facts.command_raw[0])
        .unsigned_abs() as u128
        + facts.root_local_linear_velocity_micrometres_per_second[2]
            .saturating_sub(facts.command_raw[1])
            .unsigned_abs() as u128;
    let yaw_error = facts.root_local_angular_velocity_microradians_per_second[1]
        .saturating_sub(facts.command_raw[2])
        .unsigned_abs() as u128;
    let (planar, yaw) = if dense_tracking {
        (
            dense_tracking_q16(planar_error, 500_000)?,
            dense_tracking_q16(yaw_error, 500_000)?,
        )
    } else {
        (
            square_q16(one_minus_q16(planar_error, 500_000)?)?,
            square_q16(one_minus_q16(yaw_error, 500_000)?)?,
        )
    };
    let [x, _, z, _] = facts.root_rotation_q1_30;
    let tilt = i128::from(x)
        .checked_mul(i128::from(x))
        .and_then(|value| {
            i128::from(z)
                .checked_mul(i128::from(z))
                .and_then(|other| value.checked_add(other))
        })
        .and_then(|value| value.checked_mul(2))
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let tilt_q30 = round_shift_ties_even(tilt, 30)?;
    let root_tilt_cost = ratio_q16(tilt_q30.clamp(0, 1_i64 << 30) as u128, 1_u128 << 30)?;
    let root_height_error_cost = ratio_q16(
        facts
            .root_height_micrometres
            .saturating_sub(BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES)
            .unsigned_abs() as u128,
        400_000,
    )?;
    let vertical_velocity = ratio_q16(
        facts
            .root_vertical_velocity_micrometres_per_second
            .unsigned_abs() as u128,
        2_000_000,
    )?;
    let roll_pitch_rate = ratio_q16(
        facts.root_local_angular_velocity_microradians_per_second[0].unsigned_abs() as u128
            + facts.root_local_angular_velocity_microradians_per_second[2].unsigned_abs() as u128,
        4_000_000,
    )?;
    let effort = ratio_q16(
        facts.absolute_applied_effort_sum_micronewton_metres,
        effort_normalization as u128,
    )?;
    let target_rate = ratio_q16(
        facts
            .applied_targets_microradians
            .iter()
            .zip(facts.previous_applied_targets_microradians)
            .map(|(current, previous)| current.saturating_sub(*previous).unsigned_abs() as u128)
            .sum(),
        target_rate_normalization as u128,
    )?;
    let slip_denominator = u128::from(facts.contacting_sole_count) * 2_000_000;
    let slip = if slip_denominator == 0 {
        0
    } else {
        ratio_q16(
            facts.contacting_sole_slip_sum_micrometres_per_second,
            slip_denominator,
        )?
    };
    let moving = facts.command_raw != [0; 3];
    let support = i64::from(
        (moving && facts.contacting_sole_count == 1)
            || (!moving && facts.contacting_sole_count == 2),
    ) * 65_536;
    let components = [
        planar,
        yaw,
        root_tilt_cost,
        root_height_error_cost,
        vertical_velocity,
        roll_pitch_rate,
        effort,
        target_rate,
        slip,
        support,
        i64::from(facts.fell) * 65_536,
    ];
    let total = components
        .iter()
        .zip(if dense_tracking {
            BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3
        } else {
            BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2
        })
        .try_fold(0_i64, |total, (component, coefficient)| {
            total
                .checked_add(round_shift_ties_even(
                    i128::from(*component)
                        .checked_mul(i128::from(coefficient))
                        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?,
                    16,
                )?)
                .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)
        })?;
    Ok((components, total))
}

#[must_use]
pub fn biomechanics_forward_start_stop_command_schedule_v2() -> Vec<[i64; 3]> {
    let maximum_delta = 1_000_000 / i64::from(STAGE0_MOTOR_HZ);
    let mut schedule =
        Vec::with_capacity(BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS as usize + 1);
    schedule.push([0; 3]);
    let mut forward = 0_i64;
    for tick in 1..=BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS {
        let target = if (120..BIOMECHANICS_FORWARD_START_STOP_RAMP_DOWN_TICK_V2).contains(&tick) {
            BIOMECHANICS_FORWARD_START_STOP_TARGET_MICROMETRES_PER_SECOND_V2
        } else {
            0
        };
        forward = if forward < target {
            forward.saturating_add(maximum_delta).min(target)
        } else {
            forward.saturating_sub(maximum_delta).max(target)
        };
        schedule.push([0, forward, 0]);
    }
    schedule
}

pub fn biomechanics_standing_environment_manifest_v1()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    biomechanics_standing_environment_manifest(
        &biomechanics_humanoid_body_schema_v2(),
        BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID_V1,
        "nextengine.motor.observation.humanoid-biomechanics-standing.v1",
        "nextengine.motor.action.humanoid-biomechanics-standing-residual.v1",
    )
}

pub fn biomechanics_standing_environment_manifest_v2()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    biomechanics_standing_environment_manifest(
        &biomechanics_humanoid_body_schema_v3(),
        BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID,
        BIOMECHANICS_STANDING_OBSERVATION_LAYOUT_ID,
        BIOMECHANICS_STANDING_ACTION_LAYOUT_ID,
    )
}

pub fn biomechanics_forward_start_stop_environment_manifest_v1()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    biomechanics_forward_start_stop_environment_manifest(
        &biomechanics_humanoid_body_schema_v3(),
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID,
        crate::curriculum_locomotion_stages_v2()
            .into_iter()
            .next()
            .expect("curriculum foundation stage is engine-owned")
            .command_profile
            .profile_hash()?,
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS,
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16,
        biomechanics_forward_start_stop_reward_profile_hash,
        None,
    )
}

pub fn biomechanics_forward_start_stop_environment_manifest_v2()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    biomechanics_forward_start_stop_environment_manifest(
        &schema,
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V2,
        biomechanics_forward_start_stop_command_profile_hash_v2(&compiled),
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V2,
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2,
        biomechanics_forward_start_stop_reward_profile_hash_v2,
        None,
    )
}

pub fn biomechanics_forward_start_stop_environment_manifest_v3()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    biomechanics_forward_start_stop_environment_manifest(
        &schema,
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V3,
        biomechanics_forward_start_stop_command_profile_hash_v2(&compiled),
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3,
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3,
        biomechanics_forward_start_stop_reward_profile_hash_v3,
        None,
    )
}

pub fn biomechanics_forward_start_stop_environment_manifest_v4()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    biomechanics_forward_start_stop_environment_manifest(
        &schema,
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V4,
        biomechanics_forward_start_stop_command_profile_hash_v2(&compiled),
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3,
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3,
        biomechanics_forward_start_stop_reward_profile_hash_v3,
        Some(65_536),
    )
}

pub fn biomechanics_forward_start_stop_environment_manifest_v5()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v4();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    biomechanics_forward_start_stop_environment_manifest(
        &schema,
        BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5,
        biomechanics_forward_start_stop_command_profile_hash_v2(&compiled),
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3,
        BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3,
        biomechanics_forward_start_stop_reward_profile_hash_v3,
        Some(BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5),
    )
}

pub fn biomechanics_forward_start_stop_environment_manifest_v6()
-> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let mut manifest = biomechanics_forward_start_stop_environment_manifest_v5()?;
    manifest.environment_id = id(BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V6);
    let mut observation = manifest.observation_layout_hash.as_bytes().to_vec();
    observation.extend_from_slice(
        b"append:quadrature-triangle-clock-q1.30;period=72;start=120;zero-command=off;v1",
    );
    manifest.observation_layout_hash = content_hash_from_bytes(sha256(&observation));
    let mut reward = manifest.reward_profile_hash.as_bytes().to_vec();
    reward.extend_from_slice(b"replace-support:periodic-load-transfer.v1;cycle=72;start=120;left-load-knots=0:32768,6:0,30:0,42:65536,66:65536,72:32768;credit=max(0,1-2*abs(actual-left-target-left))*(1-min(1,weighted-planar-l1-speed/1mps));no-impulse=0;stop=both-contact;coefficient=65536");
    manifest.reward_profile_hash = content_hash_from_bytes(sha256(&reward));
    manifest.reward_components[9].component_id = id(crate::WALKING_LOAD_REWARD_ID);
    manifest.reward_components[9].coefficient_q16 = 65_536;
    manifest.correspondence_profile_hash = domain_hash(
        "nextengine.motor.correspondence.biomechanics-forward-start-stop.v6-canonical-only",
        manifest.body_schema_hash,
    );
    manifest.validate_for_protocol_v2()?;
    Ok(manifest)
}

fn biomechanics_forward_start_stop_environment_manifest(
    schema: &next_contracts::body::BodySchemaV2,
    environment_profile_id: &str,
    command_profile_hash: ContentHash,
    reward_component_ids: [&str; 11],
    reward_coefficients_q16: [i64; 11],
    reward_profile_hash: fn(&CompiledBodySchemaV3) -> ContentHash,
    walking_residual_scale_multiplier_q16: Option<i64>,
) -> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let translation_invariant_reference = walking_residual_scale_multiplier_q16.is_some();
    let residual_scale_multiplier_q16 = walking_residual_scale_multiplier_q16.unwrap_or(65_536);
    let compiled = CompiledBodySchemaV3::compile(schema, PersistentId::from_bytes([0; 16]))?;
    let body_hash = compiled.base.body_schema_hash;
    let reward_components = reward_component_ids
        .into_iter()
        .zip(reward_coefficients_q16)
        .map(|(component_id, coefficient_q16)| MotorRewardComponentV1 {
            component_id: id(component_id),
            coefficient_q16,
            minimum_raw: 0,
            maximum_raw: 65_536,
        })
        .collect();
    let manifest = MotorTrainingEnvironmentManifestV2 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
        environment_id: id(environment_profile_id),
        body_schema_hash: body_hash,
        body_instance_projection_hash: domain_hash(
            "nextengine.body-instance.biomechanics-neutral.v1",
            body_hash,
        ),
        physics_catalog_hash: compiled.compiled_descriptor_hash,
        observation_layout_hash: biomechanics_standing_observation_layout_hash(
            &compiled,
            BIOMECHANICS_FORWARD_START_STOP_OBSERVATION_LAYOUT_ID,
        ),
        action_layout_hash: if translation_invariant_reference {
            biomechanics_walking_action_layout_hash(
                &compiled,
                if residual_scale_multiplier_q16 == 65_536 {
                    BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V2
                } else {
                    BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V3
                },
                residual_scale_multiplier_q16,
            )
        } else {
            biomechanics_standing_action_layout_hash(
                &compiled,
                BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID,
            )
        },
        physics_build_profile_hash: domain_hash(
            "nextengine.physx.build-profile.locked.v1",
            body_hash,
        ),
        scene_profile_hash: domain_hash(
            "nextengine.physx.scene.deterministic-humanoid.v1",
            body_hash,
        ),
        bridge_abi_hash: domain_hash("nextengine.physx.bridge-abi.v1", body_hash),
        quantization_profile_hash: domain_hash(
            "nextengine.physics.quantization.humanoid.v1",
            body_hash,
        ),
        translator_version_hash: domain_hash("nextengine.isaac.biomechanics-mirror.v2", body_hash),
        command_schedule_profile_hash: command_profile_hash,
        reward_profile_hash: reward_profile_hash(&compiled),
        termination_profile_hash: domain_hash(
            "nextengine.motor.termination.biomechanics-forward-start-stop.v1",
            body_hash,
        ),
        rng_derivation_profile_hash: domain_hash(
            "nextengine.motor.episode-seed-derivation.v1",
            body_hash,
        ),
        correspondence_profile_hash: domain_hash(
            if translation_invariant_reference {
                if residual_scale_multiplier_q16 == 65_536 {
                    "nextengine.motor.correspondence.biomechanics-forward-start-stop.v2"
                } else {
                    "nextengine.motor.correspondence.biomechanics-forward-start-stop.v3"
                }
            } else {
                "nextengine.motor.correspondence.biomechanics-forward-start-stop.v1"
            },
            body_hash,
        ),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: crate::training::MAX_CPU_VECTOR_SLOTS,
        maximum_episode_steps: BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS,
        reward_components,
    };
    manifest.validate()?;
    Ok(manifest)
}

fn biomechanics_standing_environment_manifest(
    schema: &next_contracts::body::BodySchemaV2,
    environment_profile_id: &str,
    observation_layout_id: &str,
    action_layout_id: &str,
) -> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let compiled = CompiledBodySchemaV3::compile(schema, PersistentId::from_bytes([0; 16]))?;
    let body_hash = compiled.base.body_schema_hash;
    let reward_components = BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS
        .into_iter()
        .zip(BIOMECHANICS_STANDING_REWARD_COEFFICIENTS_Q16)
        .map(|(component_id, coefficient_q16)| MotorRewardComponentV1 {
            component_id: id(component_id),
            coefficient_q16,
            minimum_raw: 0,
            maximum_raw: 65_536,
        })
        .collect();
    let manifest = MotorTrainingEnvironmentManifestV2 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
        environment_id: id(environment_profile_id),
        body_schema_hash: body_hash,
        body_instance_projection_hash: domain_hash(
            "nextengine.body-instance.biomechanics-neutral.v1",
            body_hash,
        ),
        physics_catalog_hash: compiled.compiled_descriptor_hash,
        observation_layout_hash: biomechanics_standing_observation_layout_hash(
            &compiled,
            observation_layout_id,
        ),
        action_layout_hash: biomechanics_standing_action_layout_hash(&compiled, action_layout_id),
        physics_build_profile_hash: domain_hash(
            "nextengine.physx.build-profile.locked.v1",
            body_hash,
        ),
        scene_profile_hash: domain_hash(
            "nextengine.physx.scene.deterministic-humanoid.v1",
            body_hash,
        ),
        bridge_abi_hash: domain_hash("nextengine.physx.bridge-abi.v1", body_hash),
        quantization_profile_hash: domain_hash(
            "nextengine.physics.quantization.humanoid.v1",
            body_hash,
        ),
        translator_version_hash: domain_hash("nextengine.isaac.biomechanics-mirror.v2", body_hash),
        command_schedule_profile_hash: biomechanics_standing_reference_profile_hash(&compiled),
        reward_profile_hash: biomechanics_standing_reward_profile_hash(&compiled),
        termination_profile_hash: domain_hash(
            if environment_profile_id == BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID {
                "nextengine.motor.termination.biomechanics-standing.v2"
            } else {
                "nextengine.motor.termination.biomechanics-standing.v1"
            },
            body_hash,
        ),
        rng_derivation_profile_hash: domain_hash(
            "nextengine.motor.episode-seed-derivation.v1",
            body_hash,
        ),
        correspondence_profile_hash: domain_hash(
            "nextengine.motor.correspondence.biomechanics-standing.v1",
            body_hash,
        ),
        physics_hz: STAGE0_PHYSICS_HZ,
        motor_hz: STAGE0_MOTOR_HZ,
        maximum_vector_slots: crate::training::MAX_CPU_VECTOR_SLOTS,
        maximum_episode_steps: BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS,
        reward_components,
    };
    manifest.validate()?;
    Ok(manifest)
}

pub fn biomechanics_standing_isaac_descriptor_json_v1() -> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v2();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_standing_environment_manifest_v1()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v2()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-standing.v1");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] = Value::Array(vec![profile_json(
        &manifest,
        &compiled,
        "nextengine.motor.observation.humanoid-biomechanics-standing.v1",
        "nextengine.motor.action.humanoid-biomechanics-standing-residual.v1",
    )]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_standing_isaac_descriptor_json_v2() -> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_standing_environment_manifest_v2()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v3()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-standing.v2");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] = Value::Array(vec![profile_json(
        &manifest,
        &compiled,
        BIOMECHANICS_STANDING_OBSERVATION_LAYOUT_ID,
        BIOMECHANICS_STANDING_ACTION_LAYOUT_ID,
    )]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_forward_start_stop_isaac_descriptor_json_v1()
-> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v1()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v3()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-forward-start-stop.v1");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] =
        Value::Array(vec![forward_profile_json(&manifest, &compiled)]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_forward_start_stop_isaac_descriptor_json_v2()
-> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v2()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v3()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-forward-start-stop.v2");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] = Value::Array(vec![forward_profile_json_v2_or_v3(
        &manifest,
        &compiled,
        "square(max(0, 1 - absolute_error / normalization)) in Q16",
    )]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_forward_start_stop_isaac_descriptor_json_v3()
-> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v3()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v3()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-forward-start-stop.v3");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] = Value::Array(vec![forward_profile_json_v2_or_v3(
        &manifest,
        &compiled,
        "square(1 / (1 + (absolute_error / normalization)^2)) in Q16",
    )]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_forward_start_stop_isaac_descriptor_json_v4()
-> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v4()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v3()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-forward-start-stop.v4");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] = Value::Array(vec![forward_profile_json_v4_or_v5(
        &manifest,
        &compiled,
        BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V2,
        65_536,
    )]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_forward_start_stop_isaac_descriptor_json_v5()
-> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v4();
    let compiled = CompiledBodySchemaV3::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v5()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_isaac_mirror_descriptor_json_v4()?)
            .expect("engine-generated biomechanics descriptor is valid JSON");
    descriptor["training_descriptor_id"] =
        json!("nextengine.isaac.humanoid-biomechanics-forward-start-stop.v5");
    descriptor["observation_width"] = json!(84);
    descriptor["environment_profiles"] = Value::Array(vec![forward_profile_json_v4_or_v5(
        &manifest,
        &compiled,
        BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5,
    )]);
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn biomechanics_forward_start_stop_canonical_descriptor_json_v6()
-> Result<String, MotorCompileError> {
    let compiled = CompiledBodySchemaV3::compile(
        &biomechanics_humanoid_body_schema_v4(),
        PersistentId::from_bytes([0; 16]),
    )?;
    let manifest = biomechanics_forward_start_stop_environment_manifest_v6()?;
    let mut descriptor: Value =
        serde_json::from_str(&biomechanics_forward_start_stop_isaac_descriptor_json_v5()?)
            .expect("engine-generated descriptor");
    let mut profile = forward_profile_json_v4_or_v5(
        &manifest,
        &compiled,
        BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V3,
        BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5,
    );
    profile["observation_layout_id"] =
        json!("nextengine.motor.observation.humanoid-biomechanics-periodic-walking.v1");
    profile["observation"]["channel_count"] = json!(86);
    profile["observation"]["appended_clock"] = json!({
        "offset": 84, "width": 2, "encoding": "quadrature-triangle-q1.30",
        "cycle_ticks": crate::WALKING_CYCLE_TICKS,
        "start_tick": crate::WALKING_PHASE_START_TICK, "zero_command": "off",
    });
    profile["periodic_load_credit"] = json!({
        "version": 1, "left_load_knots_q16": [[0,32768],[6,0],[30,0],[42,65536],[66,65536],[72,32768]],
        "impulse_measurement": "sum-absolute-world-Y-sole-ground-impulse-over-actual-substeps",
        "stance_speed": "target-load-weighted-world-XZ-L1-final-snapshot",
        "speed_normalization_um_s": 1_000_000,
        "flight_credit_q16": 0,
    });
    descriptor["training_descriptor_id"] =
        json!("nextengine.canonical.humanoid-biomechanics-forward-start-stop.v6");
    descriptor["backend_admission"] = json!("canonical-cpu-only;Isaac-mirror-not-implemented");
    descriptor["observation_width"] = json!(86);
    descriptor["environment_profiles"] = json!([profile]);
    let mut output = serde_json::to_string_pretty(&descriptor).expect("engine JSON");
    output.push('\n');
    Ok(output)
}

fn profile_json(
    manifest: &MotorTrainingEnvironmentManifestV2,
    compiled: &CompiledBodySchemaV3,
    observation_layout_id: &str,
    action_layout_id: &str,
) -> Value {
    let (pose_normalization, effort_normalization, target_rate_normalization) =
        standing_normalizations(compiled);
    json!({
        "profile_id": manifest.environment_id.as_str(),
        "manifest_hash": manifest.manifest_hash().expect("engine manifest is valid").to_hex(),
        "observation_layout_id": observation_layout_id,
        "observation_layout_hash": manifest.observation_layout_hash.to_hex(),
        "action_layout_id": action_layout_id,
        "action_layout_hash": manifest.action_layout_hash.to_hex(),
        "command_schedule_profile_hash": manifest.command_schedule_profile_hash.to_hex(),
        "reward_profile_hash": manifest.reward_profile_hash.to_hex(),
        "translator_version_hash": manifest.translator_version_hash.to_hex(),
        "termination_profile_hash": manifest.termination_profile_hash.to_hex(),
        "rng_derivation_profile_hash": manifest.rng_derivation_profile_hash.to_hex(),
        "correspondence_profile_hash": manifest.correspondence_profile_hash.to_hex(),
        "maximum_episode_steps": manifest.maximum_episode_steps,
        "velocity_frame": "world",
        "ground_half_extent_metres": 50,
        "command_profile": {"kind": "zero"},
        "standing_reference": {
            "profile_id": "nextengine.motor.procedural-standing.v1",
            "knee_target_microradians": PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS,
            "ankle_bias_microradians": PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS,
            "root_target_forward_micrometres": 0,
        },
        "observation": {
            "channel_count": 84,
            "root_quaternion": "engine world-from-root xyzw",
            "root_velocities": "world",
            "joint_order": "ordered actuator IDs",
            "previous_action": "post-safety applied target",
            "command": "exact zero",
            "contacts": ["contact.left-sole", "contact.right-sole"],
        },
        "action": {
            "channel_count": 23,
            "representation": "float policy value clamped to [-1,1], quantized to signed Q1.30 residual",
            "application": "procedural standing target plus actuator residual scale, then soft ROM, target slew and complete engine safety envelope",
        },
        "reward_normalizations": {
            "root_height_micrometres": 600_000,
            "joint_pose_soft_rom_span_sum_microradians": pose_normalization,
            "root_linear_l1_micrometres_per_second": 9_000_000,
            "root_angular_l1_microradians_per_second": 18_000_000,
            "applied_effort_per_motor_tick_micronewton_metres": effort_normalization,
            "applied_target_rate_microradians_per_motor_tick": target_rate_normalization,
            "contacting_sole_slip_micrometres_per_second": 4_000_000,
        },
        "reward_components": manifest.reward_components.iter().map(|value| json!({
            "component_id": value.component_id.as_str(),
            "coefficient_q16": value.coefficient_q16,
            "minimum_raw": value.minimum_raw,
            "maximum_raw": value.maximum_raw,
        })).collect::<Vec<_>>(),
        "termination": {
            "pelvis_height_micrometres_inclusive": BIOMECHANICS_FALL_HEIGHT_MICROMETRES,
            "root_tilt_degrees_inclusive": 60,
            "world_bound_micrometres_inclusive": BIOMECHANICS_WORLD_BOUND_MICROMETRES,
            "timeout_ticks": BIOMECHANICS_STANDING_MAXIMUM_EPISODE_STEPS,
            "hard_rom_or_joint_safety": "terminate",
            "hard_impact_or_self_collision": "terminate",
        },
        "target_root_height_micrometres": BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
    })
}

fn forward_profile_json(
    manifest: &MotorTrainingEnvironmentManifestV2,
    compiled: &CompiledBodySchemaV3,
) -> Value {
    let (_, effort_normalization, target_rate_normalization) = standing_normalizations(compiled);
    let command = crate::curriculum_locomotion_stages_v2()
        .into_iter()
        .next()
        .expect("curriculum foundation stage is engine-owned")
        .command_profile;
    json!({
        "profile_id": manifest.environment_id.as_str(),
        "manifest_hash": manifest.manifest_hash().expect("engine manifest is valid").to_hex(),
        "observation_layout_id": BIOMECHANICS_FORWARD_START_STOP_OBSERVATION_LAYOUT_ID,
        "observation_layout_hash": manifest.observation_layout_hash.to_hex(),
        "action_layout_id": BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID,
        "action_layout_hash": manifest.action_layout_hash.to_hex(),
        "command_schedule_profile_hash": manifest.command_schedule_profile_hash.to_hex(),
        "reward_profile_hash": manifest.reward_profile_hash.to_hex(),
        "translator_version_hash": manifest.translator_version_hash.to_hex(),
        "termination_profile_hash": manifest.termination_profile_hash.to_hex(),
        "rng_derivation_profile_hash": manifest.rng_derivation_profile_hash.to_hex(),
        "correspondence_profile_hash": manifest.correspondence_profile_hash.to_hex(),
        "maximum_episode_steps": manifest.maximum_episode_steps,
        "velocity_frame": "root-local",
        "ground_half_extent_metres": 50,
        "command_profile": {
            "kind": "sha256-counter-foundation-v1",
            "profile_id": command.profile_id.as_str(),
            "warmup_ticks": command.warmup_ticks,
            "segment_ticks": command.segment_ticks,
            "episode_ticks": command.episode_ticks,
            "mode_weights_basis_points": command.mode_weights_basis_points,
            "right_velocity_micrometres_per_second": [command.right_velocity_min_micrometres_per_second, command.right_velocity_max_micrometres_per_second],
            "forward_velocity_micrometres_per_second": [command.forward_velocity_min_micrometres_per_second, command.forward_velocity_max_micrometres_per_second],
            "yaw_rate_microradians_per_second": [command.yaw_rate_min_microradians_per_second, command.yaw_rate_max_microradians_per_second],
            "linear_rate_limit_micrometres_per_second_squared": command.linear_rate_limit_micrometres_per_second_squared,
            "yaw_rate_limit_microradians_per_second_squared": command.yaw_rate_limit_microradians_per_second_squared,
        },
        "standing_reference": {
            "profile_id": "nextengine.motor.procedural-standing.v1",
            "knee_target_microradians": PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS,
            "ankle_bias_microradians": PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS,
            "root_target_forward_micrometres": 0,
        },
        "observation": {
            "channel_count": 84,
            "root_quaternion": "engine world-from-root xyzw",
            "root_velocities": "root-local",
            "joint_order": "ordered actuator IDs",
            "previous_action": "post-safety applied target",
            "command": "local-right, local-forward, yaw-rate",
            "contacts": ["contact.left-sole", "contact.right-sole"],
        },
        "action": {
            "channel_count": 23,
            "representation": "float policy value clamped to [-1,1], quantized to signed Q1.30 residual",
            "application": "procedural standing target plus actuator residual scale, then soft ROM, target slew and complete engine safety envelope",
        },
        "reward_normalizations": {
            "planar_tracking_micrometres_per_second": 2_500_000,
            "yaw_tracking_microradians_per_second": 1_500_000,
            "root_height_micrometres": 400_000,
            "vertical_velocity_micrometres_per_second": 2_000_000,
            "roll_pitch_rate_microradians_per_second": 4_000_000,
            "applied_effort_per_motor_tick_micronewton_metres": effort_normalization,
            "applied_target_rate_microradians_per_motor_tick": target_rate_normalization,
            "contacting_sole_slip_micrometres_per_second": 2_000_000,
        },
        "reward_components": manifest.reward_components.iter().map(|value| json!({
            "component_id": value.component_id.as_str(),
            "coefficient_q16": value.coefficient_q16,
            "minimum_raw": value.minimum_raw,
            "maximum_raw": value.maximum_raw,
        })).collect::<Vec<_>>(),
        "termination": {
            "pelvis_height_micrometres_inclusive": BIOMECHANICS_FALL_HEIGHT_MICROMETRES,
            "root_tilt_degrees_inclusive": 60,
            "world_bound_micrometres_inclusive": BIOMECHANICS_WORLD_BOUND_MICROMETRES,
            "timeout_ticks": BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS,
            "hard_rom_or_joint_safety": "terminate",
            "hard_impact_or_self_collision": "terminate",
        },
        "target_root_height_micrometres": BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
    })
}

fn forward_profile_json_v2_or_v3(
    manifest: &MotorTrainingEnvironmentManifestV2,
    compiled: &CompiledBodySchemaV3,
    tracking_kernel: &str,
) -> Value {
    let (_, effort_normalization, target_rate_normalization) = standing_normalizations(compiled);
    json!({
        "profile_id": manifest.environment_id.as_str(),
        "manifest_hash": manifest.manifest_hash().expect("engine manifest is valid").to_hex(),
        "observation_layout_id": BIOMECHANICS_FORWARD_START_STOP_OBSERVATION_LAYOUT_ID,
        "observation_layout_hash": manifest.observation_layout_hash.to_hex(),
        "action_layout_id": BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID,
        "action_layout_hash": manifest.action_layout_hash.to_hex(),
        "command_schedule_profile_hash": manifest.command_schedule_profile_hash.to_hex(),
        "reward_profile_hash": manifest.reward_profile_hash.to_hex(),
        "translator_version_hash": manifest.translator_version_hash.to_hex(),
        "termination_profile_hash": manifest.termination_profile_hash.to_hex(),
        "rng_derivation_profile_hash": manifest.rng_derivation_profile_hash.to_hex(),
        "correspondence_profile_hash": manifest.correspondence_profile_hash.to_hex(),
        "maximum_episode_steps": manifest.maximum_episode_steps,
        "velocity_frame": "root-local",
        "ground_half_extent_metres": 50,
        "command_profile": {
            "kind": "fixed-forward-start-stop-v2",
            "profile_id": "nextengine.motor.command.biomechanics-forward-start-stop.v2",
            "warmup_ticks": 120,
            "ramp_down_tick": BIOMECHANICS_FORWARD_START_STOP_RAMP_DOWN_TICK_V2,
            "final_zero_ticks": BIOMECHANICS_FORWARD_START_STOP_FINAL_ZERO_TICKS_V2,
            "episode_ticks": BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS,
            "forward_velocity_micrometres_per_second": BIOMECHANICS_FORWARD_START_STOP_TARGET_MICROMETRES_PER_SECOND_V2,
            "linear_rate_limit_micrometres_per_second_squared": 1_000_000,
        },
        "standing_reference": {
            "profile_id": "nextengine.motor.procedural-standing.v1",
            "knee_target_microradians": PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS,
            "ankle_bias_microradians": PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS,
            "root_target_forward_micrometres": 0,
        },
        "observation": {
            "channel_count": 84,
            "root_quaternion": "engine world-from-root xyzw",
            "root_velocities": "root-local",
            "joint_order": "ordered actuator IDs",
            "previous_action": "post-safety applied target",
            "command": "local-right, local-forward, yaw-rate",
            "contacts": ["contact.left-sole", "contact.right-sole"],
        },
        "action": {
            "channel_count": 23,
            "representation": "float policy value clamped to [-1,1], quantized to signed Q1.30 residual",
            "application": "procedural standing target plus actuator residual scale, then soft ROM, target slew and complete engine safety envelope",
        },
        "reward_normalizations": {
            "planar_tracking_micrometres_per_second": 500_000,
            "yaw_tracking_microradians_per_second": 500_000,
            "tracking_kernel": tracking_kernel,
            "root_height_micrometres": 400_000,
            "vertical_velocity_micrometres_per_second": 2_000_000,
            "roll_pitch_rate_microradians_per_second": 4_000_000,
            "applied_effort_per_motor_tick_micronewton_metres": effort_normalization,
            "applied_target_rate_microradians_per_motor_tick": target_rate_normalization,
            "contacting_sole_slip_micrometres_per_second": 2_000_000,
        },
        "reward_components": manifest.reward_components.iter().map(|value| json!({
            "component_id": value.component_id.as_str(),
            "coefficient_q16": value.coefficient_q16,
            "minimum_raw": value.minimum_raw,
            "maximum_raw": value.maximum_raw,
        })).collect::<Vec<_>>(),
        "termination": {
            "pelvis_height_micrometres_inclusive": BIOMECHANICS_FALL_HEIGHT_MICROMETRES,
            "root_tilt_degrees_inclusive": 60,
            "world_bound_micrometres_inclusive": BIOMECHANICS_WORLD_BOUND_MICROMETRES,
            "timeout_ticks": BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS,
            "hard_rom_or_joint_safety": "terminate",
            "hard_impact_or_self_collision": "terminate",
        },
        "target_root_height_micrometres": BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
    })
}

fn forward_profile_json_v4_or_v5(
    manifest: &MotorTrainingEnvironmentManifestV2,
    compiled: &CompiledBodySchemaV3,
    action_layout_id: &str,
    residual_scale_multiplier_q16: i64,
) -> Value {
    let mut profile = forward_profile_json_v2_or_v3(
        manifest,
        compiled,
        "square(1 / (1 + (absolute_error / normalization)^2)) in Q16",
    );
    profile["action_layout_id"] = json!(action_layout_id);
    profile
        .as_object_mut()
        .expect("engine profile is a JSON object")
        .remove("standing_reference");
    profile["walking_reference"] = json!({
        "profile_id": PROCEDURAL_WALKING_REFERENCE_PROFILE_ID_V1,
        "knee_target_microradians": PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS,
        "ankle_bias_microradians": PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS,
        "root_forward_position_feedback": "disabled",
        "root_pitch_rate_and_forward_velocity_feedback": "enabled",
    });
    profile["action"]["application"] = json!(
        "translation-invariant procedural walking reference plus actuator residual scale, then soft ROM, target slew and complete engine safety envelope"
    );
    profile["action"]["residual_scale_multiplier_q16"] = json!(residual_scale_multiplier_q16);
    profile
}

fn standing_normalizations(compiled: &CompiledBodySchemaV3) -> (i64, u64, u64) {
    let joint_by_id = compiled
        .base
        .physics_descriptors
        .joints
        .iter()
        .map(|joint| (&joint.base.joint_id, joint))
        .collect::<std::collections::BTreeMap<_, _>>();
    let pose_normalization = compiled
        .base
        .actuator_definitions
        .iter()
        .map(|actuator| {
            let joint = joint_by_id[&actuator.joint_id];
            joint
                .soft_limit_max_microradians
                .checked_sub(joint.soft_limit_min_microradians)
                .expect("validated soft ROM is ordered")
        })
        .sum::<i64>();
    let effort_normalization = compiled
        .base
        .physics_descriptors
        .actuators
        .iter()
        .map(|actuator| {
            actuator
                .minimum_effort_micronewton_metres
                .unsigned_abs()
                .max(actuator.maximum_effort_micronewton_metres.unsigned_abs())
        })
        .sum::<u64>()
        * 4;
    let target_rate_normalization = compiled
        .base
        .physics_descriptors
        .actuators
        .iter()
        .map(|actuator| {
            actuator
                .minimum_target_delta_microradians_per_motor_tick
                .unsigned_abs()
                .max(
                    actuator
                        .maximum_target_delta_microradians_per_motor_tick
                        .unsigned_abs(),
                )
        })
        .sum::<u64>();
    (
        pose_normalization,
        effort_normalization,
        target_rate_normalization,
    )
}

fn biomechanics_standing_observation_layout_hash(
    compiled: &CompiledBodySchemaV3,
    observation_layout_id: &str,
) -> ContentHash {
    let mut bytes = domain_preimage(observation_layout_id, compiled.base.body_schema_hash);
    bytes.extend_from_slice(&84_u32.to_le_bytes());
    for actuator in &compiled.base.actuator_definitions {
        push_text(&mut bytes, actuator.actuator_id.as_str());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_standing_action_layout_hash(
    compiled: &CompiledBodySchemaV3,
    action_layout_id: &str,
) -> ContentHash {
    let mut bytes = domain_preimage(action_layout_id, compiled.base.body_schema_hash);
    for actuator in &compiled.base.actuator_definitions {
        push_text(&mut bytes, actuator.actuator_id.as_str());
        bytes.extend_from_slice(&actuator.residual_scale_microradians.to_le_bytes());
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_walking_action_layout_hash(
    compiled: &CompiledBodySchemaV3,
    action_layout_id: &str,
    residual_scale_multiplier_q16: i64,
) -> ContentHash {
    let mut bytes = domain_preimage(action_layout_id, compiled.base.body_schema_hash);
    for actuator in &compiled.base.actuator_definitions {
        push_text(&mut bytes, actuator.actuator_id.as_str());
        bytes.extend_from_slice(&actuator.residual_scale_microradians.to_le_bytes());
    }
    bytes.extend_from_slice(&residual_scale_multiplier_q16.to_le_bytes());
    bytes.extend_from_slice(biomechanics_walking_reference_profile_hash(compiled).as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_standing_reference_profile_hash(compiled: &CompiledBodySchemaV3) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.procedural-standing.v1",
        compiled.base.body_schema_hash,
    );
    bytes.extend_from_slice(&PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS.to_le_bytes());
    bytes.extend_from_slice(&PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS.to_le_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_walking_reference_profile_hash(compiled: &CompiledBodySchemaV3) -> ContentHash {
    let mut bytes = domain_preimage(
        PROCEDURAL_WALKING_REFERENCE_PROFILE_ID_V1,
        compiled.base.body_schema_hash,
    );
    bytes.extend_from_slice(&PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS.to_le_bytes());
    bytes.extend_from_slice(&PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS.to_le_bytes());
    bytes.extend_from_slice(b"root-forward-position-feedback-disabled\0");
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_standing_reward_profile_hash(compiled: &CompiledBodySchemaV3) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.reward.biomechanics-standing.v1",
        compiled.base.body_schema_hash,
    );
    for (component, coefficient) in BIOMECHANICS_STANDING_REWARD_COMPONENT_IDS
        .into_iter()
        .zip(BIOMECHANICS_STANDING_REWARD_COEFFICIENTS_Q16)
    {
        push_text(&mut bytes, component);
        bytes.extend_from_slice(&coefficient.to_le_bytes());
    }
    let (pose, effort, target_rate) = standing_normalizations(compiled);
    for normalization in [
        BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
        600_000,
        pose,
        9_000_000,
        18_000_000,
        i64::try_from(effort).expect("validated effort normalization fits i64"),
        i64::try_from(target_rate).expect("validated target-rate normalization fits i64"),
        4_000_000,
    ] {
        bytes.extend_from_slice(&normalization.to_le_bytes());
    }
    bytes.extend_from_slice(compiled.compiled_descriptor_hash.as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_forward_start_stop_reward_profile_hash(
    compiled: &CompiledBodySchemaV3,
) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.reward.biomechanics-forward-start-stop.v1",
        compiled.base.body_schema_hash,
    );
    for (component, coefficient) in BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS
        .into_iter()
        .zip(BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16)
    {
        push_text(&mut bytes, component);
        bytes.extend_from_slice(&coefficient.to_le_bytes());
    }
    let (_, effort, target_rate) = standing_normalizations(compiled);
    for normalization in [
        BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
        2_500_000,
        1_500_000,
        400_000,
        2_000_000,
        4_000_000,
        i64::try_from(effort).expect("validated effort normalization fits i64"),
        i64::try_from(target_rate).expect("validated target-rate normalization fits i64"),
        2_000_000,
    ] {
        bytes.extend_from_slice(&normalization.to_le_bytes());
    }
    bytes.extend_from_slice(compiled.compiled_descriptor_hash.as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_forward_start_stop_reward_profile_hash_v2(
    compiled: &CompiledBodySchemaV3,
) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.reward.biomechanics-forward-start-stop.v2",
        compiled.base.body_schema_hash,
    );
    for (component, coefficient) in BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V2
        .into_iter()
        .zip(BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V2)
    {
        push_text(&mut bytes, component);
        bytes.extend_from_slice(&coefficient.to_le_bytes());
    }
    let (_, effort, target_rate) = standing_normalizations(compiled);
    for normalization in [
        BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
        500_000,
        500_000,
        400_000,
        2_000_000,
        4_000_000,
        i64::try_from(effort).expect("validated effort normalization fits i64"),
        i64::try_from(target_rate).expect("validated target-rate normalization fits i64"),
        2_000_000,
    ] {
        bytes.extend_from_slice(&normalization.to_le_bytes());
    }
    bytes.extend_from_slice(compiled.compiled_descriptor_hash.as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_forward_start_stop_reward_profile_hash_v3(
    compiled: &CompiledBodySchemaV3,
) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.reward.biomechanics-forward-start-stop.v3",
        compiled.base.body_schema_hash,
    );
    for (component, coefficient) in BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS_V3
        .into_iter()
        .zip(BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16_V3)
    {
        push_text(&mut bytes, component);
        bytes.extend_from_slice(&coefficient.to_le_bytes());
    }
    let (_, effort, target_rate) = standing_normalizations(compiled);
    for normalization in [
        BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
        500_000,
        500_000,
        400_000,
        2_000_000,
        4_000_000,
        i64::try_from(effort).expect("validated effort normalization fits i64"),
        i64::try_from(target_rate).expect("validated target-rate normalization fits i64"),
        2_000_000,
    ] {
        bytes.extend_from_slice(&normalization.to_le_bytes());
    }
    push_text(
        &mut bytes,
        "square(1 / (1 + (absolute_error / normalization)^2)) in Q16",
    );
    bytes.extend_from_slice(compiled.compiled_descriptor_hash.as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn biomechanics_forward_start_stop_command_profile_hash_v2(
    compiled: &CompiledBodySchemaV3,
) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.command.biomechanics-forward-start-stop.v2",
        compiled.base.body_schema_hash,
    );
    for value in [
        120_i64,
        BIOMECHANICS_FORWARD_START_STOP_RAMP_DOWN_TICK_V2 as i64,
        BIOMECHANICS_FORWARD_START_STOP_FINAL_ZERO_TICKS_V2 as i64,
        BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS as i64,
        BIOMECHANICS_FORWARD_START_STOP_TARGET_MICROMETRES_PER_SECOND_V2,
        1_000_000,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for command in biomechanics_forward_start_stop_command_schedule_v2() {
        for value in command {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    content_hash_from_bytes(sha256(&bytes))
}

fn domain_hash(domain: &str, body_schema_hash: ContentHash) -> ContentHash {
    content_hash_from_bytes(sha256(&domain_preimage(domain, body_schema_hash)))
}

fn domain_preimage(domain: &str, body_schema_hash: ContentHash) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(body_schema_hash.as_bytes());
    bytes
}

fn push_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn dense_tracking_q16(
    error: u128,
    normalization: u128,
) -> Result<i64, crate::TrainingEnvironmentError> {
    let maximum_error = normalization
        .checked_mul(4_096)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let bounded_error = error.min(maximum_error);
    let normalization_squared = normalization
        .checked_mul(normalization)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let error_squared = bounded_error
        .checked_mul(bounded_error)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let denominator = normalization_squared
        .checked_add(error_squared)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    square_q16(ratio_q16(normalization_squared, denominator)?)
}

fn one_minus_q16(value: u128, maximum: u128) -> Result<i64, crate::TrainingEnvironmentError> {
    Ok(65_536_i64.saturating_sub(ratio_q16(value, maximum)?))
}

fn ratio_q16(value: u128, maximum: u128) -> Result<i64, crate::TrainingEnvironmentError> {
    if maximum == 0 {
        return Err(crate::TrainingEnvironmentError::ArithmeticOverflow);
    }
    let bounded = value.min(maximum);
    let numerator = bounded
        .checked_mul(65_536)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let quotient = numerator / maximum;
    let remainder = numerator % maximum;
    let twice_remainder = remainder
        .checked_mul(2)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let rounded = quotient
        + u128::from(
            twice_remainder > maximum || (twice_remainder == maximum && quotient % 2 == 1),
        );
    i64::try_from(rounded).map_err(|_| crate::TrainingEnvironmentError::ArithmeticOverflow)
}

fn round_shift_ties_even(value: i128, shift: u32) -> Result<i64, crate::TrainingEnvironmentError> {
    let denominator = 1_i128
        .checked_shl(shift)
        .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?;
    let quotient = value / denominator;
    let remainder = (value % denominator).unsigned_abs();
    let half = (denominator / 2) as u128;
    let adjust = remainder > half || (remainder == half && quotient.unsigned_abs() % 2 == 1);
    let rounded = if adjust {
        quotient
            .checked_add(if value.is_negative() { -1 } else { 1 })
            .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?
    } else {
        quotient
    };
    i64::try_from(rounded).map_err(|_| crate::TrainingEnvironmentError::ArithmeticOverflow)
}

fn square_q16(value: i64) -> Result<i64, crate::TrainingEnvironmentError> {
    round_shift_ties_even(
        i128::from(value)
            .checked_mul(i128::from(value))
            .ok_or(crate::TrainingEnvironmentError::ArithmeticOverflow)?,
        16,
    )
}

fn id(value: &str) -> SchemaId {
    SchemaId::new(value).expect("biomechanics standing IDs are compile-time validated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_manifest_and_training_descriptor_bind_current_biomechanics() {
        let manifest = biomechanics_standing_environment_manifest_v2().expect("manifest");
        assert_eq!(
            manifest.environment_id.as_str(),
            BIOMECHANICS_STANDING_ENVIRONMENT_PROFILE_ID
        );
        assert_eq!(manifest.maximum_episode_steps, 3_600);
        assert_eq!(manifest.reward_components.len(), 8);
        assert_eq!(manifest.reward_components[0].minimum_raw, 0);
        assert_eq!(manifest.reward_components[0].maximum_raw, 65_536);

        let descriptor: Value = serde_json::from_str(
            &biomechanics_standing_isaac_descriptor_json_v2().expect("descriptor"),
        )
        .expect("valid JSON");
        assert_eq!(
            descriptor["translator_id"],
            "nextengine.isaac.biomechanics-mirror.v2"
        );
        assert_eq!(descriptor["body_schema_id"], schema_id());
        assert_eq!(descriptor["observation_width"], 84);
        assert_eq!(
            descriptor["environment_profiles"][0]["manifest_hash"],
            manifest.manifest_hash().expect("hash").to_hex()
        );
        assert_eq!(
            descriptor["environment_profiles"][0]["reward_normalizations"]["joint_pose_soft_rom_span_sum_microradians"],
            42_847_831_i64
        );
    }

    #[test]
    fn forward_start_stop_descriptor_binds_only_the_foundation_command_stage() {
        let manifest = biomechanics_forward_start_stop_environment_manifest_v1()
            .expect("forward start/stop manifest");
        assert_eq!(
            manifest.environment_id.as_str(),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID
        );
        assert_eq!(manifest.maximum_episode_steps, 1_200);
        assert_eq!(manifest.reward_components.len(), 11);
        let descriptor: Value = serde_json::from_str(
            &biomechanics_forward_start_stop_isaac_descriptor_json_v1()
                .expect("forward descriptor"),
        )
        .expect("valid JSON");
        let profile = &descriptor["environment_profiles"][0];
        assert_eq!(profile["velocity_frame"], "root-local");
        assert_eq!(profile["command_profile"]["warmup_ticks"], 120);
        assert_eq!(
            profile["command_profile"]["forward_velocity_micrometres_per_second"],
            json!([0, 750_000])
        );
        assert_eq!(
            profile["command_profile"]["right_velocity_micrometres_per_second"],
            json!([0, 0])
        );
        assert_eq!(
            profile["command_profile"]["yaw_rate_microradians_per_second"],
            json!([0, 0])
        );
    }

    #[test]
    fn forward_start_stop_reward_has_exact_stationary_support_best_case() {
        let compiled = CompiledBodySchemaV3::compile(
            &biomechanics_humanoid_body_schema_v3(),
            PersistentId::from_bytes([0; 16]),
        )
        .expect("compiled biomechanics");
        let targets = vec![0; compiled.base.actuator_definitions.len()];
        let facts = BiomechanicsForwardStartStopRewardFactsV1 {
            root_rotation_q1_30: [0, 0, 0, 1_i64 << 30],
            root_height_micrometres: BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
            root_vertical_velocity_micrometres_per_second: 0,
            root_local_linear_velocity_micrometres_per_second: [0; 3],
            root_local_angular_velocity_microradians_per_second: [0; 3],
            command_raw: [0; 3],
            absolute_applied_effort_sum_micronewton_metres: 0,
            applied_targets_microradians: &targets,
            previous_applied_targets_microradians: &targets,
            contacting_sole_slip_sum_micrometres_per_second: 0,
            contacting_sole_count: 2,
            fell: false,
        };
        let (components, total) = biomechanics_forward_start_stop_reward_q16_v1(&compiled, &facts)
            .expect("bounded walking reward");
        assert_eq!(
            components,
            [65_536, 65_536, 65_536, 65_536, 0, 0, 0, 0, 0, 65_536, 0]
        );
        assert_eq!(total, 278_528);
    }

    #[test]
    fn forward_start_stop_v2_rejects_stationary_motion_and_closes_stop_schedule() {
        let compiled = CompiledBodySchemaV3::compile(
            &biomechanics_humanoid_body_schema_v3(),
            PersistentId::from_bytes([0; 16]),
        )
        .expect("compiled biomechanics");
        let targets = vec![0; compiled.base.actuator_definitions.len()];
        let mut facts = BiomechanicsForwardStartStopRewardFactsV1 {
            root_rotation_q1_30: [0, 0, 0, 1_i64 << 30],
            root_height_micrometres: BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
            root_vertical_velocity_micrometres_per_second: 0,
            root_local_linear_velocity_micrometres_per_second: [0; 3],
            root_local_angular_velocity_microradians_per_second: [0; 3],
            command_raw: [0, 500_000, 0],
            absolute_applied_effort_sum_micronewton_metres: 0,
            applied_targets_microradians: &targets,
            previous_applied_targets_microradians: &targets,
            contacting_sole_slip_sum_micrometres_per_second: 0,
            contacting_sole_count: 2,
            fell: false,
        };
        let (stationary_components, stationary_total) =
            biomechanics_forward_start_stop_reward_q16_v2(&compiled, &facts)
                .expect("bounded walking reward");
        assert_eq!(stationary_components[0], 0);
        assert_eq!(stationary_components[9], 0);
        assert_eq!(stationary_total, 16_384);

        facts.root_local_linear_velocity_micrometres_per_second[2] = 500_000;
        facts.contacting_sole_count = 1;
        let (tracking_components, tracking_total) =
            biomechanics_forward_start_stop_reward_q16_v2(&compiled, &facts)
                .expect("bounded walking reward");
        assert_eq!(tracking_components[0], 65_536);
        assert_eq!(tracking_components[9], 65_536);
        assert_eq!(tracking_total, 163_840);
        assert!(stationary_total * 10 <= tracking_total);

        let schedule = biomechanics_forward_start_stop_command_schedule_v2();
        assert_eq!(schedule.len(), 1_201);
        assert!(schedule[1_021..].iter().all(|command| *command == [0; 3]));
        assert_eq!(schedule[1_020], [0, 20, 0]);
        let commanded_forward_raw: i64 = schedule[1..].iter().map(|command| command[1]).sum();
        assert!(commanded_forward_raw >= 3 * 60 * 1_000_000);

        let manifest = biomechanics_forward_start_stop_environment_manifest_v2()
            .expect("forward start/stop v2 manifest");
        assert_eq!(
            manifest.environment_id.as_str(),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V2
        );
        let descriptor: Value = serde_json::from_str(
            &biomechanics_forward_start_stop_isaac_descriptor_json_v2()
                .expect("forward v2 descriptor"),
        )
        .expect("valid JSON");
        assert_eq!(
            descriptor["environment_profiles"][0]["command_profile"]["final_zero_ticks"],
            180
        );
    }

    #[test]
    fn forward_start_stop_v3_has_dense_monotonic_tracking_signal() {
        let compiled = CompiledBodySchemaV3::compile(
            &biomechanics_humanoid_body_schema_v3(),
            PersistentId::from_bytes([0; 16]),
        )
        .expect("compiled biomechanics");
        let targets = vec![0; compiled.base.actuator_definitions.len()];
        let mut facts = BiomechanicsForwardStartStopRewardFactsV1 {
            root_rotation_q1_30: [0, 0, 0, 1_i64 << 30],
            root_height_micrometres: BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
            root_vertical_velocity_micrometres_per_second: 0,
            root_local_linear_velocity_micrometres_per_second: [0; 3],
            root_local_angular_velocity_microradians_per_second: [0; 3],
            command_raw: [0, 500_000, 0],
            absolute_applied_effort_sum_micronewton_metres: 0,
            applied_targets_microradians: &targets,
            previous_applied_targets_microradians: &targets,
            contacting_sole_slip_sum_micrometres_per_second: 0,
            contacting_sole_count: 2,
            fell: false,
        };
        let (stationary, stationary_total) =
            biomechanics_forward_start_stop_reward_q16_v3(&compiled, &facts)
                .expect("dense walking reward");
        assert_eq!(stationary[0], 16_384);
        assert_eq!(stationary_total, 49_152);

        facts.root_local_linear_velocity_micrometres_per_second[2] = 250_000;
        let (halfway, _) = biomechanics_forward_start_stop_reward_q16_v3(&compiled, &facts)
            .expect("dense walking reward");
        assert_eq!(halfway[0], 41_943);
        facts.root_local_linear_velocity_micrometres_per_second[2] = 500_000;
        facts.contacting_sole_count = 1;
        let (tracking, tracking_total) =
            biomechanics_forward_start_stop_reward_q16_v3(&compiled, &facts)
                .expect("dense walking reward");
        assert!(stationary[0] < halfway[0] && halfway[0] < tracking[0]);
        assert_eq!(tracking[0], 65_536);
        assert_eq!(tracking_total, 163_840);

        let manifest = biomechanics_forward_start_stop_environment_manifest_v3()
            .expect("forward start/stop v3 manifest");
        assert_eq!(
            manifest.environment_id.as_str(),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V3
        );
        let descriptor: Value = serde_json::from_str(
            &biomechanics_forward_start_stop_isaac_descriptor_json_v3()
                .expect("forward v3 descriptor"),
        )
        .expect("valid JSON");
        assert_eq!(
            descriptor["environment_profiles"][0]["reward_normalizations"]["tracking_kernel"],
            "square(1 / (1 + (absolute_error / normalization)^2)) in Q16"
        );
    }

    #[test]
    fn forward_start_stop_v4_closes_translation_invariant_action_reference() {
        let v3 = biomechanics_forward_start_stop_environment_manifest_v3()
            .expect("forward start/stop v3 manifest");
        let v4 = biomechanics_forward_start_stop_environment_manifest_v4()
            .expect("forward start/stop v4 manifest");
        assert_eq!(
            v4.environment_id.as_str(),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V4
        );
        assert_eq!(v4.observation_layout_hash, v3.observation_layout_hash);
        assert_eq!(
            v4.command_schedule_profile_hash,
            v3.command_schedule_profile_hash
        );
        assert_eq!(v4.reward_profile_hash, v3.reward_profile_hash);
        assert_ne!(v4.action_layout_hash, v3.action_layout_hash);
        assert_ne!(
            v4.correspondence_profile_hash,
            v3.correspondence_profile_hash
        );

        let descriptor: Value = serde_json::from_str(
            &biomechanics_forward_start_stop_isaac_descriptor_json_v4()
                .expect("forward v4 descriptor"),
        )
        .expect("valid JSON");
        let profile = &descriptor["environment_profiles"][0];
        assert_eq!(
            profile["action_layout_id"],
            BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V2
        );
        assert!(profile.get("standing_reference").is_none());
        assert_eq!(
            profile["walking_reference"]["profile_id"],
            PROCEDURAL_WALKING_REFERENCE_PROFILE_ID_V1
        );
        assert_eq!(
            profile["walking_reference"]["root_forward_position_feedback"],
            "disabled"
        );

        let v5 = biomechanics_forward_start_stop_environment_manifest_v5()
            .expect("forward start/stop v5 manifest");
        assert_eq!(
            v5.environment_id.as_str(),
            BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID_V5
        );
        assert_ne!(v5.body_schema_hash, v4.body_schema_hash);
        assert_ne!(v5.observation_layout_hash, v4.observation_layout_hash);
        assert_ne!(
            v5.command_schedule_profile_hash,
            v4.command_schedule_profile_hash
        );
        assert_ne!(v5.reward_profile_hash, v4.reward_profile_hash);
        assert_eq!(v5.reward_components, v4.reward_components);
        assert_ne!(v5.action_layout_hash, v4.action_layout_hash);
        assert_ne!(
            v5.correspondence_profile_hash,
            v4.correspondence_profile_hash
        );
        let descriptor: Value = serde_json::from_str(
            &biomechanics_forward_start_stop_isaac_descriptor_json_v5()
                .expect("forward v5 descriptor"),
        )
        .expect("valid JSON");
        let profile = &descriptor["environment_profiles"][0];
        assert_eq!(
            descriptor["body_schema_id"],
            "nextengine.body.humanoid-biomechanics-raja-1700.v4"
        );
        assert_eq!(
            profile["action_layout_id"],
            BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID_V3
        );
        assert_eq!(
            profile["action"]["residual_scale_multiplier_q16"],
            BIOMECHANICS_FORWARD_START_STOP_RESIDUAL_SCALE_MULTIPLIER_Q16_V5
        );
    }

    #[test]
    fn standing_reward_has_exact_bounded_best_case() {
        let compiled = CompiledBodySchemaV3::compile(
            &biomechanics_humanoid_body_schema_v2(),
            PersistentId::from_bytes([0; 16]),
        )
        .expect("compiled biomechanics");
        let positions = vec![0; compiled.base.actuator_definitions.len()];
        let targets = positions.clone();
        let facts = BiomechanicsStandingRewardFactsV1 {
            root_rotation_q1_30: [0, 0, 0, 1_i64 << 30],
            root_height_micrometres: BIOMECHANICS_HUMANOID_ROOT_HEIGHT_MICROMETRES,
            joint_positions_microradians: &positions,
            reference_targets_microradians: &targets,
            root_linear_velocity_micrometres_per_second: [0; 3],
            root_angular_velocity_microradians_per_second: [0; 3],
            absolute_applied_effort_sum_micronewton_metres: 0,
            applied_targets_microradians: &targets,
            previous_applied_targets_microradians: &targets,
            contacting_sole_slip_sum_micrometres_per_second: 0,
            contacting_sole_count: 2,
            fell: false,
        };
        let (components, total) = biomechanics_standing_reward_q16_v1(&compiled, &facts)
            .expect("bounded standing reward");
        assert_eq!(components, [65_536, 65_536, 65_536, 0, 0, 0, 0, 0]);
        assert_eq!(total, 114_688);
    }

    fn schema_id() -> &'static str {
        "nextengine.body.humanoid-biomechanics-raja-1700.v3"
    }
}
