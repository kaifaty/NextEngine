use std::sync::{Arc, Barrier, mpsc};
use std::time::Duration;

use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

use super::{
    INTERACTIVE_SIMULATION_QUEUE_CAPACITY, InteractiveShutdownAttemptV1,
    InteractiveSimulationMessageV1, InteractiveSimulationWorkerV1,
    InteractiveWorkerDiagnosticOptionsV1, InteractiveWorkerFailureV1,
    InteractiveWorkerFinalizationV1, InteractiveWorkerFixedStepClassV1,
    PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS, QueueTelemetryV1, finalize_diagnostic_worker,
    finalize_diagnostic_worker_with_attempt_limit, prepare_production_worker_diagnostic,
    resolve_interactive_shutdown_attempt, run_production_worker_diagnostic,
};
use crate::{
    ApplicationCoordinator, CloseExecutionOptionsV1, FixedStepLiveSchedulerV1, LaunchRequestV1,
};

#[test]
fn production_worker_preserves_fifo_and_matches_serial_authoritative_roots() {
    let callback_count = PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS;
    let callback_elapsed = Duration::from_nanos(16_666_667);
    let serial_root = unique_test_directory("worker-serial-root");
    let worker_root = unique_test_directory("worker-threaded-root");

    let mut serial = ApplicationCoordinator::launch(LaunchRequestV1::reference(
        serial_root.clone(),
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("serial launch");
    serial
        .begin_reference_game_live(true)
        .expect("serial live run");
    let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
    for _ in 0..callback_count {
        scheduler
            .advance_reference_game_presentation_shared(&mut serial, callback_elapsed, &[])
            .expect("serial fixed-step callback");
    }
    let serial_run = serial.current_live_run().expect("serial final live run");
    serial
        .close(CloseExecutionOptionsV1::default())
        .expect("serial close");

    let report = run_production_worker_diagnostic(InteractiveWorkerDiagnosticOptionsV1 {
        launch: LaunchRequestV1::reference(
            worker_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        callback_count,
        callback_elapsed,
    })
    .expect("production worker diagnostic");

    assert_eq!(report.metrics.submitted_callbacks, callback_count);
    assert_eq!(report.metrics.processed_callbacks, callback_count);
    assert_eq!(report.metrics.dropped_callbacks, 0);
    assert_eq!(report.metrics.reordered_callbacks, 0);
    assert_eq!(
        report.metrics.snapshot_read_samples.len(),
        usize::try_from(callback_count).expect("callback count fits usize")
    );
    let mut previous_snapshot_sequence = report.initial_snapshot_sequence;
    for sample in &report.metrics.snapshot_read_samples {
        let expected_lag = sample
            .publication_callback_sequence
            .map_or(sample.callback_sequence.saturating_add(1), |source| {
                sample.callback_sequence.saturating_sub(source)
            });
        assert_eq!(sample.sequence_lag, expected_lag);
        assert!(sample.snapshot_sequence >= previous_snapshot_sequence);
        assert_eq!(
            sample.fresh_generation,
            sample.snapshot_sequence > previous_snapshot_sequence
        );
        previous_snapshot_sequence = sample.snapshot_sequence;
    }
    assert_eq!(
        report
            .metrics
            .message_age_samples
            .iter()
            .map(|sample| sample.callback_sequence)
            .collect::<Vec<_>>(),
        (0..callback_count).collect::<Vec<_>>()
    );
    assert_eq!(report.run_report.ticks, serial_run.ticks);
    assert_eq!(
        report.run_report.authoritative_state_root,
        serial_run.authoritative_state_root.to_hex()
    );
    assert_eq!(
        report.run_report.command_archive_root,
        serial_run.command_archive_root.to_hex()
    );
    assert_eq!(
        report.run_report.command_identity_index_root,
        serial_run.command_identity_index_root.to_hex()
    );
    assert_eq!(
        report.run_report.command_ledger_hash,
        serial_run.command_ledger_hash.to_hex()
    );

    std::fs::remove_dir_all(serial_root).expect("remove serial root");
    std::fs::remove_dir_all(worker_root).expect("remove worker root");
}

#[test]
fn prepared_production_worker_is_ready_and_matches_the_legacy_entry_point() {
    let callback_count = PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS;
    let callback_elapsed = Duration::from_nanos(16_666_667);
    let legacy_root = unique_test_directory("worker-legacy-entry-point");
    let prepared_root = unique_test_directory("worker-prepared-entry-point");
    let legacy = run_production_worker_diagnostic(InteractiveWorkerDiagnosticOptionsV1 {
        launch: LaunchRequestV1::reference(
            legacy_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        callback_count,
        callback_elapsed,
    })
    .expect("legacy diagnostic entry point");
    let mut prepared = prepare_production_worker_diagnostic(InteractiveWorkerDiagnosticOptionsV1 {
        launch: LaunchRequestV1::reference(
            prepared_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        callback_count,
        callback_elapsed,
    })
    .expect("prepare production worker diagnostic");
    let ready_snapshot = prepared
        .worker
        .as_ref()
        .expect("prepared worker is live")
        .read_latest_snapshot()
        .expect("initial snapshot is published before measured execution");
    assert_eq!(ready_snapshot.snapshot.simulation_tick, 0);
    assert_eq!(ready_snapshot.processed_callbacks, 0);

    let measurement = prepared.run_measured();
    let measured = prepared
        .finish(measurement)
        .expect("finish prepared diagnostic");
    assert_eq!(measured.callback_count, legacy.callback_count);
    assert_eq!(measured.callback_elapsed, legacy.callback_elapsed);
    assert_eq!(measured.run_report.ticks, legacy.run_report.ticks);
    assert_eq!(
        measured.run_report.authoritative_state_root,
        legacy.run_report.authoritative_state_root
    );
    assert_eq!(
        measured.run_report.command_archive_root,
        legacy.run_report.command_archive_root
    );
    assert_eq!(
        measured.run_report.command_identity_index_root,
        legacy.run_report.command_identity_index_root
    );
    assert_eq!(
        measured.run_report.command_ledger_hash,
        legacy.run_report.command_ledger_hash
    );
    assert_eq!(
        (
            measured.metrics.submitted_callbacks,
            measured.metrics.processed_callbacks,
            measured.metrics.fixed_steps,
            measured.metrics.ordinary_fixed_steps,
            measured.metrics.checkpoint_fixed_steps,
            measured.metrics.snapshot_publications,
            measured.metrics.dropped_callbacks,
            measured.metrics.reordered_callbacks,
        ),
        (
            legacy.metrics.submitted_callbacks,
            legacy.metrics.processed_callbacks,
            legacy.metrics.fixed_steps,
            legacy.metrics.ordinary_fixed_steps,
            legacy.metrics.checkpoint_fixed_steps,
            legacy.metrics.snapshot_publications,
            legacy.metrics.dropped_callbacks,
            legacy.metrics.reordered_callbacks,
        )
    );
    let callback_capacity = usize::try_from(callback_count).expect("callback count fits usize");
    assert!(measured.metrics.send_wait_samples.capacity() >= callback_capacity);
    assert!(measured.metrics.snapshot_read_samples.capacity() >= callback_capacity);

    std::fs::remove_dir_all(legacy_root).expect("remove legacy root");
    std::fs::remove_dir_all(prepared_root).expect("remove prepared root");
}

#[test]
fn prepared_production_worker_error_joins_before_return() {
    let state_root = unique_test_directory("worker-prepared-error-cleanup");
    let mut prepared = prepare_production_worker_diagnostic(InteractiveWorkerDiagnosticOptionsV1 {
        launch: LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        callback_count: PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS,
        callback_elapsed: Duration::from_nanos(16_666_667),
    })
    .expect("prepare production worker diagnostic");
    let snapshot = Arc::clone(
        &prepared
            .worker
            .as_ref()
            .expect("prepared worker exists")
            .latest_snapshot,
    );
    let poison = std::thread::spawn(move || {
        let _guard = snapshot.write().expect("snapshot lock");
        panic!("poison prepared diagnostic snapshot lock");
    });
    assert!(poison.join().is_err());

    let measurement = prepared.run_to_completion();
    assert!(prepared.worker.is_none());
    let failure = prepared
        .finish(measurement)
        .expect_err("poisoned snapshot must fail the diagnostic");
    assert_eq!(failure.code, "PLATFORM_PRESENTATION_STATE_POISONED");
    std::fs::remove_dir_all(state_root).expect("remove prepared error root");
}

#[test]
fn production_worker_reports_exact_fixed_steps_at_30_60_and_144_hz() {
    for (label, callback_elapsed, expected_steps) in [
        ("30", Duration::from_nanos(33_333_334), 240_u64),
        ("60", Duration::from_nanos(16_666_667), 120_u64),
        ("144", Duration::from_nanos(6_944_445), 50_u64),
    ] {
        let state_root = unique_test_directory(&format!("worker-cadence-{label}"));
        let report = run_production_worker_diagnostic(InteractiveWorkerDiagnosticOptionsV1 {
            launch: LaunchRequestV1::reference(
                state_root.clone(),
                CompositionRootV1::Game,
                PresentationTargetKindV1::Interactive,
            ),
            callback_count: PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS,
            callback_elapsed,
        })
        .expect("cadence diagnostic");
        assert_eq!(report.metrics.fixed_steps, expected_steps, "{label} Hz");
        assert_eq!(report.final_simulation_tick, expected_steps, "{label} Hz");
        assert_eq!(
            report.metrics.snapshot_publications,
            expected_steps + 1,
            "{label} Hz"
        );
        assert_eq!(
            report.metrics.fixed_step_samples.len(),
            usize::try_from(expected_steps).expect("test step count fits usize"),
            "{label} Hz"
        );
        std::fs::remove_dir_all(state_root).expect("remove cadence root");
    }
}

#[test]
fn production_worker_classifies_every_thirtieth_tick_as_checkpoint_work() {
    let state_root = unique_test_directory("worker-checkpoint-class");
    let report = run_production_worker_diagnostic(InteractiveWorkerDiagnosticOptionsV1 {
        launch: LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        callback_count: PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS,
        callback_elapsed: Duration::from_nanos(33_333_334),
    })
    .expect("checkpoint diagnostic");
    let checkpoint_ticks = report
        .metrics
        .fixed_step_samples
        .iter()
        .filter_map(|sample| {
            (sample.class == InteractiveWorkerFixedStepClassV1::Checkpoint)
                .then_some(sample.simulation_tick)
        })
        .collect::<Vec<_>>();
    assert_eq!(checkpoint_ticks, vec![30, 60, 90, 120, 150, 180, 210, 240]);
    assert_eq!(report.metrics.checkpoint_fixed_steps, 8);
    assert_eq!(report.metrics.ordinary_fixed_steps, 232);
    std::fs::remove_dir_all(state_root).expect("remove checkpoint root");
}

#[test]
fn bounded_queue_telemetry_tracks_exact_fifo_occupancy_at_capacity() {
    let telemetry = Arc::new(QueueTelemetryV1::default());
    let (sender, receiver) = mpsc::sync_channel(INTERACTIVE_SIMULATION_QUEUE_CAPACITY);
    let release = Arc::new(Barrier::new(2));
    let worker_release = Arc::clone(&release);
    let worker_telemetry = Arc::clone(&telemetry);
    let worker = std::thread::spawn(move || {
        worker_release.wait();
        let mut sequences = Vec::with_capacity(INTERACTIVE_SIMULATION_QUEUE_CAPACITY);
        for _ in 0..INTERACTIVE_SIMULATION_QUEUE_CAPACITY {
            let message = worker_telemetry
                .recv(&receiver)
                .expect("instrumented dequeue");
            sequences.push(message.callback_sequence().expect("advance sequence"));
        }
        sequences
    });
    for sequence in 0..INTERACTIVE_SIMULATION_QUEUE_CAPACITY {
        let sequence = u64::try_from(sequence).expect("queue sequence fits u64");
        telemetry
            .send(
                &sender,
                InteractiveSimulationMessageV1::Advance {
                    callback_sequence: sequence,
                    enqueued_at: None,
                    elapsed: Duration::ZERO,
                    events: Vec::new(),
                },
            )
            .expect("queue accepts capacity");
    }
    release.wait();
    assert_eq!(
        worker.join().expect("bounded queue worker"),
        (0..u64::try_from(INTERACTIVE_SIMULATION_QUEUE_CAPACITY).expect("queue capacity fits u64"))
            .collect::<Vec<_>>()
    );
    let snapshot = telemetry.finish_snapshot().expect("queue snapshot");
    assert_eq!(snapshot.high_water, INTERACTIVE_SIMULATION_QUEUE_CAPACITY);
    assert_eq!(
        snapshot.submitted_callbacks,
        u64::try_from(INTERACTIVE_SIMULATION_QUEUE_CAPACITY).expect("queue capacity fits u64")
    );
    assert_eq!(snapshot.submitted_callbacks, snapshot.dequeued_callbacks);
}

#[test]
fn close_failure_retries_before_closed_and_preserves_pending_runtime_failure() {
    let close_failure =
        InteractiveWorkerFailureV1::runtime("SESSION_FINAL_SAVE_FAILED", "injected close failure");
    assert!(matches!(
        resolve_interactive_shutdown_attempt::<u8>(Err(close_failure), None),
        InteractiveShutdownAttemptV1::Retry(_)
    ));
    assert!(matches!(
        resolve_interactive_shutdown_attempt(Ok(7_u8), None),
        InteractiveShutdownAttemptV1::Closed(Ok(7))
    ));
    let runtime_failure =
        InteractiveWorkerFailureV1::runtime("SESSION_RUNTIME_FAILED", "prior simulation failure");
    assert!(matches!(
        resolve_interactive_shutdown_attempt(Ok(9_u8), Some(&runtime_failure)),
        InteractiveShutdownAttemptV1::Closed(Err(_))
    ));
}

#[test]
fn production_worker_retries_one_failed_close_publication_before_terminal_receipt() {
    let state_root = unique_test_directory("worker-close-retry");
    let (mut worker, _) = InteractiveSimulationWorkerV1::spawn(LaunchRequestV1::reference(
        state_root.clone(),
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("spawn production worker");
    worker
        .inject_fail_next_close_publication()
        .expect("inject one close publication failure");

    assert!(matches!(
        worker.shutdown_attempt(None, 0),
        InteractiveWorkerFinalizationV1::Retry(_)
    ));
    let InteractiveWorkerFinalizationV1::Closed { result, .. } = worker.shutdown_attempt(None, 0)
    else {
        panic!("second close attempt must publish terminal Closed");
    };
    let report = (*result).expect("terminal worker report");
    assert_eq!(report.close_result, "Saved");
    assert_ne!(report.close_receipt_hash, "0".repeat(64));
    std::fs::remove_dir_all(state_root).expect("remove close retry root");
}

#[test]
fn diagnostic_finalization_exhaustion_disconnects_and_joins_worker() {
    let state_root = unique_test_directory("worker-close-budget");
    let (mut worker, _) = InteractiveSimulationWorkerV1::spawn_with_diagnostic_capacity(
        LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        Some(
            usize::try_from(PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS)
                .expect("diagnostic callback count fits usize"),
        ),
    )
    .expect("spawn measured worker");
    worker
        .inject_fail_next_close_publication()
        .expect("inject close publication failure");

    let failure = finalize_diagnostic_worker_with_attempt_limit(&mut worker, 1)
        .expect_err("one failed close exhausts the explicit attempt budget");
    assert_eq!(failure.code, "SESSION_STORAGE_UNAVAILABLE");
    assert!(failure.message.contains("exhausted 1 close attempts"));
    assert!(worker.worker.is_none());
    std::fs::remove_dir_all(state_root).expect("remove close budget root");
}

#[test]
fn production_worker_fast_path_disables_diagnostic_clocks_and_atomics() {
    let state_root = unique_test_directory("worker-fast-path");
    let (mut worker, _) = InteractiveSimulationWorkerV1::spawn(LaunchRequestV1::reference(
        state_root.clone(),
        CompositionRootV1::Game,
        PresentationTargetKindV1::Interactive,
    ))
    .expect("spawn production worker");
    assert!(worker.queue_telemetry.is_none());
    assert!(worker.processed_callbacks.is_none());

    let submit = worker
        .submit_advance(Duration::from_nanos(16_666_667), Vec::new())
        .expect("submit unmeasured callback");
    assert_eq!(submit.send_wait, Duration::ZERO);
    let read = worker.read_latest_snapshot().expect("read shared snapshot");
    assert_eq!(read.lock_wait, Duration::ZERO);
    assert_eq!(read.processed_callbacks, 0);

    loop {
        if matches!(
            worker.shutdown_attempt(None, 0),
            InteractiveWorkerFinalizationV1::Closed { .. }
        ) {
            break;
        }
    }
    std::fs::remove_dir_all(state_root).expect("remove fast-path root");
}

#[test]
fn diagnostic_read_failure_still_joins_worker_before_return() {
    let state_root = unique_test_directory("worker-error-finalize");
    let (mut worker, _) = InteractiveSimulationWorkerV1::spawn_with_diagnostic_capacity(
        LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        ),
        Some(
            usize::try_from(PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS)
                .expect("diagnostic callback count fits usize"),
        ),
    )
    .expect("spawn measured worker");
    let snapshot = Arc::clone(&worker.latest_snapshot);
    let poison = std::thread::spawn(move || {
        let _guard = snapshot.write().expect("snapshot lock");
        panic!("poison diagnostic snapshot lock");
    });
    assert!(poison.join().is_err());
    assert!(worker.read_latest_snapshot().is_err());

    assert!(finalize_diagnostic_worker(&mut worker).is_err());
    assert!(worker.worker.is_none());
    std::fs::remove_dir_all(state_root).expect("remove finalized error root");
}

fn unique_test_directory(label: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "nextengine-application-{label}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}
