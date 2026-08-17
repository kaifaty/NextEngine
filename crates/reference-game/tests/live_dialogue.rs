//! S4: live dialogue arbitration (ui-nav selection + ui-confirm production accept).

use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_DOWN_CONTROL_PATH_ID, KEYBOARD_E_CONTROL_PATH_ID,
    KEYBOARD_J_CONTROL_PATH_ID, KEYBOARD_RETURN_CONTROL_PATH_ID, KEYBOARD_UP_CONTROL_PATH_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_contracts::presentation::PresentationSnapshotV3;
use next_contracts::rpg::RpgAggregatePayloadV1;
use next_reference_game::ReferenceGameDriverV2;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

const HUD_IDS: [&str; 3] = [
    "nextengine.ui.element.hud.action",
    "nextengine.ui.element.hud.health",
    "nextengine.ui.element.hud.quest",
];
// A5: the dialogue-accept speech cue leaves its subtitle line on the HUD.
const SUBTITLE_IDS: [&str; 1] = ["nextengine.ui.element.hud.subtitle"];
// `semantic_ui_records` yields the canonical element-id order.
const DIALOGUE_SURFACE_IDS: [&str; 4] = [
    "nextengine.ui.element.dialogue.choice-accept",
    "nextengine.ui.element.dialogue.choice-leave",
    "nextengine.ui.element.dialogue.node-text",
    "nextengine.ui.element.dialogue.title",
];
const DIALOGUE_OFFER_NODE_ID: &str = "nextengine.reference-alpha.dialogue.offer";

fn control_event_with_sequence(
    device_class: &str,
    control_path: &str,
    phase: NormalizedControlPhaseV1,
    value: Vec<i16>,
    source_sequence: u64,
) -> PlatformEventV1 {
    let control = NormalizedControlEventV1::new(
        SchemaId::new(device_class).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(control_path).expect("control path"),
        phase,
        value,
        Vec::new(),
        0,
        source_sequence,
    )
    .expect("control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.reference-test").expect("source"),
        source_sequence,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(next_contracts::canonical::sha256(
            b"nextengine.platform.reference-test-capabilities.v1",
        )),
    )
    .expect("platform event")
}

fn test_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-reference-game-{label}-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn key_event(
    sequence: &mut u64,
    control_path: &'static str,
    phase: NormalizedControlPhaseV1,
) -> PlatformEventV1 {
    *sequence += 1;
    control_event_with_sequence(
        KEYBOARD_DEVICE_CLASS_ID,
        control_path,
        phase,
        vec![if phase == NormalizedControlPhaseV1::Started {
            i16::MAX
        } else {
            0
        }],
        *sequence,
    )
}

fn tap_key(driver: &mut ReferenceGameDriverV2, sequence: &mut u64, control_path: &'static str) {
    let press = key_event(sequence, control_path, NormalizedControlPhaseV1::Started);
    driver.advance(&[press]).expect("key press frame");
    let release = key_event(sequence, control_path, NormalizedControlPhaseV1::Completed);
    driver.advance(&[release]).expect("key release frame");
}

fn record_ids(snapshot: &PresentationSnapshotV3) -> Vec<String> {
    snapshot
        .semantic_ui_records()
        .map(|record| record.element.element_id.as_str().to_owned())
        .collect()
}

fn expected_records(dialogue_open: bool) -> Vec<String> {
    expected_records_with_subtitle(dialogue_open, false)
}

fn expected_records_with_subtitle(dialogue_open: bool, subtitle: bool) -> Vec<String> {
    let mut records: Vec<String> = Vec::new();
    if dialogue_open {
        records.extend(DIALOGUE_SURFACE_IDS.iter().map(|id| (*id).to_owned()));
    }
    records.extend(HUD_IDS.iter().map(|id| (*id).to_owned()));
    if subtitle {
        records.extend(SUBTITLE_IDS.iter().map(|id| (*id).to_owned()));
    }
    records
}

fn dialogue_selected_choice(snapshot: &PresentationSnapshotV3) -> Option<String> {
    snapshot.semantic_ui_records().find_map(|record| {
        let element_id = record.element.element_id.as_str();
        if record.element.selected
            && (element_id == "nextengine.ui.element.dialogue.choice-accept"
                || element_id == "nextengine.ui.element.dialogue.choice-leave")
        {
            Some(element_id.to_owned())
        } else {
            None
        }
    })
}

fn dialogue_node_id(driver: &ReferenceGameDriverV2) -> String {
    let state = driver.state().expect("driver state");
    state
        .checkpoint
        .rpg_snapshot
        .aggregates
        .iter()
        .find_map(|aggregate| match &aggregate.payload {
            RpgAggregatePayloadV1::Dialogue(payload) => Some(payload.node_id.as_str().to_owned()),
            _ => None,
        })
        .expect("dialogue aggregate")
}

fn quest_state_and_trust(driver: &ReferenceGameDriverV2) -> (String, i32) {
    let state = driver.state().expect("driver state");
    let mut quest_state = None;
    let mut trust = None;
    for aggregate in &state.checkpoint.rpg_snapshot.aggregates {
        match &aggregate.payload {
            RpgAggregatePayloadV1::Quest(payload) => {
                quest_state = Some(payload.state_id.as_str().to_owned());
            }
            RpgAggregatePayloadV1::Relationship(payload) => {
                for dimension in &payload.dimensions {
                    if dimension.dimension_id.as_str()
                        == "nextengine.reference-alpha.relationship.trust"
                    {
                        trust = Some(dimension.value);
                    }
                }
            }
            _ => {}
        }
    }
    (
        quest_state.expect("quest aggregate"),
        trust.expect("trust dimension"),
    )
}

#[test]
fn live_dialogue_accept_after_boundary_is_rejected_by_production_routine_gate() {
    let root = test_root("live-dialogue-accept");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut driver = ReferenceGameDriverV2::new(activated, true).expect("live driver");
    let mut sequence = 0_u64;

    // Interact near the NPC opens the modal dialogue surface instead of
    // auto-accepting; the offer node is untouched until a real accept.
    let press = key_event(
        &mut sequence,
        KEYBOARD_E_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("dialogue open frame");
    assert_eq!(record_ids(snapshot), expected_records(true));
    assert_eq!(
        dialogue_selected_choice(snapshot),
        Some("nextengine.ui.element.dialogue.choice-accept".to_owned())
    );
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_OFFER_NODE_ID);
    let release = key_event(
        &mut sequence,
        KEYBOARD_E_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("interact release frame");
    // One frame for the modal context revision to become the tagging stack.
    driver.advance(&[]).expect("context swap frame");

    // ui-nav moves the selection deterministically.
    let press = key_event(
        &mut sequence,
        KEYBOARD_DOWN_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("nav down frame");
    assert_eq!(
        dialogue_selected_choice(snapshot),
        Some("nextengine.ui.element.dialogue.choice-leave".to_owned())
    );
    let release = key_event(
        &mut sequence,
        KEYBOARD_DOWN_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("nav down release frame");
    let press = key_event(
        &mut sequence,
        KEYBOARD_UP_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("nav up frame");
    assert_eq!(
        dialogue_selected_choice(snapshot),
        Some("nextengine.ui.element.dialogue.choice-accept".to_owned())
    );
    let release = key_event(
        &mut sequence,
        KEYBOARD_UP_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("nav up release frame");

    // The modal workflow necessarily crosses the authored tick-2 Duty -> Rest
    // boundary. Confirm still submits through the production interaction path,
    // but Runtime accepts the input with no derived RPG command in Rest.
    let press = key_event(
        &mut sequence,
        KEYBOARD_RETURN_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("confirm frame");
    assert_eq!(record_ids(snapshot), expected_records(false));
    let release = key_event(
        &mut sequence,
        KEYBOARD_RETURN_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("confirm release frame");
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_OFFER_NODE_ID);
    driver.advance(&[]).expect("accept injection frame");
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_OFFER_NODE_ID);
    assert_eq!(
        quest_state_and_trust(&driver),
        ("nextengine.reference-alpha.quest.available".to_owned(), 0)
    );

    // The manual journal observation stays tied to the unchanged authoritative
    // quest state rather than presentation-only modal state.
    let press = key_event(
        &mut sequence,
        KEYBOARD_J_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let journal = driver.advance(&[press]).expect("available journal frame");
    let entry = journal
        .semantic_ui_records()
        .find(|record| {
            record.element.element_id.as_str() == "nextengine.ui.element.quest-journal.entry.0"
        })
        .expect("active quest journal entry");
    assert_eq!(
        entry
            .element
            .text_or_none
            .as_ref()
            .expect("journal text")
            .arguments
            .get(1),
        Some(&next_contracts::presentation::UiTextArgumentV1::TextId(
            SchemaId::new("nextengine.reference-alpha.quest.available")
                .expect("available quest state"),
        ))
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_dialogue_leave_then_rest_blocks_reopen_and_survives_recovery() {
    let root = test_root("live-dialogue-leave-back");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut driver = ReferenceGameDriverV2::new(activated.clone(), true).expect("live driver");
    let mut sequence = 0_u64;

    tap_key(&mut driver, &mut sequence, KEYBOARD_E_CONTROL_PATH_ID);
    driver.advance(&[]).expect("context swap frame");
    assert_eq!(
        driver
            .state()
            .expect("Rest state")
            .world_routine_snapshot_or_none
            .expect("routine snapshot")
            .record
            .current_activity,
        next_contracts::world_routine::WorldRoutineActivityV1::Rest
    );

    // ui-confirm on Leave closes the already-open surface without a domain
    // command even though the routine boundary has now committed.
    tap_key(&mut driver, &mut sequence, KEYBOARD_DOWN_CONTROL_PATH_ID);
    let press = key_event(
        &mut sequence,
        KEYBOARD_RETURN_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("leave confirm frame");
    assert_eq!(record_ids(snapshot), expected_records(false));
    let release = key_event(
        &mut sequence,
        KEYBOARD_RETURN_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("leave release frame");
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_OFFER_NODE_ID);
    assert_eq!(
        quest_state_and_trust(&driver),
        ("nextengine.reference-alpha.quest.available".to_owned(), 0)
    );

    // A fresh open request in Rest queries Runtime availability first and is
    // rejected before presentation can create a stale modal surface.
    let press = key_event(
        &mut sequence,
        KEYBOARD_E_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("Rest reopen frame");
    assert_eq!(record_ids(snapshot), expected_records(false));
    let release = key_event(
        &mut sequence,
        KEYBOARD_E_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver
        .advance(&[release])
        .expect("Rest reopen release frame");
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_OFFER_NODE_ID);

    // Recovery preserves the Rest revision and the same query result; it does
    // not reconstruct or permit a presentation-only dialogue surface.
    let saved = driver.state().expect("state before restore");
    let mut restored = ReferenceGameDriverV2::restore(
        activated,
        saved.checkpoint.clone(),
        saved.world_streaming_snapshot.clone(),
        saved.world_routine_snapshot_or_none,
        saved.world_population_snapshot,
        saved.world_activity_snapshot,
        saved.agent_cognition_snapshot,
        saved.agent_memory_snapshot,
        saved.physical_animation_snapshot,
        saved.driver_recovery.clone(),
    )
    .expect("restore");
    let snapshot = restored.presentation_snapshot().expect("restored snapshot");
    assert_eq!(record_ids(snapshot), expected_records(false));
    let press = key_event(
        &mut sequence,
        KEYBOARD_E_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = restored
        .advance(&[press])
        .expect("restored Rest reopen frame");
    assert_eq!(record_ids(snapshot), expected_records(false));
    assert_eq!(
        quest_state_and_trust(&restored),
        ("nextengine.reference-alpha.quest.available".to_owned(), 0)
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}
