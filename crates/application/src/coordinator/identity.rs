use next_contracts::canonical::sha256;
use next_contracts::ids::{
    ApplicationSessionId, CloseRequestId, ContentHash, SessionRequestId, content_hash_from_bytes,
};
use next_contracts::session::{ApplicationSessionStatusV1, CompositionRootV1};

pub(super) fn derive_session_id(
    project_lock: ContentHash,
    root: CompositionRootV1,
    sequence: u64,
) -> ApplicationSessionId {
    let hash = domain_bytes(
        b"nextengine.application-session-id.v1\0",
        &[
            project_lock.as_bytes(),
            &[root as u8],
            &sequence.to_le_bytes(),
        ],
    );
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    ApplicationSessionId::from_bytes(bytes)
}

pub(super) fn derive_request_id(
    session_id: ApplicationSessionId,
    revision: u64,
    target: ApplicationSessionStatusV1,
    causal_hash: ContentHash,
) -> SessionRequestId {
    let hash = domain_bytes(
        b"nextengine.application-request-id.v1\0",
        &[
            session_id.as_bytes(),
            &revision.to_le_bytes(),
            &[target as u8],
            causal_hash.as_bytes(),
        ],
    );
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    SessionRequestId::from_bytes(bytes)
}

pub(super) fn derive_close_request_id(
    session_id: ApplicationSessionId,
    revision: u64,
    causal_hash: ContentHash,
) -> CloseRequestId {
    let hash = domain_bytes(
        b"nextengine.close-request-id.v1\0",
        &[
            session_id.as_bytes(),
            &revision.to_le_bytes(),
            causal_hash.as_bytes(),
        ],
    );
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    CloseRequestId::from_bytes(bytes)
}

pub(super) fn domain_hash(domain: &[u8], parts: &[&[u8]]) -> ContentHash {
    content_hash_from_bytes(domain_bytes(domain, parts))
}

fn domain_bytes(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut preimage = domain.to_vec();
    for part in parts {
        preimage.extend_from_slice(part);
    }
    sha256(&preimage)
}
