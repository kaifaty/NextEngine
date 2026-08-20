use std::collections::BTreeMap;
use std::time::Duration;

use super::*;

const R5_LOGICAL_ACCOUNTING_PROFILE: &[u8] =
    b"nextengine.performance.r5-physics-16-logical-accounting.v1";
const R5_CHECKPOINT_BYTES_PER_SLOT_MAX: u64 = 4 * 1024 * 1024;
const R5_LOGICAL_HOST_BYTES_PER_SLOT_MAX: u64 = 4 * 1024 * 1024;
const R5_PROCESS_PEAK_WORKING_SET_BYTES_MAX: u64 = 320 * 1024 * 1024;
const R5_REPLAY_PREFIX_OVERHEAD_BASIS_POINTS_MAX: u64 = 12_000;
const R5_RESTORE_P95_MICROSECONDS_MAX: u64 = 3_600_000;
const R5_RESTORE_P99_MICROSECONDS_MAX: u64 = 4_000_000;

#[derive(Clone, Copy)]
struct WorkerBudget {
    worker_count: u32,
    frame_p95_microseconds: u64,
    frame_p99_microseconds: u64,
    nanoseconds_per_physics_substep_max: u64,
    nanoseconds_per_motor_frame_max: u64,
    scaling_inefficiency_basis_points_max: Option<u64>,
}

const R5_WORKER_BUDGETS: [WorkerBudget; 3] = [
    WorkerBudget {
        worker_count: 1,
        frame_p95_microseconds: 20_000,
        frame_p99_microseconds: 25_000,
        nanoseconds_per_physics_substep_max: 250_000,
        nanoseconds_per_motor_frame_max: 1_000_000,
        scaling_inefficiency_basis_points_max: None,
    },
    WorkerBudget {
        worker_count: 4,
        frame_p95_microseconds: 6_000,
        frame_p99_microseconds: 8_000,
        nanoseconds_per_physics_substep_max: 83_334,
        nanoseconds_per_motor_frame_max: 333_334,
        scaling_inefficiency_basis_points_max: Some(3_500),
    },
    WorkerBudget {
        worker_count: 8,
        frame_p95_microseconds: 4_000,
        frame_p99_microseconds: 6_000,
        nanoseconds_per_physics_substep_max: 50_000,
        nanoseconds_per_motor_frame_max: 200_000,
        scaling_inefficiency_basis_points_max: Some(4_500),
    },
];

pub(super) fn performance_report(
    request: &PerformanceArguments,
    mut run: xtask::performance::PerformanceRunV6,
    project_composition_lock_hash: String,
    profiling_enabled: bool,
    compare_baseline: bool,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let profiler_control = profiling_enabled
        .then(next_motor::run_reference_humanoid_performance_v1)
        .transpose()
        .map_err(|error| error.to_string())?;

    let counters_before = xtask::performance::inspect_process_counters();
    let report =
        next_motor::run_reference_humanoid_performance_v1().map_err(|error| error.to_string())?;
    let mut resource_counters = xtask::performance::finish_process_counters(&counters_before);
    resource_counters.device_resident_bytes = Some(0);
    resource_counters.unavailable.retain(|diagnostic| {
        !diagnostic.starts_with("Vulkan timestamps require")
            && !diagnostic.starts_with("device residency requires")
    });

    let mut recorded_spans = if profiling_enabled {
        Vec::with_capacity(
            usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD)
                .map_err(|error| error.to_string())?,
        )
    } else {
        Vec::new()
    };
    let mut dropped_spans = 0;
    let mut instrumentation_overhead_nanoseconds = 0_u128;
    for worker in &report.worker_runs {
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                "physics-motor",
                Duration::from_micros(worker.elapsed_microseconds),
            ));
    }
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "physics-motor",
            Duration::from_micros(report.restore_wall_microseconds),
        ));
    let profiled_microseconds = recorded_spans
        .iter()
        .map(|span| span.duration_microseconds)
        .try_fold(0_u64, |total, value| total.checked_add(value))
        .ok_or_else(|| "profiled R5 physics duration overflow".to_owned())?;
    let authoritative_hash_parity = profiler_control
        .as_ref()
        .map(|control| control.authoritative_root == report.authoritative_root);
    let overhead_basis_points = profiler_control.as_ref().map(|_| {
        overhead_basis_points(instrumentation_overhead_nanoseconds, profiled_microseconds)
    });
    run.instrumentation = xtask::performance::PerformanceInstrumentationV1 {
        enabled: profiling_enabled,
        max_threads: 8,
        max_spans_per_thread: xtask::performance::MAX_SPANS_PER_THREAD,
        reserved_bytes: u64::try_from(
            recorded_spans.capacity()
                * std::mem::size_of::<xtask::performance::PerformanceSpanV1>(),
        )
        .map_err(|error| error.to_string())?,
        recorded_spans,
        dropped_spans,
        unowned_spans: 0,
        overhead_basis_points,
        authoritative_hash_parity,
    };

    let checkpoint_total = report
        .checkpoint_bytes_per_slot
        .iter()
        .try_fold(0_u64, |total, value| total.checked_add(*value))
        .ok_or_else(|| "R5 checkpoint charge overflow".to_owned())?;
    let motor_staging_per_slot = u64::try_from(
        next_motor::REFERENCE_HUMANOID_DOF
            * (1 + next_contracts::motor::STAGE0_SUBSTEPS)
            * std::mem::size_of::<i64>(),
    )
    .map_err(|error| error.to_string())?;
    let required_staging_bytes = motor_staging_per_slot
        .checked_mul(u64::from(report.slot_count))
        .ok_or_else(|| "R5 motor staging charge overflow".to_owned())?;
    let timing_sample_count = report
        .worker_runs
        .iter()
        .map(|worker| worker.lockstep_motor_frame_microseconds.len())
        .sum::<usize>()
        .checked_add(report.restore_microseconds_per_slot.len())
        .and_then(|count| count.checked_add(report.checkpoint_bytes_per_slot.len()))
        .ok_or_else(|| "R5 timing sample charge overflow".to_owned())?;
    let timing_sample_bytes = u64::try_from(
        timing_sample_count
            .checked_mul(std::mem::size_of::<u64>())
            .ok_or_else(|| "R5 timing sample charge overflow".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    let maximum_checkpoint_bytes = report
        .checkpoint_bytes_per_slot
        .iter()
        .copied()
        .max()
        .unwrap_or(0);
    let tooling_transient_bytes = timing_sample_bytes
        .checked_add(maximum_checkpoint_bytes)
        .ok_or_else(|| "R5 tooling charge overflow".to_owned())?;
    let logical_charges = xtask::performance::PerformanceLogicalResourceChargesV1::new(
        xtask::performance::sha256_hex(R5_LOGICAL_ACCOUNTING_PROFILE),
        checkpoint_total,
        required_staging_bytes,
        0,
        0,
        0,
        tooling_transient_bytes,
    )?;
    let logical_host_bytes_per_slot = logical_charges
        .total_host_charged_bytes
        .div_ceil(u64::from(report.slot_count));
    resource_counters.logical_resource_charges = Some(logical_charges);
    let process_peak_working_set_bytes = resource_counters
        .process_peak_working_set_bytes
        .ok_or_else(|| {
            "PERF_REQUIRED_COUNTER_MISSING: process_peak_working_set_bytes".to_owned()
        })?;
    run.resource_counters = resource_counters;

    let eight_worker = worker_run(&report, 8)?;
    let live_prefix_microseconds = u128::from(eight_worker.elapsed_microseconds)
        .saturating_mul(u128::from(report.replay_prefix_substeps_per_slot))
        .checked_div(u128::from(report.measured_substeps_per_slot))
        .ok_or_else(|| "R5 replay prefix ratio overflow".to_owned())?
        .max(1);
    let replay_prefix_overhead_basis_points = u64::try_from(
        u128::from(report.restore_wall_microseconds)
            .saturating_mul(10_000)
            .checked_div(live_prefix_microseconds)
            .ok_or_else(|| "R5 replay prefix ratio overflow".to_owned())?,
    )
    .map_err(|error| error.to_string())?;

    run.content_hash = xtask::performance::sha256_hex(
        format!(
            "nextengine.r5-physics-16.content.v1\0{project_composition_lock_hash}\0{}",
            run.scenario_hash
        )
        .as_bytes(),
    );
    run.scenario_hash = performance_scenario_hash(request.scenario);
    run.metrics = worker_metrics(&report)?;
    run.metrics.extend([
        metric(
            "r5-physics-16.restore-fresh-scene",
            "microseconds",
            report.restore_microseconds_per_slot.clone(),
            Some(budget(
                R5_RESTORE_P95_MICROSECONDS_MAX,
                R5_RESTORE_P99_MICROSECONDS_MAX,
            )),
        )?,
        metric(
            "r5-physics-16.checkpoint-bytes-per-slot",
            "bytes",
            report.checkpoint_bytes_per_slot.clone(),
            Some(budget(
                R5_CHECKPOINT_BYTES_PER_SLOT_MAX,
                R5_CHECKPOINT_BYTES_PER_SLOT_MAX,
            )),
        )?,
        metric(
            "r5-physics-16.replay-prefix-overhead",
            "basis-points",
            vec![replay_prefix_overhead_basis_points],
            Some(budget(
                R5_REPLAY_PREFIX_OVERHEAD_BASIS_POINTS_MAX,
                R5_REPLAY_PREFIX_OVERHEAD_BASIS_POINTS_MAX,
            )),
        )?,
        metric(
            "r5-physics-16.process-peak-working-set",
            "bytes",
            vec![process_peak_working_set_bytes],
            Some(budget(
                R5_PROCESS_PEAK_WORKING_SET_BYTES_MAX,
                R5_PROCESS_PEAK_WORKING_SET_BYTES_MAX,
            )),
        )?,
        metric(
            "r5-physics-16.logical-host-bytes-per-slot",
            "bytes",
            vec![logical_host_bytes_per_slot],
            Some(budget(
                R5_LOGICAL_HOST_BYTES_PER_SLOT_MAX,
                R5_LOGICAL_HOST_BYTES_PER_SLOT_MAX,
            )),
        )?,
    ]);
    run.authoritative_hashes =
        BTreeMap::from([("r5_physics".to_owned(), report.authoritative_root.to_hex())]);
    run.verdict = xtask::performance::aggregate_metric_verdict(&run.metrics);

    if !report.worker_root_parity {
        run.diagnostics
            .push("PERF_R5_WORKER_ROOT_DIVERGENCE".to_owned());
    }
    if let Err(error) = run.instrumentation.validate() {
        run.diagnostics.push(error);
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    let evidence_validation = if request.mode == xtask::performance::PerformanceModeV1::Gate {
        run.validate_hard_evidence()
    } else {
        run.validate_report_evidence()
    };
    if let Err(diagnostics) = evidence_validation {
        run.diagnostics.extend(diagnostics);
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    if !run.diagnostics.is_empty() {
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }

    if compare_baseline
        && run.verdict != xtask::performance::PerformanceVerdict::NotRun
        && let Some(path) = &request.baseline
    {
        match read_performance_baseline(path) {
            Ok(baseline) => {
                if let Err(diagnostics) =
                    xtask::performance::compare_metrics_to_baseline(&mut run, &baseline)
                {
                    run.diagnostics.extend(diagnostics);
                    run.verdict = xtask::performance::PerformanceVerdict::NotRun;
                }
            }
            Err(error) => {
                run.diagnostics
                    .push(format!("PERF_BASELINE_INVALID: {error}"));
                run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            }
        }
    }
    if request.mode == xtask::performance::PerformanceModeV1::Report
        && request.baseline.is_none()
        && run.verdict == xtask::performance::PerformanceVerdict::Pass
    {
        run.verdict = xtask::performance::PerformanceVerdict::ReportOnly;
    }

    let worker_runs = report
        .worker_runs
        .iter()
        .map(|worker| {
            Ok(R5PhysicsWorkerPerformanceDetailsV1 {
                worker_count: worker.worker_count,
                elapsed_microseconds: worker.elapsed_microseconds,
                aggregate_physics_substeps_per_second: worker.aggregate_physics_substeps_per_second,
                aggregate_motor_frames_per_second: worker.aggregate_motor_frames_per_second,
                scaling_efficiency_basis_points: worker.scaling_efficiency_basis_points,
                motor_frame_p95_microseconds: xtask::performance::nearest_rank_percentile(
                    &worker.lockstep_motor_frame_microseconds,
                    95,
                )?,
                motor_frame_p99_microseconds: xtask::performance::nearest_rank_percentile(
                    &worker.lockstep_motor_frame_microseconds,
                    99,
                )?,
                authoritative_root: worker.authoritative_root.to_hex(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let restore_p95_microseconds =
        xtask::performance::nearest_rank_percentile(&report.restore_microseconds_per_slot, 95)?;
    let restore_p99_microseconds =
        xtask::performance::nearest_rank_percentile(&report.restore_microseconds_per_slot, 99)?;
    let status = run.verdict.command_report_status();
    Ok(CommandReportV1::new(
        "performance",
        status,
        PerformanceDetailsV1 {
            run: Some(run),
            streaming: None,
            agent_planning: None,
            render_planning: None,
            live_runtime: None,
            production_worker: None,
            r4_100npc: None,
            r5_physics: Some(R5PhysicsPerformanceDetailsV1 {
                evidence_run_count: 1,
                slot_count: report.slot_count,
                degrees_of_freedom_per_slot: next_motor::REFERENCE_HUMANOID_DOF as u32,
                physics_hz: report.physics_hz,
                motor_hz: report.motor_hz,
                warmup_substeps_per_slot: report.warmup_substeps_per_slot,
                measured_substeps_per_slot: report.measured_substeps_per_slot,
                measured_motor_frames_per_slot: report.measured_motor_frames_per_slot,
                worker_runs,
                checkpoint_bytes_per_slot: report.checkpoint_bytes_per_slot,
                restore_microseconds_per_slot: report.restore_microseconds_per_slot,
                restore_p95_microseconds,
                restore_p99_microseconds,
                restore_wall_microseconds: report.restore_wall_microseconds,
                replay_prefix_substeps_per_slot: report.replay_prefix_substeps_per_slot,
                replay_prefix_overhead_basis_points,
                process_peak_working_set_bytes,
                logical_host_bytes_per_slot,
                authoritative_root: report.authoritative_root.to_hex(),
                worker_root_parity: report.worker_root_parity,
            }),
        },
    ))
}

fn metric(
    name: &str,
    unit: &str,
    samples: Vec<u64>,
    absolute_budget: Option<xtask::performance::PerformanceBudgetV1>,
) -> Result<xtask::performance::PerformanceMetricV1, String> {
    xtask::performance::PerformanceMetricV1::from_samples(name, unit, samples, absolute_budget)
}

fn worker_run(
    report: &next_motor::HumanoidPerformanceReportV1,
    worker_count: u32,
) -> Result<&next_motor::HumanoidWorkerPerformanceV1, String> {
    report
        .worker_runs
        .iter()
        .find(|worker| worker.worker_count == worker_count)
        .ok_or_else(|| format!("MOTOR_PERF_WORKER_EVIDENCE_MISSING: {worker_count}"))
}

fn worker_metrics(
    report: &next_motor::HumanoidPerformanceReportV1,
) -> Result<Vec<xtask::performance::PerformanceMetricV1>, String> {
    let aggregate_substeps = u64::from(report.slot_count)
        .checked_mul(report.measured_substeps_per_slot)
        .ok_or_else(|| "R5 aggregate substep count overflow".to_owned())?;
    let aggregate_motor_frames = u64::from(report.slot_count)
        .checked_mul(report.measured_motor_frames_per_slot)
        .ok_or_else(|| "R5 aggregate motor-frame count overflow".to_owned())?;
    let mut metrics = Vec::with_capacity(11);
    for worker_budget in R5_WORKER_BUDGETS {
        let worker = worker_run(report, worker_budget.worker_count)?;
        let prefix = format!("r5-physics-16.worker-{}", worker_budget.worker_count);
        metrics.push(metric(
            &format!("{prefix}.physics-motor-frame"),
            "microseconds",
            worker.lockstep_motor_frame_microseconds.clone(),
            Some(budget(
                worker_budget.frame_p95_microseconds,
                worker_budget.frame_p99_microseconds,
            )),
        )?);
        metrics.push(metric(
            &format!("{prefix}.physics-substep-cost"),
            "nanoseconds-per-physics-substep",
            vec![average_nanoseconds(
                worker.elapsed_microseconds,
                aggregate_substeps,
            )?],
            Some(budget(
                worker_budget.nanoseconds_per_physics_substep_max,
                worker_budget.nanoseconds_per_physics_substep_max,
            )),
        )?);
        metrics.push(metric(
            &format!("{prefix}.motor-frame-cost"),
            "nanoseconds-per-motor-frame",
            vec![average_nanoseconds(
                worker.elapsed_microseconds,
                aggregate_motor_frames,
            )?],
            Some(budget(
                worker_budget.nanoseconds_per_motor_frame_max,
                worker_budget.nanoseconds_per_motor_frame_max,
            )),
        )?);
        if let Some(maximum) = worker_budget.scaling_inefficiency_basis_points_max {
            metrics.push(metric(
                &format!("{prefix}.scaling-inefficiency"),
                "basis-points",
                vec![10_000_u64.saturating_sub(worker.scaling_efficiency_basis_points)],
                Some(budget(maximum, maximum)),
            )?);
        }
    }
    Ok(metrics)
}

fn average_nanoseconds(elapsed_microseconds: u64, completed: u64) -> Result<u64, String> {
    elapsed_microseconds
        .checked_mul(1_000)
        .and_then(|nanoseconds| nanoseconds.checked_div(completed))
        .ok_or_else(|| "R5 average duration overflow or zero completion count".to_owned())
}

const fn budget(p95_max: u64, p99_max: u64) -> xtask::performance::PerformanceBudgetV1 {
    xtask::performance::PerformanceBudgetV1 {
        p95_max: Some(p95_max),
        p99_max: Some(p99_max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_worker_budgets_cover_one_four_and_eight_workers() {
        assert_eq!(R5_WORKER_BUDGETS.map(|entry| entry.worker_count), [1, 4, 8]);
        assert_eq!(R5_WORKER_BUDGETS[2].frame_p95_microseconds, 4_000);
        assert_eq!(R5_WORKER_BUDGETS[2].frame_p99_microseconds, 6_000);
        assert_eq!(
            R5_WORKER_BUDGETS[2].scaling_inefficiency_basis_points_max,
            Some(4_500)
        );
    }

    #[test]
    fn reciprocal_throughput_cost_is_lower_when_work_finishes_faster() {
        let slower = average_nanoseconds(1_000, 10).expect("slower cost");
        let faster = average_nanoseconds(500, 10).expect("faster cost");
        assert_eq!(slower, 100_000);
        assert_eq!(faster, 50_000);
        assert!(faster < slower);
    }
}
