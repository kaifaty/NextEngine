use super::hashes::*;
use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CausalIdentityKind {
    PlayerPrincipal = 1,
    CommandStream = 2,
    DomainEvent = 3,
    PersistentRecord = 4,
    RngStream = 5,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CausalIdentityKey {
    pub identity_kind: CausalIdentityKind,
    pub identity_bytes: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalIdentityRegistryV1 {
    pub schema_version: u16,
    pub world_namespace: WorldNamespaceId,
    pub bindings: BTreeMap<CausalIdentityKey, ContentHash>,
}

impl CausalIdentityRegistryV1 {
    pub fn compare_or_insert(
        &mut self,
        key: CausalIdentityKey,
        provenance_hash: ContentHash,
    ) -> Result<IdentityInsertResult, CommandLedgerError> {
        if self.schema_version != CAUSAL_IDENTITY_REGISTRY_SCHEMA_VERSION {
            return Err(CommandLedgerError::CausalIdentityRegistryMismatch);
        }
        match self.bindings.get(&key) {
            Some(existing) if *existing == provenance_hash => Ok(IdentityInsertResult::Existing),
            Some(_) => Ok(IdentityInsertResult::Collision),
            None => {
                let mut next = self.clone();
                next.bindings.insert(key, provenance_hash);
                *self = next;
                Ok(IdentityInsertResult::Inserted)
            }
        }
    }

    pub fn compare_or_insert_provenance(
        &mut self,
        identity_kind: CausalIdentityKind,
        identity_bytes: [u8; 16],
        canonical_provenance_bytes: &[u8],
    ) -> Result<IdentityInsertResult, CommandLedgerError> {
        let provenance_hash = causal_provenance_hash(identity_kind, canonical_provenance_bytes)?;
        self.compare_or_insert(
            CausalIdentityKey {
                identity_kind,
                identity_bytes,
            },
            provenance_hash,
        )
    }
}
