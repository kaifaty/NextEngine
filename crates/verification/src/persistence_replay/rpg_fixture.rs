use next_contracts::command::{IssuerPrincipal, WorldCommand};
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::persistence::{SaveCompatibility, TickSettings};
use next_contracts::rpg::{
    CharacterPayloadV1, InventoryPayloadV1, ItemPayloadV1, RpgAggregateKindV1,
    RpgAggregatePayloadV1, RpgAggregateRefV1, RpgCommandV1, RpgOperationPayloadV1, RpgOperationV1,
    RpgSnapshotV2,
};

use crate::cooked_project_rpg_snapshot;
use crate::player_fixture::fixture_aggregate;

use super::PersistenceReplayCheckError;

pub(super) fn aggregate_payload(
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

pub(super) fn initial_rpg_snapshot(
    fixture: &crate::NeutralPlayerFixture,
) -> Result<RpgSnapshotV2, PersistenceReplayCheckError> {
    let mut snapshot = cooked_project_rpg_snapshot(fixture);
    let item_id = PersistentId::from_bytes([0x10; 16]);
    let first_character_id = PersistentId::from_bytes([0x20; 16]);
    let first_inventory_id = PersistentId::from_bytes([0x21; 16]);
    let second_character_id = PersistentId::from_bytes([0x30; 16]);
    let second_inventory_id = PersistentId::from_bytes([0x31; 16]);
    snapshot.aggregates.extend([
        fixture_aggregate(
            first_character_id,
            0x20,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(first_inventory_id),
                equipment_id: None,
                resources: Vec::new(),
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate(
            second_character_id,
            0x30,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: Some(second_inventory_id),
                equipment_id: None,
                resources: Vec::new(),
                skills: Vec::new(),
            }),
        ),
        fixture_aggregate(
            item_id,
            0x10,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity: 1,
                durability: 100,
                custom_state: Vec::new(),
            }),
        ),
        fixture_aggregate(
            first_inventory_id,
            0x21,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: first_character_id,
                capacity: 8,
                item_ids: vec![item_id],
                reservations: Vec::new(),
            }),
        ),
        fixture_aggregate(
            second_inventory_id,
            0x31,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: second_character_id,
                capacity: 8,
                item_ids: Vec::new(),
                reservations: Vec::new(),
            }),
        ),
    ]);
    snapshot
        .aggregates
        .sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
    Ok(snapshot)
}

pub(super) fn rpg_commands(
    stream_id: next_contracts::ids::CommandStreamId,
    principal: IssuerPrincipal,
) -> Result<Vec<WorldCommand>, PersistenceReplayCheckError> {
    let item_id = PersistentId::from_bytes([0x10; 16]);
    let first_inventory_id = PersistentId::from_bytes([0x21; 16]);
    let second_inventory_id = PersistentId::from_bytes([0x31; 16]);
    let operation =
        |source_inventory_id, destination_inventory_id, inventory_revision| RpgCommandV1 {
            operations: vec![RpgOperationV1 {
                operation_slot: 0,
                targets: vec![
                    RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Item,
                        persistent_id: item_id,
                        expected_revision: 0,
                    },
                    RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Inventory,
                        persistent_id: first_inventory_id,
                        expected_revision: inventory_revision,
                    },
                    RpgAggregateRefV1 {
                        aggregate_kind: RpgAggregateKindV1::Inventory,
                        persistent_id: second_inventory_id,
                        expected_revision: inventory_revision,
                    },
                ],
                definition_policy_hashes: Vec::new(),
                payload: RpgOperationPayloadV1::TransferItem {
                    item_id,
                    source_inventory_id: Some(source_inventory_id),
                    destination_inventory_id: Some(destination_inventory_id),
                    quantity: 1,
                },
            }],
        };
    Ok(vec![
        WorldCommand::rpg(
            stream_id,
            principal.clone(),
            0,
            0,
            operation(first_inventory_id, second_inventory_id, 0),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("current RPG command", error.to_string())
        })?,
        WorldCommand::rpg(
            stream_id,
            principal,
            1,
            2,
            operation(second_inventory_id, first_inventory_id, 1),
        )
        .map_err(|error| {
            PersistenceReplayCheckError::new("future RPG command", error.to_string())
        })?,
    ])
}

pub(super) fn compatibility() -> Result<SaveCompatibility, PersistenceReplayCheckError> {
    Ok(SaveCompatibility {
        engine_build_hash: ContentHash::from_bytes([1; 32]),
        game_build_hash: ContentHash::from_bytes([2; 32]),
        project_id: SchemaId::new("nextengine.persistence-replay").map_err(|error| {
            PersistenceReplayCheckError::new("save project ID", error.to_string())
        })?,
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
    })
}
