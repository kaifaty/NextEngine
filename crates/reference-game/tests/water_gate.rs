//! Plan `continuum-water/36` G3: from the vessels' start the lever prompt
//! shows, one tap of `E` closes the gate through the accepted flow command,
//! a second tap reopens it.

use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{ContentHash, PersistentId, SchemaId};
use next_contracts::input::{KEYBOARD_DEVICE_CLASS_ID, KEYBOARD_E_CONTROL_PATH_ID};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1,
};
use next_reference_game::ReferenceGameDriverV2;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn key_event(sequence: &mut u64, phase: NormalizedControlPhaseV1) -> PlatformEventV1 {
    *sequence += 1;
    let control = NormalizedControlEventV1::new(
        SchemaId::new(KEYBOARD_DEVICE_CLASS_ID).expect("device class"),
        PersistentId::from_bytes([0x74; 16]),
        SchemaId::new(KEYBOARD_E_CONTROL_PATH_ID).expect("control path"),
        phase,
        vec![if phase == NormalizedControlPhaseV1::Started {
            i16::MAX
        } else {
            0
        }],
        Vec::new(),
        0,
        *sequence,
    )
    .expect("control");
    PlatformEventV1::new(
        PersistentId::from_bytes([0x75; 16]),
        SchemaId::new("nextengine.platform.source.reference-test").expect("source"),
        *sequence,
        0,
        PlatformEventKindV1::Control,
        PlatformEventPayloadV1::Control(control),
        ContentHash::from_bytes(next_contracts::canonical::sha256(
            b"nextengine.platform.reference-test-capabilities.v1",
        )),
    )
    .expect("platform event")
}

fn tap_e(driver: &mut ReferenceGameDriverV2, sequence: &mut u64) {
    let press = key_event(sequence, NormalizedControlPhaseV1::Started);
    driver.advance(&[press]).expect("press");
    let release = key_event(sequence, NormalizedControlPhaseV1::Completed);
    driver.advance(&[release]).expect("release");
}

fn gate_opening(driver: &ReferenceGameDriverV2) -> u32 {
    driver
        .state()
        .expect("state")
        .checkpoint
        .physics_checkpoint
        .water_flow
        .edge_states
        .get(&next_reference_game::REFERENCE_WATER_FLOW_GATE_ID)
        .expect("gate state")
        .opening_permille
}

fn hud_gate_text(driver: &ReferenceGameDriverV2) -> Option<String> {
    driver
        .state()
        .expect("state")
        .presentation_snapshot
        .semantic_ui_records()
        .find(|record| record.element.element_id.as_str() == "nextengine.ui.element.hud.gate")
        .and_then(|record| {
            record
                .element
                .text_or_none
                .as_ref()
                .map(|text| text.text_id.as_str().to_owned())
        })
}

#[test]
fn the_lever_closes_and_reopens_the_gate() {
    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-water-gate-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let mut driver = ReferenceGameDriverV2::new_with_presentation_epoch_and_spawn(
        activated,
        true,
        ContentHash::from_bytes([0x36; 32]),
        Some(next_reference_game::ReferenceSpawnOverrideV1::at_vessels()),
    )
    .expect("driver");
    let mut sequence = 0;
    driver.advance(&[]).expect("settle");
    assert_eq!(gate_opening(&driver), 1_000);
    assert_eq!(
        hud_gate_text(&driver).as_deref(),
        Some("nextengine.ui.text.hud.gate.close"),
        "the close prompt shows in reach"
    );
    let flow_emitter = |driver: &ReferenceGameDriverV2| {
        driver.audio_scene().emitters.iter().any(|emitter| {
            emitter.emitter_key.subject_id == next_reference_game::REFERENCE_WATER_FLOW_GATE_ID
        })
    };
    assert!(flow_emitter(&driver));
    tap_e(&mut driver, &mut sequence);
    driver.advance(&[]).expect("commit");
    assert_eq!(gate_opening(&driver), 0, "the first tap closes the gate");
    let gate_record = |driver: &ReferenceGameDriverV2| {
        driver
            .water_presentation_frame(0)
            .edges
            .iter()
            .any(|edge| edge.edge_id == next_reference_game::REFERENCE_WATER_FLOW_GATE_ID)
    };
    assert!(!gate_record(&driver), "no gate record with the gate closed");
    assert!(!flow_emitter(&driver), "the flow emitter is gone");
    assert_eq!(
        hud_gate_text(&driver).as_deref(),
        Some("nextengine.ui.text.hud.gate.open")
    );
    tap_e(&mut driver, &mut sequence);
    driver.advance(&[]).expect("commit");
    assert_eq!(gate_opening(&driver), 1_000, "the second tap reopens it");
    assert!(gate_record(&driver), "the gate record is back");
    assert!(flow_emitter(&driver));
    // Out of reach: the reference spawn, 16 m from the lever, shows no prompt
    // and a tap changes nothing.
    let mut far = ReferenceGameDriverV2::new_with_presentation_epoch_and_spawn(
        next_project::activate_project_package(&store).expect("activate"),
        true,
        ContentHash::from_bytes([0x37; 32]),
        None,
    )
    .expect("driver");
    far.advance(&[]).expect("settle");
    assert_eq!(hud_gate_text(&far), None);
    let mut far_sequence = 0;
    tap_e(&mut far, &mut far_sequence);
    far.advance(&[]).expect("commit");
    assert_eq!(gate_opening(&far), 1_000);
    std::fs::remove_dir_all(root).expect("remove store");
}
