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
mod live_ui;
mod reference_game_support;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn reference_source_recooks_byte_identically_and_runs_through_production_paths() {
    let first = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("first cook");
    let second = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
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
    let activated = next_project::activate_project_package(&store).expect("activate");
    let topology =
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&activated.project)
            .expect("manifest-driven topology");
    assert_eq!(
        topology.ordered_multiregion_route().len(),
        activated.project.world_partition.body.chunk_bindings.len()
    );
    assert_eq!(
        topology.ordered_multiregion_route()[0].chunk_id,
        *topology.initial_chunk_id()
    );
    assert_ne!(
        topology.initial_chunk_id(),
        topology.gameplay_target_chunk_id()
    );
    assert_reference_topology_faults_are_typed(&activated.project, &topology);
    let courier_id = activated
        .project
        .world_population_catalog
        .courier_subject_id;
    let courier_goal_node = activated
        .project
        .world_population_catalog
        .definition(courier_id)
        .expect("courier definition")
        .navigation_goal_node_id
        .clone();
    let activity_catalog = activated.project.world_activity_catalog.clone();
    let outcome = next_reference_game::run_reference_game(activated, true).expect("reference run");
    let checkpoint = outcome.runtime.world_checkpoint().expect("checkpoint");
    assert_eq!(outcome.ticks, 32);
    assert_eq!(outcome.events, 52);
    assert_eq!(outcome.rpg_events, 23);
    assert_eq!(outcome.agent_cognition_snapshot.revision, 4);
    assert_eq!(outcome.agent_memory_snapshot.revision, 4);
    assert_eq!(outcome.decision_traces.len(), 4);
    assert_eq!(
        outcome
            .decision_traces
            .iter()
            .map(|trace| trace.gameplay_tick)
            .collect::<Vec<_>>(),
        vec![1, 4, 5, 6]
    );
    assert_eq!(
        outcome.decision_traces[1].switch_reason,
        next_contracts::cognition::DecisionSwitchReasonV1::EmergencyInterrupt
    );
    assert_eq!(
        outcome.decision_traces[2].switch_reason,
        next_contracts::cognition::DecisionSwitchReasonV1::EmergencyExitResume
    );
    assert!(outcome.decision_traces.iter().all(|trace| {
        trace.planning_failure == next_contracts::cognition::PlanningFailureV1::None
            && trace.intent_id_or_none.is_some()
    }));
    assert_eq!(
        outcome
            .tick_reports
            .iter()
            .flat_map(|report| &report.events)
            .filter(|event| matches!(
                event.payload,
                next_contracts::command::EventPayload::AgentCognition(_)
            ))
            .count(),
        4
    );
    assert!(
        outcome
            .world_services_tick_commits
            .iter()
            .all(|commit| commit.systemic_failure_or_none.is_none())
    );
    let activity = outcome
        .world_activity_snapshot_or_none
        .as_ref()
        .expect("systemic activity projection");
    assert_eq!(activity.record_revision, 3);
    assert_eq!(
        activity.state,
        next_contracts::world_activity::WorldActivityStateV1::Completed
    );
    assert_eq!(activity.assigned_tick_or_none, Some(3));
    assert_eq!(activity.work_started_tick_or_none, Some(4));
    assert_eq!(activity.completed_tick_or_none, Some(5));
    assert_eq!(outcome.agent_memory_snapshot.recorded_speech_acts.len(), 5);
    for kind in [
        next_contracts::cognition::SpeechActKindV1::Ask,
        next_contracts::cognition::SpeechActKindV1::Inform,
        next_contracts::cognition::SpeechActKindV1::Offer,
        next_contracts::cognition::SpeechActKindV1::Accept,
        next_contracts::cognition::SpeechActKindV1::Threaten,
    ] {
        assert!(
            outcome
                .agent_memory_snapshot
                .recorded_speech_acts
                .iter()
                .any(|act| act.kind == kind)
        );
    }
    let systemic_rpg = outcome.runtime.rpg_snapshot();
    assert!(matches!(
        next_reference_game::aggregate_payload(
            &systemic_rpg,
            next_contracts::rpg::RpgAggregateKindV1::Commitment,
            activity_catalog.commitment_id,
        ),
        Some(next_contracts::rpg::RpgAggregatePayloadV1::Commitment(commitment))
            if commitment.state == next_contracts::rpg::CommitmentStateV1::Fulfilled
    ));
    let resource_value = |character_id: PersistentId, resource_id: &SchemaId| {
        let Some(next_contracts::rpg::RpgAggregatePayloadV1::Character(character)) =
            next_reference_game::aggregate_payload(
                &systemic_rpg,
                next_contracts::rpg::RpgAggregateKindV1::Character,
                character_id,
            )
        else {
            panic!("systemic character aggregate");
        };
        character
            .resources
            .iter()
            .find(|resource| &resource.resource_id == resource_id)
            .expect("systemic resource")
            .current_value
    };
    let profile = &activity_catalog.systemic_work;
    assert_eq!(
        resource_value(
            activity_catalog.worker_subject_id,
            &profile.currency_resource_id
        ),
        profile.wage_amount - profile.food_price
    );
    assert_eq!(
        resource_value(profile.employer_character_id, &profile.currency_resource_id),
        100 - profile.wage_amount
    );
    assert_eq!(
        resource_value(profile.seller_character_id, &profile.currency_resource_id),
        profile.food_price
    );
    assert_eq!(
        resource_value(
            activity_catalog.worker_subject_id,
            &profile.hunger_resource_id
        ),
        0
    );
    assert_eq!(
        resource_value(
            activity_catalog.worker_subject_id,
            &profile.satiety_resource_id
        ),
        profile.satiety_gain_amount
    );
    for inventory_id in [profile.worker_inventory_id, profile.seller_inventory_id] {
        assert!(matches!(
            next_reference_game::aggregate_payload(
                &systemic_rpg,
                next_contracts::rpg::RpgAggregateKindV1::Inventory,
                inventory_id,
            ),
            Some(next_contracts::rpg::RpgAggregatePayloadV1::Inventory(inventory))
                if inventory.item_ids.is_empty()
        ));
    }
    assert_eq!(outcome.world_streaming_snapshot.generation, 2);
    let courier = outcome
        .world_population_snapshot
        .record(courier_id)
        .expect("courier retains the same persistent identity");
    assert_eq!(courier.record_revision, 7);
    assert_eq!(
        courier.tier,
        next_contracts::world_population::PopulationTierV1::Dormant
    );
    assert_eq!(courier.current_node_id, courier_goal_node);
    assert_eq!(
        outcome
            .tick_reports
            .iter()
            .flat_map(|report| &report.events)
            .filter(|event| matches!(
                event.payload,
                next_contracts::command::EventPayload::WorldPopulation(_)
            ))
            .count(),
        7
    );
    assert_eq!(
        outcome
            .world_services_tick_commits
            .iter()
            .filter(|commit| commit.population_service_report_or_none.is_some())
            .count(),
        usize::try_from(outcome.ticks).expect("bounded tick count")
    );
    reference_game_support::assert_world_services_stage_order(&outcome.tick_reports);
    assert_eq!(
        outcome
            .world_services_tick_commits
            .iter()
            .filter(|commit| commit.streaming_transition_or_none.is_some())
            .count(),
        2
    );
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
        composite_report.command_batches[1]
            .body
            .envelopes
            .iter()
            .find_map(|command| {
                let command_id = command.compute_command_id().ok()?;
                (command_id == composite_receipt.derived_commands[1].command_id)
                    .then_some(command_id)
            })
            .expect("pickup command id remains in the outcome batch")
    );
    assert_ne!(
        checkpoint.runtime_snapshot.command_ledger_hash(),
        Ok(next_contracts::ids::CommandLedgerHash::default())
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

fn assert_reference_topology_faults_are_typed(
    project: &next_contracts::project::ActivatedProjectV7,
    topology: &next_reference_game::ReferenceWorldTopologyV1,
) {
    let record_index = |project: &next_contracts::project::ActivatedProjectV7,
                        chunk_id: &SchemaId| {
        let asset_id = project
            .world_partition
            .body
            .chunk_bindings
            .iter()
            .find(|binding| &binding.chunk_id == chunk_id)
            .expect("topology chunk binding")
            .chunk_asset
            .asset_id;
        project
            .neutral_records
            .iter()
            .position(|record| record.asset_id == asset_id)
            .expect("topology chunk record")
    };

    let initial_index = record_index(project, topology.initial_chunk_id());
    let target_index = record_index(project, topology.gameplay_target_chunk_id());
    let role_index = |project: &next_contracts::project::ActivatedProjectV7, index: usize| {
        project.neutral_records[index]
            .properties
            .iter()
            .position(|property| property.property_id.as_str() == "nextengine.reference.role")
            .expect("reference role property")
    };

    let mut missing_role = project.clone();
    let initial_role_index = role_index(&missing_role, initial_index);
    missing_role.neutral_records[initial_index].properties[initial_role_index].value_id =
        SchemaId::new("nextengine.reference-alpha.world-chunk.unassigned").expect("role id");
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&missing_role),
        Err(next_reference_game::ReferenceGameError::WorldChunkRoleMissing("initial"))
    ));

    let mut duplicate_role = project.clone();
    let target_role_index = role_index(&duplicate_role, target_index);
    duplicate_role.neutral_records[target_index].properties[target_role_index].value_id =
        SchemaId::new("nextengine.reference-alpha.world-chunk.relay-station").expect("role id");
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&duplicate_role),
        Err(next_reference_game::ReferenceGameError::WorldChunkRoleDuplicate("initial"))
    ));

    let mut wrong_class = project.clone();
    wrong_class.neutral_records[initial_index].kind =
        next_contracts::content::NeutralRecordKindV1::Collider;
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&wrong_class),
        Err(next_reference_game::ReferenceGameError::WorldChunkRecordKindMismatch)
    ));

    let mut missing_record = project.clone();
    missing_record.neutral_records.remove(initial_index);
    assert!(matches!(
        next_reference_game::ReferenceWorldTopologyV1::from_activated_project(&missing_record),
        Err(next_reference_game::ReferenceGameError::WorldChunkRecordMissing)
    ));
}

#[test]
fn prepared_live_advance_preserves_driver_and_commits_its_exact_preview() {
    let root = test_root("prepared-live-advance");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut prepared_driver =
        next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
            .expect("prepared driver");
    let mut ordinary_driver =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("ordinary driver");
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
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut driver =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("live driver");
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
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let driver =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("live driver");
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

#[test]
fn live_driver_continues_after_quantized_corner_contact() {
    let root = test_root("live-corner-contact");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut driver =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("live driver");

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
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");

    let mut composite = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("composite driver");
    let mut gameplay =
        next_reference_game::ReferenceGameDriverV2::new(activated, true).expect("gameplay driver");
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
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");

    let mut original = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
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

    let recovered = next_reference_game::ReferenceGameDriverV2::restore(
        activated,
        persisted.checkpoint.clone(),
        persisted.world_streaming_snapshot.clone(),
        persisted.world_routine_snapshot_or_none,
        persisted.world_population_snapshot,
        persisted.world_activity_snapshot,
        persisted.agent_cognition_snapshot,
        persisted.agent_memory_snapshot,
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
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");

    let mut driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
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
    let en_resolver = reference_game_support::text_resolver(&activated, "en");
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
    let pseudo_resolver = reference_game_support::text_resolver(&activated, "qps-ploc");
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
    let recovered = next_reference_game::ReferenceGameDriverV2::restore(
        activated.clone(),
        persisted.checkpoint.clone(),
        persisted.world_streaming_snapshot.clone(),
        persisted.world_routine_snapshot_or_none,
        persisted.world_population_snapshot,
        persisted.world_activity_snapshot,
        persisted.agent_cognition_snapshot,
        persisted.agent_memory_snapshot,
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

    // R4d always activates the owner-complete systemic RPG closure. The
    // legacy interaction toggle may suppress host input in callers, but it
    // cannot remove authoritative sources or their read-only HUD projection.
    let non_interactive = next_reference_game::ReferenceGameDriverV2::new(activated, false)
        .expect("non-interactive driver")
        .state()
        .expect("non-interactive state");
    assert_eq!(
        non_interactive
            .presentation_snapshot
            .semantic_ui_records()
            .count(),
        3
    );
    assert_eq!(
        non_interactive
            .presentation_snapshot
            .semantic_ui_batches
            .len(),
        1
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

fn assert_live_state_eq(
    left: &next_reference_game::ReferenceLiveStateV2,
    right: &next_reference_game::ReferenceLiveStateV2,
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
