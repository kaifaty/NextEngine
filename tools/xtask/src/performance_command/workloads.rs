use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use super::allocation_counter::ScenarioResourceWindow;
use super::production_worker::{
    PreparedProductionWorkerScenario, combine_production_worker_result,
    run_production_worker_scenario,
};

pub(super) struct Timed<T> {
    pub(super) report: T,
    pub(super) elapsed: Duration,
}

pub(super) struct ProfilerControl {
    pub(super) authoritative_hashes: BTreeMap<String, String>,
}

pub(super) struct ScenarioWorkloads {
    pub(super) streaming: Timed<next_verification::StreamingPerformanceReport>,
    pub(super) agent: Timed<next_verification::AgentPlanningPerformanceReport>,
    pub(super) render_planning: Timed<next_verification::RenderFramePlanningPerformanceReport>,
    pub(super) live_runtime: Timed<next_verification::LiveRuntimePerformanceReport>,
    pub(super) production_worker:
        Option<Timed<next_application::InteractiveWorkerDiagnosticReportV1>>,
    pub(super) desktop_frame_timing:
        Option<Timed<Option<next_verification::DesktopFrameTimingSmokeReport>>>,
    pub(super) r2_alpha_render: Option<Timed<next_verification::R2AlphaRenderPerformanceReportV1>>,
    pub(super) resource_counters: xtask::performance::PerformanceResourceCountersV3,
}

pub(super) fn run_scenario_workloads(
    scenario: xtask::performance::PerformanceScenarioV1,
    scenario_hash: &str,
    state_root: Option<&Path>,
    desktop_frame_timing_requested: bool,
    interactive_frame_soak_frames: u32,
) -> Result<ScenarioWorkloads, String> {
    match scenario {
        xtask::performance::PerformanceScenarioV1::Smoke => run_smoke_scenario_workloads(
            scenario_hash,
            state_root,
            desktop_frame_timing_requested,
            interactive_frame_soak_frames,
        ),
        xtask::performance::PerformanceScenarioV1::LongSessionSoak => {
            run_long_session_scenario_workloads(
                scenario_hash,
                state_root,
                desktop_frame_timing_requested,
                interactive_frame_soak_frames,
            )
        }
        xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak => {
            run_production_worker_scenario_workloads(
                scenario_hash,
                state_root,
                desktop_frame_timing_requested,
                interactive_frame_soak_frames,
            )
        }
        xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak => {
            run_interactive_frame_scenario_workloads(
                scenario_hash,
                state_root,
                interactive_frame_soak_frames,
            )
        }
        xtask::performance::PerformanceScenarioV1::R2AlphaRender => {
            run_r2_alpha_render_scenario_workloads(scenario_hash, state_root)
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    }
}

#[inline(never)]
fn run_smoke_scenario_workloads(
    scenario_hash: &str,
    state_root: Option<&Path>,
    desktop_frame_timing_requested: bool,
    interactive_frame_soak_frames: u32,
) -> Result<ScenarioWorkloads, String> {
    let scenario = xtask::performance::PerformanceScenarioV1::Smoke;
    let PreparedSmokeScenario {
        mut streaming,
        mut agent,
        mut render_planning,
        mut live_runtime,
    } = PreparedSmokeScenario::new(state_root)?;
    let window = ScenarioResourceWindow::begin(scenario, scenario_hash);
    let (streaming_measurement, streaming_elapsed) = measure(|| streaming.run_measured());
    let (agent_measurement, agent_elapsed) = measure(|| agent.run_measured());
    let (render_measurement, render_elapsed) = measure(|| render_planning.run_measured());
    let (live_measurement, live_elapsed) = measure(|| live_runtime.run_measured());
    let resource_counters = window.finish();
    let streaming = streaming
        .finish(streaming_measurement)
        .map(|report| Timed {
            report,
            elapsed: streaming_elapsed,
        })
        .map_err(|error| error.to_string());
    let agent = agent
        .finish(agent_measurement)
        .map(|report| Timed {
            report,
            elapsed: agent_elapsed,
        })
        .map_err(|error| error.to_string());
    let render_planning = render_planning
        .finish(render_measurement)
        .map(|report| Timed {
            report,
            elapsed: render_elapsed,
        })
        .map_err(|error| error.to_string());
    let live_runtime = live_runtime
        .finish(live_measurement)
        .map(|report| Timed {
            report,
            elapsed: live_elapsed,
        })
        .map_err(|error| error.to_string());
    let (streaming, agent, render_planning, live_runtime) =
        match (streaming, agent, render_planning, live_runtime) {
            (Ok(streaming), Ok(agent), Ok(render_planning), Ok(live_runtime)) => {
                (streaming, agent, render_planning, live_runtime)
            }
            (streaming, agent, render_planning, live_runtime) => {
                return Err(combine_named_errors([
                    ("streaming", streaming.err()),
                    ("agent-planning", agent.err()),
                    ("render-planning", render_planning.err()),
                    ("live-runtime", live_runtime.err()),
                ]));
            }
        };
    let desktop_frame_timing = desktop_frame_timing_requested
        .then(|| run_desktop_frame_timing(scenario, state_root, interactive_frame_soak_frames))
        .transpose()?;
    Ok(ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker: None,
        desktop_frame_timing,
        r2_alpha_render: None,
        resource_counters,
    })
}

#[inline(never)]
fn run_long_session_scenario_workloads(
    scenario_hash: &str,
    state_root: Option<&Path>,
    desktop_frame_timing_requested: bool,
    interactive_frame_soak_frames: u32,
) -> Result<ScenarioWorkloads, String> {
    let scenario = xtask::performance::PerformanceScenarioV1::LongSessionSoak;
    let streaming = run_streaming(state_root)?;
    let agent = run_agent_planning(state_root)?;
    let render_planning = run_render_planning(state_root)?;
    let mut prepared_live = prepare_live_runtime_scenario(scenario, state_root)?;
    let window = ScenarioResourceWindow::begin(scenario, scenario_hash);
    let live_started = Instant::now();
    let live_measurement = prepared_live.run_measured();
    let live_elapsed = live_started.elapsed();
    let resource_counters = window.finish();
    let live_runtime = Timed {
        report: prepared_live
            .finish(live_measurement)
            .map_err(|error| error.to_string())?,
        elapsed: live_elapsed,
    };
    let desktop_frame_timing = desktop_frame_timing_requested
        .then(|| run_desktop_frame_timing(scenario, state_root, interactive_frame_soak_frames))
        .transpose()?;
    Ok(ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker: None,
        desktop_frame_timing,
        r2_alpha_render: None,
        resource_counters,
    })
}

#[inline(never)]
fn run_production_worker_scenario_workloads(
    scenario_hash: &str,
    state_root: Option<&Path>,
    desktop_frame_timing_requested: bool,
    interactive_frame_soak_frames: u32,
) -> Result<ScenarioWorkloads, String> {
    let scenario = xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak;
    let streaming = run_streaming(state_root)?;
    let agent = run_agent_planning(state_root)?;
    let render_planning = run_render_planning(state_root)?;
    let live_runtime = run_live_runtime(scenario, state_root)?;
    let mut prepared_worker = PreparedProductionWorkerScenario::new(state_root)?;
    let window = ScenarioResourceWindow::begin(scenario, scenario_hash);
    let worker_started = Instant::now();
    let measurement = prepared_worker.run_measurement();
    let worker_elapsed = worker_started.elapsed();
    let resource_counters = window.finish();
    let production_worker = prepared_worker.finish_measurement(measurement);
    let cleanup = prepared_worker.cleanup();
    let production_worker = Some(Timed {
        report: combine_production_worker_result(production_worker, cleanup)?,
        elapsed: worker_elapsed,
    });
    let desktop_frame_timing = desktop_frame_timing_requested
        .then(|| run_desktop_frame_timing(scenario, state_root, interactive_frame_soak_frames))
        .transpose()?;
    Ok(ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker,
        desktop_frame_timing,
        r2_alpha_render: None,
        resource_counters,
    })
}

#[inline(never)]
fn run_interactive_frame_scenario_workloads(
    scenario_hash: &str,
    state_root: Option<&Path>,
    interactive_frame_soak_frames: u32,
) -> Result<ScenarioWorkloads, String> {
    let scenario = xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak;
    let streaming = run_streaming(state_root)?;
    let agent = run_agent_planning(state_root)?;
    let render_planning = run_render_planning(state_root)?;
    let live_runtime = run_live_runtime(scenario, state_root)?;
    let desktop_frame_root = state_root
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    let mut prepared_frame = next_verification::prepare_desktop_frame_timing_workload_in(
        &desktop_frame_root,
        interactive_frame_soak_frames,
        [1_920, 1_080],
    )
    .map_err(|error| error.to_string())?;
    let window = ScenarioResourceWindow::begin(scenario, scenario_hash);
    let frame_started = Instant::now();
    let frame_measurement = prepared_frame.run_measured();
    let frame_elapsed = frame_started.elapsed();
    let resource_counters = window.finish();
    let desktop_frame_timing = Some(Timed {
        report: prepared_frame
            .finish(frame_measurement)
            .map_err(|error| error.to_string())?,
        elapsed: frame_elapsed,
    });
    Ok(ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker: None,
        desktop_frame_timing,
        r2_alpha_render: None,
        resource_counters,
    })
}

#[inline(never)]
fn run_r2_alpha_render_scenario_workloads(
    scenario_hash: &str,
    state_root: Option<&Path>,
) -> Result<ScenarioWorkloads, String> {
    let scenario = xtask::performance::PerformanceScenarioV1::R2AlphaRender;
    let streaming = run_streaming(state_root)?;
    let agent = run_agent_planning(state_root)?;
    let render_planning = run_render_planning(state_root)?;
    let live_runtime = run_live_runtime(scenario, state_root)?;
    let scratch_root = state_root
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    let mut prepared =
        next_verification::prepare_r2_alpha_render_performance_check_in(&scratch_root)
            .map_err(|error| error.to_string())?;
    let window = ScenarioResourceWindow::begin(scenario, scenario_hash);
    let started = Instant::now();
    let report = prepared.run_measured().map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    let resource_counters = window.finish();
    let r2_alpha_render = report.map(|report| Timed { report, elapsed });
    Ok(ScenarioWorkloads {
        streaming,
        agent,
        render_planning,
        live_runtime,
        production_worker: None,
        desktop_frame_timing: None,
        r2_alpha_render,
        resource_counters,
    })
}

pub(super) fn run_streaming(
    state_root: Option<&Path>,
) -> Result<Timed<next_verification::StreamingPerformanceReport>, String> {
    timed(|| match state_root {
        Some(root) => next_verification::run_streaming_performance_check_in(root),
        None => next_verification::run_streaming_performance_check(),
    })
}

pub(super) fn run_agent_planning(
    state_root: Option<&Path>,
) -> Result<Timed<next_verification::AgentPlanningPerformanceReport>, String> {
    timed(|| match state_root {
        Some(root) => next_verification::run_agent_planning_performance_check_in(root),
        None => next_verification::run_agent_planning_performance_check(),
    })
}

pub(super) fn run_render_planning(
    state_root: Option<&Path>,
) -> Result<Timed<next_verification::RenderFramePlanningPerformanceReport>, String> {
    timed(|| match state_root {
        Some(root) => next_verification::run_render_frame_planning_performance_check_in(root),
        None => next_verification::run_render_frame_planning_performance_check(),
    })
}

pub(super) fn run_live_runtime(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<Timed<next_verification::LiveRuntimePerformanceReport>, String> {
    timed(|| run_live_runtime_scenario(scenario, state_root))
}

pub(super) fn run_production_worker(
    state_root: Option<&Path>,
) -> Result<Timed<next_application::InteractiveWorkerDiagnosticReportV1>, String> {
    timed(|| run_production_worker_scenario(state_root))
}

pub(super) fn run_desktop_frame_timing(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
    interactive_frame_soak_frames: u32,
) -> Result<Timed<Option<next_verification::DesktopFrameTimingSmokeReport>>, String> {
    timed(|| match (scenario, state_root) {
        (xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak, Some(root)) => {
            next_verification::run_desktop_frame_timing_workload_in(
                root,
                interactive_frame_soak_frames,
                [1_920, 1_080],
            )
        }
        (xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak, None) => {
            next_verification::run_desktop_frame_timing_workload_in(
                &std::env::temp_dir(),
                interactive_frame_soak_frames,
                [1_920, 1_080],
            )
        }
        (_, Some(root)) => next_verification::run_desktop_frame_timing_smoke_in(root),
        (_, None) => next_verification::run_desktop_frame_timing_smoke(),
    })
}

pub(super) fn run_profiler_control(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<ProfilerControl, String> {
    let streaming = run_streaming(state_root)?.report;
    let agent = run_agent_planning(state_root)?.report;
    let render = run_render_planning(state_root)?.report;
    let live = run_live_runtime(scenario, state_root)?.report;
    let production_worker =
        if scenario == xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak {
            Some(run_production_worker(state_root)?.report)
        } else {
            None
        };
    let mut authoritative_hashes = super::scenario_authoritative_hashes(
        &streaming,
        &agent,
        &render,
        &live,
        production_worker.as_ref(),
    );
    if scenario == xtask::performance::PerformanceScenarioV1::R2AlphaRender {
        let scratch_root = state_root
            .map(Path::to_path_buf)
            .unwrap_or_else(std::env::temp_dir);
        let prepared =
            next_verification::prepare_r2_alpha_render_performance_check_in(&scratch_root)
                .map_err(|error| error.to_string())?;
        authoritative_hashes.insert(
            "r2_alpha_state".to_owned(),
            prepared.authoritative_state_root().to_hex(),
        );
        authoritative_hashes.insert(
            "r2_alpha_ledger".to_owned(),
            prepared.command_ledger_hash().to_hex(),
        );
    }
    Ok(ProfilerControl {
        authoritative_hashes,
    })
}

fn timed<T, E>(workload: impl FnOnce() -> Result<T, E>) -> Result<Timed<T>, String>
where
    E: ToString,
{
    let started = Instant::now();
    let report = workload().map_err(|error| error.to_string())?;
    Ok(Timed {
        report,
        elapsed: started.elapsed(),
    })
}

fn measure<T, E>(workload: impl FnOnce() -> Result<T, E>) -> (Result<T, E>, Duration) {
    let started = Instant::now();
    let result = workload();
    (result, started.elapsed())
}

struct PreparedSmokeScenario {
    streaming: next_verification::PreparedStreamingPerformanceCheck,
    agent: next_verification::PreparedAgentPlanningPerformanceCheck,
    render_planning: next_verification::PreparedRenderFramePlanningPerformanceCheck,
    live_runtime: next_verification::PreparedLiveRuntimePerformanceCheck,
}

impl PreparedSmokeScenario {
    fn new(state_root: Option<&Path>) -> Result<Self, String> {
        let scratch_root = state_root.map_or_else(std::env::temp_dir, Path::to_path_buf);
        let streaming = next_verification::prepare_streaming_performance_check_in(&scratch_root)
            .map_err(|error| error.to_string())?;
        let agent =
            match next_verification::prepare_agent_planning_performance_check_in(&scratch_root) {
                Ok(agent) => agent,
                Err(error) => {
                    let mut failure = format!("prepare agent-planning: {error}");
                    append_cleanup_error(&mut failure, "streaming", streaming.cancel());
                    return Err(failure);
                }
            };
        let render_planning =
            match next_verification::prepare_render_frame_planning_performance_check_in(
                &scratch_root,
            ) {
                Ok(render_planning) => render_planning,
                Err(error) => {
                    let mut failure = format!("prepare render-planning: {error}");
                    append_cleanup_error(&mut failure, "agent-planning", agent.cancel());
                    append_cleanup_error(&mut failure, "streaming", streaming.cancel());
                    return Err(failure);
                }
            };
        let live_runtime =
            match next_verification::prepare_live_runtime_performance_check_in(&scratch_root) {
                Ok(live_runtime) => live_runtime,
                Err(error) => {
                    let mut failure = format!("prepare live-runtime: {error}");
                    append_cleanup_error(&mut failure, "render-planning", render_planning.cancel());
                    append_cleanup_error(&mut failure, "agent-planning", agent.cancel());
                    append_cleanup_error(&mut failure, "streaming", streaming.cancel());
                    return Err(failure);
                }
            };
        Ok(Self {
            streaming,
            agent,
            render_planning,
            live_runtime,
        })
    }
}

fn append_cleanup_error<E: ToString>(failure: &mut String, workload: &str, cleanup: Result<(), E>) {
    if let Err(error) = cleanup {
        failure.push_str(&format!("; {workload} cleanup: {}", error.to_string()));
    }
}

fn combine_named_errors<const N: usize>(errors: [(&str, Option<String>); N]) -> String {
    let diagnostics = errors
        .into_iter()
        .filter_map(|(workload, error)| error.map(|error| format!("{workload}: {error}")))
        .collect::<Vec<_>>();
    debug_assert!(!diagnostics.is_empty());
    diagnostics.join("; ")
}

fn run_live_runtime_scenario(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<
    next_verification::LiveRuntimePerformanceReport,
    next_verification::LiveRuntimePerformanceError,
> {
    match (scenario, state_root) {
        (
            xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak
            | xtask::performance::PerformanceScenarioV1::R2AlphaRender,
            Some(root),
        ) => next_verification::run_live_runtime_performance_check_in(root),
        (
            xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak
            | xtask::performance::PerformanceScenarioV1::R2AlphaRender,
            None,
        ) => next_verification::run_live_runtime_performance_check(),
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, Some(root)) => {
            next_verification::run_live_runtime_long_session_performance_check_in(root)
        }
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, None) => {
            next_verification::run_live_runtime_long_session_performance_check()
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    }
}

fn prepare_live_runtime_scenario(
    scenario: xtask::performance::PerformanceScenarioV1,
    state_root: Option<&Path>,
) -> Result<next_verification::PreparedLiveRuntimePerformanceCheck, String> {
    let prepared = match (scenario, state_root) {
        (
            xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak
            | xtask::performance::PerformanceScenarioV1::R2AlphaRender,
            Some(root),
        ) => next_verification::prepare_live_runtime_performance_check_in(root),
        (
            xtask::performance::PerformanceScenarioV1::Smoke
            | xtask::performance::PerformanceScenarioV1::InteractiveFrameSoak
            | xtask::performance::PerformanceScenarioV1::ProductionWorkerSoak
            | xtask::performance::PerformanceScenarioV1::R2AlphaRender,
            None,
        ) => next_verification::prepare_live_runtime_performance_check(),
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, Some(root)) => {
            next_verification::prepare_live_runtime_long_session_performance_check_in(root)
        }
        (xtask::performance::PerformanceScenarioV1::LongSessionSoak, None) => {
            next_verification::prepare_live_runtime_long_session_performance_check()
        }
        _ => unreachable!("unavailable representative scenarios return before execution"),
    };
    prepared.map_err(|error| error.to_string())
}
