use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::DeserializeOwned;

use self::report::{ReportHashes, ReportInputs, build_report};

use super::{
    FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_cli_path, resolve_output_path, set_once, sha256_hex,
};

mod report;

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-domain-claims.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-domain-claims.report.v1";
const PLAN_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-plan.report.v1";
const IDENTIFIED_REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-identified-corpus.report.v1";
const SPLIT_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-split-freeze.report.v1";
const INVENTORY_REPORT_SCHEMA: &str =
    "nextengine.experimental-physical-sound-corpus-inventory.report.v2";
const MATRIX_ID: &str = "physical-sound-ps2-thin-glass-vessel";
const MATRIX_REVISION: &str = "v1";
const PROFILE_ID: &str = "thin-soda-lime-glass-open-vessel-impact-v1";
const DOMAIN_ID: &str = "thin-soda-lime-glass-open-vessel-impact";
const REPORT_CLAIM: &str =
    "EXACT_DOMAIN_E2_E3_CLAIM_MATRIX_ONLY / NO_VALIDATOR_RELEASE_OR_CORPUS_ADMISSION_AUTHORITY";
const FALLBACK_DECISION: &str = "FallbackOutOfDomain";
const PARTITIONS: [&str; 4] = ["dev", "calibration", "holdout", "shadow"];
const MAX_E2_REPORTS: usize = 64;
const MAX_OBJECT_LINKS: usize = 64;
const BLUE_BOWL_LINK_ID: &str = "realimpact-objectfolder-blue-bowl-6-v1";
const BLUE_BOWL_LINK_ADAPTER: &str = "realimpact_objectfolder_blue_bowl_identity_v1";
const BLUE_BOWL_E2_ENTRY: &str = "realimpact-blue-bowl-row0000";
const BLUE_BOWL_E2_OBJECT: &str = "realimpact-6-bowl";
const BLUE_BOWL_DATASET_OBJECT: &str = "6_Bowl";
const OBJECTFOLDER_SOURCE_GROUP: &str = "stanford-vision-and-learning-lab--objectfolder-real-interactive-demos--demo-repositories-2023-02-09-v1";
const BLUE_BOWL_E3_GROUP: &str = "stanford-vision-and-learning-lab--objectfolder-real-interactive-demos--demo-repositories-2023-02-09-v1--object-6";
const BLUE_BOWL_E3_SOURCE: &str = "objectfolder-real-demo-6";
const OBSERVED_CAPABILITIES: [&str; 6] = [
    "force_deconvolved_transfer",
    "geometry",
    "impact_position",
    "listener_position",
    "object_identity",
    "real_recording",
];
const CURRENT_UNAVAILABLE_COMPONENTS: [&str; 4] = [
    "force-profile-bytes",
    "material-composition-revision",
    "repeat-recording-identity",
    "support-fixture-revision",
];

struct Request {
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
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected domain-claims argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry domain-claims requires --manifest <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry domain-claims requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = canonical_external_file(
        &root,
        &resolve_cli_path(&root, &request.manifest),
        "domain claims manifest",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let manifest_bytes =
        read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "domain claims manifest")?;
    let manifest: DomainClaimsManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "domain claims manifest has no parent directory".to_owned())?;

    let (plan, plan_sha256) = read_reference::<CorpusPlanReport>(
        &root,
        base,
        &manifest.corpus_plan_report,
        "corpus plan report",
    )?;
    validate_plan(&plan)?;
    let (identified, identified_sha256) = read_reference::<IdentifiedReport>(
        &root,
        base,
        &manifest.partitioned_identified_report,
        "partitioned identified corpus report",
    )?;
    let (split, split_sha256) = read_reference::<SplitVerificationReport>(
        &root,
        base,
        &manifest.split_verification_report,
        "split verification report",
    )?;
    let e2_reports = manifest
        .e2_inventory_reports
        .iter()
        .enumerate()
        .map(|(index, reference)| {
            read_reference::<InventoryReport>(
                &root,
                base,
                reference,
                &format!("E2 inventory report {index}"),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let e3_groups =
        validate_identified_and_split(&identified, &identified_sha256, &split, &plan_sha256)?;
    let e2_entries = validate_e2_reports(e2_reports, &plan_sha256)?;
    let links = validate_links(&manifest.object_links, &e2_entries, &e3_groups, &plan)?;
    let report = build_report(
        ReportInputs {
            hashes: ReportHashes {
                manifest: sha256_hex(&manifest_bytes),
                plan: plan_sha256,
                identified: identified_sha256,
                split: split_sha256,
            },
            plan: &plan,
            identified: &identified,
            e2_entries: &e2_entries,
            e3_groups: &e3_groups,
        },
        links,
    );
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write domain claims report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainClaimsManifest {
    schema: String,
    matrix_id: String,
    revision: String,
    profile_id: String,
    corpus_plan_report: FileRef,
    partitioned_identified_report: FileRef,
    split_verification_report: FileRef,
    e2_inventory_reports: Vec<FileRef>,
    object_links: Vec<ObjectLinkDeclaration>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectLinkDeclaration {
    link_id: String,
    adapter_id: String,
    e2_inventory_report_sha256: String,
    e2_entry_id: String,
    e3_object_group_id: String,
}

fn validate_manifest(manifest: &DomainClaimsManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.matrix_id != MATRIX_ID
        || manifest.revision != MATRIX_REVISION
        || manifest.profile_id != PROFILE_ID
    {
        return Err("domain claims manifest does not select the frozen V1 profile".to_owned());
    }
    if manifest.e2_inventory_reports.is_empty()
        || manifest.e2_inventory_reports.len() > MAX_E2_REPORTS
    {
        return Err(format!(
            "domain claims manifest requires 1..={MAX_E2_REPORTS} E2 reports"
        ));
    }
    if manifest.object_links.len() > MAX_OBJECT_LINKS {
        return Err(format!(
            "domain claims manifest exceeds {MAX_OBJECT_LINKS} object links"
        ));
    }
    let report_hashes = manifest
        .e2_inventory_reports
        .iter()
        .map(|reference| reference.sha256.as_str())
        .collect::<BTreeSet<_>>();
    if report_hashes.len() != manifest.e2_inventory_reports.len() {
        return Err("domain claims manifest repeats an E2 report hash".to_owned());
    }
    let link_ids = manifest
        .object_links
        .iter()
        .map(|link| link.link_id.as_str())
        .collect::<BTreeSet<_>>();
    if link_ids.len() != manifest.object_links.len() {
        return Err("domain claims manifest repeats an object link id".to_owned());
    }
    Ok(())
}

fn read_reference<T: DeserializeOwned>(
    root: &Path,
    base: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<(T, String), String> {
    let unresolved = Path::new(&reference.path);
    let unresolved = if unresolved.is_absolute() {
        unresolved.to_owned()
    } else {
        base.join(unresolved)
    };
    let path = canonical_external_file(root, &unresolved, role)?;
    let bytes = read_bounded_file(&path, MAX_MANIFEST_BYTES, role)?;
    let actual_sha256 = sha256_hex(&bytes);
    if actual_sha256 != reference.sha256 {
        return Err(format!(
            "{role} hash mismatch for {}: expected {}, got {actual_sha256}",
            path.display(),
            reference.sha256
        ));
    }
    let parsed = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse {role} {}: {error}", path.display()))?;
    Ok((parsed, actual_sha256))
}

#[derive(Deserialize)]
struct CorpusPlanReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    plan_id: String,
    revision: String,
    domains: Vec<PlanDomain>,
    unavailable_component_decision: String,
}

#[derive(Deserialize)]
struct PlanDomain {
    id: String,
    revision: String,
    material_family: String,
    object_family: String,
    geometry_family: String,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    impact_position_ids: Vec<String>,
    listener_condition_ids: Vec<String>,
    minimum_repeats_per_condition: usize,
}

fn validate_plan(plan: &CorpusPlanReport) -> Result<(), String> {
    if plan.schema != PLAN_REPORT_SCHEMA
        || plan.status != "Validated"
        || plan.decision != "PlanPowerSufficient"
        || plan.claim != "PREREGISTRATION_AND_POWER_PLAN_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || plan.plan_id != "physical-sound-ps2-glass-vessel"
        || plan.revision != "v1"
        || plan.unavailable_component_decision != FALLBACK_DECISION
        || plan.domains.len() != 1
    {
        return Err("corpus plan is not the frozen PS-2 glass-vessel plan".to_owned());
    }
    let domain = &plan.domains[0];
    if domain.id != DOMAIN_ID
        || domain.revision != "v1"
        || domain.material_family != "soda-lime-glass"
        || domain.object_family != "thin-wall-open-vessel"
        || domain.geometry_family != "axisymmetric-open-shell"
        || domain.geometry_revision != "measured-calipers-and-mass-v1"
        || domain.support_condition != "base-on-20mm-foam-annulus-v1"
        || domain.excitation_method != "instrumented-impact-hammer-v1"
        || domain.impact_position_ids != ["rim", "wall-midpoint"]
        || domain.listener_condition_ids != ["half-metre-axis", "half-metre-radial"]
        || domain.minimum_repeats_per_condition != 3
    {
        return Err("corpus plan changes the frozen exact-domain axes".to_owned());
    }
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
    sources: usize,
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
    object_id: String,
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
struct SplitVerificationReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    corpus_plan_report_sha256: String,
    partitioned_corpus_report_sha256: Option<String>,
    project_group_count: usize,
    partition_counts: Vec<SplitPartitionCount>,
    assignments: Vec<SplitAssignment>,
}

#[derive(Deserialize)]
struct SplitPartitionCount {
    partition: String,
    project_groups: usize,
    target_bearing_project_groups: usize,
    reject_parent_bearing_project_groups: usize,
}

#[derive(Deserialize)]
struct SplitAssignment {
    source_group_id: String,
    partition: String,
    carries_target: bool,
    carries_reject_parent: bool,
}

#[derive(Clone)]
struct E3ObjectGroup {
    object_group_id: String,
    source_group_id: String,
    source_id: String,
    object_id: String,
    material_label: String,
    partition: String,
    corpus_role: CorpusRole,
    recording_ids: BTreeSet<String>,
}

fn validate_identified_and_split(
    identified: &IdentifiedReport,
    identified_sha256: &str,
    split: &SplitVerificationReport,
    plan_sha256: &str,
) -> Result<BTreeMap<String, E3ObjectGroup>, String> {
    if identified.schema != IDENTIFIED_REPORT_SCHEMA
        || identified.status != "Validated"
        || identified.decision != "DevelopmentCoverageMeasured"
        || identified.claim != "GROUP_AND_COVERAGE_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
        || identified.target_material_label != "Glass"
        || identified.corpus_plan_report_sha256 != plan_sha256
        || identified.entries.len() != identified.recording_count
        || identified.independent_group_counts.recordings != identified.recording_count
    {
        return Err("identified report is not the frozen partitioned E3 audit".to_owned());
    }
    if split.schema != SPLIT_REPORT_SCHEMA
        || split.status != "Validated"
        || split.decision != "ProjectDisjointSplitVerified"
        || split.claim
            != "DETERMINISTIC_PROJECT_PARTITION_FREEZE_ONLY / NO_CORPUS_ADMISSION_OR_QUALITY_AUTHORITY"
        || split.corpus_plan_report_sha256 != plan_sha256
        || split.partitioned_corpus_report_sha256.as_deref() != Some(identified_sha256)
    {
        return Err("split report does not verify the supplied partitioned E3 audit".to_owned());
    }
    if identified.partition_counts.len() != PARTITIONS.len()
        || split.partition_counts.len() != PARTITIONS.len()
        || identified
            .partition_counts
            .iter()
            .zip(PARTITIONS)
            .any(|(count, expected)| count.partition != expected)
        || split
            .partition_counts
            .iter()
            .zip(PARTITIONS)
            .any(|(count, expected)| count.partition != expected)
    {
        return Err("E3 and split reports must contain the ordered four partitions".to_owned());
    }
    let assignments = split
        .assignments
        .iter()
        .map(|assignment| (assignment.source_group_id.as_str(), assignment))
        .collect::<BTreeMap<_, _>>();
    if assignments.len() != split.assignments.len()
        || assignments.len() != split.project_group_count
    {
        return Err("split report contains inconsistent project assignments".to_owned());
    }

    let mut groups = BTreeMap::<String, E3ObjectGroup>::new();
    let mut entry_ids = BTreeSet::new();
    let mut audio_hashes = BTreeSet::new();
    for entry in &identified.entries {
        if !entry_ids.insert(entry.entry_id.as_str())
            || !audio_hashes.insert(entry.audio_file_sha256.as_str())
            || entry.evidence_tier != "E3IdentifiedRecording"
            || (entry.corpus_role == CorpusRole::Target)
                != (entry.material_label == identified.target_material_label)
        {
            return Err("identified entries are not a unique role-consistent E3 set".to_owned());
        }
        let assignment = assignments
            .get(entry.source_group_id.as_str())
            .ok_or_else(|| format!("split assignment is missing {}", entry.source_group_id))?;
        if assignment.partition != entry.partition {
            return Err(format!(
                "split assignment disagrees with entry {}",
                entry.entry_id
            ));
        }
        match groups.entry(entry.object_group_id.clone()) {
            std::collections::btree_map::Entry::Vacant(vacant) => {
                vacant.insert(E3ObjectGroup {
                    object_group_id: entry.object_group_id.clone(),
                    source_group_id: entry.source_group_id.clone(),
                    source_id: entry.source_id.clone(),
                    object_id: entry.object_id.clone(),
                    material_label: entry.material_label.clone(),
                    partition: entry.partition.clone(),
                    corpus_role: entry.corpus_role,
                    recording_ids: BTreeSet::from([entry.recording_id.clone()]),
                });
            }
            std::collections::btree_map::Entry::Occupied(mut occupied) => {
                let group = occupied.get_mut();
                if group.source_group_id != entry.source_group_id
                    || group.source_id != entry.source_id
                    || group.object_id != entry.object_id
                    || group.material_label != entry.material_label
                    || group.partition != entry.partition
                    || group.corpus_role != entry.corpus_role
                    || !group.recording_ids.insert(entry.recording_id.clone())
                {
                    return Err(format!(
                        "E3 object group {} changes identity or repeats a recording",
                        entry.object_group_id
                    ));
                }
            }
        }
    }
    validate_partition_aggregates(identified, split, &groups, &assignments)?;
    Ok(groups)
}

fn validate_partition_aggregates(
    identified: &IdentifiedReport,
    split: &SplitVerificationReport,
    groups: &BTreeMap<String, E3ObjectGroup>,
    assignments: &BTreeMap<&str, &SplitAssignment>,
) -> Result<(), String> {
    for (index, partition) in PARTITIONS.into_iter().enumerate() {
        let group_values = groups
            .values()
            .filter(|group| group.partition == partition)
            .collect::<Vec<_>>();
        let recordings = group_values
            .iter()
            .map(|group| group.recording_ids.len())
            .sum::<usize>();
        let sources = group_values
            .iter()
            .map(|group| group.source_group_id.as_str())
            .collect::<BTreeSet<_>>();
        let count = &identified.partition_counts[index];
        if count.objects != group_values.len()
            || count.recordings != recordings
            || count.sources != sources.len()
        {
            return Err(format!(
                "identified partition aggregate differs for {partition}"
            ));
        }
        let assigned = assignments
            .values()
            .filter(|assignment| assignment.partition == partition)
            .copied()
            .collect::<Vec<_>>();
        let split_count = &split.partition_counts[index];
        if split_count.project_groups != assigned.len()
            || split_count.target_bearing_project_groups
                != assigned
                    .iter()
                    .filter(|assignment| assignment.carries_target)
                    .count()
            || split_count.reject_parent_bearing_project_groups
                != assigned
                    .iter()
                    .filter(|assignment| assignment.carries_reject_parent)
                    .count()
        {
            return Err(format!("split partition aggregate differs for {partition}"));
        }
    }
    let source_groups = groups
        .values()
        .map(|group| group.source_group_id.as_str())
        .collect::<BTreeSet<_>>();
    if source_groups.len() != identified.independent_group_counts.project_revisions
        || groups.len() != identified.independent_group_counts.objects
        || source_groups.len() != assignments.len()
    {
        return Err("identified and split source-group counts differ".to_owned());
    }
    Ok(())
}

#[derive(Deserialize)]
struct InventoryReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    corpus_plan_report_sha256: String,
    entry_count: usize,
    entries: Vec<InventoryEntry>,
}

#[derive(Deserialize)]
struct InventoryEntry {
    id: String,
    partition: String,
    provisional_outcome: String,
    recording_kind: String,
    material_family: String,
    object_id: String,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    impact_position_id: String,
    listener_condition_id: String,
    unavailable_components: Vec<String>,
    source_adapter_evidence: SourceAdapterEvidence,
}

#[derive(Deserialize)]
struct SourceAdapterEvidence {
    schema: String,
    evidence_tier: String,
    validated_capabilities: Vec<String>,
    dataset_object_id: String,
    material_label: String,
}

#[derive(Clone)]
struct E2Entry {
    report_sha256: String,
    id: String,
    material_family: String,
    object_id: String,
    dataset_object_id: String,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    impact_position_id: String,
    listener_condition_id: String,
    unavailable_components: BTreeSet<String>,
    validated_capabilities: BTreeSet<String>,
}

fn validate_e2_reports(
    reports: Vec<(InventoryReport, String)>,
    plan_sha256: &str,
) -> Result<BTreeMap<String, E2Entry>, String> {
    let expected_capabilities = OBSERVED_CAPABILITIES
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let expected_unavailable = CURRENT_UNAVAILABLE_COMPONENTS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let mut entries = BTreeMap::new();
    for (report, report_sha256) in reports {
        if report.schema != INVENTORY_REPORT_SCHEMA
            || report.status != "Validated"
            || report.decision != "DevelopmentPilotOnly"
            || report.claim != "INVENTORY_AND_PARTITION_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY"
            || report.corpus_plan_report_sha256 != plan_sha256
            || report.entry_count != report.entries.len()
            || report.entries.is_empty()
        {
            return Err(format!(
                "E2 report {report_sha256} is not a supported development inventory"
            ));
        }
        for entry in report.entries {
            let capabilities = entry
                .source_adapter_evidence
                .validated_capabilities
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let unavailable = entry
                .unavailable_components
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            if entry.partition != "dev"
                || entry.provisional_outcome != FALLBACK_DECISION
                || entry.recording_kind != "controlled_real_force_deconvolved_transfer"
                || entry.material_family != "glass"
                || entry.source_adapter_evidence.schema
                    != "realimpact_force_deconvolved_transfer_v1"
                || entry.source_adapter_evidence.evidence_tier != "E2TransferResponse"
                || entry.source_adapter_evidence.material_label != "glass"
                || capabilities != expected_capabilities
                || unavailable != expected_unavailable
            {
                return Err(format!(
                    "E2 entry {} exceeds or changes the frozen REALIMPACT claim profile",
                    entry.id
                ));
            }
            let id = entry.id.clone();
            let frozen = E2Entry {
                report_sha256: report_sha256.clone(),
                id: id.clone(),
                material_family: entry.material_family,
                object_id: entry.object_id,
                dataset_object_id: entry.source_adapter_evidence.dataset_object_id,
                geometry_revision: entry.geometry_revision,
                support_condition: entry.support_condition,
                excitation_method: entry.excitation_method,
                impact_position_id: entry.impact_position_id,
                listener_condition_id: entry.listener_condition_id,
                unavailable_components: unavailable,
                validated_capabilities: capabilities,
            };
            if entries.insert(id.clone(), frozen).is_some() {
                return Err(format!("duplicate E2 entry id {id}"));
            }
        }
    }
    Ok(entries)
}

#[derive(Clone)]
struct ValidatedLink {
    link_id: String,
    e2_entry_id: String,
    e3_object_group_id: String,
    partition: String,
    e3_recording_count: usize,
    material_composition_aligned: bool,
    geometry_aligned: bool,
    support_aligned: bool,
    excitation_aligned: bool,
    impact_condition_aligned: bool,
    listener_condition_aligned: bool,
    matched_condition_identity: bool,
}

impl ValidatedLink {
    const fn exact_domain_eligible(&self) -> bool {
        self.material_composition_aligned
            && self.geometry_aligned
            && self.support_aligned
            && self.excitation_aligned
            && self.impact_condition_aligned
            && self.listener_condition_aligned
            && self.matched_condition_identity
    }
}

fn validate_links(
    declarations: &[ObjectLinkDeclaration],
    e2_entries: &BTreeMap<String, E2Entry>,
    e3_groups: &BTreeMap<String, E3ObjectGroup>,
    plan: &CorpusPlanReport,
) -> Result<Vec<ValidatedLink>, String> {
    let domain = &plan.domains[0];
    let mut linked_e2 = BTreeSet::new();
    let mut linked_e3 = BTreeSet::new();
    let mut links = Vec::with_capacity(declarations.len());
    for declaration in declarations {
        let e2 = e2_entries.get(&declaration.e2_entry_id).ok_or_else(|| {
            format!(
                "object link {} names an unknown E2 entry",
                declaration.link_id
            )
        })?;
        let e3 = e3_groups
            .get(&declaration.e3_object_group_id)
            .ok_or_else(|| {
                format!(
                    "object link {} names an unknown E3 object",
                    declaration.link_id
                )
            })?;
        validate_frozen_object_link(declaration, e2, e3)?;
        if !linked_e2.insert(e2.id.as_str()) || !linked_e3.insert(e3.object_group_id.as_str()) {
            return Err("one E2 or E3 object participates in multiple object links".to_owned());
        }
        let material_composition_aligned = e2.material_family == domain.material_family
            && !e2
                .unavailable_components
                .contains("material-composition-revision");
        let geometry_aligned = e2.geometry_revision == domain.geometry_revision;
        let support_aligned = e2.support_condition == domain.support_condition
            && !e2
                .unavailable_components
                .contains("support-fixture-revision");
        let excitation_aligned = e2.excitation_method == domain.excitation_method
            && !e2.unavailable_components.contains("force-profile-bytes");
        let impact_condition_aligned = domain.impact_position_ids.contains(&e2.impact_position_id)
            && !e2.unavailable_components.contains("force-profile-bytes");
        let listener_condition_aligned = domain
            .listener_condition_ids
            .contains(&e2.listener_condition_id);
        let matched_condition_identity = !e2
            .unavailable_components
            .contains("repeat-recording-identity")
            && e3.recording_ids.len() >= domain.minimum_repeats_per_condition;
        links.push(ValidatedLink {
            link_id: declaration.link_id.clone(),
            e2_entry_id: e2.id.clone(),
            e3_object_group_id: e3.object_group_id.clone(),
            partition: e3.partition.clone(),
            e3_recording_count: e3.recording_ids.len(),
            material_composition_aligned,
            geometry_aligned,
            support_aligned,
            excitation_aligned,
            impact_condition_aligned,
            listener_condition_aligned,
            matched_condition_identity,
        });
    }
    links.sort_by(|left, right| left.link_id.cmp(&right.link_id));
    Ok(links)
}

fn validate_frozen_object_link(
    declaration: &ObjectLinkDeclaration,
    e2: &E2Entry,
    e3: &E3ObjectGroup,
) -> Result<(), String> {
    if declaration.link_id != BLUE_BOWL_LINK_ID
        || declaration.adapter_id != BLUE_BOWL_LINK_ADAPTER
        || declaration.e2_inventory_report_sha256 != e2.report_sha256
        || declaration.e2_entry_id != BLUE_BOWL_E2_ENTRY
        || declaration.e3_object_group_id != BLUE_BOWL_E3_GROUP
        || e2.id != BLUE_BOWL_E2_ENTRY
        || e2.object_id != BLUE_BOWL_E2_OBJECT
        || e2.dataset_object_id != BLUE_BOWL_DATASET_OBJECT
        || e3.object_group_id != BLUE_BOWL_E3_GROUP
        || e3.source_group_id != OBJECTFOLDER_SOURCE_GROUP
        || e3.source_id != BLUE_BOWL_E3_SOURCE
        || e3.object_id != "6"
        || e3.material_label != "Glass"
        || e3.corpus_role != CorpusRole::Target
    {
        return Err(format!(
            "object link {} is not the frozen Blue Bowl cross-tier identity",
            declaration.link_id
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e2_entry(entry_id: &str, object_id: &str, dataset_object_id: &str) -> E2Entry {
        E2Entry {
            report_sha256: "a".repeat(64),
            id: entry_id.to_owned(),
            material_family: "glass".to_owned(),
            object_id: object_id.to_owned(),
            dataset_object_id: dataset_object_id.to_owned(),
            geometry_revision: "mesh-f23127b45b0b-v1".to_owned(),
            support_condition: "thread-mesh-unversioned-in-archive".to_owned(),
            excitation_method: "instrumented-hammer-force-deconvolved".to_owned(),
            impact_position_id: "mesh-vertex-35950".to_owned(),
            listener_condition_id: "angle-000-distance-0230mm-mic-00".to_owned(),
            unavailable_components: CURRENT_UNAVAILABLE_COMPONENTS
                .into_iter()
                .map(str::to_owned)
                .collect(),
            validated_capabilities: OBSERVED_CAPABILITIES
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }

    fn e3_group(object_group_id: &str, source_id: &str, object_id: &str) -> E3ObjectGroup {
        E3ObjectGroup {
            object_group_id: object_group_id.to_owned(),
            source_group_id: OBJECTFOLDER_SOURCE_GROUP.to_owned(),
            source_id: source_id.to_owned(),
            object_id: object_id.to_owned(),
            material_label: "Glass".to_owned(),
            partition: "calibration".to_owned(),
            corpus_role: CorpusRole::Target,
            recording_ids: BTreeSet::from(["000".to_owned(), "020".to_owned(), "039".to_owned()]),
        }
    }

    fn blue_bowl_declaration() -> ObjectLinkDeclaration {
        ObjectLinkDeclaration {
            link_id: BLUE_BOWL_LINK_ID.to_owned(),
            adapter_id: BLUE_BOWL_LINK_ADAPTER.to_owned(),
            e2_inventory_report_sha256: "a".repeat(64),
            e2_entry_id: BLUE_BOWL_E2_ENTRY.to_owned(),
            e3_object_group_id: BLUE_BOWL_E3_GROUP.to_owned(),
        }
    }

    #[test]
    fn arguments_require_manifest_and_output() {
        let request = parse_arguments(
            [
                "--manifest".to_owned(),
                "/tmp/domain-claims.json".to_owned(),
                "--output".to_owned(),
                "/tmp/domain-claims-report".to_owned(),
            ]
            .into_iter(),
        )
        .expect("arguments parse");
        assert_eq!(request.manifest, PathBuf::from("/tmp/domain-claims.json"));
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(parse_arguments(["--unknown".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn frozen_blue_bowl_link_validates_without_claiming_exact_axes() {
        let e2 = e2_entry(
            BLUE_BOWL_E2_ENTRY,
            BLUE_BOWL_E2_OBJECT,
            BLUE_BOWL_DATASET_OBJECT,
        );
        let e3 = e3_group(BLUE_BOWL_E3_GROUP, BLUE_BOWL_E3_SOURCE, "6");
        validate_frozen_object_link(&blue_bowl_declaration(), &e2, &e3)
            .expect("frozen Blue Bowl link validates");
    }

    #[test]
    fn numeric_identity_does_not_link_glass_goblet_to_salad_bowl() {
        let e2 = e2_entry(
            "realimpact-glassgoblet-row0000",
            "realimpact-94-glassgoblet",
            "94_GlassGoblet",
        );
        let e3 = e3_group(
            "stanford-vision-and-learning-lab--objectfolder-real-interactive-demos--demo-repositories-2023-02-09-v1--object-94",
            "objectfolder-real-demo-94",
            "94",
        );
        let declaration = ObjectLinkDeclaration {
            link_id: "realimpact-objectfolder-94-v1".to_owned(),
            adapter_id: BLUE_BOWL_LINK_ADAPTER.to_owned(),
            e2_inventory_report_sha256: "a".repeat(64),
            e2_entry_id: e2.id.clone(),
            e3_object_group_id: e3.object_group_id.clone(),
        };
        assert!(validate_frozen_object_link(&declaration, &e2, &e3).is_err());
    }

    #[test]
    fn current_link_is_not_exact_domain_eligible() {
        let link = ValidatedLink {
            link_id: BLUE_BOWL_LINK_ID.to_owned(),
            e2_entry_id: BLUE_BOWL_E2_ENTRY.to_owned(),
            e3_object_group_id: BLUE_BOWL_E3_GROUP.to_owned(),
            partition: "calibration".to_owned(),
            e3_recording_count: 3,
            material_composition_aligned: false,
            geometry_aligned: false,
            support_aligned: false,
            excitation_aligned: false,
            impact_condition_aligned: false,
            listener_condition_aligned: false,
            matched_condition_identity: false,
        };
        assert!(!link.exact_domain_eligible());
    }

    #[test]
    fn partition_closure_requires_every_frozen_partition() {
        let exact_eligible = PARTITIONS.map(|partition| usize::from(partition == "calibration"));
        assert!(!exact_eligible.into_iter().all(|count| count > 0));
    }
}
