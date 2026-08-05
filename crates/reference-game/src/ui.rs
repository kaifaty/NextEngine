//! Semantic UI projection for the reference game.
//!
//! The projection reads the authoritative RPG snapshot through immutable
//! contract types and produces canonical `SemanticUiPresentationRecordV1`
//! records for the presentation extraction boundary. UI text travels as
//! stable text references; the localized catalog lands with the text-schema
//! sub-increment.

use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    CORE_UI_BACK_ACTION_ID, CORE_UI_CONFIRM_ACTION_ID, CORE_UI_NAVIGATE_ACTION_ID,
};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::presentation::{
    SemanticUiPresentationRecordV1, UiAccessibilityRoleV1, UiActionAffordanceV1, UiElementRoleV1,
    UiElementValueV1, UiSemanticElementV1, UiStyleRoleV1, UiTextArgumentV1, UiTextRefV1,
};
use next_contracts::project::domain_hash;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2};

use crate::ReferenceGameError;
use crate::dialogue::ReferenceDialogueUiV1;
use crate::input::ReferenceUiScreenV1;
use crate::rpg::aggregate_payload;
use crate::session::ReferenceGameSession;

pub const HUD_SURFACE_ID: &str = "nextengine.ui.surface.hud";
pub const HUD_STATUS_PANEL_ID: &str = "nextengine.ui.panel.hud.status";
pub const HUD_HEALTH_ELEMENT_ID: &str = "nextengine.ui.element.hud.health";
pub const HUD_QUEST_ELEMENT_ID: &str = "nextengine.ui.element.hud.quest";
pub const HUD_SUBTITLE_ELEMENT_ID: &str = "nextengine.ui.element.hud.subtitle";
pub const HUD_HEALTH_TEXT_ID: &str = "nextengine.ui.text.hud.health";
pub const HUD_QUEST_TEXT_ID: &str = "nextengine.ui.text.hud.quest-state";

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
    hud_semantic_ui_records_for_ids(snapshot_epoch, fixture.body_id, fixture.quest_id, rpg)
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
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(HUD_SURFACE_ID)?,
            schema_id(HUD_STATUS_PANEL_ID)?,
            source_snapshot_hash,
            UiSemanticElementV1::new(
                schema_id(HUD_HEALTH_ELEMENT_ID)?,
                UiElementRoleV1::Meter,
                UiStyleRoleV1::Default,
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
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(HUD_SURFACE_ID)?,
            schema_id(HUD_STATUS_PANEL_ID)?,
            source_snapshot_hash,
            UiSemanticElementV1::new(
                schema_id(HUD_QUEST_ELEMENT_ID)?,
                UiElementRoleV1::Label,
                UiStyleRoleV1::Muted,
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

/// Builds the full per-tick semantic UI set: the always-on HUD plus the
/// currently open read-only screen (inventory/equipment or quest journal)
/// plus the modal dialogue surface while the driver dialogue state is open
/// plus the pause menu when the committed frame carried a `ui-back` suspend
/// request.
///
/// Screen surfaces publish only while their `ReferenceUiScreenV1` state is
/// open (S3: the driver derives that state from committed
/// `ui-inventory`/`ui-journal`/`ui-back` actions); they stay read-only views
/// and never carry action affordances. The dialogue surface publishes only
/// while the `ReferenceDialogueUiV1` state is open (S4) and carries the
/// authored choices with their selection and affordances.
pub fn live_semantic_ui_records(
    snapshot_epoch: ContentHash,
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
    ui_screen: ReferenceUiScreenV1,
    dialogue: ReferenceDialogueUiV1,
    ui_suspend_causal_hash: Option<ContentHash>,
    active_subtitle: Option<SchemaId>,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let mut records = hud_semantic_ui_records(snapshot_epoch, fixture, rpg)?;
    // Voice-absent subtitle fallback (A5): the active speech cue's localized
    // text rides the HUD as a `Subtitle`-role element. It is presentation
    // projection only; the element never feeds actions or gameplay state.
    if let Some(text_id) = active_subtitle {
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(HUD_SURFACE_ID)?,
            schema_id(HUD_STATUS_PANEL_ID)?,
            domain_hash(
                "nextengine.ui-source.rpg-snapshot.v1",
                &rpg.canonical_bytes()?,
            ),
            UiSemanticElementV1::new(
                schema_id(HUD_SUBTITLE_ELEMENT_ID)?,
                UiElementRoleV1::Subtitle,
                UiStyleRoleV1::Muted,
                UiAccessibilityRoleV1::Standard,
                true,
                true,
                false,
                Some(UiTextRefV1::new(text_id, Vec::new())?),
                UiElementValueV1::None,
                Vec::new(),
            )?,
        )?);
    }
    match ui_screen {
        ReferenceUiScreenV1::None => {}
        ReferenceUiScreenV1::Inventory => {
            records.extend(inventory_semantic_ui_records(snapshot_epoch, fixture, rpg)?);
        }
        ReferenceUiScreenV1::Journal => {
            records.extend(quest_journal_semantic_ui_records(
                snapshot_epoch,
                fixture,
                rpg,
            )?);
        }
    }
    if let ReferenceDialogueUiV1::Open { selection } = dialogue {
        records.extend(crate::ui_dialogue::dialogue_semantic_ui_records(
            snapshot_epoch,
            fixture,
            rpg,
            selection,
        )?);
    }
    if let Some(causal_hash) = ui_suspend_causal_hash {
        records.extend(pause_menu_semantic_ui_records(
            snapshot_epoch,
            rpg,
            causal_hash,
        )?);
    }
    Ok(records)
}

pub const PAUSE_MENU_SURFACE_ID: &str = "nextengine.ui.surface.pause-menu";
pub const PAUSE_MENU_ROOT_PANEL_ID: &str = "nextengine.ui.panel.pause-menu.root";
pub const PAUSE_MENU_TITLE_ELEMENT_ID: &str = "nextengine.ui.element.pause-menu.title";
pub const PAUSE_MENU_RESUME_ELEMENT_ID: &str = "nextengine.ui.element.pause-menu.resume";
pub const PAUSE_MENU_SAVE_ELEMENT_ID: &str = "nextengine.ui.element.pause-menu.save";
pub const PAUSE_MENU_LOAD_ELEMENT_ID: &str = "nextengine.ui.element.pause-menu.load";
pub const PAUSE_MENU_TITLE_TEXT_ID: &str = "nextengine.ui.text.pause-menu.title";
pub const PAUSE_MENU_RESUME_TEXT_ID: &str = "nextengine.ui.text.pause-menu.resume";
pub const PAUSE_MENU_SAVE_TEXT_ID: &str = "nextengine.ui.text.pause-menu.save";
pub const PAUSE_MENU_LOAD_TEXT_ID: &str = "nextengine.ui.text.pause-menu.load";

/// Builds the canonical pause-menu records for the suspending publication.
///
/// The menu appears on the same tick whose committed `ui-back` action
/// requested the declared `Suspended` lifecycle transition; save/load
/// affordances reference the production checkpoint/restore paths through
/// their universal UI action ids and never carry rendered text.
pub fn pause_menu_semantic_ui_records(
    snapshot_epoch: ContentHash,
    rpg: &RpgSnapshotV2,
    pause_causal_hash: ContentHash,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let mut preimage = rpg.canonical_bytes()?;
    preimage.extend_from_slice(pause_causal_hash.as_bytes());
    let source_snapshot_hash = domain_hash("nextengine.ui-source.pause-menu.v1", &preimage);
    let confirm = UiActionAffordanceV1 {
        action_id: schema_id(CORE_UI_CONFIRM_ACTION_ID)?,
        enabled: true,
    };
    let back = UiActionAffordanceV1 {
        action_id: schema_id(CORE_UI_BACK_ACTION_ID)?,
        enabled: true,
    };
    let navigate = UiActionAffordanceV1 {
        action_id: schema_id(CORE_UI_NAVIGATE_ACTION_ID)?,
        enabled: true,
    };
    let mut records = Vec::new();
    let mut push = |element_id: &str,
                    role: UiElementRoleV1,
                    style: UiStyleRoleV1,
                    accessibility: UiAccessibilityRoleV1,
                    selected: bool,
                    text_id: &str,
                    affordances: Vec<UiActionAffordanceV1>|
     -> Result<(), ReferenceGameError> {
        records.push(SemanticUiPresentationRecordV1::new(
            snapshot_epoch,
            schema_id(PAUSE_MENU_SURFACE_ID)?,
            schema_id(PAUSE_MENU_ROOT_PANEL_ID)?,
            source_snapshot_hash,
            UiSemanticElementV1::new(
                schema_id(element_id)?,
                role,
                style,
                accessibility,
                true,
                true,
                selected,
                Some(UiTextRefV1::new(schema_id(text_id)?, Vec::new())?),
                UiElementValueV1::None,
                affordances,
            )?,
        )?);
        Ok(())
    };
    push(
        PAUSE_MENU_TITLE_ELEMENT_ID,
        UiElementRoleV1::Label,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Heading,
        false,
        PAUSE_MENU_TITLE_TEXT_ID,
        Vec::new(),
    )?;
    push(
        PAUSE_MENU_RESUME_ELEMENT_ID,
        UiElementRoleV1::Button,
        UiStyleRoleV1::Accent,
        UiAccessibilityRoleV1::Standard,
        true,
        PAUSE_MENU_RESUME_TEXT_ID,
        vec![confirm.clone(), back, navigate.clone()],
    )?;
    push(
        PAUSE_MENU_SAVE_ELEMENT_ID,
        UiElementRoleV1::Button,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Standard,
        false,
        PAUSE_MENU_SAVE_TEXT_ID,
        vec![confirm.clone(), navigate.clone()],
    )?;
    push(
        PAUSE_MENU_LOAD_ELEMENT_ID,
        UiElementRoleV1::Button,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Standard,
        false,
        PAUSE_MENU_LOAD_TEXT_ID,
        vec![confirm, navigate],
    )?;
    Ok(records)
}

pub const INVENTORY_SURFACE_ID: &str = "nextengine.ui.surface.inventory";
pub const INVENTORY_PANEL_ID: &str = "nextengine.ui.panel.inventory.root";
pub const INVENTORY_TITLE_ELEMENT_ID: &str = "nextengine.ui.element.inventory.title";
pub const INVENTORY_EMPTY_ELEMENT_ID: &str = "nextengine.ui.element.inventory.empty";
pub const INVENTORY_ITEM_ELEMENT_PREFIX: &str = "nextengine.ui.element.inventory.item.";
pub const EQUIPMENT_PANEL_ID: &str = "nextengine.ui.panel.equipment.root";
pub const EQUIPMENT_TITLE_ELEMENT_ID: &str = "nextengine.ui.element.equipment.title";
pub const EQUIPMENT_EMPTY_ELEMENT_ID: &str = "nextengine.ui.element.equipment.empty";
pub const EQUIPMENT_SLOT_ELEMENT_PREFIX: &str = "nextengine.ui.element.equipment.slot.";
pub const JOURNAL_SURFACE_ID: &str = "nextengine.ui.surface.quest-journal";
pub const JOURNAL_PANEL_ID: &str = "nextengine.ui.panel.quest-journal.root";
pub const JOURNAL_TITLE_ELEMENT_ID: &str = "nextengine.ui.element.quest-journal.title";
pub const JOURNAL_ENTRY_ELEMENT_PREFIX: &str = "nextengine.ui.element.quest-journal.entry.";

pub const INVENTORY_TITLE_TEXT_ID: &str = "nextengine.ui.text.inventory.title";
pub const INVENTORY_ITEM_ROW_TEXT_ID: &str = "nextengine.ui.text.inventory.item-row";
pub const INVENTORY_EMPTY_TEXT_ID: &str = "nextengine.ui.text.inventory.empty";
pub const EQUIPMENT_TITLE_TEXT_ID: &str = "nextengine.ui.text.equipment.title";
pub const EQUIPMENT_SLOT_ROW_TEXT_ID: &str = "nextengine.ui.text.equipment.slot-row";
pub const JOURNAL_TITLE_TEXT_ID: &str = "nextengine.ui.text.journal.title";
pub const JOURNAL_ENTRY_TEXT_ID: &str = "nextengine.ui.text.journal.entry";
/// Display-name text id carried by the single reference item definition; the
/// live path and verification fixtures map every known reference item
/// aggregate to it.
pub const REFERENCE_ITEM_DISPLAY_TEXT_ID: &str = "nextengine.reference.item.training-sword";
/// Display-name text id carried by the single reference quest definition.
pub const REFERENCE_QUEST_DISPLAY_TEXT_ID: &str = "nextengine.reference.quest.a-helping-hand";

/// Builds the canonical read-only inventory/equipment screen records.
///
/// The screen owns one surface with two panels. Panels appear only when the
/// player character aggregate references the corresponding inventory or
/// equipment aggregate; an empty payload yields a muted `(empty)`
/// placeholder row, while rows whose item aggregate or caller-provided
/// display name is missing are skipped deterministically. Element ids carry
/// the zero-based payload index (`...item.0`, `...slot.1`). Equipment rows
/// reference the authoritative slot id directly as a text id, so the text
/// catalog covers `nextengine.rpg.equipment-slot.*` entries. All elements
/// are enabled, visible, unselected and carry no affordances (Q3A).
pub fn inventory_semantic_ui_records_for_ids(
    snapshot_epoch: ContentHash,
    player_character_id: PersistentId,
    item_display_names: &[(PersistentId, SchemaId)],
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let source_snapshot_hash = domain_hash(
        "nextengine.ui-source.rpg-snapshot.v1",
        &rpg.canonical_bytes()?,
    );
    let mut records = Vec::new();
    let Some(RpgAggregatePayloadV1::Character(character)) =
        aggregate_payload(rpg, RpgAggregateKindV1::Character, player_character_id)
    else {
        return Ok(records);
    };
    if let Some(inventory_id) = character.inventory_id
        && let Some(RpgAggregatePayloadV1::Inventory(inventory)) =
            aggregate_payload(rpg, RpgAggregateKindV1::Inventory, inventory_id)
    {
        push_read_only_element(
            &mut records,
            snapshot_epoch,
            INVENTORY_SURFACE_ID,
            INVENTORY_PANEL_ID,
            source_snapshot_hash,
            INVENTORY_TITLE_ELEMENT_ID,
            UiElementRoleV1::Label,
            UiStyleRoleV1::Default,
            UiAccessibilityRoleV1::Heading,
            INVENTORY_TITLE_TEXT_ID,
            Vec::new(),
        )?;
        if inventory.item_ids.is_empty() {
            push_read_only_element(
                &mut records,
                snapshot_epoch,
                INVENTORY_SURFACE_ID,
                INVENTORY_PANEL_ID,
                source_snapshot_hash,
                INVENTORY_EMPTY_ELEMENT_ID,
                UiElementRoleV1::ListItem,
                UiStyleRoleV1::Muted,
                UiAccessibilityRoleV1::Standard,
                INVENTORY_EMPTY_TEXT_ID,
                Vec::new(),
            )?;
        } else {
            for (index, item_id) in inventory.item_ids.iter().enumerate() {
                let Some(RpgAggregatePayloadV1::Item(item)) =
                    aggregate_payload(rpg, RpgAggregateKindV1::Item, *item_id)
                else {
                    continue;
                };
                let Some(display_name) = item_display_names
                    .iter()
                    .find(|(id, _)| id == item_id)
                    .map(|(_, text_id)| text_id)
                else {
                    continue;
                };
                let element_id = format!("{INVENTORY_ITEM_ELEMENT_PREFIX}{index}");
                push_read_only_element(
                    &mut records,
                    snapshot_epoch,
                    INVENTORY_SURFACE_ID,
                    INVENTORY_PANEL_ID,
                    source_snapshot_hash,
                    &element_id,
                    UiElementRoleV1::ListItem,
                    UiStyleRoleV1::Default,
                    UiAccessibilityRoleV1::Standard,
                    INVENTORY_ITEM_ROW_TEXT_ID,
                    vec![
                        UiTextArgumentV1::TextId(display_name.clone()),
                        UiTextArgumentV1::SignedInteger(i64::from(item.quantity)),
                    ],
                )?;
            }
        }
    }
    if let Some(equipment_id) = character.equipment_id
        && let Some(RpgAggregatePayloadV1::Equipment(equipment)) =
            aggregate_payload(rpg, RpgAggregateKindV1::Equipment, equipment_id)
    {
        push_read_only_element(
            &mut records,
            snapshot_epoch,
            INVENTORY_SURFACE_ID,
            EQUIPMENT_PANEL_ID,
            source_snapshot_hash,
            EQUIPMENT_TITLE_ELEMENT_ID,
            UiElementRoleV1::Label,
            UiStyleRoleV1::Default,
            UiAccessibilityRoleV1::Heading,
            EQUIPMENT_TITLE_TEXT_ID,
            Vec::new(),
        )?;
        if equipment.assignments.is_empty() {
            push_read_only_element(
                &mut records,
                snapshot_epoch,
                INVENTORY_SURFACE_ID,
                EQUIPMENT_PANEL_ID,
                source_snapshot_hash,
                EQUIPMENT_EMPTY_ELEMENT_ID,
                UiElementRoleV1::ListItem,
                UiStyleRoleV1::Muted,
                UiAccessibilityRoleV1::Standard,
                INVENTORY_EMPTY_TEXT_ID,
                Vec::new(),
            )?;
        } else {
            for (index, assignment) in equipment.assignments.iter().enumerate() {
                let Some(display_name) = item_display_names
                    .iter()
                    .find(|(id, _)| *id == assignment.item_id)
                    .map(|(_, text_id)| text_id)
                else {
                    continue;
                };
                let element_id = format!("{EQUIPMENT_SLOT_ELEMENT_PREFIX}{index}");
                push_read_only_element(
                    &mut records,
                    snapshot_epoch,
                    INVENTORY_SURFACE_ID,
                    EQUIPMENT_PANEL_ID,
                    source_snapshot_hash,
                    &element_id,
                    UiElementRoleV1::ListItem,
                    UiStyleRoleV1::Default,
                    UiAccessibilityRoleV1::Standard,
                    EQUIPMENT_SLOT_ROW_TEXT_ID,
                    vec![
                        UiTextArgumentV1::TextId(assignment.slot_id.clone()),
                        UiTextArgumentV1::TextId(display_name.clone()),
                    ],
                )?;
            }
        }
    }
    Ok(records)
}

/// Builds the canonical read-only quest journal records.
///
/// One row per quest aggregate in canonical snapshot order (kind, persistent
/// id); quests without a caller-provided display name are skipped, and a
/// snapshot with no named quests yields no records at all (Q4A). Rows carry
/// the quest display name and the authoritative state id as text arguments.
pub fn quest_journal_semantic_ui_records_for_ids(
    snapshot_epoch: ContentHash,
    quest_display_names: &[(PersistentId, SchemaId)],
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let source_snapshot_hash = domain_hash(
        "nextengine.ui-source.rpg-snapshot.v1",
        &rpg.canonical_bytes()?,
    );
    let mut rows = Vec::new();
    for envelope in &rpg.aggregates {
        if envelope.aggregate_kind != RpgAggregateKindV1::Quest {
            continue;
        }
        let RpgAggregatePayloadV1::Quest(quest) = &envelope.payload else {
            continue;
        };
        let Some(display_name) = quest_display_names
            .iter()
            .find(|(id, _)| *id == envelope.persistent_id)
            .map(|(_, text_id)| text_id)
        else {
            continue;
        };
        let element_id = format!("{JOURNAL_ENTRY_ELEMENT_PREFIX}{}", rows.len());
        rows.push((element_id, display_name.clone(), quest.state_id.clone()));
    }
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    push_read_only_element(
        &mut records,
        snapshot_epoch,
        JOURNAL_SURFACE_ID,
        JOURNAL_PANEL_ID,
        source_snapshot_hash,
        JOURNAL_TITLE_ELEMENT_ID,
        UiElementRoleV1::Label,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Heading,
        JOURNAL_TITLE_TEXT_ID,
        Vec::new(),
    )?;
    for (element_id, display_name, state_id) in rows {
        push_read_only_element(
            &mut records,
            snapshot_epoch,
            JOURNAL_SURFACE_ID,
            JOURNAL_PANEL_ID,
            source_snapshot_hash,
            &element_id,
            UiElementRoleV1::ListItem,
            UiStyleRoleV1::Default,
            UiAccessibilityRoleV1::Standard,
            JOURNAL_ENTRY_TEXT_ID,
            vec![
                UiTextArgumentV1::TextId(display_name),
                UiTextArgumentV1::TextId(state_id),
            ],
        )?;
    }
    Ok(records)
}

/// Session-handle variant used by the live publication path; maps every
/// known reference item aggregate to its definition display-name text id.
pub fn inventory_semantic_ui_records(
    snapshot_epoch: ContentHash,
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    inventory_semantic_ui_records_for_ids(
        snapshot_epoch,
        fixture.body_id,
        &reference_item_display_names([fixture.pickup_item_id, fixture.npc_weapon_item_id])?,
        rpg,
    )
}

/// Session-handle variant used by the live publication path.
pub fn quest_journal_semantic_ui_records(
    snapshot_epoch: ContentHash,
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    quest_journal_semantic_ui_records_for_ids(
        snapshot_epoch,
        &[(
            fixture.quest_id,
            schema_id(REFERENCE_QUEST_DISPLAY_TEXT_ID)?,
        )],
        rpg,
    )
}

/// Builds the full always-on read-only set (HUD, inventory/equipment, quest
/// journal) without a session handle, for callers (verification fixtures,
/// tools) that hold the authoritative ids directly. `reference_item_ids`
/// lists every item aggregate defined by the reference item definition.
pub fn read_only_screen_semantic_ui_records_for_ids(
    snapshot_epoch: ContentHash,
    player_character_id: PersistentId,
    quest_id: PersistentId,
    reference_item_ids: &[PersistentId],
    rpg: &RpgSnapshotV2,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let mut records =
        hud_semantic_ui_records_for_ids(snapshot_epoch, player_character_id, quest_id, rpg)?;
    records.extend(inventory_semantic_ui_records_for_ids(
        snapshot_epoch,
        player_character_id,
        &reference_item_display_names(reference_item_ids.iter().copied())?,
        rpg,
    )?);
    records.extend(quest_journal_semantic_ui_records_for_ids(
        snapshot_epoch,
        &[(quest_id, schema_id(REFERENCE_QUEST_DISPLAY_TEXT_ID)?)],
        rpg,
    )?);
    Ok(records)
}

fn reference_item_display_names(
    item_ids: impl IntoIterator<Item = PersistentId>,
) -> Result<Vec<(PersistentId, SchemaId)>, ReferenceGameError> {
    item_ids
        .into_iter()
        .map(|item_id| Ok((item_id, schema_id(REFERENCE_ITEM_DISPLAY_TEXT_ID)?)))
        .collect()
}

#[allow(
    clippy::too_many_arguments,
    reason = "the read-only element record keeps every canonical presentation field explicit"
)]
fn push_read_only_element(
    records: &mut Vec<SemanticUiPresentationRecordV1>,
    snapshot_epoch: ContentHash,
    surface_id: &str,
    panel_id: &str,
    source_snapshot_hash: ContentHash,
    element_id: &str,
    role: UiElementRoleV1,
    style: UiStyleRoleV1,
    accessibility: UiAccessibilityRoleV1,
    text_id: &str,
    arguments: Vec<UiTextArgumentV1>,
) -> Result<(), ReferenceGameError> {
    records.push(SemanticUiPresentationRecordV1::new(
        snapshot_epoch,
        schema_id(surface_id)?,
        schema_id(panel_id)?,
        source_snapshot_hash,
        UiSemanticElementV1::new(
            schema_id(element_id)?,
            role,
            style,
            accessibility,
            true,
            true,
            false,
            Some(UiTextRefV1::new(schema_id(text_id)?, arguments)?),
            UiElementValueV1::None,
            Vec::new(),
        )?,
    )?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_contracts::ids::{AssetId, PersistentId};
    use next_contracts::rpg::{
        CharacterPayloadV1, DefinitionRefV1, EquipmentPayloadV1, EquipmentSlotAssignmentV1,
        InventoryPayloadV1, ItemPayloadV1, QuestPayloadV1, RpgAggregateEnvelopeV1,
    };

    use crate::rpg::reference_aggregate;

    fn persistent(byte: u8) -> PersistentId {
        PersistentId::from_bytes([byte; 16])
    }

    fn epoch() -> ContentHash {
        ContentHash::from_bytes([0xee; 32])
    }

    fn text_id(value: &str) -> SchemaId {
        SchemaId::new(value).expect("test text id")
    }

    fn character(
        inventory: Option<PersistentId>,
        equipment: Option<PersistentId>,
    ) -> RpgAggregateEnvelopeV1 {
        reference_aggregate(
            persistent(0x01),
            0x54,
            RpgAggregatePayloadV1::Character(CharacterPayloadV1 {
                inventory_id: inventory,
                equipment_id: equipment,
                resources: Vec::new(),
                skills: Vec::new(),
            }),
        )
    }

    fn inventory(id: PersistentId, item_ids: Vec<PersistentId>) -> RpgAggregateEnvelopeV1 {
        reference_aggregate(
            id,
            0x5d,
            RpgAggregatePayloadV1::Inventory(InventoryPayloadV1 {
                owner_id: persistent(0x01),
                capacity: 8,
                item_ids,
                reservations: Vec::new(),
            }),
        )
    }

    fn equipment(
        id: PersistentId,
        assignments: Vec<EquipmentSlotAssignmentV1>,
    ) -> RpgAggregateEnvelopeV1 {
        reference_aggregate(
            id,
            0x5e,
            RpgAggregatePayloadV1::Equipment(EquipmentPayloadV1 {
                character_id: persistent(0x01),
                slot_policy: DefinitionRefV1::Exact {
                    asset_id: AssetId::from_bytes([0x5e; 16]),
                    content_hash: ContentHash::from_bytes([0x5e; 32]),
                },
                assignments,
            }),
        )
    }

    fn item(id: PersistentId, quantity: u32) -> RpgAggregateEnvelopeV1 {
        reference_aggregate(
            id,
            0x5f,
            RpgAggregatePayloadV1::Item(ItemPayloadV1 {
                quantity,
                durability: 100,
                custom_state: Vec::new(),
            }),
        )
    }

    fn quest(id: PersistentId, state: &str) -> RpgAggregateEnvelopeV1 {
        reference_aggregate(
            id,
            0x5b,
            RpgAggregatePayloadV1::Quest(QuestPayloadV1 {
                state_id: text_id(state),
            }),
        )
    }

    fn snapshot(mut aggregates: Vec<RpgAggregateEnvelopeV1>) -> RpgSnapshotV2 {
        aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
        RpgSnapshotV2 { aggregates }
    }

    #[test]
    fn projections_are_empty_without_source_aggregates() {
        let rpg = snapshot(Vec::new());
        let records = inventory_semantic_ui_records_for_ids(epoch(), persistent(0x01), &[], &rpg)
            .expect("records");
        assert!(records.is_empty());
        let journal =
            quest_journal_semantic_ui_records_for_ids(epoch(), &[], &rpg).expect("journal");
        assert!(journal.is_empty());
    }

    #[test]
    fn empty_panels_publish_read_only_placeholder_rows() {
        let rpg = snapshot(vec![
            character(Some(persistent(0x02)), Some(persistent(0x03))),
            inventory(persistent(0x02), Vec::new()),
            equipment(persistent(0x03), Vec::new()),
        ]);
        let records = inventory_semantic_ui_records_for_ids(epoch(), persistent(0x01), &[], &rpg)
            .expect("records");
        let element_ids = records
            .iter()
            .map(|record| record.element.element_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            element_ids,
            vec![
                INVENTORY_TITLE_ELEMENT_ID,
                INVENTORY_EMPTY_ELEMENT_ID,
                EQUIPMENT_TITLE_ELEMENT_ID,
                EQUIPMENT_EMPTY_ELEMENT_ID,
            ]
        );
        assert!(
            records.iter().all(|record| {
                record.snapshot_epoch == epoch()
                    && record.surface_id.as_str() == INVENTORY_SURFACE_ID
                    && record.element.affordances.is_empty()
                    && record.element.enabled
                    && record.element.visible
                    && !record.element.selected
            }),
            "read-only rows stay enabled, visible, unselected and affordance-free"
        );
        let empty_row = records
            .iter()
            .find(|record| record.element.element_id.as_str() == INVENTORY_EMPTY_ELEMENT_ID)
            .expect("empty row");
        assert_eq!(empty_row.element.role, UiElementRoleV1::ListItem);
        assert_eq!(empty_row.element.style_role, UiStyleRoleV1::Muted);
        assert_eq!(
            empty_row
                .element
                .text_or_none
                .as_ref()
                .expect("empty text")
                .text_id
                .as_str(),
            INVENTORY_EMPTY_TEXT_ID
        );
    }

    #[test]
    fn inventory_rows_carry_display_name_and_quantity_in_payload_order() {
        let rpg = snapshot(vec![
            character(Some(persistent(0x02)), None),
            inventory(persistent(0x02), vec![persistent(0x0a), persistent(0x0b)]),
            item(persistent(0x0a), 2),
            item(persistent(0x0b), 3),
        ]);
        let names = [
            (persistent(0x0a), text_id(REFERENCE_ITEM_DISPLAY_TEXT_ID)),
            (persistent(0x0b), text_id(REFERENCE_ITEM_DISPLAY_TEXT_ID)),
        ];
        let records =
            inventory_semantic_ui_records_for_ids(epoch(), persistent(0x01), &names, &rpg)
                .expect("records");
        assert_eq!(records.len(), 3);
        let first = &records[1];
        assert_eq!(
            first.element.element_id.as_str(),
            "nextengine.ui.element.inventory.item.0"
        );
        assert_eq!(first.element.role, UiElementRoleV1::ListItem);
        assert_eq!(
            first
                .element
                .text_or_none
                .as_ref()
                .expect("row text")
                .arguments,
            vec![
                UiTextArgumentV1::TextId(text_id(REFERENCE_ITEM_DISPLAY_TEXT_ID)),
                UiTextArgumentV1::SignedInteger(2),
            ]
        );
        assert_eq!(
            records[2].element.element_id.as_str(),
            "nextengine.ui.element.inventory.item.1"
        );
    }

    #[test]
    fn inventory_rows_skip_items_without_display_names() {
        let rpg = snapshot(vec![
            character(Some(persistent(0x02)), None),
            inventory(persistent(0x02), vec![persistent(0x0a)]),
            item(persistent(0x0a), 1),
        ]);
        let records = inventory_semantic_ui_records_for_ids(epoch(), persistent(0x01), &[], &rpg)
            .expect("records");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].element.element_id.as_str(),
            INVENTORY_TITLE_ELEMENT_ID
        );
    }

    #[test]
    fn equipment_rows_reference_slot_and_item_text_ids() {
        let rpg = snapshot(vec![
            character(None, Some(persistent(0x3))),
            equipment(
                persistent(0x03),
                vec![EquipmentSlotAssignmentV1 {
                    slot_id: text_id("nextengine.rpg.equipment-slot.main-hand"),
                    item_id: persistent(0x0a),
                }],
            ),
            item(persistent(0x0a), 1),
        ]);
        let names = [(persistent(0x0a), text_id(REFERENCE_ITEM_DISPLAY_TEXT_ID))];
        let records =
            inventory_semantic_ui_records_for_ids(epoch(), persistent(0x01), &names, &rpg)
                .expect("records");
        assert_eq!(records.len(), 2);
        let row = &records[1];
        assert_eq!(row.semantic_path_id.as_str(), EQUIPMENT_PANEL_ID);
        assert_eq!(
            row.element.element_id.as_str(),
            "nextengine.ui.element.equipment.slot.0"
        );
        assert_eq!(
            row.element
                .text_or_none
                .as_ref()
                .expect("slot text")
                .arguments,
            vec![
                UiTextArgumentV1::TextId(text_id("nextengine.rpg.equipment-slot.main-hand")),
                UiTextArgumentV1::TextId(text_id(REFERENCE_ITEM_DISPLAY_TEXT_ID)),
            ]
        );
    }

    #[test]
    fn journal_lists_named_quests_in_canonical_order() {
        let rpg = snapshot(vec![
            quest(persistent(0x0b), "nextengine.reference.quest.active"),
            quest(persistent(0x0a), "nextengine.reference.quest.available"),
        ]);
        let names = [
            (persistent(0x0a), text_id(REFERENCE_QUEST_DISPLAY_TEXT_ID)),
            (persistent(0x0b), text_id(REFERENCE_QUEST_DISPLAY_TEXT_ID)),
        ];
        let records =
            quest_journal_semantic_ui_records_for_ids(epoch(), &names, &rpg).expect("records");
        assert_eq!(records.len(), 3);
        assert_eq!(
            records[0].element.element_id.as_str(),
            JOURNAL_TITLE_ELEMENT_ID
        );
        assert_eq!(records[0].element.role, UiElementRoleV1::Label);
        assert_eq!(
            records[0].element.accessibility_role,
            UiAccessibilityRoleV1::Heading
        );
        // Canonical aggregate order (persistent id) drives entry order.
        assert_eq!(
            records[1].element.element_id.as_str(),
            "nextengine.ui.element.quest-journal.entry.0"
        );
        assert_eq!(
            records[1]
                .element
                .text_or_none
                .as_ref()
                .expect("entry text")
                .arguments,
            vec![
                UiTextArgumentV1::TextId(text_id(REFERENCE_QUEST_DISPLAY_TEXT_ID)),
                UiTextArgumentV1::TextId(text_id("nextengine.reference.quest.available")),
            ]
        );
        assert!(
            records.iter().all(|record| {
                record.surface_id.as_str() == JOURNAL_SURFACE_ID
                    && record.semantic_path_id.as_str() == JOURNAL_PANEL_ID
                    && record.element.affordances.is_empty()
            }),
            "journal rows stay read-only on the quest-journal surface"
        );
    }

    #[test]
    fn journal_stays_empty_without_named_quests() {
        let rpg = snapshot(vec![quest(
            persistent(0x0a),
            "nextengine.reference.quest.available",
        )]);
        let records =
            quest_journal_semantic_ui_records_for_ids(epoch(), &[], &rpg).expect("records");
        assert!(records.is_empty());
    }
}
