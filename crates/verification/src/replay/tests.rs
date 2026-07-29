use next_contracts::{
    AuthorityGrant, CanonicalDecodeLimits, CapabilityId, ContentHash, IssuerPrincipal,
    ManifestCodecError, ManifestValidationError, NOOP_COMMAND_CAPABILITY_ID,
    PHYSICS_SNAPSHOT_OWNER_ID, PHYSICS_WORLD_CHECKPOINT_SCHEMA_ID,
    PHYSICS_WORLD_CHECKPOINT_SCHEMA_VERSION, PHYSICS_WORLD_CHECKPOINT_SEGMENT_ID,
    PhysicsWorldCheckpointV1, PlayerPrincipalId, REPLAY_MANIFEST_V4_SCHEMA_VERSION,
    RPG_AGGREGATE_SNAPSHOT_OWNER_ID, RPG_AGGREGATE_SNAPSHOT_SCHEMA_ID,
    RPG_AGGREGATE_SNAPSHOT_SCHEMA_VERSION, RPG_AGGREGATE_SNAPSHOT_SEGMENT_ID,
    RUNTIME_SNAPSHOT_OWNER_ID, RUNTIME_SNAPSHOT_SCHEMA_ID, RUNTIME_SNAPSHOT_SCHEMA_VERSION,
    RUNTIME_SNAPSHOT_SEGMENT_ID, ReplayCommandRecord, ReplayComparePointV4, ReplayManifestV4,
    ReplayOwnerSegmentV2, ReplayTickManifestV4, SaveCompatibility, SaveSegmentDescriptor, SchemaId,
    StateRoot, TickSettings, WorldCheckpointV4, WorldCommand,
};
use next_runtime::{AuthorityRegistry, RuntimeReplayDriver, RuntimeReplayError, RuntimeState};

use super::{
    ReplayError, ReplayInput, ReplayTickInput, checkpoint_segment_hashes, compare_replay_outputs,
    compute_world_checkpoint_root, replay_command_results, run_replay, run_replay_manifest,
    verify_replay,
};
use crate::{StateSegment, build_neutral_runtime_fixture, compute_state_root};

fn command(
    stream: next_contracts::CommandStreamId,
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

fn replay_manifest() -> (ReplayManifestV4, super::ReplayOutput) {
    let input = scenario();
    let expected = run_replay(&input).expect("reference replay runs");
    let mut runtime = RuntimeState::new(input.bootstrap.clone(), input.authority.clone())
        .expect("initial runtime");
    let initial_checkpoint = runtime.world_checkpoint().expect("initial checkpoint");
    let authority = input
        .authority
        .entries()
        .map(|(principal, capabilities)| AuthorityGrant {
            principal: principal.clone(),
            capabilities: capabilities.iter().cloned().collect(),
        })
        .collect();
    let mut ticks = Vec::new();
    let mut compare_points = Vec::new();
    let physics_catalog = initial_checkpoint.physics_checkpoint.catalog.clone();
    for input_tick in &input.ticks {
        let report = runtime
            .run_tick(input_tick.commands.clone())
            .expect("recorded tick");
        let checkpoint = WorldCheckpointV4::new(
            report.snapshot.clone(),
            report.rpg_snapshot.clone(),
            PhysicsWorldCheckpointV1::new(physics_catalog.clone(), report.physics_snapshot.clone())
                .expect("physics checkpoint"),
        )
        .expect("record checkpoint");
        let (runtime_segment_hash, rpg_segment_hash, physics_segment_hash) =
            checkpoint_segment_hashes(&checkpoint).expect("segment hashes");
        ticks.push(ReplayTickManifestV4 {
            tick: report.tick,
            closed_ingress_batch: report.closed_ingress_batch.clone(),
            direct_external_commands: input_tick
                .commands
                .iter()
                .map(|command| {
                    ReplayCommandRecord::from_command(command).expect("test command is canonical")
                })
                .collect(),
            expected_ingress_command_batch: report.command_batches[0].clone(),
            expected_physics_step_input: report.physics_step_input.clone(),
            expected_contact_batch: report.contact_batch.clone(),
            expected_outcome_command_batch: report.command_batches[1].clone(),
            expected_mapping_receipts: report.mapping_receipts.clone(),
            expected_command_results: replay_command_results(&report.results),
            expected_events: report.events.clone(),
        });
        compare_points.push(ReplayComparePointV4 {
            tick: report.tick,
            state_root: compute_world_checkpoint_root(&checkpoint).expect("state root"),
            command_ledger_hash: report.snapshot.command_ledger_hash().expect("ledger hash"),
            runtime_segment_hash,
            rpg_segment_hash,
            physics_segment_hash,
            closed_ingress_batch_hash: report.closed_ingress_batch.batch_hash,
            ingress_command_batch_hash: report.command_batches[0].batch_hash,
            physics_step_input_hash: report
                .physics_step_input
                .input_hash()
                .expect("physics input hash"),
            contact_batch_hash: report.contact_batch.batch_hash,
            outcome_command_batch_hash: report.command_batches[1].batch_hash,
        });
    }
    (
        ReplayManifestV4 {
            schema_version: REPLAY_MANIFEST_V4_SCHEMA_VERSION,
            compatibility: compatibility(),
            initial_owner_segments: replay_owner_segments(&initial_checkpoint),
            initial_state_root: compute_world_checkpoint_root(&initial_checkpoint)
                .expect("initial root computes"),
            authority,
            ticks,
            compare_points,
        },
        expected,
    )
}

fn replay_owner_segments(checkpoint: &WorldCheckpointV4) -> Vec<ReplayOwnerSegmentV2> {
    let mut segments = [
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
    let (manifest, expected) = replay_manifest();
    let actual = run_replay_manifest(&manifest).expect("manifest replay is exact");
    assert_eq!(actual, expected);
}

#[test]
fn replay_manifest_v4_jcs_round_trip_is_byte_exact() {
    let (manifest, _) = replay_manifest();
    let bytes = manifest.to_jcs_bytes().expect("manifest encodes");
    let decoded = ReplayManifestV4::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
        .expect("manifest decodes");
    assert_eq!(decoded, manifest);
    assert_eq!(decoded.to_jcs_bytes().expect("manifest re-encodes"), bytes);
}

#[test]
fn replay_manifest_v2_jcs_is_rejected_before_nested_decoding() {
    let bytes = br#"{"schema_version":2}"#;
    assert!(matches!(
        ReplayManifestV4::from_jcs_bytes(bytes, CanonicalDecodeLimits::default()),
        Err(ManifestCodecError::Validation(
            ManifestValidationError::UnsupportedReplayVersion(2)
        ))
    ));
}

#[test]
fn replay_manifest_v3_jcs_is_rejected_as_unsupported_rpg_schema() {
    let bytes = br#"{"schema_version":3}"#;
    assert!(matches!(
        ReplayManifestV4::from_jcs_bytes(bytes, CanonicalDecodeLimits::default()),
        Err(ManifestCodecError::Validation(
            ManifestValidationError::RpgSchemaUnsupported
        ))
    ));
}

#[test]
fn manifest_reports_first_ledger_or_state_divergence() {
    let (mut manifest, _) = replay_manifest();
    manifest.compare_points[0].state_root = StateRoot::from_bytes([9; 32]);
    let error = run_replay_manifest(&manifest).expect_err("compare point must fail");
    assert!(matches!(
        error,
        ReplayError::ComparePointMismatch(ref details)
            if details.first_divergent_tick == 0
    ));
    assert_eq!(error.stable_code(), "NONDETERMINISTIC_RESULT");
}

#[test]
fn manifest_decodes_entire_command_stream_before_runtime_restore() {
    let (mut manifest, _) = replay_manifest();
    manifest.ticks[1].direct_external_commands[0]
        .canonical_command_bytes
        .push(0);
    let error = run_replay_manifest(&manifest).expect_err("corrupt command must fail closed");
    assert!(matches!(error, ReplayError::Manifest(_)));
    assert_eq!(error.stable_code(), "REPLAY_MANIFEST_INVALID");
}

#[test]
fn replay_driver_rejects_command_batch_before_state_mutation() {
    let (manifest, _) = replay_manifest();
    let (checkpoint, mut ticks) = manifest
        .validate_and_decode(CanonicalDecodeLimits::default())
        .expect("manifest decodes");
    let initial_checkpoint = checkpoint.clone();
    let mut authority = AuthorityRegistry::new();
    for grant in &manifest.authority {
        authority
            .register(grant.principal.clone(), grant.capabilities.clone())
            .expect("authority grant");
    }
    let mut driver =
        RuntimeReplayDriver::new(checkpoint, authority).expect("replay driver restores");
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
            phase: next_contracts::CommandPhase::Ingress,
        }
    ));
    assert_eq!(
        driver.world_checkpoint().expect("driver checkpoint"),
        initial_checkpoint
    );
}

#[test]
fn v2_replay_is_rejected_before_nested_snapshot_decoding() {
    let (mut manifest, _) = replay_manifest();
    manifest.schema_version = 2;
    manifest.initial_owner_segments.clear();
    let error = run_replay_manifest(&manifest).expect_err("V2 replay fails closed");
    assert!(matches!(
        error,
        ReplayError::Manifest(ManifestValidationError::UnsupportedReplayVersion(2))
    ));
    assert_eq!(error.stable_code(), "UNSUPPORTED_REPLAY_MANIFEST_VERSION");
}

#[test]
fn v3_replay_is_rejected_as_unsupported_rpg_schema_before_nested_decoding() {
    let (mut manifest, _) = replay_manifest();
    manifest.schema_version = 3;
    manifest.initial_owner_segments.clear();
    let error = run_replay_manifest(&manifest).expect_err("V3 replay fails closed");
    assert!(matches!(
        error,
        ReplayError::Manifest(ManifestValidationError::RpgSchemaUnsupported)
    ));
    assert_eq!(error.stable_code(), "RPG_SCHEMA_UNSUPPORTED");
}

#[test]
fn scenario_final_root_pins_v4_checkpoint_identity_ledger_archive_and_physics_closure() {
    assert_eq!(
        run_replay(&scenario())
            .expect("scenario runs")
            .final_state_root()
            .expect("scenario has ticks")
            .to_hex(),
        "73b473a42565b2bf2bad98d2edca3052bcd15b2ccfe40692cd76abe7dc799de9"
    );
}
