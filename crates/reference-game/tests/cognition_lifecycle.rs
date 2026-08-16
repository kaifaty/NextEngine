use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::SchemaId;
use next_contracts::rpg::{
    RpgAggregateKindV1, RpgAggregateRefV1, RpgCommandV1, RpgOperationPayloadV1, RpgOperationV1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn production_cognition_interrupts_for_emergency_and_resumes_the_ordinary_goal() {
    let cooked = next_project::cook_project_v5(
        next_reference_game::project_source_v5().expect("reference source"),
    )
    .expect("reference cook");
    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-cognition-emergency-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let package = next_project::activate_project_package(&store).expect("activate");
    let session = next_reference_game::build_reference_game_session(package.project.clone())
        .expect("reference session");
    let mut runtime = next_runtime::RuntimeState::with_rpg_snapshot(
        session.bootstrap.clone(),
        session.authority.clone(),
        next_reference_game::cooked_project_rpg_snapshot(&session),
    )
    .expect("runtime");
    let mut world = next_world::WorldStreamerV1::activate(
        package.project.clone(),
        package.content_generation,
        session.world_topology().initial_chunk_id().clone(),
    )
    .expect("world");
    let mut routine = next_world::WorldRoutineOwnerV1::activate(
        package.project.world_routine_catalog_or_none,
        runtime.next_tick(),
    )
    .expect("routine");
    let mut population = next_world::WorldPopulationOwnerV1::activate(
        package.project.world_population_catalog.clone(),
        package.project.world_navigation_catalog.clone(),
        runtime.next_tick(),
    )
    .expect("population");
    let mut cognition = session
        .initial_cognition_owners()
        .expect("cognition owners");
    let health_id = SchemaId::new(next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID)
        .expect("health resource id");
    let resource_command =
        |sequence: u64, tick: u64, revision: u64, expected_value: i32, delta: i32| {
            let mut targets = vec![
                RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Character,
                    persistent_id: session.body_id,
                    expected_revision: 0,
                },
                RpgAggregateRefV1 {
                    aggregate_kind: RpgAggregateKindV1::Character,
                    persistent_id: session.cognition_subject_id,
                    expected_revision: revision,
                },
            ];
            targets.sort_unstable();
            next_contracts::command::WorldCommand::rpg(
                session.rpg_stream_id,
                session.principal.clone(),
                sequence,
                tick,
                RpgCommandV1 {
                    operations: vec![RpgOperationV1 {
                        operation_slot: 0,
                        targets,
                        definition_policy_hashes: Vec::new(),
                        payload: RpgOperationPayloadV1::AdjustCharacterResource {
                            source_character_id: session.body_id,
                            character_id: session.cognition_subject_id,
                            resource_id: health_id.clone(),
                            expected_value,
                            delta,
                        },
                    }],
                },
            )
            .expect("resource command")
        };
    let mut traces = Vec::new();
    for tick in 0_u64..=7 {
        let commands = match tick {
            2 => vec![resource_command(0, tick, 0, 100, -80)],
            5 => vec![resource_command(1, tick, 1, 20, 80)],
            _ => Vec::new(),
        };
        let prepared = runtime
            .tick_preparation()
            .prepare_with_world_services_and_cognition(
                commands,
                &routine,
                &population,
                &cognition,
                &world,
            )
            .expect("prepare cognition tick");
        let validated = runtime
            .validate_prepared_world_services_tick_with_cognition(
                &routine,
                &population,
                &cognition,
                &world,
                prepared,
            )
            .expect("validate cognition tick");
        let committed = runtime
            .commit_validated_world_services_tick_with_cognition(
                &mut routine,
                &mut population,
                &mut cognition,
                &mut world,
                validated,
            )
            .expect("commit cognition tick");
        if let Some(trace) = committed.decision_trace_or_none {
            traces.push(trace);
        }
    }

    assert_eq!(
        traces
            .iter()
            .map(|trace| trace.gameplay_tick)
            .collect::<Vec<_>>(),
        vec![1, 4, 7]
    );
    assert_eq!(
        traces[0].selected_goal_id,
        session
            .activated_project
            .agent_cognition_catalog
            .ordinary_goal_id
    );
    assert_eq!(
        traces[1].switch_reason,
        next_contracts::cognition::DecisionSwitchReasonV1::EmergencyInterrupt
    );
    assert_eq!(
        traces[1].selected_goal_id,
        session
            .activated_project
            .agent_cognition_catalog
            .emergency_goal_id
    );
    assert_eq!(
        traces[2].switch_reason,
        next_contracts::cognition::DecisionSwitchReasonV1::EmergencyExitResume
    );
    assert_eq!(
        traces[2].selected_goal_id,
        session
            .activated_project
            .agent_cognition_catalog
            .ordinary_goal_id
    );
    assert_eq!(cognition.agent_snapshot().revision, 3);
    assert!(cognition.agent_snapshot().suspended_goals.is_empty());
    std::fs::remove_dir_all(root).expect("cleanup");
}
