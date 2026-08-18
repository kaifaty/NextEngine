#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    CheckReport, advance_freefall, compare_freefall, output_metric, physical_totals,
    validate_scenario_metrics, vector_norm,
};
use crate::boundary::BoundarySample;
use crate::calibration::successor::build_density_support;
use crate::error::{
    BOUNDARY_CAPACITY_EXCEEDED, BOUNDARY_NEIGHBOR_CAPACITY_EXCEEDED, INVARIANT_MISMATCH,
    NEIGHBOR_CAPACITY_EXCEEDED, NONDETERMINISTIC_RESULT, NUMERIC_OVERFLOW,
    REPORT_CAPACITY_EXCEEDED, SAMPLE_CAPACITY_EXCEEDED, SCENARIO_INVALID, STEP_CAPACITY_EXCEEDED,
    WaterError,
};
use crate::hash::{self, AcceleratedPressureRoots, TrajectoryHasher};
use crate::model::{
    CanonicalSample, OutputMetric, Scenario, StorageOrder, Vec3f, Vec3i, checked_scalar,
};
use crate::oracle::command::{tool_commit, tool_tree_state, validate_output_path};
use crate::profile::{
    DT, GRAVITY_MAGNITUDE, MAXIMUM_NEIGHBORS_PER_BOUNDARY_ROW, MAXIMUM_NEIGHBORS_PER_FLUID_ROW,
    MAXIMUM_SAMPLES, MAXIMUM_STEPS, SUCCESSOR_MAXIMUM_STATIC_BOUNDARY_SAMPLES, UNIFORM_MASS,
    quantize_ppb,
};
use crate::reference::{self, CurveComparison};
use crate::{profile, scenario, solver};

const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";
const SCENARIO_IDS: [&str; 7] = [
    "CW-HYDRO-001",
    "CW-FREEFALL-001",
    "CW-DAMBREAK-001",
    "CW-STILL-001",
    "CW-ORIFICE-001",
    "CW-SEALED-001",
    "CW-ORDER-001",
];

mod attestation;
mod closure;
mod command;
mod energy;
mod pressure_closure;
mod resource;
mod validation;

use command::{command_result, parse_arguments, validate_reference_paths, write_report};
use energy::{
    EnergyClass, EnergyStageEvidence, W1OutputMetric, checked_stage_energy_sum, energy_class,
    energy_contract_projection, initial_energy_stage_evidence, step_energy_deltas,
    update_energy_stage_evidence, w1_output_metric,
};
#[cfg(test)]
use energy::{EnergyPublication, publish_energy_metrics};
use validation::{
    capacity_thresholds, corpus_run_root, primary_order, reference_path, reference_required,
    serializable_root, validate_output, validate_step,
};

#[derive(Clone, Debug)]
struct Request {
    output: PathBuf,
    scenario_id: Option<String>,
    reference: Option<PathBuf>,
    hydro_reference: Option<PathBuf>,
    dam_break_reference: Option<PathBuf>,
    orifice_reference: Option<PathBuf>,
    solver_mode: W1SolverMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum W1SolverMode {
    FrozenSuccessor,
    FrozenObserveEnergyDiagnostic,
}

impl W1SolverMode {
    fn label(self) -> &'static str {
        match self {
            Self::FrozenSuccessor => {
                "frozen-w0h-accelerated-projected-gradient50-sequential-contact"
            }
            Self::FrozenObserveEnergyDiagnostic => "diagnostic-w0h-observe-energy",
        }
    }
}

#[derive(Serialize)]
struct Envelope<'a> {
    schema_version: u32,
    status: &'a str,
    command: &'a str,
    details: &'a W1Report,
}

#[derive(Serialize)]
struct W1Report {
    report_schema: String,
    classification: String,
    tool_commit: String,
    tool_tree_state: String,
    toolchain_target: String,
    build_profile: String,
    build_rustflags: String,
    solver_mode: &'static str,
    run_execution_profile_root: String,
    scope: ScopeReport,
    roots: RootReport,
    requested_scenario: Option<String>,
    capacity_thresholds: Vec<CapacityThreshold>,
    scenarios: Vec<ScenarioEvidence>,
    corpus_run_root: Option<String>,
    external_references_complete: bool,
    product_check: String,
    disposition: String,
    failure: Option<FailureReport>,
    timing: TimingReport,
}

#[derive(Serialize)]
struct ScopeReport {
    current_target: String,
    current_gate: String,
    windows: String,
    promotion_cross_target_gate: String,
}

#[derive(Serialize)]
struct RootReport {
    w0f_document: String,
    w0f_float_profile: String,
    w0f_execution_manifest: String,
    w0f_corpus: String,
    w0f_fixtures: String,
    w0f_geometry: String,
    w0f_execution_profile: String,
    w0g_document: String,
    w0g_energy_contract: String,
    w0g_corpus: String,
    w0g_execution_profile: String,
    w0h_document: String,
    w0h_solver_profile: String,
    w0h_corpus: String,
    w0h_execution_profile: String,
    w1_reference_attestation: String,
}

#[derive(Serialize)]
struct CapacityThreshold {
    resource: String,
    below: String,
    equal: String,
    above: String,
}

#[derive(Serialize)]
struct ScenarioEvidence {
    id: String,
    kind: String,
    scenario_root: String,
    sample_count: usize,
    static_boundary_sample_count: usize,
    planned_steps: u32,
    output_every: u32,
    status: String,
    primary: Option<TrajectoryEvidence>,
    repeat_roots: Vec<NamedRoot>,
    permutation_roots: Vec<NamedRoot>,
    checks: Vec<CheckReport>,
    reference: ReferenceEvidence,
    wall_clock_nanoseconds: u64,
}

#[derive(Serialize)]
struct TrajectoryEvidence {
    storage_order: String,
    completed_steps: u32,
    last_frame_root: String,
    trajectory_root: Option<String>,
    output_metric_count: usize,
    output_metrics_root: Option<String>,
    initial_centre_of_mass_um: Vec3i,
    final_centre_of_mass_um: Vec3i,
    maximum_density_iterations: u8,
    maximum_density_error_ppb: i64,
    maximum_density_kkt_error_ppb: Option<i64>,
    maximum_divergence_iterations: u8,
    maximum_divergence_error_ppb: i64,
    maximum_penetration_um: i64,
    maximum_energy_residual_ppb: i64,
    maximum_energy_excess_ppb: i64,
    maximum_energy_deficit_ppb: i64,
    first_absolute_energy_drift_breach_step: Option<u32>,
    maximum_stage_energy_closure_ppb: i64,
    maximum_momentum_residual_ppb: i64,
    maximum_contact_constraints: usize,
    maximum_contact_components: usize,
    cumulative_stage_energy_deltas: Vec<EnergyStageEvidence>,
    cumulative_contact_energy_delta_bits: String,
    cumulative_contact_energy_loss_ppb: i64,
    contact_feature_sum_difference_ppb: i64,
}

#[derive(Serialize)]
struct NamedRoot {
    name: String,
    trajectory_root: String,
}

#[derive(Serialize)]
struct ReferenceEvidence {
    required: bool,
    status: String,
    path: Option<String>,
    expected_sha256: Option<String>,
    sha256: Option<String>,
    attested: Option<bool>,
    comparisons: Vec<CurveComparison>,
}

#[derive(Serialize)]
struct FailureReport {
    scenario_id: Option<String>,
    code: String,
    detail: String,
}

#[derive(Serialize)]
struct TimingReport {
    classification: String,
    wall_clock_nanoseconds: u64,
}

struct TrajectoryArtifacts {
    trajectory_root: [u8; 32],
    output_metrics: Vec<OutputMetric>,
    freefall_mismatch: Option<String>,
}

pub(crate) fn run_xtask(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    let request = parse_arguments(arguments)?;
    validate_output_path(repository_root, &request.output)?;
    validate_reference_paths(repository_root, &request)?;
    let roots = AcceleratedPressureRoots::verify(repository_root)?;
    let mut report = new_report(repository_root, &request, &roots);
    let started = Instant::now();
    let execution = execute(repository_root, &request, &roots, &mut report);
    report.timing.wall_clock_nanoseconds =
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let ending_tree_state = tool_tree_state(repository_root);
    if ending_tree_state != report.tool_tree_state {
        report.tool_tree_state = format!("{}->{ending_tree_state}", report.tool_tree_state);
    }
    if let Err(error) = &execution {
        if report.failure.is_none() {
            report.failure = Some(FailureReport {
                scenario_id: None,
                code: error.code().to_owned(),
                detail: error.detail().to_owned(),
            });
        }
        report.product_check = "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned();
        report.disposition = if request.solver_mode == W1SolverMode::FrozenSuccessor {
            "LINUX_W1_FAILED".to_owned()
        } else {
            "ENERGY_OBSERVATION_DIAGNOSTIC_FAILED / NO_W1_CREDIT".to_owned()
        };
    }
    write_report(&request.output, &report, execution.is_ok())?;
    match execution {
        Ok(()) => command_result(&request, &report),
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

pub(crate) fn run_closure_xtask(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    closure::run(repository_root, arguments)
}

pub(crate) fn run_pressure_closure_xtask(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    pressure_closure::run(repository_root, arguments)
}

pub(crate) fn run_resource_profile_xtask(
    repository_root: &Path,
    arguments: impl Iterator<Item = String>,
) -> Result<String, WaterError> {
    resource::run(repository_root, arguments)
}

fn new_report(
    repository_root: &Path,
    request: &Request,
    roots: &AcceleratedPressureRoots,
) -> W1Report {
    let run_execution_profile_root = effective_execution_profile_root(request.solver_mode, roots);
    W1Report {
        report_schema: "nextengine.continuum-water.w1-linux-serial.v4".to_owned(),
        classification: "W1_LINUX_SERIAL_RESEARCH_ONLY".to_owned(),
        tool_commit: tool_commit(repository_root),
        tool_tree_state: tool_tree_state(repository_root),
        toolchain_target: env!("WATER_BUILD_TARGET").to_owned(),
        build_profile: env!("WATER_BUILD_PROFILE").to_owned(),
        build_rustflags: env!("WATER_BUILD_RUSTFLAGS").replace('\u{1f}', " "),
        solver_mode: request.solver_mode.label(),
        run_execution_profile_root: hash::hex(&run_execution_profile_root),
        scope: ScopeReport {
            current_target: LINUX_TARGET.to_owned(),
            current_gate: "SAME_TARGET_SERIAL_CORRECTNESS".to_owned(),
            windows: "OUT_OF_SCOPE_FOR_CURRENT_W1_BY_USER".to_owned(),
            promotion_cross_target_gate: "DEFERRED_NOT_WAIVED".to_owned(),
        },
        roots: RootReport {
            w0f_document: hash::hex(&roots.parent.parent.document),
            w0f_float_profile: hash::hex(&roots.parent.parent.float_profile),
            w0f_execution_manifest: hash::hex(&roots.parent.parent.execution_manifest),
            w0f_corpus: hash::hex(&roots.parent.parent.corpus),
            w0f_fixtures: hash::hex(&roots.parent.parent.fixtures),
            w0f_geometry: hash::hex(&roots.parent.parent.geometry),
            w0f_execution_profile: hash::hex(&roots.parent.parent.execution_profile),
            w0g_document: hash::hex(&roots.parent.document),
            w0g_energy_contract: hash::hex(&roots.parent.contract),
            w0g_corpus: hash::hex(&roots.parent.corpus),
            w0g_execution_profile: hash::hex(&roots.parent.execution_profile),
            w0h_document: hash::hex(&roots.document),
            w0h_solver_profile: hash::hex(&roots.solver_profile),
            w0h_corpus: hash::hex(&roots.corpus),
            w0h_execution_profile: hash::hex(&roots.execution_profile),
            w1_reference_attestation: hash::hex(&reference::attestation_profile_root()),
        },
        requested_scenario: request.scenario_id.clone(),
        capacity_thresholds: capacity_thresholds(),
        scenarios: Vec::new(),
        corpus_run_root: None,
        external_references_complete: false,
        product_check: "CONTINUUM-WATER-REF-P1=NOT_RUN".to_owned(),
        disposition: "RUNNING".to_owned(),
        failure: None,
        timing: TimingReport {
            classification: "DIAGNOSTIC_ONLY".to_owned(),
            wall_clock_nanoseconds: 0,
        },
    }
}

fn effective_execution_profile_root(
    mode: W1SolverMode,
    roots: &AcceleratedPressureRoots,
) -> [u8; 32] {
    if mode == W1SolverMode::FrozenSuccessor {
        return roots.execution_profile;
    }
    let mut digest = Sha256::new();
    digest.update(b"nextengine.continuum-water.w1-energy-observation-diagnostic.v2\0");
    digest.update(roots.execution_profile);
    digest.update(mode.label().as_bytes());
    digest.finalize().into()
}

fn execute(
    repository_root: &Path,
    request: &Request,
    roots: &AcceleratedPressureRoots,
    report: &mut W1Report,
) -> Result<(), WaterError> {
    profile::validate_execution_profile()?;
    profile::validate_float_environment()?;
    if env!("WATER_BUILD_TARGET") != LINUX_TARGET {
        return Err(WaterError::new(
            SCENARIO_INVALID,
            format!(
                "W1 Linux scope requires target {LINUX_TARGET}, compiled for {}",
                env!("WATER_BUILD_TARGET")
            ),
        ));
    }
    let selected_ids: Vec<&str> = match request.scenario_id.as_deref() {
        Some(id) => vec![id],
        None => SCENARIO_IDS.to_vec(),
    };
    report
        .scenarios
        .try_reserve_exact(selected_ids.len())
        .map_err(report_reserve_error)?;
    for scenario_id in selected_ids {
        let selected = scenario::find(scenario_id)?;
        let source_projection = roots.parent.parent.scenario_projection(scenario_id)?;
        let implementation_projection = scenario::successor_projection(&selected)?;
        if source_projection != implementation_projection.as_bytes() {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("scenario {scenario_id} differs from its successor projection"),
            ));
        }
        let energy_source_projection = roots.parent.scenario_projection(scenario_id)?;
        let energy_implementation_projection = energy_contract_projection(scenario_id)?;
        if energy_source_projection != energy_implementation_projection.as_bytes() {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("scenario {scenario_id} differs from its W0G energy projection"),
            ));
        }
        let reference_path = reference_path(request, scenario_id);
        let mut evidence = scenario_evidence(&selected, roots, reference_path)?;
        let started = Instant::now();
        let result = execute_scenario(
            &selected,
            roots,
            request.solver_mode,
            reference_path,
            &mut evidence,
        );
        evidence.wall_clock_nanoseconds =
            u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        if let Err(error) = result {
            evidence.status = "FAILED".to_owned();
            report.failure = Some(FailureReport {
                scenario_id: Some(scenario_id.to_owned()),
                code: error.code().to_owned(),
                detail: error.detail().to_owned(),
            });
            report.scenarios.push(evidence);
            return Err(error);
        }
        report.scenarios.push(evidence);
    }
    report.corpus_run_root = Some(corpus_run_root(roots, &report.scenarios));
    report.external_references_complete = report
        .scenarios
        .iter()
        .all(|evidence| !evidence.reference.required || evidence.reference.status == "PASS");
    let full_corpus = request.scenario_id.is_none() && report.scenarios.len() == SCENARIO_IDS.len();
    if request.solver_mode != W1SolverMode::FrozenSuccessor {
        report.disposition = "ENERGY_OBSERVATION_DIAGNOSTIC_PASS / NO_W1_CREDIT".to_owned();
    } else if full_corpus
        && report.external_references_complete
        && report.tool_tree_state == "CLEAN"
        && tool_tree_state(repository_root) == "CLEAN"
    {
        report.product_check = "CONTINUUM-WATER-REF-P1=PASS".to_owned();
        report.disposition = "LINUX_W1_PASS".to_owned();
    } else if full_corpus && !report.external_references_complete {
        report.disposition = "LINUX_INTERNAL_CORPUS_PASS / EXTERNAL_REFERENCE_PENDING".to_owned();
    } else if full_corpus {
        report.disposition = "LINUX_INTERNAL_CORPUS_PASS / CLEAN_EVIDENCE_PENDING".to_owned();
    } else {
        report.disposition = "LINUX_SCENARIO_PASS / NO_FULL_CORPUS_CREDIT".to_owned();
    }
    Ok(())
}

fn scenario_evidence(
    selected: &Scenario,
    roots: &AcceleratedPressureRoots,
    reference_path: Option<&Path>,
) -> Result<ScenarioEvidence, WaterError> {
    let samples = scenario::initial_samples(selected, primary_order(selected))?;
    let boundary = build_density_support(selected.geometry)?;
    Ok(ScenarioEvidence {
        id: selected.id.to_owned(),
        kind: selected.kind.to_owned(),
        scenario_root: hash::hex(&roots.scenario_root(selected.id)?),
        sample_count: samples.len(),
        static_boundary_sample_count: boundary.len(),
        planned_steps: selected.steps,
        output_every: selected.output_every,
        status: "RUNNING".to_owned(),
        primary: None,
        repeat_roots: Vec::new(),
        permutation_roots: Vec::new(),
        checks: Vec::new(),
        reference: ReferenceEvidence {
            required: reference_required(selected.id),
            status: if reference_path.is_some() {
                "PENDING".to_owned()
            } else {
                "NOT_PROVIDED".to_owned()
            },
            path: reference_path.map(|path| path.display().to_string()),
            expected_sha256: reference::expected_sha256(selected.id).map(str::to_owned),
            sha256: None,
            attested: None,
            comparisons: Vec::new(),
        },
        wall_clock_nanoseconds: 0,
    })
}

fn execute_scenario(
    selected: &Scenario,
    roots: &AcceleratedPressureRoots,
    solver_mode: W1SolverMode,
    reference_path: Option<&Path>,
    evidence: &mut ScenarioEvidence,
) -> Result<(), WaterError> {
    let boundary = build_density_support(selected.geometry)?;
    let scenario_root = roots.scenario_root(selected.id)?;
    let imported_reference = if let Some(path) = reference_path {
        let imported = reference::load(path, selected, &scenario_root, evidence.sample_count)?;
        evidence.reference.sha256 = Some(imported.sha256.clone());
        let attested = match attestation::validate(
            solver_mode,
            evidence.reference.required,
            selected.id,
            &imported.sha256,
        ) {
            Ok(attested) => attested,
            Err(error) => {
                evidence.reference.status = "FAILED_ATTESTATION".to_owned();
                evidence.reference.attested = Some(false);
                return Err(error);
            }
        };
        evidence.reference.attested = Some(attested);
        Some(imported)
    } else {
        None
    };
    let execution_profile_root = effective_execution_profile_root(solver_mode, roots);
    let order = primary_order(selected);
    let primary = run_trajectory(
        selected,
        scenario::initial_samples(selected, order)?,
        order,
        &boundary,
        &execution_profile_root,
        &scenario_root,
        solver_mode,
        &mut evidence.primary,
    )?;
    let primary_root = primary.trajectory_root;
    let mut checks = Vec::new();
    validate_scenario_metrics(
        selected,
        &primary.output_metrics,
        primary.freefall_mismatch.as_deref(),
        &mut checks,
    )?;
    for check in &mut checks {
        if check.check == "common-count-mass-solver-work-bounds" {
            check.check = "common-count-mass-solver-momentum-bounds".to_owned();
            check.detail = "frozen non-energy common rules satisfied".to_owned();
        }
    }
    let (check, detail) = match energy_class(selected.id)? {
        EnergyClass::ReversibleEquilibrium => (
            "w0g-reversible-absolute-energy-drift",
            "absolute mechanical-energy drift remained at or below 10000000 ppb",
        ),
        EnergyClass::StaticImpactDissipative => (
            "w0g-static-impact-energy-excess",
            "positive mechanical-energy excess remained at or below 10000000 ppb; deficit is diagnostic",
        ),
    };
    checks.push(CheckReport {
        check: check.to_owned(),
        status: "PASS".to_owned(),
        detail: detail.to_owned(),
    });
    evidence.checks = checks;
    evidence.repeat_roots.push(NamedRoot {
        name: "run-1".to_owned(),
        trajectory_root: hash::hex(&primary_root),
    });
    if matches!(selected.id, "CW-STILL-001" | "CW-SEALED-001") {
        let mut repeated = None;
        let repeat = run_trajectory(
            selected,
            scenario::initial_samples(selected, order)?,
            order,
            &boundary,
            &execution_profile_root,
            &scenario_root,
            solver_mode,
            &mut repeated,
        )?;
        evidence.repeat_roots.push(NamedRoot {
            name: "run-2".to_owned(),
            trajectory_root: hash::hex(&repeat.trajectory_root),
        });
        if repeat.trajectory_root != primary_root {
            return Err(WaterError::new(
                NONDETERMINISTIC_RESULT,
                format!("{} repeat trajectory root differs", selected.id),
            ));
        }
    }
    if selected.id == "CW-ORDER-001" {
        evidence.permutation_roots.push(NamedRoot {
            name: order.label().to_owned(),
            trajectory_root: hash::hex(&primary_root),
        });
        for permutation in [StorageOrder::Reverse, StorageOrder::Affine] {
            let mut permuted = None;
            let run = run_trajectory(
                selected,
                scenario::initial_samples(selected, permutation)?,
                permutation,
                &boundary,
                &execution_profile_root,
                &scenario_root,
                solver_mode,
                &mut permuted,
            )?;
            evidence.permutation_roots.push(NamedRoot {
                name: permutation.label().to_owned(),
                trajectory_root: hash::hex(&run.trajectory_root),
            });
            if run.trajectory_root != primary_root {
                return Err(WaterError::new(
                    NONDETERMINISTIC_RESULT,
                    format!(
                        "{} storage order changed the trajectory root",
                        permutation.label()
                    ),
                ));
            }
        }
    }
    if let Some(imported) = imported_reference {
        let comparisons = reference::compare_curves(selected, &primary.output_metrics, &imported)?;
        if let Some(failed_metric) = comparisons
            .iter()
            .find(|comparison| !comparison.passed)
            .map(|comparison| comparison.metric)
        {
            evidence.reference.status = "FAILED".to_owned();
            evidence.reference.comparisons = comparisons;
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!(
                    "{} reference curve {} exceeds its frozen threshold",
                    selected.id, failed_metric
                ),
            ));
        }
        evidence.reference.status = if evidence.reference.attested == Some(true) {
            "PASS".to_owned()
        } else {
            "PASS_UNATTESTED_RESEARCH_ONLY".to_owned()
        };
        evidence.reference.comparisons = comparisons;
    }
    evidence.status = if evidence.reference.required && evidence.reference.status != "PASS" {
        "INTERNAL_PASS_REFERENCE_PENDING".to_owned()
    } else {
        "PASS".to_owned()
    };
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_trajectory(
    selected: &Scenario,
    samples: Vec<CanonicalSample>,
    order: StorageOrder,
    boundary: &[BoundarySample],
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    solver_mode: W1SolverMode,
    evidence_slot: &mut Option<TrajectoryEvidence>,
) -> Result<TrajectoryArtifacts, WaterError> {
    let (mut frame, initial_summary) = solver::initial_frame(
        samples,
        selected.geometry,
        boundary,
        execution_profile_root,
        scenario_root,
    )?;
    let baseline = physical_totals(&frame)?;
    let mut trajectory = TrajectoryHasher::new(
        selected
            .steps
            .checked_add(1)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "trajectory frame count overflow"))?,
    );
    trajectory.push(0, &frame.frame_root)?;
    let mut gravity_impulse = Vec3f::ZERO;
    let mut external_fluid_impulse = Vec3f::ZERO;
    let mut contact_impulse = Vec3f::ZERO;
    let mut contact_energy_delta = 0.0_f64;
    let mut stage_energy_deltas = [0.0_f64; 6];
    let mut feature_impulses = [Vec3f::ZERO; 25];
    let mut outputs = Vec::new();
    outputs
        .try_reserve_exact(reference::output_steps(selected)?.len())
        .map_err(report_reserve_error)?;
    outputs.push(w1_output_metric(
        selected,
        &frame,
        initial_summary.centre_of_mass_um,
        baseline,
        gravity_impulse,
        external_fluid_impulse,
    )?);
    *evidence_slot = Some(TrajectoryEvidence {
        storage_order: order.label().to_owned(),
        completed_steps: 0,
        last_frame_root: hash::hex(&frame.frame_root),
        trajectory_root: None,
        output_metric_count: 1,
        output_metrics_root: None,
        initial_centre_of_mass_um: initial_summary.centre_of_mass_um,
        final_centre_of_mass_um: initial_summary.centre_of_mass_um,
        maximum_density_iterations: 0,
        maximum_density_error_ppb: 0,
        maximum_density_kkt_error_ppb: None,
        maximum_divergence_iterations: 0,
        maximum_divergence_error_ppb: 0,
        maximum_penetration_um: 0,
        maximum_energy_residual_ppb: 0,
        maximum_energy_excess_ppb: 0,
        maximum_energy_deficit_ppb: 0,
        first_absolute_energy_drift_breach_step: None,
        maximum_stage_energy_closure_ppb: 0,
        maximum_momentum_residual_ppb: 0,
        maximum_contact_constraints: 0,
        maximum_contact_components: 0,
        cumulative_stage_energy_deltas: initial_energy_stage_evidence(),
        cumulative_contact_energy_delta_bits: "0x0000000000000000".to_owned(),
        cumulative_contact_energy_loss_ppb: 0,
        contact_feature_sum_difference_ppb: 0,
    });
    let mut expected_freefall = if selected.id == "CW-FREEFALL-001" {
        Some(frame.samples.clone())
    } else {
        None
    };
    let mut freefall_mismatch = None;
    for step in 1..=selected.steps {
        let next = match solver_mode {
            W1SolverMode::FrozenSuccessor | W1SolverMode::FrozenObserveEnergyDiagnostic => {
                solver::successor_accelerated_projected_gradient_substep(
                    &frame,
                    selected.geometry,
                    boundary,
                    execution_profile_root,
                    scenario_root,
                )
            }
        }
        .map_err(|error| {
            WaterError::new(
                error.code(),
                format!("{} step {step}: {}", selected.id, error.detail()),
            )
        })?;
        validate_step(selected, &next)?;
        for _sample in &next.outcome.frame.samples {
            let mass_dt = checked_scalar(UNIFORM_MASS * DT, "W1 gravity impulse mass dt")?;
            let gravity_y = checked_scalar(mass_dt * -GRAVITY_MAGNITUDE, "W1 gravity impulse y")?;
            gravity_impulse = gravity_impulse
                .add(Vec3f::new(0.0, gravity_y, 0.0))
                .checked("W1 gravity impulse reduction")?;
        }
        for impulse in next.outcome.boundary_impulses {
            external_fluid_impulse = external_fluid_impulse
                .add(impulse)
                .checked("W1 pressure impulse reduction")?;
        }
        external_fluid_impulse = external_fluid_impulse
            .add(next.projection.fluid_impulse)
            .checked("W1 contact impulse reduction")?;
        contact_impulse = contact_impulse
            .add(next.projection.fluid_impulse)
            .checked("W1 contact-only impulse reduction")?;
        contact_energy_delta = checked_scalar(
            contact_energy_delta + next.projection.kinetic_energy_delta,
            "W1 contact energy reduction",
        )?;
        for (total, delta) in stage_energy_deltas
            .iter_mut()
            .zip(step_energy_deltas(next.energy))
        {
            *total = checked_scalar(*total + delta, "W1 stage energy reduction")?;
        }
        for (feature_id, impulse) in next.projection.feature_fluid_impulses.iter().enumerate() {
            feature_impulses[feature_id] = feature_impulses[feature_id]
                .add(*impulse)
                .checked("W1 feature impulse reduction")?;
        }
        if let Some(expected) = &mut expected_freefall {
            advance_freefall(expected)?;
            if freefall_mismatch.is_none() {
                freefall_mismatch = compare_freefall(
                    selected.id,
                    step,
                    expected,
                    &next.outcome.frame,
                    &next.outcome.summary,
                );
            }
        }
        trajectory.push(step, &next.outcome.frame.frame_root)?;
        let metric = if step % selected.output_every == 0 {
            Some(w1_output_metric(
                selected,
                &next.outcome.frame,
                next.outcome.summary.centre_of_mass_um,
                baseline,
                gravity_impulse,
                external_fluid_impulse,
            )?)
        } else {
            None
        };
        let evidence = evidence_slot.as_mut().ok_or_else(|| {
            WaterError::new(INVARIANT_MISMATCH, "W1 trajectory evidence is missing")
        })?;
        let baseline_energy_denominator = checked_scalar(
            (baseline.kinetic.abs() + baseline.potential.abs()).max(1.0),
            "W1 baseline energy denominator",
        )?;
        let contact_energy_loss = (-contact_energy_delta).max(0.0);
        evidence.cumulative_contact_energy_delta_bits =
            format!("0x{:016x}", contact_energy_delta.to_bits());
        evidence.cumulative_contact_energy_loss_ppb = quantize_ppb(checked_scalar(
            contact_energy_loss / baseline_energy_denominator,
            "W1 normalized contact energy loss",
        )?)?;
        update_energy_stage_evidence(
            &mut evidence.cumulative_stage_energy_deltas,
            stage_energy_deltas,
            baseline_energy_denominator,
        )?;
        let stage_balance_ppb = quantize_ppb(checked_scalar(
            checked_stage_energy_sum(&stage_energy_deltas)? / baseline_energy_denominator,
            "W1 normalized stage energy balance",
        )?)?;
        update_trajectory_evidence(evidence, &next, metric.as_ref(), stage_balance_ppb)?;
        if let Some(metric) = metric {
            validate_output(selected, solver_mode, &metric)?;
            outputs.push(metric);
        }
        frame = next.outcome.frame;
    }
    let trajectory_root = trajectory.finish()?;
    let outputs_root = serializable_root(
        b"nextengine.continuum-water.w1-output-metrics.v1\0",
        &outputs,
    )?;
    let feature_total = feature_impulses
        .iter()
        .try_fold(Vec3f::ZERO, |total, impulse| {
            total
                .add(*impulse)
                .checked("W1 feature impulse final reduction")
        })?;
    let feature_difference = contact_impulse
        .sub(feature_total)
        .checked("W1 contact feature difference")?;
    let denominator = vector_norm(contact_impulse)?.max(1.0);
    let difference_ppb = quantize_ppb(checked_scalar(
        vector_norm(feature_difference)? / denominator,
        "W1 feature impulse normalized difference",
    )?)?;
    if difference_ppb > 10_000_000 {
        return Err(WaterError::new(
            INVARIANT_MISMATCH,
            format!(
                "{} contact feature impulse difference {difference_ppb} ppb exceeds 10000000",
                selected.id
            ),
        ));
    }
    let evidence = evidence_slot.as_mut().ok_or_else(|| {
        WaterError::new(
            INVARIANT_MISMATCH,
            "W1 final trajectory evidence is missing",
        )
    })?;
    evidence.trajectory_root = Some(hash::hex(&trajectory_root));
    evidence.output_metric_count = outputs.len();
    evidence.output_metrics_root = Some(outputs_root);
    evidence.contact_feature_sum_difference_ppb = difference_ppb;
    Ok(TrajectoryArtifacts {
        trajectory_root,
        output_metrics: outputs.into_iter().map(|metric| metric.common).collect(),
        freefall_mismatch,
    })
}

fn update_trajectory_evidence(
    evidence: &mut TrajectoryEvidence,
    next: &solver::ContactConstrainedStepOutcome,
    metric: Option<&W1OutputMetric>,
    stage_balance_ppb: i64,
) -> Result<(), WaterError> {
    let summary = &next.outcome.summary;
    evidence.completed_steps = summary.step;
    evidence.last_frame_root = summary.frame_root.clone();
    evidence.final_centre_of_mass_um = summary.centre_of_mass_um;
    evidence.maximum_density_iterations = evidence
        .maximum_density_iterations
        .max(summary.density_iterations);
    evidence.maximum_density_error_ppb = evidence
        .maximum_density_error_ppb
        .max(summary.density_error_ppb);
    if let Some(error_ppb) = summary.density_kkt_error_ppb {
        evidence.maximum_density_kkt_error_ppb = Some(
            evidence
                .maximum_density_kkt_error_ppb
                .unwrap_or(0)
                .max(error_ppb),
        );
    }
    evidence.maximum_divergence_iterations = evidence
        .maximum_divergence_iterations
        .max(summary.divergence_iterations);
    evidence.maximum_divergence_error_ppb = evidence
        .maximum_divergence_error_ppb
        .max(summary.divergence_error_ppb);
    evidence.maximum_penetration_um = evidence
        .maximum_penetration_um
        .max(summary.maximum_penetration_um);
    let constraints = next
        .projection
        .feature_active_constraints
        .iter()
        .try_fold(0_usize, |total, count| {
            total.checked_add(usize::try_from(*count).unwrap_or(usize::MAX))
        })
        .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "W1 contact count overflow"))?;
    evidence.maximum_contact_constraints = evidence.maximum_contact_constraints.max(constraints);
    evidence.maximum_contact_components = evidence
        .maximum_contact_components
        .max(next.projection.active_components);
    if let Some(metric) = metric {
        evidence.output_metric_count = evidence
            .output_metric_count
            .checked_add(1)
            .ok_or_else(|| WaterError::new(NUMERIC_OVERFLOW, "W1 output metric count overflow"))?;
        evidence.maximum_energy_residual_ppb = evidence
            .maximum_energy_residual_ppb
            .max(metric.common.energy_residual_ppb);
        evidence.maximum_energy_excess_ppb = evidence
            .maximum_energy_excess_ppb
            .max(metric.energy_excess_ppb);
        evidence.maximum_energy_deficit_ppb = evidence
            .maximum_energy_deficit_ppb
            .max(metric.energy_deficit_ppb);
        if metric.common.energy_residual_ppb > 10_000_000
            && evidence.first_absolute_energy_drift_breach_step.is_none()
        {
            evidence.first_absolute_energy_drift_breach_step = Some(metric.common.step);
        }
        let closure = (stage_balance_ppb - metric.signed_energy_balance_ppb).unsigned_abs();
        let closure = i64::try_from(closure).map_err(|_| {
            WaterError::new(NUMERIC_OVERFLOW, "W1 stage energy closure does not fit i64")
        })?;
        evidence.maximum_stage_energy_closure_ppb =
            evidence.maximum_stage_energy_closure_ppb.max(closure);
        if closure > 2_000 {
            return Err(WaterError::new(
                INVARIANT_MISMATCH,
                format!(
                    "W1 stage energy closure {closure} ppb exceeds 2000 at step {}",
                    metric.common.step
                ),
            ));
        }
        evidence.maximum_momentum_residual_ppb = evidence
            .maximum_momentum_residual_ppb
            .max(metric.common.momentum_residual_ppb);
    }
    Ok(())
}

fn report_reserve_error(error: std::collections::TryReserveError) -> WaterError {
    WaterError::new(
        REPORT_CAPACITY_EXCEEDED,
        format!("W1 report allocation failed: {error}"),
    )
}

#[cfg(test)]
mod tests;
