use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldStreamingLoadMetricsV1 {
    pub asset_count: usize,
    pub encoded_bytes: usize,
    /// Minimum logical staging storage charged by the R3a vertical.
    pub required_staging_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorldChunkLoadRequestV1 {
    pub(super) plan: WorldStreamingPlanV1,
    pub(super) project_lock_hash: ContentHash,
    pub(super) schema_registry_hash: ContentHash,
    pub(super) content_generation_id: ContentHash,
    pub(super) topology_revision: u64,
    pub(super) chunk_asset: AssetRevisionRefV1,
    pub(super) ordered_asset_revisions: Vec<AssetRevisionRefV1>,
    pub(super) request_hash: ContentHash,
}

impl WorldChunkLoadRequestV1 {
    pub(super) fn computed_hash(&self) -> Result<ContentHash, WorldStreamingError> {
        let mut bytes = WORLD_CHUNK_REQUEST_DOMAIN_V1.to_vec();
        bytes.extend_from_slice(self.project_lock_hash.as_bytes());
        bytes.extend_from_slice(self.schema_registry_hash.as_bytes());
        bytes.extend_from_slice(self.content_generation_id.as_bytes());
        bytes.extend_from_slice(self.plan.expected_partition_manifest_hash.as_bytes());
        bytes.extend_from_slice(self.plan.expected_content_manifest_hash.as_bytes());
        bytes.extend_from_slice(&self.topology_revision.to_le_bytes());
        bytes.extend_from_slice(&self.plan.expected_generation.to_le_bytes());
        extend_text(&mut bytes, self.plan.target_chunk_id.as_str())?;
        bytes.extend_from_slice(self.chunk_asset.asset_id.as_bytes());
        bytes.extend_from_slice(self.chunk_asset.record_sha256.as_bytes());
        bytes.extend_from_slice(self.plan.plan_hash.as_bytes());
        extend_count(&mut bytes, self.ordered_asset_revisions.len())?;
        for revision in &self.ordered_asset_revisions {
            bytes.extend_from_slice(revision.asset_id.as_bytes());
            bytes.extend_from_slice(revision.record_sha256.as_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }

    pub(super) fn validate(&self) -> Result<(), WorldStreamingError> {
        self.plan.validate()?;
        if self.ordered_asset_revisions.len() > WORLD_CHUNK_MAX_ASSETS {
            return Err(WorldStreamingError::AssetCountLimit {
                actual: self.ordered_asset_revisions.len(),
                limit: WORLD_CHUNK_MAX_ASSETS,
            });
        }
        if self
            .ordered_asset_revisions
            .windows(2)
            .any(|pair| pair[0].asset_id >= pair[1].asset_id)
            || self
                .ordered_asset_revisions
                .iter()
                .map(|revision| revision.asset_id)
                .ne(self.plan.ordered_required_asset_ids.iter().copied())
            || self.computed_hash()? != self.request_hash
        {
            return Err(WorldStreamingError::RequestCorrupt);
        }
        Ok(())
    }
}
