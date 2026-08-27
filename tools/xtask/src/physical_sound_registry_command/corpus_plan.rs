use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    FileRef, MAX_MANIFEST_BYTES, NumericRange, SourceClass, canonical_external_file,
    read_bounded_file, require_empty_output, resolve_cli_path, resolve_output_path, sha256_hex,
    validate_file_ref, validate_label, validate_range, validate_sorted_labels,
};

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-plan.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-plan.report.v1";
const MAX_DOMAINS: usize = 256;
const MAX_SOURCES_PER_DOMAIN: usize = 256;
const MAX_MUTATION_CLASSES: usize = 256;
const MAX_SAMPLE_SEARCH: usize = 1_000_000;
const WILSON_95_Z: f64 = 1.959_963_984_540_054;
const REQUIRED_GROUPING_KEYS: [&str; 5] = [
    "generator_revision",
    "mutation_parent_entry_id",
    "object_family_id",
    "object_id",
    "source_id",
];

pub(super) struct Request {
    manifest: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => super::set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => super::set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected corpus-plan argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry corpus-plan requires --manifest <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry corpus-plan requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CorpusPlanManifest {
    schema: String,
    plan_id: String,
    revision: String,
    source_class: SourceClass,
    research_protocol: FileRef,
    domains: Vec<AcquisitionDomain>,
    split_policy: SplitPolicy,
    risk_policy: RiskPolicy,
    mutation_policy: MutationPolicy,
    unavailable_component_policy: UnavailableComponentPolicy,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AcquisitionDomain {
    id: String,
    revision: String,
    material_family: String,
    object_family: String,
    geometry_family: String,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    geometry_scale_metres: NumericRange,
    relative_impact_speed_metres_per_second: NumericRange,
    impact_position_ids: Vec<String>,
    listener_condition_ids: Vec<String>,
    minimum_repeats_per_condition: usize,
    planned_independent_groups: PlannedIndependentGroups,
    candidate_sources: Vec<AcquisitionSource>,
    acquisition_protocol: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PlannedIndependentGroups {
    real_objects: usize,
    real_object_families: usize,
    acquisition_sources: usize,
    generator_revisions: usize,
    reject_mutation_parents: usize,
    in_domain_coverage_groups: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AcquisitionSource {
    id: String,
    revision: String,
    kind: AcquisitionSourceKind,
    exact_acquisition_manifest: FileRef,
    provenance_review: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum AcquisitionSourceKind {
    ControlledRealRecording,
    ProceduralGenerator,
}

impl AcquisitionSourceKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ControlledRealRecording => "controlled_real_recording",
            Self::ProceduralGenerator => "procedural_generator",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SplitPolicy {
    deterministic_seed: String,
    grouping_keys: Vec<String>,
    dev_basis_points: u16,
    calibration_basis_points: u16,
    holdout_basis_points: u16,
    shadow_basis_points: u16,
    threshold_selection_partition: Partition,
    shadow_sealed_before_threshold_selection: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Partition {
    Dev,
    Calibration,
    Holdout,
    Shadow,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RiskPolicy {
    confidence_level: f64,
    maximum_grouped_false_pass_risk: f64,
    unsafe_grouped_false_pass_risk: f64,
    minimum_detection_power: f64,
    minimum_useful_coverage_lower_bound: f64,
    admission_requires_zero_false_pass_parents: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MutationPolicy {
    class_ids: Vec<String>,
    minimum_severity_levels_per_class: usize,
    group_by_parent_before_scoring: bool,
    every_declared_mutation_must_reject: bool,
    severity_monotonicity: SeverityMonotonicity,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SeverityMonotonicity {
    MarginMustNotImproveWithSeverity,
}

impl SeverityMonotonicity {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MarginMustNotImproveWithSeverity => "margin_must_not_improve_with_severity",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UnavailableComponentPolicy {
    out_of_domain: UnavailableDecision,
    missing_embedding: UnavailableDecision,
    missing_reference: UnavailableDecision,
    specialist_disagreement: UnavailableDecision,
    unsupported_schema: UnavailableDecision,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum UnavailableDecision {
    FallbackOutOfDomain,
}

#[derive(Debug, Serialize)]
struct CorpusPlanReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    plan_id: String,
    revision: String,
    source_class: &'static str,
    manifest_sha256: String,
    research_protocol_sha256: String,
    split_policy: SplitPolicyReport,
    risk_power_analysis: RiskPowerAnalysis,
    mutation_policy: MutationPolicyReport,
    unavailable_component_decision: &'static str,
    domains: Vec<AcquisitionDomainReport>,
}

#[derive(Debug, Serialize)]
struct SplitPolicyReport {
    deterministic_seed: String,
    grouping_keys: Vec<String>,
    dev_basis_points: u16,
    calibration_basis_points: u16,
    holdout_basis_points: u16,
    shadow_basis_points: u16,
    threshold_selection_partition: &'static str,
    shadow_sealed_before_threshold_selection: bool,
}

#[derive(Debug, Serialize)]
struct RiskPowerAnalysis {
    method: &'static str,
    confidence_level: f64,
    maximum_grouped_false_pass_risk: f64,
    unsafe_grouped_false_pass_risk: f64,
    minimum_detection_power: f64,
    minimum_useful_coverage_lower_bound: f64,
    minimum_reject_parent_groups_for_wilson_risk: usize,
    minimum_reject_parent_groups_for_detection_power: usize,
    required_reject_parent_groups: usize,
    minimum_in_domain_groups_for_coverage: usize,
}

#[derive(Debug, Serialize)]
struct MutationPolicyReport {
    class_ids: Vec<String>,
    minimum_severity_levels_per_class: usize,
    group_by_parent_before_scoring: bool,
    every_declared_mutation_must_reject: bool,
    severity_monotonicity: &'static str,
}

#[derive(Debug, Serialize)]
struct AcquisitionDomainReport {
    id: String,
    revision: String,
    material_family: String,
    object_family: String,
    geometry_family: String,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    geometry_scale_metres: NumericRange,
    relative_impact_speed_metres_per_second: NumericRange,
    impact_position_ids: Vec<String>,
    listener_condition_ids: Vec<String>,
    minimum_repeats_per_condition: usize,
    planned_independent_groups: PlannedIndependentGroups,
    candidate_sources: Vec<AcquisitionSourceReport>,
    acquisition_protocol_sha256: String,
    planned_power_sufficient: bool,
}

#[derive(Debug, Serialize)]
struct AcquisitionSourceReport {
    id: String,
    revision: String,
    kind: &'static str,
    exact_acquisition_manifest_sha256: String,
    provenance_review_sha256: String,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path = canonical_external_file(&root, &manifest_path, "corpus plan manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "corpus plan")?;
    let manifest: CorpusPlanManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "corpus plan manifest has no parent directory".to_owned())?;
    let report = build_report(
        &root,
        manifest_directory,
        manifest,
        sha256_hex(&manifest_bytes),
    )?;
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;

    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write report.json: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn validate_manifest(manifest: &CorpusPlanManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound corpus plan schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.plan_id, "corpus plan id")?;
    validate_label(&manifest.revision, "corpus plan revision")?;
    validate_file_ref(&manifest.research_protocol, "research protocol")?;
    if manifest.domains.is_empty() || manifest.domains.len() > MAX_DOMAINS {
        return Err(format!(
            "acquisition domain count must be 1..={MAX_DOMAINS}"
        ));
    }
    validate_split_policy(&manifest.split_policy)?;
    validate_risk_policy(manifest.risk_policy)?;
    validate_mutation_policy(&manifest.mutation_policy)?;

    let mut previous_domain: Option<&str> = None;
    for domain in &manifest.domains {
        validate_label(&domain.id, "acquisition domain id")?;
        validate_label(&domain.revision, "acquisition domain revision")?;
        validate_label(&domain.material_family, "material family")?;
        validate_label(&domain.object_family, "object family")?;
        validate_label(&domain.geometry_family, "geometry family")?;
        validate_label(&domain.geometry_revision, "geometry revision")?;
        validate_label(&domain.support_condition, "support condition")?;
        validate_label(&domain.excitation_method, "excitation method")?;
        if previous_domain.is_some_and(|previous| previous >= domain.id.as_str()) {
            return Err(format!(
                "acquisition domains must be strictly sorted by id; offending id {}",
                domain.id
            ));
        }
        previous_domain = Some(&domain.id);
        validate_range(domain.geometry_scale_metres, "geometry_scale_metres", true)?;
        validate_range(
            domain.relative_impact_speed_metres_per_second,
            "relative_impact_speed_metres_per_second",
            false,
        )?;
        validate_sorted_labels(&domain.impact_position_ids, "impact position ids")?;
        validate_sorted_labels(&domain.listener_condition_ids, "listener condition ids")?;
        if domain.minimum_repeats_per_condition == 0
            || domain.minimum_repeats_per_condition > 100_000
        {
            return Err(format!(
                "domain {} minimum repeats per condition must be 1..=100000",
                domain.id
            ));
        }
        validate_planned_groups(domain)?;
        validate_file_ref(&domain.acquisition_protocol, "acquisition protocol")?;
        validate_sources(domain)?;
    }
    Ok(())
}

fn validate_split_policy(policy: &SplitPolicy) -> Result<(), String> {
    validate_label(&policy.deterministic_seed, "split deterministic seed")?;
    let expected = REQUIRED_GROUPING_KEYS.map(str::to_owned).to_vec();
    if policy.grouping_keys != expected {
        return Err(format!(
            "split grouping keys must be exactly {}",
            REQUIRED_GROUPING_KEYS.join(",")
        ));
    }
    let total = u32::from(policy.dev_basis_points)
        + u32::from(policy.calibration_basis_points)
        + u32::from(policy.holdout_basis_points)
        + u32::from(policy.shadow_basis_points);
    if total != 10_000
        || policy.dev_basis_points == 0
        || policy.calibration_basis_points == 0
        || policy.holdout_basis_points == 0
        || policy.shadow_basis_points == 0
    {
        return Err("split basis points must be non-zero and sum to 10000".to_owned());
    }
    if policy.threshold_selection_partition != Partition::Calibration {
        return Err("threshold selection partition must be calibration".to_owned());
    }
    if !policy.shadow_sealed_before_threshold_selection {
        return Err("shadow must be sealed before threshold selection".to_owned());
    }
    Ok(())
}

fn validate_risk_policy(policy: RiskPolicy) -> Result<(), String> {
    if policy.confidence_level != 0.95 {
        return Err("corpus plan v1 confidence level must be exactly 0.95".to_owned());
    }
    validate_open_probability(
        policy.maximum_grouped_false_pass_risk,
        "maximum grouped false-pass risk",
    )?;
    validate_open_probability(
        policy.unsafe_grouped_false_pass_risk,
        "unsafe grouped false-pass risk",
    )?;
    validate_open_probability(policy.minimum_detection_power, "minimum detection power")?;
    validate_open_probability(
        policy.minimum_useful_coverage_lower_bound,
        "minimum useful coverage lower bound",
    )?;
    if policy.unsafe_grouped_false_pass_risk <= policy.maximum_grouped_false_pass_risk {
        return Err("unsafe false-pass risk must exceed maximum admitted risk".to_owned());
    }
    if !policy.admission_requires_zero_false_pass_parents {
        return Err("corpus plan v1 admission must require zero false-pass parents".to_owned());
    }
    Ok(())
}

fn validate_open_probability(value: f64, role: &str) -> Result<(), String> {
    if !value.is_finite() || value <= 0.0 || value >= 1.0 {
        return Err(format!(
            "{role} must be finite and strictly between zero and one"
        ));
    }
    Ok(())
}

fn validate_mutation_policy(policy: &MutationPolicy) -> Result<(), String> {
    if policy.class_ids.is_empty() || policy.class_ids.len() > MAX_MUTATION_CLASSES {
        return Err(format!(
            "mutation class count must be 1..={MAX_MUTATION_CLASSES}"
        ));
    }
    validate_sorted_labels(&policy.class_ids, "mutation class ids")?;
    if policy.minimum_severity_levels_per_class < 2
        || policy.minimum_severity_levels_per_class > 256
    {
        return Err("minimum mutation severity levels per class must be 2..=256".to_owned());
    }
    if !policy.group_by_parent_before_scoring || !policy.every_declared_mutation_must_reject {
        return Err(
            "mutation policy must group by parent and require every mutation to reject".to_owned(),
        );
    }
    Ok(())
}

fn validate_planned_groups(domain: &AcquisitionDomain) -> Result<(), String> {
    let groups = domain.planned_independent_groups;
    if groups.real_objects < 4
        || groups.real_object_families < 4
        || groups.acquisition_sources < 4
        || groups.generator_revisions < 4
        || groups.reject_mutation_parents < 4
        || groups.in_domain_coverage_groups == 0
        || groups.real_objects > 1_000_000
        || groups.real_object_families > groups.real_objects
        || groups.acquisition_sources > 1_000_000
        || groups.generator_revisions > 1_000_000
        || groups.reject_mutation_parents > 1_000_000
        || groups.in_domain_coverage_groups > 1_000_000
    {
        return Err(format!(
            "domain {} has invalid planned independent group counts; every split axis needs at least four groups",
            domain.id
        ));
    }
    Ok(())
}

fn validate_sources(domain: &AcquisitionDomain) -> Result<(), String> {
    if domain.candidate_sources.is_empty()
        || domain.candidate_sources.len() > MAX_SOURCES_PER_DOMAIN
    {
        return Err(format!(
            "domain {} candidate source count must be 1..={MAX_SOURCES_PER_DOMAIN}",
            domain.id
        ));
    }
    if domain.planned_independent_groups.acquisition_sources < domain.candidate_sources.len() {
        return Err(format!(
            "domain {} plans fewer acquisition sources than it declares",
            domain.id
        ));
    }
    let mut previous_source: Option<&str> = None;
    let mut has_real = false;
    for source in &domain.candidate_sources {
        validate_label(&source.id, "acquisition source id")?;
        validate_label(&source.revision, "acquisition source revision")?;
        validate_file_ref(
            &source.exact_acquisition_manifest,
            "exact acquisition manifest",
        )?;
        validate_file_ref(&source.provenance_review, "source provenance review")?;
        if previous_source.is_some_and(|previous| previous >= source.id.as_str()) {
            return Err(format!(
                "domain {} candidate sources must be strictly sorted by id",
                domain.id
            ));
        }
        previous_source = Some(&source.id);
        has_real |= matches!(source.kind, AcquisitionSourceKind::ControlledRealRecording);
    }
    if !has_real {
        return Err(format!(
            "domain {} requires at least one controlled real recording source",
            domain.id
        ));
    }
    Ok(())
}

fn build_report(
    root: &Path,
    manifest_directory: &Path,
    manifest: CorpusPlanManifest,
    manifest_sha256: String,
) -> Result<CorpusPlanReport, String> {
    let research_protocol_sha256 = resolve_hash(
        root,
        manifest_directory,
        &manifest.research_protocol,
        "research protocol",
    )?;
    let risk_power_analysis = analyse_risk(manifest.risk_policy)?;
    let mut domains = Vec::with_capacity(manifest.domains.len());
    let mut all_planned_power_sufficient = true;
    for domain in manifest.domains {
        let planned_power_sufficient = domain.planned_independent_groups.reject_mutation_parents
            >= risk_power_analysis.required_reject_parent_groups
            && domain.planned_independent_groups.in_domain_coverage_groups
                >= risk_power_analysis.minimum_in_domain_groups_for_coverage;
        all_planned_power_sufficient &= planned_power_sufficient;
        let acquisition_protocol_sha256 = resolve_hash(
            root,
            manifest_directory,
            &domain.acquisition_protocol,
            "acquisition protocol",
        )?;
        let mut sources = Vec::with_capacity(domain.candidate_sources.len());
        for source in domain.candidate_sources {
            sources.push(AcquisitionSourceReport {
                id: source.id,
                revision: source.revision,
                kind: source.kind.as_str(),
                exact_acquisition_manifest_sha256: resolve_hash(
                    root,
                    manifest_directory,
                    &source.exact_acquisition_manifest,
                    "exact acquisition manifest",
                )?,
                provenance_review_sha256: resolve_hash(
                    root,
                    manifest_directory,
                    &source.provenance_review,
                    "source provenance review",
                )?,
            });
        }
        domains.push(AcquisitionDomainReport {
            id: domain.id,
            revision: domain.revision,
            material_family: domain.material_family,
            object_family: domain.object_family,
            geometry_family: domain.geometry_family,
            geometry_revision: domain.geometry_revision,
            support_condition: domain.support_condition,
            excitation_method: domain.excitation_method,
            geometry_scale_metres: domain.geometry_scale_metres,
            relative_impact_speed_metres_per_second: domain.relative_impact_speed_metres_per_second,
            impact_position_ids: domain.impact_position_ids,
            listener_condition_ids: domain.listener_condition_ids,
            minimum_repeats_per_condition: domain.minimum_repeats_per_condition,
            planned_independent_groups: domain.planned_independent_groups,
            candidate_sources: sources,
            acquisition_protocol_sha256,
            planned_power_sufficient,
        });
    }
    let decision = if all_planned_power_sufficient {
        "PlanPowerSufficient"
    } else {
        "PlanPowerInsufficient"
    };

    Ok(CorpusPlanReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision,
        claim: "PREREGISTRATION_AND_POWER_PLAN_ONLY / NO_CORPUS_ADMISSION_AUTHORITY",
        plan_id: manifest.plan_id,
        revision: manifest.revision,
        source_class: manifest.source_class.as_str(),
        manifest_sha256,
        research_protocol_sha256,
        split_policy: SplitPolicyReport {
            deterministic_seed: manifest.split_policy.deterministic_seed,
            grouping_keys: manifest.split_policy.grouping_keys,
            dev_basis_points: manifest.split_policy.dev_basis_points,
            calibration_basis_points: manifest.split_policy.calibration_basis_points,
            holdout_basis_points: manifest.split_policy.holdout_basis_points,
            shadow_basis_points: manifest.split_policy.shadow_basis_points,
            threshold_selection_partition: "calibration",
            shadow_sealed_before_threshold_selection: manifest
                .split_policy
                .shadow_sealed_before_threshold_selection,
        },
        risk_power_analysis,
        mutation_policy: MutationPolicyReport {
            class_ids: manifest.mutation_policy.class_ids,
            minimum_severity_levels_per_class: manifest
                .mutation_policy
                .minimum_severity_levels_per_class,
            group_by_parent_before_scoring: manifest.mutation_policy.group_by_parent_before_scoring,
            every_declared_mutation_must_reject: manifest
                .mutation_policy
                .every_declared_mutation_must_reject,
            severity_monotonicity: manifest.mutation_policy.severity_monotonicity.as_str(),
        },
        unavailable_component_decision: "FallbackOutOfDomain",
        domains,
    })
}

fn resolve_hash(
    root: &Path,
    manifest_directory: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<String, String> {
    Ok(super::resolve_artifact(root, manifest_directory, reference, role)?.sha256)
}

fn analyse_risk(policy: RiskPolicy) -> Result<RiskPowerAnalysis, String> {
    let minimum_reject_parent_groups_for_wilson_risk = (1..=MAX_SAMPLE_SEARCH)
        .find(|trials| wilson_interval(0, *trials).1 <= policy.maximum_grouped_false_pass_risk)
        .ok_or_else(|| "false-pass Wilson target exceeds bounded sample search".to_owned())?;
    let minimum_reject_parent_groups_for_detection_power = (1..=MAX_SAMPLE_SEARCH)
        .find(|trials| {
            1.0 - (1.0 - policy.unsafe_grouped_false_pass_risk).powf(*trials as f64)
                >= policy.minimum_detection_power
        })
        .ok_or_else(|| "false-pass detection-power target exceeds bounded search".to_owned())?;
    let minimum_in_domain_groups_for_coverage = (1..=MAX_SAMPLE_SEARCH)
        .find(|trials| {
            wilson_interval(*trials, *trials).0 >= policy.minimum_useful_coverage_lower_bound
        })
        .ok_or_else(|| "coverage Wilson target exceeds bounded sample search".to_owned())?;

    Ok(RiskPowerAnalysis {
        method: "wilson_score_95_and_exact_zero_failure_detection",
        confidence_level: policy.confidence_level,
        maximum_grouped_false_pass_risk: policy.maximum_grouped_false_pass_risk,
        unsafe_grouped_false_pass_risk: policy.unsafe_grouped_false_pass_risk,
        minimum_detection_power: policy.minimum_detection_power,
        minimum_useful_coverage_lower_bound: policy.minimum_useful_coverage_lower_bound,
        minimum_reject_parent_groups_for_wilson_risk,
        minimum_reject_parent_groups_for_detection_power,
        required_reject_parent_groups: minimum_reject_parent_groups_for_wilson_risk
            .max(minimum_reject_parent_groups_for_detection_power),
        minimum_in_domain_groups_for_coverage,
    })
}

fn wilson_interval(successes: usize, trials: usize) -> (f64, f64) {
    let trials = trials as f64;
    let probability = successes as f64 / trials;
    let z_squared = WILSON_95_Z * WILSON_95_Z;
    let denominator = 1.0 + z_squared / trials;
    let centre = probability + z_squared / (2.0 * trials);
    let radius = WILSON_95_Z
        * ((probability * (1.0 - probability) / trials) + z_squared / (4.0 * trials * trials))
            .sqrt();
    (
        ((centre - radius) / denominator).max(0.0),
        ((centre + radius) / denominator).min(1.0),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::Value;

    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nextengine-physical-sound-corpus-plan-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create corpus plan test directory");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if self.path.is_dir() {
                fs::remove_dir_all(&self.path).expect("remove corpus plan test directory");
            }
        }
    }

    #[test]
    fn risk_analysis_has_stable_known_sample_sizes() {
        let analysis = analyse_risk(test_manifest().risk_policy).expect("analyse risk");
        assert_eq!(analysis.minimum_reject_parent_groups_for_wilson_risk, 35);
        assert_eq!(
            analysis.minimum_reject_parent_groups_for_detection_power,
            14
        );
        assert_eq!(analysis.required_reject_parent_groups, 35);
        assert_eq!(analysis.minimum_in_domain_groups_for_coverage, 16);
    }

    #[test]
    fn plan_rejects_split_leakage_and_missing_real_source() {
        let mut manifest = test_manifest();
        manifest.split_policy.threshold_selection_partition = Partition::Shadow;
        assert!(
            validate_manifest(&manifest)
                .expect_err("shadow selection rejects")
                .contains("must be calibration")
        );

        let mut manifest = test_manifest();
        manifest.domains[0].candidate_sources[0].kind = AcquisitionSourceKind::ProceduralGenerator;
        assert!(
            validate_manifest(&manifest)
                .expect_err("missing real source rejects")
                .contains("controlled real recording")
        );
    }

    #[test]
    fn end_to_end_plan_is_external_hash_closed_and_repeatable() {
        let directory = TestDirectory::new();
        let mut manifest = test_manifest();
        write_artifacts(&directory.path, &mut manifest);
        let manifest_path = directory.path.join("plan.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize plan"),
        )
        .expect("write plan");
        let first = directory.path.join("first");
        run(
            workspace_root(),
            &Request {
                manifest: manifest_path.clone(),
                output: first.clone(),
            },
        )
        .expect("plan validates");
        let report: Value = serde_json::from_slice(
            &fs::read(first.join("report.json")).expect("read first report"),
        )
        .expect("parse report");
        assert_eq!(report["decision"], "PlanPowerSufficient");
        assert_eq!(report["domains"][0]["planned_power_sufficient"], true);
        assert_eq!(
            report["risk_power_analysis"]["required_reject_parent_groups"],
            35
        );

        let repeated = directory.path.join("repeated");
        run(
            workspace_root(),
            &Request {
                manifest: manifest_path,
                output: repeated.clone(),
            },
        )
        .expect("plan repeats");
        assert_eq!(
            fs::read(first.join("report.json")).expect("read first report"),
            fs::read(repeated.join("report.json")).expect("read repeated report")
        );

        manifest.domains[0]
            .planned_independent_groups
            .reject_mutation_parents = 20;
        manifest.domains[0]
            .planned_independent_groups
            .in_domain_coverage_groups = 10;
        let insufficient_path = directory.path.join("insufficient.json");
        fs::write(
            &insufficient_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize insufficient plan"),
        )
        .expect("write insufficient plan");
        let insufficient_output = directory.path.join("insufficient-report");
        run(
            workspace_root(),
            &Request {
                manifest: insufficient_path,
                output: insufficient_output.clone(),
            },
        )
        .expect("underpowered plan remains reviewable");
        let insufficient: Value = serde_json::from_slice(
            &fs::read(insufficient_output.join("report.json")).expect("read insufficient report"),
        )
        .expect("parse insufficient report");
        assert_eq!(insufficient["decision"], "PlanPowerInsufficient");
        assert_eq!(
            insufficient["domains"][0]["planned_power_sufficient"],
            false
        );
    }

    #[test]
    fn repository_local_protocol_is_rejected() {
        let directory = TestDirectory::new();
        let mut manifest = test_manifest();
        write_artifacts(&directory.path, &mut manifest);
        let repository_file = workspace_root().join("Cargo.toml");
        manifest.research_protocol = FileRef {
            path: repository_file.display().to_string(),
            sha256: sha256_hex(&fs::read(&repository_file).expect("read Cargo.toml")),
        };
        let manifest_path = directory.path.join("repository-local.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize plan"),
        )
        .expect("write plan");
        assert!(
            run(
                workspace_root(),
                &Request {
                    manifest: manifest_path,
                    output: directory.path.join("report"),
                },
            )
            .expect_err("repository-local protocol rejects")
            .contains("must stay outside the repository")
        );
    }

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
    }

    fn write_artifacts(directory: &Path, manifest: &mut CorpusPlanManifest) {
        let artifacts = [
            ("research.md", b"frozen research protocol".as_slice()),
            (
                "acquisition.json",
                b"{\"source\":\"controlled\"}".as_slice(),
            ),
            ("provenance.md", b"source review".as_slice()),
        ];
        for (name, bytes) in artifacts {
            fs::write(directory.join(name), bytes).expect("write corpus plan artifact");
        }
        manifest.research_protocol.sha256 =
            sha256_hex(&fs::read(directory.join("research.md")).expect("read research protocol"));
        manifest.domains[0].acquisition_protocol.sha256 = manifest.research_protocol.sha256.clone();
        manifest.domains[0].candidate_sources[0]
            .exact_acquisition_manifest
            .sha256 = sha256_hex(
            &fs::read(directory.join("acquisition.json")).expect("read acquisition manifest"),
        );
        manifest.domains[0].candidate_sources[0]
            .provenance_review
            .sha256 =
            sha256_hex(&fs::read(directory.join("provenance.md")).expect("read provenance review"));
    }

    fn test_manifest() -> CorpusPlanManifest {
        CorpusPlanManifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            plan_id: "physical-sound-ps2".to_owned(),
            revision: "v1".to_owned(),
            source_class: SourceClass::RigidImpact,
            research_protocol: file_ref("research.md"),
            domains: vec![AcquisitionDomain {
                id: "thin-glass-vessel-impact".to_owned(),
                revision: "v1".to_owned(),
                material_family: "soda-lime-glass".to_owned(),
                object_family: "thin-wall-vessel".to_owned(),
                geometry_family: "axisymmetric-open-shell".to_owned(),
                geometry_revision: "measured-mesh-v1".to_owned(),
                support_condition: "freely-supported-at-base".to_owned(),
                excitation_method: "instrumented-impact-hammer".to_owned(),
                geometry_scale_metres: NumericRange {
                    minimum: 0.05,
                    maximum: 0.5,
                },
                relative_impact_speed_metres_per_second: NumericRange {
                    minimum: 0.1,
                    maximum: 5.0,
                },
                impact_position_ids: vec!["rim".to_owned(), "wall-midpoint".to_owned()],
                listener_condition_ids: vec![
                    "near-field-axis".to_owned(),
                    "near-field-radial".to_owned(),
                ],
                minimum_repeats_per_condition: 3,
                planned_independent_groups: PlannedIndependentGroups {
                    real_objects: 40,
                    real_object_families: 4,
                    acquisition_sources: 4,
                    generator_revisions: 4,
                    reject_mutation_parents: 40,
                    in_domain_coverage_groups: 40,
                },
                candidate_sources: vec![AcquisitionSource {
                    id: "controlled-lab-v1".to_owned(),
                    revision: "v1".to_owned(),
                    kind: AcquisitionSourceKind::ControlledRealRecording,
                    exact_acquisition_manifest: file_ref("acquisition.json"),
                    provenance_review: file_ref("provenance.md"),
                }],
                acquisition_protocol: file_ref("research.md"),
            }],
            split_policy: SplitPolicy {
                deterministic_seed: "physical-sound-ps2-v1".to_owned(),
                grouping_keys: REQUIRED_GROUPING_KEYS.map(str::to_owned).to_vec(),
                dev_basis_points: 2_000,
                calibration_basis_points: 3_000,
                holdout_basis_points: 2_500,
                shadow_basis_points: 2_500,
                threshold_selection_partition: Partition::Calibration,
                shadow_sealed_before_threshold_selection: true,
            },
            risk_policy: RiskPolicy {
                confidence_level: 0.95,
                maximum_grouped_false_pass_risk: 0.1,
                unsafe_grouped_false_pass_risk: 0.2,
                minimum_detection_power: 0.95,
                minimum_useful_coverage_lower_bound: 0.8,
                admission_requires_zero_false_pass_parents: true,
            },
            mutation_policy: MutationPolicy {
                class_ids: vec!["envelope".to_owned(), "spectral".to_owned()],
                minimum_severity_levels_per_class: 2,
                group_by_parent_before_scoring: true,
                every_declared_mutation_must_reject: true,
                severity_monotonicity: SeverityMonotonicity::MarginMustNotImproveWithSeverity,
            },
            unavailable_component_policy: UnavailableComponentPolicy {
                out_of_domain: UnavailableDecision::FallbackOutOfDomain,
                missing_embedding: UnavailableDecision::FallbackOutOfDomain,
                missing_reference: UnavailableDecision::FallbackOutOfDomain,
                specialist_disagreement: UnavailableDecision::FallbackOutOfDomain,
                unsupported_schema: UnavailableDecision::FallbackOutOfDomain,
            },
        }
    }

    fn file_ref(path: &str) -> FileRef {
        FileRef {
            path: path.to_owned(),
            sha256: "a".repeat(64),
        }
    }
}
