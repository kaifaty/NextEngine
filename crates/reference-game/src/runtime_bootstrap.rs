use next_contracts::canonical::sha256;
use next_contracts::cognition::{
    AGENT_COGNITION_CAPABILITY_ID, AGENT_COGNITION_CAPABILITY_SUBJECT_ID, AGENT_COGNITION_SYSTEM_ID,
};
use next_contracts::command::IssuerPrincipal;
use next_contracts::identity::{
    CommandStreamRegistryV1, PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus,
    RuntimeDeterminismBundleV1, WorldIdentityManifestV1,
};
use next_contracts::ids::{
    CapabilityId, CommandStreamId, ProjectId, SchemaId, SystemId, content_hash_from_bytes,
};
use next_contracts::world_activity::{
    WORLD_ACTIVITY_CAPABILITY_ID, WORLD_ACTIVITY_CAPABILITY_SUBJECT_ID, WORLD_ACTIVITY_SYSTEM_ID,
};
use next_contracts::world_population::{
    WORLD_POPULATION_CAPABILITY_ID, WORLD_POPULATION_CAPABILITY_SUBJECT_ID,
    WORLD_POPULATION_SYSTEM_ID,
};
use next_contracts::world_routine::{
    WORLD_ROUTINE_CAPABILITY_ID, WORLD_ROUTINE_CAPABILITY_SUBJECT_ID, WORLD_ROUTINE_SYSTEM_ID,
};
use next_runtime::{AuthorityRegistry, RuntimeBootstrapV4};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ReferenceRuntimeBootstrap {
    pub bootstrap: RuntimeBootstrapV4,
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
    let profile = RuntimeDeterminismBundleV1::core_r5c()?.runtime_profile();
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
        let routine_principal =
            IssuerPrincipal::InternalSystem(SystemId::new(WORLD_ROUTINE_SYSTEM_ID)?);
        let population_principal =
            IssuerPrincipal::InternalSystem(SystemId::new(WORLD_POPULATION_SYSTEM_ID)?);
        let cognition_principal =
            IssuerPrincipal::InternalSystem(SystemId::new(AGENT_COGNITION_SYSTEM_ID)?);
        let activity_principal =
            IssuerPrincipal::InternalSystem(SystemId::new(WORLD_ACTIVITY_SYSTEM_ID)?);
        let (provenance_hash, capability_subject_id) = if principal == routine_principal {
            if capabilities.as_slice() != [CapabilityId::new(WORLD_ROUTINE_CAPABILITY_ID)?] {
                return Err(crate::ReferenceGameError::DuplicatePrincipal);
            }
            (
                content_hash_from_bytes(sha256(
                    b"nextengine.principal.world-routine-boundary.v1\0",
                )),
                SchemaId::new(WORLD_ROUTINE_CAPABILITY_SUBJECT_ID)?,
            )
        } else if principal == population_principal {
            if capabilities.as_slice() != [CapabilityId::new(WORLD_POPULATION_CAPABILITY_ID)?] {
                return Err(crate::ReferenceGameError::DuplicatePrincipal);
            }
            (
                content_hash_from_bytes(sha256(
                    b"nextengine.principal.world-population-boundary.v1\0",
                )),
                SchemaId::new(WORLD_POPULATION_CAPABILITY_SUBJECT_ID)?,
            )
        } else if principal == activity_principal {
            if capabilities.as_slice() != [CapabilityId::new(WORLD_ACTIVITY_CAPABILITY_ID)?] {
                return Err(crate::ReferenceGameError::DuplicatePrincipal);
            }
            (
                content_hash_from_bytes(sha256(
                    b"nextengine.principal.world-activity-boundary.v1\0",
                )),
                SchemaId::new(WORLD_ACTIVITY_CAPABILITY_SUBJECT_ID)?,
            )
        } else if principal == cognition_principal {
            let cognition_capability = CapabilityId::new(AGENT_COGNITION_CAPABILITY_ID)?;
            let rpg_capability = CapabilityId::new(next_contracts::rpg::RPG_COMMAND_CAPABILITY_ID)?;
            let mut expected = vec![cognition_capability];
            if capabilities.contains(&rpg_capability) {
                expected.push(rpg_capability);
                expected.sort();
            }
            if capabilities != expected {
                return Err(crate::ReferenceGameError::DuplicatePrincipal);
            }
            (
                content_hash_from_bytes(sha256(
                    b"nextengine.principal.agent-cognition-boundary.v2\0",
                )),
                SchemaId::new(AGENT_COGNITION_CAPABILITY_SUBJECT_ID)?,
            )
        } else {
            let principal_bytes = principal.canonical_bytes()?;
            let mut provenance = Vec::new();
            provenance.extend_from_slice(world_identity.world_namespace.as_bytes());
            provenance.extend_from_slice(&(principal_bytes.len() as u64).to_le_bytes());
            provenance.extend_from_slice(&principal_bytes);
            (
                content_hash_from_bytes(sha256(&provenance)),
                SchemaId::new(format!(
                    "fixture.principal.{}.{}",
                    principal.tag(),
                    hex_identifier(principal.identifier_bytes())
                ))?,
            )
        };
        principal_registry.register(
            principal.clone(),
            PrincipalRecordV1 {
                provenance_hash,
                capability_subject_id,
                status: PrincipalStatus::Active,
            },
        )?;
        let stream_id = stream_registry.allocate_stream(principal.clone())?;
        authority.register(principal.clone(), capabilities)?;
        streams.insert(principal, stream_id);
    }
    Ok(ReferenceRuntimeBootstrap {
        bootstrap: RuntimeBootstrapV4::new(
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
