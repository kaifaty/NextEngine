use super::{
    ContentManifestV1, EmptyContentManifestProfilesV1, ProjectManifestV1, SchemaEncodingV1,
    SchemaRefV1, SchemaRegistryManifestV1, SchemaRoleV1, canonical_empty_manifest_hash,
    domain_hash,
};
use crate::{CanonicalDecodeLimits, ContentHash, ProjectId, SchemaId};

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
