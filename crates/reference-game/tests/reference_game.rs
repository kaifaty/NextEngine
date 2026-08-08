use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    CORE_MOVE_ACTION_ID, CORE_PICKUP_ACTION_ID, KEYBOARD_D_CONTROL_PATH_ID,
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_ESCAPE_CONTROL_PATH_ID, KEYBOARD_I_CONTROL_PATH_ID,
    KEYBOARD_J_CONTROL_PATH_ID, KEYBOARD_S_CONTROL_PATH_ID, KEYBOARD_W_CONTROL_PATH_ID,
    MOUSE_DELTA_CONTROL_PATH_ID, MOUSE_DEVICE_CLASS_ID,
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
    assert_eq!(outcome.ticks, 32);
    assert_eq!(outcome.events, 27);
    assert_eq!(outcome.rpg_events, 13);
    assert_eq!(outcome.world_streaming_snapshot.generation, 2);
    let query_reports = outcome
        .tick_reports
        .iter()
        .filter(|report| !report.physics_query_batch.requests.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(query_reports.len(), 4);
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
                .mapping_receipts
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
fn prepared_live_advance_preserves_driver_and_commits_its_exact_preview() {
    let root = test_root("prepared-live-advance");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let mut prepared_driver =
        next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
            .expect("prepared driver");
    let mut ordinary_driver =
        next_reference_game::ReferenceGameDriverV1::new(activated, true).expect("ordinary driver");
    let event = control_event(
        KEYBOARD_DEVICE_CLASS_ID,
        KEYBOARD_W_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
        vec![i16::MAX],
    );
    let before = prepared_driver.state().expect("before state");

    let prepared = prepared_driver
        .stage_advance(std::slice::from_ref(&event))
        .expect("prepare advance");
    let after_prepare = prepared_driver.state().expect("unchanged state");
    assert_live_state_eq(&after_prepare, &before);
    let preview = prepared_driver
        .prepared_state(&prepared)
        .expect("prepared preview");
    let validated = prepared_driver
        .validate_prepared_advance(prepared)
        .expect("validate advance");
    let validated_preview = prepared_driver
        .validated_state(&validated)
        .expect("validated preview");
    assert_live_state_eq(&validated_preview, &preview);
    prepared_driver
        .commit_validated_advance(validated)
        .expect("commit validated advance");
    ordinary_driver
        .advance(&[event])
        .expect("ordinary compatible advance");

    assert_live_state_eq(
        &prepared_driver.state().expect("prepared committed state"),
        &preview,
    );
    assert_live_state_eq(
        &prepared_driver.state().expect("prepared committed state"),
        &ordinary_driver.state().expect("ordinary committed state"),
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn prepared_live_advance_rejects_a_stale_driver_generation() {
    let root = test_root("stale-prepared-live-advance");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let mut driver =
        next_reference_game::ReferenceGameDriverV1::new(activated, true).expect("live driver");
    let prepared = driver.stage_advance(&[]).expect("prepare advance");
    driver.advance(&[]).expect("advance current driver");

    let error = driver
        .validate_prepared_advance(prepared)
        .err()
        .expect("stale prepared advance");
    assert!(
        error
            .to_string()
            .contains("RUNTIME_PREPARED_GENERATION_STALE")
    );
    assert_eq!(driver.next_tick(), 1);
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn failed_live_staging_preserves_input_camera_ledger_and_physics() {
    let root = test_root("failed-live-staging");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let driver =
        next_reference_game::ReferenceGameDriverV1::new(activated, true).expect("live driver");
    let before = driver.state().expect("before state");
    let conflicting = [
        control_event_with_sequence(
            KEYBOARD_DEVICE_CLASS_ID,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
            0,
        ),
        control_event_with_sequence(
            KEYBOARD_DEVICE_CLASS_ID,
            KEYBOARD_W_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX / 2],
            0,
        ),
    ];

    driver
        .stage_advance(&conflicting)
        .err()
        .expect("identity collision must fail staging");
    assert_live_state_eq(&driver.state().expect("unchanged state"), &before);
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
fn live_driver_continues_after_quantized_corner_contact() {
    let root = test_root("live-corner-contact");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let mut driver =
        next_reference_game::ReferenceGameDriverV1::new(activated, true).expect("live driver");

    driver
        .advance(&[control_event_with_sequence(
            KEYBOARD_DEVICE_CLASS_ID,
            KEYBOARD_S_CONTROL_PATH_ID,
            NormalizedControlPhaseV1::Started,
            vec![i16::MAX],
            0,
        )])
        .expect("start moving backward");
    for _ in 0..3 {
        driver.advance(&[]).expect("continue moving backward");
    }
    driver
        .advance(&[
            control_event_with_sequence(
                KEYBOARD_DEVICE_CLASS_ID,
                KEYBOARD_S_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Completed,
                vec![0],
                1,
            ),
            control_event_with_sequence(
                KEYBOARD_DEVICE_CLASS_ID,
                KEYBOARD_D_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
                2,
            ),
        ])
        .expect("turn right");
    for _ in 0..3 {
        driver.advance(&[]).expect("continue moving right");
    }
    driver
        .advance(&[
            control_event_with_sequence(
                KEYBOARD_DEVICE_CLASS_ID,
                KEYBOARD_D_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Completed,
                vec![0],
                3,
            ),
            control_event_with_sequence(
                KEYBOARD_DEVICE_CLASS_ID,
                KEYBOARD_W_CONTROL_PATH_ID,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
                4,
            ),
        ])
        .expect("turn toward the obstacle");
    driver.advance(&[]).expect("approach the obstacle");
    driver
        .advance(&[])
        .expect("commit the quantized corner contact");
    driver
        .advance(&[])
        .expect("fork and continue from the corner-contact checkpoint");

    let state = driver.state().expect("live state");
    let body_id = next_contracts::physics::PhysicsBodyIdV1 {
        subject_id: PersistentId::from_bytes([0x54; 16]),
        body_slot: 0,
    };
    assert_eq!(
        state
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_body_states[&body_id]
            .pose
            .translation_micrometres,
        [400_000, 900_000, -123_607]
    );
    assert_eq!(
        state
            .checkpoint
            .physics_checkpoint
            .snapshot
            .sorted_contact_continuity_states
            .len(),
        2
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

#[test]
fn live_presentation_publishes_typed_semantic_ui_hud_from_rpg_state() {
    let root = test_root("live-semantic-ui-hud");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");

    let mut driver = next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
        .expect("live driver");
    let initial = driver.state().expect("initial state");
    let ui_records = initial
        .presentation_snapshot
        .semantic_ui_records()
        .collect::<Vec<_>>();
    assert_eq!(ui_records.len(), 3);
    // Screens publish only while their toggle state is open (S3): the
    // initial state shows the HUD alone.
    assert!(
        ui_records
            .iter()
            .all(|record| record.element.affordances.is_empty()),
        "the HUD stays read-only without action affordances"
    );
    let health = ui_records
        .iter()
        .find(|record| record.element.element_id.as_str() == "nextengine.ui.element.hud.health")
        .expect("health element");
    assert_eq!(health.surface_id.as_str(), "nextengine.ui.surface.hud");
    assert_eq!(
        health.semantic_path_id.as_str(),
        "nextengine.ui.panel.hud.status"
    );
    assert_eq!(
        health.element.role,
        next_contracts::presentation::UiElementRoleV1::Meter
    );
    assert_eq!(
        health.element.accessibility_role,
        next_contracts::presentation::UiAccessibilityRoleV1::Status
    );
    assert_eq!(
        health.element.value,
        next_contracts::presentation::UiElementValueV1::Scalar {
            current: 100,
            maximum: 100,
        }
    );
    let health_text = health.element.text_or_none.as_ref().expect("health text");
    assert_eq!(
        health_text.text_id.as_str(),
        "nextengine.ui.text.hud.health"
    );
    assert_eq!(
        health_text.arguments,
        vec![
            next_contracts::presentation::UiTextArgumentV1::SignedInteger(100),
            next_contracts::presentation::UiTextArgumentV1::SignedInteger(100),
        ]
    );
    let quest = ui_records
        .iter()
        .find(|record| record.element.element_id.as_str() == "nextengine.ui.element.hud.quest")
        .expect("quest element");
    assert_eq!(
        quest.element.role,
        next_contracts::presentation::UiElementRoleV1::Label
    );
    assert!(
        matches!(
            quest
                .element
                .text_or_none
                .as_ref()
                .expect("quest text")
                .arguments
                .as_slice(),
            [next_contracts::presentation::UiTextArgumentV1::TextId(_)]
        ),
        "quest label carries its state id as a text argument"
    );
    assert!(
        ui_records
            .iter()
            .all(|record| record.snapshot_epoch == initial.presentation_snapshot.snapshot_epoch)
    );

    // The cooked text catalogs resolve every emitted HUD reference
    // deterministically: the source locale directly, the pseudo-locale
    // through its declared `qps-ploc -> en` fallback chain, and quest state
    // TextId arguments recursively.
    let en_resolver =
        next_presentation::TextCatalogResolverV1::new(activated.text_catalogs.clone(), "en")
            .expect("en resolver");
    let health_ref = health.element.text_or_none.as_ref().expect("health text");
    let health_resolution = en_resolver.resolve(health_ref);
    assert_eq!(health_resolution.text, "Health 100/100");
    assert_eq!(health_resolution.diagnostic_or_none, None);
    let quest_resolution =
        en_resolver.resolve(quest.element.text_or_none.as_ref().expect("quest text"));
    assert_eq!(
        quest_resolution.text,
        "Objective - Frontier Relay: Available"
    );
    assert_eq!(quest_resolution.diagnostic_or_none, None);
    let pseudo_resolver =
        next_presentation::TextCatalogResolverV1::new(activated.text_catalogs.clone(), "qps-ploc")
            .expect("pseudo resolver");
    assert!(!pseudo_resolver.requested_locale_missing());
    assert_eq!(pseudo_resolver.resolve(health_ref).text, "⟦Ħēåłŧħ⟧ 100/100");
    // The pseudo catalog omits pause-menu.load on purpose: it falls back.
    let load_ref = next_contracts::presentation::UiTextRefV1::new(
        next_contracts::ids::SchemaId::new("nextengine.ui.text.pause-menu.load").expect("text id"),
        Vec::new(),
    )
    .expect("text ref");
    let load_resolution = pseudo_resolver.resolve(&load_ref);
    assert_eq!(load_resolution.text, "Load game");
    assert_eq!(load_resolution.diagnostic_or_none, None);

    // Recovery evidence round-trips the typed semantic UI batches: the
    // recovered driver rebuilds the same HUD under the recovery cut epoch.
    driver.advance(&[]).expect("non-cut live frame");
    let persisted = driver.state().expect("persisted state");
    let recovered = next_reference_game::ReferenceGameDriverV1::restore(
        activated.clone(),
        persisted.checkpoint.clone(),
        persisted.world_streaming_snapshot.clone(),
        persisted.driver_recovery.clone(),
    )
    .expect("recovered driver")
    .state()
    .expect("recovered state");
    let recovered_ui = recovered
        .presentation_snapshot
        .semantic_ui_records()
        .collect::<Vec<_>>();
    assert_eq!(recovered_ui.len(), 3);
    assert!(
        recovered_ui
            .iter()
            .all(|record| record.snapshot_epoch == recovered.presentation_snapshot.snapshot_epoch)
    );
    assert_ne!(
        recovered.presentation_snapshot.snapshot_epoch,
        persisted.presentation_snapshot.snapshot_epoch
    );

    // A non-interactive scenario has no RPG sources and publishes no
    // semantic UI batches.
    let non_interactive = next_reference_game::ReferenceGameDriverV1::new(activated, false)
        .expect("non-interactive driver")
        .state()
        .expect("non-interactive state");
    assert_eq!(
        non_interactive
            .presentation_snapshot
            .semantic_ui_records()
            .count(),
        0
    );
    assert!(
        non_interactive
            .presentation_snapshot
            .semantic_ui_batches
            .is_empty()
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_ui_screen_toggles_are_deterministic_and_back_closes_before_pause() {
    let root = test_root("live-ui-screen-toggles");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");

    let mut driver = next_reference_game::ReferenceGameDriverV1::new(activated.clone(), true)
        .expect("live driver");
    let en_resolver =
        next_presentation::TextCatalogResolverV1::new(activated.text_catalogs.clone(), "en")
            .expect("en resolver");

    let mut sequence = 0_u64;
    let mut key_event = |control_path: &'static str, phase: NormalizedControlPhaseV1| {
        sequence += 1;
        control_event_with_sequence(
            KEYBOARD_DEVICE_CLASS_ID,
            control_path,
            phase,
            vec![if phase == NormalizedControlPhaseV1::Started {
                i16::MAX
            } else {
                0
            }],
            sequence,
        )
    };
    fn record_ids(snapshot: &next_contracts::presentation::PresentationSnapshotV2) -> Vec<String> {
        snapshot
            .semantic_ui_records()
            .map(|record| record.element.element_id.as_str().to_owned())
            .collect()
    }
    const HUD_IDS: [&str; 3] = [
        "nextengine.ui.element.hud.action",
        "nextengine.ui.element.hud.health",
        "nextengine.ui.element.hud.quest",
    ];
    // `semantic_ui_records` yields the canonical element-id order.
    const INVENTORY_SCREEN_IDS: [&str; 4] = [
        "nextengine.ui.element.equipment.empty",
        "nextengine.ui.element.equipment.title",
        "nextengine.ui.element.inventory.empty",
        "nextengine.ui.element.inventory.title",
    ];
    const JOURNAL_SCREEN_IDS: [&str; 2] = [
        "nextengine.ui.element.quest-journal.entry.0",
        "nextengine.ui.element.quest-journal.title",
    ];
    const PAUSE_MENU_IDS: [&str; 4] = [
        "nextengine.ui.element.pause-menu.load",
        "nextengine.ui.element.pause-menu.resume",
        "nextengine.ui.element.pause-menu.save",
        "nextengine.ui.element.pause-menu.title",
    ];
    let expected = |screen: &[&str]| -> Vec<String> {
        HUD_IDS
            .iter()
            .chain(screen.iter())
            .map(|id| (*id).to_owned())
            .collect()
    };

    // Closed by default: HUD only.
    let initial = driver.presentation_snapshot().expect("initial snapshot");
    assert_eq!(record_ids(initial), expected(&[]));

    // `ui-inventory` opens the inventory/equipment screen read-only.
    let press = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("inventory open frame");
    let inventory_records = record_ids(snapshot);
    assert_eq!(inventory_records, expected(&INVENTORY_SCREEN_IDS));
    assert!(
        driver
            .presentation_snapshot()
            .expect("snapshot")
            .semantic_ui_records()
            .all(|record| record.element.affordances.is_empty()),
        "open screen stays affordance-free"
    );

    // A key release does not toggle; the next press closes the screen.
    let release = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    let snapshot = driver.advance(&[release]).expect("release frame");
    assert_eq!(record_ids(snapshot), expected(&INVENTORY_SCREEN_IDS));
    let press = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("inventory close frame");
    assert_eq!(record_ids(snapshot), expected(&[]));
    let release = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("release frame");

    // `ui-journal` opens the quest journal; its entry resolves through the
    // cooked catalogs with the quest display name and state.
    let press = key_event(
        KEYBOARD_J_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("journal open frame");
    assert_eq!(record_ids(snapshot), expected(&JOURNAL_SCREEN_IDS));
    let journal_entry = snapshot
        .semantic_ui_records()
        .find(|record| {
            record.element.element_id.as_str() == "nextengine.ui.element.quest-journal.entry.0"
        })
        .expect("journal entry element");
    let journal_resolution = en_resolver.resolve(
        journal_entry
            .element
            .text_or_none
            .as_ref()
            .expect("journal entry text"),
    );
    assert_eq!(journal_resolution.text, "Frontier Relay - Available");
    assert_eq!(journal_resolution.diagnostic_or_none, None);

    // Screens are exclusive: opening inventory replaces the journal.
    let press = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("screen switch frame");
    assert_eq!(record_ids(snapshot), expected(&INVENTORY_SCREEN_IDS));

    // `ui-back` with an open screen closes it and is consumed: no pause
    // suspend publication carries the pause-menu surface.
    let press = key_event(
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("screen close frame");
    assert_eq!(record_ids(snapshot), expected(&[]));

    // `ui-back` with no open screen requests the declared pause suspend.
    let release = key_event(
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("release frame");
    let press = key_event(
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("pause frame");
    assert_eq!(record_ids(snapshot), expected(&PAUSE_MENU_IDS));

    std::fs::remove_dir_all(root).expect("cleanup");
}

fn assert_live_state_eq(
    left: &next_reference_game::ReferenceLiveStateV1,
    right: &next_reference_game::ReferenceLiveStateV1,
) {
    assert_eq!(left.checkpoint, right.checkpoint);
    assert_eq!(
        left.checkpoint_canonical_components,
        right.checkpoint_canonical_components
    );
    assert_eq!(
        left.world_streaming_snapshot,
        right.world_streaming_snapshot
    );
    assert_eq!(left.ticks, right.ticks);
    assert_eq!(left.events, right.events);
    assert_eq!(left.rpg_events, right.rpg_events);
    assert_eq!(
        left.project_composition_lock_hash,
        right.project_composition_lock_hash
    );
    assert_eq!(left.content_manifest_hash, right.content_manifest_hash);
    assert_eq!(
        left.presentation_input_count,
        right.presentation_input_count
    );
    assert_eq!(left.presentation_snapshot, right.presentation_snapshot);
    assert_eq!(left.driver_recovery, right.driver_recovery);
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
