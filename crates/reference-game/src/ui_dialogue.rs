//! Live dialogue surface projection (semantic UI S4, Q2A).
//!
//! While the driver dialogue state is open the surface publishes the authored
//! node text and the two authored choices with their `ui-confirm`/`ui-nav`/
//! `ui-back` affordances and the deterministic selection. The projection reads
//! only the authoritative RPG snapshot and the presentation selection state;
//! the choice itself reaches the runtime exclusively through the production
//! interaction accept path (see `crate::dialogue`).

use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    CORE_UI_BACK_ACTION_ID, CORE_UI_CONFIRM_ACTION_ID, CORE_UI_NAVIGATE_ACTION_ID,
};
use next_contracts::presentation::{
    SemanticUiPresentationRecordV1, UiAccessibilityRoleV1, UiActionAffordanceV1, UiElementRoleV1,
    UiElementValueV1, UiSemanticElementV1, UiStyleRoleV1, UiTextRefV1,
};
use next_contracts::project::domain_hash;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgSnapshotV2};

use crate::ReferenceGameError;
use crate::dialogue::ReferenceDialogueChoiceV1;
use crate::rpg::aggregate_payload;
use crate::session::ReferenceGameSession;

pub const DIALOGUE_SURFACE_ID: &str = "nextengine.ui.surface.dialogue";
pub const DIALOGUE_PANEL_ID: &str = "nextengine.ui.panel.dialogue.root";
pub const DIALOGUE_TITLE_ELEMENT_ID: &str = "nextengine.ui.element.dialogue.title";
pub const DIALOGUE_NODE_ELEMENT_ID: &str = "nextengine.ui.element.dialogue.node-text";
pub const DIALOGUE_CHOICE_ACCEPT_ELEMENT_ID: &str = "nextengine.ui.element.dialogue.choice-accept";
pub const DIALOGUE_CHOICE_LEAVE_ELEMENT_ID: &str = "nextengine.ui.element.dialogue.choice-leave";

pub const DIALOGUE_TITLE_TEXT_ID: &str = "nextengine.ui.text.dialogue.title";
pub const DIALOGUE_CHOICE_ACCEPT_TEXT_ID: &str = "nextengine.ui.text.dialogue.choice-accept";
pub const DIALOGUE_CHOICE_LEAVE_TEXT_ID: &str = "nextengine.ui.text.dialogue.choice-leave";

/// Builds the canonical dialogue surface records for one live publication.
pub fn dialogue_semantic_ui_records(
    snapshot_epoch: ContentHash,
    fixture: &ReferenceGameSession,
    rpg: &RpgSnapshotV2,
    selection: ReferenceDialogueChoiceV1,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    dialogue_semantic_ui_records_for_ids(snapshot_epoch, fixture.dialogue_id, rpg, selection)
}

/// Builds the canonical dialogue surface records without a session handle,
/// for callers (verification fixtures, tools) that hold the authoritative ids
/// directly. An empty record set is returned when the dialogue aggregate is
/// absent (for example a non-interactive scenario).
pub fn dialogue_semantic_ui_records_for_ids(
    snapshot_epoch: ContentHash,
    dialogue_id: PersistentId,
    rpg: &RpgSnapshotV2,
    selection: ReferenceDialogueChoiceV1,
) -> Result<Vec<SemanticUiPresentationRecordV1>, ReferenceGameError> {
    let Some(RpgAggregatePayloadV1::Dialogue(dialogue)) =
        aggregate_payload(rpg, RpgAggregateKindV1::Dialogue, dialogue_id)
    else {
        return Ok(Vec::new());
    };
    let mut preimage = rpg.canonical_bytes()?;
    preimage.push(match selection {
        ReferenceDialogueChoiceV1::Accept => 0,
        ReferenceDialogueChoiceV1::Leave => 1,
    });
    let source_snapshot_hash = domain_hash("nextengine.ui-source.dialogue.v1", &preimage);
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
            schema_id(DIALOGUE_SURFACE_ID)?,
            schema_id(DIALOGUE_PANEL_ID)?,
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
        DIALOGUE_TITLE_ELEMENT_ID,
        UiElementRoleV1::Label,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Heading,
        false,
        DIALOGUE_TITLE_TEXT_ID,
        Vec::new(),
    )?;
    // Dialogue node identifiers double as text identifiers (S1).
    push(
        DIALOGUE_NODE_ELEMENT_ID,
        UiElementRoleV1::Label,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Standard,
        false,
        dialogue.node_id.as_str(),
        Vec::new(),
    )?;
    push(
        DIALOGUE_CHOICE_ACCEPT_ELEMENT_ID,
        UiElementRoleV1::Button,
        UiStyleRoleV1::Accent,
        UiAccessibilityRoleV1::Standard,
        selection == ReferenceDialogueChoiceV1::Accept,
        DIALOGUE_CHOICE_ACCEPT_TEXT_ID,
        vec![confirm.clone(), back.clone(), navigate.clone()],
    )?;
    push(
        DIALOGUE_CHOICE_LEAVE_ELEMENT_ID,
        UiElementRoleV1::Button,
        UiStyleRoleV1::Default,
        UiAccessibilityRoleV1::Standard,
        selection == ReferenceDialogueChoiceV1::Leave,
        DIALOGUE_CHOICE_LEAVE_TEXT_ID,
        vec![confirm, back, navigate],
    )?;
    Ok(records)
}

fn schema_id(value: &str) -> Result<SchemaId, ReferenceGameError> {
    Ok(SchemaId::new(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpg::reference_aggregate;
    use next_contracts::rpg::{DialoguePayloadV1, RpgAggregateEnvelopeV1};

    fn persistent(byte: u8) -> PersistentId {
        PersistentId::from_bytes([byte; 16])
    }

    fn epoch() -> ContentHash {
        ContentHash::from_bytes([0xee; 32])
    }

    fn dialogue(id: PersistentId, node: &str) -> RpgAggregateEnvelopeV1 {
        reference_aggregate(
            id,
            0x5a,
            RpgAggregatePayloadV1::Dialogue(DialoguePayloadV1 {
                speaker_id: persistent(0x59),
                listener_id: persistent(0x54),
                node_id: SchemaId::new(node).expect("node id"),
            }),
        )
    }

    fn snapshot(mut aggregates: Vec<RpgAggregateEnvelopeV1>) -> RpgSnapshotV2 {
        aggregates.sort_by_key(|aggregate| (aggregate.aggregate_kind, aggregate.persistent_id));
        RpgSnapshotV2 { aggregates }
    }

    #[test]
    fn dialogue_surface_is_empty_without_the_dialogue_aggregate() {
        let rpg = snapshot(Vec::new());
        let records = dialogue_semantic_ui_records_for_ids(
            epoch(),
            persistent(0x5a),
            &rpg,
            ReferenceDialogueChoiceV1::Accept,
        )
        .expect("records");
        assert!(records.is_empty());
    }

    #[test]
    fn dialogue_surface_marks_the_selection_and_choice_affordances() {
        let rpg = snapshot(vec![dialogue(
            persistent(0x5a),
            "nextengine.reference.dialogue.offer",
        )]);
        let records = dialogue_semantic_ui_records_for_ids(
            epoch(),
            persistent(0x5a),
            &rpg,
            ReferenceDialogueChoiceV1::Leave,
        )
        .expect("records");
        let mut element_ids = records
            .iter()
            .map(|record| record.element.element_id.as_str())
            .collect::<Vec<_>>();
        element_ids.sort_unstable();
        assert_eq!(
            element_ids,
            vec![
                DIALOGUE_CHOICE_ACCEPT_ELEMENT_ID,
                DIALOGUE_CHOICE_LEAVE_ELEMENT_ID,
                DIALOGUE_NODE_ELEMENT_ID,
                DIALOGUE_TITLE_ELEMENT_ID,
            ]
        );
        let leave = records
            .iter()
            .find(|record| record.element.element_id.as_str() == DIALOGUE_CHOICE_LEAVE_ELEMENT_ID)
            .expect("leave choice");
        let accept = records
            .iter()
            .find(|record| record.element.element_id.as_str() == DIALOGUE_CHOICE_ACCEPT_ELEMENT_ID)
            .expect("accept choice");
        assert!(leave.element.selected);
        assert!(!accept.element.selected);
        for choice in [accept, leave] {
            let affordance_ids = choice
                .element
                .affordances
                .iter()
                .map(|affordance| affordance.action_id.as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                affordance_ids,
                vec![
                    CORE_UI_BACK_ACTION_ID,
                    CORE_UI_CONFIRM_ACTION_ID,
                    CORE_UI_NAVIGATE_ACTION_ID
                ],
                "affordances are stored in canonical action-id order"
            );
            assert!(
                choice
                    .element
                    .affordances
                    .iter()
                    .all(|affordance| affordance.enabled)
            );
        }
        let node = records
            .iter()
            .find(|record| record.element.element_id.as_str() == DIALOGUE_NODE_ELEMENT_ID)
            .expect("node text element");
        assert_eq!(
            node.element
                .text_or_none
                .as_ref()
                .expect("node text")
                .text_id
                .as_str(),
            "nextengine.reference.dialogue.offer"
        );
        assert!(
            records
                .iter()
                .all(|record| record.surface_id.as_str() == DIALOGUE_SURFACE_ID)
        );
    }

    #[test]
    fn dialogue_source_hash_tracks_the_selection() {
        let rpg = snapshot(vec![dialogue(
            persistent(0x5a),
            "nextengine.reference.dialogue.offer",
        )]);
        let accept = dialogue_semantic_ui_records_for_ids(
            epoch(),
            persistent(0x5a),
            &rpg,
            ReferenceDialogueChoiceV1::Accept,
        )
        .expect("accept records");
        let leave = dialogue_semantic_ui_records_for_ids(
            epoch(),
            persistent(0x5a),
            &rpg,
            ReferenceDialogueChoiceV1::Leave,
        )
        .expect("leave records");
        assert_ne!(
            accept[0].source_snapshot_hash, leave[0].source_snapshot_hash,
            "selection changes the published source snapshot hash"
        );
    }
}
