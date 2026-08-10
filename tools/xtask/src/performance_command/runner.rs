use super::*;

pub(super) fn performance_report_once(
    root: &Path,
    request: &PerformanceArguments,
    state_root: Option<&Path>,
    compare_baseline: bool,
) -> Result<CommandReportV1<PerformanceDetailsV1>, String> {
    let mut run = xtask::performance::PerformanceRunV5::empty(
        request.scenario,
        request.mode,
        env!("NEXTENGINE_BUILD_PROFILE"),
    );
    run.scenario_hash = performance_scenario_hash(request.scenario);
    populate_performance_identity(root, &mut run);
    populate_performance_host(request, &mut run);

    if request.require_ready_preflight {
        let mut preflight_diagnostics = match &run.preflight {
            Some(preflight) => preflight
                .validate_ready_evidence()
                .err()
                .unwrap_or_default(),
            None => vec!["PERF_PREFLIGHT_UNAVAILABLE".to_owned()],
        };
        if !preflight_diagnostics.is_empty() || !run.diagnostics.is_empty() {
            run.diagnostics.append(&mut preflight_diagnostics);
            run.diagnostics.sort();
            run.diagnostics.dedup();
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(
                run, None, None, None, None, None,
            ));
        }
    }

    if request.mode == xtask::performance::PerformanceModeV1::Gate {
        if let Some(diagnostic) = report_only_gate_diagnostic(request.scenario) {
            run.diagnostics.push(diagnostic.to_owned());
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(
                run, None, None, None, None, None,
            ));
        }
        validate_gate_prerequisites(request, &mut run);
        if !run.diagnostics.is_empty() {
            run.verdict = xtask::performance::PerformanceVerdict::NotRun;
            return Ok(performance_command_report(
                run, None, None, None, None, None,
            ));
        }
    }

    if let Some(reason) = request.scenario.unavailable_reason() {
        run.diagnostics.push(reason.to_owned());
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
        return Ok(performance_command_report(
            run, None, None, None, None, None,
        ));
    }
    let (tool_run, _) = run_tool_session("tools-performance-v2", state_root)?;
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
    if request.scenario == xtask::performance::PerformanceScenarioV1::R3MultiregionStreaming {
        return r3_multiregion::performance_report(
            request,
            state_root,
            run,
            tool_run.project_composition_lock_hash.to_hex(),
            profiling_enabled,
            compare_baseline,
        );
    }
    if request.scenario == xtask::performance::PerformanceScenarioV1::R5Physics16 {
        return r5_physics::performance_report(
            request,
            run,
            tool_run.project_composition_lock_hash.to_hex(),
            profiling_enabled,
            compare_baseline,
        );
    }
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
    let desktop_frame_timing_requested = profiling_enabled
        || request.scenario == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak;
    let ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker,
        desktop_frame_timing,
        r2_alpha_render,
        resource_counters,
    } = run_scenario_workloads(
        request.scenario,
        state_root,
        desktop_frame_timing_requested,
        INTERACTIVE_FRAME_SOAK_FRAMES,
    )?;

    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "streaming-io",
            streaming.elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "agent-planning",
            agent.elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "render-extraction",
            render_planning.elapsed,
        ));
    instrumentation_overhead_nanoseconds =
        instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
            profiling_enabled,
            &mut recorded_spans,
            &mut dropped_spans,
            "runtime-stages",
            live_runtime.elapsed,
        ));
    if profiling_enabled && let Some(frame_timing) = &desktop_frame_timing {
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                "render-extraction",
                frame_timing.elapsed,
            ));
    }
    if profiling_enabled && let Some(r2_alpha_render) = &r2_alpha_render {
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_performance_span(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                "render-extraction",
                r2_alpha_render.elapsed,
            ));
    }

    let streaming = streaming.report;
    let agent = agent.report;
    let render_planning = render_planning.report;
    let live_runtime = live_runtime.report;
    let production_worker = production_worker.map(|workload| workload.report);
    let desktop_frame_timing = desktop_frame_timing.and_then(|workload| workload.report);
    let r2_alpha_render = r2_alpha_render.map(|workload| workload.report);

    let mut authoritative_hashes = scenario_authoritative_hashes(
        &streaming,
        &agent,
        &render_planning,
        &live_runtime,
        production_worker.as_ref(),
    );
    if let Some(report) = &r2_alpha_render {
        r2_alpha_render::append_authoritative_hashes(&mut authoritative_hashes, report);
    }
    let worker_unowned_spans = production_worker
        .as_ref()
        .map(observed_worker_unowned_spans)
        .unwrap_or(0);
    let worker_reserved_bytes = production_worker
        .as_ref()
        .map(production_worker_reserved_bytes)
        .transpose()?
        .unwrap_or(0);
    if let Some(worker) = &production_worker {
        dropped_spans = dropped_spans.saturating_add(worker.metrics.dropped_callbacks);
        instrumentation_overhead_nanoseconds =
            instrumentation_overhead_nanoseconds.saturating_add(record_worker_owned_spans(
                profiling_enabled,
                &mut recorded_spans,
                &mut dropped_spans,
                worker,
            ));
    }

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
                control.authoritative_hashes == authoritative_hashes,
            )
        })
        .map_or((None, None), |(overhead, parity)| {
            (Some(overhead), Some(parity))
        });
    run.instrumentation = xtask::performance::PerformanceInstrumentationV1 {
        enabled: profiling_enabled,
        max_threads: if production_worker.is_some() { 2 } else { 1 },
        max_spans_per_thread: xtask::performance::MAX_SPANS_PER_THREAD,
        reserved_bytes: u64::try_from(
            recorded_spans.capacity()
                * std::mem::size_of::<xtask::performance::PerformanceSpanV1>(),
        )
        .map_err(|error| error.to_string())?
        .checked_add(worker_reserved_bytes)
        .ok_or_else(|| "performance instrumentation reservation overflow".to_owned())?,
        recorded_spans,
        dropped_spans,
        unowned_spans: worker_unowned_spans,
        overhead_basis_points,
        authoritative_hash_parity,
    };
    run.resource_counters = resource_counters;
    if request.scenario == xtask::performance::PerformanceScenarioV1::Smoke {
        run.resource_counters.logical_resource_charges = Some(
            xtask::performance::PerformanceLogicalResourceChargesV1::new(
                xtask::performance::sha256_hex(b"nextengine.performance.r3a-packaged-streaming.v1"),
                0,
                streaming.required_staging_bytes,
                0,
                0,
                0,
                0,
            )?,
        );
    }
    if let Some(frame_timing) = &desktop_frame_timing {
        run.resource_counters.vulkan_timestamp_queries = frame_timing.timestamp_query_count;
        run.resource_counters.device_resident_bytes = Some(frame_timing.device_allocation_bytes);
        run.resource_counters.unavailable.retain(|diagnostic| {
            !diagnostic.starts_with("Vulkan timestamps require")
                && !diagnostic.starts_with("device residency requires")
        });
    }
    if let Some(report) = &r2_alpha_render {
        r2_alpha_render::attach_resource_evidence(&mut run.resource_counters, report)?;
    }
    run.content_hash = tool_run.project_composition_lock_hash.to_hex();
    run.scenario_hash = performance_scenario_hash(request.scenario);
    let live_metric_prefix = match request.scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => "smoke.live-runtime",
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => {
            "long-session-soak.live-runtime"
        }
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak => {
            "interactive-frame-soak.live-runtime-control"
        }
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak => {
            "production-worker-soak.live-runtime-control"
        }
        xtask::performance::PerformanceScenarioV1::R2AlphaRender => {
            "r2-alpha-render.live-runtime-control"
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
                "long-session-soak.application.sample-interval",
                "microseconds",
                live_runtime
                    .application_sample_interval_microseconds
                    .into_iter()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.non-sample-tick",
                "microseconds",
                live_runtime
                    .application_non_sample_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
        run.metrics
            .push(xtask::performance::PerformanceMetricV1::from_samples(
                "long-session-soak.application.sample-tick",
                "microseconds",
                live_runtime
                    .application_sample_tick_microseconds
                    .iter()
                    .copied()
                    .map(microseconds_u64)
                    .collect::<Result<Vec<_>, _>>()?,
                None,
            )?);
    }
    if let Some(worker) = &production_worker {
        append_production_worker_metrics(&mut run.metrics, worker)?;
    }
    if let Some(frame_timing) = &desktop_frame_timing {
        let frame_prefix = if request.scenario
            == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
        {
            "interactive-frame-soak.frame"
        } else {
            "smoke.frame"
        };
        let phase_samples = [
            (
                "cpu-extract-submit",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.cpu_extract_and_submit_microseconds)
                    .collect(),
            ),
            (
                "gpu",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.gpu_duration_microseconds)
                    .collect(),
            ),
            (
                "critical-path",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| {
                        sample
                            .cpu_extract_and_submit_microseconds
                            .max(sample.gpu_duration_microseconds)
                    })
                    .collect(),
            ),
            (
                "event-frame-source-update",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.event_and_frame_source_update_microseconds)
                    .collect(),
            ),
            (
                "frame-slot-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.frame_slot_wait_microseconds)
                    .collect(),
            ),
            (
                "image-acquire-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.image_acquire_wait_microseconds)
                    .collect(),
            ),
            (
                "swapchain-image-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.swapchain_image_wait_microseconds)
                    .collect(),
            ),
            (
                "frame-plan",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.frame_plan_microseconds)
                    .collect(),
            ),
            (
                "command-record",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.command_record_microseconds)
                    .collect(),
            ),
            (
                "queue-submit",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.queue_submit_microseconds)
                    .collect(),
            ),
            (
                "present-wait",
                frame_timing
                    .samples
                    .iter()
                    .map(|sample| sample.present_wait_microseconds)
                    .collect(),
            ),
        ];
        for (name, samples) in phase_samples {
            run.metrics
                .push(xtask::performance::PerformanceMetricV1::from_samples(
                    format!("{frame_prefix}.{name}"),
                    "microseconds",
                    samples,
                    None,
                )?);
        }
        let deadline_misses = u64::try_from(
            frame_timing
                .samples
                .iter()
                .filter(|sample| {
                    sample
                        .cpu_extract_and_submit_microseconds
                        .max(sample.gpu_duration_microseconds)
                        > 16_667
                })
                .count(),
        )
        .map_err(|error| error.to_string())?;
        for (name, unit, value) in [
            ("primary-deadline-misses", "count", deadline_misses),
            (
                "software-paced-iterations",
                "count",
                frame_timing.software_paced_iterations,
            ),
            (
                "software-pacing-sleep",
                "microseconds",
                frame_timing.software_pacing_sleep_microseconds,
            ),
            (
                "frame-plan-cache-hits",
                "count",
                frame_timing.frame_plan_cache_hits,
            ),
            (
                "frame-plan-cache-misses",
                "count",
                frame_timing.frame_plan_cache_misses,
            ),
            (
                "frame-plan-build-failures",
                "count",
                frame_timing.frame_plan_build_failures,
            ),
            (
                "frame-plan-explicit-invalidations",
                "count",
                frame_timing.frame_plan_explicit_invalidations,
            ),
        ] {
            run.metrics
                .push(xtask::performance::PerformanceMetricV1::from_samples(
                    format!("{frame_prefix}.{name}"),
                    unit,
                    vec![value],
                    None,
                )?);
        }
    }
    if let Some(report) = &r2_alpha_render {
        r2_alpha_render::append_metrics(&mut run.metrics, report)?;
    }
    run.authoritative_hashes = authoritative_hashes;
    run.verdict = xtask::performance::aggregate_metric_verdict(&run.metrics);
    if request.scenario == xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
        && desktop_frame_timing.is_none()
    {
        run.diagnostics.push(
            "PERF_INTERACTIVE_FRAME_TIMING_UNAVAILABLE: build xtask with --features desktop-sdl-ash on a supported desktop host"
                .to_owned(),
        );
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
    }
    if request.scenario == xtask::performance::PerformanceScenarioV1::R2AlphaRender
        && r2_alpha_render.is_none()
    {
        run.diagnostics.push(
            "PERF_R2_ALPHA_RENDER_UNAVAILABLE: build xtask with --features desktop-sdl-ash on a supported Windows desktop host"
                .to_owned(),
        );
        run.verdict = xtask::performance::PerformanceVerdict::NotRun;
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

    let production_worker_details = production_worker
        .as_ref()
        .map(production_worker_details)
        .transpose()?;
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
        production_worker_details,
    ))
}
