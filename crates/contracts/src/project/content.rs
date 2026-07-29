use std::collections::{BTreeMap, BTreeSet};

use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{AssetId, ContentHash, ProjectId, SchemaId};
use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};

use super::codec::{
    ProjectContractError, array, asset_id, domain_hash, enforce_limit, ensure_unique, hash, object,
    reject_unknown, string, take, text, u64_text,
};
use super::schema::{
    SchemaEncodingV1, SchemaRefV1, SchemaRoleV1, decode_schema_ref, schema_ref_value,
};
use super::{CONTENT_MANIFEST_FORMAT_V1, PROJECT_MAX_DEPENDENCIES_V1, PROJECT_MAX_RECORDS_V1};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ContentSemanticClassV1 {
    DomainRelevant = 1,
    PresentationOnly = 2,
}

impl ContentSemanticClassV1 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::DomainRelevant => "domain-relevant",
            Self::PresentationOnly => "presentation-only",
        }
    }

    fn from_str(value: &str) -> Result<Self, ProjectContractError> {
        match value {
            "domain-relevant" => Ok(Self::DomainRelevant),
            "presentation-only" => Ok(Self::PresentationOnly),
            _ => Err(ProjectContractError::UnknownClosedValue),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentProvenanceV1 {
    pub provenance_id: SchemaId,
    pub license_id: SchemaId,
    pub attribution: String,
    pub provenance_sha256: ContentHash,
}

impl ContentProvenanceV1 {
    pub fn new(
        provenance_id: SchemaId,
        license_id: SchemaId,
        attribution: impl Into<String>,
    ) -> Result<Self, ProjectContractError> {
        let attribution = attribution.into();
        validate_text(&attribution)?;
        let mut value = Self {
            provenance_id,
            license_id,
            attribution,
            provenance_sha256: ContentHash::default(),
        };
        value.provenance_sha256 =
            domain_hash("nextengine.content-provenance.v1", &value.body_bytes());
        Ok(value)
    }

    fn body_bytes(&self) -> Vec<u8> {
        encode_canonical_jcs(&content_provenance_value(self, false))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AssetRevisionRefV1 {
    pub asset_id: AssetId,
    pub record_sha256: ContentHash,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContentAssetEntryV1 {
    pub asset_revision: AssetRevisionRefV1,
    pub schema_ref: SchemaRefV1,
    pub neutral_record_blob_sha256: ContentHash,
    pub semantic_class: ContentSemanticClassV1,
    pub provenance_sha256: ContentHash,
    pub license_manifest_sha256: ContentHash,
    pub owning_bundle_id: SchemaId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContentDependencyEdgeV1 {
    pub source_asset_id: AssetId,
    pub target_asset_id: AssetId,
    pub dependency_kind: SchemaId,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentManifestBodyV1 {
    pub schema_ref: SchemaRefV1,
    pub manifest_id: SchemaId,
    pub project_id: ProjectId,
    pub content_revision: u64,
    pub schema_registry_manifest_sha256: ContentHash,
    pub canonicalization_profile_sha256: ContentHash,
    pub content_admission_limits_sha256: ContentHash,
    pub cooker_contract_sha256: ContentHash,
    pub cooker_options_sha256: ContentHash,
    pub root_assets: Vec<AssetRevisionRefV1>,
    pub provenance_records: Vec<ContentProvenanceV1>,
    pub asset_entries: Vec<ContentAssetEntryV1>,
    pub dependency_edges: Vec<ContentDependencyEdgeV1>,
    pub domain_closure_sha256: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentManifestV1 {
    pub body: ContentManifestBodyV1,
    pub content_manifest_sha256: ContentHash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyContentManifestProfilesV1 {
    pub schema_registry_manifest_sha256: ContentHash,
    pub canonicalization_profile_sha256: ContentHash,
    pub content_admission_limits_sha256: ContentHash,
    pub cooker_contract_sha256: ContentHash,
    pub cooker_options_sha256: ContentHash,
}

impl ContentManifestV1 {
    pub fn new(mut body: ContentManifestBodyV1) -> Result<Self, ProjectContractError> {
        body.root_assets
            .sort_by_key(|reference| (reference.asset_id, reference.record_sha256));
        body.provenance_records
            .sort_by_key(|record| record.provenance_sha256);
        body.asset_entries.sort();
        body.dependency_edges.sort();
        validate_content_manifest(&mut body)?;
        let bytes = content_manifest_body_bytes(&body);
        Ok(Self {
            body,
            content_manifest_sha256: domain_hash(CONTENT_MANIFEST_FORMAT_V1, &bytes),
        })
    }

    pub fn empty(
        schema_ref: SchemaRefV1,
        manifest_id: SchemaId,
        project_id: ProjectId,
        profiles: EmptyContentManifestProfilesV1,
    ) -> Result<Self, ProjectContractError> {
        Self::new(ContentManifestBodyV1 {
            schema_ref,
            manifest_id,
            project_id,
            content_revision: 1,
            schema_registry_manifest_sha256: profiles.schema_registry_manifest_sha256,
            canonicalization_profile_sha256: profiles.canonicalization_profile_sha256,
            content_admission_limits_sha256: profiles.content_admission_limits_sha256,
            cooker_contract_sha256: profiles.cooker_contract_sha256,
            cooker_options_sha256: profiles.cooker_options_sha256,
            root_assets: Vec::new(),
            provenance_records: Vec::new(),
            asset_entries: Vec::new(),
            dependency_edges: Vec::new(),
            domain_closure_sha256: domain_hash("nextengine.content-domain-closure.v1", &[]),
        })
    }

    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ProjectContractError> {
        let canonical = Self::new(self.body.clone())?;
        if canonical.content_manifest_sha256 != self.content_manifest_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
        Ok(content_manifest_body_bytes(&self.body))
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let body = decode_content_manifest_body(value)?;
        let manifest = Self::new(body)?;
        if manifest.to_jcs_bytes()? != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(manifest)
    }
}

fn validate_content_manifest(body: &mut ContentManifestBodyV1) -> Result<(), ProjectContractError> {
    if body.content_revision == 0 {
        return Err(ProjectContractError::ZeroRevision);
    }
    body.schema_ref.validate()?;
    if body.schema_ref.role != SchemaRoleV1::Manifest
        || body.schema_ref.encoding != SchemaEncodingV1::JcsRfc8785
    {
        return Err(ProjectContractError::UnknownClosedValue);
    }
    enforce_limit(body.asset_entries.len(), PROJECT_MAX_RECORDS_V1)?;
    enforce_limit(body.dependency_edges.len(), PROJECT_MAX_DEPENDENCIES_V1)?;
    ensure_unique(
        body.asset_entries
            .iter()
            .map(|entry| entry.asset_revision.asset_id),
    )?;
    ensure_unique(
        body.provenance_records
            .iter()
            .map(|record| record.provenance_sha256),
    )?;
    ensure_unique(body.dependency_edges.iter())?;

    let assets: BTreeMap<_, _> = body
        .asset_entries
        .iter()
        .map(|entry| (entry.asset_revision.asset_id, entry))
        .collect();
    for entry in &body.asset_entries {
        entry.schema_ref.validate()?;
        if entry.neutral_record_blob_sha256 != entry.asset_revision.record_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
    }
    for root in &body.root_assets {
        let Some(entry) = assets.get(&root.asset_id) else {
            return Err(ProjectContractError::MissingReference);
        };
        if entry.asset_revision != *root {
            return Err(ProjectContractError::HashMismatch);
        }
    }
    for edge in &body.dependency_edges {
        if !assets.contains_key(&edge.source_asset_id)
            || !assets.contains_key(&edge.target_asset_id)
        {
            return Err(ProjectContractError::MissingReference);
        }
    }

    let provenance: BTreeSet<_> = body
        .provenance_records
        .iter()
        .map(|record| record.provenance_sha256)
        .collect();
    let referenced_provenance: BTreeSet<_> = body
        .asset_entries
        .iter()
        .map(|entry| entry.provenance_sha256)
        .collect();
    if provenance != referenced_provenance {
        return Err(ProjectContractError::MissingReference);
    }
    for record in &body.provenance_records {
        let expected = domain_hash("nextengine.content-provenance.v1", &record.body_bytes());
        if expected != record.provenance_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
    }
    ensure_acyclic_content(&body.asset_entries, &body.dependency_edges)?;
    body.domain_closure_sha256 = content_domain_closure_hash(body);
    Ok(())
}

fn ensure_acyclic_content(
    entries: &[ContentAssetEntryV1],
    edges: &[ContentDependencyEdgeV1],
) -> Result<(), ProjectContractError> {
    let mut indegree: BTreeMap<AssetId, usize> = entries
        .iter()
        .map(|entry| (entry.asset_revision.asset_id, 0))
        .collect();
    let mut successors: BTreeMap<AssetId, Vec<AssetId>> = BTreeMap::new();
    for edge in edges.iter().filter(|edge| edge.required) {
        *indegree
            .get_mut(&edge.target_asset_id)
            .ok_or(ProjectContractError::MissingReference)? += 1;
        successors
            .entry(edge.source_asset_id)
            .or_default()
            .push(edge.target_asset_id);
    }
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .filter_map(|(asset_id, indegree)| (*indegree == 0).then_some(*asset_id))
        .collect();
    let mut visited = 0_usize;
    while let Some(asset_id) = ready.pop_first() {
        visited += 1;
        if let Some(children) = successors.get(&asset_id) {
            for child in children {
                let value = indegree
                    .get_mut(child)
                    .ok_or(ProjectContractError::MissingReference)?;
                *value -= 1;
                if *value == 0 {
                    ready.insert(*child);
                }
            }
        }
    }
    if visited != entries.len() {
        return Err(ProjectContractError::DependencyCycle);
    }
    Ok(())
}

fn content_domain_closure_hash(body: &ContentManifestBodyV1) -> ContentHash {
    let mut bytes = Vec::new();
    for entry in &body.asset_entries {
        bytes.extend_from_slice(entry.asset_revision.asset_id.as_bytes());
        bytes.extend_from_slice(entry.asset_revision.record_sha256.as_bytes());
    }
    for edge in &body.dependency_edges {
        bytes.extend_from_slice(edge.source_asset_id.as_bytes());
        bytes.extend_from_slice(edge.target_asset_id.as_bytes());
        bytes.extend_from_slice(edge.dependency_kind.as_str().as_bytes());
        bytes.push(u8::from(edge.required));
    }
    domain_hash("nextengine.content-domain-closure.v1", &bytes)
}

fn content_manifest_body_bytes(body: &ContentManifestBodyV1) -> Vec<u8> {
    let mut value = BTreeMap::new();
    value.insert(
        "asset_entries".to_owned(),
        JcsValue::Array(body.asset_entries.iter().map(content_asset_value).collect()),
    );
    value.insert(
        "canonicalization_profile_sha256".to_owned(),
        string(body.canonicalization_profile_sha256.to_hex()),
    );
    value.insert(
        "content_admission_limits_sha256".to_owned(),
        string(body.content_admission_limits_sha256.to_hex()),
    );
    value.insert(
        "content_manifest_format".to_owned(),
        string(CONTENT_MANIFEST_FORMAT_V1),
    );
    value.insert(
        "content_revision".to_owned(),
        string(body.content_revision.to_string()),
    );
    value.insert(
        "cooker_contract_sha256".to_owned(),
        string(body.cooker_contract_sha256.to_hex()),
    );
    value.insert(
        "cooker_options_sha256".to_owned(),
        string(body.cooker_options_sha256.to_hex()),
    );
    value.insert(
        "dependency_edges".to_owned(),
        JcsValue::Array(
            body.dependency_edges
                .iter()
                .map(content_dependency_value)
                .collect(),
        ),
    );
    value.insert(
        "domain_closure_sha256".to_owned(),
        string(body.domain_closure_sha256.to_hex()),
    );
    value.insert("manifest_id".to_owned(), string(body.manifest_id.as_str()));
    value.insert("project_id".to_owned(), string(body.project_id.as_str()));
    value.insert(
        "provenance_records".to_owned(),
        JcsValue::Array(
            body.provenance_records
                .iter()
                .map(|record| content_provenance_value(record, true))
                .collect(),
        ),
    );
    value.insert(
        "root_assets".to_owned(),
        JcsValue::Array(body.root_assets.iter().map(asset_revision_value).collect()),
    );
    value.insert("schema_ref".to_owned(), schema_ref_value(&body.schema_ref));
    value.insert(
        "schema_registry_manifest_sha256".to_owned(),
        string(body.schema_registry_manifest_sha256.to_hex()),
    );
    encode_canonical_jcs(&JcsValue::Object(value))
}

fn decode_content_manifest_body(
    value: JcsValue,
) -> Result<ContentManifestBodyV1, ProjectContractError> {
    let mut value = object(value, "content_manifest")?;
    let format = text(
        take(&mut value, "content_manifest_format")?,
        "content_manifest_format",
    )?;
    if format != CONTENT_MANIFEST_FORMAT_V1 {
        return Err(ProjectContractError::UnknownClosedValue);
    }
    let expected_domain_hash = hash(
        take(&mut value, "domain_closure_sha256")?,
        "domain_closure_sha256",
    )?;
    let mut body = ContentManifestBodyV1 {
        schema_ref: decode_schema_ref(take(&mut value, "schema_ref")?)?,
        manifest_id: SchemaId::new(text(take(&mut value, "manifest_id")?, "manifest_id")?)?,
        project_id: ProjectId::new(text(take(&mut value, "project_id")?, "project_id")?)?,
        content_revision: u64_text(take(&mut value, "content_revision")?, "content_revision")?,
        schema_registry_manifest_sha256: hash(
            take(&mut value, "schema_registry_manifest_sha256")?,
            "schema_registry_manifest_sha256",
        )?,
        canonicalization_profile_sha256: hash(
            take(&mut value, "canonicalization_profile_sha256")?,
            "canonicalization_profile_sha256",
        )?,
        content_admission_limits_sha256: hash(
            take(&mut value, "content_admission_limits_sha256")?,
            "content_admission_limits_sha256",
        )?,
        cooker_contract_sha256: hash(
            take(&mut value, "cooker_contract_sha256")?,
            "cooker_contract_sha256",
        )?,
        cooker_options_sha256: hash(
            take(&mut value, "cooker_options_sha256")?,
            "cooker_options_sha256",
        )?,
        root_assets: array(take(&mut value, "root_assets")?, "root_assets")?
            .into_iter()
            .map(decode_asset_revision)
            .collect::<Result<_, _>>()?,
        provenance_records: array(
            take(&mut value, "provenance_records")?,
            "provenance_records",
        )?
        .into_iter()
        .map(decode_content_provenance)
        .collect::<Result<_, _>>()?,
        asset_entries: array(take(&mut value, "asset_entries")?, "asset_entries")?
            .into_iter()
            .map(decode_content_asset)
            .collect::<Result<_, _>>()?,
        dependency_edges: array(take(&mut value, "dependency_edges")?, "dependency_edges")?
            .into_iter()
            .map(decode_content_dependency)
            .collect::<Result<_, _>>()?,
        domain_closure_sha256: expected_domain_hash,
    };
    reject_unknown(value)?;
    let actual_domain_hash = content_domain_closure_hash(&body);
    if actual_domain_hash != expected_domain_hash {
        return Err(ProjectContractError::HashMismatch);
    }
    body.domain_closure_sha256 = actual_domain_hash;
    Ok(body)
}

fn content_provenance_value(record: &ContentProvenanceV1, include_hash: bool) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert("attribution".to_owned(), string(record.attribution.clone()));
    value.insert("license_id".to_owned(), string(record.license_id.as_str()));
    value.insert(
        "provenance_id".to_owned(),
        string(record.provenance_id.as_str()),
    );
    if include_hash {
        value.insert(
            "provenance_sha256".to_owned(),
            string(record.provenance_sha256.to_hex()),
        );
    }
    JcsValue::Object(value)
}

fn decode_content_provenance(value: JcsValue) -> Result<ContentProvenanceV1, ProjectContractError> {
    let mut value = object(value, "content_provenance")?;
    let provenance_id = SchemaId::new(text(take(&mut value, "provenance_id")?, "provenance_id")?)?;
    let license_id = SchemaId::new(text(take(&mut value, "license_id")?, "license_id")?)?;
    let attribution = text(take(&mut value, "attribution")?, "attribution")?;
    let expected = hash(take(&mut value, "provenance_sha256")?, "provenance_sha256")?;
    reject_unknown(value)?;
    let record = ContentProvenanceV1::new(provenance_id, license_id, attribution)?;
    if record.provenance_sha256 != expected {
        return Err(ProjectContractError::HashMismatch);
    }
    Ok(record)
}

pub(super) fn asset_revision_value(reference: &AssetRevisionRefV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert("asset_id".to_owned(), string(reference.asset_id.to_hex()));
    value.insert(
        "record_sha256".to_owned(),
        string(reference.record_sha256.to_hex()),
    );
    JcsValue::Object(value)
}

pub(super) fn decode_asset_revision(
    value: JcsValue,
) -> Result<AssetRevisionRefV1, ProjectContractError> {
    let mut value = object(value, "asset_revision")?;
    let reference = AssetRevisionRefV1 {
        asset_id: asset_id(take(&mut value, "asset_id")?, "asset_id")?,
        record_sha256: hash(take(&mut value, "record_sha256")?, "record_sha256")?,
    };
    reject_unknown(value)?;
    Ok(reference)
}

fn content_asset_value(entry: &ContentAssetEntryV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "asset_revision".to_owned(),
        asset_revision_value(&entry.asset_revision),
    );
    value.insert(
        "license_manifest_sha256".to_owned(),
        string(entry.license_manifest_sha256.to_hex()),
    );
    value.insert(
        "neutral_record_blob_sha256".to_owned(),
        string(entry.neutral_record_blob_sha256.to_hex()),
    );
    value.insert(
        "owning_bundle_id".to_owned(),
        string(entry.owning_bundle_id.as_str()),
    );
    value.insert(
        "provenance_sha256".to_owned(),
        string(entry.provenance_sha256.to_hex()),
    );
    value.insert("schema_ref".to_owned(), schema_ref_value(&entry.schema_ref));
    value.insert(
        "semantic_class".to_owned(),
        string(entry.semantic_class.as_str()),
    );
    JcsValue::Object(value)
}

fn decode_content_asset(value: JcsValue) -> Result<ContentAssetEntryV1, ProjectContractError> {
    let mut value = object(value, "content_asset")?;
    let entry = ContentAssetEntryV1 {
        asset_revision: decode_asset_revision(take(&mut value, "asset_revision")?)?,
        schema_ref: decode_schema_ref(take(&mut value, "schema_ref")?)?,
        neutral_record_blob_sha256: hash(
            take(&mut value, "neutral_record_blob_sha256")?,
            "neutral_record_blob_sha256",
        )?,
        semantic_class: ContentSemanticClassV1::from_str(&text(
            take(&mut value, "semantic_class")?,
            "semantic_class",
        )?)?,
        provenance_sha256: hash(take(&mut value, "provenance_sha256")?, "provenance_sha256")?,
        license_manifest_sha256: hash(
            take(&mut value, "license_manifest_sha256")?,
            "license_manifest_sha256",
        )?,
        owning_bundle_id: SchemaId::new(text(
            take(&mut value, "owning_bundle_id")?,
            "owning_bundle_id",
        )?)?,
    };
    reject_unknown(value)?;
    Ok(entry)
}

fn content_dependency_value(edge: &ContentDependencyEdgeV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "dependency_kind".to_owned(),
        string(edge.dependency_kind.as_str()),
    );
    value.insert(
        "required".to_owned(),
        string(if edge.required { "true" } else { "false" }),
    );
    value.insert(
        "source_asset_id".to_owned(),
        string(edge.source_asset_id.to_hex()),
    );
    value.insert(
        "target_asset_id".to_owned(),
        string(edge.target_asset_id.to_hex()),
    );
    JcsValue::Object(value)
}

fn decode_content_dependency(
    value: JcsValue,
) -> Result<ContentDependencyEdgeV1, ProjectContractError> {
    let mut value = object(value, "content_dependency")?;
    let edge = ContentDependencyEdgeV1 {
        source_asset_id: asset_id(take(&mut value, "source_asset_id")?, "source_asset_id")?,
        target_asset_id: asset_id(take(&mut value, "target_asset_id")?, "target_asset_id")?,
        dependency_kind: SchemaId::new(text(
            take(&mut value, "dependency_kind")?,
            "dependency_kind",
        )?)?,
        required: match text(take(&mut value, "required")?, "required")?.as_str() {
            "true" => true,
            "false" => false,
            _ => return Err(ProjectContractError::UnknownClosedValue),
        },
    };
    reject_unknown(value)?;
    Ok(edge)
}

fn validate_text(value: &str) -> Result<(), ProjectContractError> {
    if value.len() > 16 * 1024 || value.contains('\0') {
        return Err(ProjectContractError::InvalidText);
    }
    SchemaId::new(value.to_owned()).map_err(|_| ProjectContractError::InvalidText)?;
    Ok(())
}
