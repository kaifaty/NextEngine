#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use xtask::report::*;

use crate::run_tool_session;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PerformanceArguments {
    scenario: xtask::performance::PerformanceScenarioV1,
    mode: xtask::performance::PerformanceModeV1,
    target: Option<String>,
    baseline: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl Default for PerformanceArguments {
    fn default() -> Self {
        Self {
            scenario: xtask::performance::PerformanceScenarioV1::Smoke,
            mode: xtask::performance::PerformanceModeV1::Report,
            target: None,
            baseline: None,
            output: None,
        }
    }
}

pub(crate) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<PerformanceArguments, String> {
    let mut request = PerformanceArguments::default();
    let mut scenario_seen = false;
    let mut mode_seen = false;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--scenario" if !scenario_seen => {
                request.scenario = xtask::performance::PerformanceScenarioV1::parse(&value)?;
                scenario_seen = true;
            }
            "--mode" if !mode_seen => {
                request.mode = xtask::performance::PerformanceModeV1::parse(&value)?;
                mode_seen = true;
            }
            "--target" if request.target.is_none() => request.target = Some(value),
            "--baseline" if request.baseline.is_none() => {
                request.baseline = Some(PathBuf::from(value));
            }
            "--output" if request.output.is_none() => {
                request.output = Some(PathBuf::from(value));
            }
            "--scenario" | "--mode" | "--target" | "--baseline" | "--output" => {
                return Err(format!("duplicate argument: {flag}"));
            }
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    if request.mode == xtask::performance::PerformanceModeV1::Gate && request.target.is_none() {
        request.target = Some(xtask::performance::THOTH_TARGET_ID.to_owned());
    }
    Ok(request)
}

pub(crate) fn performance(root: &Path, request: &PerformanceArguments) -> Result<(), String> {
    let report = performance_report_for(root, request, None)?;
    let serialized = report.to_json()?;
    println!("{serialized}");
    if let Some(output) = &request.output {
        write_performance_report(output, serialized.as_bytes())?;
    }
    if report
        .details
        .run
        .as_ref()
        .is_some_and(|run| run.verdict == xtask::performance::PerformanceVerdict::Fail)
    {
        Err("PERFORMANCE_GATE_FAILED: hard performance verdict is FAIL".to_owned())
    } else {
        Ok(())
    }
}

pub(crate) fn performance_report(
    root: &Path,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    performance_report_for(root, &PerformanceArguments::default(), state_root)
}

fn performance_report_for(
    root: &Path,
    request: &PerformanceArguments,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let mut run = xtask::performance::PerformanceRunV1::empty(
        request.scenario,
        request.mode,
        env!("NEXTENGINE_BUILD_PROFILE"),
    );
    populate_performance_identity(root, &mut run);
    populate_performance_host(request, &mut run);

    if request.mode == xtask::performance::PerformanceModeV1::Gate {
        validate_gate_prerequisites(request, &mut run);
        if !run.diagnostics.is_empty() {
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(run, None, None, None, None));
        }
    }

    if let Some(reason) = request.scenario.unavailable_reason() {
        run.diagnostics.push(reason.to_owned());
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
        return Ok(performance_command_report(run, None, None, None, None));
    }
    if request.scenario == xtask::performance::PerformanceScenarioV1::Smoke
        && request.mode == xtask::performance::PerformanceModeV1::Gate
    {
        run.diagnostics.push(
            "PERF_SMOKE_REPORT_ONLY: smoke fixtures have no hard timing budget and cannot gate"
                .to_owned(),
        );
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
        return Ok(performance_command_report(run, None, None, None, None));
    }
    if request.scenario == xtask::performance::PerformanceScenarioV1::LongSessionSoak
        && request.mode == xtask::performance::PerformanceModeV1::Gate
    {
        run.diagnostics.push(
            "PERF_LONG_SESSION_SOAK_REPORT_ONLY: the long-session soak has no hard timing budget and cannot gate".to_owned(),
        );
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
        return Ok(performance_command_report(run, None, None, None, None));
    }

    let resource_counters_before = xtask::performance::inspect_process_counters();
    let (tool_run, _) = run_tool_session("tools-performance", state_root)?;
    let profiling_enabled = match env::var("NEXTENGINE_PERFORMANCE_PROFILER") {
        Ok(value) if value.eq_ignore_ascii_case("on") || value == "1" => true,
        Ok(value) if value.eq_ignore_ascii_case("off") || value == "0" => false,
        Ok(value) => {
            return Err(format!(
                "NEXTENGINE_PERFORMANCE_PROFILER must be on/off or 1/0, got {value}"
            ));
        }
        Err(env::VarError::NotPresent) => false,
        Err(error) => return Err(error.to_string()),
    };
    let mut recorded_spans = if profiling_enabled {
        Vec::with_capacity(
            usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD)
                .map_err(|error| error.to_string())?,
        )
    } else {
        Vec::new()
    };
    let mut dropped_spans = 0_u64;
    let mut instrumentation_overhead_nanoseconds = 0_u128;
    let profiler_control = profiling_enabled
        .then(|| run_profiler_control(request.scenario, state_root))
        .transpose()?;

    let started = Instant::now();
    let streaming = match state_root {
        Some(root) => next_verification::run_streaming_performance_check_in(root),
        None => next_verification::run_streaming_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "streaming-io",
            started.elapsed(),
        ));

    let started = Instant::now();
    let agent = match state_root {
        Some(root) => next_verification::run_agent_planning_performance_check_in(root),
        None => next_verification::run_agent_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "agent-planning",
            started.elapsed(),
        ));

    let started = Instant::now();
    let render_planning = match state_root {
        Some(root) => next_verification::run_render_frame_planning_performance_check_in(root),
        None => next_verification::run_render_frame_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "render-extraction",
            started.elapsed(),
        ));

    let started = Instant::now();
    let live_runtime = run_live_runtime_scenario(request.scenario, state_root)
        .map_err(|error| error.to_string())?;
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "runtime-stages",
            started.elapsed(),
        ));

    let desktop_frame_timing = if profiling_enabled {
        let started = Instant::now();
        let report = match state_root {
            Some(root) => next_verification::run_desktop_frame_timing_smoke_in(root),
            None => next_verification::run_desktop_frame_timing_smoke(),
        }
        .map_err(|error| error.to_string())?;
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
                true,
                &mut recorded_spans,
                &mut dropped_spans,
                "render-extraction",
                started.elapsed(),
            ));
        report
    } else {
        None
    };

    let profiled_microseconds = recorded_spans
        .iter()
        .map(|span| span.duration_microseconds)
        .try_fold(0_u64, |total, value| total.checked_add(value))
        .ok_or_else(|| "profiled smoke duration overflow".to_owned())?;
    let (overhead_basis_points, authoritative_hash_parity) = profiler_control
        .as_ref()
        .map(|control| {
            (
                overhead_basis_points(instrumentation_overhead_nanoseconds, profiled_microseconds),
                control.authoritative_hashes
                    == smoke_authoritative_hashes(
                        &streaming,
                        &agent,
                        &render_planning,
                        &live_runtime,
                    ),
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
    run.resource_counters = xtask::performance::finish_process_counters(&resource_counters_before);
    if let Some(frame_timing) = &desktop_frame_timing {
        run.resource_counters.vulkan_timestamp_queries = frame_timing.timestamp_query_count;
        run.resource_counters.device_resident_bytes = Some(frame_timing.device_allocation_bytes);
        run.resource_counters.unavailable.retain(|diagnostic| {
            !diagnostic.starts_with("Vulkan timestamps require")
                && !diagnostic.starts_with("device residency requires")
        });
        run.methodology.notes.push(
            "four-frame Vulkan timing smoke uses production render inputs and remains report-only"
                .to_owned(),
        );
        run.methodology.notes.push(format!(
            "device residency uses a conservative ceiling of {} engine-owned bound Vulkan allocations; swapchain storage is driver-owned",
            frame_timing.device_allocation_count
        ));
    }
    run.content_hash = tool_run.project_composition_lock_hash.to_hex();
    run.scenario_hash = match request.scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => xtask::performance::sha256_hex(
            b"nextengine.performance.smoke.v2:two-chunk:five-object:one-agent:900-live-ticks",
        ),
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => {
            xtask::performance::sha256_hex(
                b"nextengine.performance.long-session-soak.v3:3600-live-ticks:1200-tick-windows:held-movement:camera-every-15-ticks:driver-and-interactive-application:one-fixed-step-per-measured-pump",
            )
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    };
    let live_metric_prefix = match request.scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => "smoke.live-runtime",
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => {
            "long-session-soak.live-runtime"
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    };
    run.metrics = vec![
        smoke_metric("smoke.streaming.total", streaming.elapsed_microseconds)?,
        smoke_metric("smoke.agent-planning.total", agent.elapsed_microseconds)?,
        smoke_metric(
            "smoke.render-planning.total",
            render_planning.elapsed_microseconds,
        )?,
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("{live_metric_prefix}.window"),
            "microseconds",
            live_runtime
                .window_microseconds
                .into_iter()
                .map(microseconds_u64)
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?,
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("{live_metric_prefix}.identity-index-root-probe"),
            "microseconds",
            live_runtime
                .identity_index_root_probe_microseconds
                .into_iter()
                .map(microseconds_u64)
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?,
        xtask::performance::PerformanceMetricV1::from_samples(
            format!("{live_metric_prefix}.archive-root-probe"),
            "microseconds",
            live_runtime
                .archive_root_probe_microseconds
                .into_iter()
                .map(microseconds_u64)
                .collect::<Result<Vec<_>, _>>()?,
            None,
        )?,
    ];
    if request.scenario == xtask::performance::PerformanceScenarioV1::LongSessionSoak {
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.live-runtime.driver-prepare",
                "microseconds",
                live_runtime
                    .driver_prepare_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.live-runtime.driver-commit",
                "microseconds",
                live_runtime
                    .driver_commit_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.live-runtime.checkpoint-materialization",
                "microseconds",
                live_runtime
                    .driver_checkpoint_materialization_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.window",
                "microseconds",
                live_runtime
                    .application_window_microseconds
                    .into_iter()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.checkpoint",
                "microseconds",
                live_runtime
                    .application_checkpoint_microseconds
                    .into_iter()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.ordinary-tick",
                "microseconds",
                live_runtime
                    .application_ordinary_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.checkpoint-tick",
                "microseconds",
                live_runtime
                    .application_checkpoint_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
    }
    if let Some(frame_timing) = &desktop_frame_timing {
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "smoke.frame.cpu-extract-submit",
                "microseconds",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.cpu_extract_and_submit_microseconds)
                    .collect(),
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "smoke.frame.gpu",
                "microseconds",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.gpu_duration_microseconds)
                    .collect(),
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "smoke.frame.critical-path",
                "microseconds",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| {
                        sample
                            .cpu_extract_and_submit_microseconds
                            .max(sample.gpu_duration_microseconds)
                    })
                    .collect(),
                None,
            )?);
    }
    run.authoritative_hashes =
        smoke_authoritative_hashes(&streaming, &agent, &render_planning, &live_runtime);
    run.verdict = xtask::performance::aggregate_metric_verdict(&run.metrics);
    if let Err(error) = run.instrumentation.validate() {
        run.diagnostics.push(error);
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }

    if run.verdict != xtask::performance::PerformanceVerdict::NotRun
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

    Ok(performance_command_report(
        run,
        Some(StreamingPerformanceDetailsV1 {
            cycles: streaming.cycles,
            staged_asset_references: streaming.staged_asset_references,
            elapsed_microseconds: streaming.elapsed_microseconds,
            final_generation: streaming.final_generation,
            final_world_state_hash: streaming.final_world_state_hash.to_hex(),
        }),
        Some(AgentPerformanceDetailsV1 {
            cycles: agent.cycles,
            elapsed_microseconds: agent.elapsed_microseconds,
            final_plan_hash: agent.final_plan_hash.to_hex(),
        }),
        Some(RenderPlanningPerformanceDetailsV1 {
            cycles: render_planning.cycles,
            elapsed_microseconds: render_planning.elapsed_microseconds,
            visible_object_count: render_planning.visible_object_count,
            indexed_draw_count: render_planning.indexed_draw_count,
            fallback_material_draw_count: render_planning.fallback_material_draw_count,
            frame_plan_hash: render_planning.frame_plan_hash.to_hex(),
        }),
        Some(LiveRuntimePerformanceDetailsV1 {
            ticks: live_runtime.ticks,
            command_body_count: live_runtime.command_body_count,
            elapsed_microseconds: live_runtime.elapsed_microseconds,
            window_microseconds: live_runtime.window_microseconds,
            checkpoint_microseconds: live_runtime.checkpoint_microseconds,
            final_state_root: live_runtime.final_state_root.to_hex(),
        }),
    ))
}

struct ProfilerControl {
    authoritative_hashes: BTreeMap<String, String>,
}

fn run_profiler_control(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<ProfilerControl, String> {
    let streaming = match state_root {
        Some(root) => next_verification::run_streaming_performance_check_in(root),
        None => next_verification::run_streaming_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    let agent = match state_root {
        Some(root) => next_verification::run_agent_planning_performance_check_in(root),
        None => next_verification::run_agent_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    let render = match state_root {
        Some(root) => next_verification::run_render_frame_planning_performance_check_in(root),
        None => next_verification::run_render_frame_planning_performance_check(),
    }
    .map_err(|error| error.to_string())?;
    let live =
        run_live_runtime_scenario(scenario, state_root).map_err(|error| error.to_string())?;
    Ok(ProfilerControl {
        authoritative_hashes: smoke_authoritative_hashes(&streaming, &agent, &render, &live),
    })
}

fn run_live_runtime_scenario(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<
    next_verification::LiveRuntimePerformanceReport,
    next_verification::LiveRuntimePerformanceError,
> {
    match (scenario, state_root) {
        (xtask::performance::PerformanceScenarioV1::Smoke, Some(root)) => {
            next_verification::run_live_runtime_performance_check_in(root)
        }
        (xtask::performance::PerformanceScenarioV1::Smoke, None) => {
            next_verification::run_live_runtime_performance_check()
        }
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, Some(root)) => {
            next_verification::run_live_runtime_long_session_performance_check_in(root)
        }
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, None) => {
            next_verification::run_live_runtime_long_session_performance_check()
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    }
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

fn overhead_basis_points(overhead_nanoseconds: u128, workload_microseconds: u64) -> i64 {
    if workload_microseconds == 0 {
        return if overhead_nanoseconds == 0 {
            0
        } else {
            i64::MAX
        };
    }
    let value = i128::try_from(overhead_nanoseconds)
        .unwrap_or(i128::MAX)
        .saturating_mul(10_000)
        / i128::from(workload_microseconds).saturating_mul(1_000);
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn populate_performance_identity(root: &Path, run: &mut xtask::performance::PerformanceRunV1) {
    match Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => {
            run.commit = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_COMMIT_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_COMMIT_UNAVAILABLE: {error}")),
    }
    match Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => {
            run.worktree_clean = output.stdout.is_empty();
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_WORKTREE_STATE_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_WORKTREE_STATE_UNAVAILABLE: {error}")),
    }
    match Command::new("rustc").arg("-vV").output() {
        Ok(output) if output.status.success() => {
            run.toolchain = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        }
        Ok(output) => run.diagnostics.push(format!(
            "PERF_TOOLCHAIN_UNAVAILABLE: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => run
            .diagnostics
            .push(format!("PERF_TOOLCHAIN_UNAVAILABLE: {error}")),
    }
}

fn populate_performance_host(
    request: &PerformanceArguments,
    run: &mut xtask::performance::PerformanceRunV1,
) {
    let target_id = request.target.as_deref().unwrap_or("observed-host-v1");
    match xtask::performance::inspect_current_host(target_id) {
        Ok((fingerprint, preflight)) => {
            run.target_fingerprint = Some(fingerprint);
            run.preflight = Some(preflight);
        }
        Err(error) => run.diagnostics.push(error),
    }
}

fn validate_gate_prerequisites(
    request: &PerformanceArguments,
    run: &mut xtask::performance::PerformanceRunV1,
) {
    if run.build_profile != "release" {
        run.diagnostics
            .push("PERF_GATE_REQUIRES_RELEASE_BUILD".to_owned());
    }
    if !run.worktree_clean {
        run.diagnostics
            .push("PERF_GATE_REQUIRES_CLEAN_COMMIT".to_owned());
    }
    if request.target.as_deref() != Some(xtask::performance::THOTH_TARGET_ID) {
        run.diagnostics
            .push("PERF_GATE_TARGET_UNSUPPORTED".to_owned());
    }
    match &run.target_fingerprint {
        Some(fingerprint) => run
            .diagnostics
            .extend(xtask::performance::validate_thoth_fingerprint(fingerprint)),
        None => run
            .diagnostics
            .push("PERF_TARGET_FINGERPRINT_UNAVAILABLE".to_owned()),
    }
    match &run.preflight {
        Some(preflight) if preflight.ready => {}
        Some(preflight) => run.diagnostics.extend(preflight.diagnostics.clone()),
        None => run
            .diagnostics
            .push("PERF_PREFLIGHT_UNAVAILABLE".to_owned()),
    }
    if request.baseline.is_none() {
        run.diagnostics
            .push("PERF_GATE_BASELINE_REQUIRED".to_owned());
    }
}

fn record_performance_span(
    enabled: bool,
    spans: &mut Vec<xtask::performance::PerformanceSpanV1>,
    dropped_spans: &mut u64,
    category: &str,
    elapsed: Duration,
) -> u128 {
    if enabled {
        let started = Instant::now();
        if spans.len() < usize::try_from(xtask::performance::MAX_SPANS_PER_THREAD).unwrap_or(0) {
            spans.push(xtask::performance::PerformanceSpanV1 {
                category: category.to_owned(),
                thread_index: 0,
                duration_microseconds: u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX),
            });
        } else {
            *dropped_spans = dropped_spans.saturating_add(1);
        }
        started.elapsed().as_nanos()
    } else {
        0
    }
}

fn smoke_metric(
    name: &str,
    elapsed_microseconds: u128,
) -> Result<xtask::performance::PerformanceMetricV1, String> {
    xtask::performance::PerformanceMetricV1::from_samples(
        name,
        "microseconds",
        vec![microseconds_u64(elapsed_microseconds)?],
        None,
    )
}

fn microseconds_u64(value: u128) -> Result<u64, String> {
    u64::try_from(value).map_err(|error| format!("performance duration overflow: {error}"))
}

fn read_performance_baseline(
    path: &Path,
) -> Result<xtask::performance::PerformanceBaselineV1, String> {
    const MAX_BASELINE_BYTES: u64 = 64 * 1024 * 1024;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("failed to inspect baseline {}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_BASELINE_BYTES {
        return Err(format!(
            "performance baseline must be a regular file no larger than {MAX_BASELINE_BYTES} bytes"
        ));
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read baseline {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid PerformanceBaselineV1: {error}"))
}

fn performance_command_report(
    run: xtask::performance::PerformanceRunV1,
    streaming: Option<StreamingPerformanceDetailsV1>,
    agent_planning: Option<AgentPerformanceDetailsV1>,
    render_planning: Option<RenderPlanningPerformanceDetailsV1>,
    live_runtime: Option<LiveRuntimePerformanceDetailsV1>,
) -> CommandReportV1<PerformanceDetailsV1> {
    let status = match run.verdict {
        xtask::performance::PerformanceVerdict::Pass => "PASS",
        xtask::performance::PerformanceVerdict::Fail => "FAIL",
        xtask::performance::PerformanceVerdict::Warning => "WARNING",
        xtask::performance::PerformanceVerdict::ReportOnly => "PASS",
        xtask::performance::PerformanceVerdict::NotRun => "NOT_RUN",
    };
    CommandReportV1::new(
        "performance",
        status,
        PerformanceDetailsV1 {
            run: Some(run),
            streaming,
            agent_planning,
            render_planning,
            live_runtime,
        },
    )
}

fn write_performance_report(output: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(output).map_err(|error| {
        format!(
            "failed to create performance output {}: {error}",
            output.display()
        )
    })?;
    let final_path = output.join("performance-report-v1.json");
    let temporary_path = output.join(".performance-report-v1.json.tmp");
    if final_path.exists() || temporary_path.exists() {
        return Err(format!(
            "performance output already exists: {}",
            final_path.display()
        ));
    }
    fs::write(&temporary_path, bytes).map_err(|error| {
        format!(
            "failed to write performance report {}: {error}",
            temporary_path.display()
        )
    })?;
    fs::rename(&temporary_path, &final_path).map_err(|error| {
        format!(
            "failed to publish performance report {}: {error}",
            final_path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_cli_defaults_to_smoke_report() {
        let request = parse_arguments(std::iter::empty()).expect("default arguments");
        assert_eq!(
            request.scenario,
            xtask::performance::PerformanceScenarioV1::Smoke
        );
        assert_eq!(request.mode, xtask::performance::PerformanceModeV1::Report);
        assert_eq!(request.target, None);
    }

    #[test]
    fn performance_gate_defaults_to_thoth_and_rejects_duplicate_flags() {
        let request = parse_arguments(
            ["--scenario", "r5-physics-16", "--mode", "gate"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("gate arguments");
        assert_eq!(
            request.target.as_deref(),
            Some(xtask::performance::THOTH_TARGET_ID)
        );
        assert!(
            parse_arguments(
                ["--mode", "report", "--mode", "gate"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
    }

    #[test]
    fn performance_cli_accepts_the_report_only_long_session_soak() {
        let request = parse_arguments(
            ["--scenario", "long-session-soak", "--mode", "report"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("long-session soak arguments");
        assert_eq!(
            request.scenario,
            xtask::performance::PerformanceScenarioV1::LongSessionSoak
        );
        assert_eq!(request.mode, xtask::performance::PerformanceModeV1::Report);
        assert_eq!(request.target, None);
    }
}
