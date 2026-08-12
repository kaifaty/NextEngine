use super::*;

#[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
fn two_link_catalog() -> PhysXArticulationCatalog {
    let zero = 0.0_f32.to_bits();
    let identity = [zero, zero, zero, 1.0_f32.to_bits()];
    PhysXArticulationCatalog {
        static_boxes: vec![StaticBoxInput {
            centre_bits: [zero, (-0.5_f32).to_bits(), zero],
            half_extents_bits: [5.0_f32.to_bits(), 0.5_f32.to_bits(), 5.0_f32.to_bits()],
            user_token: 1,
        }],
        links: vec![
            ArticulationLinkInput {
                user_token: 10,
                parent_link_index: next_physics_physx_ffi::NO_PARENT_LINK,
                shape_kind: next_physics_physx_ffi::SHAPE_CAPSULE,
                position_bits: [zero, 2.0_f32.to_bits(), zero],
                rotation_bits: identity,
                shape_dimensions_bits: [0.2_f32.to_bits(), 0.25_f32.to_bits(), zero],
                mass_bits: 5.0_f32.to_bits(),
                inertia_bits: [0.2_f32.to_bits(); 3],
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            },
            ArticulationLinkInput {
                user_token: 11,
                parent_link_index: 0,
                shape_kind: next_physics_physx_ffi::SHAPE_CAPSULE,
                position_bits: [zero, 1.5_f32.to_bits(), zero],
                rotation_bits: identity,
                shape_dimensions_bits: [0.15_f32.to_bits(), 0.2_f32.to_bits(), zero],
                mass_bits: 2.0_f32.to_bits(),
                inertia_bits: [0.1_f32.to_bits(); 3],
                linear_damping_bits: 0.05_f32.to_bits(),
                angular_damping_bits: 0.05_f32.to_bits(),
            },
        ],
        joints: vec![ArticulationJointInput {
            child_link_index: 1,
            reserved: 0,
            parent_position_bits: [zero, (-0.25_f32).to_bits(), zero],
            parent_rotation_bits: identity,
            child_position_bits: [zero, 0.2_f32.to_bits(), zero],
            child_rotation_bits: identity,
            lower_limit_bits: (-1.0_f32).to_bits(),
            upper_limit_bits: 1.0_f32.to_bits(),
            max_velocity_bits: 20.0_f32.to_bits(),
        }],
    }
}

#[cfg(feature = "physx-sdk")]
fn three_link_collision_catalog() -> PhysXArticulationCatalogV2 {
    let zero = 0.0_f32.to_bits();
    let identity = [zero, zero, zero, 1.0_f32.to_bits()];
    let links = (0_u32..3)
        .map(|index| ArticulationLinkInputV2 {
            user_token: 100 + u64::from(index),
            parent_link_index: if index == 0 {
                next_physics_physx_ffi::NO_PARENT_LINK
            } else {
                index - 1
            },
            first_shape_index: index,
            shape_count: 1,
            reserved: 0,
            position_bits: [zero, 2.0_f32.to_bits(), zero],
            rotation_bits: identity,
            centre_of_mass_position_bits: [zero; 3],
            centre_of_mass_rotation_bits: identity,
            mass_bits: 1.0_f32.to_bits(),
            inertia_bits: [0.1_f32.to_bits(); 3],
            linear_damping_bits: 0.05_f32.to_bits(),
            angular_damping_bits: 0.05_f32.to_bits(),
        })
        .collect();
    let shapes = (0_u32..3)
        .map(|index| ArticulationShapeInputV2 {
            user_token: 1_000 + u64::from(index),
            link_index: index,
            shape_kind: next_physics_physx_ffi::SHAPE_SPHERE,
            position_bits: [zero; 3],
            rotation_bits: identity,
            shape_dimensions_bits: [0.4_f32.to_bits(), zero, zero],
            collision_layer: 2,
            collision_mask_low: 1 << 2,
            collision_mask_high: 0,
        })
        .collect();
    let joints = (1_u32..3)
        .map(|child_link_index| ArticulationJointInput {
            child_link_index,
            reserved: 0,
            parent_position_bits: [zero; 3],
            parent_rotation_bits: identity,
            child_position_bits: [zero; 3],
            child_rotation_bits: identity,
            lower_limit_bits: (-1.0_f32).to_bits(),
            upper_limit_bits: 1.0_f32.to_bits(),
            max_velocity_bits: 20.0_f32.to_bits(),
        })
        .collect();
    PhysXArticulationCatalogV2 {
        static_boxes: Vec::new(),
        links,
        shapes,
        joints,
        collision_exclusions: Vec::new(),
    }
}

#[cfg(feature = "physx-sdk")]
fn observed_contact_pair(
    catalog: &PhysXArticulationCatalogV2,
    expected_shape_tokens: (u64, u64),
) -> bool {
    let mut profile = PhysXSceneProfile::deterministic_humanoid(128, 8, 4);
    profile.gravity_bits = [0.0_f32.to_bits(); 3];
    let mut world = PhysXArticulationWorldV2::create(profile, catalog).expect("V2 world");
    (0..4).any(|_| {
        world
            .apply_efforts_and_step(&[0; 2])
            .expect("collision step")
            .contacts
            .iter()
            .any(|contact| (contact.shape_a_token, contact.shape_b_token) == expected_shape_tokens)
    })
}

#[test]
#[cfg(not(any(feature = "physx-sdk", feature = "mock-abi")))]
fn disabled_sdk_is_reported_before_world_activation() {
    let quantization = PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("quantization");
    let numeric =
        AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization).expect("numeric");
    let error = PhysXGroundedCapsuleQuery::new(numeric, quantization).expect_err("SDK unavailable");
    assert_eq!(error.stable_code(), "PHYSX_SDK_UNAVAILABLE");
}

#[test]
fn legacy_reference_profile_is_not_accepted_by_physx() {
    let quantization = PhysicsQuantizationProfileV1::capsule_reference_v1().expect("quantization");
    let numeric =
        AuthoritativeNumericProfileV1::capsule_reference_v1(&quantization).expect("numeric");
    let error = PhysXGroundedCapsuleQuery::new(numeric, quantization)
        .expect_err("legacy profile remains reference-only");
    assert_eq!(error.stable_code(), "PHYSX_PROFILE_UNSUPPORTED");
}

#[test]
fn altered_v2_recipe_is_not_accepted_under_the_builtin_profile_id() {
    let mut quantization =
        PhysicsQuantizationProfileV1::grounded_capsule_v2().expect("quantization");
    quantization
        .rules
        .get_mut(
            &next_contracts::ids::SchemaId::new(PHYSICS_SWEEP_DISTANCE_FIELD_ID)
                .expect("distance field ID"),
        )
        .expect("distance rule")
        .scale_numerator += 1;
    let numeric =
        AuthoritativeNumericProfileV1::grounded_capsule_v2(&quantization).expect("numeric");
    let error =
        PhysXGroundedCapsuleQuery::new(numeric, quantization).expect_err("altered built-in recipe");
    assert_eq!(error.stable_code(), "PHYSX_PROFILE_UNSUPPORTED");
}

#[test]
#[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
fn articulation_adapter_steps_canonicalizes_and_restores() {
    let profile = PhysXSceneProfile::deterministic_humanoid(128, 8, 4);
    let catalog = two_link_catalog();
    let mut world = PhysXArticulationWorld::create(profile, &catalog).expect("world");
    let checkpoint = world.raw_checkpoint();
    let stepped = world.apply_efforts_and_step(&[10_000_000]).expect("step");
    assert_eq!(stepped.links.len(), 2);
    assert_eq!(stepped.joints.len(), 1);
    assert_ne!(stepped.joints[0].velocity_microradians_per_second, 0);
    let restored = world.restore(&checkpoint).expect("restore");
    assert_eq!(restored.joints[0].position_microradians, 0);
    assert_eq!(restored.joints[0].velocity_microradians_per_second, 0);
}

#[test]
#[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
fn articulation_adapter_rejects_ambiguous_construction_order() {
    let profile = PhysXSceneProfile::deterministic_humanoid(128, 8, 4);
    let mut catalog = two_link_catalog();
    catalog.links[1].user_token = catalog.links[0].user_token;
    let error =
        PhysXArticulationWorld::create(profile, &catalog).expect_err("duplicate semantic token");
    assert_eq!(
        error.stable_code(),
        "PHYSX_NON_CANONICAL_CONSTRUCTION_ORDER"
    );
}

#[test]
#[cfg(feature = "physx-sdk")]
fn v2_native_contacts_obey_masks_exclusions_and_preserve_shape_tokens() {
    let catalog = three_link_collision_catalog();
    assert!(
        observed_contact_pair(&catalog, (1_000, 1_002)),
        "the admitted non-adjacent self-collision must expose both shape tokens"
    );

    let mut mask_filtered = catalog.clone();
    mask_filtered.shapes[2].collision_mask_low = 1 << 3;
    assert!(
        !observed_contact_pair(&mask_filtered, (1_000, 1_002)),
        "a mutually incompatible collision mask must suppress the pair"
    );

    let mut explicitly_excluded = catalog;
    explicitly_excluded.collision_exclusions = vec![ArticulationCollisionExclusionV2 {
        first_link_index: 0,
        second_link_index: 2,
    }];
    assert!(
        !observed_contact_pair(&explicitly_excluded, (1_000, 1_002)),
        "an explicit self-collision exclusion must suppress the pair"
    );
}
