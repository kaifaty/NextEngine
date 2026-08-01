use super::*;

impl InteractiveSimulationWorkerV1 {
    pub fn spawn(
        launch: LaunchRequestV1,
    ) -> Result<(Self, InteractiveWorkerReadyV1), InteractiveWorkerFailureV1> {
        Self::spawn_with_diagnostic_capacity(launch, None)
    }

    pub(super) fn spawn_with_diagnostic_capacity(
        launch: LaunchRequestV1,
        diagnostic_capacity: Option<usize>,
    ) -> Result<(Self, InteractiveWorkerReadyV1), InteractiveWorkerFailureV1> {
        let capabilities = launch.platform_capability_set.clone().ok_or_else(|| {
            InteractiveWorkerFailureV1::runtime(
                "PLATFORM_CAPABILITY_SET_REQUIRED",
                "interactive launch requires a platform capability set",
            )
        })?;
        if launch.presentation_target != PresentationTargetKindV1::Interactive {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PLATFORM_FORBIDDEN_PRESENTATION_TARGET",
                "simulation worker requires the interactive presentation target",
            ));
        }
        if launch.composition_root != CompositionRootV1::Game {
            return Err(InteractiveWorkerFailureV1::runtime(
                "SESSION_COMPOSITION_ROOT_INVALID",
                "simulation worker requires the game composition root",
            ));
        }
        let latest_snapshot = Arc::new(RwLock::new(None));
        let processed_callbacks = diagnostic_capacity.map(|_| Arc::new(AtomicU64::new(0)));
        let queue_telemetry = diagnostic_capacity.map(|_| Arc::new(QueueTelemetryV1::default()));
        let (work_sender, work_receiver) =
            mpsc::sync_channel(INTERACTIVE_SIMULATION_QUEUE_CAPACITY);
        let (failure_sender, failure_receiver) = mpsc::sync_channel(1);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let worker_snapshot = Arc::clone(&latest_snapshot);
        let worker_processed_callbacks = processed_callbacks.clone();
        let worker_queue_telemetry = queue_telemetry.clone();
        let worker = std::thread::Builder::new()
            .name("next-simulation".to_owned())
            .spawn(move || {
                run_interactive_simulation_session_worker(
                    launch,
                    capabilities,
                    work_receiver,
                    worker_snapshot,
                    ready_sender,
                    failure_sender,
                    worker_processed_callbacks,
                    worker_queue_telemetry,
                    diagnostic_capacity,
                )
            })
            .map_err(|error| {
                InteractiveWorkerFailureV1::runtime(
                    "SESSION_RUNTIME_FAILED",
                    format!("failed to start the simulation worker: {error}"),
                )
            })?;

        let ready = match ready_receiver.recv() {
            Ok(Ok(ready)) => ready,
            Ok(Err(failure)) => {
                let _ = worker.join();
                return Err(failure);
            }
            Err(_) => return Err(join_failed_worker(worker)),
        };
        Ok((
            Self {
                work_sender,
                failure_receiver,
                latest_snapshot,
                processed_callbacks,
                queue_telemetry,
                next_callback_sequence: 0,
                worker: Some(worker),
            },
            ready,
        ))
    }

    pub fn submit_advance(
        &mut self,
        elapsed: Duration,
        events: Vec<PlatformEventV1>,
    ) -> Result<InteractiveMainSubmitV1, InteractiveWorkerFailureV1> {
        let callback_sequence = self.next_callback_sequence;
        let next_callback_sequence = callback_sequence
            .checked_add(1)
            .ok_or_else(|| callback_sequence_overflow("callback sequence overflow"))?;
        let send_started = self.queue_telemetry.as_ref().map(|_| Instant::now());
        self.send_message(InteractiveSimulationMessageV1::Advance {
            callback_sequence,
            enqueued_at: None,
            elapsed,
            events,
        })?;
        let send_wait = send_started.map_or(Duration::ZERO, |started| started.elapsed());
        self.next_callback_sequence = next_callback_sequence;
        Ok(InteractiveMainSubmitV1 {
            callback_sequence,
            send_wait,
        })
    }

    pub fn read_latest_snapshot(
        &self,
    ) -> Result<InteractiveMainSnapshotReadV1, InteractiveWorkerFailureV1> {
        let read_started = self.queue_telemetry.as_ref().map(|_| Instant::now());
        let latest = self.latest_snapshot.read().map_err(|_| {
            InteractiveWorkerFailureV1::runtime(
                "PLATFORM_PRESENTATION_STATE_POISONED",
                "latest presentation snapshot lock was poisoned",
            )
        })?;
        let lock_wait = read_started.map_or(Duration::ZERO, |started| started.elapsed());
        let published = latest.as_ref().cloned().ok_or_else(|| {
            InteractiveWorkerFailureV1::runtime(
                "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
                "simulation worker published no initial presentation snapshot",
            )
        })?;
        Ok(InteractiveMainSnapshotReadV1 {
            snapshot: published.snapshot,
            lock_wait,
            processed_callbacks: self
                .processed_callbacks
                .as_ref()
                .map_or(0, |processed| processed.load(Ordering::Acquire)),
            publication_callback_sequence: published.callback_sequence,
        })
    }

    #[must_use]
    pub fn try_take_failure(&self) -> Option<InteractiveWorkerFailureV1> {
        self.failure_receiver.try_recv().ok()
    }

    pub fn shutdown_attempt(
        &mut self,
        platform_close_event: Option<PlatformEventV1>,
        rendered_objects: u64,
    ) -> InteractiveWorkerFinalizationV1 {
        if self.worker.is_none() {
            return InteractiveWorkerFinalizationV1::Closed {
                result: Box::new(Err(InteractiveWorkerFailureV1::runtime(
                    "SESSION_RUNTIME_FAILED",
                    "simulation worker was finalized more than once",
                ))),
                diagnostic_metrics: None,
            };
        }
        let (completion_sender, completion_receiver) = mpsc::sync_channel(1);
        let shutdown = InteractiveSimulationMessageV1::Shutdown {
            platform_close_event: platform_close_event.map(Box::new),
            rendered_objects,
            completion_sender,
        };
        if self.send_message(shutdown).is_err() {
            return self.join_closed_worker();
        }
        match completion_receiver.recv() {
            Ok(InteractiveShutdownReplyV1::Retry(failure)) => {
                InteractiveWorkerFinalizationV1::Retry(failure)
            }
            Ok(InteractiveShutdownReplyV1::Closed) | Err(_) => self.join_closed_worker(),
        }
    }

    #[cfg(test)]
    pub(super) fn inject_fail_next_close_publication(
        &self,
    ) -> Result<(), InteractiveWorkerFailureV1> {
        let (acknowledged_sender, acknowledged_receiver) = mpsc::sync_channel(1);
        self.send_message(
            InteractiveSimulationMessageV1::InjectClosePublicationFailure {
                acknowledged_sender,
            },
        )?;
        acknowledged_receiver.recv().map_err(|_| {
            InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "simulation worker stopped before acknowledging close fault injection",
            )
        })
    }

    fn join_closed_worker(&mut self) -> InteractiveWorkerFinalizationV1 {
        let Some(worker) = self.worker.take() else {
            return InteractiveWorkerFinalizationV1::Closed {
                result: Box::new(Err(InteractiveWorkerFailureV1::runtime(
                    "SESSION_RUNTIME_FAILED",
                    "simulation worker was finalized more than once",
                ))),
                diagnostic_metrics: None,
            };
        };
        match worker.join() {
            Ok(exit) => InteractiveWorkerFinalizationV1::Closed {
                result: Box::new(exit.result),
                diagnostic_metrics: exit.diagnostic_metrics,
            },
            Err(_) => InteractiveWorkerFinalizationV1::Closed {
                result: Box::new(Err(InteractiveWorkerFailureV1::runtime(
                    "SESSION_RUNTIME_FAILED",
                    "simulation worker panicked during the interactive session",
                ))),
                diagnostic_metrics: None,
            },
        }
    }

    fn disconnected_failure(&self) -> InteractiveWorkerFailureV1 {
        self.try_take_failure().unwrap_or_else(|| {
            InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "simulation worker stopped before accepting a frame batch",
            )
        })
    }

    fn send_message(
        &self,
        message: InteractiveSimulationMessageV1,
    ) -> Result<(), InteractiveWorkerFailureV1> {
        match &self.queue_telemetry {
            Some(telemetry) => {
                let result = telemetry.send(&self.work_sender, message);
                if result.is_err() {
                    telemetry.mark_producer_closed();
                }
                result
            }
            None => self
                .work_sender
                .send(message)
                .map_err(|_| self.disconnected_failure()),
        }
    }

    fn disconnect_producer(&mut self) {
        if let Some(telemetry) = &self.queue_telemetry {
            telemetry.mark_producer_closed();
        }
        let (disconnected_sender, disconnected_receiver) = mpsc::sync_channel(0);
        drop(disconnected_receiver);
        let original_sender = std::mem::replace(&mut self.work_sender, disconnected_sender);
        drop(original_sender);
    }

    pub(super) fn disconnect_and_join(&mut self) -> InteractiveWorkerFinalizationV1 {
        self.disconnect_producer();
        self.join_closed_worker()
    }
}

impl Drop for InteractiveSimulationWorkerV1 {
    fn drop(&mut self) {
        self.disconnect_producer();
        let _detached_worker = self.worker.take();
    }
}

impl InteractiveSimulationMessageV1 {
    pub(super) fn callback_sequence(&self) -> Option<u64> {
        match self {
            Self::Advance {
                callback_sequence, ..
            } => Some(*callback_sequence),
            Self::Shutdown { .. } => None,
            #[cfg(test)]
            Self::InjectClosePublicationFailure { .. } => None,
        }
    }

    fn mark_enqueued_now(&mut self) {
        if let Self::Advance { enqueued_at, .. } = self {
            *enqueued_at = Some(Instant::now());
        }
    }
}

impl QueueTelemetryV1 {
    pub(super) fn send(
        &self,
        sender: &SyncSender<InteractiveSimulationMessageV1>,
        mut message: InteractiveSimulationMessageV1,
    ) -> Result<(), InteractiveWorkerFailureV1> {
        let mut state = self.state.lock().map_err(|_| queue_telemetry_poisoned())?;
        while state.queued_messages == INTERACTIVE_SIMULATION_QUEUE_CAPACITY && !state.worker_closed
        {
            state = self
                .not_full
                .wait(state)
                .map_err(|_| queue_telemetry_poisoned())?;
        }
        if state.worker_closed || state.producer_closed {
            return Err(InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "simulation worker stopped before accepting a frame batch",
            ));
        }
        let callback_sequence = message.callback_sequence();
        if callback_sequence.is_some_and(|sequence| sequence != state.submitted_callbacks) {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker queue submission is not FIFO",
            ));
        }
        message.mark_enqueued_now();
        match sender.try_send(message) {
            Ok(()) => {
                if callback_sequence.is_some() {
                    state.submitted_callbacks = state.submitted_callbacks.saturating_add(1);
                }
                state.queued_messages = state.queued_messages.saturating_add(1);
                state.high_water = state.high_water.max(state.queued_messages);
                self.not_empty.notify_one();
                Ok(())
            }
            Err(mpsc::TrySendError::Full(_)) => Err(InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker channel occupancy diverged from queue telemetry",
            )),
            Err(mpsc::TrySendError::Disconnected(_)) => Err(InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "simulation worker stopped before accepting a frame batch",
            )),
        }
    }

    pub(super) fn recv(
        &self,
        receiver: &Receiver<InteractiveSimulationMessageV1>,
    ) -> Result<InteractiveSimulationMessageV1, InteractiveWorkerFailureV1> {
        let mut state = self.state.lock().map_err(|_| queue_telemetry_poisoned())?;
        while state.queued_messages == 0 && !state.producer_closed {
            state = self
                .not_empty
                .wait(state)
                .map_err(|_| queue_telemetry_poisoned())?;
        }
        if state.queued_messages == 0 {
            return Err(InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "interactive frame queue disconnected before shutdown",
            ));
        }
        let message = receiver.try_recv().map_err(|error| match error {
            mpsc::TryRecvError::Empty => InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker channel occupancy diverged from queue telemetry",
            ),
            mpsc::TryRecvError::Disconnected => InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "interactive frame queue disconnected before shutdown",
            ),
        })?;
        state.queued_messages = state.queued_messages.checked_sub(1).ok_or_else(|| {
            InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker queue depth underflowed",
            )
        })?;
        if let Some(sequence) = message.callback_sequence() {
            if sequence != state.dequeued_callbacks {
                return Err(InteractiveWorkerFailureV1::runtime(
                    "PERFORMANCE_MEASUREMENT_INVALID",
                    "production worker queue dequeue is not FIFO",
                ));
            }
            state.dequeued_callbacks = state.dequeued_callbacks.saturating_add(1);
        }
        self.not_full.notify_one();
        Ok(message)
    }

    pub(super) fn finish_snapshot(
        &self,
    ) -> Result<QueueTelemetrySnapshotV1, InteractiveWorkerFailureV1> {
        let state = self.state.lock().map_err(|_| queue_telemetry_poisoned())?;
        if state.queued_messages != 0 {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PERFORMANCE_MEASUREMENT_INVALID",
                "production worker queue was not empty at terminal shutdown",
            ));
        }
        Ok(QueueTelemetrySnapshotV1 {
            submitted_callbacks: state.submitted_callbacks,
            dequeued_callbacks: state.dequeued_callbacks,
            high_water: state.high_water,
        })
    }

    fn mark_producer_closed(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.producer_closed = true;
        self.not_empty.notify_all();
    }

    fn mark_worker_closed(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.worker_closed = true;
        self.not_full.notify_all();
    }
}

struct QueueWorkerLifecycleV1(Option<Arc<QueueTelemetryV1>>);

impl Drop for QueueWorkerLifecycleV1 {
    fn drop(&mut self) {
        if let Some(telemetry) = &self.0 {
            telemetry.mark_worker_closed();
        }
    }
}

fn receive_interactive_message(
    receiver: &Receiver<InteractiveSimulationMessageV1>,
    queue_telemetry: Option<&QueueTelemetryV1>,
) -> Result<InteractiveSimulationMessageV1, InteractiveWorkerFailureV1> {
    match queue_telemetry {
        Some(telemetry) => telemetry.recv(receiver),
        None => receiver.recv().map_err(|_| {
            InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "interactive frame queue disconnected before shutdown",
            )
        }),
    }
}

struct WorkerMetricBuffersV1 {
    message_age_samples: Vec<InteractiveWorkerMessageAgeSampleV1>,
    fixed_step_samples: Vec<InteractiveWorkerFixedStepSampleV1>,
    publication_samples: Vec<InteractiveWorkerPublicationSampleV1>,
    processed_callbacks: u64,
    ordinary_fixed_steps: u64,
    checkpoint_fixed_steps: u64,
    snapshot_publications: u64,
    reordered_callbacks: u64,
    expected_callback_sequence: u64,
}

impl WorkerMetricBuffersV1 {
    fn new(callback_capacity: usize) -> Self {
        Self {
            message_age_samples: Vec::with_capacity(callback_capacity),
            fixed_step_samples: Vec::with_capacity(callback_capacity.saturating_mul(8)),
            publication_samples: Vec::with_capacity(callback_capacity.saturating_add(1)),
            processed_callbacks: 0,
            ordinary_fixed_steps: 0,
            checkpoint_fixed_steps: 0,
            snapshot_publications: 0,
            reordered_callbacks: 0,
            expected_callback_sequence: 0,
        }
    }

    fn finish(
        self,
        submitted_callbacks: u64,
        queue_high_water: usize,
    ) -> InteractiveWorkerDiagnosticMetricsV1 {
        let fixed_steps = self
            .ordinary_fixed_steps
            .saturating_add(self.checkpoint_fixed_steps);
        InteractiveWorkerDiagnosticMetricsV1 {
            submitted_callbacks,
            processed_callbacks: self.processed_callbacks,
            fixed_steps,
            ordinary_fixed_steps: self.ordinary_fixed_steps,
            checkpoint_fixed_steps: self.checkpoint_fixed_steps,
            snapshot_publications: self.snapshot_publications,
            dropped_callbacks: submitted_callbacks.saturating_sub(self.processed_callbacks),
            reordered_callbacks: self.reordered_callbacks,
            queue_high_water,
            send_wait_samples: Vec::with_capacity(0),
            message_age_samples: self.message_age_samples,
            fixed_step_samples: self.fixed_step_samples,
            publication_samples: self.publication_samples,
            snapshot_read_samples: Vec::with_capacity(0),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_interactive_simulation_session_worker(
    launch: LaunchRequestV1,
    capabilities: PlatformCapabilitySetV1,
    work_receiver: Receiver<InteractiveSimulationMessageV1>,
    latest_snapshot: Arc<RwLock<Option<InteractivePublishedSnapshotV1>>>,
    ready_sender: SyncSender<Result<InteractiveWorkerReadyV1, InteractiveWorkerFailureV1>>,
    failure_sender: SyncSender<InteractiveWorkerFailureV1>,
    processed_callbacks: Option<Arc<AtomicU64>>,
    queue_telemetry: Option<Arc<QueueTelemetryV1>>,
    diagnostic_capacity: Option<usize>,
) -> InteractiveWorkerExitV1 {
    let _queue_lifecycle = QueueWorkerLifecycleV1(queue_telemetry.clone());
    let mut metrics = diagnostic_capacity.map(WorkerMetricBuffersV1::new);
    let prepared =
        prepare_interactive_worker(launch, &capabilities, &latest_snapshot, metrics.as_mut());
    let (mut application, ready) = match prepared {
        Ok(prepared) => prepared,
        Err(failure) => {
            let _ = ready_sender.send(Err(failure.clone()));
            return InteractiveWorkerExitV1 {
                result: Err(failure),
                diagnostic_metrics: None,
            };
        }
    };
    if ready_sender.send(Ok(ready)).is_err() {
        let result = finish_interactive_worker(&mut application, None, 0).and(Err(
            InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "interactive host stopped before receiving worker readiness",
            ),
        ));
        return worker_exit(result, metrics, queue_telemetry.as_deref());
    }

    let mut fixed_step = FixedStepLiveSchedulerV1::reference_game_v1();
    let mut pending_failure = None;
    loop {
        let message = match receive_interactive_message(&work_receiver, queue_telemetry.as_deref())
        {
            Ok(message) => message,
            Err(receive_failure) => {
                let failure = pending_failure.unwrap_or(receive_failure);
                let result = finish_interactive_worker(&mut application, None, 0).and(Err(failure));
                return worker_exit(result, metrics, queue_telemetry.as_deref());
            }
        };
        match message {
            InteractiveSimulationMessageV1::Advance {
                callback_sequence,
                enqueued_at,
                elapsed,
                events,
            } => {
                if let Some(metrics) = metrics.as_mut() {
                    if let Some(enqueued_at) = enqueued_at {
                        metrics
                            .message_age_samples
                            .push(InteractiveWorkerMessageAgeSampleV1 {
                                callback_sequence,
                                nanoseconds: enqueued_at.elapsed().as_nanos(),
                            });
                    }
                    if callback_sequence != metrics.expected_callback_sequence {
                        metrics.reordered_callbacks = metrics.reordered_callbacks.saturating_add(1);
                    }
                    metrics.expected_callback_sequence = callback_sequence.saturating_add(1);
                }
                if pending_failure.is_none() {
                    let result = if let Some(metrics) = metrics.as_mut() {
                        fixed_step.advance_reference_game_presentation_shared_observed(
                            &mut application,
                            elapsed,
                            &events,
                            |simulation_tick, published_checkpoint, duration| {
                                let class = if published_checkpoint {
                                    metrics.checkpoint_fixed_steps =
                                        metrics.checkpoint_fixed_steps.saturating_add(1);
                                    InteractiveWorkerFixedStepClassV1::Checkpoint
                                } else {
                                    metrics.ordinary_fixed_steps =
                                        metrics.ordinary_fixed_steps.saturating_add(1);
                                    InteractiveWorkerFixedStepClassV1::Ordinary
                                };
                                metrics.fixed_step_samples.push(
                                    InteractiveWorkerFixedStepSampleV1 {
                                        callback_sequence,
                                        simulation_tick,
                                        class,
                                        nanoseconds: duration.as_nanos(),
                                    },
                                );
                            },
                        )
                    } else {
                        fixed_step.advance_reference_game_presentation_shared(
                            &mut application,
                            elapsed,
                            &events,
                        )
                    }
                    .map_err(InteractiveWorkerFailureV1::application);
                    match result {
                        Ok(Some(next_snapshot)) => {
                            let publication_started = metrics.as_ref().map(|_| Instant::now());
                            match latest_snapshot.write() {
                                Ok(mut latest) => {
                                    let lock_wait = publication_started
                                        .map_or(Duration::ZERO, |started| started.elapsed());
                                    if let Some(metrics) = metrics.as_mut() {
                                        metrics.snapshot_publications =
                                            metrics.snapshot_publications.saturating_add(1);
                                        metrics.publication_samples.push(
                                            InteractiveWorkerPublicationSampleV1 {
                                                callback_sequence: Some(callback_sequence),
                                                simulation_tick: next_snapshot.simulation_tick,
                                                lock_wait_nanoseconds: lock_wait.as_nanos(),
                                            },
                                        );
                                    }
                                    *latest = Some(InteractivePublishedSnapshotV1 {
                                        snapshot: next_snapshot,
                                        callback_sequence: Some(callback_sequence),
                                    });
                                }
                                Err(_) => record_interactive_worker_failure(
                                    &mut pending_failure,
                                    &failure_sender,
                                    InteractiveWorkerFailureV1::runtime(
                                        "PLATFORM_PRESENTATION_STATE_POISONED",
                                        "latest presentation snapshot lock was poisoned",
                                    ),
                                ),
                            }
                        }
                        Ok(None) => {}
                        Err(failure) => record_interactive_worker_failure(
                            &mut pending_failure,
                            &failure_sender,
                            failure,
                        ),
                    }
                }
                if let Some(metrics) = metrics.as_mut() {
                    metrics.processed_callbacks = metrics.processed_callbacks.saturating_add(1);
                }
                if let Some(processed_callbacks) = &processed_callbacks {
                    processed_callbacks.fetch_add(1, Ordering::Release);
                }
            }
            InteractiveSimulationMessageV1::Shutdown {
                platform_close_event,
                rendered_objects,
                completion_sender,
            } => {
                let report = finish_interactive_worker(
                    &mut application,
                    platform_close_event.as_deref(),
                    rendered_objects,
                );
                match resolve_interactive_shutdown_attempt(report, pending_failure.as_ref()) {
                    InteractiveShutdownAttemptV1::Closed(result) => {
                        let _ = completion_sender.send(InteractiveShutdownReplyV1::Closed);
                        return worker_exit(result, metrics, queue_telemetry.as_deref());
                    }
                    InteractiveShutdownAttemptV1::Retry(failure) => {
                        let _ = completion_sender.send(InteractiveShutdownReplyV1::Retry(failure));
                    }
                }
            }
            #[cfg(test)]
            InteractiveSimulationMessageV1::InjectClosePublicationFailure {
                acknowledged_sender,
            } => {
                application.inject_fail_next_state_publication();
                let _ = acknowledged_sender.send(());
            }
        }
    }
}

fn prepare_interactive_worker(
    launch: LaunchRequestV1,
    capabilities: &PlatformCapabilitySetV1,
    latest_snapshot: &Arc<RwLock<Option<InteractivePublishedSnapshotV1>>>,
    metrics: Option<&mut WorkerMetricBuffersV1>,
) -> Result<(ApplicationCoordinator, InteractiveWorkerReadyV1), InteractiveWorkerFailureV1> {
    let mut application = ApplicationCoordinator::launch_or_resume(launch)
        .map_err(InteractiveWorkerFailureV1::application)?;
    eprintln!(
        "next_game: session {} active",
        application.state().session_id.to_hex()
    );
    let resume_suspended_application =
        application.state().state == ApplicationSessionStatusV1::Suspended;
    let run = begin_or_resume_reference_game_live(&mut application)?;
    let initial_snapshot = Arc::new(run.presentation_snapshot.ok_or_else(|| {
        InteractiveWorkerFailureV1::runtime(
            "PLATFORM_PRESENTATION_SNAPSHOT_MISSING",
            "interactive target produced no presentation snapshot",
        )
    })?);
    let render_content_catalog = application
        .activated_project()
        .render_content_catalog
        .clone();
    let host_instance_id = application
        .register_platform_host(capabilities)
        .map_err(InteractiveWorkerFailureV1::application)?;
    let publication_started = metrics.as_ref().map(|_| Instant::now());
    let mut latest = latest_snapshot.write().map_err(|_| {
        InteractiveWorkerFailureV1::runtime(
            "PLATFORM_PRESENTATION_STATE_POISONED",
            "latest presentation snapshot lock was poisoned",
        )
    })?;
    let lock_wait = publication_started.map_or(Duration::ZERO, |started| started.elapsed());
    *latest = Some(InteractivePublishedSnapshotV1 {
        snapshot: Arc::clone(&initial_snapshot),
        callback_sequence: None,
    });
    drop(latest);
    if let Some(metrics) = metrics {
        metrics.snapshot_publications = metrics.snapshot_publications.saturating_add(1);
        metrics
            .publication_samples
            .push(InteractiveWorkerPublicationSampleV1 {
                callback_sequence: None,
                simulation_tick: initial_snapshot.simulation_tick,
                lock_wait_nanoseconds: lock_wait.as_nanos(),
            });
    }
    Ok((
        application,
        InteractiveWorkerReadyV1 {
            initial_snapshot,
            render_content_catalog,
            host_instance_id,
            resume_suspended_application,
        },
    ))
}

fn begin_or_resume_reference_game_live(
    application: &mut ApplicationCoordinator,
) -> Result<crate::ApplicationRunOutcomeV1, InteractiveWorkerFailureV1> {
    match application.current_live_run() {
        Ok(run) => Ok(run),
        Err(ApplicationError::NoLiveRun) => application
            .begin_reference_game_live(true)
            .map_err(InteractiveWorkerFailureV1::application),
        Err(error) => Err(InteractiveWorkerFailureV1::application(error)),
    }
}

fn record_interactive_worker_failure(
    pending_failure: &mut Option<InteractiveWorkerFailureV1>,
    failure_sender: &SyncSender<InteractiveWorkerFailureV1>,
    failure: InteractiveWorkerFailureV1,
) {
    if pending_failure.is_none() {
        let _ = failure_sender.try_send(failure.clone());
        *pending_failure = Some(failure);
    }
}

fn finish_interactive_worker(
    application: &mut ApplicationCoordinator,
    platform_close_event: Option<&PlatformEventV1>,
    rendered_objects: u64,
) -> Result<RunReportV1, InteractiveWorkerFailureV1> {
    let run = application
        .current_live_run()
        .map_err(InteractiveWorkerFailureV1::application)?;
    let close_options = CloseExecutionOptionsV1::default();
    let close = if let Some(event) = platform_close_event {
        application.close_from_platform_event(event, close_options)
    } else {
        application.close(close_options)
    }
    .map_err(InteractiveWorkerFailureV1::application)?;
    if !matches!(close, ApplicationCloseOutcomeV1::Closed { .. }) {
        return Err(InteractiveWorkerFailureV1::runtime(
            "SESSION_FINAL_SAVE_FAILED",
            "application close did not reach a terminal receipt",
        ));
    }
    RunReportV1::new(CompositionRootV1::Game, &run, &close, rendered_objects).ok_or_else(|| {
        InteractiveWorkerFailureV1::runtime(
            "SESSION_TERMINAL_RECEIPT_MISSING",
            "closed application has no terminal receipt",
        )
    })
}

pub(super) fn resolve_interactive_shutdown_attempt<T>(
    close_result: Result<T, InteractiveWorkerFailureV1>,
    pending_failure: Option<&InteractiveWorkerFailureV1>,
) -> InteractiveShutdownAttemptV1<T> {
    match close_result {
        Ok(value) => {
            InteractiveShutdownAttemptV1::Closed(pending_failure.cloned().map_or(Ok(value), Err))
        }
        Err(failure) => InteractiveShutdownAttemptV1::Retry(failure),
    }
}

fn worker_exit(
    mut result: Result<RunReportV1, InteractiveWorkerFailureV1>,
    metrics: Option<WorkerMetricBuffersV1>,
    queue_telemetry: Option<&QueueTelemetryV1>,
) -> InteractiveWorkerExitV1 {
    let diagnostic_metrics = metrics.and_then(|metrics| {
        let snapshot = queue_telemetry
            .ok_or_else(|| {
                InteractiveWorkerFailureV1::runtime(
                    "PERFORMANCE_MEASUREMENT_INVALID",
                    "production worker returned no queue telemetry",
                )
            })
            .and_then(QueueTelemetryV1::finish_snapshot)
            .and_then(|snapshot| {
                if snapshot.dequeued_callbacks != metrics.processed_callbacks {
                    return Err(InteractiveWorkerFailureV1::runtime(
                        "PERFORMANCE_MEASUREMENT_INVALID",
                        "production worker queue dequeue count did not match processed callbacks",
                    ));
                }
                Ok(snapshot)
            });
        match snapshot {
            Ok(snapshot) => Some(metrics.finish(snapshot.submitted_callbacks, snapshot.high_water)),
            Err(failure) => {
                result = Err(failure);
                None
            }
        }
    });
    InteractiveWorkerExitV1 {
        result,
        diagnostic_metrics,
    }
}

fn join_failed_worker(worker: JoinHandle<InteractiveWorkerExitV1>) -> InteractiveWorkerFailureV1 {
    match worker.join() {
        Ok(exit) => exit.result.err().unwrap_or_else(|| {
            InteractiveWorkerFailureV1::runtime(
                "SESSION_RUNTIME_FAILED",
                "simulation worker exited before reporting readiness",
            )
        }),
        Err(_) => InteractiveWorkerFailureV1::runtime(
            "SESSION_RUNTIME_FAILED",
            "simulation worker panicked before reporting readiness",
        ),
    }
}

fn callback_sequence_overflow(message: &'static str) -> InteractiveWorkerFailureV1 {
    InteractiveWorkerFailureV1::runtime("SESSION_RUNTIME_FAILED", message)
}

fn queue_telemetry_poisoned() -> InteractiveWorkerFailureV1 {
    InteractiveWorkerFailureV1::runtime(
        "PERFORMANCE_MEASUREMENT_INVALID",
        "production worker queue telemetry lock was poisoned",
    )
}
