use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use next_contracts::canonical::sha256;
use next_contracts::ids::ContentHash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod bem_feasibility;
mod corpus_inventory;
mod corpus_plan;
mod domain_claims;
mod internet_sources;
mod neural_data_plane;
mod realimpact_row;
mod realimpact_transfer_fixture;
mod realimpact_transfer_preregistration;
mod realimpact_transfer_projection;
mod source_feasibility;
mod split_feasibility;
mod split_freeze;
mod transfer_calibration;
use transfer_calibration::run_cli as run_transfer;

const SCHEMA: &str = "nextengine.experimental-physical-sound-research-registry.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-research-registry.report.v1";
const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_REFERENCED_FILE_BYTES: usize = 512 * 1024 * 1024;
const MAX_FORMULA_FAMILIES: usize = 256;
const MAX_DOMAINS: usize = 8_192;
const MAX_CONDITIONS_PER_AXIS: usize = 256;

pub(super) struct Request {
    manifest: PathBuf,
    output: PathBuf,
}
pub(super) fn parse_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<Request, String> {
    let mut manifest = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--manifest" => set_once(&mut manifest, PathBuf::from(value), &flag)?,
            "--output" => set_once(&mut output, PathBuf::from(value), &flag)?,
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry requires --manifest <external-json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry requires --output <external-empty-directory>".to_owned()
        })?,
    })
}
pub(super) fn run_cli(root: &Path, arguments: impl Iterator<Item = String>) -> Result<(), String> {
    let mut arguments = arguments.peekable();
    let subcommand = arguments.peek().cloned();
    match subcommand.as_deref() {
        Some("bem-feasibility") => return bem_feasibility::run_cli(root, arguments.skip(1)),
        Some("corpus-plan") => return corpus_plan::run_cli(root, arguments.skip(1)),
        Some("corpus-inventory") => {
            arguments.next();
            return corpus_inventory::run_cli(root, arguments);
        }
        Some("domain-claims") => {
            arguments.next();
            return domain_claims::run_cli(root, arguments);
        }
        Some("internet-sources") => {
            arguments.next();
            return internet_sources::run_cli(root, arguments);
        }
        Some("neural-data-plane") => {
            arguments.next();
            return neural_data_plane::run_cli(root, arguments);
        }
        Some("identified-corpus") => {
            arguments.next();
            return internet_sources::run_identified_corpus_cli(root, arguments);
        }
        Some("realimpact-row") => {
            arguments.next();
            return realimpact_row::run_cli(root, arguments);
        }
        Some("realimpact-transfer-preregister") => {
            return realimpact_transfer_preregistration::run_cli(root, arguments.skip(1));
        }
        Some("realimpact-transfer-fixture") => {
            return realimpact_transfer_fixture::run_cli(root, arguments.skip(1));
        }
        Some("realimpact-transfer-project") => {
            return realimpact_transfer_projection::run_cli(root, arguments.skip(1));
        }
        Some("source-feasibility") => return source_feasibility::run_cli(root, arguments.skip(1)),
        Some("spatial-calibration") => return realimpact_row::run_spatial(root, arguments.skip(1)),
        Some("spatial-extension") => return realimpact_row::run_extension(root, arguments.skip(1)),
        Some("spatial-shape") => return realimpact_row::run_shape(root, arguments.skip(1)),
        Some("split-feasibility") => return split_feasibility::run_cli(root, arguments.skip(1)),
        Some("split-freeze") => return split_freeze::run_cli(root, arguments.skip(1)),
        Some("transfer-calibration") => return run_transfer(root, arguments.skip(1)),
        _ => {}
    }
    let request = parse_arguments(arguments)?;
    run(root, &request)
}
fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistryManifest {
    schema: String,
    registry_id: String,
    formula_families: Vec<FormulaFamily>,
    domains: Vec<DomainRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FormulaFamily {
    id: String,
    revision: String,
    source_class: SourceClass,
    equation_id: String,
    model_definition: FileRef,
    parameter_schema: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SourceClass {
    RigidImpact,
}

impl SourceClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::RigidImpact => "rigid_impact",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DomainRecord {
    id: String,
    revision: String,
    decision: ResearchDecision,
    formula_family_id: String,
    material_family: String,
    object_family: String,
    geometry_family: String,
    support_condition: String,
    geometry_scale_metres: NumericRange,
    relative_impact_speed_metres_per_second: NumericRange,
    #[serde(default)]
    impact_impulse_newton_seconds: Option<NumericRange>,
    impact_position_ids: Vec<String>,
    listener_condition_ids: Vec<String>,
    parameter_set: FileRef,
    corpus_manifest: FileRef,
    #[serde(default)]
    validator_evidence: Option<FileRef>,
    fallback_clip: FileRef,
    fallback_provenance: FileRef,
    #[serde(default)]
    cost_evidence: Option<CostEvidence>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum ResearchDecision {
    Candidate,
    Reject,
    FallbackOutOfDomain,
}

impl ResearchDecision {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "Candidate",
            Self::Reject => "Reject",
            Self::FallbackOutOfDomain => "FallbackOutOfDomain",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NumericRange {
    minimum: f64,
    maximum: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CostEvidence {
    maximum_modes_per_voice: usize,
    maximum_voices: usize,
    rendered_samples_per_window: usize,
    report: FileRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileRef {
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct RegistryReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    registry_id: String,
    manifest_sha256: String,
    formula_family_count: usize,
    domain_count: usize,
    decision_counts: Vec<DecisionCountReport>,
    formula_families: Vec<FormulaFamilyReport>,
    domains: Vec<DomainReport>,
}

#[derive(Debug, Serialize)]
struct DecisionCountReport {
    decision: &'static str,
    count: usize,
}

#[derive(Debug, Serialize)]
struct FormulaFamilyReport {
    id: String,
    revision: String,
    source_class: &'static str,
    equation_id: String,
    model_definition: ArtifactReport,
    parameter_schema: ArtifactReport,
}

#[derive(Debug, Serialize)]
struct DomainReport {
    id: String,
    revision: String,
    decision: &'static str,
    formula_family_id: String,
    material_family: String,
    object_family: String,
    geometry_family: String,
    support_condition: String,
    geometry_scale_metres: NumericRange,
    relative_impact_speed_metres_per_second: NumericRange,
    impact_impulse_newton_seconds: Option<NumericRange>,
    impact_position_ids: Vec<String>,
    listener_condition_ids: Vec<String>,
    parameter_set: ArtifactReport,
    corpus_manifest: ArtifactReport,
    validator_evidence: Option<ArtifactReport>,
    fallback_clip: ArtifactReport,
    fallback_provenance: ArtifactReport,
    cost_evidence: Option<CostEvidenceReport>,
}

#[derive(Debug, Serialize)]
struct CostEvidenceReport {
    maximum_modes_per_voice: usize,
    maximum_voices: usize,
    rendered_samples_per_window: usize,
    report: ArtifactReport,
}

#[derive(Debug, Serialize)]
struct ArtifactReport {
    sha256: String,
    byte_count: usize,
}

pub(super) fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path = canonical_external_file(&root, &manifest_path, "manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "manifest")?;
    let manifest: RegistryManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "manifest has no parent directory".to_owned())?;
    let report = build_report(&root, manifest_directory, manifest, &manifest_bytes)?;
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

fn validate_manifest(manifest: &RegistryManifest) -> Result<(), String> {
    if manifest.schema != SCHEMA {
        return Err(format!(
            "unsupported physical sound research registry schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.registry_id, "registry id")?;
    if manifest.formula_families.is_empty()
        || manifest.formula_families.len() > MAX_FORMULA_FAMILIES
    {
        return Err(format!(
            "formula family count must be 1..={MAX_FORMULA_FAMILIES}"
        ));
    }
    if manifest.domains.is_empty() || manifest.domains.len() > MAX_DOMAINS {
        return Err(format!("domain count must be 1..={MAX_DOMAINS}"));
    }

    let mut formula_ids = BTreeSet::new();
    let mut previous_formula: Option<&str> = None;
    for formula in &manifest.formula_families {
        validate_label(&formula.id, "formula family id")?;
        validate_label(&formula.revision, "formula family revision")?;
        validate_label(&formula.equation_id, "formula equation id")?;
        validate_file_ref(&formula.model_definition, "formula model definition")?;
        validate_file_ref(&formula.parameter_schema, "formula parameter schema")?;
        if previous_formula.is_some_and(|previous| previous >= formula.id.as_str()) {
            return Err(format!(
                "formula families must be strictly sorted by id; offending id {}",
                formula.id
            ));
        }
        previous_formula = Some(&formula.id);
        formula_ids.insert(formula.id.as_str());
    }

    let mut previous_domain: Option<&str> = None;
    for domain in &manifest.domains {
        validate_label(&domain.id, "domain id")?;
        validate_label(&domain.revision, "domain revision")?;
        validate_label(&domain.formula_family_id, "domain formula family id")?;
        validate_label(&domain.material_family, "material family")?;
        validate_label(&domain.object_family, "object family")?;
        validate_label(&domain.geometry_family, "geometry family")?;
        validate_label(&domain.support_condition, "support condition")?;
        if previous_domain.is_some_and(|previous| previous >= domain.id.as_str()) {
            return Err(format!(
                "domains must be strictly sorted by id; offending id {}",
                domain.id
            ));
        }
        previous_domain = Some(&domain.id);
        if !formula_ids.contains(domain.formula_family_id.as_str()) {
            return Err(format!(
                "domain {} references missing formula family {}",
                domain.id, domain.formula_family_id
            ));
        }
        validate_range(domain.geometry_scale_metres, "geometry_scale_metres", true)?;
        validate_range(
            domain.relative_impact_speed_metres_per_second,
            "relative_impact_speed_metres_per_second",
            false,
        )?;
        if let Some(range) = domain.impact_impulse_newton_seconds {
            validate_range(range, "impact_impulse_newton_seconds", false)?;
        }
        validate_sorted_labels(&domain.impact_position_ids, "impact position ids")?;
        validate_sorted_labels(&domain.listener_condition_ids, "listener condition ids")?;
        validate_file_ref(&domain.parameter_set, "domain parameter set")?;
        validate_file_ref(&domain.corpus_manifest, "domain corpus manifest")?;
        validate_file_ref(&domain.fallback_clip, "domain fallback clip")?;
        validate_file_ref(&domain.fallback_provenance, "domain fallback provenance")?;
        if domain.decision != ResearchDecision::Candidate && domain.validator_evidence.is_none() {
            return Err(format!(
                "domain {} decision {} requires validator evidence",
                domain.id,
                domain.decision.as_str()
            ));
        }
        if let Some(evidence) = &domain.validator_evidence {
            validate_file_ref(evidence, "domain validator evidence")?;
        }
        if let Some(cost) = &domain.cost_evidence {
            if cost.maximum_modes_per_voice == 0
                || cost.maximum_modes_per_voice > 16_384
                || cost.maximum_voices == 0
                || cost.maximum_voices > 4_096
                || cost.rendered_samples_per_window == 0
                || cost.rendered_samples_per_window > 16_777_216
            {
                return Err(format!("domain {} has invalid cost bounds", domain.id));
            }
            validate_file_ref(&cost.report, "domain cost report")?;
        }
    }
    Ok(())
}

fn validate_range(range: NumericRange, role: &str, strictly_positive: bool) -> Result<(), String> {
    if !range.minimum.is_finite()
        || !range.maximum.is_finite()
        || range.minimum >= range.maximum
        || (strictly_positive && range.minimum <= 0.0)
        || (!strictly_positive && range.minimum < 0.0)
    {
        return Err(format!("{role} must be a finite increasing bounded range"));
    }
    Ok(())
}

fn validate_sorted_labels(values: &[String], role: &str) -> Result<(), String> {
    if values.is_empty() || values.len() > MAX_CONDITIONS_PER_AXIS {
        return Err(format!(
            "{role} count must be 1..={MAX_CONDITIONS_PER_AXIS}"
        ));
    }
    let mut previous: Option<&str> = None;
    for value in values {
        validate_label(value, role)?;
        if previous.is_some_and(|prior| prior >= value.as_str()) {
            return Err(format!("{role} must be strictly sorted"));
        }
        previous = Some(value);
    }
    Ok(())
}

fn validate_label(value: &str, role: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-._".contains(&byte)
        })
    {
        return Err(format!(
            "{role} must be 1..=128 lowercase ASCII label characters"
        ));
    }
    Ok(())
}

fn validate_file_ref(file: &FileRef, role: &str) -> Result<(), String> {
    if file.path.is_empty() || file.path.len() > 4_096 || file.path.contains('\0') {
        return Err(format!("{role} path is invalid"));
    }
    if file.sha256.len() != 64
        || !file
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{role} sha256 must be 64 lowercase hex digits"));
    }
    Ok(())
}

fn build_report(
    root: &Path,
    manifest_directory: &Path,
    manifest: RegistryManifest,
    manifest_bytes: &[u8],
) -> Result<RegistryReport, String> {
    let mut formula_reports = Vec::with_capacity(manifest.formula_families.len());
    for formula in &manifest.formula_families {
        formula_reports.push(FormulaFamilyReport {
            id: formula.id.clone(),
            revision: formula.revision.clone(),
            source_class: formula.source_class.as_str(),
            equation_id: formula.equation_id.clone(),
            model_definition: resolve_artifact(
                root,
                manifest_directory,
                &formula.model_definition,
                "formula model definition",
            )?,
            parameter_schema: resolve_artifact(
                root,
                manifest_directory,
                &formula.parameter_schema,
                "formula parameter schema",
            )?,
        });
    }

    let mut decision_counts = BTreeMap::<ResearchDecision, usize>::new();
    let mut domain_reports = Vec::with_capacity(manifest.domains.len());
    for domain in manifest.domains {
        *decision_counts.entry(domain.decision).or_default() += 1;
        let cost_evidence = domain
            .cost_evidence
            .map(|cost| -> Result<CostEvidenceReport, String> {
                Ok(CostEvidenceReport {
                    maximum_modes_per_voice: cost.maximum_modes_per_voice,
                    maximum_voices: cost.maximum_voices,
                    rendered_samples_per_window: cost.rendered_samples_per_window,
                    report: resolve_artifact(
                        root,
                        manifest_directory,
                        &cost.report,
                        "domain cost report",
                    )?,
                })
            })
            .transpose()?;
        domain_reports.push(DomainReport {
            id: domain.id,
            revision: domain.revision,
            decision: domain.decision.as_str(),
            formula_family_id: domain.formula_family_id,
            material_family: domain.material_family,
            object_family: domain.object_family,
            geometry_family: domain.geometry_family,
            support_condition: domain.support_condition,
            geometry_scale_metres: domain.geometry_scale_metres,
            relative_impact_speed_metres_per_second: domain.relative_impact_speed_metres_per_second,
            impact_impulse_newton_seconds: domain.impact_impulse_newton_seconds,
            impact_position_ids: domain.impact_position_ids,
            listener_condition_ids: domain.listener_condition_ids,
            parameter_set: resolve_artifact(
                root,
                manifest_directory,
                &domain.parameter_set,
                "domain parameter set",
            )?,
            corpus_manifest: resolve_artifact(
                root,
                manifest_directory,
                &domain.corpus_manifest,
                "domain corpus manifest",
            )?,
            validator_evidence: domain
                .validator_evidence
                .as_ref()
                .map(|reference| {
                    resolve_artifact(
                        root,
                        manifest_directory,
                        reference,
                        "domain validator evidence",
                    )
                })
                .transpose()?,
            fallback_clip: resolve_artifact(
                root,
                manifest_directory,
                &domain.fallback_clip,
                "domain fallback clip",
            )?,
            fallback_provenance: resolve_artifact(
                root,
                manifest_directory,
                &domain.fallback_provenance,
                "domain fallback provenance",
            )?,
            cost_evidence,
        });
    }

    Ok(RegistryReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "NoAcceptanceAuthority",
        claim: "HASH_CLOSED_RESEARCH_INDEX_ONLY / PASS_DISABLED_UNTIL_AV_P0C",
        registry_id: manifest.registry_id,
        manifest_sha256: sha256_hex(manifest_bytes),
        formula_family_count: formula_reports.len(),
        domain_count: domain_reports.len(),
        decision_counts: [
            ResearchDecision::Candidate,
            ResearchDecision::Reject,
            ResearchDecision::FallbackOutOfDomain,
        ]
        .into_iter()
        .map(|decision| DecisionCountReport {
            decision: decision.as_str(),
            count: decision_counts.get(&decision).copied().unwrap_or(0),
        })
        .collect(),
        formula_families: formula_reports,
        domains: domain_reports,
    })
}

fn resolve_artifact(
    root: &Path,
    manifest_directory: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<ArtifactReport, String> {
    let path = canonical_external_file(root, &manifest_directory.join(&reference.path), role)?;
    let metadata =
        fs::metadata(&path).map_err(|error| format!("stat {role} {}: {error}", path.display()))?;
    if metadata.len() > MAX_REFERENCED_FILE_BYTES as u64 {
        return Err(format!(
            "{role} exceeds {MAX_REFERENCED_FILE_BYTES} bytes: {}",
            path.display()
        ));
    }
    let mut file = fs::File::open(&path)
        .map_err(|error| format!("open {role} {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut byte_count = 0_usize;
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read {role} {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        byte_count = byte_count
            .checked_add(count)
            .ok_or_else(|| format!("{role} byte count overflow"))?;
        if byte_count > MAX_REFERENCED_FILE_BYTES {
            return Err(format!(
                "{role} exceeds {MAX_REFERENCED_FILE_BYTES} bytes: {}",
                path.display()
            ));
        }
        hasher.update(&buffer[..count]);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    let actual_hash = ContentHash::from_bytes(digest).to_hex();
    if actual_hash != reference.sha256 {
        return Err(format!(
            "{role} hash mismatch for {}: expected {}, got {actual_hash}",
            path.display(),
            reference.sha256
        ));
    }
    Ok(ArtifactReport {
        sha256: actual_hash,
        byte_count,
    })
}

fn resolve_cli_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn resolve_output_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let unresolved = resolve_cli_path(root, path);
    if unresolved.exists() {
        let resolved = fs::canonicalize(&unresolved)
            .map_err(|error| format!("canonicalize {}: {error}", unresolved.display()))?;
        if resolved.starts_with(root) {
            return Err(format!(
                "physical-sound-registry output must stay outside the repository: {}",
                resolved.display()
            ));
        }
        return Ok(resolved);
    }
    let parent = unresolved
        .parent()
        .ok_or_else(|| "output path has no parent directory".to_owned())?;
    let parent = fs::canonicalize(parent)
        .map_err(|error| format!("canonicalize output parent {}: {error}", parent.display()))?;
    let name = unresolved
        .file_name()
        .ok_or_else(|| "output path has no directory name".to_owned())?;
    let resolved = parent.join(name);
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-registry output must stay outside the repository: {}",
            resolved.display()
        ));
    }
    Ok(resolved)
}

fn canonical_external_file(root: &Path, path: &Path, role: &str) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(path)
        .map_err(|error| format!("canonicalize {role} {}: {error}", path.display()))?;
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-registry {role} must stay outside the repository: {}",
            resolved.display()
        ));
    }
    let metadata = fs::metadata(&resolved)
        .map_err(|error| format!("stat {role} {}: {error}", resolved.display()))?;
    if !metadata.is_file() {
        return Err(format!("{role} is not a file: {}", resolved.display()));
    }
    Ok(resolved)
}

fn require_empty_output(output: &Path) -> Result<(), String> {
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir() {
        return Err(format!("output is not a directory: {}", output.display()));
    }
    if fs::read_dir(output)
        .map_err(|error| format!("read {}: {error}", output.display()))?
        .next()
        .is_some()
    {
        return Err(format!(
            "physical-sound-registry output directory must be empty: {}",
            output.display()
        ));
    }
    Ok(())
}

fn read_bounded_file(path: &Path, maximum_bytes: usize, role: &str) -> Result<Vec<u8>, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("stat {role} {}: {error}", path.display()))?;
    if metadata.len() > maximum_bytes as u64 {
        return Err(format!(
            "{role} exceeds {maximum_bytes} bytes: {}",
            path.display()
        ));
    }
    fs::read(path).map_err(|error| format!("read {role} {}: {error}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    ContentHash::from_bytes(sha256(bytes)).to_hex()
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
                "nextengine-physical-sound-registry-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create registry test directory");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if self.path.is_dir() {
                fs::remove_dir_all(&self.path).expect("remove registry test directory");
            }
        }
    }

    #[test]
    fn arguments_require_manifest_and_output() {
        let request = parse_arguments(
            [
                "--manifest".to_owned(),
                "/tmp/registry.json".to_owned(),
                "--output".to_owned(),
                "/tmp/registry-report".to_owned(),
            ]
            .into_iter(),
        )
        .expect("arguments parse");
        assert_eq!(request.manifest, PathBuf::from("/tmp/registry.json"));
        assert!(parse_arguments(std::iter::empty()).is_err());
        assert!(parse_arguments(["--unknown".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn manifest_rejects_missing_formula_invalid_range_and_unbacked_decision() {
        let mut manifest = test_manifest();
        manifest.domains[0].formula_family_id = "missing".to_owned();
        assert!(
            validate_manifest(&manifest)
                .expect_err("missing formula rejects")
                .contains("missing formula family")
        );

        let mut manifest = test_manifest();
        manifest.domains[0].geometry_scale_metres.maximum =
            manifest.domains[0].geometry_scale_metres.minimum;
        assert!(
            validate_manifest(&manifest)
                .expect_err("empty range rejects")
                .contains("finite increasing")
        );

        let mut manifest = test_manifest();
        manifest.domains[0].decision = ResearchDecision::FallbackOutOfDomain;
        assert!(
            validate_manifest(&manifest)
                .expect_err("unbacked decision rejects")
                .contains("requires validator evidence")
        );
    }

    #[test]
    fn pass_is_not_a_v1_registry_decision() {
        let mut value = serde_json::to_value(test_manifest()).expect("serialize manifest");
        value["domains"][0]["decision"] = Value::String("pass".to_owned());
        assert!(serde_json::from_value::<RegistryManifest>(value).is_err());
    }

    #[test]
    fn end_to_end_registry_is_hash_closed_external_and_repeatable() {
        let directory = TestDirectory::new();
        let mut manifest = test_manifest();
        write_artifacts(&directory.path, &mut manifest);
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("serialize manifest");
        let manifest_path = directory.path.join("registry.json");
        fs::write(&manifest_path, manifest_bytes).expect("write registry manifest");
        let output = directory.path.join("report");
        let root = workspace_root();
        run(
            root,
            &Request {
                manifest: manifest_path.clone(),
                output: output.clone(),
            },
        )
        .expect("registry validates");

        let report: Value =
            serde_json::from_slice(&fs::read(output.join("report.json")).expect("read report"))
                .expect("parse report");
        assert_eq!(report["status"], "Validated");
        assert_eq!(report["decision"], "NoAcceptanceAuthority");
        assert_eq!(report["domain_count"], 1);
        assert_eq!(report["domains"][0]["decision"], "Candidate");

        let repeated = directory.path.join("report-repeated");
        run(
            root,
            &Request {
                manifest: manifest_path,
                output: repeated.clone(),
            },
        )
        .expect("registry repeats");
        assert_eq!(
            fs::read(output.join("report.json")).expect("read first report"),
            fs::read(repeated.join("report.json")).expect("read repeated report")
        );

        let mut invalid = manifest;
        invalid.domains[0].parameter_set.sha256 = "0".repeat(64);
        let invalid_bytes = serde_json::to_vec_pretty(&invalid).expect("serialize invalid");
        let invalid_path = directory.path.join("invalid.json");
        fs::write(&invalid_path, invalid_bytes).expect("write invalid manifest");
        assert!(
            run(
                root,
                &Request {
                    manifest: invalid_path,
                    output: directory.path.join("invalid-report"),
                }
            )
            .expect_err("hash mismatch rejects")
            .contains("hash mismatch")
        );
    }

    #[test]
    fn repository_local_research_artifact_is_rejected() {
        let directory = TestDirectory::new();
        let mut manifest = test_manifest();
        write_artifacts(&directory.path, &mut manifest);
        let repository_file = workspace_root().join("Cargo.toml");
        let repository_bytes = fs::read(&repository_file).expect("read repository Cargo.toml");
        manifest.domains[0].parameter_set = FileRef {
            path: repository_file.display().to_string(),
            sha256: sha256_hex(&repository_bytes),
        };
        let manifest_path = directory.path.join("repository-local.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");

        assert!(
            run(
                workspace_root(),
                &Request {
                    manifest: manifest_path,
                    output: directory.path.join("report"),
                },
            )
            .expect_err("repository-local artifact rejects")
            .contains("must stay outside the repository")
        );
    }

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
    }

    fn write_artifacts(directory: &Path, manifest: &mut RegistryManifest) {
        let artifacts = [
            ("model.txt", b"modal sum plus bounded residual".as_slice()),
            (
                "parameters.schema.json",
                b"{\"type\":\"object\"}".as_slice(),
            ),
            ("parameters.json", b"{\"modes\":16}".as_slice()),
            ("corpus.json", b"{\"corpus\":\"external\"}".as_slice()),
            ("fallback.wav", b"test fallback bytes".as_slice()),
            (
                "fallback.provenance.txt",
                b"engine-owned test fixture".as_slice(),
            ),
        ];
        for (name, bytes) in artifacts {
            fs::write(directory.join(name), bytes).expect("write artifact");
        }
        manifest.formula_families[0].model_definition.sha256 =
            sha256_hex(&fs::read(directory.join("model.txt")).expect("read model"));
        manifest.formula_families[0].parameter_schema.sha256 =
            sha256_hex(&fs::read(directory.join("parameters.schema.json")).expect("read schema"));
        manifest.domains[0].parameter_set.sha256 =
            sha256_hex(&fs::read(directory.join("parameters.json")).expect("read parameters"));
        manifest.domains[0].corpus_manifest.sha256 =
            sha256_hex(&fs::read(directory.join("corpus.json")).expect("read corpus"));
        manifest.domains[0].fallback_clip.sha256 =
            sha256_hex(&fs::read(directory.join("fallback.wav")).expect("read fallback"));
        manifest.domains[0].fallback_provenance.sha256 = sha256_hex(
            &fs::read(directory.join("fallback.provenance.txt")).expect("read provenance"),
        );
    }

    fn test_manifest() -> RegistryManifest {
        RegistryManifest {
            schema: SCHEMA.to_owned(),
            registry_id: "physical-sound-p0".to_owned(),
            formula_families: vec![FormulaFamily {
                id: "modal-residual".to_owned(),
                revision: "v1".to_owned(),
                source_class: SourceClass::RigidImpact,
                equation_id: "damped-modal-sum-with-bounded-residual".to_owned(),
                model_definition: file_ref("model.txt"),
                parameter_schema: file_ref("parameters.schema.json"),
            }],
            domains: vec![DomainRecord {
                id: "thin-steel-vessel-impact".to_owned(),
                revision: "v1".to_owned(),
                decision: ResearchDecision::Candidate,
                formula_family_id: "modal-residual".to_owned(),
                material_family: "steel".to_owned(),
                object_family: "thin-vessel".to_owned(),
                geometry_family: "axisymmetric-shell".to_owned(),
                support_condition: "freely-supported".to_owned(),
                geometry_scale_metres: NumericRange {
                    minimum: 0.05,
                    maximum: 0.5,
                },
                relative_impact_speed_metres_per_second: NumericRange {
                    minimum: 0.1,
                    maximum: 5.0,
                },
                impact_impulse_newton_seconds: None,
                impact_position_ids: vec!["rim".to_owned(), "wall".to_owned()],
                listener_condition_ids: vec!["fixed-near-field".to_owned()],
                parameter_set: file_ref("parameters.json"),
                corpus_manifest: file_ref("corpus.json"),
                validator_evidence: None,
                fallback_clip: file_ref("fallback.wav"),
                fallback_provenance: file_ref("fallback.provenance.txt"),
                cost_evidence: None,
            }],
        }
    }

    fn file_ref(path: &str) -> FileRef {
        FileRef {
            path: path.to_owned(),
            sha256: "a".repeat(64),
        }
    }
}
