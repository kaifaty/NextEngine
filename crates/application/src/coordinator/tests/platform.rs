use super::*;

#[test]
fn platform_suspend_resume_preserve_runtime_revision_and_retry_exactly() {
    let root = test_root("platform-suspend-resume");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    let run = application
        .run_reference_game(true)
        .expect("reference game");
    let runtime_revision = Some(run.authoritative_revision);
    assert_eq!(
        application.state().active_runtime_revision,
        runtime_revision
    );
    let suspend = platform_reason_event(
        &mut application,
        PlatformEventKindV1::SuspendRequested,
        10,
        "nextengine.platform.reason.backgrounded",
    );
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let prepared_objects_before = application.prepared_run_objects.clone();

    application.inject_fail_next_publication();
    let error = application
        .suspend_from_platform_event(&suspend)
        .expect_err("injected suspend publication failure");
    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);
    assert_eq!(application.prepared_run_objects, prepared_objects_before);

    let suspended = application
        .suspend_from_platform_event(&suspend)
        .expect("suspend retry");
    assert_eq!(suspended.from_state, ApplicationSessionStatusV1::Active);
    assert_eq!(suspended.to_state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(
        application.state().active_runtime_revision,
        runtime_revision
    );
    let suspended_generation = application.current_generation;
    let suspended_durable = application.durable.clone();
    let exact_retry = application
        .suspend_from_platform_event(&suspend)
        .expect("completed suspend exact retry");
    assert_eq!(exact_retry, suspended);
    assert_eq!(application.current_generation, suspended_generation);
    assert_eq!(application.durable, suspended_durable);

    let resume = platform_reason_event(
        &mut application,
        PlatformEventKindV1::ResumeRequested,
        11,
        "nextengine.platform.reason.foregrounded",
    );
    let resumed = application
        .resume_from_platform_event(&resume)
        .expect("resume");
    assert_eq!(resumed.from_state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(resumed.to_state, ApplicationSessionStatusV1::Active);
    assert_eq!(
        application.state().active_runtime_revision,
        runtime_revision
    );
    application
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}
#[test]
fn platform_launch_requires_capabilities_that_match_the_selected_target() {
    let missing_root = test_root("platform-capabilities-missing");
    let mut missing = interactive_launch(&missing_root);
    missing.platform_capability_set = None;
    let missing_error = match ApplicationCoordinator::launch(missing) {
        Err(error) => error,
        Ok(_) => panic!("interactive launch without capabilities must fail closed"),
    };
    assert_eq!(
        missing_error.diagnostic_code(),
        "PLATFORM_CAPABILITY_REQUIRED"
    );

    let forbidden_root = test_root("platform-capabilities-headless");
    let interactive_capabilities = interactive_launch(&forbidden_root)
        .platform_capability_set
        .expect("interactive fixture capabilities");
    let mut forbidden = headless_launch(&forbidden_root);
    forbidden.platform_capability_set = Some(interactive_capabilities);
    let forbidden_error = match ApplicationCoordinator::launch(forbidden) {
        Err(error) => error,
        Ok(_) => panic!("headless launch must not bind a platform capability set"),
    };
    assert_eq!(
        forbidden_error.diagnostic_code(),
        "PLATFORM_CAPABILITY_REQUIRED"
    );

    cleanup(missing_root);
    cleanup(forbidden_root);
}

#[test]
fn platform_host_binding_rejects_unregistered_and_stale_events_without_mutation() {
    let root = test_root("platform-host-binding");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    let capabilities = application
        .launch
        .platform_capability_set
        .clone()
        .expect("interactive capabilities");
    let unregistered = platform_reason_event_with_identity(
        PersistentId::from_bytes([0x41; 16]),
        capabilities.canonical_hash,
        PlatformEventKindV1::SuspendRequested,
        1,
        "nextengine.platform.reason.unregistered",
    );
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let error = application
        .suspend_from_platform_event(&unregistered)
        .expect_err("unregistered event must fail closed");
    assert_eq!(error.diagnostic_code(), "PLATFORM_CAPABILITY_REQUIRED");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);

    let first_host = application
        .register_platform_host(&capabilities)
        .expect("register first host lifetime");
    let first_event = platform_reason_event_with_identity(
        first_host,
        capabilities.canonical_hash,
        PlatformEventKindV1::SuspendRequested,
        2,
        "nextengine.platform.reason.first-host",
    );
    let second_host = application
        .register_platform_host(&capabilities)
        .expect("register replacement host lifetime");
    assert_ne!(first_host, second_host);

    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let stale_error = application
        .suspend_from_platform_event(&first_event)
        .expect_err("event from replaced host must fail closed");
    assert_eq!(
        stale_error.diagnostic_code(),
        "PLATFORM_EVENT_IDENTITY_COLLISION"
    );
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);

    let wrong_capability = platform_reason_event_with_identity(
        second_host,
        ContentHash::from_bytes([0x52; 32]),
        PlatformEventKindV1::SuspendRequested,
        3,
        "nextengine.platform.reason.wrong-capability",
    );
    let capability_error = application
        .suspend_from_platform_event(&wrong_capability)
        .expect_err("event with another capability hash must fail closed");
    assert_eq!(
        capability_error.diagnostic_code(),
        "PLATFORM_EVENT_IDENTITY_COLLISION"
    );
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);

    let current_event = platform_reason_event_with_identity(
        second_host,
        capabilities.canonical_hash,
        PlatformEventKindV1::SuspendRequested,
        4,
        "nextengine.platform.reason.current-host",
    );
    application
        .suspend_from_platform_event(&current_event)
        .expect("current host event");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Suspended
    );
    cleanup(root);
}

#[test]
fn headless_application_rejects_platform_events_without_mutation() {
    let root = test_root("platform-host-headless");
    let mut application =
        ApplicationCoordinator::launch(headless_launch(&root)).expect("headless launch");
    let event = platform_reason_event_with_identity(
        PersistentId::from_bytes([0x61; 16]),
        ContentHash::from_bytes([0x62; 32]),
        PlatformEventKindV1::SuspendRequested,
        1,
        "nextengine.platform.reason.headless",
    );
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let objects_before = application.objects.clone();
    let error = application
        .suspend_from_platform_event(&event)
        .expect_err("headless session has no platform ingress");
    assert_eq!(
        error.diagnostic_code(),
        "PLATFORM_FORBIDDEN_PRESENTATION_TARGET"
    );
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_eq!(application.objects, objects_before);
    cleanup(root);
}

#[test]
fn platform_suspend_archive_survives_restart_and_keeps_the_forced_live_checkpoint() {
    let root = test_root("platform-suspend-restart");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live game");
    let mut before_suspend = application.current_live_run().expect("initial live state");
    for _ in 0..5 {
        before_suspend = application
            .advance_reference_game_live(&[])
            .expect("advance in-memory state");
    }
    assert_eq!(before_suspend.ticks, 5);
    let suspend = platform_reason_event(
        &mut application,
        PlatformEventKindV1::SuspendRequested,
        40,
        "nextengine.platform.reason.backgrounded",
    );
    let suspended = application
        .suspend_from_platform_event(&suspend)
        .expect("atomic forced suspend");
    let suspended_generation = application.current_generation;
    drop(application);

    let mut resumed = ApplicationCoordinator::resume(launch).expect("resume suspended generation");
    assert_eq!(resumed.state().state, ApplicationSessionStatusV1::Suspended);
    let recovered = resumed.current_live_run().expect("forced checkpoint");
    assert_run_roots_equal(&recovered, &before_suspend);
    let before_presentation = before_suspend
        .presentation_snapshot
        .as_ref()
        .expect("persisted presentation");
    let recovered_presentation = recovered
        .presentation_snapshot
        .as_ref()
        .expect("recovery-cut presentation");
    assert_ne!(
        recovered_presentation.snapshot_epoch,
        before_presentation.snapshot_epoch
    );
    assert_eq!(recovered_presentation.snapshot_sequence, 0);
    let exact = resumed
        .suspend_from_platform_event(&suspend)
        .expect("restart exact retry");
    assert_eq!(exact, suspended);
    assert_eq!(resumed.current_generation, suspended_generation);

    let resume = platform_reason_event(
        &mut resumed,
        PlatformEventKindV1::ResumeRequested,
        41,
        "nextengine.platform.reason.foregrounded",
    );
    let resumed_event = resumed
        .resume_from_platform_event(&resume)
        .expect("resume active state");
    let active_generation = resumed.current_generation;
    drop(resumed);

    let mut active = ApplicationCoordinator::resume(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("restart active generation");
    let exact_resume = active
        .resume_from_platform_event(&resume)
        .expect("restart resume exact retry");
    assert_eq!(exact_resume, resumed_event);
    assert_eq!(active.current_generation, active_generation);
    active
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}

#[test]
fn lifecycle_archive_recovery_rejects_event_bytes_that_do_not_match_the_request() {
    let root = test_root("platform-lifecycle-archive-tamper");
    let launch = interactive_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    let suspend = platform_reason_event(
        &mut application,
        PlatformEventKindV1::SuspendRequested,
        42,
        "nextengine.platform.reason.backgrounded",
    );
    let event = application
        .suspend_from_platform_event(&suspend)
        .expect("suspend");
    let published = application
        .session_store
        .load_current()
        .expect("current suspended generation");
    let archive_entry = application
        .durable
        .lifecycle_archive
        .iter()
        .find(|entry| entry.request_id == event.request_id)
        .expect("suspend archive entry")
        .clone();
    let original_event_bytes = published
        .objects
        .get(&archive_entry.event_object_hash)
        .expect("archived event bytes");
    let tampered_event_bytes = String::from_utf8(original_event_bytes.to_vec())
        .expect("event JCS is utf8")
        .replace("\"to_state\":\"Suspended\"", "\"to_state\":\"Active\"")
        .into_bytes();
    assert_ne!(
        tampered_event_bytes.as_slice(),
        original_event_bytes.as_ref()
    );
    let tampered_event = SessionObjectV1::new(tampered_event_bytes);
    let mut durable = application.durable.clone();
    durable.store_sequence = published.sequence + 1;
    durable
        .lifecycle_archive
        .iter_mut()
        .find(|entry| entry.request_id == event.request_id)
        .expect("mutable suspend archive entry")
        .event_object_hash = tampered_event.content_hash();
    let mut objects: Vec<_> = published
        .objects
        .into_values()
        .map(SessionObjectV1::new)
        .collect();
    objects.push(tampered_event);
    let publication = SessionPublicationV1::new(
        durable.store_sequence,
        durable.manifest.body.project_composition_lock_hash,
        Some(durable.state.session_id),
        Some(published.generation_id),
        None,
        durable
            .canonical_bytes()
            .expect("tampered archive snapshot"),
        objects,
    )
    .expect("tampered archive publication");
    application
        .session_store
        .publish(&publication)
        .expect("publish structurally valid tampered generation");
    drop(application);

    assert!(ApplicationCoordinator::resume(launch).is_err());
    cleanup(root);
}

#[test]
fn platform_lifecycle_budget_survives_restart_and_reserves_close_capacity() {
    let root = test_root("platform-lifecycle-budget");
    let launch = interactive_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live game");
    let mut machine = application.machine.clone();
    let mut first_suspend = None;

    for index in 0..super::super::publication::PLATFORM_LIFECYCLE_REQUEST_BUDGET {
        let index = u64::try_from(index).expect("budget fits u64");
        let (kind, target, reason_kind, reason_code) = if index % 2 == 0 {
            (
                PlatformEventKindV1::SuspendRequested,
                ApplicationSessionStatusV1::Suspended,
                LifecycleReasonKindV1::SuspendRequested,
                "nextengine.session.suspend-requested",
            )
        } else {
            (
                PlatformEventKindV1::ResumeRequested,
                ApplicationSessionStatusV1::Active,
                LifecycleReasonKindV1::ResumeRequested,
                "nextengine.session.resume-requested",
            )
        };
        let event = platform_reason_event(
            &mut application,
            kind,
            1_000 + index,
            "nextengine.platform.reason.budget-test",
        );
        if first_suspend.is_none() {
            first_suspend = Some(event.clone());
        }
        let state = machine.state().clone();
        let request_id = super::super::identity::derive_request_id(
            state.session_id,
            state.revision,
            target,
            event.platform_event_id,
        );
        let request = ApplicationLifecycleRequestV1::new(
            request_id,
            state.session_id,
            state.revision,
            state.state,
            target,
            LifecycleReasonV1 {
                kind: reason_kind,
                reason_code: SchemaId::new(reason_code).expect("reason"),
            },
            application
                .activated_project
                .composition_lock
                .recovery_policy_sha256,
            CausalInputReferenceV1 {
                source_kind: CausalInputSourceKindV1::PlatformEvent,
                canonical_hash: event.platform_event_id,
            },
        )
        .expect("bounded platform lifecycle request");
        let plan = machine
            .plan_transition(
                request.clone(),
                SessionTransitionReferencesV1 {
                    active_runtime_revision: state.active_runtime_revision,
                    ..SessionTransitionReferencesV1::default()
                },
            )
            .expect("bounded platform lifecycle plan");
        let committed = machine.commit(plan);
        application
            .record_lifecycle_archive_entry(
                request.request_id,
                request.canonical_bytes(),
                committed.canonical_bytes(),
            )
            .expect("stage durable lifecycle archive");
    }
    assert_eq!(machine.state().state, ApplicationSessionStatusV1::Active);
    application.durable.state = machine.state().clone();
    application.machine = machine;
    let prior_generation = application.current_generation;
    let session_id = application.state().session_id;
    application
        .publish_current(Some(prior_generation), Some(session_id), None)
        .expect("publish budget boundary");
    drop(application);

    let mut resumed = ApplicationCoordinator::resume(launch).expect("restore full archive budget");
    let first_suspend = first_suspend.expect("first suspend event");
    let generation_at_budget = resumed.current_generation;
    resumed
        .suspend_from_platform_event(&first_suspend)
        .expect("exact retry does not consume budget");
    assert_eq!(resumed.current_generation, generation_at_budget);

    let overflow = platform_reason_event(
        &mut resumed,
        PlatformEventKindV1::SuspendRequested,
        9_999,
        "nextengine.platform.reason.budget-overflow",
    );
    let error = resumed
        .suspend_from_platform_event(&overflow)
        .expect_err("N + 1 platform lifecycle request");
    assert_eq!(
        error.diagnostic_code(),
        "SESSION_PLATFORM_LIFECYCLE_BUDGET_EXCEEDED"
    );
    assert_eq!(resumed.current_generation, generation_at_budget);

    let closed = resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("reserved object capacity keeps close available");
    assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
    cleanup(root);
}

#[test]
fn platform_close_cause_survives_save_crash_and_restart_continuation() {
    let root = test_root("platform-close-crash-restart");
    let launch = interactive_launch(&root);
    let mut application = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    application
        .run_reference_game(true)
        .expect("reference game");
    let close_event = platform_reason_event(
        &mut application,
        PlatformEventKindV1::CloseRequested,
        12,
        "nextengine.platform.reason.window-close",
    );
    let request = application
        .close_request_from_platform_event(&close_event, BoundedDeadlineClassV1::Standard)
        .expect("platform close request");
    assert_eq!(
        request.reason.kind,
        LifecycleReasonKindV1::HostCloseRequested
    );
    assert_eq!(
        request.reason.reason_code.as_str(),
        "nextengine.session.host-close-requested"
    );
    assert_eq!(
        request.causal_input_reference.source_kind,
        CausalInputSourceKindV1::PlatformEvent
    );
    assert_eq!(
        request.causal_input_reference.canonical_hash,
        close_event.platform_event_id
    );

    application.inject_pause_after_save_commit();
    let error = application
        .close_from_platform_event(&close_event, CloseExecutionOptionsV1::default())
        .expect_err("pause after save publication");
    assert_eq!(error.diagnostic_code(), "SESSION_FINAL_SAVE_FAILED");
    let quiesce_request = application
        .machine
        .archived_requests()
        .into_iter()
        .find(|archived| archived.event.to_state == ApplicationSessionStatusV1::Quiescing)
        .expect("quiesce archive");
    let quiesce_request = ApplicationLifecycleRequestV1::from_jcs_bytes(
        &quiesce_request.canonical_request_bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .expect("decode quiesce request");
    assert_eq!(quiesce_request.reason, request.reason);
    assert_eq!(
        quiesce_request.causal_input_reference,
        request.causal_input_reference
    );
    drop(application);

    let mut resumed = ApplicationCoordinator::resume(launch.clone()).expect("resume close");
    let closed = resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("continue archived platform close without the native event");
    let (receipt_hash, generation_hash) = match closed {
        ApplicationCloseOutcomeV1::Closed {
            receipt_hash,
            save_generation_hash: Some(generation_hash),
            ..
        } => (receipt_hash, generation_hash),
        other => panic!("unexpected close result: {other:?}"),
    };
    drop(resumed);

    let mut terminal = ApplicationCoordinator::resume(launch).expect("resume terminal");
    let retried = terminal
        .close(CloseExecutionOptionsV1::default())
        .expect("terminal exact retry");
    assert_eq!(retried.receipt_hash(), Some(receipt_hash));
    assert!(matches!(
        retried,
        ApplicationCloseOutcomeV1::Closed {
            save_generation_hash: Some(actual),
            ..
        } if actual == generation_hash
    ));
    cleanup(root);
}

#[test]
fn lifecycle_source_gap_is_rejected_before_resume_and_exact_next_sequence_still_works() {
    let root = test_root("platform-lifecycle-source-gap");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    application
        .begin_reference_game_live(true)
        .expect("begin live run");
    let suspend = platform_reason_event(
        &mut application,
        PlatformEventKindV1::SuspendRequested,
        0,
        "nextengine.platform.reason.sequence-suspend",
    );
    application
        .suspend_from_platform_event(&suspend)
        .expect("suspend sequence zero");

    let binding = test_platform_host(&mut application);
    let gap = platform_reason_event_with_identity(
        binding.host_instance_id,
        binding.capability_set_hash,
        PlatformEventKindV1::ResumeRequested,
        2,
        "nextengine.platform.reason.sequence-gap",
    );
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let error = application
        .resume_from_platform_event(&gap)
        .expect_err("sequence gap must fail before resume");
    assert_eq!(error.diagnostic_code(), "PLATFORM_EVENT_SEQUENCE_GAP");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);

    let next = platform_reason_event_with_identity(
        binding.host_instance_id,
        binding.capability_set_hash,
        PlatformEventKindV1::ResumeRequested,
        1,
        "nextengine.platform.reason.sequence-next",
    );
    application
        .resume_from_platform_event(&next)
        .expect("exact next sequence resumes");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Active
    );
    cleanup(root);
}

#[test]
fn platform_source_limit_is_a_schema_error_and_does_not_mutate_the_scheduler() {
    let root = test_root("platform-source-limit");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    let initial = application
        .begin_reference_game_live(true)
        .expect("begin live run");
    let binding = test_platform_host(&mut application);
    let source_limit = super::super::platform_host::MAXIMUM_PLATFORM_EVENT_SOURCES;
    let admitted = (0..source_limit)
        .map(|index| {
            PlatformEventV1::new(
                binding.host_instance_id,
                SchemaId::new(format!("nextengine.platform.source.limit{index}"))
                    .expect("source class"),
                0,
                u64::try_from(index).expect("sample tick"),
                PlatformEventKindV1::FocusChanged,
                PlatformEventPayloadV1::FocusChanged { focused: false },
                binding.capability_set_hash,
            )
            .expect("bounded source event")
        })
        .collect::<Vec<_>>();
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
    scheduler
        .advance_reference_game(&mut application, Duration::ZERO, &admitted)
        .expect("exact source limit");
    let pending_before = scheduler.pending_events();
    let overflow = PlatformEventV1::new(
        binding.host_instance_id,
        SchemaId::new("nextengine.platform.source.limit-overflow").expect("overflow source"),
        0,
        u64::try_from(source_limit).expect("sample tick"),
        PlatformEventKindV1::FocusChanged,
        PlatformEventPayloadV1::FocusChanged { focused: true },
        binding.capability_set_hash,
    )
    .expect("overflow source event");

    let error = scheduler
        .advance_reference_game(&mut application, Duration::ZERO, &[overflow])
        .expect_err("257th source must fail before scheduler mutation");
    assert_eq!(error.diagnostic_code(), "PLATFORM_EVENT_SCHEMA_INVALID");
    assert_eq!(scheduler.pending_events(), pending_before);
    assert_run_roots_equal(
        &application.current_live_run().expect("unchanged live run"),
        &initial,
    );
    application
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}

#[test]
fn illegal_lifecycle_batch_is_rejected_before_tick_session_or_scheduler_mutation() {
    let root = test_root("platform-lifecycle-batch-preflight");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    let initial = application
        .begin_reference_game_live(true)
        .expect("begin live run");
    let binding = test_platform_host(&mut application);
    let first = platform_reason_event_with_identity(
        binding.host_instance_id,
        binding.capability_set_hash,
        PlatformEventKindV1::SuspendRequested,
        0,
        "nextengine.platform.reason.first-suspend",
    );
    let second = platform_reason_event_with_identity(
        binding.host_instance_id,
        binding.capability_set_hash,
        PlatformEventKindV1::SuspendRequested,
        1,
        "nextengine.platform.reason.second-suspend",
    );
    let state_before = application.state().clone();
    let durable_before = application.durable.clone();
    let generation_before = application.current_generation;
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();

    let error = scheduler
        .advance_reference_game(
            &mut application,
            Duration::from_millis(34),
            &[first.clone(), second],
        )
        .expect_err("second suspend is illegal in one lifecycle plan");
    assert_eq!(error.diagnostic_code(), "SESSION_TRANSITION_INVALID");
    assert_eq!(application.state(), &state_before);
    assert_eq!(application.durable, durable_before);
    assert_eq!(application.current_generation, generation_before);
    assert_run_roots_equal(
        &application.current_live_run().expect("unchanged live run"),
        &initial,
    );
    assert_eq!(scheduler.pending_event_count(), 0);
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 0);

    assert!(
        scheduler
            .advance_reference_game(&mut application, Duration::ZERO, &[first])
            .expect("valid first suspend is still admissible")
            .is_none()
    );
    scheduler
        .advance_reference_game(&mut application, Duration::from_millis(34), &[])
        .expect("valid suspend reaches the next fixed boundary");
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Suspended
    );
    cleanup(root);
}

#[test]
fn scheduler_executes_every_event_in_a_valid_multi_cycle_lifecycle_batch() {
    let root = test_root("platform-lifecycle-multi-cycle");
    let mut application =
        ApplicationCoordinator::launch(interactive_launch(&root)).expect("launch");
    let initial = application
        .begin_reference_game_live(true)
        .expect("begin live run");
    let archived_before = application.machine.archived_requests().len();
    let binding = test_platform_host(&mut application);
    let batch = [
        platform_reason_event_with_identity(
            binding.host_instance_id,
            binding.capability_set_hash,
            PlatformEventKindV1::SuspendRequested,
            0,
            "nextengine.platform.reason.multi-suspend-zero",
        ),
        platform_reason_event_with_identity(
            binding.host_instance_id,
            binding.capability_set_hash,
            PlatformEventKindV1::ResumeRequested,
            1,
            "nextengine.platform.reason.multi-resume-one",
        ),
        platform_reason_event_with_identity(
            binding.host_instance_id,
            binding.capability_set_hash,
            PlatformEventKindV1::SuspendRequested,
            2,
            "nextengine.platform.reason.multi-suspend-two",
        ),
        platform_reason_event_with_identity(
            binding.host_instance_id,
            binding.capability_set_hash,
            PlatformEventKindV1::ResumeRequested,
            3,
            "nextengine.platform.reason.multi-resume-three",
        ),
    ];

    let direct_error = application
        .advance_reference_game_live(&batch)
        .expect_err("direct one-tick API rejects a multi-lifecycle batch");
    assert_eq!(direct_error.diagnostic_code(), "SESSION_TRANSITION_INVALID");
    assert_run_roots_equal(
        &application.current_live_run().expect("unchanged live run"),
        &initial,
    );

    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
    scheduler
        .advance_reference_game(&mut application, Duration::ZERO, &batch)
        .expect("the same batch remains admissible through the scheduler");
    let advanced = scheduler
        .advance_reference_game(&mut application, Duration::from_millis(34), &[])
        .expect("execute the complete lifecycle batch")
        .expect("one fixed tick");

    assert_eq!(advanced.ticks, 1);
    assert_eq!(
        application.state().state,
        ApplicationSessionStatusV1::Active
    );
    assert_eq!(
        application.machine.archived_requests().len(),
        archived_before + batch.len()
    );
    assert_eq!(scheduler.pending_event_count(), 0);
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 0);
    let next = platform_reason_event_with_identity(
        binding.host_instance_id,
        binding.capability_set_hash,
        PlatformEventKindV1::SuspendRequested,
        4,
        "nextengine.platform.reason.multi-next",
    );
    application
        .suspend_from_platform_event(&next)
        .expect("source cursor remains at the complete batch boundary");
    application
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}
