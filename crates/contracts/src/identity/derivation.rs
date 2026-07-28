use crate::canonical::{CanonicalError, sha256};
use crate::{
    CommandStreamId, ContentHash, IssuerPrincipal, PlayerPrincipalId, ProjectId, WorldNamespaceId,
    content_hash_from_bytes,
};

use super::runtime_profile::{RUNTIME_PROFILE_SCHEMA_ID, RuntimeDeterminismProfileV1};

pub fn derive_world_namespace(
    project_id: &ProjectId,
    world_creation_nonce: [u8; 32],
) -> Result<WorldNamespaceId, CanonicalError> {
    let project_bytes = project_id.as_str().as_bytes();
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.world-namespace.v1\0");
    extend_lp(&mut preimage, project_bytes)?;
    preimage.extend_from_slice(&world_creation_nonce);
    Ok(WorldNamespaceId::from_bytes(left128(sha256(&preimage))))
}

#[must_use]
pub fn derive_player_principal_id(
    world_namespace: WorldNamespaceId,
    player_slot: u32,
) -> PlayerPrincipalId {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.player-principal.v1\0");
    preimage.extend_from_slice(world_namespace.as_bytes());
    preimage.extend_from_slice(&player_slot.to_le_bytes());
    PlayerPrincipalId::from_bytes(left128(sha256(&preimage)))
}

pub fn derive_command_stream_id(
    world_namespace: WorldNamespaceId,
    principal: &IssuerPrincipal,
    stream_slot: u32,
    stream_epoch: u32,
) -> Result<CommandStreamId, CanonicalError> {
    let principal_bytes = principal.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.command-stream.v1\0");
    preimage.extend_from_slice(world_namespace.as_bytes());
    extend_lp(&mut preimage, &principal_bytes)?;
    preimage.extend_from_slice(&stream_slot.to_le_bytes());
    preimage.extend_from_slice(&stream_epoch.to_le_bytes());
    Ok(CommandStreamId::from_bytes(left128(sha256(&preimage))))
}

pub fn runtime_profile_hash(
    profile: &RuntimeDeterminismProfileV1,
) -> Result<ContentHash, CanonicalError> {
    let bytes = profile.canonical_bytes()?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"nextengine.runtime-profile.v1\0");
    extend_lp(&mut preimage, RUNTIME_PROFILE_SCHEMA_ID.as_bytes())?;
    extend_lp(&mut preimage, &bytes)?;
    Ok(content_hash_from_bytes(sha256(&preimage)))
}

fn left128(digest: [u8; 32]) -> [u8; 16] {
    let mut result = [0; 16];
    result.copy_from_slice(&digest[..16]);
    result
}

fn extend_lp(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CanonicalError> {
    output.extend_from_slice(
        &u64::try_from(bytes.len())
            .map_err(|_| CanonicalError::LengthOverflow)?
            .to_le_bytes(),
    );
    output.extend_from_slice(bytes);
    Ok(())
}
