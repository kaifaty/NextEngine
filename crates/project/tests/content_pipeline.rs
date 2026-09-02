use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{
    CONTENT_GENERATIONS_DIRECTORY, ContentPublicationV1, ContentStore, PublicationFileV1,
};
use next_contracts::animation_content::NeutralAnimationValueV1;
use next_contracts::content::NeutralRecordKindV1;
use next_contracts::ids::PersistentId;
use next_contracts::ids::{AssetId, ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::mechanics::interaction_definition_hash_v2;
use next_contracts::project::ProjectLockV3;
use next_contracts::render_content::{
    B0RenderContentProfileV1, NeutralBaseSkinningProfileV1, NeutralRenderRecordV1,
    RenderContentContractError,
};
use next_project::{ProjectActivationError, ProjectCookError, activate_project, cook_project_v7};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn retired_authoring_format_is_rejected_before_v7_schema_decode() {
    let root = test_root("authoring-v1");
    std::fs::create_dir_all(&root).expect("create test project");
    std::fs::write(
        root.join(next_project::PROJECT_AUTHORING_MANIFEST_FILE),
        br#"{"format":"nextengine.project-authoring.v1","legacy":true}"#,
    )
    .expect("write retired authoring manifest");
    let error = next_project::load_project_authoring_v7(&root).expect_err("v1 must reject");
    assert_eq!(
        error.diagnostic_code(),
        "UNSUPPORTED_PROJECT_AUTHORING_FORMAT"
    );
    std::fs::remove_dir_all(root).expect("remove test project");
}

#[test]
fn duplicate_authored_routine_catalog_field_is_rejected_before_source_access() {
    let root = test_root("duplicate-routine-catalog");
    std::fs::create_dir_all(&root).expect("create test project");
    std::fs::write(
        root.join(next_project::PROJECT_AUTHORING_MANIFEST_FILE),
        br#"{
            "format":"nextengine.project-authoring.v7",
            "world_routine_catalog":null,
            "world_routine_catalog":null
        }"#,
    )
    .expect("write duplicate authored catalog");
    let error = next_project::load_project_authoring_v7(&root)
        .expect_err("duplicate authored catalog must reject");
    assert_eq!(error.diagnostic_code(), "PROJECT_MANIFEST_INVALID");
    std::fs::remove_dir_all(root).expect("remove test project");
}

#[test]
fn repeated_cooking_is_byte_identical_and_activates_through_production_loader() {
    let first = cook_project_v7(next_reference_game::project_source_v7().expect("fixture"))
        .expect("first cook");
    let mut reordered = next_reference_game::project_source_v7().expect("fixture");
    reordered.records.reverse();
    reordered.render_records.reverse();
    reordered.root_asset_ids.reverse();
    reordered.chunks.reverse();
    let second = cook_project_v7(reordered).expect("second cook");
    assert_eq!(first, second);
    assert_eq!(
        first.publication().expect("publication"),
        second.publication().expect("publication")
    );

    let root = test_root("activate");
    let store = ContentStore::new(&root);
    store
        .publish(&first.publication().expect("publication"))
        .expect("atomic publication");
    let activated = activate_project(&store).expect("production activation");
    assert_eq!(
        activated.project_lock.project_lock_sha256,
        first.project_lock.project_lock_sha256
    );
    assert_eq!(activated.content_manifest.body.root_assets.len(), 39);
    assert_eq!(activated.content_manifest.body.asset_entries.len(), 125);
    assert_eq!(activated.body_schema_asset, first.body_schema_asset);
    assert_eq!(activated.neutral_records.len(), 76);
    assert_eq!(activated.world_partition.body.root_region_ids.len(), 4);
    assert_eq!(activated.world_partition.body.chunk_bindings.len(), 64);
    for region_id in &activated.world_partition.body.root_region_ids {
        assert_eq!(
            activated
                .world_partition
                .body
                .chunk_bindings
                .iter()
                .filter(|binding| &binding.region_id == region_id)
                .count(),
            16
        );
    }
    assert_eq!(activated.rpg_definitions.abilities.len(), 1);
    assert_eq!(activated.rpg_definitions.packages.len(), 2);
    let routine_catalog = activated
        .world_routine_catalog_or_none
        .as_ref()
        .expect("reference project has one routine catalog");
    assert_eq!(activated.world_population_catalog.records.len(), 100);
    assert_eq!(activated.world_navigation_catalog.nodes.len(), 64);
    activated
        .world_population_catalog
        .validate_against_navigation(&activated.world_navigation_catalog)
        .expect("population and navigation catalogs close exactly");
    let accept = activated
        .rpg_definitions
        .interactions
        .iter()
        .find(|interaction| {
            interaction.interaction_id.as_str()
                == "nextengine.reference-alpha.interaction.accept-frontier-relay"
        })
        .expect("accept interaction");
    assert_eq!(
        accept.availability_condition_or_none,
        Some(
            next_contracts::world_routine::WorldRoutineActivityConditionV1 {
                subject_id: routine_catalog.routine.subject_id,
                required_activity: next_contracts::world_routine::WorldRoutineActivityV1::Duty,
            }
        )
    );
    assert!(
        activated
            .rpg_definitions
            .interactions
            .iter()
            .find(|interaction| {
                interaction.interaction_id.as_str()
                    == "nextengine.reference-alpha.interaction.complete-frontier-relay"
            })
            .expect("complete interaction")
            .availability_condition_or_none
            .is_none()
    );
    assert!(
        activated
            .content_manifest
            .body
            .asset_entries
            .iter()
            .any(|entry| {
                entry.asset_revision.asset_id == routine_catalog.catalog_asset_id
                    && entry.asset_revision.record_sha256
                        == routine_catalog.revision().expect("catalog revision")
            })
    );

    let mut unconditioned_source = next_reference_game::project_source_v7().expect("fixture");
    let catalog_asset_id = unconditioned_source
        .world_routine_catalog_or_none
        .expect("reference catalog")
        .catalog_asset_id;
    unconditioned_source.world_routine_catalog_or_none = None;
    unconditioned_source.world_routine_interaction_binding_or_none = None;
    unconditioned_source
        .root_asset_ids
        .retain(|asset_id| *asset_id != catalog_asset_id);
    let unconditioned = cook_project_v7(unconditioned_source).expect("unconditioned cook");
    let unconditioned_accept = unconditioned
        .rpg_definitions
        .interactions
        .iter()
        .find(|interaction| interaction.interaction_id == accept.interaction_id)
        .expect("unconditioned accept interaction");
    assert_ne!(
        interaction_definition_hash_v2(accept),
        interaction_definition_hash_v2(unconditioned_accept)
    );
    assert_eq!(activated.render_content_catalog.meshes().len(), 13);
    assert_eq!(activated.render_content_catalog.materials().len(), 12);
    assert_eq!(activated.render_content_catalog.textures().len(), 7);
    assert_eq!(
        activated
            .render_content_catalog
            .base_skinning_profiles()
            .len(),
        1
    );
    let skinning = &activated.render_content_catalog.base_skinning_profiles()[0];
    assert_eq!(
        skinning.mesh_revision().asset_id,
        AssetId::from_bytes([0xc1; 16])
    );
    assert_eq!(skinning.render_joints().len(), 8);
    assert_eq!(skinning.pose_correctives().len(), 3);
    assert_eq!(
        skinning
            .pose_correctives()
            .iter()
            .filter(|corrective| {
                corrective.lod_class()
                    == next_contracts::render_content::PoseCorrectiveLodClassV1::Essential
            })
            .count(),
        1
    );
    assert_eq!(skinning.vertices().len(), 48);
    assert_eq!(skinning.max_instances_per_frame(), 2);
    let floor = activated
        .render_content_catalog
        .meshes()
        .iter()
        .find(|mesh| mesh.asset_id() == AssetId::from_bytes([0x81; 16]))
        .expect("floor mesh");
    assert_eq!(
        floor.normals_snorm16(),
        Some([[0, i16::MAX, 0]; 4].as_slice())
    );
    let humanoid = activated
        .render_content_catalog
        .meshes()
        .iter()
        .find(|mesh| mesh.asset_id() == AssetId::from_bytes([0xc1; 16]))
        .expect("fallback-normal mesh");
    assert_eq!(humanoid.normals_snorm16(), None);
    assert_eq!(activated.text_catalogs.len(), 2);
    assert_eq!(activated.text_catalogs[0].locale.as_str(), "en");
    assert_eq!(activated.text_catalogs[1].locale.as_str(), "qps-ploc");
    assert_eq!(activated.neutral_skeletons.len(), 1);
    assert_eq!(activated.neutral_skeletons[0].joints.len(), 8);
    assert_eq!(activated.neutral_animations.len(), 2);
    assert_eq!(activated.neutral_animations[0].channels.len(), 1);
    assert_eq!(activated.neutral_animations[1].channels.len(), 3);
    assert!(
        activated.neutral_animations[0]
            .root_motion_intent
            .is_empty()
    );
    let root_curve = &activated.neutral_animations[1].root_motion_intent;
    assert_eq!(root_curve.len(), 31);
    assert_eq!(root_curve[0].time_microseconds, 0);
    assert_eq!(
        root_curve[0].value,
        NeutralAnimationValueV1::Translation([0, 0, 0])
    );
    assert_eq!(root_curve[30].time_microseconds, 1_000_000);
    assert_eq!(
        root_curve[30].value,
        NeutralAnimationValueV1::Translation([0, 0, 3_000_000])
    );
    assert!(root_curve.windows(2).all(|pair| {
        let NeutralAnimationValueV1::Translation([0, 0, left]) = pair[0].value else {
            return false;
        };
        let NeutralAnimationValueV1::Translation([0, 0, right]) = pair[1].value else {
            return false;
        };
        right - left == 100_000 && pair[0].time_microseconds < pair[1].time_microseconds
    }));
    assert_eq!(
        activated.project_lock.runtime_determinism_profile_sha256,
        next_contracts::identity::RuntimeDeterminismBundleV1::core_r8d()
            .expect("current determinism bundle")
            .runtime_profile_hash(),
    );
    assert!(
        activated
            .content_manifest
            .body
            .asset_entries
            .iter()
            .filter(|entry| {
                entry.schema_ref.schema_id.as_str()
                    == next_contracts::localization::TEXT_CATALOG_SCHEMA_ID
            })
            .all(|entry| {
                entry.semantic_class
                    == next_contracts::project::ContentSemanticClassV1::PresentationOnly
            })
    );
    assert!(
        activated
            .render_content_catalog
            .cooked_meshes()
            .iter()
            .all(|mesh| !mesh.meshlets().is_empty())
    );
    std::fs::remove_dir_all(root).expect("remove test content");
}

#[test]
fn body_schema_asset_profile_root_and_character_binding_fail_closed() {
    let mut wrong_profile = next_reference_game::project_source_v7().expect("fixture");
    wrong_profile.body_schema_asset.compiler_profile_id =
        SchemaId::new("nextengine.body-projection-compiler.unsupported.v1").expect("profile id");
    assert!(matches!(
        cook_project_v7(wrong_profile),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut missing_character_binding = next_reference_game::project_source_v7().expect("fixture");
    let body_asset_id = missing_character_binding.body_schema_asset.asset_id;
    let character = missing_character_binding
        .records
        .iter_mut()
        .find(|record| record.kind == NeutralRecordKindV1::CharacterDefinition)
        .expect("character definition");
    character
        .asset_dependencies
        .retain(|asset_id| *asset_id != body_asset_id);
    assert!(matches!(
        cook_project_v7(missing_character_binding),
        Err(ProjectCookError::MissingReference)
    ));

    let mut missing_root = next_reference_game::project_source_v7().expect("fixture");
    let body_asset_id = missing_root.body_schema_asset.asset_id;
    missing_root
        .root_asset_ids
        .retain(|asset_id| *asset_id != body_asset_id);
    assert!(matches!(
        cook_project_v7(missing_root),
        Err(ProjectCookError::MissingReference)
    ));
}

#[test]
fn chunk_binding_identity_class_and_dependency_faults_fail_before_publication() {
    let mut duplicate_chunk = next_reference_game::project_source_v7().expect("fixture");
    duplicate_chunk.chunks[1].chunk_id = duplicate_chunk.chunks[0].chunk_id.clone();
    assert!(matches!(
        cook_project_v7(duplicate_chunk),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut duplicate_asset = next_reference_game::project_source_v7().expect("fixture");
    duplicate_asset.chunks[1].chunk_asset_id = duplicate_asset.chunks[0].chunk_asset_id;
    assert!(matches!(
        cook_project_v7(duplicate_asset),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut missing_asset = next_reference_game::project_source_v7().expect("fixture");
    missing_asset.chunks[2].required_asset_ids[0] = AssetId::from_bytes([0xfe; 16]);
    assert!(matches!(
        cook_project_v7(missing_asset),
        Err(ProjectCookError::MissingReference)
    ));

    let mut wrong_class = next_reference_game::project_source_v7().expect("fixture");
    wrong_class.chunks[2].chunk_asset_id = AssetId::from_bytes([0x02; 16]);
    assert!(matches!(
        cook_project_v7(wrong_class),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut dependency_mismatch = next_reference_game::project_source_v7().expect("fixture");
    dependency_mismatch.chunks[2].required_asset_ids.clear();
    assert!(matches!(
        cook_project_v7(dependency_mismatch),
        Err(ProjectCookError::InvalidValue)
    ));
}

#[test]
fn malformed_world_routine_content_fails_before_publication() {
    use next_contracts::world_routine::WorldRoutineActivityV1;

    fn source() -> next_project::NeutralProjectSourceV7 {
        next_reference_game::project_source_v7().expect("fixture")
    }

    let mut zero_ratio = source();
    zero_ratio
        .world_routine_catalog_or_none
        .as_mut()
        .expect("catalog")
        .profile
        .world_ticks_per_simulation_tick_num = 0;
    assert!(matches!(
        cook_project_v7(zero_ratio),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut overflow = source();
    let overflow_catalog = overflow
        .world_routine_catalog_or_none
        .as_mut()
        .expect("catalog");
    overflow_catalog.profile.anchor_simulation_tick = u64::MAX;
    overflow_catalog.profile.anchor_world_tick = 0;
    overflow_catalog.profile.world_ticks_per_simulation_tick_num = 1;
    overflow_catalog.profile.world_ticks_per_simulation_tick_den = 1;
    overflow_catalog.routine.transition_world_tick = 1;
    assert!(matches!(
        cook_project_v7(overflow),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut invalid_boundary = source();
    let invalid_boundary_catalog = invalid_boundary
        .world_routine_catalog_or_none
        .as_mut()
        .expect("catalog");
    invalid_boundary_catalog.routine.transition_world_tick =
        invalid_boundary_catalog.profile.anchor_world_tick;
    assert!(matches!(
        cook_project_v7(invalid_boundary),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut invalid_activity = source();
    invalid_activity
        .world_routine_catalog_or_none
        .as_mut()
        .expect("catalog")
        .routine
        .next_activity = WorldRoutineActivityV1::Duty;
    assert!(matches!(
        cook_project_v7(invalid_activity),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut invalid_condition = source();
    invalid_condition
        .world_routine_interaction_binding_or_none
        .as_mut()
        .expect("binding")
        .required_activity = WorldRoutineActivityV1::Rest;
    assert!(matches!(
        cook_project_v7(invalid_condition),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut missing_binding = source();
    missing_binding.world_routine_interaction_binding_or_none = None;
    assert!(matches!(
        cook_project_v7(missing_binding),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut missing_catalog = source();
    missing_catalog.world_routine_catalog_or_none = None;
    assert!(matches!(
        cook_project_v7(missing_catalog),
        Err(ProjectCookError::InvalidValue)
    ));

    let mut catalog_identity_collision = source();
    catalog_identity_collision
        .world_routine_catalog_or_none
        .as_mut()
        .expect("catalog")
        .catalog_asset_id = catalog_identity_collision.records[0].asset_id;
    assert!(matches!(
        cook_project_v7(catalog_identity_collision),
        Err(ProjectCookError::DuplicateIdentity)
    ));
}

#[test]
fn stale_runtime_profile_lock_fails_before_activation() {
    let cooked =
        cook_project_v7(next_reference_game::project_source_v7().expect("fixture")).expect("cook");
    let original = cooked.publication().expect("publication");
    let mut stale_lock = cooked.project_lock.clone();
    stale_lock.runtime_determinism_profile_sha256 = ContentHash::from_bytes([0xfa; 32]);
    let stale_lock = ProjectLockV3::new(stale_lock).expect("stale lock is structurally canonical");
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
        .collect::<Result<Vec<_>, _>>()
        .expect("stale-profile publication files");
    let root = test_root("stale-runtime-profile");
    let store = ContentStore::new(&root);
    store
        .publish(
            &ContentPublicationV1::new(stale_lock.project_lock_sha256, files)
                .expect("storage-valid stale-profile publication"),
        )
        .expect("publish stale-profile generation");
    assert!(matches!(
        activate_project(&store),
        Err(ProjectActivationError::HashMismatch)
    ));
    std::fs::remove_dir_all(root).expect("remove stale-profile store");
}

#[test]
fn multiple_presentation_records_do_not_change_rpg_singleton_selection() {
    let mut source = next_reference_game::project_source_v7().expect("fixture");
    let mut second_scene = source
        .records
        .iter()
        .find(|record| record.kind == NeutralRecordKindV1::Scene)
        .expect("reference scene")
        .clone();
    second_scene.asset_id = AssetId::from_bytes([0xe1; 16]);
    second_scene.record_id = PersistentId::from_bytes([0xe2; 16]);
    source.records.push(second_scene);

    let cooked = cook_project_v7(source).expect("multiple presentation records");

    assert_eq!(cooked.rpg_definitions.abilities.len(), 1);
    assert_eq!(cooked.rpg_definitions.interactions.len(), 2);
}

#[test]
fn missing_blob_and_blob_hash_mismatch_fail_before_activation() {
    let cooked =
        cook_project_v7(next_reference_game::project_source_v7().expect("fixture")).expect("cook");
    let root = test_root("missing");
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let blob_hash = cooked
        .content_manifest
        .body
        .asset_entries
        .first()
        .expect("asset")
        .neutral_record_blob_sha256;
    let blob_path = generation_path(&root, cooked.project_lock.project_lock_sha256)
        .join(format!("blobs/{}.bin", blob_hash.to_hex()));
    std::fs::remove_file(&blob_path).expect("remove test blob");
    assert!(matches!(
        activate_project(&store),
        Err(ProjectActivationError::Store(_))
    ));
    std::fs::remove_dir_all(&root).expect("remove missing store");

    let root = test_root("hash");
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let blob_path = generation_path(&root, cooked.project_lock.project_lock_sha256)
        .join(format!("blobs/{}.bin", blob_hash.to_hex()));
    std::fs::write(blob_path, b"corrupt").expect("corrupt test blob");
    assert!(matches!(
        activate_project(&store),
        Err(ProjectActivationError::Store(_))
    ));
    std::fs::remove_dir_all(root).expect("remove hash store");
}

#[test]
fn missing_cooked_mesh_payload_fails_before_activation() {
    let cooked =
        cook_project_v7(next_reference_game::project_source_v7().expect("fixture")).expect("cook");
    let root = test_root("missing-cooked-mesh");
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let payload_hash = cooked
        .render_content_catalog
        .cooked_meshes()
        .first()
        .expect("cooked mesh")
        .payload_sha256();
    let payload_path = generation_path(&root, cooked.project_lock.project_lock_sha256).join(
        format!("render-content/meshes/{}.bin", payload_hash.to_hex()),
    );
    std::fs::remove_file(payload_path).expect("remove cooked mesh payload");

    assert!(matches!(
        activate_project(&store),
        Err(ProjectActivationError::Store(_))
    ));
    std::fs::remove_dir_all(root).expect("remove cooked mesh store");
}

#[test]
fn storage_valid_but_corrupt_render_catalog_fails_activation() {
    let cooked =
        cook_project_v7(next_reference_game::project_source_v7().expect("fixture")).expect("cook");
    let original = cooked.publication().expect("publication");
    let files = original
        .files
        .iter()
        .map(|file| {
            let bytes = if file.relative_path() == "render-content/catalog.bin" {
                b"corrupt-render-catalog".to_vec()
            } else {
                file.bytes().to_vec()
            };
            PublicationFileV1::new(file.relative_path(), bytes)
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("tampered publication files");
    let root = test_root("corrupt-render-catalog");
    let store = ContentStore::new(&root);
    store
        .publish(
            &ContentPublicationV1::new(original.generation_id, files)
                .expect("storage-valid publication"),
        )
        .expect("publish storage-valid corruption");

    assert!(matches!(
        activate_project(&store),
        Err(ProjectActivationError::Render(_))
    ));
    std::fs::remove_dir_all(root).expect("remove corrupt render catalog store");
}

#[test]
fn invalid_activation_never_replaces_the_callers_active_project() {
    let cooked =
        cook_project_v7(next_reference_game::project_source_v7().expect("fixture")).expect("cook");
    let root = test_root("activation-fault");
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let active = activate_project(&store).expect("initial activation");

    let original = cooked.publication().expect("publication");
    let invalid_generation = content_hash_from_bytes([0xf1; 32]);
    let files = original
        .files
        .iter()
        .map(|file| {
            if file.relative_path() == "manifests/content.json" {
                PublicationFileV1::new(file.relative_path(), b"{\"invalid\":\"schema\"}".to_vec())
            } else if file.relative_path() == "manifests/project-lock.json" {
                let mut bytes = file.bytes().to_vec();
                let old = cooked.project_lock.project_lock_sha256.to_hex();
                let new = invalid_generation.to_hex();
                let text = String::from_utf8(bytes).expect("lock is UTF-8");
                bytes = text.replace(&old, &new).into_bytes();
                PublicationFileV1::new(file.relative_path(), bytes)
            } else {
                PublicationFileV1::new(file.relative_path(), file.bytes().to_vec())
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("rebuild invalid publication");
    store
        .publish(
            &ContentPublicationV1::new(invalid_generation, files)
                .expect("well-formed storage publication"),
        )
        .expect("publish invalid logical generation");
    assert!(activate_project(&store).is_err());
    assert!(active.validate().is_ok());
    assert_eq!(
        active.project_lock.project_lock_sha256,
        cooked.project_lock.project_lock_sha256
    );
    std::fs::remove_dir_all(root).expect("remove activation store");
}

#[test]
fn malformed_schema_missing_reference_duplicate_id_and_cycle_are_rejected() {
    let mut malformed = next_reference_game::project_source_v7().expect("fixture");
    malformed.records[0].schema_ref.schema_id =
        SchemaId::new("nextengine.content.wrong.v1").expect("valid ID");
    assert!(matches!(
        cook_project_v7(malformed),
        Err(ProjectCookError::Neutral(_))
    ));

    let mut missing = next_reference_game::project_source_v7().expect("fixture");
    missing.records[0]
        .asset_dependencies
        .push(AssetId::from_bytes([0xfe; 16]));
    assert!(matches!(
        cook_project_v7(missing),
        Err(ProjectCookError::MissingReference)
    ));

    let mut duplicate = next_reference_game::project_source_v7().expect("fixture");
    duplicate.records[1].asset_id = duplicate.records[0].asset_id;
    assert!(matches!(
        cook_project_v7(duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut cross_kind_duplicate = next_reference_game::project_source_v7().expect("fixture");
    cross_kind_duplicate.records[0].asset_id = cross_kind_duplicate.render_records[0].asset_id();
    assert!(matches!(
        cook_project_v7(cross_kind_duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut missing_fallback = next_reference_game::project_source_v7().expect("fixture");
    let fallback_texture = missing_fallback
        .render_records
        .iter()
        .find_map(|record| match record {
            NeutralRenderRecordV1::Profile(profile) => Some(profile.fallback_texture().asset_id),
            _ => None,
        })
        .expect("fallback texture");
    missing_fallback
        .render_records
        .retain(|record| record.asset_id() != fallback_texture);
    assert!(matches!(
        cook_project_v7(missing_fallback),
        Err(ProjectCookError::MissingReference)
    ));

    let mut missing_profile = next_reference_game::project_source_v7().expect("fixture");
    missing_profile
        .render_records
        .retain(|record| !matches!(record, NeutralRenderRecordV1::Profile(_)));
    assert!(matches!(
        cook_project_v7(missing_profile),
        Err(ProjectCookError::Render(
            RenderContentContractError::MissingReference
        ))
    ));

    let mut duplicate_profile = next_reference_game::project_source_v7().expect("fixture");
    let profile = duplicate_profile
        .render_records
        .iter()
        .find_map(|record| match record {
            NeutralRenderRecordV1::Profile(profile) => Some(profile),
            _ => None,
        })
        .expect("profile");
    let second_profile = B0RenderContentProfileV1::new(
        profile.schema_ref().clone(),
        AssetId::from_bytes([0xe3; 16]),
        profile.record_revision(),
        profile.shader_interface_manifest_sha256(),
        profile.fallback_material(),
        profile.fallback_texture(),
    )
    .expect("second profile");
    duplicate_profile.render_records.push(second_profile.into());
    assert!(matches!(
        cook_project_v7(duplicate_profile),
        Err(ProjectCookError::Render(
            RenderContentContractError::DuplicateIdentity
        ))
    ));

    let mut invalid_skinning_mapping = next_reference_game::project_source_v7().expect("fixture");
    let skinning_index = invalid_skinning_mapping
        .render_records
        .iter()
        .position(|record| matches!(record, NeutralRenderRecordV1::BaseSkinningProfile(_)))
        .expect("base skinning profile");
    let NeutralRenderRecordV1::BaseSkinningProfile(skinning) =
        &invalid_skinning_mapping.render_records[skinning_index]
    else {
        unreachable!("profile index was selected above")
    };
    let mut render_joints = skinning.render_joints().to_vec();
    render_joints[0].animation_joint_id =
        SchemaId::new("nextengine.missing.animation-joint").expect("joint id");
    let invalid_skinning = NeutralBaseSkinningProfileV1::new(
        skinning.schema_ref().clone(),
        skinning.asset_id(),
        skinning.record_revision(),
        skinning.mesh_revision(),
        skinning.skeleton_revision(),
        skinning.body_schema_revision(),
        skinning.mesh_origin_in_skeleton_micrometres(),
        skinning.method(),
        skinning.fallback(),
        skinning.max_instances_per_frame(),
        render_joints,
        skinning.pose_correctives().to_vec(),
        skinning.vertices().to_vec(),
    )
    .expect("mapping is structurally valid before exact skeleton closure");
    invalid_skinning_mapping.render_records[skinning_index] = invalid_skinning.into();
    assert!(matches!(
        cook_project_v7(invalid_skinning_mapping),
        Err(ProjectCookError::Render(
            RenderContentContractError::InvalidSkinningProfile
        ))
    ));

    let mut cycle = next_reference_game::project_source_v7().expect("fixture");
    let scene = cycle.records[0].asset_id;
    cycle.records[1].asset_dependencies.push(scene);
    assert!(matches!(
        cook_project_v7(cycle),
        Err(ProjectCookError::Contract(
            next_contracts::project::ProjectContractError::DependencyCycle
        ))
    ));
}

#[test]
fn localization_closure_violations_are_rejected_before_publication() {
    use next_contracts::localization::{TextCatalogV1, TextLocaleTagV1};

    fn rebuild(
        catalog: &TextCatalogV1,
        asset_id: AssetId,
        locale: &str,
        fallback: Option<&str>,
    ) -> TextCatalogV1 {
        TextCatalogV1::new(
            asset_id,
            catalog.revision,
            TextLocaleTagV1::new(locale).expect("locale"),
            fallback.map(|value| TextLocaleTagV1::new(value).expect("fallback")),
            catalog.entries.clone(),
        )
        .expect("rebuilt catalog")
    }

    let mut duplicate_locale = next_reference_game::project_source_v7().expect("fixture");
    duplicate_locale.text_catalogs[1] = rebuild(
        &duplicate_locale.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "en",
        None,
    );
    assert!(matches!(
        cook_project_v7(duplicate_locale),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut two_roots = next_reference_game::project_source_v7().expect("fixture");
    two_roots.text_catalogs[1] = rebuild(
        &two_roots.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "de",
        None,
    );
    assert!(matches!(
        cook_project_v7(two_roots),
        Err(ProjectCookError::LocalizationClosureInvalid)
    ));

    let mut missing_fallback = next_reference_game::project_source_v7().expect("fixture");
    missing_fallback.text_catalogs[1] = rebuild(
        &missing_fallback.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "qps-ploc",
        Some("de"),
    );
    assert!(matches!(
        cook_project_v7(missing_fallback),
        Err(ProjectCookError::MissingReference)
    ));

    let mut cyclic = next_reference_game::project_source_v7().expect("fixture");
    cyclic.text_catalogs[1] = rebuild(
        &cyclic.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "qps-ploc",
        Some("de"),
    );
    let cyclic_de = rebuild(
        &cyclic.text_catalogs[1].clone(),
        AssetId::from_bytes([0x93; 16]),
        "de",
        Some("qps-ploc"),
    );
    cyclic.text_catalogs.push(cyclic_de);
    assert!(matches!(
        cook_project_v7(cyclic),
        Err(ProjectCookError::LocalizationClosureInvalid)
    ));

    let mut shared_asset_id = next_reference_game::project_source_v7().expect("fixture");
    shared_asset_id.text_catalogs[1] = rebuild(
        &shared_asset_id.text_catalogs[1].clone(),
        shared_asset_id.records[0].asset_id,
        "qps-ploc",
        Some("en"),
    );
    assert!(matches!(
        cook_project_v7(shared_asset_id),
        Err(ProjectCookError::DuplicateIdentity)
    ));
}

#[test]
fn audio_clips_cook_publish_and_activate_through_production_loader() {
    use next_contracts::audio::{
        AudioLoudnessMetadataV1, AudioPcmEncodingV1, NEUTRAL_AUDIO_SCHEMA_ID, NeutralAudioV1,
    };

    let clip = NeutralAudioV1::new(
        AssetId::from_bytes([0xb3; 16]),
        1,
        48_000,
        2,
        AudioPcmEncodingV1::PcmS16Le,
        4,
        None,
        vec![0, 2],
        AudioLoudnessMetadataV1::new(-1_015_806, 45_875).expect("loudness"),
        vec![0_u8; 16],
    )
    .expect("clip");

    let mut source = next_reference_game::project_source_v7().expect("fixture");
    source.audio_records.push(clip.clone());
    let cooked = cook_project_v7(source).expect("cook with audio");
    let audio_entries: Vec<_> = cooked
        .content_manifest
        .body
        .asset_entries
        .iter()
        .filter(|entry| entry.schema_ref.schema_id.as_str() == NEUTRAL_AUDIO_SCHEMA_ID)
        .collect();
    // Four engine-owned reference clips plus the test clip.
    assert_eq!(audio_entries.len(), 5);
    let test_entry = audio_entries
        .iter()
        .find(|entry| entry.asset_revision.asset_id == clip.asset_id)
        .expect("test clip entry");
    assert_eq!(
        test_entry.semantic_class,
        next_contracts::project::ContentSemanticClassV1::PresentationOnly
    );
    assert_eq!(
        test_entry.asset_revision.record_sha256,
        clip.record_sha256().expect("hash")
    );

    let root = test_root("audio");
    let store = ContentStore::new(&root);
    store
        .publish(&cooked.publication().expect("publication"))
        .expect("publish");
    let activated = activate_project(&store).expect("activate with audio");
    assert_eq!(activated.audio_clips.len(), 5);
    assert!(activated.audio_clips.contains(&clip));
    std::fs::remove_dir_all(root).expect("remove audio store");

    let mut duplicate = next_reference_game::project_source_v7().expect("fixture");
    duplicate.audio_records.push(
        NeutralAudioV1::new(
            duplicate.records[0].asset_id,
            1,
            48_000,
            2,
            AudioPcmEncodingV1::PcmS16Le,
            4,
            None,
            Vec::new(),
            AudioLoudnessMetadataV1::new(0, 0).expect("loudness"),
            vec![0_u8; 16],
        )
        .expect("clip"),
    );
    assert!(matches!(
        cook_project_v7(duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut invalid = next_reference_game::project_source_v7().expect("fixture");
    let mut invalid_clip = clip;
    invalid_clip.sample_rate_hz = 7_999;
    invalid.audio_records.push(invalid_clip);
    assert!(matches!(
        cook_project_v7(invalid),
        Err(ProjectCookError::Audio(_))
    ));
}

fn generation_path(root: &std::path::Path, hash: ContentHash) -> std::path::PathBuf {
    root.join(CONTENT_GENERATIONS_DIRECTORY).join(hash.to_hex())
}

fn test_root(label: &str) -> std::path::PathBuf {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "nextengine-project-{label}-{}-{counter}",
        std::process::id()
    ))
}
