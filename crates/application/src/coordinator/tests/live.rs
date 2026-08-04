use super::*;

#[test]
fn game_and_headless_share_authoritative_hashes_and_headless_has_no_presentation() {
    let root = test_root("root-parity");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        root.join("game"),
        CompositionRootV1::Game,
        PresentationTargetKindV1::None,
    ))
    .expect("game launch");
    let game_run = game.run_reference_game(true).expect("game run");
    game.close(CloseExecutionOptionsV1::default())
        .expect("game close");

    let mut headless = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        root.join("headless"),
        CompositionRootV1::Headless,
        PresentationTargetKindV1::None,
    ))
    .expect("headless launch");
    let headless_run = headless.run_reference_game(true).expect("headless run");
    headless
        .close(CloseExecutionOptionsV1::default())
        .expect("headless close");

    assert_eq!(
        game_run.authoritative_state_root,
        headless_run.authoritative_state_root
    );
    assert_eq!(
        game_run.command_archive_root,
        headless_run.command_archive_root
    );
    assert_eq!(
        game_run.command_identity_index_root,
        headless_run.command_identity_index_root
    );
    assert!(headless_run.presentation_snapshot.is_none());
    cleanup(root);
}
#[test]
fn live_game_advances_runtime_and_replaces_the_typed_camera_snapshot() {
    let root = test_root("live-game");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    let initial = game
        .begin_reference_game_live(true)
        .expect("begin live reference game");
    assert_eq!(initial.ticks, 0);
    let initial_snapshot = initial
        .presentation_snapshot
        .expect("interactive camera snapshot");
    assert_eq!(initial_snapshot.camera_records().count(), 1);

    let advanced = game
        .advance_reference_game_live(&[])
        .expect("advance one logical frame");
    assert_eq!(advanced.ticks, 1);
    let advanced_snapshot = advanced
        .presentation_snapshot
        .as_ref()
        .expect("advanced presentation");
    assert_eq!(advanced_snapshot.camera_records().count(), 1);
    assert!(advanced_snapshot.snapshot_sequence > initial_snapshot.snapshot_sequence);
    assert_eq!(
        game.current_live_run().expect("published live state"),
        advanced
    );
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn live_restart_restores_exact_generation_and_continues_input_without_revision_collision() {
    let root = test_root("live-restart-exact");
    let control_root = test_root("live-restart-control");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let mut restarted = ApplicationCoordinator::launch(launch.clone()).expect("restart launch");
    let mut uninterrupted = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &control_root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("control launch");
    restarted
        .begin_reference_game_live(true)
        .expect("restart live run");
    uninterrupted
        .begin_reference_game_live(true)
        .expect("control live run");

    let restarted_started = keyboard_movement_event_at(
        &mut restarted,
        0,
        NormalizedControlPhaseV1::Started,
        i16::MAX,
    );
    let uninterrupted_started = keyboard_movement_event_at(
        &mut uninterrupted,
        0,
        NormalizedControlPhaseV1::Started,
        i16::MAX,
    );
    let mut before_restart = restarted
        .advance_reference_game_live(std::slice::from_ref(&restarted_started))
        .expect("advance held input");
    let mut control_before_restart = uninterrupted
        .advance_reference_game_live(std::slice::from_ref(&uninterrupted_started))
        .expect("advance control held input");
    for _ in 1..30 {
        before_restart = restarted
            .advance_reference_game_live(&[])
            .expect("reach durable checkpoint");
        control_before_restart = uninterrupted
            .advance_reference_game_live(&[])
            .expect("reach control checkpoint");
    }
    assert_eq!(before_restart.ticks, 30);
    assert_eq!(
        before_restart.authoritative_state_root,
        control_before_restart.authoritative_state_root
    );
    let before_presentation = before_restart
        .presentation_snapshot
        .as_ref()
        .expect("presentation before restart")
        .clone();
    let persisted_recovery_manifest_hash = restarted
        .durable
        .live_run_recovery_manifest_hash
        .expect("persisted recovery manifest");
    assert_eq!(restarted.prepared_run_objects.len(), 7);
    drop(restarted);

    let restarted = ApplicationCoordinator::resume(launch.clone()).expect("resume live generation");
    let recovered = restarted.current_live_run().expect("restored run");
    assert_run_roots_equal(&recovered, &before_restart);
    let recovered_presentation = recovered
        .presentation_snapshot
        .as_ref()
        .expect("recovery-cut presentation");
    assert_ne!(
        recovered_presentation.snapshot_epoch,
        before_presentation.snapshot_epoch
    );
    assert_eq!(recovered_presentation.snapshot_sequence, 0);
    assert!(
        recovered_presentation
            .camera_records()
            .all(|camera| camera.cut)
    );
    let recovery_cut_manifest_hash = restarted
        .durable
        .live_run_recovery_manifest_hash
        .expect("recovery-cut manifest");
    assert_ne!(recovery_cut_manifest_hash, persisted_recovery_manifest_hash);
    let disk_after_recovery = SessionStore::new(root.join("sessions"))
        .load_current()
        .expect("persisted pre-cut generation");
    let disk_durable = crate::durable::DurableApplicationSnapshotV1::from_canonical_bytes(
        &disk_after_recovery.snapshot,
    )
    .expect("persisted durable snapshot");
    assert_eq!(
        disk_durable.live_run_recovery_manifest_hash,
        Some(persisted_recovery_manifest_hash)
    );
    drop(restarted);

    let mut restarted =
        ApplicationCoordinator::resume(launch).expect("repeat the uncommitted recovery cut");
    assert_eq!(
        restarted.current_live_run().expect("repeated recovery cut"),
        recovered
    );
    assert_eq!(
        restarted.durable.live_run_recovery_manifest_hash,
        Some(recovery_cut_manifest_hash)
    );
    assert_eq!(
        restarted
            .begin_reference_game_live(true)
            .expect_err("restored driver is already active")
            .diagnostic_code(),
        "SESSION_RUNTIME_ALREADY_ACTIVE"
    );
    assert_eq!(restarted.prepared_run_objects.len(), 7);

    let restarted_changed_event = keyboard_movement_event_at(
        &mut restarted,
        1,
        NormalizedControlPhaseV1::Changed,
        i16::MAX,
    );
    let uninterrupted_changed_event = keyboard_movement_event_at(
        &mut uninterrupted,
        1,
        NormalizedControlPhaseV1::Changed,
        i16::MAX,
    );
    let restarted_changed = restarted
        .advance_reference_game_live(std::slice::from_ref(&restarted_changed_event))
        .expect("continued source after restart");
    let control_changed = uninterrupted
        .advance_reference_game_live(std::slice::from_ref(&uninterrupted_changed_event))
        .expect("continued source uninterrupted");
    assert_run_roots_equal(&restarted_changed, &control_changed);
    let changed_presentation = restarted_changed
        .presentation_snapshot
        .as_ref()
        .expect("presentation after restart");
    assert_eq!(
        changed_presentation.snapshot_epoch,
        recovered_presentation.snapshot_epoch
    );
    assert_eq!(
        changed_presentation.snapshot_sequence,
        recovered_presentation.snapshot_sequence + 1
    );

    let restarted_completed_event =
        keyboard_movement_event_at(&mut restarted, 2, NormalizedControlPhaseV1::Completed, 0);
    let uninterrupted_completed_event = keyboard_movement_event_at(
        &mut uninterrupted,
        2,
        NormalizedControlPhaseV1::Completed,
        0,
    );
    let restarted_completed = restarted
        .advance_reference_game_live(std::slice::from_ref(&restarted_completed_event))
        .expect("completed source after restart");
    let control_completed = uninterrupted
        .advance_reference_game_live(std::slice::from_ref(&uninterrupted_completed_event))
        .expect("completed source uninterrupted");
    assert_run_roots_equal(&restarted_completed, &control_completed);
    assert_eq!(restarted_completed.ticks, 32);

    restarted
        .close(CloseExecutionOptionsV1::default())
        .expect("restart close");
    uninterrupted
        .close(CloseExecutionOptionsV1::default())
        .expect("control close");
    cleanup(root);
    cleanup(control_root);
}

#[test]
fn scripted_and_live_drivers_cannot_replace_each_other_inside_one_session() {
    let live_root = test_root("live-scripted-exclusion");
    let mut live = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &live_root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("live launch");
    live.begin_reference_game_live(true).expect("begin live");
    assert_eq!(
        live.run_reference_game(true)
            .expect_err("scripted run must not replace live state")
            .diagnostic_code(),
        "SESSION_RUNTIME_ALREADY_ACTIVE"
    );
    live.close(CloseExecutionOptionsV1::default())
        .expect("live close");
    cleanup(live_root);

    let scripted_root = test_root("scripted-live-exclusion");
    let mut scripted = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &scripted_root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("scripted launch");
    scripted
        .run_reference_game(true)
        .expect("scripted reference run");
    assert_eq!(
        scripted
            .begin_reference_game_live(true)
            .expect_err("live driver must not replace scripted state")
            .diagnostic_code(),
        "SESSION_RUNTIME_ALREADY_ACTIVE"
    );
    scripted
        .close(CloseExecutionOptionsV1::default())
        .expect("scripted close");
    cleanup(scripted_root);
}

#[test]
fn presentation_epoch_changes_with_a_new_application_activation() {
    let root = test_root("presentation-epoch");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let mut first = ApplicationCoordinator::launch(launch.clone()).expect("first launch");
    let first_epoch = first
        .begin_reference_game_live(true)
        .expect("first live run")
        .presentation_snapshot
        .expect("first presentation")
        .snapshot_epoch;
    first
        .close(CloseExecutionOptionsV1::default())
        .expect("first close");

    let mut second = ApplicationCoordinator::launch(launch).expect("second launch");
    let second_epoch = second
        .begin_reference_game_live(true)
        .expect("second live run")
        .presentation_snapshot
        .expect("second presentation")
        .snapshot_epoch;
    assert_ne!(first_epoch, second_epoch);
    second
        .close(CloseExecutionOptionsV1::default())
        .expect("second close");
    cleanup(root);
}

#[test]
fn failed_live_publication_keeps_the_exact_driver_generation_for_retry() {
    let root = test_root("live-publication-rollback");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    let initial = game
        .begin_reference_game_live(true)
        .expect("begin live reference game");
    let initial_prepared_run_objects = game.prepared_run_objects.clone();
    let mut before_checkpoint = initial.clone();
    for _ in 0..29 {
        before_checkpoint = game
            .advance_reference_game_live(&[])
            .expect("advance in-memory live state");
    }
    assert_eq!(before_checkpoint.ticks, 29);

    game.inject_fail_next_state_publication();
    let error = game
        .advance_reference_game_live(&[])
        .expect_err("injected publication failure");
    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(
        game.current_live_run().expect("prior published generation"),
        before_checkpoint
    );
    assert_eq!(
        game.prepared_run_objects, initial_prepared_run_objects,
        "failed publication must restore the prior checkpoint object closure"
    );

    let retried = game
        .advance_reference_game_live(&[])
        .expect("exact staged retry");
    assert_eq!(retried.ticks, 30);
    assert_ne!(
        retried.authoritative_state_root,
        initial.authoritative_state_root
    );
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn live_checkpoint_cadence_publishes_only_ticks_zero_thirty_and_sixty() {
    let root = test_root("live-checkpoint-cadence");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let generation_at_zero = game.current_generation;
    let sequence_at_zero = game.durable.store_sequence;

    for _ in 0..29 {
        game.advance_reference_game_live(&[])
            .expect("advance through tick 29");
    }
    assert_eq!(game.current_live_run().expect("tick 29").ticks, 29);
    assert_eq!(game.current_generation, generation_at_zero);
    assert_eq!(game.durable.store_sequence, sequence_at_zero);

    game.advance_reference_game_live(&[])
        .expect("publish tick 30");
    let generation_at_thirty = game.current_generation;
    assert_ne!(generation_at_thirty, generation_at_zero);
    assert_eq!(game.durable.store_sequence, sequence_at_zero + 1);

    for _ in 0..29 {
        game.advance_reference_game_live(&[])
            .expect("advance through tick 59");
    }
    assert_eq!(game.current_live_run().expect("tick 59").ticks, 59);
    assert_eq!(game.current_generation, generation_at_thirty);
    assert_eq!(game.durable.store_sequence, sequence_at_zero + 1);

    game.advance_reference_game_live(&[])
        .expect("publish tick 60");
    assert_eq!(game.current_live_run().expect("tick 60").ticks, 60);
    assert_ne!(game.current_generation, generation_at_thirty);
    assert_eq!(game.durable.store_sequence, sequence_at_zero + 2);
    game.close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}

#[test]
fn crash_at_tick_twenty_nine_recovers_tick_zero() {
    let root = test_root("live-cadence-crash-29");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let mut game = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    let tick_zero = game
        .begin_reference_game_live(true)
        .expect("begin live reference game");
    for _ in 0..29 {
        game.advance_reference_game_live(&[])
            .expect("advance through tick 29");
    }
    assert_eq!(game.current_live_run().expect("tick 29").ticks, 29);
    drop(game);

    let mut resumed = ApplicationCoordinator::resume(launch).expect("resume durable checkpoint");
    let recovered = resumed.current_live_run().expect("recovered tick");
    assert_run_roots_equal(&recovered, &tick_zero);
    let persisted_presentation = tick_zero
        .presentation_snapshot
        .as_ref()
        .expect("tick-zero presentation");
    let recovered_presentation = recovered
        .presentation_snapshot
        .as_ref()
        .expect("recovery-cut presentation");
    assert_ne!(
        recovered_presentation.snapshot_epoch,
        persisted_presentation.snapshot_epoch
    );
    assert_eq!(recovered_presentation.snapshot_sequence, 0);
    resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}

#[test]
fn close_forces_tick_twenty_nine_into_the_final_save() {
    let root = test_root("live-cadence-close-29");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let generation_at_zero = game.current_generation;
    for _ in 0..29 {
        game.advance_reference_game_live(&[])
            .expect("advance through tick 29");
    }
    let checkpoint = game
        .prepared_run
        .as_ref()
        .expect("in-memory tick 29")
        .checkpoint
        .clone();
    let compatibility =
        super::super::recovery::save_compatibility(&game.activated_project, &checkpoint)
            .expect("save compatibility");
    assert_eq!(game.current_generation, generation_at_zero);

    game.close(CloseExecutionOptionsV1::default())
        .expect("forced checkpoint close");
    let loaded = game
        .save_store
        .load_latest(&compatibility)
        .expect("load final save");
    assert_eq!(loaded.checkpoint, checkpoint);
    cleanup(root);
}

#[test]
fn failed_forced_close_checkpoint_keeps_tick_twenty_nine_in_memory_for_retry() {
    let root = test_root("live-cadence-close-fault-29");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    for _ in 0..29 {
        game.advance_reference_game_live(&[])
            .expect("advance through tick 29");
    }
    let latest = game.current_live_run().expect("tick 29");
    let durable_before = game.durable.clone();
    let generation_before = game.current_generation;
    let objects_before = game.prepared_run_objects.clone();

    game.inject_fail_next_publication();
    let error = game
        .close(CloseExecutionOptionsV1::default())
        .expect_err("forced checkpoint publication failure");
    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    assert_eq!(
        game.current_live_run().expect("in-memory retry state"),
        latest
    );
    assert_eq!(game.durable, durable_before);
    assert_eq!(game.current_generation, generation_before);
    assert_eq!(game.prepared_run_objects, objects_before);

    let closed = game
        .close(CloseExecutionOptionsV1::default())
        .expect("retry forced checkpoint and close");
    assert!(matches!(closed, ApplicationCloseOutcomeV1::Closed { .. }));
    cleanup(root);
}

#[test]
fn ui_back_player_action_suspends_through_the_declared_lifecycle_path() {
    let root = test_root("ui-back-pause");
    let mut game = ApplicationCoordinator::launch(interactive_launch(&root)).expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let host = test_platform_host(&mut game);
    let escape = keyboard_escape_event(&host, 0);

    let sequence_before_pause = game.durable.store_sequence;
    let paused = game
        .advance_reference_game_live(std::slice::from_ref(&escape))
        .expect("committed ui-back suspends the session");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(paused.ticks, 1);
    assert_eq!(
        game.durable.store_sequence,
        sequence_before_pause + 1,
        "ui pause forces exactly one durable checkpoint generation"
    );

    // The pause request references the committed player action frame as its
    // replayable causal input instead of a platform event.
    let archived_requests = game.machine.archived_requests();
    let archived = archived_requests
        .iter()
        .find(|entry| entry.event.to_state == ApplicationSessionStatusV1::Suspended)
        .expect("archived suspend request");
    let request = ApplicationLifecycleRequestV1::from_jcs_bytes(
        &archived.canonical_request_bytes,
        next_contracts::canonical::CanonicalDecodeLimits::default(),
    )
    .expect("decode suspend request");
    assert_eq!(
        request.causal_input_reference.source_kind,
        CausalInputSourceKindV1::PlayerAction
    );
    assert_eq!(request.reason.kind, LifecycleReasonKindV1::SuspendRequested);
    assert_eq!(
        request.reason.reason_code.as_str(),
        "nextengine.session.ui-pause-requested"
    );

    // The suspending publication carries the declared pause-menu surface
    // whose save/load affordances bind the universal UI action ids.
    let snapshot = paused.presentation_snapshot.expect("pause snapshot");
    assert_eq!(
        snapshot
            .semantic_ui_records()
            .filter(|record| record.surface_id.as_str() == "nextengine.ui.surface.pause-menu")
            .count(),
        4
    );
    let save = snapshot
        .semantic_ui_records()
        .find(|record| {
            record.element.element_id.as_str() == "nextengine.ui.element.pause-menu.save"
        })
        .expect("save affordance element");
    assert!(
        save.element
            .affordances
            .iter()
            .any(
                |affordance| affordance.action_id.as_str() == "nextengine.action.ui-confirm"
                    && affordance.enabled
            )
    );

    // Resume follows the existing platform lifecycle path; gameplay then
    // advances normally.
    let resume = platform_reason_event(
        &mut game,
        PlatformEventKindV1::ResumeRequested,
        1,
        "nextengine.platform.reason.foregrounded",
    );
    game.resume_from_platform_event(&resume)
        .expect("resume from pause");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    let resumed = game
        .advance_reference_game_live(&[])
        .expect("post-resume advance");
    assert_eq!(resumed.ticks, 2);
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn pause_menu_save_persists_prepared_run_and_menu_resume_event_is_admitted() {
    let root = test_root("pause-menu-save-resume");
    let mut game = ApplicationCoordinator::launch(interactive_launch(&root)).expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let host = test_platform_host(&mut game);
    let escape = keyboard_escape_event(&host, 0);
    game.advance_reference_game_live(std::slice::from_ref(&escape))
        .expect("committed ui-back suspends the session");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Suspended);

    // The pause-menu save activates the same production save-store write the
    // final save uses; an unchanged checkpoint reuses the existing image.
    let save_generation_hash = game
        .save_current_prepared_run()
        .expect("pause-menu save persists the suspended prepared run");
    let repeated = game
        .save_current_prepared_run()
        .expect("idempotent pause-menu save");
    assert_eq!(repeated, save_generation_hash);

    // A menu-fabricated resume (dedicated source class, own admission
    // cursor) follows the same admitted lifecycle path as a platform resume.
    let menu_resume = PlatformEventV1::new(
        host.host_instance_id,
        SchemaId::new("nextengine.platform.source.pause-menu").expect("menu source class"),
        0,
        0,
        PlatformEventKindV1::ResumeRequested,
        PlatformEventPayloadV1::Reason {
            reason: SchemaId::new("nextengine.platform.reason.pause-menu-resume")
                .expect("menu resume reason"),
        },
        host.capability_set_hash,
    )
    .expect("menu resume event");
    game.resume_from_platform_event(&menu_resume)
        .expect("menu-fabricated resume is admitted");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    let resumed = game
        .advance_reference_game_live(&[])
        .expect("post-resume advance");
    assert_eq!(resumed.ticks, 2);
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn queue_host_consumed_live_input_requires_a_live_run() {
    let root = test_root("queue-host-consumed-no-run");
    let mut game = ApplicationCoordinator::launch(interactive_launch(&root)).expect("game launch");
    let error = game
        .queue_host_consumed_live_input(&[])
        .expect_err("host-consumed queue without a live run is rejected");
    assert!(matches!(error, ApplicationError::NoLiveRun));
    drop(game);
    cleanup(root);
}
