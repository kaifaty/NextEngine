use next_contracts::ids::{AssetId, ContentHash, PersistentId, SchemaId};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::rpg::CORE_INTERACTIVE_OBJECT_READY_STATE_ID;
use next_contracts::rpg::{
    CharacterPayloadV1, CharacterResourceEntryV1, DefinitionRefV1, DialoguePayloadV1,
    EquipmentPayloadV1, InteractiveObjectPayloadV1, InventoryPayloadV1, ItemPayloadV1,
    ProvenanceBindingV1, QuestPayloadV1, RelationshipDimensionV1, RelationshipPayloadV1,
    RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2,
};

use crate::ReferenceGameSession;

#[must_use]
pub fn cooked_project_rpg_snapshot(fixture: &ReferenceGameSession) -> RpgSnapshotV2 {
    let definitions = &fixture.activated_project.rpg_definitions;
    let dialogue_definition = definitions
        .dialogues
        .first()
        .expect("cooked fixture has one dialogue definition");
    let quest_definition = definitions
        .quests
        .first()
        .expect("cooked fixture has one quest definition");
    let relationship_definition = definitions
        .relationships
        .first()
        .expect("cooked fixture has one relationship definition");
    let interaction_definition = definitions
        .interactions
        .first()
        .expect("cooked fixture has one interaction definition");
    let ability_definition = definitions
        .abilities
        .first()
        .expect("cooked fixture has one ability definition");
    let health_resource = || CharacterResourceEntryV1 {
        resource_id: SchemaId::new(CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .expect("engine-owned health resource is valid"),
        current_value: 100,
        minimum_value: 0,
        maximum_value: 100,
    };
    let mut aggregates = vec![
        reference_aggregate(
            fixture.body_id,
            0x54,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(fixture.player_inventory_id),
                equipment_id: Some(fixture.player_equipment_id),
                resources: vec![health_resource()],
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.pickup_item_id,
            ability_definition.required_item_definition,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: Vec::new(),
            }),
        ),
        reference_aggregate(
            fixture.player_inventory_id,
            0x5d,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: fixture.body_id,
                capacity: 8,
                item_ids: Vec::new(),
                reservations: Vec::new(),
            }),
        ),
        reference_aggregate(
            fixture.player_equipment_id,
            0x5e,
            RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                character_id: fixture.body_id,
                slot_policy: DefinitionRefV1::Exact {
                    asset_id: AssetId::from_bytes([0x5e; 16]),
                    content_hash: next_runtime::bootstrap_equipment_slot_policy_hash_v1(),
                },
                assignments: Vec::new(),
            }),
        ),
        reference_aggregate(
            fixture.npc_character_id,
            0x59,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(fixture.npc_inventory_id),
                equipment_id: Some(fixture.npc_equipment_id),
                resources: vec![health_resource()],
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.npc_weapon_item_id,
            ability_definition.required_item_definition,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: Vec::new(),
            }),
        ),
        reference_aggregate(
            fixture.npc_inventory_id,
            0x61,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: fixture.npc_character_id,
                capacity: 1,
                item_ids: vec![fixture.npc_weapon_item_id],
                reservations: Vec::new(),
            }),
        ),
        reference_aggregate(
            fixture.npc_equipment_id,
            0x62,
            RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                character_id: fixture.npc_character_id,
                slot_policy: DefinitionRefV1::Exact {
                    asset_id: AssetId::from_bytes([0x62; 16]),
                    content_hash: next_runtime::bootstrap_equipment_slot_policy_hash_v1(),
                },
                assignments: vec![next_contracts::rpg::EquipmentSlotAssignmentV1 {
                    slot_id: ability_definition.required_equipment_slot_id.clone(),
                    item_id: fixture.npc_weapon_item_id,
                }],
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.quest_id,
            quest_definition.asset_revision,
            RpgAggregatePayloadV1::Quest(QuestPayloadV1 {
                state_id: quest_definition.entry_state_id.clone(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.dialogue_id,
            dialogue_definition.asset_revision,
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: fixture.npc_character_id,
                listener_id: fixture.body_id,
                node_id: dialogue_definition.entry_node_id.clone(),
            }),
        ),
        fixture_aggregate_from_asset(
            fixture.relationship_id,
            relationship_definition.asset_revision,
            RpgAggregatePayloadV1::Relationship(RelationshipPayloadV1 {
                source_id: fixture.npc_character_id,
                target_id: fixture.body_id,
                dimensions: vec![RelationshipDimensionV1 {
                    dimension_id: relationship_definition.dimension_id.clone(),
                    value: interaction_definition.relationship_source_value,
                }],
            }),
        ),
        reference_aggregate(
            fixture.interactive_object_id,
            0x58,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: SchemaId::new(CORE_INTERACTIVE_OBJECT_READY_STATE_ID)
                    .expect("built-in interactive-object state is valid"),
                linked_item_id: None,
            }),
        ),
        reference_aggregate(
            fixture.pickup_proxy_id,
            0x60,
            RpgAggregatePayloadV1::InteractiveObject(InteractiveObjectPayloadV1 {
                state_id: SchemaId::new(CORE_INTERACTIVE_OBJECT_READY_STATE_ID)
                    .expect("built-in interactive-object state is valid"),
                linked_item_id: Some(fixture.pickup_item_id),
            }),
        ),
    ];
    aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
    RpgSnapshotV2 { aggregates }
}

#[must_use]
pub fn cooked_interaction_outcome(
    fixture: &ReferenceGameSession,
) -> (SchemaId, SchemaId, SchemaId, i32) {
    let definitions = &fixture.activated_project.rpg_definitions;
    let interaction = definitions
        .interactions
        .first()
        .expect("cooked fixture has one interaction definition");
    let dialogue = definitions
        .dialogue(interaction.dialogue_definition)
        .expect("interaction dialogue definition is closed");
    let quest = definitions
        .quest(interaction.quest_definition)
        .expect("interaction quest definition is closed");
    let relationship = definitions
        .relationship(interaction.relationship_definition)
        .expect("interaction relationship definition is closed");
    let dialogue_target = dialogue
        .transitions
        .iter()
        .find(|transition| transition.transition_id == interaction.dialogue_transition_id)
        .expect("dialogue transition is closed")
        .target_state_id
        .clone();
    let quest_target = quest
        .transitions
        .iter()
        .find(|transition| transition.transition_id == interaction.quest_transition_id)
        .expect("quest transition is closed")
        .target_state_id
        .clone();
    (
        dialogue_target,
        quest_target,
        relationship.dimension_id.clone(),
        interaction
            .relationship_source_value
            .checked_add(interaction.relationship_delta)
            .expect("cooked relationship outcome fits i32"),
    )
}

fn fixture_aggregate_from_asset(
    persistent_id: PersistentId,
    definition: AssetRevisionRefV1,
    payload: RpgAggregatePayloadV1,
) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        persistent_id,
        1,
        0,
        DefinitionRefV1::Exact {
            asset_id: definition.asset_id,
            content_hash: definition.record_sha256,
        },
        ProvenanceBindingV1::None,
        payload,
    )
    .expect("cooked fixture aggregate is canonical")
}

pub fn reference_aggregate(
    persistent_id: PersistentId,
    definition_seed: u8,
    payload: RpgAggregatePayloadV1,
) -> RpgAggregateEnvelopeV1 {
    RpgAggregateEnvelopeV1::new(
        persistent_id,
        1,
        0,
        DefinitionRefV1::Exact {
            asset_id: AssetId::from_bytes([definition_seed; 16]),
            content_hash: ContentHash::from_bytes([definition_seed; 32]),
        },
        ProvenanceBindingV1::Exact(ContentHash::from_bytes([definition_seed; 32])),
        payload,
    )
    .expect("built-in fixture aggregate is valid")
}

pub fn aggregate_payload(
    snapshot: &RpgSnapshotV2,
    kind: RpgAggregateKindV1,
    persistent_id: PersistentId,
) -> Option<&RpgAggregatePayloadV1> {
    snapshot
        .aggregates
        .iter()
        .find(|aggregate| {
            aggregate.aggregate_kind == kind && aggregate.persistent_id == persistent_id
        })
        .map(|aggregate| &aggregate.payload)
}
