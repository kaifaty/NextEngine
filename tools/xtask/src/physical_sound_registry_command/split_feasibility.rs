use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_cli_path, resolve_output_path, set_once, sha256_hex,
};

const IDENTIFIED_REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-identified-corpus.report.v1";
const PLAN_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-plan.report.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-split-feasibility.report.v1";
const REPORT_CLAIM: &str = "PROJECT_DISJOINT_SPLIT_FEASIBILITY_AUDIT_ONLY / NO_PARTITION_FREEZE_OR_CORPUS_ADMISSION_AUTHORITY";
const PARTITIONS: [&str; 4] = ["dev", "calibration", "holdout", "shadow"];

struct Request {
    identified_report: PathBuf,
    corpus_plan_report: PathBuf,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut identified_report = None;
    let mut corpus_plan_report = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--identified-report" => set_once(&mut identified_report, PathBuf::from(value), &flag)?,
            "--corpus-plan-report" => {
                set_once(&mut corpus_plan_report, PathBuf::from(value), &flag)?
            }
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected split-feasibility argument: {flag}")),
        }
    }
    Ok(Request {
        identified_report: identified_report.ok_or_else(|| {
            "physical-sound-registry split-feasibility requires --identified-report <external-json>"
                .to_owned()
        })?,
        corpus_plan_report: corpus_plan_report.ok_or_else(|| {
            "physical-sound-registry split-feasibility requires --corpus-plan-report <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry split-feasibility requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let identified_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.identified_report),
        "identified corpus report",
    )?;
    let plan_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.corpus_plan_report),
        "corpus plan report",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let identified_bytes = read_bounded_file(
        &identified_path,
        MAX_MANIFEST_BYTES,
        "identified corpus report",
    )?;
    let identified: IdentifiedReport = serde_json::from_slice(&identified_bytes)
        .map_err(|error| format!("parse {}: {error}", identified_path.display()))?;
    let plan_bytes = read_bounded_file(&plan_path, MAX_MANIFEST_BYTES, "corpus plan report")?;
    let plan: CorpusPlanReport = serde_json::from_slice(&plan_bytes)
        .map_err(|error| format!("parse {}: {error}", plan_path.display()))?;
    let plan_sha256 = sha256_hex(&plan_bytes);
    validate_inputs(&identified, &plan, &plan_sha256)?;

    let report = build_report(
        identified,
        &plan,
        sha256_hex(&identified_bytes),
        plan_sha256,
    )?;
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write split feasibility report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[derive(Deserialize)]
struct IdentifiedReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    target_material_label: String,
    corpus_plan_report_sha256: String,
    recording_count: usize,
    independent_group_counts: IndependentGroupCounts,
    partition_counts: Vec<PartitionCount>,
    coverage_against_plan: CoverageAgainstPlan,
    entries: Vec<IdentifiedEntry>,
}

#[derive(Deserialize)]
struct IndependentGroupCounts {
    project_revisions: usize,
    objects: usize,
    recordings: usize,
}

#[derive(Deserialize)]
struct PartitionCount {
    partition: String,
    objects: usize,
    recordings: usize,
}

#[derive(Deserialize)]
struct CoverageAgainstPlan {
    target_object_groups: usize,
    minimum_in_domain_groups_for_coverage: usize,
    target_in_domain_coverage_sufficient: bool,
    reject_parent_groups_in_this_e3_corpus: usize,
    reject_parent_coverage_sufficient: bool,
    required_reject_parent_groups: usize,
}

#[derive(Deserialize)]
struct IdentifiedEntry {
    partition: String,
    corpus_role: CorpusRole,
    evidence_tier: String,
    source_group_id: String,
    object_group_id: String,
    material_label: String,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum CorpusRole {
    Target,
    RejectParent,
}

#[derive(Deserialize)]
struct CorpusPlanReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    split_policy: SplitPolicy,
    risk_power_analysis: RiskPowerAnalysis,
}

#[derive(Deserialize)]
struct SplitPolicy {
    dev_basis_points: u16,
    calibration_basis_points: u16,
    holdout_basis_points: u16,
    shadow_basis_points: u16,
    threshold_selection_partition: String,
    shadow_sealed_before_threshold_selection: bool,
}

#[derive(Deserialize)]
struct RiskPowerAnalysis {
    required_reject_parent_groups: usize,
    minimum_in_domain_groups_for_coverage: usize,
}

fn validate_inputs(
    identified: &IdentifiedReport,
    plan: &CorpusPlanReport,
    plan_sha256: &str,
) -> Result<(), String> {
    if identified.schema != IDENTIFIED_REPORT_SCHEMA
        || identified.status != "Validated"
        || identified.decision != "DevelopmentCoverageMeasured"
        || identified.claim != "GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
    {
        return Err("identified corpus report is not the supported development audit".to_owned());
    }
    if plan.schema != PLAN_REPORT_SCHEMA
        || plan.status != "Validated"
        || plan.decision != "PlanPowerSufficient"
        || plan.claim != "PREREGISTRATION_AND_POWER_PLAN_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || plan.split_policy.threshold_selection_partition != "calibration"
        || !plan.split_policy.shadow_sealed_before_threshold_selection
    {
        return Err("corpus plan report is not the supported frozen split policy".to_owned());
    }
    if identified.corpus_plan_report_sha256 != plan_sha256 {
        return Err("identified corpus and corpus plan report hashes do not match".to_owned());
    }
    let basis_points = plan.split_policy.basis_points();
    if basis_points.contains(&0)
        || basis_points
            .iter()
            .map(|points| u32::from(*points))
            .sum::<u32>()
            != 10_000
    {
        return Err(
            "corpus plan split basis points must be four positive shares summing to 10000"
                .to_owned(),
        );
    }
    if identified
        .coverage_against_plan
        .required_reject_parent_groups
        != plan.risk_power_analysis.required_reject_parent_groups
        || identified
            .coverage_against_plan
            .minimum_in_domain_groups_for_coverage
            != plan
                .risk_power_analysis
                .minimum_in_domain_groups_for_coverage
        || !identified
            .coverage_against_plan
            .target_in_domain_coverage_sufficient
        || !identified
            .coverage_against_plan
            .reject_parent_coverage_sufficient
    {
        return Err(
            "identified corpus has not closed the frozen aggregate coverage plan".to_owned(),
        );
    }
    if identified.partition_counts.len() != PARTITIONS.len()
        || identified
            .partition_counts
            .iter()
            .zip(PARTITIONS)
            .any(|(count, partition)| {
                count.partition != partition
                    || (partition == "dev"
                        && (count.objects != identified.independent_group_counts.objects
                            || count.recordings != identified.recording_count))
                    || (partition != "dev" && (count.objects != 0 || count.recordings != 0))
            })
    {
        return Err(
            "split feasibility requires the complete pre-split corpus to remain in dev".to_owned(),
        );
    }
    if identified.entries.len() != identified.recording_count
        || identified.independent_group_counts.recordings != identified.recording_count
        || identified.entries.iter().any(|entry| {
            entry.partition != "dev"
                || entry.evidence_tier != "E3IdentifiedRecording"
                || entry.source_group_id.is_empty()
                || entry.object_group_id.is_empty()
        })
    {
        return Err(
            "identified corpus entries do not form a complete dev-only E3 audit".to_owned(),
        );
    }
    Ok(())
}

impl SplitPolicy {
    const fn basis_points(&self) -> [u16; 4] {
        [
            self.dev_basis_points,
            self.calibration_basis_points,
            self.holdout_basis_points,
            self.shadow_basis_points,
        ]
    }
}

#[derive(Default)]
struct MutableProjectGroup {
    target_objects: BTreeSet<String>,
    reject_parent_objects: BTreeSet<String>,
    target_recordings: usize,
    reject_parent_recordings: usize,
}

#[derive(Serialize)]
struct SplitFeasibilityReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    identified_corpus_report_sha256: String,
    corpus_plan_report_sha256: String,
    project_group_count: usize,
    target_object_group_count: usize,
    reject_parent_object_group_count: usize,
    target_bearing_project_groups: usize,
    reject_parent_bearing_project_groups: usize,
    dual_role_project_groups: usize,
    target_only_project_groups: usize,
    reject_parent_only_project_groups: usize,
    required_roles_per_partition: [&'static str; 2],
    planned_partition_project_counts: Vec<PartitionProjectCount>,
    blockers: Vec<String>,
    project_groups: Vec<ProjectGroupReport>,
}

#[derive(Serialize)]
struct PartitionProjectCount {
    partition: &'static str,
    basis_points: u16,
    project_groups: usize,
}

#[derive(Serialize)]
struct ProjectGroupReport {
    source_group_id: String,
    target_objects: usize,
    reject_parent_objects: usize,
    target_recordings: usize,
    reject_parent_recordings: usize,
}

fn build_report(
    identified: IdentifiedReport,
    plan: &CorpusPlanReport,
    identified_sha256: String,
    plan_sha256: String,
) -> Result<SplitFeasibilityReport, String> {
    let mut projects = BTreeMap::<String, MutableProjectGroup>::new();
    let mut object_roles = BTreeMap::<String, (String, CorpusRole)>::new();
    for entry in &identified.entries {
        if (entry.corpus_role == CorpusRole::Target)
            != (entry.material_label == identified.target_material_label)
        {
            return Err(format!(
                "object {} has a material/role mismatch",
                entry.object_group_id
            ));
        }
        let identity = (entry.source_group_id.clone(), entry.corpus_role);
        if let Some(previous) = object_roles.insert(entry.object_group_id.clone(), identity.clone())
            && previous != identity
        {
            return Err(format!(
                "object {} changes project group or corpus role",
                entry.object_group_id
            ));
        }
        let project = projects.entry(entry.source_group_id.clone()).or_default();
        match entry.corpus_role {
            CorpusRole::Target => {
                project.target_objects.insert(entry.object_group_id.clone());
                project.target_recordings += 1;
            }
            CorpusRole::RejectParent => {
                project
                    .reject_parent_objects
                    .insert(entry.object_group_id.clone());
                project.reject_parent_recordings += 1;
            }
        }
    }
    let target_objects = object_roles
        .values()
        .filter(|(_, role)| *role == CorpusRole::Target)
        .count();
    let reject_objects = object_roles.len() - target_objects;
    if projects.len() != identified.independent_group_counts.project_revisions
        || object_roles.len() != identified.independent_group_counts.objects
        || target_objects != identified.coverage_against_plan.target_object_groups
        || reject_objects
            != identified
                .coverage_against_plan
                .reject_parent_groups_in_this_e3_corpus
    {
        return Err("identified report aggregate counts do not match its entries".to_owned());
    }

    let target_bearing = projects
        .values()
        .filter(|project| !project.target_objects.is_empty())
        .count();
    let reject_bearing = projects
        .values()
        .filter(|project| !project.reject_parent_objects.is_empty())
        .count();
    let dual_role = projects
        .values()
        .filter(|project| {
            !project.target_objects.is_empty() && !project.reject_parent_objects.is_empty()
        })
        .count();
    let target_only = target_bearing - dual_role;
    let reject_only = reject_bearing - dual_role;
    let partition_counts =
        allocate_partition_counts(projects.len(), plan.split_policy.basis_points());
    let mut blockers = split_blockers(target_bearing, reject_bearing, dual_role, &partition_counts);
    if blockers.is_empty() && target_only + reject_only + dual_role != projects.len() {
        blockers.push(
            "at least one project group has no explicit target or reject-parent role".to_owned(),
        );
    }
    let decision = if blockers.is_empty() {
        "ProjectDisjointSplitFeasible"
    } else {
        "ProjectDisjointSplitInfeasible"
    };
    Ok(SplitFeasibilityReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision,
        claim: REPORT_CLAIM,
        identified_corpus_report_sha256: identified_sha256,
        corpus_plan_report_sha256: plan_sha256,
        project_group_count: projects.len(),
        target_object_group_count: target_objects,
        reject_parent_object_group_count: reject_objects,
        target_bearing_project_groups: target_bearing,
        reject_parent_bearing_project_groups: reject_bearing,
        dual_role_project_groups: dual_role,
        target_only_project_groups: target_only,
        reject_parent_only_project_groups: reject_only,
        required_roles_per_partition: ["target", "reject_parent"],
        planned_partition_project_counts: PARTITIONS
            .into_iter()
            .zip(plan.split_policy.basis_points())
            .zip(partition_counts)
            .map(
                |((partition, basis_points), project_groups)| PartitionProjectCount {
                    partition,
                    basis_points,
                    project_groups,
                },
            )
            .collect(),
        blockers,
        project_groups: projects
            .into_iter()
            .map(|(source_group_id, project)| ProjectGroupReport {
                source_group_id,
                target_objects: project.target_objects.len(),
                reject_parent_objects: project.reject_parent_objects.len(),
                target_recordings: project.target_recordings,
                reject_parent_recordings: project.reject_parent_recordings,
            })
            .collect(),
    })
}

fn allocate_partition_counts(total: usize, basis_points: [u16; 4]) -> [usize; 4] {
    let mut counts = [0_usize; 4];
    let mut remainders = [(0_u16, 0_usize); 4];
    for (index, points) in basis_points.into_iter().enumerate() {
        let product = total * usize::from(points);
        counts[index] = product / 10_000;
        remainders[index] = ((product % 10_000) as u16, index);
    }
    remainders.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    for (_, index) in remainders
        .into_iter()
        .take(total - counts.iter().sum::<usize>())
    {
        counts[index] += 1;
    }
    counts
}

fn split_blockers(
    target_bearing: usize,
    reject_bearing: usize,
    dual_role: usize,
    partition_counts: &[usize; 4],
) -> Vec<String> {
    let required_partitions = partition_counts.len();
    let mut blockers = Vec::new();
    if partition_counts.contains(&0) {
        blockers.push(format!(
            "planned project counts {partition_counts:?} leave at least one partition empty"
        ));
    }
    if target_bearing < required_partitions {
        blockers.push(format!(
            "target-bearing project groups {target_bearing} < required partitions {required_partitions}"
        ));
    }
    if reject_bearing < required_partitions {
        blockers.push(format!(
            "reject-parent-bearing project groups {reject_bearing} < required partitions {required_partitions}"
        ));
    }
    let single_capacity_partitions = partition_counts.iter().filter(|count| **count == 1).count();
    if dual_role < single_capacity_partitions {
        blockers.push(format!(
            "dual-role project groups {dual_role} < single-capacity partitions {single_capacity_partitions}"
        ));
    }
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_remainder_allocation_is_stable() {
        assert_eq!(
            allocate_partition_counts(8, [2_000, 3_000, 2_500, 2_500]),
            [2, 2, 2, 2]
        );
        assert_eq!(
            allocate_partition_counts(7, [2_000, 3_000, 2_500, 2_500]),
            [1, 2, 2, 2]
        );
    }

    #[test]
    fn three_reject_projects_cannot_cover_four_partitions() {
        let blockers = split_blockers(8, 3, 3, &[2, 2, 2, 2]);
        assert_eq!(
            blockers,
            ["reject-parent-bearing project groups 3 < required partitions 4"]
        );
    }

    #[test]
    fn four_dual_role_projects_cover_four_single_capacity_partitions() {
        assert!(split_blockers(4, 4, 4, &[1, 1, 1, 1]).is_empty());
        assert!(!split_blockers(4, 4, 3, &[1, 1, 1, 1]).is_empty());
    }
}
