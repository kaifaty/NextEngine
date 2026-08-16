use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::cognition::{
    PlanningFailureV1, StrategicAgentIntentKindV1, StrategicAgentIntentV1,
    SystemicExecutionFailureV1,
};
use next_contracts::command::EventPayload;
use next_contracts::rpg::{
    CommitmentStateV1, RpgAggregateEnvelopeV1, RpgAggregateKindV1, RpgAggregatePayloadV1,
    RpgSnapshotV2,
};
use next_runtime::{RuntimeState, WorldServicesTickCommitV1};
use next_world::{
    WorldActivityOwnerV1, WorldPopulationOwnerV1, WorldRoutineOwnerV1, WorldStreamerV1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct SystemicHarness {
    session: next_reference_game::ReferenceGameSession,
    runtime: RuntimeState,
    world: WorldStreamerV1,
    routine: WorldRoutineOwnerV1,
    population: WorldPopulationOwnerV1,
    activity: WorldActivityOwnerV1,
    cognition: next_agent::cognition::StrategicAgentOwnersV1,
}

impl SystemicHarness {
    fn new(package: next_project::ActivatedProjectPackage, rpg_snapshot: RpgSnapshotV2) -> Self {
        let session = next_reference_game::build_reference_game_session(package.project.clone())
            .expect("reference session");
        let runtime = RuntimeState::with_rpg_snapshot(
            session.bootstrap.clone(),
            session.authority.clone(),
            rpg_snapshot,
        )
        .expect("runtime");
        let world = WorldStreamerV1::activate(
            package.project.clone(),
            package.content_generation,
            session.world_topology().initial_chunk_id().clone(),
        )
        .expect("world");
        let routine = WorldRoutineOwnerV1::activate(
            package.project.world_routine_catalog_or_none,
            runtime.next_tick(),
        )
        .expect("routine");
        let population = WorldPopulationOwnerV1::activate(
            package.project.world_population_catalog.clone(),
            package.project.world_navigation_catalog.clone(),
            runtime.next_tick(),
        )
        .expect("population");
        let activity = session.initial_activity_owner().expect("activity");
        let cognition = session.initial_cognition_owners().expect("cognition");
        Self {
            session,
            runtime,
            world,
            routine,
            population,
            activity,
            cognition,
        }
    }

    fn tick(&mut self) -> WorldServicesTickCommitV1 {
        let prepared = self
            .runtime
            .tick_preparation()
            .prepare_with_world_services_cognition_and_activity(
                [],
                &self.routine,
                &self.population,
                &self.activity,
                &self.cognition,
                &self.world,
            )
            .expect("prepare systemic tick");
        let validated = self
            .runtime
            .validate_prepared_world_services_tick_with_cognition_and_activity(
                &self.routine,
                &self.population,
                &self.activity,
                &self.cognition,
                &self.world,
                prepared,
            )
            .expect("validate systemic tick");
        self.runtime
            .commit_validated_world_services_tick_with_cognition_and_activity(
                &mut self.routine,
                &mut self.population,
                &mut self.activity,
                &mut self.cognition,
                &mut self.world,
                validated,
            )
            .expect("commit systemic tick")
    }

    fn replace_social_intent_creation_tick(&mut self, creation_tick: u64) {
        let mut agent = self.cognition.agent_snapshot().clone();
        let prior = agent
            .pending_intent_or_none
            .as_ref()
            .expect("pending social intent");
        let StrategicAgentIntentKindV1::CommitSocialExchange {
            exchange_hash,
            commitment_id,
        } = prior.kind
        else {
            panic!("social intent");
        };
        let replacement = StrategicAgentIntentV1::commit_social_exchange(
            prior.subject_id,
            prior.goal_id.clone(),
            prior.action_id.clone(),
            creation_tick,
            exchange_hash,
            commitment_id,
        )
        .expect("replacement intent");
        agent.pending_intent_or_none = Some(replacement);
        self.cognition = next_agent::cognition::StrategicAgentOwnersV1::restore(
            self.session
                .activated_project
                .agent_cognition_catalog
                .clone(),
            agent,
            self.cognition.memory_snapshot().clone(),
        )
        .expect("restored injected owner");
    }
}

#[test]
fn systemic_failure_branches_are_typed_and_never_partially_mutate_rpg() {
    let cooked = next_project::cook_project_v5(
        next_reference_game::project_source_v5().expect("reference source"),
    )
    .expect("reference cook");
    let root = std::env::temp_dir().join(format!(
        "nextengine-reference-systemic-failures-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let package = next_project::activate_project_package(&store).expect("activate");
    let fixture = next_reference_game::build_reference_game_session(package.project.clone())
        .expect("reference session");
    let baseline = next_reference_game::cooked_project_rpg_snapshot(&fixture);
    let activity_catalog = package.project.world_activity_catalog.clone();

    let mut no_job_snapshot = baseline.clone();
    no_job_snapshot.aggregates.retain(|aggregate| {
        aggregate.aggregate_kind != RpgAggregateKindV1::Commitment
            || aggregate.persistent_id != activity_catalog.commitment_id
    });
    no_job_snapshot.validate().expect("no-job snapshot");
    let mut no_job = SystemicHarness::new(package.clone(), no_job_snapshot.clone());
    no_job.tick();
    let no_job_commit = no_job.tick();
    let no_job_trace = no_job_commit
        .decision_trace_or_none
        .as_ref()
        .expect("typed no-job trace");
    assert_eq!(
        no_job_trace.planning_failure,
        PlanningFailureV1::JobUnavailable
    );
    assert!(no_job_trace.intent_id_or_none.is_none());
    assert_eq!(no_job.runtime.rpg_snapshot(), no_job_snapshot);
    assert_eq!(no_job.activity.snapshot().record_revision, 0);
    assert_eq!(no_job.cognition.agent_snapshot().revision, 0);
    assert_eq!(rpg_event_count(&no_job_commit), 0);

    let mut stale = SystemicHarness::new(package.clone(), baseline.clone());
    stale.tick();
    stale.tick();
    stale.replace_social_intent_creation_tick(0);
    let before_stale = stale.runtime.rpg_snapshot();
    let stale_commit = stale.tick();
    assert_eq!(
        stale_commit.systemic_failure_or_none,
        Some(SystemicExecutionFailureV1::IntentStale)
    );
    assert_eq!(stale.runtime.rpg_snapshot(), before_stale);
    assert_eq!(stale.activity.snapshot().record_revision, 0);
    assert_eq!(rpg_event_count(&stale_commit), 0);

    let no_money_snapshot = with_employer_currency(baseline, &activity_catalog, 0);
    let mut no_money = SystemicHarness::new(package, no_money_snapshot);
    for _ in 0..6 {
        no_money.tick();
    }
    let before_no_money = no_money.runtime.rpg_snapshot();
    let no_money_commit = no_money.tick();
    assert_eq!(
        no_money_commit.systemic_failure_or_none,
        Some(SystemicExecutionFailureV1::InsufficientCurrency)
    );
    assert_eq!(no_money.runtime.rpg_snapshot(), before_no_money);
    assert_eq!(rpg_event_count(&no_money_commit), 0);
    assert!(matches!(
        next_reference_game::aggregate_payload(
            &before_no_money,
            RpgAggregateKindV1::Commitment,
            activity_catalog.commitment_id,
        ),
        Some(RpgAggregatePayloadV1::Commitment(commitment))
            if commitment.state == CommitmentStateV1::Accepted
    ));

    std::fs::remove_dir_all(root).expect("cleanup");
}

fn with_employer_currency(
    mut snapshot: RpgSnapshotV2,
    catalog: &next_contracts::world_activity::WorldActivityCatalogV1,
    current_value: i32,
) -> RpgSnapshotV2 {
    let index = snapshot
        .aggregates
        .iter()
        .position(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::Character
                && aggregate.persistent_id == catalog.systemic_work.employer_character_id
        })
        .expect("employer aggregate");
    let prior = snapshot.aggregates[index].clone();
    let RpgAggregatePayloadV1::Character(mut character) = prior.payload else {
        panic!("employer character");
    };
    character
        .resources
        .iter_mut()
        .find(|resource| resource.resource_id == catalog.systemic_work.currency_resource_id)
        .expect("employer currency")
        .current_value = current_value;
    snapshot.aggregates[index] = RpgAggregateEnvelopeV1::new(
        prior.persistent_id,
        prior.schema_version,
        prior.revision,
        prior.definition_ref,
        prior.provenance,
        RpgAggregatePayloadV1::Character(character),
    )
    .expect("rebuilt employer aggregate");
    snapshot.validate().expect("no-money snapshot");
    snapshot
}

fn rpg_event_count(commit: &WorldServicesTickCommitV1) -> usize {
    commit
        .runtime_report
        .events
        .iter()
        .filter(|event| matches!(event.payload, EventPayload::Rpg(_)))
        .count()
}
