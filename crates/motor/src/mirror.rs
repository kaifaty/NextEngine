use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::motor::MotorTrainingEnvironmentManifestV2;
use next_contracts::physics::PhysicsGeometryV1;
use serde_json::{Value, json};

use crate::{
    CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID, CompiledBodySchemaV1,
    FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID, FixedPdController, JointControlStateV1,
    STANDING_ENVIRONMENT_PROFILE_ID, TrainingEnvironmentError, canonical_environment_manifest_v2,
    curriculum_locomotion_command_schedule, curriculum_locomotion_profile_hash_v2,
    curriculum_locomotion_stages_v2, derive_locomotion_episode_seed_set,
    flat_locomotion_command_profile_v1, flat_locomotion_command_schedule,
    reference_humanoid_body_schema_v1, rotate_world_to_root_local_q1_30,
};

pub const ISAAC_TRANSLATOR_VERSION: &str = "nextengine.isaac-usda-translator.v3";

pub fn stage0_isaac_mirror_descriptor_json_v2() -> Result<String, TrainingEnvironmentError> {
    let schema = reference_humanoid_body_schema_v1();
    let compiled = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0; 16]))
        .map_err(|_| TrainingEnvironmentError::Compile)?;
    let mut locomotion_compiled = compiled.clone();
    locomotion_compiled
        .apply_flat_locomotion_profile()
        .map_err(|_| TrainingEnvironmentError::Compile)?;
    let standing_manifest = canonical_environment_manifest_v2(STANDING_ENVIRONMENT_PROFILE_ID)?;
    let locomotion_manifest =
        canonical_environment_manifest_v2(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID)?;
    let curriculum_manifest =
        canonical_environment_manifest_v2(CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID)?;

    let bodies = schema
        .bodies
        .iter()
        .map(|body| {
            json!({
                "body_id": body.body_id.as_str(),
                "parent_body_id": body.parent_body_id.as_ref().map(|id| id.as_str()),
                "local_bind_translation_micrometres": body.local_bind_pose.translation_micrometres,
                "local_bind_rotation_q1_30": body.local_bind_pose.rotation_q1_30,
                "mass_microkilograms": body.mass_microkilograms,
                "center_of_mass_micrometres": body.center_of_mass_micrometres,
                "inertia_microkilogram_metre_squared": body.inertia_microkilogram_metre_squared,
                "colliders": body.colliders.iter().map(|collider| {
                    json!({
                        "collider_id": collider.collider_id.as_str(),
                        "local_translation_micrometres": collider.local_pose.translation_micrometres,
                        "local_rotation_q1_30": collider.local_pose.rotation_q1_30,
                        "geometry": geometry_json(&collider.geometry),
                        "material_id": collider.material_id.as_str(),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    let joints = schema
        .joints
        .iter()
        .map(|joint| {
            json!({
                "joint_id": joint.joint_id.as_str(),
                "parent_body_id": joint.parent_body_id.as_str(),
                "child_body_id": joint.child_body_id.as_str(),
                "parent_translation_micrometres": joint.parent_frame.translation_micrometres,
                "child_translation_micrometres": joint.child_frame.translation_micrometres,
                "axis_q1_30": joint.axis_q1_30,
                "limit_min_microradians": joint.limit_min_microradians,
                "limit_max_microradians": joint.limit_max_microradians,
                "maximum_velocity_microradians_per_second": joint.maximum_velocity_microradians_per_second,
            })
        })
        .collect::<Vec<_>>();
    let actuators = compiled
        .actuator_definitions
        .iter()
        .map(|actuator| {
            json!({
                "actuator_id": actuator.actuator_id.as_str(),
                "joint_id": actuator.joint_id.as_str(),
                "neutral_position_microradians": actuator.neutral_position_microradians,
                "stiffness_q16": actuator.stiffness_q16,
                "damping_q16": actuator.damping_q16,
                "maximum_effort_micronewton_metres": actuator.maximum_effort_micronewton_metres,
                "maximum_effort_rate_micronewton_metres_per_second": actuator.maximum_effort_rate_micronewton_metres_per_second,
            })
        })
        .collect::<Vec<_>>();
    let descriptor = json!({
        "schema_version": 2,
        "translator_version": ISAAC_TRANSLATOR_VERSION,
        "coordinate_mapping": {
            "engine_axes": "+X right, +Y up, +Z forward",
            "isaac_from_engine_vector": ["x", "-z", "y"],
            "isaac_quaternion_order": "wxyz",
            "engine_quaternion_order": "xyzw",
        },
        "body_schema_id": schema.schema_id.as_str(),
        "body_schema_revision": schema.schema_revision,
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "physics_hz": 240,
        "motor_hz": 60,
        "observation_width": 84,
        "action_width": 23,
        "ordered_body_ids": compiled.construction_order.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        "ordered_actuator_ids": compiled.actuator_definitions.iter().map(|value| value.actuator_id.as_str()).collect::<Vec<_>>(),
        "bodies": bodies,
        "joints": joints,
        "actuators": actuators,
        "effectors": schema.effectors.iter().map(|effector| json!({
            "effector_id": effector.effector_id.as_str(),
            "body_id": effector.body_id.as_str(),
            "semantic_role_id": effector.semantic_role_id.as_str(),
        })).collect::<Vec<_>>(),
        "environment_profiles": [
            profile_json(&standing_manifest, &compiled, "world", 50, json!({
                "kind": "zero",
            })),
            profile_json(
                &locomotion_manifest,
                &locomotion_compiled,
                "root-local",
                100,
                command_profile_json(),
            ),
            profile_json(
                &curriculum_manifest,
                &locomotion_compiled,
                "root-local",
                100,
                curriculum_command_profile_json(),
            ),
        ],
    });
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn stage0_isaac_mirror_golden_json_v2() -> Result<String, TrainingEnvironmentError> {
    let schema = reference_humanoid_body_schema_v1();
    let compiled = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0; 16]))
        .map_err(|_| TrainingEnvironmentError::Compile)?;
    let mut controller =
        FixedPdController::new(&compiled).map_err(|_| TrainingEnvironmentError::Compile)?;
    let states = vec![
        JointControlStateV1 {
            position_microradians: 0,
            velocity_microradians_per_second: 0,
        };
        controller.channel_count()
    ];
    let targets = vec![1_000_000; controller.channel_count()];
    let first = controller
        .step_substep(&targets, &states)
        .map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
    let second = controller
        .step_substep(&targets, &states)
        .map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
    let run_root = ContentHash::from_bytes(std::array::from_fn(|index| index as u8));
    let seeds = derive_locomotion_episode_seed_set(run_root, 17, 3)?;
    let command_seed = seeds
        .purpose_seeds
        .iter()
        .find_map(|(purpose, seed)| (purpose.as_str() == "randomization.command").then_some(*seed))
        .ok_or(TrainingEnvironmentError::SeedProfile)?;
    let schedule = flat_locomotion_command_schedule(command_seed)?;
    let curriculum_schedule = curriculum_locomotion_command_schedule(command_seed, 17)?;
    let descriptor = stage0_isaac_mirror_descriptor_json_v2()?;
    let standing_manifest = canonical_environment_manifest_v2(STANDING_ENVIRONMENT_PROFILE_ID)?;
    let locomotion_manifest =
        canonical_environment_manifest_v2(FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID)?;
    let curriculum_manifest =
        canonical_environment_manifest_v2(CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID)?;
    let quaternion_input = [0, 759_250_125, 0, 759_250_125];
    let vector_input = [1_000_000, 0, 0];
    let quaternion_output = rotate_world_to_root_local_q1_30(quaternion_input, vector_input)
        .map_err(|_| TrainingEnvironmentError::ArithmeticOverflow)?;
    let schedule_samples = [0_usize, 59, 60, 299, 300, 301, 360, 420, 540, 1_200]
        .into_iter()
        .map(|tick| {
            json!({
                "tick": tick,
                "command_raw": schedule[tick],
            })
        })
        .collect::<Vec<_>>();
    let curriculum_schedule_samples = [0_usize, 119, 120, 121, 360, 600, 1_200]
        .into_iter()
        .map(|tick| {
            json!({
                "tick": tick,
                "command_raw": curriculum_schedule[tick],
            })
        })
        .collect::<Vec<_>>();
    let golden = json!({
        "schema_version": 2,
        "descriptor_sha256": hex(&sha256(descriptor.as_bytes())),
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "ordered_actuator_ids": compiled.actuator_definitions.iter().map(|value| value.actuator_id.as_str()).collect::<Vec<_>>(),
        "profile_manifest_hashes": {
            STANDING_ENVIRONMENT_PROFILE_ID: standing_manifest.manifest_hash()?.to_hex(),
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID: locomotion_manifest.manifest_hash()?.to_hex(),
            CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID: curriculum_manifest.manifest_hash()?.to_hex(),
        },
        "reward_component_ids": {
            STANDING_ENVIRONMENT_PROFILE_ID: standing_manifest.reward_components.iter().map(|value| value.component_id.as_str()).collect::<Vec<_>>(),
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID: locomotion_manifest.reward_components.iter().map(|value| value.component_id.as_str()).collect::<Vec<_>>(),
            CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID: curriculum_manifest.reward_components.iter().map(|value| value.component_id.as_str()).collect::<Vec<_>>(),
        },
        "seed_input": {
            "run_root": run_root.to_hex(),
            "episode_ordinal": 17,
            "vector_slot": 3,
        },
        "purpose_seeds": seeds.purpose_seeds.iter().map(|(purpose, seed)| json!({
            "purpose_id": purpose.as_str(),
            "seed": hex(seed),
        })).collect::<Vec<_>>(),
        "command_schedule": {
            "sha256": command_schedule_hash(&schedule),
            "samples": schedule_samples,
        },
        "curriculum_command_schedule": {
            "episode_ordinal": 17,
            "sha256": command_schedule_hash(&curriculum_schedule),
            "samples": curriculum_schedule_samples,
        },
        "root_local_transform": {
            "quaternion_xyzw_q1_30": quaternion_input,
            "world_vector_raw": vector_input,
            "root_local_vector_raw": quaternion_output,
        },
        "pd_input": {
            "residual_target_microradians": 1_000_000,
            "position_microradians": 0,
            "velocity_microradians_per_second": 0,
        },
        "pd_channel_count": controller.channel_count(),
        "pd_first_effort": first[0].effort_micronewton_metres,
        "pd_first_flags": first[0].clamp_flags,
        "pd_second_effort": second[0].effort_micronewton_metres,
        "pd_second_flags": second[0].clamp_flags,
    });
    let mut output =
        serde_json::to_string_pretty(&golden).expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

fn profile_json(
    manifest: &MotorTrainingEnvironmentManifestV2,
    compiled: &CompiledBodySchemaV1,
    velocity_frame: &str,
    ground_half_extent_metres: u32,
    command_profile: Value,
) -> Value {
    json!({
        "profile_id": manifest.environment_id.as_str(),
        "manifest_hash": manifest.manifest_hash().expect("engine manifest is valid").to_hex(),
        "observation_layout_hash": compiled.observation_layout.layout_hash().expect("layout is valid").to_hex(),
        "action_layout_hash": compiled.action_layout.layout_hash().expect("layout is valid").to_hex(),
        "command_schedule_profile_hash": manifest.command_schedule_profile_hash.to_hex(),
        "reward_profile_hash": manifest.reward_profile_hash.to_hex(),
        "termination_profile_hash": manifest.termination_profile_hash.to_hex(),
        "rng_derivation_profile_hash": manifest.rng_derivation_profile_hash.to_hex(),
        "correspondence_profile_hash": manifest.correspondence_profile_hash.to_hex(),
        "maximum_episode_steps": manifest.maximum_episode_steps,
        "velocity_frame": velocity_frame,
        "ground_half_extent_metres": ground_half_extent_metres,
        "command_profile": command_profile,
        "observation_source_ids": compiled.observation_layout.channels.iter().map(|value| value.source_id.as_str()).collect::<Vec<_>>(),
        "reward_components": manifest.reward_components.iter().map(|value| json!({
            "component_id": value.component_id.as_str(),
            "coefficient_q16": value.coefficient_q16,
            "minimum_raw": value.minimum_raw,
            "maximum_raw": value.maximum_raw,
        })).collect::<Vec<_>>(),
        "termination": match manifest.environment_id.as_str() {
            FLAT_LOCOMOTION_ENVIRONMENT_PROFILE_ID
            | CURRICULUM_LOCOMOTION_ENVIRONMENT_PROFILE_ID => json!({
                "pelvis_height_micrometres_inclusive": 450_000,
                "world_bound_micrometres_inclusive": 90_000_000,
                "timeout_ticks": 1_200,
            }),
            _ => json!({
                "pelvis_height_micrometres_inclusive": 250_000,
                "timeout_ticks": 3_600,
            }),
        },
    })
}

fn command_profile_json() -> Value {
    let profile = flat_locomotion_command_profile_v1();
    json!({
        "kind": "sha256-counter-flat-command-v1",
        "profile_hash": profile.profile_hash().expect("command profile is valid").to_hex(),
        "randomization_stream_id": profile.randomization_stream_id.as_str(),
        "warmup_ticks": profile.warmup_ticks,
        "segment_ticks": profile.segment_ticks,
        "episode_ticks": profile.episode_ticks,
        "mode_weights_basis_points": profile.mode_weights_basis_points,
        "right_velocity_range_raw": [profile.right_velocity_min_micrometres_per_second, profile.right_velocity_max_micrometres_per_second],
        "forward_velocity_range_raw": [profile.forward_velocity_min_micrometres_per_second, profile.forward_velocity_max_micrometres_per_second],
        "yaw_rate_range_raw": [profile.yaw_rate_min_microradians_per_second, profile.yaw_rate_max_microradians_per_second],
        "linear_rate_limit_raw_per_second_squared": profile.linear_rate_limit_micrometres_per_second_squared,
        "yaw_rate_limit_raw_per_second_squared": profile.yaw_rate_limit_microradians_per_second_squared,
    })
}

fn curriculum_command_profile_json() -> Value {
    let stages = curriculum_locomotion_stages_v2();
    json!({
        "kind": "sha256-counter-episode-curriculum-v2",
        "profile_hash": curriculum_locomotion_profile_hash_v2().expect("curriculum profile is valid").to_hex(),
        "randomization_stream_id": "randomization.command",
        "episode_ticks": 1_200,
        "stages": stages.into_iter().map(|stage| {
            let profile = stage.command_profile;
            json!({
                "first_episode_ordinal": stage.first_episode_ordinal,
                "profile_id": profile.profile_id.as_str(),
                "profile_hash": profile.profile_hash().expect("stage profile is valid").to_hex(),
                "warmup_ticks": profile.warmup_ticks,
                "segment_ticks": profile.segment_ticks,
                "episode_ticks": profile.episode_ticks,
                "mode_weights_basis_points": profile.mode_weights_basis_points,
                "right_velocity_range_raw": [profile.right_velocity_min_micrometres_per_second, profile.right_velocity_max_micrometres_per_second],
                "forward_velocity_range_raw": [profile.forward_velocity_min_micrometres_per_second, profile.forward_velocity_max_micrometres_per_second],
                "yaw_rate_range_raw": [profile.yaw_rate_min_microradians_per_second, profile.yaw_rate_max_microradians_per_second],
                "linear_rate_limit_raw_per_second_squared": profile.linear_rate_limit_micrometres_per_second_squared,
                "yaw_rate_limit_raw_per_second_squared": profile.yaw_rate_limit_microradians_per_second_squared,
            })
        }).collect::<Vec<_>>(),
    })
}

fn command_schedule_hash(schedule: &[[i64; 3]]) -> String {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.motor-command-schedule.v1\0");
    preimage.extend_from_slice(&(schedule.len() as u64).to_le_bytes());
    for command in schedule {
        for value in command {
            preimage.extend_from_slice(&value.to_le_bytes());
        }
    }
    hex(&sha256(&preimage))
}

fn geometry_json(geometry: &PhysicsGeometryV1) -> Value {
    match geometry {
        PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } => json!({
            "kind": "box",
            "half_extents_micrometres": half_extents_micrometres,
        }),
        PhysicsGeometryV1::Sphere { radius_micrometres } => json!({
            "kind": "sphere",
            "radius_micrometres": radius_micrometres,
        }),
        PhysicsGeometryV1::Capsule {
            radius_micrometres,
            half_segment_micrometres,
        } => json!({
            "kind": "capsule",
            "radius_micrometres": radius_micrometres,
            "half_segment_micrometres": half_segment_micrometres,
        }),
        _ => unreachable!("reference humanoid uses primitive geometry only"),
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_python_fixture_is_generated_by_rust_authority() {
        assert_eq!(
            stage0_isaac_mirror_golden_json_v2().expect("golden"),
            include_str!("../../../lab/tests/fixtures/stage0_motor_mirror_v2.json")
        );
    }
}
