#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, mpsc};
use std::thread;

use next_assets::{ContentStoreError, PinnedContentGeneration};
use next_contracts::canonical::{CanonicalDecodeLimits, sha256};
use next_contracts::content::{NeutralRecordKindV1, NeutralRecordV1};
use next_contracts::ids::{AssetId, ContentHash, PersistentId, SchemaId, content_hash_from_bytes};
use next_contracts::project::{
    ActivatedProjectV5, AssetRevisionRefV1, ContentAssetEntryV1, ContentSemanticClassV1,
    WorldChunkBindingV1,
};
use next_contracts::world::{
    WorldChunkLifecycleV1, WorldChunkResidencyRecordV1, WorldChunkTransitionV1,
    WorldStreamingContractError, WorldStreamingPlanV1, WorldStreamingSnapshotV1,
};

const WORLD_CHUNK_REQUEST_DOMAIN_V1: &[u8] = b"nextengine.world-chunk-request.v1\0";
const WORLD_CHUNK_RESULT_DOMAIN_V1: &[u8] = b"nextengine.world-chunk-result.v1\0";
const WORLD_CHUNK_MAX_ASSETS: usize = 64;
const WORLD_CHUNK_MAX_BLOB_BYTES: usize = 1024 * 1024;
const WORLD_CHUNK_MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;
const WORLD_CHUNK_RESULT_CHANNEL_CAPACITY: usize = 64;
const WORLD_CHUNK_MAX_WORKERS: usize = 4;
pub const WORLD_CHUNK_DEFAULT_WORKERS: usize = 2;

mod commit;
mod error;
mod load;
mod load_result;
mod population;
mod request;
mod routine;

pub use commit::WorldTransitionCommitV1;
pub use error::{WorldAssetLoadErrorCodeV1, WorldStreamingError};
use load::{
    binding_asset_ids, chunk_definition, content_entries, declared_dependencies,
    eager_chunk_records, extend_count, extend_text, load_one_record, map_store_error,
    validate_record,
};
pub use population::{
    PopulationNavigationServiceReportV1, PreparedWorldPopulationPublicationV1,
    ValidatedWorldPopulationPublicationV1, WorldPopulationOwnerError, WorldPopulationOwnerV1,
    route_query,
};
use request::WorldChunkLoadRequestV1;
pub use request::WorldStreamingLoadMetricsV1;
pub use routine::{
    PreparedWorldRoutinePublicationV1, ValidatedWorldRoutinePublicationV1, WorldRoutineOwnerError,
    WorldRoutineOwnerV1,
};

/// Immutable, revision-bound output of the private R3a packaged workers.
///
/// Fields intentionally remain opaque: callers may inspect only the stable
/// hash and logical resource charge, then submit the result for validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedWorldChunkLoadV1 {
    request: WorldChunkLoadRequestV1,
    records: Vec<NeutralRecordV1>,
    ordered_record_ids: Vec<PersistentId>,
    result_hash: ContentHash,
    metrics: WorldStreamingLoadMetricsV1,
}

/// A complete next World generation which has not yet been published.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedWorldStreamingPublicationV1 {
    base_world_state_hash: ContentHash,
    expected_gameplay_tick: u64,
    next_snapshot: WorldStreamingSnapshotV1,
    next_active_records: Vec<NeutralRecordV1>,
    receipt: Option<WorldTransitionCommitV1>,
}

impl PreparedWorldStreamingPublicationV1 {
    #[must_use]
    pub const fn expected_gameplay_tick(&self) -> u64 {
        self.expected_gameplay_tick
    }

    #[must_use]
    pub fn next_world_state_hash(&self) -> ContentHash {
        // Prepared instances are constructed only after snapshot validation.
        self.next_snapshot
            .state_hash()
            .expect("validated prepared world snapshot")
    }

    #[must_use]
    pub const fn next_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.next_snapshot
    }
}

/// Runtime/World paired validation consumes a prepared publication and turns
/// it into this commit-only value. Publication itself performs no validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWorldStreamingPublicationV1 {
    base_world_state_hash: ContentHash,
    expected_gameplay_tick: u64,
    next_snapshot: WorldStreamingSnapshotV1,
    next_active_records: Vec<NeutralRecordV1>,
    receipt: Option<WorldTransitionCommitV1>,
}

impl ValidatedWorldStreamingPublicationV1 {
    #[must_use]
    pub const fn next_snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.next_snapshot
    }
}

#[derive(Clone, Debug)]
pub struct WorldStreamerV1 {
    project: Arc<ActivatedProjectV5>,
    content_generation: PinnedContentGeneration,
    snapshot: WorldStreamingSnapshotV1,
    active_records: Vec<NeutralRecordV1>,
}

impl WorldStreamerV1 {
    pub fn activate(
        project: ActivatedProjectV5,
        content_generation: PinnedContentGeneration,
        initial_chunk_id: SchemaId,
    ) -> Result<Self, WorldStreamingError> {
        project.validate()?;
        let chunks = project
            .world_partition
            .body
            .chunk_bindings
            .iter()
            .map(|binding| {
                let required_asset_ids = binding_asset_ids(binding);
                WorldChunkResidencyRecordV1 {
                    chunk_id: binding.chunk_id.clone(),
                    chunk_asset: binding.chunk_asset,
                    lifecycle: if binding.chunk_id == initial_chunk_id {
                        WorldChunkLifecycleV1::Active
                    } else {
                        WorldChunkLifecycleV1::Absent
                    },
                    lifecycle_revision: 1,
                    required_asset_ids,
                }
            })
            .collect::<Vec<_>>();
        if !chunks
            .iter()
            .any(|chunk| chunk.chunk_id == initial_chunk_id)
        {
            return Err(WorldStreamingError::UnknownChunk);
        }
        let snapshot = WorldStreamingSnapshotV1 {
            partition_manifest_hash: project.world_partition.world_partition_manifest_sha256,
            content_manifest_hash: project.content_manifest.content_manifest_sha256,
            topology_revision: project.world_partition.body.topology_revision,
            generation: 0,
            current_chunk_id: initial_chunk_id.clone(),
            chunks,
            pending_transition: None,
        };
        snapshot.validate()?;
        let active_records = eager_chunk_records(&project, &initial_chunk_id)?;
        Ok(Self {
            project: Arc::new(project),
            content_generation,
            snapshot,
            active_records,
        })
    }

    pub fn restore(
        project: ActivatedProjectV5,
        content_generation: PinnedContentGeneration,
        snapshot: WorldStreamingSnapshotV1,
    ) -> Result<Self, WorldStreamingError> {
        project.validate()?;
        snapshot.validate()?;
        let canonical = Self::activate(
            project.clone(),
            content_generation.clone(),
            snapshot.current_chunk_id.clone(),
        )?;
        if snapshot.partition_manifest_hash
            != project.world_partition.world_partition_manifest_sha256
            || snapshot.content_manifest_hash != project.content_manifest.content_manifest_sha256
            || snapshot.topology_revision != project.world_partition.body.topology_revision
            || canonical
                .snapshot
                .chunks
                .iter()
                .map(chunk_definition)
                .ne(snapshot.chunks.iter().map(chunk_definition))
        {
            return Err(WorldStreamingError::ProjectMismatch);
        }
        if let Some(pending) = &snapshot.pending_transition {
            let target = snapshot
                .chunks
                .iter()
                .find(|chunk| chunk.chunk_id == pending.target_chunk_id)
                .ok_or(WorldStreamingError::UnknownChunk)?;
            if !matches!(
                target.lifecycle,
                WorldChunkLifecycleV1::Requested
                    | WorldChunkLifecycleV1::Staged
                    | WorldChunkLifecycleV1::Validated
            ) {
                return Err(WorldStreamingError::InvalidLifecycle);
            }
        }
        Ok(Self {
            project: Arc::new(project),
            content_generation,
            snapshot,
            active_records: canonical.active_records,
        })
    }

    #[must_use]
    pub const fn snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.snapshot
    }

    #[must_use]
    pub fn active_records(&self) -> &[NeutralRecordV1] {
        &self.active_records
    }

    pub fn prepare_begin_transition(
        &self,
        target_chunk_id: SchemaId,
        gameplay_tick: u64,
    ) -> Result<PreparedWorldStreamingPublicationV1, WorldStreamingError> {
        if self.snapshot.pending_transition.is_some()
            || target_chunk_id == self.snapshot.current_chunk_id
        {
            return Err(WorldStreamingError::TransitionAlreadyPending);
        }
        let target_index = self.chunk_index(&target_chunk_id)?;
        let target = &self.snapshot.chunks[target_index];
        if matches!(
            target.lifecycle,
            WorldChunkLifecycleV1::Requested
                | WorldChunkLifecycleV1::Staged
                | WorldChunkLifecycleV1::Validated
                | WorldChunkLifecycleV1::Quiescing
                | WorldChunkLifecycleV1::Failed
        ) {
            return Err(WorldStreamingError::InvalidLifecycle);
        }
        let mut next = self.snapshot.clone();
        next.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Requested;
        next.chunks[target_index].lifecycle_revision = next.chunks[target_index]
            .lifecycle_revision
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        next.pending_transition = Some(WorldChunkTransitionV1 {
            source_chunk_id: self.snapshot.current_chunk_id.clone(),
            target_chunk_id,
            requested_at_gameplay_tick: gameplay_tick,
            expected_generation: self.snapshot.generation,
        });
        next.validate()?;
        Ok(PreparedWorldStreamingPublicationV1 {
            base_world_state_hash: self.snapshot.state_hash()?,
            expected_gameplay_tick: gameplay_tick,
            next_snapshot: next,
            next_active_records: self.active_records.clone(),
            receipt: None,
        })
    }

    pub fn load_pending(
        &self,
        worker_count: usize,
    ) -> Result<PreparedWorldChunkLoadV1, WorldStreamingError> {
        let request = self.pending_request()?;
        self.load_request(request, worker_count, None)
    }

    pub fn prepare_loaded_commit(
        &self,
        loaded: PreparedWorldChunkLoadV1,
        gameplay_tick: u64,
    ) -> Result<PreparedWorldStreamingPublicationV1, WorldStreamingError> {
        let expected_request = self.pending_request()?;
        self.validate_loaded_result(&loaded, &expected_request)?;
        let pending = self
            .snapshot
            .pending_transition
            .as_ref()
            .ok_or(WorldStreamingError::NoPendingTransition)?;
        if gameplay_tick <= pending.requested_at_gameplay_tick {
            return Err(WorldStreamingError::CompletionTickInvalid);
        }
        let source_chunk_id = pending.source_chunk_id.clone();
        let target_chunk_id = pending.target_chunk_id.clone();
        let source_index = self.chunk_index(&source_chunk_id)?;
        let target_index = self.chunk_index(&target_chunk_id)?;
        let mut next = self.snapshot.clone();
        let lifecycle_trace = vec![
            (source_chunk_id.clone(), WorldChunkLifecycleV1::Quiescing),
            (source_chunk_id.clone(), WorldChunkLifecycleV1::Unloaded),
            (target_chunk_id.clone(), WorldChunkLifecycleV1::Active),
        ];
        next.chunks[source_index].lifecycle = WorldChunkLifecycleV1::Unloaded;
        next.chunks[source_index].lifecycle_revision = next.chunks[source_index]
            .lifecycle_revision
            .checked_add(2)
            .ok_or(WorldStreamingError::Overflow)?;
        next.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Active;
        // Staged and Validated are reconstructible internal steps, so the live
        // Requested -> Active publication advances all three revisions.
        next.chunks[target_index].lifecycle_revision = next.chunks[target_index]
            .lifecycle_revision
            .checked_add(3)
            .ok_or(WorldStreamingError::Overflow)?;
        next.current_chunk_id = target_chunk_id.clone();
        next.generation = next
            .generation
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        next.pending_transition = None;
        next.validate()?;
        let world_state_hash = next.state_hash()?;
        Ok(PreparedWorldStreamingPublicationV1 {
            base_world_state_hash: self.snapshot.state_hash()?,
            expected_gameplay_tick: gameplay_tick,
            next_snapshot: next,
            next_active_records: loaded.records,
            receipt: Some(WorldTransitionCommitV1 {
                source_chunk_id,
                target_chunk_id,
                committed_generation: self.snapshot.generation + 1,
                lifecycle_trace,
                world_state_hash,
                load_metrics: Some(loaded.metrics),
            }),
        })
    }

    pub fn validate_prepared_publication(
        &self,
        prepared: PreparedWorldStreamingPublicationV1,
        gameplay_tick: u64,
    ) -> Result<ValidatedWorldStreamingPublicationV1, WorldStreamingError> {
        self.validate_prepared_stage(&prepared, gameplay_tick)?;
        Ok(ValidatedWorldStreamingPublicationV1 {
            base_world_state_hash: prepared.base_world_state_hash,
            expected_gameplay_tick: prepared.expected_gameplay_tick,
            next_snapshot: prepared.next_snapshot,
            next_active_records: prepared.next_active_records,
            receipt: prepared.receipt,
        })
    }

    /// Rechecks the concrete World stage without consuming it. Runtime calls
    /// this from its ingress pipeline immediately before the physical step;
    /// joint Runtime/World validation consumes the value later.
    pub fn validate_prepared_stage(
        &self,
        prepared: &PreparedWorldStreamingPublicationV1,
        gameplay_tick: u64,
    ) -> Result<(), WorldStreamingError> {
        if prepared.base_world_state_hash != self.snapshot.state_hash()?
            || prepared.expected_gameplay_tick != gameplay_tick
        {
            return Err(WorldStreamingError::PublicationStale);
        }
        prepared.next_snapshot.validate()?;
        Ok(())
    }

    /// Final read-only check of the captured streaming generation.
    pub fn preflight_validated_publication(
        &self,
        validated: &ValidatedWorldStreamingPublicationV1,
        gameplay_tick: u64,
    ) -> Result<(), WorldStreamingError> {
        if validated.base_world_state_hash != self.snapshot.state_hash()?
            || validated.expected_gameplay_tick != gameplay_tick
        {
            return Err(WorldStreamingError::PublicationStale);
        }
        Ok(())
    }

    /// Publishes an already jointly validated value. This operation is
    /// deliberately infallible so Runtime can commit its paired generation in
    /// the same final publication section.
    pub fn commit_validated_publication(
        &mut self,
        validated: ValidatedWorldStreamingPublicationV1,
    ) -> Option<WorldTransitionCommitV1> {
        self.snapshot = validated.next_snapshot;
        self.active_records = validated.next_active_records;
        validated.receipt
    }

    fn pending_request(&self) -> Result<WorldChunkLoadRequestV1, WorldStreamingError> {
        let pending = self
            .snapshot
            .pending_transition
            .as_ref()
            .ok_or(WorldStreamingError::NoPendingTransition)?;
        let target_index = self.chunk_index(&pending.target_chunk_id)?;
        if !matches!(
            self.snapshot.chunks[target_index].lifecycle,
            WorldChunkLifecycleV1::Requested
                | WorldChunkLifecycleV1::Staged
                | WorldChunkLifecycleV1::Validated
        ) {
            return Err(WorldStreamingError::InvalidLifecycle);
        }
        let target = &self.snapshot.chunks[target_index];
        let plan = WorldStreamingPlanV1::new(
            pending.expected_generation,
            self.snapshot.partition_manifest_hash,
            self.snapshot.content_manifest_hash,
            pending.target_chunk_id.clone(),
            target.required_asset_ids.clone(),
        )?;
        let binding = self.chunk_binding(&pending.target_chunk_id)?;
        if target.chunk_asset != binding.chunk_asset
            || target.required_asset_ids != binding_asset_ids(binding)
        {
            return Err(WorldStreamingError::ProjectMismatch);
        }
        let entries = content_entries(&self.project);
        let ordered_asset_revisions = plan
            .ordered_required_asset_ids
            .iter()
            .map(|asset_id| {
                entries
                    .get(asset_id)
                    .map(|entry| entry.asset_revision)
                    .ok_or(WorldStreamingError::RequiredAssetUnavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut request = WorldChunkLoadRequestV1 {
            plan,
            project_lock_hash: self.project.project_lock.project_lock_sha256,
            schema_registry_hash: self.project.schema_registry.schema_registry_manifest_sha256,
            content_generation_id: self.content_generation.generation_id(),
            topology_revision: self.snapshot.topology_revision,
            chunk_asset: binding.chunk_asset,
            ordered_asset_revisions,
            request_hash: ContentHash::default(),
        };
        request.request_hash = request.computed_hash()?;
        request.validate()?;
        Ok(request)
    }

    fn load_request(
        &self,
        request: WorldChunkLoadRequestV1,
        worker_count: usize,
        panic_asset: Option<AssetId>,
    ) -> Result<PreparedWorldChunkLoadV1, WorldStreamingError> {
        request.validate()?;
        if worker_count == 0 || worker_count > WORLD_CHUNK_MAX_WORKERS {
            return Err(WorldStreamingError::WorkerCountLimit {
                actual: worker_count,
                limit: WORLD_CHUNK_MAX_WORKERS,
            });
        }
        let entries = content_entries(&self.project);
        let known_assets = entries.keys().copied().collect::<BTreeSet<_>>();
        let declared_dependencies = declared_dependencies(&self.project);
        let mut total_bytes = 0_usize;
        let mut work = Vec::with_capacity(request.ordered_asset_revisions.len());
        for revision in &request.ordered_asset_revisions {
            let entry = entries
                .get(&revision.asset_id)
                .copied()
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
            let blob_len = self
                .content_generation
                .content_blob_len(entry.neutral_record_blob_sha256, WORLD_CHUNK_MAX_BLOB_BYTES)
                .map_err(|error| map_store_error(revision.asset_id, &error))?;
            total_bytes = total_bytes
                .checked_add(blob_len)
                .ok_or(WorldStreamingError::Overflow)?;
            if total_bytes > WORLD_CHUNK_MAX_TOTAL_BYTES {
                return Err(WorldStreamingError::EncodedBytesLimit {
                    actual: total_bytes,
                    limit: WORLD_CHUNK_MAX_TOTAL_BYTES,
                });
            }
            work.push((entry, blob_len));
        }

        let actual_workers = worker_count.min(work.len().max(1));
        let (sender, receiver) = mpsc::sync_channel(WORLD_CHUNK_RESULT_CHANNEL_CAPACITY);
        let mut panics = Vec::new();
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(actual_workers);
            for worker_index in 0..actual_workers {
                let sender = sender.clone();
                let generation = self.content_generation.clone();
                let worker_items = work
                    .iter()
                    .skip(worker_index)
                    .step_by(actual_workers)
                    .copied()
                    .collect::<Vec<_>>();
                let known_assets = &known_assets;
                let declared_dependencies = &declared_dependencies;
                handles.push((
                    worker_items
                        .first()
                        .map(|(entry, _)| entry.asset_revision.asset_id),
                    scope.spawn(move || {
                        for (entry, _blob_len) in worker_items {
                            let asset_id = entry.asset_revision.asset_id;
                            let result = catch_unwind(AssertUnwindSafe(|| {
                                if panic_asset == Some(asset_id) {
                                    panic!("injected R3a worker panic");
                                }
                                load_one_record(
                                    &generation,
                                    entry,
                                    known_assets,
                                    declared_dependencies,
                                )
                            }))
                            .unwrap_or(Err(WorldAssetLoadErrorCodeV1::WorkerPanic));
                            if sender.send((asset_id, result)).is_err() {
                                return;
                            }
                        }
                    }),
                ));
            }
            drop(sender);
            for (first_asset, handle) in handles {
                if handle.join().is_err() {
                    panics.push(first_asset.unwrap_or_default());
                }
            }
        });

        let mut records = BTreeMap::new();
        let mut failures = Vec::new();
        for (asset_id, result) in receiver {
            match result {
                Ok(record) => {
                    if records.insert(asset_id, record).is_some() {
                        failures.push((asset_id, WorldAssetLoadErrorCodeV1::AssetIdentity));
                    }
                }
                Err(code) => failures.push((asset_id, code)),
            }
        }
        for asset_id in panics {
            failures.push((asset_id, WorldAssetLoadErrorCodeV1::WorkerPanic));
        }
        if records.len() != request.ordered_asset_revisions.len() && failures.is_empty() {
            let missing = request
                .ordered_asset_revisions
                .iter()
                .find(|revision| !records.contains_key(&revision.asset_id))
                .map(|revision| revision.asset_id)
                .unwrap_or_default();
            failures.push((missing, WorldAssetLoadErrorCodeV1::WorkerPanic));
        }
        failures.sort_unstable();
        if let Some((asset_id, code)) = failures.first().copied() {
            return Err(WorldStreamingError::AssetLoad { asset_id, code });
        }

        let records = request
            .ordered_asset_revisions
            .iter()
            .map(|revision| {
                records
                    .remove(&revision.asset_id)
                    .ok_or(WorldStreamingError::RequiredAssetUnavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.assemble_result(request, records, total_bytes)
    }

    fn assemble_result(
        &self,
        request: WorldChunkLoadRequestV1,
        records: Vec<NeutralRecordV1>,
        encoded_bytes: usize,
    ) -> Result<PreparedWorldChunkLoadV1, WorldStreamingError> {
        if records.len() != request.ordered_asset_revisions.len()
            || records.iter().map(|record| record.asset_id).ne(request
                .ordered_asset_revisions
                .iter()
                .map(|value| value.asset_id))
        {
            return Err(WorldStreamingError::ResultCorrupt);
        }
        let mut ordered_record_ids = records
            .iter()
            .map(|record| record.record_id)
            .collect::<Vec<_>>();
        ordered_record_ids.sort_unstable();
        if ordered_record_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WorldStreamingError::ObjectIdCollision);
        }
        self.validate_chunk_record(&request, &records)?;
        let metrics = WorldStreamingLoadMetricsV1 {
            asset_count: records.len(),
            encoded_bytes,
            required_staging_bytes: encoded_bytes,
        };
        let mut result = PreparedWorldChunkLoadV1 {
            request,
            records,
            ordered_record_ids,
            result_hash: ContentHash::default(),
            metrics,
        };
        result.result_hash = result.computed_hash()?;
        Ok(result)
    }

    fn validate_loaded_result(
        &self,
        loaded: &PreparedWorldChunkLoadV1,
        expected_request: &WorldChunkLoadRequestV1,
    ) -> Result<(), WorldStreamingError> {
        if &loaded.request != expected_request
            || loaded.result_hash != loaded.computed_hash()?
            || loaded.metrics.asset_count != loaded.records.len()
            || loaded.metrics.required_staging_bytes != loaded.metrics.encoded_bytes
        {
            return Err(WorldStreamingError::ResultCorrupt);
        }
        let mut ids = loaded
            .records
            .iter()
            .map(|record| record.record_id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        if ids != loaded.ordered_record_ids {
            return Err(WorldStreamingError::ResultCorrupt);
        }
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WorldStreamingError::ObjectIdCollision);
        }
        let entries = content_entries(&self.project);
        let known_assets = entries.keys().copied().collect::<BTreeSet<_>>();
        let declared_dependencies = declared_dependencies(&self.project);
        for (record, revision) in loaded
            .records
            .iter()
            .zip(&expected_request.ordered_asset_revisions)
        {
            let entry = entries
                .get(&revision.asset_id)
                .copied()
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
            validate_record(entry, record, &known_assets, &declared_dependencies).map_err(
                |code| WorldStreamingError::AssetLoad {
                    asset_id: revision.asset_id,
                    code,
                },
            )?;
        }
        self.validate_chunk_record(expected_request, &loaded.records)
    }

    fn validate_chunk_record(
        &self,
        request: &WorldChunkLoadRequestV1,
        records: &[NeutralRecordV1],
    ) -> Result<(), WorldStreamingError> {
        let binding = self.chunk_binding(&request.plan.target_chunk_id)?;
        let chunk_record = records
            .iter()
            .find(|record| record.asset_id == binding.chunk_asset.asset_id)
            .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
        if chunk_record.kind != NeutralRecordKindV1::WorldChunk
            || chunk_record
                .asset_dependencies
                .iter()
                .any(|dependency| !binding.required_asset_ids.contains(dependency))
        {
            return Err(WorldStreamingError::ChunkDependencyMismatch);
        }
        Ok(())
    }

    fn chunk_binding(
        &self,
        chunk_id: &SchemaId,
    ) -> Result<&WorldChunkBindingV1, WorldStreamingError> {
        self.project
            .world_partition
            .body
            .chunk_bindings
            .binary_search_by(|binding| binding.chunk_id.cmp(chunk_id))
            .map(|index| &self.project.world_partition.body.chunk_bindings[index])
            .map_err(|_| WorldStreamingError::UnknownChunk)
    }

    fn chunk_index(&self, chunk_id: &SchemaId) -> Result<usize, WorldStreamingError> {
        self.snapshot
            .chunks
            .binary_search_by(|chunk| chunk.chunk_id.cmp(chunk_id))
            .map_err(|_| WorldStreamingError::UnknownChunk)
    }
}

#[cfg(test)]
mod tests;
