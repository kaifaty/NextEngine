use super::{
    CORE_DIALOGUE_ACCEPTED_NODE_ID, CORE_DIALOGUE_OFFER_NODE_ID, CORE_DIALOGUE_QUEST_TRUST_DELTA,
    CORE_HELP_DIALOGUE_DEFINITION_ID, CORE_HELP_QUEST_DEFINITION_ID, CORE_QUEST_ACTIVE_STATE_ID,
    CORE_QUEST_AVAILABLE_STATE_ID, CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID,
    CORE_RELATIONSHIP_TRUST_DIMENSION_ID, CharacterSnapshot, CoreDialogueQuestClosureError,
    CoreDialogueQuestProfileState, DialogueSnapshot, QuestSnapshot, RelationshipEntry, RpgCommand,
    RpgSnapshot, SkillProficiency, SkillProficiencyEntry,
};
use crate::{CanonicalDecodeLimits, PersistentId, SchemaId};

fn schema(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test schema id is valid")
}

fn core_dialogue_fixture(completed: bool) -> RpgSnapshot {
    let player_id = PersistentId::from_bytes([1; 16]);
    let npc_id = PersistentId::from_bytes([2; 16]);
    RpgSnapshot {
        characters: vec![
            CharacterSnapshot {
                id: player_id,
                revision: 0,
                archetype_id: schema("nextengine.rpg.character.player"),
                skills: Vec::new(),
                relationships: Vec::new(),
            },
            CharacterSnapshot {
                id: npc_id,
                revision: u64::from(completed),
                archetype_id: schema(CORE_QUEST_GIVER_CHARACTER_ARCHETYPE_ID),
                skills: Vec::new(),
                relationships: completed
                    .then(|| RelationshipEntry {
                        target: player_id,
                        dimension_id: schema(CORE_RELATIONSHIP_TRUST_DIMENSION_ID),
                        value: CORE_DIALOGUE_QUEST_TRUST_DELTA,
                    })
                    .into_iter()
                    .collect(),
            },
        ],
        quests: vec![QuestSnapshot {
            id: PersistentId::from_bytes([3; 16]),
            revision: u64::from(completed),
            definition_id: schema(CORE_HELP_QUEST_DEFINITION_ID),
            state_id: schema(if completed {
                CORE_QUEST_ACTIVE_STATE_ID
            } else {
                CORE_QUEST_AVAILABLE_STATE_ID
            }),
        }],
        dialogues: vec![DialogueSnapshot {
            id: PersistentId::from_bytes([4; 16]),
            revision: u64::from(completed),
            definition_id: schema(CORE_HELP_DIALOGUE_DEFINITION_ID),
            speaker: npc_id,
            listener: player_id,
            node_id: schema(if completed {
                CORE_DIALOGUE_ACCEPTED_NODE_ID
            } else {
                CORE_DIALOGUE_OFFER_NODE_ID
            }),
        }],
        ..RpgSnapshot::default()
    }
}

#[test]
fn snapshot_round_trip_is_byte_exact_and_order_independent() {
    let first = CharacterSnapshot {
        id: PersistentId::from_bytes([1; 16]),
        revision: 3,
        archetype_id: schema("rpg.character.generic"),
        skills: vec![
            SkillProficiencyEntry {
                skill_id: schema("rpg.skill.alchemy"),
                proficiency: SkillProficiency::new(200).expect("bounded"),
            },
            SkillProficiencyEntry {
                skill_id: schema("rpg.skill.survival"),
                proficiency: SkillProficiency::new(500).expect("bounded"),
            },
        ],
        relationships: vec![RelationshipEntry {
            target: PersistentId::from_bytes([2; 16]),
            dimension_id: schema("rpg.relationship.trust"),
            value: 7,
        }],
    };
    let second = CharacterSnapshot {
        id: PersistentId::from_bytes([2; 16]),
        revision: 0,
        archetype_id: schema("rpg.character.generic"),
        skills: vec![],
        relationships: vec![],
    };
    let left = RpgSnapshot {
        characters: vec![first.clone(), second.clone()],
        ..RpgSnapshot::default()
    };
    let right = RpgSnapshot {
        characters: vec![second, first],
        ..RpgSnapshot::default()
    };

    let bytes = left.canonical_bytes().expect("snapshot is canonical");
    assert_eq!(
        bytes,
        right.canonical_bytes().expect("order is canonicalized")
    );
    assert_eq!(
        RpgSnapshot::from_canonical_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("snapshot decodes"),
        left
    );
}

#[test]
fn command_payload_round_trip_is_byte_exact() {
    let command = RpgCommand::AdvanceDialogueQuest {
        dialogue_id: PersistentId::from_bytes([1; 16]),
        expected_dialogue_node_id: schema("dialogue.node.offer"),
        next_dialogue_node_id: schema("dialogue.node.accepted"),
        quest_id: PersistentId::from_bytes([2; 16]),
        expected_quest_state_id: schema("quest.state.available"),
        next_quest_state_id: schema("quest.state.active"),
        relationship_source: PersistentId::from_bytes([3; 16]),
        relationship_target: PersistentId::from_bytes([4; 16]),
        relationship_dimension_id: schema("relationship.trust"),
        relationship_delta: 5,
    };
    let bytes = command
        .canonical_payload_bytes()
        .expect("command is canonical");
    assert_eq!(
        RpgCommand::from_canonical_payload_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("command decodes"),
        command
    );
}

#[test]
fn proficiency_rejects_values_above_fixed_point_bound() {
    assert!(SkillProficiency::new(10_000).is_ok());
    assert!(SkillProficiency::new(10_001).is_err());
}

#[test]
fn core_dialogue_profile_resolves_absent_ready_and_completed_states() {
    let player_id = PersistentId::from_bytes([1; 16]);
    assert_eq!(
        RpgSnapshot::default()
            .resolve_core_dialogue_quest_binding(player_id)
            .expect("absent profile is valid"),
        None
    );
    assert_eq!(
        core_dialogue_fixture(false)
            .resolve_core_dialogue_quest_binding(player_id)
            .expect("ready profile is valid")
            .expect("binding")
            .state,
        CoreDialogueQuestProfileState::Ready
    );
    assert_eq!(
        core_dialogue_fixture(true)
            .resolve_core_dialogue_quest_binding(player_id)
            .expect("completed profile is valid")
            .expect("binding")
            .state,
        CoreDialogueQuestProfileState::Completed
    );
}

#[test]
fn partial_or_mixed_core_dialogue_profile_fails_closed() {
    let player_id = PersistentId::from_bytes([1; 16]);
    let mut partial = core_dialogue_fixture(false);
    partial.quests.clear();
    assert_eq!(
        partial.resolve_core_dialogue_quest_binding(player_id),
        Err(CoreDialogueQuestClosureError)
    );

    let mut mixed = core_dialogue_fixture(false);
    mixed.quests[0].state_id = schema(CORE_QUEST_ACTIVE_STATE_ID);
    assert_eq!(
        mixed.resolve_core_dialogue_quest_binding(player_id),
        Err(CoreDialogueQuestClosureError)
    );

    let mut wrong_listener = core_dialogue_fixture(false);
    wrong_listener.dialogues[0].listener = PersistentId::from_bytes([9; 16]);
    assert_eq!(
        wrong_listener.resolve_core_dialogue_quest_binding(player_id),
        Err(CoreDialogueQuestClosureError)
    );
}
