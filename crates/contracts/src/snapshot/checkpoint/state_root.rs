use crate::canonical::{CanonicalError, sha256};
use crate::ids::StateRoot;

pub(super) fn state_root_from_segments<const N: usize, B: AsRef<[u8]>>(
    segments: [(&str, &str, &str, B); N],
) -> Result<StateRoot, CanonicalError> {
    let segment_refs = segments
        .iter()
        .map(|(owner, schema, segment, bytes)| (*owner, *schema, *segment, bytes.as_ref()))
        .collect::<Vec<_>>();
    state_root_from_segment_slices(&segment_refs)
}

pub(super) fn state_root_from_segment_slices(
    segments: &[(&str, &str, &str, &[u8])],
) -> Result<StateRoot, CanonicalError> {
    let leaf_count = u64::try_from(segments.len()).map_err(|_| CanonicalError::LengthOverflow)?;
    let mut nodes = Vec::with_capacity(segments.len());
    for &(owner, schema, segment, bytes) in segments {
        let mut segment_hasher = sha2::Sha256::new();
        use sha2::Digest as _;
        segment_hasher.update(b"nextengine.state-segment.v1\0");
        segment_hasher.update(
            u64::try_from(bytes.len())
                .map_err(|_| CanonicalError::LengthOverflow)?
                .to_le_bytes(),
        );
        segment_hasher.update(bytes);

        let mut leaf_preimage = Vec::new();
        leaf_preimage.extend_from_slice(b"nextengine.state-leaf.v1\0");
        extend_state_root_identifier(&mut leaf_preimage, owner)?;
        extend_state_root_identifier(&mut leaf_preimage, schema)?;
        extend_state_root_identifier(&mut leaf_preimage, segment)?;
        leaf_preimage.extend_from_slice(&segment_hasher.finalize());
        nodes.push(sha256(&leaf_preimage));
    }
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
    let mut root_preimage = Vec::new();
    root_preimage.extend_from_slice(b"nextengine.state-root.v1\0");
    root_preimage.extend_from_slice(&leaf_count.to_le_bytes());
    root_preimage.extend_from_slice(
        nodes
            .first()
            .expect("a world checkpoint always contains owner segments"),
    );
    Ok(StateRoot::from_bytes(sha256(&root_preimage)))
}

/// Computes the canonical application root from validated save-segment
/// descriptors. Descriptor content hashes use the same state-segment domain
/// as raw checkpoint bytes, so no owner payload must be decoded a second time.
pub fn state_root_from_save_segment_descriptors(
    descriptors: &[crate::persistence::SaveSegmentDescriptor],
) -> Result<StateRoot, CanonicalError> {
    if descriptors.is_empty()
        || !descriptors.windows(2).all(|pair| {
            (&pair[0].owner_id, &pair[0].schema_id, &pair[0].segment_id)
                < (&pair[1].owner_id, &pair[1].schema_id, &pair[1].segment_id)
        })
    {
        return Err(CanonicalError::DuplicateSequenceValue);
    }
    let leaf_count =
        u64::try_from(descriptors.len()).map_err(|_| CanonicalError::LengthOverflow)?;
    let mut nodes = Vec::with_capacity(descriptors.len());
    for descriptor in descriptors {
        let mut leaf_preimage = Vec::new();
        leaf_preimage.extend_from_slice(b"nextengine.state-leaf.v1\0");
        extend_state_root_identifier(&mut leaf_preimage, descriptor.owner_id.as_str())?;
        extend_state_root_identifier(&mut leaf_preimage, descriptor.schema_id.as_str())?;
        extend_state_root_identifier(&mut leaf_preimage, descriptor.segment_id.as_str())?;
        leaf_preimage.extend_from_slice(descriptor.content_hash.as_bytes());
        nodes.push(sha256(&leaf_preimage));
    }
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
    let mut root_preimage = Vec::new();
    root_preimage.extend_from_slice(b"nextengine.state-root.v1\0");
    root_preimage.extend_from_slice(&leaf_count.to_le_bytes());
    root_preimage.extend_from_slice(&nodes[0]);
    Ok(StateRoot::from_bytes(sha256(&root_preimage)))
}

fn extend_state_root_identifier(
    target: &mut Vec<u8>,
    identifier: &str,
) -> Result<(), CanonicalError> {
    target.extend_from_slice(
        &u32::try_from(identifier.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    target.extend_from_slice(identifier.as_bytes());
    Ok(())
}
