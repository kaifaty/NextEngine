use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn reference_source_recooks_byte_identically_and_runs_through_production_paths() {
    let first = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("first cook");
    let second = next_project::cook_project_v1(
        next_reference_game::project_source_v2().expect("reference source"),
    )
    .expect("second cook");
    assert_eq!(first, second);
    assert_eq!(
        first.publication().expect("first publication"),
        second.publication().expect("second publication")
    );

    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-game-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    store
        .publish(&first.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project(&store).expect("activate");
    let outcome = next_reference_game::run_reference_game(activated, true).expect("reference run");
    let checkpoint = outcome.runtime.world_checkpoint().expect("checkpoint");
    assert_eq!(outcome.ticks, 16);
    assert_eq!(outcome.events, 17);
    assert_eq!(outcome.rpg_events, 9);
    assert_eq!(outcome.world_streaming_snapshot.generation, 2);
    assert_ne!(
        checkpoint.runtime_snapshot.command_ledger_hash(),
        Ok(next_contracts::ids::CommandLedgerHash::default())
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}
