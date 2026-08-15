use super::*;

impl PreparedWorldChunkLoadV1 {
    #[must_use]
    pub const fn result_hash(&self) -> ContentHash {
        self.result_hash
    }

    #[must_use]
    pub const fn metrics(&self) -> WorldStreamingLoadMetricsV1 {
        self.metrics
    }

    pub(super) fn computed_hash(&self) -> Result<ContentHash, WorldStreamingError> {
        let mut bytes = WORLD_CHUNK_RESULT_DOMAIN_V1.to_vec();
        bytes.extend_from_slice(self.request.request_hash.as_bytes());
        extend_count(&mut bytes, self.request.ordered_asset_revisions.len())?;
        for (revision, record) in self
            .request
            .ordered_asset_revisions
            .iter()
            .zip(&self.records)
        {
            bytes.extend_from_slice(revision.asset_id.as_bytes());
            bytes.extend_from_slice(revision.record_sha256.as_bytes());
            bytes.extend_from_slice(record.record_id.as_bytes());
        }
        extend_count(&mut bytes, self.metrics.asset_count)?;
        extend_count(&mut bytes, self.metrics.encoded_bytes)?;
        extend_count(&mut bytes, self.metrics.required_staging_bytes)?;
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}
