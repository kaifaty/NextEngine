use super::fixtures::{movement_sample, physical_fixture};
use super::*;

#[test]
fn staged_input_configuration_is_published_and_committed_with_the_tick() {
    let mut fixture = physical_fixture();
    let original = fixture
        .runtime
        .player_controller_registry
        .bindings
        .get(&fixture.source_id)
        .expect("controller binding")
        .clone();
    let next_map = ActionMapManifestV1::new(
        original.action_map.action_map_id.clone(),
        original.action_map.revision + 1,
        original.action_map.supported_device_classes.clone(),
        original.action_map.actions.clone(),
    )
    .expect("next action map");
    let next_context = InputContextStackV1::new(
        original.context_stack.stack_id.clone(),
        original.context_stack.revision + 1,
        original.context_stack.entries.clone(),
    )
    .expect("next context stack");

    let prepared = fixture
        .runtime
        .tick_preparation()
        .prepare([])
        .expect("prepare tick");
    let prepared = fixture
        .runtime
        .stage_player_input_configuration_activation(
            prepared,
            fixture.source_id,
            next_map.clone(),
            next_context.clone(),
        )
        .expect("stage input configuration");
    let staged_binding = prepared
        .report()
        .snapshot
        .player_controller_registry
        .bindings
        .get(&fixture.source_id)
        .expect("staged controller binding");
    assert_eq!(staged_binding.action_map_revision, next_map.revision);
    assert_eq!(staged_binding.context_stack_revision, next_context.revision);

    let validated = fixture
        .runtime
        .validate_prepared_tick(prepared)
        .expect("validate staged generation");
    let report = fixture.runtime.commit_validated_tick(validated);
    assert_eq!(
        report
            .snapshot
            .player_controller_registry
            .bindings
            .get(&fixture.source_id)
            .expect("reported controller binding"),
        fixture
            .runtime
            .player_controller_registry
            .bindings
            .get(&fixture.source_id)
            .expect("committed controller binding")
    );
}

#[test]
fn staged_input_configuration_failure_leaves_the_live_generation_unchanged() {
    let mut fixture = physical_fixture();
    let original_registry = fixture.runtime.player_controller_registry.clone();
    let queued = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample_for_next_tick(&fixture.principal, queued)
        .expect("queue old-revision sample beyond this close barrier");
    let prepared = fixture
        .runtime
        .tick_preparation()
        .prepare([])
        .expect("prepare tick with a retained next-boundary sample");

    let error = fixture
        .runtime
        .stage_player_input_configuration_activation(
            prepared,
            fixture.source_id,
            original_registry.bindings[&fixture.source_id]
                .action_map
                .clone(),
            original_registry.bindings[&fixture.source_id]
                .context_stack
                .clone(),
        )
        .err()
        .expect("queued old-revision input must reject activation before commit");

    assert_eq!(
        error,
        RuntimeFatalError::Input(InputContractError::InvalidProfile)
    );
    assert_eq!(fixture.runtime.next_tick(), 0);
    assert_eq!(
        fixture.runtime.player_controller_registry,
        original_registry
    );
    assert_eq!(
        fixture
            .runtime
            .snapshot()
            .ingress_checkpoint
            .next_samples
            .len(),
        1
    );
    assert!(fixture.runtime.last_closed_ingress_batch().is_none());
}

#[test]
fn prepared_tick_generation_includes_the_controller_registry() {
    let mut fixture = physical_fixture();
    let original = fixture
        .runtime
        .player_controller_registry
        .bindings
        .get(&fixture.source_id)
        .expect("controller binding")
        .clone();
    let prepared = fixture
        .runtime
        .tick_preparation()
        .prepare([])
        .expect("prepare tick");
    let next_context = InputContextStackV1::new(
        original.context_stack.stack_id.clone(),
        original.context_stack.revision + 1,
        original.context_stack.entries.clone(),
    )
    .expect("next context stack");
    fixture
        .runtime
        .activate_player_input_configuration(fixture.source_id, original.action_map, next_context)
        .expect("mutate only the live controller registry");

    let error = fixture
        .runtime
        .validate_prepared_tick(prepared)
        .err()
        .expect("controller-registry drift must stale the preparation");
    assert_eq!(error, RuntimeFatalError::PreparedGenerationStale);
    assert_eq!(fixture.runtime.next_tick(), 0);
}
