use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};

pub(super) fn core_rpg_policy_hash() -> ContentHash {
    domain_hash(
        b"nextengine.bootstrap-rpg-policy.v1\0",
        b"dialogue|quest|relationship|interactive-object",
    )
}

pub(super) fn domain_hash(domain: &[u8], body: &[u8]) -> ContentHash {
    let mut bytes = Vec::with_capacity(domain.len() + body.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(body);
    content_hash_from_bytes(sha256(&bytes))
}
