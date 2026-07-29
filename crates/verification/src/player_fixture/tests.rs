use next_contracts::{
    CORE_INTERACT_ACTION_ID, CORE_INTERACTIVE_OBJECT_READY_STATE_ID, CORE_MOVE_ACTION_ID,
    CommandLedgerHash, EventPayload, INPUT_SAMPLE_SCHEMA_VERSION, InputMappingCodeV1,
    InputSampleV1, InteractiveObjectPayloadV1, PLAYER_ACTION_FRAME_SCHEMA_ID,
    PLAYER_ACTION_FRAME_SCHEMA_VERSION, PLAYER_ACTION_SOURCE_CLASS, PlayerActionFrameV1,
    PlayerActionPhaseV1, PlayerActionV1, PlayerActionValueV1, RpgAggregateEnvelopeV1,
    RpgAggregateKindV1, RpgAggregatePayloadV1, RpgEventV1, RpgSnapshotV2, SchemaId,
};
#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_physics_api::PhysicsBackendPolicy;
#[cfg(any(feature = "physx", feature = "physx-mock"))]
use next_runtime::PhysicsLaunchOptions;
use next_runtime::{RuntimeState, SnapshotRestoreError};

use super::rpg::aggregate_payload;
use super::*;

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
fn interaction_without_eligible_contact_is_accepted_without_ledger_or_rpg_mutation() {
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
        report.mapping_receipts[0].code,
        InputMappingCodeV1::Accepted
    );
    assert_eq!(report.mapping_receipts[0].derived_command_id, None);
    assert!(report.results.is_empty());
    assert_eq!(runtime.rpg_snapshot(), initial_rpg);
    assert_eq!(ledger_hash(&runtime), ledger_before);
}

#[test]
fn invalid_mixed_and_colliding_interaction_input_never_mutates_rpg() {
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
            player_interact_sample(&fixture, 0, PlayerActionPhaseV1::Performed, true, None)
                .expect("invalid interaction"),
        )
        .expect("enqueue invalid interaction");
    let invalid = runtime.run_tick([]).expect("invalid tick is nonfatal");
    assert_eq!(
        invalid.mapping_receipts[0].code,
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
        mixed_report.mapping_receipts[0].code,
        InputMappingCodeV1::ActionUnmapped
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
    assert_eq!(runtime.rpg_snapshot(), initial_rpg);
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
    assert_eq!(retry.mapping_receipts[0].code, InputMappingCodeV1::Accepted);
    assert_eq!(retry.mapping_receipts[0].derived_command_id, None);
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
        assert_eq!(retry.mapping_receipts[0].code, InputMappingCodeV1::Accepted);
        assert_eq!(retry.mapping_receipts[0].derived_command_id, None);
        assert!(retry.events.is_empty());
        assert!(retry.results.is_empty());
    }
    assert_eq!(restored.rpg_snapshot(), state_before);
    assert_eq!(ledger_hash(&restored), ledger_before);
}

#[test]
fn cooked_dialogue_requires_contact_and_invalid_participant_closure_fails_activation() {
    let fixture =
        build_neutral_player_fixture("nextengine.test.dialogue-closure").expect("fixture");
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
    let report = runtime.run_tick([]).expect("contact-free interaction tick");
    assert_eq!(
        report.mapping_receipts[0].code,
        InputMappingCodeV1::Accepted
    );
    assert_eq!(report.mapping_receipts[0].derived_command_id, None);
    assert!(report.results.is_empty());
    assert_eq!(runtime.rpg_snapshot(), ready);
    assert_eq!(ledger_hash(&runtime), ledger_before);

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
fn same_contact_switch_precedes_npc_then_dialogue_transition_is_one_shot() {
    let fixture =
        build_neutral_player_fixture("nextengine.test.dialogue-tie-break").expect("fixture");
    let entry_node = fixture
        .activated_project
        .rpg_definitions
        .dialogues
        .first()
        .expect("dialogue definition")
        .entry_node_id
        .clone();
    let mut rpg = cooked_project_rpg_snapshot(&fixture);
    rpg.aggregates
        .iter_mut()
        .find(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::InteractiveObject
                && aggregate.persistent_id == fixture.interactive_object_id
        })
        .expect("fixture interactive object")
        .persistent_id = fixture.npc_character_id;
    let mut runtime =
        RuntimeState::with_rpg_snapshot(fixture.bootstrap.clone(), fixture.authority.clone(), rpg)
            .expect("runtime");
    let movement = [
        (PlayerActionPhaseV1::Started, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, 32_767]),
        (PlayerActionPhaseV1::Performed, [0, -32_767]),
        (PlayerActionPhaseV1::Started, [32_767, 0]),
        (PlayerActionPhaseV1::Performed, [32_767, 0]),
        (PlayerActionPhaseV1::Performed, [32_767, 0]),
    ];
    for (sequence, (phase, direction)) in movement.into_iter().enumerate() {
        let sequence = u64::try_from(sequence).expect("bounded test sequence");
        runtime
            .enqueue_input_sample(
                &fixture.principal,
                player_action_sample(&fixture, sequence, phase, direction, None)
                    .expect("movement sample"),
            )
            .expect("enqueue movement");
        let _ = runtime.run_tick([]).expect("movement tick");
    }

    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 8, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample"),
        )
        .expect("enqueue first interaction");
    let switch = runtime.run_tick([]).expect("switch wins tie");
    assert!(switch.events.iter().any(|event| matches!(
        event.payload,
        EventPayload::Rpg(RpgEventV1::InteractiveObjectTransitioned { .. })
    )));
    assert!(matches!(
        aggregate_payload(
            &runtime.rpg_snapshot(),
            RpgAggregateKindV1::Dialogue,
            fixture.dialogue_id,
        ),
        Some(RpgAggregatePayloadV1::Dialogue(dialogue))
            if dialogue.node_id == entry_node
    ));

    runtime
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 9, PlayerActionPhaseV1::Started, true, None)
                .expect("interaction sample"),
        )
        .expect("enqueue dialogue interaction");
    let dialogue = runtime.run_tick([]).expect("dialogue transition");
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

    let checkpoint = runtime.world_checkpoint().expect("checkpoint");
    let mut restored = RuntimeState::restore_world_checkpoint_with_definitions(
        checkpoint,
        fixture.authority.clone(),
        fixture.activated_project.rpg_definitions.clone(),
    )
    .expect("restore");
    let rpg_before = restored.rpg_snapshot();
    let ledger_before = ledger_hash(&restored);
    restored
        .enqueue_input_sample(
            &fixture.principal,
            player_interact_sample(&fixture, 9, PlayerActionPhaseV1::Started, true, Some(99))
                .expect("retry interaction"),
        )
        .expect("enqueue retry");
    let retry = restored.run_tick([]).expect("completed interaction retry");
    assert_eq!(retry.mapping_receipts[0].code, InputMappingCodeV1::Accepted);
    assert_eq!(retry.mapping_receipts[0].derived_command_id, None);
    assert!(retry.results.is_empty());
    assert!(retry.events.is_empty());
    assert_eq!(restored.rpg_snapshot(), rpg_before);
    assert_eq!(ledger_hash(&restored), ledger_before);
}
