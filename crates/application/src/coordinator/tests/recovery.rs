use super::*;

#[test]
fn failed_recovery_publication_keeps_the_prior_failed_session_current() {
    let root = test_root("recovery-publication-rollback");
    let launch = headless_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior safe close");
    drop(prior);

    let mut failed = ApplicationCoordinator::launch(launch.clone()).expect("failed launch");
    failed.run_reference_game(true).expect("failed run");
    let failure = failed
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.recovery-publication").expect("failure code"),
            ),
            None,
        ))
        .expect("terminal save failure becomes durable progress");
    assert!(matches!(failure, ApplicationCloseOutcomeV1::Progress(_)));
    let state_before = failed.state().clone();
    let generation_before = failed.current_generation;
    let failed_publication = failed
        .session_store
        .load_current()
        .expect("exact failed-session evidence");

    failed.inject_fail_next_publication();
    let error = match failed.recover_required_save_failure() {
        Ok(_) => panic!("injected recovery publication must fail"),
        Err(error) => error,
    };
    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");

    let resumed =
        ApplicationCoordinator::resume(launch.clone()).expect("resume prior failed session");
    assert_eq!(resumed.state(), &state_before);
    assert_eq!(resumed.current_generation, generation_before);
    let recovered = resumed
        .recover_required_save_failure()
        .expect("retry recovery");
    assert_eq!(recovered.state().state, ApplicationSessionStatusV1::Active);
    assert_eq!(recovered.durable.recovery_evidence_archive.len(), 1);
    let evidence = recovered.durable.recovery_evidence_archive[0].clone();
    let link_bytes = recovered
        .objects
        .get(&evidence.recovery_link_object_hash)
        .expect("full recovery link bytes")
        .clone();
    let link = next_contracts::session::RecoverySessionLinkV1::from_jcs_bytes(
        &link_bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .expect("decode carried recovery link");
    assert_eq!(link.canonical_hash, evidence.recovery_link_hash);
    assert_eq!(
        recovered.manifest().body.recovery_session_link_hash,
        Some(link.canonical_hash)
    );
    assert_eq!(
        recovered.objects.get(&evidence.prior_snapshot_object_hash),
        Some(&failed_publication.snapshot)
    );
    assert_eq!(
        evidence.evidence_object_hashes,
        failed_publication
            .objects
            .keys()
            .copied()
            .collect::<Vec<_>>()
    );
    for (hash, bytes) in &failed_publication.objects {
        assert_eq!(
            recovered.objects.get(hash),
            Some(bytes),
            "the complete failed-session object closure must remain readable"
        );
    }
    let recovered_generation = recovered.current_generation;
    drop(recovered);

    let restarted =
        ApplicationCoordinator::resume(launch).expect("restart recovered session evidence");
    assert_eq!(restarted.current_generation, recovered_generation);
    assert_eq!(
        restarted.durable.recovery_evidence_archive,
        vec![evidence.clone()]
    );
    assert_eq!(
        restarted.objects.get(&evidence.recovery_link_object_hash),
        Some(&link_bytes)
    );
    assert_eq!(
        restarted.objects.get(&evidence.prior_snapshot_object_hash),
        Some(&failed_publication.snapshot)
    );
    for (hash, bytes) in &failed_publication.objects {
        assert_eq!(restarted.objects.get(hash), Some(bytes));
    }
    cleanup(root);
}
#[test]
fn recovery_evidence_overflow_is_rejected_before_publication() {
    let root = test_root("recovery-evidence-overflow");
    let launch = headless_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior safe close");
    drop(prior);

    let mut failed = ApplicationCoordinator::launch(launch.clone()).expect("failed launch");
    failed.run_reference_game(true).expect("failed run");
    failed
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.recovery-evidence-overflow").expect("failure code"),
            ),
            None,
        ))
        .expect("durable required-save failure");
    failed.durable.recovery_evidence_archive = (0
        ..crate::durable::RECOVERY_EVIDENCE_ARCHIVE_MAX_ENTRIES)
        .map(|index| crate::durable::DurableRecoveryEvidenceEntryV1 {
            recovery_link_hash: ContentHash::from_bytes(
                [u8::try_from(index + 1).expect("bounded evidence index"); 32],
            ),
            recovery_link_object_hash: ContentHash::from_bytes(
                [u8::try_from(index + 65).expect("bounded evidence object index"); 32],
            ),
            prior_snapshot_object_hash: ContentHash::from_bytes(
                [u8::try_from(index + 129).expect("bounded snapshot object index"); 32],
            ),
            evidence_object_hashes: Vec::new(),
        })
        .collect();
    let generation_before = failed.current_generation;

    let error = match failed.recover_required_save_failure() {
        Ok(_) => panic!("N + 1 recovery evidence must fail"),
        Err(error) => error,
    };
    assert_eq!(
        error.diagnostic_code(),
        "SESSION_RECOVERY_EVIDENCE_BUDGET_EXCEEDED"
    );
    let current = SessionStore::new(root.join("sessions"))
        .load_current()
        .expect("unchanged failed-session generation");
    assert_eq!(current.generation_id, generation_before);
    let resumed = ApplicationCoordinator::resume(launch).expect("resume unchanged failure");
    assert_eq!(
        resumed.state().state,
        ApplicationSessionStatusV1::Finalizing
    );
    assert!(resumed.durable.recovery_evidence_archive.is_empty());
    cleanup(root);
}

#[test]
fn recovery_evidence_prior_snapshot_tamper_is_rejected_on_restart() {
    let root = test_root("recovery-evidence-tamper");
    let launch = headless_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior safe close");
    drop(prior);

    let mut failed = ApplicationCoordinator::launch(launch.clone()).expect("failed launch");
    failed.run_reference_game(true).expect("failed run");
    failed
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.recovery-evidence-tamper").expect("failure code"),
            ),
            None,
        ))
        .expect("durable required-save failure");
    let recovered = failed
        .recover_required_save_failure()
        .expect("recover required-save failure");
    let published = recovered
        .session_store
        .load_current()
        .expect("recovered generation");
    let evidence = recovered
        .durable
        .recovery_evidence_archive
        .last()
        .expect("recovery evidence")
        .clone();
    let original = published
        .objects
        .get(&evidence.prior_snapshot_object_hash)
        .expect("full prior durable snapshot");
    let mut tampered_bytes = original.clone();
    let final_byte = tampered_bytes.last_mut().expect("nonempty prior snapshot");
    *final_byte ^= 0x01;
    assert_ne!(tampered_bytes, *original);
    let tampered_object = SessionObjectV1::new(tampered_bytes);
    let mut durable = recovered.durable.clone();
    durable.store_sequence = published.sequence + 1;
    durable
        .recovery_evidence_archive
        .last_mut()
        .expect("mutable recovery evidence")
        .prior_snapshot_object_hash = tampered_object.content_hash;
    let mut objects: Vec<_> = published
        .objects
        .into_values()
        .map(SessionObjectV1::new)
        .collect();
    objects.push(tampered_object);
    let publication = SessionPublicationV1::new(
        durable.store_sequence,
        durable.manifest.body.project_composition_lock_hash,
        Some(durable.state.session_id),
        Some(published.generation_id),
        None,
        durable
            .canonical_bytes()
            .expect("tampered evidence snapshot"),
        objects,
    )
    .expect("tampered evidence publication");
    recovered
        .session_store
        .publish(&publication)
        .expect("publish structurally valid tampered evidence");
    drop(recovered);

    assert!(ApplicationCoordinator::resume(launch).is_err());
    cleanup(root);
}

#[test]
fn recovery_evidence_missing_live_payload_is_rejected_even_if_object_is_still_present() {
    let root = test_root("recovery-evidence-live-payload-missing");
    let launch = interactive_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior
        .begin_reference_game_live(true)
        .expect("prior live run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior safe close");
    drop(prior);

    let mut failed = ApplicationCoordinator::launch(launch.clone()).expect("failed launch");
    failed
        .begin_reference_game_live(true)
        .expect("failed live run");
    failed
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.recovery-live-closure").expect("failure code"),
            ),
            None,
        ))
        .expect("durable required-save failure");
    let recovered = failed
        .recover_required_save_failure()
        .expect("recover required-save failure");
    let published = recovered
        .session_store
        .load_current()
        .expect("recovered generation");
    let evidence = recovered
        .durable
        .recovery_evidence_archive
        .last()
        .expect("recovery evidence");
    let prior_snapshot = published
        .objects
        .get(&evidence.prior_snapshot_object_hash)
        .expect("prior durable snapshot");
    let prior_durable =
        crate::durable::DurableApplicationSnapshotV1::from_canonical_bytes(prior_snapshot)
            .expect("decode prior durable snapshot");
    let live_manifest_hash = prior_durable
        .live_run_recovery_manifest_hash
        .expect("prior live recovery manifest");
    let missing_payload =
        super::super::run::live_run_evidence_payload_hashes(&published.objects, live_manifest_hash)
            .expect("live payload hashes")[0];

    let mut durable = recovered.durable.clone();
    durable.store_sequence = published.sequence + 1;
    let evidence = durable
        .recovery_evidence_archive
        .last_mut()
        .expect("mutable recovery evidence");
    evidence
        .evidence_object_hashes
        .retain(|hash| *hash != missing_payload);
    assert!(
        published.objects.contains_key(&missing_payload),
        "the regression must prove the declared closure rather than mere store presence"
    );
    let publication = SessionPublicationV1::new(
        durable.store_sequence,
        durable.manifest.body.project_composition_lock_hash,
        Some(durable.state.session_id),
        Some(published.generation_id),
        None,
        durable
            .canonical_bytes()
            .expect("incomplete evidence snapshot"),
        published
            .objects
            .into_values()
            .map(SessionObjectV1::new)
            .collect(),
    )
    .expect("structurally valid incomplete evidence publication");
    recovered
        .session_store
        .publish(&publication)
        .expect("publish incomplete declared evidence");
    drop(recovered);

    let error = match ApplicationCoordinator::resume(launch) {
        Err(error) => error,
        Ok(_) => panic!("missing live payload evidence must fail closed"),
    };
    assert_eq!(error.diagnostic_code(), "SESSION_RECOVERY_INCOMPATIBLE");
    cleanup(root);
}

#[test]
fn live_recovery_evidence_rejects_hash_valid_noncanonical_payload() {
    let root = test_root("recovery-evidence-malformed-live-payload");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live run");
    let original_manifest_hash = application
        .durable
        .live_run_recovery_manifest_hash
        .expect("live recovery manifest");
    let mut objects = application.prepared_run_objects.clone();
    let malformed_manifest_hash = super::super::run::replace_live_run_evidence_payload(
        &mut objects,
        original_manifest_hash,
        5,
        b"hash-valid but not a canonical player input session".to_vec(),
    )
    .expect("replace input payload and rebuild canonical manifest");

    let error = super::super::run::validate_live_run_evidence_closure(
        &objects,
        malformed_manifest_hash,
        application.state().session_id,
        application
            .activated_project
            .composition_lock
            .composition_lock_sha256,
        application
            .activated_project
            .content_manifest
            .content_manifest_sha256,
        application
            .state()
            .active_runtime_revision
            .expect("active runtime revision"),
        PresentationTargetKindV1::Interactive,
    )
    .expect_err("typed payload decoding must reject arbitrary hash-valid bytes");
    assert_eq!(error.diagnostic_code(), "SESSION_RECOVERY_INCOMPATIBLE");
    drop(application);
    cleanup(root);
}

#[test]
fn live_recovery_evidence_binds_the_named_session_runtime_revision() {
    let root = test_root("recovery-evidence-runtime-revision");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live run");
    let manifest_hash = application
        .durable
        .live_run_recovery_manifest_hash
        .expect("live recovery manifest");
    let expected_revision = application
        .state()
        .active_runtime_revision
        .expect("active runtime revision");

    let error = super::super::run::validate_live_run_evidence_closure(
        &application.prepared_run_objects,
        manifest_hash,
        application.state().session_id,
        application
            .activated_project
            .composition_lock
            .composition_lock_sha256,
        application
            .activated_project
            .content_manifest
            .content_manifest_sha256,
        expected_revision + 1,
        PresentationTargetKindV1::Interactive,
    )
    .expect_err("historical closure cannot stand for a different runtime revision");
    assert_eq!(error.diagnostic_code(), "SESSION_RECOVERY_INCOMPATIBLE");
    drop(application);
    cleanup(root);
}

#[test]
fn consecutive_required_save_recoveries_carry_the_bounded_evidence_chain() {
    let root = test_root("recovery-evidence-carry-forward");
    let launch = headless_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior safe close");
    drop(prior);

    let mut first_failed =
        ApplicationCoordinator::launch(launch.clone()).expect("first failed launch");
    first_failed
        .run_reference_game(true)
        .expect("first failed run");
    first_failed
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.recovery-chain-first").expect("failure code"),
            ),
            None,
        ))
        .expect("first required-save failure");
    let mut first_recovered = first_failed
        .recover_required_save_failure()
        .expect("first recovery");
    assert_eq!(first_recovered.durable.recovery_evidence_archive.len(), 1);

    first_recovered
        .run_reference_game(true)
        .expect("second failed run");
    first_recovered
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.recovery-chain-second").expect("failure code"),
            ),
            None,
        ))
        .expect("second required-save failure");
    let second_recovered = first_recovered
        .recover_required_save_failure()
        .expect("second recovery");
    assert_eq!(second_recovered.durable.recovery_evidence_archive.len(), 2);
    for entry in &second_recovered.durable.recovery_evidence_archive {
        assert!(
            second_recovered
                .objects
                .contains_key(&entry.recovery_link_object_hash)
        );
        assert!(
            second_recovered
                .objects
                .contains_key(&entry.prior_snapshot_object_hash)
        );
        assert!(
            entry
                .evidence_object_hashes
                .iter()
                .all(|hash| { second_recovered.objects.contains_key(hash) })
        );
    }
    let expected_evidence = second_recovered.durable.recovery_evidence_archive.clone();
    drop(second_recovered);

    let restarted = ApplicationCoordinator::resume(launch).expect("restart evidence chain");
    assert_eq!(
        restarted.durable.recovery_evidence_archive,
        expected_evidence
    );
    cleanup(root);
}
