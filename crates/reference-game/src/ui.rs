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
use crate::rpg::aggregate_payload;
use crate::session::ReferenceGameSession;

pub const HUD_SURFACE_ID: &str = "nextengine.ui.surface.hud";
pub const HUD_STATUS_PANEL_ID: &str = "nextengine.ui.panel.hud.status";
pub const HUD_HEALTH_ELEMENT_ID: &str = "nextengine.ui.element.hud.health";
pub const HUD_QUEST_ELEMENT_ID: &str = "nextengine.ui.element.hud.quest";
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

/// Builds the full per-tick semantic UI set: the HUD projection plus the
/// pause menu when the committed frame carried a `ui-back` suspend request.
pub fn live_semantic_ui_records(
    snapshot_epoch: ContentHash,
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
    ui_suspend_causal_hash: Option<ContentHash>,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let mut records = hud_semantic_ui_records(snapshot_epoch, fixture, rpg)?;
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
