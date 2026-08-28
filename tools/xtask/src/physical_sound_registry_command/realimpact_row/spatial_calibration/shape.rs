use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use self::manifest::{
    CALIBRATION_MANIFEST_SHA256, DEVELOPMENT_MANIFEST_SHA256, DEVELOPMENT_REPORT_SHA256, Manifest,
    SIGMA_GRID,
};
use self::model::{FittedModel, TrainingSample};
use super::dsp::{Evaluation, evaluate_rbf, improved_component_fraction};
use super::*;

mod acquire;
mod frequency;
mod manifest;
mod model;
mod publish;

const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-shape-spatial-development.report.v1";
const CALIBRATION_REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-shape-spatial-calibration.report.v1";

pub(in crate::physical_sound_registry_command) fn run_cli(
    root: &Path,
    mut arguments: impl Iterator<Item = String>,
) -> Result<(), String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected spatial-shape argument: {flag}")),
        }
    }
    let manifest = manifest.ok_or_else(|| {
        "physical-sound-registry spatial-shape requires --manifest <external-json>".to_owned()
    })?;
    let output = output.ok_or_else(|| {
        "physical-sound-registry spatial-shape requires --output <external-empty-directory>"
            .to_owned()
    })?;
    let header = requested_phase(root, &manifest)?;
    match (header.study_id.as_str(), header.phase.as_str()) {
        (manifest::STUDY_ID, "development") => run_development(root, &manifest, &output),
        (manifest::STUDY_ID, "calibration") => run_calibration(root, &manifest, &output),
        ("physical-sound-realimpact-frequency-conditioned-vertical", "development") => {
            run_frequency_development(root, &manifest, &output)
        }
        ("physical-sound-realimpact-frequency-conditioned-vertical", "calibration") => {
            run_frequency_calibration(root, &manifest, &output)
        }
        ("physical-sound-realimpact-complex-modal-radiation", "development") => {
            run_radiation_development(root, &manifest, &output)
        }
        (study, phase) => Err(format!(
            "unsupported spatial-shape study/phase: {study}/{phase}"
        )),
    }
}

fn requested_phase(root: &Path, manifest_argument: &Path) -> Result<PhaseHeader, String> {
    let path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial-shape manifest",
    )?;
    let bytes = read_bounded_file(&path, 16 * 1024 * 1024, "REALIMPACT spatial-shape manifest")?;
    let header: PhaseHeader = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse spatial-shape phase: {error}"))?;
    Ok(header)
}

fn run_frequency_development(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT frequency-spatial manifest",
    )?;
    let bytes = read_bounded_file(
        &path,
        16 * 1024 * 1024,
        "REALIMPACT frequency-spatial manifest",
    )?;
    frequency::run_development(root, &path, &bytes, output_argument)
}

fn run_frequency_calibration(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT frequency-spatial calibration manifest",
    )?;
    let bytes = read_bounded_file(
        &path,
        16 * 1024 * 1024,
        "REALIMPACT frequency-spatial calibration manifest",
    )?;
    frequency::run_calibration(root, &path, &bytes, output_argument)
}

fn run_radiation_development(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT modal-radiation development manifest",
    )?;
    let bytes = read_bounded_file(
        &path,
        16 * 1024 * 1024,
        "REALIMPACT modal-radiation development manifest",
    )?;
    frequency::run_radiation_development(root, &path, &bytes, output_argument)
}

fn read_ref(root: &Path, base: &Path, reference: &manifest::FileRef) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(
        root,
        &if reference.path.is_absolute() {
            reference.path.clone()
        } else {
            base.join(&reference.path)
        },
        "shape-spatial prerequisite",
    )?;
    let bytes = read_bounded_file(&path, 128 * 1024 * 1024, "shape-spatial prerequisite")?;
    require_hash(&bytes, &reference.sha256, "shape-spatial prerequisite")?;
    Ok(bytes)
}

fn run_development(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial-shape development manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        16 * 1024 * 1024,
        "REALIMPACT spatial-shape development manifest",
    )?;
    require_hash(
        &manifest_bytes,
        DEVELOPMENT_MANIFEST_SHA256,
        "spatial-shape development manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse spatial-shape development manifest: {error}"))?;
    manifest.validate_development()?;

    let mut work = Vec::new();
    for profile in manifest.profiles_for("development") {
        let acquisition = acquire::run(profile, manifest.compressed_audio_prefix_bytes)?;
        let control = evaluate_rbf(
            &acquisition.rows,
            &acquisition.modes,
            manifest.mode_extractor.sample_rate_hz,
            manifest.control.sigma_metres,
        )?;
        let mut grid = SIGMA_GRID
            .into_iter()
            .map(|sigma_metres| {
                let evaluation = evaluate_rbf(
                    &acquisition.rows,
                    &acquisition.modes,
                    manifest.mode_extractor.sample_rate_hz,
                    sigma_metres,
                )?;
                Ok(SigmaEvaluation {
                    sigma_metres,
                    gate: gate(&evaluation),
                    evaluation,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        grid.sort_by(|left, right| left.sigma_metres.total_cmp(&right.sigma_metres));
        let target_sigma_metres = grid
            .iter()
            .min_by(|left, right| {
                left.evaluation
                    .calibration_loss
                    .total_cmp(&right.evaluation.calibration_loss)
                    .then_with(|| left.sigma_metres.total_cmp(&right.sigma_metres))
            })
            .expect("frozen sigma grid is non-empty")
            .sigma_metres;
        work.push(DevelopmentWork {
            acquisition,
            control,
            grid,
            target_sigma_metres,
        });
    }
    let training = work
        .iter()
        .map(|object| TrainingSample {
            object_id: &object.acquisition.object_id,
            descriptor: &object.acquisition.descriptor,
            target_sigma_metres: object.target_sigma_metres,
        })
        .collect::<Vec<_>>();
    let model = model::fit(&training)?;
    let object_reports = work
        .iter()
        .map(|object| development_report(object, &model, &manifest))
        .collect::<Result<Vec<_>, String>>()?;
    let target_sigma_counts = SIGMA_GRID
        .into_iter()
        .map(|sigma| {
            (
                sigma,
                work.iter()
                    .filter(|object| object.target_sigma_metres == sigma)
                    .count(),
            )
        })
        .map(|(sigma_metres, object_count)| SigmaCount {
            sigma_metres,
            object_count,
        })
        .collect();
    let report = DevelopmentReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "MeshConditionedBandwidthDevelopmentFit",
        claim: "DEVELOPMENT_FIT_ONLY / CALIBRATION_AND_HOLDOUT_UNOPENED / NO_SHAPE_TRANSFER_ANGLE_DISTANCE_3D_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: manifest::STUDY_ID,
        revision: manifest::DEVELOPMENT_REVISION,
        manifest_sha256: DEVELOPMENT_MANIFEST_SHA256,
        objective: &manifest.objective,
        model: &model,
        target_sigma_counts,
        objects: object_reports,
        condition_gate: &manifest.condition_gate,
        calibration_rule: &manifest.calibration_rule,
        holdout_rule: &manifest.holdout_rule,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        next_action: "freeze this exact development report in a calibration manifest, then evaluate the fitted model once on 10_bowl and 36_SmallMeasuringCup; do not open holdout unless calibration passes",
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    let acquisitions = work
        .into_iter()
        .map(|object| object.acquisition)
        .collect::<Vec<_>>();
    publish::development(&output, &manifest_bytes, &acquisitions, &report_bytes)?;
    println!("REALIMPACT shape-spatial development: {}", output.display());
    println!("development manifest sha256: {DEVELOPMENT_MANIFEST_SHA256}");
    println!("development object count: {}", acquisitions.len());
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn run_calibration(
    root: &Path,
    manifest_argument: &Path,
    output_argument: &Path,
) -> Result<(), String> {
    let manifest_path = canonical_external_file(
        root,
        &resolve_cli_path(root, manifest_argument),
        "REALIMPACT spatial-shape calibration manifest",
    )?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        16 * 1024 * 1024,
        "REALIMPACT spatial-shape calibration manifest",
    )?;
    require_hash(
        &manifest_bytes,
        CALIBRATION_MANIFEST_SHA256,
        "spatial-shape calibration manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse spatial-shape calibration manifest: {error}"))?;
    manifest.validate_calibration()?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "spatial-shape calibration manifest has no parent".to_owned())?;
    let development_ref = manifest
        .development_report
        .as_ref()
        .ok_or_else(|| "spatial-shape calibration has no development report".to_owned())?;
    let development_bytes = read_ref(root, base, development_ref)?;
    require_hash(
        &development_bytes,
        DEVELOPMENT_REPORT_SHA256,
        "shape-spatial development report",
    )?;
    let development: DevelopmentPrerequisite = serde_json::from_slice(&development_bytes)
        .map_err(|error| format!("parse shape-spatial development report: {error}"))?;
    validate_development_prerequisite(&development)?;

    let mut work = Vec::new();
    for profile in manifest.profiles_for("calibration") {
        let acquisition = acquire::run(profile, manifest.compressed_audio_prefix_bytes)?;
        let control = evaluate_rbf(
            &acquisition.rows,
            &acquisition.modes,
            manifest.mode_extractor.sample_rate_hz,
            manifest.control.sigma_metres,
        )?;
        let predicted_sigma_metres = development.model.predict(&acquisition.descriptor)?;
        let candidate = evaluate_rbf(
            &acquisition.rows,
            &acquisition.modes,
            manifest.mode_extractor.sample_rate_hz,
            predicted_sigma_metres,
        )?;
        let candidate_to_control_median_error_ratio =
            ratio(candidate.median_abs_error_db, control.median_abs_error_db)?;
        let p90_regression_db = candidate.p90_abs_error_db - control.p90_abs_error_db;
        let improved_component_fraction_vs_control =
            improved_component_fraction(&candidate, &control)?;
        let candidate_gate = gate(&candidate);
        work.push(ComparisonWork {
            acquisition,
            predicted_sigma_metres,
            control,
            candidate,
            candidate_gate,
            candidate_to_control_median_error_ratio,
            p90_regression_db,
            improved_component_fraction_vs_control,
        });
    }
    let aggregate = comparison_aggregate(&work)?;
    let comparison_gate = comparison_gate(&aggregate, &manifest.calibration_rule);
    let calibration_gate_passed = comparison_gate.passed;
    let decision = if comparison_gate.passed {
        "MeshConditionedBandwidthCalibrationPassed"
    } else {
        "MeshConditionedBandwidthCalibrationRejected"
    };
    let next_action = if comparison_gate.passed {
        "freeze this exact calibration report in a holdout manifest, then evaluate the model once on the two untouched ceramic holdout objects"
    } else {
        "stop before holdout access; preserve both holdout objects and reject this bbox-conditioned bandwidth hypothesis"
    };
    let objects = work
        .iter()
        .map(|object| ComparisonObjectReport {
            object_id: &object.acquisition.object_id,
            descriptor: &object.acquisition.descriptor,
            predicted_sigma_metres: object.predicted_sigma_metres,
            candidate_to_control_median_error_ratio: object.candidate_to_control_median_error_ratio,
            p90_regression_db: object.p90_regression_db,
            improved_component_fraction_vs_control: object.improved_component_fraction_vs_control,
            control_gate: gate(&object.control),
            control: &object.control,
            candidate_gate: &object.candidate_gate,
            candidate: &object.candidate,
            acquisition: &object.acquisition.summary,
        })
        .collect();
    let report = CalibrationReport {
        schema: CALIBRATION_REPORT_SCHEMA,
        status: "Validated",
        decision,
        claim: "OBJECT_DISJOINT_CALIBRATION_ONLY / HOLDOUT_UNOPENED / NO_SHAPE_TRANSFER_ANGLE_DISTANCE_3D_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: manifest::STUDY_ID,
        revision: manifest::CALIBRATION_REVISION,
        manifest_sha256: CALIBRATION_MANIFEST_SHA256,
        development_report_sha256: DEVELOPMENT_REPORT_SHA256,
        model: &development.model,
        objects,
        aggregate: &aggregate,
        rule: &manifest.calibration_rule,
        gate: comparison_gate,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        next_action,
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    let acquisitions = work
        .into_iter()
        .map(|object| object.acquisition)
        .collect::<Vec<_>>();
    publish::calibration(
        &output,
        &manifest_bytes,
        &development_bytes,
        &acquisitions,
        &report_bytes,
    )?;
    println!("REALIMPACT shape-spatial calibration: {}", output.display());
    println!("calibration manifest sha256: {CALIBRATION_MANIFEST_SHA256}");
    println!("calibration object count: {}", acquisitions.len());
    println!("calibration gate passed: {calibration_gate_passed}");
    println!("report decision: {decision}");
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn validate_development_prerequisite(report: &DevelopmentPrerequisite) -> Result<(), String> {
    let object_ids = report
        .objects
        .iter()
        .map(|object| object.object_id.as_str())
        .collect::<Vec<_>>();
    if report.schema != REPORT_SCHEMA
        || report.status != "Validated"
        || report.decision != "MeshConditionedBandwidthDevelopmentFit"
        || report.manifest_sha256 != DEVELOPMENT_MANIFEST_SHA256
        || object_ids != manifest::DEVELOPMENT_OBJECTS
        || report
            .objects
            .iter()
            .any(|object| !SIGMA_GRID.contains(&object.target_sigma_metres))
    {
        return Err("shape-spatial development report does not authorize calibration".to_owned());
    }
    report.model.predict(&report.objects[0].descriptor)?;
    Ok(())
}

fn comparison_aggregate(work: &[ComparisonWork]) -> Result<ComparisonAggregate, String> {
    if work.is_empty() {
        return Err("shape-spatial comparison has no objects".to_owned());
    }
    let mut ratios = work
        .iter()
        .map(|object| object.candidate_to_control_median_error_ratio)
        .collect::<Vec<_>>();
    ratios.sort_by(f64::total_cmp);
    let median_ratio = if ratios.len().is_multiple_of(2) {
        (ratios[ratios.len() / 2 - 1] + ratios[ratios.len() / 2]) * 0.5
    } else {
        ratios[ratios.len() / 2]
    };
    Ok(ComparisonAggregate {
        object_count: work.len(),
        every_shape_candidate_condition_gate_passed: work
            .iter()
            .all(|object| object.candidate_gate.passed),
        median_object_candidate_to_control_median_error_ratio: median_ratio,
        maximum_object_candidate_to_control_median_error_ratio: work
            .iter()
            .map(|object| object.candidate_to_control_median_error_ratio)
            .max_by(f64::total_cmp)
            .expect("non-empty comparison"),
        maximum_object_p90_regression_db: work
            .iter()
            .map(|object| object.p90_regression_db)
            .max_by(f64::total_cmp)
            .expect("non-empty comparison"),
        aggregate_improved_component_fraction_vs_control: work
            .iter()
            .map(|object| object.improved_component_fraction_vs_control)
            .sum::<f64>()
            / work.len() as f64,
    })
}

fn comparison_gate(aggregate: &ComparisonAggregate, rule: &manifest::ComparisonRule) -> Gate {
    let checks = vec![
        GateCheck {
            metric: "every_shape_candidate_condition_gate_passed",
            observed: if aggregate.every_shape_candidate_condition_gate_passed {
                1.0
            } else {
                0.0
            },
            relation: "==",
            threshold: if rule.require_every_shape_candidate_condition_gate {
                1.0
            } else {
                0.0
            },
            passed: aggregate.every_shape_candidate_condition_gate_passed
                == rule.require_every_shape_candidate_condition_gate,
        },
        check(
            "median_object_candidate_to_control_median_error_ratio",
            aggregate.median_object_candidate_to_control_median_error_ratio,
            "<=",
            rule.maximum_median_object_candidate_to_control_median_error_ratio,
        ),
        check(
            "maximum_object_candidate_to_control_median_error_ratio",
            aggregate.maximum_object_candidate_to_control_median_error_ratio,
            "<=",
            rule.maximum_each_object_candidate_to_control_median_error_ratio,
        ),
        check(
            "maximum_object_p90_regression_db",
            aggregate.maximum_object_p90_regression_db,
            "<=",
            rule.maximum_each_object_p90_regression_db,
        ),
        check(
            "aggregate_improved_component_fraction_vs_control",
            aggregate.aggregate_improved_component_fraction_vs_control,
            ">=",
            rule.minimum_aggregate_improved_component_fraction_vs_control,
        ),
    ];
    Gate {
        passed: checks.iter().all(|check| check.passed),
        checks,
    }
}

fn development_report<'a>(
    object: &'a DevelopmentWork,
    model: &FittedModel,
    manifest: &Manifest,
) -> Result<DevelopmentObjectReport<'a>, String> {
    let predicted_sigma_metres = model.predict(&object.acquisition.descriptor)?;
    let predicted = evaluate_rbf(
        &object.acquisition.rows,
        &object.acquisition.modes,
        manifest.mode_extractor.sample_rate_hz,
        predicted_sigma_metres,
    )?;
    let candidate_to_control_median_error_ratio = ratio(
        predicted.median_abs_error_db,
        object.control.median_abs_error_db,
    )?;
    let improved_component_fraction_vs_control =
        improved_component_fraction(&predicted, &object.control)?;
    Ok(DevelopmentObjectReport {
        object_id: &object.acquisition.object_id,
        descriptor: &object.acquisition.descriptor,
        target_sigma_metres: object.target_sigma_metres,
        predicted_sigma_metres,
        candidate_to_control_median_error_ratio,
        improved_component_fraction_vs_control,
        control_gate: gate(&object.control),
        control: &object.control,
        sigma_grid: &object.grid,
        predicted_gate: gate(&predicted),
        predicted,
        acquisition: &object.acquisition.summary,
    })
}

fn ratio(candidate: f64, control: f64) -> Result<f64, String> {
    let ratio = if control <= 1.0e-12 {
        if candidate <= 1.0e-12 {
            1.0
        } else {
            f64::INFINITY
        }
    } else {
        candidate / control
    };
    ratio
        .is_finite()
        .then_some(ratio)
        .ok_or_else(|| "shape-spatial control ratio is non-finite".to_owned())
}

struct DevelopmentWork {
    acquisition: acquire::Acquisition,
    control: Evaluation,
    grid: Vec<SigmaEvaluation>,
    target_sigma_metres: f64,
}

#[derive(Deserialize)]
struct PhaseHeader {
    study_id: String,
    phase: String,
}

#[derive(Serialize)]
struct SigmaEvaluation {
    sigma_metres: f64,
    gate: Gate,
    evaluation: Evaluation,
}

#[derive(Serialize)]
struct SigmaCount {
    sigma_metres: f64,
    object_count: usize,
}

#[derive(Serialize)]
struct DevelopmentObjectReport<'a> {
    object_id: &'a str,
    descriptor: &'a manifest::MeshDescriptor,
    target_sigma_metres: f64,
    predicted_sigma_metres: f64,
    candidate_to_control_median_error_ratio: f64,
    improved_component_fraction_vs_control: f64,
    control_gate: Gate,
    control: &'a Evaluation,
    sigma_grid: &'a [SigmaEvaluation],
    predicted_gate: Gate,
    predicted: Evaluation,
    acquisition: &'a acquire::AcquisitionSummary,
}

#[derive(Serialize)]
struct DevelopmentReport<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    manifest_sha256: &'static str,
    objective: &'a str,
    model: &'a FittedModel,
    target_sigma_counts: Vec<SigmaCount>,
    objects: Vec<DevelopmentObjectReport<'a>>,
    condition_gate: &'a manifest::ConditionGate,
    calibration_rule: &'a manifest::ComparisonRule,
    holdout_rule: &'a manifest::ComparisonRule,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
}

struct ComparisonWork {
    acquisition: acquire::Acquisition,
    predicted_sigma_metres: f64,
    control: Evaluation,
    candidate: Evaluation,
    candidate_gate: Gate,
    candidate_to_control_median_error_ratio: f64,
    p90_regression_db: f64,
    improved_component_fraction_vs_control: f64,
}

#[derive(Deserialize)]
struct DevelopmentPrerequisite {
    schema: String,
    status: String,
    decision: String,
    manifest_sha256: String,
    model: FittedModel,
    objects: Vec<DevelopmentObjectPrerequisite>,
}

#[derive(Deserialize)]
struct DevelopmentObjectPrerequisite {
    object_id: String,
    descriptor: manifest::MeshDescriptor,
    target_sigma_metres: f64,
}

#[derive(Serialize)]
struct ComparisonObjectReport<'a> {
    object_id: &'a str,
    descriptor: &'a manifest::MeshDescriptor,
    predicted_sigma_metres: f64,
    candidate_to_control_median_error_ratio: f64,
    p90_regression_db: f64,
    improved_component_fraction_vs_control: f64,
    control_gate: Gate,
    control: &'a Evaluation,
    candidate_gate: &'a Gate,
    candidate: &'a Evaluation,
    acquisition: &'a acquire::AcquisitionSummary,
}

#[derive(Serialize)]
struct ComparisonAggregate {
    object_count: usize,
    every_shape_candidate_condition_gate_passed: bool,
    median_object_candidate_to_control_median_error_ratio: f64,
    maximum_object_candidate_to_control_median_error_ratio: f64,
    maximum_object_p90_regression_db: f64,
    aggregate_improved_component_fraction_vs_control: f64,
}

#[derive(Serialize)]
struct CalibrationReport<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    manifest_sha256: &'static str,
    development_report_sha256: &'static str,
    model: &'a FittedModel,
    objects: Vec<ComparisonObjectReport<'a>>,
    aggregate: &'a ComparisonAggregate,
    rule: &'a manifest::ComparisonRule,
    gate: Gate,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
}
