mod activation;
mod codec;
mod content;
mod lock;
mod schema;
mod world_partition;

pub use activation::ActivatedProjectV8;
pub use codec::{ProjectContractError, canonical_empty_manifest_hash, domain_hash};
pub use content::{
    AssetRevisionRefV1, ContentAssetEntryV1, ContentDependencyEdgeV1, ContentManifestBodyV1,
    ContentManifestV1, ContentProvenanceV1, ContentSemanticClassV1, EmptyContentManifestProfilesV1,
};
pub use lock::ProjectLockV3;
pub use schema::{
    SchemaDescriptorV1, SchemaEncodingV1, SchemaRefV1, SchemaRegistryManifestBodyV2,
    SchemaRegistryManifestV2, SchemaRoleV1,
};
pub use world_partition::{
    WorldChunkBindingV1, WorldPartitionManifestBodyV1, WorldPartitionManifestV1,
};

pub const PROJECT_LOCK_FORMAT_V3: &str = "nextengine.project-lock.v3";
pub const SCHEMA_REGISTRY_MANIFEST_FORMAT_V2: &str = "nextengine.schema-registry-manifest.v2";
pub const CONTENT_MANIFEST_FORMAT_V1: &str = "nextengine.content-manifest.v1";
pub const WORLD_PARTITION_MANIFEST_FORMAT_V1: &str = "nextengine.world-partition-manifest.v1";
pub const PROJECT_MAX_RECORDS_V1: usize = 4_096;
pub const PROJECT_MAX_DEPENDENCIES_V1: usize = 16_384;

#[cfg(test)]
mod tests;
