use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    FileRef, MAX_MANIFEST_BYTES, MAX_REFERENCED_FILE_BYTES, SourceClass, canonical_external_file,
    read_bounded_file, require_empty_output, resolve_artifact, resolve_cli_path,
    resolve_output_path, sha256_hex, validate_file_ref, validate_label,
};

const MANIFEST_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-inventory.manifest.v1";
const REPORT_SCHEMA: &str = "nextengine.experimental-physical-sound-corpus-inventory.report.v1";
const MAX_ENTRIES: usize = 1_000_000;
const MAX_UNAVAILABLE_COMPONENTS: usize = 64;

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
            _ => return Err(format!("unexpected corpus-inventory argument: {flag}")),
        }
    }
    Ok(Request {
        manifest: manifest.ok_or_else(|| {
            "physical-sound-registry corpus-inventory requires --manifest <external-json>"
                .to_owned()
        })?,
        output: output.ok_or_else(|| {
            "physical-sound-registry corpus-inventory requires --output <external-empty-directory>"
                .to_owned()
        })?,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct InventoryManifest {
    schema: String,
    inventory_id: String,
    revision: String,
    source_class: SourceClass,
    scope: InventoryScope,
    corpus_plan_report: FileRef,
    entries: Vec<InventoryEntry>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum InventoryScope {
    DevelopmentPilot,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct InventoryEntry {
    id: String,
    partition: Partition,
    recording_kind: RecordingKind,
    domain_id: String,
    material_family: String,
    object_family_id: String,
    object_id: String,
    source_id: String,
    #[serde(default)]
    generator_revision: Option<String>,
    #[serde(default)]
    mutation_parent_entry_id: Option<String>,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    impact_position_id: String,
    impact_position_metres: [f64; 3],
    listener_condition_id: String,
    listener_position_metres: [f64; 3],
    sample_rate_hz: u32,
    sample_count: usize,
    audio_format: AudioFormat,
    audio_payload: FileRef,
    acquisition_metadata: FileRef,
    provenance_review: FileRef,
    unavailable_components: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum Partition {
    Dev,
    Calibration,
    Holdout,
    Shadow,
}

impl Partition {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Calibration => "calibration",
            Self::Holdout => "holdout",
            Self::Shadow => "shadow",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum RecordingKind {
    ControlledRealForceDeconvolvedTransfer,
}

impl RecordingKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ControlledRealForceDeconvolvedTransfer => {
                "controlled_real_force_deconvolved_transfer"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum AudioFormat {
    F32LeMono,
}

#[derive(Debug, Serialize)]
struct InventoryReport {
    schema: &'static str,
    status: &'static str,
    decision: &'static str,
    claim: &'static str,
    inventory_id: String,
    revision: String,
    source_class: &'static str,
    scope: &'static str,
    manifest_sha256: String,
    corpus_plan_report_sha256: String,
    entry_count: usize,
    partition_counts: Vec<PartitionCount>,
    independent_group_counts: IndependentGroupCounts,
    entries: Vec<EntryReport>,
}

#[derive(Debug, Serialize)]
struct PartitionCount {
    partition: &'static str,
    count: usize,
}

#[derive(Debug, Serialize)]
struct IndependentGroupCounts {
    object_families: usize,
    objects: usize,
    sources: usize,
    generator_revisions: usize,
    mutation_parents: usize,
}

#[derive(Debug, Serialize)]
struct EntryReport {
    id: String,
    partition: &'static str,
    provisional_outcome: &'static str,
    recording_kind: &'static str,
    domain_id: String,
    material_family: String,
    object_family_id: String,
    object_id: String,
    source_id: String,
    generator_revision: Option<String>,
    mutation_parent_entry_id: Option<String>,
    geometry_revision: String,
    support_condition: String,
    excitation_method: String,
    impact_position_id: String,
    impact_position_metres: [f64; 3],
    listener_condition_id: String,
    listener_position_metres: [f64; 3],
    sample_rate_hz: u32,
    sample_count: usize,
    audio: AudioReport,
    acquisition_metadata_sha256: String,
    provenance_review_sha256: String,
    unavailable_components: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AudioReport {
    format: &'static str,
    sha256: String,
    byte_count: usize,
    peak_abs: f64,
    rms: f64,
}

fn run(root: &Path, request: &Request) -> Result<(), String> {
    let root =
        fs::canonicalize(root).map_err(|error| format!("canonicalize repository root: {error}"))?;
    let manifest_path = resolve_cli_path(&root, &request.manifest);
    let manifest_path =
        canonical_external_file(&root, &manifest_path, "corpus inventory manifest")?;
    let output = resolve_output_path(&root, &request.output)?;
    require_empty_output(&output)?;

    let manifest_bytes = read_bounded_file(&manifest_path, MAX_MANIFEST_BYTES, "inventory")?;
    let manifest: InventoryManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    validate_manifest(&manifest)?;
    let manifest_directory = manifest_path
        .parent()
        .ok_or_else(|| "corpus inventory manifest has no parent directory".to_owned())?;
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

fn validate_manifest(manifest: &InventoryManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound corpus inventory schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.inventory_id, "inventory id")?;
    validate_label(&manifest.revision, "inventory revision")?;
    validate_file_ref(&manifest.corpus_plan_report, "corpus plan report")?;
    if manifest.entries.is_empty() || manifest.entries.len() > MAX_ENTRIES {
        return Err(format!("inventory entry count must be 1..={MAX_ENTRIES}"));
    }

    let mut previous_id: Option<&str> = None;
    let mut partitions = GroupPartitionAudit::default();
    for entry in &manifest.entries {
        validate_entry(entry)?;
        if previous_id.is_some_and(|previous| previous >= entry.id.as_str()) {
            return Err(format!(
                "inventory entries must be strictly sorted by id; offending id {}",
                entry.id
            ));
        }
        previous_id = Some(&entry.id);
        partitions.insert(entry)?;
    }
    Ok(())
}

fn validate_entry(entry: &InventoryEntry) -> Result<(), String> {
    for (value, role) in [
        (&entry.id, "entry id"),
        (&entry.domain_id, "domain id"),
        (&entry.material_family, "material family"),
        (&entry.object_family_id, "object family id"),
        (&entry.object_id, "object id"),
        (&entry.source_id, "source id"),
        (&entry.geometry_revision, "geometry revision"),
        (&entry.support_condition, "support condition"),
        (&entry.excitation_method, "excitation method"),
        (&entry.impact_position_id, "impact position id"),
        (&entry.listener_condition_id, "listener condition id"),
    ] {
        validate_label(value, role)?;
    }
    if entry.generator_revision.is_some() || entry.mutation_parent_entry_id.is_some() {
        return Err(format!(
            "controlled real entry {} cannot declare generator or mutation parent identity",
            entry.id
        ));
    }
    validate_position(entry.impact_position_metres, "impact position")?;
    validate_position(entry.listener_position_metres, "listener position")?;
    if !(8_000..=384_000).contains(&entry.sample_rate_hz) || entry.sample_count == 0 {
        return Err(format!("entry {} has invalid sample dimensions", entry.id));
    }
    entry
        .sample_count
        .checked_mul(4)
        .filter(|bytes| *bytes <= MAX_REFERENCED_FILE_BYTES)
        .ok_or_else(|| format!("entry {} audio byte count is out of bounds", entry.id))?;
    validate_file_ref(&entry.audio_payload, "audio payload")?;
    validate_file_ref(&entry.acquisition_metadata, "acquisition metadata")?;
    validate_file_ref(&entry.provenance_review, "provenance review")?;
    validate_optional_sorted_labels(&entry.unavailable_components, "unavailable component ids")?;
    Ok(())
}

fn validate_position(position: [f64; 3], role: &str) -> Result<(), String> {
    if position
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > 1_000_000.0)
    {
        return Err(format!("{role} must contain finite bounded metres"));
    }
    Ok(())
}

fn validate_optional_sorted_labels(values: &[String], role: &str) -> Result<(), String> {
    if values.len() > MAX_UNAVAILABLE_COMPONENTS {
        return Err(format!(
            "{role} count must be 0..={MAX_UNAVAILABLE_COMPONENTS}"
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

#[derive(Default)]
struct GroupPartitionAudit {
    object_families: BTreeMap<String, Partition>,
    objects: BTreeMap<String, Partition>,
    sources: BTreeMap<String, Partition>,
    generator_revisions: BTreeMap<String, Partition>,
    mutation_parents: BTreeMap<String, Partition>,
}

impl GroupPartitionAudit {
    fn insert(&mut self, entry: &InventoryEntry) -> Result<(), String> {
        insert_group(
            &mut self.object_families,
            &entry.object_family_id,
            entry.partition,
            "object_family_id",
        )?;
        insert_group(
            &mut self.objects,
            &entry.object_id,
            entry.partition,
            "object_id",
        )?;
        insert_group(
            &mut self.sources,
            &entry.source_id,
            entry.partition,
            "source_id",
        )?;
        if let Some(revision) = &entry.generator_revision {
            insert_group(
                &mut self.generator_revisions,
                revision,
                entry.partition,
                "generator_revision",
            )?;
        }
        if let Some(parent) = &entry.mutation_parent_entry_id {
            insert_group(
                &mut self.mutation_parents,
                parent,
                entry.partition,
                "mutation_parent_entry_id",
            )?;
        }
        Ok(())
    }
}

fn insert_group(
    assignments: &mut BTreeMap<String, Partition>,
    value: &str,
    partition: Partition,
    key: &str,
) -> Result<(), String> {
    if let Some(previous) = assignments.insert(value.to_owned(), partition)
        && previous != partition
    {
        return Err(format!(
            "partition leakage for {key}={value}: {} versus {}",
            previous.as_str(),
            partition.as_str()
        ));
    }
    Ok(())
}

fn build_report(
    root: &Path,
    manifest_directory: &Path,
    manifest: InventoryManifest,
    manifest_sha256: String,
) -> Result<InventoryReport, String> {
    let corpus_plan_report_sha256 = resolve_artifact(
        root,
        manifest_directory,
        &manifest.corpus_plan_report,
        "corpus plan report",
    )?
    .sha256;
    let mut partition_counts = BTreeMap::<Partition, usize>::new();
    let mut object_families = BTreeSet::new();
    let mut objects = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut generator_revisions = BTreeSet::new();
    let mut mutation_parents = BTreeSet::new();
    let mut entry_reports = Vec::with_capacity(manifest.entries.len());

    for entry in manifest.entries {
        *partition_counts.entry(entry.partition).or_default() += 1;
        object_families.insert(entry.object_family_id.clone());
        objects.insert(entry.object_id.clone());
        sources.insert(entry.source_id.clone());
        if let Some(revision) = &entry.generator_revision {
            generator_revisions.insert(revision.clone());
        }
        if let Some(parent) = &entry.mutation_parent_entry_id {
            mutation_parents.insert(parent.clone());
        }
        let audio = analyse_audio(root, manifest_directory, &entry)?;
        let acquisition_metadata_sha256 = resolve_artifact(
            root,
            manifest_directory,
            &entry.acquisition_metadata,
            "acquisition metadata",
        )?
        .sha256;
        let provenance_review_sha256 = resolve_artifact(
            root,
            manifest_directory,
            &entry.provenance_review,
            "provenance review",
        )?
        .sha256;
        let provisional_outcome = if entry.unavailable_components.is_empty() {
            "ResearchEligible"
        } else {
            "FallbackOutOfDomain"
        };
        entry_reports.push(EntryReport {
            id: entry.id,
            partition: entry.partition.as_str(),
            provisional_outcome,
            recording_kind: entry.recording_kind.as_str(),
            domain_id: entry.domain_id,
            material_family: entry.material_family,
            object_family_id: entry.object_family_id,
            object_id: entry.object_id,
            source_id: entry.source_id,
            generator_revision: entry.generator_revision,
            mutation_parent_entry_id: entry.mutation_parent_entry_id,
            geometry_revision: entry.geometry_revision,
            support_condition: entry.support_condition,
            excitation_method: entry.excitation_method,
            impact_position_id: entry.impact_position_id,
            impact_position_metres: entry.impact_position_metres,
            listener_condition_id: entry.listener_condition_id,
            listener_position_metres: entry.listener_position_metres,
            sample_rate_hz: entry.sample_rate_hz,
            sample_count: entry.sample_count,
            audio,
            acquisition_metadata_sha256,
            provenance_review_sha256,
            unavailable_components: entry.unavailable_components,
        });
    }

    Ok(InventoryReport {
        schema: REPORT_SCHEMA,
        status: "Validated",
        decision: "DevelopmentPilotOnly",
        claim: "INVENTORY_AND_PARTITION_AUDIT_ONLY / NO_CORPUS_ADMISSION_AUTHORITY",
        inventory_id: manifest.inventory_id,
        revision: manifest.revision,
        source_class: manifest.source_class.as_str(),
        scope: "development_pilot",
        manifest_sha256,
        corpus_plan_report_sha256,
        entry_count: entry_reports.len(),
        partition_counts: [
            Partition::Dev,
            Partition::Calibration,
            Partition::Holdout,
            Partition::Shadow,
        ]
        .into_iter()
        .map(|partition| PartitionCount {
            partition: partition.as_str(),
            count: partition_counts.get(&partition).copied().unwrap_or(0),
        })
        .collect(),
        independent_group_counts: IndependentGroupCounts {
            object_families: object_families.len(),
            objects: objects.len(),
            sources: sources.len(),
            generator_revisions: generator_revisions.len(),
            mutation_parents: mutation_parents.len(),
        },
        entries: entry_reports,
    })
}

fn analyse_audio(
    root: &Path,
    manifest_directory: &Path,
    entry: &InventoryEntry,
) -> Result<AudioReport, String> {
    let artifact = resolve_artifact(
        root,
        manifest_directory,
        &entry.audio_payload,
        "inventory audio payload",
    )?;
    let expected_bytes = entry
        .sample_count
        .checked_mul(4)
        .ok_or_else(|| format!("entry {} sample byte count overflow", entry.id))?;
    if artifact.byte_count != expected_bytes {
        return Err(format!(
            "entry {} audio byte count mismatch: expected {expected_bytes}, got {}",
            entry.id, artifact.byte_count
        ));
    }
    let path = canonical_external_file(
        root,
        &manifest_directory.join(&entry.audio_payload.path),
        "inventory audio payload",
    )?;
    let bytes = read_bounded_file(&path, MAX_REFERENCED_FILE_BYTES, "inventory audio payload")?;
    let mut peak_abs = 0.0_f64;
    let mut square_sum = 0.0_f64;
    for sample in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes(sample.try_into().expect("four-byte chunk"));
        if !value.is_finite() {
            return Err(format!(
                "entry {} audio contains non-finite samples",
                entry.id
            ));
        }
        let value = f64::from(value);
        peak_abs = peak_abs.max(value.abs());
        square_sum += value * value;
    }
    Ok(AudioReport {
        format: "f32_le_mono",
        sha256: artifact.sha256,
        byte_count: artifact.byte_count,
        peak_abs,
        rms: (square_sum / entry.sample_count as f64).sqrt(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::Value;

    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nextengine-physical-sound-inventory-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create inventory test directory");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            if self.0.is_dir() {
                fs::remove_dir_all(&self.0).expect("remove inventory test directory");
            }
        }
    }

    #[test]
    fn inventory_rejects_partition_leakage_and_real_generator_identity() {
        let mut manifest = test_manifest();
        let mut second = manifest.entries[0].clone();
        second.id = "real-impact-b".to_owned();
        second.partition = Partition::Shadow;
        manifest.entries.push(second);
        assert!(
            validate_manifest(&manifest)
                .expect_err("object leakage rejects")
                .contains("partition leakage")
        );

        let mut manifest = test_manifest();
        manifest.entries[0].generator_revision = Some("generator-v1".to_owned());
        assert!(
            validate_manifest(&manifest)
                .expect_err("real generator identity rejects")
                .contains("cannot declare generator")
        );
    }

    #[test]
    fn end_to_end_inventory_is_hash_closed_finite_and_repeatable() {
        let directory = TestDirectory::new();
        let mut manifest = test_manifest();
        write_artifacts(&directory.0, &mut manifest);
        let manifest_path = directory.0.join("inventory.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize inventory"),
        )
        .expect("write inventory");
        let first = directory.0.join("first");
        run(
            workspace_root(),
            &Request {
                manifest: manifest_path.clone(),
                output: first.clone(),
            },
        )
        .expect("inventory validates");
        let report: Value =
            serde_json::from_slice(&fs::read(first.join("report.json")).expect("read report"))
                .expect("parse report");
        assert_eq!(report["decision"], "DevelopmentPilotOnly");
        assert_eq!(
            report["entries"][0]["provisional_outcome"],
            "FallbackOutOfDomain"
        );
        assert_eq!(report["entries"][0]["audio"]["peak_abs"], 0.5);

        let repeated = directory.0.join("repeated");
        run(
            workspace_root(),
            &Request {
                manifest: manifest_path,
                output: repeated.clone(),
            },
        )
        .expect("inventory repeats");
        assert_eq!(
            fs::read(first.join("report.json")).expect("read first report"),
            fs::read(repeated.join("report.json")).expect("read repeated report")
        );
    }

    #[test]
    fn inventory_rejects_wrong_audio_size_and_repository_local_input() {
        let directory = TestDirectory::new();
        let mut manifest = test_manifest();
        write_artifacts(&directory.0, &mut manifest);
        manifest.entries[0].sample_count += 1;
        let wrong_size_path = directory.0.join("wrong-size.json");
        fs::write(
            &wrong_size_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize inventory"),
        )
        .expect("write inventory");
        assert!(
            run(
                workspace_root(),
                &Request {
                    manifest: wrong_size_path,
                    output: directory.0.join("wrong-size-report"),
                },
            )
            .expect_err("wrong audio size rejects")
            .contains("audio byte count mismatch")
        );

        manifest.entries[0].sample_count -= 1;
        let repository_file = workspace_root().join("Cargo.toml");
        manifest.entries[0].audio_payload = FileRef {
            path: repository_file.display().to_string(),
            sha256: sha256_hex(&fs::read(&repository_file).expect("read Cargo.toml")),
        };
        let local_path = directory.0.join("repository-local.json");
        fs::write(
            &local_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize inventory"),
        )
        .expect("write inventory");
        assert!(
            run(
                workspace_root(),
                &Request {
                    manifest: local_path,
                    output: directory.0.join("repository-local-report"),
                },
            )
            .expect_err("repository-local audio rejects")
            .contains("must stay outside the repository")
        );
    }

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
    }

    fn write_artifacts(directory: &Path, manifest: &mut InventoryManifest) {
        let audio = [0.0_f32, 0.5, -0.25, 0.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let artifacts = [
            ("plan.json", b"{\"decision\":\"plan\"}".as_slice()),
            ("audio.f32le", audio.as_slice()),
            ("metadata.json", b"{\"geometry\":\"exact\"}".as_slice()),
            ("provenance.md", b"controlled test source".as_slice()),
        ];
        for (name, bytes) in artifacts {
            fs::write(directory.join(name), bytes).expect("write inventory artifact");
        }
        manifest.corpus_plan_report.sha256 = hash_file(directory, "plan.json");
        manifest.entries[0].audio_payload.sha256 = hash_file(directory, "audio.f32le");
        manifest.entries[0].acquisition_metadata.sha256 = hash_file(directory, "metadata.json");
        manifest.entries[0].provenance_review.sha256 = hash_file(directory, "provenance.md");
    }

    fn hash_file(directory: &Path, name: &str) -> String {
        sha256_hex(&fs::read(directory.join(name)).expect("read inventory artifact"))
    }

    fn test_manifest() -> InventoryManifest {
        InventoryManifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            inventory_id: "physical-sound-pilot".to_owned(),
            revision: "v1".to_owned(),
            source_class: SourceClass::RigidImpact,
            scope: InventoryScope::DevelopmentPilot,
            corpus_plan_report: file_ref("plan.json"),
            entries: vec![InventoryEntry {
                id: "real-impact-a".to_owned(),
                partition: Partition::Dev,
                recording_kind: RecordingKind::ControlledRealForceDeconvolvedTransfer,
                domain_id: "glass-vessel".to_owned(),
                material_family: "glass".to_owned(),
                object_family_id: "goblet".to_owned(),
                object_id: "goblet-a".to_owned(),
                source_id: "controlled-source-a".to_owned(),
                generator_revision: None,
                mutation_parent_entry_id: None,
                geometry_revision: "mesh-v1".to_owned(),
                support_condition: "thread-mesh-v1".to_owned(),
                excitation_method: "force-deconvolved-hammer".to_owned(),
                impact_position_id: "vertex-1".to_owned(),
                impact_position_metres: [0.0, 0.0, 0.1],
                listener_condition_id: "listener-1".to_owned(),
                listener_position_metres: [0.5, 0.0, 0.0],
                sample_rate_hz: 48_000,
                sample_count: 4,
                audio_format: AudioFormat::F32LeMono,
                audio_payload: file_ref("audio.f32le"),
                acquisition_metadata: file_ref("metadata.json"),
                provenance_review: file_ref("provenance.md"),
                unavailable_components: vec!["force-profile".to_owned()],
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
