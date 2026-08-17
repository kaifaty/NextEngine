use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::command::IssuerPrincipal;
use next_contracts::ids::{CapabilityId, ContentHash, PersistentId};
use next_contracts::physics::PHYSICAL_COMMAND_CAPABILITY_ID;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuthorityRegistry {
    principals: BTreeMap<IssuerPrincipal, BTreeSet<CapabilityId>>,
    root_motion_sources: BTreeMap<(IssuerPrincipal, PersistentId), (ContentHash, ContentHash)>,
}

impl AuthorityRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            principals: BTreeMap::new(),
            root_motion_sources: BTreeMap::new(),
        }
    }

    pub fn register(
        &mut self,
        principal: IssuerPrincipal,
        granted_capabilities: impl IntoIterator<Item = CapabilityId>,
    ) -> Result<(), AuthorityRegistryError> {
        if self.principals.contains_key(&principal) {
            return Err(AuthorityRegistryError::DuplicatePrincipal);
        }
        self.principals
            .insert(principal, granted_capabilities.into_iter().collect());
        Ok(())
    }

    /// Binds one physical-command principal and subject to the exact active
    /// animation profile/clip revisions allowed to produce root motion. This
    /// registry is reconstructed from the project lock on restore, like the
    /// existing capability grants; it is not another mutable gameplay owner.
    pub fn register_root_motion_source(
        &mut self,
        principal: IssuerPrincipal,
        subject_id: PersistentId,
        source_graph_hash: ContentHash,
        source_clip_hash: ContentHash,
    ) -> Result<(), AuthorityRegistryError> {
        let required_capability = CapabilityId::new(PHYSICAL_COMMAND_CAPABILITY_ID)
            .expect("engine-owned physical capability id is valid");
        let Some(grants) = self.principals.get(&principal) else {
            return Err(AuthorityRegistryError::UnknownPrincipal);
        };
        if !grants.contains(&required_capability) {
            return Err(AuthorityRegistryError::PhysicalCapabilityRequired);
        }
        if subject_id == PersistentId::default()
            || source_graph_hash == ContentHash::default()
            || source_clip_hash == ContentHash::default()
        {
            return Err(AuthorityRegistryError::RootMotionSourceInvalid);
        }
        let key = (principal, subject_id);
        if self.root_motion_sources.contains_key(&key) {
            return Err(AuthorityRegistryError::DuplicateRootMotionSource);
        }
        self.root_motion_sources
            .insert(key, (source_graph_hash, source_clip_hash));
        Ok(())
    }

    #[must_use]
    pub fn root_motion_source(
        &self,
        principal: &IssuerPrincipal,
        subject_id: PersistentId,
    ) -> Option<(ContentHash, ContentHash)> {
        self.root_motion_sources
            .get(&(principal.clone(), subject_id))
            .copied()
    }

    pub fn root_motion_sources(
        &self,
    ) -> impl ExactSizeIterator<Item = (&IssuerPrincipal, PersistentId, ContentHash, ContentHash)>
    {
        self.root_motion_sources.iter().map(
            |((principal, subject_id), (source_graph_hash, source_clip_hash))| {
                (
                    principal,
                    *subject_id,
                    *source_graph_hash,
                    *source_clip_hash,
                )
            },
        )
    }

    #[must_use]
    pub fn is_authenticated(&self, principal: &IssuerPrincipal) -> bool {
        self.principals.contains_key(principal)
    }

    #[must_use]
    pub fn grants(&self, principal: &IssuerPrincipal) -> Option<&BTreeSet<CapabilityId>> {
        self.principals.get(principal)
    }

    pub fn entries(
        &self,
    ) -> impl ExactSizeIterator<Item = (&IssuerPrincipal, &BTreeSet<CapabilityId>)> {
        self.principals.iter()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityRegistryError {
    DuplicatePrincipal,
    UnknownPrincipal,
    PhysicalCapabilityRequired,
    RootMotionSourceInvalid,
    DuplicateRootMotionSource,
}

impl Display for AuthorityRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::DuplicatePrincipal => "issuer principal is already registered",
            Self::UnknownPrincipal => "root-motion source principal is not registered",
            Self::PhysicalCapabilityRequired => {
                "root-motion source principal lacks the physical command capability"
            }
            Self::RootMotionSourceInvalid => "root-motion source binding is invalid",
            Self::DuplicateRootMotionSource => "root-motion source binding is duplicated",
        })
    }
}

impl Error for AuthorityRegistryError {}
