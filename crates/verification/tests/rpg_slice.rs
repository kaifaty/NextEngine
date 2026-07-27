use next_assets::SaveImage;
use next_contracts::{
    AuthorityGrant, CharacterSnapshot, ContentHash, DialogueSnapshot, EventPayload,
    FactionSnapshot, InteractiveObjectSnapshot, IssuerPrincipal, ItemSnapshot, PersistentId,
    PhysicsWorldCheckpointV1, PlayerPrincipalId, QuestSnapshot, RPG_COMMAND_CAPABILITY_ID,
    RelationshipEntry, RpgCommand, RpgEvent, RpgSnapshot, SaveCompatibility, SchemaId,
    SkillProficiency, SkillProficiencyEntry, TickSettings, WorldCheckpointV3,
    WorldChunkRecordSnapshot, WorldCommand,
};
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
            [next_contracts::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                .expect("RPG capability is valid")],
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
                next_contracts::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                    .expect("RPG capability is valid"),
            ],
        )],
    )
    .expect("neutral RPG fixture")
}

fn bootstrap() -> RuntimeBootstrapV3 {
    runtime_fixture().bootstrap
}

fn fixture() -> RpgSnapshot {
    RpgSnapshot {
        characters: vec![
            CharacterSnapshot {
                id: id(1),
                revision: 0,
                archetype_id: schema("rpg.character.generic-npc"),
                skills: vec![SkillProficiencyEntry {
                    skill_id: schema("rpg.skill.survival"),
                    proficiency: SkillProficiency::new(400).expect("bounded proficiency"),
                }],
                relationships: vec![RelationshipEntry {
                    target: id(2),
                    dimension_id: schema("rpg.relationship.trust"),
                    value: 0,
                }],
            },
            CharacterSnapshot {
                id: id(2),
                revision: 0,
                archetype_id: schema("rpg.character.player"),
                skills: vec![],
                relationships: vec![],
            },
        ],
        items: vec![ItemSnapshot {
            id: id(3),
            revision: 0,
            archetype_id: schema("rpg.item.quest-token"),
            owner: Some(id(1)),
            quantity: 1,
        }],
        quests: vec![QuestSnapshot {
            id: id(4),
            revision: 0,
            definition_id: schema("rpg.quest.generic-help"),
            state_id: schema("rpg.quest-state.available"),
        }],
        dialogues: vec![DialogueSnapshot {
            id: id(5),
            revision: 0,
            definition_id: schema("rpg.dialogue.generic-help"),
            speaker: id(1),
            listener: id(2),
            node_id: schema("rpg.dialogue-node.offer"),
        }],
        factions: vec![FactionSnapshot {
            id: id(6),
            revision: 0,
            definition_id: schema("rpg.faction.settlers"),
            members: vec![id(1)],
        }],
        interactive_objects: vec![InteractiveObjectSnapshot {
            id: id(7),
            revision: 0,
            archetype_id: schema("rpg.interactive-object.lever"),
            state_id: schema("rpg.object-state.off"),
        }],
        world_chunk_records: vec![WorldChunkRecordSnapshot {
            id: id(8),
            revision: 0,
            record_schema_id: schema("rpg.world-chunk-record.encounter"),
            state_id: schema("rpg.world-chunk-state.dormant"),
        }],
    }
}

fn command(sequence: u64, tick: u64, payload: RpgCommand) -> WorldCommand {
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
                RpgCommand::AdvanceDialogueQuest {
                    dialogue_id: id(5),
                    expected_dialogue_node_id: schema("rpg.dialogue-node.offer"),
                    next_dialogue_node_id: schema("rpg.dialogue-node.accepted"),
                    quest_id: id(4),
                    expected_quest_state_id: schema("rpg.quest-state.available"),
                    next_quest_state_id: schema("rpg.quest-state.active"),
                    relationship_source: id(1),
                    relationship_target: id(2),
                    relationship_dimension_id: schema("rpg.relationship.trust"),
                    relationship_delta: 7,
                },
            )],
        },
        ReplayTickInput {
            commands: vec![command(
                1,
                1,
                RpgCommand::TransferItem {
                    item_id: id(3),
                    expected_owner: Some(id(1)),
                    new_owner: Some(id(2)),
                },
            )],
        },
        ReplayTickInput {
            commands: vec![command(
                2,
                2,
                RpgCommand::LearnSkill {
                    character_id: id(2),
                    skill_id: schema("rpg.skill.survival"),
                    delta: 250,
                },
            )],
        },
        ReplayTickInput {
            commands: vec![command(
                3,
                3,
                RpgCommand::SetInteractiveObjectState {
                    object_id: id(7),
                    expected_state_id: schema("rpg.object-state.off"),
                    next_state_id: schema("rpg.object-state.on"),
                },
            )],
        },
    ]
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
    assert!(expected.ticks.iter().all(|tick| {
        tick.command_results[0].disposition == CommandDisposition::Committed
            && tick.events.len() == 1
    }));
    assert!(matches!(
        expected.ticks[0].events[0].payload,
        EventPayload::Rpg(RpgEvent::DialogueQuestAdvanced {
            relationship_value: 7,
            ..
        })
    ));
    assert_eq!(
        expected.final_rpg_snapshot.dialogues[0].node_id,
        schema("rpg.dialogue-node.accepted")
    );
    assert_eq!(
        expected.final_rpg_snapshot.quests[0].state_id,
        schema("rpg.quest-state.active")
    );
    assert_eq!(
        expected.final_rpg_snapshot.characters[0].relationships[0].value,
        7
    );
    assert_eq!(expected.final_rpg_snapshot.items[0].owner, Some(id(2)));
    assert_eq!(
        expected.final_rpg_snapshot.characters[1].skills[0]
            .proficiency
            .get(),
        250
    );

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
    let checkpoint = WorldCheckpointV3::new(
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
        RpgCommand::LearnSkill {
            character_id: id(2),
            skill_id: schema("rpg.skill.survival"),
            delta: 10,
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
    let invalid = command(
        0,
        0,
        RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema("rpg.dialogue-node.offer"),
            next_dialogue_node_id: schema("rpg.dialogue-node.accepted"),
            quest_id: id(4),
            expected_quest_state_id: schema("rpg.quest-state.wrong"),
            next_quest_state_id: schema("rpg.quest-state.active"),
            relationship_source: id(1),
            relationship_target: id(2),
            relationship_dimension_id: schema("rpg.relationship.trust"),
            relationship_delta: 7,
        },
    );

    let rejected = runtime
        .run_tick([invalid])
        .expect("domain rejection is not a fatal tick error");
    assert_eq!(
        rejected.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::RpgStatePreconditionFailed)
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

    let corrected = command(
        1,
        1,
        RpgCommand::AdvanceDialogueQuest {
            dialogue_id: id(5),
            expected_dialogue_node_id: schema("rpg.dialogue-node.offer"),
            next_dialogue_node_id: schema("rpg.dialogue-node.accepted"),
            quest_id: id(4),
            expected_quest_state_id: schema("rpg.quest-state.available"),
            next_quest_state_id: schema("rpg.quest-state.active"),
            relationship_source: id(1),
            relationship_target: id(2),
            relationship_dimension_id: schema("rpg.relationship.trust"),
            relationship_delta: 7,
        },
    );
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
            next_contracts::CapabilityId::new(RPG_COMMAND_CAPABILITY_ID)
                .expect("RPG capability is valid"),
        ],
    };
    grant.validate().expect("authority grant is canonical");
}
