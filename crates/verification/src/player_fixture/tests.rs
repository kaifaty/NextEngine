use next_contracts::command::EventPayload;
use next_contracts::ids::{CommandLedgerHash, SchemaId};
use next_contracts::input::{
    CORE_INTERACT_ACTION_ID, CORE_MOVE_ACTION_ID, INPUT_SAMPLE_SCHEMA_VERSION, InputMappingCodeV1,
    InputSampleV1, PLAYER_ACTION_FRAME_SCHEMA_ID, PLAYER_ACTION_FRAME_SCHEMA_VERSION,
    PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1, PlayerActionPhaseV1, PlayerActionV1,
    PlayerActionValueV1,
};
use next_contracts::rpg::{
    CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, CORE_INTERACTIVE_OBJECT_READY_STATE_ID,
};
use next_contracts::rpg::{
    InteractiveObjectPayloadV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1,
    RpgEventV1, RpgSnapshotV2,
};
#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_physics_api::PhysicsBackendPolicy;
#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_runtime::PhysicsLaunchOptions;
use next_runtime::{RuntimeState, SnapshotRestoreError};
use next_world::{WorldRoutineOwnerV1, WorldStreamerV1};

use super::*;
use crate::scratch::ScratchContext;
use next_reference_game::aggregate_payload;

fn packaged_world_services_fixture(
    project_id: &str,
) -> (
    PreparedFixtureProjectPackage,
    NeutralPlayerFixture,
    WorldStreamerV1,
    WorldRoutineOwnerV1,
) {
    let scratch = ScratchContext::new(&std::env::temp_dir()).expect("test scratch root");
    let prepared = prepare_fixture_project_package_with_scratch(&scratch, project_id)
        .expect("prepared fixture package");
    let fixture =
        build_neutral_player_fixture_from_activated_project(prepared.package.project.clone())
            .expect("fixture");
    let world = WorldStreamerV1::activate(
        fixture.activated_project.clone(),
        prepared.package.content_generation.clone(),
        fixture.world_topology().initial_chunk_id().clone(),
    )
    .expect("world streamer");
    let routine =
        WorldRoutineOwnerV1::activate(fixture.activated_project.world_routine_catalog_or_none, 0)
            .expect("world routine");
    (prepared, fixture, world, routine)
}

fn run_joint_tick(
    runtime: &mut RuntimeState,
    routine: &mut WorldRoutineOwnerV1,
    world: &mut WorldStreamerV1,
) -> next_runtime::TickReport {
    let prepared = runtime
        .tick_preparation()
        .prepare_with_world_services([], routine, world)
        .expect("prepare joint tick");
    let validated = runtime
        .validate_prepared_world_services_tick(routine, world, prepared)
        .expect("validate joint tick");
    runtime
        .commit_validated_world_services_tick(routine, world, validated)
        .expect("commit joint tick")
        .runtime_report
}

#[test]
fn joint_world_services_commit_rejects_a_stale_candidate_without_partial_publication() {
    let (_prepared, fixture, mut world, mut routine) =
        packaged_world_services_fixture("nextengine.test.world-services-stale");
    let rpg_snapshot = cooked_project_rpg_snapshot(&fixture);
    let mut runtime =
        RuntimeState::with_rpg_snapshot(fixture.bootstrap, fixture.authority, rpg_snapshot)
            .expect("runtime");

    let first = runtime
        .tick_preparation()
        .prepare_with_world_services([], &routine, &world)
        .expect("first preparation");
    let second = runtime
        .tick_preparation()
        .prepare_with_world_services([], &routine, &world)
        .expect("second preparation");
    let first = runtime
        .validate_prepared_world_services_tick(&routine, &world, first)
        .expect("first validation");
    let second = runtime
        .validate_prepared_world_services_tick(&routine, &world, second)
        .expect("second validation");
    runtime
        .commit_validated_world_services_tick(&mut routine, &mut world, first)
        .expect("first joint commit");

    let runtime_after = runtime.snapshot();
    let routine_after = routine.snapshot_or_none().copied();
    let world_after = world.snapshot().clone();
    let error = runtime
        .commit_validated_world_services_tick(&mut routine, &mut world, second)
        .expect_err("second candidate is stale");

    assert_eq!(
        error.stable_code(),
        "PREPARED_WORLD_SERVICES_GENERATION_STALE"
    );
    assert_eq!(runtime.snapshot(), runtime_after);
    assert_eq!(routine.snapshot_or_none().copied(), routine_after);
    assert_eq!(world.snapshot(), &world_after);
}

#[test]
fn routine_snapshot_without_its_committed_ledger_receipt_fails_load_closure() {
    let (_prepared, fixture, _world, _routine) =
        packaged_world_services_fixture("nextengine.test.routine-ledger-closure");
    let rpg_snapshot = cooked_project_rpg_snapshot(&fixture);
    let catalog = fixture
        .activated_project
        .world_routine_catalog_or_none
        .expect("routine catalog");
    let mut runtime =
        RuntimeState::with_rpg_snapshot(fixture.bootstrap, fixture.authority, rpg_snapshot)
            .expect("runtime");
    for _ in 0..3 {
        runtime.run_tick([]).expect("runtime-only test tick");
    }
    let mut rest = next_contracts::world_routine::WorldRoutineSnapshotV1::initial(&catalog)
        .expect("initial routine snapshot");
    rest.record.record_revision = 1;
    rest.record.current_activity = next_contracts::world_routine::WorldRoutineActivityV1::Rest;
    let routine = WorldRoutineOwnerV1::restore(Some(catalog), Some(rest), runtime.next_tick())
        .expect("routine record is calendar-valid in isolation");

    assert!(matches!(
        runtime.validate_world_routine_ledger_closure(&routine),
        Err(SnapshotRestoreError::WorldRoutineLedgerClosureInvalid)
    ));
}

fn interactive_snapshot(fixture: &NeutralPlayerFixture, state: &str) -> RpgSnapshotV2 {
    RpgSnapshotV2 {
        aggregates: vec![fixture_aggregate(
            fixture.interactive_object_id,
            0x58,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: SchemaId::new(state).expect("interactive state"),
                linked_item_id: None,
            }),
        )],
    }
}

fn sample_with_actions(
    fixture: &NeutralPlayerFixture,
    sequence: u64,
    actions: Vec<PlayerActionV1>,
) -> InputSampleV1 {
    let frame = PlayerActionFrameV1 {
        schema_version: PLAYER_ACTION_FRAME_SCHEMA_VERSION,
        controller_id: fixture.controller_id,
        logical_frame_sequence: sequence,
        action_map_hash: fixture.action_map_hash,
        action_map_revision: 1,
        context_stack_hash: fixture.context_stack_hash,
        context_stack_revision: 1,
        actions,
    };
    InputSampleV1 {
        schema_version: INPUT_SAMPLE_SCHEMA_VERSION,
        source_class: SchemaId::new(PLAYER_ACTION_SOURCE_CLASS).expect("source class"),
        source_id: fixture.source_id,
        source_sequence: sequence,
        payload_schema_id: SchemaId::new(PLAYER_ACTION_FRAME_SCHEMA_ID).expect("frame schema"),
        payload_schema_version: u32::from(PLAYER_ACTION_FRAME_SCHEMA_VERSION),
        payload: frame.canonical_bytes().expect("canonical frame"),
        sampled_wall_time: None,
    }
}

fn ledger_hash(runtime: &RuntimeState) -> CommandLedgerHash {
    runtime
        .world_checkpoint()
        .expect("checkpoint")
        .runtime_snapshot
        .command_ledger_hash()
        .expect("ledger hash")
}

#[test]
#[cfg(any(feature = "physx", feature = "physx-mock"))]
fn prefer_physx_selects_physx_when_activation_succeeds() {
    let fixture = build_physx_player_fixture("nextengine.test.prefer-physx").expect("fixture");
    let runtime = RuntimeState::with_rpg_snapshot_and_physics_options(
        fixture.bootstrap,
        fixture.authority,
        RpgSnapshotV2::default(),
        PhysicsLaunchOptions::new(PhysicsBackendPolicy::PreferPhysXThenReference),
    )
    .expect("PhysX activation");
    assert_eq!(
        runtime.physics_backend_kind(),
        next_physics_api::PhysicsBackendKind::PhysX
    );
}

#[test]
fn interaction_query_reaches_an_eligible_target_without_contact() {
    let fixture =
        build_neutral_player_fixture("nextengine.test.interaction-no-contact").expect("fixture");
    let initial_rpg = interactive_snapshot(&fixture, CORE_INTERACTIVE_OBJECT_READY_STATE_ID);
    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg.clone(),
    )
    .expect("runtime");
    let ledger_before = ledger_hash(&runtime);
    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample"),
        )
        .expect("enqueue");
    let report = runtime.run_tick([]).expect("tick");
    assert_eq!(report.mapping_receipts.len(), 1);
    assert_eq!(
        report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(!report.mapping_receipts[0].derived_commands.is_empty());
    assert!(!report.results.is_empty());
    assert!(report.events.iter().any(|event| matches!(
        event.payload,
        EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned { .. })
    )));
    assert!(matches!(
        aggregate_payload(
            &runtime.rpg_snapshot(),
            RpgAggregateKindV1::InteractiveObject,
            fixture.interactive_object_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(payload))
            if payload.state_id.as_str() == CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
    ));
    assert_ne!(ledger_hash(&runtime), ledger_before);
}

#[test]
fn invalid_composite_and_colliding_interaction_inputs_keep_their_exact_boundaries() {
    let fixture =
        build_neutral_player_fixture("nextengine.test.interaction-invalid").expect("fixture");
    let initial_rpg = interactive_snapshot(&fixture, CORE_INTERACTIVE_OBJECT_READY_STATE_ID);
    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        initial_rpg.clone(),
    )
    .expect("runtime");

    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, false, None)
                .expect("invalid interaction"),
        )
        .expect("enqueue invalid interaction");
    let invalid = runtime.run_tick([]).expect("invalid tick is nonfatal");
    assert_eq!(
        invalid.mapping_receipts[0].frame_code,
        InputMappingCodeV1::ValueOutOfProfile
    );

    let mixed = sample_with_actions(
        &fixture,
        1,
        vec![
            PlayerActionV1 {
                action_id: SchemaId::new(CORE_INTERACT_ACTION_ID).expect("interact ID"),
                phase: PlayerActionPhaseV1::Started,
                value: PlayerActionValueV1::Digital(true),
                semantic_occurrence_ordinal: 0,
            },
            PlayerActionV1 {
                action_id: SchemaId::new(CORE_MOVE_ACTION_ID).expect("move ID"),
                phase: PlayerActionPhaseV1::Performed,
                value: PlayerActionValueV1::Vector2Q15([0, 32_767]),
                semantic_occurrence_ordinal: 1,
            },
        ],
    );
    runtime
        .enqueue_input_sample(&fixture.principal, mixed)
        .expect("enqueue mixed frame");
    let mixed_report = runtime.run_tick([]).expect("mixed tick is nonfatal");
    assert_eq!(
        mixed_report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(!mixed_report.mapping_receipts[0].derived_commands.is_empty());
    assert!(mixed_report.events.iter().any(|event| matches!(
        event.payload,
        EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned { .. })
    )));
    assert_eq!(
        mixed_report.physics_snapshot.sorted_body_states[&fixture.physics_body_id]
            .pose
            .translation_micrometres,
        [0, 900_000, 100_000]
    );

    let interact = player_interact_sample(&fixture, 2, PlayerActionPhaseV1::Started, true, None)
        .expect("interaction sample");
    let movement =
        player_action_sample(&fixture, 2, PlayerActionPhaseV1::Started, [0, 32_767], None)
            .expect("movement sample");
    runtime
        .enqueue_input_sample(&fixture.principal, interact)
        .expect("enqueue interaction collision candidate");
    runtime
        .enqueue_input_sample(&fixture.principal, movement)
        .expect("enqueue movement collision candidate");
    let rpg_before_collision = runtime.rpg_snapshot();
    let ledger_before_collision = ledger_hash(&runtime);
    let collision = runtime.run_tick([]).expect("collision tick is nonfatal");
    assert!(collision.mapping_receipts.is_empty());
    assert_eq!(
        collision
            .closed_ingress_batch
            .body
            .equivalence_receipts
            .len(),
        1
    );
    assert_eq!(runtime.rpg_snapshot(), rpg_before_collision);
    assert_eq!(ledger_hash(&runtime), ledger_before_collision);
}

#[test]
fn committed_interaction_retry_after_restore_is_an_accepted_noop() {
    let fixture =
        build_neutral_player_fixture("nextengine.test.interaction-retry").expect("fixture");
    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        interactive_snapshot(&fixture, CORE_INTERACTIVE_OBJECT_READY_STATE_ID),
    )
    .expect("runtime");
    for sequence in 0_u64..4 {
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_action_sample(
                    &fixture,
                    sequence,
                    if sequence == 0 {
                        PlayerActionPhaseV1::Started
                    } else {
                        PlayerActionPhaseV1::Performed
                    },
                    [0, 32_767],
                    None,
                )
                .expect("movement sample"),
            )
            .expect("enqueue movement");
        let _ = runtime.run_tick([]).expect("movement tick");
    }
    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample"),
        )
        .expect("enqueue interaction");
    let committed = runtime.run_tick([]).expect("interaction tick");
    assert_eq!(
        committed
            .events
            .iter()
            .filter(|event| matches!(
                event.payload,
                EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned { .. })
            ))
            .count(),
        1
    );

    let checkpoint = runtime.world_checkpoint().expect("checkpoint");
    let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
        checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .expect("restore");
    let ledger_before = ledger_hash(&restored);
    let rpg_before = restored.rpg_snapshot();
    restored
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, Some(42))
                .expect("retry sample"),
        )
        .expect("enqueue retry");
    let retry = restored.run_tick([]).expect("retry tick");
    assert_eq!(
        retry.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(retry.mapping_receipts[0].derived_commands.is_empty());
    assert!(retry.results.is_empty());
    assert!(retry.events.is_empty());
    assert_eq!(restored.rpg_snapshot(), rpg_before);
    assert_eq!(ledger_hash(&restored), ledger_before);
}

#[test]
fn pickup_and_equip_retry_after_restore_do_not_duplicate_state_or_events() {
    let fixture =
        build_neutral_player_fixture("nextengine.test.pickup-equip-retry").expect("fixture");
    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(&fixture),
    )
    .expect("runtime");
    for sequence in 0_u64..4 {
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_action_sample(
                    &fixture,
                    sequence,
                    if sequence == 0 {
                        PlayerActionPhaseV1::Started
                    } else {
                        PlayerActionPhaseV1::Performed
                    },
                    [0, 32_767],
                    None,
                )
                .expect("movement sample"),
            )
            .expect("enqueue movement");
        runtime.run_tick([]).expect("movement tick");
    }
    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_pickup_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, None)
                .expect("pickup sample"),
        )
        .expect("enqueue pickup");
    let pickup = runtime.run_tick([]).expect("pickup tick");
    assert_eq!(
        pickup
            .events
            .iter()
            .filter(|event| matches!(event.payload, EventPayload::Rpg(_)))
            .count(),
        2
    );
    assert_eq!(pickup.rpg_plan_traces.len(), 1);

    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_equip_use_sample(&fixture, 5, PlayerActionPhaseV1::Started, true, None)
                .expect("equip sample"),
        )
        .expect("enqueue equip");
    let equip = runtime.run_tick([]).expect("equip tick");
    assert_eq!(
        equip
            .events
            .iter()
            .filter(|event| matches!(
                event.payload,
                EventPayload::Rpg(RpgEventV1::EquipmentAssigned { .. })
            ))
            .count(),
        1
    );

    let checkpoint = runtime.world_checkpoint().expect("checkpoint");
    let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
        checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .expect("restore");
    let state_before = restored.rpg_snapshot();
    let ledger_before = ledger_hash(&restored);

    for (sequence, sample) in [
        (
            4,
            player_pickup_sample(&fixture, 4, PlayerActionPhaseV1::Started, true, Some(42))
                .expect("pickup retry"),
        ),
        (
            5,
            player_equip_use_sample(&fixture, 5, PlayerActionPhaseV1::Started, true, Some(43))
                .expect("equip retry"),
        ),
    ] {
        restored
            .enqueue_input_sample(&fixture.principal, sample)
            .expect("enqueue retry");
        let retry = restored.run_tick([]).expect("retry tick");
        assert_eq!(retry.mapping_receipts[0].source_sequence, sequence);
        assert_eq!(
            retry.mapping_receipts[0].frame_code,
            InputMappingCodeV1::Accepted
        );
        assert!(retry.mapping_receipts[0].derived_commands.is_empty());
        assert!(retry.events.is_empty());
        assert!(retry.results.is_empty());
    }
    assert_eq!(restored.rpg_snapshot(), state_before);
    assert_eq!(ledger_hash(&restored), ledger_before);
}

#[test]
fn cooked_dialogue_uses_query_targeting_and_invalid_participant_closure_fails_activation() {
    let (_prepared, fixture, mut world, mut routine) =
        packaged_world_services_fixture("nextengine.test.dialogue-closure");
    let ready = cooked_project_rpg_snapshot(&fixture);
    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        ready.clone(),
    )
    .expect("ready core dialogue runtime");
    let ledger_before = ledger_hash(&runtime);
    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample"),
        )
        .expect("enqueue interaction");
    let report = run_joint_tick(&mut runtime, &mut routine, &mut world);
    assert_eq!(
        report.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(!report.mapping_receipts[0].derived_commands.is_empty());
    assert!(!report.results.is_empty());
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(
                event.payload,
                EventPayload::Rpg(
                    RpgEventV1::DialogueAdvanced { .. }
                        | RpgEventV1::QuestTransitioned { .. }
                        | RpgEventV1::RelationshipAdjusted { .. }
                )
            ))
            .count(),
        3
    );
    assert_ne!(runtime.rpg_snapshot(), ready);
    assert_ne!(ledger_hash(&runtime), ledger_before);

    let mut invalid = cooked_project_rpg_snapshot(&fixture);
    let dialogue = invalid
        .aggregates
        .iter_mut()
        .find(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::Dialogue
                && aggregate.persistent_id == fixture.dialogue_id
        })
        .expect("fixture dialogue");
    let RpgAggregatePayloadV1::Dialogue(mut payload) = dialogue.payload.clone() else {
        panic!("fixture dialogue payload");
    };
    payload.listener_id = fixture.npc_character_id;
    *dialogue = RpgAggregateEnvelopeV1::new(
        dialogue.persistent_id,
        dialogue.schema_version,
        dialogue.revision,
        dialogue.definition_ref.clone(),
        dialogue.provenance.clone(),
        RpgAggregatePayloadV1::Dialogue(payload),
    )
    .expect("mutated dialogue aggregate is canonical");
    assert!(matches!(
        RuntimeState::with_rpg_snapshot(fixture.bootstrap, fixture.authority, invalid),
        Err(SnapshotRestoreError::CoreInteractionClosure(_))
    ));
}

#[test]
fn nearest_query_selects_quest_giver_and_dialogue_transition_is_one_shot() {
    let (_prepared, fixture, mut world, mut routine) =
        packaged_world_services_fixture("nextengine.test.dialogue-tie-break");
    let (accepted_node, _, _, _) = cooked_initial_interaction_outcome(&fixture);
    let mut runtime = RuntimeState::with_rpg_snapshot(
        fixture.bootstrap.clone(),
        fixture.authority.clone(),
        cooked_project_rpg_snapshot(&fixture),
    )
    .expect("runtime");

    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample"),
        )
        .expect("enqueue first interaction");
    let dialogue = run_joint_tick(&mut runtime, &mut routine, &mut world);
    assert_eq!(
        dialogue
            .events
            .iter()
            .filter(|event| matches!(
                event.payload,
                EventPayload::Rpg(
                    RpgEventV1::DialogueAdvanced { .. }
                        | RpgEventV1::QuestTransitioned { .. }
                        | RpgEventV1::RelationshipAdjusted { .. }
                )
            ))
            .count(),
        3
    );
    assert!(matches!(
        aggregate_payload(
            &runtime.rpg_snapshot(),
            RpgAggregateKindV1::Dialogue,
            fixture.dialogue_id,
        ),
        Some(RpgAggregatePayloadV1::Dialogue(dialogue))
            if dialogue.node_id == accepted_node
    ));

    let checkpoint = runtime.world_checkpoint().expect("checkpoint");
    let routine_snapshot = routine.snapshot_or_none().copied();
    let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
        checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .expect("restore");
    let mut restored_routine = WorldRoutineOwnerV1::restore(
        fixture.activated_project.world_routine_catalog_or_none,
        routine_snapshot,
        restored.next_tick(),
    )
    .expect("restore routine");
    restored
        .validate_world_routine_ledger_closure(&restored_routine)
        .expect("restore routine ledger closure");
    let rpg_before = restored.rpg_snapshot();
    let ledger_before = ledger_hash(&restored);
    restored
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Started, true, Some(99))
                .expect("retry interaction"),
        )
        .expect("enqueue retry");
    let retry = run_joint_tick(&mut restored, &mut restored_routine, &mut world);
    assert_eq!(
        retry.mapping_receipts[0].frame_code,
        InputMappingCodeV1::Accepted
    );
    assert!(retry.mapping_receipts[0].derived_commands.is_empty());
    assert!(retry.results.is_empty());
    assert!(retry.events.is_empty());
    assert_eq!(restored.rpg_snapshot(), rpg_before);
    assert_eq!(ledger_hash(&restored), ledger_before);
}
