use super::*;

#[test]
fn live_ui_screen_toggles_are_deterministic_and_back_closes_before_pause() {
    let root = test_root("live-ui-screen-toggles");
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");

    let mut driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("live driver");
    let en_resolver = reference_game_support::text_resolver(&activated, "en");

    let mut sequence = 0_u64;
    let mut key_event = |control_path: &'static str, phase: NormalizedControlPhaseV1| {
        sequence += 1;
        control_event_with_sequence(
            KEYBOARD_DEVICE_CLASS_ID,
            control_path,
            phase,
            vec![if phase == NormalizedControlPhaseV1::Started {
                i16::MAX
            } else {
                0
            }],
            sequence,
        )
    };
    fn record_ids(snapshot: &next_contracts::presentation::PresentationSnapshotV2) -> Vec<String> {
        snapshot
            .semantic_ui_records()
            .map(|record| record.element.element_id.as_str().to_owned())
            .collect()
    }
    const HUD_IDS: [&str; 3] = [
        "nextengine.ui.element.hud.action",
        "nextengine.ui.element.hud.health",
        "nextengine.ui.element.hud.quest",
    ];
    // `semantic_ui_records` yields the canonical element-id order.
    const INVENTORY_SCREEN_IDS: [&str; 4] = [
        "nextengine.ui.element.equipment.empty",
        "nextengine.ui.element.equipment.title",
        "nextengine.ui.element.inventory.empty",
        "nextengine.ui.element.inventory.title",
    ];
    const JOURNAL_SCREEN_IDS: [&str; 2] = [
        "nextengine.ui.element.quest-journal.entry.0",
        "nextengine.ui.element.quest-journal.title",
    ];
    const PAUSE_MENU_IDS: [&str; 4] = [
        "nextengine.ui.element.pause-menu.load",
        "nextengine.ui.element.pause-menu.resume",
        "nextengine.ui.element.pause-menu.save",
        "nextengine.ui.element.pause-menu.title",
    ];
    let expected = |screen: &[&str]| -> Vec<String> {
        HUD_IDS
            .iter()
            .chain(screen.iter())
            .map(|id| (*id).to_owned())
            .collect()
    };

    // Closed by default: HUD only.
    let initial = driver.presentation_snapshot().expect("initial snapshot");
    assert_eq!(record_ids(initial), expected(&[]));

    // `ui-inventory` opens the inventory/equipment screen read-only.
    let press = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("inventory open frame");
    let inventory_records = record_ids(snapshot);
    assert_eq!(inventory_records, expected(&INVENTORY_SCREEN_IDS));
    assert!(
        driver
            .presentation_snapshot()
            .expect("snapshot")
            .semantic_ui_records()
            .all(|record| record.element.affordances.is_empty()),
        "open screen stays affordance-free"
    );

    // A key release does not toggle; the next press closes the screen.
    let release = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    let snapshot = driver.advance(&[release]).expect("release frame");
    assert_eq!(record_ids(snapshot), expected(&INVENTORY_SCREEN_IDS));
    let press = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("inventory close frame");
    assert_eq!(record_ids(snapshot), expected(&[]));
    let release = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("release frame");

    // `ui-journal` opens the quest journal; its entry resolves through the
    // cooked catalogs with the quest display name and state.
    let press = key_event(
        KEYBOARD_J_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("journal open frame");
    assert_eq!(record_ids(snapshot), expected(&JOURNAL_SCREEN_IDS));
    let journal_entry = snapshot
        .semantic_ui_records()
        .find(|record| {
            record.element.element_id.as_str() == "nextengine.ui.element.quest-journal.entry.0"
        })
        .expect("journal entry element");
    let journal_resolution = en_resolver.resolve(
        journal_entry
            .element
            .text_or_none
            .as_ref()
            .expect("journal entry text"),
    );
    assert_eq!(journal_resolution.text, "Frontier Relay - Available");
    assert_eq!(journal_resolution.diagnostic_or_none, None);

    // Screens are exclusive: opening inventory replaces the journal.
    let press = key_event(
        KEYBOARD_I_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("screen switch frame");
    assert_eq!(record_ids(snapshot), expected(&INVENTORY_SCREEN_IDS));

    // `ui-back` with an open screen closes it and is consumed: no pause
    // suspend publication carries the pause-menu surface.
    let press = key_event(
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("screen close frame");
    assert_eq!(record_ids(snapshot), expected(&[]));

    // `ui-back` with no open screen requests the declared pause suspend.
    let release = key_event(
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Completed,
    );
    driver.advance(&[release]).expect("release frame");
    let press = key_event(
        KEYBOARD_ESCAPE_CONTROL_PATH_ID,
        NormalizedControlPhaseV1::Started,
    );
    let snapshot = driver.advance(&[press]).expect("pause frame");
    assert_eq!(record_ids(snapshot), expected(&PAUSE_MENU_IDS));

    std::fs::remove_dir_all(root).expect("cleanup");
}
