use super::*;

#[test]
fn checkpoint_canonical_components_preserve_exact_bytes_and_ledger_hash() {
    let fixture = physical_fixture();
    let (checkpoint, components) = fixture
        .runtime
        .world_checkpoint_with_canonical_components()
        .expect("checkpoint with canonical components");
    assert_eq!(
        components.runtime_snapshot_bytes(),
        checkpoint
            .runtime_snapshot
            .canonical_bytes()
            .expect("runtime snapshot bytes")
    );
    assert_eq!(
        components.rpg_snapshot_bytes(),
        checkpoint
            .rpg_snapshot
            .canonical_bytes()
            .expect("RPG snapshot bytes")
    );
    assert_eq!(
        components.physics_checkpoint_bytes(),
        checkpoint
            .physics_checkpoint
            .canonical_bytes()
            .expect("physics checkpoint bytes")
    );
    assert_eq!(
        components
            .command_ledger_hash()
            .expect("cached ledger hash"),
        checkpoint
            .runtime_snapshot
            .command_ledger_hash()
            .expect("reference ledger hash")
    );
}

#[test]
fn ingress_generation_exhaustion_preserves_queued_input_and_world_state() {
    let mut fixture = physical_fixture();
    fixture.runtime.ingress_checkpoint.current_generation = u64::MAX;
    let sample = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("enqueue");
    let before_snapshot = fixture.runtime.snapshot();
    let before_physics = fixture.runtime.physics_snapshot().clone();
    let error = fixture.runtime.run_tick([]).expect_err("exhaustion");
    assert_eq!(error.stable_code(), "INGRESS_QUEUE_GENERATION_EXHAUSTED");
    assert_eq!(fixture.runtime.snapshot(), before_snapshot);
    assert_eq!(fixture.runtime.physics_snapshot(), &before_physics);
}

#[test]
fn physical_retry_after_world_checkpoint_restore_never_moves_twice() {
    let mut fixture = physical_fixture();
    let sample = movement_sample(
        &fixture,
        0,
        PlayerActionPhaseV1::Performed,
        [0, 32_767],
        None,
    );
    fixture
        .runtime
        .enqueue_input_sample(&fixture.principal, sample)
        .expect("enqueue");
    let first = fixture.runtime.run_tick([]).expect("first movement");
    let command = first.command_batches[0].body.envelopes[0].clone();
    let pose = first.physics_snapshot.sorted_body_states[&fixture.physics_body_id].pose;
    let receipt_count =
        first.snapshot.command_ledger.streams[&command.stream_id].finalized_receipt_count;
    let checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
    let authority = fixture.runtime.authority.clone();
    let mut restored =
        RuntimeState::restore_world_checkpoint(checkpoint, authority).expect("restore");
    let retry = restored.run_tick([command.clone()]).expect("retry");
    assert_eq!(
        retry
            .results
            .iter()
            .find(|result| result.command_id == command.claimed_command_id.expect("claim"))
            .expect("retry result")
            .disposition,
        CommandDisposition::Deduplicated
    );
    assert!(retry.events.is_empty());
    assert_eq!(
        retry.physics_snapshot.sorted_body_states[&fixture.physics_body_id].pose,
        pose
    );
    assert_eq!(
        retry.snapshot.command_ledger.streams[&command.stream_id].finalized_receipt_count,
        receipt_count
    );
}

#[test]
fn commit_and_retry_are_exact_without_duplicate_event_or_receipt() {
    let mut fixture = fixture();
    let command = command(&fixture, 0, 0);
    let first = fixture.runtime.run_tick([command.clone()]).expect("commit");
    let receipt_count =
        first.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count;
    let retry = fixture.runtime.run_tick([command]).expect("retry");
    assert_eq!(
        retry.results[0].disposition,
        CommandDisposition::Deduplicated
    );
    assert!(retry.events.is_empty());
    assert_eq!(
        retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        receipt_count
    );
}

#[test]
fn domain_rejection_consumes_sequence_and_retry_returns_same_code() {
    let mut fixture = fixture();
    let mut rejected = command(&fixture, 0, 0);
    rejected
        .set_precondition_revision(Some(99))
        .expect("precondition");
    let first = fixture
        .runtime
        .run_tick([rejected.clone()])
        .expect("rejection");
    assert_eq!(
        first.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::PreconditionFailed)
    );
    let retry = fixture.runtime.run_tick([rejected]).expect("retry");
    assert_eq!(
        retry.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::PreconditionFailed)
    );
    assert_eq!(
        retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        1
    );
}

#[test]
fn phase_and_target_rejections_are_terminal_receipts() {
    let mut fixture = fixture();
    let mut wrong_phase = command(&fixture, 0, 0);
    wrong_phase.phase = CommandPhase::Outcome;
    wrong_phase.refresh_command_id().expect("changed phase");
    let mut forbidden_target = command(&fixture, 1, 0);
    forbidden_target.target = Some(PersistentId::from_bytes([8; 16]));
    forbidden_target
        .refresh_command_id()
        .expect("changed target");

    let report = fixture
        .runtime
        .run_tick([wrong_phase.clone(), forbidden_target.clone()])
        .expect("terminal validation rejections");
    assert_eq!(
        report
            .results
            .iter()
            .find(|result| result.sequence == 0)
            .expect("phase result")
            .disposition,
        CommandDisposition::Rejected(RejectionCode::CommandPhaseForbidden)
    );
    assert_eq!(
        report
            .results
            .iter()
            .find(|result| result.sequence == 1)
            .expect("target result")
            .disposition,
        CommandDisposition::Rejected(RejectionCode::TargetNotAllowed)
    );
    assert_eq!(
        report.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        2
    );
    assert_eq!(
        fixture
            .runtime
            .run_tick([wrong_phase])
            .expect("phase retry")
            .results[0]
            .disposition,
        CommandDisposition::Rejected(RejectionCode::CommandPhaseForbidden)
    );
    assert_eq!(
        fixture
            .runtime
            .run_tick([forbidden_target])
            .expect("target retry")
            .results[0]
            .disposition,
        CommandDisposition::Rejected(RejectionCode::TargetNotAllowed)
    );
}

#[test]
fn future_reservation_survives_restore_and_executes_once_when_due() {
    let mut fixture = fixture();
    let future = command(&fixture, 0, 2);
    let reserved = fixture.runtime.run_tick([future.clone()]).expect("reserve");
    assert_eq!(
        reserved.results[0].disposition,
        CommandDisposition::Reserved
    );
    let checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
    let authority = fixture.runtime.authority.clone();
    let mut restored =
        RuntimeState::restore_world_checkpoint(checkpoint, authority).expect("restore");
    assert!(restored.run_tick([]).expect("tick one").events.is_empty());
    let due = restored.run_tick([]).expect("tick two");
    assert_eq!(due.results[0].disposition, CommandDisposition::Committed);
    assert_eq!(due.events.len(), 1);
    let retry = restored.run_tick([future]).expect("retained retry");
    assert_eq!(
        retry.results[0].disposition,
        CommandDisposition::Deduplicated
    );
}

#[test]
fn future_horizon_expiry_and_stream_time_regression_are_terminal() {
    let mut fixture = fixture();
    let beyond_horizon = command(&fixture, 0, 121);
    let future_limit = fixture
        .runtime
        .run_tick([beyond_horizon])
        .expect("future limit");
    assert_eq!(
        future_limit.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandFutureLimit)
    );

    let expired = command(&fixture, 1, 0);
    let expired_result = fixture.runtime.run_tick([expired]).expect("expired");
    assert_eq!(
        expired_result.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandExpired)
    );

    let first_future = command(&fixture, 2, 100);
    assert_eq!(
        fixture
            .runtime
            .run_tick([first_future])
            .expect("future reservation")
            .results[0]
            .disposition,
        CommandDisposition::Reserved
    );
    let regressing = command(&fixture, 3, 99);
    let regression = fixture
        .runtime
        .run_tick([regressing])
        .expect("stable time regression");
    assert_eq!(
        regression.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandStreamTimeRegression)
    );
    let stream = &regression.snapshot.command_ledger.streams[&fixture.stream_id];
    assert_eq!(stream.pending.len(), 1);
    assert_eq!(stream.finalized_receipt_count, 3);
}

#[test]
fn external_collision_locks_stream_and_survives_restore() {
    let mut fixture = fixture();
    let first = command(&fixture, 0, 0);
    let mut second = first.clone();
    second.target_tick = 1;
    second.refresh_command_id().expect("changed body");
    let report = fixture
        .runtime
        .run_tick([first, second.clone()])
        .expect("collision");
    assert!(report.events.is_empty());
    assert_eq!(
        report.snapshot.command_ledger.streams[&fixture.stream_id].state,
        CommandStreamStateV1::CollisionLocked
    );
    assert!(report.results.iter().all(|result| {
        result.disposition == CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
    }));
    let receipt_count =
        report.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count;
    let authority = fixture.runtime.authority.clone();
    let checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
    let mut restored =
        RuntimeState::restore_world_checkpoint(checkpoint, authority).expect("restore");
    assert_eq!(
        restored.command_ledger.streams[&fixture.stream_id].state,
        CommandStreamStateV1::CollisionLocked
    );
    let retry = restored.run_tick([second]).expect("collision retry");
    assert_eq!(
        retry.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
    );
    assert_eq!(
        retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        receipt_count
    );
}

#[test]
fn collision_against_retained_receipt_locks_external_stream() {
    let mut fixture = fixture();
    let original = command(&fixture, 0, 0);
    fixture
        .runtime
        .run_tick([original.clone()])
        .expect("original commits");
    let mut conflicting = original;
    conflicting.target_tick = 1;
    conflicting.refresh_command_id().expect("changed body");
    let report = fixture
        .runtime
        .run_tick([conflicting])
        .expect("retained collision is stable");
    assert_eq!(
        report.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandSequenceCollision)
    );
    assert_eq!(
        report.snapshot.command_ledger.streams[&fixture.stream_id].state,
        CommandStreamStateV1::CollisionLocked
    );
    assert_eq!(
        report.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        1
    );
}

#[test]
fn internal_collision_rolls_back_the_entire_tick() {
    let mut fixture = fixture_for(
        "nextengine.runtime-internal-fixture",
        IssuerPrincipal::InternalSystem(
            SystemId::new("nextengine.system.fixture").expect("system ID"),
        ),
    );
    let before = fixture.runtime.snapshot();
    let first = command(&fixture, 0, 0);
    let mut second = first.clone();
    second.target_tick = 1;
    second.refresh_command_id().expect("changed body");
    let error = fixture
        .runtime
        .run_tick([first, second])
        .expect_err("internal collision is fatal");
    assert_eq!(error, RuntimeFatalError::InternalIdentityCollision);
    assert_eq!(fixture.runtime.snapshot(), before);
}

#[test]
fn event_identity_collision_rolls_back_domain_and_ledger_staging() {
    let mut fixture = fixture();
    let command = command(&fixture, 0, 0);
    let command_id = command.compute_command_id().expect("command ID");
    let event =
        DomainEvent::command_committed(0, CommandPhase::Ingress, command_id, command.sequence)
            .expect("expected event");
    fixture
        .runtime
        .command_ledger
        .causal_identity_registry
        .compare_or_insert(
            CausalIdentityKey {
                identity_kind: CausalIdentityKind::DomainEvent,
                identity_bytes: *event.event_id.as_bytes(),
            },
            content_hash_from_bytes([9; 32]),
        )
        .expect("inject conflicting provenance");
    let before = fixture.runtime.snapshot();

    let error = fixture
        .runtime
        .run_tick([command])
        .expect_err("event identity collision is fatal");
    assert_eq!(error, RuntimeFatalError::InternalIdentityCollision);
    assert_eq!(fixture.runtime.snapshot(), before);
    assert_eq!(fixture.runtime.authoritative_revision(), 0);
    assert!(fixture.runtime.rpg_snapshot().aggregates.is_empty());
}

#[test]
fn pending_capacity_rejects_257th_without_archive_or_receipt_insert() {
    let mut fixture = fixture();
    let commands: Vec<_> = (0..=next_contracts::ledger::COMMAND_PENDING_CAPACITY)
        .map(|sequence| command(&fixture, sequence as u64, 120))
        .collect();
    let report = fixture
        .runtime
        .run_tick(commands)
        .expect("bounded admission");
    let stream = &report.snapshot.command_ledger.streams[&fixture.stream_id];
    assert_eq!(
        stream.pending.len(),
        next_contracts::ledger::COMMAND_PENDING_CAPACITY
    );
    assert_eq!(report.snapshot.body_archive.entries().len(), 256);
    assert_eq!(stream.finalized_receipt_count, 0);
    assert_eq!(
        report.results.last().expect("257th result").disposition,
        CommandDisposition::Rejected(RejectionCode::CommandPendingLimit)
    );
}

#[test]
fn unretained_sequence_below_high_watermark_is_finalized_without_archive_insert() {
    let mut fixture = fixture();
    let report = fixture
        .runtime
        .run_tick([command(&fixture, 1, 0)])
        .expect("sequence gap commits");
    let stream = &report.snapshot.command_ledger.streams[&fixture.stream_id];
    assert_eq!(stream.admission_high_watermark, Some(1));
    assert_eq!(stream.receipt_window[0].subject.sequence(), 1);
    let archive_count = report.snapshot.body_archive.entries().len();
    let finalized = command(&fixture, 0, 0);
    let result = fixture
        .runtime
        .run_tick([finalized])
        .expect("unretained sequence is stable");
    assert_eq!(
        result.results[0].disposition,
        CommandDisposition::Rejected(RejectionCode::CommandSequenceFinalized)
    );
    assert_eq!(result.snapshot.body_archive.entries().len(), archive_count);
    assert_eq!(
        result.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        1
    );
}

#[test]
fn maximum_sequence_exhausts_stream_but_exact_retry_remains_idempotent() {
    let mut fixture = fixture();
    let maximum = command(&fixture, u64::MAX, 0);
    let committed = fixture
        .runtime
        .run_tick([maximum.clone()])
        .expect("maximum sequence commits");
    assert_eq!(
        committed.snapshot.command_ledger.streams[&fixture.stream_id].state,
        CommandStreamStateV1::Exhausted
    );
    let retry = fixture.runtime.run_tick([maximum]).expect("maximum retry");
    assert_eq!(
        retry.results[0].disposition,
        CommandDisposition::Deduplicated
    );
    assert_eq!(
        retry.snapshot.command_ledger.streams[&fixture.stream_id].finalized_receipt_count,
        1
    );
}

#[test]
fn invalid_claim_and_unbound_stream_are_preledger_failures() {
    let mut fixture = fixture();
    let before = fixture.runtime.snapshot();
    let mut invalid = command(&fixture, 0, 0);
    invalid.claimed_command_id = Some(CommandId::from_bytes([9; 16]));
    let unbound = WorldCommand::noop(
        CommandStreamId::from_bytes([8; 16]),
        fixture.principal.clone(),
        0,
        0,
    )
    .expect("unbound");
    let report = fixture
        .runtime
        .run_tick([invalid, unbound])
        .expect("stable rejection");
    assert!(report.events.is_empty());
    assert_eq!(report.snapshot.command_ledger, before.command_ledger);
    assert_eq!(report.snapshot.body_archive, before.body_archive);
}

#[test]
fn bootstrap_rejects_arbitrary_stream_id() {
    let mut fixture = fixture();
    let key = fixture
        .runtime
        .stream_registry
        .entries
        .keys()
        .next()
        .cloned()
        .expect("stream key");
    fixture.runtime.stream_registry.entries = BTreeMap::from([(
        CommandStreamKeyV1 {
            principal: key.principal,
            stream_slot: key.stream_slot,
            stream_epoch: key.stream_epoch,
        },
        CommandStreamId::from_bytes([9; 16]),
    )]);
    assert!(fixture.runtime.stream_registry.validate().is_err());
}

#[test]
fn restore_rejects_world_namespace_drift_before_activation() {
    let fixture = fixture();
    let mut checkpoint = fixture.runtime.world_checkpoint().expect("checkpoint");
    checkpoint
        .runtime_snapshot
        .principal_registry
        .world_namespace = WorldNamespaceId::from_bytes([9; 16]);
    assert!(RuntimeState::restore_world_checkpoint(checkpoint, fixture.runtime.authority).is_err());
}
