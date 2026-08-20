use std::collections::BTreeMap;
use std::path::Path;

use super::*;

pub(super) fn performance_report(
    request: &PerformanceArguments,
    state_root: Option<&Path>,
    mut run: xtask::performance::PerformanceRunV6,
    content_hash: String,
    profiling_enabled: bool,
    compare_baseline: bool,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let profiler_control = profiling_enabled
        .then(|| run_multiregion_streaming(state_root))
        .transpose()?;
    let StreamingOnlyWorkload {
        streaming,
        mut resource_counters,
    } = run_multiregion_streaming(state_root)?;

    let mut recorded_spans = if profiling_enabled {
        Vec::with_capacity(
            usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD)
                .map_err(|error| error.to_string())?,
        )
    } else {
        Vec::new()
    };
    let mut dropped_spans = 0_u64;
    let instrumentation_overhead_nanoseconds = record_performance_span(
        profiling_enabled,
        &mut recorded_spans,
        &mut dropped_spans,
        "streaming-io",
        streaming.elapsed,
    );
    let streaming = streaming.report;
    let authoritative_hashes = BTreeMap::from([(
        "streaming_world".to_owned(),
        streaming.final_world_state_hash.to_hex(),
    )]);
    let profiled_microseconds = recorded_spans
        .iter()
        .map(|span| span.duration_microseconds)
        .try_fold(0_u64, |total, value| total.checked_add(value))
        .ok_or_else(|| "profiled R3 streaming duration overflow".to_owned())?;
    let (overhead_basis_points, authoritative_hash_parity) = profiler_control
        .as_ref()
        .map(|control| {
            (
                overhead_basis_points(instrumentation_overhead_nanoseconds, profiled_microseconds),
                control.streaming.report.final_world_state_hash == streaming.final_world_state_hash,
            )
        })
        .map_or((None, None), |(overhead, parity)| {
            (Some(overhead), Some(parity))
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
    resource_counters.logical_resource_charges = Some(
        xtask::performance::PerformanceLogicalResourceChargesV1::new(
            xtask::performance::sha256_hex(
                b"nextengine.performance.r3-multiregion-streaming-accounting.v1",
            ),
            0,
            streaming.required_staging_bytes,
            0,
            0,
            0,
            0,
        )?,
    );
    run.resource_counters = resource_counters;
    run.content_hash = content_hash;
    run.scenario_hash = performance_scenario_hash(request.scenario);
    let metric_name = "r3-multiregion-streaming.total";
    run.metrics = vec![xtask::performance::PerformanceMetricV1::from_samples(
        metric_name,
        "microseconds",
        vec![microseconds_u64(streaming.elapsed_microseconds)?],
        xtask::performance::canonical_budget_for_metric(request.scenario, metric_name),
    )?];
    run.authoritative_hashes = authoritative_hashes;
    run.verdict = xtask::performance::aggregate_metric_verdict(&run.metrics);

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

    Ok(performance_command_report(
        run,
        Some(StreamingPerformanceDetailsV1 {
            cycles: streaming.cycles,
            staged_asset_references: streaming.staged_asset_references,
            elapsed_microseconds: streaming.elapsed_microseconds,
            final_generation: streaming.final_generation,
            final_world_state_hash: streaming.final_world_state_hash.to_hex(),
        }),
        None,
        None,
        None,
        None,
    ))
}
