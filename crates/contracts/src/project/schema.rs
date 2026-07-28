use std::collections::{BTreeMap, BTreeSet};

use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
use crate::{CanonicalDecodeLimits, ContentHash, SchemaId};

use super::codec::{
    ProjectContractError, array, domain_hash, enforce_limit, ensure_unique, expect_format, hash,
    object, plain_jcs_hash, reject_unknown, string, take, text, u32_number, u64_text,
};
use super::{PROJECT_MAX_RECORDS_V1, SCHEMA_REGISTRY_MANIFEST_FORMAT_V1};

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

pub(super) fn schema_ref_value(schema_ref: &SchemaRefV1) -> JcsValue {
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

pub(super) fn decode_schema_ref(value: JcsValue) -> Result<SchemaRefV1, ProjectContractError> {
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
