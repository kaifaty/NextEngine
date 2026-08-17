use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{AssetId, ContentHash, PersistentId, SchemaId};
use next_contracts::physics::{PhysicsBodyIdV1, PhysicsGeometryV1};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn relay_collision_matches_visible_parts_and_hidden_pickup_stays_contained() {
    let (root, activated) = activated_reference_project("collider-parity");
    let session = next_reference_game::build_reference_game_session(activated.project.clone())
        .expect("reference session");
    let catalog = &session.bootstrap.physics_checkpoint.catalog;
    let relay_body_id = PhysicsBodyIdV1 {
        subject_id: session.interactive_object_id,
        body_slot: 0,
    };
    let relay = &catalog.bodies[&relay_body_id];
    let relay_shapes = relay
        .shapes
        .values()
        .map(|shape| {
            let PhysicsGeometryV1::Box {
                half_extents_micrometres,
            } = shape.geometry
            else {
                panic!("relay shape must be a box")
            };
            (
                shape.shape_id.shape_slot,
                shape.local_pose.translation_micrometres,
                half_extents_micrometres,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        relay_shapes,
        vec![
            (0, [-1_425_000, 225_000, 0], [175_000, 1_125_000, 180_000]),
            (1, [1_425_000, 225_000, 0], [175_000, 1_125_000, 180_000]),
            (2, [0, 1_250_000, 0], [1_600_000, 200_000, 200_000]),
            (3, [0, -300_000, 0], [320_000, 600_000, 100_000]),
        ]
    );
    assert!(relay_shapes.iter().all(|(_, _, half_extents)| {
        half_extents[0] <= 1_600_000 && half_extents[2] <= 200_000
    }));

    let pickup = &catalog.bodies[&PhysicsBodyIdV1 {
        subject_id: session.pickup_proxy_id,
        body_slot: 0,
    }];
    let pickup_shape = pickup.shapes.values().next().expect("pickup proxy shape");
    let PhysicsGeometryV1::Box {
        half_extents_micrometres: pickup_half_extents,
    } = pickup_shape.geometry
    else {
        panic!("pickup proxy must be a box")
    };
    let relay_switch = &relay.shapes[&next_contracts::physics::PhysicsShapeIdV1 {
        body_id: relay_body_id,
        shape_slot: 3,
    }];
    let PhysicsGeometryV1::Box {
        half_extents_micrometres: switch_half_extents,
    } = relay_switch.geometry
    else {
        panic!("relay switch must be a box")
    };
    assert_eq!(pickup.initial_pose, relay.initial_pose);
    assert_eq!(pickup_shape.local_pose.translation_micrometres, [0; 3]);
    assert_eq!(relay_switch.local_pose.translation_micrometres[0], 0);
    assert_eq!(relay_switch.local_pose.translation_micrometres[2], 0);
    assert!(
        pickup_half_extents[0] <= switch_half_extents[0]
            && pickup_half_extents[2] <= switch_half_extents[2]
    );

    let outcome =
        next_reference_game::run_reference_game(activated, true).expect("complete reference flow");
    let defeated_enemy = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.npc_character_id)
        .expect("defeated enemy binding");
    assert!(defeated_enemy.visible);
    assert_eq!(
        defeated_enemy.material_revision.asset_id,
        AssetId::from_bytes([0xd8; 16])
    );
    let collected_pickup = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.pickup_item_id)
        .expect("collected pickup binding");
    assert!(!collected_pickup.visible);
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_player_can_walk_beyond_the_former_twenty_metre_wall() {
    let (root, activated) = activated_reference_project("relay-traversal");
    let mut driver =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("live driver");

    hold_key(
        &mut driver,
        next_contracts::input::KEYBOARD_S_CONTROL_PATH_ID,
        0,
        5,
    );
    hold_key(
        &mut driver,
        next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
        2,
        22,
    );
    hold_key(
        &mut driver,
        next_contracts::input::KEYBOARD_W_CONTROL_PATH_ID,
        4,
        17,
    );

    let body_id = PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x54; 16]),
        body_slot: 0,
    };
    let pose = driver
        .state()
        .expect("live state")
        .checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states[&body_id]
        .pose
        .translation_micrometres;
    assert_eq!(pose, [2_200_000, 900_000, 1_200_000]);
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn each_large_rock_has_an_inset_solid_proxy_and_keeps_the_central_route_open() {
    let (root, activated) = activated_reference_project("rock-collider-parity");
    let session = next_reference_game::build_reference_game_session(activated.project.clone())
        .expect("reference session");
    let expected = [
        (0x71, [-2_800_000, 520_000, -1_800_000]),
        (0x72, [2_800_000, 520_000, -1_400_000]),
        (0x73, [-3_000_000, 520_000, 2_500_000]),
        (0x74, [3_000_000, 520_000, 2_800_000]),
    ];
    for (persistent_byte, translation) in expected {
        let body_id = PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([persistent_byte; 16]),
            body_slot: 0,
        };
        let body = &session.bootstrap.physics_checkpoint.catalog.bodies[&body_id];
        assert_eq!(body.initial_pose.translation_micrometres, translation);
        let shape = body.shapes.values().next().expect("one rock shape");
        let PhysicsGeometryV1::Box {
            half_extents_micrometres,
        } = shape.geometry
        else {
            panic!("rock proxy must be a box")
        };
        assert_eq!(half_extents_micrometres, [320_000, 500_000, 300_000]);
    }

    let movement_cases = [
        (
            next_contracts::input::KEYBOARD_S_CONTROL_PATH_ID,
            18,
            next_contracts::input::KEYBOARD_A_CONTROL_PATH_ID,
            40,
            0,
            -2_500_000,
            true,
        ),
        (
            next_contracts::input::KEYBOARD_S_CONTROL_PATH_ID,
            14,
            next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
            40,
            0,
            2_500_000,
            false,
        ),
        (
            next_contracts::input::KEYBOARD_W_CONTROL_PATH_ID,
            25,
            next_contracts::input::KEYBOARD_A_CONTROL_PATH_ID,
            40,
            0,
            -2_700_000,
            true,
        ),
        (
            next_contracts::input::KEYBOARD_W_CONTROL_PATH_ID,
            28,
            next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
            40,
            0,
            2_700_000,
            false,
        ),
    ];
    for (first_key, first_ticks, second_key, second_ticks, axis, boundary, greater) in
        movement_cases
    {
        let mut driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
            .expect("live driver");
        hold_key(&mut driver, first_key, 0, first_ticks);
        hold_key(&mut driver, second_key, 2, second_ticks);
        let pose = driver
            .state()
            .expect("live state")
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&PhysicsBodyIdV1 {
            subject_id: PersistentId::from_bytes([0x54; 16]),
            body_slot: 0,
        }]
            .pose
            .translation_micrometres;
        assert_eq!(pose[1], 900_000);
        if greater {
            assert!(
                pose[axis] > boundary,
                "player entered rock proxy at {pose:?}"
            );
        } else {
            assert!(
                pose[axis] < boundary,
                "player entered rock proxy at {pose:?}"
            );
        }
    }
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn production_capsule_course_traverses_and_restores_mid_push() {
    let (root, activated) = activated_reference_project("r5b-capsule-course");
    let session = next_reference_game::build_reference_game_session(activated.project.clone())
        .expect("reference session");
    let player_body_id = session.physics_body_id;
    let course = session.r5b_course;
    let mut driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("live driver");
    hold_key(
        &mut driver,
        next_contracts::input::KEYBOARD_S_CONTROL_PATH_ID,
        0,
        60,
    );

    let mut maximum_height = i64::MIN;
    let mut recovered_from_fall = false;
    let mut sensor_seen = false;
    let mut sensor_exited = false;
    {
        let mut sample = |driver: &next_reference_game::ReferenceGameDriverV2| {
            let state = driver.state().expect("live state");
            let physics = &state.checkpoint.physics_checkpoint.snapshot;
            let player = &physics.sorted_body_states[&player_body_id];
            maximum_height = maximum_height.max(player.pose.translation_micrometres[1]);
            if maximum_height >= 1_500_000
                && player.pose.translation_micrometres[1] == 900_000
                && player.linear_velocity_micrometres_per_second[1] == 0
            {
                recovered_from_fall = true;
            }
            let sensor_now = physics
                .sorted_contact_continuity_states
                .values()
                .any(|contact| {
                    contact.participant_low == course.sensor_shape_id
                        || contact.participant_high == course.sensor_shape_id
                });
            sensor_seen |= sensor_now;
            sensor_exited |= sensor_seen && !sensor_now;
        };

        driver
            .advance(&[control_event(
                next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                i16::MAX,
                2,
            )])
            .expect("start course traversal");
        sample(&driver);
        for _ in 1..55 {
            driver.advance(&[]).expect("continue course traversal");
            sample(&driver);
        }
    }

    let saved = driver.state().expect("mid-push state");
    assert_eq!(
        saved
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&course.dynamic_body_id]
            .linear_velocity_micrometres_per_second[0],
        3_000_000,
        "the saved continuation must capture an active committed push"
    );
    let mut restored = next_reference_game::ReferenceGameDriverV2::restore(
        activated.clone(),
        saved.checkpoint,
        saved.world_streaming_snapshot,
        saved.world_routine_snapshot_or_none,
        saved.world_population_snapshot,
        saved.world_activity_snapshot,
        saved.agent_cognition_snapshot,
        saved.agent_memory_snapshot,
        saved.physical_animation_snapshot,
        saved.driver_recovery,
    )
    .expect("restore mid-push course state");
    for _ in 55..80 {
        driver.advance(&[]).expect("continue original traversal");
        restored.advance(&[]).expect("continue restored traversal");
        let original = driver.state().expect("original state");
        let recovered = restored.state().expect("restored state");
        assert_eq!(
            recovered.checkpoint.physics_checkpoint,
            original.checkpoint.physics_checkpoint
        );
        let physics = &original.checkpoint.physics_checkpoint.snapshot;
        let player = &physics.sorted_body_states[&player_body_id];
        maximum_height = maximum_height.max(player.pose.translation_micrometres[1]);
        if maximum_height >= 1_500_000
            && player.pose.translation_micrometres[1] == 900_000
            && player.linear_velocity_micrometres_per_second[1] == 0
        {
            recovered_from_fall = true;
        }
        let sensor_now = physics
            .sorted_contact_continuity_states
            .values()
            .any(|contact| {
                contact.participant_low == course.sensor_shape_id
                    || contact.participant_high == course.sensor_shape_id
            });
        sensor_seen |= sensor_now;
        sensor_exited |= sensor_seen && !sensor_now;
    }
    let completion = control_event(
        next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
        0,
        3,
    );
    driver
        .advance(std::slice::from_ref(&completion))
        .expect("complete original traversal");
    restored
        .advance(&[completion])
        .expect("complete restored traversal");
    let original = driver.state().expect("final original state");
    let recovered = restored.state().expect("final restored state");
    assert_eq!(
        recovered.checkpoint.physics_checkpoint,
        original.checkpoint.physics_checkpoint
    );
    let physics = &original.checkpoint.physics_checkpoint.snapshot;
    let player = &physics.sorted_body_states[&player_body_id];
    let dynamic = &physics.sorted_body_states[&course.dynamic_body_id];
    assert_eq!(maximum_height, 1_500_000);
    assert!(recovered_from_fall);
    assert!(sensor_seen && sensor_exited);
    assert_eq!(
        player.pose.translation_micrometres,
        [6_000_000, 900_000, -6_000_000]
    );
    assert_eq!(dynamic.pose.translation_micrometres[0], 6_500_000);
    assert_eq!(dynamic.linear_velocity_micrometres_per_second, [0; 3]);
    let push_box = driver
        .presentation_snapshot()
        .expect("final presentation")
        .scene_records()
        .find(|record| record.object_key.persistent_id == course.dynamic_body_id.subject_id)
        .expect("visible push box");
    assert_eq!(
        push_box.current_transform.translation_micrometres,
        dynamic.pose.translation_micrometres
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn production_procedural_motor_restores_inside_fall_recovery() {
    let (root, activated) = activated_reference_project("r5e-procedural-recovery");
    let session = next_reference_game::build_reference_game_session(activated.project.clone())
        .expect("reference session");
    let player_body_id = session.physics_body_id;
    assert_eq!(
        session.procedural_motor.body_projection_root(),
        session
            .body_projections
            .player
            .roots
            .projection_root()
            .expect("player projection root")
    );
    assert_eq!(
        session.procedural_motor.actuator_safety_root(),
        session.body_projections.player.roots.actuator_safety_root
    );

    let mut driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("live driver");
    hold_key(
        &mut driver,
        next_contracts::input::KEYBOARD_S_CONTROL_PATH_ID,
        0,
        60,
    );
    driver
        .advance(&[control_event(
            next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            i16::MAX,
            2,
        )])
        .expect("start traversal");
    let saved = (0..60)
        .find_map(|_| {
            let state = driver.state().expect("candidate recovery state");
            let body = &state
                .checkpoint
                .physics_checkpoint
                .snapshot
                .sorted_body_states[&player_body_id];
            if body.linear_velocity_micrometres_per_second[1] != 0 {
                Some(state)
            } else {
                driver.advance(&[]).expect("seek fall recovery");
                None
            }
        })
        .expect("production course must enter fall recovery");
    let saved_body = &saved
        .checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states[&player_body_id];
    let recovery = session
        .procedural_motor
        .evaluate(&saved.checkpoint.physics_checkpoint.snapshot, [i16::MAX, 0])
        .expect("recovery decision");
    assert_eq!(
        recovery.route,
        next_motor::CapsuleMotorRouteV1::ProceduralRecovery
    );
    assert_eq!(recovery.applied_direction_q15, [0, 0]);
    assert_eq!(recovery.clamp_mask, 0b01);
    assert_eq!(recovery.source_body_revision, saved_body.body_revision);
    let saved_x = saved_body.pose.translation_micrometres[0];

    let mut restored = next_reference_game::ReferenceGameDriverV2::restore(
        activated,
        saved.checkpoint,
        saved.world_streaming_snapshot,
        saved.world_routine_snapshot_or_none,
        saved.world_population_snapshot,
        saved.world_activity_snapshot,
        saved.agent_cognition_snapshot,
        saved.agent_memory_snapshot,
        saved.physical_animation_snapshot,
        saved.driver_recovery,
    )
    .expect("restore during procedural recovery");
    driver.advance(&[]).expect("continue original recovery");
    restored.advance(&[]).expect("continue restored recovery");
    let original = driver.state().expect("original recovery state");
    let recovered = restored.state().expect("restored recovery state");
    assert_eq!(recovered.checkpoint, original.checkpoint);
    assert_eq!(
        original
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&player_body_id]
            .pose
            .translation_micrometres[0],
        saved_x,
        "procedural recovery must remove horizontal air steering"
    );

    let mut resumed = false;
    for _ in 0..40 {
        let before = driver.state().expect("before recovery continuation");
        let before_body = &before
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&player_body_id];
        let before_x = before_body.pose.translation_micrometres[0];
        let before_vertical = before_body.linear_velocity_micrometres_per_second[1];
        driver.advance(&[]).expect("continue original traversal");
        restored.advance(&[]).expect("continue restored traversal");
        let original = driver.state().expect("original continuation");
        let recovered = restored.state().expect("restored continuation");
        assert_eq!(recovered.checkpoint, original.checkpoint);
        let after_x = original
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&player_body_id]
            .pose
            .translation_micrometres[0];
        if before_vertical != 0 {
            assert_eq!(after_x, before_x);
        } else if after_x > before_x {
            resumed = true;
            break;
        }
    }
    assert!(
        resumed,
        "stable landing must resume the held cardinal command"
    );
    let completion = control_event(
        next_contracts::input::KEYBOARD_D_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
        0,
        3,
    );
    driver
        .advance(std::slice::from_ref(&completion))
        .expect("complete original traversal");
    restored
        .advance(&[completion])
        .expect("complete restored traversal");
    assert_eq!(
        restored.state().expect("restored final").checkpoint,
        driver.state().expect("original final").checkpoint
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

fn activated_reference_project(
    label: &str,
) -> (std::path::PathBuf, next_project::ActivatedProjectPackage) {
    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-collision-{label}-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    (root, activated)
}

fn hold_key(
    driver: &mut next_reference_game::ReferenceGameDriverV2,
    control_path: &str,
    source_sequence: u64,
    ticks: usize,
) {
    assert!(ticks > 0);
    driver
        .advance(&[control_event(
            control_path,
            NormalizedControlPhaseV1::Started,
            i16::MAX,
            source_sequence,
        )])
        .expect("start held movement");
    for _ in 1..ticks {
        driver.advance(&[]).expect("continue held movement");
    }
    driver
        .advance(&[control_event(
            control_path,
            NormalizedControlPhaseV1::Completed,
            0,
            source_sequence + 1,
        )])
        .expect("complete held movement");
}

fn control_event(
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: i16,
    source_sequence: u64,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(next_contracts::input::KEYBOARD_DEVICE_CLASS_ID).expect("device class"),
        PersistentId::from_bytes([0x76; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        vec![value],
        Vec::new(),
        0,
        source_sequence,
    )
    .expect("control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x77; 16]),
        SchemaId::new("nextengine.platform.source.collision-visibility-test").expect("source"),
        source_sequence,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(next_contracts::canonical::sha256(
            b"nextengine.platform.collision-visibility-test-capabilities.v1",
        )),
    )
    .expect("platform event")
}
