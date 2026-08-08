use super::{
    ContentManifestV1, EmptyContentManifestProfilesV1, ProjectContractError, ProjectLockV3,
    SchemaEncodingV1, SchemaRefV1, SchemaRegistryManifestV2, SchemaRoleV1,
    canonical_empty_manifest_hash, domain_hash,
};
use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ContentHash, ProjectId, SchemaId};
use crate::platform::PresentationTargetKindV1;

#[test]
fn empty_manifests_have_stable_nonzero_hashes() {
    let canonical = domain_hash("canonical", b"profile");
    let registry = SchemaRegistryManifestV2::empty(canonical, domain_hash("ownership", b"empty"))
        .expect("empty registry");
    assert_ne!(
        registry.schema_registry_manifest_sha256,
        ContentHash::default()
    );
    let schema_ref = SchemaRefV1 {
        schema_id: SchemaId::new("nextengine.content.manifest").expect("schema"),
        schema_version: 1,
        descriptor_sha256: domain_hash("descriptor", b"content"),
        role: SchemaRoleV1::Manifest,
        encoding: SchemaEncodingV1::JcsRfc8785,
    };
    let content = ContentManifestV1::empty(
        schema_ref,
        SchemaId::new("nextengine.empty.content").expect("manifest ID"),
        ProjectId::new("org.nextengine.empty").expect("project"),
        EmptyContentManifestProfilesV1 {
            schema_registry_manifest_sha256: registry.schema_registry_manifest_sha256,
            canonicalization_profile_sha256: canonical,
            content_admission_limits_sha256: domain_hash("admission", b"empty"),
            cooker_contract_sha256: domain_hash("cooker", b"empty"),
            cooker_options_sha256: canonical_empty_manifest_hash("options"),
        },
    )
    .expect("empty content");
    let bytes = content.to_jcs_bytes().expect("canonical content");
    assert_eq!(
        ContentManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode"),
        content
    );
}

#[test]
fn neutral_content_schema_role_round_trips_through_registry_jcs() {
    let schema_ref = SchemaRefV1 {
        schema_id: SchemaId::new("nextengine.content.mesh").expect("schema"),
        schema_version: 1,
        descriptor_sha256: domain_hash("descriptor", b"mesh"),
        role: SchemaRoleV1::NeutralContent,
        encoding: SchemaEncodingV1::CanonicalBinaryV1,
    };
    let registry = super::SchemaRegistryManifestV2::new(super::SchemaRegistryManifestBodyV2 {
        registry_revision: 1,
        canonicalization_profile_sha256: domain_hash("canonical", b"profile"),
        ownership_registry_sha256: domain_hash("ownership", b"assets"),
        descriptors: vec![super::SchemaDescriptorV1 {
            schema_ref: schema_ref.clone(),
            owner_context_id: SchemaId::new("nextengine.assets").expect("owner"),
            field_registry_sha256: domain_hash("fields", b"mesh"),
        }],
        current_schema_refs: vec![schema_ref],
    })
    .expect("registry");
    let bytes = registry.to_jcs_bytes().expect("registry bytes");
    assert!(String::from_utf8_lossy(&bytes).contains("\"role\":\"neutral-content\""));
    assert_eq!(
        SchemaRegistryManifestV2::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode"),
        registry
    );
    let legacy = String::from_utf8(bytes).expect("JCS is UTF-8").replace(
        "nextengine.schema-registry-manifest.v2",
        "nextengine.schema-registry-manifest.v1",
    );
    assert!(matches!(
        SchemaRegistryManifestV2::from_jcs_bytes(
            legacy.as_bytes(),
            CanonicalDecodeLimits::default()
        ),
        Err(ProjectContractError::UnsupportedFormat { .. })
    ));
}

#[test]
fn project_lock_v3_round_trips_and_rejects_old_format_before_field_use() {
    let lock = project_lock_v3();
    let bytes = lock.to_jcs_bytes();
    assert_eq!(
        ProjectLockV3::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default()).expect("v3 lock"),
        lock
    );

    let legacy = String::from_utf8(bytes).expect("JCS is UTF-8").replace(
        "nextengine.project-lock.v3",
        "nextengine.project-composition-lock.v2",
    );
    assert_eq!(
        ProjectLockV3::from_jcs_bytes(legacy.as_bytes(), CanonicalDecodeLimits::default()),
        Err(ProjectContractError::UnsupportedFormat {
            expected: "nextengine.project-lock.v3",
            actual: "nextengine.project-composition-lock.v2".to_owned(),
        })
    );
}

#[test]
fn project_lock_v3_binds_exact_profiles_and_target_set() {
    let mut lock = project_lock_v3();
    lock.launch_profiles_sha256 = domain_hash("launch", b"tampered");
    assert_eq!(lock.validate(), Err(ProjectContractError::HashMismatch));

    let mut empty_targets = project_lock_v3();
    empty_targets.allowed_presentation_targets.clear();
    assert_eq!(
        ProjectLockV3::new(empty_targets),
        Err(ProjectContractError::MissingReference)
    );
}

fn project_lock_v3() -> ProjectLockV3 {
    ProjectLockV3::new(ProjectLockV3 {
        project_id: ProjectId::new("org.nextengine.project-lock-test").expect("project"),
        project_revision: 1,
        authoring_sha256: domain_hash("project", b"authoring"),
        schema_registry_manifest_sha256: domain_hash("project", b"schema"),
        content_manifest_sha256: domain_hash("project", b"content"),
        world_partition_manifest_sha256: domain_hash("project", b"partition"),
        mechanics_lock_sha256: domain_hash("project", b"mechanics"),
        runtime_determinism_profile_sha256: domain_hash("project", b"determinism"),
        launch_profiles_sha256: domain_hash("project", b"launch"),
        platform_capability_profile_sha256: domain_hash("project", b"capability"),
        platform_timebase_profile_sha256: domain_hash("project", b"timebase"),
        allowed_presentation_targets: vec![
            PresentationTargetKindV1::Interactive,
            PresentationTargetKindV1::None,
        ],
        project_lock_sha256: ContentHash::default(),
    })
    .expect("lock")
}
