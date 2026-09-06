use next_contracts::canonical::CanonicalDecodeLimits;
use next_contracts::cognition::{
    AGENT_MEMORY_SNAPSHOT_OWNER_ID, AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
    AGENT_MEMORY_SNAPSHOT_SEGMENT_ID, AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
    AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID, AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID, AgentCognitionSnapshotV1,
    AgentMemorySnapshotV1, COGNITION_SCHEMA_VERSION,
};
use next_contracts::command::{IssuerPrincipal, NOOP_COMMAND_CAPABILITY_ID, WorldCommand};
use next_contracts::ids::{
    CapabilityId, ContentHash, InputSourceId, PlayerPrincipalId, SchemaId, StateRoot,
};
use next_contracts::input::{
    INPUT_MAPPING_RECEIPT_SCHEMA_VERSION, InputMappingCodeV1, InputMappingReceiptV2,
};
use next_contracts::persistence::{
    AuthorityGrant, ManifestCodecError, ManifestValidationError, REPLAY_MANIFEST_V9_SCHEMA_VERSION,
    ReplayCommandRecord, ReplayComparePointV9, ReplayManifestV9, ReplayOwnerSegmentV2,
    ReplayTickManifestV9, SaveCompatibility, SaveSegmentDescriptor, TickSettings,
    WorldStreamingReplayInputV1, replay_physics_query_batch_hash,
    replay_physics_query_results_hash, replay_targeting_query_trace_hash,
};
use next_contracts::physics::{
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
};
use next_contracts::rpg::{
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
};
use next_contracts::snapshot::{
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, WorldCheckpointV4,
};
use next_contracts::world::{
    WORLD_STREAMING_SNAPSHOT_OWNER_ID, WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
    WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION, WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
    WorldStreamingSnapshotV1,
};
use next_contracts::world_activity::{
    WORLD_ACTIVITY_SCHEMA_VERSION, WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
    WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID, WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID, WorldActivitySnapshotV1,
};
use next_contracts::world_population::{
    WORLD_POPULATION_SCHEMA_VERSION, WORLD_POPULATION_SNAPSHOT_OWNER_ID,
    WORLD_POPULATION_SNAPSHOT_SCHEMA_ID, WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
    WorldPopulationSnapshotV1,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_SCHEMA_VERSION, WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
    WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID, WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID, WorldRoutineSnapshotV1,
    interaction_availability_batch_hash,
};
use next_runtime::{AuthorityRegistry, RuntimeReplayDriver, RuntimeReplayError, RuntimeState};

use super::{
    ReplayError, ReplayInput, ReplayOutput, ReplayTickInput, ReplayTickRecord,
    compare_replay_outputs, replay_command_results, run_replay, run_replay_manifest_v9,
    verify_replay,
};
use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;
use crate::{StateSegment, build_neutral_runtime_fixture, compute_state_root};

fn command(
    stream: next_contracts::ids::CommandStreamId,
    issuer: IssuerPrincipal,
    sequence: u64,
    tick: u64,
) -> WorldCommand {
    WorldCommand::noop(stream, issuer, sequence, tick).expect("test command is canonical")
}

fn scenario() -> ReplayInput {
    let first = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16]));
    let second = IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([1; 16]));
    let capability =
        CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID).expect("built-in capability ID is valid");
    let fixture = build_neutral_runtime_fixture(
        "nextengine.replay-test",
        [
            (first.clone(), vec![capability.clone()]),
            (second.clone(), vec![capability]),
        ],
    )
    .expect("fixture");
    let first_stream = fixture.stream_for(&first).expect("first stream");
    let second_stream = fixture.stream_for(&second).expect("second stream");
    ReplayInput {
        bootstrap: fixture.bootstrap,
        authority: fixture.authority,
        ticks: vec![
            ReplayTickInput {
                commands: vec![
                    command(first_stream, first.clone(), 0, 0),
                    command(second_stream, second, 0, 0),
                ],
            },
            ReplayTickInput {
                commands: vec![command(first_stream, first, 1, 1)],
            },
        ],
    }
}

fn compatibility() -> SaveCompatibility {
    SaveCompatibility {
        engine_build_hash: ContentHash::from_bytes([1; 32]),
        game_build_hash: ContentHash::from_bytes([2; 32]),
        project_id: SchemaId::new("nextengine.replay-test").expect("valid project"),
        schema_registry_hash: ContentHash::from_bytes([3; 32]),
        content_manifest_hash: ContentHash::from_bytes([4; 32]),
        mechanics_lock_hash: ContentHash::from_bytes([5; 32]),
        tick_settings: TickSettings {
            gameplay_hz: 30,
            physics_hz: 60,
            motor_hz: 60,
        },
        loaded_chunk_revisions: vec![],
        rng_stream_states: vec![],
        physical_bindings: vec![],
        policy_state_schemas: vec![],
        plugin_script_bindings: vec![],
    }
}

fn replay_manifest_v9() -> (
    ReplayManifestV9,
    ReplayOutput,
    crate::player_fixture::PreparedFixtureProjectPackage,
) {
    let scratch = ScratchContext::new(&std::env::temp_dir()).expect("test scratch root");
    let prepared =
        prepare_fixture_project_package_with_scratch(&scratch, "nextengine.replay-v9-test")
            .expect("fixture project package");
    let session =
        next_reference_game::build_reference_game_session(prepared.package.project.clone())
            .expect("reference session");
    let initial_chunk_id = session.world_topology().initial_chunk_id().clone();
    let mut authority = AuthorityRegistry::new();
    for (principal, capabilities) in session.authority.entries() {
        let mut capabilities = capabilities.iter().cloned().collect::<Vec<_>>();
        if principal == &session.principal {
            capabilities.push(
                CapabilityId::new(NOOP_COMMAND_CAPABILITY_ID)
                    .expect("built-in no-op capability ID is valid"),
            );
        }
        authority
            .register(principal.clone(), capabilities)
            .expect("reference authority remains unique");
    }
    let commands = vec![
        vec![command(
            session.movement_stream_id,
            session.principal.clone(),
            0,
            0,
        )],
        vec![command(
            session.movement_stream_id,
            session.principal.clone(),
            1,
            1,
        )],
    ];
    let mut runtime = RuntimeState::with_rpg_snapshot(
        session.bootstrap.clone(),
        authority.clone(),
        next_reference_game::cooked_project_rpg_snapshot(&session),
    )
    .expect("initial runtime");
    let mut world = next_world::WorldStreamerV1::activate(
        prepared.package.project.clone(),
        prepared.package.content_generation.clone(),
        initial_chunk_id,
    )
    .expect("initial replay world");
    let mut routine = next_world::WorldRoutineOwnerV1::activate(
        prepared.package.project.world_routine_catalog_or_none,
        runtime.next_tick(),
    )
    .expect("initial routine owner");
    let mut population = next_world::WorldPopulationOwnerV1::activate(
        prepared.package.project.world_population_catalog.clone(),
        prepared.package.project.world_navigation_catalog.clone(),
        runtime.next_tick(),
    )
    .expect("initial population owner");
    let mut activity = next_world::WorldActivityOwnerV1::activate(
        prepared.package.project.world_activity_catalog.clone(),
        runtime.next_tick(),
    )
    .expect("initial activity owner");
    let mut cognition = session
        .initial_cognition_owners()
        .expect("initial cognition owners");
    let initial_checkpoint = runtime.world_checkpoint().expect("initial checkpoint");
    let initial_world_snapshot = world.snapshot().clone();
    let initial_routine_snapshot = routine
        .snapshot_or_none()
        .copied()
        .expect("reference project has a routine owner segment");
    let initial_population_snapshot = population
        .snapshot_or_none()
        .cloned()
        .expect("reference project has a population owner segment");
    let initial_activity_snapshot = activity.snapshot().clone();
    let initial_agent_snapshot = cognition.agent_snapshot().clone();
    let initial_memory_snapshot = cognition.memory_snapshot().clone();
    let manifest_authority = authority
        .entries()
        .map(|(principal, capabilities)| AuthorityGrant {
            principal: principal.clone(),
            capabilities: capabilities.iter().cloned().collect(),
        })
        .collect();
    let mut ticks = Vec::new();
    let mut compare_points = Vec::new();
    let mut expected_ticks = Vec::new();
    for direct_commands in &commands {
        let prepared_tick = runtime
            .tick_preparation()
            .prepare_with_world_services_cognition_and_activity(
                direct_commands.clone(),
                &routine,
                &population,
                &activity,
                &cognition,
                &world,
            )
            .expect("record joint tick");
        let validated = runtime
            .validate_prepared_world_services_tick_with_cognition_and_activity(
                &routine,
                &population,
                &activity,
                &cognition,
                &world,
                prepared_tick,
            )
            .expect("validate joint tick");
        let commit = runtime
            .commit_validated_world_services_tick_with_cognition_and_activity(
                &mut routine,
                &mut population,
                &mut activity,
                &mut cognition,
                &mut world,
                validated,
            )
            .expect("commit joint tick");
        let report = &commit.runtime_report;
        ticks.push(ReplayTickManifestV9 {
            tick: report.tick,
            world_streaming_input: WorldStreamingReplayInputV1::None,
            closed_ingress_batch: report.closed_ingress_batch.clone(),
            direct_external_commands: direct_commands
                .iter()
                .map(|command| {
                    ReplayCommandRecord::from_command(command).expect("test command is canonical")
                })
                .collect(),
            expected_ingress_command_batch: report.command_batches[0].clone(),
            expected_physics_step_input: report.physics_step_input.clone(),
            expected_contact_batch: report.contact_batch.clone(),
            expected_targeting_intents: report.targeting_intents.clone(),
            expected_authoritative_targeting_queries: report
                .authoritative_targeting_queries
                .clone(),
            expected_physics_query_batch: report.physics_query_batch.clone(),
            expected_physics_query_results: report.physics_query_results.clone(),
            expected_outcome_command_batch: report.command_batches[1].clone(),
            expected_mapping_receipts: report.mapping_receipts.clone(),
            expected_interaction_availability: report.interaction_availability.clone(),
            expected_command_results: replay_command_results(&report.results),
            expected_events: report.events.clone(),
        });
        compare_points.push(ReplayComparePointV9 {
            tick: report.tick,
            state_root: commit.application_state_root,
            command_ledger_hash: report.snapshot.command_ledger_hash().expect("ledger hash"),
            owner_segments: commit.application_owner_segments.clone(),
            closed_ingress_batch_hash: report.closed_ingress_batch.batch_hash,
            ingress_command_batch_hash: report.command_batches[0].batch_hash,
            physics_step_input_hash: report
                .physics_step_input
                .input_hash()
                .expect("physics input hash"),
            contact_batch_hash: report.contact_batch.batch_hash,
            physics_query_batch_hash: replay_physics_query_batch_hash(&report.physics_query_batch)
                .expect("query batch hash"),
            physics_query_results_hash: replay_physics_query_results_hash(
                &report.physics_query_results,
            )
            .expect("query results hash"),
            targeting_query_trace_hash: replay_targeting_query_trace_hash(
                &report.targeting_intents,
                &report.authoritative_targeting_queries,
            )
            .expect("targeting trace hash"),
            outcome_command_batch_hash: report.command_batches[1].batch_hash,
            interaction_availability_hash: interaction_availability_batch_hash(
                &report.interaction_availability,
            )
            .expect("interaction availability hash"),
        });
        expected_ticks.push(ReplayTickRecord {
            tick: report.tick,
            command_results: report.results.clone(),
            events: report.events.clone(),
            state_root: commit.application_state_root,
            command_ledger_hash: report.snapshot.command_ledger_hash().expect("ledger hash"),
        });
    }
    let final_checkpoint = runtime.world_checkpoint().expect("final checkpoint");
    let expected = ReplayOutput {
        ticks: expected_ticks,
        final_snapshot: final_checkpoint.runtime_snapshot.clone(),
        final_checkpoint,
    };
    (
        ReplayManifestV9 {
            schema_version: REPLAY_MANIFEST_V9_SCHEMA_VERSION,
            compatibility: compatibility(),
            initial_owner_segments: replay_owner_segments(
                &initial_checkpoint,
                &initial_world_snapshot,
                &initial_routine_snapshot,
                &initial_population_snapshot,
                &initial_activity_snapshot,
                &initial_agent_snapshot,
                &initial_memory_snapshot,
            ),
            initial_state_root:
                next_contracts::snapshot::world_checkpoint_with_systemic_cognition_v1_state_root(
                    &initial_checkpoint.runtime_snapshot,
                    &initial_checkpoint.rpg_snapshot,
                    &initial_checkpoint.physics_checkpoint,
                    &initial_world_snapshot,
                    Some(&initial_routine_snapshot),
                    &initial_population_snapshot,
                    &initial_activity_snapshot,
                    &initial_agent_snapshot,
                    &initial_memory_snapshot,
                )
                .expect("initial root computes"),
            authority: manifest_authority,
            ticks,
            compare_points,
        },
        expected,
        prepared,
    )
}

fn replay_owner_segments(
    checkpoint: &WorldCheckpointV4,
    world: &WorldStreamingSnapshotV1,
    routine: &WorldRoutineSnapshotV1,
    population: &WorldPopulationSnapshotV1,
    activity: &WorldActivitySnapshotV1,
    agent: &AgentCognitionSnapshotV1,
    memory: &AgentMemorySnapshotV1,
) -> Vec<ReplayOwnerSegmentV2> {
    let mut segments = vec![
        (
            RUNTIME_SNAPSHOT_OWNER_ID,
            RUNTIME_SNAPSHOT_SCHEMA_ID,
            RUNTIME_SNAPSHOT_SEGMENT_ID,
            RUNTIME_SNAPSHOT_SCHEMA_VERSION,
            checkpoint
                .runtime_snapshot
                .canonical_bytes()
                .expect("runtime"),
        ),
        (
            RPG_AGGREGATE_SNAPSHOT_OWNER_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
            RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
            RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION,
            checkpoint.rpg_snapshot.canonical_bytes().expect("RPG"),
        ),
        (
            PHYSICS_SNAPSHOT_OWNER_ID,
            PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
            PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
            u32::from(PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION),
            checkpoint
                .physics_checkpoint
                .canonical_bytes()
                .expect("physics"),
        ),
        (
            WORLD_STREAMING_SNAPSHOT_OWNER_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_ID,
            WORLD_STREAMING_SNAPSHOT_SEGMENT_ID,
            WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION,
            world.canonical_bytes().expect("world streaming"),
        ),
        (
            WORLD_ROUTINE_SNAPSHOT_OWNER_ID,
            WORLD_ROUTINE_SNAPSHOT_SCHEMA_ID,
            WORLD_ROUTINE_SNAPSHOT_SEGMENT_ID,
            u32::from(WORLD_ROUTINE_SCHEMA_VERSION),
            routine.canonical_bytes().expect("world routine"),
        ),
        (
            WORLD_POPULATION_SNAPSHOT_OWNER_ID,
            WORLD_POPULATION_SNAPSHOT_SCHEMA_ID,
            WORLD_POPULATION_SNAPSHOT_SEGMENT_ID,
            u32::from(WORLD_POPULATION_SCHEMA_VERSION),
            population.canonical_bytes().expect("world population"),
        ),
        (
            WORLD_ACTIVITY_SNAPSHOT_OWNER_ID,
            WORLD_ACTIVITY_SNAPSHOT_SCHEMA_ID,
            WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID,
            u32::from(WORLD_ACTIVITY_SCHEMA_VERSION),
            activity.canonical_bytes().expect("world activity"),
        ),
        (
            AGENT_RUNTIME_SNAPSHOT_OWNER_ID,
            AGENT_RUNTIME_SNAPSHOT_SCHEMA_ID,
            AGENT_RUNTIME_SNAPSHOT_SEGMENT_ID,
            u32::from(COGNITION_SCHEMA_VERSION),
            agent.canonical_bytes().expect("agent cognition"),
        ),
        (
            AGENT_MEMORY_SNAPSHOT_OWNER_ID,
            AGENT_MEMORY_SNAPSHOT_SCHEMA_ID,
            AGENT_MEMORY_SNAPSHOT_SEGMENT_ID,
            u32::from(COGNITION_SCHEMA_VERSION),
            memory.canonical_bytes().expect("agent memory"),
        ),
    ]
    .into_iter()
    .map(|(owner, schema, segment, version, canonical_bytes)| {
        let descriptor = SaveSegmentDescriptor::for_bytes(
            SchemaId::new(owner).expect("owner"),
            SchemaId::new(schema).expect("schema"),
            SchemaId::new(segment).expect("segment"),
            version,
            &canonical_bytes,
        )
        .expect("descriptor");
        ReplayOwnerSegmentV2 {
            descriptor,
            canonical_bytes,
        }
    })
    .collect::<Vec<_>>();
    segments.sort_by(|left, right| {
        (
            &left.descriptor.owner_id,
            &left.descriptor.schema_id,
            &left.descriptor.segment_id,
        )
            .cmp(&(
                &right.descriptor.owner_id,
                &right.descriptor.schema_id,
                &right.descriptor.segment_id,
            ))
    });
    segments
}

#[test]
fn empty_state_root_matches_golden_vector() {
    assert_eq!(
        compute_state_root([])
            .expect("empty state tree is valid")
            .to_hex(),
        "a03902d5430adb91603bcd477a7b814b5c08ecde092f04f8c4ff359032ffca73"
    );
}

#[test]
fn state_segment_order_does_not_change_root() {
    let first = StateSegment::new(
        SchemaId::new("a").expect("valid owner"),
        SchemaId::new("schema").expect("valid schema"),
        SchemaId::new("one").expect("valid segment"),
        vec![1],
    );
    let second = StateSegment::new(
        SchemaId::new("b").expect("valid owner"),
        SchemaId::new("schema").expect("valid schema"),
        SchemaId::new("two").expect("valid segment"),
        vec![2],
    );
    assert_eq!(
        compute_state_root([first.clone(), second.clone()]).expect("valid state"),
        compute_state_root([second, first]).expect("valid state")
    );
}

#[test]
fn duplicate_state_segment_is_rejected() {
    let segment = StateSegment::new(
        SchemaId::new("runtime").expect("valid owner"),
        SchemaId::new("nextengine.runtime.snapshot").expect("valid schema"),
        SchemaId::new("command-ledger").expect("valid segment"),
        vec![1],
    );
    assert_eq!(
        compute_state_root([segment.clone(), segment]),
        Err(super::StateRootError::DuplicateSegment)
    );
}

#[test]
fn replay_of_same_input_is_exact() {
    let input = scenario();
    let expected = run_replay(&input).expect("reference replay runs");
    verify_replay(&expected, &input).expect("same input must replay exactly");
}

#[test]
fn arrival_permutation_preserves_replay_output() {
    let first = scenario();
    let mut second = first.clone();
    second.ticks[0].commands.reverse();
    let first_output = run_replay(&first).expect("first replay runs");
    let second_output = run_replay(&second).expect("second replay runs");
    compare_replay_outputs(&first_output, &second_output)
        .expect("arrival order must not affect replay");
}

#[test]
fn replay_reports_first_divergent_tick() {
    let first = scenario();
    let mut second = first.clone();
    let prior = second.ticks[1].commands[0].clone();
    second.ticks[1].commands = vec![command(prior.stream_id, prior.issuer.clone(), 2, 1)];
    let first_output = run_replay(&first).expect("first replay runs");
    let second_output = run_replay(&second).expect("second replay runs");
    let error = compare_replay_outputs(&first_output, &second_output)
        .expect_err("different causal command must diverge");

    assert!(matches!(
        error,
        ReplayError::NondeterministicResult {
            first_divergent_tick: 1,
            ..
        }
    ));
    assert_eq!(error.stable_code(), "NONDETERMINISTIC_RESULT");
}

#[test]
fn versioned_manifest_restores_snapshot_and_checks_every_compare_point() {
    let (manifest, expected, prepared) = replay_manifest_v9();
    let actual = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect("manifest replay is exact");
    assert_eq!(actual, expected);
}

#[test]
fn replay_v9_rejects_a_calendar_valid_routine_without_its_ledger_receipt() {
    let scratch = ScratchContext::new(&std::env::temp_dir()).expect("test scratch root");
    let prepared =
        prepare_fixture_project_package_with_scratch(&scratch, "nextengine.replay-v9-ledger-gap")
            .expect("fixture project package");
    let session =
        next_reference_game::build_reference_game_session(prepared.package.project.clone())
            .expect("reference session");
    let cognition = session
        .initial_cognition_owners()
        .expect("initial cognition owners");
    let mut runtime = RuntimeState::new(session.bootstrap.clone(), session.authority.clone())
        .expect("initial runtime");
    let mut world = next_world::WorldStreamerV1::activate(
        prepared.package.project.clone(),
        prepared.package.content_generation.clone(),
        session.world_topology().initial_chunk_id().clone(),
    )
    .expect("world streamer");
    let mut delayed_catalog = prepared
        .package
        .project
        .world_routine_catalog_or_none
        .expect("routine catalog");
    delayed_catalog.routine.transition_world_tick = delayed_catalog
        .profile
        .anchor_world_tick
        .checked_add(10_000)
        .expect("delayed boundary");
    let mut delayed_routine = next_world::WorldRoutineOwnerV1::activate(Some(delayed_catalog), 0)
        .expect("delayed routine owner");
    let mut population = next_world::WorldPopulationOwnerV1::activate(
        prepared.package.project.world_population_catalog.clone(),
        prepared.package.project.world_navigation_catalog.clone(),
        0,
    )
    .expect("population owner");
    for _ in 0..3 {
        let candidate = runtime
            .tick_preparation()
            .prepare_with_world_services([], &delayed_routine, &population, &world)
            .expect("prepare world-services test tick");
        let validated = runtime
            .validate_prepared_world_services_tick_without_application_evidence(
                &delayed_routine,
                &population,
                &world,
                candidate,
            )
            .expect("validate world-services test tick");
        runtime
            .commit_validated_world_services_tick_without_application_evidence(
                &mut delayed_routine,
                &mut population,
                &mut world,
                validated,
            )
            .expect("commit world-services test tick");
    }
    let checkpoint = runtime.world_checkpoint().expect("checkpoint");
    let catalog = prepared
        .package
        .project
        .world_routine_catalog_or_none
        .as_ref()
        .expect("routine catalog");
    let mut routine = WorldRoutineSnapshotV1::initial(catalog).expect("initial routine");
    routine.record.record_revision = 1;
    routine.record.current_activity = next_contracts::world_routine::WorldRoutineActivityV1::Rest;
    routine
        .validate_against(catalog, checkpoint.runtime_snapshot.next_tick)
        .expect("routine is calendar-valid in isolation");
    let world_snapshot = world.snapshot().clone();
    let population_snapshot = population
        .snapshot_or_none()
        .cloned()
        .expect("population snapshot");
    let activity_snapshot =
        WorldActivitySnapshotV1::initial(&prepared.package.project.world_activity_catalog)
            .expect("activity snapshot");
    let manifest = ReplayManifestV9 {
        schema_version: REPLAY_MANIFEST_V9_SCHEMA_VERSION,
        compatibility: compatibility(),
        initial_owner_segments: replay_owner_segments(
            &checkpoint,
            &world_snapshot,
            &routine,
            &population_snapshot,
            &activity_snapshot,
            cognition.agent_snapshot(),
            cognition.memory_snapshot(),
        ),
        initial_state_root:
            next_contracts::snapshot::world_checkpoint_with_systemic_cognition_v1_state_root(
                &checkpoint.runtime_snapshot,
                &checkpoint.rpg_snapshot,
                &checkpoint.physics_checkpoint,
                &world_snapshot,
                Some(&routine),
                &population_snapshot,
                &activity_snapshot,
                cognition.agent_snapshot(),
                cognition.memory_snapshot(),
            )
            .expect("initial root"),
        authority: session
            .authority
            .entries()
            .map(|(principal, capabilities)| AuthorityGrant {
                principal: principal.clone(),
                capabilities: capabilities.iter().cloned().collect(),
            })
            .collect(),
        ticks: Vec::new(),
        compare_points: Vec::new(),
    };

    let error = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect_err("routine/ledger mismatch must fail before replay publication");
    assert!(matches!(
        error,
        ReplayError::SnapshotRestore(
            next_runtime::SnapshotRestoreError::WorldRoutineLedgerClosureInvalid
        )
    ));
    assert!(
        error
            .to_string()
            .contains("WORLD_ROUTINE_LEDGER_CLOSURE_INVALID")
    );
}

#[test]
fn replay_manifest_v9_round_trips_and_replays_query_and_v2_receipt_facts() {
    let (manifest, expected, prepared) = replay_manifest_v9();
    let bytes = manifest.to_jcs_bytes().expect("manifest encodes");
    let decoded = ReplayManifestV9::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("manifest decodes");
    assert_eq!(decoded, manifest);
    assert_eq!(decoded.to_jcs_bytes().expect("manifest re-encodes"), bytes);
    assert_eq!(
        run_replay_manifest_v9(&decoded, prepared.package.clone()).expect("V9 replay is exact"),
        expected
    );
}

#[test]
fn replay_manifest_v9_rejects_v2_receipt_divergence() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    manifest.ticks[0]
        .expected_mapping_receipts
        .push(InputMappingReceiptV2 {
            schema_version: INPUT_MAPPING_RECEIPT_SCHEMA_VERSION,
            assigned_tick: 0,
            source_id: InputSourceId::from_bytes([0x91; 16]),
            source_sequence: 0,
            payload_hash: ContentHash::from_bytes([0x92; 32]),
            frame_code: InputMappingCodeV1::Accepted,
            action_results: Vec::new(),
            derived_commands: Vec::new(),
        });
    let error =
        run_replay_manifest_v9(&manifest, prepared.package.clone()).expect_err("receipt mismatch");
    assert!(matches!(
        error,
        ReplayError::RecordedStageMismatch {
            tick: 0,
            stage: "closed-ingress-command-outcome",
        }
    ));
}

#[test]
fn replay_manifest_v9_rejects_query_batch_divergence_after_valid_rehash() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    manifest.ticks[0]
        .expected_physics_query_batch
        .snapshot_selector
        .physics_snapshot_hash = ContentHash::from_bytes([0x93; 32]);
    manifest.compare_points[0].physics_query_batch_hash =
        replay_physics_query_batch_hash(&manifest.ticks[0].expected_physics_query_batch)
            .expect("updated query batch hash");
    let error =
        run_replay_manifest_v9(&manifest, prepared.package.clone()).expect_err("query mismatch");
    assert!(matches!(
        error,
        ReplayError::RecordedStageMismatch {
            tick: 0,
            stage: "closed-authoritative-query-outcome",
        }
    ));
}

#[test]
fn replay_manifest_v9_rejects_targeting_trace_root_mismatch() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    manifest.compare_points[0].targeting_query_trace_hash = ContentHash::from_bytes([0x94; 32]);
    let error = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect_err("target trace mismatch");
    assert!(matches!(
        error,
        ReplayError::Manifest(ManifestValidationError::ReplayQueryFactsInvalid)
    ));
}

#[test]
fn replay_manifest_v4_jcs_is_rejected_before_nested_decoding() {
    let bytes = br#"{"schema_version":4}"#;
    assert!(matches!(
        ReplayManifestV9::from_jcs_bytes(bytes, CanonicalDecodeLimits::default()),
        Err(ManifestCodecError::Validation(
            ManifestValidationError::UnsupportedReplayVersion(4)
        ))
    ));
}

#[test]
fn retired_replay_manifest_v5_jcs_is_rejected_before_nested_decoding() {
    let bytes = br#"{"schema_version":5}"#;
    assert!(matches!(
        ReplayManifestV9::from_jcs_bytes(bytes, CanonicalDecodeLimits::default()),
        Err(ManifestCodecError::Validation(
            ManifestValidationError::UnsupportedReplayVersion(5)
        ))
    ));
}

#[test]
fn retired_replay_manifest_v8_jcs_is_rejected_before_nested_decoding() {
    let bytes = br#"{"schema_version":8}"#;
    assert!(matches!(
        ReplayManifestV9::from_jcs_bytes(bytes, CanonicalDecodeLimits::default()),
        Err(ManifestCodecError::Validation(
            ManifestValidationError::UnsupportedReplayVersion(8)
        ))
    ));
}

#[test]
fn replay_manifest_v9_rejects_missing_activity_owner_segment() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    let original_len = manifest.initial_owner_segments.len();
    manifest.initial_owner_segments.retain(|segment| {
        segment.descriptor.segment_id.as_str() != WORLD_ACTIVITY_SNAPSHOT_SEGMENT_ID
    });
    assert_eq!(manifest.initial_owner_segments.len() + 1, original_len);

    let error = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect_err("missing activity owner must fail before replay");
    assert!(matches!(
        error,
        ReplayError::Manifest(ManifestValidationError::ReplayInitialSegmentsInvalid)
    ));
}

#[test]
fn manifest_reports_first_ledger_or_state_divergence() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    manifest.compare_points[0].state_root = StateRoot::from_bytes([9; 32]);
    let error = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect_err("compare point must fail");
    assert!(matches!(
        error,
        ReplayError::ComparePointMismatch(ref details)
            if details.first_divergent_tick == 0
    ));
    assert_eq!(error.stable_code(), "NONDETERMINISTIC_RESULT");
}

#[test]
fn manifest_decodes_entire_command_stream_before_runtime_restore() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    manifest.ticks[1].direct_external_commands[0]
        .canonical_command_bytes
        .push(0);
    let error = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect_err("corrupt command must fail closed");
    assert!(matches!(error, ReplayError::Manifest(_)));
    assert_eq!(error.stable_code(), "REPLAY_MANIFEST_INVALID");
}

#[test]
fn replay_driver_rejects_command_batch_before_state_mutation() {
    let (manifest, _, prepared) = replay_manifest_v9();
    let (initial, mut ticks) = manifest
        .validate_and_decode(CanonicalDecodeLimits::default())
        .expect("manifest decodes");
    let initial_checkpoint = initial.checkpoint.clone();
    let mut authority = AuthorityRegistry::new();
    for grant in &manifest.authority {
        authority
            .register(grant.principal.clone(), grant.capabilities.clone())
            .expect("authority grant");
    }
    let mut driver = RuntimeReplayDriver::new_with_definitions(
        initial.checkpoint,
        authority,
        prepared.package.project.rpg_definitions.clone(),
    )
    .expect("replay driver restores");
    let tick = ticks.remove(0);
    let mut wrong_ingress_batch = tick.expected_ingress_command_batch;
    wrong_ingress_batch.batch_hash = ContentHash::from_bytes([0xee; 32]);
    let error = driver
        .replay_tick(
            tick.closed_ingress_batch,
            tick.direct_external_commands,
            &wrong_ingress_batch,
            &tick.expected_physics_step_input,
            &tick.expected_contact_batch,
            &tick.expected_outcome_command_batch,
        )
        .expect_err("divergent batch must fail before execution");
    assert!(matches!(
        error,
        RuntimeReplayError::CommandBatchMismatch {
            tick: 0,
            phase: next_contracts::command::CommandPhase::Ingress,
        }
    ));
    assert_eq!(
        driver.world_checkpoint().expect("driver checkpoint"),
        initial_checkpoint
    );
}

#[test]
fn v4_replay_is_rejected_before_nested_snapshot_decoding() {
    let (mut manifest, _, prepared) = replay_manifest_v9();
    manifest.schema_version = 4;
    manifest.initial_owner_segments.clear();
    let error = run_replay_manifest_v9(&manifest, prepared.package.clone())
        .expect_err("V4 replay fails closed");
    assert!(matches!(
        error,
        ReplayError::Manifest(ManifestValidationError::UnsupportedReplayVersion(4))
    ));
    assert_eq!(error.stable_code(), "UNSUPPORTED_REPLAY_MANIFEST_VERSION");
}

#[test]
fn scenario_final_root_pins_current_checkpoint_identity_ledger_archive_and_physics_closure() {
    assert_eq!(
        run_replay(&scenario())
            .expect("scenario runs")
            .final_state_root()
            .expect("scenario has ticks")
            .to_hex(),
        "2d1e007374f9619c5ae7e449f1af30c02ae9c0d19ca21655c36f338049e299b3"
    );
}
