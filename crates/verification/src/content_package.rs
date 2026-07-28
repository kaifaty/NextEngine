use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::{
    CORE_CHARACTER_HEALTH_RESOURCE_ID, ContactPhaseV1, ContentHash, PhysicsContactId,
    RpgAggregateKindV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1, WorldCommand,
};
use next_project::{
    ProjectActivationError, ProjectCookError, activate_project, cook_project_v1,
    neutral_vertical_slice_source_v1,
};

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPackageCheckReport {
    pub records: usize,
    pub chunks: usize,
    pub mechanic_packages: usize,
    pub luau_packages: usize,
    pub combat_npc_health: i32,
    pub scripted_player_health: i32,
    pub luau_package_state_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub world_partition_hash: ContentHash,
    pub composition_lock_hash: ContentHash,
}

pub fn run_content_package_check() -> Result<ContentPackageCheckReport, ContentPackageCheckError> {
    let source = neutral_vertical_slice_source_v1()?;
    let cooked = cook_project_v1(source)?;
    let counter = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let output = std::env::temp_dir().join(format!(
        "nextengine-content-package-{}-{counter}",
        std::process::id()
    ));
    let result = (|| {
        let store = ContentStore::new(&output);
        store.publish(&cooked.publication()?)?;
        let activated = activate_project(&store)?;
        let gameplay = crate::run_play_check_with_activated_project(activated.clone())?;
        let (scripted_player_health, luau_package_state_hash) =
            run_reference_luau_package(activated.clone())?;
        if activated.content_manifest.body.asset_entries.len() != 13
            || activated.world_partition.body.chunk_bindings.len() != 2
            || activated.rpg_definitions.packages.len() != 2
            || activated.rpg_definitions.abilities.len() != 1
            || gameplay.npc_health != 75
            || scripted_player_health != 75
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(ContentPackageCheckReport {
            records: activated.content_manifest.body.asset_entries.len(),
            chunks: activated.world_partition.body.chunk_bindings.len(),
            mechanic_packages: activated.rpg_definitions.packages.len(),
            luau_packages: 1,
            combat_npc_health: gameplay.npc_health,
            scripted_player_health,
            luau_package_state_hash,
            schema_registry_hash: activated.schema_registry.schema_registry_manifest_sha256,
            content_manifest_hash: activated.content_manifest.content_manifest_sha256,
            world_partition_hash: activated.world_partition.world_partition_manifest_sha256,
            composition_lock_hash: activated.composition_lock.composition_lock_sha256,
        })
    })();
    if output.exists() {
        std::fs::remove_dir_all(&output).map_err(ContentPackageCheckError::Cleanup)?;
    }
    result
}

fn run_reference_luau_package(
    activated: next_contracts::ActivatedProjectV1,
) -> Result<(i32, ContentHash), ContentPackageCheckError> {
    let fixture = crate::build_neutral_player_fixture_from_activated_project(activated)?;
    let snapshot = crate::cooked_project_rpg_snapshot(&fixture);
    let manifest = next_script_luau::reference_scripted_melee_manifest_v1()?;
    let mut script = next_script_luau::LuauPackageRuntimeV1::new(
        manifest.clone(),
        next_script_luau::REFERENCE_SCRIPTED_MELEE_SOURCE
            .as_bytes()
            .to_vec(),
    )?;
    let outcome = script.execute(next_script_luau::LuauCallbackInputV1 {
        gameplay_tick: 0,
        target_health: 100,
        granted_capabilities: manifest.requested_capabilities.clone(),
    })?;
    let (subject_low, subject_high) = if fixture.npc_character_id < fixture.body_id {
        (fixture.npc_character_id, fixture.body_id)
    } else {
        (fixture.body_id, fixture.npc_character_id)
    };
    let fact = RpgPhysicalContactFactV1 {
        gameplay_tick: 0,
        contact_id: PhysicsContactId::from_bytes([0x71; 16]),
        subject_low,
        subject_high,
        physics_checkpoint_revision: 0,
        source_snapshot_hash: ContentHash::from_bytes([0x72; 32]),
        contact_batch_hash: ContentHash::from_bytes([0x73; 32]),
    };
    let compiled = next_script_luau::compile_first_scripted_action_v1(
        &outcome,
        &fixture.activated_project.rpg_definitions,
        &snapshot,
        0,
        fixture.npc_character_id,
        vec![fact],
    )?;
    let command = WorldCommand::rpg(
        fixture.agent_stream_id,
        fixture.agent_principal.clone(),
        0,
        0,
        compiled.rpg_command,
    )?;
    let mut runtime = next_runtime::RuntimeState::with_rpg_snapshot(
        fixture.bootstrap,
        fixture.authority,
        snapshot,
    )?;
    let report = runtime.run_tick([command])?;
    if !report.results.iter().any(|result| {
        matches!(
            result.disposition,
            next_runtime::CommandDisposition::Committed
        )
    }) || report
        .contact_batch
        .events
        .iter()
        .any(|event| event.phase == ContactPhaseV1::End)
    {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    }
    let player_health = runtime
        .rpg_snapshot()
        .aggregates
        .iter()
        .find(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::Character
                && aggregate.persistent_id == fixture.body_id
        })
        .and_then(|aggregate| match &aggregate.payload {
            RpgAggregatePayloadV1::Character(character) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .map(|resource| resource.current_value),
            _ => None,
        })
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    Ok((player_health, script.state().state_hash))
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ContentPackageCheckError {
    Cook(ProjectCookError),
    Store(next_assets::ContentStoreError),
    Activation(ProjectActivationError),
    Gameplay(crate::PlayCheckError),
    Fixture(crate::NeutralFixtureError),
    Luau(next_script_luau::LuauHostError),
    Runtime(next_runtime::RuntimeFatalError),
    Restore(next_runtime::SnapshotRestoreError),
    Canonical(next_contracts::CanonicalError),
    Cleanup(std::io::Error),
    FixtureClosureMismatch,
}

impl Display for ContentPackageCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cook(error) => write!(formatter, "content-package cook failed: {error}"),
            Self::Store(error) => write!(formatter, "content-package publish failed: {error}"),
            Self::Activation(error) => {
                write!(formatter, "content-package activation failed: {error}")
            }
            Self::Gameplay(error) => write!(formatter, "content-package gameplay failed: {error}"),
            Self::Fixture(error) => write!(formatter, "content-package fixture failed: {error}"),
            Self::Luau(error) => write!(formatter, "content-package Luau failed: {error}"),
            Self::Runtime(error) => write!(formatter, "content-package runtime failed: {error}"),
            Self::Restore(error) => write!(formatter, "content-package restore failed: {error}"),
            Self::Canonical(error) => write!(formatter, "content-package command failed: {error}"),
            Self::Cleanup(error) => write!(formatter, "content-package cleanup failed: {error}"),
            Self::FixtureClosureMismatch => {
                formatter.write_str("content-package fixture closure mismatch")
            }
        }
    }
}

impl Error for ContentPackageCheckError {}

impl From<ProjectCookError> for ContentPackageCheckError {
    fn from(error: ProjectCookError) -> Self {
        Self::Cook(error)
    }
}

impl From<next_assets::ContentStoreError> for ContentPackageCheckError {
    fn from(error: next_assets::ContentStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ProjectActivationError> for ContentPackageCheckError {
    fn from(error: ProjectActivationError) -> Self {
        Self::Activation(error)
    }
}

impl From<crate::PlayCheckError> for ContentPackageCheckError {
    fn from(error: crate::PlayCheckError) -> Self {
        Self::Gameplay(error)
    }
}

impl From<crate::NeutralFixtureError> for ContentPackageCheckError {
    fn from(error: crate::NeutralFixtureError) -> Self {
        Self::Fixture(error)
    }
}

impl From<next_script_luau::LuauHostError> for ContentPackageCheckError {
    fn from(error: next_script_luau::LuauHostError) -> Self {
        Self::Luau(error)
    }
}

impl From<next_runtime::RuntimeFatalError> for ContentPackageCheckError {
    fn from(error: next_runtime::RuntimeFatalError) -> Self {
        Self::Runtime(error)
    }
}

impl From<next_runtime::SnapshotRestoreError> for ContentPackageCheckError {
    fn from(error: next_runtime::SnapshotRestoreError) -> Self {
        Self::Restore(error)
    }
}

impl From<next_contracts::CanonicalError> for ContentPackageCheckError {
    fn from(error: next_contracts::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

#[cfg(test)]
mod tests {
    use super::run_content_package_check;

    #[test]
    fn content_package_uses_cooker_publisher_and_production_loader() {
        let report = run_content_package_check().expect("content-package passes");
        assert_eq!(report.records, 13);
        assert_eq!(report.chunks, 2);
        assert_eq!(report.mechanic_packages, 2);
        assert_eq!(report.combat_npc_health, 75);
        assert_eq!(report.scripted_player_health, 75);
    }
}
