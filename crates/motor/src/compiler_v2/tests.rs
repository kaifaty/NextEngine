use super::*;
use crate::{
    BIOMECHANICS_HUMANOID_BODY_COUNT, BIOMECHANICS_HUMANOID_COLLIDER_COUNT,
    BIOMECHANICS_HUMANOID_DOF, biomechanics_humanoid_body_schema_v2,
};

#[test]
fn frozen_biomechanics_profile_compiles_without_defaulted_rows() {
    let schema = biomechanics_humanoid_body_schema_v2();
    let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([9; 16]))
        .expect("compile frozen biomechanics profile");
    assert_eq!(
        compiled.construction_order.len(),
        BIOMECHANICS_HUMANOID_BODY_COUNT
    );
    assert_eq!(
        compiled.physx_catalog.shapes.len(),
        BIOMECHANICS_HUMANOID_COLLIDER_COUNT
    );
    assert_eq!(
        compiled.physx_catalog.joints.len(),
        BIOMECHANICS_HUMANOID_DOF
    );
    assert_eq!(
        compiled.actuator_definitions.len(),
        BIOMECHANICS_HUMANOID_DOF
    );
    assert_eq!(
        compiled.collider_tokens.len(),
        BIOMECHANICS_HUMANOID_COLLIDER_COUNT
    );
    for joint in &schema.joints {
        let dof = compiled.joint_dof_ordinals[&joint.joint_id] as usize;
        let ffi = compiled.physx_catalog.joints[dof];
        assert_eq!(
            ffi.child_link_index as usize,
            compiled
                .construction_order
                .iter()
                .position(|body| body == &joint.child_body_id)
                .expect("compiled child")
        );
        assert_eq!(
            ffi.parent_position_bits,
            metres_bits(joint.parent_frame.translation_micrometres).expect("frame")
        );
        assert_eq!(
            ffi.child_position_bits,
            metres_bits(joint.child_frame.translation_micrometres).expect("frame")
        );
        assert_eq!(
            ffi.lower_limit_bits,
            microradians_bits(joint.hard_minimum_microradians).expect("lower")
        );
        assert_eq!(
            ffi.upper_limit_bits,
            microradians_bits(joint.hard_maximum_microradians).expect("upper")
        );
        assert_axis_alignment(ffi.parent_rotation_bits, joint.axis_q1_30);
        assert_axis_alignment(ffi.child_rotation_bits, joint.axis_q1_30);
    }
    for body in &schema.bodies {
        let slot = compiled
            .construction_order
            .iter()
            .position(|candidate| candidate == &body.body_id)
            .expect("body slot");
        let descriptor = compiled
            .physics_descriptors
            .bodies
            .values()
            .find(|descriptor| descriptor.base.semantic_body_id == body.body_id)
            .expect("body descriptor");
        let ffi = compiled.physx_catalog.links[slot];
        assert_eq!(
            descriptor.base.mass_microkilograms,
            body.solver_mass_microkilograms
        );
        assert_eq!(
            descriptor.base.center_of_mass_micrometres,
            body.solver_center_of_mass_micrometres
        );
        assert_eq!(
            descriptor.base.inertia_microkilogram_metre_squared,
            body.solver_principal_inertia_microkilogram_metre_squared
        );
        assert_eq!(
            descriptor.authoritative_inertia_tensor_microkilogram_metre_squared,
            body.inertia_tensor_microkilogram_metre_squared
        );
        assert_eq!(descriptor.base.base.shapes.len(), body.colliders.len());
        assert_eq!(
            ffi.centre_of_mass_position_bits,
            metres_bits(body.solver_center_of_mass_micrometres).expect("CoM")
        );
        assert_eq!(
            ffi.mass_bits,
            scaled_u64_bits(body.solver_mass_microkilograms, 1_000_000.0).expect("mass")
        );
        assert_eq!(
            ffi.inertia_bits,
            body.solver_principal_inertia_microkilogram_metre_squared
                .map(|value| scaled_u64_bits(value, 1_000_000.0).expect("inertia"))
        );
        let ffi_shapes = &compiled.physx_catalog.shapes[ffi.first_shape_index as usize
            ..ffi.first_shape_index as usize + ffi.shape_count as usize];
        for (shape_slot, (collider, ffi_shape)) in body.colliders.iter().zip(ffi_shapes).enumerate()
        {
            let descriptor_shape = descriptor
                .base
                .base
                .shapes
                .values()
                .find(|shape| shape.shape_id.shape_slot == shape_slot as u32)
                .expect("shape descriptor");
            assert_eq!(
                descriptor_shape.local_pose,
                physics_pose(collider.local_pose).unwrap()
            );
            assert_eq!(descriptor_shape.geometry, collider.geometry);
            assert_eq!(descriptor_shape.material_id, collider.material_id);
            assert_eq!(descriptor_shape.collision_layer, collider.collision_layer);
            assert_eq!(descriptor_shape.collision_mask, collider.collision_mask);
            assert_eq!(descriptor_shape.participation, collider.participation);
            assert_eq!(
                descriptor_shape.contact_reporting,
                collider.contact_reporting
            );
            let (expected_kind, expected_dimensions) =
                geometry_to_ffi(&collider.geometry).expect("shape geometry");
            assert_eq!(ffi_shape.link_index, slot as u32);
            assert_eq!(ffi_shape.shape_kind, expected_kind);
            assert_eq!(ffi_shape.shape_dimensions_bits, expected_dimensions);
            assert_eq!(
                ffi_shape.position_bits,
                metres_bits(collider.local_pose.translation_micrometres).expect("shape pose")
            );
            assert_eq!(
                ffi_shape.rotation_bits,
                quaternion_bits(collider.local_pose.rotation_q1_30)
            );
            assert_eq!(
                ffi_shape.collision_layer,
                u32::from(collider.collision_layer)
            );
            assert_eq!(
                u64::from(ffi_shape.collision_mask_low)
                    | (u64::from(ffi_shape.collision_mask_high) << 32),
                collider.collision_mask
            );
            assert_eq!(
                compiled.collider_contact_roles[&ffi_shape.user_token],
                collider.contact_role
            );
        }
    }
}

#[test]
fn declaration_permutation_preserves_descriptor_and_isaac_authority() {
    let source = biomechanics_humanoid_body_schema_v2();
    let mut permuted = source.clone();
    permuted.bodies.reverse();
    for body in &mut permuted.bodies {
        body.colliders.reverse();
    }
    permuted.mass_projection_groups.reverse();
    for group in &mut permuted.mass_projection_groups {
        group.members.reverse();
    }
    permuted.joints.reverse();
    permuted.actuators.reverse();
    permuted.effectors.reverse();
    permuted.symmetry_pairs.reverse();
    permuted.collision_exclusions.reverse();
    permuted.capability_ids.reverse();
    let permuted = permuted.canonicalize();
    let subject = PersistentId::from_bytes([13; 16]);
    let expected = CompiledBodySchemaV2::compile(&source, subject).expect("source");
    let actual = CompiledBodySchemaV2::compile(&permuted, subject).expect("permuted");
    assert_eq!(expected.body_schema_hash, actual.body_schema_hash);
    assert_eq!(
        expected.compiled_descriptor_hash,
        actual.compiled_descriptor_hash
    );
    assert_eq!(expected.construction_order, actual.construction_order);
    assert_eq!(expected.physx_catalog, actual.physx_catalog);
    assert_eq!(expected.physics_descriptors, actual.physics_descriptors);
}

#[test]
#[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
fn frozen_biomechanics_profile_builds_a_fresh_native_articulation() {
    use next_physics_physx::PhysXArticulationWorldV2;

    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([11; 16]),
    )
    .expect("compile frozen biomechanics profile");
    let mut world =
        PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
            .expect("create native biomechanics articulation");
    let snapshot = world.capture().expect("capture fresh articulation");
    assert_eq!(snapshot.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
    assert_eq!(snapshot.joints.len(), BIOMECHANICS_HUMANOID_DOF);
    let stepped = world
        .apply_efforts_and_step(&[0; BIOMECHANICS_HUMANOID_DOF])
        .expect("step neutral articulation");
    assert!(
        !stepped.contacts.is_empty(),
        "neutral soles must contact ground"
    );
    let mut sole_contact_count = 0;
    for contact in stepped.contacts {
        let shape_token = if contact.actor_a_token == 1 {
            Some(contact.shape_b_token)
        } else if contact.actor_b_token == 1 {
            Some(contact.shape_a_token)
        } else {
            None
        };
        let Some(shape_token) = shape_token else {
            assert!(
                contact.separation_micrometres > 0 && contact.impulse_micronewton_seconds == [0; 3],
                "neutral self pair must remain separated and non-supporting: separation={} impulse={:?}",
                contact.separation_micrometres,
                contact.impulse_micronewton_seconds,
            );
            continue;
        };
        let role = compiled.collider_contact_roles.get(&shape_token);
        if role == Some(&BodyContactRoleV2::FootWithSoleFeature) {
            sole_contact_count += 1;
        } else {
            assert!(
                contact.separation_micrometres > 0 && contact.impulse_micronewton_seconds == [0; 3],
                "non-sole neutral contact must remain a non-supporting speculative contact: {role:?} separation={} impulse={:?}",
                contact.separation_micrometres,
                contact.impulse_micronewton_seconds,
            );
        }
    }
    assert!(
        sole_contact_count > 0,
        "both feet define the support polygon"
    );
}

#[test]
#[cfg(feature = "physx-sdk")]
fn every_native_joint_limit_resists_outward_effort() {
    use next_physics_physx::PhysXArticulationWorldV2;

    let schema = biomechanics_humanoid_body_schema_v2();
    let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([17; 16]))
        .expect("compile");
    let mut sweep_profile = compiled.physx_scene_profile;
    sweep_profile.gravity_bits = [0.0_f32.to_bits(); 3];
    let mut sweep_catalog = compiled.physx_catalog.clone();
    sweep_catalog.static_boxes.clear();
    sweep_catalog.shapes.clear();
    for link in &mut sweep_catalog.links {
        link.first_shape_index = 0;
        link.shape_count = 0;
    }
    let mut target_by_dof = [0_i64; BIOMECHANICS_HUMANOID_DOF];
    let actuator_by_dof = compiled
        .actuator_definitions
        .iter()
        .zip(&compiled.actuator_dof_ordinals)
        .map(|(actuator, dof)| (*dof as usize, actuator))
        .collect::<BTreeMap<_, _>>();
    for joint in &schema.joints {
        let dof = compiled.joint_dof_ordinals[&joint.joint_id] as usize;
        for limit in [
            joint.hard_minimum_microradians,
            joint.hard_maximum_microradians,
        ] {
            target_by_dof.fill(0);
            let outward_direction = if limit == joint.hard_minimum_microradians {
                -1
            } else {
                1
            };
            target_by_dof[dof] = limit + outward_direction * 500_000;
            let mut world = PhysXArticulationWorldV2::create(sweep_profile, &sweep_catalog)
                .expect("fresh sweep world");
            let mut snapshot = world.capture().expect("initial sweep state");
            let mut observed_minimum = snapshot.joints[dof].position_microradians;
            let mut observed_maximum = observed_minimum;
            for _ in 0..1_440 {
                let efforts = snapshot
                    .joints
                    .iter()
                    .map(|state| {
                        let actuator = actuator_by_dof[&(state.ordinal as usize)];
                        let error =
                            target_by_dof[state.ordinal as usize] - state.position_microradians;
                        let numerator = i128::from(actuator.stiffness_q16) * i128::from(error)
                            - i128::from(actuator.damping_q16)
                                * i128::from(state.velocity_microradians_per_second);
                        let effort = (numerator / 65_536) as i64;
                        effort.clamp(
                            actuator.minimum_effort_micronewton_metres,
                            actuator.maximum_effort_micronewton_metres,
                        )
                    })
                    .collect::<Vec<_>>();
                snapshot = world
                    .apply_efforts_and_step(&efforts)
                    .expect("PD sweep step");
                let position = snapshot.joints[dof].position_microradians;
                observed_minimum = observed_minimum.min(position);
                observed_maximum = observed_maximum.max(position);
            }
            let position = snapshot.joints[dof].position_microradians;
            assert!(
                observed_minimum >= joint.hard_minimum_microradians - 10_000
                    && observed_maximum <= joint.hard_maximum_microradians + 10_000,
                "{} crossed [{}, {}] during sweep: observed [{}, {}]",
                joint.joint_id.as_str(),
                joint.hard_minimum_microradians,
                joint.hard_maximum_microradians,
                observed_minimum,
                observed_maximum,
            );
            assert!(
                (position - limit).abs() <= 75_000,
                "{} did not reach hard limit {}: final {}",
                joint.joint_id.as_str(),
                limit,
                position,
            );
        }
    }
}

#[test]
#[cfg(feature = "physx-sdk")]
fn ten_thousand_native_reset_settle_cycles_are_exact_and_complete() {
    use next_physics_physx::PhysXArticulationWorldV2;

    let compiled = CompiledBodySchemaV2::compile(
        &biomechanics_humanoid_body_schema_v2(),
        PersistentId::from_bytes([19; 16]),
    )
    .expect("compile");
    let mut world =
        PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
            .expect("fresh reset world");
    let checkpoint = world.raw_checkpoint();
    let zero_efforts = [0; BIOMECHANICS_HUMANOID_DOF];
    let mut expected = world.restore(&checkpoint).expect("reference reset");
    for _ in 0..4 {
        expected = world
            .apply_efforts_and_step(&zero_efforts)
            .expect("reference settle step");
    }
    assert_eq!(expected.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
    assert_eq!(expected.joints.len(), BIOMECHANICS_HUMANOID_DOF);

    for cycle in 0..10_000 {
        let reset = world.restore(&checkpoint).expect("reset");
        assert_eq!(reset.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
        assert_eq!(reset.joints.len(), BIOMECHANICS_HUMANOID_DOF);
        let mut settled = reset;
        for _ in 0..4 {
            settled = world
                .apply_efforts_and_step(&zero_efforts)
                .expect("settle step");
        }
        assert_eq!(settled, expected, "reset/settle mismatch at cycle {cycle}");
    }
}

#[test]
#[cfg(feature = "physx-sdk")]
fn neutral_action_remains_bounded_for_sixty_simulated_seconds() {
    use next_physics_physx::PhysXArticulationWorldV2;

    let schema = biomechanics_humanoid_body_schema_v2();
    let compiled = CompiledBodySchemaV2::compile(&schema, PersistentId::from_bytes([23; 16]))
        .expect("compile");
    let hard_limits_by_dof = schema
        .joints
        .iter()
        .map(|joint| {
            (
                compiled.joint_dof_ordinals[&joint.joint_id] as usize,
                (
                    joint.hard_minimum_microradians,
                    joint.hard_maximum_microradians,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let actuator_by_dof = compiled
        .actuator_definitions
        .iter()
        .zip(&compiled.actuator_dof_ordinals)
        .map(|(actuator, dof)| (*dof as usize, actuator))
        .collect::<BTreeMap<_, _>>();
    let mut world =
        PhysXArticulationWorldV2::create(compiled.physx_scene_profile, &compiled.physx_catalog)
            .expect("fresh passive world");
    let mut snapshot = world.capture().expect("initial passive state");
    let mut previous_efforts = [0_i64; BIOMECHANICS_HUMANOID_DOF];
    let mut maximum_linear_speed_component = 0_i64;
    let mut maximum_angular_speed_component = 0_i64;
    let mut maximum_joint_speed = 0_i64;
    for step in 0..14_400 {
        let efforts = snapshot
            .joints
            .iter()
            .map(|state| {
                let dof = state.ordinal as usize;
                let actuator = actuator_by_dof[&dof];
                let requested = (-(i128::from(actuator.stiffness_q16)
                    * i128::from(state.position_microradians)
                    + i128::from(actuator.damping_q16)
                        * i128::from(state.velocity_microradians_per_second)))
                    / 65_536;
                let effort_limited = requested.clamp(
                    i128::from(actuator.minimum_effort_micronewton_metres),
                    i128::from(actuator.maximum_effort_micronewton_metres),
                );
                let maximum_delta =
                    actuator.maximum_effort_rate_micronewton_metres_per_second as i128 / 240;
                let previous = i128::from(previous_efforts[dof]);
                let rate_limited =
                    effort_limited.clamp(previous - maximum_delta, previous + maximum_delta);
                previous_efforts[dof] = rate_limited as i64;
                previous_efforts[dof]
            })
            .collect::<Vec<_>>();
        snapshot = world
            .apply_efforts_and_step(&efforts)
            .expect("passive step");
        assert_eq!(snapshot.links.len(), BIOMECHANICS_HUMANOID_BODY_COUNT);
        assert_eq!(snapshot.joints.len(), BIOMECHANICS_HUMANOID_DOF);
        for link in &snapshot.links {
            maximum_linear_speed_component = maximum_linear_speed_component.max(
                link.linear_velocity_micrometres_per_second
                    .iter()
                    .map(|value| value.abs())
                    .max()
                    .unwrap_or(0),
            );
            maximum_angular_speed_component = maximum_angular_speed_component.max(
                link.angular_velocity_microradians_per_second
                    .iter()
                    .map(|value| value.abs())
                    .max()
                    .unwrap_or(0),
            );
        }
        for joint in &snapshot.joints {
            let (minimum, maximum) = hard_limits_by_dof[&(joint.ordinal as usize)];
            assert!(
                joint.position_microradians >= minimum - 10_000
                    && joint.position_microradians <= maximum + 10_000,
                "joint {} crossed its hard ROM at step {step}: {} outside [{minimum}, {maximum}]",
                joint.ordinal,
                joint.position_microradians,
            );
            maximum_joint_speed =
                maximum_joint_speed.max(joint.velocity_microradians_per_second.abs());
        }
    }
    assert!(
        maximum_linear_speed_component <= 20_000_000,
        "passive root/link speed was not bounded: {maximum_linear_speed_component} um/s"
    );
    assert!(
        maximum_angular_speed_component <= 50_000_000,
        "passive angular speed was not bounded: {maximum_angular_speed_component} urad/s"
    );
    assert!(
        maximum_joint_speed <= 50_000_000,
        "passive joint speed was not bounded: {maximum_joint_speed} urad/s"
    );
}

fn assert_axis_alignment(rotation_bits: [u32; 4], expected_q1_30: [i32; 3]) {
    let [x, y, z, w] = rotation_bits.map(|bits| f64::from(f32::from_bits(bits)));
    let rotated_x = [
        1.0 - 2.0 * (y * y + z * z),
        2.0 * (x * y + z * w),
        2.0 * (x * z - y * w),
    ];
    let scale = (1_u64 << 30) as f64;
    for (actual, expected) in rotated_x
        .into_iter()
        .zip(expected_q1_30.map(|value| f64::from(value) / scale))
    {
        assert!((actual - expected).abs() <= 2.0e-7);
    }
}
