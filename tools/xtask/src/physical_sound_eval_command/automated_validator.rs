use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

use serde::Serialize;

use super::audio_analysis::WavAudio;
use super::*;

const ADMITTED_SOURCE_FAMILY: &str = "rigid-impact";

#[derive(Clone, Debug, Serialize)]
pub(super) struct DomainReport {
    source_family: String,
    generator_revision: String,
    generator_sha256: String,
    deterministic_probe_seed: u64,
    learned_acceptance: bool,
    fallback_action: &'static str,
    fallback: FileAnalysisReport,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct MutationSuiteReport {
    pub(super) status: &'static str,
    control_profile_sha256: String,
    repeated_control_profile_sha256: String,
    repeated_identical: bool,
    families: Vec<MutationFamilyReport>,
}

#[derive(Clone, Debug, Serialize)]
struct MutationFamilyReport {
    family: &'static str,
    status: &'static str,
    steps: Vec<MutationStepReport>,
}

#[derive(Clone, Debug, Serialize)]
struct MutationStepReport {
    severity: f64,
    unit: &'static str,
    expected_decision: &'static str,
    observed_decision: &'static str,
    hard_failure_tags: Vec<String>,
    passed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct RelationReport {
    id: String,
    kind: &'static str,
    pub(super) status: &'static str,
    pub(super) failure_tags: Vec<String>,
    measurements: Vec<RelationMeasurementReport>,
}

#[derive(Clone, Debug, Serialize)]
struct RelationMeasurementReport {
    left_entry: String,
    right_entry: String,
    exact_wav_equal: Option<bool>,
    raw_rms_step_db: Option<f64>,
    modal_assignment_cost: Option<f64>,
    gain_matched_log_spectrum_rmse_db: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct CoverageReport {
    pub(super) status: &'static str,
    admitted_source_family: &'static str,
    required_controls_per_object_material: [&'static str; 4],
    pub(super) missing_controls: Vec<String>,
    uncovered_impact_entries: Vec<String>,
}

pub(super) struct ValidatorDecisionInputs<'a> {
    pub entries: &'a [EntryReport],
    pub domain: Option<&'a DomainReport>,
    pub mutation_suite: &'a MutationSuiteReport,
    pub coverage: &'a CoverageReport,
    pub relations: &'a [RelationReport],
}

pub(super) fn build_domain_report(
    resolved: Option<(&ValidatorDeclaration, &Analysis)>,
) -> Result<Option<DomainReport>, String> {
    let Some((declaration, fallback)) = resolved else {
        return Ok(None);
    };
    let hard_failures = signal_failure_tags(&fallback.report)
        .into_iter()
        .filter(|tag| is_hard_signal_failure(tag))
        .collect::<Vec<_>>();
    if !hard_failures.is_empty() {
        return Err(format!(
            "declared authored fallback fails the hard signal gate: {}",
            hard_failures.join(",")
        ));
    }
    Ok(Some(DomainReport {
        source_family: declaration.source_family.clone(),
        generator_revision: declaration.generator_revision.clone(),
        generator_sha256: declaration.generator_sha256.clone(),
        deterministic_probe_seed: declaration.deterministic_probe_seed,
        learned_acceptance: false,
        fallback_action: "UseDeclaredAuthoredClip",
        fallback: fallback.report.clone(),
    }))
}

pub(super) fn evaluate_mutation_suite() -> Result<MutationSuiteReport, String> {
    let first = build_mutation_controls()?;
    let repeated = build_mutation_controls()?;
    let first_bytes = serde_json::to_vec(&first).map_err(|error| error.to_string())?;
    let repeated_bytes = serde_json::to_vec(&repeated).map_err(|error| error.to_string())?;
    let first_hash = sha256_hex(&first_bytes);
    let repeated_hash = sha256_hex(&repeated_bytes);
    let repeated_identical = first_bytes == repeated_bytes;
    let controls_pass = first.iter().all(|family| family.status == "Pass");
    Ok(MutationSuiteReport {
        status: if controls_pass && repeated_identical {
            "Pass"
        } else {
            "Reject"
        },
        control_profile_sha256: first_hash,
        repeated_control_profile_sha256: repeated_hash,
        repeated_identical,
        families: first,
    })
}

fn build_mutation_controls() -> Result<Vec<MutationFamilyReport>, String> {
    let sample_rate_hz = 48_000_u32;
    let baseline = (0..9_600)
        .map(|frame| {
            let time = frame as f64 / f64::from(sample_rate_hz);
            let carrier =
                (2.0 * PI * 440.0 * time).sin() + 0.35 * (2.0 * PI * 1_300.0 * time).sin();
            0.32 * carrier * (-8.0 * time).exp()
        })
        .collect::<Vec<_>>();

    let clipping = vec![
        mutation_step(
            clipped(&baseline, 0.005),
            sample_rate_hz,
            0.5,
            "percent_frames",
            "Pass",
        )?,
        mutation_step(
            clipped(&baseline, 0.02),
            sample_rate_hz,
            2.0,
            "percent_frames",
            "Reject",
        )?,
    ];
    let dc = vec![
        mutation_step(
            add_dc(&baseline, 0.005),
            sample_rate_hz,
            0.005,
            "linear_offset",
            "Pass",
        )?,
        mutation_step(
            add_dc(&baseline, 0.04),
            sample_rate_hz,
            0.04,
            "linear_offset",
            "Reject",
        )?,
    ];
    let duration = vec![
        mutation_step(
            truncate_ms(&baseline, sample_rate_hz, 75),
            sample_rate_hz,
            75.0,
            "milliseconds",
            "Pass",
        )?,
        mutation_step(
            truncate_ms(&baseline, sample_rate_hz, 25),
            sample_rate_hz,
            25.0,
            "milliseconds",
            "Reject",
        )?,
    ];
    let silence = vec![mutation_step(
        vec![0.0; baseline.len()],
        sample_rate_hz,
        -240.0,
        "peak_dbfs",
        "Reject",
    )?];

    Ok(vec![
        mutation_family("clipping", clipping),
        mutation_family("dc_offset", dc),
        mutation_family("duration", duration),
        mutation_family("silence_missing_onset", silence),
    ])
}

fn mutation_family(family: &'static str, steps: Vec<MutationStepReport>) -> MutationFamilyReport {
    MutationFamilyReport {
        family,
        status: if steps.iter().all(|step| step.passed) {
            "Pass"
        } else {
            "Reject"
        },
        steps,
    }
}

fn mutation_step(
    samples: Vec<f64>,
    sample_rate_hz: u32,
    severity: f64,
    unit: &'static str,
    expected_decision: &'static str,
) -> Result<MutationStepReport, String> {
    let analysis = analyze_wav(
        "internal-av-p0a-control.wav",
        &"0".repeat(64),
        WavAudio {
            sample_format: "internal-f64".to_owned(),
            sample_rate_hz,
            channel_count: 1,
            mono_samples: samples,
        },
    )?;
    let hard_failure_tags = signal_failure_tags(&analysis.report)
        .into_iter()
        .filter(|tag| is_hard_signal_failure(tag))
        .collect::<Vec<_>>();
    let observed_decision = if hard_failure_tags.is_empty() {
        "Pass"
    } else {
        "Reject"
    };
    Ok(MutationStepReport {
        severity,
        unit,
        expected_decision,
        observed_decision,
        hard_failure_tags,
        passed: observed_decision == expected_decision,
    })
}

fn clipped(samples: &[f64], fraction: f64) -> Vec<f64> {
    let mut output = samples.to_vec();
    let count = ((output.len() as f64 * fraction).ceil() as usize).min(output.len());
    for (index, sample) in output.iter_mut().take(count).enumerate() {
        *sample = if index % 2 == 0 { 1.0 } else { -1.0 };
    }
    output
}

fn add_dc(samples: &[f64], offset: f64) -> Vec<f64> {
    samples
        .iter()
        .map(|sample| (sample + offset).clamp(-0.98, 0.98))
        .collect()
}

fn truncate_ms(samples: &[f64], sample_rate_hz: u32, milliseconds: usize) -> Vec<f64> {
    let count = (sample_rate_hz as usize * milliseconds / 1_000).min(samples.len());
    samples[..count].to_vec()
}

pub(super) fn evaluate_relations(
    relations: &[RelationSpec],
    entries: &[ResolvedEntry],
) -> Vec<RelationReport> {
    let by_id = entries
        .iter()
        .map(|entry| (entry.manifest.id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    relations
        .iter()
        .map(|relation| evaluate_relation(relation, &by_id))
        .collect()
}

fn evaluate_relation(
    relation: &RelationSpec,
    entries: &BTreeMap<&str, &ResolvedEntry>,
) -> RelationReport {
    let mut failure_tags = Vec::new();
    let mut measurements = Vec::new();
    match relation {
        RelationSpec::ExactWavRepeat { left, right, .. } => {
            let left_entry = entries[left.as_str()];
            let right_entry = entries[right.as_str()];
            let equal = left_entry.candidate_bytes == right_entry.candidate_bytes;
            if !equal {
                failure_tags.push("EXACT_WAV_REPEAT_MISMATCH".to_owned());
            }
            measurements.push(RelationMeasurementReport {
                left_entry: left.clone(),
                right_entry: right.clone(),
                exact_wav_equal: Some(equal),
                raw_rms_step_db: None,
                modal_assignment_cost: None,
                gain_matched_log_spectrum_rmse_db: None,
            });
        }
        RelationSpec::ForceResponse {
            ordered_entries, ..
        } => {
            for pair in ordered_entries.windows(2) {
                let left = entries[pair[0].as_str()];
                let right = entries[pair[1].as_str()];
                require_same_sample_rate(left, right, &mut failure_tags);
                let comparison = matched_report(&right.candidate, &left.candidate);
                if comparison.raw_rms_delta_db < FORCE_MINIMUM_RMS_STEP_DB {
                    push_unique(&mut failure_tags, "FORCE_ENERGY_NOT_MONOTONIC");
                }
                if comparison.modal_assignment_cost > FORCE_MAXIMUM_MODAL_ASSIGNMENT_COST {
                    push_unique(&mut failure_tags, "FORCE_MODAL_FREQUENCY_DRIFT");
                }
                measurements.push(RelationMeasurementReport {
                    left_entry: pair[0].clone(),
                    right_entry: pair[1].clone(),
                    exact_wav_equal: None,
                    raw_rms_step_db: Some(comparison.raw_rms_delta_db),
                    modal_assignment_cost: Some(comparison.modal_assignment_cost),
                    gain_matched_log_spectrum_rmse_db: None,
                });
            }
        }
        RelationSpec::PositionContinuity {
            ordered_entries, ..
        } => {
            for pair in ordered_entries.windows(2) {
                let left = entries[pair[0].as_str()];
                let right = entries[pair[1].as_str()];
                require_same_sample_rate(left, right, &mut failure_tags);
                let comparison = matched_report(&right.candidate, &left.candidate);
                if comparison.raw_rms_delta_db.abs() < POSITION_MINIMUM_ABSOLUTE_RMS_DELTA_DB {
                    push_unique(&mut failure_tags, "POSITION_RESPONSE_INVARIANT");
                }
                if comparison.gain_matched_multiresolution_log_spectrum_rmse_db
                    > POSITION_MAXIMUM_LOG_SPECTRUM_RMSE_DB
                {
                    push_unique(&mut failure_tags, "POSITION_RESPONSE_DISCONTINUITY");
                }
                if comparison.modal_assignment_cost > POSITION_MAXIMUM_MODAL_ASSIGNMENT_COST {
                    push_unique(&mut failure_tags, "POSITION_INVENTED_MODAL_FREQUENCIES");
                }
                measurements.push(RelationMeasurementReport {
                    left_entry: pair[0].clone(),
                    right_entry: pair[1].clone(),
                    exact_wav_equal: None,
                    raw_rms_step_db: Some(comparison.raw_rms_delta_db),
                    modal_assignment_cost: Some(comparison.modal_assignment_cost),
                    gain_matched_log_spectrum_rmse_db: Some(
                        comparison.gain_matched_multiresolution_log_spectrum_rmse_db,
                    ),
                });
            }
        }
    }
    RelationReport {
        id: relation.id().to_owned(),
        kind: relation.kind(),
        status: if failure_tags.is_empty() {
            "Pass"
        } else {
            "Reject"
        },
        failure_tags,
        measurements,
    }
}

fn require_same_sample_rate(
    left: &ResolvedEntry,
    right: &ResolvedEntry,
    failure_tags: &mut Vec<String>,
) {
    if left.candidate.report.sample_rate_hz != right.candidate.report.sample_rate_hz {
        push_unique(failure_tags, "RELATION_SAMPLE_RATE_MISMATCH");
    }
}

pub(super) fn evaluate_coverage(
    manifest: &QualityManifest,
    relation_reports: &[RelationReport],
) -> CoverageReport {
    let mut missing_controls = Vec::new();
    if manifest.validator.is_none() {
        missing_controls.push("VALIDATOR_DECLARATION_MISSING".to_owned());
    } else if manifest
        .validator
        .as_ref()
        .is_some_and(|validator| validator.source_family != ADMITTED_SOURCE_FAMILY)
    {
        missing_controls.push("SOURCE_FAMILY_OUT_OF_DOMAIN".to_owned());
    }

    let mut relation_participants = BTreeSet::new();
    let mut relation_kinds = BTreeMap::<(&str, &str), BTreeSet<&str>>::new();
    for relation in &manifest.relations {
        let ids = relation.entry_ids();
        if let Some(first_id) = ids.first() {
            let entry = &manifest.entries[manifest
                .entries
                .binary_search_by(|entry| entry.id.as_str().cmp(first_id))
                .expect("validated relation entry exists")];
            relation_kinds
                .entry((&entry.object_id, &entry.material))
                .or_default()
                .insert(relation.kind());
        }
        relation_participants.extend(ids.into_iter().map(str::to_owned));
    }

    let mut pairs = BTreeSet::new();
    let mut zero_pairs = BTreeSet::new();
    let mut uncovered_impact_entries = Vec::new();
    for entry in &manifest.entries {
        let pair = (entry.object_id.as_str(), entry.material.as_str());
        pairs.insert(pair);
        if entry.control.is_none() {
            missing_controls.push(format!("{}:physical_control_values:MISSING", entry.id));
        }
        match entry.expected_signal {
            ExpectedSignal::Impact if !relation_participants.contains(&entry.id) => {
                uncovered_impact_entries.push(entry.id.clone());
            }
            ExpectedSignal::Silence => {
                zero_pairs.insert(pair);
            }
            ExpectedSignal::Impact => {}
        }
    }

    for (object_id, material) in pairs {
        let kinds = relation_kinds
            .get(&(object_id, material))
            .cloned()
            .unwrap_or_default();
        for required in ["exact_wav_repeat", "force_response", "position_continuity"] {
            if !kinds.contains(required) {
                missing_controls.push(format!("{object_id}/{material}:{required}:MISSING"));
            }
        }
        if !zero_pairs.contains(&(object_id, material)) {
            missing_controls.push(format!("{object_id}/{material}:zero_force_silence:MISSING"));
        }
    }
    if relation_reports.len() != manifest.relations.len() {
        missing_controls.push("RELATION_REPORT_COUNT_MISMATCH".to_owned());
    }
    let status = if missing_controls.is_empty() && uncovered_impact_entries.is_empty() {
        "Pass"
    } else {
        "FallbackOutOfDomain"
    };
    CoverageReport {
        status,
        admitted_source_family: ADMITTED_SOURCE_FAMILY,
        required_controls_per_object_material: [
            "zero_force_silence",
            "exact_wav_repeat",
            "force_response",
            "position_continuity",
        ],
        missing_controls,
        uncovered_impact_entries,
    }
}

pub(super) fn validator_decision(inputs: ValidatorDecisionInputs<'_>) -> &'static str {
    if inputs
        .entries
        .iter()
        .any(|entry| entry.decision == "Reject")
        || inputs
            .relations
            .iter()
            .any(|relation| relation.status == "Reject")
    {
        return "Reject";
    }
    if inputs.domain.is_none()
        || inputs.mutation_suite.status != "Pass"
        || inputs.coverage.status != "Pass"
        || inputs
            .entries
            .iter()
            .any(|entry| entry.decision == "FallbackOutOfDomain")
    {
        return "FallbackOutOfDomain";
    }
    "Pass"
}
