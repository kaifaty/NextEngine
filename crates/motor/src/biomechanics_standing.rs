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
    biomechanics_humanoid_body_schema_v2, biomechanics_humanoid_body_schema_v3,
    biomechanics_isaac_mirror_descriptor_json_v2, biomechanics_isaac_mirror_descriptor_json_v3,
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
pub const BIOMECHANICS_FORWARD_START_STOP_OBSERVATION_LAYOUT_ID: &str =
    "nextengine.motor.observation.humanoid-biomechanics-forward-start-stop.v1";
pub const BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID: &str =
    "nextengine.motor.action.humanoid-biomechanics-forward-start-stop-residual.v1";
pub const BIOMECHANICS_FORWARD_START_STOP_MAXIMUM_EPISODE_STEPS: u64 = 1_200;

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
    biomechanics_forward_start_stop_environment_manifest(&biomechanics_humanoid_body_schema_v3())
}

fn biomechanics_forward_start_stop_environment_manifest(
    schema: &next_contracts::body::BodySchemaV2,
) -> Result<MotorTrainingEnvironmentManifestV2, MotorCompileError> {
    let compiled = CompiledBodySchemaV3::compile(schema, PersistentId::from_bytes([0; 16]))?;
    let body_hash = compiled.base.body_schema_hash;
    let foundation = crate::curriculum_locomotion_stages_v2()
        .into_iter()
        .next()
        .expect("curriculum foundation stage is engine-owned");
    let command_profile_hash = foundation.command_profile.profile_hash()?;
    let reward_components = BIOMECHANICS_FORWARD_START_STOP_REWARD_COMPONENT_IDS
        .into_iter()
        .zip(BIOMECHANICS_FORWARD_START_STOP_REWARD_COEFFICIENTS_Q16)
        .map(|(component_id, coefficient_q16)| MotorRewardComponentV1 {
            component_id: id(component_id),
            coefficient_q16,
            minimum_raw: 0,
            maximum_raw: 65_536,
        })
        .collect();
    let manifest = MotorTrainingEnvironmentManifestV2 {
        schema_version: MOTOR_TRAINING_ENVIRONMENT_MANIFEST_V2_SCHEMA_VERSION,
        environment_id: id(BIOMECHANICS_FORWARD_START_STOP_ENVIRONMENT_PROFILE_ID),
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
        action_layout_hash: biomechanics_standing_action_layout_hash(
            &compiled,
            BIOMECHANICS_FORWARD_START_STOP_ACTION_LAYOUT_ID,
        ),
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
        reward_profile_hash: biomechanics_forward_start_stop_reward_profile_hash(&compiled),
        termination_profile_hash: domain_hash(
            "nextengine.motor.termination.biomechanics-forward-start-stop.v1",
            body_hash,
        ),
        rng_derivation_profile_hash: domain_hash(
            "nextengine.motor.episode-seed-derivation.v1",
            body_hash,
        ),
        correspondence_profile_hash: domain_hash(
            "nextengine.motor.correspondence.biomechanics-forward-start-stop.v1",
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

fn biomechanics_standing_reference_profile_hash(compiled: &CompiledBodySchemaV3) -> ContentHash {
    let mut bytes = domain_preimage(
        "nextengine.motor.procedural-standing.v1",
        compiled.base.body_schema_hash,
    );
    bytes.extend_from_slice(&PROCEDURAL_STANDING_KNEE_TARGET_MICRORADIANS.to_le_bytes());
    bytes.extend_from_slice(&PROCEDURAL_STANDING_ANKLE_BIAS_MICRORADIANS.to_le_bytes());
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
