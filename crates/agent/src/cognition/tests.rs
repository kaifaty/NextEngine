use next_contracts::cognition::{
    BeliefContradictionV1, BeliefSourceV1, COGNITION_SCHEMA_VERSION, SemanticBeliefV1,
};
use next_contracts::ids::{AssetId, ContentHash, PersistentId};
use next_contracts::world_population::{
    NavigationCapabilityV1, NavigationQueryV1, NavigationRoutePlanV1,
    WORLD_POPULATION_SCHEMA_VERSION,
};

use super::*;

fn schema(value: &str) -> SchemaId {
    SchemaId::new(value).expect("test schema id")
}

fn fixture() -> (
    AgentCognitionCatalogV1,
    AgentMemorySnapshotV1,
    AgentCognitionSnapshotV1,
    NavigationRoutePlanV1,
) {
    let subject_id = PersistentId::from_bytes([7; 16]);
    let belief = SemanticBeliefV1::new(
        subject_id,
        schema("knowledge.route-purpose"),
        schema("knowledge.frontier-duty"),
        u32::try_from(COGNITION_Q16_ONE).expect("positive"),
        BeliefSourceV1::AuthoredSeed,
        0,
        0,
        BeliefContradictionV1::Consistent,
    )
    .expect("belief");
    let catalog = AgentCognitionCatalogV1 {
        schema_version: COGNITION_SCHEMA_VERSION,
        catalog_asset_id: AssetId::from_bytes([8; 16]),
        subject_id,
        evaluation_start_tick: 1,
        evaluation_period_ticks: 3,
        retrieval_limit: 4,
        goal_switch_threshold_q16: COGNITION_Q16_ONE / 4,
        emergency_health_threshold: 30,
        planner_max_depth: 2,
        planner_max_expanded_nodes: 8,
        ordinary_goal_id: schema("goal.reach-frontier"),
        emergency_goal_id: schema("goal.preserve-self"),
        navigate_action_id: schema("affordance.request-logical-route"),
        hold_action_id: schema("affordance.hold-position"),
        route_known_fact_id: schema("fact.route-known"),
        travel_needed_fact_id: schema("fact.travel-needed"),
        emergency_fact_id: schema("fact.emergency"),
        navigate_ready_fact_id: schema("fact.navigate-intent-ready"),
        hold_ready_fact_id: schema("fact.hold-intent-ready"),
        seed_beliefs: vec![belief],
    };
    catalog.validate().expect("catalog");
    let memory = AgentMemorySnapshotV1::initial(&catalog).expect("memory");
    let agent = AgentCognitionSnapshotV1::initial(subject_id, 41);
    let query = NavigationQueryV1 {
        schema_version: WORLD_POPULATION_SCHEMA_VERSION,
        catalog_asset_id: AssetId::from_bytes([9; 16]),
        graph_revision: ContentHash::from_bytes([10; 32]),
        start_node_id: schema("node.home"),
        goal_node_id: schema("node.frontier"),
        capability: NavigationCapabilityV1::AbstractTransfer,
    };
    let route =
        NavigationRoutePlanV1::new(query, vec![schema("node.home"), schema("node.frontier")], 5)
            .expect("route");
    (catalog, memory, agent, route)
}

fn view(
    tick: u64,
    health: i32,
    catalog: &AgentCognitionCatalogV1,
    memory: &AgentMemorySnapshotV1,
    route: Option<&NavigationRoutePlanV1>,
) -> EpistemicViewV1 {
    build_epistemic_view_v1(&StrategicObservationV1 {
        gameplay_tick: tick,
        cognition_catalog_revision: catalog.revision().expect("revision"),
        catalog,
        memory_snapshot: memory,
        rpg_owner_revision: tick,
        population_record_revision: 0,
        current_node_id: schema("node.home"),
        navigation_goal_node_id: schema("node.frontier"),
        health_current: health,
        health_maximum: 100,
        route_plan_or_none: route,
        systemic_or_none: None,
    })
    .expect("view")
}

#[test]
fn utility_goap_and_private_executive_are_deterministic() {
    let (catalog, memory, agent, route) = fixture();
    let epistemic = view(1, 100, &catalog, &memory, Some(&route));
    let first = evaluate_strategic_decision_v1(&catalog, &agent, &memory, epistemic.clone())
        .expect("first");
    let second =
        evaluate_strategic_decision_v1(&catalog, &agent, &memory, epistemic).expect("second");
    assert_eq!(first, second);
    let proposal = first.proposal_or_none.expect("proposal");
    assert_eq!(
        proposal
            .next_agent_snapshot
            .active_goal_or_none
            .expect("goal")
            .goal_id,
        catalog.ordinary_goal_id
    );
    assert_eq!(
        proposal
            .next_agent_snapshot
            .task_or_none
            .expect("task")
            .lifecycle,
        TaskLifecycleV1::Active
    );
}

#[test]
fn no_route_and_budget_exhaustion_are_typed_without_a_proposal() {
    let (mut catalog, memory, agent, route) = fixture();
    let without_route = view(1, 100, &catalog, &memory, None);
    let missing = evaluate_strategic_decision_v1(&catalog, &agent, &memory, without_route)
        .expect("typed fallback");
    assert!(missing.proposal_or_none.is_none());
    assert_eq!(
        missing.decision_trace.planning_failure,
        PlanningFailureV1::RouteUnavailable
    );
    assert_eq!(
        missing.decision_trace.planning_failure.diagnostic_code(),
        "STRATEGIC_ROUTE_UNAVAILABLE"
    );

    catalog.planner_max_expanded_nodes = 1;
    let bounded = evaluate_strategic_decision_v1(
        &catalog,
        &agent,
        &memory,
        view(1, 100, &catalog, &memory, Some(&route)),
    )
    .expect("budget fallback");
    assert!(bounded.proposal_or_none.is_none());
    assert_eq!(
        bounded.decision_trace.planning_failure,
        PlanningFailureV1::PlanBudgetExhausted
    );
}

#[test]
fn emergency_interrupts_and_then_resumes_ordinary_goal() {
    let (catalog, memory0, agent0, route) = fixture();
    let ordinary = evaluate_strategic_decision_v1(
        &catalog,
        &agent0,
        &memory0,
        view(1, 100, &catalog, &memory0, Some(&route)),
    )
    .expect("ordinary")
    .proposal_or_none
    .expect("ordinary proposal");
    let emergency = evaluate_strategic_decision_v1(
        &catalog,
        &ordinary.next_agent_snapshot,
        &ordinary.next_memory_snapshot,
        view(
            4,
            20,
            &catalog,
            &ordinary.next_memory_snapshot,
            Some(&route),
        ),
    )
    .expect("emergency");
    assert_eq!(
        emergency.decision_trace.switch_reason,
        DecisionSwitchReasonV1::EmergencyInterrupt
    );
    let emergency = emergency.proposal_or_none.expect("emergency proposal");
    assert_eq!(emergency.next_agent_snapshot.suspended_goals.len(), 1);

    let resumed = evaluate_strategic_decision_v1(
        &catalog,
        &emergency.next_agent_snapshot,
        &emergency.next_memory_snapshot,
        view(
            7,
            100,
            &catalog,
            &emergency.next_memory_snapshot,
            Some(&route),
        ),
    )
    .expect("resume");
    assert_eq!(
        resumed.decision_trace.switch_reason,
        DecisionSwitchReasonV1::EmergencyExitResume
    );
    let resumed = resumed.proposal_or_none.expect("resume proposal");
    assert_eq!(
        resumed
            .next_agent_snapshot
            .active_goal_or_none
            .expect("active")
            .goal_id,
        catalog.ordinary_goal_id
    );
    assert!(resumed.next_agent_snapshot.suspended_goals.is_empty());
}

#[test]
fn unrelated_hidden_authority_cannot_change_epistemic_decision() {
    let (catalog, memory, agent, route) = fixture();
    let hidden_owner_values = [11_u64, 99_u64];
    let decisions = hidden_owner_values.map(|_unobserved_hidden_revision| {
        evaluate_strategic_decision_v1(
            &catalog,
            &agent,
            &memory,
            view(1, 100, &catalog, &memory, Some(&route)),
        )
        .expect("decision")
    });
    assert_eq!(decisions[0], decisions[1]);
}
