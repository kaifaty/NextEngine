use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    FEATURE_MATRIX_SCHEMA, MANIFEST_SCHEMA, MAX_ENTRIES, MAX_EXTERNAL_FEATURE_DIMENSIONS,
    MAX_EXTERNAL_FEATURE_SETS,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BenchmarkManifest {
    pub(super) schema: String,
    pub(super) benchmark_id: String,
    pub(super) corpus_sources: Vec<CorpusSource>,
    #[serde(default)]
    pub(super) external_feature_sets: Vec<ExternalFeatureSet>,
    pub(super) entries: Vec<CorpusEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CorpusSource {
    pub(super) id: String,
    pub(super) revision: String,
    pub(super) source_url: String,
    pub(super) attribution: String,
    #[serde(default)]
    pub(super) measurement_scope: MeasurementScope,
    pub(super) license: LicenseDeclaration,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum MeasurementScope {
    #[default]
    ControlledImpact,
    MaterialIdentityOnly,
}

impl MeasurementScope {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ControlledImpact => "controlled_impact",
            Self::MaterialIdentityOnly => "material_identity_only",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LicenseDeclaration {
    pub(super) spdx_id: String,
    pub(super) review_status: LicenseReviewStatus,
    pub(super) redistribution: RedistributionPolicy,
    pub(super) review_record: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LicenseReviewStatus {
    ApprovedExternalBenchmarkOnly,
    UnreviewedPublicResearchSource,
}

impl LicenseReviewStatus {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ApprovedExternalBenchmarkOnly => "approved_external_benchmark_only",
            Self::UnreviewedPublicResearchSource => "unreviewed_public_research_source",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RedistributionPolicy {
    ExternalOnly,
    NoRepositoryOrDistribution,
}

impl RedistributionPolicy {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalOnly => "external_only",
            Self::NoRepositoryOrDistribution => "no_repository_or_distribution",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExternalFeatureSet {
    pub(super) id: String,
    pub(super) model_revision: String,
    pub(super) model_sha256: String,
    pub(super) dimensions: usize,
    pub(super) distance: DistanceMetric,
    pub(super) matrix: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DistanceMetric {
    Cosine,
    Euclidean,
}

impl DistanceMetric {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Cosine => "cosine",
            Self::Euclidean => "euclidean",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CorpusEntry {
    pub(super) id: String,
    pub(super) source_id: String,
    pub(super) partition: Partition,
    pub(super) object_id: String,
    pub(super) object_family_id: String,
    pub(super) material: String,
    pub(super) impact_position_id: String,
    pub(super) listener_position_id: String,
    pub(super) force_band: String,
    pub(super) origin: EntryOrigin,
    pub(super) audio: FileRef,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Partition {
    Development,
    Calibration,
    Holdout,
    Shadow,
}

impl Partition {
    pub(super) const ALL: [Self; 4] = [
        Self::Development,
        Self::Calibration,
        Self::Holdout,
        Self::Shadow,
    ];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Calibration => "calibration",
            Self::Holdout => "holdout",
            Self::Shadow => "shadow",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub(super) enum EntryOrigin {
    Real,
    Generated {
        generator_revision: String,
        generator_sha256: String,
    },
    Mutation {
        mutation_family: String,
        parent_entry_id: String,
    },
}

impl EntryOrigin {
    pub(super) fn group_id(&self) -> String {
        match self {
            Self::Real => "real".to_owned(),
            Self::Generated {
                generator_revision, ..
            } => format!("generator:{generator_revision}"),
            Self::Mutation {
                mutation_family, ..
            } => format!("mutation:{mutation_family}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileRef {
    pub(super) path: String,
    pub(super) sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FeatureMatrix {
    pub(super) schema: String,
    pub(super) feature_set_id: String,
    pub(super) entries: Vec<FeatureMatrixEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FeatureMatrixEntry {
    pub(super) id: String,
    pub(super) values: Vec<f64>,
}

pub(super) fn validate_manifest(manifest: &BenchmarkManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported physical sound corpus benchmark manifest schema: {}",
            manifest.schema
        ));
    }
    validate_label(&manifest.benchmark_id, "benchmark_id")?;
    validate_sources(&manifest.corpus_sources)?;
    validate_feature_sets(&manifest.external_feature_sets)?;
    validate_entries(manifest)?;
    validate_partitions(manifest)
}

fn validate_sources(sources: &[CorpusSource]) -> Result<(), String> {
    if sources.is_empty() || sources.len() > 32 {
        return Err(format!(
            "corpus source count must be 1..=32, got {}",
            sources.len()
        ));
    }
    let mut previous: Option<&str> = None;
    for source in sources {
        validate_label(&source.id, "corpus source id")?;
        validate_label(&source.revision, "corpus source revision")?;
        if previous.is_some_and(|value| value >= source.id.as_str()) {
            return Err(format!(
                "corpus sources must be strictly sorted by id; offending id {}",
                source.id
            ));
        }
        previous = Some(&source.id);
        if !source.source_url.starts_with("https://")
            || source.source_url.chars().any(char::is_whitespace)
        {
            return Err(format!(
                "corpus source {} must use a whitespace-free https URL",
                source.id
            ));
        }
        validate_text(&source.attribution, "corpus attribution", 1_024)?;
        validate_spdx_id(&source.license.spdx_id)?;
        if source.license.review_status == LicenseReviewStatus::UnreviewedPublicResearchSource
            && source.license.redistribution != RedistributionPolicy::NoRepositoryOrDistribution
        {
            return Err(format!(
                "unreviewed public research source {} requires no_repository_or_distribution",
                source.id
            ));
        }
        validate_file_ref(&source.license.review_record, "license review record")?;
    }
    Ok(())
}

fn validate_feature_sets(feature_sets: &[ExternalFeatureSet]) -> Result<(), String> {
    if feature_sets.len() > MAX_EXTERNAL_FEATURE_SETS {
        return Err(format!(
            "external feature set count must be <= {MAX_EXTERNAL_FEATURE_SETS}, got {}",
            feature_sets.len()
        ));
    }
    let mut previous: Option<&str> = None;
    for feature_set in feature_sets {
        validate_label(&feature_set.id, "feature set id")?;
        if feature_set.id == "classical-av-p0b-v1" {
            return Err("external feature set id collides with the built-in profile".to_owned());
        }
        if previous.is_some_and(|value| value >= feature_set.id.as_str()) {
            return Err(format!(
                "external feature sets must be strictly sorted by id; offending id {}",
                feature_set.id
            ));
        }
        previous = Some(&feature_set.id);
        validate_label(&feature_set.model_revision, "feature model revision")?;
        validate_sha256(&feature_set.model_sha256, "feature model sha256")?;
        if feature_set.dimensions == 0 || feature_set.dimensions > MAX_EXTERNAL_FEATURE_DIMENSIONS {
            return Err(format!(
                "feature set {} dimensions must be 1..={MAX_EXTERNAL_FEATURE_DIMENSIONS}",
                feature_set.id
            ));
        }
        validate_file_ref(&feature_set.matrix, "feature matrix")?;
    }
    Ok(())
}

fn validate_entries(manifest: &BenchmarkManifest) -> Result<(), String> {
    if manifest.entries.is_empty() || manifest.entries.len() > MAX_ENTRIES {
        return Err(format!(
            "corpus entry count must be 1..={MAX_ENTRIES}, got {}",
            manifest.entries.len()
        ));
    }
    let source_ids = manifest
        .corpus_sources
        .iter()
        .map(|source| source.id.as_str())
        .collect::<BTreeSet<_>>();
    let entry_ids = manifest
        .entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut previous: Option<&str> = None;
    for entry in &manifest.entries {
        validate_label(&entry.id, "entry id")?;
        validate_label(&entry.source_id, "entry source_id")?;
        validate_label(&entry.object_id, "entry object_id")?;
        validate_label(&entry.object_family_id, "entry object_family_id")?;
        validate_label(&entry.material, "entry material")?;
        validate_label(&entry.impact_position_id, "entry impact_position_id")?;
        validate_label(&entry.listener_position_id, "entry listener_position_id")?;
        validate_label(&entry.force_band, "entry force_band")?;
        if previous.is_some_and(|value| value >= entry.id.as_str()) {
            return Err(format!(
                "corpus entries must be strictly sorted by id; offending id {}",
                entry.id
            ));
        }
        previous = Some(&entry.id);
        if !source_ids.contains(entry.source_id.as_str()) {
            return Err(format!(
                "entry {} references missing source {}",
                entry.id, entry.source_id
            ));
        }
        match &entry.origin {
            EntryOrigin::Real => {}
            EntryOrigin::Generated {
                generator_revision,
                generator_sha256,
            } => {
                validate_label(generator_revision, "generator revision")?;
                validate_sha256(generator_sha256, "generator sha256")?;
                if entry.partition == Partition::Development {
                    return Err(format!(
                        "generated entry {} cannot enter the real-only development gallery",
                        entry.id
                    ));
                }
            }
            EntryOrigin::Mutation {
                mutation_family,
                parent_entry_id,
            } => {
                validate_label(mutation_family, "mutation family")?;
                validate_label(parent_entry_id, "mutation parent entry id")?;
                if parent_entry_id == &entry.id || !entry_ids.contains(parent_entry_id.as_str()) {
                    return Err(format!(
                        "mutation entry {} references invalid parent {}",
                        entry.id, parent_entry_id
                    ));
                }
                if entry.partition == Partition::Development {
                    return Err(format!(
                        "mutation entry {} cannot enter the real-only development gallery",
                        entry.id
                    ));
                }
            }
        }
        validate_file_ref(&entry.audio, "entry audio")?;
    }
    Ok(())
}

fn validate_partitions(manifest: &BenchmarkManifest) -> Result<(), String> {
    let source_scopes = manifest
        .corpus_sources
        .iter()
        .map(|source| (source.id.as_str(), source.measurement_scope))
        .collect::<BTreeMap<_, _>>();
    let mut partition_counts = BTreeMap::<Partition, usize>::new();
    let mut object_partitions = BTreeMap::<&str, Partition>::new();
    let mut family_partitions = BTreeMap::<&str, Partition>::new();
    let mut object_identity = BTreeMap::<&str, (&str, &str, &str)>::new();
    let mut family_material = BTreeMap::<&str, &str>::new();
    let mut development_materials = BTreeSet::new();
    let mut development_objects_by_material = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut development_families_by_material = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut development_positions_by_object = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut non_development_materials = BTreeSet::new();
    for entry in &manifest.entries {
        let measurement_scope = source_scopes[entry.source_id.as_str()];
        if measurement_scope == MeasurementScope::MaterialIdentityOnly
            && (entry.impact_position_id != "unspecified"
                || entry.listener_position_id != "unspecified"
                || entry.force_band != "unspecified")
        {
            return Err(format!(
                "material-identity-only source {} requires unspecified impact/listener/force labels on entry {}",
                entry.source_id, entry.id
            ));
        }
        *partition_counts.entry(entry.partition).or_default() += 1;
        require_one_partition(
            &mut object_partitions,
            &entry.object_id,
            entry.partition,
            "object_id",
        )?;
        require_one_partition(
            &mut family_partitions,
            &entry.object_family_id,
            entry.partition,
            "object_family_id",
        )?;
        if let Some((family, material, source_id)) = object_identity.insert(
            &entry.object_id,
            (&entry.object_family_id, &entry.material, &entry.source_id),
        ) && (family != entry.object_family_id
            || material != entry.material
            || source_id != entry.source_id)
        {
            return Err(format!(
                "object {} changes family, material or source across entries",
                entry.object_id
            ));
        }
        if let Some(material) = family_material.insert(&entry.object_family_id, &entry.material)
            && material != entry.material
        {
            return Err(format!(
                "object family {} changes material across entries",
                entry.object_family_id
            ));
        }
        if entry.partition == Partition::Development {
            if matches!(&entry.origin, EntryOrigin::Real) {
                development_materials.insert(entry.material.as_str());
                development_objects_by_material
                    .entry(&entry.material)
                    .or_default()
                    .insert(&entry.object_id);
                development_families_by_material
                    .entry(&entry.material)
                    .or_default()
                    .insert(&entry.object_family_id);
                if measurement_scope == MeasurementScope::ControlledImpact {
                    development_positions_by_object
                        .entry(&entry.object_id)
                        .or_default()
                        .insert(&entry.impact_position_id);
                }
            }
        } else {
            non_development_materials.insert(entry.material.as_str());
        }
    }
    for partition in Partition::ALL {
        if partition_counts.get(&partition).copied().unwrap_or(0) == 0 {
            return Err(format!(
                "partition {} must contain at least one entry",
                partition.as_str()
            ));
        }
    }
    for material in non_development_materials {
        if !development_materials.contains(material) {
            return Err(format!(
                "material {material} has no real development-gallery example"
            ));
        }
    }
    if development_materials.len() < 2 {
        return Err("development gallery must contain at least two real materials".to_owned());
    }
    for material in development_materials {
        let object_count = development_objects_by_material
            .get(material)
            .map_or(0, BTreeSet::len);
        let family_count = development_families_by_material
            .get(material)
            .map_or(0, BTreeSet::len);
        if object_count < 2 || family_count < 2 {
            return Err(format!(
                "development material {material} needs at least two real objects and two object families for grouped holdouts"
            ));
        }
    }
    for (object_id, positions) in development_positions_by_object {
        if positions.len() < 2 {
            return Err(format!(
                "development object {object_id} needs at least two impact positions for leave-position-out evaluation"
            ));
        }
    }
    Ok(())
}

fn require_one_partition<'a>(
    groups: &mut BTreeMap<&'a str, Partition>,
    id: &'a str,
    partition: Partition,
    role: &str,
) -> Result<(), String> {
    if let Some(previous) = groups.insert(id, partition)
        && previous != partition
    {
        return Err(format!(
            "{role} {id} leaks across partitions {} and {}",
            previous.as_str(),
            partition.as_str()
        ));
    }
    Ok(())
}

pub(super) fn validate_feature_matrix(
    declaration: &ExternalFeatureSet,
    matrix: &FeatureMatrix,
    manifest_entries: &[CorpusEntry],
) -> Result<(), String> {
    if matrix.schema != FEATURE_MATRIX_SCHEMA {
        return Err(format!(
            "unsupported feature matrix schema for {}: {}",
            declaration.id, matrix.schema
        ));
    }
    if matrix.feature_set_id != declaration.id {
        return Err(format!(
            "feature matrix id mismatch: expected {}, got {}",
            declaration.id, matrix.feature_set_id
        ));
    }
    if matrix.entries.len() != manifest_entries.len() {
        return Err(format!(
            "feature matrix {} must contain exactly {} entries, got {}",
            declaration.id,
            manifest_entries.len(),
            matrix.entries.len()
        ));
    }
    for (feature, manifest) in matrix.entries.iter().zip(manifest_entries) {
        if feature.id != manifest.id {
            return Err(format!(
                "feature matrix {} entries must match sorted manifest ids; expected {}, got {}",
                declaration.id, manifest.id, feature.id
            ));
        }
        if feature.values.len() != declaration.dimensions {
            return Err(format!(
                "feature matrix {} entry {} has {} dimensions, expected {}",
                declaration.id,
                feature.id,
                feature.values.len(),
                declaration.dimensions
            ));
        }
        if feature.values.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "feature matrix {} entry {} contains a non-finite value",
                declaration.id, feature.id
            ));
        }
    }
    Ok(())
}

fn validate_file_ref(file: &FileRef, role: &str) -> Result<(), String> {
    validate_relative_path(&file.path, role)?;
    validate_sha256(&file.sha256, &format!("{role} sha256"))
}

fn validate_relative_path(path: &str, role: &str) -> Result<(), String> {
    let path = std::path::Path::new(path);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(format!(
            "{role} path must be non-empty and manifest-relative"
        ));
    }
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) {
        return Err(format!("{role} path cannot escape the manifest directory"));
    }
    Ok(())
}

fn validate_sha256(value: &str, role: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{role} must be 64 lowercase hexadecimal digits"));
    }
    Ok(())
}

fn validate_spdx_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'+' | b'(' | b')')
        })
    {
        return Err("license spdx_id is not a bounded SPDX expression".to_owned());
    }
    Ok(())
}

fn validate_label(value: &str, role: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 160
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        })
    {
        return Err(format!("{role} contains an invalid bounded label"));
    }
    Ok(())
}

fn validate_text(value: &str, role: &str, maximum_bytes: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > maximum_bytes || value.contains('\0') {
        return Err(format!(
            "{role} must be non-empty and <= {maximum_bytes} bytes"
        ));
    }
    Ok(())
}
