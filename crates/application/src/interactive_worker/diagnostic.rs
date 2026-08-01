use super::*;

static NEXT_DIAGNOSTIC_PREPARATION_ID: AtomicU64 = AtomicU64::new(1);

pub struct PreparedProductionWorkerDiagnosticV1 {
    preparation_id: u64,
    pub(super) worker: Option<InteractiveSimulationWorkerV1>,
    callback_count: u64,
    callback_elapsed: Duration,
    initial_snapshot_epoch: ContentHash,
    initial_snapshot_sequence: u64,
    initial_simulation_tick: u64,
    previous_generation: (ContentHash, u64),
    send_wait_samples: Vec<InteractiveWorkerTimingSampleV1>,
    snapshot_read_samples: Vec<InteractiveWorkerSnapshotReadSampleV1>,
    run_started: bool,
}

/// Opaque callback-and-close outcome. Report validation and assembly are
/// intentionally deferred until the external measurement window is closed.
pub struct ProductionWorkerDiagnosticMeasurementV1 {
    preparation_id: u64,
    run_report: RunReportV1,
    metrics: InteractiveWorkerDiagnosticMetricsV1,
    final_snapshot: Arc<PresentationSnapshotV2>,
}

pub fn prepare_production_worker_diagnostic(
    options: InteractiveWorkerDiagnosticOptionsV1,
) -> Result<PreparedProductionWorkerDiagnosticV1, InteractiveWorkerFailureV1> {
    validate_diagnostic_options(&options)?;
    let preparation_id = NEXT_DIAGNOSTIC_PREPARATION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| {
            InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker diagnostic preparation id overflow",
            )
        })?;
    let sample_capacity = usize::try_from(options.callback_count).map_err(|_| {
        InteractiveWorkerFailureV1::runtime(
            "PERFORMANCE_SCENARIO_INVALID",
            "callback count does not fit the host address space",
        )
    })?;
    let send_wait_samples = allocate_diagnostic_sample_buffer(sample_capacity)?;
    let snapshot_read_samples = allocate_diagnostic_sample_buffer(sample_capacity)?;
    let (worker, ready) = InteractiveSimulationWorkerV1::spawn_with_diagnostic_capacity(
        options.launch,
        Some(sample_capacity),
    )?;
    let initial_snapshot_epoch = ready.initial_snapshot.snapshot_epoch;
    let initial_snapshot_sequence = ready.initial_snapshot.snapshot_sequence;
    let initial_simulation_tick = ready.initial_snapshot.simulation_tick;
    Ok(PreparedProductionWorkerDiagnosticV1 {
        preparation_id,
        worker: Some(worker),
        callback_count: options.callback_count,
        callback_elapsed: options.callback_elapsed,
        initial_snapshot_epoch,
        initial_snapshot_sequence,
        initial_simulation_tick,
        previous_generation: (initial_snapshot_epoch, initial_snapshot_sequence),
        send_wait_samples,
        snapshot_read_samples,
        run_started: false,
    })
}

pub fn run_production_worker_diagnostic(
    options: InteractiveWorkerDiagnosticOptionsV1,
) -> Result<InteractiveWorkerDiagnosticReportV1, InteractiveWorkerFailureV1> {
    let mut prepared = prepare_production_worker_diagnostic(options)?;
    let measurement = prepared.run_measured();
    prepared.finish(measurement)
}

impl PreparedProductionWorkerDiagnosticV1 {
    pub fn run_measured(
        &mut self,
    ) -> Result<ProductionWorkerDiagnosticMeasurementV1, InteractiveWorkerFailureV1> {
        self.run_to_completion()
    }

    pub(super) fn run_to_completion(
        &mut self,
    ) -> Result<ProductionWorkerDiagnosticMeasurementV1, InteractiveWorkerFailureV1> {
        if self.run_started {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker diagnostic was already executed",
            ));
        }
        self.run_started = true;
        let worker = self.worker.as_mut().ok_or_else(|| {
            InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker diagnostic was already consumed",
            )
        })?;
        let workload = (|| {
            for _ in 0..self.callback_count {
                if let Some(failure) = worker.try_take_failure() {
                    return Err(failure);
                }
                let submit = worker.submit_advance(self.callback_elapsed, Vec::new())?;
                self.send_wait_samples
                    .push(InteractiveWorkerTimingSampleV1 {
                        callback_sequence: submit.callback_sequence,
                        nanoseconds: submit.send_wait.as_nanos(),
                    });
                let read = worker.read_latest_snapshot()?;
                let generation = (
                    read.snapshot.snapshot_epoch,
                    read.snapshot.snapshot_sequence,
                );
                let submitted = submit.callback_sequence.saturating_add(1);
                self.snapshot_read_samples
                    .push(InteractiveWorkerSnapshotReadSampleV1 {
                        callback_sequence: submit.callback_sequence,
                        lock_wait_nanoseconds: read.lock_wait.as_nanos(),
                        processed_callbacks: read.processed_callbacks,
                        sequence_lag: read
                            .publication_callback_sequence
                            .map_or(submitted, |source| {
                                submit.callback_sequence.saturating_sub(source)
                            }),
                        publication_callback_sequence: read.publication_callback_sequence,
                        snapshot_sequence: read.snapshot.snapshot_sequence,
                        simulation_tick: read.snapshot.simulation_tick,
                        fresh_generation: generation != self.previous_generation,
                    });
                self.previous_generation = generation;
            }
            Ok(())
        })();
        let finalization = finalize_diagnostic_worker(worker);
        self.worker = None;
        if let Err(mut failure) = workload {
            if let Err(finalization_failure) = finalization {
                failure.message.push_str(&format!(
                    "; finalization {}: {}",
                    finalization_failure.code, finalization_failure.message
                ));
                failure.exit_code = failure.exit_code.max(finalization_failure.exit_code);
            }
            return Err(failure);
        }
        let (run_report, metrics, final_snapshot) = finalization?;
        Ok(ProductionWorkerDiagnosticMeasurementV1 {
            preparation_id: self.preparation_id,
            run_report,
            metrics,
            final_snapshot,
        })
    }

    pub fn finish(
        mut self,
        measurement: Result<ProductionWorkerDiagnosticMeasurementV1, InteractiveWorkerFailureV1>,
    ) -> Result<InteractiveWorkerDiagnosticReportV1, InteractiveWorkerFailureV1> {
        let ProductionWorkerDiagnosticMeasurementV1 {
            preparation_id,
            run_report,
            mut metrics,
            final_snapshot,
        } = measurement?;
        if !self.run_started || preparation_id != self.preparation_id {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker measurement does not match its preparation",
            ));
        }
        metrics.send_wait_samples = std::mem::take(&mut self.send_wait_samples);
        metrics.snapshot_read_samples = std::mem::take(&mut self.snapshot_read_samples);
        validate_completed_diagnostic(&metrics, self.callback_count)?;
        Ok(InteractiveWorkerDiagnosticReportV1 {
            callback_count: self.callback_count,
            callback_elapsed: self.callback_elapsed,
            initial_snapshot_epoch: self.initial_snapshot_epoch,
            initial_snapshot_sequence: self.initial_snapshot_sequence,
            initial_simulation_tick: self.initial_simulation_tick,
            final_snapshot_epoch: final_snapshot.snapshot_epoch,
            final_snapshot_sequence: final_snapshot.snapshot_sequence,
            final_simulation_tick: final_snapshot.simulation_tick,
            run_report,
            metrics,
        })
    }
}

impl Drop for PreparedProductionWorkerDiagnosticV1 {
    fn drop(&mut self) {
        if let Some(mut worker) = self.worker.take() {
            let _ = worker.disconnect_and_join();
        }
    }
}

pub(super) fn finalize_diagnostic_worker(
    worker: &mut InteractiveSimulationWorkerV1,
) -> Result<
    (
        RunReportV1,
        InteractiveWorkerDiagnosticMetricsV1,
        Arc<PresentationSnapshotV2>,
    ),
    InteractiveWorkerFailureV1,
> {
    finalize_diagnostic_worker_with_attempt_limit(worker, MAXIMUM_DIAGNOSTIC_FINALIZATION_ATTEMPTS)
}

pub(super) fn finalize_diagnostic_worker_with_attempt_limit(
    worker: &mut InteractiveSimulationWorkerV1,
    attempt_limit: usize,
) -> Result<
    (
        RunReportV1,
        InteractiveWorkerDiagnosticMetricsV1,
        Arc<PresentationSnapshotV2>,
    ),
    InteractiveWorkerFailureV1,
> {
    let mut last_retry = None;
    for _ in 0..attempt_limit {
        match worker.shutdown_attempt(None, 0) {
            InteractiveWorkerFinalizationV1::Retry(failure) => last_retry = Some(failure),
            InteractiveWorkerFinalizationV1::Closed {
                result,
                diagnostic_metrics,
            } => {
                let run_report = (*result)?;
                let metrics = diagnostic_metrics.ok_or_else(|| {
                    InteractiveWorkerFailureV1::runtime(
                        "PERFORMANCE_MEASUREMENT_INVALID",
                        "production worker returned no diagnostic metrics",
                    )
                })?;
                let final_snapshot = worker.read_latest_snapshot()?.snapshot;
                return Ok((run_report, metrics, final_snapshot));
            }
        }
    }
    let cleanup = worker.disconnect_and_join();
    let cleanup_message = match cleanup {
        InteractiveWorkerFinalizationV1::Retry(failure) => failure.message,
        InteractiveWorkerFinalizationV1::Closed { result, .. } => match *result {
            Ok(_) => "worker closed during bounded disconnect cleanup".to_owned(),
            Err(failure) => failure.message,
        },
    };
    let last_retry = last_retry.unwrap_or_else(|| {
        InteractiveWorkerFailureV1::runtime(
            "SESSION_FINAL_SAVE_FAILED",
            "production worker diagnostic finalization attempt budget is zero",
        )
    });
    Err(InteractiveWorkerFailureV1::runtime(
        last_retry.code,
        format!(
            "production worker diagnostic exhausted {attempt_limit} close attempts: {}; cleanup: {cleanup_message}",
            last_retry.message
        ),
    ))
}

fn validate_diagnostic_options(
    options: &InteractiveWorkerDiagnosticOptionsV1,
) -> Result<(), InteractiveWorkerFailureV1> {
    if !(PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS..=MAXIMUM_DIAGNOSTIC_CALLBACKS)
        .contains(&options.callback_count)
    {
        return Err(InteractiveWorkerFailureV1::runtime(
            "PERFORMANCE_SCENARIO_INVALID",
            format!(
                "production worker diagnostic callback count must be between {PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS} and {MAXIMUM_DIAGNOSTIC_CALLBACKS}"
            ),
        ));
    }
    if options.callback_elapsed.is_zero()
        || options.callback_elapsed > MAXIMUM_SINGLE_STEP_CALLBACK_ELAPSED
    {
        return Err(InteractiveWorkerFailureV1::runtime(
            "PERFORMANCE_SCENARIO_INVALID",
            "production worker diagnostic callback elapsed duration must be positive and no greater than one 30 Hz fixed-step interval",
        ));
    }
    Ok(())
}

fn validate_completed_diagnostic(
    metrics: &InteractiveWorkerDiagnosticMetricsV1,
    callback_count: u64,
) -> Result<(), InteractiveWorkerFailureV1> {
    let sample_count_matches = metrics.send_wait_samples.len()
        == usize::try_from(callback_count).unwrap_or(usize::MAX)
        && metrics.message_age_samples.len()
            == usize::try_from(callback_count).unwrap_or(usize::MAX)
        && metrics.snapshot_read_samples.len()
            == usize::try_from(callback_count).unwrap_or(usize::MAX);
    if metrics.submitted_callbacks != callback_count
        || metrics.processed_callbacks != callback_count
        || metrics.dropped_callbacks != 0
        || metrics.reordered_callbacks != 0
        || !sample_count_matches
        || metrics.fixed_step_samples.len()
            != usize::try_from(metrics.fixed_steps).unwrap_or(usize::MAX)
        || metrics.snapshot_publications != metrics.fixed_steps.saturating_add(1)
    {
        return Err(InteractiveWorkerFailureV1::runtime(
            "PERFORMANCE_MEASUREMENT_INVALID",
            "production worker diagnostic lost, reordered, or failed to sample a callback",
        ));
    }
    Ok(())
}
