use std::path::Path;

use serde::{Deserialize, Serialize};

use self::model::{FittedModel, TrainingSample};
use super::super::dsp::{
    Evaluation, ModeSeed, REFERENCE_LISTENER, component_candidate_median_errors, evaluate_rbf,
    evaluate_rbf_per_mode, improved_component_fraction,
};
use super::super::*;
use super::manifest::{ArchiveProfile, Manifest, MeshDescriptor, SIGMA_GRID};

mod model;
mod publish;

const SCHEMA: &str =
    "nextengine.experimental-realimpact-frequency-spatial-preregistration.manifest.v1";
const STUDY_ID: &str = "physical-sound-realimpact-frequency-conditioned-vertical";
const REVISION: &str = "v1-development-preregistration";
const MANIFEST_SHA256: &str = "88cac5bcbc79870ac8c054288a5eb5b0f1300a9b697da71dd38c8108861c709f";
const CALIBRATION_REVISION: &str = "v1-calibration";
const CALIBRATION_MANIFEST_SHA256: &str =
    "92ebe6a7fbb3207dd7d2082076af3209304a0972430d3e59fd24ed8fa99642f8";
const DEVELOPMENT_REPORT_SHA256: &str =
    "48a150d74e478ce55fb31047bc40f43727ab183d708ce1ae063c7a6aa2ad819b";
const REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-frequency-spatial-development.report.v1";
const CALIBRATION_REPORT_SCHEMA: &str =
    "nextengine.experimental-realimpact-frequency-spatial-calibration.report.v1";
const DEVELOPMENT_OBJECTS: [&str; 12] = [
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
];
const CALIBRATION_OBJECTS: [&str; 2] = ["100_Frisbee", "32_WoodChalice"];
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
        "frequency-spatial development manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("parse frequency-spatial manifest: {error}"))?;
    validate_development_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "frequency-spatial manifest has no parent".to_owned())?;
    let source_reports = manifest
        .source_reports
        .iter()
        .map(|reference| read_reference(root, base, &reference.path, &reference.sha256))
        .collect::<Result<Vec<_>, String>>()?;
    validate_source_reports(&source_reports)?;

    let mut objects = Vec::new();
    for block in &manifest.development_blocks {
        let profile = manifest
            .archive_profiles
            .iter()
            .find(|profile| profile.dataset_object_id == block.object_id)
            .ok_or_else(|| format!("missing frequency profile for {}", block.object_id))?;
        let bytes = read_reference(root, base, &block.path, &block.sha256)?;
        objects.push(decode_block(profile, &bytes)?);
    }

    let mut work = Vec::new();
    let mut training_rows = Vec::<(usize, f64, f64)>::new();
    for (object_index, object) in objects.iter().enumerate() {
        let control = evaluate_rbf(
            &object.rows,
            &object.modes,
            manifest.mode_extractor.sample_rate_hz,
            manifest.control.sigma_metres,
        )?;
        let grid = SIGMA_GRID
            .into_iter()
            .map(|sigma_metres| {
                let evaluation = evaluate_rbf(
                    &object.rows,
                    &object.modes,
                    manifest.mode_extractor.sample_rate_hz,
                    sigma_metres,
                )?;
                let errors = component_candidate_median_errors(&evaluation);
                Ok(FrequencySigmaEvaluation {
                    sigma_metres,
                    component_errors: errors,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let targets = (0..object.modes.len())
            .map(|mode_index| {
                let frequency_hz = object.modes[mode_index].frequency_hz;
                let target_sigma_metres = grid
                    .iter()
                    .map(|candidate| {
                        let (candidate_frequency, error) = candidate.component_errors[mode_index];
                        (candidate.sigma_metres, candidate_frequency, error)
                    })
                    .min_by(|left, right| {
                        left.2
                            .total_cmp(&right.2)
                            .then_with(|| left.0.total_cmp(&right.0))
                    })
                    .map(|(sigma, candidate_frequency, _)| {
                        if candidate_frequency != frequency_hz {
                            Err("frequency grid component lineage changed".to_owned())
                        } else {
                            Ok(sigma)
                        }
                    })
                    .expect("frozen sigma grid is non-empty")?;
                training_rows.push((object_index, frequency_hz, target_sigma_metres));
                Ok(ModeTarget {
                    frequency_hz,
                    target_sigma_metres,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        work.push(FrequencyWork { control, targets });
    }
    let training = training_rows
        .iter()
        .map(
            |(object_index, frequency_hz, target_sigma_metres)| TrainingSample {
                object_id: &objects[*object_index].object_id,
                descriptor: &objects[*object_index].descriptor,
                frequency_hz: *frequency_hz,
                target_sigma_metres: *target_sigma_metres,
            },
        )
        .collect::<Vec<_>>();
    let model = model::fit(&training)?;
    let reports = objects
        .iter()
        .zip(&work)
        .map(|(object, work)| object_report(object, work, &model, &manifest))
        .collect::<Result<Vec<_>, String>>()?;
    let target_sigma_counts = SIGMA_GRID
        .into_iter()
        .map(|sigma_metres| SigmaCount {
            sigma_metres,
            mode_count: training
                .iter()
                .filter(|sample| sample.target_sigma_metres == sigma_metres)
                .count(),
        })
        .collect();
    let report = DevelopmentReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "FrequencyConditionedBandwidthDevelopmentFit",
        claim: "DEVELOPMENT_FIT_ONLY / FRESH_CALIBRATION_AND_PRESERVED_HOLDOUT_UNOPENED / NO_TRANSFER_ANGLE_DISTANCE_3D_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: STUDY_ID,
        revision: REVISION,
        manifest_sha256: MANIFEST_SHA256,
        model: &model,
        target_sigma_counts,
        objects: reports,
        condition_gate: &manifest.condition_gate,
        calibration_rule: &manifest.calibration_rule,
        holdout_rule: &manifest.holdout_rule,
        allowed_claims: &manifest.allowed_claims,
        prohibited_claims: &manifest.prohibited_claims,
        next_action: "freeze this report in a calibration manifest, evaluate once on untouched 100_Frisbee and 32_WoodChalice, and open neither ceramic holdout unless calibration passes",
    };
    let report_bytes = pretty_json(&report)?;
    let report_sha256 = sha256_hex(&report_bytes);
    let output = resolve_output_path(root, output_argument)?;
    require_empty_output(&output)?;
    publish::report(
        &output,
        "frequency-development",
        manifest_bytes,
        &[
            ("bbox-development-report.json", source_reports[0].as_slice()),
            ("bbox-calibration-report.json", source_reports[1].as_slice()),
        ],
        &report_bytes,
    )?;
    println!(
        "REALIMPACT frequency-spatial development: {}",
        output.display()
    );
    println!("frequency development manifest sha256: {MANIFEST_SHA256}");
    println!("development object count: {}", objects.len());
    println!("development mode count: {}", training.len());
    println!("report sha256: {report_sha256}");
    Ok(())
}

pub(super) fn run_calibration(
    root: &Path,
    manifest_path: &Path,
    manifest_bytes: &[u8],
    output_argument: &Path,
) -> Result<(), String> {
    require_hash(
        manifest_bytes,
        CALIBRATION_MANIFEST_SHA256,
        "frequency-spatial calibration manifest",
    )?;
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("parse frequency-spatial calibration manifest: {error}"))?;
    validate_calibration_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "frequency-spatial calibration manifest has no parent".to_owned())?;
    let development_ref = manifest
        .development_report
        .as_ref()
        .ok_or_else(|| "frequency-spatial calibration has no development report".to_owned())?;
    let development_bytes =
        read_reference(root, base, &development_ref.path, &development_ref.sha256)?;
    let development: DevelopmentPrerequisite = serde_json::from_slice(&development_bytes)
        .map_err(|error| format!("parse frequency-spatial development report: {error}"))?;
    validate_development_prerequisite(&development)?;

    let mut work = Vec::new();
    for profile in manifest
        .archive_profiles
        .iter()
        .filter(|profile| profile.role == "calibration")
    {
        let acquisition = super::acquire::run(profile, manifest.compressed_audio_prefix_bytes)?;
        let control = evaluate_rbf(
            &acquisition.rows,
            &acquisition.modes,
            manifest.mode_extractor.sample_rate_hz,
            manifest.control.sigma_metres,
        )?;
        let predicted_sigmas_metres = acquisition
            .modes
            .iter()
            .map(|mode| {
                development
                    .model
                    .predict(&acquisition.descriptor, mode.frequency_hz)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let candidate = evaluate_rbf_per_mode(
            &acquisition.rows,
            &acquisition.modes,
            manifest.mode_extractor.sample_rate_hz,
            &predicted_sigmas_metres,
        )?;
        let candidate_to_control_median_error_ratio =
            super::ratio(candidate.median_abs_error_db, control.median_abs_error_db)?;
        let p90_regression_db = candidate.p90_abs_error_db - control.p90_abs_error_db;
        let improved_component_fraction_vs_control =
            improved_component_fraction(&candidate, &control)?;
        let candidate_gate = gate(&candidate);
        work.push(CalibrationWork {
            acquisition,
            predicted_sigmas_metres,
            control,
            candidate,
            candidate_gate,
            candidate_to_control_median_error_ratio,
            p90_regression_db,
            improved_component_fraction_vs_control,
        });
    }
    let aggregate = calibration_aggregate(&work)?;
    let comparison_gate = calibration_gate(&aggregate, &manifest.calibration_rule);
    let passed = comparison_gate.passed;
    let decision = if passed {
        "FrequencyConditionedBandwidthCalibrationPassed"
    } else {
        "FrequencyConditionedBandwidthCalibrationRejected"
    };
    let next_action = if passed {
        "freeze this exact calibration report in a holdout manifest, then evaluate the frozen per-mode model once on the two untouched ceramic holdout objects"
    } else {
        "stop before holdout access; preserve both ceramic holdout objects and reject this frequency-conditioned bandwidth hypothesis"
    };
    let objects = work
        .iter()
        .map(|object| CalibrationObjectReport {
            object_id: &object.acquisition.object_id,
            descriptor: &object.acquisition.descriptor,
            mode_frequencies_hz: object
                .acquisition
                .modes
                .iter()
                .map(|mode| mode.frequency_hz)
                .collect(),
            predicted_sigmas_metres: &object.predicted_sigmas_metres,
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
        claim: "OBJECT_DISJOINT_FREQUENCY_CONDITIONED_CALIBRATION_ONLY / CERAMIC_HOLDOUT_UNOPENED / NO_TRANSFER_ANGLE_DISTANCE_3D_MATERIAL_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY",
        study_id: STUDY_ID,
        revision: CALIBRATION_REVISION,
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
    super::publish::calibration(
        &output,
        manifest_bytes,
        &development_bytes,
        &acquisitions,
        &report_bytes,
    )?;
    println!(
        "REALIMPACT frequency-spatial calibration: {}",
        output.display()
    );
    println!("frequency calibration manifest sha256: {CALIBRATION_MANIFEST_SHA256}");
    println!("calibration object count: {}", acquisitions.len());
    println!("calibration gate passed: {passed}");
    println!("report decision: {decision}");
    println!("report sha256: {report_sha256}");
    Ok(())
}

fn object_report<'a>(
    object: &'a ReusedObject,
    work: &'a FrequencyWork,
    model: &FittedModel,
    manifest: &Manifest,
) -> Result<DevelopmentObjectReport<'a>, String> {
    let predicted_sigmas = object
        .modes
        .iter()
        .map(|mode| model.predict(&object.descriptor, mode.frequency_hz))
        .collect::<Result<Vec<_>, String>>()?;
    let predicted = evaluate_rbf_per_mode(
        &object.rows,
        &object.modes,
        manifest.mode_extractor.sample_rate_hz,
        &predicted_sigmas,
    )?;
    Ok(DevelopmentObjectReport {
        object_id: &object.object_id,
        descriptor: &object.descriptor,
        source_block_sha256: &object.source_block_sha256,
        modes: &work.targets,
        predicted_sigmas_metres: predicted_sigmas,
        candidate_to_control_median_error_ratio: super::ratio(
            predicted.median_abs_error_db,
            work.control.median_abs_error_db,
        )?,
        improved_component_fraction_vs_control: improved_component_fraction(
            &predicted,
            &work.control,
        )?,
        control_gate: gate(&work.control),
        control: &work.control,
        candidate_gate: gate(&predicted),
        candidate: predicted,
    })
}

fn decode_block(profile: &ArchiveProfile, bytes: &[u8]) -> Result<ReusedObject, String> {
    let row_bytes = profile
        .audio_entry
        .sample_count
        .checked_mul(4)
        .ok_or_else(|| "frequency block row size overflow".to_owned())?;
    if bytes.len() != row_bytes * 15 {
        return Err(format!(
            "frequency block size changed for {}",
            profile.dataset_object_id
        ));
    }
    let rows = bytes
        .chunks_exact(row_bytes)
        .enumerate()
        .map(|(row_index, row)| decode_row(row, row_index))
        .collect::<Result<Vec<_>, String>>()?;
    let extracted =
        crate::physical_sound_registry_command::transfer_calibration::extract_v2_spatial_modes(
            &rows[REFERENCE_LISTENER],
        )?;
    let modes = extracted
        .into_iter()
        .map(|(frequency_hz, persistent)| ModeSeed {
            frequency_hz,
            persistent,
        })
        .collect();
    Ok(ReusedObject {
        object_id: profile.dataset_object_id.clone(),
        descriptor: profile.mesh_descriptor.clone(),
        source_block_sha256: sha256_hex(bytes),
        rows,
        modes,
    })
}

fn decode_row(bytes: &[u8], row_index: usize) -> Result<Vec<f64>, String> {
    bytes
        .chunks_exact(4)
        .enumerate()
        .map(|(sample_index, chunk)| {
            let value = f32::from_le_bytes(chunk.try_into().expect("four bytes"));
            value
                .is_finite()
                .then_some(f64::from(value))
                .ok_or_else(|| {
                    format!("frequency row {row_index} sample {sample_index} is non-finite")
                })
        })
        .collect()
}

fn read_reference(
    root: &Path,
    base: &Path,
    relative: &Path,
    expected_sha256: &str,
) -> Result<Vec<u8>, String> {
    let path = canonical_external_file(
        root,
        &if relative.is_absolute() {
            relative.to_path_buf()
        } else {
            base.join(relative)
        },
        "frequency-spatial prerequisite",
    )?;
    let bytes = read_bounded_file(&path, 128 * 1024 * 1024, "frequency-spatial prerequisite")?;
    require_hash(&bytes, expected_sha256, "frequency-spatial prerequisite")?;
    Ok(bytes)
}

fn validate_development_manifest(manifest: &Manifest) -> Result<(), String> {
    let development = manifest
        .roles
        .development
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let calibration = manifest
        .roles
        .calibration
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
    if manifest.schema != SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != REVISION
        || manifest.phase != "development"
        || development != DEVELOPMENT_OBJECTS
        || calibration != CALIBRATION_OBJECTS
        || holdout != HOLDOUT_OBJECTS
        || block_ids != DEVELOPMENT_OBJECTS
        || manifest.source_reports.len() != 2
        || manifest.source_reports[0].sha256
            != "3d18358b50442ee9b171bbacc154f7713e3240efa6f4d50f61f2598b7ac4962f"
        || manifest.source_reports[1].sha256
            != "ffb176873e0fbb8cabf3083c9e9f507a31ecdc76cea83e61c1afa28a4a00aad6"
        || manifest.archive_profiles.len() != 16
        || manifest.shape_model.id != "acoustic-scale-conditioned-mode-rbf-bandwidth-v1"
        || manifest.shape_model.speed_of_sound_metres_per_second != Some(model::SPEED_OF_SOUND)
        || manifest.shape_model.sigma_grid_metres != SIGMA_GRID
        || manifest.control.sigma_metres != 0.52
        || manifest.development_report.is_some()
        || manifest.calibration_report.is_some()
    {
        return Err("frequency-spatial manifest does not match the frozen protocol".to_owned());
    }
    validate_profiles_and_blocks(manifest)?;
    Ok(())
}

fn validate_calibration_manifest(manifest: &Manifest) -> Result<(), String> {
    let development = manifest
        .development_report
        .as_ref()
        .ok_or_else(|| "frequency calibration has no development report".to_owned())?;
    if manifest.schema != SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != CALIBRATION_REVISION
        || manifest.phase != "calibration"
        || manifest.access_state_at_freeze
            != "frequency development blocks and report opened; fresh calibration and preserved holdout deconvolved audio payloads remain unopened"
        || development.path.as_path() != Path::new("frequency-development-a/report.json")
        || development.sha256 != DEVELOPMENT_REPORT_SHA256
        || manifest.calibration_report.is_some()
        || manifest.source_reports.len() != 2
        || manifest.source_reports[0].sha256
            != "3d18358b50442ee9b171bbacc154f7713e3240efa6f4d50f61f2598b7ac4962f"
        || manifest.source_reports[1].sha256
            != "ffb176873e0fbb8cabf3083c9e9f507a31ecdc76cea83e61c1afa28a4a00aad6"
        || manifest.archive_profiles.len() != 16
        || manifest.shape_model.id != "acoustic-scale-conditioned-mode-rbf-bandwidth-v1"
        || manifest.shape_model.speed_of_sound_metres_per_second != Some(model::SPEED_OF_SOUND)
        || manifest.shape_model.sigma_grid_metres != SIGMA_GRID
        || manifest.control.sigma_metres != 0.52
    {
        return Err(
            "frequency-spatial calibration manifest does not match the frozen protocol".to_owned(),
        );
    }
    validate_profiles_and_blocks(manifest)
}

fn validate_profiles_and_blocks(manifest: &Manifest) -> Result<(), String> {
    let expected = DEVELOPMENT_OBJECTS
        .into_iter()
        .map(|object_id| ("development", object_id))
        .chain(
            CALIBRATION_OBJECTS
                .into_iter()
                .map(|object_id| ("calibration", object_id)),
        )
        .chain(
            HOLDOUT_OBJECTS
                .into_iter()
                .map(|object_id| ("holdout", object_id)),
        )
        .collect::<Vec<_>>();
    if manifest.archive_profiles.len() != expected.len() {
        return Err("frequency profile count changed".to_owned());
    }
    for (profile, (expected_role, expected_id)) in manifest.archive_profiles.iter().zip(expected) {
        if profile.role != expected_role || profile.dataset_object_id != expected_id {
            return Err("frequency profile order or role changed".to_owned());
        }
        profile.validate()?;
    }
    for block in &manifest.development_blocks {
        let profile = manifest
            .archive_profiles
            .iter()
            .find(|profile| profile.dataset_object_id == block.object_id)
            .ok_or_else(|| format!("frequency block has no profile: {}", block.object_id))?;
        if block.sha256.len() != 64 || block.sample_count != profile.audio_entry.sample_count {
            return Err(format!(
                "frequency block lineage changed: {}",
                block.object_id
            ));
        }
    }
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
        || report.decision != "FrequencyConditionedBandwidthDevelopmentFit"
        || report.study_id != STUDY_ID
        || report.revision != REVISION
        || report.manifest_sha256 != MANIFEST_SHA256
        || object_ids != DEVELOPMENT_OBJECTS
        || report.objects.iter().any(|object| object.modes.is_empty())
    {
        return Err(
            "frequency-spatial development report does not authorize calibration".to_owned(),
        );
    }
    for object in &report.objects {
        for mode in &object.modes {
            if !SIGMA_GRID.contains(&mode.target_sigma_metres) {
                return Err("frequency development mode target changed".to_owned());
            }
            report
                .model
                .predict(&object.descriptor, mode.frequency_hz)?;
        }
    }
    Ok(())
}

fn calibration_aggregate(work: &[CalibrationWork]) -> Result<CalibrationAggregate, String> {
    if work.is_empty() {
        return Err("frequency-spatial calibration has no objects".to_owned());
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
    Ok(CalibrationAggregate {
        object_count: work.len(),
        every_shape_candidate_condition_gate_passed: work
            .iter()
            .all(|object| object.candidate_gate.passed),
        median_object_candidate_to_control_median_error_ratio: median_ratio,
        maximum_object_candidate_to_control_median_error_ratio: work
            .iter()
            .map(|object| object.candidate_to_control_median_error_ratio)
            .max_by(f64::total_cmp)
            .expect("non-empty calibration"),
        maximum_object_p90_regression_db: work
            .iter()
            .map(|object| object.p90_regression_db)
            .max_by(f64::total_cmp)
            .expect("non-empty calibration"),
        aggregate_improved_component_fraction_vs_control: work
            .iter()
            .map(|object| object.improved_component_fraction_vs_control)
            .sum::<f64>()
            / work.len() as f64,
    })
}

fn calibration_gate(
    aggregate: &CalibrationAggregate,
    rule: &super::manifest::ComparisonRule,
) -> Gate {
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

fn validate_source_reports(reports: &[Vec<u8>]) -> Result<(), String> {
    let headers = reports
        .iter()
        .map(|bytes| {
            serde_json::from_slice::<SourceReportHeader>(bytes)
                .map_err(|error| format!("parse frequency source report: {error}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if headers.len() != 2
        || headers[0].decision != "MeshConditionedBandwidthDevelopmentFit"
        || headers[1].decision != "MeshConditionedBandwidthCalibrationRejected"
    {
        return Err("frequency source reports do not preserve the rejected lineage".to_owned());
    }
    Ok(())
}

struct ReusedObject {
    object_id: String,
    descriptor: MeshDescriptor,
    source_block_sha256: String,
    rows: Vec<Vec<f64>>,
    modes: Vec<ModeSeed>,
}

struct FrequencyWork {
    control: Evaluation,
    targets: Vec<ModeTarget>,
}

struct FrequencySigmaEvaluation {
    sigma_metres: f64,
    component_errors: Vec<(f64, f64)>,
}

#[derive(Deserialize, Serialize)]
struct ModeTarget {
    frequency_hz: f64,
    target_sigma_metres: f64,
}

#[derive(Serialize)]
struct SigmaCount {
    sigma_metres: f64,
    mode_count: usize,
}

#[derive(Serialize)]
struct DevelopmentObjectReport<'a> {
    object_id: &'a str,
    descriptor: &'a MeshDescriptor,
    source_block_sha256: &'a str,
    modes: &'a [ModeTarget],
    predicted_sigmas_metres: Vec<f64>,
    candidate_to_control_median_error_ratio: f64,
    improved_component_fraction_vs_control: f64,
    control_gate: Gate,
    control: &'a Evaluation,
    candidate_gate: Gate,
    candidate: Evaluation,
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
    model: &'a FittedModel,
    target_sigma_counts: Vec<SigmaCount>,
    objects: Vec<DevelopmentObjectReport<'a>>,
    condition_gate: &'a super::manifest::ConditionGate,
    calibration_rule: &'a super::manifest::ComparisonRule,
    holdout_rule: &'a super::manifest::ComparisonRule,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
}

struct CalibrationWork {
    acquisition: super::acquire::Acquisition,
    predicted_sigmas_metres: Vec<f64>,
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
    study_id: String,
    revision: String,
    manifest_sha256: String,
    model: FittedModel,
    objects: Vec<DevelopmentObjectPrerequisite>,
}

#[derive(Deserialize)]
struct DevelopmentObjectPrerequisite {
    object_id: String,
    descriptor: MeshDescriptor,
    modes: Vec<ModeTarget>,
}

#[derive(Serialize)]
struct CalibrationObjectReport<'a> {
    object_id: &'a str,
    descriptor: &'a MeshDescriptor,
    mode_frequencies_hz: Vec<f64>,
    predicted_sigmas_metres: &'a [f64],
    candidate_to_control_median_error_ratio: f64,
    p90_regression_db: f64,
    improved_component_fraction_vs_control: f64,
    control_gate: Gate,
    control: &'a Evaluation,
    candidate_gate: &'a Gate,
    candidate: &'a Evaluation,
    acquisition: &'a super::acquire::AcquisitionSummary,
}

#[derive(Serialize)]
struct CalibrationAggregate {
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
    objects: Vec<CalibrationObjectReport<'a>>,
    aggregate: &'a CalibrationAggregate,
    rule: &'a super::manifest::ComparisonRule,
    gate: Gate,
    allowed_claims: &'a [String],
    prohibited_claims: &'a [String],
    next_action: &'static str,
}

#[derive(Deserialize)]
struct SourceReportHeader {
    decision: String,
}
