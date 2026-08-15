mod codec;
mod command_kind;
mod derivation;
mod determinism_bundle;
mod error;
mod principal_registry;
mod runtime_profile;
mod schedule;
mod stream_registry;
mod world;

pub use command_kind::{
    COMMAND_KIND_REGISTRY_OWNER_ID, COMMAND_KIND_REGISTRY_SCHEMA_ID,
    COMMAND_KIND_REGISTRY_SCHEMA_VERSION, COMMAND_KIND_REGISTRY_SEGMENT_ID,
    CommandKindRegistryEntryV1, CommandKindRegistryKeyV1, CommandKindRegistryV1,
    NOOP_COMMAND_KIND_ID, NOOP_PRIORITY_CLASS, PHYSICAL_COMMAND_KIND_ID, PHYSICAL_PRIORITY_CLASS,
    RPG_COMMAND_KIND_ID, RPG_PRIORITY_CLASS,
};
pub use derivation::{
    derive_command_stream_id, derive_player_principal_id, derive_world_namespace,
    runtime_profile_hash,
};
pub use determinism_bundle::RuntimeDeterminismBundleV1;
pub use error::IdentityContractError;
pub use principal_registry::{
    PRINCIPAL_REGISTRY_SCHEMA_VERSION, PrincipalRecordV1, PrincipalRegistryV1, PrincipalStatus,
};
pub use runtime_profile::{
    RUNTIME_DETERMINISM_PROFILE_SCHEMA_VERSION, RUNTIME_MAXIMUM_FUTURE_COMMAND_TICKS,
    RuntimeDeterminismProfileV1,
};
pub use schedule::{
    AccessKeyV1, AccessSetV1, CommandAdmissionBarrierV1, CommandBarrierSourceV1, DeltaMergeOrderV1,
    LogicalShardPlanV1, QueryOrderV1, ReducerDescriptorV1, ReducerKindV1, RuntimeStageId,
    SCHEDULE_MANIFEST_OWNER_ID, SCHEDULE_MANIFEST_SCHEMA_ID, SCHEDULE_MANIFEST_SCHEMA_VERSION,
    SCHEDULE_MANIFEST_SEGMENT_ID, ScheduleManifestV1, ShardPartitionRuleV1, ShardRecordOrderV1,
    SystemDescriptorV1,
};
pub use stream_registry::{
    COMMAND_STREAM_REGISTRY_SCHEMA_VERSION, CommandStreamKeyV1, CommandStreamRegistryV1,
};
pub use world::{WORLD_IDENTITY_MANIFEST_SCHEMA_VERSION, WorldIdentityManifestV1};

#[cfg(test)]
mod tests;
