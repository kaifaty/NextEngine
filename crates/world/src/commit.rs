use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldTransitionCommitV1 {
    pub source_chunk_id: SchemaId,
    pub target_chunk_id: SchemaId,
    pub committed_generation: u64,
    pub lifecycle_trace: Vec<(SchemaId, WorldChunkLifecycleV1)>,
    pub world_state_hash: ContentHash,
    pub load_metrics: Option<WorldStreamingLoadMetricsV1>,
}
