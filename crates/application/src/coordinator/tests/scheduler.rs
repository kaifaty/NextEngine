use super::*;

#[test]
fn interactive_presentation_path_defers_full_checkpoint_until_tick_thirty() {
    let root = test_root("interactive-presentation-checkpoint-cadence");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let generation_at_zero = game.current_generation;
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();

    for expected_tick in 1_u64..30 {
        let presentation = scheduler
            .advance_reference_game_presentation(&mut game, Duration::from_millis(34), &[])
            .expect("interactive presentation step")
            .expect("one fixed step is due");
        assert_eq!(presentation.simulation_tick, expected_tick);
        assert_eq!(
            game.prepared_run
                .as_ref()
                .expect("tick-zero checkpoint")
                .summary
                .ticks,
            0,
            "ordinary presentation ticks must not rebuild a durable checkpoint"
        );
        assert_eq!(game.current_generation, generation_at_zero);
    }

    let presentation = scheduler
        .advance_reference_game_presentation(&mut game, Duration::from_millis(34), &[])
        .expect("tick-thirty presentation step")
        .expect("one fixed step is due");
    assert_eq!(presentation.simulation_tick, 30);
    assert_eq!(
        game.prepared_run
            .as_ref()
            .expect("tick-thirty checkpoint")
            .summary
            .ticks,
        30
    );
    assert_ne!(game.current_generation, generation_at_zero);

    game.close(CloseExecutionOptionsV1::default())
        .expect("interactive game close");
    cleanup(root);
}

#[test]
fn prepared_run_object_inventory_replaces_more_than_2_500_generations_in_memory() {
    let root = test_root("prepared-run-object-inventory");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::None,
    ))
    .expect("game launch");
    let durable_object_count = game.objects.len();

    for generation in 0_u64..=2_500 {
        let object_bytes = std::array::from_fn(|slot| {
            let mut bytes = b"nextengine.test.prepared-run-object.v1\0".to_vec();
            bytes.extend_from_slice(&generation.to_le_bytes());
            bytes.push(u8::try_from(slot).expect("four object slots fit u8"));
            bytes
        });
        game.replace_prepared_run_object_bytes(object_bytes);
        assert_eq!(game.prepared_run_objects.len(), 4);
        assert_eq!(game.objects.len(), durable_object_count);
    }

    cleanup(root);
}
#[test]
fn live_publications_keep_only_the_current_checkpoint_object_closure() {
    let root = test_root("bounded-live-object-closure");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let durable_object_count = game.objects.len();

    for expected_tick in 1_u64..=32 {
        let run = game
            .advance_reference_game_live(&[])
            .expect("advance bounded live generation");
        assert_eq!(run.ticks, expected_tick);
        assert_eq!(game.prepared_run_objects.len(), 7);
        assert_eq!(game.objects.len(), durable_object_count);
        let expected_inventory_count = game
            .objects
            .keys()
            .chain(game.prepared_run_objects.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        assert_eq!(
            game.session_store
                .load_current()
                .expect("published live generation")
                .objects
                .len(),
            expected_inventory_count
        );
    }

    game.close(CloseExecutionOptionsV1::default())
        .expect("bounded live game close");
    cleanup(root);
}

#[test]
fn fixed_step_live_results_ignore_30_60_and_144_hz_render_partitioning() {
    let at_30_hz = partitioned_live_result("cadence-30", 15);
    let at_60_hz = partitioned_live_result("cadence-60", 30);
    let at_144_hz = partitioned_live_result("cadence-144", 72);

    for actual in [&at_60_hz, &at_144_hz] {
        assert_eq!(actual.0, at_30_hz.0);
        assert_eq!(actual.1, at_30_hz.1);
        assert_eq!(actual.2, at_30_hz.2);
        assert_eq!(actual.3, at_30_hz.3);
    }
    assert_eq!(at_30_hz.0, 15);
}

#[test]
fn fixed_step_live_scheduler_never_assigns_callback_input_to_elapsed_ticks() {
    let one_slow_callback = delayed_callback_input_result("callback-input-slow", &[100], &[true]);
    let three_callbacks = delayed_callback_input_result(
        "callback-input-partitioned",
        &[33, 33, 34],
        &[false, false, true],
    );

    assert_eq!(one_slow_callback.0, 4);
    assert_eq!(three_callbacks.0, 4);
    assert_eq!(one_slow_callback.1, three_callbacks.1);
    assert_eq!(one_slow_callback.2, three_callbacks.2);
    assert_eq!(one_slow_callback.3, three_callbacks.3);
}

#[test]
fn fixed_step_live_scheduler_suspends_at_a_tick_boundary_without_resume_catch_up() {
    let root = test_root("fixed-step-suspend-resume");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let suspend = platform_reason_event(
        &mut game,
        PlatformEventKindV1::SuspendRequested,
        20,
        "nextengine.platform.reason.backgrounded",
    );
    let resume = platform_reason_event(
        &mut game,
        PlatformEventKindV1::ResumeRequested,
        21,
        "nextengine.platform.reason.foregrounded",
    );
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();

    scheduler
        .advance_reference_game(
            &mut game,
            Duration::from_millis(34),
            std::slice::from_ref(&suspend),
        )
        .expect("elapsed tick precedes suspend observation");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    assert_eq!(game.current_live_run().expect("live state").ticks, 1);
    let sequence_before_suspend = game.durable.store_sequence;

    scheduler
        .advance_reference_game(&mut game, Duration::from_millis(34), &[])
        .expect("suspend boundary");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(game.current_live_run().expect("suspended state").ticks, 2);
    assert_eq!(
        game.durable.store_sequence,
        sequence_before_suspend + 1,
        "tick and suspend transition publish one durable generation"
    );
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 0);

    scheduler
        .advance_reference_game(&mut game, Duration::from_secs(60), &[])
        .expect("suspended wall time is ignored");
    assert_eq!(game.current_live_run().expect("paused state").ticks, 2);
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 0);

    scheduler
        .advance_reference_game(
            &mut game,
            Duration::from_secs(60),
            std::slice::from_ref(&resume),
        )
        .expect("resume transition");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    assert_eq!(game.current_live_run().expect("resumed state").ticks, 2);
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 0);

    scheduler
        .advance_reference_game(&mut game, Duration::from_millis(34), &[])
        .expect("first post-resume boundary");
    assert_eq!(game.current_live_run().expect("post-resume state").ticks, 3);
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn failed_atomic_tick_suspend_retries_one_generation_and_recovers_suspended() {
    let root = test_root("fixed-step-atomic-suspend-fault");
    let launch = LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    );
    let mut game = ApplicationCoordinator::launch(launch.clone()).expect("launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let suspend = platform_reason_event(
        &mut game,
        PlatformEventKindV1::SuspendRequested,
        60,
        "nextengine.platform.reason.backgrounded",
    );
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
    scheduler
        .advance_reference_game(
            &mut game,
            Duration::from_millis(34),
            std::slice::from_ref(&suspend),
        )
        .expect("stage suspend for the next boundary");
    let generation_before_suspend = game.current_generation;
    let sequence_before_suspend = game.durable.store_sequence;
    assert_eq!(game.current_live_run().expect("tick one").ticks, 1);

    game.inject_fail_next_publication();
    let error = scheduler
        .advance_reference_game(&mut game, Duration::from_millis(100), &[])
        .expect_err("atomic suspend publication fault");
    assert_eq!(error.diagnostic_code(), "SESSION_STORAGE_UNAVAILABLE");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    assert_eq!(game.current_live_run().expect("rolled back tick").ticks, 1);
    assert_eq!(game.current_generation, generation_before_suspend);
    assert_eq!(game.durable.store_sequence, sequence_before_suspend);

    scheduler
        .advance_reference_game(&mut game, Duration::ZERO, &[])
        .expect("retry the same tick and suspend input");
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(game.current_live_run().expect("suspended tick").ticks, 2);
    assert_eq!(game.durable.store_sequence, sequence_before_suspend + 1);
    let suspended_generation = game.current_generation;
    drop(game);

    let mut resumed = ApplicationCoordinator::resume(launch).expect("resume suspended generation");
    assert_eq!(resumed.state().state, ApplicationSessionStatusV1::Suspended);
    assert_eq!(resumed.current_live_run().expect("recovered tick").ticks, 2);
    assert_eq!(resumed.current_generation, suspended_generation);
    resumed
        .close(CloseExecutionOptionsV1::default())
        .expect("close");
    cleanup(root);
}

#[test]
fn fixed_step_live_scheduler_accepts_exactly_4_096_pending_events() {
    let root = test_root("fixed-step-event-limit");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let event = keyboard_movement_event(&mut game);
    let first_batch = vec![event.clone(); 4_095];
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();

    scheduler
        .advance_reference_game(&mut game, Duration::from_millis(1), &first_batch)
        .expect("N - 1 pending events");
    scheduler
        .advance_reference_game(&mut game, Duration::ZERO, std::slice::from_ref(&event))
        .expect("Nth pending event");

    assert_eq!(scheduler.pending_event_count(), 4_096);
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 30_000_000);
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn fixed_step_live_scheduler_rejects_event_4_097_without_mutation() {
    let root = test_root("fixed-step-event-overflow");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let event = keyboard_movement_event(&mut game);
    let admitted = vec![event.clone(); 4_096];
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
    scheduler
        .advance_reference_game(&mut game, Duration::from_millis(1), &admitted)
        .expect("N pending events");
    let pending_before = scheduler.pending_events();
    let accumulator_before = scheduler.accumulated_scaled_nanoseconds();

    let error = scheduler
        .advance_reference_game(
            &mut game,
            Duration::from_millis(2),
            std::slice::from_ref(&event),
        )
        .expect_err("N + 1 pending events must fail");

    assert_eq!(
        error.diagnostic_code(),
        "SESSION_FIXED_TICK_EVENT_BACKLOG_EXCEEDED"
    );
    assert!(matches!(
        error,
        crate::ApplicationError::LivePlatformEventBacklogExceeded
    ));
    assert_eq!(scheduler.pending_events(), pending_before);
    assert_eq!(
        scheduler.accumulated_scaled_nanoseconds(),
        accumulator_before
    );
    assert_eq!(
        game.current_live_run().expect("unchanged live state").ticks,
        0
    );
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}

#[test]
fn fixed_step_overload_is_bounded_and_preserves_observed_input() {
    let root = test_root("fixed-step-bounded-overload");
    let mut game = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        &root,
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("game launch");
    game.begin_reference_game_live(true)
        .expect("begin live reference game");
    let first =
        keyboard_movement_event_at(&mut game, 0, NormalizedControlPhaseV1::Started, i16::MAX);
    let second =
        keyboard_movement_event_at(&mut game, 1, NormalizedControlPhaseV1::Changed, i16::MAX);
    let batch = [first, second];
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();

    let overloaded = scheduler
        .advance_reference_game(&mut game, Duration::from_secs(121), &batch)
        .expect("host overload must remain a live, bounded condition")
        .expect("bounded catch-up must publish the latest completed tick");
    assert_eq!(
        overloaded.ticks, 8,
        "one pump may execute only the bounded catch-up budget"
    );
    assert_eq!(game.state().state, ApplicationSessionStatusV1::Active);
    assert_eq!(
        scheduler.pending_event_count(),
        2,
        "events observed during catch-up remain pending for the next boundary"
    );
    assert_eq!(scheduler.accumulated_scaled_nanoseconds(), 0);

    let next = scheduler
        .advance_reference_game(&mut game, Duration::from_millis(34), &[])
        .expect("live simulation must continue after overload")
        .expect("the next fixed boundary must run");
    assert_eq!(next.ticks, 9);
    assert_eq!(
        scheduler.pending_event_count(),
        0,
        "the preserved input is consumed exactly once"
    );
    game.close(CloseExecutionOptionsV1::default())
        .expect("live game close");
    cleanup(root);
}
