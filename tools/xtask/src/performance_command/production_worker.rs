use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use xtask::report::ProductionWorkerPerformanceDetailsV1;

const PRODUCTION_WORKER_SOAK_FRAMES: u32 = 240;
const PRODUCTION_WORKER_SOAK_CADENCE_HZ: u32 = 60;
static PRODUCTION_WORKER_SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn run_production_worker_scenario(
    state_root: Option<&Path>,
) -> Result<next_application::InteractiveWorkerDiagnosticReportV1, String> {
    let scratch_root = create_production_worker_scratch_root(state_root)?;
    let callback_elapsed = Duration::from_nanos(
        1_000_000_000_u64.div_ceil(u64::from(PRODUCTION_WORKER_SOAK_CADENCE_HZ)),
    );
    let result = next_application::run_production_worker_diagnostic(
        next_application::InteractiveWorkerDiagnosticOptionsV1 {
            launch: next_application::LaunchRequestV1::reference(
                scratch_root.clone(),
                next_contracts::session::CompositionRootV1::Game,
                next_contracts::session::PresentationTargetKindV1::Interactive,
            ),
            callback_count: u64::from(PRODUCTION_WORKER_SOAK_FRAMES),
            callback_elapsed,
        },
    )
    .map_err(|error| format!("{}: {}", error.code, error));
    let cleanup = fs::remove_dir_all(&scratch_root).map_err(|error| {
        format!(
            "failed to remove production worker scratch root {}: {error}",
            scratch_root.display()
        )
    });
    match (result, cleanup) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

fn create_production_worker_scratch_root(state_root: Option<&Path>) -> Result<PathBuf, String> {
    const MAXIMUM_ATTEMPTS: u32 = 1_024;
    let base = state_root.map_or_else(std::env::temp_dir, Path::to_path_buf);
    fs::create_dir_all(&base).map_err(|error| {
        format!(
            "failed to create production worker scratch base {}: {error}",
            base.display()
        )
    })?;
    for _ in 0..MAXIMUM_ATTEMPTS {
        let sequence = PRODUCTION_WORKER_SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = base.join(format!(
            "nextengine-production-worker-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(format!(
                    "failed to create production worker scratch root {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Err("failed to reserve a unique production worker scratch root".to_owned())
}

fn smoke_authoritative_hashes(
    streaming: &next_verification::StreamingPerformanceReport,
    agent: &next_verification::AgentPlanningPerformanceReport,
    render: &next_verification::RenderFramePlanningPerformanceReport,
    live: &next_verification::LiveRuntimePerformanceReport,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("agent_plan".to_owned(), agent.final_plan_hash.to_hex()),
        (
            "live_runtime_state".to_owned(),
            live.final_state_root.to_hex(),
        ),
        (
            "render_frame_plan".to_owned(),
            render.frame_plan_hash.to_hex(),
        ),
        (
            "streaming_world".to_owned(),
            streaming.final_world_state_hash.to_hex(),
        ),
    ])
}

pub(super) fn scenario_authoritative_hashes(
    streaming: &next_verification::StreamingPerformanceReport,
    agent: &next_verification::AgentPlanningPerformanceReport,
    render: &next_verification::RenderFramePlanningPerformanceReport,
    live: &next_verification::LiveRuntimePerformanceReport,
    production_worker: Option<&next_application::InteractiveWorkerDiagnosticReportV1>,
) -> BTreeMap<String, String> {
    let mut hashes = smoke_authoritative_hashes(streaming, agent, render, live);
    if let Some(worker) = production_worker {
        hashes.insert(
            "production_worker_state".to_owned(),
            worker.run_report.authoritative_state_root.clone(),
        );
        hashes.insert(
            "production_worker_archive".to_owned(),
            worker.run_report.command_archive_root.clone(),
        );
        hashes.insert(
            "production_worker_identity_index".to_owned(),
            worker.run_report.command_identity_index_root.clone(),
        );
        hashes.insert(
            "production_worker_ledger".to_owned(),
            worker.run_report.command_ledger_hash.clone(),
        );
    }
    hashes
}

pub(super) fn append_production_worker_metrics(
    metrics: &mut Vec<xtask::performance::PerformanceMetricV1>,
    worker: &next_application::InteractiveWorkerDiagnosticReportV1,
) -> Result<(), String> {
    let timing_metric = |name: &str, samples: Vec<u64>| {
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("production-worker-soak.{name}"),
            "nanoseconds",
            samples,
            None,
        )
    };
    let nanoseconds = |value: u128| {
        u64::try_from(value)
            .map_err(|error| format!("production worker duration overflow: {error}"))
    };
    metrics.push(timing_metric(
        "main.queue-send-wait",
        worker
            .metrics
            .send_wait_samples
            .iter()
            .map(|sample| nanoseconds(sample.nanoseconds))
            .collect::<Result<Vec<_>, _>>()?,
    )?);
    metrics.push(timing_metric(
        "worker.message-age-at-dequeue",
        worker
            .metrics
            .message_age_samples
            .iter()
            .map(|sample| nanoseconds(sample.nanoseconds))
            .collect::<Result<Vec<_>, _>>()?,
    )?);
    metrics.push(timing_metric(
        "worker.fixed-step-ordinary",
        worker
            .metrics
            .fixed_step_samples
            .iter()
            .filter(|sample| {
                sample.class == next_application::InteractiveWorkerFixedStepClassV1::Ordinary
            })
            .map(|sample| nanoseconds(sample.nanoseconds))
            .collect::<Result<Vec<_>, _>>()?,
    )?);
    metrics.push(timing_metric(
        "worker.fixed-step-checkpoint",
        worker
            .metrics
            .fixed_step_samples
            .iter()
            .filter(|sample| {
                sample.class == next_application::InteractiveWorkerFixedStepClassV1::Checkpoint
            })
            .map(|sample| nanoseconds(sample.nanoseconds))
            .collect::<Result<Vec<_>, _>>()?,
    )?);
    metrics.push(timing_metric(
        "worker.snapshot-publication-lock-wait",
        worker
            .metrics
            .publication_samples
            .iter()
            .map(|sample| nanoseconds(sample.lock_wait_nanoseconds))
            .collect::<Result<Vec<_>, _>>()?,
    )?);
    metrics.push(timing_metric(
        "main.snapshot-read-lock-wait",
        worker
            .metrics
            .snapshot_read_samples
            .iter()
            .map(|sample| nanoseconds(sample.lock_wait_nanoseconds))
            .collect::<Result<Vec<_>, _>>()?,
    )?);
    metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
        "production-worker-soak.main.snapshot-sequence-lag",
        "callbacks",
        worker
            .metrics
            .snapshot_read_samples
            .iter()
            .map(|sample| sample.sequence_lag)
            .collect(),
        None,
    )?);
    metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
        "production-worker-soak.main.snapshot-fresh-generation",
        "boolean",
        worker
            .metrics
            .snapshot_read_samples
            .iter()
            .map(|sample| u64::from(sample.fresh_generation))
            .collect(),
        None,
    )?);

    let snapshot_reads = u64::try_from(worker.metrics.snapshot_read_samples.len())
        .map_err(|error| error.to_string())?;
    let fresh_snapshot_reads = u64::try_from(
        worker
            .metrics
            .snapshot_read_samples
            .iter()
            .filter(|sample| sample.fresh_generation)
            .count(),
    )
    .map_err(|error| error.to_string())?;
    for (name, value) in [
        (
            "queue-high-water",
            u64::try_from(worker.metrics.queue_high_water).map_err(|error| error.to_string())?,
        ),
        ("submitted-callbacks", worker.metrics.submitted_callbacks),
        ("processed-callbacks", worker.metrics.processed_callbacks),
        ("fixed-steps", worker.metrics.fixed_steps),
        ("ordinary-fixed-steps", worker.metrics.ordinary_fixed_steps),
        (
            "checkpoint-fixed-steps",
            worker.metrics.checkpoint_fixed_steps,
        ),
        (
            "snapshot-publications",
            worker.metrics.snapshot_publications,
        ),
        ("snapshot-reads", snapshot_reads),
        ("fresh-snapshot-reads", fresh_snapshot_reads),
        ("dropped-callbacks", worker.metrics.dropped_callbacks),
        ("reordered-callbacks", worker.metrics.reordered_callbacks),
    ] {
        metrics.push(xtask::performance::PerformanceMetricV1::from_samples(
            format!("production-worker-soak.{name}"),
            "count",
            vec![value],
            None,
        )?);
    }
    Ok(())
}

pub(super) fn production_worker_details(
    worker: &next_application::InteractiveWorkerDiagnosticReportV1,
) -> Result<ProductionWorkerPerformanceDetailsV1, String> {
    let snapshot_reads = u64::try_from(worker.metrics.snapshot_read_samples.len())
        .map_err(|error| error.to_string())?;
    let fresh_snapshot_reads = u64::try_from(
        worker
            .metrics
            .snapshot_read_samples
            .iter()
            .filter(|sample| sample.fresh_generation)
            .count(),
    )
    .map_err(|error| error.to_string())?;
    Ok(ProductionWorkerPerformanceDetailsV1 {
        diagnostic_schema_version: 1,
        diagnostic_methodology_version: "production-worker-soak.v1".to_owned(),
        callbacks: worker.callback_count,
        callback_cadence_hz: PRODUCTION_WORKER_SOAK_CADENCE_HZ,
        queue_capacity: u32::try_from(next_application::INTERACTIVE_SIMULATION_QUEUE_CAPACITY)
            .map_err(|error| error.to_string())?,
        queue_high_water: u64::try_from(worker.metrics.queue_high_water)
            .map_err(|error| error.to_string())?,
        submitted_callbacks: worker.metrics.submitted_callbacks,
        processed_callbacks: worker.metrics.processed_callbacks,
        fixed_steps: worker.metrics.fixed_steps,
        ordinary_fixed_steps: worker.metrics.ordinary_fixed_steps,
        checkpoint_fixed_steps: worker.metrics.checkpoint_fixed_steps,
        snapshot_publications: worker.metrics.snapshot_publications,
        snapshot_reads,
        fresh_snapshot_reads,
        dropped_callbacks: worker.metrics.dropped_callbacks,
        reordered_callbacks: worker.metrics.reordered_callbacks,
        final_snapshot_sequence: worker.final_snapshot_sequence,
        final_simulation_tick: worker.final_simulation_tick,
        authoritative_state_root: worker.run_report.authoritative_state_root.clone(),
        command_archive_root: worker.run_report.command_archive_root.clone(),
        command_identity_index_root: worker.run_report.command_identity_index_root.clone(),
        command_ledger_hash: worker.run_report.command_ledger_hash.clone(),
    })
}

pub(super) fn observed_worker_unowned_spans(
    worker: &next_application::InteractiveWorkerDiagnosticReportV1,
) -> u64 {
    let metrics = &worker.metrics;
    let sample_count = |length: usize| u64::try_from(length).unwrap_or(u64::MAX);
    [
        metrics
            .submitted_callbacks
            .abs_diff(sample_count(metrics.send_wait_samples.len())),
        metrics
            .processed_callbacks
            .abs_diff(sample_count(metrics.message_age_samples.len())),
        metrics
            .fixed_steps
            .abs_diff(sample_count(metrics.fixed_step_samples.len())),
        metrics
            .snapshot_publications
            .abs_diff(sample_count(metrics.publication_samples.len())),
        metrics
            .submitted_callbacks
            .abs_diff(sample_count(metrics.snapshot_read_samples.len())),
        metrics.fixed_steps.abs_diff(
            metrics
                .ordinary_fixed_steps
                .saturating_add(metrics.checkpoint_fixed_steps),
        ),
        metrics
            .snapshot_publications
            .abs_diff(metrics.fixed_steps.saturating_add(1)),
        metrics.reordered_callbacks,
    ]
    .into_iter()
    .fold(0_u64, u64::saturating_add)
}

pub(super) fn production_worker_reserved_bytes(
    worker: &next_application::InteractiveWorkerDiagnosticReportV1,
) -> Result<u64, String> {
    fn vector_bytes<T>(capacity: usize) -> Result<u64, String> {
        let bytes = capacity
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| "production worker sample reservation overflow".to_owned())?;
        u64::try_from(bytes).map_err(|error| error.to_string())
    }

    let metrics = &worker.metrics;
    [
        vector_bytes::<next_application::InteractiveWorkerTimingSampleV1>(
            metrics.send_wait_samples.capacity(),
        )?,
        vector_bytes::<next_application::InteractiveWorkerMessageAgeSampleV1>(
            metrics.message_age_samples.capacity(),
        )?,
        vector_bytes::<next_application::InteractiveWorkerFixedStepSampleV1>(
            metrics.fixed_step_samples.capacity(),
        )?,
        vector_bytes::<next_application::InteractiveWorkerPublicationSampleV1>(
            metrics.publication_samples.capacity(),
        )?,
        vector_bytes::<next_application::InteractiveWorkerSnapshotReadSampleV1>(
            metrics.snapshot_read_samples.capacity(),
        )?,
    ]
    .into_iter()
    .try_fold(0_u64, |total, value| {
        total
            .checked_add(value)
            .ok_or_else(|| "production worker sample reservation overflow".to_owned())
    })
}

pub(super) fn record_worker_owned_spans(
    enabled: bool,
    spans: &mut Vec<xtask::performance::PerformanceSpanV1>,
    dropped_spans: &mut u64,
    worker: &next_application::InteractiveWorkerDiagnosticReportV1,
) -> u128 {
    if !enabled {
        return 0;
    }
    let started = Instant::now();
    let metrics = &worker.metrics;
    let ordinary = metrics
        .fixed_step_samples
        .iter()
        .filter(|sample| {
            sample.class == next_application::InteractiveWorkerFixedStepClassV1::Ordinary
        })
        .map(|sample| sample.nanoseconds)
        .fold(0_u128, u128::saturating_add);
    let checkpoint = metrics
        .fixed_step_samples
        .iter()
        .filter(|sample| {
            sample.class == next_application::InteractiveWorkerFixedStepClassV1::Checkpoint
        })
        .map(|sample| sample.nanoseconds)
        .fold(0_u128, u128::saturating_add);
    let main_send = metrics
        .send_wait_samples
        .iter()
        .map(|sample| sample.nanoseconds)
        .fold(0_u128, u128::saturating_add);
    let main_read = metrics
        .snapshot_read_samples
        .iter()
        .map(|sample| sample.lock_wait_nanoseconds)
        .fold(0_u128, u128::saturating_add);
    let publication = metrics
        .publication_samples
        .iter()
        .map(|sample| sample.lock_wait_nanoseconds)
        .fold(0_u128, u128::saturating_add);
    for (category, thread_index, nanoseconds) in [
        ("runtime-stages", 0_u32, main_send),
        ("render-extraction", 0_u32, main_read),
        ("runtime-stages", 1_u32, ordinary),
        ("runtime-stages", 1_u32, checkpoint),
        ("render-extraction", 1_u32, publication),
    ] {
        if spans.len() < usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD).unwrap_or(0) {
            spans.push(xtask::performance::PerformanceSpanV1 {
                category: category.to_owned(),
                thread_index,
                duration_microseconds: u64::try_from(nanoseconds / 1_000).unwrap_or(u64::MAX),
            });
        } else {
            *dropped_spans = dropped_spans.saturating_add(1);
        }
    }
    started.elapsed().as_nanos()
}
