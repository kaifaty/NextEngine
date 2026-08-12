use next_contracts::ids::PersistentId;
use next_contracts::physics::PhysicsGeometryV1;
use serde_json::{Value, json};

use crate::{CompiledBodySchemaV2, MotorCompileError, biomechanics_humanoid_body_schema_v2};

pub fn biomechanics_isaac_mirror_descriptor_json_v1() -> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v2();
    let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([0; 16]))?;
    let collider_id_by_token = compiled
        .collider_tokens
        .iter()
        .map(|(id, token)| (*token, id))
        .collect::<std::collections::BTreeMap<_, _>>();
    let body_id_by_slot = compiled
        .physics_descriptors
        .bodies
        .iter()
        .map(|(id, body)| (id.body_slot, body))
        .collect::<std::collections::BTreeMap<_, _>>();
    let bodies = compiled
        .physx_catalog
        .links
        .iter()
        .enumerate()
        .map(|(slot, link)| {
            let descriptor = body_id_by_slot[&(slot as u32)];
            let shapes = descriptor
                .base
                .base
                .shapes
                .values()
                .zip(
                    &compiled.physx_catalog.shapes[link.first_shape_index as usize
                        ..link.first_shape_index as usize + link.shape_count as usize],
                )
                .map(|(shape, ffi)| {
                    json!({
                        "collider_id": collider_id_by_token[&ffi.user_token].as_str(),
                        "shape_token": ffi.user_token,
                        "local_translation_micrometres": shape.local_pose.translation_micrometres,
                        "local_rotation_q1_30": shape.local_pose.rotation_q1_30,
                        "geometry": geometry_json(&shape.geometry),
                        "material_id": shape.material_id.as_str(),
                        "collision_layer": shape.collision_layer,
                        "collision_mask": shape.collision_mask,
                        "participation": shape.participation as u8,
                        "contact_reporting": shape.contact_reporting as u8,
                        "contact_role": compiled.collider_contact_roles[&ffi.user_token] as u8,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "body_id": descriptor.base.semantic_body_id.as_str(),
                "body_slot": slot,
                "body_token": link.user_token,
                "parent_body_slot": (link.parent_link_index != u32::MAX).then_some(link.parent_link_index),
                "initial_translation_f32_bits": link.position_bits,
                "initial_rotation_f32_bits": link.rotation_bits,
                "mass_microkilograms": descriptor.base.mass_microkilograms,
                "center_of_mass_micrometres": descriptor.base.center_of_mass_micrometres,
                "authoritative_inertia_tensor_microkilogram_metre_squared": descriptor.authoritative_inertia_tensor_microkilogram_metre_squared,
                "solver_principal_inertia_microkilogram_metre_squared": descriptor.base.inertia_microkilogram_metre_squared,
                "solver_principal_frame": {
                    "translation_micrometres": descriptor.solver_principal_frame.translation_micrometres,
                    "rotation_q1_30": descriptor.solver_principal_frame.rotation_q1_30,
                },
                "solver_tensor_error_max_microkilogram_metre_squared": descriptor.solver_tensor_error_max_microkilogram_metre_squared,
                "non_colliding_carrier": descriptor.non_colliding_carrier,
                "colliders": shapes,
            })
        })
        .collect::<Vec<_>>();
    let joints = compiled
        .physics_descriptors
        .joints
        .iter()
        .map(|joint| {
            let dof = compiled.joint_dof_ordinals[&joint.base.joint_id];
            let ffi = compiled.physx_catalog.joints[dof as usize];
            json!({
                "joint_id": joint.base.joint_id.as_str(),
                "dof_ordinal": dof,
                "anatomical_semantic_id": joint.anatomical_semantic_id.as_str(),
                "parent_body_slot": joint.base.parent_body_id.body_slot,
                "child_body_slot": joint.base.child_body_id.body_slot,
                "axis_q1_30": joint.base.axis_q1_30,
                "parent_frame": {
                    "translation_micrometres": joint.parent_frame.translation_micrometres,
                    "rotation_q1_30": joint.parent_frame.rotation_q1_30,
                },
                "child_frame": {
                    "translation_micrometres": joint.child_frame.translation_micrometres,
                    "rotation_q1_30": joint.child_frame.rotation_q1_30,
                },
                "solver_parent_rotation_f32_bits": ffi.parent_rotation_bits,
                "solver_child_rotation_f32_bits": ffi.child_rotation_bits,
                "hard_limit_microradians": [joint.base.limit_min_microradians, joint.base.limit_max_microradians],
                "soft_limit_microradians": [joint.soft_limit_min_microradians, joint.soft_limit_max_microradians],
                "neutral_position_microradians": joint.neutral_position_microradians,
                "maximum_velocity_microradians_per_second": joint.base.maximum_velocity_microradians_per_second,
            })
        })
        .collect::<Vec<_>>();
    let actuators = compiled
        .physics_descriptors
        .actuators
        .iter()
        .zip(&compiled.actuator_dof_ordinals)
        .map(|(actuator, dof)| {
            json!({
                "actuator_id": actuator.base.actuator_id.as_str(),
                "joint_id": actuator.base.joint_id.as_str(),
                "dof_ordinal": dof,
                "stiffness_q16": actuator.stiffness_q16,
                "damping_q16": actuator.damping_q16,
                "effort_micronewton_metres": [actuator.minimum_effort_micronewton_metres, actuator.maximum_effort_micronewton_metres],
                "maximum_effort_rate_micronewton_metres_per_second": actuator.base.maximum_effort_rate_micronewton_metres_per_second,
                "maximum_power_microwatts": actuator.maximum_power_microwatts,
                "maximum_positive_work_microjoules_per_motor_tick": actuator.maximum_positive_work_microjoules_per_motor_tick,
                "residual_scale_microradians": actuator.residual_scale_microradians,
                "target_delta_microradians_per_motor_tick": [actuator.minimum_target_delta_microradians_per_motor_tick, actuator.maximum_target_delta_microradians_per_motor_tick],
            })
        })
        .collect::<Vec<_>>();
    let descriptor = json!({
        "schema_version": 1,
        "translator_id": "nextengine.isaac.biomechanics-mirror.v1",
        "coordinate_mapping": {
            "engine_axes": "+X right, +Y up, +Z forward",
            "isaac_from_engine_vector": ["x", "-z", "y"],
            "isaac_quaternion_order": "wxyz",
            "engine_quaternion_order": "xyzw",
        },
        "body_schema_id": schema.schema_id.as_str(),
        "body_schema_revision": schema.schema_revision,
        "body_schema_hash": compiled.body_schema_hash.to_hex(),
        "compiled_descriptor_hash": compiled.compiled_descriptor_hash.to_hex(),
        "physics_hz": 240,
        "motor_hz": 60,
        "body_count": compiled.construction_order.len(),
        "action_width": compiled.actuator_definitions.len(),
        "ordered_body_ids": compiled.construction_order.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        "ordered_actuator_ids": compiled.actuator_definitions.iter().map(|actuator| actuator.actuator_id.as_str()).collect::<Vec<_>>(),
        "bodies": bodies,
        "joints": joints,
        "actuators": actuators,
        "collision_exclusions": compiled.physx_catalog.collision_exclusions.iter().map(|pair| [pair.first_link_index, pair.second_link_index]).collect::<Vec<_>>(),
        "effectors": schema.effectors.iter().map(|effector| json!({
            "effector_id": effector.effector_id.as_str(),
            "body_id": effector.body_id.as_str(),
            "local_translation_micrometres": effector.local_pose.translation_micrometres,
            "local_rotation_q1_30": effector.local_pose.rotation_q1_30,
            "semantic_role_id": effector.semantic_role_id.as_str(),
        })).collect::<Vec<_>>(),
    });
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
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
        _ => unreachable!("BodySchemaV2 validator admits only box, sphere and capsule"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biomechanics_mirror_is_generated_from_compiled_descriptor() {
        let text = biomechanics_isaac_mirror_descriptor_json_v1().expect("mirror descriptor");
        let value: Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(value["body_count"], 24);
        assert_eq!(value["action_width"], 23);
        assert_eq!(value["bodies"].as_array().map(Vec::len), Some(24));
        assert_eq!(value["joints"].as_array().map(Vec::len), Some(23));
        assert_eq!(value["actuators"].as_array().map(Vec::len), Some(23));
        let shape_count = value["bodies"]
            .as_array()
            .expect("bodies")
            .iter()
            .map(|body| body["colliders"].as_array().expect("colliders").len())
            .sum::<usize>();
        assert_eq!(shape_count, 19);
    }
}
