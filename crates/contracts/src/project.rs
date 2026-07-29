mod activation;
mod codec;
mod content;
mod resolution;
mod schema;
mod world_partition;

pub use activation::ActivatedProjectV2;
pub use codec::{ProjectContractError, canonical_empty_manifest_hash, domain_hash};
pub use content::{
    AssetRevisionRefV1, ContentAssetEntryV1, ContentDependencyEdgeV1, ContentManifestBodyV1,
    ContentManifestV1, ContentProvenanceV1, ContentSemanticClassV1, EmptyContentManifestProfilesV1,
};
pub use resolution::{
    ProjectCatalogRecordV1, ProjectCatalogSnapshotV1, ProjectCompositionLockV2,
    ProjectDependencyKindV1, ProjectManifestV1, ProjectRequirementV1, ResolvedProjectRecordV1,
    SemanticVersionV1,
};
pub use schema::{
    SchemaDescriptorV1, SchemaEncodingV1, SchemaRefV1, SchemaRegistryManifestBodyV1,
    SchemaRegistryManifestV1, SchemaRoleV1,
};
pub use world_partition::{
    WorldChunkBindingV1, WorldPartitionManifestBodyV1, WorldPartitionManifestV1,
};

pub const PROJECT_MANIFEST_FORMAT_V1: &str = "nextengine.project-manifest.v1";
pub const PROJECT_CATALOG_FORMAT_V1: &str = "nextengine.project-catalog-snapshot.v1";
pub const PROJECT_COMPOSITION_LOCK_FORMAT_V2: &str = "nextengine.project-composition-lock.v2";
pub const SCHEMA_REGISTRY_MANIFEST_FORMAT_V1: &str = "nextengine.schema-registry-manifest.v1";
pub const CONTENT_MANIFEST_FORMAT_V1: &str = "nextengine.content-manifest.v1";
pub const WORLD_PARTITION_MANIFEST_FORMAT_V1: &str = "nextengine.world-partition-manifest.v1";
pub const PROJECT_MAX_RECORDS_V1: usize = 4_096;
pub const PROJECT_MAX_DEPENDENCIES_V1: usize = 16_384;

#[cfg(test)]
mod tests;
