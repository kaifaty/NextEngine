use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::StateRoot;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn bounded_bulk_time_converges_exactly_with_ordinary_stepped_advances() {
    let (root, package) = activated_reference_package();
    let mut bulk = next_reference_game::ReferenceGameDriverV2::new(package.clone(), true)
        .expect("bulk driver");
    let mut stepped =
        next_reference_game::ReferenceGameDriverV2::new(package, true).expect("stepped driver");
    let target_tick = 64_u64;
    let mut saw_observable_boundary = false;

    while bulk.next_tick() < target_tick {
        let budget = 5_u64.min(target_tick - bulk.next_tick());
        let report = bulk
            .advance_bulk_time(budget)
            .expect("bounded bulk advance");
        assert_eq!(report.start_tick, stepped.next_tick());
        assert!((1..=budget).contains(&report.advanced_ticks));
        for _ in 0..report.advanced_ticks {
            stepped.advance(&[]).expect("ordinary stepped advance");
        }
        assert_eq!(report.end_tick, stepped.next_tick());
        let bulk_state = bulk.state().expect("bulk state");
        let stepped_state = stepped.state().expect("stepped state");
        assert_live_state_eq(&bulk_state, &stepped_state);
        assert_eq!(
            report.checkpoint_state_root,
            bulk_state.checkpoint.state_root
        );
        assert_eq!(
            report.application_state_root,
            application_state_root(&bulk_state)
        );
        assert_eq!(report.committed_event_count, bulk_state.events);
        assert_eq!(report.rpg_event_count, bulk_state.rpg_events);
        assert_eq!(
            report.world_activity_revision,
            bulk_state.world_activity_snapshot.record_revision
        );
        assert_eq!(
            report.agent_cognition_revision,
            bulk_state.agent_cognition_snapshot.revision
        );
        match report.stop_reason {
            next_reference_game::ReferenceBulkTimeStopReasonV1::ObservableBoundary => {
                saw_observable_boundary = true;
                // R5a makes the physical-animation phase part of the exact
                // application snapshot. The phase advances every committed
                // tick, so the bounded evaluator must expose that tick as an
                // observable boundary instead of batching across it.
                assert_eq!(report.advanced_ticks, 1);
            }
            next_reference_game::ReferenceBulkTimeStopReasonV1::TickBudgetExhausted => {
                panic!("physical-animation phase must be an observable boundary");
            }
        }
    }

    assert!(saw_observable_boundary);

    let before_invalid = bulk.state().expect("state before invalid budgets");
    assert!(matches!(
        bulk.advance_bulk_time(0),
        Err(
            next_reference_game::ReferenceGameError::BulkTimeTickBudgetInvalid {
                requested: 0,
                maximum: next_reference_game::REFERENCE_BULK_TIME_MAX_TICKS_V1,
            }
        )
    ));
    let oversized = next_reference_game::REFERENCE_BULK_TIME_MAX_TICKS_V1 + 1;
    assert!(matches!(
        bulk.advance_bulk_time(oversized),
        Err(next_reference_game::ReferenceGameError::BulkTimeTickBudgetInvalid {
            requested,
            maximum: next_reference_game::REFERENCE_BULK_TIME_MAX_TICKS_V1,
        }) if requested == oversized
    ));
    let after_invalid = bulk.state().expect("state after invalid budgets");
    assert_live_state_eq(&before_invalid, &after_invalid);

    std::fs::remove_dir_all(root).expect("cleanup");
}

fn activated_reference_package() -> (std::path::PathBuf, next_project::ActivatedProjectPackage) {
    let cooked = next_project::cook_project_v6(
        next_reference_game::project_source_v6().expect("reference source"),
    )
    .expect("reference cook");
    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-bulk-time-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let package = next_project::activate_project_package(&store).expect("activate");
    (root, package)
}

fn application_state_root(state: &next_reference_game::ReferenceLiveStateV2) -> StateRoot {
    next_contracts::snapshot::
        world_checkpoint_with_physical_animation_and_systemic_cognition_v1_state_root_from_canonical_components(
            &state.checkpoint_canonical_components,
            &state.world_streaming_snapshot,
            state.world_routine_snapshot_or_none.as_ref(),
            &state.world_population_snapshot,
            &state.world_activity_snapshot,
            &state.agent_cognition_snapshot,
            &state.agent_memory_snapshot,
            &state.physical_animation_snapshot,
        )
        .expect("application state root")
}

fn assert_live_state_eq(
    left: &next_reference_game::ReferenceLiveStateV2,
    right: &next_reference_game::ReferenceLiveStateV2,
) {
    assert_eq!(left.checkpoint, right.checkpoint);
    assert_eq!(
        left.checkpoint_canonical_components,
        right.checkpoint_canonical_components
    );
    assert_eq!(
        left.world_streaming_snapshot,
        right.world_streaming_snapshot
    );
    assert_eq!(
        left.world_routine_snapshot_or_none,
        right.world_routine_snapshot_or_none
    );
    assert_eq!(
        left.world_population_snapshot,
        right.world_population_snapshot
    );
    assert_eq!(left.world_activity_snapshot, right.world_activity_snapshot);
    assert_eq!(
        left.agent_cognition_snapshot,
        right.agent_cognition_snapshot
    );
    assert_eq!(left.agent_memory_snapshot, right.agent_memory_snapshot);
    assert_eq!(
        left.physical_animation_snapshot,
        right.physical_animation_snapshot
    );
    assert_eq!(left.ticks, right.ticks);
    assert_eq!(left.events, right.events);
    assert_eq!(left.rpg_events, right.rpg_events);
    assert_eq!(
        left.project_composition_lock_hash,
        right.project_composition_lock_hash
    );
    assert_eq!(left.content_manifest_hash, right.content_manifest_hash);
    assert_eq!(
        left.presentation_input_count,
        right.presentation_input_count
    );
    assert_eq!(left.presentation_snapshot, right.presentation_snapshot);
    assert_eq!(left.driver_recovery, right.driver_recovery);
}
