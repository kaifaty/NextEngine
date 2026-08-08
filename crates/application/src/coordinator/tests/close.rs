use super::*;

#[test]
fn production_run_closes_once_and_restart_returns_the_same_receipt() {
    let root = test_root("close-restart");
    let launch = headless_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    let run = application
        .run_reference_game(true)
        .expect("reference game");
    assert_eq!(run.ticks, 32);
    assert_eq!(run.events, 27);
    let closed = application
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    let receipt_hash = closed.receipt_hash().expect("terminal receipt");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Closed
    );

    drop(application);
    let mut resumed = ApplicationCoordinator::resume(launch).expect("resume closed");
    let retried = resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("exact terminal retry");
    assert_eq!(retried.receipt_hash(), Some(receipt_hash));
    cleanup(root);
}
#[test]
fn legacy_v1_active_generation_without_live_manifest_fails_closed() {
    let root = test_root("legacy-live-recovery");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    let published = application
        .session_store
        .load_current()
        .expect("current generation");
    let mut durable = application.durable.clone();
    durable.store_sequence = published.sequence + 1;
    let legacy_snapshot =
        durable_v1_bytes(&durable.canonical_bytes().expect("durable v2 snapshot"));
    let publication = SessionPublicationV1::new(
        durable.store_sequence,
        durable.manifest.body.project_composition_lock_hash,
        Some(durable.state.session_id),
        Some(published.generation_id),
        None,
        legacy_snapshot,
        published
            .objects
            .into_values()
            .map(SessionObjectV1::new)
            .collect(),
    )
    .expect("legacy publication");
    application
        .session_store
        .publish(&publication)
        .expect("publish legacy generation");
    drop(application);

    let error = match ApplicationCoordinator::resume(launch) {
        Err(error) => error,
        Ok(_) => panic!("legacy active runtime must fail closed"),
    };
    assert_eq!(error.diagnostic_code(), "SESSION_RECOVERY_INCOMPATIBLE");
    cleanup(root);
}
#[test]
fn durable_v2_active_generation_without_a_lifecycle_archive_fails_closed() {
    let root = test_root("durable-v2-live-recovery");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live game");
    let published = application
        .session_store
        .load_current()
        .expect("current generation");
    let mut durable = application.durable.clone();
    durable.store_sequence = published.sequence + 1;
    let legacy_snapshot =
        durable_v2_bytes(&durable.canonical_bytes().expect("durable v3 snapshot"));
    let publication = SessionPublicationV1::new(
        durable.store_sequence,
        durable.manifest.body.project_composition_lock_hash,
        Some(durable.state.session_id),
        Some(published.generation_id),
        None,
        legacy_snapshot,
        published
            .objects
            .into_values()
            .map(SessionObjectV1::new)
            .collect(),
    )
    .expect("v2 publication");
    application
        .session_store
        .publish(&publication)
        .expect("publish v2 generation");
    drop(application);

    let error = match ApplicationCoordinator::resume(launch) {
        Err(error) => error,
        Ok(_) => panic!("v2 active generation has no truthful lifecycle archive"),
    };
    assert_eq!(error.diagnostic_code(), "SESSION_RECOVERY_INCOMPATIBLE");
    cleanup(root);
}

#[test]
fn durable_v3_rejects_a_truncated_lifecycle_archive_sequence() {
    let root = test_root("durable-v3-archive-malformed");
    let application = ApplicationCoordinator::launch(headless_launch(&root)).expect("launch");
    let bytes = application
        .durable
        .canonical_bytes()
        .expect("durable v3 snapshot");
    let decoded = next_contracts::canonical::decode_canonical_segment(
        &bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .expect("decode durable v3");
    let mut fields = decoded.fields;
    fields
        .iter_mut()
        .find(|field| field.field_id == 7)
        .expect("lifecycle archive field")
        .payload
        .pop();
    let malformed = next_contracts::canonical::encode_canonical_segment(
        &decoded.owner_id,
        &decoded.schema_id,
        &decoded.segment_id,
        fields,
    )
    .expect("re-encode malformed durable snapshot");

    assert!(matches!(
        crate::durable::DurableApplicationSnapshotV1::from_canonical_bytes(&malformed),
        Err(crate::ApplicationError::DurableSnapshotInvalid)
    ));
    cleanup(root);
}

#[test]
fn durable_v3_requires_the_complete_lifecycle_chain_from_revision_zero() {
    let root = test_root("durable-v3-lifecycle-chain-gap");
    let launch = headless_launch(&root);
    let application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    let published = application
        .session_store
        .load_current()
        .expect("active generation");
    let mut durable = application.durable.clone();
    durable.store_sequence = published.sequence + 1;
    durable.lifecycle_archive.remove(0);
    let publication = SessionPublicationV1::new(
        durable.store_sequence,
        durable.manifest.body.project_composition_lock_hash,
        Some(durable.state.session_id),
        Some(published.generation_id),
        None,
        durable
            .canonical_bytes()
            .expect("gapped lifecycle snapshot"),
        published
            .objects
            .into_values()
            .map(SessionObjectV1::new)
            .collect(),
    )
    .expect("gapped lifecycle publication");
    application
        .session_store
        .publish(&publication)
        .expect("publish structurally valid lifecycle gap");
    drop(application);

    let error = match ApplicationCoordinator::resume(launch) {
        Err(error) => error,
        Ok(_) => panic!("v3 lifecycle history gap must fail closed"),
    };
    assert_eq!(error.diagnostic_code(), "SESSION_RECOVERY_INCOMPATIBLE");
    cleanup(root);
}

#[test]
fn same_close_id_with_different_canonical_bytes_is_rejected() {
    let root = test_root("close-collision");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("request");
    let collision = CloseSessionRequestV1::new(
        request.close_request_id,
        request.session_id,
        request.starting_session_revision,
        request.starting_session_state,
        request.shutdown_policy_hash,
        request.final_save_policy,
        BoundedDeadlineClassV1::Immediate,
        request.reason.clone(),
        request.causal_input_reference.clone(),
    )
    .expect("colliding request");
    application
        .close_with_request(
            request,
            CloseExecutionOptionsV1::with_failure(
                FinalSaveAttemptFailureV1::Retryable(
                    SchemaId::new("nextengine.test.retry").expect("code"),
                ),
                None,
            ),
        )
        .expect("first attempt");
    let error = application
        .close_with_request(collision, CloseExecutionOptionsV1::default())
        .expect_err("collision rejected");
    assert_eq!(
        error.diagnostic_code(),
        "SESSION_FINAL_SAVE_IDENTITY_COLLISION"
    );
    cleanup(root);
}

#[test]
fn required_save_exhaustion_stays_finalizing_and_can_recover_from_last_safe() {
    let root = test_root("required-recovery");
    let launch = headless_launch(&root);
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior close");
    drop(prior);

    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application.run_reference_game(true).expect("run");
    for attempt in 0..3 {
        let outcome = application
            .close(CloseExecutionOptionsV1::with_failure(
                FinalSaveAttemptFailureV1::Retryable(
                    SchemaId::new("nextengine.test.save-unavailable").expect("code"),
                ),
                None,
            ))
            .expect("failure is durable progress");
        assert!(matches!(outcome, ApplicationCloseOutcomeV1::Progress(_)));
        assert_eq!(
            application.state().state,
            ApplicationSessionStatusV1::Finalizing
        );
        if attempt < 2 {
            assert!(application.state().terminal_receipt_hash.is_none());
        }
    }
    let failed_session_id = application.state().session_id;
    drop(application);

    let resumed = ApplicationCoordinator::resume(launch).expect("resume failed close");
    assert_eq!(
        resumed.state().state,
        ApplicationSessionStatusV1::Finalizing
    );
    let recovered = resumed.recover_required_save_failure().expect("recovery");
    assert_eq!(recovered.state().state, ApplicationSessionStatusV1::Active);
    assert_ne!(recovered.state().session_id, failed_session_id);
    assert!(recovered.state().active_save_generation_hash.is_some());
    cleanup(root);
}

#[test]
fn allow_last_safe_policy_closes_after_terminal_save_failure() {
    let root = test_root("last-safe");
    let project_root = root.join("published-project");
    let mut source = project_source_v2().expect("source");
    source.shutdown_policy =
        ShutdownPolicyV1::new(1, FailureDispositionV1::AllowLastSafeGeneration).expect("policy");
    let cooked = cook_project_v1(source).expect("cook");
    ContentStore::new(&project_root)
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let launch = LaunchRequestV1 {
        project: ProjectSelectionV1::PublishedStateRoot(project_root),
        expected_project_lock: Some(cooked.composition_lock.composition_lock_sha256),
        state_root: root.clone(),
        composition_root: CompositionRootV1::Headless,
        presentation_target: PresentationTargetKindV1::None,
        platform_capability_set: None,
    };
    let mut prior = ApplicationCoordinator::launch(launch.clone()).expect("prior launch");
    prior.run_reference_game(true).expect("prior run");
    let prior_close = prior
        .close(CloseExecutionOptionsV1::default())
        .expect("prior close");
    let last_safe = match prior_close {
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(hash),
            ..
        } => hash,
        other => panic!("unexpected prior close: {other:?}"),
    };
    drop(prior);

    let mut application = ApplicationCoordinator::launch(launch).expect("launch");
    application.run_reference_game(true).expect("run");
    let outcome = application
        .close(CloseExecutionOptionsV1::with_failure(
            FinalSaveAttemptFailureV1::Terminal(
                SchemaId::new("nextengine.test.disk-terminal").expect("code"),
            ),
            Some(last_safe),
        ))
        .expect("last-safe close");
    assert!(matches!(
        outcome,
        ApplicationCloseOutcomeV1::Closed {
            result: CloseSessionResultV1::ClosedUsingLastSafeGeneration,
            save_generation_hash: Some(hash),
            ..
        } if hash == last_safe
    ));
    cleanup(root);
}

#[test]
fn crash_after_save_publication_does_not_create_a_second_save_generation() {
    let root = test_root("save-publication-crash");
    let launch = headless_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application.run_reference_game(true).expect("run");
    application.inject_pause_after_save_commit();
    assert!(
        application
            .close(CloseExecutionOptionsV1::default())
            .is_err()
    );
    drop(application);

    let mut resumed = ApplicationCoordinator::resume(launch.clone()).expect("resume");
    let closed = resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("resume close");
    let first_hash = match closed {
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(hash),
            ..
        } => hash,
        other => panic!("unexpected close: {other:?}"),
    };
    drop(resumed);
    let mut terminal = ApplicationCoordinator::resume(launch).expect("terminal resume");
    let retried = terminal
        .close(CloseExecutionOptionsV1::default())
        .expect("terminal retry");
    assert!(matches!(
        retried,
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(hash),
            ..
        } if hash == first_hash
    ));
    cleanup(root);
}

#[test]
fn failed_close_registration_restores_every_publication_input_before_retry() {
    let root = test_root("close-registration-publication-rollback");
    let mut application = ApplicationCoordinator::launch(headless_launch(&root)).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let prepared_objects_before = application.prepared_run_objects.clone();
    let archived_requests_before = application.machine.archived_requests();
    let prepared_run_before = application
        .prepared_run
        .as_ref()
        .map(|prepared| prepared.summary.clone());

    application.inject_fail_next_publication();
    let error = application
        .close(CloseExecutionOptionsV1::default())
        .expect_err("injected registration publication failure");

    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);
    assert_eq!(application.prepared_run_objects, prepared_objects_before);
    assert_eq!(
        application.machine.archived_requests(),
        archived_requests_before
    );
    assert_eq!(
        application
            .prepared_run
            .as_ref()
            .map(|prepared| prepared.summary.clone()),
        prepared_run_before
    );

    let closed = application
        .close(CloseExecutionOptionsV1::default())
        .expect("retry after exact rollback");
    assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
    cleanup(root);
}

#[test]
fn stale_close_request_is_rejected_before_flushing_dirty_live_state() {
    let root = test_root("stale-close-before-live-flush");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live game");
    application
        .advance_reference_game_live(&[])
        .expect("advance dirty in-memory tick");
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("close request");
    let stale = CloseSessionRequestV1::new(
        request.close_request_id,
        request.session_id,
        request.starting_session_revision + 1,
        request.starting_session_state,
        request.shutdown_policy_hash,
        request.final_save_policy,
        request.bounded_deadline_class,
        request.reason,
        request.causal_input_reference,
    )
    .expect("structurally valid stale request");
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let prepared_objects_before = application.prepared_run_objects.clone();
    let archived_requests_before = application.machine.archived_requests();
    let live_before = application.current_live_run().expect("dirty live state");

    let error = application
        .close_with_request(stale, CloseExecutionOptionsV1::default())
        .expect_err("stale request must fail before the forced checkpoint");

    assert_eq!(error.diagnostic_code(), "SESSION_TRANSITION_INVALID");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);
    assert_eq!(application.prepared_run_objects, prepared_objects_before);
    assert_eq!(
        application.machine.archived_requests(),
        archived_requests_before
    );
    assert_eq!(
        application
            .current_live_run()
            .expect("unchanged live state"),
        live_before
    );
    cleanup(root);
}

#[test]
fn launch_or_resume_finishes_a_registered_close_before_starting_the_host() {
    let root = test_root("registered-close-restart");
    let launch = headless_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let prior_session_id = application.state().session_id;
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("close request");
    application
        .register_or_validate_close(&request, None)
        .expect("durable close registration");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Active
    );
    drop(application);

    let resumed =
        ApplicationCoordinator::launch_or_resume(launch).expect("finish close and relaunch");
    assert_eq!(resumed.state().state, ApplicationSessionStatusV1::Active);
    assert_ne!(resumed.state().session_id, prior_session_id);
    cleanup(root);
}

#[test]
fn failed_close_lifecycle_publication_keeps_registered_stage_for_exact_retry() {
    let root = test_root("close-lifecycle-publication-rollback");
    let mut application = ApplicationCoordinator::launch(headless_launch(&root)).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("close request");
    application
        .register_or_validate_close(&request, None)
        .expect("durable close registration");
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let archived_requests_before = application.machine.archived_requests();

    application.inject_fail_next_publication();
    let error = application
        .advance_to_finalizing(&request)
        .expect_err("injected lifecycle publication failure");

    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);
    assert_eq!(
        application.machine.archived_requests(),
        archived_requests_before
    );

    application
        .advance_to_finalizing(&request)
        .expect("retry advances only missing lifecycle stages");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Finalizing
    );
    let closed = application
        .close_with_request(request, CloseExecutionOptionsV1::default())
        .expect("finish exact close retry");
    assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
    cleanup(root);
}

#[test]
fn failed_final_save_session_publication_restores_reserved_close_state() {
    let root = test_root("final-save-publication-rollback");
    let mut application = ApplicationCoordinator::launch(headless_launch(&root)).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("close request");
    application
        .register_or_validate_close(&request, None)
        .expect("register close");
    application
        .advance_to_finalizing(&request)
        .expect("advance to finalizing");
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let prepared_objects_before = application.prepared_run_objects.clone();
    let archived_requests_before = application.machine.archived_requests();

    application.inject_fail_next_publication();
    let error = application
        .commit_final_save(&request)
        .expect_err("injected final-save session publication failure");

    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);
    assert_eq!(application.prepared_run_objects, prepared_objects_before);
    assert_eq!(
        application.machine.archived_requests(),
        archived_requests_before
    );

    let closed = application
        .close_with_request(request, CloseExecutionOptionsV1::default())
        .expect("retry reuses the already published save generation");
    assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
    cleanup(root);
}

#[test]
fn failed_save_attempt_publication_restores_the_prior_ledger_entry() {
    let root = test_root("save-attempt-publication-rollback");
    let mut application = ApplicationCoordinator::launch(headless_launch(&root)).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let request = application
        .close_request(BoundedDeadlineClassV1::Standard)
        .expect("close request");
    application
        .register_or_validate_close(&request, None)
        .expect("register close");
    application
        .advance_to_finalizing(&request)
        .expect("advance to finalizing");
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let failure = FinalSaveAttemptFailureV1::Retryable(
        SchemaId::new("nextengine.test.retryable-publication").expect("failure code"),
    );

    application.inject_fail_next_publication();
    let error = application
        .record_save_failure(&request, failure.clone())
        .expect_err("injected retry-ledger publication failure");

    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);

    let progress = application
        .record_save_failure(&request, failure)
        .expect("retry ledger publication");
    assert!(matches!(progress, ApplicationCloseOutcomeV1::Progress(_)));
    let closed = application
        .close_with_request(request, CloseExecutionOptionsV1::default())
        .expect("finish close");
    assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
    cleanup(root);
}
