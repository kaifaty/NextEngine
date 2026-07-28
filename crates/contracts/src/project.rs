use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
use crate::{
    AssetId, CanonicalDecodeLimits, ContentHash, ManifestCodecError, ProjectId, SchemaId,
    content_hash_from_bytes, sha256,
};

pub const PROJECT_MANIFEST_FORMAT_V1: &str = "nextengine.project-manifest.v1";
pub const PROJECT_CATALOG_FORMAT_V1: &str = "nextengine.project-catalog-snapshot.v1";
pub const PROJECT_COMPOSITION_LOCK_FORMAT_V1: &str = "nextengine.project-composition-lock.v1";
pub const SCHEMA_REGISTRY_MANIFEST_FORMAT_V1: &str = "nextengine.schema-registry-manifest.v1";
pub const CONTENT_MANIFEST_FORMAT_V1: &str = "nextengine.content-manifest.v1";
pub const WORLD_PARTITION_MANIFEST_FORMAT_V1: &str = "nextengine.world-partition-manifest.v1";
pub const PROJECT_MAX_RECORDS_V1: usize = 4_096;
pub const PROJECT_MAX_DEPENDENCIES_V1: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SchemaEncodingV1 {
    CanonicalBinaryV1 = 1,
    JcsRfc8785 = 2,
}

impl SchemaEncodingV1 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalBinaryV1 => "canonical-binary-v1",
            Self::JcsRfc8785 => "jcs-rfc8785",
        }
    }

    fn from_str(value: &str) -> Result<Self, ProjectContractError> {
        match value {
            "canonical-binary-v1" => Ok(Self::CanonicalBinaryV1),
            "jcs-rfc8785" => Ok(Self::JcsRfc8785),
            _ => Err(ProjectContractError::UnknownClosedValue),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SchemaRoleV1 {
    Definition = 1,
    Manifest = 2,
}

impl SchemaRoleV1 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::Manifest => "manifest",
        }
    }

    fn from_str(value: &str) -> Result<Self, ProjectContractError> {
        match value {
            "definition" => Ok(Self::Definition),
            "manifest" => Ok(Self::Manifest),
            _ => Err(ProjectContractError::UnknownClosedValue),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaRefV1 {
    pub schema_id: SchemaId,
    pub schema_version: u32,
    pub descriptor_sha256: ContentHash,
    pub role: SchemaRoleV1,
    pub encoding: SchemaEncodingV1,
}

impl SchemaRefV1 {
    pub fn validate(&self) -> Result<(), ProjectContractError> {
        if self.schema_version == 0 {
            return Err(ProjectContractError::ZeroRevision);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaDescriptorV1 {
    pub schema_ref: SchemaRefV1,
    pub owner_context_id: SchemaId,
    pub field_registry_sha256: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaRegistryManifestBodyV1 {
    pub registry_revision: u64,
    pub canonicalization_profile_sha256: ContentHash,
    pub ownership_registry_sha256: ContentHash,
    pub descriptors: Vec<SchemaDescriptorV1>,
    pub current_schema_refs: Vec<SchemaRefV1>,
    pub migration_dag_sha256: ContentHash,
    pub registry_limits_sha256: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaRegistryManifestV1 {
    pub body: SchemaRegistryManifestBodyV1,
    pub schema_registry_manifest_sha256: ContentHash,
}

impl SchemaRegistryManifestV1 {
    pub fn new(mut body: SchemaRegistryManifestBodyV1) -> Result<Self, ProjectContractError> {
        body.descriptors.sort();
        body.current_schema_refs.sort();
        validate_schema_registry(&body)?;
        let bytes = schema_registry_body_value(&body)?;
        let hash = plain_jcs_hash(&bytes);
        Ok(Self {
            body,
            schema_registry_manifest_sha256: hash,
        })
    }

    pub fn empty(
        canonicalization_profile_sha256: ContentHash,
        ownership_registry_sha256: ContentHash,
        registry_limits_sha256: ContentHash,
    ) -> Result<Self, ProjectContractError> {
        Self::new(SchemaRegistryManifestBodyV1 {
            registry_revision: 1,
            canonicalization_profile_sha256,
            ownership_registry_sha256,
            descriptors: Vec::new(),
            current_schema_refs: Vec::new(),
            migration_dag_sha256: domain_hash("nextengine.schema-migration-dag.v1", &[]),
            registry_limits_sha256,
        })
    }

    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ProjectContractError> {
        let canonical = Self::new(self.body.clone())?;
        if canonical.schema_registry_manifest_sha256 != self.schema_registry_manifest_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
        schema_registry_body_value(&self.body)
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let body = decode_schema_registry_body(value)?;
        let manifest = Self::new(body)?;
        if manifest.to_jcs_bytes()? != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(manifest)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ProjectDependencyKindV1 {
    Content = 1,
    Mechanics = 2,
    Schema = 3,
}

impl ProjectDependencyKindV1 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Mechanics => "mechanics",
            Self::Schema => "schema",
        }
    }

    fn from_str(value: &str) -> Result<Self, ProjectContractError> {
        match value {
            "content" => Ok(Self::Content),
            "mechanics" => Ok(Self::Mechanics),
            "schema" => Ok(Self::Schema),
            _ => Err(ProjectContractError::UnknownClosedValue),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SemanticVersionV1 {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersionV1 {
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    #[must_use]
    pub fn canonical_text(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    fn parse(value: &str) -> Result<Self, ProjectContractError> {
        let mut components = value.split('.');
        let major = parse_semver_component(components.next())?;
        let minor = parse_semver_component(components.next())?;
        let patch = parse_semver_component(components.next())?;
        if components.next().is_some() {
            return Err(ProjectContractError::InvalidSemanticVersion);
        }
        Ok(Self::new(major, minor, patch))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProjectRequirementV1 {
    pub kind: ProjectDependencyKindV1,
    pub identity: SchemaId,
    pub minimum_version: SemanticVersionV1,
    pub optional: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectManifestV1 {
    pub project_id: ProjectId,
    pub project_revision: u64,
    pub requirements: Vec<ProjectRequirementV1>,
    pub manifest_sha256: ContentHash,
}

impl ProjectManifestV1 {
    pub fn new(
        project_id: ProjectId,
        project_revision: u64,
        mut requirements: Vec<ProjectRequirementV1>,
    ) -> Result<Self, ProjectContractError> {
        if project_revision == 0 {
            return Err(ProjectContractError::ZeroRevision);
        }
        requirements.sort();
        ensure_unique(
            requirements
                .iter()
                .map(|requirement| (requirement.kind, requirement.identity.as_str())),
        )?;
        enforce_limit(requirements.len(), PROJECT_MAX_DEPENDENCIES_V1)?;
        let mut manifest = Self {
            project_id,
            project_revision,
            requirements,
            manifest_sha256: ContentHash::default(),
        };
        manifest.manifest_sha256 = plain_jcs_hash(&manifest.body_bytes());
        Ok(manifest)
    }

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        self.body_bytes()
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let mut object = object(value, "project_manifest")?;
        expect_format(&mut object, PROJECT_MANIFEST_FORMAT_V1)?;
        let project_id = ProjectId::new(text(take(&mut object, "project_id")?, "project_id")?)?;
        let project_revision =
            u64_text(take(&mut object, "project_revision")?, "project_revision")?;
        let requirements = decode_requirements(take(&mut object, "requirements")?)?;
        reject_unknown(object)?;
        let manifest = Self::new(project_id, project_revision, requirements)?;
        if manifest.body_bytes() != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(manifest)
    }

    fn body_bytes(&self) -> Vec<u8> {
        let mut body = BTreeMap::new();
        body.insert(
            "manifest_format".to_owned(),
            string(PROJECT_MANIFEST_FORMAT_V1),
        );
        body.insert("project_id".to_owned(), string(self.project_id.as_str()));
        body.insert(
            "project_revision".to_owned(),
            string(self.project_revision.to_string()),
        );
        body.insert(
            "requirements".to_owned(),
            JcsValue::Array(self.requirements.iter().map(requirement_value).collect()),
        );
        encode_canonical_jcs(&JcsValue::Object(body))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectCatalogRecordV1 {
    pub kind: ProjectDependencyKindV1,
    pub identity: SchemaId,
    pub version: SemanticVersionV1,
    pub artifact_sha256: ContentHash,
    pub requirements: Vec<ProjectRequirementV1>,
    pub yanked: bool,
    pub record_sha256: ContentHash,
}

impl ProjectCatalogRecordV1 {
    pub fn new(
        kind: ProjectDependencyKindV1,
        identity: SchemaId,
        version: SemanticVersionV1,
        artifact_sha256: ContentHash,
        mut requirements: Vec<ProjectRequirementV1>,
        yanked: bool,
    ) -> Result<Self, ProjectContractError> {
        requirements.sort();
        ensure_unique(
            requirements
                .iter()
                .map(|requirement| (requirement.kind, requirement.identity.as_str())),
        )?;
        let mut record = Self {
            kind,
            identity,
            version,
            artifact_sha256,
            requirements,
            yanked,
            record_sha256: ContentHash::default(),
        };
        record.record_sha256 = plain_jcs_hash(&encode_canonical_jcs(&record_body_value(&record)));
        Ok(record)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectCatalogSnapshotV1 {
    pub resolver_profile_id: SchemaId,
    pub resolver_profile_version: u32,
    pub provenance_sha256: ContentHash,
    pub records: Vec<ProjectCatalogRecordV1>,
    pub catalog_snapshot_sha256: ContentHash,
}

impl ProjectCatalogSnapshotV1 {
    pub fn new(
        resolver_profile_id: SchemaId,
        resolver_profile_version: u32,
        provenance_sha256: ContentHash,
        mut records: Vec<ProjectCatalogRecordV1>,
    ) -> Result<Self, ProjectContractError> {
        if resolver_profile_version == 0 {
            return Err(ProjectContractError::ZeroRevision);
        }
        enforce_limit(records.len(), PROJECT_MAX_RECORDS_V1)?;
        for record in &records {
            validate_record_hash(record)?;
        }
        records.sort_by(|left, right| {
            (
                left.kind,
                left.identity.as_str(),
                left.version,
                left.record_sha256,
            )
                .cmp(&(
                    right.kind,
                    right.identity.as_str(),
                    right.version,
                    right.record_sha256,
                ))
        });
        ensure_unique(records.iter().map(|record| {
            (
                record.kind,
                record.identity.as_str(),
                record.version,
                record.record_sha256,
            )
        }))?;
        let mut snapshot = Self {
            resolver_profile_id,
            resolver_profile_version,
            provenance_sha256,
            records,
            catalog_snapshot_sha256: ContentHash::default(),
        };
        snapshot.catalog_snapshot_sha256 = plain_jcs_hash(&snapshot.body_bytes());
        Ok(snapshot)
    }

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        self.body_bytes()
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let mut object = object(value, "project_catalog")?;
        expect_format(&mut object, PROJECT_CATALOG_FORMAT_V1)?;
        let resolver_profile_id = SchemaId::new(text(
            take(&mut object, "resolver_profile_id")?,
            "resolver_profile_id",
        )?)?;
        let resolver_profile_version = u32_number(
            take(&mut object, "resolver_profile_version")?,
            "resolver_profile_version",
        )?;
        let provenance_sha256 = hash(take(&mut object, "provenance_sha256")?, "provenance_sha256")?;
        let records = decode_catalog_records(take(&mut object, "records")?)?;
        reject_unknown(object)?;
        let snapshot = Self::new(
            resolver_profile_id,
            resolver_profile_version,
            provenance_sha256,
            records,
        )?;
        if snapshot.body_bytes() != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(snapshot)
    }

    fn body_bytes(&self) -> Vec<u8> {
        let mut body = BTreeMap::new();
        body.insert(
            "catalog_format".to_owned(),
            string(PROJECT_CATALOG_FORMAT_V1),
        );
        body.insert(
            "provenance_sha256".to_owned(),
            string(self.provenance_sha256.to_hex()),
        );
        body.insert(
            "records".to_owned(),
            JcsValue::Array(
                self.records
                    .iter()
                    .map(|record| {
                        let mut value = match record_body_value(record) {
                            JcsValue::Object(value) => value,
                            _ => unreachable!("record body is an object"),
                        };
                        value.insert(
                            "record_sha256".to_owned(),
                            string(record.record_sha256.to_hex()),
                        );
                        JcsValue::Object(value)
                    })
                    .collect(),
            ),
        );
        body.insert(
            "resolver_profile_id".to_owned(),
            string(self.resolver_profile_id.as_str()),
        );
        body.insert(
            "resolver_profile_version".to_owned(),
            JcsValue::Number(u64::from(self.resolver_profile_version)),
        );
        encode_canonical_jcs(&JcsValue::Object(body))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResolvedProjectRecordV1 {
    pub kind: ProjectDependencyKindV1,
    pub identity: SchemaId,
    pub version: SemanticVersionV1,
    pub record_sha256: ContentHash,
    pub artifact_sha256: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectCompositionLockV1 {
    pub project_id: ProjectId,
    pub project_manifest_sha256: ContentHash,
    pub catalog_snapshot_sha256: ContentHash,
    pub resolver_profile_sha256: ContentHash,
    pub schema_registry_manifest_sha256: ContentHash,
    pub content_manifest_sha256: ContentHash,
    pub world_partition_manifest_sha256: ContentHash,
    pub mechanics_lock_sha256: ContentHash,
    pub selected_records: Vec<ResolvedProjectRecordV1>,
    pub composition_lock_sha256: ContentHash,
}

impl ProjectCompositionLockV1 {
    pub fn new(mut value: Self) -> Result<Self, ProjectContractError> {
        value.selected_records.sort();
        ensure_unique(
            value
                .selected_records
                .iter()
                .map(|record| (record.kind, record.identity.as_str())),
        )?;
        value.composition_lock_sha256 = ContentHash::default();
        value.composition_lock_sha256 =
            domain_hash(PROJECT_COMPOSITION_LOCK_FORMAT_V1, &value.body_bytes());
        Ok(value)
    }

    #[must_use]
    pub fn to_jcs_bytes(&self) -> Vec<u8> {
        self.body_bytes()
    }

    pub fn validate(&self) -> Result<(), ProjectContractError> {
        let canonical = Self::new(self.clone())?;
        if canonical.composition_lock_sha256 != self.composition_lock_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
        Ok(())
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let mut object = object(value, "project_lock")?;
        expect_format(&mut object, PROJECT_COMPOSITION_LOCK_FORMAT_V1)?;
        let lock = Self {
            project_id: ProjectId::new(text(take(&mut object, "project_id")?, "project_id")?)?,
            project_manifest_sha256: hash(
                take(&mut object, "project_manifest_sha256")?,
                "project_manifest_sha256",
            )?,
            catalog_snapshot_sha256: hash(
                take(&mut object, "catalog_snapshot_sha256")?,
                "catalog_snapshot_sha256",
            )?,
            resolver_profile_sha256: hash(
                take(&mut object, "resolver_profile_sha256")?,
                "resolver_profile_sha256",
            )?,
            schema_registry_manifest_sha256: hash(
                take(&mut object, "schema_registry_manifest_sha256")?,
                "schema_registry_manifest_sha256",
            )?,
            content_manifest_sha256: hash(
                take(&mut object, "content_manifest_sha256")?,
                "content_manifest_sha256",
            )?,
            world_partition_manifest_sha256: hash(
                take(&mut object, "world_partition_manifest_sha256")?,
                "world_partition_manifest_sha256",
            )?,
            mechanics_lock_sha256: hash(
                take(&mut object, "mechanics_lock_sha256")?,
                "mechanics_lock_sha256",
            )?,
            selected_records: decode_resolved_records(take(&mut object, "selected_records")?)?,
            composition_lock_sha256: ContentHash::default(),
        };
        reject_unknown(object)?;
        let lock = Self::new(lock)?;
        if lock.body_bytes() != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(lock)
    }

    fn body_bytes(&self) -> Vec<u8> {
        let mut body = BTreeMap::new();
        body.insert(
            "catalog_snapshot_sha256".to_owned(),
            string(self.catalog_snapshot_sha256.to_hex()),
        );
        body.insert(
            "content_manifest_sha256".to_owned(),
            string(self.content_manifest_sha256.to_hex()),
        );
        body.insert(
            "lock_format".to_owned(),
            string(PROJECT_COMPOSITION_LOCK_FORMAT_V1),
        );
        body.insert(
            "mechanics_lock_sha256".to_owned(),
            string(self.mechanics_lock_sha256.to_hex()),
        );
        body.insert("project_id".to_owned(), string(self.project_id.as_str()));
        body.insert(
            "project_manifest_sha256".to_owned(),
            string(self.project_manifest_sha256.to_hex()),
        );
        body.insert(
            "resolver_profile_sha256".to_owned(),
            string(self.resolver_profile_sha256.to_hex()),
        );
        body.insert(
            "schema_registry_manifest_sha256".to_owned(),
            string(self.schema_registry_manifest_sha256.to_hex()),
        );
        body.insert(
            "selected_records".to_owned(),
            JcsValue::Array(
                self.selected_records
                    .iter()
                    .map(resolved_record_value)
                    .collect(),
            ),
        );
        body.insert(
            "world_partition_manifest_sha256".to_owned(),
            string(self.world_partition_manifest_sha256.to_hex()),
        );
        encode_canonical_jcs(&JcsValue::Object(body))
    }
}

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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldChunkBindingV1 {
    pub chunk_id: SchemaId,
    pub region_id: SchemaId,
    pub chunk_asset: AssetRevisionRefV1,
    pub required_asset_ids: Vec<AssetId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPartitionManifestBodyV1 {
    pub partition_id: SchemaId,
    pub coordinate_profile_id: SchemaId,
    pub topology_revision: u64,
    pub root_region_ids: Vec<SchemaId>,
    pub chunk_bindings: Vec<WorldChunkBindingV1>,
    pub initial_placement_catalog_sha256: ContentHash,
    pub residency_policy_sha256: ContentHash,
    pub schema_registry_manifest_sha256: ContentHash,
    pub content_manifest_sha256: ContentHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPartitionManifestV1 {
    pub body: WorldPartitionManifestBodyV1,
    pub world_partition_manifest_sha256: ContentHash,
}

impl WorldPartitionManifestV1 {
    pub fn new(mut body: WorldPartitionManifestBodyV1) -> Result<Self, ProjectContractError> {
        if body.topology_revision == 0 {
            return Err(ProjectContractError::ZeroRevision);
        }
        body.root_region_ids
            .sort_by(|left, right| left.as_str().cmp(right.as_str()));
        body.chunk_bindings
            .sort_by(|left, right| left.chunk_id.as_str().cmp(right.chunk_id.as_str()));
        for binding in &mut body.chunk_bindings {
            binding.required_asset_ids.sort();
            ensure_unique(binding.required_asset_ids.iter())?;
        }
        ensure_unique(body.root_region_ids.iter().map(SchemaId::as_str))?;
        ensure_unique(
            body.chunk_bindings
                .iter()
                .map(|binding| binding.chunk_id.as_str()),
        )?;
        let roots: BTreeSet<_> = body.root_region_ids.iter().map(SchemaId::as_str).collect();
        if body
            .chunk_bindings
            .iter()
            .any(|binding| !roots.contains(binding.region_id.as_str()))
        {
            return Err(ProjectContractError::MissingReference);
        }
        enforce_limit(body.chunk_bindings.len(), PROJECT_MAX_RECORDS_V1)?;
        let bytes = world_partition_body_bytes(&body);
        Ok(Self {
            body,
            world_partition_manifest_sha256: domain_hash(
                WORLD_PARTITION_MANIFEST_FORMAT_V1,
                &bytes,
            ),
        })
    }

    pub fn empty(
        partition_id: SchemaId,
        coordinate_profile_id: SchemaId,
        schema_registry_manifest_sha256: ContentHash,
        content_manifest_sha256: ContentHash,
    ) -> Result<Self, ProjectContractError> {
        Self::new(WorldPartitionManifestBodyV1 {
            partition_id,
            coordinate_profile_id,
            topology_revision: 1,
            root_region_ids: Vec::new(),
            chunk_bindings: Vec::new(),
            initial_placement_catalog_sha256: domain_hash(
                "nextengine.initial-placement-catalog.v1",
                &[],
            ),
            residency_policy_sha256: domain_hash("nextengine.residency-policy.empty.v1", &[]),
            schema_registry_manifest_sha256,
            content_manifest_sha256,
        })
    }

    pub fn to_jcs_bytes(&self) -> Result<Vec<u8>, ProjectContractError> {
        let canonical = Self::new(self.body.clone())?;
        if canonical.world_partition_manifest_sha256 != self.world_partition_manifest_sha256 {
            return Err(ProjectContractError::HashMismatch);
        }
        Ok(world_partition_body_bytes(&self.body))
    }

    pub fn from_jcs_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, ProjectContractError> {
        let value = decode_canonical_jcs(bytes, limits)?;
        let body = decode_world_partition_body(value)?;
        let manifest = Self::new(body)?;
        if manifest.to_jcs_bytes()? != bytes {
            return Err(ProjectContractError::NonCanonical);
        }
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivatedProjectV1 {
    pub composition_lock: ProjectCompositionLockV1,
    pub schema_registry: SchemaRegistryManifestV1,
    pub content_manifest: ContentManifestV1,
    pub world_partition: WorldPartitionManifestV1,
    pub rpg_definitions: crate::RpgDefinitionRegistryV1,
}

impl ActivatedProjectV1 {
    pub fn validate(&self) -> Result<(), ProjectContractError> {
        self.composition_lock.validate()?;
        self.rpg_definitions
            .validate()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        if self.composition_lock.project_id != self.content_manifest.body.project_id
            || self.composition_lock.schema_registry_manifest_sha256
                != self.schema_registry.schema_registry_manifest_sha256
            || self.composition_lock.content_manifest_sha256
                != self.content_manifest.content_manifest_sha256
            || self.composition_lock.world_partition_manifest_sha256
                != self.world_partition.world_partition_manifest_sha256
            || self.content_manifest.body.schema_registry_manifest_sha256
                != self.schema_registry.schema_registry_manifest_sha256
            || self.world_partition.body.schema_registry_manifest_sha256
                != self.schema_registry.schema_registry_manifest_sha256
            || self.world_partition.body.content_manifest_sha256
                != self.content_manifest.content_manifest_sha256
            || self.composition_lock.mechanics_lock_sha256
                != self.rpg_definitions.mechanics_lock.mechanics_lock_sha256
        {
            return Err(ProjectContractError::HashMismatch);
        }
        self.schema_registry.to_jcs_bytes()?;
        self.content_manifest.to_jcs_bytes()?;
        self.world_partition.to_jcs_bytes()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProjectContractError {
    Manifest(ManifestCodecError),
    Identifier(crate::IdentifierError),
    ZeroRevision,
    DuplicateIdentity,
    LimitExceeded { actual: usize, limit: usize },
    HashMismatch,
    NonCanonical,
    UnknownClosedValue,
    InvalidSemanticVersion,
    MissingSchema,
    MissingReference,
    DependencyCycle,
    InvalidText,
}

impl Display for ProjectContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "project manifest is invalid: {error}"),
            Self::Identifier(error) => write!(formatter, "project identifier is invalid: {error}"),
            Self::ZeroRevision => formatter.write_str("project revision/version must be positive"),
            Self::DuplicateIdentity => formatter.write_str("project identity is duplicated"),
            Self::LimitExceeded { actual, limit } => {
                write!(formatter, "project count {actual} exceeds limit {limit}")
            }
            Self::HashMismatch => formatter.write_str("project artifact hash mismatch"),
            Self::NonCanonical => formatter.write_str("project artifact is not canonical"),
            Self::UnknownClosedValue => formatter.write_str("unknown closed project value"),
            Self::InvalidSemanticVersion => formatter.write_str("invalid semantic version"),
            Self::MissingSchema => formatter.write_str("schema reference is absent from registry"),
            Self::MissingReference => formatter.write_str("content reference is absent"),
            Self::DependencyCycle => formatter.write_str("required content dependency cycle"),
            Self::InvalidText => formatter.write_str("content text is not bounded NFC text"),
        }
    }
}

impl Error for ProjectContractError {}

impl From<ManifestCodecError> for ProjectContractError {
    fn from(error: ManifestCodecError) -> Self {
        Self::Manifest(error)
    }
}

impl From<crate::IdentifierError> for ProjectContractError {
    fn from(error: crate::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

fn validate_schema_registry(
    body: &SchemaRegistryManifestBodyV1,
) -> Result<(), ProjectContractError> {
    if body.registry_revision == 0 {
        return Err(ProjectContractError::ZeroRevision);
    }
    enforce_limit(body.descriptors.len(), PROJECT_MAX_RECORDS_V1)?;
    ensure_unique(
        body.descriptors
            .iter()
            .map(|descriptor| &descriptor.schema_ref),
    )?;
    ensure_unique(
        body.current_schema_refs
            .iter()
            .map(|schema_ref| schema_ref.schema_id.as_str()),
    )?;
    let descriptors: BTreeSet<_> = body
        .descriptors
        .iter()
        .map(|descriptor| descriptor.schema_ref.clone())
        .collect();
    for schema_ref in &body.current_schema_refs {
        schema_ref.validate()?;
        if !descriptors.contains(schema_ref) {
            return Err(ProjectContractError::MissingSchema);
        }
    }
    Ok(())
}

fn schema_registry_body_value(
    body: &SchemaRegistryManifestBodyV1,
) -> Result<Vec<u8>, ProjectContractError> {
    validate_schema_registry(body)?;
    let mut value = BTreeMap::new();
    value.insert(
        "canonicalization_profile_sha256".to_owned(),
        string(body.canonicalization_profile_sha256.to_hex()),
    );
    value.insert(
        "current_schema_refs".to_owned(),
        JcsValue::Array(
            body.current_schema_refs
                .iter()
                .map(schema_ref_value)
                .collect(),
        ),
    );
    value.insert(
        "descriptors".to_owned(),
        JcsValue::Array(
            body.descriptors
                .iter()
                .map(schema_descriptor_value)
                .collect(),
        ),
    );
    value.insert(
        "manifest_schema".to_owned(),
        string(SCHEMA_REGISTRY_MANIFEST_FORMAT_V1),
    );
    value.insert(
        "migration_dag_sha256".to_owned(),
        string(body.migration_dag_sha256.to_hex()),
    );
    value.insert(
        "ownership_registry_sha256".to_owned(),
        string(body.ownership_registry_sha256.to_hex()),
    );
    value.insert(
        "registry_limits_sha256".to_owned(),
        string(body.registry_limits_sha256.to_hex()),
    );
    value.insert(
        "registry_revision".to_owned(),
        string(body.registry_revision.to_string()),
    );
    Ok(encode_canonical_jcs(&JcsValue::Object(value)))
}

fn decode_schema_registry_body(
    value: JcsValue,
) -> Result<SchemaRegistryManifestBodyV1, ProjectContractError> {
    let mut object = object(value, "schema_registry")?;
    expect_format(&mut object, SCHEMA_REGISTRY_MANIFEST_FORMAT_V1)?;
    let body = SchemaRegistryManifestBodyV1 {
        registry_revision: u64_text(take(&mut object, "registry_revision")?, "registry_revision")?,
        canonicalization_profile_sha256: hash(
            take(&mut object, "canonicalization_profile_sha256")?,
            "canonicalization_profile_sha256",
        )?,
        ownership_registry_sha256: hash(
            take(&mut object, "ownership_registry_sha256")?,
            "ownership_registry_sha256",
        )?,
        descriptors: array(take(&mut object, "descriptors")?, "descriptors")?
            .into_iter()
            .map(decode_schema_descriptor)
            .collect::<Result<_, _>>()?,
        current_schema_refs: array(
            take(&mut object, "current_schema_refs")?,
            "current_schema_refs",
        )?
        .into_iter()
        .map(decode_schema_ref)
        .collect::<Result<_, _>>()?,
        migration_dag_sha256: hash(
            take(&mut object, "migration_dag_sha256")?,
            "migration_dag_sha256",
        )?,
        registry_limits_sha256: hash(
            take(&mut object, "registry_limits_sha256")?,
            "registry_limits_sha256",
        )?,
    };
    reject_unknown(object)?;
    Ok(body)
}

fn schema_ref_value(schema_ref: &SchemaRefV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "descriptor_sha256".to_owned(),
        string(schema_ref.descriptor_sha256.to_hex()),
    );
    value.insert("encoding".to_owned(), string(schema_ref.encoding.as_str()));
    value.insert("role".to_owned(), string(schema_ref.role.as_str()));
    value.insert(
        "schema_id".to_owned(),
        string(schema_ref.schema_id.as_str()),
    );
    value.insert(
        "schema_version".to_owned(),
        JcsValue::Number(u64::from(schema_ref.schema_version)),
    );
    JcsValue::Object(value)
}

fn decode_schema_ref(value: JcsValue) -> Result<SchemaRefV1, ProjectContractError> {
    let mut value = object(value, "schema_ref")?;
    let schema_ref = SchemaRefV1 {
        schema_id: SchemaId::new(text(take(&mut value, "schema_id")?, "schema_id")?)?,
        schema_version: u32_number(take(&mut value, "schema_version")?, "schema_version")?,
        descriptor_sha256: hash(take(&mut value, "descriptor_sha256")?, "descriptor_sha256")?,
        role: SchemaRoleV1::from_str(&text(take(&mut value, "role")?, "role")?)?,
        encoding: SchemaEncodingV1::from_str(&text(take(&mut value, "encoding")?, "encoding")?)?,
    };
    reject_unknown(value)?;
    schema_ref.validate()?;
    Ok(schema_ref)
}

fn schema_descriptor_value(descriptor: &SchemaDescriptorV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "field_registry_sha256".to_owned(),
        string(descriptor.field_registry_sha256.to_hex()),
    );
    value.insert(
        "owner_context_id".to_owned(),
        string(descriptor.owner_context_id.as_str()),
    );
    value.insert(
        "schema_ref".to_owned(),
        schema_ref_value(&descriptor.schema_ref),
    );
    JcsValue::Object(value)
}

fn decode_schema_descriptor(value: JcsValue) -> Result<SchemaDescriptorV1, ProjectContractError> {
    let mut value = object(value, "schema_descriptor")?;
    let descriptor = SchemaDescriptorV1 {
        schema_ref: decode_schema_ref(take(&mut value, "schema_ref")?)?,
        owner_context_id: SchemaId::new(text(
            take(&mut value, "owner_context_id")?,
            "owner_context_id",
        )?)?,
        field_registry_sha256: hash(
            take(&mut value, "field_registry_sha256")?,
            "field_registry_sha256",
        )?,
    };
    reject_unknown(value)?;
    Ok(descriptor)
}

fn requirement_value(requirement: &ProjectRequirementV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert("identity".to_owned(), string(requirement.identity.as_str()));
    value.insert("kind".to_owned(), string(requirement.kind.as_str()));
    value.insert(
        "minimum_version".to_owned(),
        string(requirement.minimum_version.canonical_text()),
    );
    value.insert(
        "optional".to_owned(),
        string(if requirement.optional {
            "true"
        } else {
            "false"
        }),
    );
    JcsValue::Object(value)
}

fn decode_requirements(value: JcsValue) -> Result<Vec<ProjectRequirementV1>, ProjectContractError> {
    array(value, "requirements")?
        .into_iter()
        .map(|value| {
            let mut value = object(value, "requirement")?;
            let requirement = ProjectRequirementV1 {
                kind: ProjectDependencyKindV1::from_str(&text(take(&mut value, "kind")?, "kind")?)?,
                identity: SchemaId::new(text(take(&mut value, "identity")?, "identity")?)?,
                minimum_version: SemanticVersionV1::parse(&text(
                    take(&mut value, "minimum_version")?,
                    "minimum_version",
                )?)?,
                optional: match text(take(&mut value, "optional")?, "optional")?.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(ProjectContractError::UnknownClosedValue),
                },
            };
            reject_unknown(value)?;
            Ok(requirement)
        })
        .collect()
}

fn record_body_value(record: &ProjectCatalogRecordV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "artifact_sha256".to_owned(),
        string(record.artifact_sha256.to_hex()),
    );
    value.insert("identity".to_owned(), string(record.identity.as_str()));
    value.insert("kind".to_owned(), string(record.kind.as_str()));
    value.insert(
        "requirements".to_owned(),
        JcsValue::Array(record.requirements.iter().map(requirement_value).collect()),
    );
    value.insert(
        "version".to_owned(),
        string(record.version.canonical_text()),
    );
    value.insert(
        "yanked".to_owned(),
        string(if record.yanked { "true" } else { "false" }),
    );
    JcsValue::Object(value)
}

fn validate_record_hash(record: &ProjectCatalogRecordV1) -> Result<(), ProjectContractError> {
    let actual = plain_jcs_hash(&encode_canonical_jcs(&record_body_value(record)));
    if actual != record.record_sha256 {
        return Err(ProjectContractError::HashMismatch);
    }
    Ok(())
}

fn decode_catalog_records(
    value: JcsValue,
) -> Result<Vec<ProjectCatalogRecordV1>, ProjectContractError> {
    array(value, "records")?
        .into_iter()
        .map(|value| {
            let mut value = object(value, "catalog_record")?;
            let kind =
                ProjectDependencyKindV1::from_str(&text(take(&mut value, "kind")?, "kind")?)?;
            let identity = SchemaId::new(text(take(&mut value, "identity")?, "identity")?)?;
            let version =
                SemanticVersionV1::parse(&text(take(&mut value, "version")?, "version")?)?;
            let artifact_sha256 = hash(take(&mut value, "artifact_sha256")?, "artifact_sha256")?;
            let requirements = decode_requirements(take(&mut value, "requirements")?)?;
            let yanked = match text(take(&mut value, "yanked")?, "yanked")?.as_str() {
                "true" => true,
                "false" => false,
                _ => return Err(ProjectContractError::UnknownClosedValue),
            };
            let expected = hash(take(&mut value, "record_sha256")?, "record_sha256")?;
            reject_unknown(value)?;
            let record = ProjectCatalogRecordV1::new(
                kind,
                identity,
                version,
                artifact_sha256,
                requirements,
                yanked,
            )?;
            if record.record_sha256 != expected {
                return Err(ProjectContractError::HashMismatch);
            }
            Ok(record)
        })
        .collect()
}

fn resolved_record_value(record: &ResolvedProjectRecordV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "artifact_sha256".to_owned(),
        string(record.artifact_sha256.to_hex()),
    );
    value.insert("identity".to_owned(), string(record.identity.as_str()));
    value.insert("kind".to_owned(), string(record.kind.as_str()));
    value.insert(
        "record_sha256".to_owned(),
        string(record.record_sha256.to_hex()),
    );
    value.insert(
        "version".to_owned(),
        string(record.version.canonical_text()),
    );
    JcsValue::Object(value)
}

fn decode_resolved_records(
    value: JcsValue,
) -> Result<Vec<ResolvedProjectRecordV1>, ProjectContractError> {
    array(value, "selected_records")?
        .into_iter()
        .map(|value| {
            let mut value = object(value, "selected_record")?;
            let record = ResolvedProjectRecordV1 {
                kind: ProjectDependencyKindV1::from_str(&text(take(&mut value, "kind")?, "kind")?)?,
                identity: SchemaId::new(text(take(&mut value, "identity")?, "identity")?)?,
                version: SemanticVersionV1::parse(&text(take(&mut value, "version")?, "version")?)?,
                record_sha256: hash(take(&mut value, "record_sha256")?, "record_sha256")?,
                artifact_sha256: hash(take(&mut value, "artifact_sha256")?, "artifact_sha256")?,
            };
            reject_unknown(value)?;
            Ok(record)
        })
        .collect()
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

fn asset_revision_value(reference: &AssetRevisionRefV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert("asset_id".to_owned(), string(reference.asset_id.to_hex()));
    value.insert(
        "record_sha256".to_owned(),
        string(reference.record_sha256.to_hex()),
    );
    JcsValue::Object(value)
}

fn decode_asset_revision(value: JcsValue) -> Result<AssetRevisionRefV1, ProjectContractError> {
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

fn world_partition_body_bytes(body: &WorldPartitionManifestBodyV1) -> Vec<u8> {
    let mut value = BTreeMap::new();
    value.insert(
        "chunk_bindings".to_owned(),
        JcsValue::Array(body.chunk_bindings.iter().map(world_chunk_value).collect()),
    );
    value.insert(
        "content_manifest_sha256".to_owned(),
        string(body.content_manifest_sha256.to_hex()),
    );
    value.insert(
        "coordinate_profile_id".to_owned(),
        string(body.coordinate_profile_id.as_str()),
    );
    value.insert(
        "initial_placement_catalog_sha256".to_owned(),
        string(body.initial_placement_catalog_sha256.to_hex()),
    );
    value.insert(
        "partition_id".to_owned(),
        string(body.partition_id.as_str()),
    );
    value.insert(
        "residency_policy_sha256".to_owned(),
        string(body.residency_policy_sha256.to_hex()),
    );
    value.insert(
        "root_region_ids".to_owned(),
        JcsValue::Array(
            body.root_region_ids
                .iter()
                .map(|region| string(region.as_str()))
                .collect(),
        ),
    );
    value.insert(
        "schema_registry_manifest_sha256".to_owned(),
        string(body.schema_registry_manifest_sha256.to_hex()),
    );
    value.insert("schema_version".to_owned(), JcsValue::Number(1));
    value.insert(
        "topology_revision".to_owned(),
        string(body.topology_revision.to_string()),
    );
    value.insert(
        "world_partition_format".to_owned(),
        string(WORLD_PARTITION_MANIFEST_FORMAT_V1),
    );
    encode_canonical_jcs(&JcsValue::Object(value))
}

fn decode_world_partition_body(
    value: JcsValue,
) -> Result<WorldPartitionManifestBodyV1, ProjectContractError> {
    let mut value = object(value, "world_partition")?;
    let format = text(
        take(&mut value, "world_partition_format")?,
        "world_partition_format",
    )?;
    if format != WORLD_PARTITION_MANIFEST_FORMAT_V1 {
        return Err(ProjectContractError::UnknownClosedValue);
    }
    if u32_number(take(&mut value, "schema_version")?, "schema_version")? != 1 {
        return Err(ProjectContractError::UnknownClosedValue);
    }
    let body = WorldPartitionManifestBodyV1 {
        partition_id: SchemaId::new(text(take(&mut value, "partition_id")?, "partition_id")?)?,
        coordinate_profile_id: SchemaId::new(text(
            take(&mut value, "coordinate_profile_id")?,
            "coordinate_profile_id",
        )?)?,
        topology_revision: u64_text(take(&mut value, "topology_revision")?, "topology_revision")?,
        root_region_ids: array(take(&mut value, "root_region_ids")?, "root_region_ids")?
            .into_iter()
            .map(|value| Ok(SchemaId::new(text(value, "region_id")?)?))
            .collect::<Result<_, ProjectContractError>>()?,
        chunk_bindings: array(take(&mut value, "chunk_bindings")?, "chunk_bindings")?
            .into_iter()
            .map(decode_world_chunk)
            .collect::<Result<_, _>>()?,
        initial_placement_catalog_sha256: hash(
            take(&mut value, "initial_placement_catalog_sha256")?,
            "initial_placement_catalog_sha256",
        )?,
        residency_policy_sha256: hash(
            take(&mut value, "residency_policy_sha256")?,
            "residency_policy_sha256",
        )?,
        schema_registry_manifest_sha256: hash(
            take(&mut value, "schema_registry_manifest_sha256")?,
            "schema_registry_manifest_sha256",
        )?,
        content_manifest_sha256: hash(
            take(&mut value, "content_manifest_sha256")?,
            "content_manifest_sha256",
        )?,
    };
    reject_unknown(value)?;
    Ok(body)
}

fn world_chunk_value(binding: &WorldChunkBindingV1) -> JcsValue {
    let mut value = BTreeMap::new();
    value.insert(
        "chunk_asset".to_owned(),
        asset_revision_value(&binding.chunk_asset),
    );
    value.insert("chunk_id".to_owned(), string(binding.chunk_id.as_str()));
    value.insert("region_id".to_owned(), string(binding.region_id.as_str()));
    value.insert(
        "required_asset_ids".to_owned(),
        JcsValue::Array(
            binding
                .required_asset_ids
                .iter()
                .map(|asset_id| string(asset_id.to_hex()))
                .collect(),
        ),
    );
    JcsValue::Object(value)
}

fn decode_world_chunk(value: JcsValue) -> Result<WorldChunkBindingV1, ProjectContractError> {
    let mut value = object(value, "world_chunk")?;
    let binding = WorldChunkBindingV1 {
        chunk_id: SchemaId::new(text(take(&mut value, "chunk_id")?, "chunk_id")?)?,
        region_id: SchemaId::new(text(take(&mut value, "region_id")?, "region_id")?)?,
        chunk_asset: decode_asset_revision(take(&mut value, "chunk_asset")?)?,
        required_asset_ids: array(
            take(&mut value, "required_asset_ids")?,
            "required_asset_ids",
        )?
        .into_iter()
        .map(|value| asset_id(value, "required_asset_id"))
        .collect::<Result<_, _>>()?,
    };
    reject_unknown(value)?;
    Ok(binding)
}

#[must_use]
pub fn domain_hash(domain: &str, bytes: &[u8]) -> ContentHash {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain.as_bytes());
    preimage.push(0);
    preimage.extend_from_slice(
        &u64::try_from(bytes.len())
            .expect("in-memory manifest byte length fits u64")
            .to_le_bytes(),
    );
    preimage.extend_from_slice(bytes);
    content_hash_from_bytes(sha256(&preimage))
}

#[must_use]
pub fn canonical_empty_manifest_hash(domain: &str) -> ContentHash {
    domain_hash(domain, b"{}")
}

fn plain_jcs_hash(bytes: &[u8]) -> ContentHash {
    content_hash_from_bytes(sha256(bytes))
}

fn string(value: impl Into<String>) -> JcsValue {
    JcsValue::String(value.into())
}

fn object(
    value: JcsValue,
    field: &'static str,
) -> Result<BTreeMap<String, JcsValue>, ProjectContractError> {
    match value {
        JcsValue::Object(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an object",
        }
        .into()),
    }
}

fn array(value: JcsValue, field: &'static str) -> Result<Vec<JcsValue>, ProjectContractError> {
    match value {
        JcsValue::Array(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an array",
        }
        .into()),
    }
}

fn text(value: JcsValue, field: &'static str) -> Result<String, ProjectContractError> {
    match value {
        JcsValue::String(value) => Ok(value),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "a string",
        }
        .into()),
    }
}

fn take(
    object: &mut BTreeMap<String, JcsValue>,
    field: &'static str,
) -> Result<JcsValue, ProjectContractError> {
    object
        .remove(field)
        .ok_or_else(|| ManifestCodecError::MissingField(field.to_owned()).into())
}

fn reject_unknown(object: BTreeMap<String, JcsValue>) -> Result<(), ProjectContractError> {
    if let Some((field, _)) = object.into_iter().next() {
        Err(ManifestCodecError::UnknownField(field).into())
    } else {
        Ok(())
    }
}

fn expect_format(
    object: &mut BTreeMap<String, JcsValue>,
    expected: &'static str,
) -> Result<(), ProjectContractError> {
    let field = if expected == PROJECT_CATALOG_FORMAT_V1 {
        "catalog_format"
    } else if expected == PROJECT_COMPOSITION_LOCK_FORMAT_V1 {
        "lock_format"
    } else if expected == SCHEMA_REGISTRY_MANIFEST_FORMAT_V1 {
        "manifest_schema"
    } else {
        "manifest_format"
    };
    if text(take(object, field)?, field)? != expected {
        return Err(ProjectContractError::UnknownClosedValue);
    }
    Ok(())
}

fn u32_number(value: JcsValue, field: &'static str) -> Result<u32, ProjectContractError> {
    match value {
        JcsValue::Number(value) => u32::try_from(value)
            .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()).into()),
        _ => Err(ManifestCodecError::WrongType {
            field: field.to_owned(),
            expected: "an unsigned JSON integer",
        }
        .into()),
    }
}

fn u64_text(value: JcsValue, field: &'static str) -> Result<u64, ProjectContractError> {
    let value = text(value, field)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ManifestCodecError::InvalidInteger(field.to_owned()).into());
    }
    value
        .parse()
        .map_err(|_| ManifestCodecError::InvalidInteger(field.to_owned()).into())
}

fn hash(value: JcsValue, field: &'static str) -> Result<ContentHash, ProjectContractError> {
    let value = text(value, field)?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()).into());
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(content_hash_from_bytes(bytes))
}

fn asset_id(value: JcsValue, field: &'static str) -> Result<AssetId, ProjectContractError> {
    let value = text(value, field)?;
    if value.len() != 32
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestCodecError::InvalidHex(field.to_owned()).into());
    }
    let mut bytes = [0_u8; 16];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]);
    }
    Ok(AssetId::from_bytes(bytes))
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

fn parse_semver_component(value: Option<&str>) -> Result<u32, ProjectContractError> {
    let value = value.ok_or(ProjectContractError::InvalidSemanticVersion)?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ProjectContractError::InvalidSemanticVersion);
    }
    value
        .parse()
        .map_err(|_| ProjectContractError::InvalidSemanticVersion)
}

fn ensure_unique<T: Ord>(values: impl IntoIterator<Item = T>) -> Result<(), ProjectContractError> {
    let values: Vec<_> = values.into_iter().collect();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ProjectContractError::DuplicateIdentity);
    }
    Ok(())
}

fn enforce_limit(actual: usize, limit: usize) -> Result<(), ProjectContractError> {
    if actual > limit {
        Err(ProjectContractError::LimitExceeded { actual, limit })
    } else {
        Ok(())
    }
}

fn validate_text(value: &str) -> Result<(), ProjectContractError> {
    if value.len() > 16 * 1024 || value.contains('\0') {
        return Err(ProjectContractError::InvalidText);
    }
    SchemaId::new(value.to_owned()).map_err(|_| ProjectContractError::InvalidText)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ContentManifestV1, EmptyContentManifestProfilesV1, ProjectManifestV1, SchemaEncodingV1,
        SchemaRefV1, SchemaRegistryManifestV1, SchemaRoleV1, canonical_empty_manifest_hash,
        domain_hash,
    };
    use crate::{CanonicalDecodeLimits, ContentHash, ProjectId, SchemaId};

    #[test]
    fn project_manifest_jcs_round_trip_rejects_noncanonical_bytes() {
        let manifest = ProjectManifestV1::new(
            ProjectId::new("org.nextengine.contract-test").expect("project"),
            1,
            Vec::new(),
        )
        .expect("manifest");
        let bytes = manifest.to_jcs_bytes();
        assert_eq!(
            ProjectManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("decode"),
            manifest
        );
        let mut noncanonical = bytes;
        noncanonical.insert(1, b' ');
        assert!(
            ProjectManifestV1::from_jcs_bytes(&noncanonical, CanonicalDecodeLimits::default())
                .is_err()
        );
    }

    #[test]
    fn empty_manifests_have_stable_nonzero_hashes() {
        let canonical = domain_hash("canonical", b"profile");
        let registry = SchemaRegistryManifestV1::empty(
            canonical,
            domain_hash("ownership", b"empty"),
            domain_hash("limits", b"bounded"),
        )
        .expect("empty registry");
        assert_ne!(
            registry.schema_registry_manifest_sha256,
            ContentHash::default()
        );
        let schema_ref = SchemaRefV1 {
            schema_id: SchemaId::new("nextengine.content.manifest").expect("schema"),
            schema_version: 1,
            descriptor_sha256: domain_hash("descriptor", b"content"),
            role: SchemaRoleV1::Manifest,
            encoding: SchemaEncodingV1::JcsRfc8785,
        };
        let content = ContentManifestV1::empty(
            schema_ref,
            SchemaId::new("nextengine.empty.content").expect("manifest ID"),
            ProjectId::new("org.nextengine.empty").expect("project"),
            EmptyContentManifestProfilesV1 {
                schema_registry_manifest_sha256: registry.schema_registry_manifest_sha256,
                canonicalization_profile_sha256: canonical,
                content_admission_limits_sha256: domain_hash("admission", b"empty"),
                cooker_contract_sha256: domain_hash("cooker", b"empty"),
                cooker_options_sha256: canonical_empty_manifest_hash("options"),
            },
        )
        .expect("empty content");
        let bytes = content.to_jcs_bytes().expect("canonical content");
        assert_eq!(
            ContentManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
                .expect("decode"),
            content
        );
    }
}
