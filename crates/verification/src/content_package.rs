use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::Path;

use next_assets::{ContentPublicationV1, ContentStore, PublicationFileV1};
use next_contracts::command::WorldCommand;
use next_contracts::ids::{AssetId, ContentHash, PersistentId, PhysicsContactId};
use next_contracts::mechanics::CORE_CHARACTER_HEALTH_RESOURCE_ID;
use next_contracts::physics::ContactPhaseV1;
use next_contracts::project::ProjectLockV3;
use next_contracts::rpg::{RpgAggregateKindV1, RpgAggregatePayloadV1, RpgPhysicalContactFactV1};
use next_project::{
    ProjectActivationError, ProjectAuthoringError, ProjectCookError, activate_project,
    activate_project_package, cook_project_v7, load_project_authoring_v7,
};
use next_render::{RenderTargetV1, build_b0_frame_plan};

use crate::scratch::ScratchContext;

mod creator_scenario;
mod creator_template;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPackageCheckReport {
    pub records: usize,
    pub chunks: usize,
    pub creator_records: usize,
    pub creator_chunks: usize,
    pub creator_composition_lock_hash: ContentHash,
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
    let creator = verify_creator_project(scratch)?;
    let source = next_reference_game::project_source_v7()?;
    if source.root_asset_ids.len() != 42 {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    }
    verify_world_routine_source_faults()?;
    let cooked = cook_project_v7(source)?;
    verify_world_routine_publication_faults(scratch, &cooked)?;
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
        let sampled_skinning_plan = build_b0_frame_plan(
            &prepared.snapshot,
            &prepared.render_content_catalog,
            RenderTargetV1 {
                extent: [960, 540],
                target_revision: 1,
            },
        )
        .map_err(crate::PlayCheckError::from)?;
        let bind_pose_plan = bind_pose_fallback_plan(&prepared)?;
        let fallback_plan = fallback_material_plan(&prepared)?;
        let skinning_records = prepared
            .snapshot
            .character_skinning_records()
            .collect::<Vec<_>>();
        let skinning_profile = catalog
            .base_skinning_profiles()
            .first()
            .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
        let skinning_profile_revision = skinning_profile
            .asset_revision()
            .map_err(|_| ContentPackageCheckError::FixtureClosureMismatch)?;
        let humanoid_mesh = catalog
            .mesh(skinning_profile.mesh_revision())
            .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
        if activated.content_manifest.body.asset_entries.len() != 128
            || activated.text_catalogs.len() != 2
            || activated.audio_clips.len() != 4
            || activated.neutral_skeletons.len() != 1
            || activated.neutral_animations.len() != 2
            || activated.world_partition.body.root_region_ids.len() != 4
            || activated.world_partition.body.chunk_bindings.len() != 64
            || activated.rpg_definitions.packages.len() != 2
            || activated.rpg_definitions.abilities.len() != 1
            || activated.world_routine_catalog_or_none.is_none()
            || activated
                .rpg_definitions
                .interactions
                .first()
                .and_then(|interaction| interaction.availability_condition_or_none)
                .is_none()
            || catalog.meshes().len() != 16
            || catalog.materials().len() != 12
            || catalog.textures().len() != 7
            || catalog.base_skinning_profiles().len() != 1
            || skinning_profile.pose_correctives().len() != 3
            || skinning_profile
                .pose_correctives()
                .iter()
                .filter(|corrective| {
                    corrective.lod_class()
                        == next_contracts::render_content::PoseCorrectiveLodClassV1::Essential
                })
                .count()
                != 1
            || skinning_profile
                .pose_correctives()
                .iter()
                .filter(|corrective| {
                    corrective.lod_class()
                        == next_contracts::render_content::PoseCorrectiveLodClassV1::Detail
                })
                .count()
                != 2
            || catalog.cooked_meshes().len() != 16
            || catalog
                .meshes()
                .iter()
                .any(|mesh| mesh.asset_id() == AssetId::from_bytes([0x82; 16]))
            || catalog
                .cooked_meshes()
                .iter()
                .any(|mesh| mesh.meshlets().is_empty())
            || prepared.check.rendered_object_count != 13
            || prepared.check.indexed_draw_count != 13
            || prepared.check.fallback_material_draw_count != 0
            || fallback_plan.fallback_material_draw_count != 1
            || skinning_records.len() != 2
            || skinning_records.iter().any(|record| {
                record.skinning_profile_revision != skinning_profile_revision
                    || record.deformation_lod
                        != next_contracts::presentation::CharacterDeformationLodV1::FullCorrectives
            })
            || !skinning_records.iter().any(|record| {
                record.object_key.presentation_role
                    == next_contracts::presentation::PresentationRoleV1::PlayerAvatar
            })
            || !skinning_records.iter().any(|record| {
                record.object_key.presentation_role
                    == next_contracts::presentation::PresentationRoleV1::Character
            })
            || sampled_skinning_plan.skinned_vertex_streams.len() != 2
            || sampled_skinning_plan
                .skinned_vertex_streams
                .iter()
                .any(|stream| {
                    stream.used_bind_pose_fallback || stream.used_pose_corrective_fallback
                })
            || sampled_skinning_plan
                .skinned_vertex_streams
                .iter()
                .all(|stream| stream.applied_pose_corrective_count == 0)
            || sampled_skinning_plan
                .skinned_vertex_streams
                .iter()
                .all(|stream| {
                    stream.positions_micrometres.as_slice() == humanoid_mesh.positions_micrometres()
                })
            || bind_pose_plan.skinned_vertex_streams.len() != 2
            || bind_pose_plan.skinned_vertex_streams.iter().any(|stream| {
                !stream.used_bind_pose_fallback
                    || stream.used_pose_corrective_fallback
                    || stream.applied_pose_corrective_count != 0
                    || stream.positions_micrometres.as_slice()
                        != humanoid_mesh.positions_micrometres()
            })
            || gameplay.npc_health != 0
            || scripted_player_health != 50
            || wasm_player_health != 50
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(ContentPackageCheckReport {
            records: activated.content_manifest.body.asset_entries.len(),
            chunks: activated.world_partition.body.chunk_bindings.len(),
            creator_records: creator.records,
            creator_chunks: creator.chunks,
            creator_composition_lock_hash: creator.composition_lock_hash,
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

fn verify_creator_public_run_and_package(
    scratch: &ScratchContext,
    project_directory: &Path,
    expected_project_lock: ContentHash,
) -> Result<(), ContentPackageCheckError> {
    let directory = scratch
        .create_directory("content-package-creator-public")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let result = (|| {
        let authoring_run = next_cli::execute([
            OsString::from("project"),
            OsString::from("run"),
            OsString::from("--project"),
            project_directory.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Run(next_cli::CreatorRunCommandReportV1::Pass(
            authoring_run,
        )) = authoring_run
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let package_root = directory.path().join("creator-package");
        let package = next_cli::execute([
            OsString::from("project"),
            OsString::from("package"),
            OsString::from("--project"),
            project_directory.as_os_str().to_owned(),
            OsString::from("--output"),
            package_root.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Package(next_cli::CreatorPackageCommandReportV1::Pass(
            package,
        )) = package
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let packaged_run = next_cli::execute([
            OsString::from("project"),
            OsString::from("run"),
            OsString::from("--package"),
            package_root.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Run(next_cli::CreatorRunCommandReportV1::Pass(
            packaged_run,
        )) = packaged_run
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let authoring_inspection = next_cli::execute([
            OsString::from("project"),
            OsString::from("inspect"),
            OsString::from("--project"),
            project_directory.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Inspect(next_cli::CreatorInspectCommandReportV1::Pass(
            authoring_inspection,
        )) = authoring_inspection
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let package_inspection = next_cli::execute([
            OsString::from("project"),
            OsString::from("inspect"),
            OsString::from("--package"),
            package_root.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Inspect(next_cli::CreatorInspectCommandReportV1::Pass(
            package_inspection,
        )) = package_inspection
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let authoring_package_diff = next_cli::execute([
            OsString::from("project"),
            OsString::from("diff"),
            OsString::from("--base-project"),
            project_directory.as_os_str().to_owned(),
            OsString::from("--candidate-package"),
            package_root.as_os_str().to_owned(),
        ]);
        let next_cli::CreatorCliReportV1::Diff(next_cli::CreatorDiffCommandReportV1::Pass(
            authoring_package_diff,
        )) = authoring_package_diff
        else {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        };
        let expected_lock = expected_project_lock.to_hex();
        if authoring_run.details.project.project_lock_sha256 != expected_lock
            || authoring_run.details.runtime.ticks != 1
            || authoring_run.details.runtime != package.details.runtime
            || package.details.runtime != packaged_run.details.runtime
            || package.details.project != packaged_run.details.project
            || package.details.required_notices != ["NOTICE"]
            || authoring_inspection.details.project != package_inspection.details.project
            || authoring_inspection.details.projection != package_inspection.details.projection
            || authoring_package_diff.details.different
            || authoring_package_diff.details.summary.root_changes != 0
            || authoring_package_diff.details.summary.asset_changes != 0
            || authoring_package_diff.details.summary.world_chunk_changes != 0
            || authoring_package_diff
                .details
                .summary
                .mechanic_package_changes
                != 0
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(())
    })();
    directory.finish(result, ContentPackageCheckError::Cleanup)
}

struct CreatorProjectEvidence {
    records: usize,
    chunks: usize,
    composition_lock_hash: ContentHash,
}

fn verify_creator_project(
    scratch: &ScratchContext,
) -> Result<CreatorProjectEvidence, ContentPackageCheckError> {
    creator_scenario::verify(scratch)?;
    creator_template::verify(scratch)?;
    let project_directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../projects/creator-smoke");
    let source = load_project_authoring_v7(&project_directory)?;
    if source.project_id.as_str() != "org.nextengine.creator-smoke"
        || source.project_revision != 1
        || source.records.len() != 10
        || source.chunks.len() != 3
    {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    }
    let cooked = cook_project_v7(source)?;
    verify_creator_public_run_and_package(
        scratch,
        &project_directory,
        cooked.project_lock.project_lock_sha256,
    )?;
    let directory = scratch
        .create_directory("content-package-creator")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let result = (|| {
        let store = ContentStore::new(directory.path());
        store.publish(&cooked.publication()?)?;
        let activated = activate_project(&store)?;
        if activated.project_lock.project_id.as_str() != "org.nextengine.creator-smoke"
            || activated.project_lock.project_revision != 1
            || activated.neutral_records.len() != 10
            || activated.content_manifest.body.asset_entries.len() != 18
            || activated.content_manifest.body.root_assets.len() != 16
            || activated.world_partition.body.chunk_bindings.len() != 3
            || activated.rpg_definitions.abilities.len() != 1
            || activated.rpg_definitions.interactions.len() != 1
            || activated.world_routine_catalog_or_none.is_some()
            || !activated.render_content_catalog.meshes().is_empty()
            || activated.render_content_catalog.materials().len() != 1
            || activated.render_content_catalog.textures().len() != 1
        {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(CreatorProjectEvidence {
            records: activated.content_manifest.body.asset_entries.len(),
            chunks: activated.world_partition.body.chunk_bindings.len(),
            composition_lock_hash: activated.project_lock.project_lock_sha256,
        })
    })();
    directory.finish(result, ContentPackageCheckError::Cleanup)
}

fn verify_world_routine_source_faults() -> Result<(), ContentPackageCheckError> {
    use next_contracts::world_routine::WorldRoutineActivityV1;

    let invalid = |source| matches!(cook_project_v7(source), Err(ProjectCookError::InvalidValue));

    let mut zero_ratio = next_reference_game::project_source_v7()?;
    zero_ratio
        .world_routine_catalog_or_none
        .as_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?
        .profile
        .world_ticks_per_simulation_tick_num = 0;

    let mut overflow = next_reference_game::project_source_v7()?;
    let overflow_catalog = overflow
        .world_routine_catalog_or_none
        .as_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    overflow_catalog.profile.anchor_simulation_tick = u64::MAX;
    overflow_catalog.profile.anchor_world_tick = 0;
    overflow_catalog.profile.world_ticks_per_simulation_tick_num = 1;
    overflow_catalog.profile.world_ticks_per_simulation_tick_den = 1;
    overflow_catalog.routine.transition_world_tick = 1;

    let mut invalid_boundary = next_reference_game::project_source_v7()?;
    let boundary_catalog = invalid_boundary
        .world_routine_catalog_or_none
        .as_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?;
    boundary_catalog.routine.transition_world_tick = boundary_catalog.profile.anchor_world_tick;

    let mut invalid_activity = next_reference_game::project_source_v7()?;
    invalid_activity
        .world_routine_catalog_or_none
        .as_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?
        .routine
        .next_activity = WorldRoutineActivityV1::Duty;

    let mut invalid_condition = next_reference_game::project_source_v7()?;
    invalid_condition
        .world_routine_interaction_binding_or_none
        .as_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?
        .required_activity = WorldRoutineActivityV1::Rest;

    let mut missing_binding = next_reference_game::project_source_v7()?;
    missing_binding.world_routine_interaction_binding_or_none = None;

    let mut missing_catalog = next_reference_game::project_source_v7()?;
    missing_catalog.world_routine_catalog_or_none = None;

    let mut catalog_identity_collision = next_reference_game::project_source_v7()?;
    let colliding_asset_id = catalog_identity_collision.records[0].asset_id;
    catalog_identity_collision
        .world_routine_catalog_or_none
        .as_mut()
        .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?
        .catalog_asset_id = colliding_asset_id;

    if !invalid(zero_ratio)
        || !invalid(overflow)
        || !invalid(invalid_boundary)
        || !invalid(invalid_activity)
        || !invalid(invalid_condition)
        || !invalid(missing_binding)
        || !invalid(missing_catalog)
        || !matches!(
            cook_project_v7(catalog_identity_collision),
            Err(ProjectCookError::DuplicateIdentity)
        )
    {
        return Err(ContentPackageCheckError::FixtureClosureMismatch);
    }
    Ok(())
}

fn verify_world_routine_publication_faults(
    scratch: &ScratchContext,
    cooked: &next_project::CookedProjectV7,
) -> Result<(), ContentPackageCheckError> {
    let stale_directory = scratch
        .create_directory("content-package-stale-profile")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let stale_result = (|| {
        let original = cooked.publication()?;
        let mut stale_lock = cooked.project_lock.clone();
        stale_lock.runtime_determinism_profile_sha256 = ContentHash::from_bytes([0xfa; 32]);
        let stale_lock = ProjectLockV3::new(stale_lock)
            .map_err(|_| ContentPackageCheckError::FixtureClosureMismatch)?;
        let files = original
            .files
            .iter()
            .map(|file| {
                PublicationFileV1::new(
                    file.relative_path(),
                    if file.relative_path() == "manifests/project-lock.json" {
                        stale_lock.to_jcs_bytes()
                    } else {
                        file.bytes().to_vec()
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let store = ContentStore::new(stale_directory.path());
        store.publish(&ContentPublicationV1::new(
            stale_lock.project_lock_sha256,
            files,
        )?)?;
        if !matches!(
            activate_project_package(&store),
            Err(ProjectActivationError::HashMismatch)
        ) {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(())
    })();
    stale_directory.finish(stale_result, ContentPackageCheckError::Cleanup)?;

    let collision_directory = scratch
        .create_directory("content-package-subject-collision")
        .map_err(ContentPackageCheckError::Cleanup)?;
    let collision_result = (|| {
        let mut source = next_reference_game::project_source_v7()?;
        let colliding_subject = PersistentId::from_bytes([0x54; 16]);
        source
            .world_routine_catalog_or_none
            .as_mut()
            .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?
            .routine
            .subject_id = colliding_subject;
        source
            .world_routine_interaction_binding_or_none
            .as_mut()
            .ok_or(ContentPackageCheckError::FixtureClosureMismatch)?
            .subject_id = colliding_subject;
        let collision_cooked = cook_project_v7(source)?;
        let store = ContentStore::new(collision_directory.path());
        store.publish(&collision_cooked.publication()?)?;
        let activated = activate_project_package(&store)?;
        if !matches!(
            next_reference_game::build_reference_game_session(activated.project),
            Err(next_reference_game::ReferenceGameError::WorldRoutineContentInvalid)
        ) {
            return Err(ContentPackageCheckError::FixtureClosureMismatch);
        }
        Ok(())
    })();
    collision_directory.finish(collision_result, ContentPackageCheckError::Cleanup)
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
    let fallback_snapshot =
        next_contracts::presentation::PresentationSnapshotV3::new_with_character_skinning_records(
            snapshot.snapshot_epoch,
            snapshot.snapshot_sequence,
            snapshot.simulation_tick,
            snapshot.project_composition_lock_hash,
            snapshot.content_manifest_hash,
            snapshot.presentation_profile_hash,
            records,
            snapshot.camera_records().cloned().collect(),
            snapshot.semantic_ui_records().cloned().collect(),
            snapshot.character_skinning_records().cloned().collect(),
            9,
            9,
            next_contracts::presentation::PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH,
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

fn bind_pose_fallback_plan(
    prepared: &crate::PreparedGameFrameV1,
) -> Result<next_render::B0FramePlanV1, ContentPackageCheckError> {
    let snapshot = &prepared.snapshot;
    let skinning_records = snapshot
        .character_skinning_records()
        .map(|record| {
            next_contracts::presentation::CharacterSkinningPresentationRecordV1::new(
                record.object_key,
                record.mesh_revision,
                record.skinning_profile_revision,
                record.source_skeleton_revision,
                record.source_body_schema_revision,
                record.source_animation_profile_hash,
                next_contracts::presentation::BaseSkinningProjectionModeV1::BindPoseFallback,
                next_contracts::presentation::CharacterDeformationLodV1::BaseSkinningOnly,
                record.ordered_local_joint_poses.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ContentPackageCheckError::Presentation)?;
    let fallback_snapshot =
        next_contracts::presentation::PresentationSnapshotV3::new_with_character_skinning_records(
            snapshot.snapshot_epoch,
            snapshot.snapshot_sequence,
            snapshot.simulation_tick,
            snapshot.project_composition_lock_hash,
            snapshot.content_manifest_hash,
            snapshot.presentation_profile_hash,
            snapshot.scene_records().cloned().collect(),
            snapshot.camera_records().cloned().collect(),
            snapshot.semantic_ui_records().cloned().collect(),
            skinning_records,
            9,
            9,
            next_contracts::presentation::PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH,
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
    activated: next_contracts::project::ActivatedProjectV8,
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
    activated: next_contracts::project::ActivatedProjectV8,
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
    Authoring(ProjectAuthoringError),
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
            Self::Authoring(error) => {
                write!(formatter, "content-package authoring failed: {error}")
            }
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

impl From<ProjectAuthoringError> for ContentPackageCheckError {
    fn from(error: ProjectAuthoringError) -> Self {
        Self::Authoring(error)
    }
}

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
        assert_eq!(report.records, 128);
        assert_eq!(report.chunks, 64);
        assert_eq!(report.creator_records, 18);
        assert_eq!(report.creator_chunks, 3);
        assert_eq!(report.mechanic_packages, 2);
        assert_eq!(report.wasm_plugins, 1);
        assert_eq!(report.combat_npc_health, 0);
        assert_eq!(report.scripted_player_health, 50);
        assert_eq!(report.wasm_player_health, 50);
    }
}
