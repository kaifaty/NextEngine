use super::{
    ContentManifestV1, EmptyContentManifestProfilesV1, ProjectCompositionLockV2,
    ProjectContractError, ProjectManifestV1, SchemaEncodingV1, SchemaRefV1,
    SchemaRegistryManifestV1, SchemaRoleV1, canonical_empty_manifest_hash, domain_hash,
};
use crate::canonical::CanonicalDecodeLimits;
use crate::ids::{ContentHash, ProjectId, SchemaId};
use crate::platform::PresentationTargetKindV1;

#[test]
fn project_manifest_jcs_round_trip_rejects_noncanonical_bytes() {
    let manifest = ProjectManifestV1::new(
        ProjectId::new("org.nextengine.contract-test").expect("project"),
        1,
        Vec::new(),
    )
    .expect("manifest");
    let bytes = manifest.to_jcs_bytes();
    assert_eq!(
        ProjectManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("decode"),
        manifest
    );
    let mut noncanonical = bytes;
    noncanonical.insert(1, b' ');
    assert!(
        ProjectManifestV1::from_jcs_bytes(&noncanonical, CanonicalDecodeLimits::default()).is_err()
    );
}

#[test]
fn empty_manifests_have_stable_nonzero_hashes() {
    let canonical = domain_hash("canonical", b"profile");
    let registry = SchemaRegistryManifestV1::empty(
        canonical,
        domain_hash("ownership", b"empty"),
        domain_hash("limits", b"bounded"),
    )
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
    let registry = super::SchemaRegistryManifestV1::new(super::SchemaRegistryManifestBodyV1 {
        registry_revision: 1,
        canonicalization_profile_sha256: domain_hash("canonical", b"profile"),
        ownership_registry_sha256: domain_hash("ownership", b"assets"),
        descriptors: vec![super::SchemaDescriptorV1 {
            schema_ref: schema_ref.clone(),
            owner_context_id: SchemaId::new("nextengine.assets").expect("owner"),
            field_registry_sha256: domain_hash("fields", b"mesh"),
        }],
        current_schema_refs: vec![schema_ref],
        migration_dag_sha256: domain_hash("migration", b"none"),
        registry_limits_sha256: domain_hash("limits", b"bounded"),
    })
    .expect("registry");
    let bytes = registry.to_jcs_bytes().expect("registry bytes");
    assert!(String::from_utf8_lossy(&bytes).contains("\"role\":\"neutral-content\""));
    assert_eq!(
        SchemaRegistryManifestV1::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default(),)
            .expect("decode"),
        registry
    );
}

#[test]
fn project_lock_v2_round_trips_and_rejects_v1_before_field_use() {
    let lock = project_lock_v2();
    let bytes = lock.to_jcs_bytes();
    assert_eq!(
        ProjectCompositionLockV2::from_jcs_bytes(&bytes, CanonicalDecodeLimits::default())
            .expect("v2 lock"),
        lock
    );

    let legacy = String::from_utf8(bytes).expect("JCS is UTF-8").replace(
        "nextengine.project-composition-lock.v2",
        "nextengine.project-composition-lock.v1",
    );
    assert_eq!(
        ProjectCompositionLockV2::from_jcs_bytes(
            legacy.as_bytes(),
            CanonicalDecodeLimits::default()
        ),
        Err(ProjectContractError::UnknownClosedValue)
    );
}

#[test]
fn project_lock_v2_binds_policy_profiles_and_target_set() {
    let mut lock = project_lock_v2();
    lock.shutdown_policy_sha256 = domain_hash("shutdown", b"tampered");
    assert_eq!(lock.validate(), Err(ProjectContractError::HashMismatch));

    let mut empty_targets = project_lock_v2();
    empty_targets.allowed_presentation_targets.clear();
    assert_eq!(
        ProjectCompositionLockV2::new(empty_targets),
        Err(ProjectContractError::MissingReference)
    );
}

fn project_lock_v2() -> ProjectCompositionLockV2 {
    ProjectCompositionLockV2::new(ProjectCompositionLockV2 {
        project_id: ProjectId::new("org.nextengine.project-lock-test").expect("project"),
        project_manifest_sha256: domain_hash("project", b"manifest"),
        catalog_snapshot_sha256: domain_hash("project", b"catalog"),
        resolver_profile_sha256: domain_hash("project", b"resolver"),
        schema_registry_manifest_sha256: domain_hash("project", b"schema"),
        content_manifest_sha256: domain_hash("project", b"content"),
        world_partition_manifest_sha256: domain_hash("project", b"partition"),
        mechanics_lock_sha256: domain_hash("project", b"mechanics"),
        runtime_determinism_profile_sha256: domain_hash("project", b"determinism"),
        launch_profiles_sha256: domain_hash("project", b"launch"),
        recovery_policy_sha256: crate::session::RecoveryPolicyV1::reference_game_default()
            .canonical_hash,
        shutdown_policy_sha256: crate::session::ShutdownPolicyV1::reference_game_default()
            .canonical_hash,
        recovery_permit_required_save: true,
        recovery_preserve_prior_history: true,
        shutdown_maximum_attempts: 3,
        shutdown_failure_disposition: crate::session::FailureDispositionV1::RequireFinalSave,
        platform_capability_profile_sha256: domain_hash("project", b"capability"),
        platform_timebase_profile_sha256: domain_hash("project", b"timebase"),
        allowed_presentation_targets: vec![
            PresentationTargetKindV1::Interactive,
            PresentationTargetKindV1::None,
        ],
        selected_records: Vec::new(),
        composition_lock_sha256: ContentHash::default(),
    })
    .expect("lock")
}
