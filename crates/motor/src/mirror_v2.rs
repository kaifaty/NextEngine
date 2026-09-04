use next_contracts::body::BodySchemaV2;
use next_contracts::ids::PersistentId;
use next_contracts::physics::PhysicsGeometryV1;
use serde_json::{Value, json};

use crate::{
    CompiledBodySchemaV2, CompiledBodySchemaV3, MotorCompileError,
    biomechanics_humanoid_body_schema_v2, biomechanics_humanoid_body_schema_v3,
};

pub fn biomechanics_isaac_mirror_descriptor_json_v1() -> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v2();
    biomechanics_isaac_mirror_descriptor_json_v1_for_schema(&schema)
}

fn biomechanics_isaac_mirror_descriptor_json_v1_for_schema(
    schema: &BodySchemaV2,
) -> Result<String, MotorCompileError> {
    let compiled = CompiledBodySchemaV2::compile(schema, PersistentId::from_bytes([0; 16]))?;
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

pub fn biomechanics_isaac_mirror_descriptor_json_v2() -> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v2();
    biomechanics_isaac_mirror_descriptor_json_v2_for_schema(&schema)
}

pub fn biomechanics_isaac_mirror_descriptor_json_v3() -> Result<String, MotorCompileError> {
    let schema = biomechanics_humanoid_body_schema_v3();
    biomechanics_isaac_mirror_descriptor_json_v2_for_schema(&schema)
}

fn biomechanics_isaac_mirror_descriptor_json_v2_for_schema(
    schema: &BodySchemaV2,
) -> Result<String, MotorCompileError> {
    let compiled = CompiledBodySchemaV3::compile(schema, PersistentId::from_bytes([0; 16]))?;
    let mut descriptor: Value = serde_json::from_str(
        &biomechanics_isaac_mirror_descriptor_json_v1_for_schema(schema)?,
    )
    .expect("engine-generated biomechanics mirror V1 is valid JSON");
    descriptor["schema_version"] = json!(2);
    descriptor["translator_id"] = json!("nextengine.isaac.biomechanics-mirror.v2");
    descriptor["compiled_descriptor_schema_version"] = json!(3);
    descriptor["compiled_descriptor_hash"] = json!(compiled.compiled_descriptor_hash.to_hex());
    descriptor["material_lineage_hash"] = json!(compiled.material_lineage_hash.to_hex());
    descriptor["ground_material_id"] =
        json!(compiled.physics_descriptors.ground_material_id.as_str());
    descriptor["materials"] = Value::Array(
        compiled
            .physics_descriptors
            .materials
            .values()
            .map(|material| {
                json!({
                    "schema_version": material.schema_version,
                    "material_id": material.base.material_id.as_str(),
                    "descriptor_revision": material.base.descriptor_revision,
                    "static_friction_q16": material.base.static_friction_q16,
                    "dynamic_friction_q16": material.base.dynamic_friction_q16,
                    "restitution_q16": material.base.restitution_q16,
                    "rolling_friction_q16": material.rolling_friction_q16,
                    "spinning_friction_q16": material.spinning_friction_q16,
                    "surface_velocity_micrometres_per_second": material.surface_velocity_micrometres_per_second,
                    "canonical_material_tags": material.base.canonical_material_tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>(),
                })
            })
            .collect(),
    );
    let combine = &compiled.physics_descriptors.material_combine_profile;
    descriptor["material_combine_profile"] = json!({
        "schema_version": combine.schema_version,
        "profile_id": combine.profile_id.as_str(),
        "profile_revision": combine.profile_revision,
        "static_friction": combine_rule_name(combine.static_friction),
        "dynamic_friction": combine_rule_name(combine.dynamic_friction),
        "restitution": combine_rule_name(combine.restitution),
        "rolling_friction": combine_rule_name(combine.rolling_friction),
        "spinning_friction": combine_rule_name(combine.spinning_friction),
        "surface_velocity": "CanonicalParticipantOrder",
    });
    descriptor["collider_material_assignment_counts"] = Value::Array(
        compiled
            .physics_descriptors
            .collider_material_assignment_counts
            .iter()
            .map(|(material_id, count)| {
                json!({
                    "material_id": material_id.as_str(),
                    "collider_count": count,
                })
            })
            .collect(),
    );
    let mut output = serde_json::to_string_pretty(&descriptor)
        .expect("serde_json::Value serialization cannot fail");
    output.push('\n');
    Ok(output)
}

fn combine_rule_name(rule: next_contracts::physics::PhysicsMaterialCombineRuleV1) -> &'static str {
    match rule {
        next_contracts::physics::PhysicsMaterialCombineRuleV1::Minimum => "Minimum",
        next_contracts::physics::PhysicsMaterialCombineRuleV1::Maximum => "Maximum",
        next_contracts::physics::PhysicsMaterialCombineRuleV1::ArithmeticMeanTiesToEven => {
            "ArithmeticMeanTiesToEven"
        }
        next_contracts::physics::PhysicsMaterialCombineRuleV1::ProductTiesToEvenClamped => {
            "ProductTiesToEvenClamped"
        }
    }
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

    #[test]
    fn biomechanics_mirror_matches_the_tracked_cpu_isaac_golden() {
        let actual = biomechanics_isaac_mirror_descriptor_json_v1().expect("mirror descriptor");
        let expected =
            include_str!("../../../lab/tests/fixtures/biomechanics_motor_mirror_v1.json");
        assert_eq!(actual, expected);
    }

    #[test]
    fn biomechanics_material_mirror_v2_closes_exact_assignments_and_lineage() {
        let text = biomechanics_isaac_mirror_descriptor_json_v2().expect("V2 mirror descriptor");
        assert_eq!(
            next_contracts::ids::content_hash_from_bytes(next_contracts::canonical::sha256(
                text.as_bytes()
            ))
            .to_hex(),
            "7928fe23affaf9dd16a0c82db1d7da85e61af2ad0f071f9423c6df1121ba50a3"
        );
        let value: Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(value["schema_version"], 2);
        assert_eq!(value["compiled_descriptor_schema_version"], 3);
        assert_eq!(value["materials"].as_array().map(Vec::len), Some(3));
        assert_eq!(
            value["ground_material_id"],
            "physics-material.humanoid-ground.v1"
        );
        assert_eq!(
            value["material_combine_profile"]["static_friction"],
            "ArithmeticMeanTiesToEven"
        );
        assert_eq!(
            value["collider_material_assignment_counts"],
            json!([
                {
                    "material_id": "physics-material.humanoid-body.v1",
                    "collider_count": 17,
                },
                {
                    "material_id": "physics-material.humanoid-sole.v1",
                    "collider_count": 2,
                },
            ])
        );
    }
}
