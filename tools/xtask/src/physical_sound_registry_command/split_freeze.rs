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
const FEASIBILITY_REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-split-feasibility.report.v1";
const PLAN_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-plan.report.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-split-freeze.report.v1";
const REPORT_CLAIM: &str =
    "DETERMINISTIC_PROJECT_PARTITION_FREEZE_ONLY / NO_CORPUS_ADMISSION_OR_QUALITY_AUTHORITY";
const ASSIGNMENT_ALGORITHM: &str = "seeded_sha256_first_feasible_backtracking_v1";
const PARTITIONS: [&str; 4] = ["dev", "calibration", "holdout", "shadow"];
const MAX_PROJECT_GROUPS: usize = 64;
const MAX_SEARCH_NODES: usize = 1_000_000;

struct Request {
    identified_report: PathBuf,
    feasibility_report: PathBuf,
    corpus_plan_report: PathBuf,
    partitioned_report: Option<PathBuf>,
    output: PathBuf,
}

pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let request = parse_arguments(arguments)?;
    run(root, &request)
}

fn parse_arguments(mut arguments: impl Iterator<Item = String>) -> Result<Request, String> {
    let mut identified_report = None;
    let mut feasibility_report = None;
    let mut corpus_plan_report = None;
    let mut partitioned_report = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--identified-report" => set_once(&mut identified_report, PathBuf::from(value), &flag)?,
            "--feasibility-report" => {
                set_once(&mut feasibility_report, PathBuf::from(value), &flag)?
            }
            "--corpus-plan-report" => {
                set_once(&mut corpus_plan_report, PathBuf::from(value), &flag)?
            }
            "--partitioned-report" => {
                set_once(&mut partitioned_report, PathBuf::from(value), &flag)?
            }
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected split-freeze argument: {flag}")),
        }
    }
    Ok(Request {
        identified_report: identified_report.ok_or_else(|| {
            "physical-sound-registry split-freeze requires --identified-report <external-json>"
                .to_owned()
        })?,
        feasibility_report: feasibility_report.ok_or_else(|| {
            "physical-sound-registry split-freeze requires --feasibility-report <external-json>"
                .to_owned()
        })?,
        corpus_plan_report: corpus_plan_report.ok_or_else(|| {
            "physical-sound-registry split-freeze requires --corpus-plan-report <external-json>"
                .to_owned()
        })?,
        partitioned_report,
        output: output.ok_or_else(|| {
            "physical-sound-registry split-freeze requires --output <external-empty-directory>"
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
    let feasibility_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.feasibility_report),
        "split feasibility report",
    )?;
    let plan_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.corpus_plan_report),
        "corpus plan report",
    )?;
    let partitioned_path = request
        .partitioned_report
        .as_ref()
        .map(|path| {
            canonical_external_file(
                &root,
                &resolve_cli_path(&root, path),
                "partitioned identified corpus report",
            )
        })
        .transpose()?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let identified_bytes = read_bounded_file(
        &identified_path,
        MAX_MANIFEST_BYTES,
        "identified corpus report",
    )?;
    let feasibility_bytes = read_bounded_file(
        &feasibility_path,
        MAX_MANIFEST_BYTES,
        "split feasibility report",
    )?;
    let plan_bytes = read_bounded_file(&plan_path, MAX_MANIFEST_BYTES, "corpus plan report")?;
    let identified: IdentifiedReport = serde_json::from_slice(&identified_bytes)
        .map_err(|error| format!("parse {}: {error}", identified_path.display()))?;
    let feasibility: FeasibilityReport = serde_json::from_slice(&feasibility_bytes)
        .map_err(|error| format!("parse {}: {error}", feasibility_path.display()))?;
    let plan: CorpusPlanReport = serde_json::from_slice(&plan_bytes)
        .map_err(|error| format!("parse {}: {error}", plan_path.display()))?;
    let identified_sha256 = sha256_hex(&identified_bytes);
    let feasibility_sha256 = sha256_hex(&feasibility_bytes);
    let plan_sha256 = sha256_hex(&plan_bytes);
    let projects = validate_inputs(
        &identified,
        &feasibility,
        &plan,
        &identified_sha256,
        &plan_sha256,
    )?;
    let capacities = feasibility.partition_capacities()?;
    let assignment =
        solve_assignment(&projects, capacities, &plan.split_policy.deterministic_seed)?;
    let partitioned_sha256 = if let Some(partitioned_path) = partitioned_path {
        let partitioned_bytes = read_bounded_file(
            &partitioned_path,
            MAX_MANIFEST_BYTES,
            "partitioned identified corpus report",
        )?;
        let partitioned: IdentifiedReport = serde_json::from_slice(&partitioned_bytes)
            .map_err(|error| format!("parse {}: {error}", partitioned_path.display()))?;
        validate_partitioned_report(&identified, &partitioned, &projects, &assignment)?;
        Some(sha256_hex(&partitioned_bytes))
    } else {
        None
    };
    let report = build_report(
        &projects,
        assignment,
        capacities,
        &plan.split_policy.deterministic_seed,
        ReportHashes {
            identified: identified_sha256,
            feasibility: feasibility_sha256,
            plan: plan_sha256,
            partitioned: partitioned_sha256,
        },
    );
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write split freeze report: {error}"))?;
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
    internet_source_manifest_sha256: String,
    internet_source_audit_report_sha256: String,
    source_count: usize,
    recording_count: usize,
    independent_group_counts: IndependentGroupCounts,
    partition_counts: Vec<IdentifiedPartitionCount>,
    entries: Vec<IdentifiedEntry>,
}

#[derive(Deserialize)]
struct IndependentGroupCounts {
    project_revisions: usize,
    objects: usize,
    recordings: usize,
}

#[derive(Deserialize)]
struct IdentifiedPartitionCount {
    partition: String,
    objects: usize,
    recordings: usize,
}

#[derive(Deserialize)]
struct IdentifiedEntry {
    entry_id: String,
    partition: String,
    corpus_role: CorpusRole,
    evidence_tier: String,
    source_group_id: String,
    object_group_id: String,
    source_id: String,
    material_label: String,
    recording_id: String,
    audio_file_sha256: String,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum CorpusRole {
    Target,
    RejectParent,
}

#[derive(Deserialize)]
struct FeasibilityReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    identified_corpus_report_sha256: String,
    corpus_plan_report_sha256: String,
    project_group_count: usize,
    planned_partition_project_counts: Vec<PlannedPartitionCount>,
    blockers: Vec<String>,
    project_groups: Vec<FeasibilityProject>,
}

#[derive(Deserialize)]
struct PlannedPartitionCount {
    partition: String,
    basis_points: u16,
    project_groups: usize,
}

#[derive(Deserialize)]
struct FeasibilityProject {
    source_group_id: String,
    target_objects: usize,
    reject_parent_objects: usize,
    target_recordings: usize,
    reject_parent_recordings: usize,
}

impl FeasibilityReport {
    fn partition_capacities(&self) -> Result<[usize; 4], String> {
        if self.planned_partition_project_counts.len() != PARTITIONS.len() {
            return Err("split feasibility report must contain four partition counts".to_owned());
        }
        let mut capacities = [0_usize; 4];
        for (index, expected) in PARTITIONS.into_iter().enumerate() {
            let count = &self.planned_partition_project_counts[index];
            if count.partition != expected || count.project_groups == 0 {
                return Err("split feasibility partition counts are unordered or empty".to_owned());
            }
            capacities[index] = count.project_groups;
        }
        Ok(capacities)
    }
}

#[derive(Deserialize)]
struct CorpusPlanReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    split_policy: SplitPolicy,
}

#[derive(Deserialize)]
struct SplitPolicy {
    deterministic_seed: String,
    dev_basis_points: u16,
    calibration_basis_points: u16,
    holdout_basis_points: u16,
    shadow_basis_points: u16,
    threshold_selection_partition: String,
    shadow_sealed_before_threshold_selection: bool,
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct Project {
    source_group_id: String,
    target_objects: usize,
    reject_parent_objects: usize,
    target_recordings: usize,
    reject_parent_recordings: usize,
}

impl Project {
    const fn carries_target(&self) -> bool {
        self.target_objects > 0
    }

    const fn carries_reject_parent(&self) -> bool {
        self.reject_parent_objects > 0
    }
}

#[derive(Default)]
struct MutableProject {
    target_objects: BTreeSet<String>,
    reject_parent_objects: BTreeSet<String>,
    target_recordings: usize,
    reject_parent_recordings: usize,
}

fn validate_inputs(
    identified: &IdentifiedReport,
    feasibility: &FeasibilityReport,
    plan: &CorpusPlanReport,
    identified_sha256: &str,
    plan_sha256: &str,
) -> Result<Vec<Project>, String> {
    if identified.schema != IDENTIFIED_REPORT_SCHEMA
        || identified.status != "Validated"
        || identified.decision != "DevelopmentCoverageMeasured"
        || identified.claim != "GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
    {
        return Err("identified corpus report is not the supported development audit".to_owned());
    }
    if feasibility.schema != FEASIBILITY_REPORT_SCHEMA
        || feasibility.status != "Validated"
        || feasibility.decision != "ProjectDisjointSplitFeasible"
        || feasibility.claim
            != "PROJECT_DISJOINT_SPLIT_FEASIBILITY_AUDIT_ONLY / NO_PARTITION_FREEZE_OR_CORPUS_ADMISSION_AUTHORITY"
        || !feasibility.blockers.is_empty()
    {
        return Err("split feasibility report is not a blocker-free audit".to_owned());
    }
    if plan.schema != PLAN_REPORT_SCHEMA
        || plan.status != "Validated"
        || plan.decision != "PlanPowerSufficient"
        || plan.claim != "PREREGISTRATION_AND_POWER_PLAN_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || plan.split_policy.deterministic_seed.is_empty()
        || plan.split_policy.threshold_selection_partition != "calibration"
        || !plan.split_policy.shadow_sealed_before_threshold_selection
    {
        return Err("corpus plan report is not the supported frozen split policy".to_owned());
    }
    if identified.corpus_plan_report_sha256 != plan_sha256
        || feasibility.corpus_plan_report_sha256 != plan_sha256
        || feasibility.identified_corpus_report_sha256 != identified_sha256
    {
        return Err("split inputs are not hash-linked to the same corpus and plan".to_owned());
    }
    let capacities = feasibility.partition_capacities()?;
    if capacities.iter().sum::<usize>() != feasibility.project_group_count
        || feasibility.project_group_count == 0
        || feasibility.project_group_count > MAX_PROJECT_GROUPS
    {
        return Err(format!(
            "split project count must be 1..={MAX_PROJECT_GROUPS} and match capacities"
        ));
    }
    for (planned, expected_basis_points) in feasibility
        .planned_partition_project_counts
        .iter()
        .zip(plan.split_policy.basis_points())
    {
        if planned.basis_points != expected_basis_points {
            return Err("split feasibility counts do not match corpus plan shares".to_owned());
        }
    }
    if capacities
        != allocate_partition_counts(
            feasibility.project_group_count,
            plan.split_policy.basis_points(),
        )
    {
        return Err("split feasibility capacities do not match corpus plan shares".to_owned());
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
        return Err("split freeze requires the complete pre-split corpus in dev".to_owned());
    }
    if identified.entries.len() != identified.recording_count
        || identified.independent_group_counts.recordings != identified.recording_count
    {
        return Err("identified entry counts are inconsistent".to_owned());
    }

    let mut projects = BTreeMap::<String, MutableProject>::new();
    let mut objects = BTreeMap::<String, (String, CorpusRole)>::new();
    for entry in &identified.entries {
        if entry.partition != "dev"
            || entry.evidence_tier != "E3IdentifiedRecording"
            || entry.source_group_id.is_empty()
            || entry.object_group_id.is_empty()
            || (entry.corpus_role == CorpusRole::Target)
                != (entry.material_label == identified.target_material_label)
        {
            return Err(
                "identified entries are not a complete role-consistent dev E3 set".to_owned(),
            );
        }
        let identity = (entry.source_group_id.clone(), entry.corpus_role);
        if let Some(previous) = objects.insert(entry.object_group_id.clone(), identity.clone())
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
    if projects.len() != identified.independent_group_counts.project_revisions
        || objects.len() != identified.independent_group_counts.objects
        || projects.len() != feasibility.project_group_count
        || feasibility.project_groups.len() != feasibility.project_group_count
    {
        return Err("identified and feasibility project aggregates differ".to_owned());
    }
    let expected = feasibility
        .project_groups
        .iter()
        .map(|project| (project.source_group_id.as_str(), project))
        .collect::<BTreeMap<_, _>>();
    if expected.len() != feasibility.project_groups.len() {
        return Err("split feasibility contains duplicate project groups".to_owned());
    }
    projects
        .into_iter()
        .map(|(source_group_id, project)| {
            let frozen = Project {
                source_group_id: source_group_id.clone(),
                target_objects: project.target_objects.len(),
                reject_parent_objects: project.reject_parent_objects.len(),
                target_recordings: project.target_recordings,
                reject_parent_recordings: project.reject_parent_recordings,
            };
            let expected = expected
                .get(source_group_id.as_str())
                .ok_or_else(|| format!("feasibility report is missing {source_group_id}"))?;
            if frozen.target_objects != expected.target_objects
                || frozen.reject_parent_objects != expected.reject_parent_objects
                || frozen.target_recordings != expected.target_recordings
                || frozen.reject_parent_recordings != expected.reject_parent_recordings
            {
                return Err(format!(
                    "feasibility counts differ for project {source_group_id}"
                ));
            }
            Ok(frozen)
        })
        .collect()
}

fn solve_assignment(
    projects: &[Project],
    capacities: [usize; 4],
    seed: &str,
) -> Result<Vec<usize>, String> {
    let mut order = (0..projects.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| {
        seeded_key(seed, "project", &projects[*left].source_group_id)
            .cmp(&seeded_key(
                seed,
                "project",
                &projects[*right].source_group_id,
            ))
            .then(
                projects[*left]
                    .source_group_id
                    .cmp(&projects[*right].source_group_id),
            )
    });
    let mut remaining = capacities;
    let mut target_covered = [false; 4];
    let mut reject_covered = [false; 4];
    let mut ordered_assignment = vec![None; projects.len()];
    let mut visited = 0_usize;
    let solved = search_assignment(
        projects,
        &order,
        seed,
        0,
        &mut remaining,
        &mut target_covered,
        &mut reject_covered,
        &mut ordered_assignment,
        &mut visited,
    );
    if !solved {
        return Err(if visited >= MAX_SEARCH_NODES {
            format!("split assignment exceeded deterministic search bound {MAX_SEARCH_NODES}")
        } else {
            "feasibility report has no concrete project assignment".to_owned()
        });
    }
    let mut assignment = vec![0_usize; projects.len()];
    for (ordered_index, project_index) in order.into_iter().enumerate() {
        assignment[project_index] = ordered_assignment[ordered_index]
            .ok_or_else(|| "split solver returned an incomplete assignment".to_owned())?;
    }
    Ok(assignment)
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

fn validate_partitioned_report(
    pre_split: &IdentifiedReport,
    partitioned: &IdentifiedReport,
    projects: &[Project],
    assignment: &[usize],
) -> Result<(), String> {
    if partitioned.schema != IDENTIFIED_REPORT_SCHEMA
        || partitioned.status != "Validated"
        || partitioned.decision != "DevelopmentCoverageMeasured"
        || partitioned.claim != "GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || partitioned.target_material_label != pre_split.target_material_label
        || partitioned.corpus_plan_report_sha256 != pre_split.corpus_plan_report_sha256
        || partitioned.internet_source_manifest_sha256 != pre_split.internet_source_manifest_sha256
        || partitioned.internet_source_audit_report_sha256
            != pre_split.internet_source_audit_report_sha256
        || partitioned.source_count != pre_split.source_count
        || partitioned.recording_count != pre_split.recording_count
        || partitioned.independent_group_counts.project_revisions
            != pre_split.independent_group_counts.project_revisions
        || partitioned.independent_group_counts.objects
            != pre_split.independent_group_counts.objects
        || partitioned.independent_group_counts.recordings
            != pre_split.independent_group_counts.recordings
        || partitioned.entries.len() != pre_split.entries.len()
    {
        return Err("partitioned identified report changes the frozen corpus identity".to_owned());
    }
    let project_partitions = projects
        .iter()
        .zip(assignment)
        .map(|(project, partition)| (project.source_group_id.as_str(), *partition))
        .collect::<BTreeMap<_, _>>();
    let partitioned_entries = partitioned
        .entries
        .iter()
        .map(|entry| (entry.entry_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    if partitioned_entries.len() != partitioned.entries.len() {
        return Err("partitioned identified report contains duplicate entries".to_owned());
    }
    let mut partition_objects = std::array::from_fn::<_, 4, _>(|_| BTreeSet::<&str>::new());
    let mut partition_recordings = [0_usize; 4];
    for expected in &pre_split.entries {
        let actual = partitioned_entries
            .get(expected.entry_id.as_str())
            .ok_or_else(|| format!("partitioned report is missing entry {}", expected.entry_id))?;
        if !same_entry_identity(expected, actual) {
            return Err(format!(
                "partitioned report changes entry identity {}",
                expected.entry_id
            ));
        }
        let partition = *project_partitions
            .get(expected.source_group_id.as_str())
            .ok_or_else(|| {
                format!(
                    "split assignment is missing project {}",
                    expected.source_group_id
                )
            })?;
        if actual.partition != PARTITIONS[partition] {
            return Err(format!(
                "entry {} is in {}, expected {}",
                expected.entry_id, actual.partition, PARTITIONS[partition]
            ));
        }
        partition_objects[partition].insert(actual.object_group_id.as_str());
        partition_recordings[partition] += 1;
    }
    if partitioned.partition_counts.len() != PARTITIONS.len()
        || partitioned
            .partition_counts
            .iter()
            .zip(PARTITIONS)
            .enumerate()
            .any(|(index, (count, partition))| {
                count.partition != partition
                    || count.objects != partition_objects[index].len()
                    || count.recordings != partition_recordings[index]
            })
    {
        return Err("partitioned report aggregate counts do not match frozen entries".to_owned());
    }
    Ok(())
}

fn same_entry_identity(expected: &IdentifiedEntry, actual: &IdentifiedEntry) -> bool {
    expected.entry_id == actual.entry_id
        && expected.corpus_role == actual.corpus_role
        && expected.evidence_tier == actual.evidence_tier
        && expected.source_group_id == actual.source_group_id
        && expected.object_group_id == actual.object_group_id
        && expected.source_id == actual.source_id
        && expected.material_label == actual.material_label
        && expected.recording_id == actual.recording_id
        && expected.audio_file_sha256 == actual.audio_file_sha256
}

#[allow(clippy::too_many_arguments)]
fn search_assignment(
    projects: &[Project],
    order: &[usize],
    seed: &str,
    cursor: usize,
    remaining: &mut [usize; 4],
    target_covered: &mut [bool; 4],
    reject_covered: &mut [bool; 4],
    assignment: &mut [Option<usize>],
    visited: &mut usize,
) -> bool {
    if *visited >= MAX_SEARCH_NODES {
        return false;
    }
    *visited += 1;
    if cursor == order.len() {
        return remaining.iter().all(|count| *count == 0)
            && target_covered.iter().all(|covered| *covered)
            && reject_covered.iter().all(|covered| *covered);
    }
    if !can_still_cover(
        projects,
        &order[cursor..],
        remaining,
        target_covered,
        reject_covered,
    ) {
        return false;
    }
    let project = &projects[order[cursor]];
    let mut partition_order = (0..PARTITIONS.len())
        .filter(|partition| remaining[*partition] > 0)
        .collect::<Vec<_>>();
    partition_order.sort_by(|left, right| {
        seeded_key(seed, &project.source_group_id, PARTITIONS[*left])
            .cmp(&seeded_key(
                seed,
                &project.source_group_id,
                PARTITIONS[*right],
            ))
            .then(left.cmp(right))
    });
    for partition in partition_order {
        let previous_target = target_covered[partition];
        let previous_reject = reject_covered[partition];
        remaining[partition] -= 1;
        target_covered[partition] |= project.carries_target();
        reject_covered[partition] |= project.carries_reject_parent();
        assignment[cursor] = Some(partition);
        if search_assignment(
            projects,
            order,
            seed,
            cursor + 1,
            remaining,
            target_covered,
            reject_covered,
            assignment,
            visited,
        ) {
            return true;
        }
        assignment[cursor] = None;
        target_covered[partition] = previous_target;
        reject_covered[partition] = previous_reject;
        remaining[partition] += 1;
    }
    false
}

fn can_still_cover(
    projects: &[Project],
    remaining_order: &[usize],
    remaining: &[usize; 4],
    target_covered: &[bool; 4],
    reject_covered: &[bool; 4],
) -> bool {
    let target_projects = remaining_order
        .iter()
        .filter(|index| projects[**index].carries_target())
        .count();
    let reject_projects = remaining_order
        .iter()
        .filter(|index| projects[**index].carries_reject_parent())
        .count();
    let dual_projects = remaining_order
        .iter()
        .filter(|index| {
            projects[**index].carries_target() && projects[**index].carries_reject_parent()
        })
        .count();
    let missing_target = target_covered.iter().filter(|covered| !**covered).count();
    let missing_reject = reject_covered.iter().filter(|covered| !**covered).count();
    if target_projects < missing_target || reject_projects < missing_reject {
        return false;
    }
    for partition in 0..PARTITIONS.len() {
        let missing_roles =
            usize::from(!target_covered[partition]) + usize::from(!reject_covered[partition]);
        if missing_roles > 0 && remaining[partition] == 0 {
            return false;
        }
        if missing_roles == 2 && remaining[partition] == 1 && dual_projects == 0 {
            return false;
        }
    }
    true
}

fn seeded_key(seed: &str, left: &str, right: &str) -> String {
    sha256_hex(format!("{seed}\0{left}\0{right}").as_bytes())
}

#[derive(Serialize)]
struct SplitFreezeReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    identified_corpus_report_sha256: String,
    split_feasibility_report_sha256: String,
    corpus_plan_report_sha256: String,
    partitioned_corpus_report_sha256: Option<String>,
    deterministic_seed: String,
    assignment_algorithm: &'static str,
    project_group_count: usize,
    partition_counts: Vec<FrozenPartitionCount>,
    assignments: Vec<FrozenAssignment>,
}

struct ReportHashes {
    identified: String,
    feasibility: String,
    plan: String,
    partitioned: Option<String>,
}

#[derive(Serialize)]
struct FrozenPartitionCount {
    partition: &'static str,
    project_groups: usize,
    target_bearing_project_groups: usize,
    reject_parent_bearing_project_groups: usize,
}

#[derive(Serialize)]
struct FrozenAssignment {
    source_group_id: String,
    partition: &'static str,
    carries_target: bool,
    carries_reject_parent: bool,
}

fn build_report(
    projects: &[Project],
    assignment: Vec<usize>,
    capacities: [usize; 4],
    seed: &str,
    hashes: ReportHashes,
) -> SplitFreezeReport {
    let mut counts = [(0_usize, 0_usize); 4];
    let mut assignments = projects
        .iter()
        .zip(assignment)
        .map(|(project, partition)| {
            counts[partition].0 += usize::from(project.carries_target());
            counts[partition].1 += usize::from(project.carries_reject_parent());
            FrozenAssignment {
                source_group_id: project.source_group_id.clone(),
                partition: PARTITIONS[partition],
                carries_target: project.carries_target(),
                carries_reject_parent: project.carries_reject_parent(),
            }
        })
        .collect::<Vec<_>>();
    assignments.sort_by(|left, right| left.source_group_id.cmp(&right.source_group_id));
    SplitFreezeReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: if hashes.partitioned.is_some() {
            "ProjectDisjointSplitVerified"
        } else {
            "ProjectDisjointSplitFrozen"
        },
        claim: REPORT_CLAIM,
        identified_corpus_report_sha256: hashes.identified,
        split_feasibility_report_sha256: hashes.feasibility,
        corpus_plan_report_sha256: hashes.plan,
        partitioned_corpus_report_sha256: hashes.partitioned,
        deterministic_seed: seed.to_owned(),
        assignment_algorithm: ASSIGNMENT_ALGORITHM,
        project_group_count: projects.len(),
        partition_counts: PARTITIONS
            .into_iter()
            .zip(capacities)
            .zip(counts)
            .map(
                |((partition, project_groups), (target_bearing, reject_bearing))| {
                    FrozenPartitionCount {
                        partition,
                        project_groups,
                        target_bearing_project_groups: target_bearing,
                        reject_parent_bearing_project_groups: reject_bearing,
                    }
                },
            )
            .collect(),
        assignments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(id: &str, target: bool, reject: bool) -> Project {
        Project {
            source_group_id: id.to_owned(),
            target_objects: usize::from(target),
            reject_parent_objects: usize::from(reject),
            target_recordings: usize::from(target),
            reject_parent_recordings: usize::from(reject),
        }
    }

    #[test]
    fn solver_freezes_role_complete_project_disjoint_partitions() {
        let projects = vec![
            project("dual-a", true, true),
            project("dual-b", true, true),
            project("dual-c", true, true),
            project("dual-d", true, true),
            project("target-a", true, false),
            project("target-b", true, false),
            project("target-c", true, false),
            project("target-d", true, false),
        ];
        let first = solve_assignment(&projects, [2, 2, 2, 2], "frozen-seed").unwrap();
        let second = solve_assignment(&projects, [2, 2, 2, 2], "frozen-seed").unwrap();
        assert_eq!(first, second);
        for partition in 0..4 {
            let assigned = projects
                .iter()
                .zip(&first)
                .filter(|(_, assigned)| **assigned == partition)
                .map(|(project, _)| project)
                .collect::<Vec<_>>();
            assert_eq!(assigned.len(), 2);
            assert!(assigned.iter().any(|project| project.carries_target()));
            assert!(
                assigned
                    .iter()
                    .any(|project| project.carries_reject_parent())
            );
        }
    }

    #[test]
    fn solver_rejects_three_reject_projects_for_four_partitions() {
        let projects = vec![
            project("dual-a", true, true),
            project("dual-b", true, true),
            project("dual-c", true, true),
            project("target-a", true, false),
            project("target-b", true, false),
            project("target-c", true, false),
            project("target-d", true, false),
            project("target-e", true, false),
        ];
        assert!(solve_assignment(&projects, [2, 2, 2, 2], "frozen-seed").is_err());
    }

    #[test]
    fn largest_remainder_allocation_matches_feasibility_gate() {
        assert_eq!(
            allocate_partition_counts(8, [2_000, 3_000, 2_500, 2_500]),
            [2, 2, 2, 2]
        );
        assert_eq!(
            allocate_partition_counts(7, [2_000, 3_000, 2_500, 2_500]),
            [1, 2, 2, 2]
        );
    }
}
