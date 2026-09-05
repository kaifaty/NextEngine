use super::*;

/// Articulated finite-volume forefoot diagnostic. No existing environment uses it.
#[must_use]
pub fn biomechanics_humanoid_body_schema_v8() -> BodySchemaV2 {
    let mut schema = biomechanics_humanoid_body_schema_v7();
    schema.schema_id = id("nextengine.body.humanoid-biomechanics-raja-1700.v8");
    schema.schema_revision = 8;
    schema.source_provenance_hash =
        domain_hash(b"nextengine.source.raja-1700.volumetric-forefoot.v8");
    schema.solver_projection_profile_hash =
        domain_hash(b"nextengine.solver-projection.humanoid-biomechanics-raja-1700.v4");
    for (side, sign) in [("right", 1_i64), ("left", -1)] {
        let rear_id = body_id(&format!("{side}-ankle-roll"));
        let toe_id = body_id(&format!("{side}-mtp"));
        let toe_joint = joint_id(&format!("{side}-mtp"));
        let rear = schema
            .bodies
            .iter_mut()
            .find(|b| b.body_id == rear_id)
            .unwrap();
        let mut toe = rear.clone();
        // Restore the source calcaneus, not the old merged calcaneus/toe tensor.
        set_mass(rear, 1_250_000, [0, 30_000, 100_000], [4_100, 3_900, 1_400]);
        // Keep the sole fixed while containing the source inertia's mass volume.
        rear.colliders[0].geometry = box_geometry([55_000, 47_500, 102_500]);
        rear.colliders[0].local_pose = pose([0, 28_865, 87_500]);

        toe.body_id = toe_id.clone();
        toe.parent_body_id = Some(rear_id.clone());
        toe.local_bind_pose = pose([sign * 1_080, -2_000, 178_800]);
        // Homogeneous80x30x68 mm inertia proxy at the source toe COM.
        // Diagonal m*(b²+c²)/12, rounded to micro kg m²:100,199,132.
        set_mass(
            &mut toe,
            216_600,
            [-sign * 17_500, 6_000, 34_600],
            [100, 199, 132],
        );
        toe.colliders[0].collider_id = id(&format!("collider.{side}-forefoot"));
        toe.colliders[0].geometry = box_geometry([60_000, 20_000, 40_000]);
        toe.colliders[0].local_pose = pose([-sign * 1_080, 3_365, 40_000]);
        schema.bodies.push(toe);
        schema.joints.push(BodyJointDefinitionV2 {
            joint_id: toe_joint.clone(),
            parent_body_id: rear_id.clone(),
            child_body_id: toe_id.clone(),
            anatomical_semantic_id: id("anatomical-joint.mtp-extension"),
            parent_frame: pose([sign * 1_080, -2_000, 178_800]),
            child_frame: BodyPoseV2::default(),
            axis_q1_30: [-Q30, 0, 0],
            hard_minimum_microradians: -349_066,
            hard_maximum_microradians: 1_221_730,
            soft_minimum_microradians: -174_533,
            soft_maximum_microradians: 1_047_198,
            neutral_position_microradians: 0,
            maximum_velocity_microradians_per_second: 8_000_000,
        });
        schema.actuators.push(BodyActuatorDefinitionV2 {
            actuator_id: id(&format!("actuator.{side}-mtp")),
            joint_id: toe_joint,
            stiffness_q16: 25 * 65_536,
            damping_q16: 1_311,
            minimum_effort_micronewton_metres: -12_000_000,
            maximum_effort_micronewton_metres: 12_000_000,
            maximum_effort_rate_micronewton_metres_per_second: 120_000_000,
            maximum_power_microwatts: 40_000_000,
            maximum_positive_work_microjoules_per_motor_tick: 666_667,
            residual_scale_microradians: 100_000,
            minimum_target_delta_microradians_per_motor_tick: -50_000,
            maximum_target_delta_microradians_per_motor_tick: 50_000,
        });
        let group = schema
            .mass_projection_groups
            .iter_mut()
            .find(|g| g.mapping_group_id == map_id(&format!("{side}-ankle")))
            .unwrap();
        // Same total mass/first moment. New toe tensor is explicitly represented
        // in the neutral aggregate, rather than hidden behind the old source hash.
        group.source_inertia_tensor_microkilogram_metre_squared =
            [8_155, -sign * 71, sign * 309, 7_958, 645, 2_733];
        group.members.push(BodyMassProjectionMemberV2 {
            body_id: toe_id.clone(),
            body_origin_in_group_micrometres: [sign * 9_000, -43_950, 130_030],
        });
        schema.collision_exclusions.push(BodyCollisionExclusionV2 {
            first_body_id: rear_id.clone(),
            second_body_id: toe_id.clone(),
        });
        for effector in &mut schema.effectors {
            if effector.effector_id == id(&format!("effector.{side}-heel")) {
                effector.local_pose = pose([0, -18_635, 0]);
            } else if effector.effector_id == id(&format!("effector.{side}-forefoot")) {
                effector.body_id = toe_id.clone();
                effector.local_pose = pose([-sign * 1_080, -16_635, 40_000]);
            }
        }
    }
    for prefix in ["body", "joint", "actuator"] {
        schema.symmetry_pairs.push(BodySymmetryPairV2 {
            left_id: id(&format!("{prefix}.left-mtp")),
            right_id: id(&format!("{prefix}.right-mtp")),
            value_rule: BodyMirrorValueRuleV2::Preserved,
        });
    }
    let schema = schema.canonicalize();
    schema
        .validate()
        .expect("finite-volume forefoot profile is valid");
    schema
}

fn set_mass(body: &mut BodyDefinitionV2, mass: u64, com: [i64; 3], inertia: [i64; 3]) {
    body.mass_microkilograms = mass;
    body.solver_mass_microkilograms = mass;
    body.center_of_mass_micrometres = com;
    body.solver_center_of_mass_micrometres = com;
    body.inertia_tensor_microkilogram_metre_squared = [inertia[0], 0, 0, inertia[1], 0, inertia[2]];
    body.solver_principal_inertia_microkilogram_metre_squared = inertia.map(|v| v as u64);
    body.solver_principal_frame = BodyPoseV2::default();
    body.solver_tensor_error_max_microkilogram_metre_squared = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn articulated_foot_preserves_total_mass_and_nonfoot_anatomy() {
        let old = biomechanics_humanoid_body_schema_v7();
        let new = biomechanics_humanoid_body_schema_v8();
        assert_eq!(
            (new.bodies.len(), new.joints.len(), new.actuators.len()),
            (26, 25, 25)
        );
        assert_eq!(
            new.bodies.iter().map(|b| b.colliders.len()).sum::<usize>(),
            21
        );
        assert_eq!(
            new.bodies
                .iter()
                .map(|b| b.mass_microkilograms)
                .sum::<u64>(),
            75_337_000
        );
        for body in &old.bodies {
            if !body.body_id.as_str().ends_with("-ankle-roll") {
                assert_eq!(
                    new.bodies.iter().find(|b| b.body_id == body.body_id),
                    Some(body)
                );
            }
        }
        for joint in &old.joints {
            assert!(new.joints.contains(joint));
        }
        for actuator in &old.actuators {
            assert!(new.actuators.contains(actuator));
        }
        for group in &old.mass_projection_groups {
            if !group.mapping_group_id.as_str().ends_with("-ankle") {
                assert!(new.mass_projection_groups.contains(group));
            }
        }
        assert_ne!(old.schema_hash(), new.schema_hash());
    }

    #[test]
    fn forefoot_mass_proxy_is_volumetric_inside_its_collider_and_sole_is_level() {
        let body = biomechanics_humanoid_body_schema_v8();
        for side in ["left", "right"] {
            let toe = body
                .bodies
                .iter()
                .find(|b| b.body_id == body_id(&format!("{side}-mtp")))
                .unwrap();
            let collider = &toe.colliders[0];
            let PhysicsGeometryV1::Box {
                half_extents_micrometres: shape_half,
            } = collider.geometry
            else {
                panic!("toe box");
            };
            let mass_half = [40_000_i64, 15_000, 34_000];
            for axis in 0..3 {
                assert!(
                    (toe.center_of_mass_micrometres[axis]
                        - collider.local_pose.translation_micrometres[axis])
                        .abs()
                        + mass_half[axis]
                        <= shape_half[axis]
                );
            }
            let expected: [u64; 3] = std::array::from_fn(|axis| {
                let squares: u128 = (0..3)
                    .filter(|i| *i != axis)
                    .map(|i| mass_half[i] as u128 * mass_half[i] as u128)
                    .sum();
                // Half-extents formula m*(b²+c²)/3; nearest micro kg m².
                ((u128::from(toe.mass_microkilograms) * squares + 1_500_000_000_000)
                    / 3_000_000_000_000) as u64
            });
            assert_eq!(expected, [100, 199, 132]);
            assert_eq!(
                toe.solver_principal_inertia_microkilogram_metre_squared,
                expected
            );
            for principal in expected {
                assert!(expected.iter().sum::<u64>() > 2 * principal);
            }
            assert_eq!(
                toe.local_bind_pose.translation_micrometres[1]
                    + collider.local_pose.translation_micrometres[1]
                    - shape_half[1],
                -18_635
            );
            let effector = body
                .effectors
                .iter()
                .find(|e| e.effector_id == id(&format!("effector.{side}-forefoot")))
                .unwrap();
            assert_eq!(effector.body_id, toe.body_id);
            assert_eq!(
                effector.local_pose.translation_micrometres[1],
                collider.local_pose.translation_micrometres[1] - shape_half[1]
            );
        }
    }

    #[test]
    fn source_rear_inertia_has_a_finite_volume_realization_inside_rear_collider() {
        let schema = biomechanics_humanoid_body_schema_v8();
        for side in ["left", "right"] {
            let rear = schema
                .bodies
                .iter()
                .find(|b| b.body_id == body_id(&format!("{side}-ankle-roll")))
                .unwrap();
            let collider = &rear.colliders[0];
            let PhysicsGeometryV1::Box {
                half_extents_micrometres: shape_half,
            } = collider.geometry
            else {
                panic!("rear box");
            };
            let inertia = rear.solver_principal_inertia_microkilogram_metre_squared;
            for axis in 0..3 {
                let margin = inertia.iter().sum::<u64>() - 2 * inertia[axis];
                assert!(margin > 0);
                // Equivalent homogeneous cuboid: h_i² = 3*(I_j+I_k-I_i)/(2*m).
                // Compare squared lengths in exact integer units, without sqrt.
                let available = shape_half[axis]
                    - (rear.center_of_mass_micrometres[axis]
                        - collider.local_pose.translation_micrometres[axis])
                        .abs();
                assert!(available > 0);
                assert!(
                    (available as u128).pow(2) * 2 * u128::from(rear.mass_microkilograms)
                        >= 3 * u128::from(margin) * 1_000_000_000_000
                );
            }
        }
    }

    #[test]
    fn foot_descriptor_is_inspection_only() {
        let text = crate::biomechanics_body_diagnostic_descriptor_json_v8().unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["body_schema_revision"], 8);
        assert_eq!(value["body_count"], 26);
        assert_eq!(value["action_width"], 25);
        assert_eq!(value["backend_admission"], "native-body-diagnostic-only");
        assert!(value.get("environment_profiles").is_none());
        assert!(value.get("standing_reference_profile_id").is_none());
    }

    #[test]
    fn four_sole_material_closure_requires_the_exact_articulated_profile() {
        use next_contracts::ids::PersistentId;
        let original = biomechanics_humanoid_body_schema_v8();
        let subject = PersistentId::from_bytes([0; 16]);
        let compiled = crate::CompiledBodySchemaV3::compile(&original, subject).unwrap();
        assert_eq!(
            compiled
                .physics_descriptors
                .collider_material_assignment_counts[&id(crate::BIOMECHANICS_SOLE_MATERIAL_ID)],
            4
        );
        let mut changed = original.clone();
        changed.schema_id = id("nextengine.body.unadmitted-four-sole.v1");
        assert!(matches!(
            crate::CompiledBodySchemaV3::compile(&changed, subject),
            Err(crate::MotorCompileError::UnsupportedMaterialProfile)
        ));
        let mut changed = original;
        changed
            .bodies
            .iter_mut()
            .find(|b| b.body_id == body_id("right-mtp"))
            .unwrap()
            .colliders[0]
            .local_pose
            .translation_micrometres[2] += 1;
        assert!(matches!(
            crate::CompiledBodySchemaV3::compile(&changed, subject),
            Err(crate::MotorCompileError::UnsupportedMaterialProfile)
        ));
    }

    #[cfg(feature = "physx-sdk")]
    #[test]
    fn native_mtp_extension_lifts_forefoot_without_rotating_rear_foot() {
        use crate::CompiledBodySchemaV4;
        use next_contracts::ids::PersistentId;
        let schema = biomechanics_humanoid_body_schema_v8();
        let compiled =
            CompiledBodySchemaV4::compile(&schema, PersistentId::from_bytes([0; 16])).unwrap();
        let base = &compiled.base.base;
        let mut world = compiled.create_world().unwrap();
        let neutral = world.raw_checkpoint();
        let snapshot = world.capture().unwrap();
        assert_eq!(
            crate::BiomechanicsProceduralStandingControllerV2::new(&compiled, &snapshot),
            Err(crate::ProceduralStandingError::ProfileMismatch)
        );
        for side in ["left", "right"] {
            let dof = base.joint_dof_ordinals[&joint_id(&format!("{side}-mtp"))] as usize;
            let toe_slot = base
                .construction_order
                .iter()
                .position(|b| *b == body_id(&format!("{side}-mtp")))
                .unwrap();
            let rear_slot = base
                .construction_order
                .iter()
                .position(|b| *b == body_id(&format!("{side}-ankle-roll")))
                .unwrap();
            let mut state = neutral.clone();
            state.joints[dof].position_bits = (std::f32::consts::PI / 6.0).to_bits();
            world.restore(&state).unwrap();
            let actual = world.raw_checkpoint();
            assert_eq!(actual.links[rear_slot], neutral.links[rear_slot]);
            let [x, y, z, w] = actual.links[toe_slot]
                .rotation_bits
                .map(|b| f64::from(f32::from_bits(b)));
            // World-up component of the native forefoot's local forward axis.
            assert!((2.0 * (y * z - w * x) - 0.5).abs() < 1e-6);
            // Every sole corner rises: no artificial hinge below the ground.
            let toe_body = schema
                .bodies
                .iter()
                .find(|b| b.body_id == body_id(&format!("{side}-mtp")))
                .unwrap();
            let centre =
                toe_body.colliders[0].local_pose.translation_micrometres[0] as f64 / 1_000_000.0;
            for lateral in [centre - 0.06, centre + 0.06] {
                for forward in [0.0, 0.08] {
                    let raised = 2.0 * (x * y + w * z) * lateral
                        + 2.0 * (y * z - w * x) * forward
                        + (1.0 - 2.0 * (x * x + z * z)) * -0.016635;
                    assert!(raised > -0.016635);
                }
            }
            // Coupled ankle/MTP pose: rear foot pitches while toes remain level.
            // This is a native kinematic heel-rise oracle, not a loaded balance trial.
            let ankle = base.joint_dof_ordinals[&joint_id(&format!("{side}-ankle-pitch"))] as usize;
            state.joints[ankle].position_bits = (std::f32::consts::PI / 6.0).to_bits();
            world.restore(&state).unwrap();
            let heel_raised = world.raw_checkpoint();
            let [x, y, z, w] = heel_raised.links[toe_slot]
                .rotation_bits
                .map(|b| f64::from(f32::from_bits(b)));
            assert!((1.0 - 2.0 * (x * x + z * z) - 1.0).abs() < 1e-6);
            assert!((2.0 * (y * z - w * x)).abs() < 1e-6);
            let [x, y, z, w] = heel_raised.links[rear_slot]
                .rotation_bits
                .map(|b| f64::from(f32::from_bits(b)));
            assert!((2.0 * (y * z - w * x) + 0.5).abs() < 1e-6);
            world.restore(&neutral).unwrap();
            assert_eq!(world.raw_checkpoint(), neutral);
        }
        world.restore(&neutral).unwrap();
        let frame = world.apply_efforts_and_step(&[0; 25]).unwrap();
        assert_eq!(frame.links.len(), 26);
    }
}
