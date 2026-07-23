use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{CapabilityId, IssuerPrincipal};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuthorityRegistry {
    principals: BTreeMap<IssuerPrincipal, BTreeSet<CapabilityId>>,
}

impl AuthorityRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            principals: BTreeMap::new(),
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
}

impl Display for AuthorityRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::DuplicatePrincipal => "issuer principal is already registered",
        })
    }
}

impl Error for AuthorityRegistryError {}
