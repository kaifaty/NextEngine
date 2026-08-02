use super::*;
use crate::command::{IssuerPrincipal, WorldCommand};
use crate::ids::{CommandStreamId, PlayerPrincipalId, SchemaId, content_hash_from_bytes};

fn command(sequence: u64, target_tick: u64) -> WorldCommand {
    WorldCommand::noop(
        CommandStreamId::from_bytes([1; 16]),
        IssuerPrincipal::Player(PlayerPrincipalId::from_bytes([2; 16])),
        sequence,
        target_tick,
    )
    .expect("test command is canonical")
}

fn receipt(ordinal: u64, sequence: u64, result: CommandFinalResultV1) -> CommandReceiptV1 {
    let command = command(sequence, sequence);
    let body_hash = command.body_hash().expect("body hash");
    CommandReceiptV1 {
        schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
        finalization_ordinal: ordinal,
        subject: CommandReceiptSubjectV1::Command {
            stream_id: command.stream_id,
            issuer: command.issuer.clone(),
            sequence,
            command_id: command.compute_command_id().expect("command ID"),
            body_hash,
            canonical_body_ref: body_hash,
        },
        phase: CommandPhase::Ingress,
        target_tick: sequence,
        finalized_at_tick: sequence,
        priority_class: 10,
        command_kind_registry_hash: content_hash_from_bytes([3; 32]),
        result,
        diagnostic_digest: None,
        event_ids: Vec::new(),
        transaction_result_root: content_hash_from_bytes([4; 32]),
    }
}

fn collision_candidate(command: &WorldCommand) -> CommandCollisionCandidateV1 {
    let body_hash = command.body_hash().expect("body hash");
    CommandCollisionCandidateV1 {
        command_id: command.compute_command_id().expect("command ID"),
        body_hash,
        canonical_body_ref: body_hash,
    }
}

fn collision_receipt(ordinal: u64, incident: &CommandCollisionIncidentV1) -> CommandReceiptV1 {
    CommandReceiptV1 {
        schema_version: COMMAND_RECEIPT_SCHEMA_VERSION,
        finalization_ordinal: ordinal,
        subject: CommandReceiptSubjectV1::CollisionSet {
            stream_id: incident.stream_id,
            issuer: incident.issuer.clone(),
            sequence: incident.sequence,
            candidates_root: incident.candidates_root,
            candidate_count: u32::try_from(incident.candidates.len())
                .expect("bounded candidate count"),
            candidates: incident.candidates.clone(),
        },
        phase: CommandPhase::Ingress,
        target_tick: incident.sequence,
        finalized_at_tick: incident.sequence,
        priority_class: 10,
        command_kind_registry_hash: content_hash_from_bytes([3; 32]),
        result: CommandFinalResultV1::Collision {
            code: SchemaId::new("COMMAND_ID_COLLISION").expect("stable code"),
        },
        diagnostic_digest: Some(incident.incident_digest),
        event_ids: Vec::new(),
        transaction_result_root: content_hash_from_bytes([4; 32]),
    }
}

#[test]
fn body_archive_is_order_independent_and_rejects_corrupt_keys() {
    let mut forward = CommandBodyArchiveV1::default();
    let mut reverse = CommandBodyArchiveV1::default();
    let first = command(1, 1);
    let second = command(2, 2);
    forward.insert_command(&first).expect("first body inserts");
    forward
        .insert_command(&second)
        .expect("second body inserts");
    reverse
        .insert_command(&second)
        .expect("second body inserts");
    reverse.insert_command(&first).expect("first body inserts");

    assert_eq!(
        forward.manifest().expect("manifest"),
        reverse.manifest().expect("manifest")
    );
    assert_eq!(
        forward.manifest().expect("cached manifest").archive_root,
        command_body_archive_root(forward.entries()).expect("recomputed archive root")
    );
    assert_eq!(
        forward
            .insert_command(&first)
            .expect("exact retry is idempotent"),
        ArchiveInsertResult::Existing(first.body_hash().expect("body hash"))
    );
    forward.validate().expect("archive closure validates");

    let first_hash = first.body_hash().expect("first body hash");
    let mut corrupt_bytes = forward.canonical_bytes().expect("archive bytes");
    let hash_offset = corrupt_bytes
        .windows(first_hash.as_bytes().len())
        .position(|window| window == first_hash.as_bytes())
        .expect("archive contains the declared command-body hash");
    corrupt_bytes[hash_offset] ^= 0xff;
    assert!(matches!(
        CommandBodyArchiveV1::from_canonical_bytes(
            &corrupt_bytes,
            CanonicalDecodeLimits::default()
        ),
        Err(CommandLedgerError::CommandBodyArchiveCorrupt)
    ));
    assert!(matches!(
        CommandBodyArchiveV1::default().insert_body_bytes(vec![1, 2, 3]),
        Err(CommandLedgerError::Command(_))
    ));
}

#[test]
fn prepared_archive_and_identity_updates_match_eager_mutation() {
    let first = command(1, 1);
    let second = command(2, 2);

    let mut base_archive = CommandBodyArchiveV1::default();
    base_archive
        .insert_command(&first)
        .expect("base body inserts");
    let archive_before = base_archive.clone();
    let second_body_hash = second.body_hash().expect("second body hash");
    let archive_update = base_archive
        .prepare_additions(BTreeMap::from([(
            second_body_hash,
            Arc::from(second.canonical_bytes().expect("second body bytes")),
        )]))
        .expect("archive update prepares");
    assert_eq!(base_archive, archive_before);

    let mut expected_archive = base_archive.clone();
    expected_archive
        .insert_command(&second)
        .expect("eager body inserts");
    assert!(!base_archive.manifest_is_materialized());
    base_archive.commit_prepared_additions(archive_update);
    assert_eq!(base_archive, expected_archive);
    assert!(
        !base_archive.manifest_is_materialized(),
        "ordinary prepared commit must preserve deferred archive-root work"
    );
    assert_eq!(
        base_archive.manifest().expect("prepared manifest"),
        expected_archive.manifest().expect("eager manifest")
    );
    assert!(base_archive.manifest_is_materialized());

    let first_id = first.compute_command_id().expect("first command ID");
    let second_id = second.compute_command_id().expect("second command ID");
    let first_occurrence = CommandIdentityOccurrenceV1 {
        body_hash: first.body_hash().expect("first body hash"),
        first_stream_id: first.stream_id,
        first_sequence: first.sequence,
    };
    let second_occurrence = CommandIdentityOccurrenceV1 {
        body_hash: second_body_hash,
        first_stream_id: second.stream_id,
        first_sequence: second.sequence,
    };
    let mut base_index = CommandIdentityIndexV1::empty().expect("empty index");
    base_index
        .insert_occurrence(first_id, first_occurrence)
        .expect("base occurrence inserts");
    let index_before = base_index.clone();
    let index_update = base_index
        .prepare_replacements(BTreeMap::from([(
            second_id,
            CommandIdentityBindingV1 {
                command_id: second_id,
                occurrences: vec![second_occurrence.clone()],
                state: CommandIdentityBindingState::Unique,
            },
        )]))
        .expect("identity update prepares");
    assert_eq!(base_index, index_before);

    let mut expected_index = base_index.clone();
    expected_index
        .insert_occurrence(second_id, second_occurrence)
        .expect("eager occurrence inserts");
    base_index.commit_prepared_replacements(index_update);
    assert_eq!(base_index, expected_index);
    base_index.validate().expect("prepared index validates");
}

#[test]
fn prepared_archive_commit_transfers_an_already_materialized_manifest() {
    let command = command(7, 1);
    let body_hash = command.body_hash().expect("body hash");
    let mut archive = CommandBodyArchiveV1::default();
    let update = archive
        .prepare_additions(BTreeMap::from([(
            body_hash,
            Arc::from(command.canonical_bytes().expect("body bytes")),
        )]))
        .expect("archive update prepares");
    let expected_manifest = update.manifest();

    archive.commit_prepared_additions(update);

    assert!(archive.manifest_is_materialized());
    assert_eq!(
        archive.manifest().expect("transferred manifest"),
        expected_manifest
    );
}

#[test]
fn identity_index_records_full_hash_collisions_without_replacing_first_occurrence() {
    let mut index = CommandIdentityIndexV1::empty().expect("empty index");
    let command_id = CommandId::from_bytes([7; 16]);
    let first = CommandIdentityOccurrenceV1 {
        body_hash: command_body_hash_from_bytes([1; 32]),
        first_stream_id: CommandStreamId::from_bytes([2; 16]),
        first_sequence: 3,
    };
    let second = CommandIdentityOccurrenceV1 {
        body_hash: command_body_hash_from_bytes([4; 32]),
        first_stream_id: CommandStreamId::from_bytes([5; 16]),
        first_sequence: 6,
    };

    assert_eq!(
        index
            .insert_occurrence(command_id, first.clone())
            .expect("first occurrence"),
        IdentityInsertResult::Inserted
    );
    assert_eq!(
        index
            .insert_occurrence(command_id, first)
            .expect("exact occurrence"),
        IdentityInsertResult::Existing
    );
    assert_eq!(
        index
            .insert_occurrence(command_id, second)
            .expect("collision occurrence"),
        IdentityInsertResult::Collision
    );
    index.validate().expect("index remains valid");
    assert_eq!(
        index.body.bindings[&command_id].state,
        CommandIdentityBindingState::Collision
    );
    assert_eq!(index.body.occurrence_count, 2);
}

#[test]
fn complete_ledger_round_trip_rejects_corrupt_archive_index_and_chain_roots() {
    let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
    let (mut ledger, mut archive) = CommandLedgerV2::empty(
        world_namespace,
        content_hash_from_bytes([3; 32]),
        content_hash_from_bytes([5; 32]),
    )
    .expect("empty ledger");
    let command = command(0, 0);
    let command_id = command.compute_command_id().expect("command ID");
    let body_hash = command.body_hash().expect("body hash");
    archive.insert_command(&command).expect("archive command");
    ledger
        .identity_index
        .insert_occurrence(
            command_id,
            CommandIdentityOccurrenceV1 {
                body_hash,
                first_stream_id: command.stream_id,
                first_sequence: command.sequence,
            },
        )
        .expect("identity occurrence");
    let mut stream =
        CommandStreamLedgerV2::genesis(command.stream_id, command.issuer.clone(), 0, 0);
    stream
        .append_receipt(receipt(0, 0, CommandFinalResultV1::Committed))
        .expect("receipt");
    ledger.streams.insert(command.stream_id, stream);
    ledger
        .synchronize_archive(&archive)
        .expect("archive closure");

    let archive_bytes = archive.canonical_bytes().expect("archive bytes");
    let decoded_archive = CommandBodyArchiveV1::from_canonical_bytes(
        &archive_bytes,
        CanonicalDecodeLimits::default(),
    )
    .expect("archive decodes");
    assert_eq!(decoded_archive, archive);
    let bytes = ledger.canonical_bytes(&archive).expect("ledger bytes");
    let decoded =
        CommandLedgerV2::from_canonical_bytes(&bytes, &archive, CanonicalDecodeLimits::default())
            .expect("ledger decodes");
    assert_eq!(decoded, ledger);
    assert_eq!(
        decoded
            .canonical_bytes(&archive)
            .expect("ledger re-encodes"),
        bytes
    );

    let mut corrupt_chain = ledger.clone();
    corrupt_chain
        .streams
        .get_mut(&command.stream_id)
        .expect("stream")
        .receipt_chain_root = ContentHash::from_bytes([9; 32]);
    assert_eq!(
        corrupt_chain.validate(&archive),
        Err(CommandLedgerError::ReceiptChainRootMismatch)
    );

    let mut corrupt_archive_root = ledger.clone();
    corrupt_archive_root.body_archive.archive_root = ContentHash::from_bytes([9; 32]);
    assert_eq!(
        corrupt_archive_root.validate(&archive),
        Err(CommandLedgerError::CommandBodyArchiveCorrupt)
    );

    let mut corrupt_index_root = ledger.clone();
    corrupt_index_root.identity_index.index_root = ContentHash::from_bytes([9; 32]);
    assert_eq!(
        corrupt_index_root.validate(&archive),
        Err(CommandLedgerError::IdentityIndexRootMismatch)
    );

    assert_eq!(
        ledger.validate(&CommandBodyArchiveV1::default()),
        Err(CommandLedgerError::CommandBodyArchiveCorrupt)
    );
}

#[test]
fn incremental_checkpoint_rejects_a_re_rooted_identity_command_id_forgery() {
    let (mut ledger, mut archive) = CommandLedgerV2::empty(
        WorldNamespaceId::from_bytes([8; 16]),
        content_hash_from_bytes([3; 32]),
        content_hash_from_bytes([5; 32]),
    )
    .expect("empty ledger");
    let archived = command(0, 0);
    let body_hash = archived.body_hash().expect("body hash");
    archive.insert_command(&archived).expect("archive command");
    ledger.body_archive = archive.manifest().expect("archive manifest");

    let forged_command_id = CommandId::from_bytes([7; 16]);
    assert_ne!(
        forged_command_id,
        archived.compute_command_id().expect("computed command ID")
    );
    ledger.identity_index = CommandIdentityIndexV1::from_bindings(BTreeMap::from([(
        forged_command_id,
        CommandIdentityBindingV1 {
            command_id: forged_command_id,
            occurrences: vec![CommandIdentityOccurrenceV1 {
                body_hash,
                first_stream_id: archived.stream_id,
                first_sequence: archived.sequence,
            }],
            state: CommandIdentityBindingState::Unique,
        },
    )]))
    .expect("forged index has a self-consistent public root");
    ledger
        .identity_index
        .validate()
        .expect("public identity structure and root are internally valid");

    assert_eq!(
        ledger.validate_incremental_checkpoint(&archive),
        Err(CommandLedgerError::IdentityCommandIdMismatch)
    );
}

#[test]
fn pending_limit_is_exact_at_256() {
    let issuer = command(0, 0).issuer.clone();
    let mut stream =
        CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
    for sequence in 0..COMMAND_PENDING_CAPACITY as u64 {
        let command = command(sequence, sequence);
        stream
            .reserve(
                CommandReservationV1::from_command(
                    &command,
                    0,
                    10,
                    content_hash_from_bytes([3; 32]),
                )
                .expect("reservation"),
            )
            .expect("within pending limit");
        if sequence == 254 {
            assert_eq!(stream.pending.len(), 255);
        }
    }
    assert_eq!(stream.pending.len(), COMMAND_PENDING_CAPACITY);
    stream
        .reserve(
            CommandReservationV1::from_command(
                &command(0, 0),
                0,
                10,
                content_hash_from_bytes([3; 32]),
            )
            .expect("retry reservation"),
        )
        .expect("exact retry is idempotent at capacity");
    assert_eq!(stream.pending.len(), COMMAND_PENDING_CAPACITY);
    let overflow = command(
        COMMAND_PENDING_CAPACITY as u64,
        COMMAND_PENDING_CAPACITY as u64,
    );
    assert_eq!(
        stream.reserve(
            CommandReservationV1::from_command(&overflow, 0, 10, content_hash_from_bytes([3; 32]),)
                .expect("reservation"),
        ),
        Err(CommandLedgerError::PendingLimit)
    );
}

#[test]
fn receipt_window_is_the_exact_4096_suffix() {
    let issuer = command(0, 0).issuer.clone();
    let mut stream =
        CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
    assert!(stream.receipt_window.is_empty());
    assert_eq!(stream.finalized_receipt_count, 0);
    for ordinal in 0..=COMMAND_RECEIPT_WINDOW_CAPACITY as u64 {
        stream
            .append_receipt(receipt(ordinal, ordinal, CommandFinalResultV1::Committed))
            .expect("receipt appends");
        match ordinal {
            0 => {
                assert_eq!(stream.receipt_window.len(), 1);
                assert_eq!(stream.finalized_receipt_count, 1);
            }
            4094 => assert_eq!(stream.receipt_window.len(), 4095),
            4095 => assert_eq!(stream.receipt_window.len(), 4096),
            _ => {}
        }
    }
    assert_eq!(
        stream.finalized_receipt_count,
        COMMAND_RECEIPT_WINDOW_CAPACITY as u64 + 1
    );
    assert_eq!(stream.receipt_window.len(), COMMAND_RECEIPT_WINDOW_CAPACITY);
    assert_eq!(stream.receipt_window[0].finalization_ordinal, 1);
    assert_eq!(
        stream
            .receipt_window
            .last()
            .expect("last receipt")
            .finalization_ordinal,
        COMMAND_RECEIPT_WINDOW_CAPACITY as u64
    );
    stream.validate().expect("window suffix validates");

    let retained = stream.receipt_window.iter().cloned().collect();
    let repacked = CommandReceiptWindowV1::from_receipts(retained)
        .expect("the retained suffix fits the public window bound");
    assert_eq!(
        repacked, stream.receipt_window,
        "physical chunk boundaries must not affect logical equality"
    );

    let retained_before_append = stream.receipt_window.clone();
    let next_ordinal = COMMAND_RECEIPT_WINDOW_CAPACITY as u64 + 1;
    stream
        .append_receipt(receipt(
            next_ordinal,
            next_ordinal,
            CommandFinalResultV1::Committed,
        ))
        .expect("a cloned window remains isolated from the next append");
    assert_eq!(
        retained_before_append
            .first()
            .expect("cloned suffix remains populated")
            .finalization_ordinal,
        1
    );
    assert_eq!(
        stream
            .receipt_window
            .first()
            .expect("advanced suffix remains populated")
            .finalization_ordinal,
        2
    );

    for ordinal in (next_ordinal + 1)..=(COMMAND_RECEIPT_WINDOW_CAPACITY as u64 + 128) {
        stream
            .append_receipt(receipt(ordinal, ordinal, CommandFinalResultV1::Committed))
            .expect("receipt appends across multiple chunk rotations");
    }
    assert_eq!(stream.receipt_window.len(), COMMAND_RECEIPT_WINDOW_CAPACITY);
    assert_eq!(
        stream
            .receipt_window
            .first()
            .expect("rotated suffix remains populated")
            .finalization_ordinal,
        129
    );
    stream.validate().expect("rotated window suffix validates");

    let streams = BTreeMap::from([(stream.stream_id, stream.clone())]);
    let canonical = super::codec::encode_streams(&streams).expect("streams encode");
    let decoded = super::codec::decode_streams(&canonical, CanonicalDecodeLimits::default())
        .expect("streams decode");
    assert_eq!(decoded, streams);
    assert_eq!(
        super::codec::encode_streams(&decoded).expect("decoded streams re-encode"),
        canonical,
        "physical chunk packing must not affect canonical bytes"
    );
}

#[test]
fn finalized_sequence_cannot_receive_a_second_receipt() {
    let issuer = command(0, 0).issuer.clone();
    let mut stream =
        CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
    stream
        .append_receipt(receipt(0, 7, CommandFinalResultV1::Committed))
        .expect("first receipt appends");
    let before = stream.clone();
    assert_eq!(
        stream.append_receipt(receipt(1, 7, CommandFinalResultV1::Committed)),
        Err(CommandLedgerError::SequenceNotNew)
    );
    assert_eq!(stream, before);
}

#[test]
fn pending_receipt_must_match_its_reservation_without_partial_mutation() {
    let issuer = command(0, 0).issuer.clone();
    let mut stream =
        CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
    let reserved = command(3, 9);
    stream
        .reserve(
            CommandReservationV1::from_command(&reserved, 1, 10, content_hash_from_bytes([3; 32]))
                .expect("reservation"),
        )
        .expect("reservation appends");
    let before = stream.clone();
    let mismatched = receipt(0, 3, CommandFinalResultV1::Committed);
    assert_eq!(
        stream.append_receipt(mismatched),
        Err(CommandLedgerError::ReservationMismatch)
    );
    assert_eq!(stream, before);
}

#[test]
fn maximum_sequence_exhausts_stream_atomically() {
    let issuer = command(0, 0).issuer.clone();
    let mut stream =
        CommandStreamLedgerV2::genesis(CommandStreamId::from_bytes([1; 16]), issuer, 0, 0);
    stream
        .append_receipt(receipt(0, u64::MAX, CommandFinalResultV1::Committed))
        .expect("maximum sequence finalizes");
    assert_eq!(stream.state, CommandStreamStateV1::Exhausted);
    assert_eq!(stream.admission_high_watermark, Some(u64::MAX));
    stream.validate().expect("exhausted stream validates");
}

#[test]
fn maximum_sequence_pending_retry_remains_idempotent_after_exhaustion() {
    let maximum = command(u64::MAX, u64::MAX);
    let reservation =
        CommandReservationV1::from_command(&maximum, 0, 10, content_hash_from_bytes([3; 32]))
            .expect("maximum reservation");
    let mut stream =
        CommandStreamLedgerV2::genesis(maximum.stream_id, maximum.issuer.clone(), 0, 0);
    stream
        .reserve(reservation.clone())
        .expect("maximum sequence reserves");
    assert_eq!(stream.state, CommandStreamStateV1::Exhausted);
    stream
        .reserve(reservation)
        .expect("pending exact retry remains idempotent");
    assert_eq!(stream.pending.len(), 1);
}

#[test]
fn new_collision_locks_and_finalizes_in_one_atomic_operation() {
    let first = command(7, 7);
    let second = command(7, 8);
    let incident = CommandCollisionIncidentV1::new(
        first.stream_id,
        first.issuer.clone(),
        first.sequence,
        vec![collision_candidate(&second), collision_candidate(&first)],
    )
    .expect("collision incident");
    let mut stream = CommandStreamLedgerV2::genesis(first.stream_id, first.issuer.clone(), 0, 0);
    let receipt = collision_receipt(0, &incident);
    stream
        .append_collision_receipt(incident.clone(), receipt)
        .expect("collision locks and finalizes");

    assert_eq!(stream.state, CommandStreamStateV1::CollisionLocked);
    assert_eq!(stream.collision_incident, Some(incident.clone()));
    assert_eq!(stream.admission_high_watermark, Some(7));
    assert_eq!(stream.finalized_receipt_count, 1);
    assert_eq!(stream.receipt_window.len(), 1);
    stream
        .validate()
        .expect("collision-locked stream validates");

    let before = stream.clone();
    assert_eq!(
        stream.append_collision_receipt(incident.clone(), collision_receipt(1, &incident),),
        Err(CommandLedgerError::UnexpectedCollisionReceipt)
    );
    assert_eq!(stream, before);
}

#[test]
fn ledger_recomputes_command_id_from_archived_body() {
    let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
    let (mut ledger, mut archive) = CommandLedgerV2::empty(
        world_namespace,
        content_hash_from_bytes([3; 32]),
        content_hash_from_bytes([4; 32]),
    )
    .expect("empty ledger");
    let archived = command(1, 1);
    let body_hash = archived.body_hash().expect("body hash");
    archive.insert_command(&archived).expect("archive insert");
    ledger
        .identity_index
        .insert_occurrence(
            CommandId::from_bytes([9; 16]),
            CommandIdentityOccurrenceV1 {
                body_hash,
                first_stream_id: archived.stream_id,
                first_sequence: archived.sequence,
            },
        )
        .expect("index insert");
    ledger.body_archive = archive.manifest().expect("archive manifest");

    assert_eq!(
        ledger.validate(&archive),
        Err(CommandLedgerError::IdentityCommandIdMismatch)
    );
}

#[test]
fn archive_root_is_stable_across_deterministic_insert_permutations() {
    let commands: Vec<_> = (0..16)
        .map(|sequence| command(sequence, sequence + 1))
        .collect();
    let mut baseline = CommandBodyArchiveV1::default();
    for command in &commands {
        baseline.insert_command(command).expect("baseline insert");
    }
    let expected = baseline.manifest().expect("baseline manifest");

    for rotation in 0..commands.len() {
        let mut order: Vec<_> = (0..commands.len()).collect();
        order.rotate_left(rotation);
        if rotation % 2 == 1 {
            order.reverse();
        }
        let mut archive = CommandBodyArchiveV1::default();
        for index in order {
            archive
                .insert_command(&commands[index])
                .expect("permuted insert");
        }
        assert_eq!(archive.manifest().expect("permuted manifest"), expected);
    }
}

#[test]
fn archive_synchronization_is_atomic_on_inconsistent_input() {
    let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
    let (mut ledger, _) = CommandLedgerV2::empty(
        world_namespace,
        content_hash_from_bytes([3; 32]),
        content_hash_from_bytes([4; 32]),
    )
    .expect("empty ledger");
    let before = ledger.clone();
    let mut inconsistent = CommandBodyArchiveV1::default();
    inconsistent
        .insert_command(&command(1, 1))
        .expect("valid unmatched archive entry");

    assert!(ledger.synchronize_archive(&inconsistent).is_err());
    assert_eq!(ledger, before);
}

#[test]
fn receipt_canonical_bytes_and_chain_are_result_sensitive() {
    let committed = receipt(0, 0, CommandFinalResultV1::Committed);
    let rejected = receipt(
        0,
        0,
        CommandFinalResultV1::Rejected {
            code: SchemaId::new("COMMAND_PRECONDITION_FAILED").expect("stable code"),
        },
    );
    assert_ne!(
        committed.canonical_bytes().expect("receipt bytes"),
        rejected.canonical_bytes().expect("receipt bytes")
    );
    assert_ne!(
        command_receipt_chain_next(command_receipt_chain_genesis(), &committed)
            .expect("chain root"),
        command_receipt_chain_next(command_receipt_chain_genesis(), &rejected).expect("chain root")
    );
}

#[test]
fn incremental_checkpoint_accepts_a_stream_beyond_the_retained_window_capacity() {
    // At capacity + 1 the complete validator can no longer recompute the
    // receipt chain from genesis either, so both validators rely on the
    // append-time chain maintenance. The bounded incremental validator must
    // accept this exact state.
    let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
    let (mut ledger, mut archive) = CommandLedgerV2::empty(
        world_namespace,
        content_hash_from_bytes([3; 32]),
        content_hash_from_bytes([5; 32]),
    )
    .expect("empty ledger");
    let mut bindings = BTreeMap::new();
    let mut stream = None;
    for sequence in 0..=COMMAND_RECEIPT_WINDOW_CAPACITY as u64 {
        let command = command(sequence, sequence);
        let command_id = command.compute_command_id().expect("command ID");
        let body_hash = command.body_hash().expect("body hash");
        archive.insert_command(&command).expect("archive command");
        bindings.insert(
            command_id,
            CommandIdentityBindingV1 {
                command_id,
                occurrences: vec![CommandIdentityOccurrenceV1 {
                    body_hash,
                    first_stream_id: command.stream_id,
                    first_sequence: command.sequence,
                }],
                state: CommandIdentityBindingState::Unique,
            },
        );
        let stream = stream.get_or_insert_with(|| {
            CommandStreamLedgerV2::genesis(command.stream_id, command.issuer.clone(), 0, 0)
        });
        stream
            .append_receipt(receipt(sequence, sequence, CommandFinalResultV1::Committed))
            .expect("receipt");
    }
    let stream = stream.expect("stream assembled");
    assert_eq!(stream.receipt_window.len(), COMMAND_RECEIPT_WINDOW_CAPACITY);
    assert_eq!(
        stream.finalized_receipt_count,
        COMMAND_RECEIPT_WINDOW_CAPACITY as u64 + 1
    );
    ledger.identity_index =
        CommandIdentityIndexV1::from_bindings(bindings).expect("identity index");
    ledger.streams.insert(stream.stream_id, stream);
    ledger
        .synchronize_archive(&archive)
        .expect("archive closure");

    ledger.validate(&archive).expect("complete validation");
    ledger
        .validate_incremental_checkpoint(&archive)
        .expect("bounded incremental checkpoint validation");
}

#[test]
fn incremental_checkpoint_rejects_bounded_stream_and_index_faults() {
    let world_namespace = WorldNamespaceId::from_bytes([8; 16]);
    let (mut ledger, mut archive) = CommandLedgerV2::empty(
        world_namespace,
        content_hash_from_bytes([3; 32]),
        content_hash_from_bytes([5; 32]),
    )
    .expect("empty ledger");
    let command = command(0, 0);
    let command_id = command.compute_command_id().expect("command ID");
    let body_hash = command.body_hash().expect("body hash");
    archive.insert_command(&command).expect("archive command");
    ledger
        .identity_index
        .insert_occurrence_incremental(
            command_id,
            CommandIdentityOccurrenceV1 {
                body_hash,
                first_stream_id: command.stream_id,
                first_sequence: command.sequence,
            },
        )
        .expect("identity occurrence");
    let mut stream =
        CommandStreamLedgerV2::genesis(command.stream_id, command.issuer.clone(), 0, 0);
    stream
        .append_receipt(receipt(0, 0, CommandFinalResultV1::Committed))
        .expect("receipt");
    ledger.streams.insert(command.stream_id, stream);
    ledger
        .synchronize_archive(&archive)
        .expect("archive closure");
    ledger
        .validate_incremental_checkpoint(&archive)
        .expect("base state passes the bounded validator");

    let mut corrupt = ledger.clone();
    corrupt
        .streams
        .get_mut(&command.stream_id)
        .expect("stream")
        .finalized_receipt_count = 2;
    assert_eq!(
        corrupt.validate_incremental_checkpoint(&archive),
        Err(CommandLedgerError::ReceiptWindowLengthMismatch)
    );

    let mut corrupt = ledger.clone();
    let stream = corrupt.streams.get_mut(&command.stream_id).expect("stream");
    stream.receipt_window =
        CommandReceiptWindowV1::from_receipts(vec![receipt(9, 0, CommandFinalResultV1::Committed)])
            .expect("single-receipt window");
    assert_eq!(
        corrupt.validate_incremental_checkpoint(&archive),
        Err(CommandLedgerError::ReceiptWindowInvalid)
    );

    let mut corrupt = ledger.clone();
    let stream = corrupt.streams.get_mut(&command.stream_id).expect("stream");
    let mut forged_reservation =
        CommandReservationV1::from_command(&command, 7, 0, content_hash_from_bytes([3; 32]))
            .expect("reservation from command");
    forged_reservation.canonical_body_ref = command_body_hash_from_bytes([9; 32]);
    Arc::make_mut(&mut stream.pending).insert(7, forged_reservation);
    assert_eq!(
        corrupt.validate_incremental_checkpoint(&archive),
        Err(CommandLedgerError::ReservationMismatch)
    );

    let mut corrupt = ledger.clone();
    corrupt.identity_index.body.command_id_count += 1;
    assert_eq!(
        corrupt.validate_incremental_checkpoint(&archive),
        Err(CommandLedgerError::IdentityIndexCountMismatch)
    );

    let mut corrupt = ledger.clone();
    corrupt.body_archive.entry_count += 1;
    assert_eq!(
        corrupt.validate_incremental_checkpoint(&archive),
        Err(CommandLedgerError::CommandBodyArchiveCorrupt)
    );
}
