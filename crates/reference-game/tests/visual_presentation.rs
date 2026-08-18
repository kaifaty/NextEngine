use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::ContentStore;
use next_contracts::ids::{AssetId, PersistentId};
use next_contracts::presentation::{
    BaseSkinningProjectionModeV1, CharacterDeformationLodV1, CharacterSkinningPresentationRecordV1,
    PresentationSnapshotV3, ScenePresentationFlagsV1, ScenePresentationRecordV2,
};
use next_contracts::render_content::RenderContentCatalogV1;
use next_render::{RenderTargetV1, build_b0_frame_plan};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "nextengine-reference-visual-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

#[test]
fn reference_visual_bindings_replace_markers_and_follow_rpg_state() {
    let root = test_root();
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let quest_giver_character_id = activated
        .project
        .world_routine_catalog_or_none
        .as_ref()
        .expect("reference routine catalog")
        .routine
        .subject_id;

    let driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("live driver");
    let initial = driver.state().expect("initial state");
    let initial_scene = initial
        .presentation_snapshot
        .scene_records()
        .collect::<Vec<_>>();
    assert_eq!(initial_scene.len(), 10);
    assert!(
        initial_scene
            .iter()
            .all(|record| record.mesh_revision.asset_id != AssetId::from_bytes([0x82; 16]))
    );
    let action = initial
        .presentation_snapshot
        .semantic_ui_records()
        .find(|record| {
            record.element.element_id.as_str() == next_reference_game::HUD_ACTION_ELEMENT_ID
        })
        .expect("initial next-action presentation");
    assert_eq!(
        action
            .element
            .text_or_none
            .as_ref()
            .expect("action text")
            .text_id
            .as_str(),
        next_reference_game::HUD_ACTION_ACCEPT_TEXT_ID
    );

    let player = initial_scene
        .iter()
        .find(|record| {
            record.object_key.presentation_role
                == next_contracts::presentation::PresentationRoleV1::PlayerAvatar
        })
        .expect("player presentation");
    assert_eq!(
        player.mesh_revision.asset_id,
        AssetId::from_bytes([0xc1; 16])
    );
    assert_eq!(
        player.material_revision.asset_id,
        AssetId::from_bytes([0xd1; 16])
    );
    let enemy = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x59; 16]))
        .expect("enemy presentation");
    assert_eq!(
        enemy.mesh_revision.asset_id,
        AssetId::from_bytes([0xc1; 16])
    );
    assert_eq!(
        initial
            .presentation_snapshot
            .character_skinning_records()
            .count(),
        2
    );
    assert!([player, enemy].iter().all(|record| {
        record
            .feature_flags
            .contains(next_contracts::presentation::ScenePresentationFlagsV1::SKINNED)
    }));
    let quest_giver = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == quest_giver_character_id)
        .expect("quest-giver presentation");
    assert_eq!(
        quest_giver.mesh_revision.asset_id,
        AssetId::from_bytes([0xca; 16])
    );
    let focus_ring = initial_scene
        .iter()
        .find(|record| record.instance_ordinal == 240)
        .expect("focus ring presentation");
    assert_eq!(
        focus_ring.mesh_revision.asset_id,
        AssetId::from_bytes([0xc7; 16])
    );
    assert_eq!(
        focus_ring.material_revision.asset_id,
        AssetId::from_bytes([0xd7; 16])
    );
    let initial_plan = build_b0_frame_plan(
        &initial.presentation_snapshot,
        &activated.project.render_content_catalog,
        RenderTargetV1 {
            extent: [1_920, 1_080],
            target_revision: 1,
        },
    )
    .expect("initial visual plan");
    assert_eq!(initial_plan.skinned_vertex_streams.len(), 2);
    assert!(
        initial_plan
            .skinned_vertex_streams
            .iter()
            .all(|stream| !stream.used_bind_pose_fallback)
    );
    assert!(initial_plan.draws.iter().any(|draw| {
        draw.material_revision.asset_id == AssetId::from_bytes([0xd7; 16]) && !draw.casts_shadow
    }));
    assert!(initial_plan.draws.iter().all(|draw| {
        draw.material_revision.asset_id == AssetId::from_bytes([0xd7; 16])
            || draw.skinning_vertex_stream_index.is_some()
            || draw.casts_shadow
    }));

    let initial_pickup = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x5f; 16]))
        .expect("pickup presentation");
    assert_eq!(
        initial_pickup.mesh_revision.asset_id,
        AssetId::from_bytes([0xc2; 16])
    );
    assert!(initial_pickup.visible);

    let initial_relay = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x58; 16]))
        .expect("relay presentation");
    assert_eq!(
        initial_relay.mesh_revision.asset_id,
        AssetId::from_bytes([0xc3; 16])
    );
    assert_eq!(
        initial_relay.material_revision.asset_id,
        AssetId::from_bytes([0xd5; 16])
    );

    let environment_instances = initial_scene
        .iter()
        .filter(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x70; 16]))
        .collect::<Vec<_>>();
    assert_eq!(environment_instances.len(), 1);
    assert!(environment_instances.iter().all(|record| {
        record.object_key.presentation_role
            == next_contracts::presentation::PresentationRoleV1::Environment
            && record.visible
    }));
    let relay_approach = environment_instances
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x70; 16]))
        .expect("relay approach presentation");
    assert_eq!(
        relay_approach.mesh_revision.asset_id,
        AssetId::from_bytes([0xc6; 16])
    );
    assert_eq!(
        relay_approach.material_revision.asset_id,
        AssetId::from_bytes([0xd9; 16])
    );
    let capsule_course = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x78; 16]))
        .expect("R5b capsule course presentation");
    assert_eq!(
        capsule_course.mesh_revision.asset_id,
        AssetId::from_bytes([0xcb; 16])
    );
    assert_eq!(
        capsule_course.current_transform.translation_micrometres,
        [0; 3]
    );
    let push_box = initial_scene
        .iter()
        .find(|record| record.object_key.persistent_id == PersistentId::from_bytes([0x79; 16]))
        .expect("R5b push box presentation");
    assert_eq!(
        push_box.mesh_revision.asset_id,
        AssetId::from_bytes([0xcc; 16])
    );
    assert_eq!(
        push_box.current_transform.translation_micrometres,
        [5_200_000, 300_000, -6_000_000]
    );
    let outcome = next_reference_game::run_reference_game(activated, true).expect("reference run");
    let final_pickup = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.pickup_item_id)
        .expect("final pickup binding");
    assert!(!final_pickup.visible);
    let final_enemy = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.npc_character_id)
        .expect("final enemy binding");
    assert!(final_enemy.visible);
    assert_eq!(
        final_enemy.material_revision.asset_id,
        AssetId::from_bytes([0xd8; 16])
    );
    let final_relay = outcome
        .presentation_bindings
        .iter()
        .find(|binding| binding.persistent_id == outcome.interactive_object_id)
        .expect("final relay binding");
    assert_eq!(
        final_relay.material_revision.asset_id,
        AssetId::from_bytes([0xd6; 16])
    );
    assert_eq!(outcome.presentation_bindings.len(), 9);
    assert!(outcome.presentation_bindings.iter().all(|binding| {
        binding.persistent_id != PersistentId::from_bytes([0x70; 16])
            || (binding.presentation_role
                == next_contracts::presentation::PresentationRoleV1::Environment
                && binding.physics_body_id.is_none()
                && binding.visible)
    }));
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn pose_corrective_lod_and_renderer_cadence_are_presentation_only() {
    let root = test_root();
    let store = ContentStore::new(&root);
    let cooked = next_project::cook_project_v7(
        next_reference_game::project_source_v7().expect("reference source"),
    )
    .expect("cook");
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = next_project::activate_project_package(&store).expect("activate");
    let driver = next_reference_game::ReferenceGameDriverV2::new(activated.clone(), true)
        .expect("live driver");
    let before = driver.state().expect("initial state");
    let before_state_root = before.checkpoint.state_root;
    let before_ledger_root = before
        .checkpoint_canonical_components
        .command_ledger_hash()
        .expect("ledger root");
    let before_physics = before
        .checkpoint_canonical_components
        .physics_checkpoint_bytes()
        .to_vec();
    let before_physical_animation = before.physical_animation_snapshot.clone();
    let catalog = &activated.project.render_content_catalog;
    let full = character_snapshot_variant(
        &before.presentation_snapshot,
        catalog,
        BaseSkinningProjectionModeV1::Sampled,
        CharacterDeformationLodV1::FullCorrectives,
        true,
        0,
    );
    let reduced = character_snapshot_variant(
        &before.presentation_snapshot,
        catalog,
        BaseSkinningProjectionModeV1::Sampled,
        CharacterDeformationLodV1::ReducedCorrectives,
        true,
        0,
    );
    let base = character_snapshot_variant(
        &before.presentation_snapshot,
        catalog,
        BaseSkinningProjectionModeV1::Sampled,
        CharacterDeformationLodV1::BaseSkinningOnly,
        true,
        0,
    );
    let held = character_snapshot_variant(
        &before.presentation_snapshot,
        catalog,
        BaseSkinningProjectionModeV1::HeldPresentationPose,
        CharacterDeformationLodV1::FullCorrectives,
        true,
        0,
    );
    let culled = character_snapshot_variant(
        &before.presentation_snapshot,
        catalog,
        BaseSkinningProjectionModeV1::Sampled,
        CharacterDeformationLodV1::Culled,
        true,
        0,
    );
    let corrective_fallback = character_snapshot_variant(
        &before.presentation_snapshot,
        catalog,
        BaseSkinningProjectionModeV1::Sampled,
        CharacterDeformationLodV1::FullCorrectives,
        true,
        30_000,
    );
    let target = RenderTargetV1 {
        extent: [1_920, 1_080],
        target_revision: 1,
    };
    let full_plan = build_b0_frame_plan(&full, catalog, target).expect("full plan");
    let reduced_plan = build_b0_frame_plan(&reduced, catalog, target).expect("reduced plan");
    let base_plan = build_b0_frame_plan(&base, catalog, target).expect("base plan");
    let held_plan = build_b0_frame_plan(&held, catalog, target).expect("held plan");
    let culled_plan = build_b0_frame_plan(&culled, catalog, target).expect("culled plan");
    let corrective_fallback_plan = build_b0_frame_plan(&corrective_fallback, catalog, target)
        .expect("corrective fallback plan");

    assert_eq!(full_plan.skinned_vertex_streams.len(), 2);
    assert!(full_plan.skinned_vertex_streams.iter().all(|stream| {
        stream.applied_pose_corrective_count == 3
            && !stream.used_pose_corrective_fallback
            && !stream.used_bind_pose_fallback
    }));
    assert!(reduced_plan.skinned_vertex_streams.iter().all(|stream| {
        stream.applied_pose_corrective_count == 1
            && !stream.used_pose_corrective_fallback
            && !stream.used_bind_pose_fallback
    }));
    assert!(base_plan.skinned_vertex_streams.iter().all(|stream| {
        stream.applied_pose_corrective_count == 0
            && !stream.used_pose_corrective_fallback
            && !stream.used_bind_pose_fallback
    }));
    assert_eq!(
        full_plan
            .skinned_vertex_streams
            .iter()
            .map(|stream| &stream.positions_micrometres)
            .collect::<Vec<_>>(),
        held_plan
            .skinned_vertex_streams
            .iter()
            .map(|stream| &stream.positions_micrometres)
            .collect::<Vec<_>>()
    );
    assert_ne!(full_plan.frame_plan_hash, reduced_plan.frame_plan_hash);
    assert_ne!(reduced_plan.frame_plan_hash, base_plan.frame_plan_hash);
    assert_ne!(full_plan.frame_plan_hash, held_plan.frame_plan_hash);
    assert_eq!(culled_plan.skinned_vertex_streams.len(), 0);
    assert_eq!(
        culled_plan.visible_object_count + 2,
        full_plan.visible_object_count
    );
    assert!(
        corrective_fallback_plan
            .skinned_vertex_streams
            .iter()
            .all(|stream| {
                stream.applied_pose_corrective_count == 0
                    && stream.used_pose_corrective_fallback
                    && !stream.used_bind_pose_fallback
            })
    );

    for (snapshot, expected_hash) in [
        (&full, full_plan.frame_plan_hash),
        (&reduced, reduced_plan.frame_plan_hash),
        (&base, base_plan.frame_plan_hash),
        (&held, held_plan.frame_plan_hash),
        (&culled, culled_plan.frame_plan_hash),
    ] {
        for cadence_hz in [30_u32, 60, 144] {
            let repeated_frames = cadence_hz.div_ceil(30);
            for _ in 0..repeated_frames {
                assert_eq!(
                    build_b0_frame_plan(snapshot, catalog, target)
                        .expect("repeated complete snapshot")
                        .frame_plan_hash,
                    expected_hash
                );
            }
        }
    }

    let after = driver.state().expect("state after presentation work");
    assert_eq!(after.checkpoint.state_root, before_state_root);
    assert_eq!(
        after
            .checkpoint_canonical_components
            .command_ledger_hash()
            .expect("ledger root"),
        before_ledger_root
    );
    assert_eq!(
        after
            .checkpoint_canonical_components
            .physics_checkpoint_bytes(),
        before_physics
    );
    assert_eq!(after.physical_animation_snapshot, before_physical_animation);
    drop(driver);
    drop(activated);
    std::fs::remove_dir_all(root).expect("cleanup");
}

fn character_snapshot_variant(
    snapshot: &PresentationSnapshotV3,
    catalog: &RenderContentCatalogV1,
    projection_mode: BaseSkinningProjectionModeV1,
    deformation_lod: CharacterDeformationLodV1,
    activate_all_correctives: bool,
    detail_driver_overshoot_micrometres: i64,
) -> PresentationSnapshotV3 {
    let scene_records = snapshot
        .scene_records()
        .map(|record| {
            let visible = if record
                .feature_flags
                .contains(ScenePresentationFlagsV1::SKINNED)
            {
                deformation_lod != CharacterDeformationLodV1::Culled
            } else {
                record.visible
            };
            ScenePresentationRecordV2::new(
                record.presentation_layer,
                record.object_key,
                record.mesh_revision,
                record.material_revision,
                record.instance_ordinal,
                record.local_bounds,
                record.feature_flags,
                record.previous_transform,
                record.current_transform,
                visible,
            )
        })
        .collect();
    let skinning_records = snapshot
        .character_skinning_records()
        .map(|record| {
            let profile = catalog
                .base_skinning_profile(record.skinning_profile_revision)
                .expect("exact skinning profile");
            let mut poses = record.ordered_local_joint_poses.clone();
            if activate_all_correctives {
                for corrective in profile.pose_correctives() {
                    let bind_joint = profile
                        .render_joints()
                        .iter()
                        .find(|joint| joint.render_joint_id == *corrective.driver_render_joint_id())
                        .expect("corrective driver joint");
                    let pose = poses
                        .iter_mut()
                        .find(|pose| pose.render_joint_id == *corrective.driver_render_joint_id())
                        .expect("complete render pose");
                    let axis = corrective.driver_axis().index();
                    let detail_overshoot = if corrective.lod_class()
                        == next_contracts::render_content::PoseCorrectiveLodClassV1::Detail
                    {
                        detail_driver_overshoot_micrometres
                    } else {
                        0
                    };
                    pose.local_transform.translation_micrometres[axis] =
                        bind_joint.bind_transform.translation_micrometres[axis]
                            .checked_add(corrective.activation_full_delta_micrometres())
                            .and_then(|value| value.checked_add(detail_overshoot))
                            .expect("bounded corrective driver");
                }
            }
            CharacterSkinningPresentationRecordV1::new(
                record.object_key,
                record.mesh_revision,
                record.skinning_profile_revision,
                record.source_skeleton_revision,
                record.source_body_schema_revision,
                record.source_animation_profile_hash,
                projection_mode,
                deformation_lod,
                poses,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("character skinning variants");
    PresentationSnapshotV3::new_with_character_skinning_records(
        snapshot.snapshot_epoch,
        snapshot.snapshot_sequence,
        snapshot.simulation_tick,
        snapshot.project_composition_lock_hash,
        snapshot.content_manifest_hash,
        snapshot.presentation_profile_hash,
        scene_records,
        snapshot.camera_records().cloned().collect(),
        snapshot.semantic_ui_records().cloned().collect(),
        skinning_records,
        8,
        1,
        next_contracts::presentation::PRESENTATION_DEFAULT_SEMANTIC_UI_RECORDS_PER_BATCH,
        snapshot.environment_batch,
    )
    .expect("complete character presentation variant")
}
