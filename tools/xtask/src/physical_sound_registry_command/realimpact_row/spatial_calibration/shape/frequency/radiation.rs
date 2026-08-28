use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use self::model::predict;
use super::super::super::dsp::{
    Evaluation, evaluate_db_predictions, evaluate_rbf, improved_component_fraction,
    relative_complex_participation, relative_db_participation,
};
use super::super::super::*;
use super::super::manifest::{ConditionGate, Manifest as SourceManifest, MeshDescriptor};

mod model;

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-realimpact-modal-radiation-representation.manifest.v1";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-modal-radiation-representation.report.v1";
const STUDY_ID: &str = "physical-sound-realimpact-complex-modal-radiation";
const REVISION: &str = "v1-development-preregistration";
const MANIFEST_SHA256: &str = "686c1d423bc46de81ba66104fbbca90fe8ae8ff865462f5f99cfae850f47a0ff";
const SOURCE_MANIFEST_SHA256: &str =
    "92ebe6a7fbb3207dd7d2082076af3209304a0972430d3e59fd24ed8fa99642f8";
const SOURCE_DEVELOPMENT_REPORT_SHA256: &str =
    "48a150d74e478ce55fb31047bc40f43727ab183d708ce1ae063c7a6aa2ad819b";
const SOURCE_CALIBRATION_REPORT_SHA256: &str =
    "42b6605db6537e9c84f31494fd0cbc3acce41b807c00fede2b15d2f879b69983";
const DEVELOPMENT_OBJECTS: [&str; 14] = [
    "49_PlasticBowl",
    "67_IronPlate",
    "31_WoodSlab",
    "83_WoodVase",
    "81_WoodPad",
    "34_WoodMug",
    "48_PlasticBowl",
    "90_MetalLadle",
    "9_BowlCeramic",
    "68_WoodBoard",
    "10_bowl",
    "36_SmallMeasuringCup",
    "100_Frisbee",
    "32_WoodChalice",
];
const HOLDOUT_OBJECTS: [&str; 2] = ["65_PitcherCeramic", "63_SmallPlanterCeramic"];

pub(super) fn run_development(
    root: &Path,
    manifest_path: &Path,
    manifest_bytes: &[u8],
    output_argument: &Path,
) -> Result<(), String> {
    require_hash(
        manifest_bytes,
        MANIFEST_SHA256,
        "modal-radiation development manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("parse modal-radiation manifest: {error}"))?;
    validate_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "modal-radiation manifest has no parent".to_owned())?;
    let source_manifest_bytes = super::read_reference(
        root,
        base,
        &manifest.source_manifest.path,
        &manifest.source_manifest.sha256,
    )?;
    let source_manifest: SourceManifest = serde_json::from_slice(&source_manifest_bytes)
        .map_err(|error| format!("parse modal-radiation source manifest: {error}"))?;
    super::validate_calibration_manifest(&source_manifest)?;
    let source_reports = manifest
        .source_reports
        .iter()
        .map(|reference| super::read_reference(root, base, &reference.path, &reference.sha256))
        .collect::<Result<Vec<_>, String>>()?;
    validate_source_reports(&source_reports)?;

    let mut objects = Vec::new();
    for block in &manifest.development_blocks {
        let profile = source_manifest
            .archive_profiles
            .iter()
            .find(|profile| profile.dataset_object_id == block.object_id)
            .ok_or_else(|| format!("modal-radiation profile missing for {}", block.object_id))?;
        if block.sample_count != profile.audio_entry.sample_count {
            return Err(format!(
                "modal-radiation sample count changed for {}",
                block.object_id
            ));
        }
        let bytes = super::read_reference(root, base, &block.path, &block.sha256)?;
        objects.push(super::decode_block(profile, &bytes)?);
    }

    let work = objects
        .iter()
        .map(|object| evaluate_object(object, &manifest))
        .collect::<Result<Vec<_>, String>>()?;
    let aggregate = aggregate(&work)?;
    let representation_gate = representation_gate(&aggregate, &manifest.representation_rule);
    let supported = representation_gate.passed;
    let decision = if supported {
        "ComplexMultipoleRepresentationDevelopmentSupported"
    } else {
        "ComplexMultipoleRepresentationDevelopmentRejected"
    };
    let next_action = if supported {
        "freeze a fresh object/impact/angle/distance calibration before any geometry-driven cooker or holdout access; this development result alone grants no spatial claim"
    } else {
        "reject this compact complex basis; keep both ceramic holdouts sealed and do not access fresh angle/distance data before a different representation hypothesis exists"
    };
    let reports = objects
        .iter()
        .zip(&work)
        .map(|(object, work)| ObjectReport {
            object_id: &object.object_id,
            descriptor: &object.descriptor,
            source_block_sha256: &object.source_block_sha256,
            mode_count: object.modes.len(),
            candidate_to_control_median_error_ratio: work.candidate_to_control_median_error_ratio,
            p90_regression_db: work.p90_regression_db,
            improved_component_fraction_vs_control: work.improved_component_fraction_vs_control,
            control_gate: gate(&work.control),
            control: &work.control,
            candidate_gate: &work.candidate_gate,
            candidate: &work.candidate,
        })
        .collect();
    let report = Report {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision,
        claim: "ALREADY_OPEN_DEVELOPMENT_REPRESENTATION_SUFFICIENCY_ONLY / NO_FRESH_CALIBRATION_HOLDOUT_ANGLE_DISTANCE_3D_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: STUDY_ID,
        revision: REVISION,
        manifest_sha256: MANIFEST_SHA256,
        source_manifest_sha256: SOURCE_MANIFEST_SHA256,
        source_development_report_sha256: SOURCE_DEVELOPMENT_REPORT_SHA256,
        source_calibration_report_sha256: SOURCE_CALIBRATION_REPORT_SHA256,
        candidate: &manifest.candidate,
        objects: reports,
        aggregate: &aggregate,
        condition_gate: &manifest.condition_gate,
        representation_rule: &manifest.representation_rule,
        gate: representation_gate,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        next_action,
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    super::publish::report(
        &output,
        "radiation-development",
        manifest_bytes,
        &[
            ("source-manifest.json", source_manifest_bytes.as_slice()),
            (
                "source-development-report.json",
                source_reports[0].as_slice(),
            ),
            (
                "source-calibration-report.json",
                source_reports[1].as_slice(),
            ),
        ],
        &report_bytes,
    )?;
    println!(
        "REALIMPACT modal-radiation development: {}",
        output.display()
    );
    println!("modal-radiation manifest sha256: {MANIFEST_SHA256}");
    println!("development object count: {}", objects.len());
    println!(
        "development mode count: {}",
        objects
            .iter()
            .map(|object| object.modes.len())
            .sum::<usize>()
    );
    println!("representation gate passed: {supported}");
    println!("report decision: {decision}");
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn evaluate_object(
    object: &super::ReusedObject,
    manifest: &Manifest,
) -> Result<ObjectWork, String> {
    let control = evaluate_rbf(
        &object.rows,
        &object.modes,
        manifest.sample_rate_hz,
        manifest.control.sigma_metres,
    )?;
    let complex =
        relative_complex_participation(&object.rows, &object.modes, manifest.sample_rate_hz)?;
    let target_db =
        relative_db_participation(&object.rows, &object.modes, manifest.sample_rate_hz)?;
    let predicted_db = predict(
        &object.descriptor,
        &object.modes,
        &complex,
        manifest.candidate.maximum_order,
        manifest.candidate.ridge,
        manifest.candidate.speed_of_sound_metres_per_second,
    )?;
    let candidate = evaluate_db_predictions(&object.modes, &target_db, &predicted_db)?;
    let candidate_to_control_median_error_ratio =
        super::super::ratio(candidate.median_abs_error_db, control.median_abs_error_db)?;
    let p90_regression_db = candidate.p90_abs_error_db - control.p90_abs_error_db;
    let improved_component_fraction_vs_control = improved_component_fraction(&candidate, &control)?;
    let candidate_gate = gate(&candidate);
    Ok(ObjectWork {
        control,
        candidate,
        candidate_gate,
        candidate_to_control_median_error_ratio,
        p90_regression_db,
        improved_component_fraction_vs_control,
    })
}

fn aggregate(work: &[ObjectWork]) -> Result<Aggregate, String> {
    if work.len() != DEVELOPMENT_OBJECTS.len() {
        return Err("modal-radiation aggregate object count changed".to_owned());
    }
    let mut ratios = work
        .iter()
        .map(|object| object.candidate_to_control_median_error_ratio)
        .collect::<Vec<_>>();
    ratios.sort_by(f64::total_cmp);
    let median_ratio = (ratios[ratios.len() / 2 - 1] + ratios[ratios.len() / 2]) * 0.5;
    Ok(Aggregate {
        object_count: work.len(),
        every_candidate_condition_gate_passed: work
            .iter()
            .all(|object| object.candidate_gate.passed),
        median_object_candidate_to_control_median_error_ratio: median_ratio,
        maximum_object_candidate_to_control_median_error_ratio: work
            .iter()
            .map(|object| object.candidate_to_control_median_error_ratio)
            .max_by(f64::total_cmp)
            .expect("non-empty representation work"),
        maximum_object_p90_regression_db: work
            .iter()
            .map(|object| object.p90_regression_db)
            .max_by(f64::total_cmp)
            .expect("non-empty representation work"),
        aggregate_improved_component_fraction_vs_control: work
            .iter()
            .map(|object| object.improved_component_fraction_vs_control)
            .sum::<f64>()
            / work.len() as f64,
    })
}

fn representation_gate(aggregate: &Aggregate, rule: &RepresentationRule) -> Gate {
    let checks = vec![
        GateCheck {
            metric: "every_candidate_condition_gate_passed",
            observed: if aggregate.every_candidate_condition_gate_passed {
                1.0
            } else {
                0.0
            },
            relation: "==",
            threshold: if rule.require_every_candidate_condition_gate {
                1.0
            } else {
                0.0
            },
            passed: aggregate.every_candidate_condition_gate_passed
                == rule.require_every_candidate_condition_gate,
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

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    let development = manifest
        .roles
        .development
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let holdout = manifest
        .roles
        .holdout
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let block_ids = manifest
        .development_blocks
        .iter()
        .map(|block| block.object_id.as_str())
        .collect::<Vec<_>>();
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != REVISION
        || manifest.phase != "development"
        || development != DEVELOPMENT_OBJECTS
        || holdout != HOLDOUT_OBJECTS
        || block_ids != DEVELOPMENT_OBJECTS
        || manifest.source_manifest.path.as_path()
            != Path::new("frequency-calibration-manifest.json")
        || manifest.source_manifest.sha256 != SOURCE_MANIFEST_SHA256
        || manifest.source_reports.len() != 2
        || manifest.source_reports[0].sha256 != SOURCE_DEVELOPMENT_REPORT_SHA256
        || manifest.source_reports[1].sha256 != SOURCE_CALIBRATION_REPORT_SHA256
        || manifest.development_blocks.len() != DEVELOPMENT_OBJECTS.len()
        || manifest.access_state_at_freeze
            != "fourteen development blocks and their scalar reports are opened; complex per-mode responses have not been inspected; both ceramic holdout deconvolved payloads remain unopened"
        || manifest.sample_rate_hz != 48_000
        || manifest.mode_extractor_id != "injective-modal-16-fft65536-v2"
        || manifest.reference_listener != 7
        || manifest.anchor_listeners != [0, 2, 4, 6, 7, 8, 10, 12, 14]
        || manifest.held_listeners != [1, 3, 5, 9, 11, 13]
        || manifest.control.id != "coordinate-only-rbf-sigma052-ridge001-v1"
        || manifest.control.sigma_metres != 0.52
        || manifest.candidate.id != "axisymmetric-complex-multipole-order3-ridge001-v1"
        || manifest.candidate.maximum_order != 3
        || manifest.candidate.ridge != 0.001
        || manifest.candidate.speed_of_sound_metres_per_second != 343.0
        || manifest.condition_gate.minimum_component_count != 12
        || manifest.condition_gate.maximum_median_abs_error_db != 7.0
        || manifest.condition_gate.maximum_p90_abs_error_db != 18.0
        || manifest
            .condition_gate
            .maximum_persistent_median_abs_error_db
            != 8.0
        || manifest
            .condition_gate
            .minimum_improved_component_fraction_vs_constant
            != 0.5
        || manifest
            .condition_gate
            .maximum_median_error_ratio_to_constant
            != 0.9
        || !manifest
            .representation_rule
            .require_every_candidate_condition_gate
        || manifest
            .representation_rule
            .maximum_median_object_candidate_to_control_median_error_ratio
            != 0.9
        || manifest
            .representation_rule
            .maximum_each_object_candidate_to_control_median_error_ratio
            != 1.1
        || manifest
            .representation_rule
            .maximum_each_object_p90_regression_db
            != 2.0
        || manifest
            .representation_rule
            .minimum_aggregate_improved_component_fraction_vs_control
            != 0.6
    {
        return Err("modal-radiation manifest does not match the frozen protocol".to_owned());
    }
    for block in &manifest.development_blocks {
        if block.sha256.len() != 64 || block.sample_count < 131_072 {
            return Err(format!(
                "modal-radiation block lineage changed: {}",
                block.object_id
            ));
        }
    }
    Ok(())
}

fn validate_source_reports(reports: &[Vec<u8>]) -> Result<(), String> {
    if reports.len() != 2 {
        return Err("modal-radiation source report count changed".to_owned());
    }
    let development: SourceReport = serde_json::from_slice(&reports[0])
        .map_err(|error| format!("parse modal-radiation development source: {error}"))?;
    let calibration: SourceReport = serde_json::from_slice(&reports[1])
        .map_err(|error| format!("parse modal-radiation calibration source: {error}"))?;
    if development.decision != "FrequencyConditionedBandwidthDevelopmentFit"
        || calibration.decision != "FrequencyConditionedBandwidthCalibrationRejected"
    {
        return Err("modal-radiation source report decision changed".to_owned());
    }
    Ok(())
}

struct ObjectWork {
    control: Evaluation,
    candidate: Evaluation,
    candidate_gate: Gate,
    candidate_to_control_median_error_ratio: f64,
    p90_regression_db: f64,
    improved_component_fraction_vs_control: f64,
}

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    study_id: String,
    revision: String,
    phase: String,
    source_manifest: FileRef,
    source_reports: Vec<FileRef>,
    roles: Roles,
    development_blocks: Vec<BlockRef>,
    access_state_at_freeze: String,
    sample_rate_hz: u32,
    mode_extractor_id: String,
    reference_listener: usize,
    anchor_listeners: Vec<usize>,
    held_listeners: Vec<usize>,
    control: Control,
    candidate: Candidate,
    condition_gate: ConditionGate,
    representation_rule: RepresentationRule,
    allowed_claims: Vec<String>,
    prohibited_claims: Vec<String>,
}

#[derive(Deserialize)]
struct FileRef {
    path: PathBuf,
    sha256: String,
}

#[derive(Deserialize)]
struct Roles {
    development: Vec<String>,
    holdout: Vec<String>,
}

#[derive(Deserialize)]
struct BlockRef {
    object_id: String,
    path: PathBuf,
    sample_count: usize,
    sha256: String,
}

#[derive(Deserialize)]
struct Control {
    id: String,
    sigma_metres: f64,
}

#[derive(Deserialize, Serialize)]
struct Candidate {
    id: String,
    maximum_order: usize,
    speed_of_sound_metres_per_second: f64,
    basis: String,
    fit: String,
    phase: String,
    evaluation: String,
    ridge: f64,
}

#[derive(Deserialize, Serialize)]
struct RepresentationRule {
    require_every_candidate_condition_gate: bool,
    maximum_median_object_candidate_to_control_median_error_ratio: f64,
    maximum_each_object_candidate_to_control_median_error_ratio: f64,
    maximum_each_object_p90_regression_db: f64,
    minimum_aggregate_improved_component_fraction_vs_control: f64,
    on_failure: String,
}

#[derive(Deserialize)]
struct SourceReport {
    decision: String,
}

#[derive(Serialize)]
struct ObjectReport<'a> {
    object_id: &'a str,
    descriptor: &'a MeshDescriptor,
    source_block_sha256: &'a str,
    mode_count: usize,
    candidate_to_control_median_error_ratio: f64,
    p90_regression_db: f64,
    improved_component_fraction_vs_control: f64,
    control_gate: Gate,
    control: &'a Evaluation,
    candidate_gate: &'a Gate,
    candidate: &'a Evaluation,
}

#[derive(Serialize)]
struct Aggregate {
    object_count: usize,
    every_candidate_condition_gate_passed: bool,
    median_object_candidate_to_control_median_error_ratio: f64,
    maximum_object_candidate_to_control_median_error_ratio: f64,
    maximum_object_p90_regression_db: f64,
    aggregate_improved_component_fraction_vs_control: f64,
}

#[derive(Serialize)]
struct Report<'a> {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    manifest_sha256: &'static str,
    source_manifest_sha256: &'static str,
    source_development_report_sha256: &'static str,
    source_calibration_report_sha256: &'static str,
    candidate: &'a Candidate,
    objects: Vec<ObjectReport<'a>>,
    aggregate: &'a Aggregate,
    condition_gate: &'a ConditionGate,
    representation_rule: &'a RepresentationRule,
    gate: Gate,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
}
