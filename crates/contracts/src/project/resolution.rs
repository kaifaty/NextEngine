use std::collections::BTreeMap;

use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
use crate::{CanonicalDecodeLimits, ContentHash, ProjectId, SchemaId};

use super::codec::{
    ProjectContractError, array, domain_hash, enforce_limit, ensure_unique, expect_format, hash,
    object, plain_jcs_hash, reject_unknown, string, take, text, u32_number, u64_text,
};
use super::{
    PROJECT_CATALOG_FORMAT_V1, PROJECT_COMPOSITION_LOCK_FORMAT_V1, PROJECT_MANIFEST_FORMAT_V1,
    PROJECT_MAX_DEPENDENCIES_V1, PROJECT_MAX_RECORDS_V1,
};

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
