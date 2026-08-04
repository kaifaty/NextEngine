//! S4: live dialogue arbitration (ui-nav selection + ui-confirm production accept).

use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{
    KEYBOARD_D_CONTROL_PATH_ID, KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_DOWN_CONTROL_PATH_ID,
    KEYBOARD_E_CONTROL_PATH_ID, KEYBOARD_ESCAPE_CONTROL_PATH_ID, KEYBOARD_RETURN_CONTROL_PATH_ID,
    KEYBOARD_UP_CONTROL_PATH_ID,
};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_contracts::presentation::PresentationSnapshotV2;
use next_contracts::rpg::RpgAggregatePayloadV1;
use next_reference_game::ReferenceGameDriverV1;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

const HUD_IDS: [&str; 2] = [
    "nextengine.ui.element.hud.health",
    "nextengine.ui.element.hud.quest",
];
// `semantic_ui_records` yields the canonical element-id order.
const DIALOGUE_SURFACE_IDS: [&str; 4] = [
    "nextengine.ui.element.dialogue.choice-accept",
    "nextengine.ui.element.dialogue.choice-leave",
    "nextengine.ui.element.dialogue.node-text",
    "nextengine.ui.element.dialogue.title",
];
const DIALOGUE_OFFER_NODE_ID: &str = "nextengine.reference.dialogue.offer";
const DIALOGUE_ACCEPTED_NODE_ID: &str = "nextengine.reference.dialogue.accepted";

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

fn tap_key(driver: &mut ReferenceGameDriverV1, sequence: &mut u64, control_path: &'static str) {
    let press = key_event(sequence, control_path, NormalizedControlPhaseV1::Started);
    driver.advance(&[press]).expect("key press frame");
    let release = key_event(sequence, control_path, NormalizedControlPhaseV1::Completed);
    driver.advance(&[release]).expect("key release frame");
}

fn record_ids(snapshot: &PresentationSnapshotV2) -> Vec<String> {
    snapshot
        .semantic_ui_records()
        .map(|record| record.element.element_id.as_str().to_owned())
        .collect()
}

fn expected_records(dialogue_open: bool) -> Vec<String> {
    if dialogue_open {
        DIALOGUE_SURFACE_IDS
            .iter()
            .chain(HUD_IDS.iter())
            .map(|id| (*id).to_owned())
            .collect()
    } else {
        HUD_IDS.iter().map(|id| (*id).to_owned()).collect()
    }
}

fn dialogue_selected_choice(snapshot: &PresentationSnapshotV2) -> Option<String> {
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

fn dialogue_node_id(driver: &ReferenceGameDriverV1) -> String {
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

fn quest_state_and_trust(driver: &ReferenceGameDriverV1) -> (String, i32) {
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
                    if dimension.dimension_id.as_str() == "nextengine.reference.relationship.trust"
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

/// Walks from the spawn point to the NPC so the dialogue affordance is the
/// nearest one (at spawn the wall switch is nearer, mirroring the runtime).
fn move_to_npc(driver: &mut ReferenceGameDriverV1, sequence: &mut u64) {
    let press = key_event(
        sequence,
        KEYBOARD_D_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    driver.advance(&[press]).expect("move start frame");
    for _ in 0..11 {
        driver.advance(&[]).expect("movement frame");
    }
    let release = key_event(
        sequence,
        KEYBOARD_D_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("move stop frame");
}

#[test]
fn live_dialogue_arbitration_accepts_through_production_interaction_path() {
    let root = test_root("live-dialogue-accept");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let mut driver = ReferenceGameDriverV1::new(activated, true).expect("live driver");
    let mut sequence = 0_u64;

    move_to_npc(&mut driver, &mut sequence);

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

    // ui-confirm on Accept closes the surface; the committed frame injects a
    // synthetic interact through the production interaction accept path once
    // the tagging stack no longer carries the dialogue layer.
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
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_ACCEPTED_NODE_ID);
    assert_eq!(
        quest_state_and_trust(&driver),
        ("nextengine.reference.quest.active".to_owned(), 7)
    );

    // A completed dialogue does not reopen; the surface stays closed.
    let press = key_event(
        &mut sequence,
        KEYBOARD_E_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver
        .advance(&[press])
        .expect("post-accept interact frame");
    assert_eq!(record_ids(snapshot), expected_records(false));
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_ACCEPTED_NODE_ID);

    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn live_dialogue_leave_and_back_close_without_command_and_recover() {
    let root = test_root("live-dialogue-leave-back");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let mut driver = ReferenceGameDriverV1::new(activated.clone(), true).expect("live driver");
    let mut sequence = 0_u64;

    move_to_npc(&mut driver, &mut sequence);
    tap_key(&mut driver, &mut sequence, KEYBOARD_E_CONTROL_PATH_ID);
    driver.advance(&[]).expect("context swap frame");

    // ui-confirm on Leave closes the surface without any domain command.
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

    // Reopen; ui-back closes the dialogue and is consumed: no pause suspend
    // publication carries the pause-menu surface.
    tap_key(&mut driver, &mut sequence, KEYBOARD_E_CONTROL_PATH_ID);
    driver.advance(&[]).expect("context swap frame");
    let press = key_event(
        &mut sequence,
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("back close frame");
    let records = record_ids(snapshot);
    assert_eq!(records, expected_records(false));
    assert!(!records.iter().any(|id| id.contains("pause-menu")));
    let release = key_event(
        &mut sequence,
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("back release frame");
    assert_eq!(dialogue_node_id(&driver), DIALOGUE_OFFER_NODE_ID);

    // Reopen once more and restore mid-dialogue: the recovered driver
    // republishes the dialogue surface and accepts through the same path.
    tap_key(&mut driver, &mut sequence, KEYBOARD_E_CONTROL_PATH_ID);
    driver.advance(&[]).expect("context swap frame");
    let saved = driver.state().expect("state before restore");
    let mut restored = ReferenceGameDriverV1::restore(
        activated,
        saved.checkpoint.clone(),
        saved.world_streaming_snapshot.clone(),
        saved.driver_recovery.clone(),
    )
    .expect("restore");
    let snapshot = restored.presentation_snapshot().expect("restored snapshot");
    assert_eq!(record_ids(snapshot), expected_records(true));

    tap_key(
        &mut restored,
        &mut sequence,
        KEYBOARD_RETURN_CONTROL_PATH_ID,
    );
    restored.advance(&[]).expect("accept injection frame");
    assert_eq!(dialogue_node_id(&restored), DIALOGUE_ACCEPTED_NODE_ID);
    assert_eq!(
        quest_state_and_trust(&restored),
        ("nextengine.reference.quest.active".to_owned(), 7)
    );

    std::fs::remove_dir_all(root).expect("cleanup");
}
