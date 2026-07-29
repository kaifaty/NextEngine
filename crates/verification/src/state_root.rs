use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{SchemaId, StateRoot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSegment {
    pub owner_id: SchemaId,
    pub schema_id: SchemaId,
    pub segment_id: SchemaId,
    pub canonical_bytes: Vec<u8>,
}

impl StateSegment {
    #[must_use]
    pub fn new(
        owner_id: SchemaId,
        schema_id: SchemaId,
        segment_id: SchemaId,
        canonical_bytes: Vec<u8>,
    ) -> Self {
        Self {
            owner_id,
            schema_id,
            segment_id,
            canonical_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateRootError {
    DuplicateSegment,
    LengthOverflow,
}

impl Display for StateRootError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::DuplicateSegment => "duplicate owner/schema/segment tuple in authoritative state",
            Self::LengthOverflow => "state segment identifier or payload exceeds canonical length",
        })
    }
}

impl Error for StateRootError {}

pub fn compute_state_root(
    segments: impl IntoIterator<Item = StateSegment>,
) -> Result<StateRoot, StateRootError> {
    let mut segments: Vec<_> = segments.into_iter().collect();
    segments.sort_by(|left, right| {
        (
            left.owner_id.as_str(),
            left.schema_id.as_str(),
            left.segment_id.as_str(),
        )
            .cmp(&(
                right.owner_id.as_str(),
                right.schema_id.as_str(),
                right.segment_id.as_str(),
            ))
    });
    if segments.windows(2).any(|pair| {
        pair[0].owner_id == pair[1].owner_id
            && pair[0].schema_id == pair[1].schema_id
            && pair[0].segment_id == pair[1].segment_id
    }) {
        return Err(StateRootError::DuplicateSegment);
    }

    let leaf_count = u64::try_from(segments.len()).map_err(|_| StateRootError::LengthOverflow)?;
    let mut nodes = Vec::with_capacity(segments.len());
    for segment in segments {
        let mut segment_preimage = Vec::new();
        segment_preimage.extend_from_slice(b"nextengine.state-segment.v1\0");
        segment_preimage.extend_from_slice(
            &u64::try_from(segment.canonical_bytes.len())
                .map_err(|_| StateRootError::LengthOverflow)?
                .to_le_bytes(),
        );
        segment_preimage.extend_from_slice(&segment.canonical_bytes);
        let segment_hash = sha256(&segment_preimage);

        let mut leaf_preimage = Vec::new();
        leaf_preimage.extend_from_slice(b"nextengine.state-leaf.v1\0");
        extend_identifier(&mut leaf_preimage, segment.owner_id.as_str())?;
        extend_identifier(&mut leaf_preimage, segment.schema_id.as_str())?;
        extend_identifier(&mut leaf_preimage, segment.segment_id.as_str())?;
        leaf_preimage.extend_from_slice(&segment_hash);
        nodes.push(sha256(&leaf_preimage));
    }

    let merkle_root = if nodes.is_empty() {
        sha256(b"nextengine.state-empty.v1\0")
    } else {
        while nodes.len() > 1 {
            let mut parents = Vec::with_capacity(nodes.len().div_ceil(2));
            for pair in nodes.chunks(2) {
                let mut preimage = Vec::new();
                if let [left, right] = pair {
                    preimage.extend_from_slice(b"nextengine.state-node.v1\0");
                    preimage.extend_from_slice(left);
                    preimage.extend_from_slice(right);
                } else {
                    preimage.extend_from_slice(b"nextengine.state-carry.v1\0");
                    preimage.extend_from_slice(&pair[0]);
                }
                parents.push(sha256(&preimage));
            }
            nodes = parents;
        }
        nodes[0]
    };

    let mut root_preimage = Vec::new();
    root_preimage.extend_from_slice(b"nextengine.state-root.v1\0");
    root_preimage.extend_from_slice(&leaf_count.to_le_bytes());
    root_preimage.extend_from_slice(&merkle_root);
    Ok(StateRoot::from_bytes(sha256(&root_preimage)))
}

fn extend_identifier(target: &mut Vec<u8>, value: &str) -> Result<(), StateRootError> {
    target.extend_from_slice(
        &u32::try_from(value.len())
            .map_err(|_| StateRootError::LengthOverflow)?
            .to_le_bytes(),
    );
    target.extend_from_slice(value.as_bytes());
    Ok(())
}
