use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{
    CONTENT_GENERATIONS_DIRECTORY, ContentPublicationV1, ContentStore, PublicationFileV1,
};
use next_contracts::content::NeutralRecordKindV1;
use next_contracts::ids::PersistentId;
use next_contracts::ids::{AssetId, ContentHash, SchemaId, content_hash_from_bytes};
use next_contracts::project::{
    ProjectCatalogRecordV1, ProjectCatalogSnapshotV1, ProjectDependencyKindV1, ProjectManifestV1,
    ProjectRequirementV1, SemanticVersionV1,
};
use next_contracts::render_content::{
    B0RenderContentProfileV1, NeutralRenderRecordV1, RenderContentContractError,
};
use next_project::{
    ProjectActivationError, ProjectCookError, ProjectResolutionError, activate_project,
    cook_project_v1, resolve_project_records_v1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn repeated_cooking_is_byte_identical_and_activates_through_production_loader() {
    let first = cook_project_v1(next_reference_game::project_source_v2().expect("fixture"))
        .expect("first cook");
    let mut reordered = next_reference_game::project_source_v2().expect("fixture");
    reordered.records.reverse();
    reordered.render_records.reverse();
    reordered.root_asset_ids.reverse();
    reordered.chunks.reverse();
    let second = cook_project_v1(reordered).expect("second cook");
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
        activated.composition_lock.composition_lock_sha256,
        first.composition_lock.composition_lock_sha256
    );
    assert_eq!(activated.content_manifest.body.asset_entries.len(), 26);
    assert_eq!(activated.world_partition.body.chunk_bindings.len(), 2);
    assert_eq!(activated.rpg_definitions.abilities.len(), 1);
    assert_eq!(activated.rpg_definitions.packages.len(), 2);
    assert_eq!(activated.render_content_catalog.meshes().len(), 2);
    assert_eq!(activated.render_content_catalog.materials().len(), 2);
    assert_eq!(activated.render_content_catalog.textures().len(), 2);
    assert_eq!(activated.text_catalogs.len(), 2);
    assert_eq!(activated.text_catalogs[0].locale.as_str(), "en");
    assert_eq!(activated.text_catalogs[1].locale.as_str(), "qps-ploc");
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
fn multiple_presentation_records_do_not_change_rpg_singleton_selection() {
    let mut source = next_reference_game::project_source_v2().expect("fixture");
    let mut second_scene = source
        .records
        .iter()
        .find(|record| record.kind == NeutralRecordKindV1::Scene)
        .expect("reference scene")
        .clone();
    second_scene.asset_id = AssetId::from_bytes([0xe1; 16]);
    second_scene.record_id = PersistentId::from_bytes([0xe2; 16]);
    source.records.push(second_scene);

    let cooked = cook_project_v1(source).expect("multiple presentation records");

    assert_eq!(cooked.rpg_definitions.abilities.len(), 1);
    assert_eq!(cooked.rpg_definitions.interactions.len(), 1);
}

#[test]
fn missing_blob_and_blob_hash_mismatch_fail_before_activation() {
    let cooked =
        cook_project_v1(next_reference_game::project_source_v2().expect("fixture")).expect("cook");
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
    let blob_path = generation_path(&root, cooked.composition_lock.composition_lock_sha256)
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
    let blob_path = generation_path(&root, cooked.composition_lock.composition_lock_sha256)
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
        cook_project_v1(next_reference_game::project_source_v2().expect("fixture")).expect("cook");
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
    let payload_path = generation_path(&root, cooked.composition_lock.composition_lock_sha256)
        .join(format!(
            "render-content/meshes/{}.bin",
            payload_hash.to_hex()
        ));
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
        cook_project_v1(next_reference_game::project_source_v2().expect("fixture")).expect("cook");
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
        cook_project_v1(next_reference_game::project_source_v2().expect("fixture")).expect("cook");
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
            } else if file.relative_path() == "manifests/composition-lock.json" {
                let mut bytes = file.bytes().to_vec();
                let old = cooked.composition_lock.composition_lock_sha256.to_hex();
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
        active.composition_lock.composition_lock_sha256,
        cooked.composition_lock.composition_lock_sha256
    );
    std::fs::remove_dir_all(root).expect("remove activation store");
}

#[test]
fn malformed_schema_missing_reference_duplicate_id_and_cycle_are_rejected() {
    let mut malformed = next_reference_game::project_source_v2().expect("fixture");
    malformed.records[0].schema_ref.schema_id =
        SchemaId::new("nextengine.content.wrong.v1").expect("valid ID");
    assert!(matches!(
        cook_project_v1(malformed),
        Err(ProjectCookError::Neutral(_))
    ));

    let mut missing = next_reference_game::project_source_v2().expect("fixture");
    missing.records[0]
        .asset_dependencies
        .push(AssetId::from_bytes([0xfe; 16]));
    assert!(matches!(
        cook_project_v1(missing),
        Err(ProjectCookError::MissingReference)
    ));

    let mut duplicate = next_reference_game::project_source_v2().expect("fixture");
    duplicate.records[1].asset_id = duplicate.records[0].asset_id;
    assert!(matches!(
        cook_project_v1(duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut cross_kind_duplicate = next_reference_game::project_source_v2().expect("fixture");
    cross_kind_duplicate.records[0].asset_id = cross_kind_duplicate.render_records[0].asset_id();
    assert!(matches!(
        cook_project_v1(cross_kind_duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut missing_fallback = next_reference_game::project_source_v2().expect("fixture");
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
        cook_project_v1(missing_fallback),
        Err(ProjectCookError::MissingReference)
    ));

    let mut missing_profile = next_reference_game::project_source_v2().expect("fixture");
    missing_profile
        .render_records
        .retain(|record| !matches!(record, NeutralRenderRecordV1::Profile(_)));
    assert!(matches!(
        cook_project_v1(missing_profile),
        Err(ProjectCookError::Render(
            RenderContentContractError::MissingReference
        ))
    ));

    let mut duplicate_profile = next_reference_game::project_source_v2().expect("fixture");
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
        cook_project_v1(duplicate_profile),
        Err(ProjectCookError::Render(
            RenderContentContractError::DuplicateIdentity
        ))
    ));

    let mut cycle = next_reference_game::project_source_v2().expect("fixture");
    let scene = cycle.records[0].asset_id;
    cycle.records[1].asset_dependencies.push(scene);
    assert!(matches!(
        cook_project_v1(cycle),
        Err(ProjectCookError::Contract(
            next_contracts::project::ProjectContractError::DependencyCycle
        ))
    ));
}

#[test]
fn project_resolver_rejects_dependency_cycle() {
    let project = ProjectManifestV1::new(
        next_contracts::ids::ProjectId::new("org.nextengine.resolver-test").expect("project"),
        1,
        vec![requirement("a")],
    )
    .expect("manifest");
    let record_a = ProjectCatalogRecordV1::new(
        ProjectDependencyKindV1::Content,
        SchemaId::new("a").expect("ID"),
        SemanticVersionV1::new(1, 0, 0),
        content_hash_from_bytes([1; 32]),
        vec![requirement("b")],
        false,
    )
    .expect("record a");
    let record_b = ProjectCatalogRecordV1::new(
        ProjectDependencyKindV1::Content,
        SchemaId::new("b").expect("ID"),
        SemanticVersionV1::new(1, 0, 0),
        content_hash_from_bytes([2; 32]),
        vec![requirement("a")],
        false,
    )
    .expect("record b");
    let catalog = ProjectCatalogSnapshotV1::new(
        SchemaId::new("nextengine.resolver.test").expect("profile"),
        1,
        content_hash_from_bytes([3; 32]),
        vec![record_a, record_b],
    )
    .expect("catalog");
    assert_eq!(
        resolve_project_records_v1(&project, &catalog),
        Err(ProjectResolutionError::DependencyCycle)
    );
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

    let mut duplicate_locale = next_reference_game::project_source_v2().expect("fixture");
    duplicate_locale.text_catalogs[1] = rebuild(
        &duplicate_locale.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "en",
        None,
    );
    assert!(matches!(
        cook_project_v1(duplicate_locale),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut two_roots = next_reference_game::project_source_v2().expect("fixture");
    two_roots.text_catalogs[1] = rebuild(
        &two_roots.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "de",
        None,
    );
    assert!(matches!(
        cook_project_v1(two_roots),
        Err(ProjectCookError::LocalizationClosureInvalid)
    ));

    let mut missing_fallback = next_reference_game::project_source_v2().expect("fixture");
    missing_fallback.text_catalogs[1] = rebuild(
        &missing_fallback.text_catalogs[1].clone(),
        AssetId::from_bytes([0x92; 16]),
        "qps-ploc",
        Some("de"),
    );
    assert!(matches!(
        cook_project_v1(missing_fallback),
        Err(ProjectCookError::MissingReference)
    ));

    let mut cyclic = next_reference_game::project_source_v2().expect("fixture");
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
        cook_project_v1(cyclic),
        Err(ProjectCookError::LocalizationClosureInvalid)
    ));

    let mut shared_asset_id = next_reference_game::project_source_v2().expect("fixture");
    shared_asset_id.text_catalogs[1] = rebuild(
        &shared_asset_id.text_catalogs[1].clone(),
        shared_asset_id.records[0].asset_id,
        "qps-ploc",
        Some("en"),
    );
    assert!(matches!(
        cook_project_v1(shared_asset_id),
        Err(ProjectCookError::DuplicateIdentity)
    ));
}

#[test]
fn audio_clips_cook_publish_and_activate_through_production_loader() {
    use next_contracts::audio::{
        AudioLoudnessMetadataV1, AudioPcmEncodingV1, NEUTRAL_AUDIO_SCHEMA_ID, NeutralAudioV1,
    };

    let clip = NeutralAudioV1::new(
        AssetId::from_bytes([0xb1; 16]),
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

    let mut source = next_reference_game::project_source_v2().expect("fixture");
    source.audio_records.push(clip.clone());
    let cooked = cook_project_v1(source).expect("cook with audio");
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

    let mut duplicate = next_reference_game::project_source_v2().expect("fixture");
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
        cook_project_v1(duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut invalid = next_reference_game::project_source_v2().expect("fixture");
    let mut invalid_clip = clip;
    invalid_clip.sample_rate_hz = 7_999;
    invalid.audio_records.push(invalid_clip);
    assert!(matches!(
        cook_project_v1(invalid),
        Err(ProjectCookError::Audio(_))
    ));
}

fn requirement(identity: &str) -> ProjectRequirementV1 {
    ProjectRequirementV1 {
        kind: ProjectDependencyKindV1::Content,
        identity: SchemaId::new(identity).expect("dependency ID"),
        minimum_version: SemanticVersionV1::new(1, 0, 0),
        optional: false,
    }
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
