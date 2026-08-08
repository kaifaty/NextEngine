use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use next_assets::ContentStore;
use next_contracts::command::WorldCommand;
use next_contracts::ids::{AssetId, ContentHash, PhysicsContactId};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::ContactPhaseV1;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1};
use next_project::{
    ProjectActivationError, ProjectCookError, activate_project_package, cook_project_v2,
};
use next_render::{RenderTargetV1, build_b0_frame_plan};

use crate::scratch::ScratchContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPackageCheckReport {
    pub records: usize,
    pub chunks: usize,
    pub mechanic_packages: usize,
    pub luau_packages: usize,
    pub wasm_plugins: usize,
    pub combat_npc_health: i32,
    pub scripted_player_health: i32,
    pub wasm_player_health: i32,
    pub luau_package_state_hash: ContentHash,
    pub wasm_plugin_state_hash: ContentHash,
    pub wasm_host_api_major: u16,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub mechanics_lock_hash: ContentHash,
    pub world_partition_hash: ContentHash,
    pub composition_lock_hash: ContentHash,
}

pub fn run_content_package_check() -> Result<ContentPackageCheckReport, ContentPackageCheckError> {
    run_content_package_check_in(&std::env::temp_dir())
}

pub fn run_content_package_check_in(
    scratch_root: &Path,
) -> Result<ContentPackageCheckReport, ContentPackageCheckError> {
    let scratch = ScratchContext::new(scratch_root).map_err(ContentPackageCheckError::Cleanup)?;
    run_content_package_check_with_scratch(&scratch)
}

pub(crate) fn run_content_package_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<ContentPackageCheckReport, ContentPackageCheckError> {
    let source = next_reference_game::project_source_v2()?;
    let cooked = cook_project_v2(source)?;
    let directory = scratch
        .create_directory("content-package")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let result = (|| {
        let store = ContentStore::new(directory.path());
        store.publish(&cooked.publication()?)?;
        let package = activate_project_package(&store)?;
        let activated = package.project.clone();
        let prepared = crate::prepare_game_frame_with_activated_project(package)?;
        let gameplay = &prepared.check.play;
        let (scripted_player_health, luau_package_state_hash) =
            run_reference_luau_package(activated.clone())?;
        let (wasm_player_health, wasm_plugin_state_hash, wasm_host_api_major) =
            run_reference_wasm_plugin(activated.clone())?;
        let catalog = &activated.render_content_catalog;
        let fallback_plan = fallback_material_plan(&prepared)?;
        if activated.content_manifest.body.asset_entries.len() != 51
            || activated.text_catalogs.len() != 2
            || activated.audio_clips.len() != 4
            || activated.neutral_skeletons.len() != 1
            || activated.neutral_animations.len() != 1
            || activated.world_partition.body.chunk_bindings.len() != 2
            || activated.rpg_definitions.packages.len() != 2
            || activated.rpg_definitions.abilities.len() != 1
            || catalog.meshes().len() != 10
            || catalog.materials().len() != 11
            || catalog.textures().len() != 7
            || catalog.cooked_meshes().len() != 10
            || catalog
                .meshes()
                .iter()
                .any(|mesh| mesh.asset_id() == AssetId::from_bytes([0x82; 16]))
            || catalog
                .cooked_meshes()
                .iter()
                .any(|mesh| mesh.meshlets().is_empty())
            || prepared.check.rendered_object_count != 6
            || prepared.check.indexed_draw_count != 6
            || prepared.check.fallback_material_draw_count != 0
            || fallback_plan.fallback_material_draw_count != 1
            || gameplay.npc_health != 0
            || scripted_player_health != 50
            || wasm_player_health != 50
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(ContentPackageCheckReport {
            records: activated.content_manifest.body.asset_entries.len(),
            chunks: activated.world_partition.body.chunk_bindings.len(),
            mechanic_packages: activated.rpg_definitions.packages.len(),
            luau_packages: 1,
            wasm_plugins: 1,
            combat_npc_health: gameplay.npc_health,
            scripted_player_health,
            wasm_player_health,
            luau_package_state_hash,
            wasm_plugin_state_hash,
            wasm_host_api_major,
            schema_registry_hash: activated.schema_registry.schema_registry_manifest_sha256,
            content_manifest_hash: activated.content_manifest.content_manifest_sha256,
            mechanics_lock_hash: activated
                .rpg_definitions
                .mechanics_lock
                .mechanics_lock_sha256,
            world_partition_hash: activated.world_partition.world_partition_manifest_sha256,
            composition_lock_hash: activated.project_lock.project_lock_sha256,
        })
    })();
    directory.finish(result, ContentPackageCheckError::Cleanup)
}

fn fallback_material_plan(
    prepared: &crate::PreparedGameFrameV1,
) -> Result<next_render::B0FramePlanV1, ContentPackageCheckError> {
    let snapshot = &prepared.snapshot;
    let records = snapshot
        .scene_records()
        .enumerate()
        .map(|(index, record)| {
            next_contracts::presentation::ScenePresentationRecordV2::new(
                record.presentation_layer,
                record.object_key,
                record.mesh_revision,
                if index == 0 {
                    next_contracts::project::AssetRevisionRefV1 {
                        asset_id: AssetId::from_bytes([0xee; 16]),
                        record_sha256: next_contracts::project::domain_hash(
                            "nextengine.verification.missing-material.v1",
                            b"missing",
                        ),
                    }
                } else {
                    record.material_revision
                },
                record.instance_ordinal,
                record.local_bounds,
                record.feature_flags,
                record.previous_transform,
                record.current_transform,
                record.visible,
            )
        })
        .collect();
    let fallback_snapshot = next_contracts::presentation::PresentationSnapshotV2::new(
        snapshot.snapshot_epoch,
        snapshot.snapshot_sequence,
        snapshot.simulation_tick,
        snapshot.project_composition_lock_hash,
        snapshot.content_manifest_hash,
        snapshot.presentation_profile_hash,
        records,
        8,
        snapshot.environment_batch,
    )
    .map_err(ContentPackageCheckError::Presentation)?;
    build_b0_frame_plan(
        &fallback_snapshot,
        &prepared.render_content_catalog,
        RenderTargetV1 {
            extent: [960, 540],
            target_revision: 1,
        },
    )
    .map_err(crate::PlayCheckError::from)
    .map_err(ContentPackageCheckError::from)
}

fn run_reference_wasm_plugin(
    activated: next_contracts::project::ActivatedProjectV3,
) -> Result<(i32, ContentHash, u16), ContentPackageCheckError> {
    let fixture = crate::build_neutral_player_fixture_from_activated_project(activated)?;
    let snapshot = crate::cooked_project_rpg_snapshot(&fixture);
    let manifest = next_plugin_host::reference_wasm_manifest_v1(true)?;
    let startup = next_plugin_host::WasmPluginRuntimeV1::activate(
        manifest.clone(),
        Some(
            next_plugin_host::REFERENCE_COMPONENT_WAT
                .as_bytes()
                .to_vec(),
        ),
        manifest.requested_capabilities.clone(),
    )?;
    let next_plugin_host::WasmPluginStartupV1::Active(mut plugin) = startup else {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    };
    let outcome = plugin.execute_i32(0, 0)?;
    let (subject_low, subject_high) = if fixture.npc_character_id < fixture.body_id {
        (fixture.npc_character_id, fixture.body_id)
    } else {
        (fixture.body_id, fixture.npc_character_id)
    };
    let fact = RpgPhysicalContactFactV1 {
        gameplay_tick: 0,
        contact_id: PhysicsContactId::from_bytes([0x81; 16]),
        subject_low,
        subject_high,
        physics_checkpoint_revision: 0,
        source_snapshot_hash: ContentHash::from_bytes([0x82; 32]),
        contact_batch_hash: ContentHash::from_bytes([0x83; 32]),
    };
    let compiled = next_plugin_host::compile_reference_wasm_action_v1(
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
    }) {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    }
    let player_health = character_health(&runtime, fixture.body_id)?;

    let optional = next_plugin_host::reference_wasm_manifest_v1(false)?;
    if !matches!(
        next_plugin_host::WasmPluginRuntimeV1::activate(
            optional.clone(),
            None,
            optional.requested_capabilities,
        )?,
        next_plugin_host::WasmPluginStartupV1::OptionalDisabled {
            diagnostic: next_plugin_host::WasmDiagnosticCodeV1::OptionalPluginUnavailable,
            ..
        }
    ) {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    }
    Ok((
        player_health,
        plugin.state().state_hash,
        outcome.selected_host_api_major,
    ))
}

fn run_reference_luau_package(
    activated: next_contracts::project::ActivatedProjectV3,
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
    let player_health = character_health(&runtime, fixture.body_id)?;
    Ok((player_health, script.state().state_hash))
}

fn character_health(
    runtime: &next_runtime::RuntimeState,
    character_id: next_contracts::ids::PersistentId,
) -> Result<i32, ContentPackageCheckError> {
    runtime
        .rpg_snapshot()
        .aggregates
        .iter()
        .find(|aggregate| {
            aggregate.aggregate_kind == RpgAggregateKindV1::Character
                && aggregate.persistent_id == character_id
        })
        .and_then(|aggregate| match &aggregate.payload {
            RpgAggregatePayloadV1::Character(character) => character
                .resources
                .iter()
                .find(|resource| resource.resource_id.as_str() == CORE_CHARACTER_HEALTH_RESOURCE_ID)
                .map(|resource| resource.current_value),
            _ => None,
        })
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ContentPackageCheckError {
    Cook(ProjectCookError),
    Store(next_assets::ContentStoreError),
    Activation(ProjectActivationError),
    Gameplay(crate::PlayCheckError),
    Presentation(next_contracts::presentation::PresentationContractError),
    Fixture(crate::NeutralFixtureError),
    Luau(next_script_luau::LuauHostError),
    Wasm(next_plugin_host::WasmHostError),
    Runtime(next_runtime::RuntimeFatalError),
    Restore(next_runtime::SnapshotRestoreError),
    Canonical(next_contracts::canonical::CanonicalError),
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
            Self::Presentation(error) => {
                write!(formatter, "content-package presentation failed: {error}")
            }
            Self::Fixture(error) => write!(formatter, "content-package fixture failed: {error}"),
            Self::Luau(error) => write!(formatter, "content-package Luau failed: {error}"),
            Self::Wasm(error) => write!(formatter, "content-package Wasm failed: {error}"),
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

impl From<next_plugin_host::WasmHostError> for ContentPackageCheckError {
    fn from(error: next_plugin_host::WasmHostError) -> Self {
        Self::Wasm(error)
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

impl From<next_contracts::canonical::CanonicalError> for ContentPackageCheckError {
    fn from(error: next_contracts::canonical::CanonicalError) -> Self {
        Self::Canonical(error)
    }
}

#[cfg(test)]
mod tests {
    use super::run_content_package_check;

    #[test]
    fn content_package_uses_cooker_publisher_and_production_loader() {
        let report = run_content_package_check().expect("content-package passes");
        assert_eq!(report.records, 51);
        assert_eq!(report.chunks, 2);
        assert_eq!(report.mechanic_packages, 2);
        assert_eq!(report.wasm_plugins, 1);
        assert_eq!(report.combat_npc_health, 0);
        assert_eq!(report.scripted_player_health, 50);
        assert_eq!(report.wasm_player_health, 50);
    }
}
