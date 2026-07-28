use std::sync::atomic::{AtomicU64, Ordering};

use next_assets::{
    CONTENT_GENERATIONS_DIRECTORY, ContentPublicationV1, ContentStore, PublicationFileV1,
};
use next_contracts::{
    AssetId, ContentHash, ProjectCatalogRecordV1, ProjectCatalogSnapshotV1,
    ProjectDependencyKindV1, ProjectManifestV1, ProjectRequirementV1, SchemaId, SemanticVersionV1,
    content_hash_from_bytes,
};
use next_project::{
    ProjectActivationError, ProjectCookError, ProjectResolutionError, activate_project,
    cook_project_v1, neutral_vertical_slice_source_v1, resolve_project_records_v1,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn repeated_cooking_is_byte_identical_and_activates_through_production_loader() {
    let first =
        cook_project_v1(neutral_vertical_slice_source_v1().expect("fixture")).expect("first cook");
    let second =
        cook_project_v1(neutral_vertical_slice_source_v1().expect("fixture")).expect("second cook");
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
    assert_eq!(activated.content_manifest.body.asset_entries.len(), 13);
    assert_eq!(activated.world_partition.body.chunk_bindings.len(), 2);
    assert_eq!(activated.rpg_definitions.abilities.len(), 1);
    assert_eq!(activated.rpg_definitions.packages.len(), 2);
    std::fs::remove_dir_all(root).expect("remove test content");
}

#[test]
fn missing_blob_and_blob_hash_mismatch_fail_before_activation() {
    let cooked =
        cook_project_v1(neutral_vertical_slice_source_v1().expect("fixture")).expect("cook");
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
fn invalid_activation_never_replaces_the_callers_active_project() {
    let cooked =
        cook_project_v1(neutral_vertical_slice_source_v1().expect("fixture")).expect("cook");
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
    let mut malformed = neutral_vertical_slice_source_v1().expect("fixture");
    malformed.records[0].schema_ref.schema_id =
        SchemaId::new("nextengine.content.wrong.v1").expect("valid ID");
    assert!(matches!(
        cook_project_v1(malformed),
        Err(ProjectCookError::Neutral(_))
    ));

    let mut missing = neutral_vertical_slice_source_v1().expect("fixture");
    missing.records[0]
        .asset_dependencies
        .push(AssetId::from_bytes([0xfe; 16]));
    assert!(matches!(
        cook_project_v1(missing),
        Err(ProjectCookError::MissingReference)
    ));

    let mut duplicate = neutral_vertical_slice_source_v1().expect("fixture");
    duplicate.records[1].asset_id = duplicate.records[0].asset_id;
    assert!(matches!(
        cook_project_v1(duplicate),
        Err(ProjectCookError::DuplicateIdentity)
    ));

    let mut cycle = neutral_vertical_slice_source_v1().expect("fixture");
    let scene = cycle.records[0].asset_id;
    cycle.records[1].asset_dependencies.push(scene);
    assert!(matches!(
        cook_project_v1(cycle),
        Err(ProjectCookError::Contract(
            next_contracts::ProjectContractError::DependencyCycle
        ))
    ));
}

#[test]
fn project_resolver_rejects_dependency_cycle() {
    let project = ProjectManifestV1::new(
        next_contracts::ProjectId::new("org.nextengine.resolver-test").expect("project"),
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
