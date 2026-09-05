use super::*;

#[test]
fn sdk_and_abi_version_must_match_exactly() {
    assert_eq!(std::mem::size_of::<PhysXVersion>(), 16);
    assert_eq!(std::mem::size_of::<RawSweepOutput>(), 32);
    assert_eq!(std::mem::size_of::<SceneProfileInput>(), 36);
    assert_eq!(std::mem::size_of::<MaterialProfileInput>(), 72);
    assert_eq!(std::mem::size_of::<RigidBodyInput>(), 64);
    assert_eq!(std::mem::size_of::<ArticulationLinkInput>(), 80);
    assert_eq!(std::mem::size_of::<ArticulationJointInput>(), 76);
    assert_eq!(std::mem::size_of::<ArticulationLinkInputV2>(), 104);
    assert_eq!(std::mem::size_of::<ArticulationShapeInputV2>(), 72);
    assert_eq!(std::mem::size_of::<ArticulationCollisionExclusionV2>(), 8);
    assert_eq!(std::mem::size_of::<LinkState>(), 64);
    assert_eq!(std::mem::size_of::<JointState>(), 8);
    assert_eq!(std::mem::size_of::<ContactOutput>(), 56);
    assert_eq!(std::mem::size_of::<ContactOutputV2>(), 72);
    assert_eq!(validate_version(EXPECTED_PHYSX_VERSION), Ok(()));
    let mut wrong_patch = EXPECTED_PHYSX_VERSION;
    wrong_patch.patch = 1;
    assert_eq!(
        validate_version(wrong_patch),
        Err(PhysXFfiError::VersionMismatch)
    );
    let mut wrong_abi = EXPECTED_PHYSX_VERSION;
    wrong_abi.abi += 1;
    assert_eq!(
        validate_version(wrong_abi),
        Err(PhysXFfiError::VersionMismatch)
    );
}

#[test]
#[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
fn force_schedule_extension_rejects_invalid_tag_without_consuming_scene() {
    let mut world = NativeWorld::create().expect("world");
    let scene = SceneProfileInput {
        gravity_bits: [0.0_f32.to_bits(), (-9.81_f32).to_bits(), 0.0_f32.to_bits()],
        timestep_bits: (1.0_f32 / 240.0).to_bits(),
        position_iterations: 16,
        velocity_iterations: 4,
        max_contacts: 128,
        max_actors: 8,
        max_joints: 4,
    };
    let schedule = ExternalForceScheduleV1::EverySolverPositionIteration;
    assert_eq!(
        world.configure_scene_with_force_schedule_v1(scene, schedule),
        Err(PhysXFfiError::InvalidArgument)
    );
    world
        .configure_material(MaterialProfileInput {
            coefficient_encoding: MATERIAL_COEFFICIENT_ENCODING_Q16,
            static_friction: 52_429,
            dynamic_friction: 45_875,
            restitution: 0,
            rolling_friction: 0,
            spinning_friction: 0,
            surface_velocity_micrometres_per_second: [0; 3],
            coefficient_combine_rules: [MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN; 5],
            surface_velocity_combine_rule: MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER,
        })
        .expect("material");
    assert_eq!(
        // SAFETY: live owned world and valid fixed-layout input. An invalid numeric
        // tag is legal C input and must be rejected, never cast into a Rust enum.
        unsafe {
            raw::world_configure_scene_with_force_schedule_v1(world.handle.as_ptr(), &scene, 2)
        },
        STATUS_INVALID_ARGUMENT
    );
    world
        .configure_scene_with_force_schedule_v1(scene, schedule)
        .expect("valid retry after rejected input");
    assert_eq!(
        world.configure_scene(scene),
        Err(PhysXFfiError::InvalidArgument)
    );
    assert_eq!(
        world.configure_scene_with_force_schedule_v1(scene, schedule),
        Err(PhysXFfiError::InvalidArgument)
    );
}

#[test]
#[cfg(not(any(feature = "physx-sdk", feature = "mock-abi")))]
fn disabled_sdk_fails_before_allocating_a_world() {
    assert!(matches!(
        NativeWorld::create(),
        Err(PhysXFfiError::Unavailable)
    ));
}

#[test]
#[cfg(feature = "mock-abi")]
fn mock_abi_owns_and_destroys_world() {
    for _ in 0..1_000 {
        let mut world = NativeWorld::create().expect("mock world");
        world.reserve_static_boxes(1).expect("reserve");
        world
            .add_static_box(StaticBoxInput {
                centre_bits: [0.0_f32.to_bits(); 3],
                half_extents_bits: [1.0_f32.to_bits(); 3],
                user_token: 7,
            })
            .expect("box");
    }
}

#[test]
#[cfg(feature = "mock-abi")]
fn mock_abi_reports_capacity_and_native_validation_failures() {
    let mut world = NativeWorld::create().expect("mock world");
    world.reserve_static_boxes(0).expect("zero capacity");
    assert_eq!(
        world.add_static_box(StaticBoxInput {
            centre_bits: [0.0_f32.to_bits(); 3],
            half_extents_bits: [1.0_f32.to_bits(); 3],
            user_token: 7,
        }),
        Err(PhysXFfiError::CapacityExceeded)
    );
    assert_eq!(
        world.sweep_capsule_axis(CapsuleAxisSweepInput {
            centre_bits: [f32::NAN.to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits()],
            radius_bits: 1.0_f32.to_bits(),
            half_segment_bits: 1.0_f32.to_bits(),
            axis: 0,
            direction_sign: 1,
            distance_bits: 1.0_f32.to_bits(),
        }),
        Err(PhysXFfiError::InvalidArgument)
    );
}

#[test]
#[cfg(feature = "mock-abi")]
fn material_must_be_explicit_zero_extended_and_configured_once() {
    let mut world = NativeWorld::create().expect("mock world");
    let scene = SceneProfileInput {
        gravity_bits: [0.0_f32.to_bits(), (-9.81_f32).to_bits(), 0.0_f32.to_bits()],
        timestep_bits: (1.0_f32 / 240.0).to_bits(),
        position_iterations: 8,
        velocity_iterations: 2,
        max_contacts: 128,
        max_actors: 8,
        max_joints: 4,
    };
    assert_eq!(
        world.configure_scene(scene),
        Err(PhysXFfiError::InvalidArgument)
    );
    let mut material = MaterialProfileInput {
        coefficient_encoding: MATERIAL_COEFFICIENT_ENCODING_Q16,
        static_friction: 52_429,
        dynamic_friction: 45_875,
        restitution: 0,
        rolling_friction: 0,
        spinning_friction: 0,
        surface_velocity_micrometres_per_second: [0; 3],
        coefficient_combine_rules: [MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN; 5],
        surface_velocity_combine_rule: MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER,
    };
    material.rolling_friction = 1;
    assert_eq!(
        world.configure_material(material),
        Err(PhysXFfiError::InvalidArgument)
    );
    material.rolling_friction = 0;
    world.configure_material(material).expect("material");
    assert_eq!(
        world.configure_material(material),
        Err(PhysXFfiError::InvalidArgument)
    );
    world.configure_scene(scene).expect("scene after material");
}

#[test]
#[cfg(any(feature = "physx-sdk", feature = "mock-abi"))]
fn scene_articulation_effort_step_and_state_round_trip() {
    let zero = 0.0_f32.to_bits();
    let one = 1.0_f32.to_bits();
    let mut world = NativeWorld::create().expect("world");
    world
        .configure_material(MaterialProfileInput {
            coefficient_encoding: MATERIAL_COEFFICIENT_ENCODING_F32_BITS,
            static_friction: 0.8_f32.to_bits(),
            dynamic_friction: 0.7_f32.to_bits(),
            restitution: 0.0_f32.to_bits(),
            rolling_friction: 0,
            spinning_friction: 0,
            surface_velocity_micrometres_per_second: [0; 3],
            coefficient_combine_rules: [MATERIAL_COMBINE_ARITHMETIC_MEAN_TIES_TO_EVEN; 5],
            surface_velocity_combine_rule: MATERIAL_SURFACE_VELOCITY_CANONICAL_PARTICIPANT_ORDER,
        })
        .expect("material");
    world
        .configure_scene(SceneProfileInput {
            gravity_bits: [zero, (-9.81_f32).to_bits(), zero],
            timestep_bits: (1.0_f32 / 240.0).to_bits(),
            position_iterations: 8,
            velocity_iterations: 2,
            max_contacts: 128,
            max_actors: 8,
            max_joints: 4,
        })
        .expect("scene");
    world.reserve_static_boxes(1).expect("reserve ground");
    world
        .add_static_box(StaticBoxInput {
            centre_bits: [zero, (-0.5_f32).to_bits(), zero],
            half_extents_bits: [5.0_f32.to_bits(), 0.5_f32.to_bits(), 5.0_f32.to_bits()],
            user_token: 1,
        })
        .expect("ground");
    let identity = [zero, zero, zero, one];
    let links = [
        ArticulationLinkInput {
            user_token: 10,
            parent_link_index: NO_PARENT_LINK,
            shape_kind: SHAPE_CAPSULE,
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
            shape_kind: SHAPE_CAPSULE,
            position_bits: [zero, 1.5_f32.to_bits(), zero],
            rotation_bits: identity,
            shape_dimensions_bits: [0.15_f32.to_bits(), 0.2_f32.to_bits(), zero],
            mass_bits: 2.0_f32.to_bits(),
            inertia_bits: [0.1_f32.to_bits(); 3],
            linear_damping_bits: 0.05_f32.to_bits(),
            angular_damping_bits: 0.05_f32.to_bits(),
        },
    ];
    let joints = [ArticulationJointInput {
        child_link_index: 1,
        reserved: 0,
        parent_position_bits: [zero, (-0.25_f32).to_bits(), zero],
        parent_rotation_bits: identity,
        child_position_bits: [zero, 0.2_f32.to_bits(), zero],
        child_rotation_bits: identity,
        lower_limit_bits: (-1.0_f32).to_bits(),
        upper_limit_bits: 1.0_f32.to_bits(),
        max_velocity_bits: 20.0_f32.to_bits(),
    }];
    world
        .add_articulation(&links, &joints, 8, 2)
        .expect("articulation");
    let initial = world.export_articulation_state().expect("initial state");
    world
        .apply_articulation_efforts(&[10.0_f32.to_bits()])
        .expect("effort");
    for _ in 0..4 {
        world.step().expect("step");
    }
    let stepped = world.export_articulation_state().expect("stepped state");
    assert_ne!(stepped.1[0].velocity_bits, initial.1[0].velocity_bits);
    world
        .import_articulation_state(initial.0[0], &initial.1)
        .expect("restore");
    let restored = world.export_articulation_state().expect("restored state");
    assert_eq!(restored.0[0], initial.0[0]);
    assert_eq!(restored.1, initial.1);
}
