use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::canonical::{CanonicalDecodeLimits, sha256};
use crate::ids::{AssetId, ContentHash, SchemaId, content_hash_from_bytes};
use crate::project::AssetRevisionRefV1;

pub const WORLD_STREAMING_SNAPSHOT_OWNER_ID: &str = "nextengine.world-services";
pub const WORLD_STREAMING_SNAPSHOT_SCHEMA_ID: &str = "nextengine.world-streaming-snapshot";
pub const WORLD_STREAMING_SNAPSHOT_SEGMENT_ID: &str = "v1";
pub const WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const WORLD_STREAMING_MAX_CHUNKS: usize = 1_024;
pub const WORLD_STREAMING_MAX_ASSETS_PER_GROUP: usize = 16_384;
const SNAPSHOT_PREFIX: &[u8] = b"nextengine.world-streaming-snapshot.v1\0";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WorldChunkLifecycleV1 {
    Absent = 1,
    Requested = 2,
    Staged = 3,
    Validated = 4,
    Active = 5,
    Quiescing = 6,
    Unloaded = 7,
    Failed = 8,
}

impl WorldChunkLifecycleV1 {
    fn from_tag(tag: u8) -> Result<Self, WorldStreamingContractError> {
        match tag {
            1 => Ok(Self::Absent),
            2 => Ok(Self::Requested),
            3 => Ok(Self::Staged),
            4 => Ok(Self::Validated),
            5 => Ok(Self::Active),
            6 => Ok(Self::Quiescing),
            7 => Ok(Self::Unloaded),
            8 => Ok(Self::Failed),
            _ => Err(WorldStreamingContractError::UnknownLifecycle(tag)),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldChunkResidencyRecordV1 {
    pub chunk_id: SchemaId,
    pub chunk_asset: AssetRevisionRefV1,
    pub lifecycle: WorldChunkLifecycleV1,
    pub lifecycle_revision: u64,
    pub required_asset_ids: Vec<AssetId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorldChunkTransitionV1 {
    pub source_chunk_id: SchemaId,
    pub target_chunk_id: SchemaId,
    pub requested_at_gameplay_tick: u64,
    pub expected_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldStreamingSnapshotV1 {
    pub partition_manifest_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub topology_revision: u64,
    pub generation: u64,
    pub current_chunk_id: SchemaId,
    pub chunks: Vec<WorldChunkResidencyRecordV1>,
    pub pending_transition: Option<WorldChunkTransitionV1>,
}

impl WorldStreamingSnapshotV1 {
    pub fn validate(&self) -> Result<(), WorldStreamingContractError> {
        if self.topology_revision == 0
            || self.chunks.is_empty()
            || self.chunks.len() > WORLD_STREAMING_MAX_CHUNKS
            || self
                .chunks
                .windows(2)
                .any(|pair| pair[0].chunk_id >= pair[1].chunk_id)
        {
            return Err(WorldStreamingContractError::SnapshotInvalid);
        }
        for chunk in &self.chunks {
            if chunk.required_asset_ids.len() > WORLD_STREAMING_MAX_ASSETS_PER_GROUP
                || !strictly_sorted(&chunk.required_asset_ids)
            {
                return Err(WorldStreamingContractError::SnapshotInvalid);
            }
        }
        let current = self
            .chunks
            .binary_search_by(|chunk| chunk.chunk_id.cmp(&self.current_chunk_id))
            .ok()
            .map(|index| &self.chunks[index])
            .ok_or(WorldStreamingContractError::SnapshotInvalid)?;
        if current.lifecycle != WorldChunkLifecycleV1::Active {
            return Err(WorldStreamingContractError::SnapshotInvalid);
        }
        let active_count = self
            .chunks
            .iter()
            .filter(|chunk| chunk.lifecycle == WorldChunkLifecycleV1::Active)
            .count();
        if active_count != 1 {
            return Err(WorldStreamingContractError::SnapshotInvalid);
        }
        if let Some(transition) = &self.pending_transition {
            let target = self
                .chunks
                .binary_search_by(|chunk| chunk.chunk_id.cmp(&transition.target_chunk_id))
                .ok()
                .map(|index| &self.chunks[index])
                .ok_or(WorldStreamingContractError::SnapshotInvalid)?;
            if transition.source_chunk_id != self.current_chunk_id
                || transition.target_chunk_id == transition.source_chunk_id
                || transition.expected_generation != self.generation
                || !matches!(
                    target.lifecycle,
                    WorldChunkLifecycleV1::Requested
                        | WorldChunkLifecycleV1::Staged
                        | WorldChunkLifecycleV1::Validated
                )
            {
                return Err(WorldStreamingContractError::SnapshotInvalid);
            }
        } else if self.chunks.iter().any(|chunk| {
            matches!(
                chunk.lifecycle,
                WorldChunkLifecycleV1::Requested
                    | WorldChunkLifecycleV1::Staged
                    | WorldChunkLifecycleV1::Validated
                    | WorldChunkLifecycleV1::Quiescing
            )
        }) {
            return Err(WorldStreamingContractError::SnapshotInvalid);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, WorldStreamingContractError> {
        self.validate()?;
        let mut bytes = SNAPSHOT_PREFIX.to_vec();
        bytes.extend_from_slice(&WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION.to_le_bytes());
        bytes.extend_from_slice(self.partition_manifest_hash.as_bytes());
        bytes.extend_from_slice(self.content_manifest_hash.as_bytes());
        bytes.extend_from_slice(&self.topology_revision.to_le_bytes());
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        extend_text(&mut bytes, self.current_chunk_id.as_str())?;
        extend_count(&mut bytes, self.chunks.len())?;
        for chunk in &self.chunks {
            extend_text(&mut bytes, chunk.chunk_id.as_str())?;
            bytes.extend_from_slice(chunk.chunk_asset.asset_id.as_bytes());
            bytes.extend_from_slice(chunk.chunk_asset.record_sha256.as_bytes());
            bytes.push(chunk.lifecycle as u8);
            bytes.extend_from_slice(&chunk.lifecycle_revision.to_le_bytes());
            extend_count(&mut bytes, chunk.required_asset_ids.len())?;
            for asset_id in &chunk.required_asset_ids {
                bytes.extend_from_slice(asset_id.as_bytes());
            }
        }
        match &self.pending_transition {
            None => bytes.push(0),
            Some(transition) => {
                bytes.push(1);
                extend_text(&mut bytes, transition.source_chunk_id.as_str())?;
                extend_text(&mut bytes, transition.target_chunk_id.as_str())?;
                bytes.extend_from_slice(&transition.requested_at_gameplay_tick.to_le_bytes());
                bytes.extend_from_slice(&transition.expected_generation.to_le_bytes());
            }
        }
        Ok(bytes)
    }

    pub fn from_canonical_bytes(
        bytes: &[u8],
        limits: CanonicalDecodeLimits,
    ) -> Result<Self, WorldStreamingContractError> {
        if bytes.len() > limits.max_total_bytes {
            return Err(WorldStreamingContractError::DecodeLimit);
        }
        let mut cursor = Cursor::new(bytes);
        if cursor.read_exact(SNAPSHOT_PREFIX.len())? != SNAPSHOT_PREFIX {
            return Err(WorldStreamingContractError::WrongEnvelope);
        }
        let version = cursor.read_u32()?;
        if version != WORLD_STREAMING_SNAPSHOT_SCHEMA_VERSION {
            return Err(WorldStreamingContractError::UnsupportedVersion(version));
        }
        let partition_manifest_hash = ContentHash::from_bytes(cursor.read_array()?);
        let content_manifest_hash = ContentHash::from_bytes(cursor.read_array()?);
        let topology_revision = cursor.read_u64()?;
        let generation = cursor.read_u64()?;
        let current_chunk_id = read_schema_id(&mut cursor, limits)?;
        let chunk_count = cursor.read_count(WORLD_STREAMING_MAX_CHUNKS)?;
        let mut chunks = Vec::with_capacity(chunk_count);
        for _ in 0..chunk_count {
            let chunk_id = read_schema_id(&mut cursor, limits)?;
            let chunk_asset = AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes(cursor.read_array()?),
                record_sha256: ContentHash::from_bytes(cursor.read_array()?),
            };
            let lifecycle = WorldChunkLifecycleV1::from_tag(cursor.read_u8()?)?;
            let lifecycle_revision = cursor.read_u64()?;
            let asset_count = cursor.read_count(WORLD_STREAMING_MAX_ASSETS_PER_GROUP)?;
            let mut required_asset_ids = Vec::with_capacity(asset_count);
            for _ in 0..asset_count {
                required_asset_ids.push(AssetId::from_bytes(cursor.read_array()?));
            }
            chunks.push(WorldChunkResidencyRecordV1 {
                chunk_id,
                chunk_asset,
                lifecycle,
                lifecycle_revision,
                required_asset_ids,
            });
        }
        let pending_transition = match cursor.read_u8()? {
            0 => None,
            1 => Some(WorldChunkTransitionV1 {
                source_chunk_id: read_schema_id(&mut cursor, limits)?,
                target_chunk_id: read_schema_id(&mut cursor, limits)?,
                requested_at_gameplay_tick: cursor.read_u64()?,
                expected_generation: cursor.read_u64()?,
            }),
            tag => return Err(WorldStreamingContractError::UnknownOption(tag)),
        };
        cursor.finish()?;
        let snapshot = Self {
            partition_manifest_hash,
            content_manifest_hash,
            topology_revision,
            generation,
            current_chunk_id,
            chunks,
            pending_transition,
        };
        snapshot.validate()?;
        if snapshot.canonical_bytes()? != bytes {
            return Err(WorldStreamingContractError::NonCanonical);
        }
        Ok(snapshot)
    }

    pub fn state_hash(&self) -> Result<ContentHash, WorldStreamingContractError> {
        Ok(domain_hash(
            "nextengine.world-streaming-state.v1",
            &self.canonical_bytes()?,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldStreamingPlanV1 {
    pub expected_generation: u64,
    pub expected_partition_manifest_hash: ContentHash,
    pub expected_content_manifest_hash: ContentHash,
    pub target_chunk_id: SchemaId,
    pub ordered_required_asset_ids: Vec<AssetId>,
    pub plan_hash: ContentHash,
}

impl WorldStreamingPlanV1 {
    pub fn new(
        expected_generation: u64,
        expected_partition_manifest_hash: ContentHash,
        expected_content_manifest_hash: ContentHash,
        target_chunk_id: SchemaId,
        mut ordered_required_asset_ids: Vec<AssetId>,
    ) -> Result<Self, WorldStreamingContractError> {
        ordered_required_asset_ids.sort_unstable();
        if ordered_required_asset_ids.len() > WORLD_STREAMING_MAX_ASSETS_PER_GROUP
            || ordered_required_asset_ids
                .windows(2)
                .any(|pair| pair[0] == pair[1])
        {
            return Err(WorldStreamingContractError::PlanInvalid);
        }
        let mut plan = Self {
            expected_generation,
            expected_partition_manifest_hash,
            expected_content_manifest_hash,
            target_chunk_id,
            ordered_required_asset_ids,
            plan_hash: ContentHash::default(),
        };
        plan.plan_hash = plan.computed_hash()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), WorldStreamingContractError> {
        if !strictly_sorted(&self.ordered_required_asset_ids)
            || self.computed_hash()? != self.plan_hash
        {
            return Err(WorldStreamingContractError::PlanInvalid);
        }
        Ok(())
    }

    fn computed_hash(&self) -> Result<ContentHash, WorldStreamingContractError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.expected_generation.to_le_bytes());
        bytes.extend_from_slice(self.expected_partition_manifest_hash.as_bytes());
        bytes.extend_from_slice(self.expected_content_manifest_hash.as_bytes());
        extend_text(&mut bytes, self.target_chunk_id.as_str())?;
        extend_count(&mut bytes, self.ordered_required_asset_ids.len())?;
        for asset_id in &self.ordered_required_asset_ids {
            bytes.extend_from_slice(asset_id.as_bytes());
        }
        Ok(domain_hash("nextengine.world-streaming-plan.v1", &bytes))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorldStreamingContractError {
    SnapshotInvalid,
    PlanInvalid,
    WrongEnvelope,
    UnsupportedVersion(u32),
    UnknownLifecycle(u8),
    UnknownOption(u8),
    DecodeLimit,
    Truncated,
    TrailingBytes,
    LengthOverflow,
    Identifier(crate::ids::IdentifierError),
    NonCanonical,
}

impl Display for WorldStreamingContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::SnapshotInvalid => "WORLD_STREAM_SNAPSHOT_INVALID",
            Self::PlanInvalid => "WORLD_STREAM_PLAN_INVALID",
            Self::WrongEnvelope => "WORLD_STREAM_SNAPSHOT_ENVELOPE_INVALID",
            Self::UnsupportedVersion(_) => "WORLD_STREAM_SCHEMA_UNSUPPORTED",
            Self::UnknownLifecycle(_) | Self::UnknownOption(_) => {
                "WORLD_STREAM_CLOSED_VALUE_INVALID"
            }
            Self::DecodeLimit => "WORLD_STREAM_DECODE_LIMIT",
            Self::Truncated => "WORLD_STREAM_INPUT_TRUNCATED",
            Self::TrailingBytes => "WORLD_STREAM_TRAILING_BYTES",
            Self::LengthOverflow => "WORLD_STREAM_LENGTH_OVERFLOW",
            Self::Identifier(_) => "WORLD_STREAM_IDENTIFIER_INVALID",
            Self::NonCanonical => "WORLD_STREAM_NONCANONICAL",
        })
    }
}

impl Error for WorldStreamingContractError {}

impl From<crate::ids::IdentifierError> for WorldStreamingContractError {
    fn from(value: crate::ids::IdentifierError) -> Self {
        Self::Identifier(value)
    }
}

fn domain_hash(domain: &str, body: &[u8]) -> ContentHash {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    bytes.extend_from_slice(body);
    content_hash_from_bytes(sha256(&bytes))
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), WorldStreamingContractError> {
    extend_count(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn extend_count(bytes: &mut Vec<u8>, count: usize) -> Result<(), WorldStreamingContractError> {
    bytes.extend_from_slice(
        &u32::try_from(count)
            .map_err(|_| WorldStreamingContractError::LengthOverflow)?
            .to_le_bytes(),
    );
    Ok(())
}

fn read_schema_id(
    cursor: &mut Cursor<'_>,
    limits: CanonicalDecodeLimits,
) -> Result<SchemaId, WorldStreamingContractError> {
    let length = cursor.read_count(limits.max_field_payload_bytes)?;
    let text = std::str::from_utf8(cursor.read_exact(length)?)
        .map_err(|_| WorldStreamingContractError::NonCanonical)?;
    Ok(SchemaId::new(text)?)
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], WorldStreamingContractError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(WorldStreamingContractError::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(WorldStreamingContractError::Truncated)?;
        self.position = end;
        Ok(value)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], WorldStreamingContractError> {
        self.read_exact(N)?
            .try_into()
            .map_err(|_| WorldStreamingContractError::Truncated)
    }

    fn read_u8(&mut self) -> Result<u8, WorldStreamingContractError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, WorldStreamingContractError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    fn read_u64(&mut self) -> Result<u64, WorldStreamingContractError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    fn read_count(&mut self, maximum: usize) -> Result<usize, WorldStreamingContractError> {
        let count = usize::try_from(self.read_u32()?)
            .map_err(|_| WorldStreamingContractError::LengthOverflow)?;
        if count > maximum {
            return Err(WorldStreamingContractError::DecodeLimit);
        }
        Ok(count)
    }

    fn finish(self) -> Result<(), WorldStreamingContractError> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(WorldStreamingContractError::TrailingBytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trip_and_plan_hash_are_canonical() {
        let chunks = vec![
            WorldChunkResidencyRecordV1 {
                chunk_id: SchemaId::new("nextengine.test.chunk.a").expect("ID"),
                chunk_asset: AssetRevisionRefV1 {
                    asset_id: AssetId::from_bytes([1; 16]),
                    record_sha256: ContentHash::from_bytes([1; 32]),
                },
                lifecycle: WorldChunkLifecycleV1::Active,
                lifecycle_revision: 1,
                required_asset_ids: vec![AssetId::from_bytes([3; 16])],
            },
            WorldChunkResidencyRecordV1 {
                chunk_id: SchemaId::new("nextengine.test.chunk.b").expect("ID"),
                chunk_asset: AssetRevisionRefV1 {
                    asset_id: AssetId::from_bytes([2; 16]),
                    record_sha256: ContentHash::from_bytes([2; 32]),
                },
                lifecycle: WorldChunkLifecycleV1::Unloaded,
                lifecycle_revision: 1,
                required_asset_ids: vec![AssetId::from_bytes([4; 16])],
            },
        ];
        let snapshot = WorldStreamingSnapshotV1 {
            partition_manifest_hash: ContentHash::from_bytes([5; 32]),
            content_manifest_hash: ContentHash::from_bytes([6; 32]),
            topology_revision: 1,
            generation: 0,
            current_chunk_id: chunks[0].chunk_id.clone(),
            chunks,
            pending_transition: None,
        };
        let bytes = snapshot.canonical_bytes().expect("encode");
        assert_eq!(
            WorldStreamingSnapshotV1::from_canonical_bytes(
                &bytes,
                CanonicalDecodeLimits::default()
            )
            .expect("decode"),
            snapshot
        );
        let plan = WorldStreamingPlanV1::new(
            0,
            snapshot.partition_manifest_hash,
            snapshot.content_manifest_hash,
            SchemaId::new("nextengine.test.chunk.b").expect("ID"),
            vec![AssetId::from_bytes([4; 16])],
        )
        .expect("plan");
        plan.validate().expect("plan validates");
    }
}
