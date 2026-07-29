use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::{
    CanonicalError, CapabilityId, CommandStreamId, CommandStreamRegistryV1, IssuerPrincipal,
    PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus, ProjectId,
    RuntimeDeterminismProfileV1, SchemaId, WorldIdentityManifestV1, content_hash_from_bytes,
    sha256,
};
use next_runtime::{
    AuthorityRegistry, AuthorityRegistryError, CommandKindRegistry, RuntimeBootstrapV3,
};

#[derive(Clone, Debug)]
pub struct NeutralRuntimeFixture {
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub streams: BTreeMap<IssuerPrincipal, CommandStreamId>,
}

impl NeutralRuntimeFixture {
    #[must_use]
    pub fn stream_for(&self, principal: &IssuerPrincipal) -> Option<CommandStreamId> {
        self.streams.get(principal).copied()
    }
}

pub fn build_neutral_runtime_fixture(
    project_id: &str,
    grants: impl IntoIterator<Item = (IssuerPrincipal, Vec<CapabilityId>)>,
) -> Result<NeutralRuntimeFixture, NeutralFixtureError> {
    let registry_hash = CommandKindRegistry::core_v1().canonical_hash();
    let profile = RuntimeDeterminismProfileV1::bootstrap_default(registry_hash);
    let world_identity = WorldIdentityManifestV1::new(
        ProjectId::new(project_id)?,
        sha256(format!("nextengine.fixture.nonce:{project_id}").as_bytes()),
        sha256(format!("nextengine.fixture.rng:{project_id}").as_bytes()),
        profile.profile_hash()?,
    )?;
    let mut principal_registry = PrincipalRegistryV1::empty(world_identity.world_namespace);
    let mut stream_registry = CommandStreamRegistryV1::empty(world_identity.world_namespace);
    let mut authority = AuthorityRegistry::new();
    let mut streams = BTreeMap::new();
    let mut grants: Vec<_> = grants.into_iter().collect();
    grants.sort_by(|left, right| left.0.cmp(&right.0));
    if grants.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(NeutralFixtureError::DuplicatePrincipal);
    }
    for (principal, capabilities) in grants {
        let principal_bytes = principal.canonical_bytes()?;
        let mut provenance = Vec::new();
        provenance.extend_from_slice(world_identity.world_namespace.as_bytes());
        provenance.extend_from_slice(&(principal_bytes.len() as u64).to_le_bytes());
        provenance.extend_from_slice(&principal_bytes);
        let provenance_hash = content_hash_from_bytes(sha256(&provenance));
        principal_registry.register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash,
                capability_subject_id: SchemaId::new(format!(
                    "fixture.principal.{}.{}",
                    principal.tag(),
                    hex_identifier(principal.identifier_bytes())
                ))?,
                status: PrincipalStatus::Active,
            },
        )?;
        let stream_id = stream_registry.allocate_stream(principal.clone())?;
        authority.register(principal.clone(), capabilities)?;
        streams.insert(principal, stream_id);
    }
    Ok(NeutralRuntimeFixture {
        bootstrap: RuntimeBootstrapV3::new(
            world_identity,
            principal_registry,
            stream_registry,
            profile,
        ),
        authority,
        streams,
    })
}

#[derive(Debug)]
pub enum NeutralFixtureError {
    Canonical(CanonicalError),
    Identifier(next_contracts::IdentifierError),
    Identity(next_contracts::IdentityContractError),
    Physics(next_contracts::PhysicsContractError),
    Authority(AuthorityRegistryError),
    ProjectCook(next_project::ProjectCookError),
    ProjectStore(next_assets::ContentStoreError),
    ProjectActivation(next_project::ProjectActivationError),
    DuplicatePrincipal,
}

impl Display for NeutralFixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canonical(error) => write!(formatter, "fixture canonicalization failed: {error}"),
            Self::Identifier(error) => write!(formatter, "fixture identifier failed: {error}"),
            Self::Identity(error) => write!(formatter, "fixture identity failed: {error}"),
            Self::Physics(error) => write!(formatter, "fixture physics failed: {error}"),
            Self::Authority(error) => write!(formatter, "fixture authority failed: {error}"),
            Self::ProjectCook(error) => write!(formatter, "fixture project cook failed: {error}"),
            Self::ProjectStore(error) => {
                write!(formatter, "fixture project publication failed: {error}")
            }
            Self::ProjectActivation(error) => {
                write!(formatter, "fixture project activation failed: {error}")
            }
            Self::DuplicatePrincipal => formatter.write_str("fixture principal is duplicated"),
        }
    }
}

impl Error for NeutralFixtureError {}

impl From<CanonicalError> for NeutralFixtureError {
    fn from(error: CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

impl From<next_contracts::IdentifierError> for NeutralFixtureError {
    fn from(error: next_contracts::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<next_contracts::IdentityContractError> for NeutralFixtureError {
    fn from(error: next_contracts::IdentityContractError) -> Self {
        Self::Identity(error)
    }
}

impl From<next_contracts::PhysicsContractError> for NeutralFixtureError {
    fn from(error: next_contracts::PhysicsContractError) -> Self {
        Self::Physics(error)
    }
}

impl From<AuthorityRegistryError> for NeutralFixtureError {
    fn from(error: AuthorityRegistryError) -> Self {
        Self::Authority(error)
    }
}

impl From<next_project::ProjectCookError> for NeutralFixtureError {
    fn from(error: next_project::ProjectCookError) -> Self {
        Self::ProjectCook(error)
    }
}

impl From<next_assets::ContentStoreError> for NeutralFixtureError {
    fn from(error: next_assets::ContentStoreError) -> Self {
        Self::ProjectStore(error)
    }
}

impl From<next_project::ProjectActivationError> for NeutralFixtureError {
    fn from(error: next_project::ProjectActivationError) -> Self {
        Self::ProjectActivation(error)
    }
}

fn hex_identifier(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}
