#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    ActivatedProjectV1, AssetId, AssetRevisionRefV1, ContentHash, NeutralRecordV1, PersistentId,
    SchemaId, WorldChunkLifecycleV1, WorldChunkResidencyRecordV1, WorldChunkTransitionV1,
    WorldStreamingContractError, WorldStreamingPlanV1, WorldStreamingSnapshotV1,
    content_hash_from_bytes, sha256,
};

const STAGED_WORLD_CHUNK_GROUP_DOMAIN_V1: &[u8] = b"nextengine.staged-world-chunk-group.v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedWorldChunkGroupV1 {
    pub plan: WorldStreamingPlanV1,
    pub chunk_asset: AssetRevisionRefV1,
    pub ordered_asset_revisions: Vec<AssetRevisionRefV1>,
    pub persistent_ids: Vec<PersistentId>,
    pub staging_hash: ContentHash,
}

impl StagedWorldChunkGroupV1 {
    pub fn validate(&self) -> Result<(), WorldStreamingError> {
        self.plan.validate()?;
        if self
            .ordered_asset_revisions
            .windows(2)
            .any(|pair| pair[0].asset_id >= pair[1].asset_id)
            || self
                .persistent_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self
                .ordered_asset_revisions
                .iter()
                .map(|reference| reference.asset_id)
                .collect::<Vec<_>>()
                != self.plan.ordered_required_asset_ids
            || self.computed_hash()? != self.staging_hash
        {
            return Err(WorldStreamingError::CorruptStagedGroup);
        }
        Ok(())
    }

    fn computed_hash(&self) -> Result<ContentHash, WorldStreamingError> {
        let mut bytes = STAGED_WORLD_CHUNK_GROUP_DOMAIN_V1.to_vec();
        bytes.extend_from_slice(self.plan.plan_hash.as_bytes());
        bytes.extend_from_slice(self.chunk_asset.asset_id.as_bytes());
        bytes.extend_from_slice(self.chunk_asset.record_sha256.as_bytes());
        extend_count(&mut bytes, self.ordered_asset_revisions.len())?;
        for reference in &self.ordered_asset_revisions {
            bytes.extend_from_slice(reference.asset_id.as_bytes());
            bytes.extend_from_slice(reference.record_sha256.as_bytes());
        }
        extend_count(&mut bytes, self.persistent_ids.len())?;
        for persistent_id in &self.persistent_ids {
            bytes.extend_from_slice(persistent_id.as_bytes());
        }
        Ok(content_hash_from_bytes(sha256(&bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldTransitionCommitV1 {
    pub source_chunk_id: SchemaId,
    pub target_chunk_id: SchemaId,
    pub committed_generation: u64,
    pub lifecycle_trace: Vec<(SchemaId, WorldChunkLifecycleV1)>,
    pub world_state_hash: ContentHash,
}

#[derive(Clone, Debug)]
pub struct WorldStreamerV1 {
    project: ActivatedProjectV1,
    snapshot: WorldStreamingSnapshotV1,
}

impl WorldStreamerV1 {
    pub fn activate(
        project: ActivatedProjectV1,
        initial_chunk_id: SchemaId,
    ) -> Result<Self, WorldStreamingError> {
        project.validate()?;
        let chunks = project
            .world_partition
            .body
            .chunk_bindings
            .iter()
            .map(|binding| {
                let mut required_asset_ids = binding.required_asset_ids.clone();
                required_asset_ids.push(binding.chunk_asset.asset_id);
                required_asset_ids.sort_unstable();
                required_asset_ids.dedup();
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
            current_chunk_id: initial_chunk_id,
            chunks,
            pending_transition: None,
        };
        snapshot.validate()?;
        Ok(Self { project, snapshot })
    }

    pub fn restore(
        project: ActivatedProjectV1,
        snapshot: WorldStreamingSnapshotV1,
    ) -> Result<Self, WorldStreamingError> {
        project.validate()?;
        snapshot.validate()?;
        let canonical = Self::activate(project.clone(), snapshot.current_chunk_id.clone())?;
        if snapshot.partition_manifest_hash
            != project.world_partition.world_partition_manifest_sha256
            || snapshot.content_manifest_hash != project.content_manifest.content_manifest_sha256
            || snapshot.topology_revision != project.world_partition.body.topology_revision
            || canonical
                .snapshot
                .chunks
                .iter()
                .map(chunk_definition)
                .collect::<Vec<_>>()
                != snapshot
                    .chunks
                    .iter()
                    .map(chunk_definition)
                    .collect::<Vec<_>>()
        {
            return Err(WorldStreamingError::ProjectMismatch);
        }
        Ok(Self { project, snapshot })
    }

    #[must_use]
    pub const fn snapshot(&self) -> &WorldStreamingSnapshotV1 {
        &self.snapshot
    }

    pub fn begin_transition(
        &mut self,
        target_chunk_id: SchemaId,
        gameplay_tick: u64,
    ) -> Result<WorldStreamingPlanV1, WorldStreamingError> {
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
        let plan = WorldStreamingPlanV1::new(
            self.snapshot.generation,
            self.snapshot.partition_manifest_hash,
            self.snapshot.content_manifest_hash,
            target_chunk_id.clone(),
            target.required_asset_ids.clone(),
        )?;
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
        self.snapshot = next;
        Ok(plan)
    }

    pub fn stage(
        &mut self,
        plan: &WorldStreamingPlanV1,
        worker_completion_order: &[AssetId],
    ) -> Result<StagedWorldChunkGroupV1, WorldStreamingError> {
        self.recheck_plan(plan, WorldChunkLifecycleV1::Requested)?;
        let mut received = worker_completion_order.to_vec();
        received.sort_unstable();
        if received != plan.ordered_required_asset_ids
            || worker_completion_order.len() != received.len()
        {
            return Err(WorldStreamingError::RequiredAssetUnavailable);
        }
        let entries = self
            .project
            .content_manifest
            .body
            .asset_entries
            .iter()
            .map(|entry| (entry.asset_revision.asset_id, entry.asset_revision))
            .collect::<BTreeMap<_, _>>();
        let records = self
            .project
            .neutral_records
            .iter()
            .map(|record| (record.asset_id, record))
            .collect::<BTreeMap<_, _>>();
        let mut ordered_asset_revisions = Vec::with_capacity(received.len());
        let mut persistent_ids = Vec::new();
        for asset_id in &received {
            let revision = entries
                .get(asset_id)
                .copied()
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
            let record = records
                .get(asset_id)
                .copied()
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
            if record.record_sha256()? != revision.record_sha256 {
                return Err(WorldStreamingError::CorruptStagedGroup);
            }
            ordered_asset_revisions.push(revision);
            persistent_ids.push(record.record_id);
        }
        persistent_ids.sort_unstable();
        if persistent_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WorldStreamingError::ObjectIdCollision);
        }
        let target_index = self.chunk_index(&plan.target_chunk_id)?;
        let chunk_asset = self.snapshot.chunks[target_index].chunk_asset;
        let mut staged = StagedWorldChunkGroupV1 {
            plan: plan.clone(),
            chunk_asset,
            ordered_asset_revisions,
            persistent_ids,
            staging_hash: ContentHash::default(),
        };
        staged.staging_hash = staged.computed_hash()?;
        staged.validate()?;
        let mut next = self.snapshot.clone();
        next.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Staged;
        next.chunks[target_index].lifecycle_revision = next.chunks[target_index]
            .lifecycle_revision
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        next.validate()?;
        self.snapshot = next;
        Ok(staged)
    }

    pub fn validate_staged(
        &mut self,
        staged: &StagedWorldChunkGroupV1,
    ) -> Result<(), WorldStreamingError> {
        self.recheck_plan(&staged.plan, WorldChunkLifecycleV1::Staged)?;
        staged.validate()?;
        let expected = self.rebuild_staged_group(&staged.plan)?;
        if &expected != staged {
            return Err(WorldStreamingError::CorruptStagedGroup);
        }
        let target_index = self.chunk_index(&staged.plan.target_chunk_id)?;
        let mut next = self.snapshot.clone();
        next.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Validated;
        next.chunks[target_index].lifecycle_revision = next.chunks[target_index]
            .lifecycle_revision
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        next.validate()?;
        self.snapshot = next;
        Ok(())
    }

    pub fn commit(
        &mut self,
        staged: &StagedWorldChunkGroupV1,
        inject_publication_fault: bool,
    ) -> Result<WorldTransitionCommitV1, WorldStreamingError> {
        self.recheck_plan(&staged.plan, WorldChunkLifecycleV1::Validated)?;
        staged.validate()?;
        let expected = self.rebuild_staged_group(&staged.plan)?;
        if &expected != staged {
            return Err(WorldStreamingError::CorruptStagedGroup);
        }
        let source_chunk_id = self.snapshot.current_chunk_id.clone();
        let source_index = self.chunk_index(&source_chunk_id)?;
        let target_index = self.chunk_index(&staged.plan.target_chunk_id)?;
        let mut next = self.snapshot.clone();
        let mut lifecycle_trace = Vec::with_capacity(3);
        next.chunks[source_index].lifecycle = WorldChunkLifecycleV1::Quiescing;
        next.chunks[source_index].lifecycle_revision = next.chunks[source_index]
            .lifecycle_revision
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        lifecycle_trace.push((source_chunk_id.clone(), WorldChunkLifecycleV1::Quiescing));
        next.chunks[source_index].lifecycle = WorldChunkLifecycleV1::Unloaded;
        next.chunks[source_index].lifecycle_revision = next.chunks[source_index]
            .lifecycle_revision
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        lifecycle_trace.push((source_chunk_id.clone(), WorldChunkLifecycleV1::Unloaded));
        next.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Active;
        next.chunks[target_index].lifecycle_revision = next.chunks[target_index]
            .lifecycle_revision
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        lifecycle_trace.push((
            staged.plan.target_chunk_id.clone(),
            WorldChunkLifecycleV1::Active,
        ));
        next.current_chunk_id = staged.plan.target_chunk_id.clone();
        next.generation = next
            .generation
            .checked_add(1)
            .ok_or(WorldStreamingError::Overflow)?;
        next.pending_transition = None;
        next.validate()?;
        let world_state_hash = next.state_hash()?;
        if inject_publication_fault {
            return Err(WorldStreamingError::PublicationAborted);
        }
        self.snapshot = next;
        Ok(WorldTransitionCommitV1 {
            source_chunk_id,
            target_chunk_id: staged.plan.target_chunk_id.clone(),
            committed_generation: self.snapshot.generation,
            lifecycle_trace,
            world_state_hash,
        })
    }

    pub fn resume_pending(
        &mut self,
    ) -> Result<(WorldStreamingPlanV1, StagedWorldChunkGroupV1), WorldStreamingError> {
        let pending = self
            .snapshot
            .pending_transition
            .clone()
            .ok_or(WorldStreamingError::NoPendingTransition)?;
        let target_index = self.chunk_index(&pending.target_chunk_id)?;
        let plan = WorldStreamingPlanV1::new(
            pending.expected_generation,
            self.snapshot.partition_manifest_hash,
            self.snapshot.content_manifest_hash,
            pending.target_chunk_id,
            self.snapshot.chunks[target_index]
                .required_asset_ids
                .clone(),
        )?;
        let staged = self.rebuild_staged_group(&plan)?;
        let lifecycle = self.snapshot.chunks[target_index].lifecycle;
        if lifecycle == WorldChunkLifecycleV1::Requested {
            self.snapshot.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Staged;
            self.snapshot.chunks[target_index].lifecycle_revision = self.snapshot.chunks
                [target_index]
                .lifecycle_revision
                .checked_add(1)
                .ok_or(WorldStreamingError::Overflow)?;
        }
        if matches!(
            self.snapshot.chunks[target_index].lifecycle,
            WorldChunkLifecycleV1::Staged
        ) {
            self.snapshot.chunks[target_index].lifecycle = WorldChunkLifecycleV1::Validated;
            self.snapshot.chunks[target_index].lifecycle_revision = self.snapshot.chunks
                [target_index]
                .lifecycle_revision
                .checked_add(1)
                .ok_or(WorldStreamingError::Overflow)?;
        }
        self.snapshot.validate()?;
        Ok((plan, staged))
    }

    fn rebuild_staged_group(
        &self,
        plan: &WorldStreamingPlanV1,
    ) -> Result<StagedWorldChunkGroupV1, WorldStreamingError> {
        plan.validate()?;
        let entries = self
            .project
            .content_manifest
            .body
            .asset_entries
            .iter()
            .map(|entry| (entry.asset_revision.asset_id, entry.asset_revision))
            .collect::<BTreeMap<_, _>>();
        let records = self
            .project
            .neutral_records
            .iter()
            .map(|record| (record.asset_id, record))
            .collect::<BTreeMap<_, _>>();
        let ordered_asset_revisions = plan
            .ordered_required_asset_ids
            .iter()
            .map(|asset_id| {
                entries
                    .get(asset_id)
                    .copied()
                    .ok_or(WorldStreamingError::RequiredAssetUnavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut persistent_ids = Vec::new();
        for asset_id in &plan.ordered_required_asset_ids {
            let record: &NeutralRecordV1 = records
                .get(asset_id)
                .copied()
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
            let revision = entries
                .get(asset_id)
                .ok_or(WorldStreamingError::RequiredAssetUnavailable)?;
            if record.record_sha256()? != revision.record_sha256 {
                return Err(WorldStreamingError::CorruptStagedGroup);
            }
            persistent_ids.push(record.record_id);
        }
        persistent_ids.sort_unstable();
        if persistent_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(WorldStreamingError::ObjectIdCollision);
        }
        let target_index = self.chunk_index(&plan.target_chunk_id)?;
        let mut staged = StagedWorldChunkGroupV1 {
            plan: plan.clone(),
            chunk_asset: self.snapshot.chunks[target_index].chunk_asset,
            ordered_asset_revisions,
            persistent_ids,
            staging_hash: ContentHash::default(),
        };
        staged.staging_hash = staged.computed_hash()?;
        Ok(staged)
    }

    fn recheck_plan(
        &self,
        plan: &WorldStreamingPlanV1,
        expected_lifecycle: WorldChunkLifecycleV1,
    ) -> Result<(), WorldStreamingError> {
        plan.validate()?;
        if plan.expected_generation != self.snapshot.generation
            || plan.expected_partition_manifest_hash != self.snapshot.partition_manifest_hash
            || plan.expected_content_manifest_hash != self.snapshot.content_manifest_hash
        {
            return Err(WorldStreamingError::PlanStale);
        }
        let target_index = self.chunk_index(&plan.target_chunk_id)?;
        let target = &self.snapshot.chunks[target_index];
        if target.lifecycle != expected_lifecycle
            || target.required_asset_ids != plan.ordered_required_asset_ids
            || self
                .snapshot
                .pending_transition
                .as_ref()
                .is_none_or(|transition| {
                    transition.target_chunk_id != plan.target_chunk_id
                        || transition.expected_generation != plan.expected_generation
                })
        {
            return Err(WorldStreamingError::PlanStale);
        }
        Ok(())
    }

    fn chunk_index(&self, chunk_id: &SchemaId) -> Result<usize, WorldStreamingError> {
        self.snapshot
            .chunks
            .binary_search_by(|chunk| chunk.chunk_id.cmp(chunk_id))
            .map_err(|_| WorldStreamingError::UnknownChunk)
    }
}

fn chunk_definition(
    chunk: &WorldChunkResidencyRecordV1,
) -> (&SchemaId, AssetRevisionRefV1, &[AssetId]) {
    (
        &chunk.chunk_id,
        chunk.chunk_asset,
        &chunk.required_asset_ids,
    )
}

fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), WorldStreamingError> {
    bytes.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| WorldStreamingError::Overflow)?
            .to_le_bytes(),
    );
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldStreamingError {
    Contract(WorldStreamingContractError),
    Project(next_contracts::ProjectContractError),
    Neutral(next_contracts::NeutralRecordError),
    UnknownChunk,
    ProjectMismatch,
    TransitionAlreadyPending,
    NoPendingTransition,
    InvalidLifecycle,
    PlanStale,
    RequiredAssetUnavailable,
    ObjectIdCollision,
    CorruptStagedGroup,
    PublicationAborted,
    Overflow,
}

impl Display for WorldStreamingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Contract(error) => return Display::fmt(error, formatter),
            Self::Project(error) => return Display::fmt(error, formatter),
            Self::Neutral(error) => return Display::fmt(error, formatter),
            Self::UnknownChunk => "WORLD_STREAM_CHUNK_UNKNOWN",
            Self::ProjectMismatch => "WORLD_STREAM_PROJECT_MISMATCH",
            Self::TransitionAlreadyPending => "WORLD_STREAM_TRANSITION_PENDING",
            Self::NoPendingTransition => "WORLD_STREAM_TRANSITION_MISSING",
            Self::InvalidLifecycle => "WORLD_STREAM_LIFECYCLE_INVALID",
            Self::PlanStale => "WORLD_STREAM_PLAN_STALE",
            Self::RequiredAssetUnavailable => "WORLD_STREAM_REQUIRED_UNAVAILABLE",
            Self::ObjectIdCollision => "WORLD_STREAM_OBJECT_ID_COLLISION",
            Self::CorruptStagedGroup => "WORLD_STREAM_STAGED_GROUP_CORRUPT",
            Self::PublicationAborted => "WORLD_STREAM_PUBLICATION_ABORTED",
            Self::Overflow => "WORLD_STREAM_OVERFLOW",
        })
    }
}

impl Error for WorldStreamingError {}

impl From<WorldStreamingContractError> for WorldStreamingError {
    fn from(value: WorldStreamingContractError) -> Self {
        Self::Contract(value)
    }
}

impl From<next_contracts::ProjectContractError> for WorldStreamingError {
    fn from(value: next_contracts::ProjectContractError) -> Self {
        Self::Project(value)
    }
}

impl From<next_contracts::NeutralRecordError> for WorldStreamingError {
    fn from(value: next_contracts::NeutralRecordError) -> Self {
        Self::Neutral(value)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use next_assets::ContentStore;
    use next_project::{activate_project, cook_project_v1, neutral_vertical_slice_source_v1};

    use super::*;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn worker_permutations_produce_the_same_atomic_transition() {
        let project = fixture_project("permutations");
        let initial = SchemaId::new("nextengine.fixture.chunk.start").expect("initial");
        let target = SchemaId::new("nextengine.fixture.chunk.frontier").expect("target");
        let mut forward =
            WorldStreamerV1::activate(project.clone(), initial.clone()).expect("activate");
        let plan = forward
            .begin_transition(target.clone(), 7)
            .expect("begin transition");
        let mut order = plan.ordered_required_asset_ids.clone();
        let staged_forward = forward.stage(&plan, &order).expect("stage forward");
        forward
            .validate_staged(&staged_forward)
            .expect("validate forward");
        let commit_forward = forward
            .commit(&staged_forward, false)
            .expect("commit forward");

        let mut reverse = WorldStreamerV1::activate(project, initial).expect("activate");
        let reverse_plan = reverse.begin_transition(target, 7).expect("begin reverse");
        order.reverse();
        let staged_reverse = reverse.stage(&reverse_plan, &order).expect("stage reverse");
        reverse
            .validate_staged(&staged_reverse)
            .expect("validate reverse");
        let commit_reverse = reverse
            .commit(&staged_reverse, false)
            .expect("commit reverse");

        assert_eq!(staged_forward, staged_reverse);
        assert_eq!(commit_forward, commit_reverse);
        assert_eq!(forward.snapshot(), reverse.snapshot());
    }

    #[test]
    fn transition_round_trip_preserves_generation_and_unloads_previous_chunk() {
        let project = fixture_project("round-trip");
        let start = SchemaId::new("nextengine.fixture.chunk.start").expect("start");
        let frontier = SchemaId::new("nextengine.fixture.chunk.frontier").expect("frontier");
        let mut streamer = WorldStreamerV1::activate(project, start.clone()).expect("activate");
        execute_transition(&mut streamer, frontier, 10);
        execute_transition(&mut streamer, start.clone(), 20);
        assert_eq!(streamer.snapshot().generation, 2);
        assert_eq!(streamer.snapshot().current_chunk_id, start);
        assert!(streamer.snapshot().pending_transition.is_none());
        assert_eq!(
            streamer
                .snapshot()
                .chunks
                .iter()
                .filter(|chunk| chunk.lifecycle == WorldChunkLifecycleV1::Active)
                .count(),
            1
        );
    }

    #[test]
    fn save_restore_reconstructs_pending_transition_without_worker_state() {
        let project = fixture_project("restore");
        let start = SchemaId::new("nextengine.fixture.chunk.start").expect("start");
        let frontier = SchemaId::new("nextengine.fixture.chunk.frontier").expect("frontier");
        let mut streamer = WorldStreamerV1::activate(project.clone(), start).expect("activate");
        let plan = streamer
            .begin_transition(frontier.clone(), 31)
            .expect("begin");
        let order = plan.ordered_required_asset_ids.clone();
        let _staged = streamer.stage(&plan, &order).expect("stage");
        let saved = streamer.snapshot().clone();
        let bytes = saved.canonical_bytes().expect("snapshot bytes");
        let decoded = WorldStreamingSnapshotV1::from_canonical_bytes(
            &bytes,
            next_contracts::CanonicalDecodeLimits::default(),
        )
        .expect("decode");
        let mut restored = WorldStreamerV1::restore(project, decoded).expect("restore");
        let (_, rebuilt) = restored.resume_pending().expect("resume");
        restored.commit(&rebuilt, false).expect("commit");
        assert_eq!(restored.snapshot().current_chunk_id, frontier);
        assert_eq!(restored.snapshot().generation, 1);
    }

    #[test]
    fn corrupt_collision_and_publication_fault_leave_previous_generation_intact() {
        let project = fixture_project("faults");
        let start = SchemaId::new("nextengine.fixture.chunk.start").expect("start");
        let frontier = SchemaId::new("nextengine.fixture.chunk.frontier").expect("frontier");
        let mut streamer = WorldStreamerV1::activate(project, start).expect("activate");
        let plan = streamer.begin_transition(frontier, 42).expect("begin");
        let order = plan.ordered_required_asset_ids.clone();
        let staged = streamer.stage(&plan, &order).expect("stage");
        streamer.validate_staged(&staged).expect("validate");
        let before = streamer.snapshot().clone();

        let mut corrupt = staged.clone();
        corrupt.staging_hash = ContentHash::from_bytes([0xee; 32]);
        assert_eq!(
            streamer.commit(&corrupt, false),
            Err(WorldStreamingError::CorruptStagedGroup)
        );
        assert_eq!(streamer.snapshot(), &before);

        let mut collision = staged.clone();
        collision.persistent_ids.push(collision.persistent_ids[0]);
        collision.persistent_ids.sort_unstable();
        collision.staging_hash = collision.computed_hash().expect("recompute");
        assert_eq!(
            streamer.commit(&collision, false),
            Err(WorldStreamingError::CorruptStagedGroup)
        );
        assert_eq!(streamer.snapshot(), &before);

        assert_eq!(
            streamer.commit(&staged, true),
            Err(WorldStreamingError::PublicationAborted)
        );
        assert_eq!(streamer.snapshot(), &before);
    }

    fn execute_transition(streamer: &mut WorldStreamerV1, target: SchemaId, tick: u64) {
        let plan = streamer
            .begin_transition(target, tick)
            .expect("begin transition");
        let staged = streamer
            .stage(&plan, &plan.ordered_required_asset_ids)
            .expect("stage");
        streamer.validate_staged(&staged).expect("validate");
        streamer.commit(&staged, false).expect("commit");
    }

    fn fixture_project(label: &str) -> ActivatedProjectV1 {
        let cooked = cook_project_v1(neutral_vertical_slice_source_v1().expect("fixture source"))
            .expect("cook fixture");
        let root = std::env::temp_dir().join(format!(
            "nextengine-world-{label}-{}-{}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let store = ContentStore::new(&root);
        store
            .publish(&cooked.publication().expect("publication"))
            .expect("publish");
        let project = activate_project(&store).expect("activate project");
        std::fs::remove_dir_all(root).expect("remove fixture");
        project
    }
}
