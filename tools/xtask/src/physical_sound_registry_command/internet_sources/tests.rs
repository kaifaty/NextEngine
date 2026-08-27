use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use super::*;

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nextengine-physical-sound-internet-sources-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create internet source test directory");
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.0.is_dir() {
            fs::remove_dir_all(&self.0).expect("remove internet source test directory");
        }
    }
}

#[test]
fn arguments_require_external_cache_and_bound_downloads() {
    let request = parse_arguments(
        [
            "--manifest",
            "/tmp/sources.json",
            "--cache",
            "/tmp/source-cache",
            "--output",
            "/tmp/source-report",
            "--fetch-missing",
            "--maximum-download-bytes",
            "1024",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("arguments parse");
    assert!(request.fetch_missing);
    assert_eq!(request.maximum_download_bytes, 1024);
    assert!(parse_arguments(std::iter::empty()).is_err());
    assert!(
        parse_arguments(
            [
                "--manifest",
                "/tmp/sources.json",
                "--cache",
                "/tmp/cache",
                "--output",
                "/tmp/report",
                "--maximum-download-bytes",
                "0",
            ]
            .into_iter()
            .map(str::to_owned)
        )
        .is_err()
    );
}

#[test]
fn manifest_rejects_unsafe_url_missing_artifact_and_unsorted_capabilities() {
    let mut manifest = test_manifest();
    manifest.sources[0].artifacts[0].url = "http://example.org/audio.raw".to_owned();
    assert!(
        validate_manifest(&manifest)
            .expect_err("HTTP rejects")
            .contains("must use https")
    );

    let mut manifest = test_manifest();
    manifest.sources[0].capability_evidence[0].artifact_ids[0] = "missing".to_owned();
    assert!(
        validate_manifest(&manifest)
            .expect_err("missing artifact rejects")
            .contains("references missing artifact")
    );

    let mut manifest = test_manifest();
    manifest.sources[0].capability_evidence.swap(0, 1);
    assert!(
        validate_manifest(&manifest)
            .expect_err("unsorted capabilities reject")
            .contains("strictly sorted")
    );
}

#[test]
fn fetch_endpoint_policy_rejects_private_and_documentation_addresses() {
    assert!(!fetch::is_public_ip(IpAddr::V4(Ipv4Addr::LOCALHOST)));
    assert!(!fetch::is_public_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    assert!(!fetch::is_public_ip(IpAddr::V4(Ipv4Addr::new(
        203, 0, 113, 1
    ))));
    assert!(!fetch::is_public_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert!(!fetch::is_public_ip(IpAddr::V6(Ipv6Addr::new(
        0x2001, 0x0db8, 0, 0, 0, 0, 0, 1,
    ))));
    assert!(fetch::is_public_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
}

#[test]
fn cached_e4_source_is_hash_closed_claim_scoped_and_repeatable() {
    let directory = TestDirectory::new();
    let mut manifest = test_manifest();
    write_provenance(&directory.0, &mut manifest);
    let cache = directory.0.join("cache");
    fs::create_dir(&cache).expect("create cache");
    write_cached_artifacts(&cache, &mut manifest);
    let manifest_path = directory.0.join("sources.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize source manifest"),
    )
    .expect("write source manifest");

    let first = directory.0.join("first-report");
    run(
        workspace_root(),
        &Request {
            manifest: manifest_path.clone(),
            cache: cache.clone(),
            output: first.clone(),
            fetch_missing: false,
            maximum_download_bytes: DEFAULT_MAXIMUM_DOWNLOAD_BYTES,
        },
    )
    .expect("cached source validates");
    let report: Value =
        serde_json::from_slice(&fs::read(first.join("report.json")).expect("read source report"))
            .expect("parse source report");
    assert_eq!(report["decision"], "SourceSetComplete");
    assert_eq!(report["ready_source_count"], 1);
    assert_eq!(
        report["sources"][0]["supported_tiers"][0],
        "E4SyntheticGenerated"
    );
    assert_eq!(
        report["sources"][0]["artifacts"][0]["cache_status"],
        "CachedVerified"
    );

    let repeated = directory.0.join("repeated-report");
    run(
        workspace_root(),
        &Request {
            manifest: manifest_path,
            cache,
            output: repeated.clone(),
            fetch_missing: false,
            maximum_download_bytes: DEFAULT_MAXIMUM_DOWNLOAD_BYTES,
        },
    )
    .expect("cached source repeats");
    assert_eq!(
        fs::read(first.join("report.json")).expect("read first report"),
        fs::read(repeated.join("report.json")).expect("read repeated report")
    );
}

#[test]
fn source_without_integrity_metadata_is_discovery_only() {
    let directory = TestDirectory::new();
    let mut manifest = test_manifest();
    write_provenance(&directory.0, &mut manifest);
    for artifact in &mut manifest.sources[0].artifacts {
        artifact.expected_byte_count = None;
        artifact.expected_sha256 = None;
    }
    let manifest_path = directory.0.join("sources.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize source manifest"),
    )
    .expect("write source manifest");
    let output = directory.0.join("report");
    run(
        workspace_root(),
        &Request {
            manifest: manifest_path,
            cache: directory.0.join("cache"),
            output: output.clone(),
            fetch_missing: true,
            maximum_download_bytes: 1,
        },
    )
    .expect("discovery source reports");
    let report: Value =
        serde_json::from_slice(&fs::read(output.join("report.json")).expect("read source report"))
            .expect("parse source report");
    assert_eq!(report["decision"], "SourceSetIncomplete");
    assert_eq!(report["sources"][0]["source_status"], "DiscoveryOnly");
    assert_eq!(
        report["sources"][0]["artifacts"][0]["cache_status"],
        "MissingIntegrityMetadata"
    );
}

#[test]
fn corrupt_cached_artifact_rejects_without_report() {
    let directory = TestDirectory::new();
    let mut manifest = test_manifest();
    write_provenance(&directory.0, &mut manifest);
    let cache = directory.0.join("cache");
    fs::create_dir(&cache).expect("create cache");
    write_cached_artifacts(&cache, &mut manifest);
    let first_hash = manifest.sources[0].artifacts[0]
        .expected_sha256
        .as_ref()
        .expect("artifact hash");
    let target = cache
        .join("objects")
        .join(&first_hash[..2])
        .join(first_hash);
    fs::write(target, b"corrupt").expect("corrupt cache entry");
    let manifest_path = directory.0.join("sources.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize source manifest"),
    )
    .expect("write source manifest");
    let output = directory.0.join("report");
    assert!(
        run(
            workspace_root(),
            &Request {
                manifest: manifest_path,
                cache,
                output: output.clone(),
                fetch_missing: false,
                maximum_download_bytes: DEFAULT_MAXIMUM_DOWNLOAD_BYTES,
            },
        )
        .expect_err("corrupt cache rejects")
        .contains("byte count mismatch")
    );
    assert!(!output.exists());
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

fn write_provenance(directory: &Path, manifest: &mut InternetSourceManifest) {
    let bytes = b"official source review";
    fs::write(directory.join("provenance.md"), bytes).expect("write provenance review");
    manifest.sources[0].provenance_review.sha256 = sha256_hex(bytes);
}

fn write_cached_artifacts(cache: &Path, manifest: &mut InternetSourceManifest) {
    for (artifact, bytes) in manifest.sources[0]
        .artifacts
        .iter_mut()
        .zip([b"audio".as_slice(), b"metadata".as_slice()])
    {
        let hash = sha256_hex(bytes);
        artifact.expected_sha256 = Some(hash.clone());
        artifact.expected_byte_count = Some(bytes.len() as u64);
        let directory = cache.join("objects").join(&hash[..2]);
        fs::create_dir_all(&directory).expect("create cache object directory");
        fs::write(directory.join(hash), bytes).expect("write cache object");
    }
}

fn test_manifest() -> InternetSourceManifest {
    InternetSourceManifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        registry_id: "internet-source-test".to_owned(),
        revision: "v1".to_owned(),
        sources: vec![InternetSource {
            id: "identified-real-source".to_owned(),
            publisher_id: "example-publisher".to_owned(),
            project_id: "example-project".to_owned(),
            declared_revision: "commit-001".to_owned(),
            review_date: "2026-08-27".to_owned(),
            landing_page_url: "https://example.org/dataset".to_owned(),
            terms_url: Some("https://example.org/license".to_owned()),
            adapter_id: "hash-closed-synthetic-v1".to_owned(),
            license_expression: "CC-BY-4.0".to_owned(),
            redistribution_policy: RedistributionPolicy::ExternalResearchOnly,
            provenance_review: FileRef {
                path: "provenance.md".to_owned(),
                sha256: "a".repeat(64),
            },
            artifacts: vec![
                RemoteArtifact {
                    id: "generator".to_owned(),
                    role: ArtifactRole::SourceArchive,
                    url: "https://example.org/generator.tar".to_owned(),
                    maximum_bytes: 1024,
                    expected_byte_count: None,
                    expected_sha256: None,
                },
                RemoteArtifact {
                    id: "metadata".to_owned(),
                    role: ArtifactRole::Metadata,
                    url: "https://example.org/metadata.json".to_owned(),
                    maximum_bytes: 1024,
                    expected_byte_count: None,
                    expected_sha256: None,
                },
            ],
            capability_evidence: vec![
                CapabilityEvidence {
                    capability: EvidenceCapability::MaterialIdentity,
                    artifact_ids: vec!["metadata".to_owned()],
                },
                CapabilityEvidence {
                    capability: EvidenceCapability::SyntheticLineage,
                    artifact_ids: vec!["generator".to_owned(), "metadata".to_owned()],
                },
            ],
        }],
    }
}
