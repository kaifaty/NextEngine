pub use next_contracts::identity::{
    CommandKindRegistryEntryV1 as CommandKindDescriptor,
    CommandKindRegistryV1 as CommandKindRegistry, NOOP_PRIORITY_CLASS, PHYSICAL_PRIORITY_CLASS,
    RPG_PRIORITY_CLASS,
};
use next_contracts::ids::ContentHash;

#[must_use]
pub fn core_command_kind_registry() -> CommandKindRegistry {
    CommandKindRegistry::core_r4c().expect("the engine-owned command registry is canonical")
}

#[must_use]
pub fn command_kind_registry_hash(registry: &CommandKindRegistry) -> ContentHash {
    registry
        .canonical_hash()
        .expect("the engine-owned command registry is canonical")
}
