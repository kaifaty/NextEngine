mod codec;
mod derivation;
mod error;
mod principal_registry;
mod runtime_profile;
mod stream_registry;
mod world;

pub use derivation::{
    derive_command_stream_id, derive_player_principal_id, derive_world_namespace,
    runtime_profile_hash,
};
pub use error::IdentityContractError;
pub use principal_registry::{
    PRINCIPAL_REGISTRY_SCHEMA_VERSION, PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus,
};
pub use runtime_profile::{
    RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION, RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS,
    RuntimeDeterminismProfileV1,
};
pub use stream_registry::{
    COMMAND_STREAM_REGISTRY_SCHEMA_VERSION, CommandStreamKeyV1, CommandStreamRegistryV1,
};
pub use world::{WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION, WorldIdentityManifestV1};

#[cfg(test)]
mod tests;
