use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::presentation::{
    SemanticUiPresentationRecordV1, UiAccessibilityRoleV1, UiElementRoleV1, UiElementValueV1,
    UiSemanticElementV1, UiStyleRoleV1, UiTextArgumentV1, UiTextRefV1,
};
use next_contracts::project::domain_hash;
use next_contracts::rpg::{
    CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, CORE_QUEST_ACTIVE_STATE_ID, RpgAggregateKindV1,
    RpgAggregatePayloadV1, RpgSnapshotV2,
};

use crate::ReferenceGameError;
use crate::rpg::aggregate_payload;
use crate::session::ReferenceGameSession;

pub const HUD_SURFACE_ID: &str = "nextengine.ui.surface.hud";
pub const HUD_STATUS_PANEL_ID: &str = "nextengine.ui.panel.hud.status";
pub const HUD_ACTION_PANEL_ID: &str = "nextengine.ui.panel.hud.action";
pub const HUD_TARGET_PANEL_ID: &str = "nextengine.ui.panel.hud.target";
pub const HUD_HEALTH_ELEMENT_ID: &str = "nextengine.ui.element.hud.health";
pub const HUD_QUEST_ELEMENT_ID: &str = "nextengine.ui.element.hud.quest";
pub const HUD_ACTION_ELEMENT_ID: &str = "nextengine.ui.element.hud.action";
pub const HUD_TARGET_HEALTH_ELEMENT_ID: &str = "nextengine.ui.element.hud.target-health";
pub const HUD_SUBTITLE_ELEMENT_ID: &str = "nextengine.ui.element.hud.subtitle";
pub const HUD_HEALTH_TEXT_ID: &str = "nextengine.ui.text.hud.health";
pub const HUD_QUEST_TEXT_ID: &str = "nextengine.ui.text.hud.quest-state";
pub const HUD_ACTION_ACCEPT_TEXT_ID: &str = "nextengine.ui.text.hud.action.accept";
pub const HUD_ACTION_PICKUP_TEXT_ID: &str = "nextengine.ui.text.hud.action.pickup";
pub const HUD_ACTION_EQUIP_TEXT_ID: &str = "nextengine.ui.text.hud.action.equip";
pub const HUD_ACTION_COMBAT_TEXT_ID: &str = "nextengine.ui.text.hud.action.combat";
pub const HUD_ACTION_RELAY_TEXT_ID: &str = "nextengine.ui.text.hud.action.relay";
pub const HUD_ACTION_RETURN_TEXT_ID: &str = "nextengine.ui.text.hud.action.return";
pub const HUD_ACTION_COMPLETE_TEXT_ID: &str = "nextengine.ui.text.hud.action.complete";
pub const HUD_TARGET_HEALTH_TEXT_ID: &str = "nextengine.ui.text.hud.target-health";

/// Builds the canonical HUD records for one presentation publication.
///
/// Elements appear only when their authoritative source aggregate exists;
/// an empty RPG snapshot (for example a non-interactive scenario) yields an
/// empty record set and therefore no semantic UI batches.
pub fn hud_semantic_ui_records(
    snapshot_epoch: ContentHash,
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let mut records =
        hud_semantic_ui_records_for_ids(snapshot_epoch, fixture.body_id, fixture.quest_id, rpg)?;
    if let Some(action_text_id) = frontier_relay_action_text_id(fixture, rpg) {
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(HUD_SURFACE_ID)?,
            schema_id(HUD_ACTION_PANEL_ID)?,
            domain_hash(
                "nextengine.ui-source.rpg-snapshot.v1",
                &rpg.canonical_bytes()?,
            ),
            UiSemanticElementV1::new(
                schema_id(HUD_ACTION_ELEMENT_ID)?,
                UiElementRoleV1::Label,
                UiStyleRoleV1::Accent,
                UiAccessibilityRoleV1::Status,
                true,
                true,
                false,
                Some(UiTextRefV1::new(schema_id(action_text_id)?, Vec::new())?),
                UiElementValueV1::None,
                Vec::new(),
            )?,
        )?);
        if action_text_id == HUD_ACTION_COMBAT_TEXT_ID
            && let Some(RpgAggregatePayloadV1::Character(enemy)) =
                aggregate_payload(rpg, RpgAggregateKindV1::Character, fixture.npc_character_id)
            && let Some(health) = enemy
                .resources
                .iter()
                .find(|entry| entry.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
        {
            records.push(SemanticUiPresentationRecordV1::new(
                snapshot_epoch,
                schema_id(HUD_SURFACE_ID)?,
                schema_id(HUD_TARGET_PANEL_ID)?,
                domain_hash(
                    "nextengine.ui-source.rpg-snapshot.v1",
                    &rpg.canonical_bytes()?,
                ),
                UiSemanticElementV1::new(
                    schema_id(HUD_TARGET_HEALTH_ELEMENT_ID)?,
                    UiElementRoleV1::Meter,
                    UiStyleRoleV1::Danger,
                    UiAccessibilityRoleV1::Status,
                    true,
                    true,
                    false,
                    Some(UiTextRefV1::new(
                        schema_id(HUD_TARGET_HEALTH_TEXT_ID)?,
                        vec![
                            UiTextArgumentV1::SignedInteger(i64::from(health.current_value)),
                            UiTextArgumentV1::SignedInteger(i64::from(health.maximum_value)),
                        ],
                    )?),
                    UiElementValueV1::Scalar {
                        current: i64::from(health.current_value),
                        maximum: i64::from(health.maximum_value),
                    },
                    Vec::new(),
                )?,
            )?);
        }
    }
    Ok(records)
}

/// Selects the next visible action from immutable authoritative RPG state.
/// This is guidance only: input targeting and command validation stay on the
/// production gameplay path, and no prompt state can affect the simulation.
fn frontier_relay_action_text_id(
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
) -> Option<&'static str> {
    let RpgAggregatePayloadV1::Quest(quest) =
        aggregate_payload(rpg, RpgAggregateKindV1::Quest, fixture.quest_id)?
    else {
        return None;
    };
    let quest_definition = fixture.activated_project.rpg_definitions.quests.first()?;
    if quest.state_id == quest_definition.entry_state_id {
        return Some(HUD_ACTION_ACCEPT_TEXT_ID);
    }
    let has_outgoing_transition = quest_definition
        .transitions
        .iter()
        .any(|transition| transition.source_state_id == quest.state_id);
    if !has_outgoing_transition {
        return Some(HUD_ACTION_COMPLETE_TEXT_ID);
    }

    let has_pickup = matches!(
        aggregate_payload(rpg, RpgAggregateKindV1::Inventory, fixture.player_inventory_id),
        Some(RpgAggregatePayloadV1::Inventory(inventory))
            if inventory.item_ids.contains(&fixture.pickup_item_id)
    );
    let has_equipped_pickup = matches!(
        aggregate_payload(rpg, RpgAggregateKindV1::Equipment, fixture.player_equipment_id),
        Some(RpgAggregatePayloadV1::Equipment(equipment))
            if equipment
                .assignments
                .iter()
                .any(|assignment| assignment.item_id == fixture.pickup_item_id)
    );
    if !has_equipped_pickup {
        return Some(if has_pickup {
            HUD_ACTION_EQUIP_TEXT_ID
        } else {
            HUD_ACTION_PICKUP_TEXT_ID
        });
    }

    let npc_alive =
        match aggregate_payload(rpg, RpgAggregateKindV1::Character, fixture.npc_character_id) {
            Some(RpgAggregatePayloadV1::Character(character)) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .is_none_or(|resource| resource.current_value > 0),
            _ => true,
        };
    if npc_alive {
        return Some(HUD_ACTION_COMBAT_TEXT_ID);
    }

    let relay_active = matches!(
        aggregate_payload(
            rpg,
            RpgAggregateKindV1::InteractiveObject,
            fixture.interactive_object_id,
        ),
        Some(RpgAggregatePayloadV1::InteractiveObject(relay))
            if relay.state_id.as_str() == CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID
    );
    Some(if relay_active {
        HUD_ACTION_RETURN_TEXT_ID
    } else {
        HUD_ACTION_RELAY_TEXT_ID
    })
}

/// Builds the canonical HUD records without a session handle, for callers
/// (verification fixtures, tools) that hold the authoritative ids directly.
pub fn hud_semantic_ui_records_for_ids(
    snapshot_epoch: ContentHash,
    player_character_id: PersistentId,
    quest_id: PersistentId,
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let source_snapshot_hash = domain_hash(
        "nextengine.ui-source.rpg-snapshot.v1",
        &rpg.canonical_bytes()?,
    );
    let mut records = Vec::new();
    if let Some(RpgAggregatePayloadV1::Character(character)) =
        aggregate_payload(rpg, RpgAggregateKindV1::Character, player_character_id)
        && let Some(health) = character
            .resources
            .iter()
            .find(|entry| entry.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
    {
        let current = i64::from(health.current_value);
        let maximum = i64::from(health.maximum_value);
        let health_style = if maximum > 0 && current <= maximum / 4 {
            UiStyleRoleV1::Danger
        } else if maximum > 0 && current <= maximum / 2 {
            UiStyleRoleV1::Warning
        } else {
            UiStyleRoleV1::Accent
        };
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(HUD_SURFACE_ID)?,
            schema_id(HUD_STATUS_PANEL_ID)?,
            source_snapshot_hash,
            UiSemanticElementV1::new(
                schema_id(HUD_HEALTH_ELEMENT_ID)?,
                UiElementRoleV1::Meter,
                health_style,
                UiAccessibilityRoleV1::Status,
                true,
                true,
                false,
                Some(UiTextRefV1::new(
                    schema_id(HUD_HEALTH_TEXT_ID)?,
                    vec![
                        UiTextArgumentV1::SignedInteger(current),
                        UiTextArgumentV1::SignedInteger(maximum),
                    ],
                )?),
                UiElementValueV1::Scalar { current, maximum },
                Vec::new(),
            )?,
        )?);
    }
    if let Some(RpgAggregatePayloadV1::Quest(quest)) =
        aggregate_payload(rpg, RpgAggregateKindV1::Quest, quest_id)
    {
        let quest_style = if quest.state_id.as_str() == CORE_QUEST_ACTIVE_STATE_ID {
            UiStyleRoleV1::Accent
        } else {
            UiStyleRoleV1::Muted
        };
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(HUD_SURFACE_ID)?,
            schema_id(HUD_STATUS_PANEL_ID)?,
            source_snapshot_hash,
            UiSemanticElementV1::new(
                schema_id(HUD_QUEST_ELEMENT_ID)?,
                UiElementRoleV1::Label,
                quest_style,
                UiAccessibilityRoleV1::Standard,
                true,
                true,
                false,
                Some(UiTextRefV1::new(
                    schema_id(HUD_QUEST_TEXT_ID)?,
                    vec![UiTextArgumentV1::TextId(quest.state_id.clone())],
                )?),
                UiElementValueV1::None,
                Vec::new(),
            )?,
        )?);
    }
    Ok(records)
}

fn schema_id(value: &str) -> Result<SchemaId, ReferenceGameError> {
    Ok(SchemaId::new(value)?)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use next_assets::ContentStore;
    use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
    use next_contracts::rpg::{
        CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID, EquipmentSlotAssignmentV1, RpgAggregateKindV1,
        RpgAggregatePayloadV1,
    };

    use super::{
        HUD_ACTION_ACCEPT_TEXT_ID, HUD_ACTION_COMBAT_TEXT_ID, HUD_ACTION_COMPLETE_TEXT_ID,
        HUD_ACTION_EQUIP_TEXT_ID, HUD_ACTION_PICKUP_TEXT_ID, HUD_ACTION_RELAY_TEXT_ID,
        HUD_ACTION_RETURN_TEXT_ID, frontier_relay_action_text_id,
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn frontier_relay_action_hint_covers_every_manual_acceptance_stage() {
        let root = std::env::temp_dir().join(format!(
            "nextengine-reference-action-hint-{}-{}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let store = ContentStore::new(&root);
        let cooked =
            next_project::cook_project_v5(crate::project_source_v5().expect("reference source"))
                .expect("cook");
        store
            .publish(&cooked.publication().expect("publication"))
            .expect("publish");
        let activated = next_project::activate_project(&store).expect("activate");
        let fixture = crate::build_reference_game_session(activated).expect("fixture");
        let mut rpg = crate::cooked_project_rpg_snapshot(&fixture);

        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_ACCEPT_TEXT_ID)
        );

        let quest_definition = fixture
            .activated_project
            .rpg_definitions
            .quests
            .first()
            .expect("quest definition");
        let active_state = quest_definition
            .transitions
            .iter()
            .find(|transition| transition.source_state_id == quest_definition.entry_state_id)
            .expect("accept transition")
            .target_state_id
            .clone();
        let completed_state = quest_definition
            .transitions
            .iter()
            .find(|transition| {
                !quest_definition
                    .transitions
                    .iter()
                    .any(|candidate| candidate.source_state_id == transition.target_state_id)
            })
            .expect("terminal transition")
            .target_state_id
            .clone();
        let quest = rpg
            .aggregates
            .iter_mut()
            .find(|aggregate| {
                aggregate.aggregate_kind == RpgAggregateKindV1::Quest
                    && aggregate.persistent_id == fixture.quest_id
            })
            .expect("quest aggregate");
        let RpgAggregatePayloadV1::Quest(quest) = &mut quest.payload else {
            panic!("quest payload")
        };
        quest.state_id = active_state;
        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_PICKUP_TEXT_ID)
        );

        let inventory = rpg
            .aggregates
            .iter_mut()
            .find(|aggregate| aggregate.persistent_id == fixture.player_inventory_id)
            .expect("inventory aggregate");
        let RpgAggregatePayloadV1::Inventory(inventory) = &mut inventory.payload else {
            panic!("inventory payload")
        };
        inventory.item_ids.push(fixture.pickup_item_id);
        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_EQUIP_TEXT_ID)
        );

        let slot_id = fixture
            .activated_project
            .rpg_definitions
            .abilities
            .first()
            .expect("ability definition")
            .required_equipment_slot_id
            .clone();
        let equipment = rpg
            .aggregates
            .iter_mut()
            .find(|aggregate| aggregate.persistent_id == fixture.player_equipment_id)
            .expect("equipment aggregate");
        let RpgAggregatePayloadV1::Equipment(equipment) = &mut equipment.payload else {
            panic!("equipment payload")
        };
        equipment.assignments.push(EquipmentSlotAssignmentV1 {
            slot_id,
            item_id: fixture.pickup_item_id,
        });
        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_COMBAT_TEXT_ID)
        );

        let npc = rpg
            .aggregates
            .iter_mut()
            .find(|aggregate| aggregate.persistent_id == fixture.npc_character_id)
            .expect("npc aggregate");
        let RpgAggregatePayloadV1::Character(npc) = &mut npc.payload else {
            panic!("npc payload")
        };
        npc.resources
            .iter_mut()
            .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
            .expect("npc health")
            .current_value = 0;
        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_RELAY_TEXT_ID)
        );

        let relay = rpg
            .aggregates
            .iter_mut()
            .find(|aggregate| aggregate.persistent_id == fixture.interactive_object_id)
            .expect("relay aggregate");
        let RpgAggregatePayloadV1::InteractiveObject(relay) = &mut relay.payload else {
            panic!("relay payload")
        };
        relay.state_id =
            next_contracts::ids::SchemaId::new(CORE_INTERACTIVE_OBJECT_ACTIVATED_STATE_ID)
                .expect("activated state");
        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_RETURN_TEXT_ID)
        );

        let quest = rpg
            .aggregates
            .iter_mut()
            .find(|aggregate| aggregate.persistent_id == fixture.quest_id)
            .expect("quest aggregate");
        let RpgAggregatePayloadV1::Quest(quest) = &mut quest.payload else {
            panic!("quest payload")
        };
        quest.state_id = completed_state;
        assert_eq!(
            frontier_relay_action_text_id(&fixture, &rpg),
            Some(HUD_ACTION_COMPLETE_TEXT_ID)
        );

        std::fs::remove_dir_all(root).expect("cleanup");
    }
}
