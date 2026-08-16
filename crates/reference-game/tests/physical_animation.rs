use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_W_CONTROL_PATH_ID, MOUSE_DELTA_CONTROL_PATH_ID,
    MOUSE_DEVICE_CLASS_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn live_normalized_controls_drive_player_animation_without_camera_authority() {
    let root = test_root("live-physical-animation");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");

    let mut movement = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("movement driver");
    let initial = movement.state().expect("initial state");
    assert_eq!(initial.physical_animation_snapshot.records.len(), 2);
    assert!(
        initial
            .physical_animation_snapshot
            .records
            .iter()
            .all(|record| {
                record.graph_state
                    == next_contracts::physical_animation::PhysicalAnimationGraphStateV1::Idle
            })
    );
    let body_id = next_contracts::physics::PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x54; 16]),
        body_slot: 0,
    };
    let initial_pose = initial
        .checkpoint
        .physics_checkpoint
        .snapshot
        .sorted_body_states[&body_id]
        .pose;
    movement
        .advance(&[control_event(
            KEYBOARD_DEVICE_CLASS_ID,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
        )])
        .expect("normalized movement");
    let moved = movement.state().expect("moved state");
    assert_eq!(moved.physical_animation_snapshot.next_simulation_tick, 1);
    assert_eq!(
        moved
            .physical_animation_snapshot
            .records
            .iter()
            .find(|record| record.subject_id == PersistentId::from_bytes([0x54; 16]))
            .expect("player animation record")
            .graph_state,
        next_contracts::physical_animation::PhysicalAnimationGraphStateV1::Locomotion
    );
    assert_eq!(
        moved
            .physical_animation_snapshot
            .records
            .iter()
            .find(|record| record.subject_id == PersistentId::from_bytes([0x59; 16]))
            .expect("NPC animation record")
            .graph_state,
        next_contracts::physical_animation::PhysicalAnimationGraphStateV1::Idle
    );
    assert_eq!(
        moved
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&body_id]
            .pose
            .translation_micrometres[2],
        initial_pose.translation_micrometres[2] + 100_000
    );

    let mut camera = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("camera driver");
    let mut neutral =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("neutral driver");
    camera
        .advance(&[control_event(
            MOUSE_DEVICE_CLASS_ID,
            MOUSE_DELTA_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Changed,
            vec![12, -7],
        )])
        .expect("camera frame");
    neutral.advance(&[]).expect("neutral frame");
    let camera = camera.state().expect("camera state");
    let neutral = neutral.state().expect("neutral state");
    let camera_record = camera
        .presentation_snapshot
        .camera_records()
        .next()
        .expect("typed camera record");
    let camera_offset = std::array::from_fn(|axis| {
        camera_record
            .current_result_sample
            .pose
            .translation_micrometres[axis]
            - camera_record.current_result_sample.focus_point_micrometres[axis]
    });
    assert_eq!(camera_offset, [446_593, 1_091_808, -3_838_100]);
    assert_eq!(
        camera.checkpoint.physics_checkpoint,
        neutral.checkpoint.physics_checkpoint
    );
    assert_eq!(
        camera.checkpoint.rpg_snapshot,
        neutral.checkpoint.rpg_snapshot
    );
    assert_eq!(
        camera.checkpoint.runtime_snapshot.command_ledger,
        neutral.checkpoint.runtime_snapshot.command_ledger
    );
    assert_eq!(camera.checkpoint, neutral.checkpoint);
    assert_ne!(
        camera.presentation_snapshot.canonical_hash,
        neutral.presentation_snapshot.canonical_hash
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

fn control_event(
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(device_class).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        value,
        Vec::new(),
        0,
        0,
    )
    .expect("control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.reference-test").expect("source"),
        0,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(next_contracts::canonical::sha256(
            b"nextengine.platform.reference-test-capabilities.v1",
        )),
    )
    .expect("platform event")
}

fn test_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-reference-game-{label}-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}
