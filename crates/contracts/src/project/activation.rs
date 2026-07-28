use super::codec::ProjectContractError;
use super::content::ContentManifestV1;
use super::resolution::ProjectCompositionLockV1;
use super::schema::SchemaRegistryManifestV1;
use super::world_partition::WorldPartitionManifestV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivatedProjectV1 {
    pub composition_lock: ProjectCompositionLockV1,
    pub schema_registry: SchemaRegistryManifestV1,
    pub content_manifest: ContentManifestV1,
    pub world_partition: WorldPartitionManifestV1,
    pub neutral_records: Vec<crate::NeutralRecordV1>,
    pub rpg_definitions: crate::RpgDefinitionRegistryV1,
}

impl ActivatedProjectV1 {
    pub fn validate(&self) -> Result<(), ProjectContractError> {
        self.composition_lock.validate()?;
        self.rpg_definitions
            .validate()
            .map_err(|_| ProjectContractError::HashMismatch)?;
        if self
            .neutral_records
            .windows(2)
            .any(|pair| pair[0].asset_id >= pair[1].asset_id)
        {
            return Err(ProjectContractError::DuplicateIdentity);
        }
        for record in &self.neutral_records {
            let record_hash = record
                .record_sha256()
                .map_err(|_| ProjectContractError::HashMismatch)?;
            if !self
                .content_manifest
                .body
                .asset_entries
                .iter()
                .any(|entry| {
                    entry.asset_revision.asset_id == record.asset_id
                        && entry.asset_revision.record_sha256 == record_hash
                })
            {
                return Err(ProjectContractError::HashMismatch);
            }
        }
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
