use std::collections::BTreeMap;
use std::time::Duration;

use super::*;

const R4_LOGICAL_ACCOUNTING_PROFILE: &[u8] =
    b"nextengine.performance.r4-100npc-logical-accounting.v1";
const R4_NAVIGATION_P95_MICROSECONDS_MAX: u64 = 1_250;
const R4_NAVIGATION_P99_MICROSECONDS_MAX: u64 = 1_500;
const R4_COGNITION_DISPATCH_P95_MICROSECONDS_MAX: u64 = 1_250;
const R4_COGNITION_DISPATCH_P99_MICROSECONDS_MAX: u64 = 1_500;
const R4_INTEGRATED_P95_MICROSECONDS_MAX: u64 = 8_000;
const R4_INTEGRATED_P99_MICROSECONDS_MAX: u64 = 12_000;

pub(super) fn performance_report(
    request: &PerformanceArguments,
    mut run: xtask::performance::PerformanceRunV6,
    project_composition_lock_hash: String,
    profiling_enabled: bool,
    compare_baseline: bool,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let profiler_control = profiling_enabled
        .then(next_verification::run_population_performance_check)
        .transpose()
        .map_err(|error| error.to_string())?;

    let counters_before = xtask::performance::inspect_process_counters();
    let report =
        next_verification::run_population_performance_check().map_err(|error| error.to_string())?;
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
    let mut dropped_spans = 0_u64;
    let navigation_elapsed = sample_duration(&report.navigation_tick_microseconds)?;
    let cognition_dispatch_elapsed = sample_duration(&report.cognition_dispatch_tick_microseconds)?;
    let world_services_elapsed = sample_duration(&report.world_services_tick_microseconds)?;
    let mut instrumentation_overhead_nanoseconds = record_performance_span(
        profiling_enabled,
        &mut recorded_spans,
        &mut dropped_spans,
        "navigation",
        navigation_elapsed,
    );
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "tier-cognition",
            cognition_dispatch_elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "runtime-stages",
            world_services_elapsed,
        ));
    let profiled_microseconds = recorded_spans
        .iter()
        .map(|span| span.duration_microseconds)
        .try_fold(0_u64, |total, value| total.checked_add(value))
        .ok_or_else(|| "profiled R4 population duration overflow".to_owned())?;
    let authoritative_hash_parity = profiler_control
        .as_ref()
        .map(|control| authoritative_parity(control, &report));
    let overhead_basis_points = profiler_control.as_ref().map(|_| {
        overhead_basis_points(instrumentation_overhead_nanoseconds, profiled_microseconds)
    });
    run.instrumentation = xtask::performance::PerformanceInstrumentationV1 {
        enabled: profiling_enabled,
        max_threads: 1,
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

    let timing_sample_count = report
        .navigation_tick_microseconds
        .len()
        .checked_add(report.cognition_dispatch_tick_microseconds.len())
        .ok_or_else(|| "R4 timing sample count overflow".to_owned())?
        .checked_add(report.world_services_tick_microseconds.len())
        .ok_or_else(|| "R4 timing sample count overflow".to_owned())?;
    let tooling_transient_bytes = u64::try_from(
        timing_sample_count
            .checked_mul(std::mem::size_of::<u64>())
            .ok_or_else(|| "R4 timing sample byte count overflow".to_owned())?,
    )
    .map_err(|error| error.to_string())?;
    resource_counters.logical_resource_charges = Some(
        xtask::performance::PerformanceLogicalResourceChargesV1::new(
            xtask::performance::sha256_hex(R4_LOGICAL_ACCOUNTING_PROFILE),
            report.final_population_snapshot_bytes,
            0,
            0,
            0,
            0,
            tooling_transient_bytes,
        )?,
    );
    run.resource_counters = resource_counters;
    run.content_hash = xtask::performance::sha256_hex(
        format!(
            "nextengine.r4-100npc.content.v1\0{project_composition_lock_hash}\0{}",
            run.scenario_hash
        )
        .as_bytes(),
    );
    run.scenario_hash = performance_scenario_hash(request.scenario);
    run.metrics = vec![
        metric(
            "r4-100npc.navigation-due-work",
            "microseconds",
            report.navigation_tick_microseconds.clone(),
            Some(budget(
                R4_NAVIGATION_P95_MICROSECONDS_MAX,
                R4_NAVIGATION_P99_MICROSECONDS_MAX,
            )),
        )?,
        metric(
            "r4-100npc.tier-cognition-due-work",
            "microseconds",
            report.cognition_dispatch_tick_microseconds.clone(),
            Some(budget(
                R4_COGNITION_DISPATCH_P95_MICROSECONDS_MAX,
                R4_COGNITION_DISPATCH_P99_MICROSECONDS_MAX,
            )),
        )?,
        metric(
            "r4-100npc.integrated-world-services-tick",
            "microseconds",
            report.world_services_tick_microseconds.clone(),
            Some(budget(
                R4_INTEGRATED_P95_MICROSECONDS_MAX,
                R4_INTEGRATED_P99_MICROSECONDS_MAX,
            )),
        )?,
        metric(
            "r4-100npc.maximum-due-queue-depth",
            "queries",
            vec![u64::from(report.maximum_queue_depth)],
            None,
        )?,
        metric(
            "r4-100npc.maximum-cognition-queue-depth",
            "work-items",
            vec![u64::from(report.maximum_cognition_queue_depth)],
            None,
        )?,
        metric(
            "r4-100npc.population-snapshot-bytes",
            "bytes",
            vec![report.final_population_snapshot_bytes],
            None,
        )?,
    ];
    run.authoritative_hashes = BTreeMap::from([
        (
            "r4_application_state".to_owned(),
            report.final_application_state_root.to_hex(),
        ),
        (
            "r4_command_ledger".to_owned(),
            report.final_command_ledger_hash.to_hex(),
        ),
        ("r4_due_trace".to_owned(), report.due_trace_root.to_hex()),
        (
            "r4_tier_cognition_trace".to_owned(),
            report.tier_cognition_trace_root.to_hex(),
        ),
        (
            "r4_activity_state".to_owned(),
            report.final_activity_state_hash.to_hex(),
        ),
        (
            "r4_agent_state".to_owned(),
            report.final_agent_state_hash.to_hex(),
        ),
        (
            "r4_memory_state".to_owned(),
            report.final_memory_state_hash.to_hex(),
        ),
        (
            "r4_population_state".to_owned(),
            report.final_population_state_hash.to_hex(),
        ),
    ]);
    run.verdict = xtask::performance::aggregate_metric_verdict(&run.metrics);

    if report.deferred_work != 0
        || report.dropped_work != 0
        || report.maximum_starvation_age_ticks != 0
        || report.fabricated_outcomes != 0
        || report.abstract_outcome_deferrals
            != report.tier_cognition_due_counts.abstract_maintenance
        || report.tier_cognition_due_counts.work_items != report.due_counts.queries
        || report.due_counts.queries
            != report
                .due_counts
                .active
                .saturating_add(report.due_counts.near)
                .saturating_add(report.due_counts.background)
    {
        run.diagnostics
            .push("PERF_R4_POPULATION_DUE_WORK_INCOMPLETE".to_owned());
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
    run.verdict =
        preserve_report_only_scenario_verdict(request.scenario, request.mode, run.verdict);

    let navigation_p95_microseconds =
        xtask::performance::nearest_rank_percentile(&report.navigation_tick_microseconds, 95)?;
    let navigation_p99_microseconds =
        xtask::performance::nearest_rank_percentile(&report.navigation_tick_microseconds, 99)?;
    let cognition_dispatch_p95_microseconds = xtask::performance::nearest_rank_percentile(
        &report.cognition_dispatch_tick_microseconds,
        95,
    )?;
    let cognition_dispatch_p99_microseconds = xtask::performance::nearest_rank_percentile(
        &report.cognition_dispatch_tick_microseconds,
        99,
    )?;
    let world_services_p95_microseconds =
        xtask::performance::nearest_rank_percentile(&report.world_services_tick_microseconds, 95)?;
    let world_services_p99_microseconds =
        xtask::performance::nearest_rank_percentile(&report.world_services_tick_microseconds, 99)?;
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
            r4_100npc: Some(R4PopulationPerformanceDetailsV1 {
                npc_count: report.npc_count,
                active_count: report.active_count,
                near_count: report.near_count,
                background_count: report.background_count,
                warmup_ticks: report.warmup_ticks,
                measured_ticks: report.measured_ticks,
                active_due: report.due_counts.active,
                near_due: report.due_counts.near,
                background_due: report.due_counts.background,
                navigation_queries: report.due_counts.queries,
                full_evaluation_due: report.tier_cognition_due_counts.full_evaluation,
                reduced_evaluation_due: report.tier_cognition_due_counts.reduced_evaluation,
                abstract_maintenance_due: report.tier_cognition_due_counts.abstract_maintenance,
                dormant_wake_due: report.tier_cognition_due_counts.dormant_wake,
                cognition_work_items: report.tier_cognition_due_counts.work_items,
                maximum_queue_depth: report.maximum_queue_depth,
                maximum_cognition_queue_depth: report.maximum_cognition_queue_depth,
                deferred_work: report.deferred_work,
                dropped_work: report.dropped_work,
                maximum_starvation_age_ticks: report.maximum_starvation_age_ticks,
                abstract_outcome_deferrals: report.abstract_outcome_deferrals,
                fabricated_outcomes: report.fabricated_outcomes,
                navigation_p95_microseconds,
                navigation_p99_microseconds,
                cognition_dispatch_p95_microseconds,
                cognition_dispatch_p99_microseconds,
                world_services_p95_microseconds,
                world_services_p99_microseconds,
                elapsed_microseconds: report.elapsed_microseconds,
                command_body_count: report.command_body_count,
                final_population_snapshot_bytes: report.final_population_snapshot_bytes,
                due_trace_root: report.due_trace_root.to_hex(),
                tier_cognition_trace_root: report.tier_cognition_trace_root.to_hex(),
                final_population_state_hash: report.final_population_state_hash.to_hex(),
                final_activity_state_hash: report.final_activity_state_hash.to_hex(),
                final_agent_state_hash: report.final_agent_state_hash.to_hex(),
                final_memory_state_hash: report.final_memory_state_hash.to_hex(),
                final_application_state_root: report.final_application_state_root.to_hex(),
                final_command_ledger_hash: report.final_command_ledger_hash.to_hex(),
            }),
            r5_physics: None,
        },
    ))
}

fn authoritative_parity(
    left: &next_verification::PopulationPerformanceReportV1,
    right: &next_verification::PopulationPerformanceReportV1,
) -> bool {
    left.due_counts == right.due_counts
        && left.tier_cognition_due_counts == right.tier_cognition_due_counts
        && left.maximum_queue_depth == right.maximum_queue_depth
        && left.maximum_cognition_queue_depth == right.maximum_cognition_queue_depth
        && left.due_trace_root == right.due_trace_root
        && left.tier_cognition_trace_root == right.tier_cognition_trace_root
        && left.final_population_state_hash == right.final_population_state_hash
        && left.final_activity_state_hash == right.final_activity_state_hash
        && left.final_agent_state_hash == right.final_agent_state_hash
        && left.final_memory_state_hash == right.final_memory_state_hash
        && left.final_application_state_root == right.final_application_state_root
        && left.final_command_ledger_hash == right.final_command_ledger_hash
}

fn sample_duration(samples: &[u64]) -> Result<Duration, String> {
    samples
        .iter()
        .try_fold(0_u64, |total, value| total.checked_add(*value))
        .map(Duration::from_micros)
        .ok_or_else(|| "R4 duration sample sum overflow".to_owned())
}

fn metric(
    name: &str,
    unit: &str,
    samples: Vec<u64>,
    absolute_budget: Option<xtask::performance::PerformanceBudgetV1>,
) -> Result<xtask::performance::PerformanceMetricV1, String> {
    xtask::performance::PerformanceMetricV1::from_samples(name, unit, samples, absolute_budget)
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
    fn r4_budgets_bind_navigation_and_integrated_tick_rows() {
        assert_eq!(R4_NAVIGATION_P95_MICROSECONDS_MAX, 1_250);
        assert_eq!(R4_NAVIGATION_P99_MICROSECONDS_MAX, 1_500);
        assert_eq!(R4_COGNITION_DISPATCH_P95_MICROSECONDS_MAX, 1_250);
        assert_eq!(R4_COGNITION_DISPATCH_P99_MICROSECONDS_MAX, 1_500);
        assert_eq!(R4_INTEGRATED_P95_MICROSECONDS_MAX, 8_000);
        assert_eq!(R4_INTEGRATED_P99_MICROSECONDS_MAX, 12_000);
    }
}
