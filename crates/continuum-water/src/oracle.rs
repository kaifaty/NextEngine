#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use crate::boundary;
use crate::error::{
    INVARIANT_MISMATCH, NONDETERMINISTIC_RESULT, NUMERIC_OVERFLOW, REPORT_CAPACITY_EXCEEDED,
    SCENARIO_INVALID, WaterError,
};
use crate::hash::{self, FrozenRoots, TrajectoryHasher};
use crate::model::{
    AcceptedFrame, CanonicalSample, OutputMetric, Scenario, StepSummary, StorageOrder, Vec3f,
    Vec3i, checked_scalar,
};
use crate::profile::{
    DT, GRAVITY_MAGNITUDE, MAXIMUM_REPORT_BYTES, UNIFORM_MASS, decode_micrometres, decode_velocity,
    quantize_micrometres, quantize_ppb, quantize_velocity,
};
use crate::reference::{self, CurveComparison};
use crate::{profile, scenario, solver};

pub(crate) mod command;
pub(crate) mod successor;

use command::{
    parse_arguments, preflight_report_plan, report_reserve_error, tool_commit, tool_tree_state,
    validate_output_path, write_report,
};

#[derive(Clone, Debug)]
struct Request {
    scenario_id: String,
    output: PathBuf,
    reference: Option<PathBuf>,
    repeat: u8,
    storage_order: StorageOrder,
}

#[derive(Clone, Debug, Serialize)]
struct CommandEnvelope<T> {
    schema_version: u32,
    status: String,
    command: String,
    details: T,
}

#[derive(Clone, Debug, Serialize)]
struct CommandDetails {
    report: String,
    scenario: String,
    scenario_evidence_status: String,
    product_check: String,
    trajectory_root: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct WaterLabDetails {
    report_schema: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    roots: Option<RootsReport>,
    scenario: Option<ScenarioReport>,
    terminal: TerminalReport,
    product_check: ProductCheckReport,
    step_summaries: Vec<StepSummary>,
    output_metrics: Vec<OutputMetric>,
    analytical_checks: Vec<CheckReport>,
    reference: ReferenceReport,
    repeat_roots: Vec<NamedRoot>,
    permutation_roots: Vec<NamedRoot>,
    trajectory_root: Option<String>,
    timing: TimingReport,
}

#[derive(Clone, Debug, Serialize)]
struct RootsReport {
    w0b_document_root: String,
    float_profile_root: String,
    corpus_root: String,
    execution_profile_root: String,
    scenario_root: String,
}

#[derive(Clone, Debug, Serialize)]
struct ScenarioReport {
    id: String,
    kind: String,
    smoke_only: bool,
    storage_order: String,
    sample_count: usize,
    boundary_sample_count: usize,
    steps: u32,
    output_every: u32,
}

#[derive(Clone, Debug, Serialize)]
struct TerminalReport {
    status: String,
    code: Option<String>,
    detail: Option<String>,
    last_accepted_step: Option<u32>,
    last_accepted_frame_root: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ProductCheckReport {
    id: String,
    status: String,
    reason: String,
    scenario_evidence_status: String,
}

#[derive(Clone, Debug, Serialize)]
struct CheckReport {
    check: String,
    status: String,
    detail: String,
}

#[derive(Clone, Debug, Serialize)]
struct ReferenceReport {
    status: String,
    path: Option<String>,
    sha256: Option<String>,
    comparisons: Vec<CurveComparison>,
}

#[derive(Clone, Debug, Serialize)]
struct NamedRoot {
    name: String,
    trajectory_root: String,
}

#[derive(Clone, Debug, Serialize)]
struct TimingReport {
    classification: String,
    wall_clock_nanoseconds: u64,
}

struct ReportState {
    details: WaterLabDetails,
}

struct RunEvidence {
    trajectory_root: [u8; 32],
    freefall_mismatch: Option<String>,
}

#[derive(Clone, Copy)]
struct PhysicalTotals {
    kinetic: f64,
    potential: f64,
    momentum: Vec3f,
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let request = parse_arguments(arguments)?;
    validate_output_path(repository_root, &request.output)?;
    let mut state = ReportState::new(repository_root, &request);
    let execution = execute(repository_root, &request, &mut state);
    if let Err(error) = &execution {
        state.fail(error);
    }
    write_report(&request.output, &mut state)?;
    match execution {
        Ok(()) => {
            let command = CommandEnvelope {
                schema_version: 1,
                status: "REPORT_ONLY".to_owned(),
                command: "continuum water oracle".to_owned(),
                details: CommandDetails {
                    report: request.output.display().to_string(),
                    scenario: request.scenario_id,
                    scenario_evidence_status: state
                        .details
                        .product_check
                        .scenario_evidence_status
                        .clone(),
                    product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
                    trajectory_root: state.details.trajectory_root.clone(),
                },
            };
            serde_json::to_string(&command).map_err(|error| {
                WaterError::new(
                    REPORT_CAPACITY_EXCEEDED,
                    format!("cannot serialize command report: {error}"),
                )
            })
        }
        Err(error) => Err(WaterError::new(
            error.code(),
            format!(
                "{}; failure report written to {}",
                error.detail(),
                request.output.display()
            ),
        )),
    }
}

impl ReportState {
    fn new(repository_root: &Path, request: &Request) -> Self {
        Self {
            details: WaterLabDetails {
                report_schema: "nextengine.continuum-water.lab-report.v1".to_owned(),
                tool_commit: tool_commit(repository_root),
                tool_tree_state: tool_tree_state(repository_root),
                toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
                build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
                build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
                roots: None,
                scenario: None,
                terminal: TerminalReport {
                    status: "RUNNING".to_owned(),
                    code: None,
                    detail: None,
                    last_accepted_step: None,
                    last_accepted_frame_root: None,
                },
                product_check: ProductCheckReport {
                    id: "CONTINUUM-WATER-REF-P1".to_owned(),
                    status: "NOT_RUN".to_owned(),
                    reason: "one scenario report cannot satisfy the full W0B corpus gate"
                        .to_owned(),
                    scenario_evidence_status: "NOT_EXECUTED".to_owned(),
                },
                step_summaries: Vec::new(),
                output_metrics: Vec::new(),
                analytical_checks: Vec::new(),
                reference: ReferenceReport {
                    status: if request.reference.is_some() {
                        "PENDING".to_owned()
                    } else {
                        "REFERENCE_NOT_PROVIDED".to_owned()
                    },
                    path: request
                        .reference
                        .as_ref()
                        .map(|path| path.display().to_string()),
                    sha256: None,
                    comparisons: Vec::new(),
                },
                repeat_roots: Vec::new(),
                permutation_roots: Vec::new(),
                trajectory_root: None,
                timing: TimingReport {
                    classification: "DIAGNOSTIC_ONLY".to_owned(),
                    wall_clock_nanoseconds: 0,
                },
            },
        }
    }

    fn accept_frame(&mut self, frame: &AcceptedFrame, summary: StepSummary) {
        self.details.terminal.last_accepted_step = Some(frame.step);
        self.details.terminal.last_accepted_frame_root = Some(hash::hex(&frame.frame_root));
        self.details.step_summaries.push(summary);
    }

    fn fail(&mut self, error: &WaterError) {
        self.details.terminal.status = "FAILED".to_owned();
        self.details.terminal.code = Some(error.code().to_owned());
        self.details.terminal.detail = Some(error.detail().to_owned());
        self.details.product_check.scenario_evidence_status = "FAILED".to_owned();
    }
}

fn execute(
    repository_root: &Path,
    request: &Request,
    state: &mut ReportState,
) -> Result<(), WaterError> {
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    let roots = FrozenRoots::verify(repository_root)?;
    let scenario = scenario::find(&request.scenario_id)?;
    let scenario_root = scenario::root_for(&scenario, &roots)?;
    let order = if scenario.id.ends_with("ORDER-001") {
        StorageOrder::Identity
    } else {
        request.storage_order
    };
    let samples = scenario::initial_samples(&scenario, order)?;
    let boundary = boundary::build(scenario.geometry)?;
    state.details.roots = Some(RootsReport {
        w0b_document_root: hash::hex(&roots.document),
        float_profile_root: hash::hex(&roots.float_profile),
        corpus_root: hash::hex(&roots.corpus),
        execution_profile_root: hash::hex(&roots.execution_profile),
        scenario_root: hash::hex(&scenario_root),
    });
    state.details.scenario = Some(ScenarioReport {
        id: scenario.id.to_owned(),
        kind: scenario.kind.to_owned(),
        smoke_only: scenario.smoke_only,
        storage_order: order.label().to_owned(),
        sample_count: samples.len(),
        boundary_sample_count: boundary.len(),
        steps: scenario.steps,
        output_every: scenario.output_every,
    });
    let frame_count = usize::try_from(scenario.steps)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| WaterError::new(REPORT_CAPACITY_EXCEEDED, "report frame count overflow"))?;
    state
        .details
        .step_summaries
        .try_reserve_exact(frame_count)
        .map_err(report_reserve_error)?;
    let output_count = reference::output_steps(&scenario)?.len();
    state
        .details
        .output_metrics
        .try_reserve_exact(output_count)
        .map_err(report_reserve_error)?;
    state
        .details
        .analytical_checks
        .try_reserve_exact(4)
        .map_err(report_reserve_error)?;
    state
        .details
        .repeat_roots
        .try_reserve_exact(usize::from(request.repeat))
        .map_err(report_reserve_error)?;
    state
        .details
        .permutation_roots
        .try_reserve_exact(3)
        .map_err(report_reserve_error)?;
    preflight_report_plan(frame_count, output_count)?;
    if let Some(path) = &request.reference {
        let _ = reference::preflight_size(path)?;
    }

    let primary_started = Instant::now();
    let primary_result = run_once(
        &scenario,
        samples,
        &boundary,
        &roots.execution_profile,
        &scenario_root,
        Some(state),
    );
    state.details.timing.wall_clock_nanoseconds =
        u64::try_from(primary_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let primary = primary_result?;
    state.details.trajectory_root = Some(hash::hex(&primary.trajectory_root));

    validate_common_metrics(
        &scenario,
        &state.details.step_summaries,
        &state.details.output_metrics,
    )?;
    validate_scenario_metrics(
        &scenario,
        &state.details.output_metrics,
        primary.freefall_mismatch.as_deref(),
        &mut state.details.analytical_checks,
    )?;

    run_repeat_and_permutation_checks(
        request,
        &scenario,
        &boundary,
        &roots.execution_profile,
        &scenario_root,
        primary.trajectory_root,
        state,
    )?;

    if let Some(reference_path) = &request.reference {
        let reference = reference::load(
            reference_path,
            &scenario,
            &scenario_root,
            state
                .details
                .scenario
                .as_ref()
                .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "scenario report is missing"))?
                .sample_count,
        )?;
        let comparisons =
            reference::compare_curves(&scenario, &state.details.output_metrics, &reference)?;
        if let Some(failed) = comparisons.iter().find(|comparison| !comparison.passed) {
            let failed_metric = failed.metric;
            state.details.reference.status = "FAILED".to_owned();
            state.details.reference.sha256 = Some(reference.sha256);
            state.details.reference.comparisons = comparisons;
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!(
                    "reference curve {} exceeds its frozen threshold",
                    failed_metric
                ),
            ));
        }
        state.details.reference.status = "IMPORTED_AND_COMPARED".to_owned();
        state.details.reference.sha256 = Some(reference.sha256);
        state.details.reference.comparisons = comparisons;
    }

    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != state.details.tool_tree_state {
        state.details.tool_tree_state =
            format!("{}->{}", state.details.tool_tree_state, ending_tree_state);
    }
    let scenario_status = scenario_evidence_status(request, &scenario, state);
    state.details.product_check.scenario_evidence_status = scenario_status.to_owned();
    state.details.terminal.status = "COMPLETED".to_owned();
    state.details.product_check.reason = if state.details.tool_tree_state != "CLEAN" {
        "the implementation worktree is not clean; this run has diagnostic value only".to_owned()
    } else if scenario.smoke_only {
        "SMOKE_ONLY scenarios have distinct roots and receive no W0B corpus credit".to_owned()
    } else {
        "scenario-local evidence is recorded; the full nominal corpus, external inputs and repeats are still aggregated separately".to_owned()
    };
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_once(
    scenario: &Scenario,
    samples: Vec<CanonicalSample>,
    boundary: &[boundary::BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    mut report: Option<&mut ReportState>,
) -> Result<RunEvidence, WaterError> {
    let (mut frame, initial_summary) = solver::initial_frame(
        samples,
        scenario.geometry,
        boundary,
        execution_profile_root,
        scenario_root,
    )?;
    let mut trajectory = TrajectoryHasher::new(
        scenario
            .steps
            .checked_add(1)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "frame count overflow"))?,
    );
    trajectory.push(0, &frame.frame_root)?;
    let baseline = physical_totals(&frame)?;
    let mut gravity_impulse = Vec3f::ZERO;
    let mut boundary_impulse = Vec3f::ZERO;
    let initial_metric = output_metric(
        scenario,
        &frame,
        initial_summary.centre_of_mass_um,
        baseline,
        gravity_impulse,
        boundary_impulse,
    )?;
    let mut expected_freefall = if scenario.id.ends_with("FREEFALL-001") {
        let mut expected = Vec::new();
        expected
            .try_reserve_exact(frame.samples.len())
            .map_err(report_reserve_error)?;
        expected.extend_from_slice(&frame.samples);
        Some(expected)
    } else {
        None
    };
    if let Some(report) = report.as_deref_mut() {
        report.accept_frame(&frame, initial_summary);
        report.details.output_metrics.push(initial_metric);
    }
    let mut freefall_mismatch = None;
    for step in 1..=scenario.steps {
        let outcome = solver::substep(
            &frame,
            scenario.geometry,
            boundary,
            execution_profile_root,
            scenario_root,
        )?;
        for _sample in &outcome.frame.samples {
            let mass_dt = checked_scalar(UNIFORM_MASS * DT, "gravity impulse mass dt")?;
            let gravity_y = checked_scalar(mass_dt * -GRAVITY_MAGNITUDE, "gravity impulse y")?;
            let gravity_row = Vec3f::new(0.0, gravity_y, 0.0);
            gravity_impulse = gravity_impulse
                .add(gravity_row)
                .checked("gravity impulse reduction")?;
        }
        for impulse in outcome.boundary_impulses {
            boundary_impulse = boundary_impulse
                .add(impulse)
                .checked("run boundary impulse reduction")?;
        }
        if let Some(expected) = &mut expected_freefall {
            advance_freefall(expected)?;
            if freefall_mismatch.is_none() {
                freefall_mismatch = compare_freefall(
                    scenario.id,
                    step,
                    expected,
                    &outcome.frame,
                    &outcome.summary,
                );
            }
        }
        trajectory.push(step, &outcome.frame.frame_root)?;
        let output_due = step % scenario.output_every == 0;
        let metric = if output_due {
            Some(output_metric(
                scenario,
                &outcome.frame,
                outcome.summary.centre_of_mass_um,
                baseline,
                gravity_impulse,
                boundary_impulse,
            )?)
        } else {
            None
        };
        frame = outcome.frame;
        if let Some(report) = report.as_deref_mut() {
            report.accept_frame(&frame, outcome.summary);
            if let Some(metric) = metric {
                report.details.output_metrics.push(metric);
            }
        }
    }
    Ok(RunEvidence {
        trajectory_root: trajectory.finish()?,
        freefall_mismatch,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_repeat_and_permutation_checks(
    request: &Request,
    scenario: &Scenario,
    boundary: &[boundary::BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    primary_root: [u8; 32],
    state: &mut ReportState,
) -> Result<(), WaterError> {
    state.details.repeat_roots.push(NamedRoot {
        name: "run-1".to_owned(),
        trajectory_root: hash::hex(&primary_root),
    });
    for repeat in 2..=request.repeat {
        let samples = scenario::initial_samples(
            scenario,
            if scenario.id.ends_with("ORDER-001") {
                StorageOrder::Identity
            } else {
                request.storage_order
            },
        )?;
        let evidence = run_once(
            scenario,
            samples,
            boundary,
            execution_profile_root,
            scenario_root,
            None,
        )?;
        state.details.repeat_roots.push(NamedRoot {
            name: format!("run-{repeat}"),
            trajectory_root: hash::hex(&evidence.trajectory_root),
        });
        if evidence.trajectory_root != primary_root {
            return Err(WaterError::new(
                NONDETERMINISTIC_RESULT,
                format!("repeat {repeat} trajectory root differs from run 1"),
            ));
        }
    }
    if scenario.id.ends_with("ORDER-001") {
        state.details.permutation_roots.push(NamedRoot {
            name: StorageOrder::Identity.label().to_owned(),
            trajectory_root: hash::hex(&primary_root),
        });
        for order in [StorageOrder::Reverse, StorageOrder::Affine] {
            let evidence = run_once(
                scenario,
                scenario::initial_samples(scenario, order)?,
                boundary,
                execution_profile_root,
                scenario_root,
                None,
            )?;
            state.details.permutation_roots.push(NamedRoot {
                name: order.label().to_owned(),
                trajectory_root: hash::hex(&evidence.trajectory_root),
            });
            if evidence.trajectory_root != primary_root {
                return Err(WaterError::new(
                    NONDETERMINISTIC_RESULT,
                    format!(
                        "{} storage order changed the trajectory root",
                        order.label()
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn physical_totals(frame: &AcceptedFrame) -> Result<PhysicalTotals, WaterError> {
    let mut kinetic = 0.0;
    let mut potential = 0.0;
    let mut momentum = Vec3f::ZERO;
    for sample in &frame.samples {
        let velocity = Vec3f::new(
            decode_velocity(sample.velocity_um_s.x)?,
            decode_velocity(sample.velocity_um_s.y)?,
            decode_velocity(sample.velocity_um_s.z)?,
        );
        let y = decode_micrometres(sample.position_um.y)?;
        let speed_squared = checked_scalar(velocity.dot(velocity), "kinetic speed squared")?;
        let row_kinetic = checked_scalar(0.5 * UNIFORM_MASS * speed_squared, "kinetic row")?;
        kinetic = checked_scalar(kinetic + row_kinetic, "kinetic reduction")?;
        let row_potential = checked_scalar(UNIFORM_MASS * GRAVITY_MAGNITUDE * y, "potential row")?;
        potential = checked_scalar(potential + row_potential, "potential reduction")?;
        momentum = momentum
            .add(velocity.scale(UNIFORM_MASS))
            .checked("momentum reduction")?;
    }
    Ok(PhysicalTotals {
        kinetic,
        potential,
        momentum,
    })
}

fn output_metric(
    scenario: &Scenario,
    frame: &AcceptedFrame,
    centre_of_mass_um: Vec3i,
    baseline: PhysicalTotals,
    gravity_impulse: Vec3f,
    boundary_impulse: Vec3f,
) -> Result<OutputMetric, WaterError> {
    let count = frame.samples.len();
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    xs.try_reserve_exact(count).map_err(report_reserve_error)?;
    ys.try_reserve_exact(count).map_err(report_reserve_error)?;
    let mut receiver_count = 0_u32;
    for sample in &frame.samples {
        xs.push(sample.position_um.x);
        ys.push(sample.position_um.y);
        let partition_x = scenario
            .geometry
            .aperture
            .map_or(1_000_000, |aperture| aperture.wall_x_um);
        if sample.position_um.x >= partition_x {
            receiver_count = receiver_count.checked_add(1).ok_or_else(|| {
                WaterError::new(NUMERIC_OVERFLOW, "receiver sample count overflow")
            })?;
        }
    }
    xs.sort_unstable();
    ys.sort_unstable();
    let current = physical_totals(frame)?;
    let baseline_energy = checked_scalar(
        baseline.kinetic + baseline.potential,
        "baseline mechanical energy",
    )?;
    let current_energy = checked_scalar(
        current.kinetic + current.potential,
        "current mechanical energy",
    )?;
    let energy_denominator_sum = checked_scalar(
        baseline.kinetic.abs() + baseline.potential.abs(),
        "energy denominator",
    )?;
    let energy_denominator = if energy_denominator_sum > 1.0 {
        energy_denominator_sum
    } else {
        1.0
    };
    let energy_residual = checked_scalar(
        (current_energy - baseline_energy).abs() / energy_denominator,
        "energy residual",
    )?;

    let expected_momentum = baseline
        .momentum
        .add(gravity_impulse)
        .add(boundary_impulse)
        .checked("expected momentum")?;
    let momentum_delta = current
        .momentum
        .sub(expected_momentum)
        .checked("momentum delta")?;
    let denominator_sum = vector_norm(baseline.momentum)?
        + vector_norm(gravity_impulse)?
        + vector_norm(boundary_impulse)?;
    let momentum_denominator = if denominator_sum > 1.0 {
        denominator_sum
    } else {
        1.0
    };
    let momentum_residual = checked_scalar(
        vector_norm(momentum_delta)? / momentum_denominator,
        "momentum residual",
    )?;
    let sample_count = u32::try_from(count)
        .map_err(|_| WaterError::new(NUMERIC_OVERFLOW, "metric sample count overflow"))?;
    let mass_mg = u64::try_from(count)
        .ok()
        .and_then(|value| value.checked_mul(125_000))
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "exact mass overflow"))?;
    Ok(OutputMetric {
        step: frame.step,
        sample_count,
        mass_mg,
        centre_of_mass_um,
        q99_x_um: reference::nearest_rank_99(&xs)?,
        q99_y_um: reference::nearest_rank_99(&ys)?,
        receiver_count,
        energy_residual_ppb: quantize_ppb(energy_residual)?,
        momentum_residual_ppb: quantize_ppb(momentum_residual)?,
    })
}

fn vector_norm(value: Vec3f) -> Result<f64, WaterError> {
    checked_scalar(value.dot(value), "vector norm square")
        .and_then(|square| checked_scalar(square.sqrt(), "vector norm"))
}

fn validate_common_metrics(
    scenario: &Scenario,
    summaries: &[StepSummary],
    outputs: &[OutputMetric],
) -> Result<(), WaterError> {
    let expected_count = scenario::initial_samples(scenario, StorageOrder::Identity)?.len();
    let expected_mass = u64::try_from(expected_count)
        .ok()
        .and_then(|value| value.checked_mul(125_000))
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "expected mass overflow"))?;
    for summary in summaries.iter().skip(1) {
        if !(2..=20).contains(&summary.density_iterations)
            || summary.density_error_ppb > 100_000
            || !(1..=20).contains(&summary.divergence_iterations)
            || summary.divergence_error_ppb > 1_000_000
            || summary.maximum_penetration_um > 2_500
        {
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!("step {} violates a common solver bound", summary.step),
            ));
        }
    }
    for output in outputs {
        if output.sample_count as usize != expected_count || output.mass_mg != expected_mass {
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!("step {} changed exact count or mass", output.step),
            ));
        }
        if output.energy_residual_ppb > 10_000_000 {
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!(
                    "step {} energy residual {} ppb exceeds 10000000",
                    output.step, output.energy_residual_ppb
                ),
            ));
        }
        if output.momentum_residual_ppb > 10_000_000 {
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!(
                    "step {} momentum residual {} ppb exceeds 10000000",
                    output.step, output.momentum_residual_ppb
                ),
            ));
        }
    }
    Ok(())
}

fn validate_scenario_metrics(
    scenario: &Scenario,
    outputs: &[OutputMetric],
    freefall_mismatch: Option<&str>,
    checks: &mut Vec<CheckReport>,
) -> Result<(), WaterError> {
    if let Some(mismatch) = freefall_mismatch {
        return Err(WaterError::new(INVARIANT_MISMATCH, mismatch.to_owned()));
    }
    match scenario.id {
        "CW-HYDRO-001" => {
            for output in outputs.iter().filter(|output| output.step >= 960) {
                if (output.centre_of_mass_um.x - 500_000).unsigned_abs() > 2_500
                    || (output.centre_of_mass_um.z - 500_000).unsigned_abs() > 2_500
                    || (output.centre_of_mass_um.y - 375_000).unsigned_abs() > 25_000
                {
                    return Err(WaterError::new(
                        INVARIANT_MISMATCH,
                        format!("hydrostatic COM bound failed at step {}", output.step),
                    ));
                }
            }
            checks.push(pass_check("hydrostatic-centre-of-mass"));
        }
        "CW-FREEFALL-001" | "SMOKE-CW-FREEFALL-001" => {
            checks.push(pass_check("exact-canonical-gravity-recurrence"));
        }
        "CW-STILL-001" => {
            let initial = outputs.first().ok_or_else(|| {
                WaterError::new(INVARIANT_MISMATCH, "still tank has no initial output")
            })?;
            for output in outputs {
                if (output.centre_of_mass_um.x - initial.centre_of_mass_um.x).unsigned_abs() > 2_500
                    || (output.centre_of_mass_um.z - initial.centre_of_mass_um.z).unsigned_abs()
                        > 2_500
                {
                    return Err(WaterError::new(
                        INVARIANT_MISMATCH,
                        format!(
                            "still-tank lateral COM drift failed at step {}",
                            output.step
                        ),
                    ));
                }
            }
            checks.push(pass_check("still-tank-lateral-com"));
        }
        "CW-ORIFICE-001" => {
            let expected =
                u32::try_from(scenario::initial_samples(scenario, StorageOrder::Identity)?.len())
                    .map_err(|_| WaterError::new(NUMERIC_OVERFLOW, "partition count overflow"))?;
            for output in outputs {
                let left = expected.checked_sub(output.receiver_count).ok_or_else(|| {
                    WaterError::new(INVARIANT_MISMATCH, "receiver count exceeds total")
                })?;
                if left + output.receiver_count != expected {
                    return Err(WaterError::new(
                        INVARIANT_MISMATCH,
                        "orifice partition accounting changed total count",
                    ));
                }
            }
            checks.push(pass_check("orifice-exact-partition-accounting"));
        }
        "CW-SEALED-001" => checks.push(pass_check("sealed-count-and-mass")),
        _ => {}
    }
    checks.push(pass_check("common-count-mass-solver-work-bounds"));
    Ok(())
}

fn pass_check(name: &str) -> CheckReport {
    CheckReport {
        check: name.to_owned(),
        status: "PASS".to_owned(),
        detail: "frozen scenario-local rule satisfied".to_owned(),
    }
}

fn advance_freefall(samples: &mut [CanonicalSample]) -> Result<(), WaterError> {
    for sample in samples {
        let velocity_y = checked_scalar(
            decode_velocity(sample.velocity_um_s.y)? + (DT * -GRAVITY_MAGNITUDE),
            "freefall expected velocity",
        )?;
        let position_y = checked_scalar(
            decode_micrometres(sample.position_um.y)? + (DT * velocity_y),
            "freefall expected position",
        )?;
        sample.velocity_um_s.y = quantize_velocity(velocity_y)?;
        sample.position_um.y = quantize_micrometres(position_y)?;
    }
    Ok(())
}

fn compare_freefall(
    scenario_id: &str,
    step: u32,
    expected: &[CanonicalSample],
    actual: &AcceptedFrame,
    summary: &StepSummary,
) -> Option<String> {
    if expected != actual.samples {
        return Some(format!("freefall canonical frame differs at step {step}"));
    }
    if summary.density_iterations != 2
        || summary.divergence_iterations != 1
        || summary.density_maximum_multiplier_bits != "0x0000000000000000"
        || summary.divergence_maximum_multiplier_bits != "0x0000000000000000"
    {
        return Some(format!(
            "freefall multiplier or iteration invariant differs at step {step}"
        ));
    }
    let checkpoint = if scenario_id == "CW-FREEFALL-001" {
        match step {
            1 => Some((1_274_830, -40_875)),
            2 => Some((1_274_489, -81_750)),
            24 => Some((1_223_907, -981_000)),
            48 => Some((1_074_713, -1_962_000)),
            72 => Some((827_419, -2_943_000)),
            96 => Some((482_025, -3_924_000)),
            _ => None,
        }
    } else {
        None
    };
    if let Some((expected_y, expected_vy)) = checkpoint {
        let lowest = actual
            .samples
            .iter()
            .min_by_key(|sample| sample.position_um.y)?;
        if (lowest.position_um.y, lowest.velocity_um_s.y) != (expected_y, expected_vy) {
            return Some(format!(
                "freefall checkpoint {step} is ({}, {}), expected ({expected_y}, {expected_vy})",
                lowest.position_um.y, lowest.velocity_um_s.y
            ));
        }
    }
    None
}

fn scenario_evidence_status<'a>(
    request: &Request,
    scenario: &Scenario,
    state: &'a ReportState,
) -> &'a str {
    if state.details.tool_tree_state != "CLEAN" {
        "UNCOMMITTED_NO_EVIDENCE_CREDIT"
    } else if scenario.smoke_only {
        "SMOKE_ONLY_NO_CORPUS_CREDIT"
    } else {
        let reference_ready = !matches!(
            scenario.id,
            "CW-HYDRO-001" | "CW-DAMBREAK-001" | "CW-ORIFICE-001"
        ) || request.reference.is_some();
        let repeat_ready =
            !matches!(scenario.id, "CW-STILL-001" | "CW-SEALED-001") || request.repeat >= 2;
        let permutations_ready =
            !scenario.id.ends_with("ORDER-001") || state.details.permutation_roots.len() == 3;
        if reference_ready && repeat_ready && permutations_ready {
            "SCENARIO_PASS"
        } else {
            "EVIDENCE_INCOMPLETE"
        }
    }
}
