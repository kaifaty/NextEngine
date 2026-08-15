use next_contracts::canonical::sha256;
use next_contracts::command::IssuerPrincipal;
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus,
    RuntimeDeterminismBundleV1, WorldIdentityManifestV1,
};
use next_contracts::ids::{
    CapabilityId, CommandStreamId, ProjectId, SchemaId, content_hash_from_bytes,
};
use next_runtime::{AuthorityRegistry, RuntimeBootstrapV3};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ReferenceRuntimeBootstrap {
    pub bootstrap: RuntimeBootstrapV3,
    pub authority: AuthorityRegistry,
    pub streams: BTreeMap<IssuerPrincipal, CommandStreamId>,
}

impl ReferenceRuntimeBootstrap {
    #[must_use]
    pub fn stream_for(&self, principal: &IssuerPrincipal) -> Option<CommandStreamId> {
        self.streams.get(principal).copied()
    }
}

pub fn build_reference_runtime_bootstrap(
    project_id: &str,
    grants: impl IntoIterator<Item = (IssuerPrincipal, Vec<CapabilityId>)>,
) -> Result<ReferenceRuntimeBootstrap, crate::ReferenceGameError> {
    let profile = RuntimeDeterminismBundleV1::core_r4a()?.runtime_profile();
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
        return Err(crate::ReferenceGameError::DuplicatePrincipal);
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
    Ok(ReferenceRuntimeBootstrap {
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

fn hex_identifier(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(DIGITS[usize::from(byte >> 4)]));
        value.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    value
}
