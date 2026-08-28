use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    FileRef, MAX_MANIFEST_BYTES, canonical_external_file, read_bounded_file, require_empty_output,
    resolve_cli_path, resolve_output_path, set_once, sha256_hex,
};

const MANIFEST_SCHEMA: &str =
    "nextengine.experimental-physical-sound-source-feasibility.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-source-feasibility.report.v1";
const DOMAIN_REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-domain-claims.report.v1";
const STUDY_ID: &str = "physical-sound-ps2-internet-source-feasibility";
const STUDY_REVISION: &str = "v1";
const DOMAIN_PROFILE_ID: &str = "thin-soda-lime-glass-open-vessel-impact-v1";
const DOMAIN_DECISION: &str = "DomainEvidenceIncomplete";
const REPORT_CLAIM: &str = "BOUNDED_PUBLISHED_SOURCE_FEASIBILITY_AND_NEXT_CALIBRATION_ROUTE_ONLY / NO_EXACT_DOMAIN_CLOSURE_VALIDATOR_RELEASE_OR_CORPUS_ADMISSION_AUTHORITY";
const REVIEWED_SET_DECISION: &str = "ReviewedSourcesCannotCloseV1";
const NEXT_ROUTE_DECISION: &str = "InternetNativeTransferCandidate";
const NEXT_PROFILE_ID: &str = "realimpact-normalized-transfer-calibration-v1";

const V1_BLOCKERS: [&str; 8] = [
    "material_composition_revision_unavailable",
    "geometry_revision_not_domain_aligned",
    "support_fixture_revision_unavailable",
    "absolute_excitation_profile_unavailable",
    "impact_condition_not_domain_aligned",
    "listener_condition_not_domain_aligned",
    "matched_cross_tier_condition_identity_unavailable",
    "four_partition_exact_domain_coverage_incomplete",
];

const TRANSFER_CAPABILITIES: [&str; 5] = [
    "force_deconvolved_transfer",
    "impact_vertex_identity",
    "listener_grid_identity",
    "real_object_identity",
    "scanned_mesh_geometry",
];

const TRANSFER_PROHIBITIONS: [&str; 7] = [
    "absolute_amplitude_claim",
    "exact_material_composition_claim",
    "exact_support_fixture_claim",
    "matched_cross_tier_condition_claim",
    "physical_sound_pass",
    "production_corpus_admission",
    "runtime_content_role",
];

const RECONSIDERATION_CONDITIONS: [&str; 3] = [
    "realimpact_raw_force_archive_is_published_with_stable_per-recording-lineage",
    "av-msf_code_and_training-data-lineage_are_published",
    "a_primary_source_publishes_exact_v1_composition-geometry-support-impact-listener-lineage",
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
            _ => return Err(format!("unexpected source-feasibility argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry source-feasibility requires --manifest <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry source-feasibility requires --output <external-empty-directory>"
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
        "source feasibility manifest",
    )?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;
    let manifest_bytes = read_bounded_file(
        &manifest_path,
        MAX_MANIFEST_BYTES,
        "source feasibility manifest",
    )?;
    let manifest: SourceFeasibilityManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| "source feasibility manifest has no parent directory".to_owned())?;

    let domain_bytes = read_reference(
        &root,
        base,
        &manifest.domain_claims_report,
        "domain claims report",
    )?;
    let domain: DomainClaimsReport = serde_json::from_slice(&domain_bytes)
        .map_err(|error| format!("parse domain claims report: {error}"))?;
    validate_domain_report(&domain)?;

    let sources = manifest
        .sources
        .iter()
        .map(|source| validate_source(&root, base, source))
        .collect::<Result<Vec<_>, _>>()?;
    let report = build_report(
        sha256_hex(&manifest_bytes),
        sha256_hex(&domain_bytes),
        sources,
    );
    let report_json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    fs::create_dir_all(&output).map_err(|error| format!("create {}: {error}", output.display()))?;
    fs::write(output.join("report.json"), &report_json)
        .map_err(|error| format!("write source feasibility report: {error}"))?;
    println!(
        "{}",
        String::from_utf8(report_json).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFeasibilityManifest {
    schema: String,
    study_id: String,
    revision: String,
    reviewed_at: String,
    domain_claims_report: FileRef,
    sources: Vec<SourceDeclaration>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDeclaration {
    source_id: String,
    profile_id: String,
    artifacts: Vec<SourceArtifact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceArtifact {
    role: String,
    primary_url: String,
    artifact: FileRef,
}

#[derive(Deserialize)]
struct DomainClaimsReport {
    schema: String,
    status: String,
    decision: String,
    claim: String,
    profile_id: String,
    blockers: Vec<String>,
}

struct SourceProfile {
    source_id: &'static str,
    profile_id: &'static str,
    disposition: &'static str,
    required_artifacts: &'static [ArtifactRequirement],
    supported_capabilities: &'static [&'static str],
    limitations: &'static [&'static str],
}

struct ArtifactRequirement {
    role: &'static str,
    primary_url: &'static str,
}

const REALIMPACT_ARTIFACTS: [ArtifactRequirement; 3] = [
    ArtifactRequirement {
        role: "paper",
        primary_url: "https://ai.stanford.edu/~rhgao/publications/RealImpact.pdf",
    },
    ArtifactRequirement {
        role: "repository_readme",
        primary_url: "https://raw.githubusercontent.com/samuel-clarke/RealImpact/main/README.md",
    },
    ArtifactRequirement {
        role: "raw_release_issue",
        primary_url: "https://api.github.com/repos/samuel-clarke/RealImpact/issues/3",
    },
];

const OBJECTFOLDER_ARTIFACTS: [ArtifactRequirement; 2] = [
    ArtifactRequirement {
        role: "benchmark_paper",
        primary_url: "https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf",
    },
    ArtifactRequirement {
        role: "official_download_page",
        primary_url: "https://objectfolder.stanford.edu/objectfolder-real-download",
    },
];

const AV_MSF_ARTIFACTS: [ArtifactRequirement; 2] = [
    ArtifactRequirement {
        role: "project_page",
        primary_url: "https://zisenshao.github.io/AV-MSF/",
    },
    ArtifactRequirement {
        role: "repository_readme",
        primary_url: "https://raw.githubusercontent.com/ZisenShao/AV-MSF/main/README.md",
    },
];

const SOURCE_PROFILES: [SourceProfile; 3] = [
    SourceProfile {
        source_id: "realimpact",
        profile_id: "realimpact-public-preprocessed-2026-08-28-v1",
        disposition: "SelectedTransferCalibration",
        required_artifacts: &REALIMPACT_ARTIFACTS,
        supported_capabilities: &[
            "force_deconvolved_transfer",
            "impact_vertex_identity",
            "listener_grid_identity",
            "real_object_identity",
            "scanned_mesh_geometry",
        ],
        limitations: &[
            "broad_material_family_only",
            "raw_force_archive_not_published",
            "support_fixture_not_v1_aligned",
            "listener_grid_not_v1_aligned",
            "no_cross_tier_repeat_lineage",
        ],
    },
    SourceProfile {
        source_id: "objectfolder-real",
        profile_id: "objectfolder-real-public-2026-08-28-v1",
        disposition: "DeferredForceCalibration",
        required_artifacts: &OBJECTFOLDER_ARTIFACTS,
        supported_capabilities: &[
            "contact_force_profile",
            "impact_coordinate",
            "real_recording",
            "scanned_mesh_geometry",
        ],
        limitations: &[
            "broad_material_family_only",
            "per-object-support-revision-not-published-in-reviewed-metadata",
            "listener-condition-not-v1-aligned",
            "large-non-seekable-audio-archives",
            "no-project-disjoint-exact-domain-partitions",
        ],
    },
    SourceProfile {
        source_id: "av-msf",
        profile_id: "av-msf-public-2026-08-28-v1",
        disposition: "UnavailableCodeOnly",
        required_artifacts: &AV_MSF_ARTIFACTS,
        supported_capabilities: &["published_method_description"],
        limitations: &[
            "code-coming-soon",
            "no-published-training-data-lineage",
            "does-not-add-independent-acquisition-lineage",
        ],
    },
];

fn validate_manifest(manifest: &SourceFeasibilityManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.study_id != STUDY_ID
        || manifest.revision != STUDY_REVISION
        || manifest.reviewed_at != "2026-08-28"
    {
        return Err("source feasibility manifest does not select the frozen V1 study".to_owned());
    }
    let expected = SOURCE_PROFILES
        .iter()
        .map(|profile| profile.source_id)
        .collect::<BTreeSet<_>>();
    let observed = manifest
        .sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect::<BTreeSet<_>>();
    if observed.len() != manifest.sources.len() || observed != expected {
        return Err(
            "source feasibility manifest must contain each frozen source exactly once".to_owned(),
        );
    }
    Ok(())
}

fn validate_domain_report(report: &DomainClaimsReport) -> Result<(), String> {
    if report.schema != DOMAIN_REPORT_SCHEMA
        || report.status != "Validated"
        || report.decision != DOMAIN_DECISION
        || report.profile_id != DOMAIN_PROFILE_ID
        || report.claim
            != "EXACT_DOMAIN_E2_E3_CLAIM_MATRIX_ONLY / NO_VALIDATOR_RELEASE_OR_CORPUS_ADMISSION_AUTHORITY"
    {
        return Err(
            "source feasibility requires the frozen incomplete V1 domain report".to_owned(),
        );
    }
    let expected = V1_BLOCKERS.into_iter().collect::<BTreeSet<_>>();
    let observed = report
        .blockers
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if observed.len() != report.blockers.len() || observed != expected {
        return Err("domain report does not contain the frozen eight V1 blockers".to_owned());
    }
    Ok(())
}

fn validate_source(
    root: &Path,
    base: &Path,
    source: &SourceDeclaration,
) -> Result<SourceReport, String> {
    let profile = SOURCE_PROFILES
        .iter()
        .find(|profile| profile.source_id == source.source_id)
        .ok_or_else(|| format!("unknown source id: {}", source.source_id))?;
    if source.profile_id != profile.profile_id {
        return Err(format!(
            "source {} does not select frozen profile {}",
            source.source_id, profile.profile_id
        ));
    }
    if source.artifacts.len() != profile.required_artifacts.len() {
        return Err(format!(
            "source {} requires {} primary artifacts",
            source.source_id,
            profile.required_artifacts.len()
        ));
    }
    let roles = source
        .artifacts
        .iter()
        .map(|artifact| artifact.role.as_str())
        .collect::<BTreeSet<_>>();
    if roles.len() != source.artifacts.len() {
        return Err(format!(
            "source {} repeats an artifact role",
            source.source_id
        ));
    }

    let artifacts = profile
        .required_artifacts
        .iter()
        .map(|requirement| {
            let declaration = source
                .artifacts
                .iter()
                .find(|artifact| artifact.role == requirement.role)
                .ok_or_else(|| {
                    format!(
                        "source {} is missing artifact role {}",
                        source.source_id, requirement.role
                    )
                })?;
            if declaration.primary_url != requirement.primary_url {
                return Err(format!(
                    "source {} artifact {} must use frozen primary URL {}",
                    source.source_id, requirement.role, requirement.primary_url
                ));
            }
            let bytes = read_reference(
                root,
                base,
                &declaration.artifact,
                &format!("{} {}", source.source_id, requirement.role),
            )?;
            Ok(ArtifactReport {
                role: requirement.role,
                primary_url: requirement.primary_url,
                sha256: sha256_hex(&bytes),
                byte_count: bytes.len(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(SourceReport {
        source_id: profile.source_id,
        profile_id: profile.profile_id,
        disposition: profile.disposition,
        artifacts,
        supported_capabilities: profile.supported_capabilities,
        limitations: profile.limitations,
        closes_v1_blockers: Vec::new(),
    })
}

fn read_reference(
    root: &Path,
    base: &Path,
    reference: &FileRef,
    role: &str,
) -> Result<Vec<u8>, String> {
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
    Ok(bytes)
}

#[derive(Serialize)]
struct SourceFeasibilityReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    study_id: &'static str,
    revision: &'static str,
    domain_profile_id: &'static str,
    manifest_sha256: String,
    domain_claims_report_sha256: String,
    reviewed_source_count: usize,
    closed_v1_blockers: Vec<&'static str>,
    open_v1_blockers: Vec<&'static str>,
    sources: Vec<SourceReport>,
    next_route: NextRoute,
    reconsideration_conditions: Vec<&'static str>,
}

#[derive(Serialize)]
struct SourceReport {
    source_id: &'static str,
    profile_id: &'static str,
    disposition: &'static str,
    artifacts: Vec<ArtifactReport>,
    supported_capabilities: &'static [&'static str],
    limitations: &'static [&'static str],
    closes_v1_blockers: Vec<&'static str>,
}

#[derive(Serialize)]
struct ArtifactReport {
    role: &'static str,
    primary_url: &'static str,
    sha256: String,
    byte_count: usize,
}

#[derive(Serialize)]
struct NextRoute {
    decision: &'static str,
    profile_id: &'static str,
    source_profile_id: &'static str,
    allowed_capabilities: Vec<&'static str>,
    prohibited_claims: Vec<&'static str>,
    smallest_next_action: &'static str,
}

fn build_report(
    manifest_sha256: String,
    domain_claims_report_sha256: String,
    sources: Vec<SourceReport>,
) -> SourceFeasibilityReport {
    SourceFeasibilityReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: REVIEWED_SET_DECISION,
        claim: REPORT_CLAIM,
        study_id: STUDY_ID,
        revision: STUDY_REVISION,
        domain_profile_id: DOMAIN_PROFILE_ID,
        manifest_sha256,
        domain_claims_report_sha256,
        reviewed_source_count: sources.len(),
        closed_v1_blockers: Vec::new(),
        open_v1_blockers: V1_BLOCKERS.to_vec(),
        sources,
        next_route: NextRoute {
            decision: NEXT_ROUTE_DECISION,
            profile_id: NEXT_PROFILE_ID,
            source_profile_id: "realimpact-public-preprocessed-2026-08-28-v1",
            allowed_capabilities: TRANSFER_CAPABILITIES.to_vec(),
            prohibited_claims: TRANSFER_PROHIBITIONS.to_vec(),
            smallest_next_action: "preregister-and-execute-realimpact-normalized-transfer-calibration-v1",
        },
        reconsideration_conditions: RECONSIDERATION_CONDITIONS.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> SourceFeasibilityManifest {
        SourceFeasibilityManifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            study_id: STUDY_ID.to_owned(),
            revision: STUDY_REVISION.to_owned(),
            reviewed_at: "2026-08-28".to_owned(),
            domain_claims_report: FileRef {
                path: "/tmp/domain-report.json".to_owned(),
                sha256: "a".repeat(64),
            },
            sources: SOURCE_PROFILES
                .iter()
                .map(|profile| SourceDeclaration {
                    source_id: profile.source_id.to_owned(),
                    profile_id: profile.profile_id.to_owned(),
                    artifacts: profile
                        .required_artifacts
                        .iter()
                        .map(|artifact| SourceArtifact {
                            role: artifact.role.to_owned(),
                            primary_url: artifact.primary_url.to_owned(),
                            artifact: FileRef {
                                path: format!("/tmp/{}-{}", profile.source_id, artifact.role),
                                sha256: "b".repeat(64),
                            },
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    fn domain_report() -> DomainClaimsReport {
        DomainClaimsReport {
            schema: DOMAIN_REPORT_SCHEMA.to_owned(),
            status: "Validated".to_owned(),
            decision: DOMAIN_DECISION.to_owned(),
            claim: "EXACT_DOMAIN_E2_E3_CLAIM_MATRIX_ONLY / NO_VALIDATOR_RELEASE_OR_CORPUS_ADMISSION_AUTHORITY".to_owned(),
            profile_id: DOMAIN_PROFILE_ID.to_owned(),
            blockers: V1_BLOCKERS.into_iter().map(str::to_owned).collect(),
        }
    }

    #[test]
    fn arguments_require_manifest_and_output() {
        let request = parse_arguments(
            [
                "--manifest".to_owned(),
                "/tmp/source-feasibility.json".to_owned(),
                "--output".to_owned(),
                "/tmp/source-feasibility-report".to_owned(),
            ]
            .into_iter(),
        )
        .expect("arguments parse");
        assert_eq!(
            request.manifest,
            PathBuf::from("/tmp/source-feasibility.json")
        );
        assert!(parse_arguments(std::iter::empty()).is_err());
    }

    #[test]
    fn frozen_manifest_requires_all_three_source_profiles() {
        let mut manifest = manifest();
        validate_manifest(&manifest).expect("frozen source set validates");
        manifest.sources.pop();
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn domain_report_requires_all_eight_open_blockers() {
        let mut report = domain_report();
        validate_domain_report(&report).expect("frozen domain report validates");
        report.blockers.pop();
        assert!(validate_domain_report(&report).is_err());
    }

    #[test]
    fn next_route_is_transfer_only_and_cannot_claim_pass() {
        let report = build_report("a".repeat(64), "b".repeat(64), Vec::new());
        assert_eq!(report.decision, REVIEWED_SET_DECISION);
        assert!(report.closed_v1_blockers.is_empty());
        assert_eq!(report.open_v1_blockers.len(), V1_BLOCKERS.len());
        assert_eq!(report.next_route.decision, NEXT_ROUTE_DECISION);
        assert!(
            report
                .next_route
                .prohibited_claims
                .contains(&"physical_sound_pass")
        );
        assert!(
            report
                .next_route
                .allowed_capabilities
                .contains(&"force_deconvolved_transfer")
        );
    }

    #[test]
    fn source_profile_rejects_substituted_primary_url_before_io() {
        let mut manifest = manifest();
        let source = &mut manifest.sources[0];
        source.artifacts[0].primary_url = "https://example.invalid/substitute".to_owned();
        let error = match validate_source(Path::new("/"), Path::new("/tmp"), source) {
            Ok(_) => panic!("substituted primary URL must reject"),
            Err(error) => error,
        };
        assert!(error.contains("frozen primary URL"));
    }
}
