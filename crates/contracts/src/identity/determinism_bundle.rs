use crate::canonical::CanonicalDecodeLimits;
use crate::ids::ContentHash;

use super::{
    CommandKindRegistryV1, IdentityContractError, RuntimeDeterminismProfileV1, ScheduleManifestV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDeterminismBundleV1 {
    command_kind_registry: CommandKindRegistryV1,
    command_kind_registry_bytes: Vec<u8>,
    command_kind_registry_hash: ContentHash,
    schedule_manifest: ScheduleManifestV1,
    schedule_manifest_bytes: Vec<u8>,
    schedule_manifest_hash: ContentHash,
    runtime_profile: RuntimeDeterminismProfileV1,
    runtime_profile_bytes: Vec<u8>,
    runtime_profile_hash: ContentHash,
}

impl RuntimeDeterminismBundleV1 {
    pub fn core_r4b() -> Result<Self, IdentityContractError> {
        let command_kind_registry = CommandKindRegistryV1::core_r4b()?;
        let schedule_manifest = ScheduleManifestV1::core_r4b()?;
        Self::materialize(command_kind_registry, schedule_manifest)
    }

    pub fn core_r4c() -> Result<Self, IdentityContractError> {
        let command_kind_registry = CommandKindRegistryV1::core_r4c()?;
        let schedule_manifest = ScheduleManifestV1::core_r4c()?;
        Self::materialize(command_kind_registry, schedule_manifest)
    }

    pub fn core_r4d() -> Result<Self, IdentityContractError> {
        let command_kind_registry = CommandKindRegistryV1::core_r4d()?;
        let schedule_manifest = ScheduleManifestV1::core_r4d()?;
        Self::materialize(command_kind_registry, schedule_manifest)
    }

    fn materialize(
        command_kind_registry: CommandKindRegistryV1,
        schedule_manifest: ScheduleManifestV1,
    ) -> Result<Self, IdentityContractError> {
        let command_kind_registry_bytes = command_kind_registry.canonical_bytes()?;
        let command_kind_registry_hash = command_kind_registry.canonical_hash()?;
        let schedule_manifest_bytes = schedule_manifest.canonical_bytes()?;
        let schedule_manifest_hash = schedule_manifest.profile_hash()?;
        let runtime_profile = RuntimeDeterminismProfileV1::from_materialized_r4b(
            command_kind_registry_hash,
            schedule_manifest_hash,
        );
        runtime_profile.validate()?;
        let runtime_profile_bytes = runtime_profile.canonical_bytes()?;
        let runtime_profile_hash = runtime_profile.profile_hash()?;

        let limits = CanonicalDecodeLimits::default();
        if CommandKindRegistryV1::from_canonical_bytes(&command_kind_registry_bytes, limits)?
            != command_kind_registry
            || ScheduleManifestV1::from_canonical_bytes(&schedule_manifest_bytes, limits)?
                != schedule_manifest
            || RuntimeDeterminismProfileV1::from_canonical_bytes(&runtime_profile_bytes, limits)?
                != runtime_profile
        {
            return Err(IdentityContractError::NonCanonicalEncoding);
        }

        Ok(Self {
            command_kind_registry,
            command_kind_registry_bytes,
            command_kind_registry_hash,
            schedule_manifest,
            schedule_manifest_bytes,
            schedule_manifest_hash,
            runtime_profile,
            runtime_profile_bytes,
            runtime_profile_hash,
        })
    }

    #[must_use]
    pub fn command_kind_registry(&self) -> &CommandKindRegistryV1 {
        &self.command_kind_registry
    }

    #[must_use]
    pub fn command_kind_registry_bytes(&self) -> &[u8] {
        &self.command_kind_registry_bytes
    }

    #[must_use]
    pub const fn command_kind_registry_hash(&self) -> ContentHash {
        self.command_kind_registry_hash
    }

    #[must_use]
    pub fn schedule_manifest(&self) -> &ScheduleManifestV1 {
        &self.schedule_manifest
    }

    #[must_use]
    pub fn schedule_manifest_bytes(&self) -> &[u8] {
        &self.schedule_manifest_bytes
    }

    #[must_use]
    pub const fn schedule_manifest_hash(&self) -> ContentHash {
        self.schedule_manifest_hash
    }

    #[must_use]
    pub const fn runtime_profile(&self) -> RuntimeDeterminismProfileV1 {
        self.runtime_profile
    }

    #[must_use]
    pub fn runtime_profile_bytes(&self) -> &[u8] {
        &self.runtime_profile_bytes
    }

    #[must_use]
    pub const fn runtime_profile_hash(&self) -> ContentHash {
        self.runtime_profile_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r4c_builder_closes_registry_schedule_and_runtime_profile() {
        let bundle = RuntimeDeterminismBundleV1::core_r4c().expect("bundle builds");
        assert_eq!(bundle.command_kind_registry().entries.len(), 6);
        assert_eq!(bundle.schedule_manifest().stage_order.len(), 12);
        assert_eq!(bundle.schedule_manifest().systems.len(), 3);
        assert_eq!(
            bundle.runtime_profile().command_kind_registry_hash,
            bundle.command_kind_registry_hash()
        );
        assert_eq!(
            bundle.runtime_profile().schedule_manifest_hash,
            bundle.schedule_manifest_hash()
        );
        assert_eq!(
            bundle.runtime_profile().profile_hash(),
            Ok(bundle.runtime_profile_hash())
        );
    }
}
