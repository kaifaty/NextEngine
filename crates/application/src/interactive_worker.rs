use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::platform::{PlatformCapabilitySetV1, PlatformEventV1};
use next_contracts::presentation::PresentationSnapshotV3;
use next_contracts::render_content::RenderContentCatalogV1;
use next_contracts::session::{
    ApplicationSessionStatusV1, CompositionRootV1, PresentationTargetKindV1,
};

use crate::{
    ApplicationCloseOutcomeV2, ApplicationCoordinator, ApplicationError, FixedStepLiveSchedulerV1,
    LaunchRequestV1, RunReportV1,
};

pub const INTERACTIVE_SIMULATION_QUEUE_CAPACITY: usize = 8;
const INTERACTIVE_PRESENTATION_BOUNDARY_CAPACITY: usize = INTERACTIVE_SIMULATION_QUEUE_CAPACITY;
pub const PRODUCTION_WORKER_DIAGNOSTIC_MINIMUM_CALLBACKS: u64 = 240;
const MAXIMUM_DIAGNOSTIC_CALLBACKS: u64 = 1_000_000;
const MAXIMUM_SINGLE_STEP_CALLBACK_ELAPSED: Duration = Duration::from_nanos(33_333_334);
const MAXIMUM_DIAGNOSTIC_FINALIZATION_ATTEMPTS: usize = 8;

fn allocate_diagnostic_sample_buffer<T>(
    capacity: usize,
) -> Result<Vec<T>, InteractiveWorkerFailureV1> {
    let mut samples = Vec::new();
    samples.try_reserve_exact(capacity).map_err(|_| {
        InteractiveWorkerFailureV1::runtime(
            "PERFORMANCE_SCENARIO_INVALID",
            "production worker diagnostic sample reservation failed",
        )
    })?;
    Ok(samples)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerFailureV1 {
    pub code: &'static str,
    pub message: String,
    pub exit_code: i32,
}

impl InteractiveWorkerFailureV1 {
    #[must_use]
    pub fn runtime(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            exit_code: 2,
        }
    }

    fn application(error: ApplicationError) -> Self {
        Self {
            code: error.diagnostic_code(),
            message: error.to_string(),
            exit_code: 1,
        }
    }
}

impl Display for InteractiveWorkerFailureV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for InteractiveWorkerFailureV1 {}

impl From<ApplicationError> for InteractiveWorkerFailureV1 {
    fn from(value: ApplicationError) -> Self {
        Self::application(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveWorkerFixedStepClassV1 {
    Ordinary,
    LifecycleBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerTimingSampleV1 {
    pub callback_sequence: u64,
    pub nanoseconds: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerMessageAgeSampleV1 {
    pub callback_sequence: u64,
    pub nanoseconds: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerFixedStepSampleV1 {
    pub callback_sequence: u64,
    pub simulation_tick: u64,
    pub class: InteractiveWorkerFixedStepClassV1,
    pub nanoseconds: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerPublicationSampleV1 {
    pub callback_sequence: Option<u64>,
    pub simulation_tick: u64,
    pub lock_wait_nanoseconds: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerSnapshotReadSampleV1 {
    pub callback_sequence: u64,
    pub lock_wait_nanoseconds: u128,
    pub processed_callbacks: u64,
    pub sequence_lag: u64,
    pub publication_callback_sequence: Option<u64>,
    pub snapshot_sequence: u64,
    pub simulation_tick: u64,
    pub fresh_generation: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerDiagnosticMetricsV1 {
    pub submitted_callbacks: u64,
    pub processed_callbacks: u64,
    pub fixed_steps: u64,
    pub ordinary_fixed_steps: u64,
    pub lifecycle_boundary_fixed_steps: u64,
    pub snapshot_publications: u64,
    pub dropped_callbacks: u64,
    pub reordered_callbacks: u64,
    pub queue_high_water: usize,
    pub send_wait_samples: Vec<InteractiveWorkerTimingSampleV1>,
    pub message_age_samples: Vec<InteractiveWorkerMessageAgeSampleV1>,
    pub fixed_step_samples: Vec<InteractiveWorkerFixedStepSampleV1>,
    pub publication_samples: Vec<InteractiveWorkerPublicationSampleV1>,
    pub snapshot_read_samples: Vec<InteractiveWorkerSnapshotReadSampleV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerDiagnosticOptionsV1 {
    pub launch: LaunchRequestV1,
    pub callback_count: u64,
    pub callback_elapsed: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveWorkerDiagnosticReportV1 {
    pub callback_count: u64,
    pub callback_elapsed: Duration,
    pub initial_snapshot_epoch: ContentHash,
    pub initial_snapshot_sequence: u64,
    pub initial_simulation_tick: u64,
    pub final_snapshot_epoch: ContentHash,
    pub final_snapshot_sequence: u64,
    pub final_simulation_tick: u64,
    pub run_report: RunReportV1,
    pub metrics: InteractiveWorkerDiagnosticMetricsV1,
}

#[derive(Clone, Debug)]
pub struct InteractiveWorkerReadyV1 {
    pub initial_snapshot: Arc<PresentationSnapshotV3>,
    pub render_content_catalog: RenderContentCatalogV1,
    pub text_catalogs: Vec<next_contracts::localization::TextCatalogV1>,
    pub host_instance_id: PersistentId,
    pub resume_suspended_application: bool,
}

#[derive(Clone, Debug)]
pub struct InteractiveMainSubmitV1 {
    pub callback_sequence: u64,
    pub send_wait: Duration,
}

#[derive(Clone, Debug)]
pub struct InteractiveMainSnapshotReadV1 {
    pub snapshot: Arc<PresentationSnapshotV3>,
    pub lock_wait: Duration,
    pub processed_callbacks: u64,
    pub publication_callback_sequence: Option<u64>,
}

#[derive(Debug)]
pub enum InteractiveWorkerFinalizationV1 {
    Retry(InteractiveWorkerFailureV1),
    Closed {
        result: Box<Result<RunReportV1, InteractiveWorkerFailureV1>>,
        diagnostic_metrics: Option<InteractiveWorkerDiagnosticMetricsV1>,
    },
}

#[derive(Clone)]
struct InteractivePublishedSnapshotV1 {
    snapshot: Arc<PresentationSnapshotV3>,
    callback_sequence: Option<u64>,
}

/// One atomic handoff between the simulation worker and desktop consumer.
/// Ordinary presentation generations are latest-wins, while recovery cuts
/// are lossless and must be observed before any later generation from their
/// epoch. Keeping both classes under one lock prevents a reader from racing
/// between a boundary enqueue and a later latest-snapshot replacement.
#[derive(Default)]
struct InteractivePresentationMailboxV1 {
    latest: Option<InteractivePublishedSnapshotV1>,
    required_recovery_boundaries: VecDeque<InteractivePublishedSnapshotV1>,
}

impl InteractivePresentationMailboxV1 {
    fn publish_latest(&mut self, published: InteractivePublishedSnapshotV1) {
        self.latest = Some(published);
    }

    fn publish_required_recovery_boundary(
        &mut self,
        published: InteractivePublishedSnapshotV1,
    ) -> Result<(), InteractiveWorkerFailureV1> {
        if published.snapshot.snapshot_sequence != 0
            || published
                .snapshot
                .camera_records()
                .any(|camera| !camera.cut)
        {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PLATFORM_PRESENTATION_RECOVERY_BOUNDARY_INVALID",
                "required presentation recovery boundary must be sequence zero with camera cuts",
            ));
        }
        if self.required_recovery_boundaries.len() >= INTERACTIVE_PRESENTATION_BOUNDARY_CAPACITY {
            return Err(InteractiveWorkerFailureV1::runtime(
                "PLATFORM_PRESENTATION_RECOVERY_BOUNDARY_BUDGET_EXCEEDED",
                "required presentation recovery boundary queue is full",
            ));
        }
        self.required_recovery_boundaries
            .push_back(published.clone());
        self.latest = Some(published);
        Ok(())
    }

    fn latest(&self) -> Option<InteractivePublishedSnapshotV1> {
        self.latest.clone()
    }

    fn next_for_consumer(&mut self) -> Option<InteractivePublishedSnapshotV1> {
        self.required_recovery_boundaries
            .pop_front()
            .or_else(|| self.latest.clone())
    }
}

pub struct InteractiveSimulationWorkerV1 {
    work_sender: SyncSender<InteractiveSimulationMessageV1>,
    failure_receiver: Receiver<InteractiveWorkerFailureV1>,
    presentation_mailbox: Arc<Mutex<InteractivePresentationMailboxV1>>,
    latest_audio: Arc<RwLock<Option<crate::ApplicationAudioFrameV1>>>,
    processed_callbacks: Option<Arc<AtomicU64>>,
    queue_telemetry: Option<Arc<QueueTelemetryV1>>,
    next_callback_sequence: u64,
    worker: Option<JoinHandle<InteractiveWorkerExitV1>>,
}

enum InteractiveSimulationMessageV1 {
    Advance {
        callback_sequence: u64,
        enqueued_at: Option<Instant>,
        elapsed: Duration,
        events: Vec<PlatformEventV1>,
    },
    Shutdown {
        platform_close_event: Option<Box<PlatformEventV1>>,
        rendered_objects: u64,
        completion_sender: SyncSender<InteractiveShutdownReplyV1>,
    },
}

enum InteractiveShutdownReplyV1 {
    Retry(InteractiveWorkerFailureV1),
    Closed,
}

struct InteractiveWorkerExitV1 {
    result: Result<RunReportV1, InteractiveWorkerFailureV1>,
    diagnostic_metrics: Option<InteractiveWorkerDiagnosticMetricsV1>,
}

#[derive(Default)]

struct QueueTelemetryV1 {
    state: Mutex<QueueTelemetryStateV1>,
    not_empty: Condvar,
    not_full: Condvar,
}

#[derive(Default)]

struct QueueTelemetryStateV1 {
    submitted_callbacks: u64,
    dequeued_callbacks: u64,
    queued_messages: usize,
    high_water: usize,
    producer_closed: bool,
    worker_closed: bool,
}

struct QueueTelemetrySnapshotV1 {
    submitted_callbacks: u64,
    dequeued_callbacks: u64,
    high_water: usize,
}

enum InteractiveShutdownAttemptV1<T> {
    Retry(InteractiveWorkerFailureV1),
    Closed(Result<T, InteractiveWorkerFailureV1>),
}

mod diagnostic;
mod pause_menu;
mod runtime;

mod audio;

pub use diagnostic::{
    PreparedProductionWorkerDiagnosticV1, ProductionWorkerDiagnosticMeasurementV1,
    prepare_production_worker_diagnostic, run_production_worker_diagnostic,
};

#[cfg(test)]
use diagnostic::finalize_diagnostic_worker;
#[cfg(test)]
use runtime::resolve_interactive_shutdown_attempt;
#[cfg(test)]
mod tests;
