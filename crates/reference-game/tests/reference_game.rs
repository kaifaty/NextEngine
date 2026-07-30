use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    CORE_MOVE_ACTION_ID, CORE_PICKUP_ACTION_ID, KEYBOARD_DEVICE_CLASS_ID,
    KEYBOARD_W_CONTROL_PATH_ID, MOUSE_DELTA_CONTROL_PATH_ID, MOUSE_DEVICE_CLASS_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn reference_source_recooks_byte_identically_and_runs_through_production_paths() {
    let first = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("first cook");
    let second = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("second cook");
    assert_eq!(first, second);
    assert_eq!(
        first.publication().expect("first publication"),
        second.publication().expect("second publication")
    );

    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-game-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    store
        .publish(&first.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let outcome = next_reference_game::run_reference_game(activated, true).expect("reference run");
    let checkpoint = outcome.runtime.world_checkpoint().expect("checkpoint");
    assert_eq!(outcome.ticks, 16);
    assert_eq!(outcome.events, 17);
    assert_eq!(outcome.rpg_events, 9);
    assert_eq!(outcome.world_streaming_snapshot.generation, 2);
    let query_reports = outcome
        .tick_reports
        .iter()
        .filter(|report| !report.physics_query_batch.requests.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(query_reports.len(), 3);
    for report in &outcome.tick_reports {
        report
            .physics_query_batch
            .validate()
            .expect("closed query batch");
        assert_eq!(
            report.targeting_intents.len(),
            report.physics_query_batch.requests.len()
        );
        assert_eq!(
            report.authoritative_targeting_queries.len(),
            report.physics_query_batch.requests.len()
        );
        assert_eq!(
            report.physics_query_results.len(),
            report.physics_query_batch.requests.len()
        );
        for (request, result) in report
            .physics_query_batch
            .requests
            .iter()
            .zip(&report.physics_query_results)
        {
            result
                .validate_against_request(request)
                .expect("query result is bound to its closed request");
        }
    }
    let (composite_report, composite_receipt) = outcome
        .tick_reports
        .iter()
        .find_map(|report| {
            report
                .mapping_receipts_v2
                .iter()
                .find(|receipt| {
                    receipt
                        .action_results
                        .iter()
                        .any(|action| action.action_id.as_str() == CORE_PICKUP_ACTION_ID)
                })
                .map(|receipt| (report, receipt))
        })
        .expect("held movement and pickup share one canonical action frame");
    assert_eq!(composite_receipt.action_results.len(), 2);
    assert_eq!(
        composite_receipt.action_results[0].action_id.as_str(),
        CORE_MOVE_ACTION_ID
    );
    assert_eq!(
        composite_receipt.action_results[1].action_id.as_str(),
        CORE_PICKUP_ACTION_ID
    );
    assert_eq!(
        composite_receipt
            .action_results
            .iter()
            .map(|action| (action.first_command_ordinal, action.command_count))
            .collect::<Vec<_>>(),
        vec![(0, 1), (1, 1)]
    );
    assert_eq!(
        composite_receipt
            .derived_commands
            .iter()
            .map(|command| (
                command.command_ordinal,
                command.source_action_ordinal,
                command.mapper_command_slot,
            ))
            .collect::<Vec<_>>(),
        vec![(0, 0, 0), (1, 1, 0)]
    );
    assert_eq!(
        composite_receipt.derived_commands[0].command_id,
        composite_report.command_batches[0].body.envelopes[0]
            .compute_command_id()
            .expect("movement command id")
    );
    assert_eq!(
        composite_receipt.derived_commands[1].command_id,
        composite_report.command_batches[1].body.envelopes[0]
            .compute_command_id()
            .expect("pickup command id")
    );
    assert_ne!(
        checkpoint.runtime_snapshot.command_ledger_hash(),
        Ok(next_contracts::ids::CommandLedgerHash::default())
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_normalized_controls_move_the_player_while_camera_input_stays_nonauthoritative() {
    let root = test_root("live-input");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");

    let mut movement = next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
        .expect("movement driver");
    let initial = movement.state().expect("initial state");
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

    let mut camera = next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
        .expect("camera driver");
    let mut neutral =
        next_reference_game::ReferenceGameDriverV1::new(activated, true).expect("neutral driver");
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

#[test]
fn live_composite_camera_and_gameplay_frame_preserves_gameplay_root() {
    let root = test_root("live-composite-camera-gameplay");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");

    let mut composite = next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
        .expect("composite driver");
    let mut gameplay =
        next_reference_game::ReferenceGameDriverV1::new(activated, true).expect("gameplay driver");
    let movement = control_event_with_sequence(
        KEYBOARD_DEVICE_CLASS_ID,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
        0,
    );
    let camera = control_event_with_sequence(
        MOUSE_DEVICE_CLASS_ID,
        MOUSE_DELTA_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Changed,
        vec![12, -7],
        1,
    );
    composite
        .advance(&[movement.clone(), camera])
        .expect("composite frame");
    gameplay.advance(&[movement]).expect("gameplay-only frame");

    let composite = composite.state().expect("composite state");
    let gameplay = gameplay.state().expect("gameplay state");
    assert_eq!(composite.checkpoint, gameplay.checkpoint);
    assert_ne!(
        composite.presentation_snapshot.canonical_hash,
        gameplay.presentation_snapshot.canonical_hash
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_recovery_republishes_sequence_zero_camera_cut_under_a_new_epoch() {
    let root = test_root("live-presentation-recovery-cut");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");

    let mut original = next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
        .expect("original driver");
    original.advance(&[]).expect("non-cut live frame");
    let persisted = original.state().expect("persisted live state");
    let persisted_snapshot = persisted.presentation_snapshot.clone();
    assert_eq!(persisted_snapshot.snapshot_sequence, 1);
    assert!(
        persisted_snapshot
            .camera_records()
            .all(|camera| !camera.cut)
    );

    let recovered = next_reference_game::ReferenceGameDriverV1::restore(
        activated,
        persisted.checkpoint.clone(),
        persisted.world_streaming_snapshot.clone(),
        persisted.driver_recovery.clone(),
    )
    .expect("recovered driver")
    .state()
    .expect("recovered state");
    let recovered_snapshot = recovered.presentation_snapshot;
    assert_eq!(recovered.checkpoint, persisted.checkpoint);
    assert_eq!(
        recovered.world_streaming_snapshot,
        persisted.world_streaming_snapshot
    );
    assert_ne!(
        recovered_snapshot.snapshot_epoch,
        persisted_snapshot.snapshot_epoch
    );
    assert_eq!(recovered_snapshot.snapshot_sequence, 0);
    assert_eq!(
        recovered_snapshot.presentation_profile_hash,
        next_reference_game::reference_b0_presentation_profile_hash()
    );
    assert!(
        recovered_snapshot
            .scene_records()
            .all(|record| record.object_key.snapshot_epoch == recovered_snapshot.snapshot_epoch)
    );
    assert!(recovered_snapshot.camera_records().all(|camera| {
        camera.snapshot_epoch == recovered_snapshot.snapshot_epoch
            && camera.cut
            && camera.previous_result_sample == camera.current_result_sample
    }));
    std::fs::remove_dir_all(root).expect("cleanup");
}

fn control_event(
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
) -> PlatformEventV1 {
    control_event_with_sequence(device_class, control_path, phase, value, 0)
}

fn control_event_with_sequence(
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
    source_sequence: u64,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(device_class).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        value,
        Vec::new(),
        0,
        source_sequence,
    )
    .expect("control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.reference-test").expect("source"),
        source_sequence,
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
