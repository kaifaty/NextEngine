use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::physics::PhysicsGeometryV1;
use serde_json::{Value, json};

use crate::{
    CompiledBodySchemaV1, FixedPdController, JointControlStateV1, MotorCompileError,
    STANDING_REWARD_COMPONENT_IDS, derive_episode_seed_set, reference_humanoid_body_schema_v1,
};

pub const ISAAC_TRANSLATOR_VERSION: &str = "nextengine.isaac-usda-translator.v1";

pub fn stage0_isaac_mirror_descriptor_json_v1() -> Result<String, MotorCompileError> {
    let schema = reference_humanoid_body_schema_v1();
    let compiled = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0; 16]))?;

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
        "schema_version": 1,
        "translator_version": ISAAC_TRANSLATOR_VERSION,
        "body_schema_id": schema.schema_id.as_str(),
        "body_schema_revision": schema.schema_revision,
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "physics_hz": 240,
        "motor_hz": 60,
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
        "reward_component_ids": STANDING_REWARD_COMPONENT_IDS,
    });
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

pub fn stage0_isaac_mirror_golden_json_v1() -> Result<String, MotorCompileError> {
    let schema = reference_humanoid_body_schema_v1();
    let compiled = CompiledBodySchemaV1::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let mut controller =
        FixedPdController::new(&compiled).map_err(|_| MotorCompileError::InvalidReference)?;
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
        .map_err(|_| MotorCompileError::NumericOverflow)?;
    let second = controller
        .step_substep(&targets, &states)
        .map_err(|_| MotorCompileError::NumericOverflow)?;
    let run_root = ContentHash::from_bytes(std::array::from_fn(|index| index as u8));
    let seeds =
        derive_episode_seed_set(run_root, 17, 3).map_err(|_| MotorCompileError::NumericOverflow)?;
    let descriptor = stage0_isaac_mirror_descriptor_json_v1()?;
    let golden = json!({
        "schema_version": 1,
        "descriptor_sha256": hex(&sha256(descriptor.as_bytes())),
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "ordered_actuator_ids": compiled.actuator_definitions.iter().map(|value| value.actuator_id.as_str()).collect::<Vec<_>>(),
        "reward_component_ids": STANDING_REWARD_COMPONENT_IDS,
        "seed_input": {
            "run_root": run_root.to_hex(),
            "episode_ordinal": 17,
            "vector_slot": 3,
        },
        "purpose_seeds": seeds.purpose_seeds.iter().map(|(purpose, seed)| json!({
            "purpose_id": purpose.as_str(),
            "seed": hex(seed),
        })).collect::<Vec<_>>(),
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
            stage0_isaac_mirror_golden_json_v1().expect("golden"),
            include_str!("../../../lab/tests/fixtures/stage0_motor_mirror_v1.json")
        );
    }
}
