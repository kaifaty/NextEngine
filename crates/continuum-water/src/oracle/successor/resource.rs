#![forbid(unsafe_code)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::calibration::successor::build_density_support;
use crate::error::{PROFILE_MISMATCH, REPORT_CAPACITY_EXCEEDED, SCENARIO_INVALID, WaterError};
use crate::hash::{self, AcceleratedPressureRoots, TrajectoryHasher};
use crate::model::StorageOrder;
use crate::oracle::command::{
    tool_commit, tool_tree_state, validate_output_path, validate_report_capacity,
};
use crate::solver::StepStageTimings;
use crate::{profile, scenario, solver};

use super::validation::validate_step;

const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";
const SCENARIO_ID: &str = "CW-SEALED-001";
const WARMUP_STEPS: u32 = 1;
const MEASURED_STEPS: u32 = 3;
const WORKLOAD_PROJECTION: &str = concat!(
    "workload=continuum-water-50k-stage-profile.v0\n",
    "classification=diagnostic-only-no-w2-credit\n",
    "scenario=CW-SEALED-001\n",
    "nominal-samples=48000\n",
    "hard-sample-cap=50000\n",
    "warmup-substeps=1\n",
    "measured-substeps=3\n",
    "solver=frozen-w0h-accelerated-projected-gradient50-sequential-contact\n",
    "workers=1\n",
    "setup-and-report-io=excluded\n",
    "substep-rebuild-through-canonical-root=included\n",
);

struct Request {
    output: PathBuf,
}

#[derive(Serialize)]
struct Envelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a ResourceProfileReport,
}

#[derive(Serialize)]
struct ResourceProfileReport {
    report_schema: &'static str,
    classification: &'static str,
    disposition: &'static str,
    product_check: &'static str,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: &'static str,
    build_profile: &'static str,
    build_rustflags: String,
    scope: ScopeReport,
    host: HostReport,
    workload: WorkloadReport,
    roots: RootReport,
    setup_wall_clock_nanoseconds: u64,
    initial_frame_wall_clock_nanoseconds: u64,
    warmup_wall_clock_nanoseconds: u64,
    measured_loop_wall_clock_nanoseconds: u64,
    command_wall_clock_nanoseconds: u64,
    warmup_frame_root: String,
    measured_steps: Vec<StepObservation>,
    aggregate_stage_timings: StepStageTimings,
    aggregate_stage_shares: Vec<StageShare>,
    short_trajectory_root: String,
    final_frame_root: String,
}

#[derive(Serialize)]
struct ScopeReport {
    current_target: &'static str,
    windows: &'static str,
    timing_semantics: &'static str,
    gate_credit: &'static str,
}

#[derive(Serialize)]
struct HostReport {
    available_logical_parallelism: usize,
    configured_workers: usize,
    single_worker_logical_capacity_ceiling_ppm: u64,
    process_cpu_utilization_evidence: &'static str,
}

#[derive(Serialize)]
struct WorkloadReport {
    name: &'static str,
    projection: &'static str,
    projection_root: String,
    scenario_id: &'static str,
    sample_count: usize,
    hard_sample_cap: usize,
    static_boundary_sample_count: usize,
    warmup_steps: u32,
    measured_steps: u32,
    physical_frequency_hz: u32,
    solver: &'static str,
}

#[derive(Serialize)]
struct RootReport {
    w0h_execution_profile: String,
    scenario: String,
    initial_frame: String,
}

#[derive(Serialize)]
struct StepObservation {
    step: u32,
    frame_root: String,
    density_iterations: u8,
    divergence_iterations: u8,
    outer_wall_clock_nanoseconds: u64,
    stages: StepStageTimings,
}

#[derive(Serialize)]
struct StageShare {
    stage: &'static str,
    nanoseconds: u64,
    share_ppm: u64,
}

pub(super) fn run(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let request = parse_arguments(arguments)?;
    validate_output_path(repository_root, &request.output)?;
    let started = Instant::now();
    let mut report = execute(repository_root)?;
    report.command_wall_clock_nanoseconds = elapsed_nanoseconds(started);
    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != report.tool_tree_state {
        report.tool_tree_state = format!("{}->{ending_tree_state}", report.tool_tree_state);
    }
    write_report(&request.output, &report)?;
    command_result(&request, &report)
}

fn execute(repository_root: &Path) -> Result<ResourceProfileReport, WaterError> {
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    if env!("WATER_BUILD_TARGET") != LINUX_TARGET {
        return Err(WaterError::new(
            PROFILE_MISMATCH,
            format!(
                "W2 resource profile requires {LINUX_TARGET}, compiled for {}",
                env!("WATER_BUILD_TARGET")
            ),
        ));
    }
    let roots = AcceleratedPressureRoots::verify(repository_root)?;
    let selected = scenario::find(SCENARIO_ID)?;
    if roots.parent.parent.scenario_projection(SCENARIO_ID)?
        != scenario::successor_projection(&selected)?.as_bytes()
    {
        return Err(WaterError::new(
            PROFILE_MISMATCH,
            "W2 resource profile scenario differs from the frozen successor projection",
        ));
    }
    let scenario_root = roots.scenario_root(SCENARIO_ID)?;
    let setup_started = Instant::now();
    let samples = scenario::initial_samples(&selected, StorageOrder::Reverse)?;
    let boundary = build_density_support(selected.geometry)?;
    let setup_wall_clock_nanoseconds = elapsed_nanoseconds(setup_started);
    let sample_count = samples.len();
    let static_boundary_sample_count = boundary.len();

    let initial_started = Instant::now();
    let (mut frame, _summary) = solver::initial_frame(
        samples,
        selected.geometry,
        &boundary,
        &roots.execution_profile,
        &scenario_root,
    )?;
    let initial_frame_wall_clock_nanoseconds = elapsed_nanoseconds(initial_started);
    let initial_frame_root = hash::hex(&frame.frame_root);
    let frame_count = 1_u32
        .checked_add(WARMUP_STEPS)
        .and_then(|count| count.checked_add(MEASURED_STEPS))
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "profile frame count overflow"))?;
    let mut trajectory = TrajectoryHasher::new(frame_count);
    trajectory.push(0, &frame.frame_root)?;

    let warmup_started = Instant::now();
    for _ in 0..WARMUP_STEPS {
        let next = solver::successor_accelerated_projected_gradient_substep(
            &frame,
            selected.geometry,
            &boundary,
            &roots.execution_profile,
            &scenario_root,
        )?;
        validate_step(&selected, &next)?;
        trajectory.push(next.outcome.frame.step, &next.outcome.frame.frame_root)?;
        frame = next.outcome.frame;
    }
    let warmup_wall_clock_nanoseconds = elapsed_nanoseconds(warmup_started);
    let warmup_frame_root = hash::hex(&frame.frame_root);

    let mut measured_steps = Vec::new();
    measured_steps
        .try_reserve_exact(MEASURED_STEPS as usize)
        .map_err(|error| {
            WaterError::new(
                REPORT_CAPACITY_EXCEEDED,
                format!("cannot reserve W2 resource observations: {error}"),
            )
        })?;
    let mut aggregate = StepStageTimings::default();
    let measured_started = Instant::now();
    for _ in 0..MEASURED_STEPS {
        let step_started = Instant::now();
        let (next, stages) = solver::timed_successor_accelerated_projected_gradient_substep(
            &frame,
            selected.geometry,
            &boundary,
            &roots.execution_profile,
            &scenario_root,
        )?;
        let outer_wall_clock_nanoseconds = elapsed_nanoseconds(step_started);
        validate_step(&selected, &next)?;
        trajectory.push(next.outcome.frame.step, &next.outcome.frame.frame_root)?;
        aggregate.saturating_add_assign(stages);
        measured_steps.push(StepObservation {
            step: next.outcome.frame.step,
            frame_root: hash::hex(&next.outcome.frame.frame_root),
            density_iterations: next.outcome.summary.density_iterations,
            divergence_iterations: next.outcome.summary.divergence_iterations,
            outer_wall_clock_nanoseconds,
            stages,
        });
        frame = next.outcome.frame;
    }
    let measured_loop_wall_clock_nanoseconds = elapsed_nanoseconds(measured_started);
    let logical_parallelism = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let projection_root = workload_projection_root(&roots.execution_profile, &scenario_root);
    Ok(ResourceProfileReport {
        report_schema: "nextengine.continuum-water.w2-resource-profile.v1",
        classification: "W2_RESOURCE_UTILIZATION_BASELINE_DIAGNOSTIC",
        disposition: "BASELINE_RECORDED / NO_W2_CREDIT",
        product_check: "NOT_APPLICABLE_DIAGNOSTIC_ONLY",
        tool_commit: tool_commit(repository_root),
        tool_tree_state: tool_tree_state(repository_root),
        toolchain_target: env!("WATER_BUILD_TARGET"),
        build_profile: env!("WATER_BUILD_PROFILE"),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        scope: ScopeReport {
            current_target: LINUX_TARGET,
            windows: "OUT_OF_SCOPE_BY_USER / DEFERRED_NOT_WAIVED",
            timing_semantics: "MONOTONIC_WALL_CLOCK / OUTSIDE_CANONICAL_STATE_AND_ROOTS",
            gate_credit: "SHORT_STAGE_DISCRIMINATOR_ONLY / NOT_THE_W2_PERCENTILE_GATE",
        },
        host: HostReport {
            available_logical_parallelism: logical_parallelism,
            configured_workers: 1,
            single_worker_logical_capacity_ceiling_ppm: 1_000_000_u64
                / u64::try_from(logical_parallelism).unwrap_or(u64::MAX),
            process_cpu_utilization_evidence: "CAPTURE_EXTERNALLY_WITH_/usr/bin/time_-v",
        },
        workload: WorkloadReport {
            name: "continuum-water-50k-stage-profile.v0",
            projection: WORKLOAD_PROJECTION,
            projection_root: hash::hex(&projection_root),
            scenario_id: SCENARIO_ID,
            sample_count,
            hard_sample_cap: 50_000,
            static_boundary_sample_count,
            warmup_steps: WARMUP_STEPS,
            measured_steps: MEASURED_STEPS,
            physical_frequency_hz: 240,
            solver: "frozen-w0h-accelerated-projected-gradient50-sequential-contact",
        },
        roots: RootReport {
            w0h_execution_profile: hash::hex(&roots.execution_profile),
            scenario: hash::hex(&scenario_root),
            initial_frame: initial_frame_root,
        },
        setup_wall_clock_nanoseconds,
        initial_frame_wall_clock_nanoseconds,
        warmup_wall_clock_nanoseconds,
        measured_loop_wall_clock_nanoseconds,
        command_wall_clock_nanoseconds: 0,
        warmup_frame_root,
        measured_steps,
        aggregate_stage_timings: aggregate,
        aggregate_stage_shares: stage_shares(aggregate),
        short_trajectory_root: hash::hex(&trajectory.finish()?),
        final_frame_root: hash::hex(&frame.frame_root),
    })
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, WaterError> {
    let flag = arguments.next().ok_or_else(|| {
        WaterError::new(
            SCENARIO_INVALID,
            "profile-w2-linux requires --output <absolute-path>",
        )
    })?;
    let value = arguments
        .next()
        .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "--output requires a value"))?;
    if flag != "--output" || arguments.next().is_some() {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            "profile-w2-linux accepts only --output <absolute-path>",
        ));
    }
    Ok(Request {
        output: PathBuf::from(value),
    })
}

fn workload_projection_root(
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"nextengine.continuum-water.w2-resource-profile.v1\0");
    hasher.update(WORKLOAD_PROJECTION.as_bytes());
    hasher.update(execution_profile_root);
    hasher.update(scenario_root);
    hasher.finalize().into()
}

fn stage_shares(timings: StepStageTimings) -> Vec<StageShare> {
    let stages = [
        ("decode", timings.decode_nanoseconds),
        ("reconstruction", timings.reconstruction_nanoseconds),
        (
            "initial-diagnostics",
            timings.initial_diagnostics_nanoseconds,
        ),
        ("divergence", timings.divergence_nanoseconds),
        ("gravity", timings.gravity_nanoseconds),
        ("density", timings.density_nanoseconds),
        ("contact", timings.contact_nanoseconds),
        ("integration", timings.integration_nanoseconds),
        ("publication", timings.publication_nanoseconds),
    ];
    stages
        .into_iter()
        .map(|(stage, nanoseconds)| StageShare {
            stage,
            nanoseconds,
            share_ppm: ratio_ppm(nanoseconds, timings.total_nanoseconds),
        })
        .collect()
}

fn ratio_ppm(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        return 0;
    }
    u64::try_from((u128::from(numerator) * 1_000_000) / u128::from(denominator)).unwrap_or(u64::MAX)
}

fn elapsed_nanoseconds(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn write_report(output: &Path, report: &ResourceProfileReport) -> Result<(), WaterError> {
    let envelope = Envelope {
        schema_version: 1,
        status: "REPORT_ONLY",
        command: "continuum water profile-w2-linux",
        details: report,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W2 resource profile: {error}"),
        )
    })?;
    bytes.push(b'\n');
    validate_report_capacity(bytes.len())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| {
            WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "cannot create W2 resource report {}: {error}",
                    output.display()
                ),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot write W2 resource report {}: {error}",
                output.display()
            ),
        )
    })?;
    file.sync_all().map_err(|error| {
        WaterError::new(
            SCENARIO_INVALID,
            format!(
                "cannot sync W2 resource report {}: {error}",
                output.display()
            ),
        )
    })
}

fn command_result(request: &Request, report: &ResourceProfileReport) -> Result<String, WaterError> {
    serde_json::to_string(&serde_json::json!({
        "schema_version": 1,
        "status": "REPORT_ONLY",
        "command": "continuum water profile-w2-linux",
        "details": {
            "report": request.output.display().to_string(),
            "disposition": report.disposition,
            "product_check": report.product_check,
            "short_trajectory_root": report.short_trajectory_root,
        }
    }))
    .map_err(|error| {
        WaterError::new(
            REPORT_CAPACITY_EXCEEDED,
            format!("cannot serialize W2 resource command result: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_only_one_output() {
        let request = parse_arguments(
            ["--output", "/tmp/w2-resource.json"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(request.output, PathBuf::from("/tmp/w2-resource.json"));
        assert!(parse_arguments(["--workers", "4"].into_iter().map(str::to_owned)).is_err());
    }

    #[test]
    fn stage_share_uses_bounded_integer_arithmetic() {
        assert_eq!(ratio_ppm(1, 4), 250_000);
        assert_eq!(ratio_ppm(u64::MAX, u64::MAX), 1_000_000);
        assert_eq!(ratio_ppm(1, 0), 0);
    }

    #[test]
    fn timing_does_not_change_the_published_step() {
        let selected = scenario::find("SMOKE-CW-FREEFALL-001").unwrap();
        let samples = scenario::initial_samples(&selected, StorageOrder::Reverse).unwrap();
        let boundary = build_density_support(selected.geometry).unwrap();
        let execution_root = [0x5a; 32];
        let scenario_root = [0xa5; 32];
        let (frame, _) = solver::initial_frame(
            samples,
            selected.geometry,
            &boundary,
            &execution_root,
            &scenario_root,
        )
        .unwrap();
        let serial = solver::successor_accelerated_projected_gradient_substep(
            &frame,
            selected.geometry,
            &boundary,
            &execution_root,
            &scenario_root,
        )
        .unwrap();
        let (timed, _) = solver::timed_successor_accelerated_projected_gradient_substep(
            &frame,
            selected.geometry,
            &boundary,
            &execution_root,
            &scenario_root,
        )
        .unwrap();
        assert_eq!(
            timed.outcome.frame.frame_root,
            serial.outcome.frame.frame_root
        );
        assert_eq!(timed.outcome.frame.samples, serial.outcome.frame.samples);
        assert_eq!(
            serde_json::to_vec(&timed.outcome.summary).unwrap(),
            serde_json::to_vec(&serial.outcome.summary).unwrap()
        );
    }
}
