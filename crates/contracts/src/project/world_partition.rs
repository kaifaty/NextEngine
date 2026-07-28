use std::collections::{BTreeMap, BTreeSet};

use crate::manifest_jcs::{JcsValue, decode_canonical_jcs, encode_canonical_jcs};
use crate::{AssetId, CanonicalDecodeLimits, ContentHash, SchemaId};

use super::codec::{
    ProjectContractError, array, asset_id, domain_hash, enforce_limit, ensure_unique, hash, object,
    reject_unknown, string, take, text, u32_number, u64_text,
};
use super::content::{AssetRevisionRefV1, asset_revision_value, decode_asset_revision};
use super::{PROJECT_MAX_RECORDS_V1, WORLD_PARTITION_MANIFEST_FORMAT_V1};

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
