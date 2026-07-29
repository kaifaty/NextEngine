use next_assets::SaveImage;
use next_contracts::command::{EventPayload, IssuerPrincipal, WorldCommand};
use next_contracts::ids::{AssetId, ContentHash, PersistentId, PlayerPrincipalId, SchemaId};
use next_contracts::persistence::{AuthorityGrant, SaveCompatibility, TickSettings};
use next_contracts::physics::PhysicsWorldCheckpointV1;
use next_contracts::rpg::{
    CharacterPayloadV1, DefinitionRefV1, DialoguePayloadV1, InteractiveObjectPayloadV1,
    InventoryPayloadV1, ItemPayloadV1, ProvenanceBindingV1, QuestPayloadV1,
    RelationshipDimensionV1, RelationshipPayloadV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1,
    RpgAggregatePayloadV1, RpgAggregateRefV1, RpgCommandV1, RpgEventV1, RpgOperationPayloadV1,
    RpgOperationV1, RpgRuntimeBindingsV1, RpgSnapshotV2, SkillProficiencyEntryV1,
};
use next_contracts::rpg::{RPG_COMMAND_CAPABILITY_ID, SkillProficiency};
use next_contracts::snapshot::WorldCheckpointV4;
use next_runtime::{
    AuthorityRegistry, CommandDisposition, RejectionCode, RuntimeBootstrapV3, RuntimeState,
};
use next_verification::{
    NeutralRuntimeFixture, ReplayTickInput, RpgReplayInput, build_neutral_runtime_fixture,
    compute_world_checkpoint_root, run_rpg_replay,
};

fn id(value: u8) -> PersistentId {
    PersistentId::from_bytes([value; 16])
}

fn schema(value: &str) -> SchemaId {
    SchemaId::new(value).expect("fixture schema is valid")
}

fn principal() -> IssuerPrincipal {
    IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([9; 16]))
}

fn authority() -> AuthorityRegistry {
    let mut authority = AuthorityRegistry::new();
    authority
        .register(
            principal(),
            [
                next_contracts::ids::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                    .expect("RPG capability is valid"),
            ],
        )
        .expect("fixture principal is unique");
    authority
}

fn runtime_fixture() -> NeutralRuntimeFixture {
    build_neutral_runtime_fixture(
        "nextengine.rpg-slice",
        [(
            principal(),
            vec![
                next_contracts::ids::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                    .expect("RPG capability is valid"),
            ],
        )],
    )
    .expect("neutral RPG fixture")
}

fn bootstrap() -> RuntimeBootstrapV3 {
    runtime_fixture().bootstrap
}

fn aggregate(value: u8, payload: RpgAggregatePayloadV1) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        id(value),
        1,
        0,
        DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes([value; 16]),
            content_hash: ContentHash::from_bytes([value; 32]),
        },
        ProvenanceBindingV1::Exact(ContentHash::from_bytes([value; 32])),
        payload,
    )
    .expect("fixture aggregate is valid")
}

fn fixture() -> RpgSnapshotV2 {
    let mut aggregates = vec![
        aggregate(
            1,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(id(9)),
                equipment_id: None,
                resources: vec![],
                skills: vec![SkillProficiencyEntryV1 {
                    skill_id: schema("rpg.skill.survival"),
                    proficiency: SkillProficiency::new(400).expect("bounded proficiency"),
                }],
            }),
        ),
        aggregate(
            2,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(id(10)),
                equipment_id: None,
                resources: vec![],
                skills: vec![],
            }),
        ),
        aggregate(
            3,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: vec![],
            }),
        ),
        aggregate(
            9,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: id(1),
                capacity: 8,
                item_ids: vec![id(3)],
                reservations: vec![],
            }),
        ),
        aggregate(
            10,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: id(2),
                capacity: 8,
                item_ids: vec![],
                reservations: vec![],
            }),
        ),
        aggregate(
            4,
            RpgAggregatePayloadV1::Quest(QuestPayloadV1 {
                state_id: schema("rpg.quest-state.available"),
            }),
        ),
        aggregate(
            5,
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: id(1),
                listener_id: id(2),
                node_id: schema("rpg.dialogue-node.offer"),
            }),
        ),
        aggregate(
            6,
            RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                source_id: id(1),
                target_id: id(2),
                dimensions: vec![RelationshipDimensionV1 {
                    dimension_id: schema("rpg.relationship.trust"),
                    value: 0,
                }],
            }),
        ),
        aggregate(
            7,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: schema("rpg.object-state.off"),
                linked_item_id: None,
            }),
        ),
    ];
    aggregates.sort_by_key(|record| (record.aggregate_kind, record.persistent_id));
    RpgSnapshotV2 { aggregates }
}

fn target(
    aggregate_kind: RpgAggregateKindV1,
    persistent_id: PersistentId,
    expected_revision: u64,
) -> RpgAggregateRefV1 {
    RpgAggregateRefV1 {
        aggregate_kind,
        persistent_id,
        expected_revision,
    }
}

fn operation(
    operation_slot: u32,
    targets: Vec<RpgAggregateRefV1>,
    payload: RpgOperationPayloadV1,
) -> RpgOperationV1 {
    RpgOperationV1 {
        operation_slot,
        targets,
        definition_policy_hashes: vec![],
        payload,
    }
}

fn dialogue_quest_command(expected_quest_state: &str) -> RpgCommandV1 {
    RpgCommandV1 {
        operations: vec![
            operation(
                0,
                vec![target(RpgAggregateKindV1::Dialogue, id(5), 0)],
                RpgOperationPayloadV1::AdvanceDialogue {
                    dialogue_id: id(5),
                    expected_node_id: schema("rpg.dialogue-node.offer"),
                    next_node_id: schema("rpg.dialogue-node.accepted"),
                },
            ),
            operation(
                1,
                vec![target(RpgAggregateKindV1::Quest, id(4), 0)],
                RpgOperationPayloadV1::TransitionQuest {
                    quest_id: id(4),
                    expected_state_id: schema(expected_quest_state),
                    next_state_id: schema("rpg.quest-state.active"),
                },
            ),
            operation(
                2,
                vec![target(RpgAggregateKindV1::Relationship, id(6), 0)],
                RpgOperationPayloadV1::AdjustRelationship {
                    relationship_id: id(6),
                    dimension_id: schema("rpg.relationship.trust"),
                    delta: 7,
                },
            ),
        ],
    }
}

fn command(sequence: u64, tick: u64, payload: RpgCommandV1) -> WorldCommand {
    let fixture = runtime_fixture();
    WorldCommand::rpg(
        fixture
            .stream_for(&principal())
            .expect("fixture stream is allocated"),
        principal(),
        sequence,
        tick,
        payload,
    )
    .expect("fixture command is canonical")
}

fn slice_ticks() -> Vec<ReplayTickInput> {
    vec![
        ReplayTickInput {
            commands: vec![command(
                0,
                0,
                dialogue_quest_command("rpg.quest-state.available"),
            )],
        },
        ReplayTickInput {
            commands: vec![command(
                1,
                1,
                RpgCommandV1 {
                    operations: vec![operation(
                        0,
                        vec![
                            target(RpgAggregateKindV1::Item, id(3), 0),
                            target(RpgAggregateKindV1::Inventory, id(9), 0),
                            target(RpgAggregateKindV1::Inventory, id(10), 0),
                        ],
                        RpgOperationPayloadV1::TransferItem {
                            item_id: id(3),
                            source_inventory_id: Some(id(9)),
                            destination_inventory_id: Some(id(10)),
                            quantity: 1,
                        },
                    )],
                },
            )],
        },
        ReplayTickInput {
            commands: vec![command(
                2,
                2,
                RpgCommandV1 {
                    operations: vec![operation(
                        0,
                        vec![target(RpgAggregateKindV1::Character, id(2), 0)],
                        RpgOperationPayloadV1::SetSkillProficiency {
                            character_id: id(2),
                            skill_id: schema("rpg.skill.survival"),
                            expected_value: 0,
                            new_value: 250,
                        },
                    )],
                },
            )],
        },
        ReplayTickInput {
            commands: vec![command(
                3,
                3,
                RpgCommandV1 {
                    operations: vec![operation(
                        0,
                        vec![target(RpgAggregateKindV1::InteractiveObject, id(7), 0)],
                        RpgOperationPayloadV1::TransitionInteractiveObject {
                            object_id: id(7),
                            expected_state_id: schema("rpg.object-state.off"),
                            next_state_id: schema("rpg.object-state.on"),
                        },
                    )],
                },
            )],
        },
    ]
}

fn payload(
    snapshot: &RpgSnapshotV2,
    kind: RpgAggregateKindV1,
    persistent_id: PersistentId,
) -> &RpgAggregatePayloadV1 {
    &snapshot
        .aggregates
        .iter()
        .find(|record| record.aggregate_kind == kind && record.persistent_id == persistent_id)
        .expect("fixture aggregate exists")
        .payload
}

fn compatibility() -> SaveCompatibility {
    SaveCompatibility {
        engine_build_hash: ContentHash::from_bytes([1; 32]),
        game_build_hash: ContentHash::from_bytes([2; 32]),
        project_id: schema("nextengine.rpg-slice"),
        schema_registry_hash: ContentHash::from_bytes([3; 32]),
        content_manifest_hash: ContentHash::from_bytes([4; 32]),
        mechanics_lock_hash: ContentHash::from_bytes([5; 32]),
        tick_settings: TickSettings {
            gameplay_hz: 30,
            physics_hz: 60,
            motor_hz: 60,
        },
        loaded_chunk_revisions: vec![],
        rng_stream_states: vec![],
        physical_bindings: vec![],
        policy_state_schemas: vec![],
        plugin_script_bindings: vec![],
    }
}

#[test]
fn generic_rpg_slice_repeats_with_exact_state_event_and_ledger_hashes() {
    let input = RpgReplayInput {
        bootstrap: bootstrap(),
        authority: authority(),
        initial_rpg_snapshot: fixture(),
        ticks: slice_ticks(),
    };
    let expected = run_rpg_replay(&input).expect("baseline RPG replay completes");

    assert_eq!(expected.ticks.len(), 4);
    assert!(
        expected
            .ticks
            .iter()
            .all(|tick| { tick.command_results[0].disposition == CommandDisposition::Committed })
    );
    assert_eq!(expected.ticks[0].events.len(), 3);
    assert!(matches!(
        expected.ticks[0].events[2].payload,
        EventPayload::Rpg(RpgEventV1::RelationshipAdjusted { value: 7, .. })
    ));
    assert!(matches!(
        payload(
            &expected.final_rpg_snapshot,
            RpgAggregateKindV1::Dialogue,
            id(5)
        ),
        RpgAggregatePayloadV1::Dialogue(value)
            if value.node_id == schema("rpg.dialogue-node.accepted")
    ));
    assert!(matches!(
        payload(
            &expected.final_rpg_snapshot,
            RpgAggregateKindV1::Quest,
            id(4)
        ),
        RpgAggregatePayloadV1::Quest(value)
            if value.state_id == schema("rpg.quest-state.active")
    ));
    assert!(matches!(
        payload(
            &expected.final_rpg_snapshot,
            RpgAggregateKindV1::Relationship,
            id(6)
        ),
        RpgAggregatePayloadV1::Relationship(value) if value.dimensions[0].value == 7
    ));
    assert!(matches!(
        payload(
            &expected.final_rpg_snapshot,
            RpgAggregateKindV1::Inventory,
            id(10)
        ),
        RpgAggregatePayloadV1::Inventory(value) if value.item_ids == vec![id(3)]
    ));
    assert!(matches!(
        payload(
            &expected.final_rpg_snapshot,
            RpgAggregateKindV1::Character,
            id(2)
        ),
        RpgAggregatePayloadV1::Character(value)
            if value.skills[0].proficiency.get() == 250
    ));

    for _ in 0..100 {
        assert_eq!(
            run_rpg_replay(&input).expect("repeat RPG replay completes"),
            expected
        );
    }
}

#[test]
fn world_save_load_restores_rpg_owner_segment_and_exact_continuation() {
    let input = RpgReplayInput {
        bootstrap: bootstrap(),
        authority: authority(),
        initial_rpg_snapshot: fixture(),
        ticks: slice_ticks(),
    };
    let output = run_rpg_replay(&input).expect("RPG replay completes");
    let checkpoint = WorldCheckpointV4::new(
        output.final_runtime_snapshot.clone(),
        output.final_rpg_snapshot.clone(),
        PhysicsWorldCheckpointV1::new(
            input.bootstrap.physics_checkpoint.catalog.clone(),
            output.final_physics_snapshot.clone(),
        )
        .expect("physics checkpoint is valid"),
    )
    .expect("world checkpoint is valid");
    let image = SaveImage::from_world_checkpoint(0, compatibility(), &checkpoint)
        .expect("world save image is valid");
    let loaded = image.validate_world().expect("world save image loads");
    assert_eq!(loaded.runtime_snapshot, output.final_runtime_snapshot);
    assert_eq!(loaded.rpg_snapshot, output.final_rpg_snapshot);

    let continuation = command(
        4,
        4,
        RpgCommandV1 {
            operations: vec![operation(
                0,
                vec![target(RpgAggregateKindV1::Character, id(2), 1)],
                RpgOperationPayloadV1::SetSkillProficiency {
                    character_id: id(2),
                    skill_id: schema("rpg.skill.survival"),
                    expected_value: 250,
                    new_value: 260,
                },
            )],
        },
    );
    let mut direct = RuntimeState::restore_world_checkpoint(checkpoint, authority())
        .expect("direct state restores");
    let mut restored = RuntimeState::restore_world_checkpoint(loaded.checkpoint, authority())
        .expect("loaded state restores");

    assert_eq!(
        direct
            .run_tick([continuation.clone()])
            .expect("direct continuation commits"),
        restored
            .run_tick([continuation])
            .expect("loaded continuation commits")
    );
    assert_eq!(
        compute_world_checkpoint_root(&direct.world_checkpoint().expect("direct checkpoint"))
            .expect("direct root"),
        compute_world_checkpoint_root(&restored.world_checkpoint().expect("restored checkpoint"))
            .expect("restored root")
    );
}

#[test]
fn rejected_rpg_transition_does_not_partially_mutate_and_consumes_sequence() {
    let initial_rpg = fixture();
    let mut runtime =
        RuntimeState::with_rpg_snapshot(bootstrap(), authority(), initial_rpg.clone())
            .expect("fixture restores");
    let invalid = command(0, 0, dialogue_quest_command("rpg.quest-state.wrong"));

    let rejected = runtime
        .run_tick([invalid])
        .expect("domain rejection is not a fatal tick error");
    assert_eq!(
        rejected.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::RpgTransitionInvalid)
    );
    assert!(rejected.events.is_empty());
    assert_eq!(
        rejected.snapshot.authoritative_revision, 1,
        "the RPG rejection is mutation-free, while the mandatory empty physics step advances"
    );
    let stream = rejected
        .snapshot
        .command_ledger
        .streams
        .values()
        .next()
        .expect("predeclared stream exists");
    assert_eq!(stream.receipt_window.len(), 1);
    assert_eq!(rejected.rpg_snapshot, initial_rpg);

    let corrected = command(1, 1, dialogue_quest_command("rpg.quest-state.available"));
    assert_eq!(
        runtime
            .run_tick([corrected])
            .expect("corrected command commits")
            .results[0]
            .disposition,
        CommandDisposition::Committed
    );
}

#[test]
fn public_manifest_authority_shape_remains_engine_owned() {
    let grant = AuthorityGrant {
        principal: principal(),
        capabilities: vec![
            next_contracts::ids::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                .expect("RPG capability is valid"),
        ],
    };
    grant.validate().expect("authority grant is canonical");
}

#[test]
fn exact_rpg_runtime_bindings_survive_checkpoint_restore() {
    let bootstrap = bootstrap();
    let bindings = RpgRuntimeBindingsV1 {
        project_composition_lock_hash: ContentHash::from_bytes([0xa1; 32]),
        schema_registry_hash: bootstrap.rpg_bindings.schema_registry_hash,
        budget_policy_hash: ContentHash::from_bytes([0xa2; 32]),
        active_definition_policy_hashes: vec![
            ContentHash::from_bytes([0xa3; 32]),
            ContentHash::from_bytes([0xa4; 32]),
        ],
    };
    let runtime = RuntimeState::with_rpg_snapshot(
        bootstrap.with_rpg_bindings(bindings.clone()),
        authority(),
        RpgSnapshotV2::default(),
    )
    .expect("runtime accepts exact bindings");
    assert_eq!(runtime.snapshot().rpg_runtime_bindings, bindings);

    let restored = RuntimeState::restore_world_checkpoint(
        runtime.world_checkpoint().expect("checkpoint"),
        authority(),
    )
    .expect("checkpoint restores");
    assert_eq!(restored.snapshot().rpg_runtime_bindings, bindings);
}
